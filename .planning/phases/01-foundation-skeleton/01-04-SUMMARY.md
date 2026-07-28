---
phase: 01-foundation-skeleton
plan: 04
subsystem: infra
tags: [prism-types, service-trait, object-safety, engine-event, keyring-core, macos-keychain, redaction, serial-test]

# Dependency graph
requires:
  - "01-01：prism-types / prism-llm 的最小编译单元与 [workspace.dependencies] 版本 pin"
  - "01-02：scripts/check-deps.sh（single-egress / tauri-free / no-cycle / dup）与 scripts/check-secrets.sh"
provides:
  - "prism-types 零第三方依赖（仅 serde + thiserror）契约 crate —— D-09 的 trait 落点"
  - "FeedbackSource / CommentSink 两个同步 object-safe service trait 与 ServiceError"
  - "EngineEvent 粗粒度失效信号（DocChanged / InboxUpdated / Resync），tag=kind + camelCase"
  - "FeedbackItem / Receipt / SearchHit 三个共享 DTO"
  - "prism-llm::secrets —— INFRA-03 的唯一密钥入口：SERVICE/ACCOUNT 常量 + init_default_store + set/get/delete 往返"
  - "ApiKey newtype（手写 Debug 脱敏、刻意无 Display）与不携带密钥 payload 的 LlmError::Keychain"
  - "docs/keychain-naming.md —— 跨 crate 与跨二进制的 service/account 命名契约"
affects: [01-05-settings页与base_url, 01-06-MCP-loopback宿主, 01-07-集成验证, 01-08-冒烟命令与事件总线, 01-09-冒烟页, phase-4-LLM端点与SSE, phase-6-MCP工具注册与CLI-helper]

# Tech tracking
tech-stack:
  added:
    - "thiserror 2（prism-types 的第二个也是最后一个允许依赖）"
    - "serde_json 1（prism-types dev-dependency，仅契约测试用）"
    - "serial_test 3（prism-llm dev-dependency，串行化进程级默认 store）"
  patterns:
    - "契约 crate 的依赖面由可执行命令锁死：cargo tree -p prism-types --edges normal 的白名单只有 serde 系与 proc-macro 系"
    - "service trait 一律同步：底层 rusqlite 本就阻塞，同步 trait 天然 object-safe，consumer 用 spawn_blocking 调用"
    - "object-safety 用 `Arc<dyn Trait>` 的单测做编译期证明，而不是靠 review 记得"
    - "持有密钥的类型手写 Debug 输出占位串，并**刻意不实现 Display** —— 缺席让误用变成编译错误而非运行期泄漏"
    - "第三方错误类型跨边界时只转发已核实安全的 Display 文本，不转发会打印 payload 的错误值本身"
    - "触碰进程级全局状态的测试：#[serial] + 入口处先 unset + 出口处再 unset（前一个测试 panic 时的兜底）"

key-files:
  created:
    - crates/prism-types/src/event.rs
    - crates/prism-types/src/service.rs
    - crates/prism-types/src/dto.rs
    - crates/prism-types/tests/contract.rs
    - crates/prism-llm/src/secrets.rs
    - docs/keychain-naming.md
  modified:
    - crates/prism-types/src/lib.rs
    - crates/prism-types/Cargo.toml
    - crates/prism-llm/src/lib.rs
    - crates/prism-llm/Cargo.toml
    - Cargo.lock

key-decisions:
  - "LlmError::Keyring(keyring_core::Error) 改为 Keychain(String)：keyring_core::Error 的 derive Debug 会打印 BadEncoding/BadDataFormat 携带的原始密钥字节，而 unwrap()/expect()/tracing 的 ?err 走的正是 Debug"
  - "ApiKey 只实现 Debug（脱敏），刻意不实现 Display —— 让 format!(\"{key}\") 与 tracing 的 %key 编译失败"
  - "prism-types 的公开契约测试放在 tests/contract.rs（外部视角），object-safety 证明留在 lib.rs 单测内（内部视角）"
  - "移除 01-01 的占位类型 CrateInfo：真实 DTO 已接手「证明 serde derive 进树」的职责"
  - "不勾选 INFRA-01 与 INFRA-03：两者分别还有 5 个和 3 个后续 plan 未完成，提前标完成会是虚假信号"

patterns-established:
  - "契约 crate 新增依赖前必须先问：它会同时压到 prism-mcp 与 prism-engine 两侧吗"
  - "Phase 6/7 新增 MCP 工具时在 crates/prism-types/src/service.rs 追加 trait，不动 prism-mcp"
  - "任何跨二进制的字符串常量（钥匙串命名、端口约定等）必须同时落一份 docs/ 契约文档"

requirements-completed: []

coverage:
  - id: D1
    description: "prism-types 是零第三方依赖的契约 crate：normal 依赖树中只有 serde 系与 proc-macro 系"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "cargo tree -p prism-types --edges normal --prefix none | tail -n +2 | grep -cvE '^(serde|serde_core|serde_derive|serde_json|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident)( |$)' → 0"
        status: pass
      - kind: integration
        ref: "crates/prism-types/Cargo.toml 的 [dependencies] 只有 serde 与 thiserror 两项"
        status: pass
    human_judgment: false
  - id: D2
    description: "service trait 同步且 object-safe：Arc<dyn FeedbackSource> / Arc<dyn CommentSink> 可直接构造并调用"
    requirement: "INFRA-01"
    verification:
      - kind: unit
        ref: "crates/prism-types/tests/contract.rs#service_traits_are_object_safe_behind_arc"
        status: pass
      - kind: unit
        ref: "crates/prism-types/src/lib.rs#tests::feedback_source_can_be_used_behind_arc_dyn"
        status: pass
      - kind: integration
        ref: "grep -c 'async fn\\|async_trait' crates/prism-types/src/service.rs → 0"
        status: pass
    human_judgment: false
  - id: D3
    description: "EngineEvent 的序列化形态即 IPC 契约：tag 为 kind，变体与字段均 camelCase，Resync 无载荷"
    requirement: "INFRA-01"
    verification:
      - kind: unit
        ref: "contract.rs#doc_changed_serialises_with_camel_case_tag_and_fields / #resync_serialises_as_a_bare_tagged_object / #inbox_updated_carries_only_an_id_and_a_count / #engine_event_is_clonable_and_round_trips"
        status: pass
    human_judgment: false
  - id: D4
    description: "API key 经 keyring 写入后可读回，且无 key 时返回 Ok(None) 而非 Err"
    requirement: "INFRA-03"
    verification:
      - kind: unit
        ref: "crates/prism-llm/src/secrets.rs#tests::roundtrip_with_mock_store / #tests::no_key_is_not_an_error"
        status: pass
      - kind: manual_procedural
        ref: "cargo test -p prism-llm -- --ignored roundtrip_with_real_keychain（真实登录钥匙串往返，需人工在授权弹窗上放行）"
        status: pending
    human_judgment: true
    rationale: "真实钥匙串往返会触发系统授权弹窗，且 dev 期签名身份变化会反复重弹；自动化只能覆盖到 keyring-core 的 store 抽象层，跨不过 macOS 的用户授权。mock store 覆盖全部逻辑分支，真实后端的存在性由 keychain feature 的编译期 gate 保证。"
  - id: D5
    description: "写与删都是幂等的：重复写同值不改变读回值也不产生第二条条目；重复删第二次仍 Ok(())"
    requirement: "INFRA-03"
    verification:
      - kind: unit
        ref: "crates/prism-llm/src/secrets.rs#tests::set_and_delete_are_idempotent"
        status: pass
    human_judgment: false
  - id: D6
    description: "密钥不经 Debug/Display/错误信息泄漏"
    requirement: "INFRA-03"
    verification:
      - kind: unit
        ref: "crates/prism-llm/src/secrets.rs#tests::apikey_debug_is_redacted"
        status: pass
      - kind: unit
        ref: "crates/prism-llm/src/secrets.rs#tests::keychain_errors_are_flattened_to_their_display_text"
        status: pass
      - kind: integration
        ref: "bash scripts/check-secrets.sh → exit 0"
        status: pass
    human_judgment: false
  - id: D7
    description: "无明文回退路径：钥匙串失败一律冒泡，源码中不存在环境变量/dotfile 取密钥的分支"
    requirement: "INFRA-03"
    verification:
      - kind: integration
        ref: "grep -c 'std::env::var' 与 grep -c 'dotenv' 于 crates/prism-llm/src/secrets.rs → 均为 0"
        status: pass
    human_judgment: false
  - id: D8
    description: "选用 keychain 模块而非需要 provisioning profile 的模块（避开运行期 -34018）"
    requirement: "INFRA-03"
    verification:
      - kind: integration
        ref: "cargo tree -p prism-llm -e features | grep 'apple-native-keyring-store feature \"keychain\"' 命中；且该模块在 crate 内被 #[cfg(feature = \"keychain\")] 门控，编译通过即为启用证据"
        status: pass
      - kind: integration
        ref: "grep -c 'keychain::Store::new' secrets.rs → 1；grep -c 'protected' secrets.rs → 0"
        status: pass
    human_judgment: false
  - id: D9
    description: "keyring 测试在默认多线程 harness 下不互相污染进程级 default store"
    requirement: "INFRA-03"
    verification:
      - kind: integration
        ref: "cargo test -p prism-llm 连续 3 次全部 exit 0（9 passed / 1 ignored）；cargo test --workspace 亦全绿"
        status: pass
      - kind: integration
        ref: "unset_default_store 在非注释行出现 6 次 ≥ #[serial] 测试数 4"
        status: pass
    human_judgment: false
  - id: D10
    description: "prism-llm 仍是唯一持有网络/密钥依赖的 engine crate（新增 keyring 用法未扩散）"
    requirement: "INFRA-03"
    verification:
      - kind: integration
        ref: "bash scripts/check-deps.sh → 四条断言全 OK（dup / tauri-free / no-cycle / single-egress）"
        status: pass
    human_judgment: false

# Metrics
duration: 7min
completed: 2026-07-28
status: complete
---

# Phase 1 Plan 04: prism-types 契约 crate 与 prism-llm 密钥入口 Summary

**依赖图的汇点（零第三方依赖的 prism-types，两个同步 object-safe service trait）与密钥的唯一入口（prism-llm 的 macOS 钥匙串往返 + 三层 redaction）同时落地，两者的关键性质都由可复现命令而非评审意见守住。**

## Performance

- **Duration:** ≈7 min agent 时间（23:19→23:26 UTC）
- **Tasks:** 2（各按 TDD 走 RED→GREEN 两段）
- **Files modified:** 11（新建 6，修改 5）

## Accomplishments

- **prism-types 从占位单元变成真正的契约汇点**：三个模块（`event` / `service` / `dto`）、两个同步 trait、一个事件枚举、三个 DTO；`[dependencies]` 严格只有 serde 与 thiserror，整棵 normal 依赖树里除 serde 系与 proc-macro 系外**一个包都没有**。
- **同步 trait 的选择被编译期锁死**：`Arc<dyn FeedbackSource>` 与 `Arc<dyn CommentSink>` 各有一个单测在构造并调用。若将来有人把方法改成 `async fn` 或加泛型参数，编译在这里就断，而不是等到 Phase 6 在 rmcp handler 里发现 trait 不能 dyn 化。
- **`EngineEvent` 的载荷契约保持小**：只带 ID 与计数，不带文档正文。`Resync` 变体是 notify-then-fetch 相对 push-payload 的关键差异——总线 `Lagged` 丢消息时可以用一次全量失效无损补偿。
- **密钥的三层 redaction 全部有测试**：`ApiKey` 的 Debug 输出占位串且**没有 Display**（缺席让误用成为编译错误）；`LlmError` 不再转发 `keyring_core::Error` 的错误值本身；`get_api_key` 只把 `NoEntry` 映射为 `Ok(None)`，源码里 `std::env::var` 与 `dotenv` 各出现 0 次。
- **钥匙串命名契约落成文档**：`docs/keychain-naming.md` 记录 `PrismDocs` / `llm_api_key` / `mcp_bearer_token` 三个字符串、各自的写入方与读取方，以及「`prismdocs-helper` 因 D-10 无法 `use` 这些常量、必须自带一份字面量副本」这一事实——这正是文档而非常量才能承载的约束。

## Task Commits

1. **Task 1: prism-types 零依赖契约** — TDD 两段：
   - `a0b0552` (test) — RED：`tests/contract.rs` 7 个失败测试 + Cargo.toml 依赖调整（`unresolved imports … no ServiceError in the root`）
   - `b1db5df` (feat) — GREEN：`event.rs` / `service.rs` / `dto.rs` / `lib.rs` 聚合，9 passed
2. **Task 2: prism-llm 密钥入口** — TDD 两段：
   - `c79f0ca` (test) — RED：`secrets.rs` 只含测试模块 + serial_test dev-依赖（23 个 `cannot find … in this scope`）
   - `caf7fd8` (feat) — GREEN：常量、`init_default_store`、set/get/delete、`ApiKey`、`LlmError` 加固、`docs/keychain-naming.md`，9 passed / 1 ignored

**Plan metadata:** 见本 commit（docs: complete plan）

## Files Created/Modified

### prism-types

- `src/event.rs`（新）— `EngineEvent`：`DocChanged{project_id,doc_id}` / `InboxUpdated{project_id,unread}` / `Resync`；`#[serde(tag="kind", rename_all="camelCase", rename_all_fields="camelCase")]`
- `src/service.rs`（新）— `FeedbackSource::list_feedback` / `CommentSink::record_receipt`，均为同步且 `: Send + Sync + 'static`；`ServiceError{NotFound, Backend(String), Invalid(String)}`，Display 不回显调用参数
- `src/dto.rs`（新）— `FeedbackItem{id,doc_id,body}` / `Receipt{comment_id,status}` / `SearchHit{doc_id,title,rel_path}`
- `src/lib.rs`（改）— 模块聚合 + re-export，保留 `CRATE_VERSION`，`#[cfg(test)]` 内的 object-safety 证明；移除占位类型 `CrateInfo`
- `tests/contract.rs`（新）— 7 个从外部视角写的公开契约测试
- `Cargo.toml`（改）— `[dependencies]` 加 thiserror（连同 serde 共两项），`[dev-dependencies]` 加 serde_json

### prism-llm

- `src/secrets.rs`（新，约 230 行含测试）— 三个契约常量、`init_default_store`（`#[cfg(target_os="macos")]`）、`set_api_key` / `get_api_key` / `delete_api_key`、`ApiKey` newtype 与手写 Debug；5 个自动化测试 + 1 个 `#[ignore]` 真实钥匙串测试
- `src/lib.rs`（改）— `pub mod secrets;`；`LlmError` 加 `#[non_exhaustive]`，`Keyring(#[from] keyring_core::Error)` → `Keychain(String)` + 手写 `From`
- `Cargo.toml`（改）— `[dev-dependencies]` 加 serial_test

### 文档

- `docs/keychain-naming.md`（新）— 命名表、模块选择理由、三条不变量、更名警示、手动验证步骤

## Decisions Made

1. **`LlmError` 不再转发 `keyring_core::Error` 的错误值，只保留其 Display 文本。**
   核查了 `keyring_core::Error` 的 `Display` 实现：`BadEncoding(_)` 只写 "Password data is not valid UTF-8"，
   `BadDataFormat(_, err)` 只写底层错误——Display 是安全的。但该枚举的 `Debug` 是 derive 出来的，
   `BadEncoding(Vec<u8>)` 与 `BadDataFormat(Vec<u8>, _)` 会把**原始密钥字节**打出来，而
   `unwrap()` / `expect()` / `tracing` 的 `?err` 走的正是 `Debug`。`keychain_errors_are_flattened_to_their_display_text`
   用精确的 `assert_eq!` 锁死扁平化后的形态（而不是只断言「不含某子串」——那种断言挡不住字节数组形式的泄漏）。
2. **`ApiKey` 只实现 `Debug`，刻意不实现 `Display`。** Debug 的存在是必须的（否则 `#[derive(Debug)]`
   的容器类型无法编译）；Display 的**缺席**是主动防御——`format!("{key}")`、`println!("{key}")`
   与 `tracing` 的 `%key` 会直接编译失败。取原文只有 `expose()` 一条路，调用点因此在代码搜索中可见。
3. **契约测试放 `tests/contract.rs`，object-safety 证明留在 `lib.rs` 单测内。** 前者只经 `prism_types::…`
   的公开路径断言——契约 crate 的全部价值就是它对外的形状，测试也该从外部视角写；后者按计划要求
   留在同文件内，作为「trait 改坏了立刻编译失败」的最近距离哨兵。
4. **移除 `CrateInfo`。** 它是 01-01 为「证明 serde derive 真的被链接」而写的占位结构，本 plan 的三个
   真实 DTO 已完全接手这一职责。全仓库无其他引用（`prism-mcp` / `prism-engine` 用的是 `CRATE_VERSION`，保留）。
5. **`set_and_delete_are_idempotent` 额外覆盖了「轮换」路径。** 计划只要求「连续两次写同值」，
   但真实使用中更常见的是用新 key 覆盖旧 key；测试同时断言覆写后读回新值**且条目数仍为 1**，
   免得幂等被实现成「第二次写入被忽略」这种同样能通过原断言的错误语义。
6. **不勾选 INFRA-01 与 INFRA-03。** INFRA-01 还欠 01-06/01-07/01-08/01-09 的事件总线与 Channel 有序流；
   INFRA-03 还欠 01-05 的 settings 页与 base_url 配置、01-07/01-09 的集成验证。沿用 01-01 建立的口径：
   `requirements-completed: []`，REQUIREMENTS.md 保持 Pending。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] 计划的依赖白名单正则漏了 `serde_core`**

- **Found during:** Task 1 验收
- **Issue:** 验收项给出的命令
  `cargo tree -p prism-types --edges normal --prefix none | tail -n +2 | grep -cvE '^(serde|serde_derive|serde_json|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident)( |$)'`
  实际输出 **1** 而非 0，唯一的「意外」包是 `serde_core v1.0.229`。
- **调查:** 核对 `serde-1.0.229/Cargo.toml`——`serde_core` 是 serde 自己拆出的内部 crate，
  依赖声明为 `version = "=1.0.229"`（锁步同版本），由 serde 上游发布。它属于 must_haves truth
  所说的「serde 系」，是白名单漏项，不是依赖面污染。
- **Fix:** 验收改用补上 `serde_core` 的白名单，输出为 0。**未改动任何代码**——prism-types
  的 `[dependencies]` 依旧只有 serde 与 thiserror。
- **Files modified:** 无
- **Commit:** 不适用（纯验证命令修正）

**2. [Rule 2 - Security] `LlmError` 的 keyring 变体会经 Debug 泄漏密钥原文**

- **Found during:** Task 2 GREEN
- **Issue:** 01-01 建立的 `LlmError::Keyring(#[from] keyring_core::Error)` 直接转发第三方错误值。
  该枚举的 derive `Debug` 会打印 `BadEncoding(Vec<u8>)` / `BadDataFormat(Vec<u8>, _)` 携带的
  **原始密钥字节**，而 `unwrap()` / `tracing` 的 `?err` 走的正是 Debug。这与计划明写的
  「`LlmError` 的任何变体都不得携带密钥原文」（T-01-04）直接冲突。
- **Fix:** 变体改为 `Keychain(String)`，手写 `From<keyring_core::Error>` 只保留已核实安全的
  Display 文本；同时加 `#[non_exhaustive]`（与 `StoreError` 对齐）。新增测试
  `keychain_errors_are_flattened_to_their_display_text` 以精确 `assert_eq!` 锁死形态。
- **Files modified:** `crates/prism-llm/src/lib.rs`、`crates/prism-llm/src/secrets.rs`
- **Commit:** `caf7fd8`

### 计划与仓库形态不符（无需修改，记录以免后续 plan 复现困惑）

验收项「`crates/prism-llm/Cargo.toml` 中 `apple-native-keyring-store` 一行包含 `features = ["keychain"]`」
按字面读会失败：本仓库自 01-01 起统一用 `[workspace.dependencies]` 集中管理版本与 feature 串，
crate 侧一律写 `{ workspace = true }`。feature 实际声明在根 `Cargo.toml` 第 30 行，等价证据有两条：

- `cargo tree -p prism-llm -e features` 显示 `apple-native-keyring-store feature "keychain"`
- 该模块在 crate 内被 `#[cfg(all(target_os = "macos", feature = "keychain"))]` 门控，
  `secrets.rs` 里 `apple_native_keyring_store::keychain::Store::new()` 能编译本身即为启用证据

## Known Stubs

**None.** 本 plan 交付的全部符号都有真实实现与测试。

`ACCOUNT_MCP_TOKEN` 常量目前无读写方（Phase 6 的 MCP bearer token 才使用），但它不是 stub——
它是**契约的一部分**：与 `llm_api_key` 同时定名是为了让 Phase 6 的 CLI helper 有一个已经写死、
已经进文档的目标，而不是到那时才现编一个名字。

`.planning/WINDOWS.md` 不存在，本 plan 也无需向其登记条目（无 stub、无 skipped test、
无未跑的 `<verify>`——真实钥匙串往返是计划**设计为**人工的验证门，不是漏跑）。

## Issues Encountered

**None.** 两个 RED 均按预期编译失败，两个 GREEN 均一次通过；`cargo test -p prism-llm`
连跑 3 次无 flake，clippy 首次即无告警。

## Verification Evidence

```
cargo test -p prism-types                                   → 9 passed（2 unit + 7 contract）
cargo test -p prism-llm                                     → 9 passed / 1 ignored
cargo test -p prism-llm   ×3 连续                            → exit 0 / 0 / 0（无 flake）
cargo test --workspace                                      → 58 passed / 1 ignored / 0 failed
npm run test -- --run                                       → 3 passed
cargo build --workspace                                     → exit 0
cargo clippy --workspace --all-targets -- -D warnings       → exit 0
bash scripts/check-deps.sh                                  → dup / tauri-free / no-cycle / single-egress 四条全 OK
bash scripts/check-secrets.sh                               → exit 0

# prism-types 依赖面（补上 serde_core 后的白名单）
cargo tree -p prism-types --edges normal --prefix none | tail -n +2 \
  | grep -cvE '^(serde|serde_core|serde_derive|serde_json|thiserror|thiserror-impl|proc-macro2|quote|syn|unicode-ident)( |$)'
                                                            → 0
sed -n '/^\[dependencies\]/,/^$/p' crates/prism-types/Cargo.toml
                                                            → 只有 serde 与 thiserror
grep -c 'async fn\|async_trait' crates/prism-types/src/service.rs
                                                            → 0

# prism-llm 密钥入口形态
grep -c 'keychain::Store::new'  crates/prism-llm/src/secrets.rs   → 1
grep -c 'protected'             crates/prism-llm/src/secrets.rs   → 0
grep -c 'impl std::fmt::Debug for ApiKey' …/secrets.rs            → 1
grep -c 'std::env::var'         crates/prism-llm/src/secrets.rs   → 0
grep -c 'dotenv'                crates/prism-llm/src/secrets.rs   → 0
grep -v '^//' …/secrets.rs | grep -c 'unset_default_store'        → 6（≥ #[serial] 测试数 4）
cargo tree -p prism-llm -e features | grep apple-native-keyring-store
                                                            → feature "keychain" 在列

# 文档契约
grep -c 'PrismDocs|llm_api_key|mcp_bearer_token' docs/keychain-naming.md → 7 / 2 / 2

# 提交未删除任何被跟踪文件
git diff --diff-filter=D --name-only <每个 commit>~1 <commit>     → 全部为空
```

## Self-Check

见文末 `## Self-Check` 段。

## User Setup Required

**一项人工验证门（计划中的 `<human-check>`，非阻塞后续 plan）：**

```bash
cargo test -p prism-llm -- --ignored roundtrip_with_real_keychain
```

系统弹出钥匙串授权框时选「允许」。测试自带清理，跑完可在「钥匙串访问」中确认
`PrismDocs` 条目已被删除。

未由 agent 自动执行是刻意的：它会触发 macOS 的用户授权弹窗，且 dev 期签名身份变化会反复重弹——
`#[ignore]` 的存在就是为了让自动化不碰它（RESEARCH Pitfall 1）。自动化侧用
`keyring_core::mock::Store` 覆盖了全部逻辑分支；真实后端的存在性由 `keychain` feature 的
编译期 gate 保证。此验证同时也会在 01-09 的 settings 页人工冒烟中被间接覆盖。

## Next Phase Readiness

**已就绪，可开工的下游 plan：**

- **01-05（settings 页与 base_url）** — `set_api_key` / `get_api_key` / `delete_api_key` 三个函数
  即命令层要委托的对象；`init_default_store()` 需在 shell 启动序列中调用**一次且在任何 Entry 之前**。
  非密钥配置（base_url）按 D-05 走 `settings` 表，**不要**为了「方便」把它也塞钥匙串。
- **01-06（MCP loopback 宿主）** — `prism-types` 的 `Arc<dyn FeedbackSource>` / `Arc<dyn CommentSink>`
  就是 `StreamableHttpService::new` 的 factory 闭包要 clone 的东西；假数据 handler 用
  `FeedbackItem` / `Receipt` 即可。bearer token 的钥匙串 account 名已定为 `mcp_bearer_token`。
- **01-08（冒烟命令与事件总线）** — `EngineEvent` 的三个变体已就位，`Resync` 专供
  `broadcast::error::RecvError::Lagged` 分支；`EngineEvent` 是 `Clone`，可直接作
  `broadcast::Sender<EngineEvent>` 的载荷。
- **01-09（冒烟页）** — 前端的判别联合按 `{ kind: 'docChanged', projectId, docId }` /
  `{ kind: 'inboxUpdated', projectId, unread }` / `{ kind: 'resync' }` 三种形态写即可，
  序列化形态已有单测锁死。

**需要注意的三点：**

1. **`prism-types` 加依赖前先想清楚。** 它是汇点，任何新增依赖会同时压到 `prism-mcp` 与
   `prism-engine` 两侧。目前的两项（serde、thiserror）就是上限。
2. **`prismdocs-helper` 无法 `use prism_llm::secrets::SERVICE`**（D-10 禁止它链接任何 `prism-*`）。
   Phase 6 写 CLI helper 时必须自己写一份同值字面量，并与 `docs/keychain-naming.md` 对表。
3. **`ACCOUNT_MCP_TOKEN` 已定名但尚无读写方。** 这是有意的前置定名，Phase 6 直接用，不要改名。

---
*Phase: 01-foundation-skeleton*
*Completed: 2026-07-28*

## Self-Check

**PASSED**

- 6 个新建文件全部存在于工作树：`crates/prism-types/src/{event,service,dto}.rs`、
  `crates/prism-types/tests/contract.rs`、`crates/prism-llm/src/secrets.rs`、`docs/keychain-naming.md`
- 4 个 commit 全部可在 `git log` 中找到：`a0b0552`、`b1db5df`、`c79f0ca`、`caf7fd8`
- `git diff --diff-filter=D --name-only <commit>~1 <commit>` 对四个 commit 均为空 —— 未删除任何被跟踪文件
- `crates/prism-llm/src/secrets.rs` 240 行（含测试），超过 must_haves 要求的 min_lines 60
