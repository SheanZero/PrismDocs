# Walking Skeleton — PrismDocs

**Phase:** 1
**Generated:** 2026-07-28

> The Walking Skeleton is the Phase-1 special case of the tracer: one production-quality thread through every layer the project will ever have. It is not a prototype — every line of it is written for keeps, and later phases expand outward from it without altering the decisions below. Stubs exist only where they can later be filled without an architectural change.

## Capability Proven End-to-End

A user launches PrismDocs on macOS, enters a custom LLM endpoint and API key, and sees a real model list returned from that endpoint — with the key stored in the system Keychain and the provider profile persisted in a WAL SQLite database under a single backup-able directory outside their repository.

That single thread touches SC-1, SC-2, and SC-3 literally, and exercises every layer the rest of the project builds on: React webview → typed Tauri IPC → shell-agnostic Engine facade → LLM boundary with real network egress → sidecar store with real persistence.

## Architectural Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Shell | Tauri v2 (2.11.5), macOS Apple Silicon first | ~8–12 MB bundle versus Electron's 150 MB+ for an always-resident local-first app; the Rust backend is the natural home for the CPU-bound engine; Windows stays reachable at near-zero incremental cost (P1). |
| Core language | Rust, workspace MSRV **1.95** | Driven by `rusqlite_migration` 2.6. Owns filesystem, database, parsing, anchoring, keychain, LLM, and MCP — one deterministic source of truth. |
| Module boundary | Cargo workspace: `prism-core` / `prism-store` / `prism-llm` / `prism-mcp`, with `src-tauri` as a thin command layer (D-01) | The boundary is enforced by the compiler rather than by convention. Phase 3's anchoring engine must be drivable headless against an adversarial corpus, so no core crate may depend on `tauri`. Slots reserved for `prism-anchor` (Phase 3) and `prism-lens` (Phase 4). |
| Facade | A single `Engine` in `prism-core` with three traits: `ReadOps` (both doors), `AgentWriteOps` (one method — the comment receipt), `AppWriteOps` (app only) (D-02/D-03) | One engine, two doors. AGENT-03's "the agent cannot create or delete comments and cards" becomes a compile-time property; `prism-mcp` is generic over the first two traits and names the third nowhere. |
| Data layer | SQLite via `rusqlite` 0.40 (`bundled`), **WAL mode**, migrations by `rusqlite_migration` 2.6, connections from an `r2d2` pool | `bundled` statically links SQLite — no system dependency, reproducible offline builds. WAL lets the MCP reader and the app writer proceed without blocking (D-14). The facade holds a **pool**, not a shared handle: `rusqlite::Connection` is `Send` but not `Sync`, and the obvious mutex-wrapped alternative serializes reads behind writes and cancels WAL's entire benefit. Every store call reachable from async is wrapped in `spawn_blocking` inside the store crate. |
| Storage location | `~/Library/Application Support/PrismDocs/` — `app.db` for install-scoped state, `<project-id>/prism.db` per project (D-11/D-13) | Never inside the user's repository: zero git noise, no private comments leaked. Keyed by project id, never by path, so a moved or renamed folder keeps its data. Deliberately the human-readable product name rather than Tauri's reverse-DNS bundle identifier, because it is a user-visible backup location. The core never resolves this path itself — it is injected. |
| Backup | Explicit export via SQLite `VACUUM INTO` plus a zip of the project data directory (D-12) | In WAL mode a database is three files, so a copy of the main file silently loses uncheckpointed writes. `VACUUM INTO` produces one consistent standalone file. NFR-02's "single backup-able directory" is met by an export feature, not by moving the database into the repo. |
| Secrets | `keyring` 4.1 directly from `prism-llm` — macOS Keychain, service `PrismDocs` | NFR-04 says system keychain. `tauri-plugin-stronghold` is slated for removal in Tauri v3; `tauri-plugin-keyring` is unmaintained and would put secret access behind the shell. Two entries: the LLM key and the per-install MCP token. |
| LLM boundary | One `reqwest` client private to `prism-llm`; `LlmProvider` trait with `async-openai` and a thin hand-rolled Anthropic client (D-04/D-05/D-15) | A single egress choke point makes NFR-03 structural rather than a policy. The Anthropic implementation sits behind the trait so the Node-sidecar fallback swaps one impl and no caller. Both families reachable from one call site selected by a stored `ProviderProfile`. |
| Agent integration | App-hosted loopback streamable HTTP: `rmcp` 2.2 `StreamableHttpService` mounted on axum at `127.0.0.1:47917`, bearer token layered outside it, non-empty Origin allowlist (D-07/D-09/D-10) | Literally satisfies AGENT-03's "127.0.0.1 + token + Origin" wording, which a stdio pipe cannot carry. The app is itself the server — no subprocess, no second IPC hop. Protocol revision 2025-11-25 is the library default. The same listener hosts the Phase 6 clip route (D-08). |
| Typed IPC | `tauri-specta` with the generated `src/bindings.ts` **committed**; fewer than ten commands in Phase 1 | Still RC after two years, so committing the output means an RC break can never block a build. Keeping the surface small caps the blast radius. |
| Directory layout | `crates/prism-*` for the core, `src-tauri/` for the shell, `src/` for the webview, `docs/adr/` for decisions | Feature crates, not layer crates. Later phases add a crate, not a layer. |
| Runnable command | `npm run tauri dev` exercises the full stack locally | No hosted dev environment in Phase 1; the desktop app is the deployment target and dev builds run unsigned. |

## Stack Touched in Phase 1

- [x] Project scaffold — Cargo workspace, Tauri v2 shell, Vite + React 19 + TypeScript, clippy, rustfmt, `cargo test`
- [x] Routing — onboarding steps 1–4 and a project home in the webview; `/mcp` on the loopback HTTP host
- [x] Database — real write (provider profile, workspace registration) and real read (relaunch reads both back with no network)
- [x] UI — the onboarding step-1 form wired through typed IPC to a real network call, rendering the real response
- [x] Deployment — documented local full-stack run command `npm run tauri dev`; CI on `macos-latest` splits a Node-free `core` job from a full `shell` build

## Out of Scope (Deferred to Later Slices)

This list exists so a future phase does not re-litigate Phase 1's minimalism.

- **Document import for real** — configurable globs, `.html` to Markdown, frontmatter parsing and byte-stable round-trip, the filesystem watcher and its 2s debounce, rename-by-content-hash, incremental sync. Phase 2 (F1). Phase 1's enumeration is strictly read-only display feedback: paths and a count, no file bodies opened, no watcher, no document table (D-18).
- **Block anchoring** — the Block tree, stable Block IDs, multi-signal migration, downgrade-never-drop, the adversarial golden corpus. Phase 3. `comrak` is not yet a dependency.
- **Lens projection and comments** — colloquial Chinese projection, incremental re-projection, the fidelity guardrail, threaded block comments. Phase 4. Phase 1's `LlmProvider` trait implements only the model-list probe; SSE streaming is deliberately not hand-rolled yet.
- **MCP tool implementations** — `list_feedback`, `get_feedback`, `respond_to_comment`, `get_document_comments`, `get_context_pack`, `list_cards`, `export_okf_bundle`. Phases 5/6/7. Phase 1 ships the transport and the security boundary with zero tools registered.
- **Feedback Bundle semantics** — the contents of `.prismdocs/feedback/`. Phase 5. Phase 1 creates the empty directory and an English README describing its purpose.
- **The clip WebSocket protocol and the Chrome extension** — Phase 6. Phase 1 leaves a marked route slot on the loopback router and nothing else.
- **Context Pack and OKF export** — Phase 7. Phase 1's README declares OKF v0.1 conformance for the directory's outputs; nothing materializes frontmatter yet.
- **Telemetry** — NFR-07's opt-in instrumentation is Phase 5. Phase 1 ships no analytics dependency at all, so "off by default" is provable from the lockfile.
- **Cost display and token accounting** — NFR-05, Phase 4. `tiktoken-rs` is not yet a dependency.
- **Code signing, notarization, universal binaries, Windows** — release phase and P1 respectively. Dev builds run unsigned on Apple Silicon only.

## Subsequent Slice Plan

Each later phase adds one vertical slice on top of this skeleton without altering the architectural decisions above.

- **Phase 2** — a user points PrismDocs at a folder or repo and their Markdown becomes a live, disk-authoritative Base layer that tracks external edits within 10s. Adds import and watch behind `AppWriteOps`; adds the `documents` table as a clean `CREATE`.
- **Phase 3** — every Base document gets stable Block IDs that survive an AI rewrite. Adds `crates/prism-anchor`, driven headless against an adversarial corpus — which is only possible because the core crates carry no shell dependency.
- **Phase 4** — a user reads a Chinese Lens of any document and comments on any block. Adds `crates/prism-lens`; the `LlmProvider` trait gains streaming; comment tables land and `AgentWriteOps`' single receipt method finally has something to write to.
- **Phase 5** — the North-Star loop closes: comment → bundle → agent → re-project → resolve. Fills `.prismdocs/feedback/` with real semantics and registers the first MCP tools on the transport this phase built.
- **Phase 6** — cards and the Chrome clipper. The clip WebSocket takes the route slot reserved on the loopback router.
- **Phase 7** — Context Pack assembly and OKF Bundle export.
