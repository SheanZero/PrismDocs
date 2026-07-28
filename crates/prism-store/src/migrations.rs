//! schema 迁移集合。
//!
//! **不可变性契约**：已发布的迁移**绝不修改**。后续 phase 只在 `MIGRATIONS` 的 `vec!` 末尾
//! append 新的 `M::up`——`rusqlite_migration` 用 SQLite 原生的 `user_version` 追踪进度，
//! 改动一条已执行过的迁移不会重跑，只会让新老库的 schema 永久分叉。
//!
//! **迁移里不写 PRAGMA**：`journal_mode` 等属于连接打开流程（见 `open.rs` 的六步序），
//! 不属于 schema 版本；放进迁移会让 `validate()` 的内存库行为与真实库分叉。

use std::sync::LazyLock;

use rusqlite_migration::{Migrations, M};

static MIGRATIONS: LazyLock<Migrations<'static>> = LazyLock::new(|| {
    Migrations::new(vec![
        M::up(include_str!("../migrations/001_schema_v1.sql")),
        // Phase 2/3/5/7 各自 append 自己的 M::up，绝不修改上面这条。
    ])
});

/// 全局唯一的迁移集合。
#[must_use]
pub fn migrations() -> &'static Migrations<'static> {
    &MIGRATIONS
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    /// 在内存库上跑完整套迁移，返回已迁移的连接。
    fn migrated() -> Connection {
        let mut conn = Connection::open_in_memory().expect("in-memory db");
        super::migrations()
            .to_latest(&mut conn)
            .expect("to_latest on a fresh in-memory db");
        conn
    }

    fn user_version(conn: &Connection) -> i64 {
        conn.query_row("PRAGMA user_version", [], |r| r.get(0))
            .expect("read user_version")
    }

    #[test]
    fn migrations_are_valid() {
        // 官方推荐的迁移单测：在内存库跑完整套迁移。
        super::migrations()
            .validate()
            .expect("migration set is invalid");
    }

    #[test]
    fn schema_v1_creates_the_four_minimal_objects() {
        let conn = migrated();
        let n: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master \
                 WHERE name IN ('projects','documents','documents_fts','settings')",
                [],
                |r| r.get(0),
            )
            .expect("count schema objects");
        assert_eq!(n, 4, "D-04 的最小集边界应恰好落地四个对象");
    }

    #[test]
    fn schema_v1_creates_the_three_fts_sync_triggers() {
        let conn = migrated();
        let n: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='trigger' \
                 AND name IN ('documents_ai','documents_ad','documents_au')",
                [],
                |r| r.get(0),
            )
            .expect("count triggers");
        assert_eq!(
            n, 3,
            "external-content FTS 的索引同步由三触发器强制，缺一个就有写路径能静默漏同步"
        );
    }

    #[test]
    fn documents_rowid_pk_is_an_explicit_integer_primary_key() {
        let conn = migrated();
        let (pk, ty): (i64, String) = conn
            .query_row(
                "SELECT pk, type FROM pragma_table_info('documents') WHERE name='rowid_pk'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .expect("documents.rowid_pk should exist");
        assert_eq!(pk, 1, "rowid_pk 必须是主键");
        assert_eq!(
            ty, "INTEGER",
            "必须是显式 INTEGER PRIMARY KEY——隐式 rowid 会被 VACUUM 重编号，\
             让 FTS 的 content_rowid 与内容表静默错位"
        );
    }

    #[test]
    fn to_latest_is_idempotent() {
        let mut conn = Connection::open_in_memory().expect("in-memory db");
        super::migrations().to_latest(&mut conn).expect("first run");
        let first = user_version(&conn);
        super::migrations().to_latest(&mut conn).expect("second run");
        let second = user_version(&conn);
        assert_eq!(first, second, "重复迁移不应推进 user_version");
        assert!(first > 0, "迁移执行后 user_version 应大于 0，实得 {first}");
    }
}
