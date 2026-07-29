---
phase: 01-foundation-skeleton
plan: 07
subsystem: infra
tags: [prism-engine, facade, event-bus, tokio-broadcast, trait-inversion, single-writer, single-egress, counter-proof-landing]

# Dependency graph
requires:
  - "01-01：prism-engine 的 tracer 版 Engine（new / ping）与 [workspace.dependencies] 版本 pin"
  - "01-02：scripts/check-deps.sh 的 dup / tauri-free / no-cycle / single-egress 四条断言"
  - "01-03：Store::write / Store::read 闭包式 API 与 schema v1"
  - "01-04：prism-types 的 EngineEvent / FeedbackSource / CommentSink / ServiceError；prism-llm::secrets 的密钥入口"
  - "01-05：prism_store::search 与 settings::{get_setting, set_setting, SETTING_BASE_URL}"
  - "01-06：McpDeps / serve_loopback / MCP_MOUNT_PATH 与三层门禁的合法请求头组合"
provides:
  - "EventBus —— tokio broadcast 事件总线（BUS_CAPACITY=256），publish 返回单元值并吞掉 SendError"
  - "Engine 门面 —— 单写者句柄持有者：subscribe / publish / ping / search / get_setting / set_base_url / init_secrets / set_api_key / api_key_status / delete_api_key"
  - "EngineError —— #[from] StoreError / LlmError，transparent 转发不追加任何文本"
  - "impl FeedbackSource / CommentSink for Engine —— D-09 注入面的实现侧"
  - "scripts/check-deps.sh facade-egress —— 「网络/密钥只能经 prism-llm 进入 facade」的反向闭包断言"
  - "三条源码级哨兵：不外泄连接句柄 / 密钥只经 prism_llm::secrets / 门面方法保持同步"
affects: [01-08-冒烟命令与事件总线, 01-09-冒烟页, phase-2-watcher事件接入, phase-6-MCP工具注册]

# Tech tracking
tech-stack:
  added:
    - "tokio（prism-engine 普通依赖：broadcast 总线；dev 侧加 time 供 timeout）"
    - "tracing 0.1（prism-engine：吞掉 SendError 与 receipt 记录）"
    - "prism-llm / prism-mcp（prism-engine 普通依赖，方向单向）"
    - "reqwest + tokio-util + keyring-core + serde_json（prism-engine **dev-only**）"
  patterns:
    - "总线 publish 返回 `()`：空输入（无订阅者）的正确语义是成功，不是错误"
    - "等事件的测试一律包 timeout：不包会在「publish 被改空」时挂住，反证就没有落点"
    - "把可能抢跑的前置断言移出判别性测试：它会让反证落在前置条件上而不是被守的那条断言上"
    - "facade 方法保持同步，spawn_blocking 由调用方负责——async fn 会废掉 MutexGuard !Send 的编译期保护"
    - "源码级哨兵要先剥掉注释行与测试模块，否则会被自己的字面量命中而恒红"
    - "「唯一出口」对叶子 crate 是「整棵树里没有」，对 facade 是「只能经某一条边进来」——后者用 cargo tree --invert 的反向闭包表达"

key-files:
  created:
    - crates/prism-engine/src/bus.rs
    - crates/prism-engine/src/error.rs
    - crates/prism-engine/src/facade.rs
    - crates/prism-engine/src/services.rs
    - crates/prism-engine/tests/facade.rs
  modified:
    - crates/prism-engine/src/lib.rs
    - crates/prism-engine/Cargo.toml
    - scripts/check-deps.sh
    - Cargo.lock

key-decisions:
  - "check-deps.sh 的 single-egress 拆成两条：叶子 crate 保持整树断言，prism-engine 改为「直接依赖里没有 + 反向闭包里除 prism-llm 外没有 prism-*」——计划的两条验收项在原脚本下互斥（见 Deviations 1）"
  - "端到端注入测试的判别性落在**空 project_id 的校验文本**上，而不是「响应里有 []」：Phase 1 的 list_feedback 返回空 vec，空结果与「handler 根本没调注入的 trait」不可区分"
  - "bus_broadcasts_to_multiple_subscribers 刻意不断言 subscriber_count()==2：该前置条件会抢在判别性断言之前变红，把反证落点带偏"
  - "list_feedback 的边界校验现在就位（不等 Phase 6）：它与有没有 comments 表无关，且是端到端测试唯一的判别性数据来源"
  - "EngineError 只有两个 transparent 变体：任何带格式串的新变体都可能把路径/密钥重新引进错误文本"
  - "不勾选 INFRA-01/02/03：三者分别还有 01-08 / 01-09 未完成（沿用 01-01/01-04/01-05 的口径）"

patterns-established:
  - "Phase 2+ 的写路径一律 engine.xxx() → store.write(|tx| …)：facade 不提供任何返回连接的方法，绕过的语法不存在"
  - "Phase 6/7 加 MCP 工具：prism-types 追 trait + services.rs 加 impl，**不动 prism-mcp**"
  - "反证跑完必须逐字节 diff 复原（本 plan 三个被改文件均已确认）"

requirements-completed: []

coverage:
  - id: D1
    description: "prism-engine 是唯一编排入口且不依赖 tauri：engine 选择集全绿且全程不编译 tauri"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "cargo test -p prism-types -p prism-store -p prism-fs -p prism-parse -p prism-anchor -p prism-llm -p prism-mcp -p prism-engine → 全绿；grep -c 'Compiling tauri v' → 0"
        status: pass
      - kind: integration
        ref: "cargo tree -p prism-engine --edges normal,build,dev --prefix none | grep -c '^tauri ' → 0（比 build 输出更确定：不受编译缓存影响）"
        status: pass
      - kind: integration
        ref: "bash scripts/check-deps.sh tauri-free → OK"
        status: pass
    human_judgment: false
  - id: D2
    description: "engine 持有单写者句柄，facade 不暴露任何返回连接的公开方法（T-01-31）"
    requirement: "INFRA-01"
    verification:
      - kind: unit
        ref: "facade.rs#tests::no_public_method_hands_out_a_connection（源码级：-> Connection / -> &Connection / -> rusqlite::Connection / PooledConnection 各 0 次）"
        status: pass
      - kind: integration
        ref: "生产代码计数：`-> Connection|PooledConnection` → 0；`.write(|` → 1（写路径统一经闭包）"
        status: pass
      - kind: integration
        ref: "tests/facade.rs 的 seed_document 自己持 Arc<Store> 写入——这正是该纪律下调用方唯一能用的形态"
        status: pass
    human_judgment: false
    rationale: "这里只能是源码断言：一个「返回连接」的方法一旦存在，它本身就是漏洞，没有哪次调用会失败，因此没有任何行为测试能红。"
  - id: D3
    description: "订阅者能收到 publish 的事件"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "tests/facade.rs#bus_delivers_to_subscriber（单独运行 exit 0）"
        status: pass
      - kind: other
        ref: "反证 CP-1：publish 改为空实现 → 落点 facade.rs:53 `订阅者 在 2s 内没有收到事件`，2.00s 内变红而非挂住"
        status: pass
    human_judgment: false
  - id: D4
    description: "无订阅者时 publish 不 panic、不把 SendError 冒泡给调用方"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "tests/facade.rs#publish_with_no_subscriber_is_ok（单独运行 exit 0）；`let outcome: () = bus.publish(..)` 同时是编译期断言"
        status: pass
      - kind: unit
        ref: "bus.rs#tests::publish_returns_the_unit_value"
        status: pass
      - kind: other
        ref: "反证 CP-2：publish 改为 .expect(\"send\") → **只有**这两个测试变红（落点 bus.rs:52），其余四个集成测试全绿——落点隔离"
        status: pass
    human_judgment: false
  - id: D5
    description: "多个订阅者各自独立收到同一条事件（广播语义，非竞争消费）"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "tests/facade.rs#bus_broadcasts_to_multiple_subscribers（单独运行 exit 0）"
        status: pass
      - kind: other
        ref: "反证 CP-3：subscribe() 第 2 次起返回与总线无关的接收端 → 落点 facade.rs:53 `第二个订阅者 在 2s 内没有收到事件`，且**只有**这一个集成测试变红"
        status: pass
    human_judgment: false
  - id: D6
    description: "engine 实现 FeedbackSource / CommentSink，可装进 Arc<dyn …> 注入真实 prism-mcp 并完成一次端到端请求"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "tests/facade.rs#engine_satisfies_service_traits：真实 Engine → Arc<dyn> → McpDeps → serve_loopback → initialize / notifications/initialized / tools/call 三步握手"
        status: pass
      - kind: other
        ref: "反证 CP-9：拆掉 list_feedback 的空 project_id 校验 → 落点 facade.rs:389 判别性断言；且响应体证明 **rmcp 没有对 projectId 做任何兜底校验**（无第三方 backstop）"
        status: pass
      - kind: other
        ref: "反证 CP-10：list_feedback 改为返回一条非空项 → 落点 facade.rs:379 断言①——证明 `[]` 确实来自工具载荷而非蹭到的字符串"
        status: pass
      - kind: integration
        ref: "bash scripts/check-deps.sh no-cycle → OK: prism-mcp -> prism-types only（D-09 编译期性质未回退）"
        status: pass
    human_judgment: false
  - id: D7
    description: "搜索经 prism-store，且不是硬编码结果"
    requirement: "INFRA-02"
    verification:
      - kind: integration
        ref: "tests/facade.rs#search_delegates_to_store（4 字中文词命中 1 行 + 阴性对照 0 行）"
        status: pass
      - kind: other
        ref: "反证 CP-4：search 改为返回硬编码 SearchHit → 落点 facade.rs:170 **阴性对照**那一条（第一条断言仍绿——正是它证明阴性对照不可省）"
        status: pass
    human_judgment: false
  - id: D8
    description: "非密钥配置走 store settings 表，不误走钥匙串；写成功后广播失效信号"
    requirement: "INFRA-03"
    verification:
      - kind: integration
        ref: "tests/facade.rs#base_url_goes_to_settings_not_keychain（读回值 + mock 钥匙串条目数为 0 + Resync 事件 + 非法 scheme 被拒）"
        status: pass
      - kind: other
        ref: "反证 CP-5：set_base_url 追加一次 set_api_key → 落点 facade.rs:220 `写 base_url 在钥匙串里留下了条目`"
        status: pass
      - kind: other
        ref: "反证 CP-6：删掉 publish 一行 → 落点 facade.rs:229 `set_base_url 成功后应向总线发一条失效信号: Empty`"
        status: pass
    human_judgment: false
  - id: D9
    description: "密钥读写唯一经 prism_llm::secrets，engine 自身不持有 keyring/reqwest（NFR-03 / T-01-04a）"
    requirement: "INFRA-03"
    verification:
      - kind: unit
        ref: "facade.rs#tests::secrets_are_only_ever_delegated_to_prism_llm（生产代码：prism_llm::secrets 4 次；keyring_core / apple_native_keyring_store / reqwest:: 各 0 次）"
        status: pass
      - kind: integration
        ref: "bash scripts/check-deps.sh facade-egress → OK: prism-engine only ever reaches network/secrets through prism-llm"
        status: pass
      - kind: integration
        ref: "api_key_status 返回 bool 而非密钥原文；bash scripts/check-secrets.sh → exit 0"
        status: pass
    human_judgment: false
  - id: D10
    description: "EngineError 的 Display 不携带数据库路径、密钥原文或用户内容（T-01-20a）"
    requirement: "INFRA-03"
    verification:
      - kind: unit
        ref: "error.rs#tests::error_display_does_not_carry_a_filesystem_path"
        status: pass
      - kind: unit
        ref: "error.rs#tests::variants_forward_the_lower_layer_text_verbatim（transparent 不得在下层文本之外追加任何东西）"
        status: pass
    human_judgment: false
  - id: D11
    description: "service trait 实现为同步，不含 .await（否则 Arc<dyn> 的 object-safety 失效）"
    requirement: "INFRA-01"
    verification:
      - kind: unit
        ref: "services.rs#tests::the_service_impls_contain_no_await（生产代码 .await 0 次；两个 impl 各 1 次）"
        status: pass
      - kind: unit
        ref: "facade.rs#tests::facade_methods_are_synchronous（无 `pub async fn`）"
        status: pass
    human_judgment: false
  - id: D12
    description: "list_feedback 的空集是 Ok 不是 Err；空 project_id 是 Err 且文本不回显参数"
    requirement: "INFRA-01"
    verification:
      - kind: unit
        ref: "services.rs#tests::no_pending_feedback_is_ok_not_an_error / ::an_empty_project_id_is_rejected / ::rejection_text_does_not_echo_the_caller_argument"
        status: pass
      - kind: unit
        ref: "services.rs#tests::record_receipt_accepts_a_well_formed_receipt / ::record_receipt_rejects_a_receipt_without_a_comment_id"
        status: pass
    human_judgment: false

# Metrics
duration: 31min
completed: 2026-07-29
status: complete
---

# Phase 01 Plan 07: prism-engine 门面与 D-09 注入面 Summary

**编排层落地：单写者句柄的持有者、事件总线的唯一订阅点、D-09 注入面的实现侧三者合一，全程不依赖 tauri；六条判别性断言各配一条已实跑确认落点的反证，其中两条专门用来证明另一条断言不可省。**

## Performance

- **Duration:** ≈31 min agent 时间
- **Tasks:** 2（各按 TDD 走 RED→GREEN 两段）
- **Files created/modified:** 9（新建 5，修改 4）
- **测试增量:** workspace 87 → **107 passed / 1 ignored / 0 failed**（prism-engine 3 → 23）

## Accomplishments

- **D-09 的两侧现在真的合上了。** 01-06 用假实现证明了「注入通路通」；本 plan 把
  `Arc<Engine>`（真实类型，持着真实 SQLite 库）装进 `Arc<dyn FeedbackSource>` 注入
  `McpDeps`，起 `serve_loopback`，走完 `initialize` → `notifications/initialized` →
  `tools/call` 三步握手。`check-deps.sh no-cycle` 在 prism-engine 新增两条普通依赖之后
  依然报 `OK: prism-mcp -> prism-types only`——方向没有回退。
- **端到端注入测试的判别性没有落在「响应里有 `[]`」上。** Phase 1 的 `list_feedback`
  对合法 project 返回空 vec，而**空结果与「handler 根本没调注入的 trait」是不可区分的**。
  判别性因此落在第二次调用：空 `project_id` 时 Engine 返回它自己写的校验文本
  `invalid request: project id must not be empty`——那段字符串只可能来自 engine 侧。
  反证 CP-9 确认了落点，**并顺带确认 rmcp 对 `projectId` 没有任何兜底校验**
  （这正是 01-06 栽过的那个坑：被测层之上若有第三方 backstop，反证会被掩盖）。
- **总线的三条语义各有一条落点隔离的反证。** 尤其是 CP-2：把 `publish` 改成
  `.expect("send")` 之后**只有** `publish_with_no_subscriber_is_ok` 变红，另外四个
  集成测试全绿——因为它们都有订阅者。落点隔离说明这三条断言互不替代。
- **「等事件的测试要包 timeout」这条是设计出来的，不是防御性编程。** 不包的话，
  `publish` 被改空时 `recv().await` 会**一直等下去**，测试挂住而不是变红，反证就没有
  落点可看。CP-1 实测：三个总线测试在 **2.00s** 内落在具名的 timeout `expect` 上。
- **发现并修复了计划两条验收项在现有脚本下互斥。** 详见 Deviations 1——这不是可以
  绕过去的小事：`src-tauri` 只依赖 `prism-engine`，所以 shell 通往钥匙串的路线必然经过
  facade，「facade 不许碰 prism-llm」与「密钥唯一入口是 prism-llm」在原脚本下不可兼得。

## Task Commits

1. **Task 1: EventBus 与 Engine 门面编排** — TDD 两段：
   - `5e3b747` (test) — RED：`tests/facade.rs` 五个测试 + Cargo 依赖
     （落点：`unresolved import prism_engine::EventBus` + 6 个 `method not found`）
   - `07f7ee9` (feat) — GREEN：`bus.rs` / `error.rs` / `facade.rs` / `lib.rs` +
     `check-deps.sh` 的 facade-egress 断言，11 unit + 5 integration passed
2. **Task 2: Engine 实现服务 trait 并可注入 prism-mcp** — TDD 两段：
   - `67cff6b` (test) — RED：`engine_satisfies_service_traits`
     （落点：`the trait bound Engine: FeedbackSource is not satisfied`，CommentSink 同）
   - `07292c5` (feat) — GREEN：`services.rs` + `lib.rs` 模块聚合，17 unit + 6 integration passed

**Plan metadata:** 见本 commit（docs: complete plan）

## Files Created/Modified

### prism-engine

- `src/bus.rs`（新，95 行）— `BUS_CAPACITY = 256`、`EventBus`（`new` / `subscribe` /
  `publish` / `subscriber_count`）+ `Default`；3 个单测
- `src/facade.rs`（新，207 行）— `Engine { store: Arc<Store>, bus: EventBus }`，十个门面方法；
  三个**源码级哨兵**单测（不外泄连接句柄 / 密钥只经 prism-llm / 无 `pub async fn`），
  共用一个 `production_source()` 助手先剥掉注释行与测试模块
- `src/services.rs`（新，168 行）— `impl FeedbackSource for Engine`（空集是 `Ok` + 空 id 是 `Err`）、
  `impl CommentSink for Engine`（只记 id 与 status）；6 个单测
- `src/error.rs`（新，71 行）— `EngineError`（`#[non_exhaustive]` + 两个 transparent 变体）；2 个单测
- `src/lib.rs`（改）— 四个 `pub mod` + re-export；保留 01-01 的三个 tracer 测试
- `tests/facade.rs`（新，393 行）— 6 个集成测试 + MCP 握手助手
- `Cargo.toml`（改）— 普通依赖加 prism-llm / prism-mcp / tokio / tracing；
  dev 加 serial_test / tokio(time) / tokio-util / keyring-core / reqwest / serde_json

### workspace

- `scripts/check-deps.sh`（改）— `single-egress` 拆两条 + 新增 `facade-egress` 子命令
- `Cargo.lock`（改）

## Decisions Made

1. **端到端注入测试的判别性落在校验文本上，不落在空集上。** 见 Accomplishments 第二条。
   代价是 `list_feedback` 的边界校验必须现在就写（计划把它写成"可以顺带"）——
   但它本来就与"有没有 comments 表"无关，是这个方法永远要做的事。
2. **`bus_broadcasts_to_multiple_subscribers` 刻意不断言 `subscriber_count() == 2`。**
   写第一版时它在，跑 CP-3 时发现：把第二个订阅者换成与总线无关的接收端之后，
   **前置条件那一行会先红**，判别性断言根本没机会执行——反证落在了前置条件上。
   该前置条件移交给 `bus::tests::subscribing_twice_yields_two_independent_receivers`
   之后，CP-3 精确落在「第二个订阅者没收到」上。这是 01-05/01-06 那条教训的直接应用。
3. **源码级哨兵先剥注释行与测试模块。** 第一版 `secrets_are_only_ever_delegated_to_prism_llm`
   直接 `include_str!` 全文，结果被**文档注释里举例用的 `keyring_core`** 和**断言自己的
   字符串字面量**双重命中，恒红。`production_source()` 把 `#[cfg(test)]` 之后截断、
   过滤掉 `//` 开头的行之后才是可用的哨兵。
4. **`EngineError` 只保留两个 `transparent` 变体，并为此专门写了一条断言。**
   `variants_forward_the_lower_layer_text_verbatim` 守的不是当前状态，而是
   「有人给变体加了 `#[error("engine failed at {path}: {0}")]`」这类改动——
   那正是路径与密钥重新溜进错误文本的典型方式。
5. **`init_secrets` 沿用 `prism_llm::secrets::init_default_store` 的 `#[cfg(target_os = "macos")]` 门控。**
   不在 facade 层做 `#[cfg]` 分支包装成"跨平台空实现"——那会让 Windows 端口（P1）
   在真正接后端之前一直得到一个静默成功的密钥初始化。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] 计划的两条验收项在现有 `check-deps.sh` 下互斥**

- **Found during:** Task 1，加完 `prism-llm` 依赖后首次跑 `check-deps.sh`
- **Issue:** 计划同时要求
  (a)「密钥读写唯一经 `prism_llm::secrets`」＋「`facade.rs` 中 `prism_llm::secrets` ≥1 次」，
  (b)「`bash scripts/check-deps.sh single-egress` 退出 0」。
  实跑 `FAIL: prism-engine has network/secret dependency` —— 01-02 的
  `PURE_CRATES` 含 `prism-engine`，而该断言用 `cargo tree --edges normal --prefix none`
  看的是**整棵扁平化依赖树**，`prism-llm` 带进来的 `keyring-core` /
  `apple-native-keyring-store` / `reqwest` 三个都在里面。两条要求不可兼得。
- **调查:** 核对 `src-tauri/Cargo.toml`——shell 只依赖 `prism-engine` / `prism-types` /
  `prism-store`，**不依赖 prism-llm**。所以 shell 通往钥匙串的唯一路线必然经过 facade。
  另一个选项（把 prism-llm 直接给 shell）才是真正破坏 NFR-03「唯一入口」的那个。
  结论：不是代码错了，是断言对 facade 用错了形态。
- **Fix:** 把 NFR-03 拆成两条形态不同的断言，两条都保留牙齿：
  - `check_single_egress`（叶子 crate：prism-types / store / fs / parse / anchor）——
    **整棵普通依赖树里不得出现**这三个包。未削弱，只是移出了 prism-engine。
  - `check_facade_egress`（prism-engine 专用）——(a) 直接依赖（`--depth 1`）里没有这三个包；
    (b) 在 prism-engine 的依赖树内，用 `cargo tree --invert` 取这三个包的**反向依赖闭包**，
    其中除 `prism-llm` 与 `prism-engine` 自身外不得出现任何 `prism-*` crate。
  (b) 是真正有牙的一条：哪天 `prism-store` 或 `prism-anchor` 悄悄加了 keyring，
  它会作为一个新的 `prism-*` 名字出现在反向闭包里而被抓住——而整棵树的断言那时
  只会说「prism-engine 有密钥依赖」，与现状**无法区分**。
  `single-egress` 子命令现在同时跑两条，所以计划验收项的字面命令仍然覆盖 facade。
- **Files modified:** `scripts/check-deps.sh`
- **Verification:** `bash scripts/check-deps.sh` 五条全 OK；
  `single-egress` / `tauri-free` / `facade-egress` 单独运行均 exit 0
- **Commit:** `07f7ee9`

**2. [Rule 2 - Missing Critical] 端到端注入测试按计划写法是恒真的**

- **Found during:** Task 2 设计阶段
- **Issue:** 计划写「发一次全合法请求，**响应中出现 engine 返回的数据**」。但 Phase 1 的
  `list_feedback` 返回空 vec——响应里"engine 返回的数据"就是 `[]`，而 `[]` 同样会由
  一个完全不调注入 trait 的实现产生。这条断言对它要排除的失败模式**不敏感**。
- **Fix:** 追加第二次调用（空 `project_id`），断言响应里出现 Engine 自己写的
  `invalid request: project id must not be empty`。为此 `list_feedback` 的边界校验
  从"可以顺带"提升为必需，并配 `rejection_text_does_not_echo_the_caller_argument`
  把这段文本用 `assert_eq!` 钉死（它是端到端测试的判别依据，漂移即失效）。
- **Verification:** 反证 CP-9 落在该断言上；反证 CP-10（返回非空项）落在断言①上，
  证明 `[]` 也确实来自工具载荷而非蹭到的字符串
- **Commit:** `67cff6b`（RED）/ `07292c5`（GREEN）

**3. [Rule 2 - Missing Critical] 等事件的断言若不包 timeout，反证会挂住而不是变红**

- **Found during:** Task 1 写测试时（预判），CP-1 实跑确认
- **Issue:** `broadcast::Receiver::recv()` 在 sender 存活时会一直等。若 `publish` 被改成
  空实现，裸 `recv().await` 的测试会**挂住**——反证既不红也不绿，没有落点可看，
  正是 STATE.md 记录的那类失效模式的一个新变种。
- **Fix:** 所有 `recv` 经 `recv_within()` 包 `tokio::time::timeout(2s)`，超时落在具名 `expect` 上。
- **Verification:** CP-1 实测三个总线测试在 **2.00s** 内变红，落点均为 `facade.rs:53`
- **Commit:** `5e3b747`

**4. [Rule 2 - Missing Critical] 前置条件断言会抢走反证落点**

- **Found during:** Task 1 跑 CP-3 前
- **Issue:** 见 Decisions 2。
- **Fix:** 把 `subscriber_count() == 2` 移到 `bus.rs` 的独立单测。
- **Verification:** CP-3 现在**只**让 `bus_broadcasts_to_multiple_subscribers` 与那条
  独立单测变红，且前者落在「第二个订阅者」而非前置条件上
- **Commit:** `07f7ee9`

### 计划与仓库形态不符（无需修改）

计划 `<action>` 说 `tests/facade.rs` 属于 Task 2，但 `must_haves.artifacts` 要求该文件
提供「总线交付、无订阅者、多订阅者、trait 实现、门面委托**五组**集成测试」且 `min_lines: 70`。
按 TDD，Task 1 的五条 `<behavior>` 必须在 Task 1 的 RED 里就有测试。因此该文件在
Task 1 创建（5 个测试），Task 2 追加第 6 个。最终形态满足 must_haves（6 个测试，393 行）。

---

**Total deviations:** 4 auto-fixed（1 个 Rule 3 - 阻塞，3 个 Rule 2 - 缺失的关键性）
**Impact on plan:** 一条是计划验收项自相矛盾（必须改脚本），三条是让断言与反证真的有判别性。
计划的 `must_haves`、`artifacts`、`key_links` 与 `prohibitions` 全部满足。

## Known Stubs

**None（按「阻碍本 plan 目标达成」的口径）。**

`list_feedback` 返回空 vec 与 `record_receipt` 只记日志，是**计划明写的交付边界**
（schema v1 按 D-04 还没有 comments 表，真实落库属于 Phase 5/6）。两者都不是硬编码
占位：`list_feedback` 有真实的边界校验分支且该分支正是端到端测试的判别依据；
`record_receipt` 有真实校验 + 真实日志。四个单测覆盖两条分支各自的正反面。

`delete_api_key` 与 `EventBus::subscriber_count` 目前只被测试消费（真实读写方分别是
01-09 的 settings 页与后续 plan），但它们不是 stub——与 01-04 的 `ACCOUNT_MCP_TOKEN`
同理，是接口的前置定名。

已向 `.planning/WINDOWS.md` 登记 1 条 `deviation`（计划两条验收项互斥及其解法）。

## Threat Flags

None——本 plan 未引入计划 `<threat_model>` 之外的安全面。五条处置全部落地：

| Threat | 处置 | 证据 |
|--------|------|------|
| T-01-31（外泄连接句柄） | mitigate | `no_public_method_hands_out_a_connection`；生产代码 `-> Connection\|PooledConnection` 计数 0 |
| T-01-04a（门面回传密钥原文） | mitigate | `api_key_status` 返回 `bool`；`secrets_are_only_ever_delegated_to_prism_llm`（`keyring_core` 0 次）；`check-deps.sh facade-egress` |
| T-01-20a（错误携带路径/内容） | mitigate | `error_display_does_not_carry_a_filesystem_path` + `variants_forward_the_lower_layer_text_verbatim` |
| T-01-32（慢订阅者拖慢 publish） | accept | `BUS_CAPACITY = 256`；broadcast 丢最旧而非阻塞发送端，`Lagged` → `Resync` 由 01-08 补偿 |
| T-01-33（receipt 正文入日志） | mitigate | `record_receipt` 的 `tracing::info!` 只有 `comment_id` 与 `status` 两个字段 |

## Issues Encountered

**一个阻塞问题（Deviations 1）与三个判别性缺口（Deviations 2–4），均已解决。**

值得单独记一笔的是 Deviations 1 的**发现方式**：如果按验收项顺序先跑测试再跑
`check-deps.sh`，会在 Task 1 快结束时才撞上，且第一反应容易是「把 prism-engine 从
`PURE_CRATES` 删掉」——那会静默地削弱一条 CI 断言，而且没人会注意到削弱了什么。
真正让解法成形的是先问「这条断言想守的到底是什么」：它守的是**网络与密钥只有一个
出口**，而不是「某个 crate 的依赖树里没有某些名字」。对叶子 crate 这两者等价，
对 facade 不等价——facade 的正确形态是「只能经这一条边进来」，
而 `cargo tree --invert` 的反向闭包正好表达得了它。

其余顺利：两个 RED 均按预期以「符号/trait 不存在」失败；两个 GREEN 各一次通过。
三步 MCP 握手在 0.18s 内跑完。八条反证全部一次到位地落在预期断言上。

## Verification Evidence

```
cargo test -p prism-engine                                  → 17 unit + 6 integration passed
  bus_delivers_to_subscriber                                → exit 0（单独运行）
  publish_with_no_subscriber_is_ok                          → exit 0（单独运行）
  bus_broadcasts_to_multiple_subscribers                    → exit 0（单独运行）
  search_delegates_to_store                                 → exit 0（单独运行）
  base_url_goes_to_settings_not_keychain                    → exit 0（单独运行）
  --test facade engine_satisfies_service_traits             → exit 0
cargo test --workspace                                      → 107 passed / 1 ignored / 0 failed
npm run test -- --run                                       → 3 passed
cargo clippy -p prism-engine --all-targets -- -D warnings   → exit 0
cargo clippy --workspace --all-targets -- -D warnings       → exit 0
bash scripts/check-deps.sh                                  → 五条全 OK
bash scripts/check-deps.sh single-egress                    → exit 0
bash scripts/check-deps.sh tauri-free                       → exit 0
bash scripts/check-secrets.sh                               → exit 0

# D-01（engine 选择集，成功标准 1 第一条）
cargo test -p prism-types -p prism-store -p prism-fs -p prism-parse \
           -p prism-anchor -p prism-llm -p prism-mcp -p prism-engine   → 全绿
  grep -c 'Compiling tauri v' <该输出>                                  → 0
cargo tree -p prism-engine --edges normal,build,dev --prefix none \
  | grep -c '^tauri '                                                  → 0

# D-09（依赖方向未回退）
bash scripts/check-deps.sh no-cycle          → OK: prism-mcp -> prism-types only
cargo tree -p prism-engine --edges normal --depth 1                     → 只有 prism-* 与 thiserror/tokio/tracing
cargo tree -p prism-engine -i keyring-core --prefix none | grep '^prism-'
                                             → prism-llm / prism-engine 二者而已

# 源码形态（**生产代码**计数：去注释行、去 #[cfg(test)] 之后）
services.rs  impl FeedbackSource for Engine   → 1
services.rs  impl CommentSink for Engine      → 1
services.rs  .await                           → 0
facade.rs    prism_llm::secrets               → 4  （≥1）
facade.rs    keyring_core                     → 0
facade.rs    reqwest                          → 0
facade.rs    -> Connection | PooledConnection → 0
facade.rs    .write(|                         → 1  （key_links）
bus.rs       pub fn publish(&self, ev: EngineEvent) {   ← 返回类型为 ()，无 `->`
crates/prism-engine/src/ 下 INSERT INTO documents_fts   → 0（触发器仍是唯一同步路径）
wc -l facade.rs / tests/facade.rs             → 207 / 393（≥ min_lines 70 / 70）

# 八条反证的落点（每条都确认了红在哪一条断言，而不只是红绿）
CP-1  publish → 空实现            → facade.rs:53  `订阅者 在 2s 内没有收到事件`
                                     三个总线测试均落在此，**2.00s 变红而非挂住**；
                                     search_delegates_to_store 保持绿（落点隔离）
CP-2  publish → .expect("send")   → bus.rs:52，**只有** publish_with_no_subscriber_is_ok
                                     与 bus::tests::publish_returns_the_unit_value 变红，
                                     另四个集成测试全绿 ✔ 落点隔离
CP-3  subscribe 第2次起返回无关接收端 → facade.rs:53 `第二个订阅者 …没有收到事件`，
                                     **只有** bus_broadcasts_to_multiple_subscribers +
                                     bus.rs:100 的前置条件单测变红 ✔
CP-4  search → 硬编码 SearchHit    → facade.rs:170 **阴性对照**那条（第一条断言仍绿
                                     ——正是它证明阴性对照不可省）✔
CP-5  set_base_url 追加 set_api_key → facade.rs:220 `写 base_url 在钥匙串里留下了条目` ✔
CP-6  删掉 set_base_url 的 publish  → facade.rs:229 `…应向总线发一条失效信号: Empty` ✔
CP-9  拆掉 list_feedback 空 id 校验 → facade.rs:389 判别性断言；响应体证明
                                     **rmcp 对 projectId 无任何兜底校验**（无第三方 backstop）✔
CP-10 list_feedback 返回非空项      → facade.rs:379 断言①，证明 `[]` 来自工具载荷 ✔
恢复后 diff /tmp/{bus,facade,services}.bak → 三个文件均与反证前**逐字节一致**

# 提交未删除任何被跟踪文件
git diff --diff-filter=D --name-only HEAD~4 HEAD                        → 空
git log -1 -- crates/prism-store/migrations/001_schema_v1.sql           → 875b9c8（01-03，未动）
```

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

**已就绪，可开工的下游 plan：**

- **01-08（冒烟命令与事件总线）** — `Engine::subscribe()` 返回
  `tokio::sync::broadcast::Receiver<EngineEvent>`，shell adapter 直接 `rx.recv().await`；
  **`RecvError::Lagged` 分支必须写**，翻成 `EngineEvent::Resync` 发给前端
  （`BUS_CAPACITY = 256` 溢出时就是它）。命令层的每个 `#[tauri::command]` 应是对
  facade 方法的单行委托，且因为 facade 方法是**同步**的，命令体里要用
  `tokio::task::spawn_blocking` 包一层。
- **01-09（冒烟页）** — settings 页要的四个方法齐了：`set_base_url` / `get_setting` /
  `set_api_key` / `api_key_status`。**`api_key_status` 返回 `bool` 不返回 key**，
  前端只能显示"已配置/未配置"——这是设计不是缺功能。
- **Phase 2（导入与搜索）** — 所有写路径经 `Engine` 的新方法 → `store.write(|tx| …)`；
  facade 不提供也不会提供返回连接的方法。watcher 事件直接 `engine.publish(...)`。
- **Phase 6（MCP 工具注册）** — 在 `prism-types/src/service.rs` 追 trait、
  在 `crates/prism-engine/src/services.rs` 加 impl、在 `PrismHandler::call_tool` 分派。
  bearer 由 facade 从钥匙串 `mcp_bearer_token` 读出后经 `McpDeps::new` 注入。

**需要注意的四点：**

1. **`prism-engine` 的 `reqwest` 必须留在 dev 边。** 挪到普通依赖会让
   `check_facade_egress` 的直接依赖检查立刻变红——那正是它存在的意义。
2. **不要为了"方便"把 `prism-llm` 直接给 `src-tauri`。** shell 一旦能直接调
   `prism_llm::secrets`，NFR-03 的「唯一入口」就从"一条路"变成"两条路"，
   而 `check-deps.sh` 现有的任何一条都抓不到它（shell 不在受检集合里）。
   Phase 6 若真需要，先给 `check-deps.sh` 补一条针对 `prismdocs-shell` 的断言。
3. **facade 方法保持同步。** 加 `pub async fn` 会让
   `facade::tests::facade_methods_are_synchronous` 变红——那不是测试挡路，
   是 `std::sync::MutexGuard` 的 `!Send` 保护正在失效的信号。
4. **端到端注入测试靠 `invalid request: project id must not be empty` 这段字面量判别。**
   Phase 6 改写 `list_feedback` 时若要改这段文本，必须同步改
   `tests/facade.rs` 与 `services.rs#rejection_text_does_not_echo_the_caller_argument`，
   否则测试会绿着失去判别性。

---
*Phase: 01-foundation-skeleton*
*Completed: 2026-07-29*

## Self-Check: PASSED

- 5 个新建文件全部在盘上：`crates/prism-engine/src/{bus,error,facade,services}.rs`、
  `crates/prism-engine/tests/facade.rs`
- 4 个 commit 全部可在 `git log` 中找到：`5e3b747` / `07f7ee9` / `67cff6b` / `07292c5`
- `git diff --diff-filter=D --name-only HEAD~4 HEAD` 为空——未删除任何被跟踪文件
- `facade.rs` 207 行、`tests/facade.rs` 393 行，均 ≥ must_haves 的 `min_lines` 70
- must_haves 的 `key_links` 三条各自可 grep（生产代码）：`.write(|`（facade.rs ×1）、
  `prism_llm::secrets`（facade.rs ×4）、`impl FeedbackSource for Engine`（services.rs ×1）
- 八条反证全部实跑并逐条确认落点；复原后三个被改文件与反证前**逐字节一致**
- `cargo test --workspace` → 107 passed / 1 ignored / 0 failed；`npm run test -- --run` → 3 passed
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0
- `bash scripts/check-deps.sh` 五条全 OK；`bash scripts/check-secrets.sh` → exit 0
