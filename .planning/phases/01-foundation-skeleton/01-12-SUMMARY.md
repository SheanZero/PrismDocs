---
phase: 01-foundation-skeleton
plan: 12
subsystem: infra
tags: [mcp, rmcp, axum, bearer-auth, constant-time, fail-closed, defence-in-depth, tdd]

# Dependency graph
requires:
  - phase: 01-foundation-skeleton (plan 01-06)
    provides: McpDeps 注入容器、三层门禁 middleware、B 组隔离反证 harness
  - phase: 01-foundation-skeleton (plan 01-11)
    provides: check-secrets.sh 加宽后的 fixture 命名约定（fixture_bearer / configured）
provides:
  - "可失败构造 McpDeps::new(...) -> Result<Self, McpError>：空/纯空白 bearer 在构造期即被拒"
  - "McpError::EmptyBearer：零插值、rule-shaped 的错误变体"
  - "constant_time_eq 的空 expected 早退：纵深第二层，独立于构造期守卫"
  - "被反转（而非删除）的 fail-open 钉子断言"
  - "middleware_gate B 组两条新用例：端到端空呈递 token + 构造期拒绝"
affects: [phase-06-mcp-server, keychain-token-injection]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "可失败构造器承担配置校验：`McpDeps` 一旦存在，其 bearer 保证非空——比较层不必「每次都记得」"
    - "纵深两层各有落点唯一的反证：删任一层只红对应那一层的测试，证明不是同一个检查被数了两次"
    - "被钉住的错误行为用「反转断言」而非「删除断言」修复——被删掉的形态就是没人看着的形态"

key-files:
  created: []
  modified:
    - crates/prism-mcp/src/lib.rs
    - crates/prism-mcp/src/deps.rs
    - crates/prism-mcp/src/middleware.rs
    - crates/prism-mcp/tests/middleware_gate.rs
    - crates/prism-mcp/tests/trait_injection.rs
    - crates/prism-engine/tests/facade.rs

key-decisions:
  - "空判定用 `bearer.trim().is_empty()`：「配置了但配了个空白」与「根本没配」是同一件事"
  - "`McpError::EmptyBearer` 的 `#[error(...)]` 零插值——Phase 6 被拒的值可能是真 token 的畸形前缀，错误只陈述规则"
  - "比较层只加一条空 expected 早退，不做 WR-15 的整体重写：把两件事绑在一起会让反证落点不再唯一"
  - "四处 `McpDeps::new` 调用点全部在 Task 1 一次性适配，`.expect` 消息写成说明「测试 token 非空」的完整句子"
  - "留给 Phase 6：按 D-06「无 key 时应用照常启动」，构造 `Err` 须降级为「MCP 不启动 + 一条 warn」，不得 `unwrap()`（T-01-54）"

patterns-established:
  - "Fail-closed 配置校验：降级态（空/缺失/未配发）必须解析成「拒绝」且留下可诊断痕迹，绝不静默放行"
  - "反证的落点纪律：反证不只看红绿，还要看红在哪一条断言上；两层纵深的反证必须互不重叠"

requirements-completed: [INFRA-01]

coverage:
  - id: D1
    description: "空/纯空白 bearer 在 McpDeps 构造期被拒，错误 rule-shaped 且不回显被拒的值；非空 token 仍可构造"
    requirement: "INFRA-01"
    verification:
      - kind: unit
        ref: "crates/prism-mcp/src/deps.rs#an_empty_bearer_is_refused_at_construction"
        status: pass
      - kind: integration
        ref: "crates/prism-mcp/tests/middleware_gate.rs#an_empty_configured_bearer_cannot_be_constructed_in_the_first_place"
        status: pass
    human_judgment: false
  - id: D2
    description: "constant_time_eq 对空 expected 返回 false；被钉住的 fail-open 断言已反转而非删除；源码哨兵仍绿"
    requirement: "INFRA-01"
    verification:
      - kind: unit
        ref: "crates/prism-mcp/src/middleware.rs#constant_time_eq_agrees_with_equality_on_every_shape"
        status: pass
      - kind: unit
        ref: "crates/prism-mcp/src/middleware.rs#the_comparison_is_not_a_plain_equality"
        status: pass
    human_judgment: false
  - id: D3
    description: "CR-03 的现实攻击形态：合法配置的门禁收到 `Authorization: Bearer `（空 token）返回 403、空正文、未到 sentinel"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "crates/prism-mcp/tests/middleware_gate.rs#an_empty_presented_token_is_denied_by_the_bearer_layer_alone"
        status: pass
    human_judgment: false
  - id: D4
    description: "四处 McpDeps::new 调用点全部适配可失败构造，语义未变，workspace 编译与测试全绿"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "cargo test --workspace（27 个测试目标全部 ok，0 FAILED）"
        status: pass
      - kind: integration
        ref: "cargo clippy --workspace --all-targets -- -D warnings"
        status: pass
    human_judgment: false
  - id: D5
    description: "五条反证（L/M/N/O/P）各自使对应测试变红且落点唯一，证明两层纵深互相独立、均非恒真"
    requirement: "INFRA-01"
    verification:
      - kind: other
        ref: "执行期临时改源码重跑（见本文档 ## 反证记录 一节的实测落点）"
        status: pass
    human_judgment: true
    rationale: "反证是执行期的一次性破坏性实验，不留在代码里；其证据是当时的测试输出落点，需人工核对本文档记录的落点与描述一致"

# Metrics
duration: 6min
completed: 2026-07-29
status: complete
---

# Phase 01 Plan 12: MCP bearer 门禁 fail-closed Summary

**`McpDeps::new` 改为可失败构造 + `constant_time_eq` 空 expected 早退，构成两层互相独立的纵深；被单测钉成预期的 fail-open 行为被反转而非删除，01-REVIEW.md CR-03 关闭**

## Performance

- **Duration:** 6 min
- **Started:** 2026-07-29T05:37:15Z
- **Completed:** 2026-07-29T05:43:00Z
- **Tasks:** 2（均为 TDD，4 次 commit）
- **Files modified:** 6

## Accomplishments

- **构造期拒绝（纵深第一层）**：`McpDeps::new(feedback, comments, "")` 现在返回
  `Err(McpError::EmptyBearer)`。判定是 `bearer.trim().is_empty()`，因此空串与纯空白串都拒——
  「配置了但配了个空白」与「根本没配」是同一件事。`McpDeps` 一旦存在，其 bearer 保证非空。
- **比较期拒绝（纵深第二层）**：`constant_time_eq` 在函数体最前面对空 `expected` 直接返回 false。
  这条早退刻意放在进入常数时间路径**之前**——空 expected 是配置错误而非比较结果，不随呈递值变化，
  因此不构成侧信道；这里也没有可泄漏的秘密（配置本身就是空的），真正的泄漏是放行。
- **钉子断言被反转而非删除**：`middleware.rs` 的
  `constant_time_eq_agrees_with_equality_on_every_shape` 中，原先
  `assert!(constant_time_eq("", ""))` 那一行现在断言其为 **false**，行内注释改写成陈述它现在守什么。
  其余七条 case 一条未改、结果一条未变。
- **端到端形态被覆盖**：B 组新增
  `an_empty_presented_token_is_denied_by_the_bearer_layer_alone`——向一个**合法配置**
  （`GOOD_BEARER`）的门禁发 `Authorization: Bearer `，`strip_prefix("Bearer ")` 给出 `Some("")`，
  空呈递值确实一路走到比较层，被 403 拒且正文为空、未到 sentinel。
- **四处调用点一次性适配**：`deps.rs` 的 `deps()` 助手、`tests/trait_injection.rs` 的
  `deps_returning()`、`tests/middleware_gate.rs` 的 `deps()`、`prism-engine/tests/facade.rs` 的
  `engine_satisfies_service_traits`。`.expect` 消息统一写成说明「测试 token 是非空的 32 字节 hex 常量」
  的完整句子——它是将来读 backtrace 的人唯一能看到的线索。
- **D-09 依赖方向一条边未变**：注入形态仍是 `Arc<dyn FeedbackSource>` / `Arc<dyn CommentSink>`，
  `bash scripts/check-deps.sh all` 七条 OK 全绿（含 01-13 新加的 `tracing-subscriber-free`）。

## Task Commits

Each task was committed atomically（TDD 任务 test → feat 两次 commit）：

1. **Task 1: McpDeps::new 改为可失败构造** — `30d8622` (test, RED) → `ba4d87c` (feat, GREEN)
2. **Task 2: 比较层空值早退 + 反转钉子断言 + 端到端用例** — `8f0006c` (test, RED) → `fc1d3f3` (feat, GREEN)

**Plan metadata:** 见下方 docs commit

## Files Created/Modified

- `crates/prism-mcp/src/lib.rs` — 新增 `McpError::EmptyBearer` 变体。文案
  `"the injected bearer token must not be empty"` **零插值**，与 `McpDeps` 手写 `Debug` 的
  `<redacted>` 同一口径（T-01-29 / T-01-53）。`McpError` 本就是 `#[non_exhaustive]`，
  加变体不破坏下游穷尽匹配；其 16 行文档注释早已预告「token 缺失」是它的扩展点。
- `crates/prism-mcp/src/deps.rs` — `McpDeps::new` 返回类型 `Self` → `Result<Self, McpError>`；
  参数仍是 `impl Into<Arc<str>>`（先 `.into()` 再检查）。函数上方注释写明为什么检查放在构造期
  而不是比较层，并指明 Phase 6 的三条现实命中路径。新增单测
  `an_empty_bearer_is_refused_at_construction`（含 ④ 阴性对照）。`expose_bearer` 的签名/可见性、
  手写 `Debug`、`Arc<dyn …>` 注入形态一律未动。
- `crates/prism-mcp/src/middleware.rs` — `constant_time_eq` 最前面加空 expected 早退；文档注释
  补两段说明「这条早退与『长度不等时也不提前返回』为何不矛盾」以及「这是纵深第二层」。
  折叠缓冲、`same_len`、`ct_eq`、源码哨兵 `the_comparison_is_not_a_plain_equality` 一律未动。
- `crates/prism-mcp/tests/middleware_gate.rs` — `deps()` 助手加 `.expect(...)`；B 组新增两条测试
  （10 → 12）。新用例逐字照抄 `host_layer_alone_is_what_rejects_a_foreign_host` 的三段式形状，
  复用既有 `sentinel_router()` / `request()` / `oneshot()` 助手，未新建第二套 harness。
- `crates/prism-mcp/tests/trait_injection.rs` — `deps_returning()` 加 `.expect(...)`。
- `crates/prism-engine/tests/facade.rs` — `engine_satisfies_service_traits` 的注入点拆成
  `let deps = McpDeps::new(...).expect(...)` 再传给 `serve_loopback`，附注说明 Phase 6 的真实注入路径
  建在这个形状上。

## Decisions Made

- **`trim()` 而非 `is_empty()`**：纯空白配置与未配置在运行期是同一种失效，都必须拒。
- **错误文案零插值**：Phase 1 命中这个变体的值恰好是空白串，但 Phase 6 由钥匙串注入时，
  被拒的值可能是一个真 token 的畸形前缀。错误只陈述规则，不回显值。
- **不做 WR-15 的整体重写**：WR-15 建议把 `constant_time_eq` 简化成
  `!expected.is_empty() && expected.as_bytes().ct_eq(...)`。本轮只加一条早退，因为
  ① 它是反转钉子断言所需的最小改动；② WR-15 把「去掉手写折叠」与「加空值检查」绑在一起，
  会让「哪一层挡住了」这个反证落点不再唯一。WR-15 与 WR-14 均留在本轮圈定的五项之外。
- **构造期拒绝的测试放在 `deps.rs` 的 `mod tests`，端到端形态放在 `middleware_gate.rs` 的 B 组**：
  两层各有自己的测试文件位置，删任一层的守卫只会红对应那一处。
- **`an_empty_configured_bearer_cannot_be_constructed_in_the_first_place` 绕开 `deps()` 助手**：
  助手带 `.expect(...)`，经它调用会让断言落在 panic 上而不是返回值上。

## 反证记录

五条反证均为执行期一次性破坏性实验，实测落点如下（每条实验后均已还原并确认全绿）：

| 反证 | 临时改动 | 实测结果 | 落点 |
|------|---------|---------|------|
| **L** | 删掉 `McpDeps::new` 的空判定 | `an_empty_bearer_is_refused_at_construction` FAILED | `deps.rs:135` — `"an empty bearer must not construct a gate"`（断言 ①） |
| **M** | 把空判定改成 `if true`（一律 Err） | 同一测试 FAILED；且 `middleware_gate` 7/10 failed、`trait_injection` 3/3 failed、`facade` 1/6 failed，全部 panic 在 `.expect` 上 | `deps.rs:169` — `"a non-empty bearer must still construct: EmptyBearer"`（断言 ④）。三个集成目标一起红，正是「四个调用点都真的走了新构造契约」的旁证 |
| **N** | 删掉 `constant_time_eq` 的空值早退 | `constant_time_eq_agrees_with_equality_on_every_shape` FAILED（10 passed / 1 failed）；`an_empty_bearer_is_refused_at_construction` **仍绿** | `middleware.rs:225` — `assertion failed: !constant_time_eq("", "")`（正是被反转的那一条） |
| **O** | 删掉构造期空判定、保留比较层早退 | `an_empty_configured_bearer_cannot_be_constructed_in_the_first_place` FAILED；`constant_time_eq_agrees_with_equality_on_every_shape` **仍绿** | `middleware_gate.rs:407` — `"空配置构造出了一个门禁"` |
| **P** | 把 `require_bearer` 的比较调用改为无条件放行 | `an_empty_presented_token_is_denied_by_the_bearer_layer_alone` FAILED | `middleware_gate.rs:369` — `left: 200 / right: 403`，`"空呈递 token 未被 require_bearer 拒绝"`（守卫版 403 那一条，**不是**摘层后直达 sentinel 那一条） |

N 与 O 互为镜像，且各自的「另一层仍绿」证明两层是独立的检查，而不是同一个检查被数了两次。

## Deviations from Plan

None — plan executed exactly as written.

（说明：Task 2 的两条新用例中，`an_empty_presented_token_is_denied_by_the_bearer_layer_alone`
在写入时即为绿——一个**合法配置**的门禁面对空呈递 token 本来就会因长度不等而拒。它是回归/覆盖
用例而非 RED 用例，其价值由反证 P 证明：把比较调用改为无条件放行时它会红。Task 2 真正的 RED
信号来自被反转的那条钉子断言，实测在 `middleware.rs:218` 变红。这与 plan 的意图一致，未构成偏离。）

## Issues Encountered

- **Task 1 的 RED 是编译失败而非断言失败**：测试要断言 `Err(McpError::EmptyBearer)`，而当时
  `new` 返回 `Self` 且该变体尚不存在，因此 RED 表现为 5 个 `E0599`/`E0433`。这是 Rust 中签名变更类
  TDD 的正常 RED 形态，已按此提交 `test(...)` commit，随后的 `feat(...)` commit 转绿。
- 无其他问题。全程未触发任何 deviation rule。

## Known Stubs

None — 本 plan 未引入任何 stub、TODO、skipped test 或未跑的 verify。

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- **CR-03 关闭**。成功标准 1 覆盖的 prism-mcp 门禁不再有 fail-open 的降级态：空/空白 bearer
  在构造期造不出门禁、在比较期也不放行，且这两件事各自独立可证。
- **留给 Phase 6 的一条硬要求（T-01-54，plan 已标 `accept`）**：`McpDeps::new` 现在会返回 `Err`。
  Phase 6 从钥匙串（`docs/keychain-naming.md` 的 `mcp_bearer_token`）读出 token 后注入时，
  **不得 `unwrap()`**——按 D-06「无 key 时应用照常启动」，`Err(McpError::EmptyBearer)` 必须降级为
  「MCP 服务不启动 + 一条 warn」，否则「token 没配」会从一个开着的门变成一个启动崩溃。
- **未做且已知的两项**（均在本轮圈定的五项之外，非本 plan 遗漏）：WR-15（`constant_time_eq`
  整体简化）与 WR-14（`accepts_fully_valid_request` 用 `!is_client_error()` 作阳性对照）。
- 全 workspace 绿：`cargo test --workspace` 27 个测试目标全部 ok / 0 FAILED；
  `cargo clippy --workspace --all-targets -- -D warnings` 退出 0；
  `bash scripts/check-deps.sh all` 七条 OK；`bash scripts/check-secrets.sh all` 退出 0。

## Self-Check: PASSED

- 7/7 声称的文件存在于磁盘
- 4/4 声称的 commit 存在于 git 历史（`30d8622` / `ba4d87c` / `8f0006c` / `fc1d3f3`）

---
*Phase: 01-foundation-skeleton*
*Completed: 2026-07-29*
