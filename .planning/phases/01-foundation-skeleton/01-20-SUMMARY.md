---
phase: 01-foundation-skeleton
plan: 20
subsystem: service-traits
tags: [security, logging, mcp, input-validation, dead-code, gap-closure]
status: complete

requires:
  - "crates/prism-engine/src/services.rs（01-09 建立的两条纪律：同步实现无 .await；错误文本不回显调用参数）"
  - "01-13 装上的 tracing subscriber（WR-04 关闭之后那行日志真的会写）"
  - "01-REVIEW-prior.md WR-13 / IN-05"
provides:
  - "`RECEIPT_STATUSES` 三值受控取值集合（applied / rejected / deferred），精确匹配不做大小写折叠"
  - "`record_receipt` 在 `tracing::info!` **之前**校验 status；被拒的值不进日志、不进错误文本"
  - "逐字固定的 status 拒绝文本 `invalid request: status is not a recognised value`"
  - "一条源码顺序断言：校验语句必须早于日志语句（行为测试对顺序无判别力）"
  - "`ServiceError` 不再有零构造点的 `Backend` 变体；重新引入的条件就地记录"
affects:
  - "Phase 5（COMMENT-03）：真实评论状态机定案时，受控取值集合上移到 prism-types、Receipt.status 改 enum 的衔接点已写在常量注释里"
  - "Phase 5/6：第一个真实会失败的存储/解析/FS 调用方出现时，就地加回后端失败变体（文本约束已写明）"
  - "MCP 线协议：`Receipt.status` 的 serde 形态**未变**，仍是 String，本 plan 未与 01-16/01-17 抢文件"

tech-stack:
  added: []
  patterns:
    - "外部字段进日志前必须先过取值域校验；「校验在日志之前」这件事本身需要一条源码顺序断言来看住——行为测试对顺序完全没有判别力（本 plan 实测反证 B：19 passed / 1 failed）"
    - "源码断言的锚点取**完整语句**而非裸标识符：`RECEIPT_STATUSES` 单独出现在常量声明处，用它做锚点会锚到声明而不是校验点"
    - "受控取值测试写死字面量而不是遍历常量：遍历版本在常量被清空时依然全绿（零次迭代），丧失判别力"
    - "拒绝文本的双重断言：逐字相等 + 显式 `!msg.contains(probe)`——只有前者时，一个把值拼进固定前缀的实现仍可能漏过"
    - "零构造点的枚举变体不是「预留」而是死代码：要么给它构造点，要么移除并把重新引入的条件写在原位置"

key-files:
  created: []
  modified:
    - crates/prism-engine/src/services.rs
    - crates/prism-types/src/service.rs

decisions:
  - "受控取值集合放在 prism-engine 侧的模块级常量，**不**上移到 prism-types 做成 enum：改 `Receipt.status` 的类型会连带改 dto.rs 的 serde 形态与 prism-mcp 三个测试文件的构造点，与本波的 01-16/01-17 抢文件；且真实状态机要到 Phase 5（COMMENT-03）才定案。上移条件写在常量注释里"
  - "精确匹配，不做大小写折叠：`Applied` 被拒。线协议值应当是确定的小写 token，大小写宽容只会把「哪些值合法」重新变成不确定的"
  - "`ServiceError::Backend` 选择**移除**而非保留：Phase 5 位于关键路径 1→2→3→5→6 的下游（Phase 2/3 在前），不属于上轮 IN-05 判据里的「很快落地」。重新引入的检查点是 Phase 5/6 第一个真实会失败的调用方，届时文本仍须由实现方写死——rusqlite/io 的原始错误串可能带路径与 SQL 片段，不能直接 to_string() 塞进去"
  - "`#[non_exhaustive]` 保留，因此移除变体对 crate 外的 match 无影响（外部本就必须带 `_` 臂）；工作区内的 match 点（prism-types/tests/contract.rs、prism-engine/tests/facade.rs）都不匹配 Backend"

metrics:
  duration: ~20min
  completed: 2026-07-29
  tasks: 2
  files: 2
---

# Phase 01 Plan 20: 回执 status 取值域约束与死变体清理 Summary

给外部 agent 提供的 `Receipt.status` 加一个三值受控取值集合，校验点排在 `tracing::info!` 之前并由一条源码顺序断言看住；同时移除零构造点的 `ServiceError::Backend`，把重新引入的条件写在它原来的位置上。

## 关闭的上轮问题

| 上轮 ID | 问题 | 处置 |
|---|---|---|
| WR-13 | `record_receipt` 的文档注释精确写下「日志里只有 comment_id 与 status，没有正文」的规则，然后把线上直接反序列化的 `status` 不加约束地记进 `tracing::info!`。01-13 关闭 WR-04 之后 subscriber 装上了，这条从 latent 变成 live | Task 1：三值受控集合 + 校验前置 + 顺序断言 |
| IN-05 | `ServiceError::Backend` 零构造点（`grep -rn` 无命中），既没人造也没人接 | Task 2：移除，重新引入条件就地记录 |

## Task 1: record_receipt 约束 status 的取值域

新增模块级常量：

```rust
const RECEIPT_STATUSES: [&str; 3] = ["applied", "rejected", "deferred"];
```

校验插在 `comment_id` 校验之后、`tracing::info!` **之前**，拒绝文本只陈述规则（`status is not a recognised value`），不回显传入的值。

`<behavior>` 六条逐条落实：

| 输入 | 结果 | 覆盖测试 |
|---|---|---|
| `status: "applied"` | `Ok(())` | `record_receipt_accepts_a_well_formed_receipt`（既有，未改动） |
| `status: "rejected"` / `"deferred"` | `Ok(())` | `every_controlled_receipt_status_is_accepted_and_an_empty_one_is_not` |
| `status: ""` | `Err(Invalid)` | 同上 |
| `status: "Applied"` | `Err(Invalid)` | `status_rejection_text_does_not_echo_the_caller_argument` |
| 含换行的超长 status | `Err(Invalid)`，且不入日志/错误文本 | `record_receipt_rejects_a_long_multiline_status` |
| `comment_id: ""` | `Err(Invalid)`（行为不变） | `record_receipt_rejects_a_receipt_without_a_comment_id`（既有，未改动） |

顺序断言加在既有的 `the_service_impls_contain_no_await` 里，锚点取完整语句：

```rust
let guard = production
    .find("!RECEIPT_STATUSES.contains(&receipt.status.as_str())")
    .expect("status 受控取值校验不见了——外部字段会直接进日志");
let log = production
    .find("\"recorded an agent receipt\"")
    .expect("回执日志行不见了");
assert!(guard < log, "status 校验被排到了 tracing::info! 之后：被拒的值仍会先写进日志");
```

锚点刻意不取裸 `RECEIPT_STATUSES`——那个标识符单独出现在常量声明处（文件更靠前），会锚到声明而不是校验点，顺序比较随即恒真。这是本 phase 已有的实测教训。

### 非恒真反证 A（删掉 status 校验）—— 实跑输出

```
guard removed
---- services::tests::record_receipt_rejects_a_long_multiline_status stdout ----
thread 'services::tests::record_receipt_rejects_a_long_multiline_status' panicked at crates/prism-engine/src/services.rs:234:14:
含换行的超长 status 应被拒: ()

failures:
    services::tests::every_controlled_receipt_status_is_accepted_and_an_empty_one_is_not
    services::tests::record_receipt_rejects_a_long_multiline_status
    services::tests::status_rejection_text_does_not_echo_the_caller_argument
    services::tests::the_service_impls_contain_no_await

test result: FAILED. 16 passed; 4 failed
```

还原后：`test result: ok. 20 passed; 0 failed`。

### 非恒真反证 B（把校验移到日志之后）—— 实跑输出

```
guard moved AFTER tracing::info!
test services::tests::every_controlled_receipt_status_is_accepted_and_an_empty_one_is_not ... ok
test services::tests::record_receipt_rejects_a_long_multiline_status ... ok
test services::tests::status_rejection_text_does_not_echo_the_caller_argument ... ok

---- services::tests::the_service_impls_contain_no_await stdout ----
thread 'services::tests::the_service_impls_contain_no_await' panicked at crates/prism-engine/src/services.rs:290:9:
status 校验被排到了 tracing::info! 之后：被拒的值仍会先写进日志

failures:
    services::tests::the_service_impls_contain_no_await

test result: FAILED. 19 passed; 1 failed
```

**这正是那条源码断言存在的理由**：三条行为测试全部仍绿——一个把校验写在日志之后的实现在功能上「也拒了」——而被拒的值已经进了日志。行为面对顺序完全没有判别力，顺序只能由源码断言看住。还原后：`20 passed; 0 failed`。

## Task 2: 处置 ServiceError::Backend

实测确认现状与预期一致：`grep -rn "ServiceError::Backend" crates src-tauri` 零命中（改动前后皆然）。工作区内 `match ServiceError` 的两处（`prism-types/tests/contract.rs`、`prism-engine/tests/facade.rs` 的间接使用）都不匹配 `Backend`。

按上轮 IN-05 的判据处置：Phase 5 位于关键路径 1→2→3→5→6 的下游（Phase 2 与 Phase 3 在前），不属于「很快落地」，因此**移除**。文档注释里留下了移除理由与重新引入的条件：

- 触发点：Phase 5（评论落库）或 Phase 6（评论回流）第一个真实会失败的调用方
- 届时的文本约束：仍须由实现方写死，不含调用参数——rusqlite / io 的原始错误串可能带路径与 SQL 片段，不能直接 `to_string()` 塞进去
- `#[non_exhaustive]` 保留，加回变体对 crate 外的 `match` 无影响

`NotFound` / `Invalid` 两个变体的 `#[error(...)]` 文本未动。威胁模型段落（T-01-04 / T-01-20）保留，仅把枚举里那句「`Backend` / `Invalid` 携带的必须是……」改为「`Invalid` 携带的必须是……」——变体没了，句子里的引用也不能留。

## Deviations from Plan

None - plan executed exactly as written.

两处值得记录的判断（都在 plan 授权范围内，非偏离）：

1. Task 2 的 plan 写明「若实测发现存在构造点，改为保留变体」。实测零命中，走了移除分支。
2. Task 2 的验收要求「其余两个变体的文本与文档注释里的威胁模型说明不变」。枚举文档注释里原有一句同时点名 `Backend` 与 `Invalid`，移除变体后必须把 `Backend` 从这句里去掉，否则注释指向一个不存在的变体。威胁模型的实质（错误文本不回显调用参数、T-01-04 / T-01-20 编号）一字未动。

## Verification

| 命令 | 结果 |
|---|---|
| `cargo test -p prism-engine` | 20 passed（lib）+ 6 passed（facade 集成）+ 0 doc，全绿 |
| `cargo test --workspace` | 全绿（无 failed，1 ignored 为既有） |
| `cargo clippy -p prism-engine --all-targets -- -D warnings` | 0 warning |
| `cargo clippy -p prism-types --all-targets -- -D warnings` | 0 warning |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 warning |
| `bash scripts/check-deps.sh no-cycle` | `OK: prism-mcp -> prism-types only` |
| `grep -rn "ServiceError::Backend" crates src-tauri` | 零命中 |
| `grep -c "Phase 5" crates/prism-types/src/service.rs` | 1（重新引入条件的注释段仍在） |

`cargo fmt` 按执行上下文的约定**未运行**（仓库存在 rustfmt drift 且无 CI fmt gate，已登记在 `deferred-items.md`，由 plan 01-28 关闭）；新增代码按 rustfmt 风格手写。

## Success Criteria

- [x] 外部 agent 提供的 `status` 在受控取值集合之外时被拒，且被拒的值不进日志、不进错误文本
- [x] 校验与日志的先后顺序由一条落点唯一的源码断言看住（反证 B 证明其判别力）
- [x] `ServiceError` 里不再有死变体，重新引入的条件已就地记录

## Known Stubs

None.

## Threat Flags

无新增安全面。本 plan 只收窄既有面：`T-01G-18`（未约束 status 进 tracing）与 `T-01G-19`（嵌入换行伪造日志行）由三值受控集合关闭，`T-01G-20`（拒绝文本回显被拒值）由逐字断言 + `!contains(probe)` 双重断言关闭。`T-01G-21`（移除 Backend 后失去表达力）按 plan 的 disposition 为 `accept`——零构造点即无表达力可失。无新增依赖（T-01-SC 不触发）。

## Commits

| Gate | Hash | Message |
|---|---|---|
| RED | `5a9fe4d` | test(01-20): add failing tests for receipt status value-domain guard |
| GREEN | `3787c40` | feat(01-20): constrain receipt status to a controlled value set before logging |
| Task 2 | `dc108e2` | refactor(01-20): remove the unconstructed ServiceError::Backend variant |

## Self-Check: PASSED

三个 commit hash 均在 `git log --all` 中命中；`01-20-SUMMARY.md` 存在；`RECEIPT_STATUSES` 在 `services.rs` 中出现 6 次（常量声明 + 文档引用 + 校验点 + 测试注释与断言锚点）。
</content>
</invoke>
