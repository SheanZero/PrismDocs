---
phase: 01-foundation-skeleton
plan: 06
subsystem: infra
tags: [prism-mcp, rmcp, axum, trait-inversion, dns-rebinding, bearer, constant-time, defense-in-depth, counter-proof-isolation]

# Dependency graph
requires:
  - "01-01：prism-mcp 的最小编译单元、rmcp/axum 版本 pin 与已实证的 feature 名"
  - "01-02：scripts/check-deps.sh 的 no-cycle / tauri-free / single-egress 断言"
  - "01-04：prism-types 的 FeedbackSource / CommentSink / FeedbackItem / Receipt / ServiceError"
provides:
  - "McpDeps 注入容器（Arc<dyn FeedbackSource> + Arc<dyn CommentSink> + 私有 bearer，脱敏 Debug、无 Display）"
  - "PrismHandler —— rmcp ServerHandler 的最小实现，list_feedback 工具经 spawn_blocking 调同步 trait"
  - "build_router / serve_loopback —— StreamableHttpService 挂在 127.0.0.1:0 的 axum 0.8 上"
  - "require_local_host / require_origin_allowlist / require_bearer 三层门禁与 ALLOWED_HOSTS / ALLOWED_ORIGINS"
  - "constant_time_eq —— subtle ct_eq + 异或折叠，长度不等时不提前返回"
  - "sentinel-router 隔离反证形态：被测中间件之上有第三方兜底时，反证要放进没有兜底的最小链路"
affects: [01-07-集成验证, 01-08-冒烟命令与事件总线, phase-6-MCP工具注册与CLI-helper, phase-7-get_context_pack]

# Tech tracking
tech-stack:
  added:
    - "tokio-util 0.7（CancellationToken —— graceful shutdown 与 rmcp 会话终止）"
    - "tower 0.5（prism-mcp dev-dependency，ServiceExt::oneshot 驱动隔离测试）"
    - "subtle 2（常数时间比较，workspace 已 pin，本 plan 首个消费者）"
    - "reqwest 0.13（prism-mcp **dev-only**：本 crate 托管服务端，客户端能力仅测试需要）"
  patterns:
    - "注入用 Arc<dyn Trait> 而非泛型：engine 具体类型不经 S 泄漏进 axum service 的类型签名"
    - "同步 service trait 在 async handler 中的唯一正确形态：先 clone Arc，再 spawn_blocking，最后处理 JoinError"
    - "持有密钥的容器：私有字段 + pub(crate) expose_* + 手写脱敏 Debug + 刻意无 Display（沿用 01-04 的 ApiKey 口径）"
    - "拒绝响应无差别化：三层统一 403 + 空正文，真实原因只进 tracing"
    - "反证的隔离性：当被测层之上还有第三方兜底（SDK 自带校验）时，摘层反证会被兜底掩盖；必须另建无兜底的最小链路"
    - "第三方 crate 的默认安全配置要显式复述一遍（with_allowed_hosts/with_allowed_origins），默认值可被一行改掉而 review 看不见"

key-files:
  created:
    - crates/prism-mcp/src/deps.rs
    - crates/prism-mcp/src/handler.rs
    - crates/prism-mcp/src/middleware.rs
    - crates/prism-mcp/src/server.rs
    - crates/prism-mcp/tests/trait_injection.rs
    - crates/prism-mcp/tests/middleware_gate.rs
  modified:
    - crates/prism-mcp/src/lib.rs
    - crates/prism-mcp/Cargo.toml
    - Cargo.toml
    - Cargo.lock

key-decisions:
  - "McpDeps.bearer 为私有字段 + pub(crate) expose_bearer，而非计划写的 pub 字段——token 的取用点在代码搜索中唯一可见"
  - "三层一律 403 + 空正文，不给 bearer 缺失单开 401：状态码差异本身就是逐层试探的信息源（T-01-29）"
  - "rmcp SDK 侧 allowed_hosts/allowed_origins 与应用层配成同一份做防御纵深；代价是端到端摘层反证失效，改由 sentinel-router 隔离测试承担"
  - "常数时间比较用异或折叠而非截断：长度不等的 presented 全部字节参与折叠，不制造「前缀即通过」的形状"
  - "Phase 1 保留 rmcp 的 stateful 会话模式（SDK 默认），不为测试便利改成 stateless+json_response"

patterns-established:
  - "Phase 6 加 MCP 工具：在 prism-types 追 trait、在 PrismHandler 的 call_tool 分派，**不动 middleware.rs 与 build_router 的三层**"
  - "任何声称「摘掉 X 就变红」的反证，先确认 X 之上没有第三方兜底"

requirements-completed: []

coverage:
  - id: D1
    description: "prism-mcp 只经注入的 service trait 取数据，编译期不可能依赖 prism-engine"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "cargo tree -p prism-mcp --edges normal --prefix none | tail -n +2 | grep -c '^prism-engine ' → 0"
        status: pass
      - kind: integration
        ref: "cargo tree -p prism-mcp --edges dev --prefix none | tail -n +2 | grep -c '^prism-engine ' → 0（dev 逃逸口也未被使用）"
        status: pass
      - kind: integration
        ref: "bash scripts/check-deps.sh no-cycle → OK: prism-mcp -> prism-types only"
        status: pass
    human_judgment: false
  - id: D2
    description: "注入通路真的通：响应体携带假实现独有的数据，且换成空实现后该数据消失"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "crates/prism-mcp/tests/trait_injection.rs#injected_feedback_source_is_reached（断言 fb-1 出现）"
        status: pass
      - kind: integration
        ref: "crates/prism-mcp/tests/trait_injection.rs#empty_source_yields_no_item（阴性对照：fb-1 不出现）"
        status: pass
      - kind: integration
        ref: "grep -c 'spawn_blocking' crates/prism-mcp/src/handler.rs → 2（同步 trait 不在 async 上下文里直接阻塞）"
        status: pass
    human_judgment: false
  - id: D3
    description: "MCP server 只绑 127.0.0.1，端口由 OS 分配"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "trait_injection.rs#serve_loopback_binds_to_loopback_on_an_os_assigned_port（IP == 127.0.0.1 且 port != 0）"
        status: pass
      - kind: integration
        ref: "grep -c '0.0.0.0' crates/prism-mcp/src/server.rs → 0"
        status: pass
    human_judgment: false
  - id: D4
    description: "Host 层：外域 Host 被拒，且拒绝确由 require_local_host 产生（落点唯一）"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "middleware_gate.rs#rejects_foreign_host（端到端 403）"
        status: pass
      - kind: integration
        ref: "middleware_gate.rs#host_layer_alone_is_what_rejects_a_foreign_host（sentinel 链路：挂层 403 / 摘层 200+sentinel）"
        status: pass
      - kind: integration
        ref: "反证实跑：把 require_local_host 的 allowlist 判断改为直通 → 落点为 middleware_gate.rs:254 `require_local_host 未拦下外域 Host`（left 200 / right 403）"
        status: pass
    human_judgment: false
  - id: D5
    description: "Origin 层：外域 Origin 被拒；无 Origin 的非浏览器客户端放行；tauri://localhost 在 allowlist 内"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "middleware_gate.rs#rejects_foreign_origin（端到端 403）"
        status: pass
      - kind: integration
        ref: "middleware_gate.rs#origin_layer_alone_is_what_rejects_a_foreign_origin（含无 Origin 放行与 tauri:// 放行两条对照）"
        status: pass
      - kind: integration
        ref: "反证实跑：把 allowlist 判断改为直通 → 落点为 middleware_gate.rs:281 `require_origin_allowlist 未拦下外域 Origin`（left 200 / right 403）"
        status: pass
    human_judgment: false
  - id: D6
    description: "bearer 层：缺失 / 等长错误 / 变长错误 / 前缀加后缀 / 非 Bearer scheme 五种情况全被拒"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "middleware_gate.rs#rejects_missing_or_wrong_bearer（端到端四种）"
        status: pass
      - kind: integration
        ref: "middleware_gate.rs#bearer_layer_alone_is_what_rejects_a_bad_token（sentinel 链路 × 5 种，各带摘层对照）"
        status: pass
      - kind: integration
        ref: "反证实跑：从 build_router 摘掉 require_bearer → 落点为 middleware_gate.rs:191 `无 Authorization 头却未被拒`"
        status: pass
    human_judgment: false
  - id: D7
    description: "三层不是把所有请求都拒了：全合法请求可到达 mcp service"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "middleware_gate.rs#accepts_fully_valid_request（非 4xx）"
        status: pass
      - kind: integration
        ref: "trait_injection.rs 的三个测试全部走完整 MCP 握手 + tools/call —— 三层放行的最强证据"
        status: pass
    human_judgment: false
  - id: D8
    description: "bearer 比较为常数时间，不使用 =="
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "grep -c 'ct_eq' crates/prism-mcp/src/middleware.rs → 4；ConstantTimeEq → 1"
        status: pass
      - kind: unit
        ref: "middleware.rs#tests::the_comparison_is_not_a_plain_equality（源码级守卫：函数体内必须含 ct_eq、不得含短路比较）"
        status: pass
      - kind: unit
        ref: "middleware.rs#tests::constant_time_eq_agrees_with_equality_on_every_shape（8 种形状，含前缀、加后缀、整数倍长度三种折叠碰撞候选）"
        status: pass
    human_judgment: false
  - id: D9
    description: "bearer token 不进日志、错误文本、响应体"
    requirement: "INFRA-03"
    verification:
      - kind: unit
        ref: "deps.rs#tests::debug_does_not_reveal_the_bearer_token（Debug 输出 <redacted>；McpDeps 刻意无 Display）"
        status: pass
      - kind: integration
        ref: "middleware_gate.rs#the_bearer_token_never_appears_in_a_response（正确/错误/缺失三种情况响应体均不含 token）"
        status: pass
      - kind: integration
        ref: "bash scripts/check-secrets.sh → exit 0"
        status: pass
    human_judgment: false
  - id: D10
    description: "拒绝响应不透露是哪一层挂了（T-01-29）"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "middleware_gate.rs#rejections_do_not_disclose_which_layer_denied（三层的 (status, body) 两两 assert_eq!，且 body 为空）"
        status: pass
    human_judgment: false
  - id: D11
    description: "A5 假设关闭：axum 0.8 的 .layer() 后加的先执行"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "middleware_gate.rs#layers_run_outermost_first_meaning_last_added_runs_first（先加的那层置位一个 AtomicBool，后加的 require_local_host 拒掉请求后该 bool 仍为 false）"
        status: pass
    human_judgment: false

# Metrics
duration: 26min
completed: 2026-07-29
status: complete
---

# Phase 1 Plan 06: prism-mcp trait 反转与 D-07 三层鉴权 Summary

**D-09 的注入通路与 D-07 的三层门禁同时落地，两者各自的关键性质都由「落点唯一」的反证守住——包括发现并修正了计划自带的两条反证实际不成立这件事。**

## Performance

- **Duration:** ≈26 min agent 时间
- **Tasks:** 2（各按 TDD 走 RED→GREEN 两段）
- **Files modified:** 10（新建 6，修改 4）

## Accomplishments

- **prism-mcp 从「能编译的骨架」变成「能起服务、能经注入 trait 返回数据」的宿主**：完整走通
  `initialize` → `notifications/initialized` → `tools/call` 三步 MCP 握手，响应体里出现的是假
  `FeedbackSource` 独有的 `fb-1`，而不是 handler 内的常量——阴性对照（空实现）证明了这一点。
- **D-09 的编译期性质在新增 8 个依赖之后依然成立**：`cargo tree` 的普通边与 dev 边都没有
  `prism-engine`；`reqwest` 只在 dev 边，`check-deps.sh` 的 single-egress 未被破坏
  （prism-engine 的普通依赖树看不到 prism-mcp 的 dev 依赖）。
- **D-07 的三层形态定死并各有落点唯一的证据**：Host / Origin / bearer 三层各自独立成立，
  全合法请求可到达 mcp service，拒绝一律 403 + 空正文。
- **发现并修正了计划反证的一个结构性缺陷**（详见「Deviations」与「Issues」）：计划要求的
  「从 `build_router` 摘掉 `require_local_host` → `rejects_foreign_host` 变红」实跑**全绿**——
  rmcp 2.2 的 `StreamableHttpService` 自己也校验 Host。同样的问题也存在于 Origin 层。
  已用 sentinel-router 隔离测试替代，并把这一类失效模式写进 STATE.md。
- **RESEARCH Assumptions Log A5 经实测关闭**：axum 0.8 的 `.layer()` **后加的先执行**，
  由一个「先加的层置位 AtomicBool、后加的层拒掉请求后该 bool 仍为 false」的测试锁死，
  不是靠读文档推断。
- **STATE.md 的 rmcp feature-flag 待办再获一次实证**：`server` + `transport-streamable-http-server`
  在 `cargo tree -p prism-mcp -e features` 中在列，且 `StreamableHttpService` / `LocalSessionManager` /
  `StreamableHttpServerConfig` 三个类型全部实际编译并运行通过。

## Task Commits

1. **Task 1: McpDeps 注入容器、最小 handler 与 rmcp/axum 挂载形态** — TDD 两段：
   - `d4eeb5f` (test) — RED：`tests/trait_injection.rs` + Cargo 依赖调整
     （落点：`could not find deps / server in prism_mcp`）
   - `d71cbb6` (feat) — GREEN：`deps.rs` / `handler.rs` / `server.rs` / `lib.rs`，3 passed
2. **Task 2: Host / Origin / bearer 三层中间件与 403 门禁测试** — TDD 两段：
   - `a88e7df` (test) — RED：`tests/middleware_gate.rs`
     （落点：`could not find middleware in prism_mcp`）
   - `5f2080a` (feat) — GREEN：`middleware.rs` + `build_router` 叠三层 + SDK 侧 allowlist，19 passed

**Plan metadata:** 见本 commit（docs: complete plan）

## Files Created/Modified

### prism-mcp

- `src/deps.rs`（新，96 行）— `McpDeps { feedback: Arc<dyn FeedbackSource>, comments: Arc<dyn CommentSink>, bearer: Arc<str> }`；
  bearer 私有 + `pub(crate) expose_bearer()`；手写脱敏 `Debug`，**刻意无 `Display`**；2 个单测
- `src/handler.rs`（新，104 行）— `PrismHandler` 实现 rmcp `ServerHandler`：`get_info`（宣告 tools 能力）、
  `list_tools`、`call_tool`。`call_tool` 先 `Arc::clone` 再 `spawn_blocking` 调同步 trait，并分别处理
  JoinError 与 `ServiceError`
- `src/middleware.rs`（新，226 行）— `ALLOWED_HOSTS` / `ALLOWED_ORIGINS` 常量；三个 `from_fn` 可用的
  async 校验函数；`host_of`（剥端口/剥 IPv6 方括号/小写化）、`origin_tuple`（拆 scheme+host、拒路径）、
  `constant_time_eq`（subtle `ct_eq` + 异或折叠）；4 个单测
- `src/server.rs`（新，96 行）— `MCP_MOUNT_PATH` / `LOOPBACK_BIND`；`build_router`（`nest_service` 挂
  `StreamableHttpService` + 三层 `.layer()` + SDK 侧 allowlist）；`serve_loopback`（`127.0.0.1:0`、
  读回实际地址、后台任务 + `with_graceful_shutdown`）
- `src/lib.rs`（改）— 四个 `pub mod`；`McpError` 加 `#[non_exhaustive]`
- `Cargo.toml`（改）— 普通依赖加 tokio(net)/tokio-util/subtle/serde/serde_json/tracing；
  `[dev-dependencies]` 加 tokio(net,time)/reqwest/tower(util)
- `tests/trait_injection.rs`（新，205 行）— 3 个测试（注入通路、阴性对照、loopback 绑定）
- `tests/middleware_gate.rs`（新，429 行）— 10 个测试：A 组端到端 4 项 + B 组隔离 3 项 + T-01-29 无差别拒绝 +
  token 不回显 + A5 顺序

### workspace

- `Cargo.toml`（改）— `[workspace.dependencies]` 加 `tokio-util = "0.7"` 与 `tower = "0.5"`
- `Cargo.lock`（改）

## Decisions Made

1. **`McpDeps.bearer` 是私有字段 + `pub(crate) expose_bearer()`，不是计划写的 `pub bearer: Arc<str>`。**
   计划的 `pub` 字段让 token 原文的读取点散布不可控；私有 + 单一具名访问器让「谁读了 token」
   在一次 `grep expose_bearer` 里就能穷举。注入方仍可经 `McpDeps::new` 传入，**能力面没有损失**。
2. **三层一律 403 + 空正文，不给 bearer 缺失单开 401。** 计划说「bearer 缺失时 401 更准确，
   但两者都必须是 4xx 且不泄漏哪一层挂了」——这两条要求在实现上是冲突的：只要 bearer 层的状态码
   与另两层不同，攻击者就能靠状态码判定自己已经过了 Host 与 Origin 两关，把三层试探降成逐层试探。
   T-01-29 优先，统一 403。`rejections_do_not_disclose_which_layer_denied` 用 `assert_eq!` 把
   三层的 `(status, body)` 两两锁死。
3. **rmcp SDK 侧的 `allowed_hosts` / `allowed_origins` 显式配成与应用层同一份。**
   SDK 的 `allowed_hosts` 默认已是 loopback 三项，但 `allowed_origins` **默认为空 = 不校验**。
   显式复述一遍让两处不会因 SDK 默认值变更而漂移。这是知情的取舍：它让计划的端到端摘层反证失效
   （见下），换来的是「即使有人误删应用层中间件，Origin 校验仍在」。
4. **常数时间比较用异或折叠而非截断/填零。** 若把过长的 presented 截断到 expected 长度再比较，
   「正确 token + 任意后缀」在字节层面就等于正确 token，只剩长度断言在挡——单点故障。折叠让
   超出部分也进入比较结果。`constant_time_eq_agrees_with_equality_on_every_shape` 专门覆盖了
   前缀、加后缀、整数倍长度三种折叠碰撞候选。
5. **Phase 1 保留 rmcp 的 stateful 会话模式（SDK 默认），不改成 stateless + `json_response`。**
   后者能把测试从三步握手简化成一次 POST，但那是为测试便利改生产形态。实测三步握手在
   0.1s 内跑完且不挂（request-wise SSE 流在响应后即关闭），没有理由妥协。
6. **`ALLOWED_HOSTS` 存无括号的 `"::1"` 而非计划写的 `"[::1]"`。** `host_of` 在比较前统一剥掉
   方括号，两端形态一致更不易出错；`host_of_strips_port_and_brackets` 覆盖了 `[::1]` / `[::1]:8080` /
   `[::1`（畸形）三种输入。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] 计划的两条端到端摘层反证实跑不成立**

- **Found during:** Task 2 验收
- **Issue:** 验收项要求「临时把 `require_local_host` 从 `build_router` 中摘掉后 `rejects_foreign_host`
  变红」。实跑结果是 **10 passed / 0 failed** —— 一条也没红。原因是 rmcp 2.2 的
  `StreamableHttpService::handle` 在进入路由之前会调 `validate_dns_rebinding_headers`，
  其 `allowed_hosts` 默认值就是 `["localhost", "127.0.0.1", "::1"]`（GHSA-89vp-x53w-74fx 的上游修复）。
  应用层中间件被摘掉后，SDK 替它拒掉了 `evil.example.com`。
  Origin 层同理：因本 plan 把 SDK 侧 `allowed_origins` 也配成同一份，单摘应用层 Origin 中间件
  时 `rejects_foreign_origin` 同样保持全绿；只有**同时**摘掉 SDK 侧的 `.with_allowed_origins(...)`
  才落到 `middleware_gate.rs:156`（left 200 / right 403）。
  这正是 STATE.md 记录的失效模式第三次出现——反证变红与否不足以说明问题，要看**落点**；
  而当被测层之上还有第三方兜底时，摘层反证连红都不会红。
- **Fix:** 新增 B 组 **sentinel-router 隔离测试**：把每一层单独 `.layer()` 在一个返回固定字符串的
  最小 handler 上（链路里没有 rmcp、没有任何兜底），并在同一个测试里对比「挂了这层」与
  「没挂这层」——没挂时请求直达 sentinel。这样断言失败的落点唯一，只可能是被测的那一层没生效。
  三条隔离测试各自的反证已实跑验证落点：
  - 把 `require_local_host` 的 allowlist 判断改为直通 → `middleware_gate.rs:254`
    `require_local_host 未拦下外域 Host`（left 200 / right 403）
  - 把 `require_origin_allowlist` 的 allowlist 判断改为直通 → `middleware_gate.rs:281`
    `require_origin_allowlist 未拦下外域 Origin`（left 200 / right 403）
  - 从 `build_router` 摘掉 `require_bearer` → `middleware_gate.rs:191`
    `无 Authorization 头却未被拒`（bearer 无第三方兜底，摘层反证在这一层是成立的）
- **Files modified:** `crates/prism-mcp/tests/middleware_gate.rs`（新增 B 组）
- **Commit:** `a88e7df`（RED，反证形态即写在测试里）/ `5f2080a`（GREEN）

**2. [Rule 2 - Security] `McpDeps.bearer` 由 `pub` 字段收紧为私有 + 具名访问器**

- **Found during:** Task 1 GREEN
- **Issue:** 计划写 `pub struct McpDeps { …, pub bearer: Arc<str> }`。公开字段意味着 token 原文的
  读取点无法穷举，且 `#[derive(Debug)]` 一旦被后续 plan 加上就会把 token 打进日志——
  与 01-04 为 `ApiKey` 建立的口径（私有 + 单一 `expose()` + 脱敏 Debug + 无 Display）直接冲突。
- **Fix:** 字段私有，加 `pub(crate) expose_bearer()`；手写 `Debug` 输出 `<redacted>`；
  刻意不实现 `Display`。`debug_does_not_reveal_the_bearer_token` 与
  `the_bearer_token_never_appears_in_a_response` 两个测试锁死。
- **Files modified:** `crates/prism-mcp/src/deps.rs`
- **Commit:** `d71cbb6`

**3. [Rule 2 - Security] bearer 缺失的状态码由计划建议的 401 改为与另两层一致的 403**

- **Found during:** Task 2 GREEN
- **Issue:** 计划同时要求「bearer 缺失时用 401 更准确」与「不泄漏哪一层挂了」（T-01-29）。
  两者不可兼得：状态码差异本身就是落点信息。
- **Fix:** 统一 403 + 空正文，真实原因只进 `tracing::warn!`。计划的 must_haves truth 写的是
  「得到 401/403」，403 满足该条。
- **Files modified:** `crates/prism-mcp/src/middleware.rs`
- **Commit:** `5f2080a`

### 计划与仓库形态不符（无需修改，与 01-04 同一条）

验收项「`crates/prism-mcp/Cargo.toml` 的 rmcp 一行同时包含 `"server"` 与
`"transport-streamable-http-server"`」按字面读会失败：本仓库自 01-01 起用
`[workspace.dependencies]` 集中管理版本与 feature 串，crate 侧一律写 `{ workspace = true }`。
feature 实际声明在根 `Cargo.toml` 第 33 行，等价证据：

```
cargo tree -p prism-mcp -e features | grep 'rmcp feature'
  → rmcp feature "server"
  → rmcp feature "transport-streamable-http-server"
  → rmcp feature "server-side-http" / "transport-streamable-http-server-session" / …
```

## Known Stubs

**None（按「阻碍本 plan 目标达成」的口径）。**

`PrismHandler` 只注册了一个 `list_feedback` 工具，这**不是 stub 而是计划明写的交付边界**
（RESEARCH § Pattern 4「Phase 1 交付边界」：「只需要一个能编译、能起 axum、能通过一次注入 trait
返回假数据的最小 handler。工具面、端口发现、CLI helper 契约全部是 Phase 6」）。
它有真实实现、真实数据来源（注入的 trait）与两条测试，不存在硬编码返回值。

`McpDeps.comments`（`Arc<dyn CommentSink>`）目前无消费者——与 01-04 的 `ACCOUNT_MCP_TOKEN` 同理，
是**契约的前置定名**：Phase 6 的评论回流工具直接用，容器形状不必到那时再改。

已向 `.planning/WINDOWS.md` 登记 1 条 `deviation`（计划反证不成立及其替代形态）。

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: information-disclosure | `crates/prism-mcp/src/server.rs` | rmcp SDK 自己的 Host 拒绝响应体是 `"Forbidden: Host header is not allowed"`，与本项目 T-01-29 的无差别拒绝口径不一致。当前应用层在外先拒使其**不可达**（两处 allowlist 相同 ⇒ 能过应用层的必能过 SDK 层），但若 Phase 6 调整中间件顺序或让两份 allowlist 不再等价，SDK 的正文会泄漏落点。已记入 STATE.md Blockers。 |

## Issues Encountered

**一个真实问题，已解决（见 Deviations 第 1 条）：计划的两条反证被第三方兜底掩盖。**

值得单独记一笔的是**发现方式**：如果只按验收项「摘掉 → 看是否变红」执行，会得到「全绿 ⇒ 反证不成立
⇒ 大概是我摘错了地方，改改再试」这条错误路径。真正让问题浮出来的是先问「除了我这一层，
还有谁可能拒掉这个请求」，然后去读 rmcp 的 `tower.rs` 源码，看到 `validate_dns_rebinding_headers`
在 `handle` 的第一行。**第三方 crate 的默认安全行为是反证的隐形干扰项**——这条已写进 STATE.md。

其余顺利：两个 RED 均按预期以「模块不存在」失败，两个 GREEN 各只有一次编译错误
（rmcp 的 `ServerInfo`/`Implementation` 是 `#[non_exhaustive]`，不能用结构体字面量构造，
改为 `Default::default()` + 字段赋值）。三步 MCP 握手一次通过且不挂。

## Verification Evidence

```
cargo test -p prism-mcp                                     → 10 unit + 10 middleware_gate + 3 trait_injection
cargo test -p prism-mcp --test trait_injection              → 3 passed（≥2，两个必需测试都在）
cargo test -p prism-mcp --test trait_injection injected_feedback_source_is_reached → 1 passed
cargo test -p prism-mcp --test trait_injection empty_source_yields_no_item         → 1 passed
cargo test -p prism-mcp --test middleware_gate              → 10 passed
  rejects_foreign_host                                      → 1 passed / 9 filtered
  rejects_foreign_origin                                    → 1 passed / 9 filtered
  rejects_missing_or_wrong_bearer                           → 1 passed / 9 filtered
  accepts_fully_valid_request                               → 1 passed / 9 filtered
cargo test --workspace                                      → 87 passed / 1 ignored / 0 failed
npm run test -- --run                                       → 3 passed
cargo clippy -p prism-mcp --all-targets -- -D warnings      → exit 0
cargo clippy --workspace --all-targets -- -D warnings       → exit 0
bash scripts/check-deps.sh                                  → dup / tauri-free / no-cycle / single-egress 四条全 OK
bash scripts/check-secrets.sh                               → exit 0

# D-09 依赖方向（普通边与 dev 边）
cargo tree -p prism-mcp --edges normal --prefix none | tail -n +2 | grep -c '^prism-engine ' → 0
cargo tree -p prism-mcp --edges dev    --prefix none | tail -n +2 | grep -c '^prism-engine ' → 0
cargo tree -p prism-mcp -e features | grep 'rmcp feature'    → server / transport-streamable-http-server 在列

# 源码形态
grep -c 'spawn_blocking'          crates/prism-mcp/src/handler.rs     → 2
grep -c 'nest_service'            crates/prism-mcp/src/server.rs      → 2
grep -c '127.0.0.1'               crates/prism-mcp/src/server.rs      → 2
grep -c '0.0.0.0'                 crates/prism-mcp/src/server.rs      → 0
grep -c 'Arc<dyn FeedbackSource>' crates/prism-mcp/src/deps.rs        → 2
grep -c 'Arc<dyn CommentSink>'    crates/prism-mcp/src/deps.rs        → 2
grep -c 'ct_eq'                   crates/prism-mcp/src/middleware.rs  → 4
grep -c 'ConstantTimeEq'          crates/prism-mcp/src/middleware.rs  → 1
wc -l crates/prism-mcp/src/middleware.rs                              → 226（≥ min_lines 50）
wc -l crates/prism-mcp/tests/middleware_gate.rs                       → 429（≥ min_lines 60）

# 反证的落点（每条都确认了红在哪一条断言，而不只是红绿）
摘 require_local_host（从 build_router）        → 10 passed —— 未变红，SDK 兜底掩盖（见 Deviations 1）
require_local_host allowlist 判断改直通         → middleware_gate.rs:254 `require_local_host 未拦下外域 Host`
                                                  left 200 / right 403 ✔ 落点正确
摘 require_origin_allowlist（从 build_router）  → 10 passed —— 未变红，SDK 兜底掩盖
再摘 SDK 侧 .with_allowed_origins(...)          → middleware_gate.rs:156 `只有 Origin 是外域，却没有 403`
                                                  left 200 / right 403 ✔
require_origin_allowlist allowlist 判断改直通   → middleware_gate.rs:281 `require_origin_allowlist 未拦下外域 Origin`
                                                  left 200 / right 403 ✔ 落点正确
摘 require_bearer（从 build_router）            → middleware_gate.rs:191 `无 Authorization 头却未被拒`
                                                  ✔ 落点正确（bearer 层无第三方兜底）
恢复后                                          → 10 passed；diff 与反证前源码逐字节一致

# 提交未删除任何被跟踪文件
git diff --diff-filter=D --name-only HEAD~4 HEAD                      → 空
```

## Self-Check

见文末 `## Self-Check` 段。

## Next Phase Readiness

**已就绪，可开工的下游 plan：**

- **01-07（集成验证）** — `serve_loopback(deps, ct)` 返回 `(SocketAddr, JoinHandle<()>)`，
  `ct.cancel()` 即优雅关停；三层合法头的最小组合是
  `Host: 127.0.0.1:<port>` + `Origin: http://127.0.0.1` + `Authorization: Bearer <token>` +
  `Accept: application/json, text/event-stream` + `Content-Type: application/json`。
- **01-08（冒烟命令与事件总线）** — 与本 plan 无耦合；若冒烟页要展示 MCP 端口，
  `MCP_MOUNT_PATH` 与 `serve_loopback` 返回的 `SocketAddr` 是唯一来源。
- **Phase 6（MCP 工具注册与 CLI helper）** — 在 `prism-types/src/service.rs` 追 trait、
  在 `PrismHandler::call_tool` 分派、在 `McpDeps` 加字段；**不动 `middleware.rs` 与
  `build_router` 的三层**。bearer 由 `prism-engine` 从钥匙串 `mcp_bearer_token` 读出后
  经 `McpDeps::new` 注入（CSPRNG ≥256-bit 生成也在那时落地——prism-mcp 自己永远不生成 token）。

**需要注意的四点：**

1. **不要把「摘掉某层看是否变红」当作充分的反证。** rmcp 自带 Host/Origin 校验会掩盖它。
   本 plan 的 B 组 sentinel-router 形态是可复用的模板：被测层 + 最小 handler，没有第三方兜底。
2. **rmcp SDK 的拒绝正文会透露落点**（`"Forbidden: Host header is not allowed"`）。
   当前不可达，但两份 allowlist 一旦不再等价就会暴露——已记入 STATE.md Blockers。
3. **`prism-mcp` 的 `reqwest` 必须留在 dev 边。** 一旦挪到普通依赖，`check-deps.sh` 的
   single-egress 会经 `prism-engine → prism-mcp` 的路径抓到它，NFR-03 的「唯一网络出口」失守。
4. **`McpDeps` 加字段时保持「不 derive Debug」。** 容器里有 token；一次
   `#[derive(Debug)]` 就会把 01-04 与本 plan 建立的三层 redaction 一起作废。

---
*Phase: 01-foundation-skeleton*
*Completed: 2026-07-29*

## Self-Check

**PASSED**

- 6 个新建文件全部存在于工作树：`crates/prism-mcp/src/{deps,handler,middleware,server}.rs`、
  `crates/prism-mcp/tests/{trait_injection,middleware_gate}.rs`
- 4 个 commit 全部可在 `git log` 中找到：`d4eeb5f`、`d71cbb6`、`a88e7df`、`5f2080a`
- `git diff --diff-filter=D --name-only HEAD~4 HEAD` 为空 —— 未删除任何被跟踪文件
- `middleware.rs` 226 行 ≥ must_haves 的 min_lines 50；`middleware_gate.rs` 429 行 ≥ min_lines 60
- must_haves 的 key_links 三条各自可 grep：`nest_service`（server.rs ×2）、
  `spawn_blocking`（handler.rs ×2）、`from_fn`（server.rs ×3，含 `from_fn_with_state`）
- 反证恢复后的源码与反证前逐字节一致（`diff` 无输出）
