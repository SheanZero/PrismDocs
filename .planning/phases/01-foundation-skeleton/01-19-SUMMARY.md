---
phase: 01-foundation-skeleton
plan: 19
subsystem: store-tests
tags: [tautological-assertion, sqlite, concurrency, version-floor, gap-closure]
status: complete

requires:
  - "crates/prism-store/src/open.rs（01-18 落地的 journal_mode 校验、checkpoint busy 上报、MIN_SQLITE 常量）"
  - "01-REVIEW-prior.md WR-01 / IN-02"
  - "01-VERIFICATION.md § Anti-Patterns Found 第三条新增 warning（lib.rs:53-60）"
provides:
  - "`writer_commits_while_a_reader_holds_a_pooled_connection`：恒真断言换成 `assert_eq!(after, 2)`，名字/注释/断言对同一条真实性质一致"
  - "`sqlite_version_meets_the_pinned_minimum`：SC-3 的 ≥3.51.3 下界被断言钉住（比到 patch 位）"
  - "下界数字在仓库里只有 `open.rs::MIN_SQLITE` 一个来源；两条测试都引用它"
  - "`insert_samples` 返回累加自 `stmt.execute()` 的实际受影响行数"
  - "两条新单测：返回值与库内实际计数对账；未跑迁移时必须 Err"
affects:
  - "`/gsd-verify-work`：两处测试改名，旧命令行失效（新旧名字对照见下）"
  - "改 SQLite pin 时只需改 `MIN_SQLITE` 一处，两条测试自动跟上"
  - "`Engine::seed_sample_docs` 的调用点未改动（签名仍为 `Result<usize, StoreError>`）"

tech-stack:
  added: []
  patterns:
    - "测试的名字、注释、断言三者必须对同一条**真实存在**的性质陈述一致；被削弱到恒真的断言往往正是让三者不撞车的东西"
    - "判别力要实测而不是推断：写者不被读者阻塞这一条在任何 journal 模式下都成立（读走 autocommit），必须在读者持未提交事务时才有判别力"
    - "断言落在被守的量本身（版本元组 ≥ 下界）而不是它的一个弱化投影（major == 3）"
    - "返回值应来自执行结果；常量返回值报告不了它被要求报告的失败"
    - "常量与实际值在顺境下相等时，测试的判别力必须由扰动实验证明，并把这一事实写进测试的 doc 注释"

key-files:
  created: []
  modified:
    - crates/prism-store/tests/concurrency.rs
    - crates/prism-store/src/lib.rs
    - crates/prism-store/src/seed.rs

decisions:
  - "测试名用 `writer_commits_while_a_reader_holds_a_pooled_connection` 而非评审建议的 `writer_is_not_blocked_by_an_open_reader`：后者的「open reader」会被读成「持有未提交读事务的读者」，而实测表明那种读者**确实**会阻塞写者（非 WAL 下），今天的读者只是持有一条池连接。名字过度承诺正是本 plan 要关掉的形态"
  - "`insert_samples` 走路径 A（累加 `stmt.execute()`）而非改签名：调用点 `Engine::seed_sample_docs` 因此一行未动，且返回值获得报告失败的能力"
  - "项目行的 `INSERT ... DO NOTHING` 不计入返回值：它不是文档，且第二次播种起返回 0"
  - "版本解析助手在 `lib.rs` 与 `concurrency.rs` 各留一份按位解析副本，不改 `open.rs`：那一份是 `open` 模块私有的，两处测试都访问不到（集成测试更是跨 crate 边界）；三处注释互相点名同源"
  - "两处测试改名造成的失效引用（01-VALIDATION.md / 01-RESEARCH.md / 01-03-PLAN.md / 01-03-SUMMARY.md）不改：都是历史规划产物，新名字在本 SUMMARY 登记"

requirements-completed: [INFRA-02]

coverage:
  - id: D1
    description: "并发纪律测试里不再有恒真断言；第二次读的断言落在实际的读可见性语义上"
    requirement: "INFRA-02"
    verification:
      - kind: integration
        ref: "crates/prism-store/tests/concurrency.rs#writer_commits_while_a_reader_holds_a_pooled_connection"
        status: pass
      - kind: other
        ref: "反证 A：`Store::read` 临时改为持 deferred 事务 → assert_eq!(after, 2) 变红（left: 1 / right: 2）"
        status: pass
      - kind: other
        ref: "反证 B：反证 A + journal_mode=DELETE → 写者那条 expect 变红（DatabaseBusy，5.19s）"
        status: pass
    human_judgment: false
  - id: D2
    description: "SC-3 的 SQLite 版本下界被断言钉住，且数字只有一个来源"
    requirement: "INFRA-02"
    verification:
      - kind: unit
        ref: "crates/prism-store/src/lib.rs#sqlite_version_meets_the_pinned_minimum"
        status: pass
      - kind: integration
        ref: "crates/prism-store/tests/concurrency.rs#bundled_sqlite_meets_minimum（引用 MIN_SQLITE）"
        status: pass
      - kind: other
        ref: "反证 D：MIN_SQLITE 抬到 (3,99,0) 并旁路 open() 闸门 → 两条断言直接变红；同条件下旧形态（p[0]==3）保持绿"
        status: pass
    human_judgment: false
  - id: D3
    description: "insert_samples 的返回值来自执行结果并能报告失败"
    requirement: "INFRA-02"
    verification:
      - kind: unit
        ref: "crates/prism-store/src/seed.rs#the_returned_count_matches_the_rows_actually_in_the_database"
        status: pass
      - kind: unit
        ref: "crates/prism-store/src/seed.rs#a_failed_statement_propagates_instead_of_reporting_a_count"
        status: pass
      - kind: other
        ref: "反证 C：跳过循环里一条文档 → 常量实现红（left 3 / right 2），累加实现绿（2 / 2）"
        status: pass
    human_judgment: false

metrics:
  duration: ~8min
  tasks: 3
  files: 3
completed: 2026-07-29
---

# Phase 01 Plan 19: 三条「看不见降级」的断言关闭 Summary

把 prism-store 侧三处「无论代码是否工作都成立」的东西换成有判别力的：并发测试里 `assert!(after >= 1)` 的恒真断言换成 `assert_eq!(after, 2)`（并让测试名与注释与它对齐）；SQLite 版本测试从只判 major 改成比到 patch 位且引用唯一的 `MIN_SQLITE`；`insert_samples` 的常量返回值改成累加自 `stmt.execute()` 的实际受影响行数。四条非恒真反证全部实跑，`open.rs` 收尾未留任何改动。

## Performance

- **Duration:** ~8 min
- **Started:** 2026-07-29T14:03:04Z
- **Tasks:** 3/3
- **Files modified:** 3

## Task Commits

1. **Task 1: 恒真断言 → 实际成立的读可见性事实** — `f9aeb5d`
2. **Task 2: 版本下界钉成断言 + 数字来源唯一** — `ea8c918`
3. **Task 3: `insert_samples` 返回实际写入** — `fbc32f4`（test, RED 证据）→ `a7a5b7a`（fix, GREEN）

## 测试改名对照（供 `/gsd-verify-work`）

| 旧名 | 新名 | 位置 |
|---|---|---|
| `reader_snapshot_is_isolated` | `writer_commits_while_a_reader_holds_a_pooled_connection` | `crates/prism-store/tests/concurrency.rs` |
| `sqlite_version_returns_three_dotted_numbers` | `sqlite_version_meets_the_pinned_minimum` | `crates/prism-store/src/lib.rs` |

新命令行：

```bash
cargo test -p prism-store --test concurrency writer_commits_while_a_reader_holds_a_pooled_connection
cargo test -p prism-store --lib sqlite_version_meets_the_pinned_minimum
```

**已随改名失效的引用**（全部是历史规划产物，按计划**未改**）：

- `.planning/phases/01-foundation-skeleton/01-VALIDATION.md:100`（INFRA-02 Test Map 行）
- `.planning/phases/01-foundation-skeleton/01-RESEARCH.md:1148`（同一张表的研究期版本；另有 856 行的示例代码用的是第三个名字 `reader_snapshot_is_isolated_from_concurrent_write`，从未落地）
- `.planning/phases/01-foundation-skeleton/01-03-PLAN.md:21/27/41/102/268/322`
- `.planning/phases/01-foundation-skeleton/01-03-SUMMARY.md:104`（coverage ref）

源码里**没有**任何一处引用旧名字（`grep -rn` 全仓，排除 `target/` 与 `.git/`）。

## What Was Built

### Task 1 —— 恒真断言换成实际成立的事实（`f9aeb5d`）

`reader_snapshot_is_isolated` 的三个部件原本各说各的：名字承诺快照隔离、第 49 行注释断言快照隔离、被测的 `Store::read` 不实现快照隔离——唯一让三者不撞车的是一条被削弱到恒真的断言（`before` 已断言为 1、行只增不减，`after ∈ {1,2}` 恒满足 `>= 1`）。

改动：

- 改名为 `writer_commits_while_a_reader_holds_a_pooled_connection`
- 第 49 行注释「同一个读连接仍在自己的事务快照里」改为「写已经提交；这一读在 autocommit 下取新快照，因此看得见它」
- `assert!(after >= 1)` → `assert_eq!(after, 2, "autocommit 下的第二次读应看见期间提交的那一行")`，并在断言上方写明「写成 `>= 1` 就是恒真」
- 测试上方的 doc 注释写清三件事：这条测试真正独有的价值、`Store::read` 为什么不提供快照隔离、以及「若将来需要该性质必须**先实现再断言**」的次序要求

`before == 1` 的前置断言与末尾「写者的提交对之后取出的读连接可见」两条保留。

### Task 2 —— 版本下界钉成断言，数字来源唯一（`ea8c918`）

- `lib.rs`：测试改名为 `sqlite_version_meets_the_pinned_minimum`；断言从 `p[0] == 3` 改成解析元组 `>= MIN_SQLITE`；「三段点分」的形态断言保留（`version.split('.').count() == 3`，与下界是两条不同的性质）
- `concurrency.rs::bundled_sqlite_meets_minimum`：写死的 `(3, 51, 3)` 改为引用 `prism_store::MIN_SQLITE`
- 两处的版本解析助手都改成**按位**解析（`Option<(u32,u32,u32)>`，任一分量缺失或不可解析即 `None`），注释点名它与 `open.rs::parse_sqlite_version` 同源以及为什么是副本

数字来源核对：

```bash
$ grep -rn '3, 51, 3\|3\.51\.3' crates/prism-store/
crates/prism-store/src/open.rs:30:pub const MIN_SQLITE: (u32, u32, u32) = (3, 51, 3);
crates/prism-store/src/open.rs:274:            ("3.51.3", Some((3, 51, 3))), // 恰好等于下界
crates/prism-store/src/lib.rs:62:    /// 成功标准 3 的「bundled SQLite ≥3.51.3」在**运行期**的落点。
```

三处命中的性质：第一处是唯一的下界定义；`open.rs:274` 是 01-18 表驱动**解析**测试的输入夹具（它的准入期望值算自 `MIN_SQLITE`，不是第二份下界，改 pin 不会让它失真，只有那行行内注释的措辞会过时——`open.rs` 不在本 plan 范围内，未动）；`lib.rs:62` 是对 SC-3 原文的引用性 doc 注释。**没有第二处参与比较的字面量。**

### Task 3 —— `insert_samples` 返回实际写入（`fbc32f4` → `a7a5b7a`）

选路径 A：`written += stmt.execute(...)?`，返回累加值。文档注释同步说明为什么不是 `SAMPLE_DOCS.len()`、项目行那条 `DO NOTHING` 为什么不计入、以及 `DO UPDATE` 每行仍算一次受影响行（因此重复播种的返回值仍等于文档条数）。签名不变，`Engine::seed_sample_docs`（`crates/prism-engine/src/facade.rs:96`）一行未动。

两条新单测：

- `the_returned_count_matches_the_rows_actually_in_the_database`：返回值 vs `SELECT count(*) FROM documents WHERE project_id = ?`（**不是** vs `SAMPLE_DOCS.len()`），首播与重复播种各一次
- `a_failed_statement_propagates_instead_of_reporting_a_count`：未跑迁移的连接上调用必须 `Err`（`<behavior>` 第三条）

## 四条非恒真反证（全部实跑）

### 反证 A —— 真的实现快照隔离即变红（Task 1）

`open.rs::read` 临时改成 `let tx = conn.unchecked_transaction()?; f(&tx)`：

```
$ cargo test -p prism-store --test concurrency writer_commits
panicked at crates/prism-store/tests/concurrency.rs:88:5:
assertion `left == right` failed: autocommit 下的第二次读应看见期间提交的那一行
  left: 1
 right: 2
test result: FAILED. 0 passed; 1 failed
```

落点确实是「读的可见性语义」——闭包内一旦持事务，第二次读只看到 1 行。这同时证明：**要断言快照隔离，先实现它，那条断言才是真的。**

### 反证 B —— 写者不被阻塞那条的判别力边界（Task 1）

先按计划试「journal_mode 改 DELETE」单独一项。01-18 的校验会让 `open()` 先行报错，落点在 `open()` 而不在写者那条 expect 上；于是改用**组合**形态（反证 A 的持事务改动 + `journal_mode=DELETE` 并放宽校验），让读者真的持一个未提交读事务：

```
$ cargo test -p prism-store --test concurrency writer_commits
panicked at crates/prism-store/tests/concurrency.rs:81:10:
writer is not blocked by an open reader: Sqlite(SqliteFailure(
  Error { code: DatabaseBusy, extended_code: 5 }, Some("database is locked")))
test result: FAILED. 0 passed; 1 failed; finished in 5.19s
```

（5.19s = 走满 `BUSY_TIMEOUT_MS`。）

**同时跑的对照实验揭示了一条计划未预见的事实**：只把 journal 改成 DELETE、读仍走 autocommit（即今天 `Store::read` 的形态）时，写者**照常提交**：

```
$ cargo test -p prism-store --test concurrency writer_commits   # DELETE + autocommit 读
test writer_commits_while_a_reader_holds_a_pooled_connection ... ok
```

原因是 autocommit 的读语句一结束就放锁，读者在 `recv()` 上等待期间不持任何锁。也就是说「写者不被持连接的读者阻塞」这一条在**任何** journal 模式下都成立，判别力有限；这条测试真正判别的是 `assert_eq!(after, 2)`。该事实已写进测试的 doc 注释（见偏离 2），并指明真正把 WAL 的必要性钉住的是 `open.rs::open_leaves_the_database_in_wal_mode`。

### 反证 C —— 常量返回值 vs 累加返回值（Task 3）

扰动：循环里 `if id == "smoke-doc-3" { continue; }`，不动返回逻辑。

**常量实现下（改动前）：**

```
$ cargo test -p prism-store --lib seed::tests::the_returned_count
panicked at crates/prism-store/src/seed.rs:160:9:
assertion `left == right` failed: 首次播种的返回值应等于库里实际存在的样例文档数
  left: 3
 right: 2
test result: FAILED. 0 passed; 1 failed
```

**累加实现下（改动后，同一扰动）：**

```
$ cargo test -p prism-store --lib seed::tests::the_returned_count
test seed::tests::the_returned_count_matches_the_rows_actually_in_the_database ... ok
```

返回值跟着库状态一起变成 2，两者仍然一致。同一次扰动下 `seeding_twice_leaves_exactly_one_copy` 变红——因为**它**比的是常量 `SAMPLE_DOCS.len()`。两条测试在同一个扰动下的相反表现，正好是「与库状态对账」和「与常量对账」的差别。

（方向与计划的预判相反，见偏离 1。）

### 反证 D —— 版本下界（Task 2）

**第一步**：`MIN_SQLITE` 临时改成 `(3, 99, 0)`。两条测试都红，但落点在 `open()` 的准入闸门上，不在新断言上：

```
panicked at crates/prism-store/src/lib.rs:71:67:        open store: SqliteTooOld("3.53.2")
panicked at crates/prism-store/tests/concurrency.rs:162:61: open store: SqliteTooOld("3.53.2")
```

**第二步**（隔离断言本身）：同时临时旁路 `assert_sqlite_version(&writer)?`，让 `open()` 成功：

```
panicked at crates/prism-store/src/lib.rs:80:9:
bundled SQLite 3.53.2 is older than the pinned minimum (3, 99, 0)

panicked at crates/prism-store/tests/concurrency.rs:169:5:
bundled SQLite 3.53.2 is older than the pinned minimum (3, 99, 0)
```

**第三步（判别性对照，本 Task 的关键证据）**：同样条件下把 `lib.rs` 的断言临时改回旧形态（`p[0] == 3`）：

```
$ cargo test -p prism-store --lib sqlite_version
test tests::sqlite_version_meets_the_pinned_minimum ... ok
test result: ok. 1 passed
```

旧形态在 pin 回退下**保持绿**——这正是 `01-VERIFICATION.md` 第三条 warning 描述的形态，现在被关闭。

### 收尾：临时改动全部还原

```
$ git status --porcelain            # 反证全部还原后
 M crates/prism-store/src/seed.rs   # 仅当次任务的正式改动
```

`crates/prism-store/src/open.rs` 未出现在任何一次提交的改动集中，也不在收尾的 `git status` 里（`git diff --stat crates/prism-store/src/open.rs` 为空）。

## Verification

```
$ cargo test -p prism-store
test result: ok. 28 passed    (lib，原 26 + 新 2)
test result: ok. 6 passed     (tests/concurrency.rs，一条不减)
test result: ok. 4 passed     (tests/fts_cjk.rs)

$ cargo clippy -p prism-store --all-targets -- -D warnings
Finished `dev` profile — 0 warning

$ cargo test --workspace
27 个 "test result: ok" 行，0 failed

$ cargo test -p prismdocs-shell --features test
test result: ok. 21 passed / 2 passed

$ bash scripts/check-secrets.sh all
OK: pattern discriminates (19 positive / 10 negative samples)
OK: no plaintext secret in 114 version-controlled files
```

`lib` 的 5.48s 里约 5.2s 是 01-18 刻意保留的 busy 复现测试（见 01-18-SUMMARY 偏离 3），本 plan 未触碰。

## Deviations from Plan

### 1. [Rule 1 - 计划预判方向相反] 反证 C 的红绿方向与计划所写相反

- **Found during:** Task 3 反证
- **Issue:** 计划的验收准则写「扰动后新单测变红；**改动前**同一实验不会变红（返回的是常量）」。实测正好相反：常量实现下扰动让测试**变红**（返回 3、实际 2），累加实现下**保持绿**（返回 2、实际 2）。
- **处置:** 不改代码，改叙述——因为实测方向才是正确的判别方向：一条把返回值与库状态对账的断言，本来就应该在「返回值撒谎」时红、在「返回值诚实」时绿。计划把它写反了。
- **Verification:** 两次输出都抄在反证 C。
- **Committed in:** `fbc32f4` / `a7a5b7a`（提交信息里已按实测方向记录）

### 2. [Rule 2 - 补关键事实] 「写者不被阻塞」这一条的判别力有限，已写进 doc 注释

- **Found during:** Task 1 反证 B
- **Issue:** 计划与评审都把「读者持连接期间写者能提交」当作这条测试「唯一有价值的性质」。实测表明：因为闭包里的读走 autocommit、语句一结束就放锁，这条性质在 **DELETE journal 下也成立**——它只有在读者持一个未提交读事务时才有判别力。若只按计划改而不记这一笔，测试的名字会重新开始过度承诺（正是本 plan 要关掉的形态）。
- **Fix:** 在测试的 doc 注释里加一节「判别力边界（实测）」，写明这条性质在任何 journal 模式下都成立、真正的判别点是 `assert_eq!(after, 2)`、并指向 `open.rs::open_leaves_the_database_in_wal_mode` 作为 WAL 必要性的真正哨兵。
- **Files modified:** `crates/prism-store/tests/concurrency.rs`
- **Committed in:** `f9aeb5d`

### 3. [Rule 2 - 命名精度] 未采用评审建议的测试名

- **Found during:** Task 1
- **Issue:** 评审建议 `writer_is_not_blocked_by_an_open_reader`。「open reader」最自然的读法是「持有未提交读事务的读者」，而反证 B 表明**那种**读者确实会阻塞写者（非 WAL 下）。用这个名字等于换一个更隐蔽的过度承诺。
- **Fix:** 改用 `writer_commits_while_a_reader_holds_a_pooled_connection`——「持有一条池连接」是测试里真实发生的事。
- **Committed in:** `f9aeb5d`

### 4. [Rule 3 - 反证需要旁路] 反证 D 需临时旁路 `open()` 的版本闸门才能隔离断言落点

- **Found during:** Task 2 反证
- **Issue:** 单抬 `MIN_SQLITE` 时 `open()` 先行返回 `SqliteTooOld`，两条测试红在 `expect("open store")` 上，无法证明**新断言本身**有判别力（旧断言在那种条件下同样会红）。
- **Fix:** 分三步跑：抬常量 → 再旁路 `assert_sqlite_version` 隔离断言落点 → 在同条件下把断言换回旧形态做对照（旧形态保持绿）。三段输出都抄进反证 D。旁路改动全部还原，`open.rs` 未进入任何提交。
- **Committed in:** 无代码残留（反证过程）

---

**Total deviations:** 4（Rule 1 × 1、Rule 2 × 2、Rule 3 × 1）
**Impact on plan:** 无 scope 蔓延，三个 Task 的产出与计划一致。两条 Rule 2 都是把实测发现回写进注释与命名，反而让「名字不过度承诺」这条主线更硬。

## Known Stubs

无。本 plan 未引入桩、未跳过任何测试、`<verify>` 全部实跑。

## Issues Encountered

- 无。

## Edge Coverage Disposition（承接 INFRA-01 的 `concurrency` 探针在 prism-store 侧的落点）

计划的判定成立并已关闭：六条并发纪律测试中原本恰有一条（`reader_snapshot_is_isolated`）断言恒真，本 plan 关闭后六条各自具备判别力：

| 测试 | 判别点 | 反证 |
|---|---|---|
| `writer_commits_while_a_reader_holds_a_pooled_connection` | `assert_eq!(after, 2)`（读可见性语义） | 反证 A（read 持事务即红） |
| `pooled_connection_cannot_write` | 池连接写入必须报 readonly | 去掉 `query_only=ON` 即红（01-03 已跑） |
| `every_pooled_connection_is_query_only` | 池里**每一条**连接分别断言 | 同上 |
| `bundled_sqlite_meets_minimum` | 版本元组 ≥ `MIN_SQLITE` | 反证 D 第二步 |
| `wal_truncated_on_close` | `-wal` 长度收敛到 0 | 01-18 反证 1 已实测其会红 |
| `reader_sees_migrated_schema` | 池连接看得见迁移后的表 | 去掉 `to_latest` 即红（01-18 记录） |

**补充一条判定修正**（不静默丢弃）：「写者不被读者阻塞」这条性质在当前 `Store::read` 形态下判别力有限（详见反证 B 与偏离 2）——它不是恒真断言（写者真失败时会红），但今天没有任何现实回归能让它红。已在测试 doc 注释里显式记账，不新增未决探针条目。

本 plan 不实现读闭包内的快照隔离；若将来需要该性质，必须先实现（`conn.unchecked_transaction()` + `TransactionBehavior::Deferred` 并在闭包期间持有）再断言——这条次序要求已写进测试的 doc 注释，不只留在本 SUMMARY 里。

## Next Phase Readiness

- 上轮 WR-01、IN-02 与 `01-VERIFICATION.md` 第三条 warning 全部关闭。
- `open.rs` 一字未动，与 01-18 的产出无冲突。
- 遗留一处措辞（非正确性）：`open.rs:274` 那行 `("3.51.3", Some((3, 51, 3))), // 恰好等于下界` 的行内注释在改 pin 后会过时；该文件属 01-18 范围，本 plan 未动，登记在此供后续 plan 顺手处理。

## Self-Check: PASSED

- `crates/prism-store/tests/concurrency.rs`、`crates/prism-store/src/lib.rs`、`crates/prism-store/src/seed.rs`、本 SUMMARY 四个文件均存在
- 四个提交（`f9aeb5d` `ea8c918` `fbc32f4` `a7a5b7a`）均在 `git log` 中
- `crates/prism-store/src/open.rs` 不在任一提交的改动集里

---
*Phase: 01-foundation-skeleton*
*Completed: 2026-07-29*
