# Feature Research

**Domain:** Local-first two-layer AI-doc workbench for non-English-native vibe coders (desktop + MV3 extension + local MCP)
**Researched:** 2026-07-27
**Confidence:** MEDIUM (competitor mechanics cross-checked across vendor docs + secondary coverage; anchor-survival internals are proprietary/opaque across the field, so those claims are HIGH-confidence on "hard/unsolved" but LOW on any single vendor's exact algorithm)

> Scope note: PrismDocs' F1–F7 are already specified in `docs/PRD_PrismDocs_MVP.md`. This file does **not** re-invent them. It classifies each capability as table-stakes / differentiator / anti-feature, grounds each in how named adjacent tools actually implement it, notes complexity, and maps the F1–F7 dependency graph for roadmap sequencing. The controlling insight for the roadmap: **Block anchoring (PRD §2.4) is the shared substrate for F2, F3, and F4 — it is the single highest-risk, must-come-first piece.**

---

## Feature Landscape

### Table Stakes (Users Expect These)

Missing these = product feels broken. Users give zero credit for having them.

| Feature (PRD ref) | Why Expected | Complexity | Grounding / Notes |
|---|---|---|---|
| Point at a local folder / Git repo, auto-import `.md`, keep in sync (F1) | Every doc/wiki tool ingests a source; DeepWiki/Code Wiki point at a repo; Obsidian opens a vault | MEDIUM | FS watcher + 2s debounce for agent burst-writes; rename detection by content hash; disk is source of truth, no file locking. Edge cases (external delete, >1MB, non-UTF8) are the real work |
| YAML frontmatter parse into structured metadata (F1.8) | Standard for any markdown KB; OKF makes `type` the one required field | LOW | Round-trip must not corrupt existing frontmatter; no-frontmatter files stay untouched (sidecar) |
| Select text → threaded margin comment with state (F3) | Google Docs / HackMD / markupmarkdown all do PR-style anchored prose comments; users expect "comment like Google Docs" | MEDIUM | markupmarkdown 1.0 is the direct model: drag-select → threaded margin comment + review state (approve / request-changes / comment). Comment CRUD + threads + filter sidebar is expected baseline |
| AI summary / speed-read of a doc (F2 speed-read region, REQ-2.3) | DeepWiki, Google Code Wiki, Mintlify all auto-summarize; summarization is now assumed, not novel | MEDIUM | The 3–5 sentence summary + decision-list header is table-stakes AI behavior; the *cross-language colloquial projection* around it is the differentiator (below) |
| Web page → clean Markdown, code fences preserved (F6) | Obsidian Web Clipper, Readwise Reader, Fabric, MarkSnip/MarkDownload all do this | MEDIUM | Proven stack: **Defuddle** (successor to Mozilla Readability, handles SPAs Readability misses) + **Turndown** + **Joplin GFM plugin** for tables/task-lists/fenced code. Do NOT hand-roll extraction |
| Bidirectional links / backlinks between cards & docs (F5.3) | Obsidian, Logseq, Roam, Tana, Heptabase — `[[`-autocomplete + backlink panel is the PKM baseline | MEDIUM | Users coming from Obsidian expect `[[` invocation and a "who links here" panel. Cycles allowed (Zettelkasten norm) |
| Full-text search + tag/link filtering (F5.6) | Any KB with >50 notes is unusable without it | MEDIUM | NFR: <300ms at 500 docs / 2000 cards. Local index (SQLite FTS) |
| Show token count for anything sent to an LLM (F6.3, F7.1) | Context-cost anxiety is universal in AI tooling in 2026; users expect to see the number | LOW-MEDIUM | AC target: ≤10% error vs real tokenizer. Table-stakes *display*; the budgeting UX is a differentiator |

### Differentiators (Competitive Advantage)

These are the moat. They align directly with PROJECT.md Core Value ("双层文档 + 评论回流"). Defense-order matches BRD §5.

| Feature (PRD ref) | Value Proposition | Complexity | Grounding / Notes |
|---|---|---|---|
| **Two-layer Base(EN) / Lens(native) with one-way block-anchored projection** (F2) | No competitor has a native-language human layer. DeepWiki/Code Wiki/Mintlify/markupmarkdown/Plannotator are all single-layer English for engineers who read English fine. This is the #1 moat and the reason the whole product exists | HIGH | Lens = projection not copy; not editable (prevents drift). Each Lens segment anchors to a Base Block. Reading modes: Lens-only / side-by-side / Base-only |
| **Anchor survival across AI rewrites — 0 silent loss, graceful degradation** (F3 / §2.4) | Google Docs orphans anchored comments the moment the text is deleted ("Original content deleted"); the whole field treats this as unsolved. Content-hash + position-heuristic migration with a confidence threshold that *degrades to doc-level comment rather than dropping* is the deepest, least-copyable moat | **VERY HIGH** | AC-3b: ≥90% correct migration or explicit degrade after 50% rewrite, 0 silent loss. Shared substrate for F2/F3/F4. **Flag for a dedicated research spike + adversarial test harness** |
| **Comment → structured bilingual feedback bundle → coding agent, dual channel** (F4) | Plannotator & markupmarkdown send feedback but single-layer English. PrismDocs bridges *native-language comment → product-generated English intent summary → agent*, plus closed-loop recycling and degraded file-only path. The bilingual bridge is unique | HIGH | Bundle = comment + evaluated block excerpt + parent heading path + EN intent summary + thread ctx; NOT whole doc (≤30% of doc tokens, AC-4b). Dual channel: `.prismdocs/feedback/*.md` (P0) + local MCP `list/get_feedback`, `respond_to_comment`, `get_document_comments` (P0). Plannotator's proven pattern: anchored feedback as structured Markdown + direct edits as unified diff, Claude Code hook on `ExitPlanMode` — adopt the shape |
| **Incremental re-projection + change-highlight since last read** (F2.4/2.5) | Code Wiki "rewrites docs after each change" but whole-doc; PrismDocs regenerates only affected Blocks (+ diff-adjacent) and shows per-segment change bars for human re-review. This is both cost design and the review-loop UX | HIGH | AC-2b: edit 1 paragraph → only affected segments re-called, ≤10s. Cache persistence across restart (NFR) |
| **Fidelity safeguards: forced Base excerpt on decision blocks + "report distortion" button** (F2.6) | Auto-wiki tools have no fidelity guardrail because their English readers just read the source. PrismDocs' native-language readers *can't*, so mistranslation → wrong decision is a top risk. Forced source excerpt on ⚖️/❓ blocks + distortion-report telemetry is a novel guardrail (also a North-Star protector metric source) | LOW-MEDIUM | AC-2c: every "needs-decision" segment carries Base excerpt, no exceptions. AC-2d: distortion rate <5% |
| **Zettelkasten "in your own words" nudge + deliberate NO AI-ghostwriting of card body** (F5.2) | Obsidian/Logseq/Tana have backlinks but no originality enforcement; most AI note tools push auto-generation, which *destroys* the comprehension-retention mechanism. Withholding the ghostwrite button is a differentiator-by-deliberate-absence, backed by comprehension-debt research | LOW | Quoted source folds into a citation region; body must be re-written; soft nag if body ≈ quote. AI only allowed post-publish "restatement QC" (P0.5) |
| **Clip framed as agent-spec material w/ token accounting** (F6) | Obsidian/Readwise clip into general notes. Framing the clip as *reusable context for a coding agent* — code-run-fidelity + token cost + "why I clipped it" note — is the unoccupied engineering-facing niche | LOW-MEDIUM (on top of table-stakes clipper) | AC-6a: Stack Overflow code copies & runs (no highlight-span noise). Site adapters: SO, GitHub README/Issue/Discussion, MDN, tech blogs |
| **Context Pack assembly with live token budget + OKF Bundle export** (F7) | Turns the KB into an on-demand, inspectable context builder (vs bloated CLAUDE.md). OKF export = "your KB is portable, readable by any OKF consumer, no lock-in" trust play; CoWiki already aligns to OKF so matching avoids losing the comparison | MEDIUM | Tree-select docs/cards/clips, live token total + per-item %, save as template. OKF export materializes sidecar → compliant frontmatter + auto `index.md` (P0.5) |
| **OKF compatibility as architecture-level decision** (§2.5) | Low-cost interop → no-lock-in trust sell; one concept = one file already isomorphic to OKF. Controlled `type` vocab (9 terms) answers the community critique of OKF's unregistered vocab | LOW-MEDIUM | Only `type` required by OKF v0.1; PrismDocs keeps `index.md`/`log.md` reserved semantics. Decide before build (it's a data-model choice) |
| **Agent contribution provenance** (F4.7) | Borrowed from CoWiki's "stop error propagation via source tracking"; ahead of OKF v0.2's planned provenance. needs-review UI shows "who changed this, triggered by which comment" | MEDIUM | Data structure now, UI at review time. Guards against unknown-origin content being amplified by agents |

### Anti-Features (Deliberately NOT Built)

The PRD is unusually disciplined here — these are product-principle refusals, not "later" items. Documenting so the roadmap does not accidentally build them.

| Anti-Feature | Why It Gets Requested | Why Problematic | What PrismDocs Does Instead |
|---|---|---|---|
| **Editing the Lens layer directly** | "I see the mistake in Chinese, let me just fix it here" | Two independently-editable layers → drift → the exact failure the product exists to prevent | Lens is read-only projection; all dissatisfaction routes through comments → Base (F2.7) |
| **AI ghostwrites the card body** | "Auto-summarize this doc into a card for me" | Kills the comprehension-retention mechanism; card becomes another unread AI artifact (the "理解债" the product fights) | Only the injection one-liner (for AI) may be AI-translated; body is human-only. Post-publish "restatement QC" allowed (F5.2) |
| **Comment on clips** | "This clip is wrong, let me annotate it" | Muddies the core semantic "comment = drive AI to change a doc"; clips are external raw material | Understanding a clip → write a card that links to it (F6.7) |
| **Built-in IDE / code editor / code-gen / embedded coding agent** | "Just let me edit code here too" | Head-on collision with Claude Code / Cursor / the giants; abandons the symbiotic wedge | Connect to agents via MCP + file protocol; be the human layer *outside* the IDE |
| **General-purpose / "life" note app** | "Can I keep all my notes here?" | Competes with Obsidian on its home turf; dilutes the engineering-context focus | Engineering context only; Obsidian keeps life notes |
| **Public documentation site / publishing** | "Export this as a docs site" | Competes with Mintlify/GitBook; different buyer, different job | Out of scope; OKF export covers portability |
| **Team collaboration, multi-user, permissions** | "Share with my team" | Huge surface (auth, sync, conflict, RBAC) for the P3 audience before the P1 hypothesis is validated | Single-Workspace single-user MVP; team version post-PMF |
| **Server-side / forced cloud** | "Sync across my machines" | Data-sovereignty concern for privacy-sensitive users; server cost with no revenue | Local-first, SQLite + files, single backup-able dir; git sync as P1 "portable mode" |
| **Write Block IDs / metadata into user's `.md` source** | "Just put the IDs in the file so agents see them" | Pollutes user files, breaks git/agent workflow, the thing that makes adoption zero-friction | Sidecar store; materialize to frontmatter only on explicit OKF export (Q7 keeps opt-in on the table) |
| **Multi-branch document views** | "Show me docs per git branch" | Scope explosion in F1 before core loop is proven | Git-*aware* (branch + commit hash association, P0.5); no multi-branch view in MVP (F1.7) |

---

## Capability-Area Findings (the 6 research questions)

**1. Block-level commenting + anchor survival.** Commenting UX is *table stakes* (markupmarkdown = "Google Docs for Markdown"; HackMD, CodeRabbit inline comments). Anchor *survival across edits* is the hard, differentiating part and is **unsolved in the open**: Google Docs anchors to a text range and orphans the comment to "Original content deleted" when the text is removed (its Drive-API even ignores externally-set anchors). CodeRabbit anchors to diff hunks/line ranges, which drift across force-pushes. None survive an *AI 50%-rewrite* gracefully. PrismDocs' content-hash + position-heuristic migration with a confidence-threshold degrade (never silent-drop) is genuinely novel — and the top technical risk (VERY HIGH). Roadmap must treat §2.4 as a first-class, spiked, adversarially-tested component before F2/F3/F4.

**2. Comment-to-agent structured feedback loops.** Plannotator is the closest existing implementation: it *exports anchored feedback as structured Markdown* (selected text + comment/deletion + source line info) with *direct edits as a unified diff*, and integrates with Claude Code by intercepting the `ExitPlanMode` hook; it runs fully local. markupmarkdown lands agent revisions as "proposed" behind a human accept-gate. Neither bridges languages. Confirmed design choices for F4: (a) structured-Markdown bundle is the right file format; (b) MCP + file dual-channel is ahead of Plannotator (file/hook only); (c) the EN intent-summary of a native-language comment is unique; (d) closed-loop recycling via FS-change detection is the essential degraded path (AC-4c). Complexity HIGH.

**3. AI summarization / "conversational projection".** DeepWiki (Cognition), Google Code Wiki (Gemini), Mintlify all auto-generate and *whole-doc* re-generate on change; Code Wiki even regenerates diagrams per commit. Summarization is table-stakes; three PrismDocs behaviors are differentiating and absent in all three: (a) **incremental per-Block re-projection** (cost + change-highlight, not whole-doc), (b) **change-highlight since last human read** with "mark as read", (c) **fidelity safeguards** (forced Base excerpt on decision blocks, distortion-report). The auto-wiki lane is giant-occupied (Code Wiki) — do not compete on "generate a wiki"; compete on the human-review projection loop. Complexity HIGH.

**4. Zettelkasten cards + backlinks + inject-to-context.** Obsidian, Logseq, Tana, Heptabase all nail `[[`-linking + backlink panels (table stakes). The originality enforcement — "how would you explain this to a friend?" placeholder, quote/body separation, soft nag on near-duplicate, and *withholding* AI ghostwrite — is the differentiator, grounded in comprehension-debt research (Addy Osmani). "Inject to context" (context-worthy flag → auto-picked by F7) is the bridge that makes cards a queryable external memory replacing bloated CLAUDE.md. Complexity LOW — deliberately simple (no folders, links only).

**5. Web clipper → clean Markdown for AI.** Red-ocean feature; adopt the proven stack (Defuddle + Turndown + Joplin GFM) rather than build extraction. Differentiation is *engineering framing*: code-run fidelity (AC-6a, no highlight-span noise), token estimation (AC-6c ≤10% error), site adapters for the actual pain sites (Stack Overflow, GitHub, MDN), and "why I clipped it" nudge. Native-messaging/local-port to desktop with offline queue is standard MV3 plumbing. Complexity LOW-MEDIUM.

**6. Context-pack assembly + OKF export.** No product occupies "assemble a token-budgeted English context pack from curated docs/cards/clips for a coding agent." OKF v0.1 (Google Cloud, 2026-06) standardized the container: dir of Markdown, one concept/file, `type` the only required frontmatter field, reserved `index.md`/`log.md`, plain-Markdown cross-links = queryable graph, Apache-2.0, no runtime. Its documented gap (Marc Bara critique: no registered vocab, no link typing, no trust/provenance) is *exactly* PrismDocs' value layer. Export = materialize sidecar → compliant frontmatter + auto `index.md`. Complexity MEDIUM. OKF-compat is a cheap no-lock-in trust sell; CoWiki already aligns.

---

## Feature Dependencies

```
F1 (import + Markdown-AST Block parse + Block-anchor substrate §2.4)
     ├──required-by──> F2 (Lens: each segment anchors to a Block)
     ├──required-by──> F3 (Comment: anchors to a Block; shares migration algo)
     │                      └──required-by──> F4 (reflow: packages comments → agent)
     ├──required-by──> F7 (docs are selectable Pack sources; OKF frontmatter model)
     └──enables─────> F5/F6 (project model to attach cards/clips to)

F2 (re-projection) ──enables──> F4 closed-loop recycle (re-project after agent edit)
F1 (FS-watch)      ──enables──> F4 closed-loop recycle (detect Base change = degraded path)

F5 (cards, context-worthy flag) ──feeds──> F7 (Context Pack auto-pick)
F6 (clips)                      ──feeds──> F5 (card link target) & F7 (Pack source)

F1 (frontmatter/type model)     ──required-by──> F7 (OKF Bundle export)
F4 (provenance data)            ──enhances────> F7 (OKF v0.2 provenance direction)

CONFLICT: "Lens editing" ⨯ F2 one-way projection  → Lens editing is anti-feature
CONFLICT: "AI card ghostwrite" ⨯ F5 comprehension goal → ghostwrite is anti-feature
```

### Dependency Notes

- **Block anchoring (F1/§2.4) is the keystone.** F2, F3, and F4 all sit on it. It must be built and hardened *first*; if migration is weak, comments drop (F3 fails), Lens mis-maps (F2 fails), and reflow targets wrong blocks (F4 fails). This is the ordering constraint that dominates the roadmap.
- **F4 requires F3 which requires F1.** The AC-4a closed loop (write comment → reflow → agent edits Base → re-project + needs-review → resolve) chains F1→F3→F4 with F2 re-projection and F1 FS-watch closing the loop. This chain *is* the North-Star hypothesis; sequence it first and end-to-end.
- **F2 re-projection + F1 FS-watch jointly enable F4's degraded path.** Users without MCP still close the loop because FS-watch detects the Base change and re-projection surfaces needs-review (AC-4c). Do not let F4 depend solely on MCP.
- **F5/F6/F7 are far more parallelizable.** Cards, the Chrome extension, and pack assembly touch the Block-anchor substrate only lightly (they attach to docs/projects, not to Blocks). They can proceed once F1's project/data model exists, largely in parallel with F2–F4 hardening.
- **F7 depends on F1 (frontmatter/type), F5 (cards), F6 (clips)** as selectable sources — it is a late/integrative feature by nature.

---

## MVP Definition

The PRD marks F1–F7 all P0, but the Key-Decision milestone strategy sequences the loop first. Reflecting that:

### Launch With — Core Loop (validate the central hypothesis)

- [ ] **F1** import + Block-anchor substrate — foundation for everything
- [ ] **F2** Lens projection + incremental re-projection + fidelity safeguards — the "why we exist" layer
- [ ] **F3** block-level comments + anchor migration (0 silent loss) — the review surface
- [ ] **F4** comment → bilingual bundle → agent, dual-channel + closed-loop recycle — **the North-Star loop (AC-4a)**

Rationale: AC-4a end-to-end in a real Claude Code project is the make-or-break validation. Everything here chains through Block anchoring.

### Add To Complete MVP (P0, after the loop works)

- [ ] **F5** understanding cards (originality nudge, backlinks, inject-to-context) — comprehension-retention pillar
- [ ] **F6** Chrome clipper (clean MD, code fidelity, token estimate) — low-friction acquisition entry
- [ ] **F7** Context Pack assembly + OKF export — replaces bloated CLAUDE.md; no-lock-in trust sell

### Future Consideration (P1+, architecture-reserved)

- [ ] Multi-language Lens (JA/EN) — after zh-Hans projection quality is proven
- [ ] Document graph view — after enough docs/links exist to matter
- [ ] Ingestion conflict detection (new content vs existing decision cards) — CoWiki-inspired, needs card mass first
- [ ] GitHub PR integration, multi-user comments — after single-user PMF
- [ ] Spaced-repetition card review; `export_okf_bundle` MCP tool; auto decision-log extraction

---

## Feature Prioritization Matrix

| Feature | User Value | Impl. Cost | Priority | Roadmap flag |
|---|---|---|---|---|
| F1 import + Block anchor substrate | HIGH | HIGH (anchor) | P1 | Anchor migration = **research spike + adversarial test** |
| F2 Lens + incremental re-projection | HIGH | HIGH | P1 | Projection-quality prompt eng.; model-tier A/B (Q1) |
| F3 comments + migration | HIGH | HIGH | P1 | Shares §2.4 with F1/F2; needs the migration test harness |
| F4 reflow loop (file + MCP) | HIGH | HIGH | P1 | MCP server build; Claude Code hook/skill install UX |
| F5 cards | MEDIUM-HIGH | LOW | P1 | Deliberately simple; standard patterns |
| F6 Chrome clipper | MEDIUM | MEDIUM | P1 | Reuse Defuddle+Turndown+GFM; MV3 native-messaging |
| F7 context pack + OKF export | MEDIUM-HIGH | MEDIUM | P1 | OKF round-trip; token estimator |
| Multi-language Lens | MEDIUM | MEDIUM | P2 | Architecture-reserved |
| Conflict detection | MEDIUM | MEDIUM | P3 | Needs card mass |

(All F1–F7 are P0/"P1 = launch" per PRD; the "Roadmap flag" column is the actionable output for phase sequencing.)

---

## Competitor Feature Analysis

| Capability | Plannotator | markupmarkdown | DeepWiki / Google Code Wiki | Obsidian (+Web Clipper) | PrismDocs approach |
|---|---|---|---|---|---|
| Block/prose comments | selection annotations on plans/diffs | ✅ Google-Docs-style threaded, review states | ❌ (read-only) | ❌ | ✅ F3, on Lens *and* Base |
| Anchor survival across AI rewrite | line-info, plan-scoped | PR-anchored to prose | n/a (regenerates whole) | n/a | ✅ content-hash+heuristic migrate, 0 silent loss (moat) |
| Feedback → coding agent | ✅ structured MD + unified diff, Claude `ExitPlanMode` hook | ✅ agent revisions gated by human | ❌ | ❌ | ✅ F4 dual-channel MCP+file, **+ EN intent summary of native comment** |
| Native-language human layer | ❌ EN only | ❌ EN only | ❌ EN only | ❌ | ✅ **Lens (moat #1)** |
| Incremental re-gen + change-highlight | n/a | per-revision | whole-doc rewrite per commit | n/a | ✅ per-Block incremental + "since last read" bars |
| Fidelity safeguard | n/a | n/a | ❌ | n/a | ✅ forced Base excerpt + distortion report |
| "Write in your own words" cards | ❌ | ❌ | ❌ | backlinks yes, no originality nudge | ✅ F5, no-ghostwrite by design |
| Web clip → clean MD for AI | ❌ | ❌ | ❌ | ✅ (general notes) | ✅ F6, engineering-framed + token cost |
| Context pack + OKF export | ❌ | ❌ | ❌ | ❌ | ✅ F7, OKF no-lock-in |
| Local-first, no server | ✅ local | ❌ (Mongo backend) | ❌ cloud | ✅ vault | ✅ SQLite + files |

**Reading:** every single-capability cell has a strong incumbent, but no competitor holds the *combination*, and none serves the non-English-native reader at all. The unoccupied, hardest-to-copy square is native-language Lens + surviving-anchor comment reflow.

---

## Sources

- Plannotator — [plannotator.ai](https://plannotator.ai/), [docs.plannotator.ai plan-review workflow](https://docs.plannotator.ai/open-source/workflows/plan-review), [GitHub backnotprop/plannotator](https://github.com/backnotprop/plannotator) (feedback = structured Markdown + unified diff; Claude Code `ExitPlanMode` hook; local, no network)
- markupmarkdown 1.0 — [metavert.io/markupmarkdown](https://metavert.io/markupmarkdown), [mumd.metavert.io](https://mumd.metavert.io/) (Google-Docs-style threaded comments, review states, human-gated agent revisions, GitHub round-trip)
- Google Code Wiki — [analyticsvidhya overview](https://www.analyticsvidhya.com/blog/2025/12/google-code-wiki/), [devops.com](https://devops.com/google-code-wiki-aims-to-solve-documentations-oldest-problem/) (Gemini auto-gen, re-checks & rewrites per change, regenerating diagrams, chat over wiki)
- Obsidian Web Clipper stack — [web2md.org 2026 guide](https://web2md.org/blog/best-web-clipper-obsidian-ai-2026), [markdownwebclipper.com compare](https://markdownwebclipper.com/blog/best-markdown-web-clippers-2026) (Defuddle + Turndown + Joplin GFM; fenced code + language tags preserved)
- Google Open Knowledge Format v0.1 — [Google Cloud blog](https://cloud.google.com/blog/products/data-analytics/how-the-open-knowledge-format-can-improve-data-sharing), [spec repo](https://github.com/GoogleCloudPlatform/knowledge-catalog), [MarkTechPost](https://www.marktechpost.com/2026/06/16/google-cloud-introduces-open-knowledge-format-okf-a-vendor-neutral-markdown-spec-for-giving-ai-agents-curated-context/), critique [Marc Bara, Medium](https://medium.com/@marc.bara.iniesta/googles-new-format-for-agent-context-a-standard-or-just-a-folder-82fb21d92041) (`type` sole required field; reserved index.md/log.md; no vocab/link-typing/provenance)
- Google Docs comment anchoring limits — [Drive API manage-comments](https://developers.google.com/workspace/drive/api/guides/manage-comments), [googleworkspace/cli #169 "Original content deleted"](https://github.com/googleworkspace/cli/issues/169) (anchored comments orphan on text deletion; UI ignores API anchors)
- Internal authoritative specs: `docs/PRD_PrismDocs_MVP.md` (F1–F7 REQ+AC), `docs/BRD_PrismDocs_MVP.md` (§3 competitors, §6 mechanisms), `docs/调研补充_CoWiki与OKF对BRD_PRD的影响.md` (OKF + CoWiki)

---
*Feature research for: two-layer AI-doc workbench for non-English-native vibe coders*
*Researched: 2026-07-27*
