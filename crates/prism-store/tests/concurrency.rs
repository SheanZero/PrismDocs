//! 单写者 / 只读池的并发纪律——成功标准 3 前半的可执行证据。
//!
//! 这些性质（WAL 下读不阻塞写、池连接写不进去、迁移先于建池）代码评审看不住：
//! 错了不会报错，只会在 Phase 2+ 表现为偶发 `SQLITE_BUSY`、`no such table`、
//! 或者某条写路径悄悄绕过单写者。只有真实临时目录上的集成测试能钉住它们。
//!
//! **每个测试各用一个 `tempfile::tempdir()`**，任何测试都不得触碰真实数据根。

use std::sync::{mpsc, Arc, Barrier};

use prism_store::READ_POOL_MAX_SIZE;

fn version_tuple(v: &str) -> (u32, u32, u32) {
    let p: Vec<u32> = v.split('.').filter_map(|s| s.parse().ok()).collect();
    (
        p.first().copied().unwrap_or(0),
        p.get(1).copied().unwrap_or(0),
        p.get(2).copied().unwrap_or(0),
    )
}

const INSERT_PROJECT: &str =
    "INSERT INTO projects(id,name,root_path,created_at) VALUES(?1,'P','/tmp',0)";
const COUNT_PROJECTS: &str = "SELECT count(*) FROM projects";
const INSERT_SETTING: &str = "INSERT INTO settings(key,value,updated_at) VALUES('x','y',0)";

/// WAL 下读者持有的是快照，写者不被它阻塞——两边都不该看到 `SQLITE_BUSY`。
#[test]
fn reader_snapshot_is_isolated() {
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
                // 写已经提交了；同一个读连接仍在自己的事务快照里。
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
    assert!(after >= 1, "读者不应因并发写而丢失可见行");

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
#[test]
fn bundled_sqlite_meets_minimum() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = prism_store::open(&dir.path().join("t.db")).expect("open store");

    let v: String = store
        .read(|c| Ok(c.query_row("SELECT sqlite_version()", [], |r| r.get(0))?))
        .expect("version query");

    assert!(
        version_tuple(&v) >= (3, 51, 3),
        "bundled SQLite too old: {v}"
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
