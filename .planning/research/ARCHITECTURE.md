# Architecture Research

**Domain:** Local-first desktop knowledge workbench (macOS-first) + Chrome MV3 extension + embedded local MCP server; two-layer docs (English Base ↔ derived native-language Lens) with block-level anchoring and a comment-to-agent loop
**Researched:** 2026-07-27
**Confidence:** HIGH on the anchoring model, incremental-projection design, data model, and MCP security boundary (grounded in the PRD/BRD + well-established patterns: unist/mdast positions, Myers/patience diff, SQLite FTS5, loopback MCP). MEDIUM on the exact confidence thresholds and the desktop-shell choice (needs a prototype + STACK.md decision).

> This document is the architectural **spine** for PrismDocs. The Block-anchoring subsystem (BRD §11 top risk) is treated as the load-bearing component: F1 must ship stable Block IDs + migration + downgrade before F2/F3/F4 can exist.

---

## Standard Architecture

### System Overview

Three OS processes plus one external browser process. All persistent state lives behind the **Core Engine**; the UI is a thin projection of it; disk Markdown is the authoritative Base.

```
┌───────────────────────────────────────────────────────────────────────┐
│  RENDERER PROCESS  (WebView / Chromium — React)                        │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐    │
│  │ Reader   │ │ Comment  │ │  Inbox   │ │  Cards   │ │  Clips + │    │
│  │ 3-view   │ │ sidebar  │ │ (main    │ │  view    │ │  Ctx     │    │
│  │Lens/split│ │ +threads │ │  entry)  │ │          │ │  assembler│   │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘    │
│       └────────────┴─────  typed IPC  ─┴────────────┴──────────┘      │
│                    (query/command; UI never touches disk or DB)        │
├───────────────────────────────────────────────────────────────────────┤
│  CORE ENGINE  (main process — single writer, all domain logic)         │
│                                                                         │
│  ┌───────────────┐  ┌────────────────────┐  ┌──────────────────────┐  │
│  │ Import/Project │  │  Markdown Parser   │  │  ★ ANCHOR ENGINE ★   │  │
│  │  + FS Watcher  │→ │  → Block Tree      │→ │  ID assign +         │  │
│  │  (chokidar,    │  │  (remark/mdast,    │  │  diff-migration +    │  │
│  │  glob, debounce│  │  unist positions)  │  │  confidence gate +   │  │
│  │  frontmatter,  │  └────────────────────┘  │  downgrade-not-drop  │  │
│  │  HTML→MD)      │                           └──────────┬───────────┘  │
│  └───────────────┘                                       │              │
│  ┌───────────────┐  ┌────────────────────┐  ┌───────────▼───────────┐  │
│  │ Lens Scheduler │  │  Domain Services   │  │  Provenance Tracker   │  │
│  │ incremental    │  │  Comment · Card ·  │  │  base_changes: which  │  │
│  │ re-projection, │  │  Clip · CtxPack ·  │  │  bundle/agent caused  │  │
│  │ cache, stream  │  │  FeedbackBundle    │  │  this Base edit       │  │
│  └───────┬────────┘  └─────────┬──────────┘  └───────────────────────┘  │
│  ┌───────▼──────┐   ┌──────────▼─────────┐   ┌──────────────────────┐  │
│  │ LLM Client   │   │  Store Layer       │   │  MCP Host  (loopback) │  │
│  │ Anthropic /  │   │  SQLite (index +   │   │  127.0.0.1 + token +  │  │
│  │ OpenAI-compat│   │  sidecar) + FS I/O │   │  Origin check;        │  │
│  │ keychain,    │   │  disk .md = truth  │   │  read tools + 1 write │  │
│  │ retry, tokens│   │                    │   │  (respond_to_comment) │  │
│  └──────────────┘   └─────────┬──────────┘   └──────────┬────────────┘  │
│  ┌────────────────────────────┼─────────────────────────┼────────────┐ │
│  │ Extension Bridge Host  (native messaging / loopback + queue)      │ │
│  └────────────────────────────┼─────────────────────────┼────────────┘ │
├───────────────────────────────┼─────────────────────────┼──────────────┤
│  STORAGE                       │                         │              │
│  ┌──────────────────┐   ┌──────▼───────────┐   ┌─────────▼──────────┐   │
│  │ SQLite (WAL,FTS5)│   │  Base .md on disk│   │ .prismdocs/        │   │
│  │ blocks, lens,    │   │  (AUTHORITATIVE, │   │  feedback/ context/│   │
│  │ comments, cards, │   │  user-owned, not │   │  README.md         │   │
│  │ clips, prov.     │   │  polluted)       │   │  (materialized)    │   │
│  └──────────────────┘   └──────────────────┘   └────────────────────┘   │
└──────────────────────────────────────────────┬────────────────────────┘
        loopback / native-messaging boundary    │        stdio/HTTP MCP boundary
┌───────────────────────────────┐   ┌───────────▼────────────────────────┐
│  CHROME MV3 EXTENSION          │   │  CODING AGENT (external process)    │
│  service worker + content      │   │  Claude Code / Cursor / other       │
│  script (Readability+Turndown) │   │  reads .prismdocs/ (file protocol)  │
│  offline queue → bridge host   │   │  and/or connects to local MCP       │
└───────────────────────────────┘   └─────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility (what it owns) | Typical Implementation |
|-----------|-------------------------------|------------------------|
| **Renderer / UI** | Reader 3-view (Lens / split / Base), comment sidebar + threads, Inbox, cards, clips, context assembler. Renders projections; issues typed commands. Zero disk/DB access. | React in WebView; typed IPC client |
| **Import / FS Watcher** | Glob scan (REQ-1.2), FS events (add/change/unlink/rename) with 2s debounce (REQ-1.4), HTML→MD (REQ-1.3), frontmatter parse (REQ-1.8), rename-by-content-hash (REQ-1.1 edge). Disk is authoritative (REQ-1.5). | `chokidar`, `gray-matter`, Readability/Turndown |
| **Markdown Parser → Block Tree** | Parse `.md` to mdast; segment into Blocks (heading/paragraph/list/code/table); attach `unist` position (line/col) for the *current* version only. | `remark` / `remark-parse` (unified) |
| **★ Anchor Engine ★** | Assign stable opaque Block IDs; on every Base change, migrate IDs old→new via diff-alignment + multi-signal scoring; gate by confidence; **downgrade, never drop**. The spine everything else anchors to. | Custom; Myers/patience diff over block-hash sequence + similarity |
| **Lens Scheduler** | Compute dirty set from anchor migration; invalidate only changed + context-adjacent segments; cancellable debounced queue; stream per-segment; persist cache across restarts. | Task queue keyed by `block_id`; SQLite cache |
| **Domain Services** | CRUD + state machines for Comment (thread FSM), Card (double-links, injection line), Clip, ContextPack, FeedbackBundle. Single writer. | Service modules over Store Layer |
| **Provenance Tracker** | For each Base change, record source: `bundle` / `mcp_respond` / `external_unknown` + agent + bundle_id + optional commit hash (REQ-4.7). | `base_changes` table + correlation window |
| **LLM Client** | Anthropic + OpenAI-compatible (custom base_url); key in OS keychain; retry; per-call token accounting; model routing (fast baseline + strong for decision segments, Q1). | Provider SDKs; keytar/Keychain |
| **Store Layer** | SQLite (index, sidecar metadata, Lens cache, FTS5 search) + FS I/O for Base. Rebuildable index; disk wins on conflict. | `better-sqlite3` (WAL) + fs |
| **MCP Host** | Expose read tools + one scoped write to agents; loopback-only; token + Origin validation; current-Workspace scope. | MCP TS SDK; 127.0.0.1 HTTP or stdio shim |
| **Extension Bridge Host** | Receive clips from MV3 extension; offline queue reconciliation. | Native messaging host or loopback HTTP + token |

---

## Recommended Project Structure

```
prismdocs/
├── apps/
│   ├── desktop/                # shell (Electron main OR Tauri+node sidecar)
│   │   ├── main/               # process bootstrap, IPC bridge, window
│   │   └── renderer/           # React UI (reader, comments, inbox, cards, clips)
│   └── extension/              # Chrome MV3
│       ├── background/         # service worker: queue, bridge client
│       ├── content/            # Readability + Turndown capture
│       └── popup/              # capture panel (title, project, tags, token est.)
├── packages/
│   ├── core/                   # SHELL-AGNOSTIC domain engine (the value)
│   │   ├── parse/              # remark → Block tree, unist positions
│   │   ├── anchor/             # ★ ID assignment, diff-migration, confidence, downgrade
│   │   ├── lens/               # projection scheduler, invalidation, cache, streaming
│   │   ├── comments/           # thread FSM, downgrade handling
│   │   ├── cards/  clips/  contextpack/  feedback/   # domain services
│   │   ├── provenance/         # base-change attribution
│   │   ├── llm/                # provider clients, routing, token accounting
│   │   └── store/              # SQLite schema+migrations, FTS5, FS I/O, OKF materialize
│   ├── mcp-server/             # loopback MCP host + tool surface
│   ├── okf/                    # frontmatter parse/preserve + export materialization
│   └── ipc-contract/           # typed query/command schema shared UI↔core
└── fixtures/
    └── anchor/                 # adversarial rewrite corpora for AC-3b test harness
```

### Structure Rationale

- **`packages/core/` is shell-agnostic:** the moat (anchoring, projection, provenance) lives in plain TypeScript independent of Electron/Tauri, so the shell decision (STACK.md) does not block the hard problems and the extension/MCP can reuse the same store.
- **`anchor/` is its own package with a dedicated fixture corpus:** it is the #1 risk; it must be testable in isolation against adversarial rewrites (AC-3b) before F2/F3/F4 depend on it.
- **`okf/` isolated:** OKF compatibility is an architecture-level decision (PROJECT.md); keeping parse-preserve and export-materialization in one place makes round-trip safety auditable.
- **`ipc-contract/` shared:** enforces the single-writer boundary — UI can only issue typed commands/queries, never touch disk or DB.

---

## Architectural Patterns

### Pattern 1: Sidecar Index over Authoritative Files

**What:** Disk Markdown is the single source of truth for Base content; SQLite is a **rebuildable index + metadata sidecar** (Block IDs, comments, cards, Lens cache, provenance). On conflict, disk wins (REQ-1.5). Block IDs and frontmatter are NEVER written into user `.md` (Key Decision; keeps files clean and git/agent-friendly).

**When to use:** Local-first apps that must coexist with external editors (IDE, coding agent, `git pull`) writing the same files.

**Trade-offs:** (+) zero file pollution, zero lock conflicts, backup = one directory (SQLite + `.prismdocs/`). (−) index can drift from disk → needs reconciliation on startup and on every FS event; identity of a moved/renamed block is a *derived* judgment, not a stored fact (hence the anchor engine).

```typescript
// On FS change: disk is truth; sidecar reconciles to it, never the reverse.
async function onBaseChanged(path: string) {
  const md = await fs.readFile(path, "utf8");
  const newTree = parseBlocks(md);                 // remark → Block[]
  const oldBlocks = store.getBlocks(documentId);   // last known, with IDs
  const migration = anchorEngine.migrate(oldBlocks, newTree); // see Pattern 2
  store.applyMigration(migration);                 // carry IDs, mark dirty/deleted
  lensScheduler.invalidate(migration.dirtyBlockIds);
  provenance.record(documentId, migration.changedBlockIds); // source attribution
}
```

### Pattern 2: Diff-Driven Anchor Migration with a Confidence Gate (THE spine)

**What:** We only ever see *before/after snapshots* of a file (the agent edits on disk, we don't observe keystroke ops). So we align two Block sequences with a robust diff and score each candidate correspondence on multiple signals; high confidence carries the ID, medium flags "changed", low is an explicit downgrade — **never a silent reattach or drop**.

**When to use:** Anchoring durable metadata (comments, Lens segments) to content in files edited by external, non-cooperative tools.

**Trade-offs:** (+) works without controlling the editor; tolerant of insert/delete/move/rewrite; auditable confidence. (−) heuristic, not exact — must be tuned against a corpus; ambiguous cases cost the user a confirmation click (acceptable: the alternative is data loss).

**Data model (sidecar SQLite — nothing in the `.md`):**

```
blocks(
  block_id        TEXT PRIMARY KEY,   -- opaque stable, e.g. blk_<ULID>
  document_id     TEXT,
  content_hash    TEXT,   -- sha256 of NORMALIZED text (trim, collapse ws, drop soft-wrap)
  block_type      TEXT,   -- heading|paragraph|list|code|table
  heading_path    TEXT,   -- ancestor breadcrumb, "Architecture > Storage"
  ordinal         INTEGER,-- doc-order index (volatile, heuristic only)
  prev_hash       TEXT,   -- neighbor fingerprint (context signal)
  next_hash       TEXT,
  text_snapshot   TEXT,   -- for downgrade quote + similarity
  start_line INT, end_line INT,       -- CURRENT version only; volatile, never the anchor
  version_seen    INTEGER
)
```

**Migration algorithm (multi-pass, git-hunk style):**

```typescript
function migrate(oldB: Block[], newB: NewBlock[]): Migration {
  // Pass 1 — EXACT: same content_hash AND same heading_path → carry id, conf = 1.0
  //          (covers unchanged blocks even when neighbors changed)
  // Pass 2 — STRUCTURAL: unmatched, same heading_path + block_type + near ordinal,
  //          content edited → similarity = tokenJaccard / normLevenshtein
  // Pass 3 — FUZZY ALIGNMENT: run Myers/patience diff over the block-hash sequence
  //          (each block = one "line"); the alignment yields insert/delete/move/edit,
  //          exactly like a git-diff hunk match. Refine with similarity from Pass 2.
  // SCORE per candidate old→new:
  //   conf = 0.55*content_sim + 0.20*headingPathMatch
  //        + 0.15*ordinalProximity + 0.10*neighborHashMatch
  // ASSIGN greedy (or Hungarian) over conf:
  //   conf >= HIGH (~0.85) → migrate id silently (change bar only if hash differs)
  //   LOW (~0.5) <= conf < HIGH → migrate id BUT flag needs_confirmation
  //   conf < LOW → old = DELETED (its anchors orphaned), new = INSERTED (fresh id)
}
```

**Confidence-threshold downgrade path — 0 silent loss (AC-3b):**

| Outcome for a commented Block | What happens to the comment |
|-------------------------------|-----------------------------|
| ID survives, conf ≥ HIGH | Stays anchored; change bar if content changed. |
| ID migrated, LOW ≤ conf < HIGH | Stays on migrated block, flagged **"原文已变化，请确认"**, shows quote snapshot from comment time. |
| No match ≥ LOW (block deleted/rewritten past recognition) | **Downgraded to document-level**, retains quote snapshot + heading_path breadcrumb, marked "原位置已变化，请确认", surfaced in Inbox. Never deleted, never silently reattached. |

**Why this over the alternatives (reference approaches):**

| Approach | Verdict for PrismDocs |
|----------|----------------------|
| **Google Docs OT / index anchors** | Rejected. Requires observing every edit operation. Agents edit `.md` on disk out-of-band; we never see the ops. |
| **CRDT positions (Yjs/Automerge, fractional index / RGA)** | Rejected for MVP. Stable IDs survive concurrent edits *only if all writers mutate through the CRDT*. External agents write plain Markdown; reconstructing a CRDT from snapshots defeats the purpose. Revisit as a P1 "portable mode" if PrismDocs ever owns the editor. |
| **mdast + line/position ranges (`unist`)** | Adopted as the *parse layer only*. Gives clean block boundaries + current line ranges for selection/render. Line ranges are volatile → never the persistent anchor. |
| **git-diff hunk / Myers–patience diff** | Adopted as the *migration backbone*. We have two snapshots and need insert/delete/move/edit tolerance — exactly what a diff aligner gives. Block-content-hash is the diff token; similarity refines it. |

### Pattern 3: Content-Hash Invalidation for Incremental Lens Re-Projection

**What:** Each Lens segment is keyed by `block_id` and stores the `base_content_hash` it was generated from. A segment is valid iff `stored_hash == current_block_hash`. On a Base change, only blocks the anchor engine marked changed/new are dirty; deletions drop their segment. **Context-adjacent** blocks (immediate prev/next sibling in the same heading section, and the prose flanking a changed code block per REQ-2.2) are marked *soft-dirty* and regenerated too, keeping the blast radius minimal (AC-2b).

**When to use:** Expensive per-unit derivations (LLM output) cached against mutable source; cost is the design constraint (BRD §10).

**Trade-offs:** (+) minimal LLM spend, restart-safe cache (persisted in SQLite → no recompute on restart, non-functional req), isolated failures (per-segment retry). (−) neighbor heuristics can occasionally regenerate a segment that didn't semantically need it; over-narrow neighbor rules can miss a context shift → tune conservatively (prefer one extra neighbor over a stale Lens).

```typescript
// Scheduler: debounced (2s, matches FS), cancellable, streaming.
lensScheduler.on("invalidate", (dirty: BlockId[]) => {
  cancelInFlightFor(dirty);                    // Base changed again mid-projection → reschedule
  const targets = expandContextAdjacent(dirty);// + soft-dirty neighbors
  for (const id of targets) {
    queue.run(id, async () => {
      const model = isDecisionBlock(id) ? STRONG : FAST;    // Q1 routing
      for await (const chunk of llm.projectStream(block(id), model))
        ipc.emit("lens:chunk", { id, chunk }); // per-segment streaming render (REQ-2.8)
      store.saveLens(id, currentHash(id), model, tokenCost);
    });
  }
});
// Startup: validate each cached segment's hash vs current block; regenerate only mismatches.
```

### Pattern 4: Materialize-on-Export (OKF at the boundary only)

**What:** During normal operation, all PrismDocs metadata (Block IDs, comments, cards, clips, provenance) lives in the sidecar; user source files are untouched. Frontmatter is *materialized* only when exporting an OKF Bundle (REQ-7.6): Cards → files with `type: Card` + injection line; Clips → `type: Clip`, URL→`resource`; docs copied with the six-field frontmatter; `index.md` generated. Export writes to a **separate bundle directory** — source stays clean.

**When to use:** Interop with an open standard (OKF v0.1) without accepting lock-in or file pollution.

**Trade-offs:** (+) round-trip safe (import parses + byte-preserves existing frontmatter; export is a pure function sidecar→frontmatter; re-import of an export reproduces the sidecar state = idempotent). (−) two representations to keep in sync (sidecar ↔ materialized) — mitigated by making materialization a pure, tested function with no back-write into sources. Optional `frontmatter-write` mode (Q7) is project-level opt-in only.

### Pattern 5: Loopback-Only, Capability-Scoped MCP Host (least privilege)

**What:** The running desktop app hosts the MCP server so tools see *live* Workspace state. Bind to `127.0.0.1` only; authenticate with a per-Workspace bearer token stored in the keychain and written into the generated client-config snippet; validate the `Origin` header (defeats DNS-rebinding — the documented attack against local MCP/HTTP servers per the 2025-11-25 transports spec). Scope: current Workspace only; the only write tool is `respond_to_comment` (agents cannot create/delete comments or cards — PRD §4.2 security).

**When to use:** Any local server exposing user data to another local process (the coding agent).

**Trade-offs:** (+) live shared state, minimal attack surface, capability-scoped. (−) the app must be running for MCP to answer → the **file protocol** (`.prismdocs/feedback/*.md` + FS-watch closure detection) is the mandatory no-MCP fallback (AC-4c), so the loop still closes when the agent lacks MCP.

**Transport choice:** two viable shapes, both loopback:
- **stdio shim** the agent launches (best default isolation per spec) that proxies over a unix socket / loopback to the running app — good for Claude Code/Cursor stdio configs.
- **Streamable HTTP on 127.0.0.1** (ephemeral port + token + Origin check) — simpler single host, agent connects by URL.

Recommend shipping the **stdio shim as primary** (matches "use stdio whenever you can", zero open port) with loopback-HTTP as an option.

### Pattern 6: Single-Writer Core, Thin Renderer

**What:** Every mutation flows through Core domain services over a typed IPC contract; the UI holds no authoritative state and never touches disk or SQLite directly. Lens is a read-only projection (no editing — Key Decision, prevents two-layer drift).

**When to use:** Local apps where an FS watcher, an agent, and the UI all race to mutate the same logical state.

**Trade-offs:** (+) one reconciliation point, deterministic ordering, testable core. (−) all writes are async round-trips (fine at local latency).

---

## Data Flow

### The core closed loop (AC-4a) — explicit directions

```
Agent writes docs/*.md  ──►  FS Watcher (debounce 2s)  ──►  Parser → Block Tree
                                                                    │
                                                                    ▼
                                                         ★ Anchor Engine (migrate IDs)
                                                                    │
                              ┌─────────────────────────────────────┼───────────────────┐
                              ▼                                     ▼                     ▼
                   Lens Scheduler (dirty set)          Comment re-anchor / downgrade   Provenance
                              │                                     │                (source attrib.)
                              ▼                                     ▼
                   LLM projectStream ──► Lens cache (SQLite) ──► IPC ──► UI (change bars, Inbox)
                              │                                     │
                              ▼                                     ▼
                   commented blocks that changed  ──►  status → needs-review  ──►  Inbox
                                                                    │
User reads Lens ──► writes Comment (block_id + quote + type + zh body) ──► Core ──► SQLite
                                                                    │
User clicks "回流" ──► FeedbackBundle builder (comments + block excerpts + heading paths
                        + product-generated EN intent summary; NO full doc by default, AC-4b)
                              │                        │
                              ▼                        ▼
             .prismdocs/feedback/<ts>.md        MCP: list_feedback / get_feedback
             (+ one-line trigger to clipboard)  (loopback, token, Origin-checked)
                              │                        │
                              └──────► Agent ◄─────────┘
                                        │
        respond_to_comment(id, done, note)  OR  edits .md (file-protocol fallback)
                                        │
                                        ▼
                          back to FS Watcher / MCP write → status flips, provenance links bundle
                                        │
                          User reviews (change bar + comment context) → resolve (loop_closed++) / reopen
```

### Clip flow (F6)

```
Web page ─► content script (Readability extract + Turndown → clean MD, code-block sanitize)
         ─► popup (title, project, tags, token estimate, note)
         ─► service worker ─► Extension Bridge Host (native messaging / loopback + token)
         ─► Clip service ─► SQLite (clips)      [app offline → queue in extension storage, replay on connect]
```

### Context Pack flow (F7)

```
UI selection tree (docs + cards[injection line] + clips) ─► live token tally
  ─► ContextPack builder (structured EN, optional AI compression of clips)
  ─► .prismdocs/context/<name>.md   AND   MCP get_context_pack
  ─► (P0.5) OKF export: sidecar → frontmatter materialization → bundle dir + index.md
```

### State management (UI)

```
Core (authoritative)  ──emit(events)──►  UI store (read-model cache)
        ▲                                        │
        └────────  IPC command (typed) ◄─────────┘   (UI never mutates authoritative state directly)
```

---

## Scaling Considerations

This is a **single-user, local** app. The scaling axis is **content volume** (docs / cards / blocks), not concurrent users. Target: 500 docs / 2000 cards → order 10^4–10^5 blocks; search <300ms, doc open <500ms, FS change surfaced <10s (non-functional reqs).

| Scale | Architecture adjustments |
|-------|--------------------------|
| Typical project (≤100 docs) | SQLite (WAL) + FTS5 handles everything in-process; no tuning needed. |
| Target (500 docs / 2000 cards) | FTS5 for full-text (<300ms); index `content_hash`, `document_id`, `heading_path`; startup reconciliation uses mtime+size fast-skip so unchanged files aren't re-parsed; anchor fuzzy pass scoped **within a single document's block set** (never global) to bound cost. |
| Large / power user (thousands of docs) | Lazy-parse on document open (parse-on-demand, keep only hashes hot); background reconciliation queue; consider `sqlite-vec`/embeddings for card search only if FTS5 relevance proves insufficient. |

### Scaling priorities

1. **First bottleneck — startup reconciliation** of the whole tree against the sidecar. Fix: mtime+size+hash quick-skip; only changed files hit the parser/anchor engine.
2. **Second bottleneck — LLM projection cost/latency**, not compute. Fix: incremental per-block invalidation (Pattern 3) + manual-trigger threshold for >5k-token docs (REQ-2.9) + fast/strong model routing.
3. **Third — anchor fuzzy matching** on huge single files. Fix: keep alignment per-document; cap similarity computation to candidates the diff aligner already paired.

---

## Anti-Patterns

### Anti-Pattern 1: Writing Block IDs or frontmatter into the user's `.md`
**What people do:** Persist anchors/metadata inline (HTML comments, `<!-- id -->`, injected frontmatter) so they "travel with the file."
**Why it's wrong:** Pollutes the user's source, creates git diffs and merge conflicts, fights the agent that owns the file, violates the core Key Decision.
**Do this instead:** Sidecar SQLite; identity is *derived* each parse by the anchor engine (Pattern 2). Materialize frontmatter only on OKF export (Pattern 4).

### Anti-Pattern 2: Line numbers / char offsets as the persistent anchor
**What people do:** Store `start_line`/`char_offset` and reattach comments there after an edit.
**Why it's wrong:** Every insertion above shifts all offsets; a one-line change silently moves every downstream comment.
**Do this instead:** Content-hash + heading-path + neighbor fingerprints; keep line ranges as *current-version, volatile* render/selection data only.

### Anti-Pattern 3: Full-document re-projection on any change
**What people do:** Re-run the LLM over the whole doc whenever it changes.
**Why it's wrong:** Cost blowup (the primary model cost, BRD §10); violates AC-2b.
**Do this instead:** Content-hash invalidation of the dirty set + minimal context-adjacent neighbors (Pattern 3); persist cache across restarts.

### Anti-Pattern 4: Silent re-anchor or silent drop of low-confidence comments
**What people do:** When unsure, either guess a new block or quietly discard the comment.
**Why it's wrong:** Destroys the trust that is the entire value prop; violates AC-3b (0 silent loss).
**Do this instead:** Confidence gate with an explicit **downgrade to document-level + snapshot + "请确认" flag + Inbox surfacing**.

### Anti-Pattern 5: Editable Lens
**What people do:** Let users edit the projected native-language layer.
**Why it's wrong:** Two-layer drift — the Lens and Base diverge with no source of truth.
**Do this instead:** Lens is a one-way projection; all dissatisfaction is expressed via comments on Base (Key Decision / product principle).

### Anti-Pattern 6: MCP bound to `0.0.0.0` / no token / no Origin check
**What people do:** Expose the local server broadly for "convenience."
**Why it's wrong:** DNS-rebinding lets a malicious web page drive the local server and exfiltrate the Workspace (documented attack in the MCP transports spec).
**Do this instead:** `127.0.0.1` only + keychain bearer token + `Origin` validation + Workspace-scoped + single narrow write tool (Pattern 5).

### Anti-Pattern 7: Treating SQLite as the source of truth for Base content
**What people do:** Cache doc text in SQLite and write it back to disk from the DB.
**Why it's wrong:** Fights external edits (IDE, agent, `git pull`); races and clobbers user changes.
**Do this instead:** Disk `.md` authoritative; SQLite is a rebuildable index that reconciles *to* disk (Pattern 1).

---

## Integration Points

### External Services / Processes

| Service | Integration pattern | Notes / gotchas |
|---------|---------------------|-----------------|
| **Coding agent (Claude Code, Cursor, other)** | Dual channel: file protocol (`.prismdocs/feedback/*.md`, always works) + loopback MCP (richer, live). | File protocol is the mandatory fallback (AC-4c). Provide a one-click CLAUDE.md/AGENTS.md snippet + MCP config generator. Suggest `.prismdocs/` in `.gitignore` by default. |
| **LLM endpoint (Anthropic / OpenAI-compatible)** | HTTP with user key from OS keychain; custom `base_url` for proxies/local models. | All calls retryable, failure must not corrupt data (non-functional req). Per-call token accounting for the cost dashboard. |
| **Chrome MV3 extension** | Native messaging host (preferred) or loopback HTTP + token. | Service-worker lifecycle is ephemeral → the extension keeps an offline queue and replays when the app connects. Content script does Readability + Turndown; sanitize code blocks (AC-6a: SO code copies clean). |
| **Git** | Read-only awareness (branch, file status, optional commit-hash on change records; REQ-1.7 P0.5). | Never write git; associate provenance with commit hash when detectable. |
| **OKF consumers** | Export a materialized bundle directory (Pattern 4). | Import parses + preserves existing frontmatter; export never mutates source. Controlled `type` vocabulary (nine values) validated on import. |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Renderer ↔ Core | Typed IPC (query/command) via `ipc-contract` | UI never touches disk/DB (single-writer). |
| Core ↔ SQLite | In-process `better-sqlite3` (WAL) | Index/sidecar; rebuildable. |
| Core ↔ Base `.md` | FS read (authoritative) + FS watch | Disk wins on conflict. |
| Core ↔ MCP host | In-process (state) exposed over loopback stdio/HTTP boundary to agent | Token + Origin + Workspace scope; one write tool. |
| Core ↔ Extension | Native messaging / loopback + token + queue | App-offline tolerant. |
| Parser → Anchor → Lens/Comments/Provenance | In-process function pipeline on each Base change | Anchor migration is the fan-out point for everything downstream. |

### Desktop shell (defer final choice to STACK.md)

- **Electron (recommended for MVP):** one Node runtime — `remark`, `better-sqlite3`, `chokidar`, the MCP TS SDK, and provider SDKs are all first-class in-process; the extension bridge and MCP host share the same runtime. Cost: larger binary/memory footprint. This is the fastest path to the hard problems.
- **Tauri v2 + node sidecar (leaner alternative):** ~3× smaller, lower idle memory, but the JS-heavy core (remark/MCP/SQLite) must run as a pkg-compiled node **sidecar** with Rust↔node IPC — more moving parts. Choose if binary size/footprint is a launch priority and the team is comfortable with the two-runtime split. `packages/core/` is written shell-agnostic precisely so this decision does not gate F1.

---

## Build Order / Dependency Graph (F1–F7)

The anchor engine is the spine: **F1 must deliver Block tree + stable IDs + migration + downgrade before F2/F3/F4 can be built** (they all anchor to `block_id`).

```
                 ┌──────────────────────────────────────────────┐
                 │  F1  Import · FS watch · Parser→Block tree     │
                 │      SQLite schema · frontmatter               │
                 │      ★ ANCHOR ENGINE (ID + migration + gate)   │  ← risk item, build FIRST
                 └───────────────┬───────────────┬───────────────┘
                                 │               │
                 ┌───────────────▼───┐   ┌───────▼───────────────┐
                 │ F2 Lens projection│   │ F3 Paragraph comments │   (parallel; both consume anchor)
                 │  (+ LLM client)   │   │  (+ downgrade path)    │
                 └───────────────┬───┘   └───────┬───────────────┘
                                 └───────┬────────┘
                                 ┌───────▼─────────────────────────┐
                                 │ F4 Comment→Agent loop  ★CORE★    │  ← MVP target = F1–F4 (AC-4a)
                                 │  MCP host + file protocol +      │
                                 │  provenance + closure detection  │
                                 └───────┬─────────────────────────┘
        ┌────────────────────────┬───────┴───────────┐
   ┌────▼─────┐            ┌──────▼──────┐      ┌──────▼──────────────────┐
   │ F5 Cards │            │ F6 Chrome   │      │ F7 Context Pack + OKF    │
   │ (store   │            │  clip +     │      │  export (needs F1 docs + │
   │  only)   │            │  bridge)    │      │  F5 cards + F6 clips)    │
   └──────────┘            └─────────────┘      └──────────────────────────┘
   (independent; parallel)  (most independent;    (comes last; assembles the
                            separate track)        other three + LLM compress)
```

**Recommended sequence:**

1. **F1 with the anchor spine, decomposed:** (1a) project/import + FS watch + SQLite schema + IPC contract; (1b) Markdown→Block tree (remark); (1c) **anchor ID + migration + confidence gate + downgrade**, built and hardened against an adversarial-rewrite fixture corpus *before* anything depends on it (this is the AC-3b test harness).
2. **F2 + F3 in parallel** once the anchor engine is stable — both consume `block_id`; F3 shares the migration/downgrade with F2.
3. **F4** — closes the core loop (MVP milestone = F1–F4). Needs F3 comments, F1 FS-watch closure detection, the MCP host, and provenance.
4. **F5 + F6 in parallel** — F5 depends only on the store; F6 is the most independent (separate extension + bridge track, can start anytime).
5. **F7 last** — assembles F1 docs + F5 cards + F6 clips, adds LLM compression, MCP `get_context_pack`, and OKF export.

**Cross-cutting, from day one (not a phase):** SQLite schema + migrations, LLM client (keychain, retry, token accounting), the typed IPC contract, and an MCP loopback host skeleton.

**Build-order implication for the roadmap:** the anchoring subsystem is not a feature slice inside F1 — it is a *prerequisite phase* with its own verification gate (adversarial rewrite corpus, AC-3b) that must pass before F2/F3/F4 planning is meaningful. Flag it for deeper phase-specific research (threshold tuning, similarity metric choice, embedding vs. lexical similarity).

---

## Sources

- PrismDocs internal (authoritative for requirements): `.planning/PROJECT.md`; `docs/PRD_PrismDocs_MVP.md` (§2 IA & domain objects, §2.4 Block anchoring, §2.5 OKF, F1–F7, §4 agent protocol, §5 non-functional); `docs/BRD_PrismDocs_MVP.md` (§6 core mechanisms, §11 risks). — HIGH
- Model Context Protocol — Transports (spec 2025-11-25): stdio vs Streamable HTTP; local server SHOULD authenticate; DNS-rebinding risk for local HTTP MCP → validate Origin, bind loopback. https://modelcontextprotocol.io/specification/2025-11-25/basic/transports — HIGH (curated/official)
- "stdio vs Streamable HTTP: Choosing the Right MCP Transport" — use stdio for local per-user integrations, HTTP for shared/remote. https://kirkryan.co.uk/stdio-vs-streamable-http-choosing-the-right-mcp-transport/ — MEDIUM
- Tauri v2 — Node.js as a sidecar (pkg-compiled binary; footprint ~3× smaller than Electron when bundling node). https://v2.tauri.app/learn/sidecar-nodejs/ ; comparison https://www.dolthub.com/blog/2025-11-13-electron-vs-tauri/ — MEDIUM
- Established patterns referenced (well-known, cross-checked): `unist`/mdast positions (remark), Myers/patience diff (git hunk matching), CRDT positional identity (Yjs/Automerge fractional indexing / RGA), Operational Transform anchored positions (Google Docs), SQLite FTS5. — HIGH

---
*Architecture research for: local-first two-layer-docs desktop workbench + MV3 extension + loopback MCP*
*Researched: 2026-07-27*
