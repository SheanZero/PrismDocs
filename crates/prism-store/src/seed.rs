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
