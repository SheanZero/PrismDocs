---
phase: 01-foundation-skeleton
plan: 13
subsystem: infrastructure
tags: [csp, webview-hardening, asset-protocol, tracing-subscriber, observability, dependency-assertion, gap-closure, supply-chain, counterproof, T-01-55, T-01-56, T-01-57, T-01-58, T-01-59, T-01-60, T-01-61]
status: complete

# Dependency graph
requires:
  - "01-02：`scripts/check-deps.sh` 的结构范式（`set -euo pipefail`、长文件头写明「为什么这条断言存在 + 本文件是唯一实现」、具名 `check_*` 函数 + `main()` 分派 `${1:-all}` + `usage:` exit 2），以及**herestring 喂 grep 不得改管道**这条本 phase 的既有教训"
  - "01-02 / 01-09：`TAURI_FREE_CRATES`（八个 engine crate + prism-cli）这个受检集合的定义与它存在的理由"
  - "01-03：`crates/prism-store/src/open.rs` 的 `migration_runs_before_the_read_pool_is_built` —— 本仓库「用 `include_str!` 断言自身源码次序」的范式原型"
  - "01-09：`src/lib/capabilities.test.ts` —— 静态 import JSON 而非 `fs.existsSync` 的手法（省掉 `@types/node`，且文件被删时同时炸在 `tsc --noEmit` 与 vitest 两处），本 plan 的 `tauri-security.test.ts` 逐字照它"
  - "01-09：`src-tauri/capabilities/default.json` 当前授予的权限里没有资源协议相关项——这是关闭 assetProtocol 的前提之一"
  - "01-REVIEW.md § CR-02（171-213）与 01-VERIFICATION.md § WR-04（357-394）：两条 pre-phase-2 项的完整问题陈述"
  - "01-PATTERNS.md § `tracing-subscriber` init：7 处发射点的清单（5 个 crate）与「subscriber 必须只是 src-tauri 的依赖」这条约束"
provides:
  - "`app.security.csp` —— 发布形态的严格内容安全策略（7 条指令，含 `default-src 'self'` / `script-src 'self'` / `object-src 'none'`）"
  - "`app.security.devCsp` —— 开发形态，与 `csp` 同构，只在 `script-src` 加 `'unsafe-inline'`、`connect-src` 加 `ws://localhost:1420` 与 `http://localhost:1420`"
  - "`assetProtocol.enable: false` + 空 scope，且 `src-tauri/Cargo.toml` 的 tauri feature 列表里不再有 `protocol-asset`——配置与 feature 两半一起关"
  - "`src/lib/tauri-security.test.ts` —— 把 CSP 形态与 assetProtocol 关闭状态钉成断言的前端测试"
  - "`prismdocs_shell::init_tracing() -> bool` —— 全局 tracing subscriber 的唯一安装点，`try_init` 语义，`RUST_LOG` 可覆盖，默认档 `info`"
  - "`prismdocs_shell` 的两条测试：`tracing_init_installs_a_global_subscriber_and_is_idempotent`、`run_installs_tracing_before_it_builds_the_app`"
  - "`bash scripts/check-deps.sh subscriber-free` —— 第七条依赖方向断言，已纳入 `all`（零调用点改动即成为 CI 与 justfile 的闸门）"
  - "workspace 里 7 处 `tracing` 发射点第一次真的有落点"
affects: [phase-3-外部-Markdown-渲染, phase-4-Mermaid-与-LLM-chat-client, phase-5-本地图片渲染, phase-6-MCP-与-externalBin-公证, 全部后续-phase-的新-tracing-发射点]

# Tech tracking
tech-stack:
  added:
    - "tracing-subscriber 0.3（features `[\"env-filter\"]`）—— 根 `[workspace.dependencies]` 一处 pin，`src-tauri` 一处消费，engine 侧零消费"
  patterns:
    - "安全配置的「两半」形态：`assetProtocol.enable` 与 cargo 的 `protocol-asset` feature 是配套的两半，只关一半等于没关。同理 Phase 4/5 真要用时两半一起开并配具体 scope"
    - "CSP 分 `csp` / `devCsp` 两份：让「为开发方便而放宽」在结构上**不可能**误伤发布形态，而不是靠记得改回来"
    - "静默失效型配置需要专门的钉子测试：`csp` 回到 `null` 不会让任何东西报错，只是安静地把 WebView 敞开——这是一次代码评审最容易放过的 diff（与 capabilities.test.ts 同一类）"
    - "源码序断言的锚点必须是**完整语句**而不是裸名字：`include_str!` 的匹配面里同时含代码与注释，一条提到 `tauri::Builder` 的解释性注释就能让断言在实现正确时变红（本 plan 实测撞上）"
    - "「壳专属依赖」是一个会漏且不以编译错误呈现的性质，必须由 `cargo tree` 断言看住而不是注释约定"
    - "日志 sink 本身是新增的外泄面：装 subscriber 这个动作把此前写向虚无的调用变成落盘内容，默认档位越窄，「日志里到底会出现什么」的不确定性越少"

key-files:
  created:
    - src/lib/tauri-security.test.ts
  modified:
    - src-tauri/tauri.conf.json
    - src-tauri/Cargo.toml
    - src-tauri/src/lib.rs
    - Cargo.toml
    - Cargo.lock
    - scripts/check-deps.sh
    - .planning/phases/01-foundation-skeleton/01-RESEARCH.md

key-decisions:
  - "01-13: CSP 做成 `csp` / `devCsp` 两份而不是一份放宽版——两处放宽（`script-src` 的 `'unsafe-inline'`、`connect-src` 的 Vite HMR 来源）只进 dev 那一份，发布形态一个字不改。Tauri v2 在 `devCsp` 缺席时会退回用 `csp`，所以这一份不是可选项"
  - "01-13: `csp` 刻意**不含** `asset:`——资源协议已在下面关掉，留着它是自相矛盾的声明；将来两半一起开时再一起加"
  - "01-13: `style-src` 必须带 `'unsafe-inline'`（Vite 以 `<style>` 元素注入 CSS），`connect-src` 必须带 `ipc:` 与 `http://ipc.localhost`（Tauri v2 自有命令通道的来源，去掉它所有 `invoke` 被拦）——两处都是非显然的必需项，不是宽松"
  - "01-13: 关闭 assetProtocol 时同步移除 `src-tauri/Cargo.toml` 的 `protocol-asset` feature。`cargo build -p prismdocs-shell` 仍绿即为「没有任何代码路径依赖资源协议」的证明"
  - "01-13: `init_tracing()` 用 `try_init()` 而非 `init()`——后者在全局 dispatcher 已就位时 panic，「装日志这件事本身把应用弄崩」是最不该发生的失败模式。返回 bool 让调用方与测试都能分辨「这次装上了」与「早就装好了」"
  - "01-13: 默认档取 `info` 且**不**给 prism_mcp 单开 `debug`（01-REVIEW.md 的修复建议里写的是 `\"info,prism_mcp=debug\"`）。核对过 `middleware.rs:36-39`：`deny(reason: &'static str)` 是 `warn!` 且 reason 为编译期常量、不含请求内容，`info` 已覆盖它与另外两条 `warn!`。少开一个 target 的 debug，就少一份「日志里会出现什么」的不确定性（T-01-58）"
  - "01-13: `init_tracing()` 的返回值在 `run()` 里显式丢弃——装不上不该阻断启动，与 `lib.rs` 既有的「钥匙串后端注册失败不阻断启动」同一口径"
  - "01-13: 源码序断言的两个锚点改用完整语句（`let _ = init_tracing();` / `tauri::Builder::default()`）。实测发现裸名字 `\"tauri::Builder\"` 会命中 `run()` 里那条解释性注释里的同名字样——它排在真正的调用之前，使断言在实现完全正确时变红。这是 open.rs 范式的一条补充：`include_str!` 的匹配面含注释"
  - "01-13: `subscriber-free` 的受检集合用 `TAURI_FREE_CRATES`（含 prism-cli）而不是 `ENGINE_CRATES`——prism-cli 将来作为 externalBin 单独签名公证，它悄悄链上一个日志栈同样是要到 Phase 6 才炸的那类问题，纳入成本为零"
  - "01-13: `subscriber-free` 只看 `--edges normal`：dev-dependencies 里出现 subscriber 是合理的（测试可以自己装一个），这条断言守的是普通依赖边。反证 S 因此必须注入 `[dependencies]` 而非 `[dev-dependencies]`（01-VERIFICATION.md § SC-1 记录过这个陷阱）"
  - "01-13: 不给 `subscriber-free` 加 justfile recipe 与 CI 步骤——它已纳入 `all`，而 justfile 的 `check-all` 与 CI 的依赖断言步骤跑的都是 `bash scripts/check-deps.sh all`，因此零调用点改动即成为闸门。同时避开了与同波次 plan 01-11 在 justfile / ci.yml 上的文件冲突。这是决定，不是遗漏"
  - "01-13: 包合法性闸门（Task 2）不可自动放行，即使 `workflow.auto_advance` 为真——缺失的审计行是一个执行器无法自行确立的事实，不是人可以橡皮图章的验证步骤"

patterns-established:
  - "配置类安全项的钉子测试形态：静态 import JSON + 断言「非空」而不只是「含某串」（`csp` 非空字符串这条就是「不许回到 null」本身）+ 断言否定面（`script-src` 内无通配符来源、无整协议放行、无字符串求值关键字）"
  - "源码序断言的锚点选取规则：取只有语句本身能命中的完整形态；锚点过宽的失败方向是**假红**（安全），锚点过窄的风险是命中不到而 `expect` 炸出明确信息——两者都不会静默恒绿"
  - "新增壳专属依赖的三步：根 Cargo.toml 一处 pin + 唯一消费方一处 `workspace = true` + 一条把「唯一」变成机制的 cargo tree 断言"

requirements-completed: [INFRA-01]

# Metrics
duration: 14min
completed: 2026-07-29
tasks_total: 3
tasks_completed: 3
files_changed: 7
commits: 3
---

# Phase 01 Plan 13: WebView 内容安全策略 + 日志 sink 落地 Summary

给薄 shell 补上两件「现在零成本、进 Phase 2 之后越来越贵」的事：WebView 从「无策略 + 开着一个零消费方的资源协议」变成「发布/开发双份 CSP + 两半一起关闭的资源协议」，workspace 里 7 处此前写向虚无的 `tracing` 发射点第一次有了落点，且「subscriber 只属于壳」由一条会真的变红的 `cargo tree` 断言看住。

## What Was Built

### Task 1 — WebView 内容安全策略 + 关闭资源协议（commit `7f431fd`，上一会话完成）

`src-tauri/tauri.conf.json` 的 `app.security` 从两个键变成三个：

- **`csp`（发布形态）**：`default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' ipc: http://ipc.localhost; object-src 'none'; base-uri 'self'`
- **`devCsp`（开发形态）**：与上面同构，只在两处放宽——`script-src` 追加 `'unsafe-inline'`（Vite dev 注入内联引导脚本），`connect-src` 追加 `ws://localhost:1420` 与 `http://localhost:1420`（HMR）
- **`assetProtocol`**：`enable` 由 `true` 改为 `false`，`scope` 保持空数组

同时 `src-tauri/Cargo.toml` 的 tauri feature 列表移除 `protocol-asset`——配置侧与 feature 侧是配套的两半，只关一半等于没关。`cargo build -p prismdocs-shell` 移除后仍绿，即为「没有任何代码路径依赖资源协议」的证明。

新建 `src/lib/tauri-security.test.ts`（74 行），形状照 `capabilities.test.ts`：静态 import JSON，断言 `csp` 非空字符串、含 `default-src 'self'` 与 `script-src 'self'`、`script-src` 内无通配符来源与整协议放行、无字符串求值关键字；`devCsp` 非空且同样含 `default-src 'self'`；`assetProtocol.enable === false` 且 `scope` 深等于 `[]`。

### Task 2 — 包合法性闸门（commit `213aaa8`）

`gate="blocking-human"` 的供应链闸门，不可自动放行。`01-RESEARCH.md` 的 `## Package Legitimacy Audit` 表建于 phase 规划期、早于本次 gap-closure 运行，表中没有 `tracing-subscriber` 这一行；未经审计的包一律按 `[ASSUMED]` 处理。

人工于 2026-07-29 在 crates.io 上逐项核对后回复 "approved"：

| 核对项 | 结果 |
|---|---|
| 名字逐字 | `tracing-subscriber`（连字符、单数 `subscriber`；非 `tracing-subscribers` / `tracing_subscriber` 等形近名） |
| 仓库 | `https://github.com/tokio-rs/tracing` —— 与本 workspace 已在用的 `tracing` 同仓库同 owner |
| 首发 | 2019-06-27 |
| 下载量 | 累计 ~523M，90 天 ~128M（约 10M/wk） |
| 最高稳定版 | 0.3.23（2026-03-13），未被 yank |
| 许可 / feature | MIT；`env-filter` 存在 |

审计行与一段补录说明已写回 `01-RESEARCH.md`，下一次运行不必重问。

### Task 3 — 壳里装上 subscriber + 依赖断言（commit `8b03c4b`，TDD）

- 根 `Cargo.toml`：`tracing-subscriber = { version = "0.3", features = ["env-filter"] }`，旁注说明它只属于壳、约束由 `check-deps.sh` 看住
- `src-tauri/Cargo.toml`：唯一消费点 `tracing-subscriber = { workspace = true }`
- `src-tauri/src/lib.rs`：`const DEFAULT_LOG_FILTER: &str = "info"` + `pub fn init_tracing() -> bool`（`EnvFilter::try_from_default_env()` 失败回落默认档，收尾 `.try_init().is_ok()`），并作为 `run()` 的第一条语句调用、返回值显式丢弃
- `scripts/check-deps.sh`：新增 `check_subscriber_free()`，受检集合 `TAURI_FREE_CRATES`，`--edges normal`，herestring 喂 grep；`subscriber-free` 进 `case` 分派、`usage:` 串与 `all`

## Key Implementation Details

**为什么 `init_tracing` 必须在 `tauri::Builder` 之前。** 装在 `Builder` 之后就漏掉了 `AppState::bootstrap()` 的失败与 `lib.rs:41` 那条「钥匙串不可用」降级提示——而那两条正是 WR-04 点名「无处可去」的日志。这个顺序没有任何运行期兜底，因此配了一条源码序断言。

**判别性落在哪。** 装上 subscriber 这件事没有行为面的观察点：7 处发射点在没有 subscriber 时照常编译、照常执行、照常返回，只是写向虚无。于是 `tracing_init_installs_a_global_subscriber_and_is_idempotent` 的判别性全部落在第 ② 条 `tracing::dispatcher::has_been_set()` 上——反证 T 验证了这一点。按本 phase 已记录的教训，刻意**不写**「调用前 `has_been_set()` 为 false」这条前置断言（全局 dispatcher 是进程级的，前置条件会把反证的落点从被守的断言上移开）。

**默认档为什么是 `info` 而不是 `"info,prism_mcp=debug"`。** 01-REVIEW.md 的修复建议给的是后者并提醒确认「把 MCP 拒绝原因写进本地日志是否有意」。核对 `middleware.rs:36-39` 后确认 `deny(reason: &'static str)` 用的是 `warn!` 且 reason 是编译期字符串常量、不含任何请求内容，`info` 已经覆盖它，也覆盖 `settings.rs` 的明文 http 告警与 `lib.rs` 的钥匙串降级提示（都是 `warn!`）。少开一个 target 的 debug 就少一份 T-01-58 的暴露面。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] 源码序断言的锚点过宽，在实现完全正确时变红**

- **Found during:** Task 3 的 GREEN 步（首次跑 `cargo test -p prismdocs-shell --features test`）
- **Issue:** `run_installs_tracing_before_it_builds_the_app` 按计划写法用 `body.find("tauri::Builder")` 取锚点。`run()` 内我为解释「为什么必须是第一步」写了一条注释，其中含 `tauri::Builder` 字样且**排在真正的调用之前**——`include_str!` 的匹配面同时含代码与注释，于是 `builder_at` 命中注释、断言在实现正确时判定为「init 在 Builder 之后」。测试红，落点是被守的那条断言，但原因是锚点而非实现
- **Fix:** 两个锚点都收窄为完整语句：`let _ = init_tracing();` 与 `tauri::Builder::default()`。收窄后反证 U（把调用移进 `.setup()` 闭包）仍然变红，判别力未受损。理由写进测试的文档注释，作为 `open.rs` 那条范式的补充
- **Files modified:** `src-tauri/src/lib.rs`
- **Commit:** `8b03c4b`

这条属于本 phase 反复出现的那族问题（「反证本身需要被验证」）的一个新变种：这次是**正证**在实现正确时变红，方向相反但根因同类——断言的匹配面比设想的宽。值得记住的是它的失败方向是安全的（假红而非静默恒绿）。

### 计划内但需说明的偏差

**Task 3 的 TDD 三段未拆成三个 commit。** RED 已实际执行并观察到（新增两条测试后 `cargo test` 报 `error[E0425]: cannot find function 'init_tracing' in this scope`，两处），但 RED 状态是**编译失败**，单独提交会在历史里留下一个不可构建的 commit。因此 Task 3 按 plan 的任务原子性合成一个 `feat` commit，RED 的证据记录在此。REFACTOR 段无独立改动（锚点修正已计入上面的 Rule 1 偏差）。

## Counterproofs

五条反证全部实跑，**逐条记录落点**而非只记红绿：

| 反证 | 注入 | 结果 | 落点 |
|---|---|---|---|
| Q（csp 钉子） | `csp` 改回 `null` | 红 | 「csp 是非空字符串」那一条 |
| R（assetProtocol 钉子） | `enable` 改回 `true` | 红 | assetProtocol 那一条 |
| S（依赖断言） | `crates/prism-store/Cargo.toml` 的 `[dependencies]` 加 `tracing-subscriber` | exit 1 | `FAIL: prism-store depends on tracing-subscriber` |
| T（安装断言） | `init_tracing()` 函数体换成裸 `true` | 红 | `lib.rs:103` 第 ② 条 `has_been_set()` —— 不是 ① 也不是 ③，正是设计时指定的判别点 |
| U（顺序断言） | `init_tracing()` 调用移进 `.setup()` 闭包 | 红 | `lib.rs:151` 顺序断言 |

Q 与 R 由 Task 1 在上一会话完成并记录；S / T / U 在本次会话实跑。三条撤销后各自转绿，`git status` 干净。

反证 S 特别注意：注入点必须是 `[dependencies]` 而不是 `[dev-dependencies]`——dev 边不进 `--edges normal`，把它注入 dev 段会让反证「成功地什么都没证明」（01-VERIFICATION.md § SC-1 记录过这个陷阱）。

## Verification Evidence

| 检查 | 结果 |
|---|---|
| `cargo test -p prismdocs-shell --features test` | 0 —— lib 13 passed（既有 11 + 新增 2）、ipc 2 passed |
| `cargo test --workspace` | 0 —— 27 个 `test result: ok`，零 FAILED |
| `cargo test -p prism-types … -p prism-engine`（engine 选择集） | 0 —— D-01 仍成立 |
| `cargo clippy -p prismdocs-shell --all-targets -- -D warnings` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `bash scripts/check-deps.sh subscriber-free` | 0，打印 `OK: all checked crates are tracing-subscriber-free (engine set + CLI helper)` |
| `bash scripts/check-deps.sh all` | 0，7 行 `OK:`，第 7 行即 subscriber 那条（证明真的进了 `all`） |
| `bash scripts/check-deps.sh nonsense-subcommand` | 2，usage 串含 `subscriber-free` |
| `npm run test -- --run` | 0 —— 7 files / 34 tests passed |
| `npx tsc --noEmit` | 0 |
| `npm run build` | 0 |

源码断言：`tracing-subscriber` 在全仓 `Cargo.toml` 中只出现两处（根 `[workspace.dependencies]` 与 `src-tauri`），`crates/` 下零命中；`init_tracing` 用的是 `try_init` 而非 `init`。三个 commit 均无文件删除。

## Outstanding Human Verification

**Task 1 的 `<human-check>` 五步尚未执行**，按 `workflow.human_verify_mode: end-of-phase` 顺延至 phase 收尾的人工验证。这不是可以跳过的项——**CSP 只在真实 WebView 里生效，jsdom 与 `cargo test` 都看不见它**，本 phase 已反复踩到「把被测性质放进一个没有替身的链路里跑一次」这条。待办：

1. `npm run tauri dev` 启动，窗口正常出现且不是白屏
2. 设置页完整渲染：API key 状态行、端点输入框、两个按钮
3. dev 冒烟页三个验证入口：① 总线事件往返计数 1:1；② Channel 有序流跑完并显示「seq 校验通过」；③ 中文搜索命中 > 0、阴性对照词返回 0
4. WebView 开发者工具 Console 无 `Content Security Policy` 违规报告
5. `npm run tauri build` 产出的 dmg 安装后启动，重复 2–3 步（发布形态走 `csp` 而非 `devCsp`，这是唯一能验证严格那一份的路径）

若第 4 步出现违规：只放宽 `devCsp`（dev 侧违规），或按违规报告点名的那条指令逐项追加来源到 `csp`（打包侧违规），并逐条记录追加了什么、为什么。**不要用「先设成 null 回头再收」绕过**——那正是 CR-02 的起点。

**Task 3 的行为断言（可在上面第 1 步顺带完成）**：`npm run tauri dev` 的终端里应能看到 tracing 格式的行；把 base_url 设成一个非 loopback 的 `http://` 端点，`settings.rs` 那条明文 http 告警应实际出现在终端——这是「sink 不再是空的」最直接的证据。目前 `has_been_set()` 断言证明了 dispatcher 就位，尚未有端到端「日志真的打出来了」的人工确认。

## Requirements

- **INFRA-01** —— 已于此前 plan 勾选，本 plan 继续巩固（薄 shell 的可观测性与 WebView 加固）
- **INFRA-03 仍不勾**，沿用 01-09 / 01-10 / 01-11 的同一判据：密钥存取（写入侧、证据侧）两半均已关闭，剩余阻塞只有需求文本的「支持 Anthropic/OpenAI 兼容端点」半句——要到 Phase 4 有 chat client 时才成立。本 plan 未触及该半句

## Threat Model Closure

| Threat ID | 处置 |
|---|---|
| T-01-55（`csp: null` → 任意来源脚本 + 全部 IPC 命令面） | 已缓解：严格 `csp` + 独立 `devCsp` + 钉子测试 + 反证 Q |
| T-01-56（零消费方的 assetProtocol 本地文件读取面） | 已缓解：配置侧与 cargo feature 侧两半一起关 + 反证 R |
| T-01-57（三条安全决策的日志写向不存在的 sink） | 已缓解：`run()` 第一步安装，`has_been_set()` 断言 + 源码序断言，各自可变红 |
| T-01-58（新建的 sink 本身成为外泄面） | 已缓解：默认档只到 `info`、不给 prism_mcp 单开 debug；prohibition 已登记供后续 phase 的新发射点继承 |
| T-01-59（subscriber 悄悄进入 engine 依赖树） | 已缓解：`subscriber-free` 具名断言纳入 `all` + 反证 S |
| T-01-60（新依赖未经合法性审计） | 已缓解：Task 2 的 blocking-human 闸门已由人完成，审计行写回 RESEARCH.md |
| T-01-61（过严 CSP 让打包形态白屏） | **部分缓解**：钉子测试与自动验证已就位，但覆盖 dev 与 dmg 两形态的 `<human-check>` 五步仍待执行（见 Outstanding Human Verification） |

## Known Stubs

无。本 plan 未引入任何占位实现、硬编码空值或未接线的数据源。

## Prohibitions

**INFRA-01 / privacy（本 plan 新登记，供后续 phase 继承）：** 不得让本 plan 新装的日志 sink 记录文档正文、API key 或 MCP bearer token。这个 sink 是本次改动**新创造**的外泄面——在它存在之前，那些 `tracing` 调用写向虚无。现有 7 处发射点已按脱敏口径写过（`deny` 只发 `&'static str` 的 reason、`ApiKey` 与 `McpDeps` 都手写 Debug 且刻意不实现 Display），但**从此以后每一个新的 `tracing::` 调用点都要自己承担这条约束**。

## Commits

| Commit | Task | 内容 |
|---|---|---|
| `7f431fd` | 1 | `feat(01-13)`: 双份 CSP + 关闭资源协议（配置 + cargo feature 两半）+ 钉子测试 |
| `213aaa8` | 2 | `docs(01-13)`: `tracing-subscriber` 的人工合法性审计行写回 01-RESEARCH.md |
| `8b03c4b` | 3 | `feat(01-13)`: `init_tracing()` + 两条测试 + `subscriber-free` 依赖断言 |

## Self-Check: PASSED
