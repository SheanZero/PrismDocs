<!-- GSD:project-start source:PROJECT.md -->

## Project

**PrismDocs**

PrismDocs 是面向 vibe coder 的工程文档工作台：AI 用英文维护紧凑、技术完备的技术文档（Base 层，唯一真相源、省 token），产品自动投影出用户母语（首发简体中文）的口语化理解层（Lens 层，供人快速 review）。用户在文档上写评论，评论结构化回流给 Claude Code / Cursor 等编码 agent 驱动下一轮迭代；文档以 LLM Wiki + 卡片笔记（Zettelkasten）组织，配套 Chrome 插件把网页资料剪藏进知识库。

主力用户是使用 AI 编码工具、中文（或日文）为母语、英文技术阅读中等的独立开发者（P1）和产品型创始人（P2）。定位是 IDE 之外、面向人的工程知识层——「OKF 之上的人类理解与决策层」。

**Core Value:** **「双层文档 + 评论回流」显著降低 vibe coder review AI 文档的成本** —— 若一切从简，必须跑通的是：AI 写英文 Base → 产品投影中文 Lens → 人在 Lens 上批注 → 结构化回流 Claude Code → AI 改 Base → Lens 重投影提醒复核（F1–F4 闭环，AC-4a）。

### Constraints

- **平台**: macOS（Apple Silicon）首发桌面应用 + Chrome 扩展（MV3，Edge 兼容顺带）；Windows 列为 P1 — 主力用户在 Mac，聚焦单平台降低 MVP 成本
- **架构**: 本地优先，文档/评论/卡片全落本地，不强制上云；单目录可备份（SQLite + 文件）— 数据主权 + 无服务端成本
- **真相源**: 磁盘 Markdown 文件是 Base 层权威副本，PrismDocs 不锁文件；Block ID / 评论 / 卡片存 sidecar 不污染用户源文件 — 与用户 IDE/agent/git 工作流零冲突
- **Agent 接口**: 优先 MCP + 文件协议（`.prismdocs/feedback/*.md`），一级支持 Claude Code，兼容 Cursor，其他 agent 纯文件兜底 — 接入成本压到最低
- **LLM**: 用户自备 API key（Anthropic / OpenAI 兼容端点，支持自定义 base_url），key 存系统钥匙串；订阅制内置额度后置 — 成本转嫁 + 隐私
- **成本**: Lens 投影是主要模型成本，增量重投影（只重生成受影响段落）+ 紧凑 Base 层既是体验也是成本设计；投影调用显示预估 token
- **数据格式**: Base 层存储/导出遵循 OKF v0.1 核心约定（文件即概念、frontmatter 六字段、受控 type 词表、index.md/log.md 保留名）— 换取无锁定互操作
- **性能**: 500 文档 / 2000 卡片规模下全文搜索 <300ms、文档打开 <500ms、FS 变更呈现 <10s

<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->

## Technology Stack

## Headline Decision: Tauri v2, Rust-centric core

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| **Tauri** | 2.11.5 | Desktop app shell (macOS-first, Windows P1) | ~8-12 MB bundles vs Electron's 150 MB+; a local-first app that stays resident must be light. Rust backend is the natural home for the CPU-bound engine (FS watch, markdown parse, diff-based anchor migration, SQLite). Cross-platform: same codebase reaches Windows for P1 at near-zero incremental cost — a SwiftUI app could not. System WebView means the rich editor UI is still web tech. |
| **Rust** | 1.95+ | Engine language (Tauri core) | Workspace MSRV, pinned in `rust-toolchain.toml`. Driven by **rusqlite_migration 2.6**, not by comrak — comrak's 1.85 and keyring's 1.88 are both below it. Owns FS/DB/parse/anchoring/keychain/LLM/MCP. Deterministic, fast, single source of truth for anchoring logic. |
| **React** | 19.x | Webview UI framework | Largest editor/annotation component ecosystem; Tauri's official templates support it. (Svelte 5 or SolidJS are fine substitutes if the team prefers — not load-bearing.) |
| **Vite** | 7.x | Webview build tool | Tauri default; fast HMR against the Rust dev server. |
| **rusqlite** | 0.40.x (bundled feature) | Sidecar metadata store (Block IDs, comments, cards, Lens cache, anchors, provenance, clips) | For a pure-SQLite embedded desktop app, rusqlite is the no-debate choice: direct, zero ORM overhead, statically links SQLite via the `bundled` feature (no system libsqlite dependency). Use **WAL mode** so the MCP reader and the app writer don't block each other. **0.40 is not optional:** rusqlite_migration 2.6 requires `rusqlite ^0.40`, and `libsqlite3-sys` declares `links = "sqlite3"`, so a version split is a hard Cargo error rather than a silent duplicate. |
| **r2d2** + **r2d2_sqlite** | 0.8.x / 0.35.x | Connection pool in front of the sidecar store | **Required, not optional.** `rusqlite::Connection` is `Send` but **not `Sync`**, so the obvious "shared DB handle" spelling is `Arc<Mutex<Connection>>` — which serializes every read behind every write and cancels the entire benefit of WAL. The facade holds a pool. 0.35 is currently the only pool crate on `rusqlite ^0.40` (deadpool-sqlite and tokio-rusqlite are both still on older majors). |
| **axum** | 0.8.x | Loopback HTTP host for the MCP endpoint | rmcp ships a `tower_service::Service`, **not a server** — something has to mount it. axum 0.8 is what the official rmcp example uses, and it also supplies `middleware::from_fn_with_state` for the bearer-token layer and the route slot for the Phase-6 clip WebSocket. |
| **SQLite FTS5** | (bundled in rusqlite) | Full-text search (<300ms over 500 docs / 2000 cards, per PRD §5) | Built into SQLite, no extra service, trivially hits the perf target. Index doc/card/clip text into an FTS5 virtual table. |

> **Version source:** every pin in this table was verified against crates.io on **2026-07-28** and is recorded in `.planning/phases/01-foundation-core-engine-skeleton/01-RESEARCH.md` § Standard Stack. The authoritative pins live in the root `Cargo.toml` `[workspace.dependencies]` block; this table mirrors it. The `rmcp` transport row reflects decision **D-07**, which overrides the earlier stdio description.
| **comrak** | 0.54.0 | Markdown → Block AST (source of truth for anchoring) | CommonMark 0.31.2 + GFM compatible; **exposes `sourcepos`** (line/col span per node) — essential for mapping Blocks to file offsets, extracting comment quotes, and re-rendering. It produces a real arena AST tree (not just an event stream), which is what block-level anchoring needs. Used by crates.io, docs.rs, GitLab, Deno. |
| **rmcp** | 2.2.0 | Local MCP server (Rust official SDK) | Official `modelcontextprotocol/rust-sdk`. Tracks the 2026-07-28 draft while staying compatible with the 2025-11-25 stable protocol. Keeps MCP in Rust → no Node runtime to bundle, one process, one DB handle. Exposes `list_feedback` / `get_feedback` / `respond_to_comment` / `get_document_comments` / `get_context_pack` (PRD §4.2) over **app-hosted loopback streamable HTTP** — the resident app itself serves `rmcp`'s `StreamableHttpService` on `127.0.0.1:<port>`, guarded by a per-install bearer token and a non-empty `Origin` allowlist. **No subprocess and no second IPC hop:** the app *is* the MCP server and wraps the Engine facade directly. Requires features `server` + `macros` + `transport-streamable-http-server`. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **notify** + **notify-debouncer-full** | notify 8.x / debouncer 0.7.x | FS watcher (REQ-1.4) | Standard Rust file-watching stack. `notify-debouncer-full` gives the 2s debounce/coalescing PRD requires for agents that write in bursts, plus rename/move coalescing needed for AC-1c. |
| **similar** | 3.1.x | Diff-based Block anchor migration | mitsuhiko's diffing crate (LCS / patience). This is the engine behind "match old Blocks to new Blocks by content similarity + relative position," the moat feature. Pair with a content-hash + heading-path heuristic. |
| **blake3** | 1.x | Content hashing for stable Block IDs | Fast, collision-resistant hash of normalized Block text → the content-hash half of the Block ID (positional heuristic is the other half). Also used to detect renamed/moved files (REQ-1.2 edge case) as the same document. |
| **rusqlite_migration** | 2.6.x | Schema migrations for the sidecar DB | Keeps the SQLite schema versioned; ships migrations with the app. Tracks the version in SQLite's `user_version` header field, so there is no migrations table to query. **This crate sets the workspace MSRV of 1.95 and requires `rusqlite ^0.40`.** Enable WAL and foreign keys *outside* migrations, in the pool customizer. |
| **gray_matter** (Rust crate) | 0.3.x | YAML frontmatter parse (REQ-1.8, OKF six fields) | Parses existing frontmatter into structured metadata and round-trips without corrupting user files (§2.5 "round-trip不破坏"). Materializes frontmatter only on OKF export (REQ-7.6). |
| **htmd** | 0.5.x | HTML → Markdown on the desktop side (REQ-1.3, AI-generated .html import) | Turndown-inspired Rust converter; keeps HTML import in the Rust core. (The Chrome extension uses turndown.js separately — see below.) |
| **keyring** crate (used **directly**, not via a Tauri plugin) | 4.1.x | API keys in the OS keychain (PRD §5, macOS Keychain / Windows Credential Manager) | Native secure storage directly from Rust, no master-password prompt. `Entry::new(service, account)` → a macOS Keychain generic-password item; service is `PrismDocs`, with two accounts (the LLM key and the per-install MCP token). 4.x is a thin `v1` facade over `keyring-core` — same `Entry` API, different dependency graph. **Prefer this over `tauri-plugin-stronghold`** (deprecation slated for Tauri v3) **and over `tauri-plugin-keyring`** (0.1.0, last published 2024-12-23, no repository URL — and it would wrongly put secret access behind the shell). |
| **reqwest** | 0.13.x | HTTP client for LLM calls (Rust core) | Async, TLS, streaming bodies. Keys never leave the Rust process (read from keychain). Supports custom `base_url` for OpenAI-compatible / proxy / local endpoints (PRD §5). 0.13 is where rmcp 2.2, async-openai 0.41 and tauri 2.11 all converge — pinning 0.12 forks the tree into two TLS stacks. **Feature names changed in 0.13:** 0.12's `rustls-tls` is now `rustls`, and trust anchors are a separate feature (`rustls-native-certs` to read the macOS system trust store). Declared in exactly one crate, `prism-llm`, so NFR-03's single-egress guarantee is a property of the module graph. |
| **eventsource-stream** | 0.2.x | SSE parsing for streaming LLM responses | Parses Anthropic `content_block_delta` / OpenAI `chat.completion.chunk` SSE for per-segment streaming Lens rendering (REQ-2.8). |
| **async-openai** | 0.41.x | OpenAI-compatible client (streaming, base_url) | Mature Rust client; `with_config` supports custom base_url → covers OpenAI + the long tail of OpenAI-compatible providers/proxies/local models in one client. |
| **tiktoken-rs** | 0.12.x | Local token counting for OpenAI-family models (o200k_base) | In-process token estimate for cost display (REQ-2.9, F7 token totals) without a network round-trip. For Claude, call Anthropic's `messages/count_tokens` REST endpoint for exact counts. |
| **tokio** | 1.x | Async runtime | Required by rmcp, reqwest, axum, notify debouncer; Tauri v2 is async-native. `rusqlite` is blocking, so every store call reached from async goes through `spawn_blocking`. |

> **Version source:** verified against crates.io on **2026-07-28**; see `.planning/phases/01-foundation-core-engine-skeleton/01-RESEARCH.md` § Standard Stack. Crates already in the tree are pinned in the root `Cargo.toml`; the rest (comrak, similar, blake3, gray_matter, htmd, notify, tiktoken-rs) arrive in Phases 2–4 and their versions here are advisory until then.

### Chrome MV3 Extension (separate artifact)

| Library | Version | Purpose | Notes |
|---------|---------|---------|-------|
| **WXT** | 0.20.x | MV3 extension framework/build | The 2026 standard for MV3: handles manifest generation, HMR, TS, and cross-browser (Edge) output. Preferred over hand-rolled Vite + @crxjs. |
| **@mozilla/readability** | 0.6.0 | Main-content extraction (REQ-6.1, Readability-class) | Run on a *clone* of the live DOM in the content script. Battle-tested (Firefox Reader View). |
| **turndown** | 7.2.4 | HTML → Markdown (REQ-6.2) | Add **turndown-plugin-gfm** for tables/strikethrough. **Code blocks are the real work (AC-6a):** write a custom `turndown` rule that reads `code.textContent` (not innerHTML) to strip Prism/highlight.js/`hljs`/Stack Overflow `prettyprint` span noise, and infers the language from `class` (`language-xxx`, `lang-xxx`). This custom rule is what makes SO answers paste-and-run clean. |
| **gpt-tokenizer** | 3.x (o200k_base) | Token estimation (REQ-6.3, AC-6c ±10%) | ~50 KB, fast, runs in the MV3 service worker. `o200k_base` is now the standard encoding (GPT-4o/GPT-5 family). Use it as the universal estimate in the extension; exact Claude counts come later from the desktop app. (js-tiktoken is the heavier "official" alternative — not needed here.) |
| **Native bridge: loopback WebSocket** | — | Extension ↔ desktop app (REQ-6.5) | Extension connects to the running app's `127.0.0.1:<port>` WS endpoint with a token handshake. Chosen over Chrome Native Messaging because: no native-host manifest install step, works identically on Edge, and lets the extension **queue clips in `chrome.storage` when the app is down** and flush on reconnect (REQ-6.5). Handle MV3 service-worker idle-termination by reconnecting on wake. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| **Tauri CLI** (`@tauri-apps/cli` 2.x) | Dev/build/bundle | `tauri dev` / `tauri build`; code-signing + notarization config for macOS distribution. |
| **CodeMirror 6** | Base-layer markdown source editing (REQ-1.6) | Decorations/gutters are ideal for comment anchors on source. Keep Base as markdown source, not WYSIWYG. |
| **react-markdown** (remark) | Rendering Lens + Base read views with Block anchors | Web-side rendering only — **not** the anchoring source of truth (comrak in Rust is). Supports streaming/incremental render for REQ-2.8. |
| **cargo + clippy + rustfmt** | Rust build/lint/format | Standard. |

## Installation

# --- Desktop app (Tauri v2 scaffold) ---

# Webview UI deps

# Tauri plugins (JS side)

# Rust core deps — declared once in the ROOT Cargo.toml [workspace.dependencies]

# and inherited by each crate with `.workspace = true`. Not in src-tauri/Cargo.toml:

# src-tauri is a thin shell member, and the core crates must not depend on tauri (D-01).

#   tauri = { version = "2.11", features = [] }

#   rusqlite = { version = "0.40", features = ["bundled"] }

#   rusqlite_migration = "2.6"

#   r2d2 = "0.8"

#   r2d2_sqlite = "0.35"

#   axum = "0.8"

#   reqwest = { version = "0.13", default-features = false, features = ["json", "stream", "rustls", "rustls-native-certs", "http2"] }

#   eventsource-stream = "0.2"

#   async-openai = "0.41"

#   rmcp = { version = "2.2", features = ["server", "macros", "transport-streamable-http-server"] }

#   tokio = { version = "1", features = ["full"] }

#   keyring = "4.1"   # the crate directly — NOT tauri-plugin-keyring

# Arriving in later phases (Phase 2 import, Phase 3 anchoring, Phase 4 lens):

#   comrak = "0.54"

#   similar = "3.1"

#   blake3 = "1"

#   gray_matter = "0.3"

#   htmd = "0.5"

#   notify = "8"

#   notify-debouncer-full = "0.7"

#   tiktoken-rs = "0.12"

# --- Chrome MV3 extension (separate package) ---

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| **Tauri v2** | **Electron** | If the team is pure-JS and wants the MCP + LLM logic in-process using the most mature official SDKs (`@modelcontextprotocol/sdk`, `@anthropic-ai/sdk`, `openai`), and bundle size is acceptable. Electron's one genuine edge for THIS app: it can host the MCP server and LLM clients in its Node main process with zero extra runtime. But it costs ~150 MB + higher idle memory on an always-resident local-first app, and gives no engine-in-Rust benefit. Net: not worth it here. |
| **Tauri v2** | **Native SwiftUI** | Only if macOS-exclusive forever and a fully native feel is a hard requirement. Rejected because Windows is P1 (SwiftUI = full rewrite) and the rich document/comment editor would be built from scratch in AppKit/SwiftUI — far slower than reusing web editor components. |
| **rmcp (Rust MCP)** | **@modelcontextprotocol/sdk 1.29.0 (TS)** in a bundled Node sidecar | If MCP tool logic gets complex, you want Claude-Code-parity tooling, or you want to prototype the server fast. This is the most battle-tested MCP SDK (Claude Code itself is TS). Cost: reintroduces a Node runtime into the bundle. **Note:** TS SDK v2 (beta, splits into `@modelcontextprotocol/server` + `/client`) lands 2026-07-28 with the new spec; v1.x stays supported ≥6 months. |
| **Rust LLM clients (async-openai + thin reqwest Anthropic)** | **Node sidecar with official `@anthropic-ai/sdk` (~0.7x) + `openai` (6.49.0)** | If hand-rolling the Anthropic streaming client proves painful, or you want the SDKs' polished retry/streaming/`countTokens` helpers. The official SDKs handle SSE edge cases and exact token counting better than hand-rolled code. Cost: Node runtime in the bundle. Pairs naturally with choosing the TS MCP SDK above (one sidecar for both). |
| **comrak** | **pulldown-cmark** | If you only need fast one-way HTML rendering and never mutate/traverse the tree. pulldown-cmark is a faster pull-parser but emits an event stream, not a mutable AST with easy node addressing — worse for block anchoring. |
| **rusqlite** | **SQLx / SeaORM** | Use SQLx if you want compile-time-checked SQL across multiple DB backends; SeaORM if you want ActiveRecord-style ergonomics. Neither is warranted for a single-file embedded SQLite store — they add async/build complexity and (SQLx sqlite) native-client dependencies for no benefit here. |
| **loopback WebSocket bridge** | **Chrome Native Messaging** | Use Native Messaging if you need the browser to launch the helper on demand even when the app isn't running, and you accept installing a native-host manifest. The WS approach is simpler and cross-browser, and the offline-queue requirement (REQ-6.5) removes Native Messaging's main advantage. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| **Electron (as default)** | ~150 MB bundle + high idle RAM on an always-on local-first app; no Rust-engine benefit. | Tauri v2. |
| **tauri-plugin-stronghold** for API keys | Officially slated for deprecation and removal in Tauri v3; you'd be building on a dead-end. | The `keyring` crate 4.1 used **directly from `prism-llm`** (native OS keychain). Not `tauri-plugin-keyring` either: 0.1.0, last published 2024-12-23, no repository URL, and it would put secret access behind the shell in violation of D-01. |
| **Writing Block IDs into user `.md` files** | Violates the hard "don't pollute source" constraint (PROJECT.md, PRD §2.4); breaks git/IDE/agent workflows. | Store all Block IDs/comments/cards in the sidecar SQLite DB; materialize frontmatter only on OKF export. |
| **Two markdown parsers for anchoring** (comrak in Rust *and* remark in JS both authoritative) | Non-deterministic Block boundaries across parsers → silent anchor drift, the #1 moat risk. | comrak (Rust) is the single source of truth for Block boundaries/IDs; remark/react-markdown is render-only. |
| **Storing the metadata DB inside the repo's `.prismdocs/`** | Would create git noise and risk committing private comments/cards; conflicts with Q3's sidecar decision. | Keep the SQLite DB in the app data dir (`~/Library/Application Support/PrismDocs/`), keyed by **project-id, never by project path** (D-13) — a path key loses the data the moment the user moves or renames the folder. Note that Tauri's `PathResolver::app_data_dir()` yields `~/Library/Application Support/<bundle-identifier>`, **not** the human-readable `.../PrismDocs/`; since this is a user-visible backup location (D-12), compute it with `dirs::data_dir().join("PrismDocs")` in the shell and inject the path into the core. Reserve in-repo `.prismdocs/feedback` + `.prismdocs/context` only for agent-handoff files (gitignored by default per PRD §4.1). |
| **Calling `app_data_dir()` (or any `AppHandle` API) from a core crate** | It would make `prism-store` untestable without a running Tauri app, defeating the entire point of the workspace split (D-01). | The shell resolves paths and passes them down; core crates take an injected `PathBuf` and tests pass a `TempDir`. |
| **`Arc<Mutex<Connection>>` as the facade's shared DB handle** | Literally satisfies "one shared handle" and silently cancels WAL: `Connection` is `!Sync`, the mutex is forced, and every read then queues behind every write. | An `r2d2::Pool<SqliteConnectionManager>`, which is `Send + Sync`; call `pool.get()` per operation. |
| **Leaving rmcp's `allowed_origins` at its default** | The default is an **empty vec, which disables Origin validation entirely** — the security acceptance test would pass vacuously. | Call `.with_allowed_origins([...])` explicitly. A *missing* Origin header must still pass: Claude Code is not a browser and sends none. |
| **Writing the MCP bearer token into `.mcp.json`** | Project-scoped `.mcp.json` sits at the repo root and is designed to be committed — that leaks a live loopback credential into git. | Use Claude Code's `headersHelper` indirection (a small script that reads the token from the Keychain) or `${VAR}` env expansion. The token itself lives only in the Keychain. |
| **Binding the MCP/bridge servers to `0.0.0.0`** | Security: PRD requires loopback-only, workspace-scoped, read-mostly. | Bind strictly to `127.0.0.1` (or a Unix domain socket in the app data dir); scope to current workspace; only `respond_to_comment` writes. |
| **js-tiktoken in the extension** | ~200 KB, heavier than needed for a ±10% estimate in an MV3 worker. | gpt-tokenizer (o200k_base, ~50 KB). |
| **`chrono` for time / naive FS polling** | Polling misses agent burst-writes and wastes CPU; use event-based watching. | `notify` + `notify-debouncer-full` (event-based, debounced). |

## Stack Patterns by Variant

- Rust core owns MCP (rmcp) and LLM (async-openai + thin reqwest Anthropic client, tiktoken-rs for local counts).
- Zero Node runtime shipped → smallest bundle, unified toolchain.
- Cost: hand-roll the Anthropic SSE client (the API is simple: `message_start` / `content_block_delta` / `message_stop`).
- Bundle one Node sidecar hosting both the MCP server (`@modelcontextprotocol/sdk`) and LLM clients (`@anthropic-ai/sdk`, `openai`).
- Rust core still owns FS watch, SQLite, parse, anchoring, keychain.
- Cost: +~40-80 MB Node runtime (still far below Electron); IPC between Rust core and the Node sidecar (loopback).
- Tauri: mostly free. Swap keyring backend (Windows Credential Manager — the `keyring` crate handles this transparently). Verify `notify` on Windows (ReadDirectoryChangesW) rename semantics. Re-sign/package for Windows.

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| **Workspace MSRV** | **Rust 1.95** | The binding constraint, pinned in `rust-toolchain.toml`. Set by **rusqlite_migration 2.6**. comrak's 1.85 and keyring's 1.88 are both below it, so satisfying 1.95 satisfies everything. |
| comrak 0.54 | Rust 1.85+ | Its own MSRV, but not the workspace floor — see the row above. |
| Tauri 2.11 | `@tauri-apps/api` 2.x, `@tauri-apps/cli` 2.x | Keep JS API and Rust crate on the same major (v2). |
| rusqlite 0.40 (`bundled`) | rusqlite_migration 2.6, r2d2_sqlite 0.35 | `bundled` statically links SQLite; no system libsqlite needed → reproducible builds across macOS/Windows. These three versions move as a set: `libsqlite3-sys` declares `links = "sqlite3"`, so two rusqlite versions in one graph is a hard Cargo error. `cargo tree -d` is the gate. |
| reqwest 0.13 | rmcp 2.2, async-openai 0.41, tauri 2.11 | The whole HTTP ecosystem moved together. Feature rename from 0.12: `rustls-tls` → `rustls`, with trust anchors split out (`rustls-native-certs`). |
| rmcp 2.2 | MCP protocol 2025-11-25 (stable) + 2026-07-28 (draft) | **Satisfied by construction — no action needed.** `ProtocolVersion::LATEST` in rmcp 2.2.0 already resolves to `V_2025_11_25` and is the `Default`. `KNOWN_VERSIONS` also contains the 2026-07-28 draft, but it is never selected by default. Upgrade only after Claude Code / Cursor adopt the new spec. |
| @modelcontextprotocol/sdk (TS) | v1.29.0 = stable; v2 = beta | If you go the Node-sidecar route, ship v1.x for production; v2 API can still change until 2026-07-28. |
| gpt-tokenizer o200k_base | GPT-4o/4.1/5, o1/o3/o4 | Correct encoding for current OpenAI models; use as the universal extension estimate. |

## Sources

- Tauri v2 stable + release cadence — https://v2.tauri.app/blog/tauri-20/ , https://v2.tauri.app/release/ (v2.10.1, 2026-03-04) — HIGH
- MCP TypeScript SDK — https://www.npmjs.com/package/@modelcontextprotocol/sdk , https://github.com/modelcontextprotocol/typescript-sdk (v1.29.0 stable; v2 beta, spec 2026-07-28) — HIGH
- Rust MCP SDK (rmcp) — https://crates.io/crates/rmcp , https://github.com/modelcontextprotocol/rust-sdk (v2.2.0, tracks 2026-07-28 draft, compatible 2025-11-25) — HIGH
- comrak — https://crates.io/crates/comrak , https://docs.rs/comrak (v0.54.0, CommonMark 0.31.2 + GFM, Rust 1.85+, sourcepos) — HIGH
- Rust SQLite/ORM comparison — https://rustify.rs/articles/rust-sqlx-vs-diesel-vs-seaorm-2026 , https://byteiota.com/rust-orms-2026-sqlx-vs-diesel-vs-seaorm-comparison/ (rusqlite for embedded SQLite) — HIGH
- turndown — https://www.npmjs.com/package/turndown (v7.2.4) — HIGH
- @mozilla/readability — https://www.npmjs.com/package/@mozilla/readability , https://github.com/mozilla/readability (v0.6.0) — HIGH
- gpt-tokenizer vs js-tiktoken — https://github.com/niieani/gpt-tokenizer , https://www.pkgpulse.com/guides/gpt-tokenizer-vs-js-tiktoken-vs-xenova-transformers-llm-2026 (o200k_base, ~50 KB) — HIGH
- OpenAI Node SDK — https://www.npmjs.com/package/openai (v6.49.0, custom base_url, streaming) — HIGH
- @anthropic-ai/sdk (messages.countTokens, SSE streaming) — https://www.npmjs.com/package/@anthropic-ai/sdk (~0.7x series) — MEDIUM (exact patch not pinned)
- Tauri secure storage / keychain — https://v2.tauri.app/plugin/stronghold/ (Stronghold deprecated in v3) , https://crates.io/crates/tauri-plugin-keyring , keyring crate — MEDIUM
- async-openai / tiktoken-rs (Rust LLM path) — crates.io — MEDIUM

<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->

## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->

## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->

## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, `.github/skills/`, or `.codex/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->

## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:

- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->

<!-- GSD:profile-start -->

## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
