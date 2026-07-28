---
phase: 1
slug: foundation-core-engine-skeleton
# status lifecycle: draft (seeded by plan-phase) → validated (set by validate-phase §6)
# audit-milestone §5.5 distinguishes NOT-VALIDATED (draft) from PARTIAL (validated + nyquist_compliant: false) (#2117)
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-28
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Seeded from `01-RESEARCH.md` § Validation Architecture. Per-task rows are filled by `/gsd-validate-phase` once PLAN.md task IDs exist.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` / `#[tokio::test]` (`cargo test`) — no third-party runner |
| **Config file** | none — Wave 0 creates the workspace manifest and test dirs (greenfield) |
| **Quick run command** | `cargo test -p prism-store -p prism-mcp -p prism-llm` |
| **Full suite command** | `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all --check` |
| **Estimated runtime** | ~30s warm (first build compiles bundled SQLite; cached thereafter) |

Loopback integration tests bind `TcpListener::bind("127.0.0.1:0")` (OS-assigned port) so they never collide with the running app or with each other.

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p <touched crate>`
- **After every plan wave:** Run `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite green + `cargo tree -d` clean + `cargo fmt --all --check`
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | AGENT-03 | T-1-01 DNS rebinding | Binds 127.0.0.1 only; LAN IP refused | integration | `cargo test -p prism-mcp bind_is_loopback_only` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | AGENT-03 | T-1-02 missing auth | No `Authorization` → 401 | integration | `cargo test -p prism-mcp rejects_missing_token` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | AGENT-03 | T-1-03 token guess | Wrong bearer token → 401 (constant-time compare) | integration | `cargo test -p prism-mcp rejects_wrong_token` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | AGENT-03 | T-1-01 DNS rebinding | Valid token + foreign `Origin` → 403 | integration | `cargo test -p prism-mcp rejects_foreign_origin` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | AGENT-03 | T-1-01 DNS rebinding | Valid token + no `Origin` → 200 (Claude Code sends none) | integration | `cargo test -p prism-mcp accepts_missing_origin` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | AGENT-03 | T-1-01 DNS rebinding | Valid token + foreign `Host` → 403 | integration | `cargo test -p prism-mcp rejects_foreign_host` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | AGENT-03 | T-1-04 write-surface escape | `prism-mcp` cannot reference the write-ops trait (structural, D-03) | unit (source assertion) | `cargo test -p prism-mcp agent_write_surface_is_minimal` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | AGENT-03 | T-1-05 cross-workspace read | A second workspace's rows are invisible | unit | `cargo test -p prism-store workspace_scoping` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | NFR-02 | — | `journal_mode` reads back `wal` after open and after close+reopen | unit | `cargo test -p prism-store wal_is_persistent` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | NFR-02 | — | Migrations apply empty → latest, idempotent on re-run | unit | `cargo test -p prism-store migrations_to_latest` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | NFR-02 | — | `Migrations::validate()` passes | unit | `cargo test -p prism-store migrations_validate` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | NFR-02 | — | Open + read an existing DB with no network | unit | `cargo test -p prism-store offline_open_and_read` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | NFR-02 | — | Backup/export yields a standalone file; row counts match | integration | `cargo test -p prism-store backup_round_trip` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | NFR-02 | T-1-20 unrestorable backup | Restore with no marker and no registry row adopts the archive manifest's `project_id` rather than minting a new one | integration | `cargo test -p prism-store restore_from_archive_adopts_manifest_project_id` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | NFR-03 | T-1-06 egress sprawl | `reqwest` appears once in the dep graph, only under `prism-llm` | unit (manifest assertion) | `cargo test -p prism-core single_egress_path` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | NFR-03 | T-1-07 key in logs | `Debug` on the secret newtype renders no key material | unit | `cargo test -p prism-llm secret_debug_is_redacted` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | NFR-03 | — | No telemetry crate in `Cargo.lock` | unit (manifest assertion) | `cargo test -p prism-core no_telemetry_dependency` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | NFR-04 | — | `SecretStore` round-trip: set → get → delete → `NoEntry` | unit (in-memory impl) | `cargo test -p prism-llm secret_store_round_trip` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | NFR-04 | — | Real macOS keychain round-trip | integration, `#[ignore]`d | `cargo test -p prism-llm --ignored keychain_real` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | NFR-04 | T-1-08 SSRF blast radius | `base_url` normalization + scheme allowlist (http/https only) | unit | `cargo test -p prism-llm base_url_normalization` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | NFR-04 | — | `GET /v1/models` against a local mock, both provider families, correct auth header per family | integration (mock server) | `cargo test -p prism-llm models_probe_both_families` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Root `Cargo.toml` workspace manifest — prerequisite for every test
- [ ] `rust-toolchain.toml` pinning ≥ 1.95 — `rusqlite_migration` 2.6 hard MSRV
- [ ] Toolchain probe: Rust ≥ 1.95 and Xcode Command Line Tools (`cc`) present — **blocking if absent**, not probed during research (assumption A6)
- [ ] `crates/prism-store/tests/` + `fn test_db() -> (TempDir, Pool)` fixture — covers NFR-02
- [ ] `crates/prism-mcp/tests/` + `fn spawn_test_server() -> (SocketAddr, String /*token*/, CancellationToken)` fixture binding port 0 — covers AGENT-03
- [ ] `crates/prism-llm/tests/` + hand-rolled axum mock HTTP server (axum is already in the tree → zero new deps) — covers NFR-03/NFR-04
- [ ] `SecretStore` trait + `InMemorySecretStore` — makes NFR-04 testable in CI without a Keychain prompt
- [ ] `.github/workflows/ci.yml` on `macos-latest` (arm64), split `core` (no Tauri/Node/webview) from `shell` — the split is the executable proof that D-01 holds
- [ ] Framework install: **none.** Only `tempfile` is a new dev-dependency.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| App launches on macOS Apple Silicon; 4-step onboarding completes against a **real** endpoint | SC-1 / NFR-04 | Requires the user's real API key and a live network endpoint; cannot be automated without shipping a credential. Everything except the live call is covered by `models_probe_both_families`. | `npm run tauri dev` → complete onboarding steps 1–4 → confirm key lands in Keychain (Keychain Access → search `PrismDocs`) and no key appears in stdout |
| Real Claude Code connects to the loopback MCP endpoint using the generated `.mcp.json` | AGENT-03 / SC-4 | Validates assumption A1 (real clients send no `Origin`). No substitute for the real client handshake. | Run onboarding step 4 → point Claude Code at the generated config → confirm the server appears connected and rejections are logged for a hand-crafted foreign-`Origin` request |
| ADR names the rejected Node-sidecar alternative and its downgrade trigger | SC-5 / D-06 | Prose quality is not machine-checkable; a file-existence CI check is near-worthless | Read the ADR; confirm it contains the Rust-native choice, the rejected alternative, the downgrade trigger condition (D-05), and the accepted SSRF risk |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
