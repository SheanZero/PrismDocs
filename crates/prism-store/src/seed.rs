//! dev 冒烟页的样例数据（D-06：冒烟页是脚手架，后续 phase 逐步替换）。
//!
//! 存在的理由只有一条：「4 字中文词返回非零结果」这条验收在**空库**上，
//! 无论 trigram 索引是对是错都返回 0——那种绿证明不了任何事。样例数据是那条断言的分母。
//!
//! 内容刻意**不含**阴性对照词「量子纠缠」：一个「搜什么都命中」的实现必须能被看出来。

use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Transaction;

use crate::error::StoreError;

/// 冒烟页固定使用的 project id。**只在这里定义一次**——前端从命令返回值拿它，
/// 而不是另抄一份常量（两份必然漂移）。
pub const SAMPLE_PROJECT_ID: &str = "smoke-project";

const SAMPLE_PROJECT_NAME: &str = "冒烟样例项目";
/// 样例项目不对应真实仓库；Phase 2 的导入才有真 root_path。
const SAMPLE_PROJECT_ROOT: &str = "<smoke-sample>";

/// `(id, rel_path, title, content)`。
pub const SAMPLE_DOCS: [(&str, &str, &str, &str); 3] = [
    (
        "smoke-doc-1",
        "docs/anchor.md",
        "锚定引擎设计说明",
        "锚定引擎在文档被大规模重写之后，仍然把评论挂回原来的段落。",
    ),
    (
        "smoke-doc-2",
        "docs/feedback.md",
        "评论回流闭环",
        "评论回流把结构化的反馈交回 AI，驱动下一轮迭代，并保留可追溯的因果链。",
    ),
    (
        "smoke-doc-3",
        "docs/contract.md",
        "Contract subscription",
        "Downstream projects are alerted to re-check when an upstream API spec changes.",
    ),
];

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

/// 写入（或刷新）样例项目与样例文档，返回写入的文档条数。
///
/// **幂等**：冒烟页上的按钮会被反复点击，两次播种必须留下一份而不是两份。
/// 走 `ON CONFLICT DO UPDATE` 而不是先 DELETE 再 INSERT——前者触发 `documents_au`
/// 的删增两半，FTS 索引照常跟上；后者会白白多走一遍 rowid 分配。
pub fn insert_samples(tx: &Transaction) -> Result<usize, StoreError> {
    let now = now_secs();

    tx.execute(
        "INSERT INTO projects(id, name, root_path, created_at) VALUES(?1, ?2, ?3, ?4) \
         ON CONFLICT(id) DO NOTHING",
        (
            SAMPLE_PROJECT_ID,
            SAMPLE_PROJECT_NAME,
            SAMPLE_PROJECT_ROOT,
            now,
        ),
    )?;

    let mut stmt = tx.prepare(
        "INSERT INTO documents(id, project_id, rel_path, title, content, content_hash, updated_at) \
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7) \
         ON CONFLICT(id) DO UPDATE SET \
           title = excluded.title, content = excluded.content, \
           content_hash = excluded.content_hash, updated_at = excluded.updated_at",
    )?;

    for (id, rel_path, title, content) in SAMPLE_DOCS {
        let hash = blake3::hash(content.as_bytes()).to_hex().to_string();
        stmt.execute((
            id,
            SAMPLE_PROJECT_ID,
            rel_path,
            title,
            content,
            hash.as_str(),
            now,
        ))?;
    }

    Ok(SAMPLE_DOCS.len())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{insert_samples, SAMPLE_DOCS, SAMPLE_PROJECT_ID};
    use crate::search::search;

    fn count_samples(conn: &Connection) -> i64 {
        conn.query_row(
            "SELECT count(*) FROM documents WHERE project_id = ?1",
            [SAMPLE_PROJECT_ID],
            |r| r.get(0),
        )
        .expect("count documents")
    }

    fn seeded() -> Connection {
        let mut conn = Connection::open_in_memory().expect("in-memory db");
        crate::migrations::migrations()
            .to_latest(&mut conn)
            .expect("to_latest");
        let tx = conn.transaction().expect("tx");
        insert_samples(&tx).expect("insert samples");
        tx.commit().expect("commit");
        conn
    }

    /// 成功标准 3 的分母：4 字中文词在 trigram 索引上必须真的命中。
    #[test]
    fn a_four_character_chinese_word_hits_the_samples() {
        let conn = seeded();
        let hits = search(&conn, SAMPLE_PROJECT_ID, "锚定引擎").expect("search");
        assert!(!hits.is_empty(), "样例数据里的 4 字中文词应当命中");
    }

    /// **阴性对照。** 没有这一条，上一条在「搜什么都命中」的实现下也是绿的。
    #[test]
    fn a_word_absent_from_the_samples_returns_no_hits() {
        let conn = seeded();
        let hits = search(&conn, SAMPLE_PROJECT_ID, "量子纠缠").expect("search");
        assert!(hits.is_empty(), "库里没有的词不应命中，实得 {hits:?}");
    }

    /// `insert_samples` 的返回值必须来自执行结果，而不是一个与执行无关的常量。
    ///
    /// 断言把返回值与**库里的实际计数**对上，而不是与 `SAMPLE_DOCS.len()` 对上——
    /// 后者是拿常量比常量，函数写没写进去都成立。
    ///
    /// **判别力说明（实测，见 01-19-SUMMARY 反证 C）**：顺境下常量与实际计数恰好相等，
    /// 这条断言对两种实现都是绿的。它的判别点是「返回值与库状态发生分歧」的那一刻——
    /// 把循环里某一条文档跳过（不动返回逻辑）时，常量实现报 3 而库里只有 2，这条立刻红；
    /// 累加实现报 2，与库状态一致，保持绿。这正是一个「能报告失败」的返回值与一个
    /// 「报告不了它被要求报告的失败」的返回值的区别（上轮 IN-02）。
    #[test]
    fn the_returned_count_matches_the_rows_actually_in_the_database() {
        let mut conn = Connection::open_in_memory().expect("in-memory db");
        crate::migrations::migrations()
            .to_latest(&mut conn)
            .expect("to_latest");

        let tx = conn.transaction().expect("tx");
        let first = insert_samples(&tx).expect("first seed");
        tx.commit().expect("commit");
        assert_eq!(
            first as i64,
            count_samples(&conn),
            "首次播种的返回值应等于库里实际存在的样例文档数"
        );

        // 重复播种：`ON CONFLICT DO UPDATE` 每行仍算一次受影响行，
        // 返回值因此继续与库状态一致（而不是悄悄归零或翻倍）。
        let tx = conn.transaction().expect("tx");
        let second = insert_samples(&tx).expect("second seed");
        tx.commit().expect("commit");
        assert_eq!(
            second as i64,
            count_samples(&conn),
            "重复播种的返回值应反映本次实际生效的行数"
        );
    }

    /// 任一 `execute` 失败必须传播成 `Err`，而不是照常返回一个条数。
    #[test]
    fn a_failed_statement_propagates_instead_of_reporting_a_count() {
        // 没跑迁移：`projects` / `documents` 都不存在。
        let mut conn = Connection::open_in_memory().expect("in-memory db");
        let tx = conn.transaction().expect("tx");
        insert_samples(&tx).expect_err("missing tables must surface as Err");
    }

    /// 冒烟页会被反复点击，重复播种不得堆出重复文档。
    #[test]
    fn seeding_twice_leaves_exactly_one_copy() {
        let mut conn = seeded();
        let tx = conn.transaction().expect("tx");
        insert_samples(&tx).expect("second insert");
        tx.commit().expect("commit");

        let n: i64 = conn
            .query_row(
                "SELECT count(*) FROM documents WHERE project_id = ?1",
                [SAMPLE_PROJECT_ID],
                |r| r.get(0),
            )
            .expect("count documents");
        assert_eq!(n, SAMPLE_DOCS.len() as i64);
    }
}
