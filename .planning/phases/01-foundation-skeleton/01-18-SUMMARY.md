---
phase: 01-foundation-skeleton
plan: 18
subsystem: store-open
tags: [sqlite, wal, checkpoint, version-parsing, silent-degradation, gap-closure]
status: complete

requires:
  - "crates/prism-store/src/open.rs（01-03 建立的 writer-first 六步序与只读池）"
  - "crates/prism-store/src/error.rs（T-01-20：错误文本不含库路径的约定）"
  - "01-REVIEW-prior.md WR-02 / WR-03 / IN-03"
provides:
  - "`open()` 读回 `PRAGMA journal_mode=WAL` 的返回行并校验；不是 wal 时当场 Err（只带模式串）"
  - "`close()` 读出 `PRAGMA wal_checkpoint(TRUNCATE)` 的 busy 列；非 0 时 Err 而不是静默成功"
  - "`parse_sqlite_version`：按位解析的私有纯函数，畸形串返回 None 而不是被重排成一个碰巧够新的元组"
  - "busy 分支的行为哨兵测试（真实复现，非源码断言）"
affects:
  - "Phase 2+：WAL 起不来不再表现为 5 秒 busy_timeout 下的偶发 SQLITE_BUSY，而是 open() 当场报错"
  - "备份完整性：未 checkpoint 的 WAL 内容不再被静默留在 -wal 里"
  - "plan 01-19 将在 lib.rs / seed.rs / tests/concurrency.rs 上继续改动——本 plan 未触碰它的范围"

tech-stack:
  added: []
  patterns:
    - "SQLite 里「用返回行而不是错误报告结果」的 pragma（journal_mode、wal_checkpoint）必须 query_row 读回校验；execute_batch 对它们等于把结果扔掉"
    - "断言要落在**解析结果**上而不是落在**下游布尔**上：下游布尔可能因为常量取值碰巧一致而丧失判别力（本 plan 实测，见偏离 2）"
    - "反证的价值取决于反例的选取：`3.x.51` 重排成 (3,51,0) 恰好也不够新，真正的放行口是 `3.x.53` → (3,53,0)"
    - "新增 StoreError 变体优先落在 `EngineError::Store(_)` 兜底臂，不扩 IPC 短码表（沿用 01-10 决策）"

key-files:
  created: []
  modified:
    - crates/prism-store/src/open.rs
    - crates/prism-store/src/error.rs

decisions:
  - "两个新变体（JournalModeNotWal / CheckpointBusy）都走 `EngineError::Store(_)` → `store_error` 兜底臂，IPC 短码契约与前端 ERROR_COPY 均不动（已核对 src-tauri/src/commands.rs:38-40）"
  - "畸形版本串复用 SqliteTooOld 而不是加第三个变体：语义上「无法证明它够新」与「确实太旧」同一处置"
  - "把计划标为「跑完即删」的 busy 复现测试**留下**（Rule 2）：删掉它，close() 里读 busy 列的三行就再没有任何行为面保护，下一次重构改回 `|_| Ok(())` 不会有东西变红"
  - "版本表驱动测试断言在 parse 结果（Option）上，并补第七条用例 `3.x.53`——只断言准入布尔在当前 MIN_SQLITE 下完全不具判别力"

requirements-completed: [INFRA-02]

coverage:
  - id: D1
    description: "open() 读回 journal_mode 的返回行并校验，非 wal 时返回只带模式串的错误"
    requirement: "INFRA-02"
    verification:
      - kind: unit
        ref: "crates/prism-store/src/open.rs#open_leaves_the_database_in_wal_mode"
        status: pass
      - kind: unit
        ref: "crates/prism-store/src/open.rs#journal_mode_error_carries_no_path"
        status: pass
      - kind: other
        ref: "反证：临时把 PRAGMA journal_mode=WAL 改成 DELETE → 7 条 lib 测试变红，失败文本为 JournalModeNotWal(\"delete\")"
        status: pass
    human_judgment: false
  - id: D2
    description: "close() 读出 checkpoint 的 busy 列，非 0 时返回 CheckpointBusy 而不是 Ok(())"
    requirement: "INFRA-02"
    verification:
      - kind: unit
        ref: "crates/prism-store/src/open.rs#close_reports_busy_when_another_connection_holds_the_wal"
        status: pass
      - kind: unit
        ref: "crates/prism-store/src/open.rs#checkpoint_busy_error_carries_no_path"
        status: pass
      - kind: integration
        ref: "crates/prism-store/tests/concurrency.rs#wal_truncated_on_close"
        status: pass
    human_judgment: false
  - id: D3
    description: "SQLite 版本串按位解析，任一分量缺失或不可解析即拒绝准入"
    requirement: "INFRA-02"
    verification:
      - kind: unit
        ref: "crates/prism-store/src/open.rs#version_admission_rejects_malformed_strings（7 条用例）"
        status: pass
      - kind: other
        ref: "反证：把 parse_sqlite_version 改回 filter_map 形态 → parse of \"3.x.51\" left Some((3,51,0)) / admission of \"3.x.53\" left true"
        status: pass
    human_judgment: false

metrics:
  duration: ~15min
  tasks: 3
  files: 2
completed: 2026-07-29
---

# Phase 01 Plan 18: open.rs 三条静默降级关闭 Summary

把 `open.rs` 里三处「错了不会有任何东西变红」的地方变成显式错误：`journal_mode=WAL` 的返回行被 `query_row` 读回并 `eq_ignore_ascii_case` 校验；`close()` 读出 `wal_checkpoint(TRUNCATE)` 的 busy 列并在非 0 时报错；版本串改为按位解析、畸形串直接拒绝，并配一条真正有判别力的表驱动测试。

## Performance

- **Duration:** ~15 min
- **Started:** 2026-07-29T12:19:47Z
- **Tasks:** 3/3
- **Files modified:** 2

## Task Commits

1. **Task 1: journal_mode 返回行校验** — `59e609f` (test, RED) → `194bd96` (fix, GREEN)
2. **Task 2: close() 上报 checkpoint busy** — `1693d6a` (test, RED) → `50c010e` (fix, GREEN)
3. **Task 3: 版本串按位解析** — `e6460ca` (test, RED) → `f1fc3e0` (fix, GREEN)

## What Was Built

### Task 1 —— journal_mode 的返回行被读回并校验（`194bd96`）

`PRAGMA journal_mode=WAL` 从四条 pragma 的 `execute_batch` 里拆出来单走 `query_row`，读第 0 列（结果模式串），`eq_ignore_ascii_case("wal")` 不成立即 `Err(StoreError::JournalModeNotWal(mode))`。余下三条（`synchronous=NORMAL` / `busy_timeout` / `foreign_keys=ON`）仍在同一批，顺序与语义不变。模块头的六步序文档同步改为「设 journal_mode 并**读回它的返回行校验**——只设一次的东西更要验一次」。

新变体只携带模式串，不带库路径（T-01-20）。

### Task 2 —— close() 读出并上报 busy 列（`50c010e`）

`conn.query_row(..., |_| Ok(()))` 改为读第 0 列 `i64`（busy 标志），非 0 返回 `StoreError::CheckpointBusy`。「先 `drop(readers)` 再 checkpoint」的顺序与解释性注释保留；注释从「那正是这类 bug 静默的地方」改写成「busy 列现在被读出来并上报，这类 bug 不再静默」，并点名两条仍能复现 busy 的现实路径（另一线程未归还的 `PooledConnection`、关停时仍停在 `read()` 闭包里的读者）。

### Task 3 —— 版本串按位解析（`f1fc3e0`）

抽出私有纯函数 `parse_sqlite_version(&str) -> Option<(u32, u32, u32)>`：三个分量全部存在且全部 `parse::<u32>()` 成功才得元组，否则 `None`。`assert_sqlite_version` 据此判定，畸形串复用既有 `SqliteTooOld` 变体。函数上方注释点名仓库里另外三处同惯用法（`lib.rs` 的 `parts`、`tests/concurrency.rs` 的 `version_tuple`、`prism-engine/src/lib.rs`）**不是漏改**——只有这一处进入 `open()` 的准入判定路径。

`MIN_SQLITE` 仍为 `(3, 51, 3)`，未改。

## 三条非恒真反证（全部实跑）

### 反证 1 —— journal_mode

**改动前**（原 `execute_batch` 形态，`WAL` → `DELETE`）：

```
$ cargo test -p prism-store
running 21 tests → test result: ok. 21 passed
running 6 tests  → test result: FAILED. 5 passed; 1 failed
---- wal_truncated_on_close stdout ----
panicked at crates/prism-store/tests/concurrency.rs:164:5:
expected the -wal file to have grown before close
```

**与计划的预判不符**：计划称改动前「实测结果是全绿」。实测有一条会红（`wal_truncated_on_close`），但它红在一个**间接症状**上（`-wal` 没长大），错误文本完全不提 journal 模式——把它作为 WR-02 的哨兵，诊断路径仍然很长，且它只在写够 64 行之后才触发。见偏离 1。

**改动后**（`query_row` 形态，`WAL` → `DELETE`）：

```
$ cargo test -p prism-store --lib
test result: FAILED. 16 passed; 7 failed
---- open::tests::open_leaves_the_database_in_wal_mode stdout ----
panicked at crates/prism-store/src/open.rs:182:59:
open store: JournalModeNotWal("delete")
---- tests::open_creates_missing_parent_directories stdout ----
open store: JournalModeNotWal("delete")
```

失败落点是新的 journal 校验，错误里直接出现 `delete`，且每一条打开库的测试都在同一处失败。

### 反证 2 —— checkpoint busy（两种形态都跑了）

**形态一（真实复现 busy）**：另开一个裸 `rusqlite::Connection` 持一个未提交的读事务，再 `store.close()`：

```
$ cargo test -p prism-store --lib close_reports_busy
test open::tests::close_reports_busy_when_another_connection_holds_the_wal ... ok
test result: ok. 1 passed; finished in 5.20s
```

（5.2 秒是 checkpointer 走满 writer 上的 `BUSY_TIMEOUT_MS=5000` 才认输。）

**形态二（把读 busy 列的代码改回 `|_| Ok(())`）**：

```
$ cargo test -p prism-store --lib close_reports_busy
panicked at crates/prism-store/src/open.rs:234:33:
close should report busy: ()
test result: FAILED. 0 passed; 1 failed; finished in 5.42s
```

断言落点确实在被修的那一行。

### 反证 3 —— 版本串解析

**第一次尝试失败（重要）**：按计划写的「准入布尔」表驱动测试在 `filter_map` 形态下**全绿**：

```
$ cargo test -p prism-store --lib version_admission
test open::tests::version_admission_rejects_malformed_strings ... ok
```

原因见偏离 2：`3.x.51` 塌缩成 `(3, 51, 0)`，而 `(3,51,0) < MIN_SQLITE (3,51,3)`——准入结果与正确实现一致，测试完全不具判别力。

**改测试后重跑反证**（断言落到 parse 结果上，并补第七条用例 `3.x.53`）：

```
$ cargo test -p prism-store --lib version_admission     # filter_map 形态
panicked at crates/prism-store/src/open.rs:283:13:
assertion `left == right` failed: parse of "3.x.51"
  left: Some((3, 51, 0))
 right: None
```

单独暴露准入层的放行口（临时去掉 parse 断言，只留准入断言）：

```
panicked at crates/prism-store/src/open.rs:284:13:
assertion `left == right` failed: admission of "3.x.53"
  left: true
 right: false
```

`3.x.53` 在 `filter_map` 形态下被重排成 `(3, 53, 0)`，**高于**下界因而被放行——这才是 IN-03 真正的失败形态。还原后全绿。

## Verification

```
$ cargo test -p prism-store
test result: ok. 26 passed   (lib，原 21 + 新 5)
test result: ok. 6 passed    (tests/concurrency.rs，一条不减)
test result: ok. 4 passed    (tests/fts_cjk.rs)

$ cargo clippy -p prism-store --all-targets -- -D warnings
Finished `dev` profile — 0 warning

$ bash scripts/check-secrets.sh all
OK: pattern discriminates (19 positive / 10 negative samples)
OK: no plaintext secret in 114 version-controlled files

$ cargo test --workspace          # 全绿
$ cargo test -p prismdocs-shell --features test    # 13 + 2 passed
```

`migration_runs_before_the_read_pool_is_built` 的源码序断言仍绿（锚点 `to_latest(&mut writer)` 与 `Pool::builder()` 未被本 plan 移动）。`src-tauri/src/commands.rs:38-40` 核对：两个新变体落在 `EngineError::Store(_) => "store_error"` 兜底臂，IPC 短码契约未变。

## Deviations from Plan

### 1. [Rule 1 - 计划预判有误] 改动前的 journal_mode 反证并非「全绿」

- **Found during:** Task 1 反证（改动前实验）
- **Issue:** 计划断言「改动前同一实验的实测结果是全绿（这正是 WR-02 的要害）」。实测 `wal_truncated_on_close` 会红。
- **处置:** 不改代码，改叙述。WR-02 仍然成立，但要害要重述为「唯一的哨兵是一条间接症状测试」而不是「完全没有哨兵」：它报的是 `-wal` 没长大，不提 journal 模式，且依赖先写 64 行才触发；改动后失败信息直接是 `JournalModeNotWal("delete")`，落在 `open()` 本身。
- **Verification:** 两次输出都抄在上文反证 1。

### 2. [Rule 1 - 测试不具判别力] 计划给的六条版本用例在当前 MIN_SQLITE 下全部恒真

- **Found during:** Task 3 反证
- **Issue:** 计划的验收准则说「`3.x.51` 与 `3.51` 两条变红（前者被重排成 (3,51,0) 而**通过下界**）」。但 `(3,51,0) < (3,51,3)`——重排结果恰好也不够新，六条用例在 buggy 与正确实现下结果**完全相同**，反证跑出全绿。
- **Fix:** 表驱动测试改为断言 `parse_sqlite_version` 的 `Option` 返回值（畸形串必须是 `None`，而不是「碰巧不够新的元组」），并补第七条用例 `3.x.53`——它重排成 `(3, 53, 0)`，**高于**下界因而在 buggy 形态下被放行，是准入层唯一真正的漏口。准入布尔断言保留，作为「畸形串与太旧同一处置」的行为陈述。
- **Files modified:** `crates/prism-store/src/open.rs`
- **Verification:** 反证 3 的三段输出。
- **Committed in:** `f1fc3e0`

### 3. [Rule 2 - 补关键覆盖] busy 复现测试留下，未按计划删除

- **Found during:** Task 2
- **Issue:** 计划要求 busy 反证「临时测试跑完即删」。但删掉之后，`close()` 里读 busy 列的三行就只剩一条 `Display` 文本断言看着，没有任何行为面覆盖——下一次重构把它改回 `|_| Ok(())`（正是上轮 WR-03 的原样）不会有东西变红。这正是本 plan 要关闭的那类缺口。
- **Fix:** 保留 `close_reports_busy_when_another_connection_holds_the_wal`，去掉 TEMP 标记，doc 注释写明它耗约 5 秒（走满 `BUSY_TIMEOUT_MS`）以及为什么这个价钱值得付。
- **代价:** `cargo test -p prism-store --lib` 从 0.03s 变成 ~5.5s。
- **Committed in:** `50c010e`

### 4. [Rule 3 - clippy] 表驱动用例的显式类型标注触发 `type_complexity`

- **Found during:** Task 3
- **Issue:** `let cases: [(&str, Option<(u32, u32, u32)>); 7]` 触发 `-D clippy::type-complexity`。
- **Fix:** 去掉显式标注交给推断（数组里既有 `Some(..)` 也有 `None`，推断成立）。
- **Committed in:** `f1fc3e0`

---

**Total deviations:** 4（Rule 1 × 2、Rule 2 × 1、Rule 3 × 1）
**Impact on plan:** 无 scope 蔓延。两条 Rule 1 都是计划对失败形态的推断与实测不符，修正后反证反而更硬（反证 3 从恒真变成真正有判别力）。

## Issues Encountered

- 插入新测试时把 `migration_runs_before_the_read_pool_is_built` 的 doc 注释挤到了新测试头上（注释与函数脱钩）。当场发现并复位，注释内容一字未改。

## Edge Coverage Disposition（承接 INFRA-02 的 unclassified 探针）

计划的人工判定成立并已全部关闭：INFRA-02 未被既有测试覆盖的边界就是 open.rs 的三条静默降级（WAL 回落 / checkpoint busy / 版本串重排）。三条各配了一条实跑反证，无未分类残留。**补充一条判定修正**：其中「版本串重排」的真实边界不是计划所写的 `3.x.51`，而是任何**丢掉的分量位于高位、且左移后仍满足下界**的串（`3.x.53` 是最小示例）——已写进测试用例与注释。

## Next Phase Readiness

- 01-19 的范围（`lib.rs` 的 `parts`、`seed.rs`、`tests/concurrency.rs`）本 plan 一字未动，可直接叠加。
- `open.rs` 里 `parse_sqlite_version` 的注释已声明另外三处同惯用法的处置归属，01-19 处理 `lib.rs` 那一处时无需再判断是否漏改。

## Self-Check: PASSED

- `crates/prism-store/src/open.rs`、`crates/prism-store/src/error.rs`、本 SUMMARY 三个文件均存在
- 七个提交（`59e609f` `194bd96` `1693d6a` `50c010e` `e6460ca` `f1fc3e0` `3857f98`）均在 git log 中

---
*Phase: 01-foundation-skeleton*
*Completed: 2026-07-29*
