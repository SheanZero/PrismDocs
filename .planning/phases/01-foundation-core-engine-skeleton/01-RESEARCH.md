# Phase 1: Foundation & Core Engine Skeleton - Research

**Researched:** 2026-07-28
**Domain:** Rust desktop-app skeleton — Tauri v2 shell + shell-agnostic Cargo workspace core, SQLite/WAL sidecar store, OS-keychain-backed LLM client boundary, loopback streamable-HTTP MCP server with token + Origin enforcement
**Confidence:** HIGH on the load-bearing items (rmcp transport surface, MCP spec Origin semantics, keyring 4 API, rusqlite/WAL/migration compatibility); MEDIUM on typed-IPC tooling maturity and CI shape

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**整体架构分层**

- **D-01:** `core` 拆成 **Cargo workspace 多 crate**（如 `prism-store` / `prism-llm` / `prism-mcp`，为 Phase 3/4 的 `prism-anchor` / `prism-lens` 预留位置），`src-tauri` 只做薄 command 层。理由：边界由编译器强制而非约定；Phase 3 的锚定引擎（护城河 + AC-3b CI 门）必须能脱离 shell 被对抗语料单测/CLI 驱动。
- **D-02:** `core` 对外暴露**单一 Engine facade**，持有共享 DB handle；Tauri command 层与 MCP server 都只是它的薄包装，不各自直连 store。理由：一个引擎两扇门，业务逻辑不重写、两边行为不漂移。
- **D-03:** **AGENT-03 的写面限制由 facade 结构性执行** —— 写操作只在 facade 暴露给 MCP 的那个子集里（评论回执）。agent 不能创建/删除评论与卡片这条约束落在类型/接口层，而非运行时 if 判断。

**运行时架构 ADR（SC-5 必须落章）**

- **D-04:** **先全 Rust 原生**：`rmcp` 做 MCP、`async-openai` 做 OpenAI 兼容端点、薄 `reqwest` 手写 Anthropic SSE。零 Node runtime → 最小包体、单工具链、密钥不出 Rust 进程。
- **D-05:** **Anthropic 客户端隔离在 trait 后**，保留 Node sidecar 降级退路。若手写 SSE 实际痛苦（边缘 case、精确 token 计数），只换实现不动上层。不预先支付双套成本。
- **D-06:** SC-5 要求这个决策**记录成正式 ADR 文件**（不是只写在 CONTEXT 里）。ADR 需说明 Rust-native 选择、被拒的 Node sidecar 方案及其触发降级的条件。

**MCP 传输与回环边界**

- **D-07:** **App 自托回环 HTTP**：resident app 自己在 `127.0.0.1:<port>` 起 `rmcp` streamable-HTTP 服务，per-install token + Origin 校验。理由：字面命中 AGENT-03 的「127.0.0.1 + token + Origin」措辞（stdio 套不上 Origin 语义），且 app 本身即 MCP server，直接接 Engine facade —— 无子进程、无二次回环 IPC。
  - **注意：** 这与 `.claude/CLAUDE.md` 技术栈文档里「rmcp over **stdio**」的写法相冲突，本决策**覆盖**该处描述。planner 应据此更新技术栈文档或在 ADR 中标注。
- **D-08:** **剪藏 WS bridge（Phase 6 F6）合用同一回环服务**，走不同路由。一个回环监听面同时服务两个需求。Phase 1 只立骨架，不实现剪藏协议。
- **D-09:** MCP server **随 app 常驻**（app 起即起）。
- **D-10:** **固定默认端口，被占则向上扫**；onboarding 把实际 port + token 回写进项目的 `.mcp.json` / 协议片段。token 本体 per-install 存**系统钥匙串**，不进 git（`.prismdocs/` 建议 gitignore）。理由：与 F4「一键生成 CLAUDE.md/AGENTS.md 协议片段」一体，也最好调试。

**存储位置与备份模型**

- **D-11:** sidecar SQLite 存 **`~/Library/Application Support/PrismDocs/<项目路径 keyed>/`**，不放进用户仓库。理由：源仓库全程干净、零 git 噪声、不泄露私人评论（与 CLAUDE.md "What NOT to Use" 一致）。
- **D-12:** NFR-02「单目录可备份」靠**显式的「备份/导出本项目数据」功能**满足（把 app-support 那份打包），而非靠把 DB 放进项目里。
- **D-13:** 项目被移动/重命名后靠 **project-id 映射存活**（项目侧留标记 → app-support 目录），不因路径变化丢数据。具体形式由 planner 决定。
- **D-14:** SQLite 开 **WAL 模式**（MCP reader 与 app writer 不互相阻塞）。

**首次运行引导与 LLM provider**

- **D-15:** **双族 provider 首发都要**：Anthropic + OpenAI 兼容（自定义 `base_url`，`async-openai` 覆盖代理/本地模型/长尾兼容端点）。
- **D-16:** onboarding 做**一次轻量真实连接测试**（如 count_tokens / models / 极小 completion），把「钥匙串 → reqwest → 用户端点」整条路提前跑通。理由：LLM 边界是隐性风险点（SSE、错误处理、base_url 兼容性），骨架期用一次廉价测试摧实，比推到 Phase 4 才爆强。这同时字面验证 SC-1 / SC-3 / NFR-04。
- **D-17:** **完整四步引导**：LLM 配置 → workspace 注册 → `.prismdocs/` 初始化 → MCP 协议片段。理由自洽于 D-07/D-10：既然 MCP 走固定端口 + 回写配置，协议片段必须在 Phase 1 存在，否则回环端点无从验证 SC-4。
- **D-18:** 引导第二步的 workspace 注册**顺手只读枚举一遍默认 glob 下的 MD 文件**做可见反馈。**硬边界：只枚举，不解析 frontmatter、不建 FS watcher、不写入 documents 表。** glob 配置化、`.html` 转换、异常处理、增量同步、重命名识别全部仍归 Phase 2 F1 —— 本阶段不得实现，避免返工。
- **D-19:** `.prismdocs/` 初始化包含骨架目录 + 自动生成的英文 `README.md`（向 agent 解释协议）+ `.gitignore` 建议（PRD §4.1）。feedback/context 的实际内容语义分别归 Phase 5 / Phase 7。

### Claude's Discretion

用户明确交给我和下游 agent 拍板的部分：

- SQLite schema 广度（骨架期建多少表 / 哪些 stub）、migration 框架细节
- typed IPC 契约的具体形式（命令命名、序列化、错误类型）
- 错误处理与重试语义、数据完整性保障（NFR 可靠性）
- 测试策略与 TDD 在 Rust + Tauri 上的落地形式、CI 门的构成
- 项目 re-key（移动/重命名存活）的具体机制

### Deferred Ideas (OUT OF SCOPE)

- **剪藏 WS bridge 的实际协议实现** — Phase 6（F6）。Phase 1 只保证回环服务能容纳它（D-08）。
- **MCP 工具的具体实现**（`list_feedback` / `get_feedback` / `respond_to_comment` / `get_document_comments` / `get_context_pack` / `list_cards` / `export_okf_bundle`）— 分散在 Phase 5 / Phase 6 / Phase 7。Phase 1 只立传输层与安全边界骨架。
- **文档导入的全部真实逻辑**（glob 配置化、`.html` → Markdown、frontmatter 解析与 round-trip、FS watcher + 2s debounce、重命名按内容哈希识别）— Phase 2 F1。Phase 1 的枚举严格只读（D-18）。
- **`.prismdocs/feedback/` 与 `context/` 的内容语义** — 分别 Phase 5 / Phase 7。Phase 1 只建目录骨架与 README（D-19）。
- **Q1 Lens 模型路由**（快速模型打底 + 需决策段落强模型复核）— Phase 4 决议。Phase 1 的 LLM trait 设计应不阻碍多模型路由。
- **Q3 评论/卡片 git 同步「便携模式」** — 标记为 P1，MVP 走 sidecar + 导出备份（D-11/D-12 已按此定）。
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| **AGENT-03** | MCP 安全——仅本地回环、仅暴露当前 Workspace、写操作限于评论回执（agent 不能创建/删除评论与卡片）；per-install token + Origin 校验 (PRD §4.2) | `## MCP Loopback Security` — rmcp 2.2's `StreamableHttpServerConfig.allowed_hosts` / `allowed_origins` give Host + Origin validation natively with documented 403/400 status codes (§ AC-testable). Token auth is **not** in rmcp — must be an axum middleware layer (`## Pattern 3`). Workspace scoping + write-surface limit are type-level facade concerns (`## Pattern 2`). |
| **NFR-02** | 本地优先——断网可读/可评/可写卡（LLM 功能除外）；数据库单目录可备份（SQLite + 文件） (PRD §5) | `## SQLite Sidecar Store` — rusqlite 0.40 `bundled` (statically links SQLite, no system dep, no network), WAL enabled durably via `pragma_update_and_check`. **WAL means the backup unit is 3 files** (`.db`, `.db-wal`, `.db-shm`) — see Pitfall 4 for the safe-backup command. |
| **NFR-03** | 隐私——文档内容仅发送到用户配置的 LLM 端点；无遥测默认开启（埋点 opt-in）；剪藏与文档不经我方服务器 (PRD §5) | `## LLM Client Boundary` — one `reqwest::Client` confined to `prism-llm`, all egress through a user-supplied `base_url`. Enforceable structurally (`## Pattern 5`) and testable in CI (`## Validation Architecture` REQ NFR-03 row). |
| **NFR-04** | 密钥——API key 存系统钥匙串；支持自定义 base_url（兼容代理/本地模型/OpenAI 兼容端点） (PRD §5) | `## Secrets & Keychain` — `keyring` 4.1.5 `Entry::new(service, user)` → macOS Keychain Services generic-password item. Two entries needed (LLM key + per-install MCP token, D-10). `tauri-plugin-stronghold` correctly avoided; `tauri-plugin-keyring` is stale — use the crate directly from core. |
</phase_requirements>

## Summary

This phase is a **greenfield Rust scaffold**, and almost all of its risk is concentrated in four version-and-API-shape questions that the planner would otherwise guess wrong. The good news, verified against pinned sources this session: **rmcp 2.2.0 already ships exactly the transport D-07 needs** — a tower/axum-mountable `StreamableHttpService` whose config carries first-class `allowed_hosts` and `allowed_origins` DNS-rebinding validation, returning **403 Forbidden** on a disallowed Host or Origin and **400 Bad Request** on a malformed one. Those status codes make SC-4 mechanically testable rather than merely assertable. `ProtocolVersion::LATEST` in rmcp 2.2 is already `2025-11-25`, which is the revision CLAUDE.md asks us to pin — no extra work.

The bad news: **`.claude/CLAUDE.md` § Technology Stack is materially stale on versions.** Seven of the pinned crates have moved a major or effectively-major version since that doc was written — `rusqlite` 0.32 → **0.40**, `rusqlite_migration` 1 → **2.6**, `keyring` 3 → **4.1.5**, `reqwest` 0.12 → **0.13**, `async-openai` 0.27 → **0.41**, `tiktoken-rs` 0.6 → **0.12**, `notify-debouncer-full` 0.4 → **0.7**. Two of these are not cosmetic: `rusqlite_migration` 2.6 **requires** `rusqlite ^0.40` (so the 0.32 pin is unbuildable against it), and the whole tree — rmcp 2.2, async-openai 0.41, tauri 2.11 — now converges on **`reqwest` 0.13**, so pinning 0.12 would drag two TLS stacks into the binary. The workspace MSRV is driven to **1.95** by `rusqlite_migration` 2.6, not the 1.85 the doc states.

The single highest-value non-obvious finding is about **D-02 + D-14 interacting badly**: rusqlite's `Connection` is `Send` but **`!Sync`**. A facade that literally "holds a shared DB handle" as `Arc<Mutex<Connection>>` serializes every read behind every write and makes WAL pointless — the exact opposite of D-14's stated goal. The facade must hold a **pool** (`r2d2_sqlite` 0.35, the only pool crate currently on `rusqlite ^0.40`), and because rusqlite is blocking while the rest of the stack is Tokio-async, every DB call reached from an async context must go through `spawn_blocking`.

**Primary recommendation:** Scaffold a root Cargo workspace with `src-tauri` as a member alongside `prism-core` (facade) / `prism-store` / `prism-llm` / `prism-mcp`; make the core take its data directory, its keychain accessor, and its clock as **injected parameters** (never calling Tauri's `app_data_dir()`, which needs an `AppHandle` and would break D-01); mount `rmcp`'s `StreamableHttpService` under an axum router bound to `127.0.0.1` with a bearer-token middleware **in front of** it, set `allowed_origins` non-empty so Origin validation actually engages, and prove the whole LLM boundary in onboarding with a zero-token `GET /v1/models` call against both provider families.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| SQLite schema, migrations, queries | Rust core (`prism-store`) | — | D-01: must be unit-testable with no shell. Owns the pool and the WAL pragma. |
| Engine facade / business logic | Rust core (`prism-core`) | — | D-02: one engine, two doors. Both the Tauri command layer and the MCP server are thin wrappers. |
| Keychain read/write | Rust core (`prism-llm` or a `prism-secrets` seam) | Tauri shell (first-run prompt UX only) | NFR-04. `keyring` is a plain crate with no Tauri dependency; keeping it in core lets `prism-llm` fetch the key without an `AppHandle`. |
| LLM HTTP egress (Anthropic SSE + OpenAI-compatible) | Rust core (`prism-llm`) | — | NFR-03: single egress choke point makes "content only goes to the user's endpoint" a structural property, not a policy. |
| MCP transport + auth middleware | Rust core (`prism-mcp`) | Tauri shell (spawns the listener task at startup, D-09) | AGENT-03. The listener is `axum` + `tokio`, neither of which needs Tauri. |
| Loopback port selection + lifecycle | Tauri shell | Rust core (`prism-mcp` provides `serve(listener, engine)`) | D-09/D-10: port scan and "app is resident" are shell lifecycle concerns; the serving logic is not. |
| App-support directory resolution | Tauri shell | Rust core (accepts an injected `PathBuf`) | Tauri's `PathResolver::app_data_dir()` requires an `AppHandle` — the core must not depend on it (see Pitfall 6). |
| Typed IPC contract (commands + error type) | Tauri shell (`src-tauri`) ↔ Webview | — | The command layer is by definition the shell boundary. Generated TS types land in the webview build. |
| First-run onboarding UI (4 steps, D-17) | Webview (React) | Tauri shell (commands), core (the actual work) | Presentation only; every step's effect is a core operation. |
| Project-id marker file + re-key mapping | Rust core | Tauri shell (picks the workspace directory) | D-13. Pure filesystem + registry logic; must be testable headless. |

## Standard Stack

> **Every version below was verified against crates.io on 2026-07-28.** Where it differs from `.claude/CLAUDE.md` § Technology Stack, the CLAUDE.md value is stale and the table's "CLAUDE.md says" column records the conflict for the planner to reconcile (see `## Doc Corrections Required`).

### Core

| Crate | Verified version | CLAUDE.md says | Purpose | Why standard |
|-------|------------------|----------------|---------|--------------|
| `tauri` | **2.11.5** (2026-07-01) | 2.10.1 | Desktop shell | Locked by CLAUDE.md; 2.11.5 is same-major, drop-in. [VERIFIED: crates.io] |
| `rmcp` | **2.2.0** (2026-07-08) | 2.2.0 ✓ | MCP server (streamable HTTP) | Official `modelcontextprotocol/rust-sdk`. `ProtocolVersion::LATEST == V_2025_11_25` — the pin CLAUDE.md wants is already the default. [VERIFIED: crates.io + crate source] |
| `rusqlite` | **0.40.1** (2026-06-06) | 0.32.x ❌ | Sidecar store | `bundled` feature statically links SQLite. **0.32 is incompatible with `rusqlite_migration` 2.6.** [VERIFIED: crates.io] |
| `rusqlite_migration` | **2.6.0** (2026-05-28) | 1.x ❌ | Schema migrations | Requires `rusqlite ^0.40`, **MSRV 1.95** (drives the whole workspace). Tracks version in SQLite `user_version` — no migrations table. [VERIFIED: crates.io deps API] |
| `r2d2_sqlite` + `r2d2` | **0.35.0** / 0.8.10 | *(absent)* | Connection pool | **Required, not optional** — see Pitfall 1. 0.35 is the only pool crate on `rusqlite ^0.40`. [VERIFIED: crates.io deps API] |
| `keyring` | **4.1.5** (2026-07-14) | 3 ❌ | OS keychain (NFR-04) | MSRV 1.88, edition 2024. Default `v1` feature gives the classic `Entry` API on macOS Keychain Services. [VERIFIED: crate source `src/v1.rs`] |
| `reqwest` | **0.13.4** (2026-05-25) | 0.12.x ❌ | HTTP client | rmcp 2.2 depends on `reqwest 0.13.2`; async-openai 0.41 and tauri 2.11 both on `^0.13`. Pinning 0.12 forks the tree. [VERIFIED: rmcp Cargo.toml + crates.io deps] |
| `async-openai` | **0.41.1** (2026-06-18) | 0.27.x ❌ | OpenAI-compatible client (D-15) | `with_config` supports custom `base_url`. Already depends on `eventsource-stream ^0.2`. [VERIFIED: crates.io deps API] |
| `axum` | **0.8.9** (2026-04-14) | *(absent)* | Loopback HTTP host | rmcp ships a `tower_service::Service`, **not** a server. axum 0.8 (http 1, tower-service 0.3) is what the official rmcp example uses. [VERIFIED: rmcp example source] |
| `tokio` | **1.53.1** | 1.x ✓ | Async runtime | Required by rmcp, reqwest, axum. |

### Supporting

| Crate | Verified version | Purpose | When to use |
|-------|------------------|---------|-------------|
| `eventsource-stream` | 0.2.3 | SSE frame parsing for the hand-rolled Anthropic client (D-04/D-05) | Anthropic streaming only; async-openai already handles its own. |
| `tiktoken-rs` | **0.12.0** (CLAUDE.md says 0.6 ❌) | Local token estimate for OpenAI-family | Phase 4 cost display (NFR-05). Phase 1 may skip entirely. |
| `thiserror` | 2.0.19 | Typed error enums across crate boundaries | Every core crate's `Error`; the IPC error type serializes from it. |
| `serde` / `serde_json` | 1.x / 1.0.151 | Wire types | IPC + `.mcp.json` generation. |
| `tracing` + `tracing-subscriber` | 0.1.44 | Structured logging | **Load-bearing for NFR-04's "never logged"** — see Pattern 5. rmcp already emits `tracing::warn!` on rejected Host/Origin, which is free evidence for SC-4. |
| `dirs` | 6.0.0 | Shell-agnostic app-support path | Lets `prism-store` compute `~/Library/Application Support/PrismDocs` without an `AppHandle` (Pitfall 6). |
| `uuid` | 1.x (`v4`) | Project-id marker (D-13) + per-install MCP token | Already in the tree via rmcp's `server-side-http`. |
| `rand` | 0.9.x | CSPRNG for the per-install token | Use `rand::rngs::OsRng` — **not** a `Uuid::new_v4()` as a security token (see Security Domain). |
| `tempfile` | 3.27.0 | `TempDir` for store tests | Every `prism-store` test opens a DB in a temp dir. |
| `tauri-specta` + `specta` | **2.0.0-rc.25** (2026-05-08) | Typed IPC codegen | See "Alternatives Considered" — still RC after 2 years. |

### Alternatives Considered

| Instead of | Could use | Tradeoff |
|------------|-----------|----------|
| `r2d2_sqlite` 0.35 | `deadpool-sqlite` 0.13 or `tokio-rusqlite` 0.7 | **Both are hard-blocked today:** deadpool-sqlite needs `rusqlite ^0.38`, tokio-rusqlite needs `^0.37`. Because `libsqlite3-sys` declares a `links = "sqlite3"` key, Cargo **refuses** two versions in one graph — this is a compile error, not a silent duplicate. Revisit when they bump. |
| `tauri-specta` (typed IPC) | Hand-written mirrored TS types + a `justfile`/`build.rs` check | tauri-specta 2.0 has been in `-rc` since 2024-08 (25 RCs; latest 2026-05-08, 93k downloads). It is the de-facto standard and actively released, but **it is not GA and there is no stable-tagged release for Tauri v2.** Because D-Discretion hands this to us, the safe call is: use tauri-specta, but keep the IPC surface small (≤10 commands in Phase 1) and commit the generated `bindings.ts` so an RC break never blocks a build. |
| `tauri-specta` | `ts-rs` | `ts-rs` is GA and stable but only derives types — it does not derive the command *signatures* or the typed `invoke` wrapper, which is most of the value. |
| `keyring` crate directly | `tauri-plugin-keyring` | **Do not.** Last published 0.1.0 on 2024-12-23, no repository URL on crates.io. Also wrong per D-01: it would put secret access behind the shell. [VERIFIED: crates.io] |
| `keyring` crate directly | `tauri-plugin-stronghold` | Already correctly rejected in CLAUDE.md (deprecation slated for Tauri v3). Keep the rejection. |
| axum | `hyper` directly | rmcp's `nest_service` example is axum; axum gives `.nest_service()`, `middleware::from_fn`, and the Phase-6 WS route (D-08) for free. No reason to hand-roll. |

**Installation (workspace root `Cargo.toml`, `[workspace.dependencies]`):**

```toml
[workspace]
resolver = "2"
members  = ["src-tauri", "crates/prism-core", "crates/prism-store", "crates/prism-llm", "crates/prism-mcp"]

[workspace.package]
edition      = "2021"
rust-version = "1.95"          # driven by rusqlite_migration 2.6

[workspace.dependencies]
tauri              = { version = "2.11", features = [] }
rmcp               = { version = "2.2", features = ["server", "macros", "transport-streamable-http-server"] }
rusqlite           = { version = "0.40", features = ["bundled"] }
rusqlite_migration = "2.6"
r2d2               = "0.8"
r2d2_sqlite        = "0.35"
keyring            = "4.1"
reqwest            = { version = "0.13", default-features = false, features = ["json", "stream", "rustls-tls"] }
async-openai       = "0.41"
eventsource-stream = "0.2"
axum               = "0.8"
tokio              = { version = "1", features = ["full"] }
tower              = "0.5"
http               = "1"
serde              = { version = "1", features = ["derive"] }
serde_json         = "1"
thiserror          = "2"
tracing            = "0.1"
uuid               = { version = "1", features = ["v4"] }
rand               = "0.9"
dirs               = "6"

[dev-dependencies]        # per-crate
tempfile = "3"
```

**Version verification** (run these before writing the manifest — they are cheap and the numbers above will drift):

```bash
cargo search rmcp rusqlite rusqlite_migration keyring reqwest async-openai axum r2d2_sqlite
cargo tree -d          # MUST print nothing for libsqlite3-sys and reqwest
```

## Package Legitimacy Audit

Every crate below was checked via `gsd-tools query package-legitimacy check --ecosystem crates` on 2026-07-28.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| `rmcp` | crates.io | 1.4 yr | 685k/wk | github.com/modelcontextprotocol/rust-sdk | OK | Approved |
| `rusqlite` | crates.io | 11 yr | 2.1M/wk | github.com/rusqlite/rusqlite | OK | Approved |
| `rusqlite_migration` | crates.io | 5.7 yr | 202k/wk | github.com/cljoly/rusqlite_migration | OK | Approved |
| `keyring` | crates.io | 10 yr | 548k/wk | github.com/open-source-cooperative/keyring-rs | OK | Approved |
| `reqwest` | crates.io | 9.8 yr | 11.6M/wk | github.com/seanmonstar/reqwest | OK | Approved |
| `async-openai` | crates.io | 3.7 yr | 164k/wk | github.com/64bit/async-openai | OK | Approved |
| `eventsource-stream` | crates.io | 6.1 yr | 475k/wk | github.com/jpopesculian/eventsource-stream | OK | Approved |
| `tokio` | crates.io | 10 yr | 15.1M/wk | github.com/tokio-rs/tokio | OK | Approved |
| `axum` | crates.io | 5 yr | 7.9M/wk | github.com/tokio-rs/axum | OK | Approved |
| `tauri` | crates.io | 6.7 yr | 671k/wk | github.com/tauri-apps/tauri | OK | Approved |
| `r2d2_sqlite` | crates.io | 11 yr | 90k/wk | github.com/ivanceras/r2d2-sqlite | OK | Approved |
| `tauri-specta` | crates.io | 3.7 yr | 35k/wk | github.com/specta-rs/tauri-specta | OK | Approved *(low downloads reflect a niche codegen tool, not novelty — see Alternatives for the RC-status caveat, which is a stability concern, not a legitimacy one)* |
| `specta` | crates.io | 4 yr | 61k/wk | github.com/specta-rs/specta | OK | Approved |
| `dirs` | crates.io | 10.7 yr | 5.1M/wk | github.com/soc/dirs-rs | OK | Approved |
| `tempfile` | crates.io | 11.3 yr | 12.2M/wk | github.com/Stebalien/tempfile | OK | Approved |
| `blake3` | crates.io | 6.9 yr | 2.7M/wk | github.com/BLAKE3-team/BLAKE3 | OK | Approved *(Phase 3, listed for completeness)* |
| `comrak` | crates.io | 9.3 yr | 130k/wk | github.com/kivikakk/comrak | OK | Approved *(Phase 3)* |
| `tiktoken-rs` | crates.io | 3.5 yr | 494k/wk | github.com/zurawiki/tiktoken-rs | OK | Approved *(Phase 4)* |

**Packages removed due to [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none
**Packages removed for staleness (not slop):** `tauri-plugin-keyring` — exists and is legitimate, but last published 0.1.0 on 2024-12-23 with no repository URL. Removed on maintenance grounds, replaced by the `keyring` crate used directly.

Cargo has no `postinstall` equivalent, but three crates in this tree run **build scripts that compile C** (`libsqlite3-sys` via `rusqlite/bundled`, `blake3`, `ring`/`rustls`). That is expected and is the reason `bundled` gives reproducible offline builds; it is not a red flag.

## Architecture Patterns

### System Architecture Diagram

```
   ┌──────────────┐            ┌─────────────────────────┐
   │  Webview UI  │            │  Claude Code / Cursor   │
   │   (React)    │            │      (MCP client)       │
   └──────┬───────┘            └────────────┬────────────┘
          │ typed invoke()                  │ HTTP POST /mcp
          │ Result<T, IpcError>             │ Authorization: Bearer <token>
          │                                 │ Accept: application/json, text/event-stream
          ▼                                 ▼
┌─────────────────────┐        ┌──────────────────────────────────────┐
│  src-tauri          │        │  axum Router  (127.0.0.1:<port> only)│
│  #[tauri::command]  │        │                                      │
│  thin wrappers      │        │   ① TcpListener::bind("127.0.0.1")   │
│  + AppHandle        │        │   ② middleware: bearer-token check ──┼─► 401
│  + app_data_dir()   │        │   ③ nest_service("/mcp", …) ─────────┼─► 403 bad Host/Origin
└──────────┬──────────┘        │        rmcp StreamableHttpService    │   400 malformed
           │                   │   ④ (Phase 6) route "/clip" → WS     │
           │                   └──────────────────┬───────────────────┘
           │                                      │
           │  inject(data_dir, secrets, clock)    │  read-mostly; ONE write op
           ▼                                      ▼
      ┌───────────────────────────────────────────────────┐
      │        prism-core :: Engine  (the facade)         │
      │                                                   │
      │   pub trait ReadOps      ← both doors             │
      │   pub trait AgentWriteOps ← MCP door ONLY (D-03)  │
      │   pub trait AppWriteOps   ← Tauri door ONLY       │
      │   workspace_id: scopes every query (AGENT-03)     │
      └───┬───────────────────────┬───────────────────────┘
          │                       │
          ▼                       ▼
  ┌──────────────────┐   ┌────────────────────────────────┐
  │  prism-store     │   │  prism-llm                     │
  │  r2d2 Pool       │   │  trait LlmProvider  (D-05)     │
  │  ↳ WAL, migrate  │   │   ├ AnthropicClient (reqwest + │
  │  spawn_blocking  │   │   │   eventsource-stream, SSE) │
  └────────┬─────────┘   │   └ OpenAiCompatClient          │
           │             │       (async-openai, base_url) │
           ▼             │  keyring::Entry → OS Keychain  │
  ~/Library/Application  └───────────────┬────────────────┘
   Support/PrismDocs/                    │ ONLY egress in the whole app
     <project-id>/                       ▼
       prism.db  (+ -wal, -shm)   user-configured LLM endpoint
       backups/                    (api.anthropic.com | custom base_url)

  Project repo (never written to by the store):
    <workspace>/.prismdocs/            ← D-19 skeleton + README.md
    <workspace>/.prismdocs/project.json ← {"project_id": "<uuid>"}  (D-13 marker)
    <workspace>/.mcp.json               ← generated in onboarding step 4 (D-10/D-17)
```

Data flows one direction through the facade in every case: a request enters through exactly one of the two doors, is scoped to the current `workspace_id`, and reaches the store or the LLM boundary only via `Engine`. Nothing bypasses it — that is what makes AGENT-03's write-surface limit a compile-time property (D-03) rather than a runtime `if`.

### Component Responsibilities

| Path | Owns | Must NOT |
|------|------|----------|
| `crates/prism-core` | `Engine` facade, the three op traits, `WorkspaceId`, domain error enum | Depend on `tauri`, `axum`, or `keyring` implementations directly (take them as trait objects / params) |
| `crates/prism-store` | Pool construction, WAL pragma, migrations, all SQL | Know what a Tauri `AppHandle` is; resolve its own base directory |
| `crates/prism-llm` | `LlmProvider` trait + 2 impls, `SecretStore` trait + keyring impl, the one `reqwest::Client` | Log request/response bodies or key material |
| `crates/prism-mcp` | `serve(listener, engine) -> impl Future`, auth middleware, tool registration | Bind the socket itself, or choose the port |
| `src-tauri` | Commands, `AppHandle`, port scan, listener bind, window lifecycle | Contain business logic or SQL |

### Recommended Project Structure

```
PrismDocs/
├── Cargo.toml                 # [workspace] members = ["src-tauri", "crates/*"]
├── Cargo.lock                 # committed; NOTE the tauri-info caveat (Pitfall 5)
├── package.json               # webview deps + "tauri" script
├── vite.config.ts
├── index.html
├── src/                       # React webview
│   ├── bindings.ts            # generated by tauri-specta; COMMITTED
│   └── onboarding/            # the 4 steps (D-17)
├── src-tauri/
│   ├── Cargo.toml             # depends on the prism-* crates via path
│   ├── tauri.conf.json        # identifier, beforeDevCommand, beforeBuildCommand
│   ├── build.rs
│   └── src/
│       ├── main.rs            # setup(): resolve data dir → build Engine → spawn MCP task
│       ├── commands/          # #[tauri::command] fns, one file per domain
│       └── ipc_error.rs       # serializable error type
├── crates/
│   ├── prism-core/            # Engine facade + traits (D-02/D-03)
│   ├── prism-store/           # pool, migrations/, SQL
│   │   └── src/migrations/    # const M::up(...) slice
│   ├── prism-llm/             # LlmProvider trait, 2 impls, SecretStore
│   └── prism-mcp/             # axum router, auth layer, rmcp service
├── docs/adr/
│   └── 0001-tauri-vs-node-sidecar.md   # SC-5 / D-06 deliverable
└── .github/workflows/ci.yml
```

Phase 3 adds `crates/prism-anchor`, Phase 4 adds `crates/prism-lens` — the layout reserves the slots without creating empty crates now.

### Pattern 1: rmcp streamable-HTTP mounted on a loopback axum listener

**What:** rmcp 2.2 does not run a server. It gives you a `tower_service::Service` that you mount.
**When to use:** This is the whole of D-07.

```rust
// Source: modelcontextprotocol/rust-sdk @ rmcp-v2.2.0, examples/servers/src/counter_streamhttp.rs
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};

let ct = tokio_util::sync::CancellationToken::new();

let service = StreamableHttpService::new(
    || Ok(Counter::new()),                                   // service_factory
    LocalSessionManager::default().into(),                   // Arc<M>
    StreamableHttpServerConfig::default().with_cancellation_token(ct.child_token()),
);

let router = axum::Router::new().nest_service("/mcp", service);
let tcp_listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await?;
axum::serve(tcp_listener, router)
    .with_graceful_shutdown(async move { tokio::signal::ctrl_c().await.unwrap(); ct.cancel(); })
    .await?;
```

Exact signature, from the crate source [VERIFIED: rmcp 2.2.0 `src/transport/streamable_http_server/tower.rs`]:

```rust
pub fn new(
    service_factory: impl Fn() -> Result<S, std::io::Error> + Send + Sync + 'static,
    session_manager: Arc<M>,
    config: StreamableHttpServerConfig,
) -> Self
```

`impl tower_service::Service<http::Request<B>>` with `Response = http::Response<BoxBody<Bytes, Infallible>>` and `Error = Infallible`. There is also a direct `pub async fn handle<B>(&self, request: Request<B>) -> Response<…>` if you ever need to call it without tower.

### Pattern 2: The facade's two doors (D-02 + D-03)

**What:** Make "agent cannot create or delete comments/cards" unrepresentable rather than checked.
**When to use:** AGENT-03's write-surface clause.

```rust
// crates/prism-core/src/lib.rs
pub struct Engine { pool: Pool<SqliteConnectionManager>, workspace: WorkspaceId, /* … */ }

/// Reads available to BOTH doors. Every method is workspace-scoped.
pub trait ReadOps {
    fn list_feedback(&self) -> Result<Vec<FeedbackSummary>, EngineError>;
    fn get_document_comments(&self, doc: DocId) -> Result<Vec<Comment>, EngineError>;
}

/// The COMPLETE agent write surface. AGENT-03 is satisfied by this trait having
/// exactly one method and by `prism-mcp` depending on nothing else.
pub trait AgentWriteOps {
    fn respond_to_comment(&self, id: CommentId, body: AgentReply) -> Result<(), EngineError>;
}

/// App-only writes. `prism-mcp` must never import this.
pub trait AppWriteOps {
    fn create_comment(&self, /* … */) -> Result<CommentId, EngineError>;
    fn delete_comment(&self, id: CommentId) -> Result<(), EngineError>;
}
```

`prism-mcp`'s handler is then generic over `E: ReadOps + AgentWriteOps` — adding a destructive tool later would not compile without someone deliberately widening the bound, which is exactly the review signal we want. **Make this an explicit CI assertion too** (see Validation Architecture): a test that greps `crates/prism-mcp` for `AppWriteOps` and fails if found.

### Pattern 3: Token auth middleware in front of the MCP service

**What:** rmcp 2.2 has **no** server-side bearer-token support. Its `auth` feature is OAuth2 *client*-side. The per-install token (D-10 / AGENT-03) must be an axum layer wrapping the nested service.
**When to use:** Mandatory for SC-4's "rejects any request lacking the per-install token".

```rust
use axum::{extract::State, http::{Request, StatusCode, header}, middleware::{self, Next}, response::Response};

async fn require_token(
    State(expected): State<Arc<str>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let presented = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match presented {
        // constant-time compare — see Security Domain
        Some(t) if subtle::ConstantTimeEq::ct_eq(t.as_bytes(), expected.as_bytes()).into() => {
            Ok(next.run(req).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

let router = axum::Router::new()
    .nest_service("/mcp", mcp_service)
    .layer(middleware::from_fn_with_state(token.clone(), require_token));
```

Order matters: `.layer()` applies **outside** `nest_service`, so an unauthenticated request never reaches rmcp at all. That is what makes the 401 and the 403 independently observable in tests.

### Pattern 4: Enabling WAL correctly (D-14)

**What:** `journal_mode` is the one pragma that returns a row, so the obvious call fails.
**When to use:** In the pool's `ConnectionCustomizer` / `init`, once per connection.

```rust
// Source: docs.rs/rusqlite_migration/2.6.0 — "Enable WAL and foreign keys OUTSIDE migrations"
conn.pragma_update_and_check(None, "journal_mode", &"WAL", |_row| Ok(()))?;  // NOT pragma_update
conn.pragma_update(None, "synchronous", &"NORMAL")?;   // the standard WAL pairing
conn.pragma_update(None, "foreign_keys", &true)?;      // OFF by default in SQLite, per-connection
conn.busy_timeout(std::time::Duration::from_secs(5))?; // avoids spurious SQLITE_BUSY
```

`journal_mode=WAL` is **persistent in the database file** — it survives close/reopen and does not need re-setting. `synchronous`, `foreign_keys`, and `busy_timeout` are **per-connection** and must be set on every pooled connection, which is why they belong in the pool customizer rather than in a one-shot open.

### Pattern 5: Structural NFR-03 / NFR-04 enforcement

**What:** Make "content only goes to the user's endpoint" and "key never logged" properties of the module graph.

- **One `reqwest::Client`, constructed once in `prism-llm`, never made `pub`.** No other crate takes a `reqwest` dependency. A CI check (`cargo tree -i reqwest` / a manifest grep) then proves there is no second egress path.
- **Newtype the secret and hand-write `Debug`:**

```rust
pub struct ApiKey(String);
impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("ApiKey(***)") }
}
```
  This makes `tracing::debug!(?config)` and `dbg!()` safe by construction — the usual way keys leak into logs is a derived `Debug` on a config struct.
- **Telemetry off by default (NFR-03):** ship no analytics dependency at all in Phase 1. NFR-07's opt-in instrumentation is Phase 5. "Not enabled by default" is easiest to verify when the crate isn't in `Cargo.lock`.

### Anti-Patterns to Avoid

- **`Arc<Mutex<Connection>>` as the facade's "shared DB handle".** Literally satisfies D-02's wording and silently defeats D-14. Use `r2d2::Pool` — see Pitfall 1.
- **Calling `app.path().app_data_dir()` from a core crate.** It needs an `AppHandle`; the core would then require a running Tauri app to be unit-tested, breaking D-01's entire purpose. Inject the `PathBuf`.
- **Leaving `allowed_origins` at its default.** The default is an **empty vec, which disables Origin validation entirely.** SC-4 would be unverifiable. See Pitfall 2.
- **Writing the MCP token into `.mcp.json` in plaintext.** `.mcp.json` at the project root is designed to be committed. Use `headersHelper` (Pattern 6) or `${VAR}` expansion.
- **Blocking rusqlite calls inside an async task.** Stalls the Tokio worker that also serves MCP and LLM streaming. Wrap in `tokio::task::spawn_blocking`.
- **Implementing anything from D-18's forbidden list** (frontmatter parse, FS watcher, `documents` writes). The enumeration is read-only display feedback and nothing else.

## Don't Hand-Roll

| Problem | Don't build | Use instead | Why |
|---------|-------------|-------------|-----|
| Host/Origin DNS-rebinding validation | Custom header parser | `StreamableHttpServerConfig.allowed_hosts` / `.allowed_origins` | rmcp 2.2 already normalizes per RFC 6454 (scheme/host/port, `null` origin, IPv6, default-port stripping, HTTP/2 `:authority` fallback when `nest` drops `Host`). Hand-rolling reproduces ~150 lines of edge cases. |
| MCP session lifecycle (`Mcp-Session-Id`, SSE resumption, `Last-Event-ID`) | Custom SSE bookkeeping | `LocalSessionManager` + `StreamableHttpService` | Spec-conformant replay, keep-alives, and cancellation are already implemented. |
| Schema versioning table | A `schema_version` table + hand-written `if version < N` | `rusqlite_migration` `M::up()` + `to_latest()` | Uses SQLite's `user_version` (a fixed-offset integer in the file header) — no table, no parse cost, atomic per-migration transactions, plus a free `validate()` unit test. |
| Connection pooling / thread affinity | `thread_local!` connections or a hand-rolled queue | `r2d2` + `r2d2_sqlite` | `Connection` is `!Sync`; getting this wrong is a data race or a deadlock, not a compile error. |
| Secret storage | Encrypted file + a master password | `keyring` 4 (`Entry::new` → macOS Keychain Services) | NFR-04 says system keychain. Anything else re-implements key derivation, and Stronghold is already rejected. |
| SSE frame parsing for Anthropic | Manual `\n\n` splitting on the byte stream | `eventsource-stream` | Handles multi-line `data:`, `event:`, comments, `retry:`, and chunk boundaries that split a frame. |
| OpenAI-compatible request/response types | Hand-written structs | `async-openai` with `with_config(...base_url)` | Covers the long tail of proxies/local models (D-15), including streaming. |
| Constant-time token comparison | `a == b` on `&str` | `subtle::ConstantTimeEq` | `==` short-circuits on first mismatch — a timing oracle for a loopback secret. |
| Cryptographic token generation | `Uuid::new_v4().to_string()` | `rand::rngs::OsRng` → 32 bytes → base64url | UUIDv4 is 122 bits from a PRNG whose CSPRNG-ness is implementation-defined; `OsRng` is unambiguous. |

**Key insight:** Nearly every hand-roll temptation in this phase is in the *security boundary* — Origin parsing, token comparison, token generation, secret storage. Those are precisely the places where a subtly-wrong custom implementation still passes a happy-path test. Prefer the library and spend the saved effort on the negative tests (SC-4).

## Runtime State Inventory

*Not applicable — this is a greenfield scaffold, not a rename/refactor/migration. There is no prior runtime state: the repository contains only `.planning/`, `docs/`, `.claude/`, and `LICENSE` (verified by `ls -a`), with no `Cargo.toml`, `package.json`, or `src-tauri/`.*

However, **this phase creates the runtime state that every later phase's rename/refactor must inventory.** Recording it now so future phases don't have to rediscover it:

| Category created in Phase 1 | Location | Future-rename hazard |
|---|---|---|
| Stored data | `~/Library/Application Support/PrismDocs/<project-id>/prism.db` | The directory name embeds the product name; a rebrand needs a data migration, not just a code edit. |
| OS-registered state | macOS Keychain generic-password items, service = `"PrismDocs"` (2 entries: LLM key, MCP token) | Keychain entries are keyed by `(service, account)` — renaming the service string orphans the stored secrets. |
| Live service config (in user repos) | `<workspace>/.mcp.json` server key, `<workspace>/.prismdocs/` | Written into the *user's* repo; PrismDocs cannot retroactively rewrite it. Keep the server key stable forever. |
| Build artifacts | `target/`, `src/bindings.ts` (generated, committed) | `bindings.ts` is generated — regenerate rather than hand-edit. |

## Common Pitfalls

### Pitfall 1: `Arc<Mutex<Connection>>` silently cancels WAL

**What goes wrong:** D-02 says the facade "holds a shared DB handle." The idiomatic-looking way to share a `rusqlite::Connection` across the Tauri command layer and the MCP handler is `Arc<Mutex<Connection>>`. It compiles, it works, and it makes D-14 a no-op.
**Why it happens:** `rusqlite::Connection` is `Send` but **`impl !Sync for Connection`** [VERIFIED: docs.rs/rusqlite/0.40.1]. `Arc<T>` requires `T: Sync` to be `Send`, so the compiler forces you to wrap it — and the only obvious wrapper is a `Mutex`, which serializes reads behind writes at the *application* layer, above the level where WAL's reader/writer concurrency lives.
**How to avoid:** The facade holds `r2d2::Pool<SqliteConnectionManager>` (which is `Send + Sync`) and calls `pool.get()` per operation. WAL then does what D-14 wants: the MCP reader and the app writer touch different connections and don't block.
**Warning signs:** `Mutex<Connection>` anywhere in `prism-store`; MCP tool calls that visibly stall while the app is saving.

### Pitfall 2: `allowed_origins` defaults to empty, which turns Origin validation OFF

**What goes wrong:** You use `StreamableHttpServerConfig::default()`, assume "rmcp validates Origin," and ship. Every Origin is accepted. SC-4's Origin clause is unmet and the acceptance test passes vacuously.
**Why it happens:** From the crate source [VERIFIED: rmcp 2.2.0 `tower.rs` `impl Default`]:

```rust
allowed_hosts:   vec!["localhost".into(), "127.0.0.1".into(), "::1".into()],
allowed_origins: vec![],   // ← "Defaults to an empty list, which disables Origin validation."
```

and in `origin_is_allowed` / `validate_origin_header`:

```rust
if allowed_origins.is_empty() { return Ok(()); }        // validation skipped entirely
let Some(origin_header) = headers.get(ORIGIN) else { return Ok(()) };  // missing Origin always passes
```

**How to avoid:** Call `.with_allowed_origins([...])` explicitly with the origins the app itself uses. Note this is *correct*, not a workaround: the MCP spec says "**If the `Origin` header is present and invalid**, servers MUST respond with HTTP 403 Forbidden" [CITED: modelcontextprotocol.io/specification/2025-11-25/basic/transports]. Claude Code sends no `Origin` (it is not a browser), so a missing-Origin pass is required for the product to work at all. The token layer (Pattern 3) is what stops unauthenticated callers; Origin validation stops *browser-originated* cross-site calls.
**Warning signs:** A test that sends `Origin: https://evil.example` and gets 200 instead of 403.

### Pitfall 3: `rusqlite_migration` 2.6 will not build against the CLAUDE.md-pinned `rusqlite` 0.32

**What goes wrong:** You follow `.claude/CLAUDE.md` literally, pin `rusqlite = "0.32"` and `rusqlite_migration = "1"`, and either fail to resolve or end up on a two-year-old migration API.
**Why it happens:** `rusqlite_migration` 2.6.0 declares `rusqlite ^0.40.0` [VERIFIED: crates.io deps API] and MSRV **1.95**. Worse, if two `rusqlite` versions do end up in the graph, `libsqlite3-sys` declares `links = "sqlite3"` in its manifest, and **Cargo refuses to build a graph with two crates claiming the same native library** — a hard error, not a warning.
**How to avoid:** Pin `rusqlite = "0.40"`, `rusqlite_migration = "2.6"`, `r2d2_sqlite = "0.35"`, and set `rust-version = "1.95"` in `[workspace.package]`. Run `cargo tree -d` in CI and fail on any duplicate.
**Warning signs:** `error: multiple packages link to native library 'sqlite3'`.

### Pitfall 4: WAL makes the backup unit three files, not one

**What goes wrong:** D-12's "backup/export this project's data" copies `prism.db` while the app is running. The `-wal` file holds committed-but-uncheckpointed transactions; the restored copy silently loses the most recent writes.
**Why it happens:** In WAL mode a database is `prism.db` + `prism.db-wal` + `prism.db-shm`. A naive file copy of only the first is a torn backup.
**How to avoid:** Use SQLite's own online-backup path rather than the filesystem. Either `VACUUM INTO '<dest>'` (single statement, produces a consistent standalone file, available since SQLite 3.27) or rusqlite's `backup` feature. Ship the export as a zip of that vacuumed file plus the sidecar files — never a raw copy of the live directory.
**Warning signs:** A restored backup missing the last few comments; `-wal` files present in the exported archive.

### Pitfall 5: Top-level workspace changes where `Cargo.lock` lives, and `tauri info` notices

**What goes wrong:** After adding a root `Cargo.toml` with `members = ["src-tauri", ...]`, `cargo update` writes `Cargo.lock` at the repo root, and `tauri info` reports `(no lockfile)` — or, if a stale `src-tauri/Cargo.lock` exists, reports **older dependency versions than you're actually building**.
**Why it happens:** Tauri's CLI historically probes `src-tauri/Cargo.lock`. [CITED: github.com/tauri-apps/tauri/issues/4232]
**How to avoid:** Delete any `src-tauri/Cargo.lock` after creating the workspace, keep the single root lockfile, and treat `tauri info`'s dependency section as advisory. Do not "fix" it by re-adding a nested lockfile. Also add a root `.taurignore` (Tauri v2 reads one at the Cargo workspace root) so `tauri dev`'s file watcher ignores `.planning/`, `docs/`, and `target/`. [CITED: v2.tauri.app/develop/]
**Warning signs:** `tauri info` and `cargo tree` disagree on a version.

### Pitfall 6: `app_data_dir()` is `~/Library/Application Support/<bundle-identifier>`, not `.../PrismDocs`

**What goes wrong:** D-11 specifies `~/Library/Application Support/PrismDocs/<project keyed>/`. If you call Tauri's `PathResolver::app_data_dir()`, you get `$HOME/Library/Application Support/${bundle_identifier}` [VERIFIED: docs.rs/tauri/2.11.5 `PathResolver`] — i.e. `~/Library/Application Support/com.prismdocs.desktop/`, since Tauri's `identifier` must be reverse-DNS for macOS bundling and code-signing. The path in CONTEXT.md and the path the API produces are different.
**Why it happens:** Two different conventions: Tauri follows the Apple bundle-identifier convention; D-11 states a human-readable name.
**How to avoid:** Two things, and the second is the load-bearing one:
1. Decide explicitly which path wins and record it in the ADR. Recommendation: use `dirs::data_dir().join("PrismDocs")` to match D-11 literally, since it is a user-visible backup location (D-12) and a reverse-DNS directory is hostile to "find my backup."
2. **Regardless of which you pick, the core must not call either API itself.** `app_data_dir()` requires an `&AppHandle`, which would make `prism-store` untestable without a running Tauri app — a direct violation of D-01. `prism-store::open(base_dir: &Path, project_id: &ProjectId)` takes the path; `src-tauri` resolves and passes it; tests pass a `TempDir`.
**Warning signs:** `tauri::AppHandle` appearing in any `crates/prism-*` signature.

### Pitfall 7: `.mcp.json` is meant to be committed — the token must not be in it

**What goes wrong:** D-10 says onboarding writes the port and token into `.mcp.json`. Project-scoped `.mcp.json` lives at the repo root and is explicitly "designed to be checked into version control" [CITED: code.claude.com/docs/en/mcp]. Writing `"Authorization": "Bearer <token>"` there leaks a live loopback credential into git.
**Why it happens:** The straightforward reading of "回写进项目的 `.mcp.json`."
**How to avoid:** Claude Code supports two indirections; use one:
- `headersHelper` — a command whose stdout is a JSON object of headers, run at connect time with a 10s timeout. PrismDocs generates a tiny script that reads the token from the keychain: `{"headersHelper": "/usr/local/bin/prismdocs-mcp-auth"}`. Nothing secret is in the file.
- `${VAR}` / `${VAR:-default}` env expansion, which Claude Code applies to `url` and `headers` for HTTP servers.
**Warning signs:** A bearer token visible in `git diff`.

### Pitfall 8: Blocking SQLite calls inside async handlers

**What goes wrong:** An MCP tool handler is `async fn`; it calls `pool.get()?` and runs a query directly. Under load, the Tokio worker thread is parked in SQLite while it should be driving the SSE stream and the LLM request.
**Why it happens:** `rusqlite` is synchronous by design and none of the async wrappers (`tokio-rusqlite`, `deadpool-sqlite`) are currently compatible with `rusqlite 0.40`.
**How to avoid:** Wrap every store call reached from async in `tokio::task::spawn_blocking`. Do it *inside* `prism-store` so the facade exposes `async fn` and the discipline can't be forgotten at call sites:

```rust
pub async fn list_feedback(&self, ws: WorkspaceId) -> Result<Vec<FeedbackSummary>, StoreError> {
    let pool = self.pool.clone();
    tokio::task::spawn_blocking(move || { let conn = pool.get()?; /* … */ }).await?
}
```
**Warning signs:** SSE keep-alives arriving late; UI freezing during a large query.

## Code Examples

### Verified: full loopback MCP bring-up with token + Origin enforcement

```rust
// Composition of two VERIFIED sources:
//   rmcp 2.2.0 examples/servers/src/counter_streamhttp.rs (mounting)
//   rmcp 2.2.0 src/transport/streamable_http_server/tower.rs (config builders)
use std::sync::Arc;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};

pub async fn serve(
    listener: tokio::net::TcpListener,   // caller binds; see D-10 port scan
    engine: Arc<Engine>,
    token: Arc<str>,
    ct: tokio_util::sync::CancellationToken,
) -> anyhow::Result<()> {
    let port = listener.local_addr()?.port();

    let config = StreamableHttpServerConfig::default()
        .with_cancellation_token(ct.child_token())
        // Default is already loopback-only; restate it so intent is greppable.
        .with_allowed_hosts(["localhost", "127.0.0.1", "::1"])
        // MUST be non-empty or Origin validation is skipped (Pitfall 2).
        .with_allowed_origins([
            format!("http://127.0.0.1:{port}"),
            format!("http://localhost:{port}"),
        ]);

    let mcp = StreamableHttpService::new(
        move || Ok(PrismMcpServer::new(engine.clone())),
        LocalSessionManager::default().into(),
        config,
    );

    let router = axum::Router::new()
        .nest_service("/mcp", mcp)
        // Phase 6 (D-08) adds .route("/clip", get(ws_upgrade)) here.
        .layer(axum::middleware::from_fn_with_state(token, require_token));

    axum::serve(listener, router)
        .with_graceful_shutdown(async move { ct.cancelled().await })
        .await?;
    Ok(())
}
```

### Verified: keyring 4 API (both entries per D-10)

```rust
// Source: keyring 4.1.5 crate source, src/v1.rs
use keyring::Entry;

const SERVICE: &str = "PrismDocs";

// NFR-04 — the user's LLM API key
let e = Entry::new(SERVICE, "llm.anthropic.api_key")?;
e.set_password(&key)?;
let key = e.get_password()?;          // Err(keyring::Error::NoEntry) when absent
e.delete_credential()?;

// D-10 — the per-install MCP token
let e = Entry::new(SERVICE, "mcp.install_token")?;
```

`Entry::new` initializes the platform default store on first call (macOS: `apple_native_keyring_store::keychain::Store`). Also available: `set_secret(&[u8])` / `get_secret() -> Vec<u8>` for non-UTF-8 material.

### Verified: Anthropic Messages streaming, hand-rolled (D-04)

Required headers [CITED: platform.claude.com/docs/en/build-with-claude/streaming]:

```
x-api-key: $ANTHROPIC_API_KEY
anthropic-version: 2023-06-01
content-type: application/json
```

Event order on the wire, and the exact `data:` shapes:

```
event: message_start
data: {"type":"message_start","message":{"id":"msg_…","role":"assistant","content":[],
       "model":"claude-opus-5","stop_reason":null,"usage":{"input_tokens":25,"output_tokens":1}}}

event: content_block_start
data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}

event: ping
data: {"type":"ping"}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}

event: content_block_stop
data: {"type":"content_block_stop","index":0}

event: message_delta
data: {"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":15}}

event: message_stop
data: {"type":"message_stop"}
```

Four things a hand-rolled parser must handle, all confirmed in the spec text:
1. **`ping` events appear anywhere** in the stream and carry no content — ignore, don't error.
2. **`error` events arrive mid-stream** with HTTP 200 already sent: `event: error` / `data: {"type":"error","error":{"type":"overloaded_error",…}}`. Surface as a stream error, not a parse failure.
3. **`message_delta.usage` counts are cumulative** — the docs explicitly warn about this.
4. **"New event types may be added; your code should handle unknown event types gracefully."** Use `#[serde(other)]` or an untagged catch-all; do not `deny_unknown_fields`.

For Phase 1 only the request/response plumbing is needed — Lens streaming is Phase 4 (LENS-09).

### Verified: Anthropic `count_tokens`

```bash
curl https://api.anthropic.com/v1/messages/count_tokens \
  -H 'Content-Type: application/json' \
  -H 'anthropic-version: 2023-06-01' \
  -H "X-Api-Key: $ANTHROPIC_API_KEY" \
  -d '{"messages":[{"role":"user","content":"Hello, world"}],"model":"claude-opus-5"}'
# → {"input_tokens": 2095}
```

Useful in Phase 4 for exact Claude counts (NFR-05). **Not** the best onboarding smoke test — see below.

### The D-16 connection test: use `GET /v1/models` for both provider families

This is a recommendation with a concrete rationale, so the planner can write it as an acceptance criterion.

| Candidate | Anthropic | OpenAI-compatible | Verdict |
|---|---|---|---|
| Minimal completion (`max_tokens: 1`) | Bills tokens; needs a valid model id | Same | Rejected — costs money, and fails confusingly if the user typed a wrong model id |
| `POST /v1/messages/count_tokens` | Free-ish, but **requires a valid `model`** | No equivalent endpoint | Rejected — asymmetric, and couples the test to model-id validity |
| **`GET /v1/models`** | Available on the first-party Claude API | Universal in the OpenAI-compatible ecosystem; `async-openai` exposes `client.models().list()` | **Recommended** |

`GET /v1/models` is the right choice because it (a) consumes **zero tokens and zero dollars**, (b) exercises the identical path D-16 wants proven — keychain read → `reqwest` client → user-supplied `base_url` → auth header → TLS → parse, (c) is **symmetric across both provider families**, so one code path and one acceptance criterion cover D-15, (d) does **not** require the user to have already picked a valid model id, and (e) returns data the onboarding UI can immediately reuse to populate a model picker, which turns a throwaway test into a feature.

Auth differs only in the header:

| Family | Method + path | Headers |
|---|---|---|
| Anthropic | `GET {base_url}/v1/models` | `x-api-key: <key>`, `anthropic-version: 2023-06-01` |
| OpenAI-compatible | `GET {base_url}/v1/models` | `Authorization: Bearer <key>` |

Failure taxonomy the onboarding UI must distinguish (this is the risk D-16 exists to destroy):

| Symptom | Almost always means |
|---|---|
| DNS / connect error | Bad `base_url` host, or offline |
| TLS handshake failure | Corporate proxy with a private CA, or `http://` vs `https://` mismatch |
| 401 / 403 | Wrong key, or key pasted for the other provider family |
| 404 on `/v1/models` | `base_url` already ends in `/v1` (double-`/v1`) — the single most common user error with custom endpoints |
| 200 but unparseable body | Endpoint isn't actually OpenAI-compatible |

Recommend normalizing `base_url` (strip a trailing `/` and a trailing `/v1`) before the first request, and showing the raw status + first 200 bytes of the body on failure. **Do not log the key or the full response.**

### Recommended: project-id marker for move/rename survival (D-13)

Claude's discretion; this is the standard shape.

```jsonc
// <workspace>/.prismdocs/project.json     — created in onboarding step 3 (D-19)
{ "project_id": "0f4d…-uuid-v4", "created_at": "2026-07-28T…Z", "schema": 1 }
```

Resolution order on workspace open:
1. Read `<workspace>/.prismdocs/project.json`. If present → `project_id` → `<data_dir>/PrismDocs/<project_id>/`. **Path is never part of the key**, so move/rename is free.
2. If the marker is absent but a registered workspace in the app-level registry has this exact path → adopt that `project_id` and rewrite the marker (recovers a deleted `.prismdocs/`).
3. Otherwise → new project: mint a UUIDv4, create both.

Keep an app-level `<data_dir>/PrismDocs/workspaces.json` mapping `project_id → last-known path` for the project switcher; treat the *marker* as authoritative and the registry as a cache. `.prismdocs/project.json` must **not** be gitignored even though the rest of `.prismdocs/` is (D-19 / AGENT-02) — otherwise a `git clone` on another machine can't re-associate. Flag this to the user: the file contains only a random UUID, no secrets.

## State of the Art

| Old approach | Current approach | When changed | Impact on this phase |
|--------------|------------------|--------------|----------------------|
| MCP over stdio subprocess | Streamable HTTP as a co-equal transport with mandatory Origin validation | Spec 2025-03-26, refined through 2025-11-25 | Makes D-07 spec-blessed rather than exotic. stdio remains "SHOULD support" for clients. |
| HTTP+SSE two-endpoint transport | Single "MCP endpoint" supporting POST + GET | 2025-03-26 replaced 2024-11-05 | rmcp 2.2 implements the new one; no backward-compat shim needed for Claude Code/Cursor. |
| `tauri-plugin-stronghold` for secrets | OS-native keychain via `keyring` | Stronghold slated for removal in Tauri v3 | Already reflected in CLAUDE.md; keyring 4 (2026-04) restructured onto `keyring-core` with pluggable stores. |
| `keyring` 3.x monolith | `keyring` 4.x = thin `v1` facade over `keyring-core` + per-platform store crates | 4.0.0, 2026-04-26 | Same `Entry` API surface; the change is in the dependency graph, not your code. |
| `reqwest` 0.12 | `reqwest` 0.13 | 2026-05 | The whole Rust HTTP ecosystem moved together (rmcp, async-openai, tauri all on ^0.13). |
| `rusqlite_migration` 1.x | 2.x (`from_slice`, const-friendly, MSRV 1.95) | 2.0, 2025 | `Migrations` can be a `const` built from a `&[M]` slice — nicer for a static migration list. |

**Deprecated / outdated for this phase:**
- `tauri-plugin-keyring` — 0.1.0, last touched 2024-12-23, no repo link.
- `deadpool-sqlite` / `tokio-rusqlite` — not deprecated, just **currently incompatible** with `rusqlite 0.40`. Recheck each phase.
- `.claude/CLAUDE.md` § Technology Stack version pins — see below.

## Doc Corrections Required

SC-5 and D-06 make the ADR a deliverable; two of these corrections belong in it.

| Location | Current text | Correction | Authority |
|---|---|---|---|
| `.claude/CLAUDE.md` § Technology Stack, `rmcp` row | "Exposes `list_feedback` / … over **stdio** to the agent, with loopback IPC back to the running app." | App-hosted loopback **streamable HTTP** on `127.0.0.1:<port>`, per-install bearer token + Origin allowlist. No subprocess, no second IPC hop. | CONTEXT.md **D-07** explicitly overrides. Confirmed feasible: rmcp 2.2 feature `transport-streamable-http-server` + `StreamableHttpService`. |
| `.claude/CLAUDE.md` § Version Compatibility, "rmcp 2.2 … **Pin to the 2025-11-25 stable protocol**" | — | **No action needed** — `ProtocolVersion::LATEST` in rmcp 2.2.0 *is* `V_2025_11_25`, and it is the `Default`. `KNOWN_VERSIONS` also contains `V_2026_07_28` (draft) but it is not selected by default. Record that this is satisfied by construction. | rmcp 2.2.0 `src/model.rs` |
| `.claude/CLAUDE.md` § Core Technologies + Supporting Libraries | rusqlite 0.32 / rusqlite_migration 1 / keyring 3 / reqwest 0.12 / async-openai 0.27 / tiktoken-rs 0.6 / notify-debouncer-full 0.4 / similar 2 / gray_matter 0.2 / htmd 0.1 / tauri 2.10.1 | 0.40 / 2.6 / 4.1 / 0.13 / 0.41 / 0.12 / 0.7 / 3.1 / 0.3 / 0.5 / 2.11.5 | crates.io, 2026-07-28. Two are correctness-critical (rusqlite ↔ rusqlite_migration; reqwest 0.13 tree convergence). |
| `.claude/CLAUDE.md` § Version Compatibility, "comrak 0.54 \| Rust 1.85+ \| Hard MSRV" | — | Workspace MSRV is **1.95**, set by `rusqlite_migration` 2.6 (comrak's 1.85 and keyring's 1.88 are both below it). | crates.io version metadata |
| `.claude/CLAUDE.md` § What NOT to Use | "Keep the SQLite DB in the app data dir (`~/Library/Application Support/PrismDocs/`, keyed by project path)" | Keyed by **project-id**, not project path (D-13). Also note Tauri's `app_data_dir()` yields `…/<bundle-identifier>`, not `…/PrismDocs` (Pitfall 6). | CONTEXT.md D-13; docs.rs/tauri 2.11.5 |

## Environment Availability

Probed on the target machine (macOS 25.5.0, Apple Silicon) on 2026-07-28. **The planner should re-run these as a Wave-0 task** — the phase cannot start without a Rust toolchain, and this research ran without probing the local machine's `cargo`/`node` presence.

| Dependency | Required by | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain ≥ **1.95** | Every crate; hard MSRV from `rusqlite_migration` 2.6 | **UNPROBED** | — | `rustup toolchain install 1.95` + `rust-toolchain.toml` pinning. **Blocking if absent.** |
| `cargo`, `clippy`, `rustfmt` | Build + CI gate | UNPROBED | — | `rustup component add clippy rustfmt` |
| Node.js + npm/pnpm | Vite webview, `beforeDevCommand` | UNPROBED | — | Blocking for `tauri dev`; the core crates build and test without it |
| Xcode Command Line Tools | Links `libsqlite3-sys` (bundled C), `ring`, WebKit | UNPROBED | — | `xcode-select --install`. **Blocking if absent** — `cc` is required to compile bundled SQLite. |
| C compiler (`cc`) | `rusqlite` `bundled` feature | Implied by CLT | — | — |
| macOS Keychain | NFR-04 | ✓ (OS built-in) | — | For CI/headless: a `SecretStore` trait with an in-memory impl (see Validation Architecture) |
| Network access | D-16 connection test only | User-dependent | — | **By design:** everything except the LLM test must work offline (NFR-02). `rusqlite` `bundled` needs no network at runtime. |
| Apple Developer ID cert | Code signing + notarization | UNPROBED | — | Not needed in Phase 1 — dev builds run unsigned. Defer to release phase. |

**Missing dependencies with no fallback:** Rust ≥ 1.95 and a C compiler. Both must be verified in Wave 0 before any implementation task.
**Missing dependencies with fallback:** Node (core crates are independently buildable); Keychain in CI (trait + in-memory impl).

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` / `#[tokio::test]` (`cargo test`); no third-party runner needed |
| Config file | **none — Wave 0.** Greenfield: no `Cargo.toml`, no `.github/workflows/`, no test dirs exist. Verified by `ls -a`. |
| Quick run command | `cargo test -p prism-store -p prism-mcp -p prism-llm` |
| Full suite command | `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all --check` |

Integration tests for the loopback server use `reqwest` as a **dev-dependency** against a real `TcpListener::bind("127.0.0.1:0")` (port 0 = OS-assigned, so tests never collide with the running app or each other). This is the same shape rmcp's own test suite uses.

### Phase Requirements → Test Map

| Req ID | Behavior | Test type | Automated command | File exists? |
|--------|----------|-----------|-------------------|--------------|
| AGENT-03 | Server binds 127.0.0.1 only — connecting to the LAN IP is refused | integration | `cargo test -p prism-mcp bind_is_loopback_only` | ❌ Wave 0 |
| AGENT-03 | Request with **no** `Authorization` → **401** | integration | `cargo test -p prism-mcp rejects_missing_token` | ❌ Wave 0 |
| AGENT-03 | Request with a **wrong** bearer token → **401** | integration | `cargo test -p prism-mcp rejects_wrong_token` | ❌ Wave 0 |
| AGENT-03 | Valid token + `Origin: https://evil.example` → **403** | integration | `cargo test -p prism-mcp rejects_foreign_origin` | ❌ Wave 0 |
| AGENT-03 | Valid token + **no** `Origin` → **200** (Claude Code sends none) | integration | `cargo test -p prism-mcp accepts_missing_origin` | ❌ Wave 0 |
| AGENT-03 | Valid token + `Host: evil.example` → **403** | integration | `cargo test -p prism-mcp rejects_foreign_host` | ❌ Wave 0 |
| AGENT-03 | `prism-mcp` does not reference `AppWriteOps` (write-surface limit is structural, D-03) | unit (source assertion) | `cargo test -p prism-mcp agent_write_surface_is_minimal` | ❌ Wave 0 |
| AGENT-03 | Queries are workspace-scoped: a second workspace's rows are invisible | unit | `cargo test -p prism-store workspace_scoping` | ❌ Wave 0 |
| NFR-02 | `journal_mode` reads back `wal` after open **and** after close+reopen | unit | `cargo test -p prism-store wal_is_persistent` | ❌ Wave 0 |
| NFR-02 | Migrations apply from empty → latest, and are idempotent on re-run | unit | `cargo test -p prism-store migrations_to_latest` | ❌ Wave 0 |
| NFR-02 | `Migrations::validate()` passes (built-in guard against a malformed set) | unit | `cargo test -p prism-store migrations_validate` | ❌ Wave 0 |
| NFR-02 | Open + read an existing DB with **no** network (no HTTP client constructed on this path) | unit | `cargo test -p prism-store offline_open_and_read` | ❌ Wave 0 |
| NFR-02 | Backup/export produces a standalone file that opens and matches row counts | integration | `cargo test -p prism-store backup_round_trip` | ❌ Wave 0 |
| NFR-03 | `reqwest` appears exactly once in the workspace dep graph, only under `prism-llm` | unit (manifest assertion) | `cargo test -p prism-core single_egress_path` | ❌ Wave 0 |
| NFR-03 | `Debug` on the config/secret newtype does not render key material | unit | `cargo test -p prism-llm secret_debug_is_redacted` | ❌ Wave 0 |
| NFR-03 | No telemetry crate in `Cargo.lock` | unit (manifest assertion) | `cargo test -p prism-core no_telemetry_dependency` | ❌ Wave 0 |
| NFR-04 | `SecretStore` round-trip: set → get → delete → `NoEntry` | unit (in-memory impl) | `cargo test -p prism-llm secret_store_round_trip` | ❌ Wave 0 |
| NFR-04 | Real keychain round-trip on macOS | integration, **`#[ignore]`d** | `cargo test -p prism-llm --ignored keychain_real` | ❌ Wave 0 |
| NFR-04 | `base_url` normalization: trailing `/` and trailing `/v1` collapse correctly | unit | `cargo test -p prism-llm base_url_normalization` | ❌ Wave 0 |
| NFR-04 / SC-1,3 | `GET /v1/models` against a local mock for both families; correct auth header per family | integration (mock server) | `cargo test -p prism-llm models_probe_both_families` | ❌ Wave 0 |
| SC-1 (manual) | App launches on macOS Apple Silicon; 4-step onboarding completes against a **real** endpoint | manual-only | — | Justification: requires the user's real API key and a real network endpoint. Cannot be automated without shipping a credential. Covered by the mock-server test above for everything except the live call. |
| SC-5 | ADR file exists and names the rejected alternative + downgrade trigger | manual-only | — | Justification: prose quality is not machine-checkable. A CI file-existence check is possible but near-worthless. |

### Sampling Rate

- **Per task commit:** `cargo test -p <touched crate>` (sub-second for `prism-store`; the `bundled` SQLite build is cached after the first compile)
- **Per wave merge:** `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`
- **Phase gate:** full suite green + `cargo tree -d` clean + `cargo fmt --all --check` before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] Root `Cargo.toml` workspace manifest — prerequisite for every test
- [ ] `rust-toolchain.toml` pinning ≥ 1.95 — otherwise `rusqlite_migration` 2.6 won't compile
- [ ] `crates/prism-store/tests/` + a `fn test_db() -> (TempDir, Pool)` fixture — covers NFR-02
- [ ] `crates/prism-mcp/tests/` + a `fn spawn_test_server() -> (SocketAddr, String /*token*/, CancellationToken)` fixture binding port 0 — covers AGENT-03
- [ ] `crates/prism-llm/tests/` + a mock HTTP server (`wiremock` 0.6 or a hand-rolled `axum` fixture) — covers NFR-03/NFR-04. Prefer the hand-rolled axum fixture: axum is already in the tree, so it adds zero new dependencies.
- [ ] `SecretStore` trait + `InMemorySecretStore` — makes NFR-04 testable in CI without a keychain prompt
- [ ] `.github/workflows/ci.yml` on `macos-latest` (arm64) — see below
- [ ] Framework install: **none.** Rust's test harness is built in; only `tempfile` is a new dev-dependency.

### CI shape (macOS, skeleton stage)

```yaml
# .github/workflows/ci.yml
jobs:
  core:                      # fast gate — NO Tauri, NO Node, NO webview
    runs-on: macos-latest    # arm64, matches the target platform
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.95
        with: { components: clippy, rustfmt }
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all --check
      - run: cargo clippy -p prism-core -p prism-store -p prism-llm -p prism-mcp --all-targets -- -D warnings
      - run: cargo test  -p prism-core -p prism-store -p prism-llm -p prism-mcp
      - run: cargo tree -d              # fail on duplicate libsqlite3-sys / reqwest
  shell:                     # slower gate — full app build
    runs-on: macos-latest
    steps: [ …, npm ci, npm run tauri build -- --debug ]
```

Splitting `core` from `shell` is not cosmetic — it is the executable proof that D-01 holds. The day `cargo test -p prism-store` starts needing a webview, the `core` job breaks and tells you the boundary leaked. Keep the keychain integration test `#[ignore]`d so the `core` job never hangs on a headless Keychain prompt; run it locally and in a nightly job.

## Security Domain

`security_enforcement: true`, `security_asvs_level: 1` per `.planning/config.json`.

### Applicable ASVS Categories

| ASVS category | Applies | Standard control |
|---------------|---------|------------------|
| V1 Encoding & Sanitization | yes | Generated `.mcp.json` / `.prismdocs/README.md` are `serde_json` / templated writes, never string concatenation of user paths. Workspace paths must be canonicalized before use. |
| V2 Validation & Business Logic | yes | `base_url` parsed with `url::Url` and scheme-checked (`http`/`https` only — reject `file:`) before any request. Workspace path must be an existing directory. |
| V3 Web Frontend Security | partial | The webview is local-origin. Set Tauri's CSP in `tauri.conf.json` (`default-src 'self'`) — Tauri v2 leaves it null by default, which is permissive. |
| V6 Authentication | **yes** | Per-install bearer token on the loopback MCP endpoint. 32 bytes from `OsRng`, base64url-encoded, compared in constant time, stored only in the Keychain. |
| V7 Session Management | yes (delegated) | `Mcp-Session-Id` is generated and validated by rmcp's `LocalSessionManager` — do not reimplement. Note the ID is a session correlator, **not** an authenticator; the bearer token is. |
| V8 Authorization | **yes** | AGENT-03's two clauses: workspace scoping (every query carries `WorkspaceId`) and write-surface limit (Pattern 2's trait split). Both are type-level, per D-03. |
| V11 Cryptography | yes | Only for token generation. **Never hand-roll** — `OsRng` + `subtle::ConstantTimeEq`. Everything else (TLS) is `rustls` via reqwest. |
| V16 Security Logging | yes | rmcp already `tracing::warn!`s on rejected Host/Origin ("possible DNS rebinding attempt" / "possible cross-origin attack") — wire a subscriber so this is captured. Log **rejections**, never the token or the API key. |
| V4 Access Control (RBAC/multi-tenant) | no | Single-user local app; no user accounts in MVP (multi-user is V2-04, deferred). |
| V13 API & Web Service | partial | The MCP endpoint is the only server surface. Loopback-only + token + Origin covers L1. |
| V14 Files & Resources | yes | Path traversal: the workspace root must be canonicalized and every derived path checked to remain under it, especially for the D-18 read-only enumeration. |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard mitigation |
|---------|--------|---------------------|
| **DNS rebinding against the local MCP server** — a malicious page resolves a hostname to 127.0.0.1 and issues cross-origin requests | Spoofing / Elevation | `allowed_hosts` (rmcp default, loopback-only) + non-empty `allowed_origins` → 403. This is the exact attack the MCP spec's security warning names. |
| **Token exfiltration via committed `.mcp.json`** | Information disclosure | `headersHelper` indirection (Pitfall 7). Never write the token into a repo file. |
| **Timing attack on token comparison** | Spoofing | `subtle::ConstantTimeEq`, not `==`. |
| **API key leaking into logs / crash reports** | Information disclosure | Redacted `Debug` impl (Pattern 5); no telemetry crate in the tree at all. |
| **Binding `0.0.0.0` "for convenience"** | Spoofing / Elevation | Bind `127.0.0.1` explicitly; assert it in a test (`bind_is_loopback_only`). CLAUDE.md already forbids this — keep the test as the enforcement. |
| **SQL injection into workspace-scoped queries** | Tampering | rusqlite named/positional parameters exclusively. Never `format!` a query. |
| **Path traversal via a crafted workspace path or a symlinked `.prismdocs/`** | Tampering | `std::fs::canonicalize` the root; verify every derived path still starts with it. |
| **SSRF via user-supplied `base_url`** | — (accepted) | NFR-04 *requires* arbitrary `base_url` (proxies, local models). Not mitigatable without breaking the requirement. Mitigate the blast radius instead: restrict schemes to http/https, and make the destination visible in the UI so the user can see where content is going. **Record this as an accepted risk in the ADR.** |
| **Malicious MCP client enumerating other workspaces** | Information disclosure | Server instance is constructed with the current `WorkspaceId` baked in; there is no tool parameter that selects a workspace. |

## Assumptions Log

| # | Claim | Section | Risk if wrong |
|---|-------|---------|---------------|
| A1 | Claude Code and Cursor send **no** `Origin` header when connecting to an HTTP MCP server (they are not browsers). | Pitfall 2, Validation Architecture | If a client *does* send one, a non-empty `allowed_origins` that omits it would 403 the primary integration. **Mitigation: the `accepts_missing_origin` test plus a manual smoke test against real Claude Code in onboarding step 4 (D-17) — which the phase already requires.** |
| A2 | `~/Library/Application Support/PrismDocs/` (human-readable) is preferable to Tauri's `…/<bundle-identifier>/` for D-11/D-12. | Pitfall 6 | Cosmetic if wrong, but it is a user-visible backup path and a later change is a data migration. **Needs a user decision — surface in discuss-phase or the ADR.** |
| A3 | The project-id marker at `.prismdocs/project.json` should be **committed** (not gitignored) so a clone re-associates. | D-13 recommendation | If gitignored, cloning to a second machine mints a fresh project and orphans the data. Contains only a random UUID, but it is still a file added to the user's repo — **worth confirming with the user**, since the whole product promise is "your repo stays clean." |
| A4 | tauri-specta 2.0-rc is acceptable for a production-bound skeleton despite never reaching GA. | Alternatives Considered | An RC break could block a build. Mitigated by committing generated `bindings.ts`. Explicitly D-Discretion, so this is our call to make — but the planner should know it is a knowingly-accepted risk, not an oversight. |
| A5 | The fixed default MCP port (D-10) — a specific number was not chosen. | D-10 | Picking one already used by a common dev tool causes a confusing first-run scan. Suggest a high, uncommon port (e.g. 47xxx range) and verify against IANA/common-use lists before committing. |
| A6 | A Rust 1.95 toolchain and Xcode CLT are installed on the target machine. | Environment Availability | **Blocking if wrong.** Not probed this session. Wave 0 must verify. |
| A7 | `wiremock`-free hand-rolled axum mock is sufficient for the LLM boundary tests. | Validation Architecture | Low risk; if the fixture gets unwieldy, add `wiremock` 0.6 as a dev-dependency. |
| A8 | Bundling both `x86_64` and `aarch64` is out of scope; Apple Silicon only. | Environment Availability | Matches ROADMAP SC-1 ("macOS Apple Silicon") and CLAUDE.md constraints. |

## Open Questions

1. **Which app-support path convention wins?**
   - What we know: Tauri's `app_data_dir()` returns `…/<bundle-identifier>`; D-11 states `…/PrismDocs/`. Both are achievable; the core takes an injected path either way.
   - What's unclear: whether the user cares that the backup directory is human-findable.
   - Recommendation: go with `…/PrismDocs/` (matches D-11 verbatim, better for D-12's manual backup story), record the deviation from Tauri's default in the ADR, and never call `app_data_dir()` from core regardless.

2. **Is `.prismdocs/project.json` committed or gitignored?**
   - What we know: D-19 says `.prismdocs/` gets a `.gitignore` suggestion (PRD §4.1). D-13 needs a stable marker that survives a move.
   - What's unclear: whether the marker specifically should be exempted from that gitignore.
   - Recommendation: exempt it (`.prismdocs/*` + `!.prismdocs/project.json`) and tell the user why in onboarding. It contains one random UUID and no secrets. **This is a user-facing decision about their repo — worth an explicit confirmation.**

3. **What is the fixed default MCP port?**
   - What we know: D-10 says fixed default, scan upward if occupied.
   - What's unclear: the number.
   - Recommendation: pick from the dynamic/private range (49152–65535) or a high registered-but-unused port; avoid 3000/5173/8000/8080/9000. Verify against IANA before committing, since changing it later invalidates every already-written `.mcp.json`.

4. **How much SQLite schema is in scope for the skeleton?**
   - What we know: D-Discretion. Phase 2+ each append tables.
   - What's unclear: whether Phase 1 creates stub tables it doesn't use.
   - Recommendation: create **only** what Phase 1 actually writes — `workspaces`, `settings`, and the `schema_meta` implied by `user_version`. **Do not stub `documents`/`comments`/`cards`**: D-18 explicitly forbids writing to `documents`, and an empty table invites a Phase-2 migration to alter rather than create, which is strictly more work.

5. **Does the Anthropic `GET /v1/models` response shape match what the model picker needs?**
   - What we know: the Models API is documented as available on the first-party Claude API and returns `id`, `display_name`, `max_input_tokens`, `max_tokens`, `capabilities`.
   - What's unclear: whether a user's custom `base_url` proxy for Anthropic also proxies `/v1/models` (many only proxy `/v1/messages`).
   - Recommendation: treat a 404 on `/v1/models` as a **soft** failure for Anthropic-family custom endpoints — fall back to `count_tokens` with a user-entered model id, and let the user type a model name manually. Do not block onboarding on it.

## Sources

### Primary (HIGH confidence)

- **rmcp 2.2.0 crate source** (downloaded from static.crates.io, inspected directly) — `src/transport/streamable_http_server/tower.rs` (`StreamableHttpService::new` signature, `StreamableHttpServerConfig` fields + `Default` impl, `host_is_allowed` / `origin_is_allowed` / `validate_dns_rebinding_headers` / `bad_request_response` / `forbidden_response`); `src/model.rs` (`ProtocolVersion::LATEST`, `KNOWN_VERSIONS`); `Cargo.toml.orig` (feature graph, `reqwest 0.13.2`, no axum dep); `README.md` (transport table)
- **modelcontextprotocol/rust-sdk @ tag `rmcp-v2.2.0`** — `examples/servers/src/counter_streamhttp.rs` (canonical axum mounting)
- **keyring 4.1.5 crate source** — `src/lib.rs`, `src/v1.rs` (exact `Entry` API, macOS store selection, feature gating)
- **rusqlite_migration 2.6.0** — crate source + README (`Migrations::from_slice`, `to_latest`, `user_version` strategy)
- **crates.io API** — version, publish date, MSRV, edition, feature list, and normal-dependency requirements for every crate in the Standard Stack table (queried 2026-07-28)
- **modelcontextprotocol.io/specification/2025-11-25/basic/transports** — the Security Warning text (Origin MUST validate, 403 on present-and-invalid, SHOULD bind 127.0.0.1), Accept/`MCP-Protocol-Version`/`Mcp-Session-Id` requirements
- **platform.claude.com/docs/en/build-with-claude/streaming** — full SSE event sequence, raw wire examples, required headers, cumulative-usage warning, unknown-event-type guidance
- **platform.claude.com/docs/en/api/messages-count-tokens** — endpoint, headers, request/response shape
- **code.claude.com/docs/en/mcp** — `.mcp.json` HTTP shape, `type: "http"` / `"streamable-http"`, `headers`, `headersHelper` (with requirements), `${VAR}` expansion locations, scope precedence
- **docs.rs/rusqlite/0.40.1** — `impl Send` / `impl !Sync for Connection`, `SQLITE_OPEN_NO_MUTEX` rationale
- **docs.rs/tauri/2.11.5 `PathResolver`** — `app_data_dir()` → `$HOME/Library/Application Support/${bundle_identifier}`
- **`gsd-tools query package-legitimacy check --ecosystem crates`** — all 18 crates, verdict OK

### Secondary (MEDIUM confidence)

- **Context7 `/websites/v2_tauri_app`** — top-level workspace `Cargo.toml` shape (`members = ["src-tauri"]`), `dataDir()` macOS resolution
- **docs.rs/rusqlite_migration/2.6.0** (via WebFetch summary) — `pragma_update_and_check` for WAL, `validate()` test pattern
- **v2.tauri.app/develop/** — `.taurignore` at the Cargo workspace root, `tauri dev` watching dependent workspace crates
- **claude-api skill (bundled reference)** — current model IDs, required Anthropic headers, Models API availability. Confirms there is **no official Anthropic Rust SDK** (raw HTTP is the documented path for unsupported languages), which independently validates D-04's hand-rolled client.

### Tertiary (LOW confidence — flagged, not load-bearing)

- **WebSearch: Tauri workspace pitfalls** → github.com/tauri-apps/tauri#4232 (`tauri info` + top-level lockfile). Symptom is cosmetic; treated as a Pitfall note, not a design constraint.

## Metadata

**Confidence breakdown:**

- **Standard stack:** HIGH — every version, MSRV, feature list, and inter-crate dependency requirement read directly from the crates.io API on 2026-07-28; the two build-breaking incompatibilities (rusqlite↔rusqlite_migration, rusqlite↔pool crates) confirmed from dependency metadata, not inference.
- **MCP transport + security surface:** HIGH — read from the rmcp 2.2.0 crate source itself, including the `Default` impl and the validation functions, and cross-checked against the MCP specification's own normative text. Status codes (401/403/400) are quoted from the source, so acceptance criteria can be written against them.
- **Keychain:** HIGH — full `Entry` API read from `keyring-4.1.5/src/v1.rs`.
- **Anthropic wire format:** HIGH — official docs with raw SSE examples.
- **Architecture patterns (facade split, injection, pooling):** MEDIUM-HIGH — the constraints they resolve are verified facts (`!Sync`, `AppHandle` requirement, `links` key); the specific shapes are reasoned design, and they are D-Discretion territory.
- **Typed IPC:** MEDIUM — tauri-specta's RC status is verified; whether it is the *right* call for this project is a judgment.
- **CI shape:** MEDIUM — standard Rust/macOS practice, not verified against a running pipeline. The core/shell split is a design recommendation.
- **Environment availability:** LOW — **not probed.** Wave 0 must verify.

**Research date:** 2026-07-28
**Valid until:** ~2026-08-28 for the architecture and spec findings; **~7 days for the version pins** — rmcp shipped 3.0.0-beta.3 on 2026-07-27 (one day before this research) and the MCP 2026-07-28 protocol revision lands today. Neither should be adopted for launch (CLAUDE.md correctly pins 2025-11-25 stable, and rmcp 2.2 defaults to it), but re-verify the crate table if planning slips more than a week.
