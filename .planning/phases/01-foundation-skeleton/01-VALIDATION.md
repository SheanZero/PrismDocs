---
phase: 1
slug: foundation-skeleton
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
| **Framework** | `cargo test` built-in harness (+ optional `cargo-nextest` for process isolation) + `tempfile` 3 + `serial_test` 3; Vitest for the frontend smoke surface |
| **Config file** | none — Wave 0 creates all of it (no `Cargo.toml`, no `vite.config.ts`, no `package.json`; repo currently holds only `docs/` + `.planning/`) |
| **Quick run command** | `cargo test -p prism-types -p prism-store -p prism-llm -p prism-mcp -p prism-engine` |
| **Full suite command** | `just check-dup && just check-tauri-free && just check-no-cycle && just check-single-egress && cargo test --workspace && npm run test` |
| **Estimated runtime** | ~60 seconds (greenfield; re-measure after Wave 0) |
| **Coverage** | Measured, not gated, in Phase 1 — `just coverage` (`cargo llvm-cov` + `vitest --coverage`). Rationale and the Phase-2 hard-gate handoff are recorded in `01-02-PLAN.md` Task 2. |

> **Filter form (hard rule — a green that proves nothing is worse than a red).**
> `cargo test -p <crate> <FILTER>` filters by **test name**, not by file name. A filter that names an
> integration-test *file* matches zero tests and **still exits 0**. Select a file with `--test <file>`;
> use a bare filter only when it is a substring of a real `fn` name (or of the `module::tests::fn`
> path for in-module unit tests). Every bare filter in the tables below is a function or module name
> that exists in the plan which defines it.

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p <changed crate>` + `cargo clippy -p <changed crate> -- -D warnings`
- **After every plan wave:** Run `just check-dup && just check-tauri-free && just check-no-cycle && just check-single-egress && cargo test --workspace`
- **Before `/gsd-verify-work`:** Full suite green + the four manual smoke-page checks below + `just coverage` run once and its two numbers recorded in the phase SUMMARY
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| _pending_ | — | — | INFRA-01 | — | — | — | _filled by `/gsd-validate-phase` after planning_ | — | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

### Requirement → Automated Command (from RESEARCH.md, pre-task-ID)

| Req ID | Behavior | Test Type | Automated Command | File Exists |
|--------|----------|-----------|-------------------|-------------|
| INFRA-01 | engine workspace testable without tauri (D-01) | dependency assertion | `just check-tauri-free` | ❌ W0 |
| INFRA-01 | engine-only test suite green | integration | `cargo test -p prism-types -p prism-store -p prism-llm -p prism-mcp -p prism-engine` | ❌ W0 |
| INFRA-01 | no duplicate rusqlite/reqwest/libsqlite3-sys | dependency assertion | `just check-dup` | ❌ W0 |
| INFRA-01 | prism-mcp has no facade dependency (D-09) | dependency assertion | `just check-no-cycle` | ❌ W0 |
| INFRA-01 | prism-mcp returns data through the injected trait | integration | `cargo test -p prism-mcp --test trait_injection` | ❌ W0 |
| INFRA-01 | bus event → coarse payload mapping (all three arms) | unit | `cargo test -p prismdocs-shell bus_adapter` (module-path filter; matches `bus_adapter::tests::*` and `bus_adapter_maps_event_to_emit` in either layout) | ❌ W0 |
| INFRA-01 | Lagged→Resync specifically (the silent-failure arm) | unit | `cargo test -p prismdocs-shell lagged_maps_to_resync` | ❌ W0 |
| INFRA-01 | Channel command invocable and returns Ok | integration (`tauri::test`) | `cargo test -p prismdocs-shell --features test --test ipc` | ❌ W0 |
| INFRA-02 | project-scoped search isolation | integration | `cargo test -p prism-store --test fts_cjk search_is_scoped_to_project` | ❌ W0 |
| INFRA-02 | migration set valid | unit | `cargo test -p prism-store migrations_are_valid` | ❌ W0 |
| INFRA-02 | concurrent read/write under WAL, no BUSY | integration | `cargo test -p prism-store --test concurrency reader_snapshot_is_isolated` | ❌ W0 |
| INFRA-02 | pooled connection cannot write (`query_only=ON`) | integration | `cargo test -p prism-store --test concurrency pooled_connection_cannot_write` | ❌ W0 |
| INFRA-02 | bundled SQLite ≥3.51.3 | integration | `cargo test -p prism-store --test concurrency bundled_sqlite_meets_minimum` | ❌ W0 |
| INFRA-02 | Chinese query returns non-zero rows (trigram) | integration | `cargo test -p prism-store --test fts_cjk chinese_query_returns_nonzero_rows` | ❌ W0 |
| INFRA-02 | FTS index follows UPDATE/DELETE (triggers) | integration | `cargo test -p prism-store --test fts_cjk fts_index_follows_update_and_delete` | ❌ W0 |
| INFRA-02 | index/content rowid stay aligned across VACUUM | integration | `cargo test -p prism-store --test fts_cjk search_survives_vacuum` | ❌ W0 |
| INFRA-02 | `wal_checkpoint(TRUNCATE)` on close | integration | `cargo test -p prism-store --test concurrency wal_truncated_on_close` | ❌ W0 |
| INFRA-03 | secret round-trip (mock store) | unit | `cargo test -p prism-llm roundtrip_with_mock_store` | ❌ W0 |
| INFRA-03 | app starts with no key (`NoEntry` → `Ok(None)`) | unit | `cargo test -p prism-llm no_key_is_not_an_error` | ❌ W0 |
| INFRA-03 | only prism-llm holds network/secret deps | dependency assertion | `just check-single-egress` | ❌ W0 |
| INFRA-03 | no plaintext secrets in code/config | static check | `git grep -nE '(sk-[A-Za-z0-9]{16,}\|api[_-]?key\s*=\s*["\x27][^"\x27]{8,})' -- ':!*.planning/*'` returns nothing | ❌ W0 |
| INFRA-03 | base_url validation (http/https only) | unit | `cargo test -p prism-store settings_base_url_validation` | ❌ W0 |

---

## Wave 0 Requirements

Greenfield project — test infrastructure is 100% absent. Wave 0 must establish:

- [ ] Root `Cargo.toml` (`[workspace] members` + `[workspace.dependencies]`) and each engine crate's `Cargo.toml`
- [ ] `src-tauri/Cargo.toml` (with `[features] test = ["tauri/test"]`) and `tauri.conf.json`
- [ ] `package.json` + `vite.config.ts` + vitest config
- [ ] `justfile` (4 dependency-direction assertions + `test-engine`) or equivalent `scripts/check-deps.sh`
- [ ] `crates/prism-store/tests/concurrency.rs` — INFRA-02 WAL / query_only / version
- [ ] `crates/prism-store/tests/fts_cjk.rs` — INFRA-02 Chinese hit + trigger sync + VACUUM alignment + project scoping (4 test fns)
- [ ] `migrations_are_valid` unit test inside `crates/prism-store/src/migrations.rs`
- [ ] mock/real keyring tests in `crates/prism-llm/src/secrets.rs` (real path `#[ignore]`)
- [ ] `crates/prism-mcp/tests/trait_injection.rs` — fake `FeedbackSource` impl
- [ ] `src-tauri/tests/ipc.rs` — `tauri::test::mock_builder` command registration test
- [ ] Pure-function mapping unit tests for `src-tauri/src/bus_adapter.rs` (incl. Lagged→Resync)
- [ ] Isolation convention: every store test injects a `tempfile::TempDir` data root — must never touch the real `~/Library/Application Support/PrismDocs/`
- [ ] Framework install: Rust harness is built in; `npm install -D vitest`; optionally `cargo install cargo-nextest`
- [ ] Coverage tooling: `cargo llvm-cov` (CI installs it) + `@vitest/coverage-v8` — measured and reported, not gated, in Phase 1 (see `01-02-PLAN.md` Task 2)
- [ ] CI workflow (GitHub Actions, macOS runner) wiring the per-wave commands

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Event round-trip + Channel ordering through a real WebView | INFRA-01 | `tauri::test` mock runtime has no real WebView, so the emit→frontend→render path cannot be asserted in-process | Smoke page: click → event count matches 1:1; streaming `seq` strictly increasing with no gaps (total=1000) |
| Secret round-trip against the real macOS Keychain | INFRA-03 | CI/headless has no unlocked keychain; `keyring_core::set_default_store` is process-global | `cargo test -p prism-llm -- --ignored roundtrip_with_real_keychain`, or perform it through the settings page |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
