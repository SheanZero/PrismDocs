---
phase: 01-foundation-skeleton
plan: 08
subsystem: infra
tags: [tauri-shell, ipc, broadcast-bridge, lagged-resync, tauri-channel, mock-runtime, acl-origin, error-mapping, counter-proof-landing]

# Dependency graph
requires:
  - "01-01：src-tauri 薄壳、AppState、dev_ping tracer、`[features] test = [\"tauri/test\"]`"
  - "01-04：prism-types 的 EngineEvent（DocChanged / InboxUpdated / Resync）"
  - "01-05：prism-store 的 settings 与 search"
  - "01-07：Engine::subscribe / publish / search / get_setting / set_base_url / init_secrets / set_api_key / api_key_status"
provides:
  - "bus_adapter —— broadcast → `prism://changed` 粗粒度 Tauri event 的桥；map_recv 纯函数（Emit / Resync / Stop）+ EVENT_CHANGED 常量"
  - "smoke —— SmokeEvent 与 generate/collect（Started 首、seq 0..total 的 Tick、Finished 末），不依赖 tauri"
  - "commands —— 八个 #[tauri::command]，统一经 delegate + spawn_blocking 单行委托到 facade"
  - "commands::map_err —— EngineError → 稳定短错误码串（invalid_url / invalid_setting / store_error / secret_error / engine_error）"
  - "src-tauri/tests/ipc.rs —— mock_builder 进程内 IPC 测试（含未注册命令负对照与 devUrl 来源常量）"
  - "scripts/check-deps.sh shell-egress —— 「src-tauri 不得直接依赖 prism-llm」的第六条断言（补上 01-07 交接的缺口）"
  - "prism_engine::LlmError re-export —— 公开变体的载荷类型对不依赖 prism-llm 的上层可命名"
affects: [01-09-冒烟页, phase-2-watcher事件接入, phase-4-LLM设置页, phase-6-MCP工具注册]

# Tech tracking
tech-stack:
  added:
    - "tracing（src-tauri 普通依赖：发射失败与钥匙串不可用的 warn）"
    - "serde_json + tempfile（src-tauri **dev-only**：ipc 测试的载荷与隔离库）"
  patterns:
    - "「收到什么 → 该做什么」抽成纯函数枚举，让 tauri 运行时不再是单测的前置条件"
    - "有序性断言必须是**序列比较**（`assert_eq!(seqs, 0..total)`），集合/排序后比较对乱序不敏感"
    - "进程内 IPC 测试的来源 URL 必须等于 tauri.conf.json 的 devUrl —— `http://tauri.localhost` 是 Windows 形态，在 macOS 上非本地来源会强制走 ACL"
    - "「命令已注册」的断言要配一个未注册命令的负对照，否则 marker 写错时恒真"
    - "测试进程不调 init_secrets，keyring 无默认后端 → 密钥类命令快速失败，既不弹授权框也不污染真实钥匙串"
    - "依赖方向断言的受检集合要随新 crate 增长：shell 不在集合里时，`prism-llm = {…}` 一行就能让 NFR-03 的单一入口变两条而无人察觉"

key-files:
  created:
    - src-tauri/src/bus_adapter.rs
    - src-tauri/src/smoke.rs
    - src-tauri/tests/ipc.rs
  modified:
    - src-tauri/src/commands.rs
    - src-tauri/src/lib.rs
    - src-tauri/Cargo.toml
    - scripts/check-deps.sh
    - crates/prism-engine/src/lib.rs
    - Cargo.lock

key-decisions:
  - "ipc 测试的来源 URL 用 devUrl（`http://localhost:1420`）而非研究示例里的 `http://tauri.localhost`：后者在 macOS 上 is_local=false，会让每个命令被 ACL 拒成 `not allowed. Plugin not found`——与「命令未注册」肉眼难分。是负对照抓出来的（见 Deviations 1）"
  - "`all_commands_are_registered` 从「错误不像未注册」加强为「六个非密钥命令必须返回 Ok + 两个密钥命令的错误必须恰为映射后的短码」：前者对「注册了但委托写错」不敏感"
  - "delegate 泛型助手 + spawn_blocking：facade 是同步的（MutexGuard !Send 的编译期保护靠这一点），命令层因此每个都只剩一行真正的委托"
  - "map_err 用 `#[non_exhaustive]` 兜底臂而不是穷举：新变体默认落到最粗类别，而不是编译失败后被人顺手改成 to_string()"
  - "prism-engine 转出 LlmError：`EngineError::Llm` 是公开变体，载荷类型不可命名会让 shell 无法为「密钥类错误映射成什么」写测试；转的是错误类型，不是密钥入口"
  - "check-deps.sh 补第六条 shell-egress，形态与 01-07 的 facade-egress 同构（直接依赖 + 反向闭包），允许名单多一个 prism-engine（合法中间跳）"
  - "不勾选 INFRA-01：成功标准 2 的人工半边（真实 WebView）在 01-09"

patterns-established:
  - "Phase 2 的 watcher：engine.publish(...) 即可，shell 侧无需改动——bus_adapter 已经在消费 subscribe()"
  - "新增命令：在 commands.rs 写一行 delegate + 在 lib.rs 的 generate_handler! 与 tests/ipc.rs 的两处清单里各加一行（COMMANDS 与 handler 列表刻意分离，漏一处即被负对照抓住）"
  - "任何新的 engine crate 或新的上层 crate 落地时，同步检查 check-deps.sh 的受检集合是否覆盖它"

requirements-completed: []

coverage:
  - id: D1
    description: "总线事件经粗粒度 Tauri event 到达前端：每条 EngineEvent 映射为一次 `prism://changed` 发射，映射为纯函数"
    requirement: "INFRA-01"
    verification:
      - kind: unit
        ref: "bus_adapter::tests::bus_adapter_maps_event_to_emit / ::bus_adapter_event_name_is_the_frontend_contract；`cargo test -p prismdocs-shell bus_adapter` → 4 passed"
        status: pass
      - kind: integration
        ref: "src-tauri/src/lib.rs setup 中 `bus_adapter::spawn(app.handle().clone(), state.engine.subscribe())`；`subscribe()` 在 lib.rs 出现 1 次，`prism://changed` 在 bus_adapter.rs 出现 3 次"
        status: pass
    human_judgment: false
  - id: D2
    description: "broadcast 落后不静默漏更新：Lagged→恰好一次 Resync；Closed→正常退出不 panic"
    requirement: "INFRA-01"
    verification:
      - kind: unit
        ref: "bus_adapter::tests::lagged_maps_to_resync / ::closed_stops_loop（各自单独运行 exit 0）"
        status: pass
      - kind: other
        ref: "反证 CP-1：Lagged 分支改为 Stop → 落点 bus_adapter.rs:91（`lagged_maps_to_resync` 的断言，left: Stop / right: Resync），**只有这一个**测试变红；恢复后逐字节一致"
        status: pass
    human_judgment: false
  - id: D3
    description: "Channel 有序流的序号严格单调无缺口（total=1000）"
    requirement: "INFRA-01"
    verification:
      - kind: unit
        ref: "smoke::tests::smoke_stream_seq_is_strictly_monotonic（单独运行 exit 0）—— 断言形态是**序列比较** `assert_eq!(seqs, (0..total).collect())` + 逐对 `pair[1] == pair[0] + 1`，非集合比较；首 Started、末 Finished、长度 total+2"
        status: pass
    human_judgment: false
  - id: D4
    description: "空输入语义明确：total=0 发出 Started 与 Finished、0 条 Tick、返回 Ok"
    requirement: "INFRA-01"
    verification:
      - kind: unit
        ref: "smoke::tests::smoke_stream_total_zero_emits_no_ticks（单独运行 exit 0）"
        status: pass
      - kind: unit
        ref: "smoke::tests::smoke_stream_stops_at_the_first_sink_failure —— sink 失败必须冒泡（吞掉的表现是命令恒 Ok 而前端一条没收到）"
        status: pass
    human_judgment: false
  - id: D5
    description: "全部八个命令已注册且可经进程内 mock runtime 调用"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "`cargo test -p prismdocs-shell --features test --test ipc` → `test result: ok. 2 passed`（≥2，非空绿）"
        status: pass
      - kind: other
        ref: "负对照：`definitely_not_a_command` 报 `Command definitely_not_a_command not found`，八个真命令均不同形"
        status: pass
      - kind: other
        ref: "反证 CP-2：从测试的 generate_handler! 摘掉 set_base_url → 落点 ipc.rs:141 `命令 set_base_url 未注册: \"Command set_base_url not found\"`，**只有** all_commands_are_registered 变红"
        status: pass
      - kind: integration
        ref: "六个非密钥命令实测返回 Ok（dev_ping → \"3.53.2\"、search_documents → []、get_setting → null、set_base_url / dev_emit_bus_event / dev_smoke_stream → null）"
        status: pass
    human_judgment: false
  - id: D6
    description: "tests/ipc.rs 的 cfg 门有效：未开 test feature 时编译为零个测试而非编译失败"
    requirement: "INFRA-01"
    verification:
      - kind: build
        ref: "`head -n 5 src-tauri/tests/ipc.rs | grep -c '#!\\[cfg(feature = \"test\")\\]'` → 1；`cargo test -p prismdocs-shell`（无 feature）exit 0，ipc 目标 `0 passed`；`cargo test --workspace` → 118 passed"
        status: pass
      - kind: other
        ref: "反证 CP-3：删掉首行 cfg → `cargo test -p prismdocs-shell` **编译期**失败，落点 `error[E0432]: unresolved import tauri::test` + `E0433: cannot find test in tauri`；恢复后逐字节一致"
        status: pass
    human_judgment: false
  - id: D7
    description: "命令体是对 facade 的单行委托，不含业务逻辑（T-01-14a）"
    requirement: "INFRA-01"
    verification:
      - kind: unit
        ref: "commands::tests::commands_carry_no_business_logic（生产代码 Connection / prepare / query_row / keyring 各 0 次）"
        status: pass
      - kind: integration
        ref: "生产代码计数：`#[tauri::command]` 8、`state.engine` 1（只在 delegate 里）、`spawn_blocking` 1"
        status: pass
    human_judgment: false
    rationale: "源码断言：一个把业务写进命令体的实现不会有哪次调用失败，能观测的只有它写在哪一层。"
  - id: D8
    description: "命令层不把内部错误原文透传前端（T-01-11）"
    requirement: "INFRA-03"
    verification:
      - kind: unit
        ref: "commands::tests::no_command_forwards_the_raw_engine_error_text（`e.to_string()` / `err.to_string()` / `|e| e.to_string` 各 0 次）"
        status: pass
      - kind: unit
        ref: "commands::tests::error_codes_are_stable_short_strings（invalid_url / invalid_setting / secret_error 三条字面值钉死）"
        status: pass
      - kind: unit
        ref: "commands::tests::mapped_errors_do_not_carry_lower_layer_text（把一段绝对路径塞进下层错误，映射结果里连 `/` 都不出现）"
        status: pass
      - kind: integration
        ref: "ipc 测试实测：set_api_key / api_key_status 的错误恰为 `\"secret_error\"`，不是 keyring 的平台错误原文"
        status: pass
    human_judgment: false
  - id: D9
    description: "密钥不经命令层回传；无 key 不阻断启动（T-01-04b / D-06）"
    requirement: "INFRA-03"
    verification:
      - kind: integration
        ref: "`api_key_status` 返回 bool；无任何命令返回密钥；`bash scripts/check-secrets.sh` → exit 0"
        status: pass
      - kind: integration
        ref: "src-tauri/src/lib.rs 的 init_secrets 失败路径只 `tracing::warn!`，不 `?`、不返回 Err 给 setup"
        status: pass
    human_judgment: false
  - id: D10
    description: "shell 通往钥匙串的路线唯一（NFR-03），且该性质现在有 CI 断言看着"
    requirement: "INFRA-03"
    verification:
      - kind: integration
        ref: "`bash scripts/check-deps.sh` → 六条全 OK（原五条未回退 + 新增 shell-egress）"
        status: pass
      - kind: other
        ref: "反证 CP-4：给 src-tauri 加 `prism-llm = { workspace = true }` → shell-egress 报 `FAIL: prismdocs-shell depends on prism-llm directly`，而**原四条（dup / tauri-free / no-cycle / facade-egress）全部保持 OK**——缺口真实存在，不是补了条恒真的装饰"
        status: pass
    human_judgment: false

# Metrics
duration: 11min
completed: 2026-07-29
status: complete
---

# Phase 01 Plan 08: 冒烟命令与事件总线桥 Summary

**A1 要验证的两条 IPC 通路在薄 shell 上打通：broadcast 经 `prism://changed` 粗粒度事件出界（Lagged 补一次 Resync），命令经 `tauri::ipc::Channel` 有序流式返回（seq 严格单调无缺口）；八个命令全部单行委托、错误经统一映射收敛，并补上了 01-07 交接的 `prismdocs-shell` 依赖断言缺口。**

## Performance

- **Duration:** ≈11 min agent 时间
- **Tasks:** 2（各按 TDD 走 RED→GREEN 两段）
- **Files created/modified:** 9（新建 3，修改 6）
- **测试增量:** workspace 107 → **118 passed**（prismdocs-shell 0 → 11 unit + 2 ipc）

## Accomplishments

- **两条通路各自验证了一次，且各自的失败模式都有落点确认的反证。** 事件侧守的是
  「静默漏更新」（Lagged 被丢弃），Channel 侧守的是「乱序」与「空输入挂起」。
- **有序性断言的形态是序列比较而不是集合比较。** `assert_eq!(seqs, (0..1000).collect())`
  在乱序时会红；把两边各自 `sort()` 或塞进 `HashSet` 再比就不会——后者只能证明
  「这些数都出现过」，而乱序正是这条通路唯一要防的失败模式。另配逐对
  `pair[1] == pair[0] + 1` 把「缺口」与「重复」分开命名。
- **ipc 测试的两个测试真的跑了，不是 0 个测试的空绿。** `--features test --test ipc`
  输出 `test result: ok. 2 passed`；反证 CP-3 确认 cfg 门不是装饰（删掉首行则
  `cargo test -p prismdocs-shell` 在编译期就炸）。
- **补上了 01-07 明确交接的断言缺口，并证明它不是恒真的。** 反证 CP-4：给 `src-tauri`
  加一行 `prism-llm` 依赖之后，**原有四条断言（dup / tauri-free / no-cycle /
  facade-egress）全部保持 OK**，只有新增的 `shell-egress` 变红——这正是缺口的形状。
- **测试不碰真实登录钥匙串。** 测试进程从不调 `init_secrets`，`keyring_core` 没有默认
  后端，`set_api_key` / `api_key_status` 在触碰 Keychain 之前就失败。副产品：它们的
  错误串恰好是验证 T-01-11 映射是否生效的现成材料。

## Task Commits

1. **Task 1: bus adapter（broadcast → 粗粒度 Tauri event）** — TDD 两段：
   - `2bed3d4` (test) — RED：三条映射的纯函数测试 + 事件名契约
     （落点：`error[E0432]: unresolved imports super::map_recv, super::BusOutcome, super::EVENT_CHANGED`）
   - `9d5f51b` (feat) — GREEN：`bus_adapter.rs` + lib.rs setup 接线 + tracing 依赖，4 passed
2. **Task 2: 全部命令与 Channel 有序流 + 进程内 IPC 测试** — TDD 两段：
   - `fdead9d` (test) — RED：smoke 三个测试 + ipc 两个测试
     （落点：`unresolved imports super::{collect, generate, SmokeEvent, SMOKE_DEFAULT_TOTAL}`；
     ipc 侧 `cannot find __cmd__search_documents in commands` 等 14 条）
   - `ffccc9e` (feat) — GREEN：`smoke.rs` / `commands.rs` / `generate_handler!` 八条 /
     `check-deps.sh shell-egress` / `prism-engine` re-export LlmError，11 unit + 2 ipc passed

**Plan metadata:** 见本 commit（docs: complete plan）

## Files Created/Modified

### src-tauri

- `src/bus_adapter.rs`（新，103 行）— `EVENT_CHANGED`、`BusOutcome`、`map_recv`（纯函数）、
  `spawn`（`tauri::async_runtime` 内消费 `broadcast::Receiver`）；4 个单测
- `src/smoke.rs`（新，114 行）— `SMOKE_DEFAULT_TOTAL = 1000`、`SmokeEvent`
  （`tag = "event", content = "data"`）、`generate`（sink 闭包，错误冒泡）、`collect`；3 个单测
- `src/commands.rs`（改，218 行）— 八个 `#[tauri::command]`、`delegate` 泛型助手、
  `map_err` 统一映射；4 个单测（两个源码级哨兵 + 两个行为断言）
- `src/lib.rs`（改）— `bus_adapter::spawn` 接线、`init_secrets` 只 warn、
  `generate_handler!` 八条
- `tests/ipc.rs`（新，185 行）— 首行 `#![cfg(feature = "test")]`；`mock_builder` +
  `get_ipc_response`；`LOCAL_ORIGIN` 常量；未注册命令负对照
- `Cargo.toml`（改）— 普通依赖加 `tracing`；dev 加 `serde_json` / `tempfile`

### workspace

- `scripts/check-deps.sh`（改）— 新增 `check_shell_egress` 与 `shell-egress` 子命令，
  并入 `single-egress` 与 `all`
- `crates/prism-engine/src/lib.rs`（改）— `pub use prism_llm::LlmError;`
- `Cargo.lock`（改）

## Decisions Made

1. **ipc 测试的来源 URL 必须是 devUrl。** 见 Deviations 1。这条值得写进 Phase 2+ 的
   记忆：任何新增 ipc 测试沿用 `LOCAL_ORIGIN` 常量即可，别再抄研究文档里的那个字面量。
2. **`all_commands_are_registered` 断言 Ok 而不只是「错误不像未注册」。** 后者对
   「命令注册了但委托写错了」完全不敏感——一个 `unimplemented!()` 的命令体也会通过。
   现在六个非密钥命令必须返回 Ok，两个密钥命令的错误必须**恰等于**映射后的短码。
3. **`delegate` 泛型助手承担 clone + spawn_blocking。** 这样每个命令体才真的是一行。
   facade 保持同步是 01-07 的纪律（`MutexGuard` 的 `!Send` 编译期保护），代价必须由
   调用方付，而把它付在一个地方比付在八个地方好。
4. **`map_err` 用兜底臂而非穷举 match。** `EngineError` 与 `StoreError` 都是
   `#[non_exhaustive]`。穷举会在下层加变体时编译失败，而那个时刻最容易发生的
   「修复」恰恰是把整个函数改回 `e.to_string()`。
5. **`prism-engine` 转出 `LlmError`。** 见 Deviations 2。
6. **`shell-egress` 的允许名单里有 `prism-engine`。** 它在 shell → engine → llm → keyring
   这条链上是**合法的中间跳**，不是第二个出口；断言要抓的是「有没有第二条边」。
7. **不勾选 INFRA-01。** 成功标准 2 的人工半边（真实 WebView 中事件到达 JS、
   前端收 Channel 流校验 seq）在 01-09。沿用 01-01/01-07 的口径。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] 研究文档给的 ipc 请求 URL 在 macOS 上不是本地来源，会让每个命令被 ACL 拒掉**

- **Found during:** Task 2 GREEN，ipc 测试首跑
- **Issue:** 计划与 `01-RESEARCH.md` § Code Examples 的示例都写
  `url: "http://tauri.localhost".parse().unwrap()`。首跑两个 ipc 测试**全红**，
  错误是 `"dev_ping not allowed. Plugin not found"`。
- **调查:** 读 tauri 2.11.5 源码 `webview/mod.rs:1698 is_local_url` 与 `1823` 的 ACL 分支：
  非本地来源会**强制**走 ACL；本项目没有 `capabilities/` 目录（`gen/schemas/capabilities.json`
  是 `{}`），于是每个命令都被拒。而 `http://<protocol>.localhost` 是 **Windows/Android**
  上的自定义协议形态（源码 1725–1732 行的 `#[cfg(any(windows, target_os = "android"))]` 分支），
  macOS 走的是 1703–1706 行的 `tauri://` scheme 比较或 1709 行的 `get_app_url` 相对判断。
  真实 app 在 dev 下的来源是 `tauri.conf.json` 的 `devUrl`。
- **Fix:** 引入 `LOCAL_ORIGIN = "http://localhost:1420"` 常量（附注释说明它必须等于 devUrl）。
- **为什么这条特别值得记：** `not allowed. **Plugin not found**` 与真正的未注册错误
  `Command X **not found**` 都含 `not found`。我的负对照 marker 正是 `not found`——
  换句话说，**如果我没写负对照、只断言「错误不含 not found」，这个测试会在
  「八个命令全部被 ACL 拒掉」的情况下照样以为自己在测注册。**
  负对照在这里不是形式主义，它是唯一发现问题的东西。
- **Files modified:** `src-tauri/tests/ipc.rs`
- **Verification:** 修正后 2 passed；反证 CP-2（摘掉一个命令）落在 ipc.rs:141
- **Commit:** `ffccc9e`

**2. [Rule 3 - Blocking] `EngineError::Llm` 的载荷类型在 shell 里不可命名**

- **Found during:** Task 2 GREEN，写 `error_codes_are_stable_short_strings` 时
- **Issue:** 要断言「密钥类错误映射成 `secret_error`」就得构造一个
  `EngineError::Llm(...)`，而 `LlmError` 只能从 `prism_llm` 命名。`src-tauri`
  **不依赖也必须一直不依赖** `prism-llm`（NFR-03，且正是本 plan 新增断言守的那条）。
  两条要求直接对撞。
- **调查:** 加 `prism-llm` 到 shell 是错的方向（它正是 CP-4 要抓的东西）。
  真正的问题在 `prism-engine`：`EngineError::Llm(LlmError)` 是**公开变体**，
  它的载荷类型对下游不可命名——这本身就是 API 的漏洞，不是本 plan 制造的。
- **Fix:** `crates/prism-engine/src/lib.rs` 加 `pub use prism_llm::LlmError;`
  （附注释说明转的是错误类型不是密钥入口：`secrets` 模块仍只有 prism-engine 够得着）。
- **Files modified:** `crates/prism-engine/src/lib.rs`
- **Verification:** `check-deps.sh` 六条全 OK（re-export 不产生依赖边）；
  `cargo test -p prism-engine` 未受影响
- **Commit:** `ffccc9e`

**3. [Rule 2 - Missing Critical] 计划的注册断言对「注册了但委托写错」不敏感**

- **Found during:** Task 2 GREEN，跑完负对照后
- **Issue:** 计划的验收项是「均不返回『命令未注册』类错误」。一个命令体写成
  `unimplemented!()` 或委托到错误的 facade 方法，都能通过这条。
- **Fix:** 把测试分成三段：负对照 → 八条不同形 → **六个非密钥命令必须返回 Ok** +
  **两个密钥命令的错误必须恰等于 `"secret_error"`**。第三段同时成了 T-01-11
  「错误经映射收敛」的行为侧证据（原本只有源码级哨兵）。
- **Files modified:** `src-tauri/tests/ipc.rs`
- **Verification:** 实测六条 Ok（`dev_ping → "3.53.2"` 证明委托一路走到 SQLite）
- **Commit:** `ffccc9e`

**4. [Rule 3 - Blocking] `clippy::unit_arg`**

- **Found during:** Task 2 GREEN 后的 clippy 门
- **Issue:** `dev_emit_bus_event` 写成 `Ok(engine.publish(...))`（`publish` 返回 `()`）。
- **Fix:** 拆成两行。
- **Commit:** `ffccc9e`

### 计划验收项的一处措辞修正（无需改代码）

计划的验收项 `head -n 5 src-tauri/tests/ipc.rs | grep -c '#!\[cfg(feature = "test")\]'` 期望输出 1，
但文件的模块文档注释里原本也写了这个字面量，实测输出 **2**。改的是注释措辞
（「首行那条 inner attribute」），不是代码——验收项现在字面成立，输出 1。

---

**Total deviations:** 4 auto-fixed（2 个 Rule 3 - 阻塞，1 个 Rule 1 - bug，1 个 Rule 2 - 缺失的关键性）
**Impact on plan:** Deviation 1 是本 plan 最有价值的发现（研究文档的示例在本平台上会产生
一个"红得像未注册"的假象）；其余三条是让断言真有判别性。计划的 `must_haves`、
`artifacts`、`key_links` 与 `prohibitions` 全部满足。

## Known Stubs

**None（按「阻碍本 plan 目标达成」的口径）。**

`dev_smoke_stream` 与 `dev_emit_bus_event` 是**计划明写的冒烟入口**（D-06：冒烟页是脚手架），
不是占位——两者都有真实实现、真实断言。`smoke::SmokeEvent` 的数据是假的，但那是
D-06 明确允许的（「假数据流即可，不必真功能」），且有序性验证本来就不需要真数据。

`Engine::delete_api_key` 与 `EventBus::subscriber_count` 仍未被命令层消费——沿用 01-07
的判断，它们是接口的前置定名而非 stub。

已向 `.planning/WINDOWS.md` 登记 1 条 `deviation`（研究示例的 ipc 来源 URL 在 macOS 上
会让 ACL 误伤，且其错误串与「未注册」同含 `not found`）。

## Threat Flags

None——本 plan 未引入计划 `<threat_model>` 之外的安全面。五条处置全部落地：

| Threat | 处置 | 证据 |
|--------|------|------|
| T-01-11（内部错误原文透传前端） | mitigate | `map_err` 统一映射；`no_command_forwards_the_raw_engine_error_text`（三种 `to_string` 写法各 0 次）+ `mapped_errors_do_not_carry_lower_layer_text` + ipc 实测 `"secret_error"` |
| T-01-04b（命令返回密钥原文） | mitigate | `api_key_status -> Result<bool, String>`；八个命令中无一返回密钥；`check-secrets.sh` exit 0 |
| T-01-34（Lagged 被静默丢弃） | mitigate | `map_recv` 的 Lagged→Resync；反证 CP-1 落点确认 |
| T-01-14a（业务逻辑写进命令体） | mitigate | `commands_carry_no_business_logic`（Connection / prepare / query_row / keyring 各 0 次）；`state.engine` 仅出现 1 次（在 `delegate` 内） |
| T-01-35（事件载荷携带正文） | mitigate | 本层不新增载荷字段，直接转发 01-04 定形的 `EngineEvent`；`src-tauri/` 内 `INSERT INTO documents_fts` 0 次 |

**额外收紧（不在计划的威胁模型内）：** NFR-03 的「唯一密钥入口」此前在 `src-tauri`
层面**没有任何断言**。新增的 `check_shell_egress` 补上了，反证 CP-4 证明它非恒真。

## Issues Encountered

**一个真问题（Deviations 1）与三个判别性/工具性缺口，均已解决。**

Deviations 1 值得单独记一笔，因为它是"反证/负对照制度"第一次抓到的**不是我自己写错的**
东西——研究文档抄的官方示例在 macOS 上会触发一条与被测性质无关的拒绝路径，且
拒绝文本与真正的失败文本共享 `not found` 子串。如果按计划字面写「断言错误里没有
not found」而不配负对照，这个测试会在**每个命令都没被执行**的情况下报绿。
这是 01-06 那条教训（「被测层之上若有第三方 backstop，反证会被掩盖」）的一个新变种：
这次 backstop 不在被测层之上，而是**平行**的一条拒绝路径，它的错误文本恰好与
被测失败模式撞车。

其余顺利：两个 RED 均按预期以「符号不存在」失败；两个 GREEN 各一次通过（除 clippy 一处）。
四条反证全部一次到位地落在预期断言上，三个被临时改动的文件恢复后均逐字节一致。

## Verification Evidence

```
cargo test -p prismdocs-shell                       → 11 passed（lib）+ ipc 目标 0 passed（cfg 门生效）
cargo test -p prismdocs-shell --features test       → 11 unit + 2 ipc passed
cargo test -p prismdocs-shell --features test --test ipc
                                                    → test result: ok. 2 passed   ← ≥2，非空绿
cargo test -p prismdocs-shell bus_adapter           → 4 passed
cargo test -p prismdocs-shell lagged_maps_to_resync → 1 passed（单独运行）
cargo test -p prismdocs-shell closed_stops_loop     → 1 passed（单独运行）
cargo test -p prismdocs-shell smoke_stream_seq_is_strictly_monotonic  → 1 passed（total=1000）
cargo test -p prismdocs-shell smoke_stream_total_zero_emits_no_ticks  → 1 passed
cargo test --workspace                              → 118 passed / 1 ignored / 0 failed（107 → 118）
npm run test -- --run                               → 3 passed
cargo clippy -p prismdocs-shell --all-targets --features test -- -D warnings → exit 0
cargo clippy --workspace --all-targets -- -D warnings                        → exit 0
bash scripts/check-deps.sh                          → **六条全 OK**（原五条 + shell-egress）
bash scripts/check-deps.sh shell-egress             → exit 0
bash scripts/check-secrets.sh                       → exit 0

# cfg 门（precondition 1）
head -n 5 src-tauri/tests/ipc.rs | grep -c '#!\[cfg(feature = "test")\]'     → 1

# 源码形态（**生产代码**计数：去注释行、去 #[cfg(test)] 之后）
commands.rs  #[tauri::command]                 → 8
commands.rs  state.engine                      → 1（只在 delegate 内）
commands.rs  spawn_blocking                    → 1
commands.rs  Connection / prepare / query_row / keyring → 各 0
lib.rs       generate_handler! 内 commands::   → 8
bus_adapter.rs  Lagged                         → 5   （≥1，分支产出 Resync）
bus_adapter.rs  prism://changed                → 3   （key_links）
lib.rs       subscribe()                       → 1   （key_links）
src-tauri/ 下 INSERT INTO documents_fts        → 0   （触发器仍是唯一同步路径）
wc -l bus_adapter.rs / commands.rs             → 103 / 218（≥ min_lines 50 / 60）

# ipc 测试实测的八条响应（证明断言不是在空跑）
dev_ping           => Ok("3.53.2")      ← 一路走到 bundled SQLite
search_documents   => Ok([])
get_setting        => Ok(null)
set_base_url       => Ok(null)
dev_emit_bus_event => Ok(null)
dev_smoke_stream   => Ok(null)
set_api_key        => Err("secret_error")   ← 映射后的短码，非 keyring 原文
api_key_status     => Err("secret_error")
definitely_not_a_command => Err("Command definitely_not_a_command not found")  ← 负对照

# 四条反证的落点（每条都确认了红在哪一条断言，而不只是红绿）
CP-1  Lagged 分支改为 Stop        → bus_adapter.rs:91 `left: Stop / right: Resync`，
                                     **只有** lagged_maps_to_resync 变红 ✔ 落点隔离
CP-2  测试 handler 摘掉 set_base_url → ipc.rs:141 `命令 set_base_url 未注册:
                                     "Command set_base_url not found"`，
                                     **只有** all_commands_are_registered 变红 ✔
CP-3  删掉 ipc.rs 首行 cfg        → `cargo test -p prismdocs-shell` **编译期**失败，
                                     落点 E0432 `unresolved import tauri::test`
                                     + E0433 `cannot find test in tauri` ✔
CP-4  src-tauri 加 prism-llm 直接依赖 → shell-egress `FAIL: prismdocs-shell depends on
                                     prism-llm directly`；而 dup / tauri-free /
                                     no-cycle / facade-egress **四条全部保持 OK**
                                     ——证明这是一个真实缺口而非恒真装饰 ✔
恢复后 diff /tmp/{bus_adapter,ipc2,shell-cargo}.bak → 三处均与反证前**逐字节一致**

# 提交未删除任何被跟踪文件
git diff --diff-filter=D --name-only HEAD~2 HEAD                              → 空
```

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

**已就绪，可开工的下游 plan：**

- **01-09（冒烟页）** — 前端需要的三件东西齐了：
  1. `listen<EngineEvent>('prism://changed', …)` —— 事件名见 `bus_adapter::EVENT_CHANGED`；
     载荷是 `EngineEvent` 的 serde 形态（内部标签 `kind`，camelCase），
     `kind === 'resync'` 时做**全量** `invalidateQueries`；
  2. `invoke('dev_emit_bus_event', { projectId, docId })` 触发一次往返；
  3. `new Channel<SmokeEvent>()` + `invoke('dev_smoke_stream', { onEvent, total: 1000 })`，
     载荷形态是 `{ event: 'started'|'tick'|'finished', data: {...} }`
     （`tag = "event", content = "data"` + camelCase）。前端跑与 Rust 侧**同口径**的
     `seq[i] === i` 校验。
  4. settings 页四个命令：`set_api_key` / `api_key_status`（**布尔**）/ `get_setting` /
     `set_base_url`。错误串是**短码**（`invalid_url` / `invalid_setting` / `store_error` /
     `secret_error` / `task_failed` / `channel_send_failed`），不是给人读的句子——
     前端据码分支、自己出中文文案。
- **Phase 2（导入与文件监视）** — watcher 只需 `engine.publish(...)`，shell 侧零改动。
- **Phase 4（LLM 设置页）** — 命令面已定形，只需在 `map_err` 加新码 + 在两处清单各加一行。

**需要注意的五点：**

1. **新增命令要改三处：** `commands.rs`（一行 delegate）、`lib.rs` 的 `generate_handler!`、
   `tests/ipc.rs` 的 `COMMANDS` 清单（以及 `COMMANDS_EXPECTED_OK` 或
   `COMMANDS_NEEDING_KEYCHAIN` 之一）。后两处**刻意分离**：只改一处会被负对照抓住。
2. **ipc 测试的 `url` 一律用 `LOCAL_ORIGIN` 常量，别再抄 `http://tauri.localhost`。**
   见 Deviations 1。如果哪天 `tauri.conf.json` 的 `devUrl` 端口变了，这个常量要跟着改，
   否则所有 ipc 测试会以 `not allowed` 集体变红（好消息：会红，不会静默）。
3. **将来若真给项目加 `capabilities/` 目录，ipc 测试的行为会变。** 有了 app ACL manifest
   之后，`has_app_acl_manifest` 为 true，即使本地来源也会走 ACL——届时测试要么加一份
   测试用 capability，要么显式记录这层新前置条件。
4. **不要为了"方便"把 `prism-llm` 给 `src-tauri`。** 现在这条有断言看着了
   （`check-deps.sh shell-egress`），但断言只会在 CI 里红，不会替人做决定。
   需要新的密钥能力时，正确做法是给 `Engine` 加方法。
5. **`cargo test --workspace` 不覆盖 ipc 两项。** 阶段闸门（01-09）必须显式追加
   `cargo test -p prismdocs-shell --features test`——这是 `01-VALIDATION.md`
   precondition 1 的原文要求，本 plan 的 cfg 门让它成为硬性的两条命令而非一条。

---
*Phase: 01-foundation-skeleton*
*Completed: 2026-07-29*

## Self-Check: PASSED

- 3 个新建文件全部在盘上：`src-tauri/src/{bus_adapter,smoke}.rs`、`src-tauri/tests/ipc.rs`
- 4 个 commit 全部可在 `git log` 中找到：`2bed3d4` / `9d5f51b` / `fdead9d` / `ffccc9e`
- `git diff --diff-filter=D --name-only HEAD~2 HEAD` 为空——未删除任何被跟踪文件
- `bus_adapter.rs` 103 行、`commands.rs` 218 行，均 ≥ must_haves 的 `min_lines` 50 / 60
- must_haves 的 `key_links` 三条各自可 grep：`subscribe()`（lib.rs ×1）、
  `prism://changed`（bus_adapter.rs ×3）、`state.engine`（commands.rs ×1，在 delegate 内）
- ipc 目标实测 `test result: ok. 2 passed`（≥2，不是 0 个测试的空绿）
- 四条反证全部实跑并逐条确认落点；恢复后三处被改文件与反证前**逐字节一致**
- `cargo test --workspace` → 118 passed / 1 ignored / 0 failed；`npm run test -- --run` → 3 passed
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0
- `bash scripts/check-deps.sh` 六条全 OK；`bash scripts/check-secrets.sh` → exit 0
