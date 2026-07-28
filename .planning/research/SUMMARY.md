# Project Research Summary

**Project:** PrismDocs
**Domain:** Local-first macOS desktop knowledge workbench for AI-generated engineering docs (Tauri v2 + pure-Rust engine + React; block anchoring, comment-to-agent loop, Chinese digest layer, cross-project contract subscriptions)
**Researched:** 2026-07-28
**Confidence:** HIGH (stack verified against registries same-day; architecture/features/pitfalls cross-verified against official docs, advisories, and prior art)

## Executive Summary

This was a **verification pass**, not exploratory research: the project arrives with a mature docs/ set (BRD/PRD v0.3, TD-01 anchoring contract, phase-plan 调研). The verdict across all four research tracks is that the existing design **holds up against the mid-2026 state of practice**. Of 20 stack pins, 18 are CONFIRMED, 1 is AMENDED (Vite 7 → Vite 8, with a documented zero-cost fallback to 7), and 1 needs a CLARIFICATION (keyring 4 restructured into keyring-core + per-platform stores; the recommended concrete form is `keyring-core 1.0` + `apple-native-keyring-store 1.0`). The BRD's feature categorization survives market validation: block anchoring on externally-rewritten disk files, the 中文速读区, the three-signal closed loop with provenance, and personal-level cross-project drift alerts are all confirmed whitespace; every hard table stake is already in scope. TD-01's anchoring design is assessed at or above the state of practice (Hypothesis fuzzy anchoring, MSR robust anchoring) — no structural change needed.

The recommended approach is the docs' own Phase 1–9 build order (skeleton → F1 import → anchoring engine → F2′ digest ∥ F3 comments → F4 loop → F5+F7+F8 → release prep), with **three build-order amendments**: A1 — Phase 1 must include the event-bus spine + IPC adapter proof (one bus event round-trips as a coarse Tauri event; one command streams via Channel) and lock the single-writer SQLite discipline; A2 — the anchoring pure core (prism-parse + prism-anchor + calibration harness) may start in parallel with Phase 2, since it is a pure function over fixture trees; A3 — inside Phase 4, sequence prism-llm transport first so a digest-quality slip cannot block the critical path to F4.

The key risks cluster in two places. **Phase 1 carries five irreversible decisions** (single-writer + read-pool SQLite, FTS5 CJK tokenizer choice, keyring-core usage model, prism-mcp trait inversion to avoid a facade↔mcp dependency cycle, notify-then-fetch IPC pattern) — all cheap now, expensive to retrofit. **Phase 2 is the pitfall-densest phase**: atomic-save (temp+rename) misjudged as delete+create orphaning all comments, self-write echo loops, git checkout transient storms, cloud-sync silent rollback, and frontmatter byte-preserving write-back all land there. One TD-01 gap was found: A→B→A branch-switch semantics — degraded anchors currently have no revival path; this needs a TD-01 v0.2 amendment before Phase 3 closes.

## Key Findings

### Recommended Stack

The pinned stack is current and mutually compatible; dependency-tree alignment was stress-tested at the three risk points (rusqlite, reqwest, axum) and passes — `cargo tree -d` should show no duplicate rusqlite or reqwest. Full install block in STACK.md.

**Core technologies:**
- **tauri 2.11 (thin shell) + prism-engine Rust workspace** — engine testable without Tauri (D-01), the mainstream 2026 shape
- **rusqlite 0.40 `bundled`** — FTS5 verified compiled-in unconditionally; r2d2_sqlite 0.35 aligns exactly; pin bundled SQLite ≥3.51.3 (WAL corruption fix)
- **comrak 0.54** — sole block-boundary truth source; sourcepos + frontmatter delimiter cover TD-01
- **notify 8.2 + notify-debouncer-full 0.7** — rename stitching and FS-ID tracking built in; echo suppression is app-level by design
- **rmcp 2.2 StreamableHttpService on axum 0.8** — the upstream-blessed embedding pattern for D-07; verify exact feature-flag names at Phase 1
- **async-openai 0.41 + eventsource-stream 0.2 + reqwest 0.13** — single reqwest tree; hand-roll only the Anthropic-native SSE path
- **keyring-core 1.0 + apple-native-keyring-store 1.0** — the concrete form of the "keyring 4.1" pin (v4 is now a facade; upstream says link the core + store directly)
- **React 19 + Vite 8 (AMENDED from 7) + react-markdown 10 (render only) + CodeMirror 6** — Vite 8/Rolldown stable 4+ months; fallback to Vite 7 costs nothing if week-1 friction appears
- **similar 3.1, blake3 1.8, tiktoken-rs 0.12 (o200k, matches gpt-tokenizer), gray_matter 0.3 (parse only — never re-serialize frontmatter), htmd 0.5, ulid 1, dirs 6**

**What NOT to use:** any second Markdown parser for anchoring; tauri-plugin-stronghold/keyring; serde_yaml; stdio MCP proxy; tokio-rusqlite/sqlx; uniform-write connection pools.

### Expected Features

MVP scope (F1–F5, F7, F8-lite + anchoring engine) covers every hard table stake for the category. Three small additions found, none expanding scope materially:

**Must have (table stakes):**
- Rendered markdown review surface, block-anchored comments, structured feedback-to-agent, MCP + file-protocol fallback, approve/reject semantics, disk-canonical + sidecar, FS watch, change summaries + read baseline, version history, FTS search (CJK-capable), backlinks, local-first, BYO key — all already in scope
- **ADD:** Mermaid + full GFM rendering in Base view (LOW cost, credibility table stakes)
- **ADD (P0.5):** macOS system notifications for needs-review / drift alerts (loop-closure delivery)
- **ADD (onboarding):** frame 提问评论 as the "ask about docs" surface (expectation management vs DeepWiki-style chat)

**Should have (differentiators — all confirmed absent from every surveyed competitor):**
- 中文速读区 + ❓决策清单 with mandatory source excerpts (unique for CJK segment; funnel head for activation)
- Block anchoring with 0 silent loss under external agent rewrites (the moat)
- Closed loop with three-signal recovery + provenance + timeline
- Cross-project contract subscription + drift alerts (confirmed whitespace at personal level)
- Hand-written cards + originality nudge; Context Pack with token accounting; OKF typed cross-links

**Defer (v1.x/v2+):** 全文 Lens, F6 clipper, semantic drift detection, doc Q&A panel, team/cloud anything. Anti-features validated correct: no auto-wiki-generation, no editable digest, no AI auto-review of every revision, no AI-written cards.

### Architecture Approach

The 调研 §2.1 shape — Engine Facade + 6 domain crates + thin Tauri shell — matches established Tauri v2 best practice. Frontend holds view-model state only; SQLite is the single source of truth; invalidation is **notify-then-fetch** (coarse Tauri events carry IDs/counters only; Channels for request-scoped streams; queries for truth) because Tauri events are documented to process out of order under async listeners.

**Major components:**
1. **prism-engine (facade)** — orchestration, tokio-broadcast event bus, service-trait impls; the only crate the shell calls
2. **prism-store** — WAL + single dedicated writer + r2d2 read pool (`query_only=ON`), FTS5, migrations; migration persist stays in one writer transaction (TD-01 §5.1)
3. **prism-fs** — watch/debounce, rename stitching, the single `write_registered` echo-suppression primitive, 5-min content-hash reconciliation scan
4. **prism-parse / prism-anchor** — comrak pinned options → Block tree; pure `migrate(old, new) → MigrationResult` (frozen contract, 4 consumers)
5. **prism-llm** — only network egress + only key reader; SSE streaming, retry, cache-key discipline
6. **prism-mcp** — rmcp on loopback axum; **depends on service traits injected at construction, never on prism-engine** (trait inversion, Phase 1 decision) — this also enforces "writes = comment receipts only" at one choke point
7. **CLI helper binary** — keyring token read + SessionStart check-feedback; links no engine crates; needs the Phase 6 port-discovery pre-decision (a file in Application Support, since D-07 killed `.prismdocs/mcp.json`)

### Critical Pitfalls

1. **Second parser creeping into anchoring** — frontend only ever consumes engine-shipped `Block[]` spans; make full-text-only interfaces unavailable in the IPC contract; CI byte-compare smoke test (Phase 3/5)
2. **Orphan/degraded path itself failing silently** (Hypothesis's own production bug class) — property-test the completeness invariant, automate consumer-side reconciliation (comments table ↔ migration_log), make Degraded comments first-class visible in UI (Phase 3/5/8)
3. **Atomic save (temp+rename) misjudged as delete+create** → whole document's comments orphaned — delete grace window + content-hash identity; test matrix hard-wired: VS Code save, vim `:w`, Claude Code Write, `git checkout`, each with 100% comment retention (Phase 2, its single most important acceptance)
4. **Self-write echo loops + git transient storms** — one `write_registered` primitive with hash matching (time windows are unreliable), no naked `fs::write` to user repos; watch `.git/HEAD`/`.git/index` for batch mode; **A→B→A degraded-anchor revival semantics are undefined in TD-01 — amend as v0.2 before Phase 3 closes** (Phase 2/3/4)
5. **Phase-1 irreversibles:** single-writer SQLite (WAL write-write is still exclusive; pool-wide writes → BUSY storms and a real corruption class) and FTS5 CJK tokenizer (`trigram` or external; unicode61 returns silent zero results for Chinese and tokenizer change = full index rebuild)
6. **MCP DNS rebinding** — rmcp had a real CVE (GHSA-89vp-x53w-74fx); Host check + Origin allowlist + bearer token, all three explicit, with malicious-Origin 403 tests (Phase 6)
7. **Calibration corpus must be real agent diffs** (Claude Code/Cursor sessions, CJK-mixed), not synthetic edits — agent rewrites are bimodal and synthetic thresholds fail silently in production (Phase 0/3)

Also on the radar: partial LLM streams must never be cached (finish_reason + structure check + version-bound cancellation, Phase 4); frontmatter write-back must be byte-preserving (never `serde_yaml::to_string`, AC-1g-2 with CRLF/no-trailing-newline/YAML-anchor samples, Phase 2); cloud-sync directories (iCloud/Dropbox) silently roll back writes — detect + write-then-read-back verify (Phase 2/6); signing/notarization smoke test after Phase 6, not first attempted at Phase 8.

## Implications for Roadmap

The docs' Phase 1–9 structure is dependency-correct and confirmed. Roadmap should adopt it with amendments A1–A3 folded in.

### Phase 1: 基建骨架 (skeleton + irreversible decisions)
**Rationale:** Five decisions are cheap now, prohibitive later.
**Delivers:** Workspace + schema v1 + keyring + **event-bus spine and IPC adapter proof (A1: one bus→coarse-event round-trip, one Channel stream)**.
**Locks in:** single-writer + read-pool SQLite with PRAGMA set; FTS5 CJK tokenizer (trigram or dual-index); keyring-core vs keyring-v1 choice; prism-mcp trait-inversion boundary; notify-then-fetch pattern. Exit: `cargo tree -d` clean; verify rmcp feature-flag names.
**Avoids:** Pitfalls 7 (SQLite concurrency) and 8 (FTS CJK).

### Phase 2: F1 导入/同步 (highest pitfall density — budget accordingly)
**Rationale:** Everything downstream consumes its identity/version layer.
**Delivers:** Import wizard, watcher pipeline, doc identity + version snapshots, `write_registered` primitive, git batch mode, sync-dir detection, byte-preserving frontmatter write-back, FTS acceptance with Chinese queries.
**Avoids:** Pitfalls 4 (atomic-save deletion misjudgment — the phase's #1 acceptance), 5 (echo), 6 (git storms), 11 (cloud sync), 13 (frontmatter).
**Parallel (A2):** anchoring pure core + calibration harness on fixture corpora starts now.

### Phase 3: 锚定引擎 (the moat; interface freeze)
**Rationale:** Four consumers (F3/F2′/F4/F8) read the frozen MigrationResult/ChangeSet contract — the load-bearing wall.
**Delivers:** Store/pipeline integration, threshold calibration on real agent diffs, completeness property tests, "frontend zero-parsing" written into the consumer contract.
**Pre-work / blocker:** define **A→B→A degraded-anchor revival semantics as TD-01 v0.2** before phase close; CJK-mixed similarity tokenization spike (TD-01 §12-2).

### Phase 4 ∥ Phase 5: F2′ 速读区 ∥ F3 评论
**Rationale:** Share nothing but Phase 3 outputs; safe to parallelize.
**Phase 4 (A3):** prism-llm transport (streaming/retry/keyring) first, digest features second — narrows the 4→6 edge to "transport done". Cache only on normal stream termination + structure check; regen concurrency cap + cost gate; Mermaid/GFM rendering completeness lands with the Base view.
**Phase 5:** comment store/UI + consumer-side reconciliation + Degraded-comment visibility.

### Phase 6: F4 回流闭环 (integration terminus)
**Rationale:** Needs comments (5) and LLM channel (4); where AC-4a becomes verifiable.
**Delivers:** Bundle (compact Markdown, length-capped), three-signal recovery with FS fallback at **equal** implementation strength, MCP server with Host+Origin+bearer (rmcp pinned, malicious-Origin tests), port-discovery decision, per-agent install wizard (Cursor token path ≠ Claude Code headersHelper), real-session execution-rate measurement.
**After close:** end-to-end notarization smoke on a clean machine (CLI helper externalBin is the known landmine).

### Phase 7: F5 + F7 + F8-lite
**Rationale:** F8 composes anchoring (3) + F4 loop (6) — do not parallelize ahead of F4. Cards/Context Pack register tools into the Phase 6 MCP server.
**Delivers:** Cards, Context Pack, contract subscription + **aggregated** drift alerts (batch-level, anti-fatigue), one-click downstream check, system notifications (P0.5).

### Phase 8: 发布准备
**Delivers:** signing/updater full pass, anchor-loss release-gate retest (AC-3b end-to-end, not engine-only), kill -9 data-safety, snapshot retention policy, sidecar backup.

### Phase Ordering Rationale

- 2→3 because anchoring consumes F1's version rows; 3 before 4/5 because both consume the frozen contract; 6 after 4+5; 7 after 6 (F8's "zero new protocol" claim depends on the F4 Bundle shipping first).
- A1/A2/A3 shorten the critical path 1→2→3→5→6 without adding integration risk.
- F2′ quality is an **activation dependency** (速读区 → ❓ → comment is the funnel head for the 7-day first-loop metric), not just a feature.

### Research Flags

Phases likely needing phase-time research:
- **Phase 2:** macOS FSEvents behavior under git operations; reconciliation scan sizing
- **Phase 3:** similar algorithm variants + CJK-mixed tokenization for word-level ratio (spike)
- **Phase 6:** Claude Code / Cursor MCP client config specifics (headersHelper contract, hook syntax — client formats churn); rmcp pin vs MCP spec revisions

Standard patterns (skip research-phase):
- **Phase 1:** all decisions pre-made here with HIGH-confidence sources; only the rmcp feature-flag names need a 5-minute README check
- **Phases 4/5/7:** well-documented patterns (SSE streaming, CRUD + UI, composition of existing engines)

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All versions verified same-day against crates.io/npm/docs.rs; only rmcp feature-flag names and Vite-8 ecosystem judgment are MEDIUM |
| Features | MEDIUM | WebSearch cross-checked against official product pages; internal BRD/PRD analysis treated as primary input |
| Architecture | MEDIUM-HIGH | Converges across Tauri official docs, rmcp SDK patterns, SQLite practice, and anchoring prior art |
| Pitfalls | MEDIUM-HIGH | Critical items backed by official advisories/specs/production evidence (HIGH); some single-source items (LOW) flagged inline |

**Overall confidence:** HIGH — verification pass over an already deep design; findings converge rather than conflict.

### Gaps to Address

- **A→B→A degraded-anchor revival**: undefined in TD-01; amend as v0.2 during Phase 3 (blocks phase close, not phase start)
- **rmcp feature-flag exact names**: verify against rmcp 2.2 README at Phase 1
- **CJK trigram tokenizer vs project corpus**: documented behavior, not yet tested against real data — validate in Phase 2 FTS acceptance
- **Bundle execution rate by real agents**: no way to de-risk except M0/Phase 6 measurement with real Claude Code + Cursor sessions
- **Vite 8 friction**: watch week 1; fallback to Vite 7 is zero-cost

## Sources

### Primary (HIGH confidence)
- crates.io API + npm registry + docs.rs — all version pins and dependency manifests (2026-07-28)
- libsqlite3-sys build.rs — FTS5 in bundled; SQLite WAL docs + forum (corruption mechanics, 3.51.3 fix)
- GHSA-89vp-x53w-74fx (rmcp DNS rebinding) + MCP transports spec (Origin MUST)
- Hypothesis fuzzy anchoring + orphaned-annotation study + product-backlog#954; MSR robust anchoring
- Internal: BRD/PRD v0.3, TD-01, 调研_技术基建与开发Phase, 调研_整体构想v2

### Secondary (MEDIUM confidence)
- Tauri v2 docs (events vs Channels, workspace structure), notify-debouncer-full docs, watchexec FSEvents notes
- vite.dev release blog (Vite 8 / Rolldown timeline)
- Competitor pages: Plannotator, markupmarkdown, HackMD, Kiro, Code Wiki/DeepWiki; Riftmap/Mabl cross-repo posts
- SQLite pooling write-performance posts (multiple independent sources agree)

### Tertiary (LOW confidence)
- Individual GitHub issues on agent instruction-following and sync-dir rollback — directional, validate via M0 measurement

---
*Research completed: 2026-07-28*
*Ready for roadmap: yes*
