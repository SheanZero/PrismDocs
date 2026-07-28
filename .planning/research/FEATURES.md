# Feature Research

**Domain:** AI-era engineering documentation workbench (AI doc review + comment-to-agent loop + bilingual digest + cross-project contracts) for solo vibe coders
**Researched:** 2026-07-28
**Confidence:** MEDIUM (WebSearch cross-checked against official product pages: plannotator.ai docs, metavert.io/markupmarkdown, HackMD blog, Google Developers Blog, kiro.dev, Riftmap/Mabl engineering posts)

**Scope note:** The project already has a mature competitive analysis (BRD v0.3 §3) and a fully-scoped MVP (F1–F5, F7, F8). This file **validates** that categorization against the mid-2026 market rather than re-deriving it. Verdict up front: the categorization holds. The MVP scope covers every hard table stake for this category; three soft gaps were found (Mermaid/GFM rendering completeness, needs-review system notifications, doc Q&A expectation management) — none requires new scope beyond small additions, detailed below.

## Feature Landscape

### Table Stakes (Users Expect These)

What the 2026 market has trained users of adjacent tools to assume. All are covered by the current MVP scope unless flagged ⚠️.

| Feature | Why Expected | Complexity | MVP Coverage | Notes |
|---------|--------------|------------|--------------|-------|
| Rendered markdown review surface (not raw diffs) | HackMD's entire 2026 governance pitch is "open it, scan it, see the rendered version"; markupmarkdown renders in CodeMirror | LOW | ✅ F1/F2 Base view | ⚠️ **Sub-gap: rendering completeness.** Agent-generated docs routinely contain Mermaid diagrams, GFM tables, task lists, code fences. Code Wiki/DeepWiki both ship auto-diagrams, and the target user's own complaint is "英文、太长、**没有图**". PRD never mentions Mermaid rendering. Rendering (not generating) Mermaid + full GFM in the Base view is table stakes for a 2026 markdown tool. LOW cost, add to F1/F2 view requirements. |
| Paragraph/sentence-anchored comment threads | markupmarkdown ("Google-Docs-style comments"), HackMD ("guided comments against a specific sentence"), Google Docs mental model | MEDIUM | ✅ F3 | Market anchors on live editors they own; nobody solves anchoring when an external agent rewrites the file on disk — see Differentiators. |
| Structured feedback back to the coding agent | Plannotator (annotate → one-click send to agent), markupmarkdown (agents via MCP propose edits) made this the baseline in 2026 for this exact user | HIGH | ✅ F4 | Free/open-source competitors exist (Plannotator is free). Merely having "send comments to agent" is no longer differentiating — the closed loop + provenance is (see below). |
| MCP integration + file-protocol fallback | markupmarkdown agents join via MCP with scoped tokens; Plannotator ships as a Claude Code plugin; Claude Code/Cursor users expect MCP | MEDIUM | ✅ F4 REQ-4.2, §4.2 D-07 | Scoped write surface (respond-only) matches markupmarkdown's scoped-token precedent. Correctly designed. |
| Approve / reject / approve-with-notes decision semantics | Plannotator's plan gate (approve / deny-with-annotations / approve-with-notes); Kiro's phase-gated explicit human approval | LOW | ✅ F3 REQ-3.2 | Kiro normalized "explicit human approval before AI proceeds" as the expected workflow shape. |
| File-stays-canonical, no second source of truth | markupmarkdown's headline claim ("the file in your repo stays canonical"); disk authority is a trust requirement for repo-adjacent tools | MEDIUM | ✅ F1 REQ-1.5 磁盘权威 + sidecar 不污染 | PrismDocs goes further (sidecar everything). Correct. |
| Filesystem watch + near-real-time sync | Code Wiki refreshes per commit; agents write continuously; stale views kill trust | MEDIUM | ✅ F1 REQ-1.4 | 10s presentation budget is competitive. |
| "What changed since I last looked" summaries | AI-review-fatigue research: AI-generated change snapshots per revision are the #1 requested antidote to volume; Code Wiki auto-refresh sets the expectation that docs track change | MEDIUM | ✅ F2′ 变更摘要 + REQ-2.5 变更条/已读基线 | The read-baseline design maps directly to what the market is asking for. |
| Version history of documents | markupmarkdown revisions, HackMD versioning; needed for trust in agent-modified files | MEDIUM | ✅ REQ-1.NEW-2 快照 + REQ-4.8 时间线 | |
| Full-text search across the knowledge base | PKM baseline (Obsidian et al.); non-negotiable | MEDIUM | ✅ F5 REQ-5.6, NFR <300ms, FTS5 | Ensure search covers documents + cards workspace-wide, not cards only (PRD §2.1 lists 跨项目搜索 under Workspace — keep it P0). CJK tokenization matters for Chinese comments/cards. |
| Backlinks / `[[` wiki links | Obsidian/Heptabase/Logseq established; "LLM Wiki" framing implies it | MEDIUM | ✅ F5 REQ-5.3 | |
| Local-first, data ownership, no lock-in | Obsidian's core appeal to this exact demographic; OKF gives it a standard | LOW–MEDIUM | ✅ NFR 本地优先 + §2.5 OKF | OKF compatibility is correctly treated as trust/table-stakes, not differentiator — CoWiki already aligned. |
| BYO API key + custom base_url | Standard for indie AI tools in CN market (proxies, local models) | LOW | ✅ §1.3, NFR 密钥 | Graceful no-key degradation (read/comment/cards still work) already specified — important for activation funnel. |
| Notification when the agent has responded / work awaits review | The loop's whole point is "AI finished — come look"; Plannotator auto-opens the browser at the gate; Inbox-only discovery assumes the app is frontmost | LOW | ⚠️ **Partial gap** | PRD has in-app Inbox + badges but no macOS system notification / menu-bar badge for needs-review and drift alerts. For a desktop app whose rhythm is "打开 → 清 Inbox → 关掉", an OS-level nudge when a Bundle gets a response is expected behavior. LOW cost (Tauri notification API), suggest adding to F4/F8 as P0.5. |
| Direct minor-edit escape hatch | Google-Docs paradigm: reviewers fix typos inline (markupmarkdown one-click suggested edits) rather than round-tripping through an agent | LOW | ✅ (weakened by design) REQ-1.6 | In-app Base editing exists but is de-emphasized (Q2). Acceptable; watch内测 for "typo → agent round-trip" friction complaints. Do not cut REQ-1.6. |

### Differentiators (Competitive Advantage)

Validated against the market: each of these is absent from all adjacent products surveyed.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| 中文速读区 (F2′: digest + ❓ decision list + change summary) | No surveyed product has any native-language understanding layer — markupmarkdown/Plannotator/HackMD/Kiro are all single-layer English. Unique to PrismDocs; directly serves the underserved CJK vibe-coder segment | MEDIUM | ❓ list with mandatory source excerpts is the anti-mistranslation guardrail that makes this trustworthy. Wisely descoped from full Lens. |
| Block anchoring with 0 silent loss under external agent rewrites (TD-01 engine) | Competitors anchor inside editors they control (CRDT/live sync). Nobody anchors comments on files an *external* agent rewrites on disk. This is the technical moat — and the market's own framing (review states, paragraph citations) collapses without it | HIGH | One engine, four consumers (F3/F4/F2/F8) — correct architecture; keeps the moat concentrated. Release-gated by AC-3b. |
| Closed loop with three-signal recovery + agent provenance (F4 REQ-4.4/4.7) | Plannotator's loop is session-bound (gate → feedback → gone); markupmarkdown's agents are reviewers, not driven executors. Persistent "comment → change → who/why → resolve" with a fallback when the agent doesn't ack (file-change detection) is unmatched | HIGH | The 48h "AI 似乎没处理" nudge and no-MCP degradation path (AC-4c) handle real-world flakiness competitors ignore. |
| Change timeline + per-Block "why is this like this" (REQ-4.8) | CoWiki has agent audit at team level; no personal tool answers "which comment caused this change". Transparency narrative made concrete | MEDIUM | Honest "来源不明" labeling for external changes protects trust. |
| Cross-project contract subscription + drift alerts + one-click downstream check (F8) | Confirmed whitespace: enterprise tools (Riftmap, Mabl's 850-line coordination graph) are team-priced and code-graph-centric; individuals hand-maintain meta-repos and self-report doc-drift pain; GitHub Copilot cross-repo context is an open unmet feature request. "Personal-level, document-layer, closed-loop cross-repo consistency" has no occupant | MEDIUM (reuses 4 existing engines) | Subscription-only alerting is the right alert-fatigue gate — review-fatigue research confirms notification governance is what users demand. Mabl's 40%→5% drift-failure stat is strong marketing ammunition for this feature. |
| Understanding cards with originality nudge (F5) | Zettelkasten exists in PKM, but no engineering tool operationalizes "write it in your own words" against comprehension debt, wired into the review loop (resolve → card prompt) and back into agent context (F7 injection) | LOW–MEDIUM | The deliberate absence of an AI-write button is the feature. |
| Workspace-scoped Context Pack with live token accounting (F7) | Token-cost transparency addresses a documented anxiety; no competitor shows per-item token budgets for agent context assembly | MEDIUM | Also the MVP delivery channel for cross-project context (no new agent protocol). |
| OKF bundle-of-bundles export with typed cross-links (`x-link-type`) | Answers OKF's two known criticisms (no cross-bundle semantics, untyped links) — reinforces "the layer above OKF" positioning | LOW | Keep P0.5; it is positioning, not adoption-critical. |

### Anti-Features (Commonly Requested, Often Problematic)

The current scope explicitly and correctly avoids these. Validated against market evidence.

| Feature | Why Requested | Why Problematic | Alternative (current scope) |
|---------|---------------|-----------------|------------------------------|
| Auto-generating the wiki from code | "DeepWiki/Code Wiki do it" | Giant-owned, read-only browse mode drives zero loops; competing head-on with Google/Cognition is unwinnable | Manage/understand what agents already write; let Claude Code generate |
| Editable understanding layer (digest/Lens) | "Let me fix the translation" | Creates a second source of truth → bilingual drift, the exact disease the product cures | 只读投影; opinions go through comments (REQ-2.7) — a product principle, keep it |
| AI auto-review of every revision | markupmarkdown ships it ("agent reviews each revision within a minute") | For a solo user this manufactures alert fatigue — review-fatigue research shows template-blindness and reflexive approvals follow volume | ❓ 宁缺毋滥, subscription-gated alerts, human-triggered loops |
| Semantic drift detection (LLM compares upstream/downstream claims) in MVP | Obvious "smart" extension of F8 | Precision unproven; false positives would poison the <10% alert error budget on day one | Structural signals only (subscribed-Block hits); semantic P1 after calibration |
| Chat-with-your-docs (RAG Q&A panel) | DeepWiki/Code Wiki made it the flagship AI-docs interaction | Duplicates the coding agent the user already pays for and has open; splits the Q&A surface; adds RAG infra to a local-first app | 💬 提问 comment routed to the agent via F4; user's Claude Code answers with full repo context. ⚠️ Expectation-management gap, not scope gap: first-run/onboarding should visibly frame "问问题 → 提问评论" so DeepWiki-conditioned users don't perceive absence. |
| Real-time multi-user collaboration | HackMD/markupmarkdown have it; teams will ask | Sync service, identity, permissions — an entire second product; P1 git-portable mode is the cheap path | MVP single-user; Q9 git 便携模式 for teams later |
| In-app IDE / code editing / built-in agent | "One tool for everything" | Positions against Cursor/Kiro head-on; agent vendors iterate faster | Knowledge layer beside the agent, MCP + file protocol only |
| General-purpose note-taking | Obsidian users will map cards → notes | Feature treadmill vs. Obsidian's decade of plugins; dilutes engineering context | Cards stay engineering-scoped, doc/clip-linked |
| Published docs site / sharing | Mintlify comparison inevitable | Different buyer, hosting burden, SEO arms race | OKF export lets any publisher consume the bundle |
| AI-written cards | "Save me typing" | Defeats the comprehension-retention mechanism (the 17% comprehension-loss research is the product's own argument) | Placeholder nudge + post-hoc AI 复述质检 (P0.5) only |
| Frontmatter/IDs written into user files by default | Agents & some users like self-describing files | Violates 不污染 trust contract; git noise in user repos | Sidecar default + Q7 opt-in switch, round-trip byte-exact |
| Cloud sync / server-side anything (MVP) | Cross-device is a real want | Kills local-first trust story, adds compliance surface before PMF | Single-directory backup + OKF export; revisit post-PMF |

## Feature Dependencies

```
锚定引擎 (TD-01, comrak truth source)
    ├──required-by──> F3 评论锚点迁移
    ├──required-by──> F2′ 变更条 / 变更摘要 / 已读基线
    ├──required-by──> F4 命中判定（三信号回收兜底）
    └──required-by──> F8 订阅 Block 命中

F1 导入/watch ──required-by──> 锚定引擎(变更事件源), F2′, F8(多项目前提)
F3 评论 ──required-by──> F4 Feedback Bundle
F4 闭环(Bundle+双通道+回收+溯源) ──required-by──> F8 一键下游核对(复用), REQ-4.8 时间线
F5 卡片 ──required-by──> F7 注入(context-worthy)
F7 Context Pack ──carries──> F8 跨项目上下文 (MVP delivery channel, no new MCP tools)
F2′ 速读区 ──enhances──> F3 (❓清单 → 跳转 → 评论 is the primary comment entry)

[System notifications] ──enhances──> F4/F8 (needs-review & drift alert delivery)
[Mermaid/GFM rendering] ──required-by──> Base view credibility (F1/F2)
```

### Dependency Notes

- **Anchor engine before F3/F2′/F8:** all four consumers read its output; interface freeze (TD-01 §7) before Phase 4+ is correctly sequenced in the planned roadmap.
- **F8 rides on F4:** the "zero new agent protocol" claim only holds if F4's Bundle format + three-signal recovery ship first. Do not parallelize F8 ahead of F4 completion.
- **F2′ is the funnel head:** activation metric (first loop in 7 days) depends on 速读区 → ❓ → comment being frictionless; treat F2′ quality as an activation dependency, not just a feature.
- **Conflict:** chat-with-docs (anti-feature) conflicts with F4 提问 semantics — adding both would split question routing. Keep one surface.

## MVP Definition

### Launch With (v1) — current scope confirmed

- [x] F1 import/sync — table stakes, everything depends on it
- [x] 锚定引擎 (TD-01) — the moat; release-gated by AC-3b
- [x] F2′ 速读区 — the unique differentiator for the target segment; funnel head
- [x] F3 block comments — table stakes for the category
- [x] F4 comment-to-agent loop — the north-star mechanism; parity-plus vs. Plannotator/markupmarkdown
- [x] F5 cards — differentiator, low cost
- [x] F7 Workspace context pack — token transparency + F8 carrier
- [x] F8-lite contract subscription + drift alerts — second moat, confirmed whitespace
- [ ] **ADD (small): Mermaid + full GFM rendering in Base view** — table-stakes rendering completeness, LOW
- [ ] **ADD (small, P0.5): macOS system notifications for needs-review / drift alerts** — loop-closure delivery, LOW
- [ ] **ADD (onboarding note): frame 提问评论 as the "ask about docs" surface** — expectation management vs. DeepWiki-style chat, LOW

### Add After Validation (v1.x) — current P1 list confirmed

- [ ] 全文 Lens — trigger conditions already written (BRD §6.1); keep data-driven
- [ ] F6 clipper — correctly deferred; Obsidian Web Clipper + AI interpreters have raised the bar here, so shipping a mediocre clipper in MVP would compare badly. Deferral avoids that.
- [ ] Semantic drift detection — after structural-signal precision is proven <10% error
- [ ] Doc Q&A panel — only if 内测 shows 提问-comment friction is a real activation blocker
- [ ] Git-portable team mode (Q9), multi-language digests, graph view, GitHub PR integration

### Future Consideration (v2+)

- [ ] Team version with shared timeline/identity — different buyer, needs PMF first
- [ ] Cloud sync / hosted anything — post-PMF, legal review first
- [ ] Windows / MAS distribution — per existing platform decisions

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| 锚定引擎 + F3 | HIGH (trust foundation) | HIGH | P1 |
| F4 closed loop | HIGH (north star) | HIGH | P1 |
| F2′ 速读区 | HIGH (segment differentiator, funnel head) | MEDIUM | P1 |
| F1 import/sync | HIGH (table stakes) | MEDIUM | P1 |
| Mermaid/GFM rendering completeness | MEDIUM (credibility) | LOW | P1 |
| F8-lite | HIGH (second moat) | MEDIUM (reuse) | P1 |
| F7 context pack | MEDIUM–HIGH | MEDIUM | P1 |
| F5 cards | MEDIUM (retention play) | LOW–MEDIUM | P1 |
| System notifications (needs-review/drift) | MEDIUM | LOW | P2 (P0.5) |
| OKF export | MEDIUM (positioning) | LOW | P2 (P0.5) |
| 全文 Lens / F6 / semantic drift / Q&A panel | conditional | MEDIUM–HIGH | P3 (triggered) |

## Competitor Feature Analysis

| Feature | markupmarkdown | Plannotator | HackMD | Kiro / Spec Kit | DeepWiki / Code Wiki | PrismDocs approach |
|---------|----------------|-------------|--------|-----------------|----------------------|--------------------|
| Comment anchoring | Google-Docs-style on live editor it owns | Annotation UI on submitted plan/diff (session) | Paragraph citations, guided comments | GitHub-flow comments on spec files | none | Block anchor on **externally rewritten disk files**, 0 silent loss (moat) |
| Feedback to agent | MCP agents as reviewers w/ scoped tokens | One-click structured feedback, Claude Code plugin | none (human review only) | Phase-gated approve/continue in IDE | none | Bundle 双通道 + three-signal recovery + provenance + timeline |
| Understanding layer | English only | English only | English only | English only | English wiki + diagrams + chat | 中文速读区 + ❓决策清单 (unique) |
| Cross-repo consistency | none (per-repo index) | none | none | none | none (per-repo) | Contract subscription + drift alert + one-click downstream check (unique at personal level) |
| Knowledge retention | none | none | none | steering files (AI-consumed) | none | Hand-written cards + originality nudge + context injection |
| Review gating | Review states gate pushes | Plan gate blocks agent | Human gate before merge to truth | Explicit approval per phase | none | Read-baseline + needs-review Inbox (no push gating — agent isn't controlled by us; correct for disk-authoritative model) |
| Data ownership | Files stay canonical in repo | Local, open source | Cloud | Local files in repo | Cloud (self-host option) | Local-first + sidecar + OKF export |

## Sources

Confidence tier: WebSearch cross-verified = MEDIUM (per classify-confidence seam). Existing internal analysis (BRD/PRD/调研 v2) treated as primary input per task instructions.

- [Plannotator](https://plannotator.ai/) · [GitHub](https://github.com/backnotprop/plannotator) · [Markdown Annotation docs](https://mintlify.wiki/backnotprop/plannotator/features/markdown-annotation)
- [markupmarkdown 1.0](https://metavert.io/markupmarkdown) · [product site](https://mumd.metavert.io/)
- [HackMD: AI writes your docs governance layer](https://homepage.hackmd.io/blog/2026/04/22/AI-writes-your-docs-hackmd) · [Stoffel Labs case](https://homepage.hackmd.io/blog/2026/05/07/hackmd-governance-layer-ai)
- [Google Code Wiki announcement](https://developers.googleblog.com/introducing-code-wiki-accelerating-your-code-understanding/) · [Code Wiki overview](https://www.analyticsvidhya.com/blog/2025/12/google-code-wiki/) · [DeepWiki overview](https://ghost.codersera.com/blog/what-is-deepwiki-ai-code-documentation-github-repository/)
- [Kiro spec-driven guide](https://pingax.com/kiro-spec-driven-development/) · [Kiro intro](https://kiro.dev/blog/introducing-kiro/) · [SDD tools comparison](https://medium.com/@visrow/comprehensive-guide-to-spec-driven-development-kiro-github-spec-kit-and-bmad-method-5d28ff61b9b1)
- [Riftmap: cross-repo context](https://riftmap.dev/blog/ai-coding-agents-need-cross-repo-context/) · [Mabl 75+ repos system](https://www.mabl.com/blog/how-we-built-a-system-for-ai-agents-to-ship-real-code-across-75-repos) · [Agent meta-repo pattern](https://seylox.github.io/2026/03/05/blog-agents-meta-repo-pattern.html) · [Copilot cross-repo feature request](https://github.com/orgs/community/discussions/189213)
- [Obsidian Web Clipper + AI](https://web2md.org/blog/best-web-clipper-obsidian-ai-2026) · [claude-obsidian pattern](https://github.com/AgriciDaniel/claude-obsidian)
- [AI review overload](https://www.codeant.ai/blogs/prevent-ai-code-review-overload) · [Code review bottleneck in AI era](https://asyncsquadlabs.com/blog/code-review-bottleneck-ai-era/)
- Internal: `docs/BRD_PrismDocs_MVP.md` v0.3, `docs/PRD_PrismDocs_MVP.md` v0.3, `docs/调研_整体构想v2_多项目知识层.md` v0.1

---
*Feature research for: AI-era engineering documentation workbench (PrismDocs)*
*Researched: 2026-07-28*
