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
