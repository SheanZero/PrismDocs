# API Coverage — Phase 1

> Full coverage by default. Opt-outs are explicit, reasoned decisions.
>
> This phase touches three external surfaces: it **consumes** the Anthropic Messages API and OpenAI-compatible APIs, and it **hosts** an MCP server surface consumed by Claude Code and Cursor. It also consumes the macOS Keychain and writes a Claude Code client-config file.
>
> Phase 1 is deliberately a skeleton: on the LLM side it implements only a zero-token connection probe (D-16), and on the MCP side only the transport plus the security boundary (`01-CONTEXT.md` `<deferred>` assigns every MCP tool to Phase 5/6/7). Those opt-outs are reasoned scope decisions, not oversights, and each carries the phase that owns it.
>
> **Two-integration rule applied:** the Anthropic and OpenAI-compatible families are two integrations against the same need (D-15 requires both at launch). Each was re-decided from a full-coverage baseline rather than inheriting the other's opt-outs, which is why neither family is a second-class fallback — both reach the same capability set through one call site.

---

## Anthropic Messages API (consumed)

| capability | decision | reason |
|---|---|---|
| `models.list` — `GET /v1/models` | INTEGRATE | |
| custom `base_url` | INTEGRATE | |
| auth headers (`x-api-key`, `anthropic-version: 2023-06-01`) | INTEGRATE | |
| error taxonomy (DNS, TLS, 401/403, 404, unparseable 200) | INTEGRATE | |
| `models.retrieve` — `GET /v1/models/{id}` | OPT-OUT | not needed — the list response already carries everything the model picker shows |
| `messages.create` — `POST /v1/messages` | OPT-OUT | not needed yet — first real generation is Lens projection, Phase 4 (LENS-01). The Phase 1 probe is deliberately zero-token and zero-cost |
| `messages` SSE streaming | OPT-OUT | not needed yet — Phase 4 (LENS-09) needs per-segment streaming; hand-rolling the SSE loop now would be unexercised code, and D-05 keeps it swappable |
| `messages.count_tokens` | OPT-OUT | not needed yet — Phase 4 (NFR-05) cost display. Retained as the documented fallback probe when a user's proxy does not forward `/v1/models` (RESEARCH Open Question 5) |
| Message Batches API | OPT-OUT | explicitly out of scope — no batch/async projection in the MVP |
| Files API | OPT-OUT | explicitly out of scope — documents are read from the user's disk, never uploaded |
| Admin API | OPT-OUT | explicitly out of scope — single-user local app, no org management |
| prompt caching | OPT-OUT | not needed yet — a Phase 4 cost optimization once projection volume exists |
| tool use / function calling | OPT-OUT | explicitly out of scope — PrismDocs calls the model, it does not give the model tools |
| extended thinking | OPT-OUT | not needed yet — a Phase 4 model-routing question (Q1) |

## OpenAI-compatible API (consumed)

| capability | decision | reason |
|---|---|---|
| `models.list` — `GET /v1/models` | INTEGRATE | |
| custom `base_url` via client config | INTEGRATE | |
| `Authorization: Bearer` auth | INTEGRATE | |
| error taxonomy (DNS, TLS, 401/403, 404, unparseable 200) | INTEGRATE | |
| `chat.completions.create` | OPT-OUT | not needed yet — Phase 4 (LENS-01) |
| `chat.completions` streaming | OPT-OUT | not needed yet — Phase 4 (LENS-09) |
| legacy `completions` | OPT-OUT | explicitly out of scope — deprecated upstream; the chat endpoint is the compatible-ecosystem standard |
| `embeddings` | OPT-OUT | explicitly out of scope — search is SQLite FTS5, local and offline (NFR-02); no vector store in the MVP |
| `moderations` | OPT-OUT | explicitly out of scope — content is the user's own private documents |
| `images`, `audio` | OPT-OUT | explicitly out of scope — text-only product |
| function/tool calling | OPT-OUT | explicitly out of scope — same reason as the Anthropic row |

## MCP server surface (hosted by PrismDocs)

### Transport and protocol

| capability | decision | reason |
|---|---|---|
| streamable HTTP transport on `127.0.0.1` | INTEGRATE | |
| `initialize` handshake, protocol revision 2025-11-25 | INTEGRATE | |
| `tools/list` | INTEGRATE | returns an empty list in Phase 1 — the handshake surface is real, the tools are not yet |
| session management (`Mcp-Session-Id`) | INTEGRATE | delegated to the library's session manager; not reimplemented |
| SSE resumption / `Last-Event-ID` | INTEGRATE | delegated to the library |
| `Host` validation (DNS-rebinding defence) | INTEGRATE | |
| `Origin` validation with an explicitly non-empty allowlist | INTEGRATE | |
| per-install bearer-token authentication | INTEGRATE | not provided by the library — implemented as a middleware layer outside the mounted service |
| cancellation / graceful shutdown | INTEGRATE | |
| stdio transport | OPT-OUT | explicitly out of scope — D-07 chose app-hosted loopback HTTP because AGENT-03's `Origin` clause has no meaning over a stdio pipe. Reconsider only if a first-class client drops HTTP support |
| OAuth 2.1 authorization | OPT-OUT | not needed — single-user loopback server; a per-install bearer token plus Origin/Host validation is the ASVS L1-appropriate control for this trust boundary |
| `resources/*` | OPT-OUT | not needed yet — documents reach agents through the file protocol and, from Phase 7, `get_context_pack` |
| `prompts/*` | OPT-OUT | not needed — PrismDocs does not supply prompt templates to agents |
| `sampling/*` (server-initiated model calls) | OPT-OUT | explicitly out of scope — NFR-03 requires content to go only to the user-configured endpoint; routing generation through the agent's model would violate that |
| `logging/*` | OPT-OUT | not needed — rejections and diagnostics go to the app's own tracing subscriber, which the user can see |

### Tools (PRD §4.2)

Every tool is deferred by `01-CONTEXT.md` `<deferred>`; Phase 1 ships the transport and the security boundary that will host them.

| capability | decision | reason |
|---|---|---|
| `list_feedback` | OPT-OUT | not needed yet — Phase 5 (LOOP-03) |
| `get_feedback` | OPT-OUT | not needed yet — Phase 5 (LOOP-03) |
| `respond_to_comment` | OPT-OUT | not needed yet — Phase 5. The single-method agent write trait that will carry it exists now, so the write surface is already bounded |
| `get_document_comments` | OPT-OUT | not needed yet — Phase 5 (LOOP-03) |
| `get_context_pack` | OPT-OUT | not needed yet — Phase 7 (PACK-05) |
| `list_cards` | OPT-OUT | not needed yet — Phase 6 (CARD-05 feeds it) |
| `export_okf_bundle` | OPT-OUT | not needed yet — Phase 7 (PACK-06) |

## Claude Code client configuration (written by PrismDocs)

| capability | decision | reason |
|---|---|---|
| project-scope `.mcp.json` HTTP server entry | INTEGRATE | |
| `headersHelper` credential indirection | INTEGRATE | the token must never be written into `.mcp.json`, which is designed to be committed |
| merge into an existing `.mcp.json` preserving other servers | INTEGRATE | |
| CLAUDE.md / AGENTS.md protocol snippet | INTEGRATE | |
| `${VAR}` environment expansion | OPT-OUT | not needed — `headersHelper` already solves the same problem; supporting a second indirection adds surface with no user-visible gain. Revisit if a user environment cannot execute the helper |
| user-scope and local-scope config | OPT-OUT | not needed yet — PrismDocs is project-scoped by design; a global entry would point at whichever project happened to be open |
| Claude Code hooks | OPT-OUT | not needed yet — AGENT-04 lists hooks as part of first-class support, and they land with the loop in Phase 5 |

## macOS Keychain via `keyring` (consumed)

| capability | decision | reason |
|---|---|---|
| `set_password` / `get_password` / `delete_credential` | INTEGRATE | |
| distinct entries per account (LLM key, MCP install token) | INTEGRATE | |
| `set_secret` / `get_secret` (raw bytes) | OPT-OUT | not needed — both stored secrets are UTF-8 text; the byte API would add a second code path for no benefit |
| non-default credential stores / cross-platform backends | OPT-OUT | not needed yet — Windows Credential Manager is handled transparently by the same crate when Windows lands (P1) |
| ambiguous-credential resolution APIs | OPT-OUT | not needed — exactly one entry exists per `(service, account)` pair by construction, asserted by `token_is_minted_once_and_reused` and `secret_set_twice_overwrites` |
