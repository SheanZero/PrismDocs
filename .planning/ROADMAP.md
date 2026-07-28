# Roadmap: PrismDocs

## Overview

PrismDocs is built goal-backward from one hypothesis: **the F1–F4 comment→AI→re-review loop (AC-4a) meaningfully lowers the cost of reviewing AI-written docs for non-English-native vibe coders.** The build order is forced by a single spine — the Block anchoring engine — which every downstream capability anchors to. We lay a cross-cutting Foundation (Tauri/Rust core, SQLite sidecar, typed IPC, keychain LLM client, loopback MCP skeleton, and the resolved Tauri-vs-sidecar ADR), then land Import + FS-watch so disk stays authoritative, then harden the anchoring engine in isolation against an adversarial golden corpus (0 silent comment loss is a hard gate) *before* anything consumes `block_id`. On that stable anchor we build the Lens projection and block comments in parallel, then close the North-Star loop (dual-channel MCP + file protocol). Cards and the Chrome clipper follow as independent parallel tracks, and Context Pack + OKF export assembles everything last.

## Phases

**Phase Numbering:**

- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Foundation & Core Engine Skeleton** - Cross-cutting infra: Tauri/Rust core, SQLite sidecar, typed IPC, keychain LLM client, loopback MCP skeleton, shell ADR
- [ ] **Phase 2: Project Import & FS Watcher (F1)** - Folder/Git import, disk-authoritative FS sync, frontmatter round-trip, sidecar discipline
- [ ] **Phase 3: ★ Block Anchoring Engine (prerequisite moat)** - Stable Block IDs + multi-strategy migration + downgrade-never-drop, verified against an adversarial rewrite corpus
- [ ] **Phase 4: Lens Projection ‖ Block Comments (F2 ‖ F3)** - Colloquial Chinese Lens with incremental re-projection + fidelity guards, and threaded block comments on the stable anchor
- [ ] **Phase 5: ★ Comment → Agent Loop (F4)** - Feedback Bundle dual-channel (file + MCP), closed-loop recycling, provenance — closes the North-Star loop (AC-4a)
- [ ] **Phase 6: Cards ‖ Chrome Clipper (F5 ‖ F6)** - Originality-nudged Zettelkasten cards, and the MV3 web→clean-Markdown clipper with loopback bridge + offline queue
- [ ] **Phase 7: Context Pack & OKF Export (F7)** - Token-budgeted English Context Pack, MCP get_context_pack, and validated OKF Bundle export

## Phase Details

### Phase 1: Foundation & Core Engine Skeleton

**Goal**: Stand up the cross-cutting engine skeleton every later phase depends on — shell-agnostic core, local-first store, secure LLM + MCP boundaries — and resolve the one load-bearing architecture decision so it never gates F1.
**Mode:** mvp
**Depends on**: Nothing (first phase)
**Requirements**: AGENT-03, NFR-02, NFR-03, NFR-04
**Success Criteria** (what must be TRUE):

  1. The app launches on macOS (Apple Silicon); on first run the user configures an LLM endpoint with a custom `base_url`, and the API key is stored in the system keychain (never plaintext, never logged). [NFR-04]
  2. All project state lives in a single backup-able directory (SQLite WAL + files); the app opens and reads existing data with no network connection. [NFR-02]
  3. Document content is only ever sent to the user-configured LLM endpoint, and no telemetry is enabled by default. [NFR-03]
  4. The loopback MCP endpoint binds 127.0.0.1 only, rejects any request lacking the per-install token or bearing a foreign `Origin`, scopes to the current Workspace, and exposes no write surface beyond comment回执. [AGENT-03]
  5. The Tauri-vs-Node-sidecar ADR is resolved and recorded, and the shell-agnostic `core` skeleton, SQLite schema + migrations, and typed IPC contract are in place.

**Plans:** 7 plans in 6 waves

Plans:
**Wave 1**

- [ ] 01-01-PLAN.md — Preflight: toolchain gate, Cargo workspace + Tauri/React scaffold, stack-doc version corrections

**Wave 2** *(blocked on Wave 1 completion)*

- [ ] 01-02-PLAN.md — TRACER: end-to-end "configure my LLM endpoint and see it connect" walking skeleton

**Wave 3** *(blocked on Wave 2 completion)*

- [ ] 01-03-PLAN.md — Local-first store: project identity that survives a move, WAL durability, backup/export
- [ ] 01-04-PLAN.md — Loopback MCP engine: transport + the full AGENT-03 security boundary, headless

**Wave 4** *(blocked on Wave 3 completion)*

- [ ] 01-05-PLAN.md — Shell integration: MCP host resident with the app on a scanned port, project home

**Wave 5** *(blocked on Wave 4 completion)*

- [ ] 01-06-PLAN.md — Onboarding steps 2–4: workspace registration, `.prismdocs/` init, `.mcp.json` + protocol snippet

**Wave 6** *(blocked on Wave 5 completion)*

- [ ] 01-07-PLAN.md — ADR 0001 (Tauri vs Node sidecar) and macOS CI with a core/shell split

### Phase 2: Project Import & FS Watcher (F1)

**Goal**: Turn a local folder or Git repo into a live, disk-authoritative Base layer — imported, watched, frontmatter-parsed — with a change-event contract robust enough for everything downstream to trust.
**Mode:** mvp
**Depends on**: Phase 1
**Requirements**: IMPORT-01, IMPORT-02, IMPORT-03, IMPORT-04, IMPORT-05, IMPORT-06, IMPORT-07, IMPORT-08, IMPORT-09, NFR-01
**Success Criteria** (what must be TRUE):

  1. The user points PrismDocs at a local folder or Git repo root and the configurable globs import matching Markdown (default `docs/**/*.md`, root `*.md`, `CLAUDE.md`, `AGENTS.md`, `.claude/**/*.md`; excluding `node_modules`/`.git`/build output) as Base docs; AI `.html` reports convert to Markdown or degrade to an attachment with a notice when lossy. [IMPORT-01, IMPORT-02, IMPORT-03]
  2. When an external tool (IDE / agent / `git pull`) adds, modifies, deletes, or renames files, PrismDocs reflects it from disk within 10s — coalescing burst writes with a 2s debounce, never locking files, and creating no duplicate change records; a renamed/moved file is recognized as the same document by content hash, not path. [IMPORT-04, IMPORT-05, AC-1b]
  3. Existing YAML frontmatter round-trips byte-stable on untouched keys; the six OKF fields are indexed for filter/search; a `type` outside the controlled vocabulary is surfaced as "unregistered" (never silently accepted or dropped), and Block IDs/metadata are never written into the user's source `.md`. [IMPORT-07]
  4. The user can edit the Base layer in-app and write back to disk, gated behind default-read-only + explicit unlock; Git branch/status awareness and optional OKF `log.md` materialization are available as degradable extras. [IMPORT-06, IMPORT-08, IMPORT-09]
  5. Full-text search returns in <300ms and a document opens in <500ms at the 500-doc scale. [NFR-01]

**Plans**: TBD

### Phase 3: ★ Block Anchoring Engine (prerequisite moat)

**Goal**: Build and harden the product's spine and #1 risk in isolation — stable Block IDs that migrate across AI rewrites and degrade gracefully — so that F2/F3/F4 can safely anchor to `block_id` with a proven "0 silent loss" guarantee.
**Mode:** mvp
**Depends on**: Phase 2
**Requirements**: ANCHOR-01, ANCHOR-02, ANCHOR-03, ANCHOR-04, ANCHOR-05
**Success Criteria** (what must be TRUE):

  1. Every Base document parses into a Block tree (headings section; paragraphs / code blocks / tables as leaves) and each block gets a stable opaque ID (content hash + position heuristic + heading-path / neighbor fingerprints) stored in the sidecar — never written into the user's `.md`. [ANCHOR-01, ANCHOR-02, ANCHOR-05]
  2. When Base changes — including split, merge, reorder, and heavy rewrite — block IDs migrate via multi-signal diff matching (content similarity + relative position) so downstream comment/Lens anchors stay alive. [ANCHOR-03]
  3. On an adversarial AI-rewrite golden corpus (small edit → heading rename → paragraph split → section reorder → 50% rewrite → full rewrite → formatting-only churn), ≥90% of anchors are correctly migrated-or-downgraded with 0 silent loss, and the count invariant (migrated + downgraded == original) is asserted as a CI release gate. [ANCHOR-04, AC-3b]
  4. When migration confidence falls below threshold, an anchor is downgraded to a document-level anchor carrying an immutable quote snapshot + heading-path breadcrumb + "原位置已变化，请确认" flag — there is no code path that silently drops or silently re-attaches it. [ANCHOR-04]

**Plans**: TBD

### Phase 4: Lens Projection ‖ Block Comments (F2 ‖ F3)

**Goal**: On the stable anchor, deliver the two human-facing halves in parallel — a trustworthy colloquial Chinese Lens the user reads, and threaded comments the user writes on any block — so a vibe coder can review AI docs in their own language and register every reaction.
**Mode:** mvp
**Depends on**: Phase 3
**Requirements**: LENS-01, LENS-02, LENS-03, LENS-04, LENS-05, LENS-06, LENS-07, LENS-08, LENS-09, LENS-10, COMMENT-01, COMMENT-02, COMMENT-03, COMMENT-04, COMMENT-05, COMMENT-06, COMMENT-07, NFR-05, NFR-06
**Success Criteria** (what must be TRUE):

  1. For any Base doc the user reads a Simplified-Chinese Lens that is colloquial re-statement (not translation) — compresses repetition and explicitly flags ⚖️取舍 / ⚠️风险 / ❓需决策 — with an auto-generated 速读区 (3–5 sentence summary + linked decision list); each Lens segment is anchored 1:1 to a Base block, streams in per-segment, and offers per-segment retry on failure. The user switches between Lens-only / split / Base-only views. [LENS-01, LENS-02, LENS-03, LENS-04, LENS-09]
  2. After a Base change, only affected blocks (plus diff-determined neighbors) are re-projected — logs prove reprojected ≈ changed — while unchanged segments reuse a restart-persistent cache; changed segments show a change bar until "标记本文档已读". [LENS-05, LENS-06, NFR-06, AC-2b]
  3. Every Lens segment can one-click-expand its Base original; "需决策"/risk segments force-attach the Base excerpt, and a "报告失真" button feeds the <5% fidelity guardrail; projection shows estimated token cost with an auto (≤5k) / manual threshold, and the global settings page shows the month's per-type LLM token spend. [LENS-07, LENS-10, NFR-05, AC-2c, AC-2d]
  4. The user comments on any Lens or Base block (hover button / selection floater, capturing a quote), choosing 💬提问 / ✏️修改 (default) / ✅Approve / ❌Reject with free-language body (Chinese-first), and replies in threads driven by an `open → sent → needs-review → resolved / reopened` state machine; the Lens itself is never editable. [COMMENT-01, COMMENT-02, COMMENT-03, COMMENT-04, LENS-08, AC-3c]
  5. Comments persist in the local store (never the source `.md`), aggregate in a per-doc sidebar filterable by status/type with a cross-doc needs-review Inbox, support document-level (non-block) comments, and retain 100% across a file rename. [COMMENT-05, COMMENT-06, COMMENT-07, AC-1c]

**Plans**: TBD
**UI hint**: yes

### Phase 5: ★ Comment → Agent Loop (F4)

**Goal**: Close the North-Star loop — comment → structured bilingual feedback → agent edits Base → re-project + needs-review → resolve — over a dual channel (loopback MCP + file protocol) so the core hypothesis (AC-4a) is validated end-to-end, including for users without MCP.
**Mode:** mvp
**Depends on**: Phase 4
**Requirements**: LOOP-01, LOOP-02, LOOP-03, LOOP-04, LOOP-05, LOOP-06, LOOP-07, LOOP-08, AGENT-01, AGENT-02, AGENT-04, NFR-07
**Success Criteria** (what must be TRUE):

  1. From selected open comments the user generates a Feedback Bundle (target file path + block locator via heading path + original excerpt + type + user's Chinese body + product-generated English intent summary + thread context + explicit instruction header), scoped to commented blocks + necessary parent headings — full doc excluded by default, with an optional "附带全文" toggle held to ≤30% of doc tokens. [LOOP-01, LOOP-04, AC-4b]
  2. The bundle delivers over two channels — a human-readable `.prismdocs/feedback/<ts>.md` (with a one-line agent trigger copied to clipboard) AND a loopback MCP server exposing `list_feedback` / `get_feedback` / `respond_to_comment` / `get_document_comments` — set up by a one-click generated CLAUDE.md/AGENTS.md protocol snippet + `.prismdocs/` `.gitignore` suggestion, with Claude Code first-class, Cursor supported, and other agents on file-only fallback. [LOOP-02, LOOP-03, AGENT-01, AGENT-02, AGENT-04]
  3. The full loop closes end-to-end: user comments → reflow → agent edits Base → PrismDocs re-projects, flips the affected comments to needs-review, and notifies via Inbox → user reviews in change-highlight context → resolve (counts as a closed loop, the North-Star event) or reopen. [LOOP-05, AC-4a]
  4. The loop also closes with no MCP installed — via the file protocol alone, PrismDocs detects the agent's Base change and flips comments to needs-review. [LOOP-02, AC-4c]
  5. Agent text replies to 提问 comments appear in-thread; every Base change records its provenance (triggering bundle / MCP responder / external-unknown) shown in the review UI; bundle history is queryable; and opt-in telemetry captures the north-star `loop_closed` / `first_loop_closed` plus guardrail events. [LOOP-06, LOOP-07, LOOP-08, NFR-07]

**Plans**: TBD
**UI hint**: yes

### Phase 6: Cards ‖ Chrome Clipper (F5 ‖ F6)

**Goal**: Add the two independent knowledge-capture tracks in parallel — originality-nudged understanding cards that live only in the store, and the MV3 clipper that turns web pages into clean Markdown and survives MV3's service-worker lifecycle.
**Mode:** mvp
**Depends on**: Phase 5 (core loop shipped first); internally F5 needs the store [Phase 1/2] and F6 needs the bridge skeleton [Phase 1] — the two tracks parallelize.
**Requirements**: CARD-01, CARD-02, CARD-03, CARD-04, CARD-05, CARD-06, CLIP-01, CLIP-02, CLIP-03, CLIP-04, CLIP-05, CLIP-06, CLIP-07
**Success Criteria** (what must be TRUE):

  1. The user creates atomic Zettelkasten cards (title + Markdown body + tags, no folder hierarchy) with `[[`-invoked backlinks to docs/cards/clips and a backlinks panel; originality is nudged (the "你会怎么向朋友解释这件事？" placeholder, selected source dropped into a collapsed quote区 with a soft pre-publish reminder when the body is empty or highly duplicative) — and card bodies get NO AI ghostwriting. [CARD-01, CARD-02, CARD-03]
  2. Cards can be created in-flow — prompted on comment `resolve` (pre-filled with the decision's context link) and via "save selection as card" while reading — and a card list with full-text search filters by tag / project / linked object. [CARD-04, CARD-06, AC-5a]
  3. A card can be flagged `context-worthy` (with an optional AI-translated one-line English version) so Phase 7 picks it up by default. [CARD-05]
  4. The Chrome MV3 extension clips a full page (Readability) or a selection into clean Markdown with faithful code blocks (language-tagged, de-noised so Stack Overflow code copies clean), tables, lists, and images, capturing URL / site / capture-time / source language; the panel shows an editable title, target project, tags, a token estimate, and an optional "为什么剪它" note. [CLIP-01, CLIP-02, CLIP-03, CLIP-04, AC-6a, AC-6b, AC-6c]
  5. Clips reach the desktop app over a loopback bridge and survive both a 30s service-worker idle and the app being closed (an offline queue back-fills on reconnect); unfiled clips land in an Inbox for batch file / archive / delete; clips cannot be commented on. [CLIP-05, CLIP-06, CLIP-07]

**Plans**: TBD
**UI hint**: yes

### Phase 7: Context Pack & OKF Export (F7)

**Goal**: Assemble everything built so far into a token-budgeted, English-leaning Context Pack an agent can consume, and prove no-lock-in by exporting a genuinely consumable OKF Bundle.
**Mode:** mvp
**Depends on**: Phase 6 (assembles F1 docs + F5 cards + F6 clips) and Phase 5 (MCP host)
**Requirements**: PACK-01, PACK-02, PACK-03, PACK-04, PACK-05, PACK-06
**Success Criteria** (what must be TRUE):

  1. The user assembles a Context Pack via a tree selection of docs (Base) / cards (injection line first, body optional) / clips, seeing a live total token count and each item's share; `context-worthy` cards are pre-checked. [PACK-01, PACK-04, AC-7b]
  2. The pack writes to `.prismdocs/context/<name>.md` as structured, source-labeled, English-leaning output (Chinese cards get a machine English translation or原文 retained per the user's choice). [PACK-02, AC-7a]
  3. Frequently-used packs save as templates, and regenerating one after a source doc changed warns that the content has changed. [PACK-03, AC-7c]
  4. An agent can pull a pack directly via the MCP `get_context_pack` tool. [PACK-05]
  5. Selected content exports as a spec-compliant OKF Bundle directory (sidecar metadata materialized into six-field frontmatter, auto-generated `index.md`) that a real external OKF consumer can read back. [PACK-06]

**Plans**: TBD
**UI hint**: yes

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation & Core Engine Skeleton | 0/7 | Planned | - |
| 2. Project Import & FS Watcher (F1) | 0/TBD | Not started | - |
| 3. Block Anchoring Engine | 0/TBD | Not started | - |
| 4. Lens Projection ‖ Block Comments (F2 ‖ F3) | 0/TBD | Not started | - |
| 5. Comment → Agent Loop (F4) | 0/TBD | Not started | - |
| 6. Cards ‖ Chrome Clipper (F5 ‖ F6) | 0/TBD | Not started | - |
| 7. Context Pack & OKF Export (F7) | 0/TBD | Not started | - |
