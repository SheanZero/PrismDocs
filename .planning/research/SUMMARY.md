# Project Research Summary

**Project:** PrismDocs
**Domain:** Local-first two-layer AI-doc workbench (macOS-first desktop + Chrome MV3 extension + embedded local MCP server) for non-English-native vibe coders
**Researched:** 2026-07-27
**Confidence:** MEDIUM-HIGH

## Executive Summary

PrismDocs is a local-first desktop knowledge workbench that maintains a compact English "Base" layer (source of truth, owned by the coding agent) and auto-projects a native-language, colloquial "Lens" layer for humans to review. Humans comment on blocks; comments flow back to Claude Code / Cursor as structured bilingual feedback to drive the next iteration. Experts in this space build such tools as a **single-writer core engine over authoritative disk files with a rebuildable SQLite sidecar** — never polluting the user's `.md`, reconciling *to* disk on every change, and exposing state to agents over a loopback-only, capability-scoped MCP server plus a file-protocol fallback. All four research dimensions converge on one conclusion: **the Block-anchoring + anchor-migration subsystem is simultaneously the product's deepest moat and its #1 existential risk.**

The recommended build is **Tauri v2 with a Rust-max core** (FS watch, Block AST parse via comrak, anchor migration via `similar`, SQLite via rusqlite, keychain, LLM, and MCP via rmcp — zero bundled Node runtime), a React/Vite webview for UI, and a separate WXT-based MV3 extension. The one significant open architecture decision (captured as an ADR): Rust-max vs. a Node-sidecar fallback for MCP/LLM — the sidecar buys the most battle-tested SDKs at the cost of ~40-80 MB of Node runtime. The core value loop is F1→(F2‖F3)→F4, with F5‖F6 and F7 following; this loop (AC-4a: comment → reflow → agent edits Base → re-project + needs-review → resolve) *is* the North-Star hypothesis and must ship end-to-end first.

The dominant risk is **silent comment loss when the AI heavily rewrites Base** (AC-3b: ≥90% correctly migrated-or-downgraded, 0 silent loss). Anchoring must be multi-strategy from day one (content-hash + heading-path + neighbor fingerprints + fuzzy quote match, git-diff-style alignment, confidence-gated downgrade to document-level with an immutable quote snapshot — never a silent drop), and must be built and hardened as a **prerequisite phase against an adversarial golden-corpus test harness before F2/F3/F4 can exist.** Secondary risks: two-layer projection staleness/races (version-gated cache + cancellable scheduler), FS-watcher hazards (atomic-save churn, agent burst-writes), LLM cost blowup + fidelity distortion (measurable incrementality + forced Base excerpt on decision blocks), the MV3 service-worker-kills-native-messaging bug (push toward a stateless loopback HTTP bridge + offline queue), and loopback MCP security (127.0.0.1 + token + Origin check + minimal write surface).

## Key Findings

### Recommended Stack

Build on **Tauri v2 with a Rust-centric core** ("Rust-max, zero bundled Node runtime"): the Rust engine owns FS watch, Block AST parse, anchor migration, SQLite, keychain, LLM calls, and MCP; a React + Vite webview renders the UI. This keeps the always-resident local-first app light (~8-12 MB vs Electron's 150 MB+), unifies the toolchain, and reaches Windows (P1) at near-zero incremental cost. The load-bearing caveat is a documented **ADR: Rust-max vs. Node-sidecar** — if hand-rolling the Anthropic streaming client or the MCP tool logic proves painful, bundle one Node sidecar hosting `@modelcontextprotocol/sdk` + `@anthropic-ai/sdk` + `openai` (Rust core still owns FS/DB/parse/anchor/keychain). Architecture research leaned Electron-for-speed; stack research recommends Tauri; the shell-agnostic `core` package is designed so this decision does not gate F1.

**Core technologies:**
- **Tauri v2 (2.10) + Rust 1.85+** — desktop shell + engine language; small resident footprint, cross-platform, natural home for the CPU-bound anchoring engine.
- **comrak 0.54** — Markdown → Block AST with `sourcepos`; the single source of truth for Block boundaries/IDs (remark/react-markdown is render-only — never a second authoritative parser).
- **rusqlite 0.32 (bundled, WAL) + FTS5** — sidecar metadata store (Block IDs, comments, cards, Lens cache, provenance) and <300ms full-text search; disk `.md` remains authoritative.
- **similar 2.x + blake3** — diff-based Block anchor migration (the moat) + content hashing for stable IDs and rename detection.
- **rmcp 2.2** — Rust MCP server over stdio/loopback (pin to 2025-11-25 stable protocol for launch); `keyring` crate for API keys (NOT tauri-plugin-stronghold — deprecated in v3).
- **WXT 0.20 + @mozilla/readability + turndown (+gfm) + gpt-tokenizer** — MV3 extension; custom turndown code-block rule is the real work (AC-6a clean Stack Overflow code). Extension↔app via **loopback WebSocket/HTTP bridge** (chosen over Native Messaging for offline queue + cross-browser + no MV3 SW death).

### Expected Features

The PRD's F1–F7 are all P0 and already specified; the research classifies them and — critically — maps the dependency graph. No competitor holds the *combination*, and none serves the non-English-native reader at all.

**Must have (table stakes):** folder/Git import with FS-sync (F1), YAML frontmatter parse (F1.8), Google-Docs-style threaded block comments (F3), AI doc summary (F2 baseline), web→clean-Markdown clip (F6), `[[`-backlinks (F5.3), FTS + filtering (F5.6), token-count display (F6.3/F7.1). Missing any = product feels broken; zero credit for having them.

**Should have (differentiators — the moat, defense-ordered):**
- **Two-layer Base(EN)/Lens(native) one-way block-anchored projection (F2)** — the #1 moat, the reason the product exists; no competitor has a native-language human layer.
- **Anchor survival across AI rewrites — 0 silent loss, graceful degrade (F3/§2.4)** — VERY HIGH complexity; the deepest, least-copyable moat and the top technical risk. Shared substrate for F2/F3/F4.
- **Comment → structured bilingual feedback bundle → agent, dual channel (F4)** — native comment + product-generated EN intent summary; ahead of Plannotator (file/hook only).
- Incremental per-Block re-projection + change-highlight (F2.4/2.5); fidelity safeguards — forced Base excerpt on decision blocks + distortion-report (F2.6); Zettelkasten "in your own words" nudge with deliberate NO AI-ghostwrite (F5.2); engineering-framed clip with token accounting (F6); Context Pack + OKF export (F7).

**Defer (P1+, architecture-reserved):** multi-language Lens (JA/EN), document graph view, ingestion conflict detection (CoWiki-style), GitHub PR / multi-user, spaced-repetition review.

**Explicit anti-features (product-principle refusals, not "later"):** editing the Lens layer, AI ghostwriting card bodies, commenting on clips, built-in IDE/code-gen, general-purpose note app, publishing site, team/multi-user, server-side sync, writing Block IDs into user `.md`, multi-branch doc views.

### Architecture Approach

Three OS processes plus one external browser process, with **all persistent state behind a single-writer Core Engine**; the UI is a thin projection over a typed IPC contract and never touches disk or DB; disk Markdown is authoritative and SQLite is a rebuildable index/sidecar (disk wins on conflict). The core package is deliberately shell-agnostic so the Tauri/Electron decision doesn't block the hard problems, and the `anchor/` subsystem is its own package with a dedicated adversarial fixture corpus.

**Major components:**
1. **Import / FS Watcher** — glob scan, debounced (2s) FS events, HTML→MD, frontmatter parse, rename-by-content-hash; disk authoritative.
2. **Markdown Parser → Block Tree** — comrak/mdast; block boundaries + volatile current-version line ranges only.
3. **★ Anchor Engine ★** — stable opaque Block IDs; multi-pass diff-migration (exact → structural → fuzzy alignment) with multi-signal confidence scoring; confidence-gate that **downgrades to document-level, never drops**. The spine everything anchors to.
4. **Lens Scheduler** — content-hash-invalidated incremental re-projection, context-adjacent soft-dirty, cancellable/debounced, streaming, restart-safe cache.
5. **Domain Services + Provenance Tracker** — Comment thread FSM, Card, Clip, ContextPack, FeedbackBundle; base-change source attribution.
6. **LLM Client, Store Layer, MCP Host, Extension Bridge Host** — keychain keys + custom base_url; SQLite WAL + FS I/O; loopback+token+Origin MCP with one write tool; offline-queue-tolerant extension bridge.

### Critical Pitfalls

1. **Silent comment loss on heavy AI rewrite (THE #1 RISK)** — Design for downgrade, never drop (no code path deletes an orphaned comment); multi-strategy anchoring (quote+context+position+structural); handle split/merge explicitly (never force 1:1); immutable quote snapshot at creation; adversarial golden-corpus test as a CI release gate with the count-invariant assertion (migrated + downgraded == original).
2. **Two-layer sync corruption / stale-Lens race** — Version every Base block (cache key = blockId+baseVersion); single cancellable projection scheduler (newer version supersedes in-flight); wait for FS settle + valid-parse before projecting; never present stale Lens as current; enforce Lens read-only at the storage layer.
3. **FS watcher hazards (atomic-save churn, burst writes, missed events)** — `awaitWriteFinish` size-stable polling; identity by content-hash+inode not path; trailing debounce with max-wait cap; exclude `.prismdocs/` from the watcher; explicit delete/rename state machine.
4. **LLM cost blowup + fidelity distortion** — Make incrementality measurable and gated (reprojected/changed ratio alarm); forced Base excerpt on every decision block (AC-2c); one-click "report distortion" feeding the <5% guardrail (AC-2d); two-tier model routing (fast floor + strong on decision blocks); treat doc/clip text as untrusted data (prompt-injection).
5. **MCP + MV3 lifecycle/bridge failures** — MV3 service worker kills native messaging after ~30s idle (verified, bit Claude Code itself) → prefer a stateless loopback HTTP endpoint + retry + mandatory offline queue; lock down loopback (127.0.0.1 + per-install token + Origin validation); minimal MCP write surface (read + `respond_to_comment` only, Workspace-scoped).
6. **File pollution / round-trip breakage / OKF type sprawl** — Sidecar by default (source writes opt-in only, Q7); round-trip-preserving YAML (byte-stable untouched keys); registration-gated controlled `type` vocabulary (9 built-ins, Q6); validate OKF export is actually consumable.
7. **Adoption inertia + alert fatigue** — Zero-config first value (one-click generated MCP/CLAUDE.md snippet); Inbox as single batched rhythm, not per-event toasts; file-protocol degrade path (AC-4c) as a first-class tested flow.

## Implications for Roadmap

Based on research, the F1–F7 dependency graph and the pitfall-to-phase mapping strongly suggest the following structure. The through-line: **the anchoring subsystem is not a slice inside F1 — it is a hardened prerequisite phase with its own verification gate that must pass before F2/F3/F4 planning is meaningful.**

### Phase 0 (cross-cutting foundation, from day one)
**Rationale:** Several concerns are needed by every subsequent phase and should not be a discrete feature phase.
**Delivers:** SQLite schema + migrations, LLM client (keychain, retry, token accounting), typed IPC contract, MCP loopback host skeleton, shell-agnostic `core` package scaffold, **and the Tauri-vs-Node-sidecar ADR resolved.**

### Phase 1: Import + FS Watcher + Block Tree
**Rationale:** The watcher's output contract (debounced, settled, identity-resolved change events) is a hard dependency for everything; disk-authoritative sidecar discipline is set here.
**Delivers:** folder/Git import, FS watch with atomic-save + burst handling, comrak Block tree, frontmatter round-trip, controlled `type` vocabulary, sidecar store.
**Addresses:** F1. **Avoids:** Pitfalls 3 (watcher) and 6 (pollution/round-trip/OKF sprawl). **Verify:** AC-1b (5 docs/10s), AC-1c (rename retains 100% comments).

### Phase 2: ★ Block Anchoring Engine ★ (PREREQUISITE, hardened before F2/F3/F4)
**Rationale:** The #1 moat and #1 risk. Weak migration means comments drop (F3 fails), Lens mis-maps (F2 fails), reflow targets wrong blocks (F4 fails). Must be built and adversarially tested in isolation first.
**Delivers:** stable opaque Block IDs; multi-strategy migration (content-hash + heading-path + neighbor fingerprints + fuzzy quote/diff alignment); confidence gate with downgrade-not-drop; immutable quote snapshots; adversarial golden-corpus harness.
**Avoids:** Pitfall 1. **Verify:** AC-3b (≥90% migrated-or-downgraded, 0 silent loss) + count-invariant assertion in CI.

### Phase 3: Lens Projection (F2) ‖ Block Comments (F3) — parallel
**Rationale:** Both consume the stable `block_id` from Phase 2; F3 shares the migration/downgrade path with F2. Parallelizable once anchoring is stable.
**Delivers:** F2 incremental re-projection scheduler (version-gated, cancellable, streaming cache) + fidelity safeguards + two-tier model routing; F3 threaded comments with anchor-migration downgrade UI.
**Uses:** comrak + `similar` + LLM client. **Implements:** Lens Scheduler, Comment domain service. **Avoids:** Pitfalls 2 and 4.

### Phase 4: Comment → Agent Loop (F4) — ★ MVP milestone (AC-4a)
**Rationale:** Closes the North-Star loop; the make-or-break validation. Needs F3 comments, F1 FS-watch closure detection, MCP host, provenance.
**Delivers:** FeedbackBundle builder (bilingual, ≤30% doc tokens), dual channel (`.prismdocs/feedback/*.md` + loopback MCP), closed-loop recycling, one-click config, provenance.
**Avoids:** Pitfalls 5 (MCP auth/tool surface) and 7 (inertia/degrade path). **Verify:** AC-4a end-to-end, AC-4c file-only path.

### Phase 5: Cards (F5) ‖ Chrome Clipper (F6) — parallel
**Rationale:** F5 depends only on the store; F6 is the most independent (separate extension + bridge track, can start anytime after Phase 0's bridge skeleton).
**Delivers:** F5 originality-nudge cards + backlinks + inject-to-context flag; F6 WXT extension (Readability + turndown code-fidelity, token estimate, loopback bridge + offline queue).
**Avoids:** Pitfall 5 (MV3 SW/native-messaging → loopback HTTP + offline queue, store-review hygiene). **Verify:** AC-6a clean SO code; clip survives 30s idle + app-closed.

### Phase 6: Context Pack + OKF Export (F7)
**Rationale:** Late/integrative by nature — assembles F1 docs + F5 cards + F6 clips, adds LLM compression, MCP `get_context_pack`, OKF materialize-on-export.
**Delivers:** token-budgeted Context Pack builder + OKF Bundle export (validated consumable).
**Avoids:** Pitfall 6 (OKF export validation).

### Phase Ordering Rationale
- **Anchoring dominates the ordering.** F2/F3/F4 all sit on `block_id`; the anchoring engine is pulled out of F1 into its own hardened prerequisite phase (Phase 2) with a dedicated adversarial verification gate.
- **F1→(F2‖F3)→F4→(F5‖F6)→F7** follows the discovered dependency graph; F5/F6/F7 attach to docs/projects (not to Blocks) so they parallelize and come after the core loop is proven.
- Grouping F2+F3 and F5+F6 in parallel matches the architecture's fan-out at the anchor migration point and the independence of the store-only / extension tracks.
- The F1–F4 core loop is sequenced first to validate the North-Star hypothesis before building peripheral capabilities (matches the PRD milestone strategy).

### Research Flags

Phases likely needing deeper research during planning (`/gsd-plan-phase --research-phase`):
- **Phase 2 (Anchoring):** deepest-research phase in the project — similarity metric choice (lexical vs embedding), confidence threshold tuning, split/merge handling, golden-corpus design. Flag for a dedicated technical design + spike.
- **Phase 3 (Lens/F2):** Q1 model-routing A/B, projection prompt as an eval-worthy AI-integration artifact (fidelity <5%, prompt-injection). Flag for AI-integration spec.
- **Phase 5 (F6 clipper):** MV3 service-worker-lifecycle + native-messaging-vs-loopback decision is architectural. Flag for an MV3-lifecycle spike.
- **Phase 4 (F4/MCP):** loopback security model + rmcp protocol version pinning; lighter research (patterns well-documented in ARCHITECTURE.md).

Phases with standard patterns (skip research-phase):
- **Phase 5 (F5 cards):** deliberately simple — backlinks + FTS are well-established.
- **Phase 1 (F1 import):** chokidar/notify + gray_matter patterns well-documented (watcher hazards are known, mitigations in PITFALLS.md).

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | MEDIUM-HIGH | Shell/storage/AST/extension HIGH (official docs, current versions); Rust LLM clients + MCP spec churn MEDIUM; Tauri-vs-sidecar shell choice needs a prototype/ADR. |
| Features | MEDIUM | Competitor mechanics cross-checked; anchor-survival internals are proprietary/opaque across the field — HIGH on "hard/unsolved", LOW on any single vendor's exact algorithm. |
| Architecture | HIGH | Anchoring model, incremental-projection, data model, MCP security grounded in PRD/BRD + established patterns (mdast positions, Myers/patience diff, FTS5, loopback MCP). MEDIUM on exact confidence thresholds and shell choice. |
| Pitfalls | HIGH | Technical fundamentals verified against Chrome/Hypothesis/OKF official sources; MV3 native-messaging failure independently verified (Chrome tracker + Claude Code issue). |

**Overall confidence:** MEDIUM-HIGH

### Gaps to Address
- **Desktop shell decision (Tauri Rust-max vs Node-sidecar):** stack recommends Tauri, architecture leaned Electron-for-speed. → Resolve as a Phase 0 ADR; `core` is shell-agnostic so it doesn't gate F1.
- **Anchor confidence thresholds + similarity metric:** research gives a scoring formula but exact HIGH/LOW cutoffs are un-tuned. → Tune against the golden corpus in Phase 2; treat thresholds as empirically set, not guessed.
- **Q1 Lens model routing (cost vs quality):** unresolved and load-bearing. → M0/Phase 3 A/B test with token-cost display.
- **Q6 controlled `type` vocabulary + Q7 frontmatter-to-source opt-in:** architecture-level. → Resolve in F1/Phase 1 design review.
- **rmcp protocol version churn:** new spec lands 2026-07-28. → Pin to 2025-11-25 stable for launch; upgrade only after Claude Code/Cursor adopt.
- **Anthropic exact token counts:** local tiktoken-rs covers OpenAI-family; Claude needs the `count_tokens` REST endpoint. → Acceptable ±10% estimate in the extension; exact counts from the desktop app.

## Sources

### Primary (HIGH confidence)
- Internal authoritative specs: `.planning/PROJECT.md`, `docs/PRD_PrismDocs_MVP.md` (F1–F7 REQ+AC, §2.4 anchoring, §2.5 OKF, §4 agent protocol, §5 NFR, §8 open questions), `docs/BRD_PrismDocs_MVP.md` (§6 mechanisms, §11 risks), `docs/调研补充_CoWiki与OKF对BRD_PRD的影响.md`.
- Tauri v2 (v2.10.1), comrak 0.54 (CommonMark + GFM + sourcepos), rmcp / MCP TS SDK, rusqlite/ORM comparison, turndown, @mozilla/readability, gpt-tokenizer, OpenAI Node SDK — official docs / crates.io / npm.
- MCP Transports spec (2025-11-25): stdio vs Streamable HTTP, DNS-rebinding / Origin validation for local servers.
- Chrome service-worker lifecycle + "longer ESW lifetimes" (official; 30s idle, port no longer resets timer); anthropics/claude-code #16350 (verified native-host-dies reproduction).
- Hypothesis Fuzzy Anchoring + judell/TextQuoteAndPosition (canonical multi-strategy text anchoring).
- Google OKF v0.1 spec + Marc Bará critique (type-vocabulary sprawl).

### Secondary (MEDIUM confidence)
- Plannotator, markupmarkdown 1.0, Google Code Wiki, Obsidian Web Clipper stack (Defuddle + Turndown + Joplin GFM) — vendor docs + secondary coverage.
- Tauri Node-sidecar footprint comparison; async-openai / tiktoken-rs Rust LLM path; @anthropic-ai/sdk (exact patch unpinned).
- stdio-vs-HTTP MCP transport guidance; Hypothesis fuzzy-anchoring perf-cliff issue.

### Tertiary (LOW confidence)
- Any single vendor's exact anchor-survival algorithm (proprietary/opaque across the field) — the "hard/unsolved" claim is HIGH, the internals are inferred.

---
*Research completed: 2026-07-27*
*Ready for roadmap: yes*
