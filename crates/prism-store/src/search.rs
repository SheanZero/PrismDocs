//! 按查询长度分流的搜索层。
//!
//! # 为什么要分流（D-01 + D-02）
//!
//! `documents_fts` 用 trigram tokenizer，而 trigram 的最短匹配长度是 **3 个 unicode 字符**
//! ——短于此的子串在全文查询里不匹配任何行。典型的 2 字中文词（「评论」「锚点」）正好落在
//! 这条线下面，所以 <3 字符的查询降级为 `documents` 表上的 `LIKE '%…%'` 线性扫描。
//! 500 文档规模下全扫远在 300ms 预算内，分流对调用方不可见。
//!
//! 回退分支打在 `documents` 而不是 FTS 表上：语义等价、路径更短，并且把 FTS5 对 LIKE
//! 的索引化能力完整留作后续 phase 的余地。
//!
//! # 两层转义
//!
//! rusqlite 的参数绑定只保护 **SQL 语法层**。MATCH 的实参会被 FTS5 当作**查询语言**再解析
//! 一次（`"` `*` `^` `:` 与 `AND`/`OR`/`NOT` 均有特殊含义），LIKE 的实参会被当作**模式语言**
//! 再解析一次（`%` `_`）。两条分支因此各自还需要一层转义，否则轻则语义错、重则一次查询
//! 返回全库。

use rusqlite::Connection;

use crate::error::StoreError;
use prism_types::SearchHit;

/// trigram 的最短匹配长度。短于此走 LIKE 回退（D-02）。
pub const MIN_TRIGRAM_CHARS: usize = 3;

/// LIKE 模式里用来转义 `%` / `_` / 自身的字符，配合 SQL 中的 `ESCAPE` 子句。
const LIKE_ESCAPE: char = '\\';

/// FTS 表**不能起别名**：`MATCH` 的左操作数必须是 fts5 表在作用域里的名字，
/// 写成 `documents_fts f … WHERE f MATCH ?` 会得到 `no such column: f`。
/// 于是 JOIN 条件直接打在 `d.rowid_pk = documents_fts.rowid` 上——
/// `content_rowid='rowid_pk'` 这条 schema 事实在这里被消费。
const SQL_MATCH: &str = "SELECT d.id, d.title, d.rel_path \
     FROM documents_fts \
     JOIN documents d ON d.rowid_pk = documents_fts.rowid \
     WHERE documents_fts MATCH ?1 AND d.project_id = ?2 \
     ORDER BY rank";

const SQL_LIKE: &str = "SELECT id, title, rel_path FROM documents \
     WHERE project_id = ?1 \
       AND (content LIKE '%' || ?2 || '%' ESCAPE '\\' \
         OR title   LIKE '%' || ?2 || '%' ESCAPE '\\')";

/// 把用户输入包成**一个字面短语**交给 FTS5。
///
/// 整串加双引号 + 内部双引号加倍，是 FTS5 查询语言里表达「按字面匹配这一段」的唯一写法。
/// 不做这一步，`设计" OR 1=1` 会变成一次语法错误（Err）而不是一次 0 命中，
/// 而 `foo NOT bar` 会变成一次布尔查询——既是功能 bug 也是注入面。
pub fn escape_fts_query(q: &str) -> String {
    let mut out = String::with_capacity(q.len() + 4);
    out.push('"');
    for ch in q.chars() {
        if ch == '"' {
            out.push('"');
        }
        out.push(ch);
    }
    out.push('"');
    out
}

/// 把用户输入转义成 LIKE 的**字面**模式（配 `ESCAPE '\'`）。
///
/// 不做这一步，一次 `%` 查询会命中全库——与未转义的 MATCH 是同一类漏洞的两个面。
fn escape_like_pattern(q: &str) -> String {
    let mut out = String::with_capacity(q.len());
    for ch in q.chars() {
        if ch == '%' || ch == '_' || ch == LIKE_ESCAPE {
            out.push(LIKE_ESCAPE);
        }
        out.push(ch);
    }
    out
}

fn row_to_hit(row: &rusqlite::Row<'_>) -> rusqlite::Result<SearchHit> {
    Ok(SearchHit {
        doc_id: row.get(0)?,
        title: row.get(1)?,
        rel_path: row.get(2)?,
    })
}

/// 在一个 project 内搜索文档。
///
/// 查询长度分流对调用方不可见——两条分支返回同一种结果，只是走的索引不同。
/// 空查询（去掉首尾空白后为空）返回空结果：那既不是 FTS 表达得了的查询，
/// 在 LIKE 分支上又会退化成「匹配全部」。
pub fn search(conn: &Connection, project_id: &str, q: &str) -> Result<Vec<SearchHit>, StoreError> {
    let trimmed = q.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let rows = if trimmed.chars().count() >= MIN_TRIGRAM_CHARS {
        let mut stmt = conn.prepare(SQL_MATCH)?;
        let hits = stmt
            .query_map((escape_fts_query(trimmed), project_id), row_to_hit)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        hits
    } else {
        let mut stmt = conn.prepare(SQL_LIKE)?;
        let hits = stmt
            .query_map((project_id, escape_like_pattern(trimmed)), row_to_hit)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        hits
    };

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_wraps_the_whole_input_in_one_literal_phrase() {
        assert_eq!(escape_fts_query("锚定引擎"), "\"锚定引擎\"");
        assert_eq!(escape_fts_query("foo NOT bar"), "\"foo NOT bar\"");
    }

    #[test]
    fn escape_doubles_inner_quotes() {
        // 内部引号不加倍的话，短语会在此处提前闭合，剩下的部分被当作查询语法。
        assert_eq!(escape_fts_query("a\"b"), "\"a\"\"b\"");
    }

    #[test]
    fn like_pattern_escapes_wildcards_and_the_escape_char_itself() {
        assert_eq!(escape_like_pattern("%"), "\\%");
        assert_eq!(escape_like_pattern("a_b"), "a\\_b");
        assert_eq!(escape_like_pattern("a\\b"), "a\\\\b");
        assert_eq!(escape_like_pattern("锚定"), "锚定");
    }
}
