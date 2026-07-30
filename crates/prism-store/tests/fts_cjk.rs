//! 查询层的**判别性**用例——成功标准 3 后半（FTS5 中文查询返回非零结果）的可执行证据。
//!
//! 这一层的失败模式全部是静默的：tokenizer 选错表现为「搜不到」而不是报错，触发器漏同步
//! 表现为「搜不到」，`VACUUM` 后 rowid 错位表现为「搜到错文档」。所以这组用例的价值不在
//! 覆盖率，而在**每一条被删掉之后都会有一个具体的静默失败模式重新变得可能**——
//! 每条断言旁的注释写的就是那个模式。
//!
//! 四个测试函数，各用独立的 `tempfile::TempDir`，任何一个都不触碰真实数据根。

use prism_store::{open, search, Store};
use prism_types::SearchHit;

const INSERT_PROJECT: &str =
    "INSERT INTO projects(id,name,root_path,created_at) VALUES(?1,'P','/tmp',0)";

const INSERT_DOC: &str = "INSERT INTO documents\
     (id,project_id,rel_path,title,content,content_hash,updated_at) \
     VALUES(?1,?2,?3,?4,?5,'h',0)";

/// RESEARCH § Code Examples 的中英混排样本：标题含 4 字中文词，正文含 3 字边界词、
/// CJK 混排与英文子串——一篇文档同时喂饱六组断言。
const SAMPLE_TITLE: &str = "锚定引擎设计";
const SAMPLE_BODY: &str =
    "本文描述 Block 锚定引擎的设计与迁移契约，覆盖 CJK 混排 mixed English content。";

fn seed_one_doc(store: &Store) {
    store
        .write(|tx| {
            tx.execute(INSERT_PROJECT, ["p1"])?;
            tx.execute(INSERT_DOC, ("d1", "p1", "a.md", SAMPLE_TITLE, SAMPLE_BODY))?;
            Ok(())
        })
        .expect("seed");
}

fn hits(store: &Store, project_id: &str, q: &str) -> Vec<SearchHit> {
    store
        .read(|c| search(c, project_id, q))
        .unwrap_or_else(|e| panic!("search({q:?}) failed: {e}"))
}

fn n(store: &Store, project_id: &str, q: &str) -> usize {
    hits(store, project_id, q).len()
}

/// 六组断言共用一篇样本文档。判别性说明见每条注释。
#[test]
fn chinese_query_returns_nonzero_rows() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = open(&dir.path().join("t.db")).expect("open store");
    seed_one_doc(&store);

    // ① 4 字中文走 trigram MATCH。换回 unicode61 → 0（unicode61 不切分 CJK）；
    //    误加 detail=none/column → MATCH 直接报错（>3 unicode 字符的 token 被禁）。
    assert_eq!(n(&store, "p1", "锚定引擎"), 1, "4 字中文词应命中");

    // ② 3 字边界——trigram 的最短匹配长度正好是 3，少一个字就落到另一条分支上。
    assert_eq!(n(&store, "p1", "迁移契"), 1, "3 字边界词应命中");

    // ③ 2 字中文走 D-02 的 LIKE 回退。忘记分流时**只有这一条**变红——
    //    这正是它与 ①② 分开存在的理由。
    assert_eq!(n(&store, "p1", "锚定"), 1, "2 字中文词应经 LIKE 回退命中");

    // ④ 中英混排里的英文子串：trigram 对英文同样是 substring 匹配。
    assert_eq!(n(&store, "p1", "mixed"), 1, "混排英文子串应命中");

    // ⑤ 阴性对照。删掉它，「搜索永远返回全部」这种实现也能让 ①–④ 全绿。
    assert_eq!(n(&store, "p1", "量子纠缠"), 0, "库中不存在的词应返回 0 行");

    // ⑤b **回退分支自己的**阴性对照。⑤ 是 4 字词，走的是 MATCH——把 LIKE 分支写成
    //     无条件返回全部行，⑤ 照样绿。两条分支各要一条阴性对照，缺一条就有半边恒真。
    assert_eq!(
        n(&store, "p1", "量子"),
        0,
        "回退分支上不存在的词也应返回 0 行"
    );

    // ⑥ FTS5 查询语法层的注入：`"` 与布尔算子在 MATCH 串里有特殊含义，
    //    参数绑定只挡 SQL 层挡不住这层。未转义时这里会是 `fts5: syntax error`（Err）而非 0 行。
    assert_eq!(
        n(&store, "p1", "设计\" OR 1=1"),
        0,
        "含双引号与布尔算子的输入应被转义成字面短语"
    );

    // ⑦ LIKE 模式层的同类漏洞：`%` 未转义时回退分支会匹配全部文档。
    assert_eq!(n(&store, "p1", "%"), 0, "LIKE 通配符应被转义为字面量");
}

/// 触发器同步：外部内容表的索引不是写路径维护的，是 documents_au / documents_ad 维护的。
/// 触发器写错时的表现是「搜不到」，不是报错——所以必须从**搜索 API** 这一侧验证，
/// 而不是只在 SQL 层数一数触发器是否存在。
#[test]
fn fts_index_follows_update_and_delete() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = open(&dir.path().join("t.db")).expect("open store");
    seed_one_doc(&store);
    assert_eq!(n(&store, "p1", "锚定引擎"), 1, "前置条件：插入后可搜到");

    store
        .write(|tx| {
            tx.execute(
                "UPDATE documents SET title = ?1, content = ?2 WHERE id = 'd1'",
                ("全新内容标题", "这段正文只谈全新内容，与旧词无关。"),
            )?;
            Ok(())
        })
        .expect("update");

    assert_eq!(n(&store, "p1", "锚定引擎"), 0, "UPDATE 后旧词应搜不到");
    assert_eq!(n(&store, "p1", "全新内容"), 1, "UPDATE 后新词应搜得到");

    store
        .write(|tx| {
            tx.execute("DELETE FROM documents WHERE id = 'd1'", [])?;
            Ok(())
        })
        .expect("delete");

    assert_eq!(n(&store, "p1", "全新内容"), 0, "DELETE 后应归零");

    // 上面那条断言**单独存在时守不住 documents_ad**：JOIN 会把 documents 表里已经消失的
    // 行过滤掉，哪怕 FTS 索引里还留着陈旧条目，结果照样是 0 行。陈旧条目的真实后果要等到
    // 新文档复用同一个 rowid_pk 时才显形——那时旧词会指向一篇根本不含它的新文档。
    // 实测：阉割 documents_ad 后只有下面这条变红，上面那条依然绿。
    store
        .write(|tx| {
            tx.execute(
                INSERT_DOC,
                ("d2", "p1", "b.md", "另一篇", "这篇只谈别的主题。"),
            )?;
            Ok(())
        })
        .expect("insert after delete");

    assert_eq!(
        n(&store, "p1", "全新内容"),
        0,
        "陈旧的 FTS 条目不得把旧词指向复用了 rowid_pk 的新文档"
    );
    assert_eq!(n(&store, "p1", "别的主题"), 1, "新文档自身应可被搜到");
}

/// `VACUUM` 会重编号**没有显式 INTEGER PRIMARY KEY** 的表的 rowid。external content FTS
/// 绑的是 `content_rowid='rowid_pk'`，一旦那一列变成隐式 rowid，压紧后索引与内容表错位——
/// 搜索返回**错误的文档**且不报错。先删中间一篇再 VACUUM，是为了让压紧真的有事可做。
#[test]
fn search_survives_vacuum() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("t.db");
    let store = open(&db).expect("open store");

    store
        .write(|tx| {
            tx.execute(INSERT_PROJECT, ["p1"])?;
            tx.execute(
                INSERT_DOC,
                ("d1", "p1", "a.md", "第一篇", "只有第一篇谈甲方案。"),
            )?;
            tx.execute(
                INSERT_DOC,
                ("d2", "p1", "b.md", "第二篇", "只有第二篇谈乙方案。"),
            )?;
            tx.execute(
                INSERT_DOC,
                ("d3", "p1", "c.md", "第三篇", "只有第三篇谈丙方案。"),
            )?;
            tx.execute("DELETE FROM documents WHERE id = 'd2'", [])?;
            Ok(())
        })
        .expect("seed three docs");

    let before = hits(&store, "p1", "丙方案");
    assert_eq!(before.len(), 1);
    assert_eq!(before[0].doc_id, "d3");

    // VACUUM 不能在事务里跑，只读池又是 query_only——收尾后用裸连接执行。
    store.close().expect("close before vacuum");
    rusqlite::Connection::open(&db)
        .expect("raw connection")
        .execute_batch("VACUUM;")
        .expect("vacuum");

    let store = open(&db).expect("reopen store");
    let after = hits(&store, "p1", "丙方案");
    assert_eq!(after.len(), 1, "VACUUM 后仍应命中一行");
    assert_eq!(
        after[0].doc_id, "d3",
        "VACUUM 后返回的必须还是同一篇——不同则说明 rowid 已错位"
    );
}

/// 同一段内容分别写进两个 project：搜索必须按 project 隔离。
/// 两条分支各测一次——`project_id` 在 MATCH 分支上是 JOIN 之后的过滤条件，
/// 在 LIKE 分支上是另一条独立 SQL，两处都可能各自漏掉。
#[test]
fn search_is_scoped_to_project() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = open(&dir.path().join("t.db")).expect("open store");

    store
        .write(|tx| {
            tx.execute(INSERT_PROJECT, ["p1"])?;
            tx.execute(INSERT_PROJECT, ["p2"])?;
            tx.execute(INSERT_DOC, ("d1", "p1", "a.md", SAMPLE_TITLE, SAMPLE_BODY))?;
            tx.execute(INSERT_DOC, ("d2", "p2", "a.md", SAMPLE_TITLE, SAMPLE_BODY))?;
            Ok(())
        })
        .expect("seed two projects");

    let long = hits(&store, "p1", "锚定引擎");
    assert_eq!(long.len(), 1, "MATCH 分支应只返回本 project 的一行");
    assert_eq!(long[0].doc_id, "d1");

    let short = hits(&store, "p1", "锚定");
    assert_eq!(short.len(), 1, "LIKE 分支应只返回本 project 的一行");
    assert_eq!(short[0].doc_id, "d1");

    assert_eq!(
        n(&store, "p2", "锚定引擎"),
        1,
        "另一 project 自查应仍能命中"
    );
}
