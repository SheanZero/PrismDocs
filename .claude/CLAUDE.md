<!-- GSD:project-start source:PROJECT.md -->

## Project

**PrismDocs**

PrismDocs 是 vibe coder 的多项目工程知识工作台（macOS 桌面应用，Tauri v2）：集中管理 Claude Code / Codex 等编码 agent 在各仓库生成的英文技术文档（Base 层，唯一真相源、OKF 兼容、紧凑省 token），为每份文档生成中文速读区（3–5 句摘要 + ❓ 需决策清单 + 变更摘要）；用户在文档上写段落级评论，评论结构化回流给 AI 驱动下一轮迭代，每次变更可追溯到驱动它的评论；项目间文档相互引用、契约可订阅——上游（如 server 的 API spec）变更时自动警示下游核对，让独立开发的多个仓库不产生偏差。

面向 P1 独立 vibe coder（中文母语、英文技术阅读能力中等，用 Claude Code / Cursor 开发）；P2 产品型创始人为次要人群（随全文 Lens P1 回归再覆盖）。

**Core Value:** **「评论 → AI 修改 → 复核通过」的闭环**：用户能在 10 分钟内看懂 AI 写的英文文档、批注两句让它接着干、且评论在 AI 大规模重写下 0 静默丢失。北极星指标 = 每周闭环数（跨项目闭环计入）。

### Constraints

- **Tech stack**（调研 2026-07-28 对照 crates.io 验证 pin）: Tauri v2 + React 19 + Vite 7（前端仅渲染）；纯 Rust Engine Facade workspace（不依赖 tauri，可独立测试，D-01）：prism-store（rusqlite 0.40 + r2d2 + FTS5 + rusqlite_migration）、prism-fs（notify 8 + debouncer-full）、prism-parse（comrak 0.54 sourcepos）、prism-anchor（blake3 + similar 3.1）、prism-llm（reqwest 0.13 + async-openai + eventsource-stream + keyring 4.1）、prism-mcp（rmcp 2.2 StreamableHttpService + axum 0.8）
- **锚定真相源**: comrak 是唯一 Block 边界真相源，前端 react-markdown 仅渲染——两个 parser 各自分块必然锚点漂移（What-NOT-to-use 首条）
- **不污染原则**: Block ID / 评论 / Xref / 元数据全部存 sidecar（~/Library/Application Support/PrismDocs/，按 project-id 索引，D-13）；用户 repo 内仅 .prismdocs/ 协议产物；frontmatter 写回按字节保留原块只替换正文（round-trip `git diff` 0 变化）
- **MCP 传输 (D-07)**: app 自身托管 loopback streamable HTTP（127.0.0.1）+ per-install bearer token（钥匙串）+ Origin allowlist，无子进程；配套轻量 CLI helper（headersHelper + SessionStart hook check-feedback）；子 PRD-F4 早期 stdio 方案已作废
- **性能**: 500 文档/2000 卡片下搜索 <300ms、文档打开 <500ms、FS 变更呈现 <10s；单文档锚点迁移 P95 <300ms
- **隐私/密钥**: 本地优先、断网可读可评可写卡；API key 存系统钥匙串（keyring 直连，不用 stronghold）；文档内容仅发送到用户配置的 LLM 端点；埋点 opt-in 本地暂存
- **发布门槛**: 锚点 0 静默丢失（AC-3b）是 MVP 发布标准，非普通验收项
- **Timeline 参照**: BRD M1 = 6–8 周（对应 Phase 1–6）；MVP′ 缩围后基本可达

<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->

## Technology Stack

## Verdict on the Pinned Stack

## Recommended Stack

### Core Technologies (Rust engine workspace)

| Technology | Version (crates.io, 2026-07-28) | Pinned | Verdict | Why |
|------------|--------------------------------|--------|---------|-----|
| tauri | 2.11.5 | v2 | **CONFIRM** | Only sane Rust-native desktop shell; thin-shell design (D-01) keeps engine testable without it |
| rusqlite | 0.40.1 | 0.40 | **CONFIRM** | Pin is the current minor. Use `features = ["bundled"]` — verified in libsqlite3-sys build.rs that bundled builds compile with `-DSQLITE_ENABLE_FTS5` unconditionally, so FTS5 needs no extra feature flag |
| r2d2 + r2d2_sqlite | 0.8.10 / 0.35.0 | r2d2 | **CONFIRM** | r2d2_sqlite 0.35.0 (2026-07-06) depends on `rusqlite ^0.40` — exactly aligned with the pin, no duplicate rusqlite in the tree |
| rusqlite_migration | 2.6.0 | (unversioned pin) | **CONFIRM** | Current stable 2.6.0 (2026-05-28); pin `2.x` |
| notify + notify-debouncer-full | 8.2.0 / 0.7.0 | notify 8 + debouncer-full | **CONFIRM** | debouncer-full 0.7.0 depends on `notify ^8.2` — aligned. FSEvents backend on macOS. REQ-1.4.3 merge semantics remain self-built on top, as the 调研 doc already states |
| comrak | 0.54.0 | 0.54 | **CONFIRM** | Pin is the latest release (2026-07-12). Sourcepos on AST nodes + `front_matter_delimiter` cover TD-01 §3.1 needs. TD-01's `parse_options_version` escape hatch is the right mitigation for future comrak major bumps |
| blake3 | 1.8.5 | 1.x | **CONFIRM** | Current stable; linear-time content hashing fits the P95 <300ms migration budget |
| similar | 3.1.1 | 3.1 | **CONFIRM** | Pin is current minor (patch 3.1.1). TextDiff ratio + Myers/patience choice left as implementation freedom per TD-01 §1 — correct |
| reqwest | 0.13.4 | 0.13 | **CONFIRM** | Current minor. Critically: both async-openai 0.41 (`reqwest ^0.13`) and rmcp 2.2 (`reqwest ^0.13.2`, optional client-side) now sit on 0.13 — single reqwest/hyper tree |
| async-openai | 0.41.1 | (unversioned pin) | **CONFIRM** | 0.41.1 (2026-06-18); depends on `reqwest ^0.13` and `eventsource-stream ^0.2` — both aligned with project pins. `with_api_base` covers user-configured OpenAI-compatible endpoints |
| eventsource-stream | 0.2.3 | (unversioned pin) | **CONFIRM** (with note) | Unchanged since 2022 but it is a tiny, spec-stable SSE parser — dormancy is not risk here. Needed for non-OpenAI-shaped streams (e.g. Anthropic Messages API native SSE); async-openai uses the same crate internally so it is in-tree anyway |
| keyring | 4.1.5 | 4.1 | **CONFIRM with CLARIFICATION** (see below) | Version pin correct; v4 usage model differs from v3 |
| rmcp | 2.2.0 | 2.2 | **CONFIRM** | 2.2.0 released 2026-07-08. `transport/streamable_http_server` module verified present on docs.rs; official examples mount it into `axum ^0.8` (axum is rmcp's dev-dependency for exactly this pattern) — D-07's StreamableHttpService-on-axum design is the upstream-blessed shape |
| axum | 0.8.9 | 0.8 | **CONFIRM** | Current major; hyper 1 / http 1 based, matching rmcp's optional `hyper ^1` / `http ^1` deps. Same instance can host the clip WebSocket endpoint later (F6, P1) |
| gray_matter | 0.3.2 | (unversioned pin) | **CONFIRM** (with note) | Last release 2025-07-10 — ~1 year dormant, but the job (frontmatter parse) is small and stable. The real AC-1g-2 risk is write-back, and the byte-preserving strategy (never re-serialize the frontmatter block) is already mandated in 调研 §2.2 — keep that rule, the crate choice is then low-stakes |
| htmd | 0.5.5 | (unversioned pin) | **CONFIRM** | Actively maintained (updated 2026-07-27). HTML→MD for F1 import |
| tiktoken-rs | 0.12.0 | (unversioned pin) | **CONFIRM** | Current; o200k_base supported. Matches gpt-tokenizer (npm 3.4.0) o200k for the ≤10% cross-surface token-count tolerance (AC-6c/7b — extension side is P1 but the desktop-side口径 is set now) |

### Core Technologies (frontend)

| Technology | Version (npm, 2026-07-28) | Pinned | Verdict | Why |
|------------|---------------------------|--------|---------|-----|
| React | 19.2.8 | 19 | **CONFIRM** | Current stable line |
| Vite | 8.1.5 | 7 | **AMEND → Vite 8** | Vite 8 went stable 2026-03-12 with Rolldown as the unified Rust bundler; Rolldown itself hit 1.0 (locked API) 2026-05-07. Four months of stable patch releases (now 8.1.5), a compat layer auto-converts esbuild/rollupOptions config, and this project's frontend is a thin render-only layer with a minimal plugin surface (@vitejs/plugin-react) — the classic "wait for plugin ecosystem" reason to stay on 7 doesn't apply. Starting a greenfield on the previous major in this window just schedules a migration for later. Fallback: Vite 7 remains fine if any Tauri-template friction appears; `rolldown-vite` on 7 is the documented intermediate step |
| react-markdown | 10.1.0 | (render only) | **CONFIRM** | Render-only per the anchoring truth-source rule — see What NOT to Use |
| CodeMirror | 6 (meta 6.0.2) | 6 | **CONFIRM** | Base editing; the `@codemirror/*` package family is the actual dependency set |
| @tauri-apps/api / cli | 2.11.1 / 2.11.4 | v2 | **CONFIRM** | Matches tauri 2.11.x crate line |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| keyring-core + apple-native-keyring-store | 1.0.0 / 1.0.1 | Explicit macOS Keychain access | Preferred concrete form of the "keyring 4.1" pin — see clarification below |
| dirs | 6.x | `data_dir()` for the sidecar root (D-13 mandates dirs::data_dir, not Tauri's app_data_dir) | prism-store, Phase 1 |
| ulid | 1.x | Block ID allocation (TD-01 §2: opaque stable ULID) | prism-anchor |
| serde / serde_json / thiserror / tracing / tokio | current | Standard plumbing; already forced by rmcp/axum/async-openai | workspace-wide |
| tauri-plugin-dialog | 2.x | Native folder picker for import wizard (no App Sandbox, so plain picker suffices) | Phase 2 (F1) |

### keyring 4 clarification (affects Phase 1)

## Installation

# Engine workspace (root Cargo.toml [workspace.dependencies]) — verified 2026-07-28

# Frontend

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| comrak (CommonMark+GFM, sourcepos) | pulldown-cmark | Faster, but event-based with weaker AST/sourcepos ergonomics; TD-01's whole design (root-level AST nodes + byte spans) is comrak-shaped. No reason to switch |
| similar (TextDiff ratio) | imara-diff, strsim | imara-diff if Step-3 O(u×v) scoring ever blows the 300ms budget on pathological docs (unlikely — residual sets are small); strsim only if a cheap prefilter is wanted |
| rusqlite + FTS5 | tantivy | Tantivy only if search requirements outgrow FTS5 (500 docs / <300ms is trivially within FTS5's envelope; tantivy would add a second index lifecycle for nothing) |
| async-openai (OpenAI-compatible endpoints) | hand-rolled reqwest client | Hand-roll only the Anthropic-native path (Messages API SSE via reqwest + eventsource-stream); don't hand-roll the OpenAI-compatible path — async-openai's retry/types are free |
| rmcp official SDK | mcp-sdk-rs / community crates | Never — rmcp is the official modelcontextprotocol Rust SDK, tracks spec revisions (streamable HTTP transport) fastest |
| Vite 8 | Vite 7 (+ optional rolldown-vite) | If a Tauri template or a required plugin shows Vite-8 friction in week 1, drop to 7 — zero architectural impact, frontend is render-only |
| keyring-core + apple store | keyring 4 `v1` feature | When Windows port (P1) starts, the `v1` facade or adding `windows-native-keyring-store` are both cheap paths |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| react-markdown / remark as anchoring source | Two parsers segment differently → anchor drift is guaranteed; this is the project's own What-NOT-to-use #1 and it is correct | comrak as the sole Block-boundary truth source; frontend renders only |
| tauri-plugin-stronghold | Deprecated for Tauri v3; heavyweight for two secrets | keyring-core + Keychain (above) |
| tauri-plugin-keyring | Thin third-party wrapper, lags upstream keyring; engine crates shouldn't depend on Tauri anyway (D-01) | Direct keyring-core in prism-llm |
| serde_yaml (for frontmatter) | Archived/unmaintained | gray_matter (parse) + byte-preserving write-back (never re-serialize) |
| stdio MCP proxy subprocess | Already rejected (D-07); two transport versions = doc drift, and F4 sub-PRD still needs the回改 | rmcp StreamableHttpService on loopback axum + bearer token |
| rusqlite `bundled-full` or system SQLite | bundled-full drags unneeded extensions; system SQLite versions vary across macOS releases | `bundled` (FTS5 verified included) |
| tokio-rusqlite / sqlx | Async SQLite adds ceremony without benefit for a local single-user app; sqlx compile-time checks don't fit dynamic FTS5 queries well | rusqlite + r2d2 pool, blocking calls on tokio blocking threads |
| eventsource-client / reqwest-eventsource as extra SSE deps | Redundant — eventsource-stream is already in-tree via async-openai | eventsource-stream directly |

## Stack Patterns by Variant

- Use reqwest 0.13 + eventsource-stream for the Messages API SSE stream; async-openai only for OpenAI-compatible base_url endpoints
- Because async-openai's types don't model Anthropic's native event shapes (`content_block_delta` etc.), and both crates share the same reqwest tree so cost is zero
- Create the FTS5 table with `tokenize = 'trigram'` for CJK columns (or dual-index: unicode61 for English Base docs, trigram for Chinese cards/comments)
- Because the default unicode61 tokenizer does not segment CJK — Chinese search silently degrades to whole-string matching. This is a schema-v1 decision (Phase 1), painful to retrofit
- Swap similar's Step-1 sequence diff for imara-diff and/or add a blake3-prefix bucketing prefilter before Step 3
- Because TD-01 explicitly leaves algorithm internals unfrozen — the contract survives the swap

## Version Compatibility

| Package A | Compatible With | Verified How |
|-----------|-----------------|--------------|
| r2d2_sqlite 0.35.0 | rusqlite ^0.40 | crates.io dependency manifest, 2026-07-28 — no duplicate rusqlite |
| notify-debouncer-full 0.7.0 | notify ^8.2.0 | crates.io dependency manifest |
| async-openai 0.41.1 | reqwest ^0.13, eventsource-stream ^0.2 | crates.io dependency manifest — single reqwest tree with project pin |
| rmcp 2.2.0 | reqwest ^0.13.2 (optional), hyper ^1 / http ^1, axum ^0.8 (dev-dep, mounting pattern) | docs.rs rmcp 2.2.0 page |
| rusqlite 0.40 `bundled` | FTS5 | libsqlite3-sys build.rs: `-DSQLITE_ENABLE_FTS5` unconditional in bundled |
| tiktoken-rs 0.12 (o200k_base) | gpt-tokenizer 3.4 (o200k_base) | Same encoding family — satisfies ≤10% cross-surface tolerance |
| Vite 8.1.5 | React 19.2, @vitejs/plugin-react, Tauri v2 dev server | Vite 8 stable since 2026-03; Tauri is bundler-agnostic (dev server URL + dist dir) |

## Confidence Assessment

| Claim | Confidence | Basis |
|-------|-----------|-------|
| All crate/npm version pins current | HIGH | Direct crates.io API / npm registry queries, 2026-07-28 |
| Dependency-tree alignment (no dup rusqlite/reqwest) | HIGH | crates.io dependency manifests of the exact pinned versions |
| FTS5 in rusqlite bundled | HIGH | libsqlite3-sys build.rs source read |
| rmcp 2.2 StreamableHttpService + axum 0.8 pattern | HIGH | docs.rs module page + rmcp dev-deps; exact feature-flag names MEDIUM (verify at Phase 1) |
| Vite 8 amendment | MEDIUM-HIGH | Official vite.dev release posts (stable 2026-03-12, Rolldown 1.0 2026-05-07); ecosystem-maturity judgment, with documented Vite 7 fallback |
| keyring v4 usage-model clarification | HIGH | keyring 4.1.5 docs.rs intro text + crates.io feature list + keyring-core/apple-native-keyring-store existence |
| CJK trigram tokenizer note | MEDIUM | SQLite FTS5 documented behavior (unicode61 vs trigram); not yet tested against project corpus |

## Sources

- crates.io API (`/api/v1/crates/*` + `/dependencies`) — all 19 Rust crate versions + dependency manifests, queried 2026-07-28 (HIGH, official registry)
- npm registry (`npm view`) — react 19.2.8, vite 8.1.5, react-markdown 10.1.0, @tauri-apps/api 2.11.1, @tauri-apps/cli 2.11.4, codemirror 6.0.2, gpt-tokenizer 3.4.0 (HIGH, official registry)
- docs.rs — rmcp 2.2.0 `streamable_http_server` module + dep list; keyring 4.1.5 crate intro (HIGH, generated from published source)
- github.com/rusqlite/rusqlite — libsqlite3-sys build.rs FTS5 flags (HIGH, source)
- vite.dev release blog — [Vite 8.0 is out](https://vite.dev/blog/announcing-vite8), [Vite 8 Beta](https://vite.dev/blog/announcing-vite8-beta), [Migration from v7](https://vite.dev/guide/migration); [InfoQ coverage](https://www.infoq.com/news/2026/05/vite-v8-rust/) (MEDIUM-HIGH, official project blog via web search)

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
