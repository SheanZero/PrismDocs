---
gsd_state_version: 1.0
milestone: v0.2
milestone_name: milestone
current_phase: 01
current_phase_name: foundation-skeleton
status: executing
stopped_at: Completed 01-04-PLAN.md
last_updated: "2026-07-28T23:28:41.754Z"
last_activity: 2026-07-28
last_activity_desc: Phase 01 execution started
progress:
  total_phases: 1
  completed_phases: 0
  total_plans: 9
  completed_plans: 4
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-07-28)

**Core value:** 「评论 → AI 修改 → 复核通过」的闭环：10 分钟看懂 AI 英文文档、批注两句让它接着干、评论在 AI 大规模重写下 0 静默丢失。北极星 = 每周闭环数。
**Current focus:** Phase 01 — foundation-skeleton

## Current Position

Phase: 01 (foundation-skeleton) — EXECUTING
Plan: 5 of 9
Status: Ready to execute
Last activity: 2026-07-28 — Phase 01 execution started

Progress: [████░░░░░░] 44%

## Performance Metrics

**Velocity:**

- Total plans completed: 0
- Average duration: -
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

**Recent Trend:**

- Last 5 plans: -
- Trend: -

*Updated after each plan completion*
**Per-Plan Metrics:**

| Plan | Duration | Tasks | Files |
|------|----------|-------|-------|
| Phase 01 P01 | 39min | 3 tasks | 53 files |
| Phase 01 P02 | 8min | 2 tasks | 8 files |
| Phase 01 P03 | 68min | 2 tasks | 8 files |
| Phase 01 P04 | 7min | 2 tasks | 11 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Init]: Roadmap 采用调研文档 Phase 划分 + 修正 A1/A2/A3；关键路径 1→2→3→5→6，Phase 4 ∥ 5
- [Init]: Phase 1 承载五项不可逆决策（单写者 SQLite + 读池、FTS5 CJK tokenizer、keyring-core、prism-mcp trait 反转、notify-then-fetch）
- [Init]: INFRA-04/05 为跨切预算——自 Phase 1 起执行，Phase 8 验证级验收
- [Init]: comrak 唯一锚定真相源；MigrationResult/ChangeSet 接口在 Phase 3 冻结（TD-01 §7）
- [Phase 1]: workspace members 用 crates/* 通配 + src-tauri：新增 crate 无需改动根 Cargo.toml
- [Phase 1]: prism-mcp 的 protocol_version() 用 static 承载 rmcp ProtocolVersion::LATEST（内含 Cow，const 提升不适用）
- [Phase 1]: 暂无自然用途的依赖（prism-llm serde、prism-anchor similar/ulid、prism-mcp axum/tokio）以依赖可用性单测引用，不提前发明后续 phase 的公开 API
- [Phase 1]: INFRA-01 不在 plan 01-01 勾选：该需求横跨本 phase 的 7 个 plan，事件总线与 Channel 有序流在 01-04/01-08/01-09
- [Phase ?]: 依赖方向断言用 herestring 而非管道喂 grep：pipefail + grep -q 早退会因 SIGPIPE 让四条断言全部静默恒绿
- [Phase ?]: 覆盖率 Phase 1 只测量不设阈值（engine 85.48% / 前端 10%），Phase 2 按排除已登记人工与 ignored 路径后 >=80% 开硬闸门
- [Phase ?]: schema v1 定案方案 A：external-content FTS5 + rowid_pk 显式 INTEGER PRIMARY KEY + 三同步触发器 + STRICT 表，索引粒度保持默认全粒度（不声明该选项，降粒度会废掉 4 字中文 MATCH）
- [Phase ?]: 只读池用 SQLITE_OPEN_READ_WRITE + query_only=ON 而非 READ_ONLY flags：只读连接在崩溃后 -shm 缺失时无法重建它
- [Phase ?]: 「颠倒迁移与建池顺序」的行为反证实测不成立（六个并发测试仍全绿），改用 open.rs 内的源码顺序断言作为常驻哨兵
- [Phase ?]: prism-types 的依赖上限就是 serde + thiserror 两项：它是 prism-mcp 与 prism-engine 的共同汇点，任何新增依赖会同时压到两侧（D-09）
- [Phase ?]: service trait 一律同步（非 async）：底层 rusqlite 本就阻塞，同步 trait 天然 object-safe，consumer 用 spawn_blocking 调用
- [Phase ?]: 跨边界转发第三方错误时只保留已核实安全的 Display 文本：keyring_core::Error 的 derive Debug 会打印原始密钥字节，而 unwrap()/tracing 的 ?err 走的正是 Debug
- [Phase ?]: 持有密钥的类型手写 Debug 输出占位串，并刻意不实现 Display——缺席让 format!("{key}") 成为编译错误而非运行期泄漏
- [Phase ?]: 钥匙串 service/account 命名（PrismDocs / llm_api_key / mcp_bearer_token）是跨二进制契约，固化于 docs/keychain-naming.md；prismdocs-helper 因 D-10 必须自带字面量副本

### Pending Todos

None yet.

### Blockers/Concerns

- [Phase 3 前]: TD-01 阈值 T_high/T_low 与权重待 Track B 真实 agent diff 语料标定（Phase 3 内 harness 完成）
- [Phase 3 关闭前]: A→B→A 降级锚点复活语义未定义，需修订为 TD-01 v0.2（阻塞 phase close，不阻塞 start）
- [Phase 5/6 计划前]: F3 OQ-2/OQ-3、F4 OQ-1（declined 语义）需拍板（阻塞 F4 状态机）
- [Phase 4 前]: Q1 速读区模型档位待 M0 评测定档
- ~~[Phase 1]: rmcp 2.2 feature-flag 确切名称需对照 README 核验（5 分钟检查）~~ — RESOLVED (01-01)：`rmcp = { version = "2.2", features = ["server", "transport-streamable-http-server"] }` 已在 prism-mcp 中实际编译通过
- [数据勘误]: REQUIREMENTS.md 原 Coverage 写 51 条，实际 v1 REQ-ID 为 61 条，已于 roadmap 创建时更正

## Deferred Items

Items acknowledged and carried forward from previous milestone close:

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| *(none)* | | | |

## Session Continuity

Last session: 2026-07-28T23:28:33.078Z
Stopped at: Completed 01-04-PLAN.md
Resume file: None
