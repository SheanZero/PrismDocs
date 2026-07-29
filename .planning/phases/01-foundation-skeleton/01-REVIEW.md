---
phase: 01-foundation-skeleton
reviewed: 2026-07-29T00:00:00Z
depth: standard
files_reviewed: 63
files_reviewed_list:
  - .github/workflows/ci.yml
  - crates/prism-anchor/src/lib.rs
  - crates/prism-cli/src/main.rs
  - crates/prism-engine/Cargo.toml
  - crates/prism-engine/src/bus.rs
  - crates/prism-engine/src/error.rs
  - crates/prism-engine/src/facade.rs
  - crates/prism-engine/src/lib.rs
  - crates/prism-engine/src/services.rs
  - crates/prism-engine/tests/facade.rs
  - crates/prism-fs/src/lib.rs
  - crates/prism-llm/Cargo.toml
  - crates/prism-llm/src/lib.rs
  - crates/prism-llm/src/secrets.rs
  - crates/prism-mcp/Cargo.toml
  - crates/prism-mcp/src/deps.rs
  - crates/prism-mcp/src/handler.rs
  - crates/prism-mcp/src/lib.rs
  - crates/prism-mcp/src/middleware.rs
  - crates/prism-mcp/src/server.rs
  - crates/prism-mcp/tests/middleware_gate.rs
  - crates/prism-mcp/tests/trait_injection.rs
  - crates/prism-parse/src/lib.rs
  - crates/prism-store/Cargo.toml
  - crates/prism-store/migrations/001_schema_v1.sql
  - crates/prism-store/src/error.rs
  - crates/prism-store/src/lib.rs
  - crates/prism-store/src/migrations.rs
  - crates/prism-store/src/open.rs
  - crates/prism-store/src/search.rs
  - crates/prism-store/src/seed.rs
  - crates/prism-store/src/settings.rs
  - crates/prism-store/tests/concurrency.rs
  - crates/prism-store/tests/fts_cjk.rs
  - crates/prism-types/Cargo.toml
  - crates/prism-types/src/dto.rs
  - crates/prism-types/src/event.rs
  - crates/prism-types/src/lib.rs
  - crates/prism-types/src/service.rs
  - crates/prism-types/tests/contract.rs
  - docs/keychain-naming.md
  - scripts/check-deps.sh
  - scripts/check-secrets.sh
  - src-tauri/Cargo.toml
  - src-tauri/capabilities/default.json
  - src-tauri/src/bus_adapter.rs
  - src-tauri/src/commands.rs
  - src-tauri/src/lib.rs
  - src-tauri/src/smoke.rs
  - src-tauri/tauri.conf.json
  - src-tauri/tests/ipc.rs
  - src/App.test.tsx
  - src/App.tsx
  - src/lib/capabilities.test.ts
  - src/lib/ipc.ts
  - src/lib/queryClient.ts
  - src/lib/useEngineInvalidation.test.ts
  - src/lib/useEngineInvalidation.ts
  - src/main.tsx
  - src/pages/DevSmoke.test.tsx
  - src/pages/DevSmoke.tsx
  - src/pages/Settings.test.tsx
  - src/pages/Settings.tsx
findings:
  critical: 3
  warning: 16
  info: 6
  total: 25
status: issues_found
---

# Phase 1: Code Review Report

**Reviewed:** 2026-07-29
**Depth:** standard
**Files Reviewed:** 63
**Status:** issues_found

## Summary

The architectural invariants this phase set out to establish do hold. I independently
re-derived the ones that mattered rather than trusting the prose: the dependency
direction is genuinely acyclic, `prism-types` really is serde+thiserror only, the FTS
index is trigger-driven with no manual `INSERT INTO documents_fts` anywhere in Rust,
and the three MCP gates each have an isolation counter-proof whose failure point is
unique. The `documents.rowid_pk` / `content_rowid` pairing and the `VACUUM` regression
test are correct and the reasoning behind them is sound. I also pulled the vendored
rmcp 2.2 source to check a suspicion that the SDK's port-less `allowed_origins`
entries would reject the app's real ported Origin — they don't
(`origin_is_allowed` treats `a_port.is_none()` as "any port"), so that is a
non-finding and the defence-in-depth layering is correctly configured.

What the phase did not get right clusters in three places.

**Secrets can still reach disk by a route the guard does not cover.** `settings` is
protected against secret-shaped *key names* but not against secret-shaped *values*:
`validate_base_url` happily accepts `https://user:sk-…@host/v1` and persists the
credential in the SQLite file that `docs/keychain-naming.md` explicitly says gets
carried away by whole-directory backups. The guard was built as "机制 not 约定" and
then left with a hole exactly the size of the most common way a user pastes an
OpenAI-compatible endpoint.

**The shell's security posture is weaker than the engine's.** CSP is switched off
entirely, the asset protocol is enabled with no consumer, and four `dev_*` commands —
including one that writes fixture rows into the user's real database — are registered
unconditionally in the release `generate_handler!`. The frontend hides the dev *page*
in production builds; it does not remove the *commands*. Separately, nothing in the
workspace installs a `tracing` subscriber, so every `tracing::warn!` the security
design relies on ("真实原因只进本地 tracing", T-01-29; the agent-receipt audit line,
T-01-33) writes to nowhere.

**One test in the recurring vacuous-assertion class survived.**
`reader_snapshot_is_isolated` asserts `after >= 1` on a counter that starts at 1 and
only ever grows, so it holds for both possible outcomes; and its inline comment claims
a snapshot isolation that `Store::read` does not actually provide (no explicit
transaction is opened, so the second `query_row` runs in autocommit and gets a fresh
snapshot). The test's real content is "the writer is not blocked", which is worth
having — but that is not what its name, comment, or assertion say. Several smaller
assertions in the same family are listed under Warnings.

Findings below are ordered by severity, not by file.

## Critical Issues

### CR-01: `validate_base_url` lets a credential-bearing URL into the settings table

**File:** `crates/prism-store/src/settings.rs:48-71` (guard), `crates/prism-store/src/settings.rs:88-104` (write path)

**Issue:** `set_setting` enforces two rules — the key must not *look* like a secret,
and `llm.base_url` must parse as an http/https URL with a non-empty host. Neither rule
inspects the URL's userinfo component. `Url::parse("https://user:sk-abc123@api.vendor.com/v1")`
yields scheme `https` and `host_str() == Some("api.vendor.com")`, so it passes both
checks and the full string — password included — is written verbatim into
`settings.value`.

This defeats the invariant the module header and `docs/keychain-naming.md:41-45` both
state as absolute: *密钥的唯一存放地是系统钥匙串；不进 SQLite（含 settings 表）*.
The stated reason that invariant exists is that the sidecar directory is backed up as
a unit, so one backup exfiltrates the credential — which is exactly what happens here.
Embedding the key in the base URL is not exotic: several OpenAI-compatible proxies and
gateways document precisely that form, and the Settings page's own front-end guard
(`looksLikeHttpUrl`, `src/pages/Settings.tsx:17-20`) accepts it too.

`is_secret_like_key` was deliberately written "宽进严出" for key names. The same
posture is missing on the value side, where the consequence is worse.

**Fix:** reject userinfo in `validate_base_url` (and keep the message value-free, per
T-01-26):

```rust
if !url.username().is_empty() || url.password().is_some() {
    return Err(StoreError::InvalidUrl(
        "must not embed credentials; store the API key in the system keychain".into(),
    ));
}
```

Add the negative control to `settings_base_url_validation`, and a matching
`invalid_url` copy path is already present in `ERROR_COPY`. Consider also rejecting a
non-empty `url.query()` / fragment on the same grounds — some gateways accept
`?api-key=…`.

---

### CR-02: CSP is disabled and the asset protocol is enabled with no consumer

**File:** `src-tauri/tauri.conf.json:20-26`

**Issue:**

```json
"security": {
  "csp": null,
  "assetProtocol": { "enable": true, "scope": [] }
}
```

`"csp": null` means Tauri injects no Content-Security-Policy at all — the WebView will
load and execute script from any origin, and inline script is unrestricted. That is
the one control that turns "a string got into the DOM" from a full compromise into a
rendering bug. This project's whole point is to render Markdown that an external
coding agent wrote (Phase 3+), and Phase 6 puts an LLM in the loop; by then the
absence of a CSP is not a hardening gap but the primary exploit path. The IPC surface
that a successful injection would inherit is not small — see WR-07 — and the shipping
target is already live (`bundle.active: true`, `targets: "dmg"`).

`assetProtocol.enable: true` compounds it: the `protocol-asset` Cargo feature is also
enabled (`src-tauri/Cargo.toml:21`), yet nothing in the Phase 1 frontend calls
`convertFileSrc` and the capability file grants no `asset:allow-read`. It is declared
attack surface with zero consumers — the opposite of the least-privilege posture the
capability file itself takes.

**Fix:** set a restrictive policy now, while the frontend has no external assets and
the cost is zero:

```json
"security": {
  "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: data:; connect-src 'self' ipc: http://ipc.localhost",
  "assetProtocol": { "enable": false, "scope": [] }
}
```

and drop `"protocol-asset"` from the `tauri` feature list until something actually
needs it. Re-enable both with a real `scope` in the phase that introduces local image
rendering. Pin the policy with a test in the same spirit as `capabilities.test.ts` so
it cannot silently regress to `null`.

---

### CR-03: the bearer gate fails open on an empty token, and a unit test codifies it

**File:** `crates/prism-mcp/src/deps.rs:30-40`, `crates/prism-mcp/src/middleware.rs:143-162`, `crates/prism-mcp/src/middleware.rs:208`

**Issue:** `McpDeps::new` accepts `impl Into<Arc<str>>` with no validation, so
`McpDeps::new(feedback, comments, "")` constructs successfully. With an empty expected
token, `constant_time_eq("", "")` returns `true` — and the test suite asserts that
this is the intended behaviour (`middleware.rs:208`). The consequence is that a
misconfigured or not-yet-provisioned install turns the third gate into "send
`Authorization: Bearer ` with an empty token and you are through". Nothing anywhere
errors, warns, or refuses to start.

Phase 6 is where the CSPRNG token gets generated and read from the keychain
(`deps.rs:12-15`), and that is precisely the code path where an empty value is
plausible: a keychain read that returns an empty string, a first-run race before the
token is written, a `unwrap_or_default()` at the call site. A security gate whose
degraded mode is "allow everyone" and whose degraded mode is silent is a fail-open
gate, regardless of how carefully the comparison itself is written.

The rest of this middleware is defensive to a fault (constant-time comparison, a
source-level sentinel forbidding `==`, uniform 403s). This one hole is out of
character with all of it.

**Fix:** make the constructor fallible, or refuse at the comparison:

```rust
impl McpDeps {
    pub fn new(
        feedback: Arc<dyn FeedbackSource>,
        comments: Arc<dyn CommentSink>,
        bearer: impl Into<Arc<str>>,
    ) -> Result<Self, McpError> {
        let bearer: Arc<str> = bearer.into();
        if bearer.len() < MIN_BEARER_LEN {   // 32 hex chars minimum
            return Err(McpError::WeakBearer);
        }
        Ok(Self { feedback, comments, bearer })
    }
}
```

and change `constant_time_eq` to return `false` unconditionally when `expected` is
empty, replacing the `constant_time_eq("", "")` assertion with its inverse. Both
changes are cheap now and expensive after Phase 6 wires the real token.

## Warnings

### WR-01: `reader_snapshot_is_isolated` asserts a tautology and documents behaviour that does not exist

**File:** `crates/prism-store/tests/concurrency.rs:43-68`

**Issue:** Two problems in one test.

The assertion `assert!(after >= 1, "读者不应因并发写而丢失可见行")` (line 68) cannot
fail. `before` was already asserted to be 1, rows are only ever inserted, so `after` is
1 or 2 and both satisfy `>= 1`. It is the "assertion that holds regardless of whether
the code works" pattern verbatim.

The comment it is guarding (line 49, *「写已经提交了；同一个读连接仍在自己的事务快照里」*)
is also wrong. `Store::read` (`open.rs:131-137`) hands out a pooled `Connection` and
calls the closure; it never opens an explicit transaction. Each `query_row` therefore
runs in autocommit and acquires a fresh read snapshot, so the second read observes the
committed row. The test's name promises snapshot isolation, its comment asserts
snapshot isolation, and the code under test does not implement it — the weakened
assertion is what keeps all three from colliding.

What the test genuinely proves is valuable and unique: the writer commits while a
reader holds a pooled connection, without `SQLITE_BUSY`. That deserves to be the
test's stated purpose.

**Fix:** rename to `writer_is_not_blocked_by_an_open_reader`, delete the misleading
comment, and replace the tautology with the fact that actually holds:

```rust
assert_eq!(after, 2, "autocommit read should observe the committed row");
```

If snapshot isolation across a `read()` closure is a property the design wants, it has
to be implemented first (`conn.unchecked_transaction()` with
`TransactionBehavior::Deferred` held for the closure's duration) and only then
asserted.

---

### WR-02: `PRAGMA journal_mode=WAL` result is discarded — a silent fallback to rollback journal

**File:** `crates/prism-store/src/open.rs:53-59`

**Issue:** `journal_mode` is the one pragma in the six-step sequence that can *fail
without erroring*. SQLite returns the resulting mode as a result row; if WAL cannot be
enabled (database on a network filesystem, a `-shm` that cannot be created, a
read-only directory), it returns `delete` and `execute_batch` reports success because
no SQL error occurred.

Every concurrency property this crate is built around — readers not blocking the
writer, the read pool being useful at all, `close()`'s TRUNCATE checkpoint having
anything to truncate — silently degrades to rollback-journal semantics, which surfaces
in Phase 2+ as intermittent `SQLITE_BUSY` under the 5s `busy_timeout`. That is the
hardest possible failure to trace back to here. The module header calls
`journal_mode` a "持久设置，只需设一次"; that is exactly why it needs to be verified
once.

**Fix:** set it with `query_row` and check the answer before the rest of the batch:

```rust
let mode: String = writer.query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))?;
if !mode.eq_ignore_ascii_case("wal") {
    return Err(StoreError::JournalModeNotWal(mode)); // mode string only, no path (T-01-20)
}
writer.execute_batch(&format!(
    "PRAGMA synchronous=NORMAL; PRAGMA busy_timeout={BUSY_TIMEOUT_MS}; PRAGMA foreign_keys=ON;"
))?;
```

---

### WR-03: `close()` discards the checkpoint's busy flag

**File:** `crates/prism-store/src/open.rs:147-153`

**Issue:** The doc comment identifies the exact failure mode — *"TRUNCATE checkpoint
在还有其他连接开着时会「成功但没做事」（返回 busy 标志而不是报错），那正是这类 bug
静默的地方"* — and then the code reads the result with `|_| Ok(())`, throwing away the
`busy` column it just described. Dropping the pool first makes the busy case unlikely,
not impossible: an outstanding `PooledConnection` on another thread, or a reader still
inside a `read()` closure at shutdown, reproduces it. The stated consequence (a backup
that copies the main database while WAL content is still uncheckpointed) is a data
integrity issue, and the only test coverage is `wal_truncated_on_close`, which runs
with no concurrent readers.

**Fix:** read column 0 and surface it:

```rust
let busy: i64 = conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |r| r.get(0))?;
if busy != 0 {
    return Err(StoreError::CheckpointBusy);
}
```

---

### WR-04: no `tracing` subscriber is ever installed — every log call in the workspace is a no-op

**File:** `src-tauri/src/lib.rs:30-62` (no init), `crates/prism-mcp/src/middleware.rs:37`, `crates/prism-engine/src/services.rs:57-61`, `crates/prism-store/src/settings.rs:64-67`, `src-tauri/src/bus_adapter.rs:66-68`

**Issue:** `tracing` is a dependency of six crates and `tracing-subscriber` is a
dependency of none — `grep -rn "tracing_subscriber" crates src-tauri Cargo.toml`
returns nothing, and neither `main.rs` nor `run()` installs a global default. Every
`tracing::warn!` / `info!` / `trace!` in the workspace is discarded at runtime.

That is not merely missing observability; three deliberate security decisions are
built on top of a sink that does not exist:

- `middleware.rs:36-39` — the uniform 403 is justified by *"真实原因只进本地 tracing，
  不进响应"* (T-01-29). The real reason goes nowhere. An operator debugging a rejected
  agent has no signal at all, which makes the uniform-403 design much more expensive
  to live with than intended.
- `services.rs:57-61` — the agent-receipt audit line (T-01-33) is the only record that
  an external agent acted on a comment. It is not recorded.
- `settings.rs:64-67` — the plaintext-http-to-a-non-loopback-host warning, described as
  "只警告不阻断", neither warns nor blocks.
- `lib.rs:40-42` — "keychain backend unavailable; secrets are disabled" is the sole
  notification for a startup degradation the user is otherwise never told about.

**Fix:** add `tracing-subscriber` to `src-tauri` and initialise it as the first
statement of `run()`, before `tauri::Builder`:

```rust
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info,prism_mcp=debug".into()),
    )
    .init();
```

Keep it in the shell only, so the engine crates stay subscriber-agnostic and testable.
Note that `prism-mcp`'s deny reasons will then be written to a local log — confirm
that is intended before enabling `debug` for that target by default.

---

### WR-05: `errorCopy` resolves prototype-chain members and can return a function typed as `string`

**File:** `src/lib/ipc.ts:41-68`

**Issue:** `ERROR_COPY` is an object literal, so it inherits from `Object.prototype`,
and the lookup uses `??` — which only falls back on `null`/`undefined`:

```ts
return ERROR_COPY[code] ?? "操作失败，请重试。";
```

`errorCopy("toString")`, `errorCopy("constructor")`, `errorCopy("valueOf")`,
`errorCopy("hasOwnProperty")` all return a *function*, not the fallback string. The
declared return type is `string`, so TypeScript will not catch it; the value flows
into `setKeyNotice({ text })` / `setNotice(...)` and then into JSX, where React
rejects a function child.

Today's error codes are a closed set produced by `map_err`, so this is latent rather
than live. But `errorCopy` is also invoked on arbitrary caught values in
`DevSmoke.tsx:110/125/140/149` and on mutation rejections in `Settings.tsx`, and the
whole point of the function's doc comment is that unknown input must land on the
generic fallback rather than reaching the DOM. On these specific inputs it does the
opposite.

**Fix:** use a prototype-free container or an own-property check:

```ts
const ERROR_COPY: Record<string, string> = Object.assign(Object.create(null), {
  invalid_url: "…",
  // …
});
```

or

```ts
export function errorCopy(err: unknown): string {
  const code = typeof err === "string" ? err : "";
  return Object.hasOwn(ERROR_COPY, code) ? ERROR_COPY[code] : "操作失败，请重试。";
}
```

Add `expect(errorCopy("toString")).toBe("操作失败，请重试。")` — it fails today.

---

### WR-06: a failed status query renders as "未配置", making a keychain outage look like an empty keychain

**File:** `src/pages/Settings.tsx:29-33`, `src/pages/Settings.tsx:100-102`, `src/pages/Settings.tsx:142`

**Issue:** Neither `useQuery` result handles the error state:

```tsx
{keyStatus.isPending ? "读取中…" : keyStatus.data ? "已配置" : "未配置"}
```

In TanStack Query v5, a rejected query settles with `isPending === false` and
`data === undefined`, so after the default three retries the page states positively
that no key is configured — when in fact the keychain was locked, permission was
denied, or the IPC call failed. `baseUrl.data ?? "（未设置）"` (line 142) has the same
shape: a failed read is presented as "not set".

This is the same class of defect the phase already fixed on the `listen()` path
(`useEngineInvalidation.ts:15-19`: *"这一类失败不能被丢进未处理的 Promise"*, and
`DevSmoke.test.tsx:148-161`'s "计数为 0 不足以作断言：正常状态下它也是 0"). The
reasoning transfers exactly — "未配置" is indistinguishable from "read failed" — but
the fix was not applied here. A user who sees "未配置" will re-enter their key, which
then also fails to save, with no indication why.

The test file mirrors the gap: `Settings.test.tsx` covers `mockResolvedValue(false)`
and `mockResolvedValue(true)` but never `mockRejectedValue`.

**Fix:**

```tsx
{keyStatus.isPending
  ? "读取中…"
  : keyStatus.isError
    ? "读取失败"
    : keyStatus.data
      ? "已配置"
      : "未配置"}
```

with an accompanying `role="alert"` line carrying `errorCopy(keyStatus.error)`, the
same treatment for `baseUrl`, and a test that asserts a rejected `apiKeyStatus` does
**not** render "未配置".

---

### WR-07: dev-only commands ship in the release IPC surface, and one of them writes to the user's real database

**File:** `src-tauri/src/lib.rs:48-59`, `crates/prism-engine/src/facade.rs:95-99`

**Issue:** `generate_handler!` registers `dev_ping`, `dev_emit_bus_event`,
`dev_smoke_stream` and `dev_seed_sample_docs` unconditionally. Nothing is gated on
`#[cfg(debug_assertions)]`.

`App.tsx:28` removes the dev *route button* from production builds
(`import.meta.env.DEV` is statically false and the block is shaken out), and
`DevSmoke.tsx:60-63` calls the page "隐藏 dev 冒烟页". Neither removes the *commands*.
Any script executing in the WebView — which, given CR-02, includes remotely loaded
script — can call `invoke("dev_seed_sample_docs")` and write three fixture documents
plus a `smoke-project` row into the user's live `~/Library/Application Support/PrismDocs/prismdocs.db`,
or spam `dev_emit_bus_event` to force UI invalidation storms.

`dev_seed_sample_docs` is the one that actually mutates user data. Its project id is a
hardcoded `"smoke-project"` (`seed.rs:16`), so the rows are indistinguishable from a
real import at the schema level and will still be there in Phase 2.

**Fix:** gate the dev handlers at registration:

```rust
let builder = tauri::Builder::default();
#[cfg(debug_assertions)]
let builder = builder.invoke_handler(tauri::generate_handler![ /* prod + dev */ ]);
#[cfg(not(debug_assertions))]
let builder = builder.invoke_handler(tauri::generate_handler![ /* prod only */ ]);
```

At minimum gate `dev_seed_sample_docs`, which is the only one with a persistent side
effect. `src-tauri/tests/ipc.rs` builds its own handler list and is unaffected either
way.

---

### WR-08: `dev_smoke_stream` takes an unbounded `total` and runs it synchronously on the async runtime

**File:** `src-tauri/src/commands.rs:131-137`

**Issue:** Two departures from this file's own stated discipline, in one four-line
command.

`total: u32` is unvalidated. `smoke::generate` loops `0..total` sending one IPC message
per iteration; `total = u32::MAX` is 4.29 billion messages. The frontend always passes
1000 (`DevSmoke.tsx:20`), but the parameter is attacker-reachable from any script in
the WebView (WR-07, CR-02).

More structurally: `dev_smoke_stream` is `async fn` but calls `smoke::generate`
directly rather than through `delegate`'s `spawn_blocking`. The module header
(`commands.rs:47-51`) explains why that matters — *"一次慢查询会卡住整个 IPC 线程"* —
and this is the one command in the file that does not follow it. Even at `total = 1000`
the whole loop runs inline on the async runtime; at any larger value it blocks the IPC
executor outright. This is not a channel-send-bound loop that yields: `Channel::send`
is synchronous.

**Fix:** clamp and offload:

```rust
const SMOKE_MAX_TOTAL: u32 = 10_000;

#[tauri::command]
pub async fn dev_smoke_stream(
    on_event: tauri::ipc::Channel<SmokeEvent>,
    total: u32,
) -> Result<(), String> {
    let total = total.min(SMOKE_MAX_TOTAL);
    tauri::async_runtime::spawn_blocking(move || {
        smoke::generate(total, |ev| on_event.send(ev))
    })
    .await
    .map_err(|_| ERR_TASK.to_string())?
    .map_err(|_| ERR_CHANNEL.to_string())
}
```

`smoke::collect`'s `Vec::with_capacity(total as usize + 2)` (`smoke.rs:44`) is the
same unbounded value; it is test-only today but will over-allocate if ever reused.

---

### WR-09: the "least privilege" capability assertion is a denylist, so new permissions pass silently

**File:** `src/lib/capabilities.test.ts:26-32`

**Issue:**

```ts
const forbidden = capability.permissions.filter((p) =>
  /^(fs|shell|http|dialog|core:webview|core:window):/.test(p),
);
expect(forbidden).toEqual([]);
```

The comment above it claims to assert 最小权限, but the mechanism is a fixed denylist
of six prefixes. Anything outside those prefixes is admitted without comment —
including `core:event:allow-emit`, which would let frontend script forge
`prism://changed` events straight into the invalidation pipeline, and every
`core:app:*`, `core:path:*`, `core:resources:*`, `core:tray:*`, `core:menu:*` and
third-party `plugin:*` permission that later phases will be tempted to add.

The test is exactly as strong as the imagination of whoever wrote the regex, which is
the failure mode a least-privilege assertion exists to prevent. Since the granted set
is currently two entries and the file's whole purpose is that additions be deliberate,
an exact-equality check costs nothing and is strictly stronger.

**Fix:**

```ts
expect(capability.permissions).toEqual([
  "core:event:allow-listen",
  "core:event:allow-unlisten",
]);
```

Adding a permission then requires editing this line, which is the review checkpoint
the denylist was trying to approximate.

---

### WR-10: the secret scanner misses Anthropic key format and every non-`api_key` spelling

**File:** `scripts/check-secrets.sh:18-23`

**Issue:** The pattern is

```
sk-[A-Za-z0-9]{16,}|api[_-]?key[[:space:]]*=[[:space:]]*"…"
```

Three gaps, the first of which matters most for this project:

1. **`sk-ant-…` does not match.** `sk-` must be followed by ≥16 consecutive
   alphanumerics; an Anthropic key is `sk-ant-api03-…`, which breaks at the hyphen
   after three characters. `CLAUDE.md` names Anthropic's Messages API as a first-class
   endpoint and `Settings.tsx:149` uses `https://api.anthropic.com` as the placeholder,
   so the provider whose keys this project is most likely to leak is the one the
   scanner cannot see.
2. **Only `=` assignment with `api_key`-ish names.** `apiKey: "…"` (TS/JSON — the
   dominant form in this repo's frontend and in `tauri.conf.json`), `token = "…"`,
   `secret = "…"`, `password = "…"`, `Authorization: "Bearer …"` and
   `ANTHROPIC_API_KEY=…` in a committed `.env` all pass.
3. **`docs/` is excluded wholesale.** The exclusion is justified for `.planning/`
   (which quotes the regexes), but `docs/` is ordinary version-controlled prose that a
   future runbook or troubleshooting note could easily paste a real key into.

**Fix:** widen the alternation and narrow the exclusion:

```bash
PATTERN="sk-[A-Za-z0-9_-]{20,}|(api[_-]?key|secret|token|password)[[:space:]]*[=:][[:space:]]*${QUOTE}${NOT_QUOTE}{12,}|ghp_[A-Za-z0-9]{36}|AKIA[0-9A-Z]{16}"
```

Drop `':(exclude)docs/'` and instead exclude the two or three specific documents that
quote patterns. Verify the widened pattern against the existing fixtures
(`FIXTURE_SECRET`, `FAKE_KEY`) — both were chosen not to trip the scanner and should
still not trip it.

---

### WR-11: `check_dup` swallows `cargo tree` failure and passes vacuously

**File:** `scripts/check-deps.sh:34-43`

**Issue:**

```bash
out=$(cargo tree --workspace --duplicates --edges normal || true)
if grep -Eq '^(rusqlite|reqwest|libsqlite3-sys) v' <<<"$out"; then
```

If `cargo tree` fails for any reason — a lock-file conflict, an unresolvable feature
combination, a registry outage, a `cargo` version whose flag surface changed — `|| true`
converts the failure into an empty string, `grep` finds nothing, and the function
prints `OK: no duplicate rusqlite/reqwest/libsqlite3-sys`. The assertion that Success
Criterion 1-b depends on reports success precisely when it has learned nothing. This is
the shell equivalent of the vacuous-assertion class the phase has been hunting.

The five other checks in this file all let `cargo tree` fail loudly under `set -e`,
which is the correct behaviour; only this one opts out, and the comment gives no reason.

**Fix:**

```bash
if ! out=$(cargo tree --workspace --duplicates --edges normal); then
  echo "FAIL: cargo tree --duplicates could not run" >&2
  return 1
fi
```

---

### WR-12: the MCP handler ignores its own schema's `required` field

**File:** `crates/prism-mcp/src/handler.rs:36-53` (schema), `crates/prism-mcp/src/handler.rs:85-91` (extraction)

**Issue:** The tool descriptor declares `"required": ["projectId"]` and a
`"type": "string"`, then the handler drops both:

```rust
let project_id = request.arguments.as_ref()
    .and_then(|args| args.get("projectId"))
    .and_then(|v| v.as_str())
    .unwrap_or_default()
    .to_owned();
```

A call with no `arguments` at all, with `projectId` absent, or with
`projectId: 42` / `projectId: null` is silently coerced to `""` and passed to
`list_feedback`. Today `Engine::list_feedback` happens to reject the empty string
(`services.rs:38-41`) so the outcome is an error rather than a wrong answer — but that
is the *injected implementation's* validation covering for the handler's, and the
handler is documented as the place Phase 6 extends. A future `FeedbackSource` that
treats `""` as "all projects" turns this into a cross-project data leak with no code
change here.

The error the caller receives is also misleading: a *malformed request* is reported as
`internal_error` (line 99) rather than as an invalid-params error.

**Fix:** validate at the boundary, where the schema says the contract is:

```rust
let project_id = request
    .arguments
    .as_ref()
    .and_then(|args| args.get("projectId"))
    .and_then(|v| v.as_str())
    .filter(|s| !s.trim().is_empty())
    .ok_or_else(|| ErrorData::invalid_params("projectId must be a non-empty string", None))?
    .to_owned();
```

Keep the message rule-shaped, not value-shaped (T-01-04). Add a test that
`tools/call` with `arguments: {}` returns invalid-params rather than internal error.

---

### WR-13: agent-supplied `status` is logged verbatim, contradicting the comment above it

**File:** `crates/prism-engine/src/services.rs:53-63`

**Issue:** The doc comment states the rule precisely — *"日志里只有 comment_id 与
status，没有正文（T-01-33）：回执正文来自外部 agent，可能整段引用用户文档"* — and then
logs `status` with no validation:

```rust
tracing::info!(comment_id = %receipt.comment_id, status = %receipt.status, "recorded an agent receipt");
```

`Receipt.status` is a `String` deserialised straight off the MCP wire
(`prism-types/src/dto.rs:19-24`). Only `comment_id` is checked, and only for emptiness.
Nothing constrains `status` to the small enum the field clearly intends
(`applied` / `rejected` / …): an external agent can put a megabyte of document text, or
embedded newlines forging additional log lines, into a field the comment asserts is
safe to log. The reasoning that excluded the receipt body applies verbatim to `status`;
it just was not carried across.

Currently harmless only because WR-04 means the line is never written — which is not a
mitigation to rely on.

**Fix:** constrain the field to its intended domain and reject anything else, keeping
the rejection text rule-shaped:

```rust
const ALLOWED_STATUS: [&str; 3] = ["applied", "rejected", "deferred"];
if !ALLOWED_STATUS.contains(&receipt.status.as_str()) {
    return Err(ServiceError::Invalid("status is not a recognised value".into()));
}
```

Better still, make it an enum in `prism-types` so serde rejects it at the boundary.

---

### WR-14: `accepts_fully_valid_request` passes on a 500

**File:** `crates/prism-mcp/tests/middleware_gate.rs:207-217`

**Issue:** The A-group negative control asserts `!status.is_client_error()`. A 500, a
502, or any other server-side failure satisfies it. The test is titled "三层不是把所有
请求都拒了" and its stated job is to prove a fully valid request reaches the MCP
service — but a request that reaches the service and then explodes is scored as a pass.

`rejects_missing_or_wrong_bearer` (lines 191-203) uses the same loose
`is_client_error()` shape. That one is defensible (the non-disclosure design
deliberately leaves 401-vs-403 unspecified), but the positive control has no such
excuse: it knows exactly what success looks like.

`trait_injection.rs` does assert `is_success()` on the full handshake, so the property
is covered elsewhere — which is an argument for tightening this assertion, not for
leaving it loose.

**Fix:**

```rust
assert!(status.is_success(), "三层头全合法的请求未到达 mcp service: {status}");
```

---

### WR-15: `constant_time_eq`'s XOR fold is unreachable logic in a security primitive

**File:** `crates/prism-mcp/src/middleware.rs:143-162`

**Issue:** The function folds `presented` into an `expected`-length buffer, then ANDs
the byte comparison with a length comparison:

```rust
let same_len = (expected.len() as u64).ct_eq(&(presented.len() as u64));
let same_bytes = expected.ct_eq(padded);
(same_len & same_bytes).into()
```

Because the result requires `same_len`, the only case that can return `true` is
`presented.len() == expected.len()` — and in that case the fold is the identity
function (each slot is XORed exactly once from zero). Every branch the fold exists to
handle is already excluded by `same_len`. The heap allocation, the modulo, and the
`&folded[..0]` sentinel are all dead weight, and the four "fold must not collide" test
cases at lines 201-205 are testing a code path that cannot influence the result.

The behaviour is correct — I traced it — but hand-rolled complexity inside an
authentication comparison is exactly where a future edit introduces a real bug, and
the accompanying comment ("超出部分参与折叠而非被丢弃") describes a security property
the `same_len` gate already provides more simply.

**Fix:** `subtle`'s slice `ct_eq` already returns `Choice(0)` on a length mismatch:

```rust
fn constant_time_eq(expected: &str, presented: &str) -> bool {
    !expected.is_empty() && expected.as_bytes().ct_eq(presented.as_bytes()).into()
}
```

The leading emptiness check also closes CR-03. Keep the existing test cases — they all
still apply, and `constant_time_eq("", "")` flips to `false`, which is what it should
have been.

---

### WR-16: clippy `-D warnings` does not cover `prism-cli` or `prismdocs-shell`, and there is no frontend linter

**File:** `.github/workflows/ci.yml:37-38`, `.github/workflows/ci.yml:85-101`

**Issue:** The clippy step names the eight engine crates explicitly. `prism-cli` and
`prismdocs-shell` are excluded, so `src-tauri/src/{lib,commands,bus_adapter,smoke}.rs`
and `crates/prism-cli/src/main.rs` — the two crates carrying the IPC boundary and the
future `externalBin` — are the only Rust in the repo with no lint gate. The
`prism-cli` package is also absent from both the clippy and the `cargo test` steps; its
tests run only as a side effect of `cargo llvm-cov --no-report --workspace` in the
coverage step, which is an accidental rather than a declared gate.

On the frontend, `npm run build` does run `tsc --noEmit` (verified in `package.json`),
so type checking is gated — but there is no ESLint configuration in the repo at all, so
no rule catches unused variables, missing hook dependencies, floating promises, or
`no-console` in a codebase that deliberately routes every error through `errorCopy`.

**Fix:** add the two crates to the clippy invocation (`-p prism-cli -p prismdocs-shell`;
the latter needs `--features test` to cover `tests/ipc.rs`), add `-p prism-cli` to the
engine test step, and add a minimal `eslint` + `typescript-eslint` +
`eslint-plugin-react-hooks` config with an `npm run lint` step in the frontend job.

## Info

### IN-01: stale counts in `tests/ipc.rs` comments

**File:** `src-tauri/tests/ipc.rs:125-128`, `src-tauri/tests/ipc.rs:138-143`, `src-tauri/tests/ipc.rs:145`

**Issue:** *"不需要钥匙串的六个命令"* precedes a `[&str; 7]`; *"需要钥匙串的两个命令"*
precedes a `[&str; 3]`; *"八个命令全部可经 IPC 到达"* describes a `COMMANDS: [&str; 10]`.
The arrays are right and the prose is stale.

**Fix:** update the three counts, or drop the numerals from the comments so they cannot
drift again.

---

### IN-02: `insert_samples` returns a constant, not a row count

**File:** `crates/prism-store/src/seed.rs:51-92`

**Issue:** The doc comment says *"返回写入的文档条数"*, but the function returns
`SAMPLE_DOCS.len()` regardless of what the statements actually did. It is currently
harmless (`Engine::seed_sample_docs` discards the value), but it is a return value that
cannot report a failure it was asked to report.

**Fix:** accumulate `stmt.execute(...)?` return values, or change the signature to
`Result<(), StoreError>` and let the constant live at the call site.

---

### IN-03: `assert_sqlite_version` silently reindexes malformed version strings

**File:** `crates/prism-store/src/open.rs:93-105`

**Issue:** `version.split('.').filter_map(|s| s.parse().ok())` drops unparsable
components rather than failing, so a version like `3.x.51` collapses to `(3, 51, 0)` —
the dropped element shifts every later component one position left. The same idiom
appears in `lib.rs:39-41`, `concurrency.rs:13-20` and `prism-engine/src/lib.rs:56`.
Contrived for `sqlite_version()`, which is well-formed in practice, but the failure
mode is a wrong comparison rather than an error.

**Fix:** parse positionally and reject a malformed string outright:

```rust
let mut it = version.split('.').map(str::parse::<u32>);
let got = match (it.next(), it.next(), it.next()) {
    (Some(Ok(a)), Some(Ok(b)), Some(Ok(c))) => (a, b, c),
    _ => return Err(StoreError::SqliteTooOld(version)),
};
```

---

### IN-04: `require_local_host` rejects HTTP/2 requests that carry the authority in `:authority`

**File:** `crates/prism-mcp/src/middleware.rs:77-91`

**Issue:** The layer requires a literal `Host` header and denies when it is absent.
Over HTTP/2 the authority arrives in the `:authority` pseudo-header and hyper does not
synthesise a `Host` header — rmcp handles exactly this case
(`tower.rs:397-408`, with a comment noting that `axum::Router::nest` can drop the
synthesised header). Cleartext HTTP/2 requires prior-knowledge negotiation, so no MCP
client in scope will hit it today, but any future h2-capable client is silently 403'd
with no diagnosable reason (see WR-04).

**Fix:** fall back to `request.uri().authority()` when the `Host` header is absent,
matching the SDK's behaviour, before denying.

---

### IN-05: `ServiceError::Backend` has no construction site

**File:** `crates/prism-types/src/service.rs:40-42`

**Issue:** Neither implementation of `FeedbackSource`/`CommentSink` (nor any test)
constructs `Backend`. It is dead today. The enum is `#[non_exhaustive]` and the variant
is clearly reserved for Phase 5/6 storage failures, so this is a note rather than a
defect — but the same reasoning that produced `prismdocs-helper doctor` (giving
`HelperError`'s variants a construction site so `clippy -D warnings` passes) argues for
either using it or deferring it.

**Fix:** leave as-is if Phase 5 lands soon; otherwise remove and reintroduce with its
first real caller.

---

### IN-06: error notices on the smoke page use `role="status"` rather than `role="alert"`

**File:** `src/pages/DevSmoke.tsx:159`

**Issue:** A single `notice` state carries both successes ("样例文档已写入。") and
failures (`errorCopy(err)`, including the listen-rejected copy), and all of them render
in a `role="status"` region. `status` is a polite live region; assistive technology
will not interrupt for it. `Settings.tsx`'s `NoticeLine` gets this right by switching
to `role="alert"` on the error tone, and `DevSmoke.test.tsx:157` asserts against
`findByRole("status")`, so the test encodes the weaker behaviour.

**Fix:** adopt `Settings.tsx`'s `Notice` shape (`{ tone, text }`) on this page and
switch the role accordingly; update the test to `findByRole("alert")` for the
listen-failure case.

---

_Reviewed: 2026-07-29_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
