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

    /// 在 external-content FTS 索引里数命中行数。查询串统一包成双引号短语，
    /// 避免用户输入被当作 FTS5 查询语法解析。
    fn fts_hits(conn: &Connection, needle: &str) -> i64 {
        let phrase = format!("\"{}\"", needle.replace('"', "\"\""));
        conn.query_row(
            "SELECT count(*) FROM documents_fts WHERE documents_fts MATCH ?1",
            [&phrase],
            |r| r.get(0),
        )
        .unwrap_or_else(|e| panic!("fts match on {needle:?} failed: {e}"))
    }

    /// 方案 A 的已知弱点是「触发器出错时表现为搜不到而非报错」。这个测试是它的兜底：
    /// INSERT / UPDATE / DELETE 三条同步路径逐条走一遍，任何一个触发器缺失或写错都会变红。
    /// 顺带证明索引粒度保持全粒度——「锚定引擎」是 4 个 unicode 字符，
    /// 若 detail 被降级，这一句 MATCH 会直接失效。
    #[test]
    fn fts_index_stays_in_sync_across_insert_update_and_delete() {
        let conn = migrated();
        conn.execute(
            "INSERT INTO projects(id,name,root_path,created_at) VALUES('p1','P','/tmp',0)",
            [],
        )
        .expect("seed project");
        conn.execute(
            "INSERT INTO documents(id,project_id,rel_path,title,content,content_hash,updated_at) \
             VALUES('d1','p1','a.md','设计说明','锚定引擎的实现细节','h1',0)",
            [],
        )
        .expect("seed document");

        assert_eq!(
            fts_hits(&conn, "锚定引擎"),
            1,
            "documents_ai 未把新行同步进 FTS 索引"
        );

        conn.execute(
            "UPDATE documents SET content='评论回流的闭环说明' WHERE id='d1'",
            [],
        )
        .expect("update document");

        assert_eq!(
            fts_hits(&conn, "锚定引擎"),
            0,
            "documents_au 的 delete 半边未撤下旧内容——索引会留下搜得到的幽灵行"
        );
        assert_eq!(
            fts_hits(&conn, "评论回流"),
            1,
            "documents_au 的 insert 半边未写入新内容"
        );

        conn.execute("DELETE FROM documents WHERE id='d1'", [])
            .expect("delete document");

        assert_eq!(
            fts_hits(&conn, "评论回流"),
            0,
            "documents_ad 未从 FTS 索引撤下已删除的行"
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
