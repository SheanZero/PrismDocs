# Phase 01: foundation-skeleton — Pattern Map (gap-closure run)

**Mapped:** 2026-07-29
**Mode:** gap_closure — file list derived from `01-VERIFICATION.md` frontmatter `gaps:` + the
`### Anti-Patterns Found` rows CR-02 / CR-03 / WR-04. NOT from CONTEXT/RESEARCH scope.
**Files analyzed:** 9 (6 modified, 3 new/added-to)
**Analogs found:** 6 exact / 1 partial / 2 none

---

## File Classification

| Target file | Change | Role | Data flow | Closest analog | Match |
|---|---|---|---|---|---|
| `crates/prism-store/src/settings.rs` (`validate_base_url` 48-71) | modify | model / validator | request-response (write-path guard) | its own sibling guard `is_secret_like_key` + `set_setting`, same file 35-104 | **exact (in-file)** |
| `crates/prism-store/src/settings.rs` `mod tests` (169-204) | modify | test | — | `settings_rejects_secret_like_keys` 206-234 (has the negative control) | **exact** |
| `src/pages/Settings.tsx` (`looksLikeHttpUrl` 17-20) | modify | component / validator | request-response | same function; error copy from `src/lib/ipc.ts` `ERROR_COPY` 41-50 | **exact** |
| `src/pages/Settings.test.tsx` | add test | test | — | `never echoes the typed key back into the DOM` 72-80 (two-assertion idiom) | **exact** |
| `scripts/check-secrets.sh` (PATTERN 13-23) | modify | config / CI gate | batch scan | `scripts/check-deps.sh` — six named `check_*` fns, `set -euo pipefail` | **role-match** |
| *(new)* self-test fixture proving the scanner trips on `sk-ant-api03-…` | new | test | batch | **NO ANALOG** — see §No Analog Found | — |
| `src-tauri/tauri.conf.json` (`security` 20-26) | modify | config | — | **NO ANALOG** — only config file of its kind | — |
| `crates/prism-mcp/src/deps.rs` (`McpDeps::new` 30-40) + `src/middleware.rs` (143-162, test 206-208) | modify | middleware / model | request-response | `middleware.rs` `deny()` 36-39 + `the_comparison_is_not_a_plain_equality` 211-225 | **exact** |
| `crates/prism-mcp/tests/middleware_gate.rs` (bearer B-group) | modify | test | — | `host_layer_alone_is_what_rejects_a_foreign_host` 249-260 | **exact** |
| *(new)* `tracing-subscriber` init | new | provider / wiring | event-driven | `src-tauri/src/lib.rs` `.setup(|app| …)` 32-47 (the seam) — but **no init analog exists** | **partial** |

---

## Pattern Assignments

### `crates/prism-store/src/settings.rs` — add userinfo/query guard (gap 1, blocker)

**Analog: the guard's own file, `set_setting` 88-104.** The rule that guards live on the
write path, not at call sites, is stated in the doc comment there and must be preserved:

```rust
/// 两道守卫都长在这里而不是调用方：疑似密钥的键名一律拒绝，`base_url` 一律先过 scheme 校验。
/// 放在调用方就等于「每个调用点都记得」，那是约定；放在这里它才是机制。
pub fn set_setting(tx: &Transaction, key: &str, value: &str) -> Result<(), StoreError> {
    if is_secret_like_key(key) { … }
    if key == SETTING_BASE_URL { validate_base_url(value)?; }
```

→ The new check belongs **inside `validate_base_url`**, not in `set_setting`. Mirror the
existing early-return shape at 52-59 exactly:

```rust
    if !ALLOWED_URL_SCHEMES.contains(&url.scheme()) {
        return Err(StoreError::InvalidUrl(format!(
            "scheme must be one of {ALLOWED_URL_SCHEMES:?}"
        )));
    }
    if url.host_str().is_none_or(str::is_empty) {
        return Err(StoreError::InvalidUrl("host must not be empty".into()));
    }
```

**Error-copy convention (T-01-26) — rule-shaped, never echo the value.** Authoritative
statement is the `validate_base_url` doc comment 44-47 and `crates/prism-store/src/error.rs`:

```rust
    /// `base_url` 不是一个可接受的 URL。同样只描述规则，不回显传入的值。
    #[error("invalid url: {0}")]
    InvalidUrl(String),
```

So the new message must read like `"must not contain credentials (user:password@)"` — no
interpolation of `raw`, `url.username()`, or `url.password()`. Note `StoreError` is
`#[non_exhaustive]`; reuse `InvalidUrl`, do not add a variant.

**Test analog — `settings_rejects_secret_like_keys` (206-234)** is the strongest
non-vacuous idiom in this crate: reject-loop + count assertion + explicit negative control.

```rust
        for key in ["llm.api_key", "LLM.API_KEY", "mcp.bearer_token", …] {
            assert!(is_secret_like_key(key), "{key} 应被判定为疑似密钥");
            let rejected = store.write(|tx| set_setting(tx, key, "placeholder-value"));
            assert!(matches!(rejected, Err(StoreError::InvalidSetting(_))), …);
            assert_eq!(count_key(&store, key), 0, "{key} 不得出现在 settings 表中");
        }
        // 阴性对照：守卫若写成「一律拒绝」，上面每一条也都会绿。
        assert!(!is_secret_like_key(SETTING_BASE_URL));
        store.write(|tx| set_setting(tx, SETTING_MODEL, "ordinary-value")).expect(…);
        assert_eq!(count(&store), 1);
```

And from `settings_base_url_validation` (184-198), two more shapes to reuse verbatim:

```rust
        // T-01-26：错误消息只描述规则，不回显传入的 value（它可能就是被误填的密钥）。
        let msg = validate_base_url("javascript:alert(1)").unwrap_err().to_string();
        assert!(!msg.contains("alert"), "错误消息不得回显 value: {msg}");
        …
        let rejected = store.write(|tx| set_setting(tx, SETTING_BASE_URL, "file:///etc/passwd"));
        assert!(matches!(rejected, Err(StoreError::InvalidUrl(_))), …);
        assert_eq!(count(&store), 0, "被拒的 base_url 不得进表");
```

New test must assert all four: (a) credential URL rejected, (b) `count == 0` so nothing was
persisted, (c) the error string contains neither username nor password substring,
(d) negative control — `https://api.example.com/v1` still accepted. Helpers `fixture()`,
`count()`, `count_key()`, `read()` already exist at 111-137; reuse, do not redefine.

---

### `src/pages/Settings.tsx` — tighten `looksLikeHttpUrl` (gap 1, frontend half)

**Analog: itself, 14-20.** The comment establishing this is UX not a security boundary must
survive the change:

```tsx
/// 前端的轻量 scheme 提示。**这不是安全边界**——绕过界面直接 invoke 就没有它。
/// 权威校验长在 engine 的 `set_setting` 写入路径上（01-05）。
function looksLikeHttpUrl(raw: string): boolean {
  const trimmed = raw.trim();
  return trimmed.startsWith("http://") || trimmed.startsWith("https://");
}
```

**Error copy analog — `src/lib/ipc.ts` 38-50.** Codes are stable short strings; user-facing
Chinese copy lives only in `ERROR_COPY`, and the code string never reaches the DOM:

```ts
const ERROR_COPY: Record<string, string> = {
  invalid_url: "链接必须以 http:// 或 https:// 开头，并带有主机名。",
  invalid_setting: "这个配置项不被接受（疑似密钥的键名一律不入库）。",
  …
};
export function errorCopy(err: unknown): string {
  const code = typeof err === "string" ? err : "";
  return ERROR_COPY[code] ?? "操作失败，请重试。";
}
```

The call site is `submitUrl()` (Settings.tsx 80-87): `errorCopy("invalid_url")`. If the copy
should distinguish "contains credentials", add a **new key** to `ERROR_COPY` (e.g.
`invalid_url_credentials`) — do not inline a literal in the component; no component in this
repo carries its own error text. Copy must stay rule-shaped (no echo of `urlDraft`).

---

### `src/pages/Settings.test.tsx` — frontend guard test

**Analog: 72-80**, whose comment states the two-assertion rule this new test must follow:

```tsx
  // T-01-04c。两条断言缺一不可：只有「页面上没有密钥」时，一个**根本没提交表单**的
  // 组件也会通过；配上「setApiKey 确实收到了这个密钥」才说明这条路真的走过。
```

Applied here: assert (a) `role="alert"` copy appears **and** (b) `setBaseUrl` was **not**
called — plus a negative control that a clean `https://api.example.com` **does** call
`setBaseUrl`. Mock/reset scaffolding at 6-55 is already in place (`vi.mock("../lib/ipc")`
with `importOriginal` spread, `afterEach(cleanup)`, per-test `mockReset()` loop) — extend
the existing `describe`, do not build a second harness.

**Fixture-naming convention (line 27-29)** governs any credential-looking string added:

```tsx
/// 测试用的假密钥。刻意**不**长得像真密钥前缀，以免 check-secrets.sh 把它当成
/// 提交进仓库的明文密钥（那个扫描器不该为了迁就 fixture 而放宽）。
const FAKE_KEY = "fixture-not-a-real-credential";
```

⚠️ Planner conflict to resolve: the userinfo test *needs* a `sk-`-shaped string, which is
exactly what this convention forbids. Same tension as the Rust side —
`crates/prism-llm/src/secrets.rs:103-106`:

```rust
    /// 测试用的假密钥。刻意不用 `sk-` 开头的长串，避免与
    const FIXTURE_SECRET: &str = "prism-test-secret-value";
```

Resolution the planner must choose explicitly: either the URL fixture uses a non-`sk-`
password (`https://u:prism-test-secret-value@host/v1` — still exercises the guard, keeps the
scanner clean), or the widened scanner grows a scoped allowlist. Prefer the former; the
scanner allowlist is reserved for the scanner's own self-test fixture below.

---

### `scripts/check-secrets.sh` — widen PATTERN (gap 2, blocker)

**Analog: `scripts/check-deps.sh`.** Structure to mirror:

- Shebang + `set -euo pipefail` (line 12), long header comment stating *why* the assertion
  exists and that this file is the single implementation, callers are `justfile` / CI only.
- Named `check_*` functions, one per assertion, each printing `FAIL: …` to **stderr** and
  `return 1`; success prints an `OK: …` line. `main()` dispatches on `${1:-all}` with a
  `usage:` fallback that `exit 2`.

```bash
check_no_cycle() {
  local out body
  out=$(cargo tree -p prism-mcp --edges normal --prefix none)
  …
  if grep -q '^prism-engine' <<<"$body"; then
    echo "FAIL: prism-mcp depends on prism-engine" >&2
    return 1
  fi
  echo "OK: prism-mcp -> prism-types only"
}
```

`check-secrets.sh` today has no functions and no OK line; adopting this shape lets the
widened scan and a new self-test be separate subcommands.

**Critical: WR-11 is the anti-pattern to avoid, and it lives in `check-secrets.sh` too.**
`check-deps.sh:36` `cargo tree … || true` was flagged because a swallowed failure makes the
subsequent grep see an empty string and report OK. `check-secrets.sh:20-23` has the identical
`|| true` on `git grep` — but there it is **load-bearing** (git grep exits 1 on no-match).
Keep it, and pair it with the self-test so a broken invocation cannot masquerade as clean.

**Current pattern, and the exclusions to preserve** (13-23):

```bash
QUOTE="[\"']"
NOT_QUOTE="[^\"']"
PATTERN="sk-[A-Za-z0-9]{16,}|api[_-]?key[[:space:]]*=[[:space:]]*${QUOTE}${NOT_QUOTE}{8,}"

hits=$(git grep -nE "$PATTERN" -- \
  ':(exclude).planning/' \
  ':(exclude)docs/' \
  ':(exclude)scripts/check-secrets.sh' || true)
```

The split-string trick (`QUOTE`/`NOT_QUOTE`, comment at line 13) exists so the script does not
itself contain a literal that trips the scan — retain it for any new sub-pattern. When
narrowing `':(exclude)docs/'`, verify against the two known allowed occurrences that must
remain non-matching: `crates/prism-llm/src/secrets.rs:106` `FIXTURE_SECRET` and
`src/pages/Settings.test.tsx:29` `FAKE_KEY` (both deliberately avoid `sk-`), plus the
`docs/keychain-naming.md` prose. `.github/workflows/ci.yml:35-36` and `justfile:24-25,30`
already invoke the script — if a new subcommand is added, both call sites need updating.

---

### `crates/prism-mcp/src/deps.rs` + `src/middleware.rs` — fail closed on empty bearer (CR-03)

**Where the fail-open is pinned** (`middleware.rs:206-208`, inside
`constant_time_eq_agrees_with_equality_on_every_shape`):

```rust
        assert!(!constant_time_eq(token, ""));
        assert!(!constant_time_eq("", token));
        assert!(constant_time_eq("", ""));   // ← this line pins fail-open as expected
```

Combined with `require_bearer` (132-135), an empty configured token accepts an empty
presented token. The fix should reject at construction (`McpDeps::new`, deps.rs 30-40) so
`constant_time_eq` keeps its narrow, source-guarded contract.

**Source-level guard idiom to extend** (`middleware.rs:211-225`) — this repo already asserts
on its own source text to stop a future regression:

```rust
    /// 源码层面的守卫：这一层永远不能退回 `==`。
    #[test]
    fn the_comparison_is_not_a_plain_equality() {
        let source = include_str!("middleware.rs");
        let body = source.split("fn constant_time_eq").nth(1).expect(…);
        …
        assert!(body.contains("ct_eq"), "常数时间比较被换掉了");
        assert!(!body.contains("expected == presented"), "退回了短路比较 `==`");
    }
```

**Redaction contract that constrains the fix** — `deps.rs:51-62`. Any new error/panic path in
`McpDeps::new` must not include the token:

```rust
/// 手写 `Debug`：token 绝不能经 `?deps` / `{:?}` 进日志或错误文本（T-01-29 同源要求）。
impl std::fmt::Debug for McpDeps { … .field("bearer", &"<redacted>") … }
```

**Deny shape (uniform 403, reason only to tracing)** — `middleware.rs:35-39`, unchanged:

```rust
fn deny(reason: &'static str) -> Response {
    tracing::warn!(reason, "rejected an MCP request at the loopback gate");
    StatusCode::FORBIDDEN.into_response()
}
```

⚠️ `McpDeps::new` currently returns `Self` and is called in three test files. Whichever shape
the planner picks (`Result`, `debug_assert`, or a private newtype), it must keep
`StreamableHttpService`'s `'static` factory closure workable — the reason for `Arc<dyn …>`
over generics is documented at deps.rs 3-8 and is load-bearing.

---

### `crates/prism-mcp/tests/middleware_gate.rs` — fail-closed assertion

**Analog: the B-group isolation idiom, 249-260.** Copy this exact shape (assert the guarded
router denies, then remove the layer and prove the same request reaches the sentinel — so the
failure locus is unique):

```rust
#[tokio::test]
async fn host_layer_alone_is_what_rejects_a_foreign_host() {
    let guarded = sentinel_router().layer(from_fn(require_local_host));
    let (status, body) = oneshot(guarded, request("evil.example.com", None, None)).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "require_local_host 未拦下外域 Host");
    assert!(!body.contains(SENTINEL), "请求仍到达了 handler: {body}");

    // 反证（落点唯一）：摘掉这一层，同一请求直达 sentinel。
    let bare = sentinel_router();
    let (status, body) = oneshot(bare, request("evil.example.com", None, None)).await;
    assert_eq!(status, StatusCode::OK);
```

The file header (1-18) explains *why* B-group exists (rmcp's own Host check would make an
A-group negative control vacuous) — the new empty-bearer test belongs in **B group**, built
on `sentinel_router()` / `request()` / `oneshot()` (helpers at 221-247), with
`McpDeps::new(…, "")`. Token constants and their rationale are at 36-40:

```rust
const GOOD_BEARER: &str = "0123…";
/// 与 `GOOD_BEARER` **等长**、仅末位不同：比较不得因长度短路而放行。
const WRONG_SAME_LEN: &str = "…dee";
```

Note WR-14 (`assert!(!status.is_client_error())` at 205-217 as the positive control) is a
*separate* listed warning, not in this gap set — leave it unless the planner scopes it in.

---

### `tracing-subscriber` init (WR-04) — **partial analog only**

**Emitters already in place** (5 crates, 7 call sites) — all currently write to a null sink:

| Site | Level | What it guards |
|---|---|---|
| `crates/prism-store/src/settings.rs:64` | `warn!` | plaintext-http endpoint notice |
| `crates/prism-mcp/src/middleware.rs:37` | `warn!` | uniform-403's real reason (T-01-29) |
| `crates/prism-mcp/src/server.rs:86` | `warn!` | loopback server stopped |
| `crates/prism-engine/src/services.rs:57` | `info!` | agent receipt audit (T-01-33) |
| `crates/prism-engine/src/bus.rs:53` | `trace!` | publish with no subscribers |
| `src-tauri/src/bus_adapter.rs:67` | `warn!` | emit failure |
| `src-tauri/src/lib.rs:41` | `warn!` | **keychain unavailable startup degradation** |

`tracing = "0.1"` is declared once in the root `Cargo.toml:53` `[workspace.dependencies]` and
consumed as `tracing = { workspace = true }` by prism-store / prism-mcp / prism-engine /
src-tauri. A `tracing-subscriber` dep must follow the same two-step (root pin + `workspace = true`).

**Wiring seam — `src-tauri/src/lib.rs:30-47`.** The subscriber init goes at the top of `run()`
or first inside `.setup()`, ahead of `AppState::bootstrap()` (which itself can fail and
currently logs nothing):

```rust
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            use tauri::Manager;
            let state = AppState::bootstrap()?;

            // 钥匙串后端注册失败**不阻断启动**（D-06：无 key 时应用照常启动）。
            #[cfg(target_os = "macos")]
            if let Err(err) = state.engine.init_secrets() {
                tracing::warn!(error = %err, "keychain backend unavailable; secrets are disabled");
            }
```

**Constraint:** `tracing-subscriber` must be a dependency of **`src-tauri` only**, never of an
engine crate — `scripts/check-deps.sh` `check_tauri_free` / `check_single_egress` cover the
egress crate list, not this one, so nothing will catch a misplacement automatically. D-01
(engine testable without the shell) means engine crates emit but never initialize.

**No analog for the init call itself** — zero `tracing_subscriber::` occurrences workspace-wide.
The planner must source this from RESEARCH/upstream docs (`EnvFilter` + `fmt` layer), not from
the repo.

---

## Shared Patterns

### Rule-shaped errors that never echo the value (T-01-26)
**Source:** `crates/prism-store/src/error.rs` (whole file) + `settings.rs:44-47`
**Apply to:** the settings userinfo guard, any new `McpDeps::new` error, the frontend copy.
Errors carry category + rule only; `StoreError` is `#[non_exhaustive]`; `Display` must not
contain paths, user content, or the rejected value.

### Non-vacuous test discipline (this phase's signature strength)
**Sources:** `crates/prism-store/tests/fts_cjk.rs:1-8` header; `settings_rejects_secret_like_keys`;
`middleware_gate.rs:1-18` B-group rationale; `pooled_connection_cannot_write` (`expect_err`).
**Apply to:** every new test in this gap-closure run. Rule from `fts_cjk.rs`:

```
//! 这组用例的价值不在覆盖率，而在**每一条被删掉之后都会有一个具体的静默失败模式重新变得可能**
```

Concretely: each new guard test pairs a rejection assertion with (a) a persistence/reachability
assertion proving nothing slipped through, and (b) a negative control proving the guard is not
"reject everything".

### Secret redaction in `Debug`, no `Display`
**Sources:** `crates/prism-mcp/src/deps.rs:51-62`; `prism_llm::secrets::ApiKey` (same idiom,
hand-written `Debug`, deliberately no `Display` so `format!("{x}")` is a compile error).
**Apply to:** anything touching the bearer token in CR-03.

### CI/justfile dual call sites for every script
**Sources:** `.github/workflows/ci.yml:31-36`; `justfile:8-30`.
Any new `check-secrets.sh` subcommand must be added to **both**, or it is a convention, not a gate.

---

## No Analog Found

| File / need | Role | Reason |
|---|---|---|
| Self-test fixture proving `check-secrets.sh` trips on `sk-ant-api03-…` | test (shell) | **No script in this repo tests itself.** `check-deps.sh` was proven non-vacuous only by a human doing a manual reqwest injection (VERIFICATION §SC-1). The planner must invent the mechanism — e.g. a `selftest` subcommand that pipes known-bad and known-good samples through `grep -E "$PATTERN"` in-process, avoiding a committed bad file (which would make the main scan red forever). |
| `tracing-subscriber` initialization | provider | Zero occurrences workspace-wide. Only the *seam* (`src-tauri/src/lib.rs` `.setup()`) and the *dep-declaration convention* (root `[workspace.dependencies]`) exist. |
| `src-tauri/tauri.conf.json` CSP + assetProtocol policy (CR-02) | config | Single config file of its kind; `capabilities/` (see `src/lib/capabilities.test.ts:26-32`, itself flagged as WR-09 for being a denylist) is the nearest neighbour but is a different mechanism. No in-repo precedent for a CSP string. |

---

## Metadata

**Search scope:** `crates/`, `src/`, `src-tauri/`, `scripts/`, `.github/workflows/`, `justfile`, root `Cargo.toml`
**Files read:** 14
**Date:** 2026-07-29
