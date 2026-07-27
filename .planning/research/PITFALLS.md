# Pitfalls Research

**Domain:** Local-first two-layer (Base/Lens) Markdown docs workbench + Chrome MV3 clipper + local MCP server, with block-anchored comments that round-trip to coding agents
**Researched:** 2026-07-27
**Confidence:** HIGH (technical fundamentals verified against Chrome/Hypothesis/OKF sources; product-specific failure modes derived from BRD/PRD; MEDIUM where noted)

> Priority ordering matches BRD §11 and PRD Release Criteria: **anchor drift / silent comment loss is pitfall #1** (AC-3b: 0 silent loss is a hard release gate). Everything else follows.

---

## Critical Pitfalls

### Pitfall 1: Silent comment loss when the AI rewrites Base heavily (THE #1 RISK)

**What goes wrong:**
A user writes comments on Lens/Base blocks. The coding agent then rewrites the Base file substantially (AC-3b scenario: 50%+ rewrite). The block-anchor migration algorithm re-parses the new AST, fails to confidently match some old blocks to new blocks, and the associated comments quietly vanish — no thread, no downgrade marker, no snapshot. The user loses trust permanently the first time this happens, because a lost review comment is invisible by definition (you cannot notice the absence of something you can no longer see).

Concrete migration-algorithm failure modes:
- **Content-hash brittleness:** anchor = content hash + position heuristic. A one-word edit changes the hash → 0% hash match → falls entirely to positional heuristic → wrong block or no block.
- **Split/merge blindness:** agent splits one paragraph into three (or merges three into one). Naive 1:1 matching has no correct answer, so it either drops the comment or attaches it to an arbitrary fragment.
- **Reorder + rewrite compound:** blocks move AND change text simultaneously. Position heuristic and content similarity point at different blocks; tie-breaking picks wrong.
- **Threshold cliff:** a single confidence threshold means everything just under it is silently discarded instead of downgraded. The "0 silent loss" guarantee requires that *below threshold => downgrade to doc-level with quote snapshot*, never *below threshold => drop*.
- **Whole-file replacement:** agent rewrites the file from scratch (common with "rewrite this doc" prompts). Every block is "new"; without a fallback the entire comment set is orphaned.
- **Non-deterministic re-parse:** Markdown AST parser produces different block boundaries after a trivial formatting change (e.g. list tightening, trailing whitespace), so blocks that look identical to the user fail to match.

**Why it happens:**
Teams treat anchoring as a "match by hash, else match by position" two-step and ship it. They test on small edits (rename a heading) and never adversarially test large rewrites. The failure is asymmetric: false-drop is invisible in demos but catastrophic in trust.

**How to avoid:**
- **Design for downgrade, never drop.** Below the confidence threshold the comment MUST become a document-level comment carrying: the original quote snapshot, the old heading path, and an "original location changed — please confirm" flag (PRD §2.4, REQ-3 edge cases mandate exactly this). Make "drop" impossible in code — there is no code path that deletes an orphaned comment.
- **Multi-strategy anchoring, not single hash.** Adopt the Hypothesis model: store a TextQuoteSelector (the quoted text + ~32 chars of prefix/suffix context) AND a TextPositionSelector (char offsets) AND the structural path (heading path + block index). Migration tries structural match → fuzzy quote match (diff-match-patch / Levenshtein over the quote+context) → positional fallback, and records which strategy won plus a confidence score.
- **Handle split/merge explicitly.** When one old block maps to N new candidates above threshold, attach to all and mark "block was split"; when N old map to one, keep all comments and mark "blocks merged." Never force 1:1.
- **Always keep an immutable quote snapshot at comment-creation time.** Even total re-parse failure then degrades to "here is what you commented on, and here is roughly where it went," not silence.
- **Adversarial golden-corpus test suite as a release gate.** Curate real before/after doc pairs (small edit, heading rename, paragraph split, section reorder, 50% rewrite, full rewrite, formatting-only churn) with hand-labeled expected outcomes. AC-3b (≥90% correctly migrated-or-downgraded, 0 silent loss) is measured against this corpus in CI, not vibes.
- **Provenance link (REQ-4.7).** Tie each Base change to its triggering Feedback Bundle so migration can prefer the block the agent claims it edited.

**Warning signs:**
- Comment count before an agent edit ≠ (migrated + downgraded) count after. This invariant check is the single most important early-detection mechanism — assert it after every re-parse.
- Anchoring code has any branch that returns without either re-anchoring or downgrading.
- Test suite only covers small edits.
- Confidence score not persisted (you can't audit misfires post-hoc).

**Phase to address:**
Foundational **Block Anchoring** phase, which MUST land before F3 (comments) and be re-verified in F4 (agent round-trip). This is the deepest-research phase in the whole project — flag it for a dedicated technical design + spike before implementation.

---

### Pitfall 2: Two-layer sync corruption — projecting a stale Base / race during projection

**What goes wrong:**
Base is the sole source of truth; Lens is a derived projection. Corruption modes:
- **Stale Lens shown as current:** Base changed on disk, incremental re-projection ran on an *older* in-memory Base snapshot (or a race between the FS-watcher event and the projection scheduler), so the user reviews and approves a Lens that no longer reflects the file. They make a decision on fiction.
- **Race when Base changes mid-projection:** projection of block N is in flight (streaming from the LLM) when the same block changes again on disk. Without cancellation the stale result overwrites the cache and the Lens silently lags one revision behind (PRD REQ-2.9 / edge case: "Base changed during projection → cancel in-flight, reschedule on newest").
- **Partial-write projection:** agent is mid-write (file momentarily truncated/half-flushed); watcher fires; projector reads a syntactically broken Markdown, produces garbage Lens, caches it.
- **Cache-key desync:** incremental re-projection keys the Lens cache on block ID but not on Base content version, so after an edit-then-revert the stale cached Lens is reused for changed content.
- **Someone treats Lens as editable / round-trips through it:** any code path (or future feature) that writes Lens back toward Base reintroduces the exact double-layer drift the single-direction-projection principle exists to prevent (PROJECT.md Out of Scope: "Lens 层直接编辑").

**Why it happens:**
Async LLM calls + FS events + a cache is a three-way concurrency problem. Developers assume file writes are atomic and events are ordered; neither holds. The "project only affected blocks" optimization multiplies the state to keep consistent.

**How to avoid:**
- **Version every Base block.** Cache key = (blockId, baseContentVersion). A Lens fragment is only "fresh" if its version matches the current Base block version; otherwise it renders with a "regenerating" state, never as authoritative.
- **Single-writer projection scheduler with cancellation.** All projection work goes through one queue keyed by blockId. A newer Base version for a block cancels any in-flight task for it and supersedes queued ones (last-write-wins on *input* version, not on completion time). Confirms PRD REQ-2.9 edge case.
- **Debounce + settle before reading.** Wait for the FS-watcher debounce window (REQ-1.4: 2s) AND verify the file parses as valid Markdown before projecting. Never project a file that failed to parse — retry after settle.
- **Never present a stale Lens as current.** If Base version > Lens version for a block, show an explicit "out of date / regenerating" affordance. Staleness must be visible, never silent.
- **Enforce Lens read-only at the type/storage layer,** not just the UI. Lens has no write-back API surface at all.
- **Persist the projection cache** (REQ-5 reliability: restart must not recompute) and include the Base version in the persisted record so a restart can detect and re-project stale entries.

**Warning signs:**
- Users report "the Chinese version doesn't match the English" (this is the AC-2d fidelity-report channel AND a desync alarm — separate the two causes).
- Projection completes for a block whose Base version already advanced (log it; should be near-zero if cancellation works).
- Lens cache hit on a block whose content changed.

**Phase to address:**
**F2 (Lens generation)** — the scheduler, versioning, and cancellation are core to F2, not add-ons. Design the projection scheduler as a first-class component with the FS-watcher (Pitfall 3) as its input contract.

---

### Pitfall 3: FS watcher hazards — missed events, atomic-save churn, agent burst writes

**What goes wrong:**
The file watcher is the input to the entire two-layer system; its failure modes poison everything downstream.
- **Editor atomic-save rename churn:** most editors (Vim, VS Code with certain settings, many agents) save by writing a temp file then renaming over the target. Naive watchers see this as delete+create (or fire on the temp file), lose the identity of the document, and either drop comments or create a duplicate document. This is *the* classic chokidar/fsevents footgun.
- **Missed events under burst:** AC-1b requires 5 docs modified in one agent session to all appear within 10s with no duplicate change records. Aggressive debounce merges/drops the tail of a burst; too-short debounce fires mid-write on half-flushed files (see Pitfall 2). Native fsevents on macOS coalesces and can drop events under heavy load.
- **Debounce-vs-correctness tension:** a per-file debounce that resets on every write means an agent streaming a large file keeps resetting and the doc never updates until the agent pauses.
- **External delete/rename mishandled:** file deleted while open must archive the doc and preserve comments/cards (REQ-1 edge case), not crash or orphan. Rename/move must be recognized as the same doc via content hash so anchors migrate (AC-1c: 100% comment retention across rename).
- **Non-UTF-8 / >1MB files:** must be skipped and listed (REQ-1 edge case), not fed to the parser/LLM where they corrupt state or blow cost.
- **Watcher on `.prismdocs/` feedback loop:** the app writes Feedback Bundles and Context Packs into `.prismdocs/`; if the watcher watches its own output directory it can trigger self-reprocessing loops.
- **Symlinks / case-insensitive HFS+ / iCloud/Dropbox-synced folders:** duplicate or phantom events, path-case mismatches.

**Why it happens:**
Watchers abstract away OS-specific realities that leak badly (fsevents coalescing, inotify limits, atomic-save patterns). Developers test by editing files by hand in one editor and never simulate an agent hammering the disk.

**How to avoid:**
- **Use a battle-tested watcher (chokidar) with `awaitWriteFinish`** (size-stable polling) so a file is only processed once it stops growing — directly defuses partial-write projection and atomic-save churn.
- **Identity by content hash + inode, not path.** Rename/move/atomic-save all resolve to "same document" via hash so anchors follow (AC-1c). Path is a label, not identity.
- **Trailing-edge debounce with a max-wait cap.** Coalesce a burst but force a flush after a ceiling so a long stream still updates (AC-1b's 10s). Deduplicate change records per (doc, contentVersion) so bursts don't create duplicate history entries.
- **Filter at ingest:** enforce the glob include/exclude (REQ-1.2), skip non-UTF-8 and >1MB up front, and **exclude `.prismdocs/` from the watcher** to prevent self-triggering.
- **Explicit delete/rename state machine:** delete → archived (retain comments/cards with "source deleted"); rename → rehash → same doc.
- **Disk is always authoritative** (REQ-1.5): the watcher never fights the file; on conflict, reload from disk.

**Warning signs:**
- Duplicate documents appearing after a save (atomic-save not handled).
- Comment loss specifically after saving in a particular editor.
- "N change records for one logical edit" in history (dedup broken).
- CPU spin or event storms when a synced folder (iCloud/Dropbox) is chosen as project root.
- Change latency > 10s under agent burst (AC-1b failing).

**Phase to address:**
**F1 (import + FS watcher)**. The watcher's output contract (debounced, settled, identity-resolved change events) is a hard dependency for F2 and F3, so nail it first with a burst-write test harness.

---

### Pitfall 4: LLM projection cost blowup and fidelity distortion causing wrong decisions

**What goes wrong:**
- **Cost blowup:** incremental re-projection is claimed but not actually incremental — a one-block edit re-projects the whole document (or the "adjacent affected blocks" heuristic over-expands to the whole doc). At 500 docs, autoprojection on every agent write burns the user's API budget silently. Users on their own key see a surprise bill and churn.
- **Fidelity distortion (the dangerous one):** the Lens is a *lossy口语化 rewrite*, not a translation. The LLM omits a caveat, softens a risk, or hallucinates a rationale. The user makes a wrong go/no-go decision on a distorted Lens. BRD §11 lists this as a "中" risk; PRD sets a <5% distortion guardrail (AC-2d) as a north-star protection metric.
- **Streaming/retry data corruption:** projection streams per block (REQ-2.8); a mid-stream failure + retry double-writes or leaves a half-rendered Lens cached; a retry of a non-idempotent write corrupts the cache.
- **Model-choice trap:** cheap model everywhere → Lens quality too low, users read Base anyway (product value gone); strong model everywhere → cost blowup. Q1 open question (fast model floor + strong model only on "needs decision" blocks) is unresolved and load-bearing.
- **Prompt-injection via document content:** Base docs and clips contain arbitrary text (including "ignore previous instructions"); a naive projection prompt lets document content hijack the projection or the English-intent-summary generation in F4.

**Why it happens:**
"Incremental" is easy to claim and hard to verify; the affected-block set is fuzzy. Fidelity is invisible unless explicitly measured — a fluent wrong Lens looks great. Retries around streaming are underspecified.

**How to avoid:**
- **Make incrementality measurable and gated.** AC-2b requires logs proving only affected blocks were re-projected. Add a metric: blocks-reprojected / blocks-changed; alarm if >> 1. Cache by (blockId, baseVersion) so unchanged blocks are never recomputed.
- **Forced Base excerpt on decision blocks (REQ-2.6, AC-2c).** Every "needs decision"/risk-flagged Lens block MUST show the Base original alongside — no exceptions. This is the primary defense against distortion-driven wrong decisions: the user can always check the source on exactly the blocks that matter.
- **One-click "report distortion" on every Lens block (REQ-2.6)** feeds the AC-2d <5% guardrail and prompt iteration. Treat this rate as a release/ongoing health metric.
- **Idempotent, cancelable projection writes.** A block projection either fully commits or not at all (write to temp, atomic swap into cache). Retries are safe; in-flight tasks cancel cleanly (ties to Pitfall 2).
- **Two-tier model routing (resolve Q1 early via M0 A/B):** fast model as the floor, escalate only decision/risk blocks to a stronger model. Show estimated token cost before projecting (REQ-2.9); auto-project only ≤5k-token docs, prompt above that.
- **Treat document/clip content as untrusted data, not instructions.** Delimit document content clearly in the projection/summary prompt; never interpolate it into the instruction section. This matters most in F4's "English intent summary" and F7's context assembly.

**Warning signs:**
- Token spend per edit not proportional to edit size.
- Distortion-report rate climbing toward/over 5% (AC-2d).
- Users toggling to "Base only" view frequently (Lens not trusted → product core failing).
- Retry logs showing duplicate cache writes for one block.

**Phase to address:**
**F2 (Lens)** for incrementality, fidelity guardrails, and model routing. Prompt-injection hardening spans **F2, F4, F7** (anywhere document/clip text enters a prompt). Flag F2 for AI-integration spec (projection prompt is an eval-worthy artifact).

---

### Pitfall 5: MCP server + Chrome MV3 lifecycle and bridge failures

**What goes wrong:**
- **MV3 service worker kills native messaging (VERIFIED, and it has bitten Claude Code itself):** the MV3 service worker terminates after ~30s idle. Opening a native-messaging port **no longer resets the idle timer** (behavior changed from earlier MV3); only *messages* count as activity. So the clipper's native-messaging bridge to the desktop app dies ~30s after the last message, and the next clip silently fails or hangs. This exact class of bug is documented in Chrome's own tracker and in a Claude Code native-host issue.
- **Desktop app not running when user clips:** native messaging requires the desktop host process; if it's not running the clip must not be lost. REQ-6.5 mandates a local queue that back-fills on reconnect — easy to forget, and its absence means silent data loss.
- **Loopback security:** a local MCP server / local HTTP port is reachable by *any* process on the machine, including other browser tabs (DNS-rebinding / CSRF against localhost). If the port exposes workspace document content with no auth, any webpage the user visits could exfiltrate it.
- **MCP write-surface overreach:** MCP is supposed to expose only comment回执 (agent can't create/delete comments or cards — PRD §4.2). A too-broad tool surface lets a compromised/confused agent mutate the knowledge base.
- **Store-review constraints (MV3):** native messaging + broad host permissions + remote-code concerns draw Chrome Web Store review scrutiny and rejection; `<all_urls>` host permissions and unclear data use trigger review delays. The clipper is also the low-friction acquisition channel (BRD), so a store rejection is a go-to-market blocker, not just a bug.

**Why it happens:**
MV3's ephemeral service worker is a fundamental break from MV2 background pages; native messaging keep-alive semantics are subtle and have changed across Chrome versions. Localhost is falsely assumed to be a trust boundary.

**How to avoid:**
- **Do not rely on a persistent native-messaging port.** Either (a) send periodic keep-alive messages (<30s) while a bridge is genuinely needed and re-establish the port on demand, or (b) prefer a **local HTTP loopback endpoint on the desktop app** that the extension calls per-clip (stateless, survives SW termination), rather than a long-lived native-messaging channel. Design the clip flow as fire-and-retry, not persistent connection.
- **Offline queue is mandatory (REQ-6.5):** extension persists clips locally (IndexedDB) and back-fills when the desktop app/endpoint is reachable. Test the "desktop app closed" path explicitly.
- **Lock down the loopback:** bind to 127.0.0.1 only; require a per-install token/secret (shared out-of-band via native messaging handshake or a file in the workspace) on every request; validate `Origin`/no-Origin and reject browser-tab requests; short-lived tokens. Never expose document content without auth.
- **Minimal MCP tool surface (PRD §4.2):** read tools + comment回执 only; no create/delete of comments/cards; scope strictly to current Workspace; loopback only.
- **Store-review hygiene:** request the narrowest host permissions (activeTab / explicit per-site rather than `<all_urls>` where possible), a clear privacy disclosure ("content sent only to the user's local desktop app / configured LLM endpoint; nothing to our servers"), no remotely-hosted code. Budget review iterations into the M3 timeline.

**Warning signs:**
- Clips work for the first ~30s after launching the browser, then fail (SW-idle native-host death — the verified signature).
- Clips lost when desktop app was closed (no offline queue).
- Any process/tab on localhost can read workspace data (no loopback auth).
- Store review flags permissions/data-use.

**Phase to address:**
**F6 (Chrome clipper)** for the bridge/lifecycle/offline-queue and store hygiene; **F4 (MCP server)** for loopback auth and minimal tool surface. Flag F6 for a dedicated MV3-lifecycle spike (the native-messaging-vs-loopback decision is architectural). Security-review both before ship.

---

### Pitfall 6: Polluting user files / round-trip breakage + OKF type-vocabulary sprawl

**What goes wrong:**
- **Silent frontmatter injection:** the "don't pollute user files" principle (PROJECT.md constraint, PRD §2.5) is violated the moment any code path writes Block IDs, metadata, or frontmatter into the user's source `.md`. Users' git diffs light up with PrismDocs noise; agents and IDEs see churn; trust (the "no lock-in, keep your files clean" selling point) evaporates.
- **Round-trip corruption of existing frontmatter:** files that already have YAML frontmatter must parse in and re-serialize *byte-stable* where untouched (REQ-1.8, PRD §2.5 "round-trip 不破坏"). Naive YAML libraries reorder keys, restyle quotes, drop comments, normalize dates, or change list indentation — so merely opening a file and (later) exporting mangles the user's frontmatter.
- **OKF type-vocabulary sprawl (the documented OKF criticism):** OKF's own weakness is that `type` has no registered vocabulary — one project writes `Architecture`, another `arch`, another `architecture` (Marc Bará's critique: "a standard, or just a folder?"). If PrismDocs lets users add types freely, the controlled 9-type vocabulary (REQ-1.8) rots into the same unregistered sprawl, breaking filtering/search and undermining the OKF-compatibility selling point. Q6 flags this as unresolved.
- **Lossy HTML→Markdown or OKF export:** REQ-1.3 (HTML import) and REQ-7.6 (OKF Bundle export) can silently drop content; an export that isn't actually OKF-consumable defeats the no-lock-in promise.

**Why it happens:**
"Just write the metadata into the file" is the easy implementation; sidecar storage is more work. YAML round-trip fidelity is a known-hard problem most devs underestimate. Controlled vocabularies feel bureaucratic so teams make them open "for flexibility" and get entropy.

**How to avoid:**
- **Sidecar by default, opt-in only for source writes (REQ-3.7, §2.5, Q7).** Block IDs, comments, cards, and PrismDocs metadata live in the local SQLite/sidecar store. The source `.md` is only written when the user explicitly edits Base in-app (REQ-1.6) or explicitly enables a "write frontmatter to source" project mode (Q7, default off). Materialize OKF frontmatter *only* on explicit export (REQ-7.6).
- **Use a round-trip-preserving YAML approach.** Parse frontmatter into a model that preserves key order, styles, and unknown fields; on any write, re-emit only changed keys and leave the rest byte-identical. Add a round-trip test: parse→serialize of a corpus of real frontmatter files must be a no-op diff.
- **Controlled vocabulary with a registration gate (REQ-1.8, Q6):** ship the 9 built-in types; unknown types go to an "unregistered" bucket with a prompt to register in settings (not silently accepted, not silently rejected). Registration is per-workspace and visible, so drift is at least surfaced and curatable.
- **Round-trip and export as tested guarantees:** golden-file tests for HTML→MD (code-block fidelity, AC-6a is the clipper analog) and for OKF export (the exported bundle must be re-ingestable / parseable by an OKF consumer — validate against the OKF spec, not just "we wrote some frontmatter").

**Warning signs:**
- `git status` in a user's repo shows changes after merely opening/projecting a doc in PrismDocs (pollution).
- Diff of a frontmatter file after round-trip is non-empty for untouched keys.
- "unregistered" type count climbing; filters/search returning inconsistent buckets.
- OKF export that no external OKF tool can actually read.

**Phase to address:**
**F1** for frontmatter parse + round-trip fidelity + sidecar discipline + controlled vocabulary; **F7** for OKF Bundle export validation; **F6** for HTML→MD fidelity. Resolve Q6 and Q7 in F1 design review (they're architecture-level).

---

### Pitfall 7: Product-adoption traps — workflow inertia and alert fatigue

**What goes wrong:**
- **Asking users to change their agent workflow (inertia, BRD §11 "中" risk):** the whole value prop dies if the user must change how they use Claude Code/Cursor. If setup requires manual MCP config, editing CLAUDE.md by hand, or remembering to "export feedback then paste a command," the friction exceeds the payoff and they stay in the IDE chat. BRD's own mitigation: "监听文件夹即用，不要求改变 agent 使用习惯."
- **Alert fatigue (BRD §4.2 design constraint):** the product's reason to exist is *reducing* review load. If it pings on every FS change, every re-projection, every agent edit, it recreates the exact "review fatigue" (痛点 #1) it's meant to cure. Too-frequent AI feedback → users mute notifications → Inbox becomes noise → churn. The BRD explicitly constrains AI feedback to "少而准."
- **Degraded path not actually usable:** AC-4c requires users *without* MCP to still close the loop via the file protocol. If the file-only path is an afterthought, the majority who don't install MCP can't get value.
- **The core loop is too slow to be a habit:** activation target is first closed loop within 7 days for ≥40%. If the first-run "point at folder → see Chinese Lens → comment → send → AI fixes → review" path has any dead end, activation collapses.

**Why it happens:**
Engineers optimize the happy path (MCP installed, Claude Code first-class) and treat the file-protocol degrade path and notification tuning as polish. Notification frequency is added reactively ("let's tell the user when X happens") without a fatigue budget.

**How to avoid:**
- **Zero-config first value:** point at a folder → Lens appears. MCP/hook install is a *one-click generated config snippet* (REQ-4.2), never manual. The 10-line CLAUDE.md/AGENTS.md protocol block is generated and appended for the user (PRD §4.1), with `.prismdocs/` auto-suggested for `.gitignore`.
- **Inbox is the single rhythm, batched not streamed (PRD §2.3).** Aggregate into "打开 → 清 Inbox → 关掉," not a stream of toasts. Batch AI feedback; one digest per meaningful change set, not per file event. Give the user a frequency budget and default it conservative.
- **File-protocol degrade path is a first-class, tested flow (AC-4c),** verified end-to-end without MCP: agent detects `.prismdocs/feedback/*.md`, edits Base, PrismDocs detects the Base change and flips comments to needs-review (REQ-4.4 fallback via FS detection — reuses Pitfall 3's watcher).
- **Instrument the activation funnel** (`first_loop_closed`, `bundle_no_response_48h`) so the first-loop drop-off is visible and fixable.

**Warning signs:**
- Setup instructions contain manual editing steps.
- Notification/Inbox item volume per day trending up; users disabling notifications.
- `bundle_no_response_48h` high (agents not picking up feedback → the bridge/protocol isn't frictionless).
- First-loop activation < 40% in internal test.

**Phase to address:**
**F4** for one-click config, batched回流, and the file-protocol degrade path. Notification-frequency budget spans **F2/F3/F4** (anything that can generate an Inbox item). Validate in M2 internal test against the activation metric.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Single content-hash anchor, no fuzzy/positional fallback | Ships F3 fast | Silent comment loss on any edit; violates AC-3b; unrecoverable trust loss | **Never** — anchoring must be multi-strategy from day one |
| Re-project whole document on any change | Simpler scheduler | Cost blowup, fails AC-2b, kills unit economics on user keys | Never for autoproject; acceptable only for an explicit manual "reproject whole doc" action |
| Persistent native-messaging port assumed alive | Simple clipper bridge | Clips silently fail ~30s after browser launch (verified MV3 behavior) | Never — design stateless/keep-alive from the start |
| Write Block IDs/metadata into source `.md` | No sidecar store needed | Pollutes user git; breaks "no lock-in" selling point; agent/IDE churn | Never by default; only via explicit opt-in project mode (Q7) |
| Open `type` vocabulary (accept any string) | User flexibility, less UI | OKF vocabulary sprawl (the documented criticism); broken filter/search | Never fully open; registration-gated extension only |
| Toast per FS/agent event | Easy to implement | Alert fatigue → the exact pain the product cures | Never; batch into Inbox digests |
| Naive YAML load/dump for frontmatter | One library call | Round-trip mangles user frontmatter (REQ-1.8 violation) | Never on files that already have frontmatter |
| Skip the file-protocol path, MCP-only | Faster F4 | Majority-without-MCP can't close loop (AC-4c fails) | Never — degrade path is a release gate |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Chrome MV3 native messaging | Assume opening a port keeps the service worker alive | Port no longer resets idle timer; use <30s keep-alive messages or a stateless loopback HTTP endpoint + retry (VERIFIED behavior) |
| Local MCP / loopback server | Treat 127.0.0.1 as a trust boundary | Per-install token auth + Origin validation; any local process/tab can otherwise reach it (DNS-rebind/CSRF) |
| FS watcher (chokidar/fsevents) | Handle save as delete+create; process mid-write files | `awaitWriteFinish`; identity by content-hash+inode; exclude own `.prismdocs/` output dir |
| Claude Code / Cursor round-trip | Require manual MCP + CLAUDE.md editing | Generate one-click config + protocol snippet; file-protocol fallback fully tested (AC-4c) |
| LLM endpoints (user's own key) | Hardcode Anthropic; ignore custom base_url; log prompts | Support OpenAI-compatible + custom base_url; key in system keychain; no telemetry on content |
| OKF export/import | "We wrote frontmatter, so it's OKF" | Validate exported bundle is actually consumable by an OKF reader; controlled `type` vocabulary |
| git-tracked repos | Write noise into tracked files; watch `.git` | Sidecar by default; suggest `.prismdocs/` in `.gitignore`; disk is authoritative on pull |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Non-incremental re-projection | LLM cost per small edit ~ full-doc; slow Lens updates | Cache by (blockId, baseVersion); reproject only affected blocks (AC-2b) | Immediately visible on user's own API bill; worsens with doc size |
| Fuzzy anchoring on short/generic quotes | Anchoring hangs on large docs | Structural match first, fuzzy only on sufficiently distinctive quote+context; cap search | Long docs (Hypothesis-documented perf cliff) |
| Full-text search without an index | Search > 300ms | SQLite FTS index (PRD NFR: <300ms @ 500 docs/2000 cards) | ~500 docs / 2000 cards |
| Debounce reset on every write | Doc never updates while agent streams a large file | Trailing debounce with max-wait cap | Any large single-file agent write |
| Projection cache not persisted | Full recompute on every app restart | Persist cache with Base version (REQ-5 reliability) | Every restart; painful at 500 docs |
| Watching a cloud-synced folder | Event storms, CPU spin | Warn on iCloud/Dropbox roots; coalesce; content-hash dedupe | Immediately on synced project roots |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Unauthenticated loopback MCP/HTTP port | Any local process or visited webpage exfiltrates workspace docs | Bind 127.0.0.1; per-install token; Origin/no-Origin validation; short-lived tokens |
| MCP write surface too broad | Confused/compromised agent mutates knowledge base | Expose read + comment回执 only; no create/delete of comments/cards; scope to current Workspace |
| Document/clip content interpolated into prompts as instructions | Prompt injection hijacks projection or F4 English-intent summary | Treat doc/clip text as untrusted data; delimit; never place in instruction section |
| API key in plaintext / config file | Key theft | System keychain only (PRD NFR); never log; support custom base_url without leaking key |
| Clipper requests `<all_urls>` + unclear data use | Store rejection; over-broad access | Narrowest host permissions; clear privacy disclosure (local-only) |
| Sending doc content to unexpected endpoints | Privacy breach, contradicts local-first promise | Content only to user-configured LLM endpoint; nothing to our servers (MVP no backend) |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Silent comment loss on rewrite | Catastrophic trust loss; the one thing that must never happen | Always downgrade + snapshot + "location changed, confirm"; never drop (AC-3b) |
| Stale Lens shown as current | User decides on outdated/fiction content | Version-gate Lens; show "regenerating/out-of-date" explicitly |
| Alert on every change | Recreates review fatigue the product cures | Batch into Inbox digests; conservative frequency budget |
| Decision Lens block without Base excerpt | Wrong go/no-go on distorted summary | Force Base original on all decision/risk blocks (AC-2c) |
| Manual MCP/CLAUDE.md setup | Users bounce before first value | Zero-config folder-watch; one-click generated config |
| Low-quality Lens (over-cheap model) | Users read Base anyway; product value gone | Fast floor + strong model on decision blocks (Q1) |
| No feedback when API key absent/exhausted | Confusing empty Lens | Base reading unaffected; guidance card in Lens area (REQ-2 edge case) |

## "Looks Done But Isn't" Checklist

- [ ] **Block anchoring:** works on small edits but untested on 50%/full rewrite, split/merge, reorder — verify against an adversarial golden corpus with the count-invariant assertion (migrated+downgraded == original).
- [ ] **Lens projection:** looks incremental but a log actually proves only affected blocks re-projected (AC-2b), and cancellation fires when Base changes mid-projection.
- [ ] **FS watcher:** works with hand edits but untested under agent burst-writes (AC-1b) and editor atomic-save; rename retains 100% comments (AC-1c).
- [ ] **Clipper bridge:** works right after launch but not after 30s idle (MV3 SW death) or with desktop app closed (offline queue REQ-6.5).
- [ ] **Feedback loop:** works with MCP installed but the file-only degrade path (AC-4c) is untested end-to-end.
- [ ] **Frontmatter:** parses but round-trip is not byte-stable on untouched keys; source files get polluted on open.
- [ ] **OKF export:** writes frontmatter but no external OKF consumer has actually read the bundle back.
- [ ] **Fidelity:** Lens reads well but the <5% distortion guardrail (AC-2d) is not being measured via the report channel.

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Silent comment loss already shipped | HIGH | Cannot recover lost comments; must add quote snapshots retroactively where possible, ship downgrade-never-drop, rebuild trust with visible "no loss" guarantee + audit log |
| Stale Lens shown as current | MEDIUM | Add (blockId, baseVersion) gating; invalidate cache; force re-projection of version-mismatched blocks on next open |
| Native-messaging bridge dying on idle | LOW-MEDIUM | Switch clip flow to stateless loopback endpoint + retry, or add <30s keep-alive; add offline queue |
| Source-file pollution shipped | MEDIUM | Migrate metadata to sidecar; provide a "clean my files" pass; make source-write opt-in; document the mistake |
| OKF type sprawl | LOW-MEDIUM | Introduce registration gate; provide a mapping/merge tool for existing unregistered types |
| Alert fatigue | LOW | Collapse to Inbox digests; add frequency budget + snooze; default conservative |
| Cost blowup from non-incremental projection | LOW-MEDIUM | Add (blockId, baseVersion) cache; add reprojected/changed ratio alarm; cap autoproject to ≤5k-token docs |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| 1. Silent comment loss / anchor drift | Foundational Block-Anchoring phase (before F3), re-verified F4 | AC-3b golden-corpus test: ≥90% migrated-or-downgraded, 0 silent loss; count-invariant assertion in CI |
| 2. Two-layer sync corruption | F2 (Lens scheduler/versioning) | Cancellation-on-mid-projection test; no stale-cache-hit on changed block; restart reuses cache correctly |
| 3. FS watcher hazards | F1 (import + watcher) | AC-1b (5 docs/10s, no dup records), AC-1c (rename retains 100% comments), atomic-save + burst harness |
| 4. LLM cost blowup / fidelity distortion | F2 (incrementality, guardrails, model routing); injection hardening F2/F4/F7 | AC-2b (only affected blocks logged), AC-2c (Base excerpt on all decision blocks), AC-2d (<5% distortion) |
| 5. MCP + MV3 lifecycle/bridge | F6 (clipper bridge + store), F4 (MCP auth + tool surface) | Clip works after 30s idle & with app closed; loopback rejects unauthenticated/browser-tab requests; MCP read+回执-only |
| 6. File pollution / round-trip / OKF sprawl | F1 (frontmatter, sidecar, vocabulary), F7 (OKF export), F6 (HTML→MD) | Zero git-diff after open; byte-stable frontmatter round-trip; external OKF reader consumes export; unregistered-type gate works |
| 7. Adoption inertia / alert fatigue | F4 (config + degrade path), F2/F3/F4 (notification budget) | AC-4c (file-only loop works); zero-manual-config setup; M2 activation ≥40%; notification-volume trend flat |

## Sources

- [The extension service worker lifecycle | Chrome for Developers](https://developer.chrome.com/docs/extensions/develop/concepts/service-workers/lifecycle) — HIGH (official; 30s idle timeout, keep-alive semantics)
- [Longer extension service worker lifetimes | Chrome for Developers](https://developer.chrome.com/blog/longer-esw-lifetimes) — HIGH (official; port no longer resets idle timer)
- [Chrome native host dies when extension service worker goes idle · anthropics/claude-code #16350](https://github.com/anthropics/claude-code/issues/16350) — HIGH (real-world reproduction of the exact MV3 native-messaging failure in a comparable product)
- [Native messaging port keep-alive claim untrue · developer.chrome.com #2688](https://github.com/GoogleChrome/developer.chrome.com/issues/2688) — HIGH (documents the behavior change)
- [Fuzzy Anchoring | Hypothesis](https://web.hypothes.is/blog/fuzzy-anchoring/) — HIGH (canonical multi-strategy text-anchoring approach: quote + context + position)
- [judell/TextQuoteAndPosition](https://github.com/judell/TextQuoteAndPosition) — HIGH (TextQuoteSelector + TextPositionSelector reference implementation)
- [Notes for an annotation SDK — Jon Udell](https://blog.jonudell.net/2021/09/03/notes-for-an-annotation-sdk/) — MEDIUM (anchoring design tradeoffs)
- [Quote anchoring blocks execution for a long time · hypothesis/client #3919](https://github.com/hypothesis/client/issues/3919) — MEDIUM (fuzzy-anchoring perf cliff on short/generic quotes)
- [Google OKF: A Standard, or Just a Folder? — Marc Bará](https://medium.com/@marc.bara.iniesta/googles-new-format-for-agent-context-a-standard-or-just-a-folder-82fb21d92041) — HIGH (the documented OKF type-vocabulary-sprawl criticism)
- Project docs: `.planning/PROJECT.md`, `docs/BRD_PrismDocs_MVP.md` (§4.2, §11), `docs/PRD_PrismDocs_MVP.md` (F1–F7 edge cases, §5 NFR, §8 open questions), `docs/调研补充_CoWiki与OKF对BRD_PRD的影响.md` — HIGH (product-specific failure modes and acceptance criteria)

---
*Pitfalls research for: PrismDocs — two-layer docs workbench + MV3 clipper + local MCP*
*Researched: 2026-07-27*
