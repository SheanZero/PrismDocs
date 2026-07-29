//! 单写者 / 只读池的并发纪律——成功标准 3 前半的可执行证据。
//!
//! 这些性质（WAL 下读不阻塞写、池连接写不进去、迁移先于建池）代码评审看不住：
//! 错了不会报错，只会在 Phase 2+ 表现为偶发 `SQLITE_BUSY`、`no such table`、
//! 或者某条写路径悄悄绕过单写者。只有真实临时目录上的集成测试能钉住它们。
//!
//! **每个测试各用一个 `tempfile::tempdir()`**，任何测试都不得触碰真实数据根。

use std::sync::{mpsc, Arc, Barrier};

use prism_store::{MIN_SQLITE, READ_POOL_MAX_SIZE};

/// `major.minor.patch` **按位**解析——与 `open.rs::parse_sqlite_version` 同源。
///
/// 那一份在 `open()` 的准入判定路径上且是 crate 私有的（集成测试访问不到，因此这里是
/// 副本而不是复用）。两处都必须按位解析：`filter_map(|s| s.parse().ok())` 会丢掉不可解析的
/// 分量并把后面每一位左移一格，于是 `3.x.53` 塌缩成 `(3, 53, 0)` —— 一个碰巧够新的元组
/// （上轮 IN-03）。
fn version_tuple(v: &str) -> Option<(u32, u32, u32)> {
    let mut parts = v.split('.').map(str::parse::<u32>);
    match (parts.next(), parts.next(), parts.next()) {
        (Some(Ok(major)), Some(Ok(minor)), Some(Ok(patch))) => Some((major, minor, patch)),
        _ => None,
    }
}

const INSERT_PROJECT: &str =
    "INSERT INTO projects(id,name,root_path,created_at) VALUES(?1,'P','/tmp',0)";
const COUNT_PROJECTS: &str = "SELECT count(*) FROM projects";
const INSERT_SETTING: &str = "INSERT INTO settings(key,value,updated_at) VALUES('x','y',0)";

/// 写者在读者持有池连接期间提交，两边都不该看到 `SQLITE_BUSY`。
///
/// 三件事必须说清楚，否则这条测试会再次长成「名字承诺一件事、断言守着另一件」的形态：
///
/// 1. **它真正独有的价值**是「读者从池里借出一条连接、还没归还时，写者仍能提交且不超时」
///    ——WAL 单写者 + 只读池的核心性质。判别性断言是第二次读得到 **2**：
///    读的可见性语义变了（例如闭包内改成持事务），这条立刻红。
/// 2. `Store::read` **不**提供跨闭包的快照隔离：它从池里取一条连接就直接调闭包
///    （`open.rs::read`），从不显式开事务，因此闭包里每条 `query_row` 都在 autocommit 下
///    取一份新快照——第二次读**看得见**期间提交的那一行。
/// 3. 若将来设计确实需要闭包内快照隔离，必须**先实现**（`conn.unchecked_transaction()` +
///    `TransactionBehavior::Deferred`，并在闭包期间持有）**再断言**，不得反过来。
///    反过来就是上轮 WR-01 的原样：注释与测试名描述了一个不存在的性质，
///    而唯一让三者不撞车的东西是一条被削弱到恒真的断言（`assert!(after >= 1)`）。
///
/// **判别力边界（实测，见 01-19-SUMMARY 反证 B）**：因为闭包里的读走 autocommit、
/// 语句一结束就放锁，「写者不被阻塞」这一条在**任何** journal 模式下都成立——
/// 它只有在读者持一个未提交的读事务时才有判别力。这条测试守的是「今天的 `read()` 形态下
/// 二者并发不互相踩」；真正把 WAL 的必要性钉住的是 `open.rs::open_leaves_the_database_in_wal_mode`。
#[test]
fn writer_commits_while_a_reader_holds_a_pooled_connection() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Arc::new(prism_store::open(&dir.path().join("t.db")).expect("open store"));

    store
        .write(|tx| {
            tx.execute(INSERT_PROJECT, ["p1"])?;
            Ok(())
        })
        .expect("first write");

    let (tx_started, rx_started) = mpsc::channel();
    let (tx_go, rx_go) = mpsc::channel();
    let reader_store = Arc::clone(&store);
    let reader = std::thread::spawn(move || {
        reader_store
            .read(|c| {
                let before: i64 = c.query_row(COUNT_PROJECTS, [], |r| r.get(0))?;
                tx_started.send(()).expect("signal reader started");
                rx_go.recv().expect("wait for the writer");
                // 写已经提交；这一读在 autocommit 下取新快照，因此看得见它。
                let after: i64 = c.query_row(COUNT_PROJECTS, [], |r| r.get(0))?;
                Ok((before, after))
            })
            .expect("reader never sees SQLITE_BUSY")
    });

    rx_started.recv().expect("reader started");
    // 关键断言：读者持连接期间写者仍能提交且不超时。
    store
        .write(|tx| {
            tx.execute(INSERT_PROJECT, ["p2"])?;
            Ok(())
        })
        .expect("writer is not blocked by an open reader");
    tx_go.send(()).expect("release the reader");

    let (before, after) = reader.join().expect("reader thread");
    assert_eq!(before, 1, "读者起手应看到第一行");
    // 判别点：`Store::read` 的闭包不持事务，第二次读必须看见期间提交的那一行。
    // 写成 `>= 1` 就是恒真（before 已断言为 1，行只增不减），那正是上轮 WR-01。
    assert_eq!(after, 2, "autocommit 下的第二次读应看见期间提交的那一行");

    let total: i64 = store
        .read(|c| Ok(c.query_row(COUNT_PROJECTS, [], |r| r.get(0))?))
        .expect("post-write read");
    assert_eq!(total, 2, "写者的提交应对之后取出的读连接可见");
}

/// `query_only=ON` 是纪律的强制点：拿到池连接也写不进去。
#[test]
fn pooled_connection_cannot_write() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = prism_store::open(&dir.path().join("t.db")).expect("open store");

    let err = store
        .read(|c| Ok(c.execute(INSERT_SETTING, [])?))
        .expect_err("a pooled connection must not be able to write");

    assert!(
        format!("{err}").contains("readonly"),
        "query_only=ON not enforced: {err}"
    );
}

/// `query_only` 是**每连接**设置。这个测试同时握住池里的每一条连接分别断言，
/// 证明 `with_init` 覆盖了每一条，而不是只有第一条碰巧对。
#[test]
fn every_pooled_connection_is_query_only() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Arc::new(prism_store::open(&dir.path().join("t.db")).expect("open store"));

    let n = READ_POOL_MAX_SIZE as usize;
    // 所有线程都拿到连接后才开始断言——保证 n 条连接是彼此不同的实例。
    let all_acquired = Arc::new(Barrier::new(n));

    let handles: Vec<_> = (0..n)
        .map(|_| {
            let store = Arc::clone(&store);
            let all_acquired = Arc::clone(&all_acquired);
            std::thread::spawn(move || {
                store
                    .read(|c| {
                        all_acquired.wait();
                        let err = c
                            .execute(INSERT_SETTING, [])
                            .expect_err("pooled connection accepted a write");
                        Ok(format!("{err}"))
                    })
                    .expect("read closure")
            })
        })
        .collect();

    for handle in handles {
        let message = handle.join().expect("pool worker thread");
        assert!(
            message.contains("readonly"),
            "some pooled connection was not query_only: {message}"
        );
    }
}

/// bundled SQLite 必须新到含 WAL-reset 修复。
///
/// 下界取自 [`MIN_SQLITE`] 而不是写死的字面量：数字在仓库里只能有一个来源，
/// 否则改 pin 时两处各自漂移，而漂移的一处会继续绿。
#[test]
fn bundled_sqlite_meets_minimum() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = prism_store::open(&dir.path().join("t.db")).expect("open store");

    let v: String = store
        .read(|c| Ok(c.query_row("SELECT sqlite_version()", [], |r| r.get(0))?))
        .expect("version query");

    let got = version_tuple(&v).unwrap_or_else(|| panic!("unparsable sqlite version: {v}"));
    assert!(
        got >= MIN_SQLITE,
        "bundled SQLite {v} is older than the pinned minimum {MIN_SQLITE:?}"
    );
}

/// `close()` 要把 WAL 收干净——否则备份走了主库却漏掉未 checkpoint 的部分。
#[test]
fn wal_truncated_on_close() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("t.db");
    let store = prism_store::open(&db_path).expect("open store");

    for i in 0..64 {
        store
            .write(|tx| {
                tx.execute(INSERT_PROJECT, [format!("p{i}")])?;
                Ok(())
            })
            .expect("write row");
    }

    let wal_path = db_path.with_file_name("t.db-wal");
    let grew = std::fs::metadata(&wal_path).map(|m| m.len()).unwrap_or(0);
    assert!(grew > 0, "expected the -wal file to have grown before close");

    store.close().expect("close store");

    let after = std::fs::metadata(&wal_path).map(|m| m.len()).unwrap_or(0);
    assert_eq!(after, 0, "-wal should be truncated (or gone) after close");
}

/// writer-first 的顺序断言：迁移必须在建池之前完成，否则池连接看到的是空库
/// ——而它们 `query_only=ON`，连补建都做不到。
#[test]
fn reader_sees_migrated_schema() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = prism_store::open(&dir.path().join("t.db")).expect("open store");

    let count: i64 = store
        .read(|c| Ok(c.query_row(COUNT_PROJECTS, [], |r| r.get(0))?))
        .expect("a pooled connection must see the migrated schema, not `no such table`");

    assert_eq!(count, 0, "全新库里 projects 应为空表而不是不存在");
}
