# Stack Research

**Domain:** Local-first AI-era engineering documentation workbench (macOS desktop: Tauri v2 + pure-Rust engine workspace + React; Markdown block anchoring, SQLite FTS5, FS watching, streaming LLM, embedded loopback MCP server)
**Researched:** 2026-07-28
**Confidence:** HIGH (all version facts verified today against crates.io API / npm registry / docs.rs directly — primary sources, not training data)

## Verdict on the Pinned Stack

**The project's pinned stack holds up.** Of 20 pinned choices, 18 are CONFIRMED as current and mutually compatible, 1 is AMENDED (Vite 7 → Vite 8), and 1 needs a CLARIFICATION (keyring 4's usage model changed structurally from v3). No deprecated or dangerous choices found. Dependency-tree alignment was stress-tested at the three risk points (rusqlite, reqwest, axum) and passes cleanly — `cargo tree -d` (Phase 1 exit criterion) should show no duplicate rusqlite or reqwest.

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

keyring 4.1.5's feature set is just `["v1", "cli"]` — v4 restructured the crate into a facade over a new `keyring-core` (1.0.0, 2026-04-21) + per-platform store crates. The crate's own docs state: applications that want to control which credential store they use "should not be linking to this library at all; they should instead be linking to the keyring-core library and any specific credential stores." Two valid concretizations of the project's pin:

1. **Recommended:** `keyring-core = "1.0"` + `apple-native-keyring-store = "1.0"` — explicit macOS Keychain, minimal dependency surface, matches the macOS-only MVP and upstream guidance.
2. **Acceptable:** `keyring = { version = "4.1", features = ["v1"] }` — classic cross-platform Entry API; slightly more deps, but eases the Windows P1 port.

Either satisfies NFR-03 (service=PrismDocs, two accounts: LLM key + MCP token). Decide in Phase 1; do not use keyring 4 with default features (it exposes nothing useful without `v1` or `cli`).

## Installation

```toml
# Engine workspace (root Cargo.toml [workspace.dependencies]) — verified 2026-07-28
rusqlite            = { version = "0.40", features = ["bundled"] }  # FTS5 included in bundled
r2d2                = "0.8"
r2d2_sqlite         = "0.35"
rusqlite_migration  = "2.6"
notify              = "8.2"
notify-debouncer-full = "0.7"
comrak              = "0.54"
blake3              = "1.8"
similar             = "3.1"
reqwest             = { version = "0.13", features = ["json", "stream"] }
async-openai        = "0.41"
eventsource-stream  = "0.2"
keyring-core        = "1.0"
apple-native-keyring-store = "1.0"      # or: keyring = { version = "4.1", features = ["v1"] }
rmcp                = { version = "2.2", features = ["server", "transport-streamable-http-server"] }
axum                = "0.8"
gray_matter         = "0.3"
htmd                = "0.5"
tiktoken-rs         = "0.12"
ulid                = "1"
dirs                = "6"
tauri               = "2"               # thin shell crate only
```

```bash
# Frontend
npm create tauri-app@latest   # then align to:
npm install react@19 react-dom@19 react-markdown@10 codemirror@6
npm install -D vite@8 @vitejs/plugin-react @tauri-apps/cli@2 typescript
```

Note: verify rmcp feature-flag names (`server`, `transport-streamable-http-server`) against the rmcp 2.2 README at Phase 1 — the module path is confirmed on docs.rs, but rmcp has renamed features across its rapid 1.x→2.x releases (MEDIUM confidence on exact flag names only).

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

**If Anthropic-native API support is in scope for prism-llm (likely, given Claude Code users):**
- Use reqwest 0.13 + eventsource-stream for the Messages API SSE stream; async-openai only for OpenAI-compatible base_url endpoints
- Because async-openai's types don't model Anthropic's native event shapes (`content_block_delta` etc.), and both crates share the same reqwest tree so cost is zero

**If FTS5 must search Chinese card/comment text (F5 cards are Chinese):**
- Create the FTS5 table with `tokenize = 'trigram'` for CJK columns (or dual-index: unicode61 for English Base docs, trigram for Chinese cards/comments)
- Because the default unicode61 tokenizer does not segment CJK — Chinese search silently degrades to whole-string matching. This is a schema-v1 decision (Phase 1), painful to retrofit

**If the 300ms P95 migration budget fails on large docs during Phase 3 calibration:**
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

---
*Stack research for: PrismDocs — local-first Tauri v2 + Rust engineering docs workbench*
*Researched: 2026-07-28*
