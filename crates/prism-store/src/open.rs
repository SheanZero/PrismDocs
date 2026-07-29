//! 库的打开流程与并发纪律。
//!
//! # writer-first 六步序
//!
//! 顺序是有语义的，不能并行、不能颠倒：
//!
//! 1. 建目录，**先**开 writer（读写 flags），由它创建库文件
//! 2. writer 上设 `journal_mode=WAL`（**持久设置**，随库文件走，只需设一次），
//!    并**读回它的返回行校验**——只设一次的东西更要验一次
//! 3. 再一批设每连接设置：`busy_timeout` / `synchronous` / `foreign_keys`
//! 4. 校验 SQLite 版本，然后在**裸 Connection**上跑迁移（`to_latest` 要 `&mut`）
//! 5. 迁移完成**之后**才建只读池
//! 6. `close()` 时在 writer 上做 TRUNCATE checkpoint
//!
//! 为什么顺序不可变：除 `journal_mode` 外的 pragma 全是**每连接**的，所以只读池的每条
//! 连接都得重新设 `query_only`——`with_init` 是唯一正确的注入点。而 r2d2 建池时会立即
//! 预热连接：池若先于 writer 建立，它会自己创建一个空库文件，迁移随后跑在错误的文件上。

use std::path::Path;

use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, OpenFlags};

use crate::error::StoreError;

/// 撞锁时的等待上限。单用户本地场景下，超过这个时长说明有人违反了单写者纪律。
pub const BUSY_TIMEOUT_MS: u32 = 5_000;

/// bundled SQLite 的下限（含 WAL-reset 修复）。
pub const MIN_SQLITE: (u32, u32, u32) = (3, 51, 3);

/// 只读池大小。真正的约束不是这个数字，而是「读连接用完即还」——
/// 闭包式 `read()` 从形态上保证了这一点。
pub const READ_POOL_MAX_SIZE: u32 = 4;

/// 单写者句柄 + 只读连接池。
///
/// writer 用 `std::sync::Mutex` 而**不是** tokio 的异步 Mutex：前者的 guard 是 `!Send`，
/// 于是「在 `.await` 上持有写锁」是编译错误而不是运行期的长事务。这条纪律因此由编译器执行，
/// 而不是靠代码评审。
pub struct Store {
    writer: std::sync::Mutex<Connection>,
    readers: r2d2::Pool<SqliteConnectionManager>,
}

/// 打开（必要时创建）库，跑完迁移，建好只读池。
pub fn open(db_path: &Path) -> Result<Store, StoreError> {
    if let Some(dir) = db_path.parent() {
        std::fs::create_dir_all(dir)?;
    }

    // 1–2. writer 先行。journal_mode 单独走 query_row：它那一行返回值**就是**结果模式，
    //      而不是执行回执。WAL 起不来时（网络卷、只读目录、`-shm` 建不出来）SQLite 返回
    //      `delete` 且不报错——不读这一行，整个 crate 的并发语义就静默降级成 rollback
    //      journal，并在 Phase 2+ 表现为 5 秒 busy_timeout 下的偶发 SQLITE_BUSY。
    //      大小写不作保证，因此用 eq_ignore_ascii_case 比。
    let mut writer = Connection::open(db_path)?;
    let mode: String = writer.query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))?;
    if !mode.eq_ignore_ascii_case("wal") {
        return Err(StoreError::JournalModeNotWal(mode));
    }

    // 3. 余下三条是每连接设置，没有返回行，照旧一批设完。
    writer.execute_batch(&format!(
        "PRAGMA synchronous=NORMAL;\
         PRAGMA busy_timeout={BUSY_TIMEOUT_MS};\
         PRAGMA foreign_keys=ON;"
    ))?;

    // 4. 版本校验 + 迁移。to_latest 要 &mut Connection，所以迁移必须在
    //    Mutex::new(writer) 之前用裸 Connection 完成。
    assert_sqlite_version(&writer)?;
    crate::migrations::migrations().to_latest(&mut writer)?;

    // 5. 只读池——严格在迁移之后。
    //
    //    用读写 flags + query_only=ON，而不是 SQLITE_OPEN_READ_ONLY：只读 flags 的连接
    //    在 `-shm` 缺失时（崩溃后首次打开）无法创建它，会拿到 SQLITE_CANTOPEN。读写 flags
    //    保留 WAL 恢复能力，query_only 保证写被拒。
    //
    //    query_only 放 with_init 的最后一条：此后该连接连自己的设置都改不了。
    //    它不暴露任何开关或配置项——去掉它，FTS 触发器就会与 writer 事务并发。
    let manager = SqliteConnectionManager::file(db_path)
        .with_flags(OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_URI)
        .with_init(|c| {
            c.execute_batch(&format!(
                "PRAGMA busy_timeout={BUSY_TIMEOUT_MS};\
                 PRAGMA foreign_keys=ON;\
                 PRAGMA query_only=ON;"
            ))
        });
    let readers = r2d2::Pool::builder()
        .max_size(READ_POOL_MAX_SIZE)
        .build(manager)?;

    Ok(Store {
        writer: std::sync::Mutex::new(writer),
        readers,
    })
}

fn assert_sqlite_version(conn: &Connection) -> Result<(), StoreError> {
    let version: String = conn.query_row("SELECT sqlite_version()", [], |r| r.get(0))?;
    let parts: Vec<u32> = version.split('.').filter_map(|s| s.parse().ok()).collect();
    let got = (
        parts.first().copied().unwrap_or(0),
        parts.get(1).copied().unwrap_or(0),
        parts.get(2).copied().unwrap_or(0),
    );
    if got < MIN_SQLITE {
        return Err(StoreError::SqliteTooOld(version));
    }
    Ok(())
}

impl Store {
    /// 打开库。等价于自由函数 [`open`]，保留给已经在用 `Store::open` 的调用方。
    pub fn open(db_path: &Path) -> Result<Store, StoreError> {
        open(db_path)
    }

    /// 单写者事务。**阻塞调用**——facade 侧必须包在 `tokio::task::spawn_blocking` 里。
    ///
    /// 闭包式而非「借出连接」式：调用方拿不到可长期持有的句柄，
    /// 于是「跨事件循环持有写锁」在这个 API 下无从表达。
    pub fn write<T>(
        &self,
        f: impl FnOnce(&rusqlite::Transaction) -> Result<T, StoreError>,
    ) -> Result<T, StoreError> {
        // 毒化的锁照常使用：writer 的不变量由 SQLite 事务而不是 Rust 类型维护，
        // 一个 panic 掉的写者留下的是已回滚的事务，不是半个数据结构。
        let mut guard = self.writer.lock().unwrap_or_else(|e| e.into_inner());
        let tx = guard.transaction()?;
        let out = f(&tx)?;
        tx.commit()?;
        Ok(out)
    }

    /// 从只读池取一条连接。连接出闭包作用域即归还。
    pub fn read<T>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, StoreError>,
    ) -> Result<T, StoreError> {
        let conn = self.readers.get()?;
        f(&conn)
    }

    pub fn sqlite_version(&self) -> Result<String, StoreError> {
        self.read(|c| Ok(c.query_row("SELECT sqlite_version()", [], |r| r.get(0))?))
    }

    /// 收尾：把 WAL 全量搬回主库并把 `-wal` 截断。
    ///
    /// 先扔掉只读池——TRUNCATE checkpoint 在还有其他连接开着时会「成功但没做事」
    /// （返回 busy 标志而不是报错），那正是这类 bug 静默的地方。
    pub fn close(self) -> Result<(), StoreError> {
        let Store { writer, readers } = self;
        drop(readers);
        let conn = writer.into_inner().unwrap_or_else(|e| e.into_inner());
        conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    /// WAL 是这个 crate 每一条并发性质的前提，而 SQLite 用**返回行**而不是错误告知结果模式：
    /// WAL 起不来时它返回 `delete`，`execute_batch` 照样报告成功。这条守的是「open() 之后
    /// 库确实处在 WAL 下」。
    #[test]
    fn open_leaves_the_database_in_wal_mode() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = super::open(&dir.path().join("t.db")).expect("open store");
        let mode: String = store
            .read(|c| Ok(c.query_row("PRAGMA journal_mode", [], |r| r.get(0))?))
            .expect("query journal_mode");
        assert!(
            mode.eq_ignore_ascii_case("wal"),
            "expected journal_mode=wal after open(), got {mode}"
        );
    }

    /// T-01-20：错误会跨 IPC 边界到达前端 DOM，库路径是内部事实。
    /// 形态沿用 `commands.rs::mapped_errors_do_not_carry_lower_layer_text`。
    #[test]
    fn journal_mode_error_carries_no_path() {
        let text = crate::error::StoreError::JournalModeNotWal("delete".to_owned()).to_string();
        assert!(
            !text.contains('/'),
            "journal-mode error must not carry a path: {text}"
        );
        assert!(
            text.contains("delete"),
            "journal-mode error should name the mode it actually got: {text}"
        );
    }

    /// T-01-20，同上：busy 只陈述规则性事实，不带路径也不带连接数。
    #[test]
    fn checkpoint_busy_error_carries_no_path() {
        let text = crate::error::StoreError::CheckpointBusy.to_string();
        assert!(
            !text.contains('/'),
            "checkpoint-busy error must not carry a path: {text}"
        );
    }

    /// 第 4 步（迁移）必须排在第 5 步（建池）之前。
    ///
    /// 这条断言看的是源码顺序，不是行为——因为**行为测不出来**：实测把建池挪到迁移之前，
    /// `tests/concurrency.rs` 的六个测试全绿（同一个库文件路径下，池连接会在迁移提交后
    /// 重读 schema cookie，于是照样看得见新表）。也就是说 `reader_sees_migrated_schema`
    /// 守的是「迁移到底跑没跑」（去掉 `to_latest` 它立刻变红），守不住「跑在建池之前」。
    ///
    /// 顺序本身仍然重要：它是 writer-first 序列里唯一没有运行期兜底的一步。既然没有
    /// 行为面的哨兵，就在这里放一个结构面的。
    #[test]
    fn migration_runs_before_the_read_pool_is_built() {
        let source = include_str!("open.rs");
        let migrate_at = source
            .find("to_latest(&mut writer)")
            .expect("open() should call to_latest on the bare writer");
        let pool_at = source
            .find("Pool::builder()")
            .expect("open() should build the read pool");
        assert!(
            migrate_at < pool_at,
            "read pool is built before migrations run — pooled connections are query_only \
             and cannot repair a schema they opened too early"
        );
    }
}
