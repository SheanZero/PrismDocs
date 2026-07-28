# Architecture Research

**Domain:** Local-first desktop knowledge app — Tauri v2 shell + pure-Rust engine workspace (SQLite sidecar, FS watcher pipeline, Markdown block anchoring, streaming LLM, embedded loopback MCP server)
**Researched:** 2026-07-28
**Confidence:** MEDIUM-HIGH (proposed architecture cross-checked against official Tauri v2 docs, rmcp/rust-sdk patterns, SQLite WAL practice, notify-debouncer-full docs, and annotation-anchoring prior art; all findings converge)

## Verdict Up Front

The architecture proposed in `docs/调研_技术基建与开发Phase.md` §2.1 (Engine Facade + 6 domain crates + thin Tauri shell) **matches the established best-practice shape** for Tauri v2 apps with substantial Rust logic. The build order (1 skeleton → 2 import/sync → 3 anchoring → 4 digest ∥ 5 comments → 6 feedback loop → 7 cards/pack/cross-project → release prep) is **dependency-correct and confirmed**, with three amendments recommended below (§Build Order): pull the event-bus spine into Phase 1, allow the anchoring pure core to start during Phase 2, and sequence prism-llm transport as the first deliverable inside Phase 4.

## Standard Architecture

### System Overview

This is the validated shape (engine-side detail added to the doc's §2.1 diagram):

```
┌─ PrismDocs.app (Tauri v2) ──────────────────────────────────────────────┐
│  React 19 WebView — rendering only (react-markdown renders; NEVER       │
│  anchors). State = notify-then-fetch + streamed channels.               │
│  ────────────────────── Tauri IPC boundary ──────────────────────────── │
│  Shell (src-tauri): commands = thin delegates to facade; adapter maps   │
│  engine bus → ① Tauri events (coarse invalidation) ② Channels (streams) │
│  ────────────────────────────────────────────────────────────────────── │
│  prism-engine (Facade crate) — owns orchestration, internal event bus   │
│  (tokio broadcast), service traits; the ONLY thing the shell calls      │
│   ├─ prism-store   SQLite WAL: 1 writer conn + read pool, FTS5,         │
│   │                migrations. All writes serialized through it.        │
│   ├─ prism-fs      notify 8 + debouncer-full; echo-suppression          │
│   │                primitive; 5-min reconciliation scan                 │
│   ├─ prism-parse   comrak (pinned options) → Block tree ← SINGLE        │
│   │                anchoring truth source                               │
│   ├─ prism-anchor  pure migrate(old_tree, new_tree) → MigrationResult   │
│   │                (TD-01 frozen contract; 4 consumers)                 │
│   ├─ prism-llm     reqwest + SSE streaming + retry/backoff + keyring;   │
│   │                the ONLY network egress + ONLY key access            │
│   └─ prism-mcp     rmcp StreamableHttpService on axum @127.0.0.1        │
│                    bearer token + Origin allowlist; depends on service  │
│                    traits, NOT on prism-engine (avoid cycle)            │
└─────────────────────────────────────────────────────────────────────────┘
   sidecar data: ~/Library/Application Support/PrismDocs/ (by project-id)
   user repo: only .prismdocs/ protocol artifacts (feedback/ context/)
   + tiny CLI helper binary (workspace member): keyring read (headersHelper)
     + check-feedback hook — links NO engine crates, just keyring + HTTP
```

**Why this shape is the established pattern (not just this project's invention):** Tauri v2's own project-structure docs endorse `src-tauri` as one member of a larger Cargo workspace; community reference apps (e.g. Yerd) follow exactly this discipline — pure core crates with zero `tauri` dependency, side effects behind traits, binaries thin ("wiring, not behavior"). The D-01 decision (engine testable without Tauri) is the mainstream 2026 answer, not an exotic one.

### Component Responsibilities

| Component | Responsibility | Boundary rule |
|-----------|----------------|---------------|
| React WebView | Rendering, interaction, optimistic UI | Never parses for anchoring; never talks to engine crates directly — IPC only |
| Tauri shell (`src-tauri`) | Window/tray/lifecycle, path injection, command registration, bus→IPC adapter | No business logic in `#[tauri::command]` bodies; each command ≤ a facade call |
| `prism-engine` (facade) | Orchestration, internal event bus, transaction choreography (e.g. version→migrate→persist→publish), service trait impls | Only crate the shell depends on; owns the tokio runtime handles for background tasks |
| `prism-store` | Schema, migrations, FTS5, connection discipline (single writer + read pool), version snapshots | No other crate opens SQLite connections; all writes flow through its writer |
| `prism-fs` | Watch, debounce (2s/10s), rename stitching, echo suppression primitive, reconciliation scan, atomic self-writes | The ONE place that both reads FS events and registers self-writes (调研 §2.3-2 confirmed correct) |
| `prism-parse` | comrak with pinned `PARSE_OPTIONS_V1` → Block tree with sourcepos | Single truth source for block boundaries (TD-01 §3) |
| `prism-anchor` | Pure migration algorithm + confidence tiers + `MigrationResult`/`ChangeSet` | Pure function over trees; no I/O, no store dependency — this is what makes the calibration harness cheap |
| `prism-llm` | Streaming completion, retry/429 backoff, token counting, keyring access | Only network egress; only key reader (NFR-03) |
| `prism-mcp` | MCP tool surface (`list_feedback` etc.), session mgmt, bearer auth, Origin allowlist | Depends on service **traits** (defined in a small shared crate or in prism-store), implemented by facade — never on `prism-engine` itself |
| CLI helper | `headersHelper` (keyring→token), SessionStart `check-feedback` | Separate tiny binary; keyring + HTTP client only |

### The one dependency-graph trap to design around

`prism-mcp` needs to answer tool calls with data owned by the facade layer (feedback bundles, comments, context packs). If it imports `prism-engine`, and `prism-engine` hosts the server, you get a cycle. Two clean resolutions, pick one in Phase 1:

1. **Trait inversion (recommended):** `prism-mcp` defines (or imports from a small `prism-types` crate) traits like `FeedbackSource` / `CommentSink`; `prism-engine` implements them and hands `Arc<dyn …>` in at server construction. rmcp's per-session service factory takes this naturally.
2. Direct `prism-store` dependency: simpler, but leaks storage schema into the protocol layer and makes the "writes limited to comment receipts" security invariant harder to enforce centrally.

## Architectural Patterns

### Pattern 1: Notify-then-fetch over the IPC boundary (engine → WebView)

**What:** The internal engine bus (tokio broadcast of typed events: `DocChanged`, `MigrationCompleted{doc_id}`, `InboxUpdated`, `DriftAlert`) is adapted by the shell into **coarse, small Tauri events** carrying only IDs/counters. The frontend reacts by invoking a query command to fetch fresh state. Streams (LLM tokens for digest generation, long imports) use `tauri::ipc::Channel` instead.

**Why (verified against Tauri v2 docs):** Tauri events are JSON-string payloads with no strong typing, no capability scoping, and — critically — **async listeners can process rapid events out of order**. Official guidance is explicit: for ordered, high-throughput delivery use Channels. So: never push `MigrationResult` or diffs through events; push "doc X changed, unread=3" and let the UI pull.

**Trade-offs:** One extra round-trip per invalidation — negligible on loopback IPC; buys ordering safety, small payloads, and a frontend that can't drift from store truth.

**Channel caveat:** Channels are created frontend-side and passed into a command invocation — they fit request-scoped streams (digest generation the user just triggered). For engine-initiated pushes there is no standing channel; that's exactly why the coarse-event + fetch pattern covers the FS-driven flows.

### Pattern 2: Single-writer SQLite discipline (WAL)

**What:** WAL mode + `busy_timeout` on every connection + `synchronous=NORMAL`; **one dedicated writer connection** (mutex-guarded or a dedicated writer task consuming a command queue) + an r2d2 read pool.

**Why (verified):** WAL gives many readers + exactly one writer. A naive uniform r2d2 pool lets multiple connections attempt writes, producing `SQLITE_BUSY` storms and degraded write latency — a documented, common failure mode. Mirroring SQLite's own concurrency model at the app level (read pool / single writer) is the consensus fix.

**Project-specific consequence:** TD-01 §5.1 requires version-persist + migration + `migration_log` in one transaction tail. That transaction plus FTS5 index updates must all flow through the single writer. Make the writer an explicit facade-owned handle in Phase 1 so no later phase "just grabs a pooled connection" to write.

### Pattern 3: Echo-suppressed, reconciled FS pipeline

**What:** `disk → notify → debouncer-full (2s window / 10s cap) → echo filter → identity/version → store → anchor migrate → bus`. Self-writes (in-app Base edit, `log.md` materialization, `.prismdocs/` writes) register `(path, content_hash)` in prism-fs **before** writing; the watcher drops matching events. A periodic (5-min) reconciliation scan re-hashes and repairs anything the watcher missed.

**Why (verified):** notify-debouncer-full handles the hard mechanical parts — rename From/To stitching into a single `Rename` event, path fix-up for pending events across renames, FS-ID tracking on FSEvents/Windows, and merge of high-frequency writes. It does **not** provide self-write suppression; that is app-level by design, which confirms 调研 §2.3-2's "one unified primitive in prism-fs, not three ad-hoc implementations." Editor/agent atomic saves (tmp-write + rename) surface as rename events post-delay — the identity layer (content-hash based, REQ-1.NEW-1) must treat them as modifications of the same document, which the design already does.

**Correctness risks to test explicitly (Phase 2):** (a) hash-registry entries must expire/limit, or a failed self-write leaks a permanent suppression that later swallows a genuine external edit with identical content; (b) FSEvents coalescing can drop events under burst — the reconciliation scan is the safety net, keep it in scope, don't cut it; (c) the 10s debounce cap + serialized per-document migration (TD-01 §9-12) is the right answer for "agent writes continuously" — verify the version chain never skips.

### Pattern 4: Embedded loopback MCP server as an app-lifetime service

**What:** rmcp `StreamableHttpService` mounted on an axum router bound to `127.0.0.1:<port>`, spawned on the tokio runtime in Tauri's `setup` hook; graceful shutdown tied to app exit; bearer-token check + Origin allowlist as axum middleware layers ahead of the MCP route.

**Why (verified):** This is the documented rmcp pattern (per-session service factory, session management built in; bearer middleware on the router). Loopback-bind + Origin validation is also what the MCP spec requires of HTTP transports (DNS-rebinding defense) — D-07's design is aligned with both.

**Lifecycle details the docs don't decide for you (design in Phase 6, note in Phase 1):**
- **Port strategy + discovery:** fixed preferred port with fallback scan; persist the actual port where the CLI helper can find it (e.g. a small file in Application Support). D-07 killed `.prismdocs/mcp.json`, so the helper — not a repo file — is the discovery mechanism. This must be pinned before writing the Claude Code install guide.
- **Desktop-app-not-running path:** helper/hook must fail with a clear message ("start PrismDocs"), and the file protocol remains the degraded loop (AC-4c). Already in the PRD; keep it in Phase 6 acceptance.
- **Session ↔ DB budget:** per-session rmcp instances must share the read pool and route their one write kind (`respond_to_comment`) through the single writer — natural if Pattern 2's writer handle is the only write path.
- **macOS window-close vs quit:** server lives for app lifetime; tray keeps the app (and server) alive when windows close — shell concern, note in Phase 1 skeleton.

### Pattern 5: Attribute-matching anchor migration with confidence-tiered degradation (validation of TD-01)

**What:** TD-01's design — opaque stable ID + {content-hash, heading-path, ordinal} attributes, three-step matching (exact / moved / weighted similarity), three confidence tiers with explicit degradation and never-silent-loss.

**Why this is the right shape (verified against prior art):** Hypothesis's fuzzy anchoring — the most battle-tested annotation-migration system in production — uses the same fundamental strategy: multiple redundant selectors per anchor (structural, positional, quoted-context) with a cascading fallback chain and explicit "orphaned" state when confidence is too low. Microsoft's robust-anchoring research (keyword anchoring, US7747943B2) reaches the same conclusion: multi-attribute matching + graceful, visible degradation. TD-01 additionally gets two things prior art lacks that this domain enables: a deterministic block segmentation (single pinned parser — Hypothesis has to fight arbitrary DOM), and a completeness invariant (every old block ID accounted for per migration) that mechanically guarantees the 0-silent-loss release gate. **Assessment: TD-01 is at or above the state of practice; no structural change recommended.** The open items (thresholds, CJK tokenization for similarity ratio) are correctly scoped as calibration, not architecture.

## Data Flow

### Key Data Flows

1. **FS ingest (the spine):** `disk write (agent/editor/git) → notify → debounce/merge (2s/10s) → echo filter → doc identity + version snapshot (prism-store, writer txn) → prism-anchor migrate (same txn tail) → MigrationResult on engine bus → fan-out: F3 comment migration · F4 hit detection · F2 digest re-gen scheduler · F8 subscription match → shell adapter → coarse Tauri event → UI fetches`. Budget: <10s end-to-end; migration itself P95 <300ms.
2. **Comment → feedback loop (the product):** `UI comment (command → facade → writer) → user triggers回流 → bundle assembly + LLM intent summary (prism-llm) → atomic write .prismdocs/feedback/ (echo-registered) + clipboard → agent consumes (file or MCP) → return signals: MCP receipt (prism-mcp → trait → writer) OR FS change hitting commented block (flow 1's F4 consumer) → needs-review → UI review (Base diff + comments) → resolve = closed loop`.
3. **LLM streaming (request-scoped):** `UI invokes command with Channel<DigestEvent> → facade → prism-llm SSE stream → channel.send per token (ordered) → UI renders → on completion, digest persisted to cache (writer) keyed by content-hash+prompt-ver+model`.
4. **MCP serve (agent-facing):** `agent → 127.0.0.1 axum → bearer + Origin middleware → rmcp session → service trait → read pool (reads) / writer (receipt only) → response`; receipt write re-enters flow 2's state machine.
5. **Cross-project drift (F8):** flow 1's subscription consumer on the **upstream** project → drift alert in downstream Inbox → one-click check bundle → re-enters flow 2 in the downstream project. No new machinery — this composition is why F8 is cheap, and why it must come after Phases 3 and 6.

### State Management

Frontend holds view-model state only; SQLite is the single source of truth. Invalidations arrive as coarse events (Pattern 1); React Query-style fetch-on-invalidate against query commands keeps the WebView from ever owning authoritative state.

## Build Order (confirm / amend)

### Confirmed

The doc's dependency chain is sound: **1 → 2 → 3 → 5 → 6 critical path; 4 ∥ 5; 7 after 5+6; 8 (release) last.** Specifically verified:

- **2 before 3 (integration-wise):** anchoring consumes `document_version` rows produced by F1's identity/snapshot layer.
- **3 before both 4 and 5:** F2′'s change bars and re-gen triggers, and F3's anchor migration, both consume the frozen `MigrationResult`/`ChangeSet` contract. Freezing this interface before its consumers exist (TD-01) is exactly right — it's the system's load-bearing wall.
- **4 ∥ 5:** digest (LLM read-path) and comments (store write-path + UI) share nothing but Phase 3 outputs.
- **6 after 4+5:** F4 needs comments (5) and the LLM channel (4) for intent summaries; it is correctly the integration terminus where AC-4a becomes verifiable.
- **7 after 6:** F7/F5 register tools into the MCP server that Phase 6 builds; F8 composes anchoring (3) + the F4 loop (6).

### Amendments (3)

| # | Amendment | Rationale |
|---|-----------|-----------|
| A1 | **Phase 1 must include the event-bus spine and IPC adapter proof**, not just workspace+schema+keyring. Exit criterion to add: one engine-bus event round-trips to the WebView (coarse event) and one command streams via Channel. Also fix the single-writer discipline (Pattern 2) in Phase 1's store setup. | Every later phase publishes or consumes bus events; retrofitting ordering/adapter decisions after Phase 2's watcher exists is the classic source of out-of-order UI bugs Tauri's docs warn about. Cheap now, expensive later. |
| A2 | **Anchoring pure core (prism-parse + prism-anchor algorithm + calibration harness) may start in parallel with Phase 2**; only its store/pipeline integration waits for Phase 2. | prism-anchor is a pure function over two trees (TD-01); its interface is already frozen and its harness runs on fixture corpora, not on the live pipeline. This overlap shortens the critical path 1→2→3 without adding integration risk — the M0 Track B calibration corpus work wants this early anyway. |
| A3 | **Inside Phase 4, sequence prism-llm transport (streaming/retry/keyring wiring) as the first deliverable**, digest features second. | Phase 6 depends on Phase 4 only for the LLM channel (intent summaries), not for the digest feature. Narrowing the 4→6 edge to "transport done" means a digest-quality iteration slip cannot block the critical path to AC-4a. |

Minor note, not an amendment: Phase 6's MCP server work should inherit the port/discovery decision (Pattern 4) as a written pre-decision, since the CLI helper and Claude Code install docs both depend on it.

### Phases likely needing deeper (phase-time) research

- **Phase 3:** `similar` algorithm variants + CJK-mixed tokenization for word-level ratio (TD-01 §12-2 spike).
- **Phase 6:** Claude Code / Cursor MCP client config specifics (headersHelper contract, hook syntax — client-side formats change frequently); rmcp version pin against MCP spec revisions.
- **Phase 2:** macOS FSEvents behavior under git operations (checkout/rebase = mass events; reconciliation scan sizing).

## Anti-Patterns

### Anti-Pattern 1: Business logic in Tauri commands

**What people do:** grow `#[tauri::command]` bodies into the app's real logic.
**Why it's wrong:** untestable without the shell, kills D-01's independent-test property, entangles IPC types with domain types.
**Do this instead:** every command is a one-line delegate to `prism-engine`; the facade's API is the tested surface.

### Anti-Pattern 2: Pushing large/ordered payloads through Tauri events

**What people do:** emit diffs, migration results, or LLM tokens as events.
**Why it's wrong:** JSON-string payloads, no type safety, and documented out-of-order processing under async listeners — silent UI/store divergence.
**Do this instead:** Pattern 1 — coarse events for invalidation, Channels for streams, queries for truth.

### Anti-Pattern 3: Uniform connection pool doing writes

**What people do:** one r2d2 pool, any connection writes.
**Why it's wrong:** `SQLITE_BUSY` storms and degraded write latency under WAL; worst during the exact hot path (burst agent writes → version+migration transactions).
**Do this instead:** Pattern 2 — read pool + single writer handle owned by the facade.

### Anti-Pattern 4: Second Markdown parser creeping into anchoring or diffing

**What people do:** "the frontend already has an AST, let's compute change bars there."
**Why it's wrong:** two segmenters guarantee anchor drift — the project's own What-NOT-to-use #1, and the reason Hypothesis-class systems fight DOM instability.
**Do this instead:** all block boundaries, diffs, and change bars derive from comrak output shipped from the engine; react-markdown renders only.

### Anti-Pattern 5: Ad-hoc echo suppression per write site

**What people do:** each feature that writes to disk implements its own "ignore my own event" logic.
**Why it's wrong:** three implementations drift; one missed site creates an infinite loop (write → watch → regenerate → write). The digest re-gen + `log.md` materialization + feedback writes triangle is a real loop risk here.
**Do this instead:** the single prism-fs primitive (register-before-write), with entry expiry, used by every write site including `.prismdocs/` outputs.

## Scaling Considerations

| Scale | Assessment |
|-------|------------|
| MVP target (500 docs / 2000 cards) | SQLite + FTS5 comfortably meets <300ms search and <500ms open; no architectural pressure |
| First bottleneck | LLM latency/cost on digest regeneration bursts (git pull touches 50 docs → 50 re-gens). Mitigate in Phase 4: re-gen queue with concurrency cap + the existing ≤5k-token auto/manual policy; cache keys already prevent redundant calls |
| Second bottleneck | Migration bursts on mass FS events (branch switch). Bounded by 1MB/doc cap + P95<300ms/doc; serialize per-doc, parallelize across docs on the read side, but keep persist on the single writer |
| Explicitly not a concern | Multi-user/server scale — out of scope by design (local-first, zero server) |

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| LLM endpoints (Anthropic/OpenAI-compat) | prism-llm only; SSE streaming; keyring for keys | User-configured base_url; retries must be idempotent (cache-key check before re-call) |
| Claude Code / Cursor / other agents | MCP (loopback HTTP + bearer) ∥ file protocol `.prismdocs/` | File protocol is the degraded path and must stay independently sufficient (AC-4c) |
| macOS keychain | keyring crate, two accounts (LLM key, MCP token) | Direct keyring, no stronghold — confirmed current guidance |
| git (P0.5) | read-only status/hash association | Never write to the repo beyond `.prismdocs/` |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| WebView ↔ shell | Tauri commands (request/response), events (invalidate), Channels (streams) | Pattern 1; capability-scope commands |
| Shell ↔ facade | Direct calls + bus subscription | Shell is the only bus→IPC adapter |
| Facade ↔ domain crates | Direct calls; anchor results republished on bus | Migration runs inside the store writer transaction tail (TD-01 §5.1) |
| prism-mcp ↔ engine | Service traits injected at construction | Avoids the facade↔mcp cycle; enforces "writes = receipts only" at one choke point |
| CLI helper ↔ app | Keyring (token) + HTTP to loopback port | Needs the Phase 6 port-discovery decision |

## Sources

- Tauri v2 official docs — [Calling the Frontend / events vs channels](https://github.com/tauri-apps/tauri-docs/blob/v2/src/content/docs/develop/calling-frontend.mdx), [Calling Rust / Channel streaming](https://github.com/tauri-apps/tauri-docs/blob/v2/src/content/docs/develop/calling-rust.mdx), [Project structure (workspace membership)](https://v2.tauri.app/concept/architecture/) — MEDIUM (curated docs via Context7)
- [Yerd — Tauri v2 app architecture case study](https://yerd.app/developer/architecture) (pure core crates, trait-bound side effects, thin binaries) — MEDIUM (corroborates official docs)
- SQLite WAL + pooling: [Evan Schwartz — connection pool write-performance PSA](https://emschwartz.me/psa-your-sqlite-connection-pool-might-be-ruining-your-write-performance/), [Cruz Luna — SQLite pooling](https://www.cruzluna.dev/posts/sqliteconnectionpool/), [WAL connection strategies](https://dev.to/software_mvp-factory/sqlite-wal-mode-and-connection-strategies-for-high-throughput-mobile-apps-beyond-the-basics-eh0) — MEDIUM (multiple independent sources agree)
- [notify-debouncer-full docs](https://docs.rs/notify-debouncer-full/latest/notify_debouncer_full/) (rename stitching, FS-ID tracking, atomic-save handling; no built-in echo suppression) — MEDIUM
- rmcp / MCP embedding: [official Rust SDK](https://github.com/modelcontextprotocol/rust-sdk), [Shuttle — Streamable HTTP MCP server in Rust](https://www.shuttle.dev/blog/2025/10/29/stream-http-mcp), [SSE MCP + auth middleware](https://www.shuttle.dev/blog/2025/08/13/sse-mcp-server-with-oauth-in-rust) — MEDIUM
- Anchoring prior art: [Hypothesis — Fuzzy Anchoring](https://web.hypothes.is/blog/fuzzy-anchoring/), [W3C annotation workshop report](https://www.w3.org/2014/04/annotation/report.html), [MSR — Robustly Anchoring Annotations Using Keywords](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/tr-2001-107.pdf) — MEDIUM (verified across sources)
- Project inputs: `docs/调研_技术基建与开发Phase.md` §2.1/§2.3/§4, `docs/技术设计_Block锚定与迁移契约.md` (TD-01), `docs/PRD_PrismDocs_MVP.md` §2/§4

---
*Architecture research for: PrismDocs — local-first Tauri v2 + Rust engine document workbench*
*Researched: 2026-07-28*
