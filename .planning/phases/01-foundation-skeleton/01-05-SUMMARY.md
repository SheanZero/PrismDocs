---
phase: 01-foundation-skeleton
plan: 05
subsystem: database
tags: [fts5, trigram, cjk, like-fallback, query-escaping, settings, url-validation, secret-guard]

# Dependency graph
requires:
  - "01-03：schema v1（documents.rowid_pk 显式整型主键、documents_fts external-content trigram、三同步触发器、settings 表）与 Store::read / Store::write 闭包式 API"
  - "01-04：prism_types::SearchHit 作为查询结果 DTO；prism-llm::secrets 界定了「密钥不入库」的另一半"
provides:
  - "prism_store::search()：按 chars().count() 分流的搜索（≥3 走 trigram MATCH，<3 走 documents 表 LIKE 回退），分流对调用方不可见"
  - "prism_store::escape_fts_query()：FTS5 查询语法层转义（整串包字面短语 + 内部双引号加倍）"
  - "escape_like_pattern + ESCAPE 子句：LIKE 模式语言层转义（私有，经 search 消费）"
  - "常量 MIN_TRIGRAM_CHARS = 3"
  - "prism_store::settings：get_setting / set_setting / is_secret_like_key / validate_base_url"
  - "常量 SETTING_BASE_URL / SETTING_MODEL / ALLOWED_URL_SCHEMES"
  - "StoreError 新增 InvalidSetting(String) / InvalidUrl(String) 两个变体"
  - "tests/fts_cjk.rs：四个判别性集成测试，三条触发器路径各有一条落点不同的反证"
affects: [01-08-冒烟命令, 01-09-冒烟页settings, phase-2-导入与搜索, phase-4-LLM端点]

# Tech tracking
tech-stack:
  added:
    - "prism-types（prism-store 首次消费共享 DTO）"
    - "tracing 0.1（prism-store：非 loopback http 端点的明文告警）"
  patterns:
    - "两层转义纪律：参数绑定只保护 SQL 语法层，MATCH 实参还要过 FTS5 查询语言层，LIKE 实参还要过模式语言层——每多一层解析器就多一层转义"
    - "校验与守卫长在写入路径内部而非调用方：放调用方是约定，放 set_setting 里才是机制"
    - "每条分支各自配一条阴性对照——共用一条阴性对照会让另一条分支半边恒真"
    - "SQL 一律为 &str 常量 + 位置参数；search.rs 里 format! 出现 0 次"

key-files:
  created:
    - crates/prism-store/src/search.rs
    - crates/prism-store/src/settings.rs
    - crates/prism-store/tests/fts_cjk.rs
  modified:
    - crates/prism-store/src/lib.rs
    - crates/prism-store/src/error.rs
    - crates/prism-store/Cargo.toml
    - Cargo.lock

key-decisions:
  - "FTS 表在 SQL 里**不能起别名**：`FROM documents_fts f … WHERE f MATCH ?` 得到 `no such column: f`（计划与 RESEARCH 给的 SQL 都带别名，实跑不通）。JOIN 条件改打在 `d.rowid_pk = documents_fts.rowid` 上"
  - "LIKE 回退分支补了模式语言层转义（`%` `_` `\\` + ESCAPE 子句）：未转义时一次 `%` 查询命中全库——与未转义 MATCH 是同一类漏洞的两个面，计划只写了 MATCH 那一面"
  - "空查询（trim 后为空）提前返回空结果：LIKE 分支上它会退化成「匹配全部」"
  - "set_setting 内部对 SETTING_BASE_URL 强制跑 validate_base_url：计划写的是「写入该键的路径必须先过校验」，落在调用方就是约定，落在 set_setting 里才是机制"
  - "不勾选 INFRA-03：base_url 校验只是其中一块，01-07 集成验证与 01-09 settings 页仍未完成（沿用 01-01/01-04 的口径）"

patterns-established:
  - "触发器同步必须从**搜索 API** 一侧验证，且 DELETE 路径要让新文档复用 rowid_pk 才验得出来（见 Deviations #2）"
  - "反证要落在**具体哪一条断言**上，而不只是「测试变红」——落点不同才证明各条断言互不替代"

requirements-completed: []

coverage:
  - id: D1
    description: "中文查询返回非零结果：4 字词与 3 字边界词走 trigram MATCH 各命中 1 行"
    requirement: "INFRA-02"
    verification:
      - kind: integration
        ref: "crates/prism-store/tests/fts_cjk.rs#chinese_query_returns_nonzero_rows 断言①②"
        status: pass
      - kind: other
        ref: "反证 A：migration 001 的 tokenize 临时改为 unicode61 → 断言①（4 字中文词应命中）变红 left=0；撤销后转绿"
        status: pass
    human_judgment: false
  - id: D2
    description: "2 字中文词经 D-02 的 LIKE 回退命中，分流对调用方不可见"
    requirement: "INFRA-02"
    verification:
      - kind: integration
        ref: "crates/prism-store/tests/fts_cjk.rs#chinese_query_returns_nonzero_rows 断言③"
        status: pass
      - kind: other
        ref: "反证 B：MIN_TRIGRAM_CHARS 临时改为 1 → **只有**断言③变红，①②④ 仍绿；撤销后转绿"
        status: pass
    human_judgment: false
  - id: D3
    description: "中英混排文档中的英文子串可被检索命中"
    requirement: "INFRA-02"
    verification:
      - kind: integration
        ref: "crates/prism-store/tests/fts_cjk.rs#chinese_query_returns_nonzero_rows 断言④（mixed）"
        status: pass
    human_judgment: false
  - id: D4
    description: "阴性对照成立：两条分支各自都不是「永远匹配」"
    requirement: "INFRA-02"
    verification:
      - kind: integration
        ref: "fts_cjk.rs#chinese_query_returns_nonzero_rows 断言⑤（量子纠缠，MATCH 分支）与 ⑤b（量子，LIKE 分支）"
        status: pass
      - kind: other
        ref: "反证 C：LIKE 分支临时改为无条件返回全部行 → 断言⑤b 变红 left=1（⑤ 仍绿，因它走 MATCH）；撤销后转绿"
        status: pass
    human_judgment: false
  - id: D5
    description: "FTS5 查询语法注入被挡住：含双引号与布尔算子的输入不报语法错误也不返回意外行"
    requirement: "INFRA-02"
    verification:
      - kind: integration
        ref: "fts_cjk.rs#chinese_query_returns_nonzero_rows 断言⑥（`设计\" OR 1=1` → 0 行且非 Err）"
        status: pass
      - kind: unit
        ref: "crates/prism-store/src/search.rs#tests::escape_doubles_inner_quotes / ::escape_wraps_the_whole_input_in_one_literal_phrase"
        status: pass
    human_judgment: false
  - id: D6
    description: "LIKE 模式语言层的通配符注入被挡住（计划未覆盖，Rule 2 补充）"
    requirement: "INFRA-02"
    verification:
      - kind: integration
        ref: "fts_cjk.rs#chinese_query_returns_nonzero_rows 断言⑦（`%` → 0 行）"
        status: pass
      - kind: unit
        ref: "crates/prism-store/src/search.rs#tests::like_pattern_escapes_wildcards_and_the_escape_char_itself"
        status: pass
    human_judgment: false
  - id: D7
    description: "FTS 索引在 INSERT / UPDATE / DELETE 三条触发器路径上均与内容表同步，且**经搜索 API** 可验"
    requirement: "INFRA-02"
    verification:
      - kind: integration
        ref: "crates/prism-store/tests/fts_cjk.rs#fts_index_follows_update_and_delete"
        status: pass
      - kind: other
        ref: "反证：阉割 documents_ai → 4 个测试全红（含「前置条件：插入后可搜到」）"
        status: pass
      - kind: other
        ref: "反证：阉割 documents_au → 只有「UPDATE 后旧词应搜不到」变红"
        status: pass
      - kind: other
        ref: "反证：阉割 documents_ad → 只有「陈旧的 FTS 条目不得把旧词指向复用了 rowid_pk 的新文档」变红（见 Deviations #2）"
        status: pass
    human_judgment: false
  - id: D8
    description: "VACUUM 之后 FTS 索引与内容表仍对齐，搜索返回的仍是同一篇文档"
    requirement: "INFRA-02"
    verification:
      - kind: integration
        ref: "crates/prism-store/tests/fts_cjk.rs#search_survives_vacuum（插三篇 → 删中间一篇 → VACUUM → 断言 doc_id 仍为 d3）"
        status: pass
    human_judgment: false
  - id: D9
    description: "搜索结果按 project_id 隔离，MATCH 与 LIKE 两条分支各测一次"
    requirement: "INFRA-02"
    verification:
      - kind: integration
        ref: "crates/prism-store/tests/fts_cjk.rs#search_is_scoped_to_project"
        status: pass
    human_judgment: false
  - id: D10
    description: "settings 可读写非密钥配置：往返、覆盖写只留一行、未写入的 key 读回 Ok(None)、updated_at 为当前秒"
    requirement: "INFRA-03"
    verification:
      - kind: unit
        ref: "crates/prism-store/src/settings.rs#tests::settings_roundtrip"
        status: pass
    human_judgment: false
  - id: D11
    description: "base_url 只接受 http/https，非法值不进表，错误消息不回显 value"
    requirement: "INFRA-03"
    verification:
      - kind: unit
        ref: "crates/prism-store/src/settings.rs#tests::settings_base_url_validation"
        status: pass
      - kind: other
        ref: "反证 D：拆掉 set_setting 内的 `key == SETTING_BASE_URL` 校验 → 该测试变红；撤销后转绿"
        status: pass
    human_judgment: false
  - id: D12
    description: "疑似密钥的键名被写入层拒绝且表内查无此行（T-01-03b 的机制隔离）"
    requirement: "INFRA-03"
    verification:
      - kind: unit
        ref: "crates/prism-store/src/settings.rs#tests::settings_rejects_secret_like_keys"
        status: pass
      - kind: other
        ref: "反证 E：拆掉 is_secret_like_key 守卫 → 该测试变红；撤销后转绿"
        status: pass
      - kind: integration
        ref: "bash scripts/check-secrets.sh → exit 0"
        status: pass
    human_judgment: false
  - id: D13
    description: "prism-store 新增依赖未破坏依赖方向（prism-llm 仍是唯一网络/密钥出口）"
    requirement: "INFRA-03"
    verification:
      - kind: integration
        ref: "bash scripts/check-deps.sh → dup / tauri-free / no-cycle / single-egress 四条全 OK"
        status: pass
    human_judgment: false

# Metrics
duration: 38min
completed: 2026-07-29
status: complete
---

# Phase 01 Plan 05: 查询层与 settings Summary

**按查询长度分流的中文可用搜索（≥3 字符 trigram MATCH / <3 字符 LIKE 回退）落地，配一组各有落点的判别性用例——三条计划反证加三条触发器反证，每条都落在不同的断言上；settings k/v 的 base_url 校验与密钥键名守卫长在写入路径内部而非调用方。**

## Performance

- **Duration:** ≈38 min
- **Tasks:** 2（各按 TDD 走 RED→GREEN 两段）
- **Files created/modified:** 7（新建 3，修改 4）
- **测试增量:** workspace 58 → **68 passed / 0 failed**（+4 fts_cjk 集成、+3 search 单测、+3 settings 单测）

## Accomplishments

- **成功标准 3 的后半兑现了，而且是可证伪地兑现的。** 中文查询返回非零结果这件事，单条冒烟断言也能"证明"——真正的问题是它同时也能被一个"永远返回全部"的实现证明。这组用例的价值在于**每一条被删掉之后都有一个具体的静默失败模式重新变得可能**，且这一点是跑出来的不是推出来的：反证 A（tokenizer 换回 unicode61）让 4 字中文那条变红，反证 B（取消分流）**只**让 2 字中文那条变红而 ①②④ 全绿，反证 C（回退分支恒真）让新加的 ⑤b 变红。三条反证落点互不重叠，这才说明三条断言互不替代。
- **触发器同步这次是从搜索 API 一侧真验到了，而且过程中发现原设计验不出来。** 详见 Deviations #2——按计划写法阉割 `documents_ad` 测试照样全绿，因为 JOIN 会把已删除的行过滤掉，把陈旧索引条目掩盖得干干净净。补上"新文档复用 rowid_pk"这一步之后，三个触发器各自对应一条落点不同的反证。
- **两层转义补齐成了两条分支各一层。** 计划只写了 MATCH 侧的 FTS5 查询语法转义；LIKE 侧有完全同构的洞——一次 `%` 查询命中全库。现在 `search.rs` 里 `format!` 出现 0 次，SQL 全是 `&str` 常量配位置参数。
- **settings 的两道守卫都是机制不是约定。** `is_secret_like_key` 与 `base_url` 校验都长在 `set_setting` 内部，调用方绕不过去；两者各有一条拆掉即变红的反证。
- **migration 001 一个字符没动。** 五轮反证全部改完即刻复原，`git log -1 -- 001_schema_v1.sql` 仍停在 01-03 的 `875b9c8`。

## Task Commits

1. **Task 1: 分流搜索、FTS5 语法转义与判别性 CJK 测试** — TDD 两段
   - `bc8cf38` (test) — RED：`tests/fts_cjk.rs` 四个测试 + prism-types 依赖（`no 'search' in the root`，失败隔离在该测试目标内）
   - `63a73aa` (feat) — GREEN：`src/search.rs` + `lib.rs` re-export（4 passed）
2. **Task 2: settings k/v、base_url 校验与密钥键名守卫** — TDD 两段
   - `9d6c06f` (test) — RED：`settings.rs` 只含测试模块（6 个 `cannot find … in this scope`）
   - `a2f4eeb` (feat) — GREEN：常量、四个函数、`StoreError` 两个新变体、tracing 依赖（17 lib passed）
3. `cf565a7` (chore) — Cargo.lock 锁定新增依赖

## Files Created/Modified

- `crates/prism-store/src/search.rs`（新，141 行）— `MIN_TRIGRAM_CHARS`、`SQL_MATCH` / `SQL_LIKE` 两条 SQL 常量、`escape_fts_query`（pub）、`escape_like_pattern`（私有）、`search`、3 个转义单测
- `crates/prism-store/src/settings.rs`（新，235 行含测试）— `SETTING_BASE_URL` / `SETTING_MODEL` / `ALLOWED_URL_SCHEMES`、`is_secret_like_key` / `validate_base_url` / `get_setting` / `set_setting`、3 个单测
- `crates/prism-store/tests/fts_cjk.rs`（新，204 行）— 恰好 4 个 `#[test]` 函数；第一个含七组断言（① 4 字中文 ② 3 字边界 ③ 2 字回退 ④ 混排英文 ⑤ MATCH 阴性 ⑤b LIKE 阴性 ⑥ FTS 语法注入 ⑦ LIKE 通配符）
- `crates/prism-store/src/error.rs`（改）— `StoreError` 追加 `InvalidSetting(String)` / `InvalidUrl(String)`，两者的 `Display` 只带键名与规则
- `crates/prism-store/src/lib.rs`（改）— `pub mod search; pub mod settings;` + re-export
- `crates/prism-store/Cargo.toml` / `Cargo.lock`（改）— 新增 `prism-types` 与 `tracing`

## Decisions Made

除 frontmatter `key-decisions` 外，两处值得单独说明：

- **`search` 与模块同名不是问题。** `pub mod search;` 与 `pub use search::search;` 各自落在类型命名空间与值命名空间，`prism_store::search(conn, …)` 与 `prism_store::search::escape_fts_query` 两种路径同时成立。
- **`search_survives_vacuum` 先删中间一篇再 VACUUM。** 只插一篇再 VACUUM 的话压紧无事可做，rowid 是否会重编号根本没机会显形——那种写法下即使把 `rowid_pk` 换成隐式 rowid 也可能照样绿。删掉中间一篇是为了让"如果 rowid 会被重编号，它现在就该被重编号了"。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] 计划与 RESEARCH 给出的 MATCH SQL 带表别名，实跑不通**

- **Found during:** Task 1 GREEN 首跑
- **Issue:** 计划 `<action>` 与 RESEARCH § Pattern 3 的 SQL 都写成
  `FROM documents_fts f JOIN documents d ON d.rowid_pk = f.rowid WHERE documents_fts MATCH ?1`。
  四个测试全部报 `sqlite error: no such column: f`——FTS5 的 `MATCH` 左操作数必须是 fts5 表
  在当前作用域里的名字，一旦起了别名，原表名不在作用域内，而别名又不被 `MATCH` 接受。
- **Fix:** 去掉别名，`FROM documents_fts JOIN documents d ON d.rowid_pk = documents_fts.rowid
  WHERE documents_fts MATCH ?1`。计划 `key_links` 要求的 `rowid_pk` JOIN 语义不变。
- **Files modified:** `crates/prism-store/src/search.rs`
- **Verification:** 4 个测试全绿。
- **Committed in:** `63a73aa`

**2. [Rule 2 - Missing Critical] DELETE 触发器的验证按计划写法是恒真的**

- **Found during:** Task 1 验收（跑三个触发器反证时）
- **Issue:** 计划的 `fts_index_follows_update_and_delete` 到"DELETE 该文档后新词也返回 0 行"为止。
  实测**阉割 `documents_ad` 后这条断言依然绿**——搜索的 MATCH 分支要
  `JOIN documents d ON d.rowid_pk = documents_fts.rowid`，文档行既已从 `documents` 删除，
  JOIN 就把它过滤掉了，哪怕 FTS 索引里还留着陈旧条目，结果照样是 0 行。也就是说这条断言
  守的是"documents 表里删掉了"，守不住"FTS 索引里也删掉了"。而本 plan 正是被指定来兜底
  方案 A「触发器出错表现为搜不到而非报错」这一弱点的。
- **调查:** 陈旧条目的真实后果要等到**新文档复用同一个 rowid_pk** 时才显形——
  `rowid_pk INTEGER PRIMARY KEY` 无 AUTOINCREMENT，删空后下一次 INSERT 拿回同一个 rowid，
  于是旧词会经陈旧索引条目指向一篇根本不含它的新文档。这才是"搜到错文档且不报错"。
- **Fix:** DELETE 之后追加一次 INSERT（复用 rowid_pk）并断言旧词仍返回 0 行，
  外加一条"新文档自身应可被搜到"。
- **Files modified:** `crates/prism-store/tests/fts_cjk.rs`
- **Verification:** 阉割 `documents_ad` 后**只有**新加的这条变红（原断言仍绿），复原后转绿；
  阉割 `documents_ai` / `documents_au` 各自落在另外两条不同的断言上。
- **Committed in:** `63a73aa`

**3. [Rule 2 - Missing Critical] LIKE 回退分支缺模式语言层转义**

- **Found during:** Task 1 实现
- **Issue:** 计划给 MATCH 分支写了 FTS5 查询语法转义（Pitfall 6），LIKE 分支却是裸的
  `content LIKE '%'||?2||'%'`。`%` 与 `_` 在 LIKE 里是通配符——一次 `%` 查询命中全库。
  这与未转义的 MATCH 是同一类漏洞（参数绑定只保护 SQL 层，保护不了被绑定值所进入的
  那门子语言）的两个面，计划只写了一面。
- **Fix:** 加 `escape_like_pattern`（转义 `%` / `_` / `\`）并在两条 LIKE 上加 `ESCAPE '\'`；
  测试补断言⑦。
- **Files modified:** `crates/prism-store/src/search.rs`、`crates/prism-store/tests/fts_cjk.rs`
- **Committed in:** `63a73aa`

**4. [Rule 2 - Missing Critical] 回退分支缺自己的阴性对照**

- **Found during:** Task 1 跑反证 C 时
- **Issue:** 计划的验收项写"把 LIKE 回退分支改成无条件返回全部行后，失败断言指向**不存在词**那一条"。
  实跑不成立：唯一的阴性对照 ⑤ 用的是 4 字词「量子纠缠」，走的是 MATCH 分支，
  LIKE 分支怎么改它都不动。两条分支共用一条阴性对照 = 有半边恒真。
- **Fix:** 补断言 ⑤b——2 字的「量子」（走 LIKE）返回 0 行。
- **Files modified:** `crates/prism-store/tests/fts_cjk.rs`
- **Verification:** 反证 C 现在落在 ⑤b 上（left=1），⑤ 仍绿——正好印证了这条缺口是真的。
- **Committed in:** `63a73aa`

**5. [Rule 2 - Missing Critical] `base_url` 校验从调用方移进 `set_setting`**

- **Found during:** Task 2 实现
- **Issue:** 计划写"写入 `SETTING_BASE_URL` 的路径必须先过这个校验"。若停在字面，
  `set_setting(tx, SETTING_BASE_URL, "file:///etc/passwd")` 直接绕过——校验成了调用方的义务，
  也就是约定。而同一个文件里的 `is_secret_like_key` 计划明确要求是机制不是约定，两者标准应当一致。
- **Fix:** `set_setting` 内部对 `key == SETTING_BASE_URL` 强制跑 `validate_base_url`。
- **Files modified:** `crates/prism-store/src/settings.rs`
- **Verification:** 反证 D——拆掉这三行后 `settings_base_url_validation` 变红。
- **Committed in:** `a2f4eeb`

**6. [Rule 2 - Missing Critical] 空查询提前返回**

- **Found during:** Task 1 实现
- **Issue:** 空串或纯空白的查询 `chars().count() == 0 < 3`，落到 LIKE 分支就是 `LIKE '%%'`——
  一次"搜索空字符串"返回全库。
- **Fix:** `search` 开头 `trim()` 后为空即返回空 `Vec`。
- **Files modified:** `crates/prism-store/src/search.rs`
- **Committed in:** `63a73aa`

---

**Total deviations:** 6 auto-fixed（1 个 Rule 1 - Bug，5 个 Rule 2 - 缺失的关键性）
**Impact on plan:** 一条是计划 SQL 实跑不通（必须改），五条是补断言/补守卫——都没有引入计划外的功能面。
计划的 `must_haves`、`artifacts`、`key_links` 与 `prohibitions` 全部满足，`tests/fts_cjk.rs`
仍然恰好四个测试函数。

## Issues Encountered

- **本次最值得记下的一条：三条计划反证里有一条（反证 C）实跑不成立，而它不成立的原因恰好
  暴露了一个真实缺口。** 这与 01-03 记下的教训是同一件事的第二次出现——
  01-03 发现"颠倒迁移与建池顺序应当让某测试变红"是推断而非事实。**反证本身也需要被验证**：
  如果这次只跑到"反证 C 让测试变红了"就收工（它确实会红，红在断言⑦上），就会带着
  "LIKE 分支有阴性对照"这个错误认识进入 Phase 2。真正有用的是**落点**，不是红绿。
- 触发器反证（Deviations #2）同理：不逐个阉割三个触发器、不看红在哪一条，
  就不会发现 DELETE 那条是恒真的。
- `cargo fmt --all -- --check` 的既有差异仍在，早于本 plan 且 CI 无 fmt 步骤，
  按 scope boundary 未动（已在 01-03 登记于 `deferred-items.md`）。

## Known Stubs

**None.** 本 plan 交付的全部符号都有真实实现与测试。

`SETTING_MODEL` 与 `ALLOWED_URL_SCHEMES` 目前只被测试消费（真实读写方是 01-09 的 settings 页
与 Phase 4 的 LLM 客户端），但它们不是 stub——与 01-04 的 `ACCOUNT_MCP_TOKEN` 同理，
是**契约的一部分**：键名现在定死，下游才有一个已经进了代码的目标可对。

`.planning/WINDOWS.md` 不存在，本 plan 也无需登记（无 stub、无 skipped test、无未跑的 `<verify>`）。

## Threat Flags

None——本 plan 未引入计划 `<threat_model>` 之外的安全面。六条 `mitigate` 处置全部落地并各有反证：
T-01-01（FTS5 语法注入，断言⑥ + 两个转义单测）、T-01-25（`format!` 拼 SQL，search.rs 中 0 次）、
T-01-03b（密钥入库，反证 E）、T-01-07（base_url scheme，反证 D）、
T-01-26（错误回显 value，`settings_base_url_validation` 内的 `!msg.contains("alert")`）、
T-01-27（VACUUM rowid 错位，`search_survives_vacuum`）。

计划外补上的 LIKE 模式语言层转义（Deviations #3）属于 T-01-01 的同类面，已一并覆盖。

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- **01-08（冒烟命令与事件总线）** — `prism_store::search(conn, project_id, q)` 即命令层要委托的对象，
  经 `Store::read` 调用；返回的 `Vec<SearchHit>` 已是 serde-ready 的共享 DTO，命令层不必再做映射。
- **01-09（冒烟页 settings）** — `base_url` 走 `settings` 表（`set_setting` 会自己校验，
  命令层不必重复校验），API key 走 `prism_llm::secrets`。**不要**把 key 塞进 `settings`——
  `is_secret_like_key` 会直接拒掉，那是设计而不是障碍。
- **Phase 2（导入与搜索）** — 写路径**不需要**碰 `documents_fts`，三个触发器包办；
  Rust 侧任何 `INSERT INTO documents_fts` 都是设计错误（当前全仓库 0 处）。
  搜索的长度分流已封在 `search()` 内，调用方不必自己判断查询长度。
- **一句提醒:** `documents_fts` 在 SQL 里不能起别名（见 Decisions），
  后续任何新写的 MATCH 查询照抄 `SQL_MATCH` 的形状即可。

---
*Phase: 01-foundation-skeleton*
*Completed: 2026-07-29*

## Self-Check: PASSED

- 3 个新建文件全部在盘上：`crates/prism-store/src/search.rs`（141 行）、
  `crates/prism-store/src/settings.rs`（235 行）、`crates/prism-store/tests/fts_cjk.rs`（204 行）
  ——均超过 must_haves 的 min_lines（50 / 40 / 90）
- 5 个声明的提交全部在 `git log` 中：`bc8cf38` / `63a73aa` / `9d6c06f` / `a2f4eeb` / `cf565a7`
- `git diff --diff-filter=D --name-only <commit>~1 <commit>` 对五个 commit 均为空——未删除任何被跟踪文件
- `git log -1 -- crates/prism-store/migrations/001_schema_v1.sql` 仍为 `875b9c8`（01-03）——
  五轮反证改动全部复原，已发布的迁移一个字符没动
- `cargo test --workspace` → **68 passed / 0 failed**；`npm run test -- --run` → 3 passed
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0
- `bash scripts/check-deps.sh` → 四条全 OK；`bash scripts/check-secrets.sh` → exit 0
- `cargo test -p prism-store --test fts_cjk` → `test result: ok. 4 passed`；
  四个函数各自单独 `--test fts_cjk <name>` 运行亦 exit 0
- `grep -c '^#\[test\]' tests/fts_cjk.rs` → 4（恰好四个测试函数）
- `grep -c 'format!' src/search.rs` → 0；`grep -c 'Url::parse' src/settings.rs` → 1
- `grep -rn 'INSERT INTO documents_fts' --include='*.rs' crates/` → 0（触发器仍是唯一同步路径）
