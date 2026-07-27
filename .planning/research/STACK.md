# Stack Research

**Domain:** Local-first desktop knowledge/documentation workbench (macOS-first) + Chrome MV3 extension + embedded local MCP server
**Researched:** 2026-07-27
**Confidence:** HIGH (shell, storage, markdown AST, extension); MEDIUM (LLM Rust clients, MCP spec churn)

---

## Headline Decision: Tauri v2, Rust-centric core

**Build the desktop app on Tauri v2 with a Rust core that owns the engine (FS watch, Block AST parse, anchor migration, SQLite, keychain, LLM calls, MCP), and a web (React + Vite) webview for UI.** This is the single most consequential choice and everything below assumes it. Rationale and the Tauri-vs-Electron-vs-SwiftUI tradeoff are in [Core Technologies](#core-technologies) and [Alternatives Considered](#alternatives-considered).

The core architectural commitment is **"Rust-max, zero bundled Node runtime"**: the webview is JS/TS (that is just Tauri's rendering model — the system WebView, not a shipped Node), but there is no Node.js process in the bundle. MCP and LLM are done in Rust. This keeps the installed footprint small (a local-first app users leave running) and the toolchain unified. The one place JS runs outside the webview is the Chrome extension, which is a separate artifact.

---

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| **Tauri** | 2.10.1 | Desktop app shell (macOS-first, Windows P1) | ~8-12 MB bundles vs Electron's 150 MB+; a local-first app that stays resident must be light. Rust backend is the natural home for the CPU-bound engine (FS watch, markdown parse, diff-based anchor migration, SQLite). Cross-platform: same codebase reaches Windows for P1 at near-zero incremental cost — a SwiftUI app could not. System WebView means the rich editor UI is still web tech. |
| **Rust** | 1.85+ | Engine language (Tauri core) | Required by comrak 0.54; owns FS/DB/parse/anchoring/keychain/LLM/MCP. Deterministic, fast, single source of truth for anchoring logic. |
| **React** | 19.x | Webview UI framework | Largest editor/annotation component ecosystem; Tauri's official templates support it. (Svelte 5 or SolidJS are fine substitutes if the team prefers — not load-bearing.) |
| **Vite** | 6.x | Webview build tool | Tauri default; fast HMR against the Rust dev server. |
| **rusqlite** | 0.32.x (bundled feature) | Sidecar metadata store (Block IDs, comments, cards, Lens cache, anchors, provenance, clips) | For a pure-SQLite embedded desktop app, rusqlite is the no-debate choice: direct, zero ORM overhead, statically links SQLite via the `bundled` feature (no system libsqlite dependency). Use **WAL mode** so the MCP reader and the app writer don't block each other. |
| **SQLite FTS5** | (bundled in rusqlite) | Full-text search (<300ms over 500 docs / 2000 cards, per PRD §5) | Built into SQLite, no extra service, trivially hits the perf target. Index doc/card/clip text into an FTS5 virtual table. |
| **comrak** | 0.54.0 | Markdown → Block AST (source of truth for anchoring) | CommonMark 0.31.2 + GFM compatible; **exposes `sourcepos`** (line/col span per node) — essential for mapping Blocks to file offsets, extracting comment quotes, and re-rendering. It produces a real arena AST tree (not just an event stream), which is what block-level anchoring needs. Used by crates.io, docs.rs, GitLab, Deno. |
| **rmcp** | 2.2.0 | Local MCP server (Rust official SDK) | Official `modelcontextprotocol/rust-sdk`. Tracks the 2026-07-28 draft while staying compatible with the 2025-11-25 stable protocol. Keeps MCP in Rust → no Node runtime to bundle, one process, one DB handle. Exposes `list_feedback` / `get_feedback` / `respond_to_comment` / `get_document_comments` / `get_context_pack` (PRD §4.2) over **stdio** to the agent, with loopback IPC back to the running app. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **notify** + **notify-debouncer-full** | notify 8.x | FS watcher (REQ-1.4) | Standard Rust file-watching stack. `notify-debouncer-full` gives the 2s debounce/coalescing PRD requires for agents that write in bursts, plus rename/move coalescing needed for AC-1c. |
| **similar** | 2.x | Diff-based Block anchor migration | mitsuhiko's diffing crate (LCS / patience). This is the engine behind "match old Blocks to new Blocks by content similarity + relative position," the moat feature. Pair with a content-hash + heading-path heuristic. |
| **blake3** | 1.x | Content hashing for stable Block IDs | Fast, collision-resistant hash of normalized Block text → the content-hash half of the Block ID (positional heuristic is the other half). Also used to detect renamed/moved files (REQ-1.2 edge case) as the same document. |
| **rusqlite_migration** | 1.x | Schema migrations for the sidecar DB | Keeps the SQLite schema versioned; ships migrations with the app. |
| **gray_matter** (Rust crate) | 0.2.x | YAML frontmatter parse (REQ-1.8, OKF six fields) | Parses existing frontmatter into structured metadata and round-trips without corrupting user files (§2.5 "round-trip不破坏"). Materializes frontmatter only on OKF export (REQ-7.6). |
| **htmd** | 0.1.x | HTML → Markdown on the desktop side (REQ-1.3, AI-generated .html import) | Turndown-inspired Rust converter; keeps HTML import in the Rust core. (The Chrome extension uses turndown.js separately — see below.) |
| **keyring** crate (via **tauri-plugin-keyring**) | keyring-core based | API keys in the OS keychain (PRD §5, macOS Keychain / Windows Credential Manager) | Native secure storage directly from Rust, no master-password prompt. **Prefer this over `tauri-plugin-stronghold`** — Stronghold is officially slated for deprecation/removal in Tauri v3. |
| **reqwest** | 0.12.x | HTTP client for LLM calls (Rust core) | Async, TLS, streaming bodies. Keys never leave the Rust process (read from keychain). Supports custom `base_url` for OpenAI-compatible / proxy / local endpoints (PRD §5). |
| **eventsource-stream** | 0.2.x | SSE parsing for streaming LLM responses | Parses Anthropic `content_block_delta` / OpenAI `chat.completion.chunk` SSE for per-segment streaming Lens rendering (REQ-2.8). |
| **async-openai** | 0.27.x | OpenAI-compatible client (streaming, base_url) | Mature Rust client; `with_config` supports custom base_url → covers OpenAI + the long tail of OpenAI-compatible providers/proxies/local models in one client. |
| **tiktoken-rs** | 0.6.x | Local token counting for OpenAI-family models (o200k_base) | In-process token estimate for cost display (REQ-2.9, F7 token totals) without a network round-trip. For Claude, call Anthropic's `messages/count_tokens` REST endpoint for exact counts. |
| **tokio** | 1.x | Async runtime | Required by rmcp, reqwest, notify debouncer; Tauri v2 is async-native. |

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

---

## Installation

```bash
# --- Desktop app (Tauri v2 scaffold) ---
npm create tauri-app@latest        # choose React + TypeScript + Vite

# Webview UI deps
npm install react@19 react-dom@19 react-markdown
npm install @codemirror/state @codemirror/view @codemirror/lang-markdown

# Tauri plugins (JS side)
npm install @tauri-apps/api@2

# Rust core deps (src-tauri/Cargo.toml)
#   tauri = "2.10"
#   rusqlite = { version = "0.32", features = ["bundled"] }
#   rusqlite_migration = "1"
#   comrak = "0.54"
#   similar = "2"
#   blake3 = "1"
#   gray_matter = "0.2"
#   htmd = "0.1"
#   notify = "8"
#   notify-debouncer-full = "0.4"
#   reqwest = { version = "0.12", features = ["json", "stream"] }
#   eventsource-stream = "0.2"
#   async-openai = "0.27"
#   tiktoken-rs = "0.6"
#   rmcp = { version = "2.2", features = ["server", "transport-io"] }
#   tokio = { version = "1", features = ["full"] }
#   keyring = "3"   # or tauri-plugin-keyring

# --- Chrome MV3 extension (separate package) ---
npm install -D wxt
npm install @mozilla/readability turndown turndown-plugin-gfm gpt-tokenizer
```

---

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

---

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| **Electron (as default)** | ~150 MB bundle + high idle RAM on an always-on local-first app; no Rust-engine benefit. | Tauri v2. |
| **tauri-plugin-stronghold** for API keys | Officially slated for deprecation and removal in Tauri v3; you'd be building on a dead-end. | `keyring` crate / `tauri-plugin-keyring` (native OS keychain). |
| **Writing Block IDs into user `.md` files** | Violates the hard "don't pollute source" constraint (PROJECT.md, PRD §2.4); breaks git/IDE/agent workflows. | Store all Block IDs/comments/cards in the sidecar SQLite DB; materialize frontmatter only on OKF export. |
| **Two markdown parsers for anchoring** (comrak in Rust *and* remark in JS both authoritative) | Non-deterministic Block boundaries across parsers → silent anchor drift, the #1 moat risk. | comrak (Rust) is the single source of truth for Block boundaries/IDs; remark/react-markdown is render-only. |
| **Storing the metadata DB inside the repo's `.prismdocs/`** | Would create git noise and risk committing private comments/cards; conflicts with Q3's sidecar decision. | Keep the SQLite DB in the app data dir (`~/Library/Application Support/PrismDocs/`, keyed by project path). Reserve in-repo `.prismdocs/feedback` + `.prismdocs/context` only for agent-handoff files (gitignored by default per PRD §4.1). |
| **Binding the MCP/bridge servers to `0.0.0.0`** | Security: PRD requires loopback-only, workspace-scoped, read-mostly. | Bind strictly to `127.0.0.1` (or a Unix domain socket in the app data dir); scope to current workspace; only `respond_to_comment` writes. |
| **js-tiktoken in the extension** | ~200 KB, heavier than needed for a ±10% estimate in an MV3 worker. | gpt-tokenizer (o200k_base, ~50 KB). |
| **`chrono` for time / naive FS polling** | Polling misses agent burst-writes and wastes CPU; use event-based watching. | `notify` + `notify-debouncer-full` (event-based, debounced). |

---

## Stack Patterns by Variant

**If the team is Rust-comfortable (recommended default):**
- Rust core owns MCP (rmcp) and LLM (async-openai + thin reqwest Anthropic client, tiktoken-rs for local counts).
- Zero Node runtime shipped → smallest bundle, unified toolchain.
- Cost: hand-roll the Anthropic SSE client (the API is simple: `message_start` / `content_block_delta` / `message_stop`).

**If the team is JS-first or wants fastest MCP/LLM velocity:**
- Bundle one Node sidecar hosting both the MCP server (`@modelcontextprotocol/sdk`) and LLM clients (`@anthropic-ai/sdk`, `openai`).
- Rust core still owns FS watch, SQLite, parse, anchoring, keychain.
- Cost: +~40-80 MB Node runtime (still far below Electron); IPC between Rust core and the Node sidecar (loopback).

**When Windows P1 arrives:**
- Tauri: mostly free. Swap keyring backend (Windows Credential Manager — the `keyring` crate handles this transparently). Verify `notify` on Windows (ReadDirectoryChangesW) rename semantics. Re-sign/package for Windows.

---

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| comrak 0.54 | Rust 1.85+ | Hard MSRV; set toolchain accordingly. |
| Tauri 2.10 | `@tauri-apps/api` 2.x, `@tauri-apps/cli` 2.x | Keep JS API and Rust crate on the same major (v2). |
| rusqlite (`bundled`) | — | `bundled` feature statically links SQLite; no system libsqlite needed → reproducible builds across macOS/Windows. |
| rmcp 2.2 | MCP protocol 2025-11-25 (stable) + 2026-07-28 (draft) | **Pin to the 2025-11-25 stable protocol for launch;** upgrade to the new spec only after Claude Code / Cursor adopt it (new spec lands 2026-07-28, one day out). |
| @modelcontextprotocol/sdk (TS) | v1.29.0 = stable; v2 = beta | If you go the Node-sidecar route, ship v1.x for production; v2 API can still change until 2026-07-28. |
| gpt-tokenizer o200k_base | GPT-4o/4.1/5, o1/o3/o4 | Correct encoding for current OpenAI models; use as the universal extension estimate. |

---

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

---
*Stack research for: local-first desktop doc workbench + MV3 extension + embedded MCP*
*Researched: 2026-07-27*
