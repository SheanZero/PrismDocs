---
phase: 01-foundation-skeleton
plan: 17
subsystem: mcp-gate-contract
tags: [security, mcp, http-contract, input-validation, defense-in-depth, gap-closure]
status: complete

requires:
  - "crates/prism-mcp/src/middleware.rs（01-06 的三层门禁 + 01-12 / 01-16 的 bearer 加固）"
  - "crates/prism-mcp/src/handler.rs（01-06 的最小 handler）"
  - "01-REVIEW.md WR-03；01-REVIEW-prior.md WR-14 / WR-12 / IN-04"
provides:
  - "「三层一律 403 空正文」在**实发路由**上为真，且状态码/正文回归会变红"
  - "host_of 与 rmcp SDK 内层解析器的关系已定案：两侧必须对同一个主机名达成一致，否则拒"
  - "require_local_host 支持 HTTP/2 的 :authority 回退，与 SDK 行为一致"
  - "PrismHandler 执行自己 schema 声明的 projectId 契约，执行力独立于被注入实现"
affects:
  - "Phase 6：真实 FeedbackSource 即便把空 project id 当作「所有项目」，也够不到 handler 这一层"
  - "Phase 6：h2 MCP 客户端不再拿到无从诊断的 403"
  - "01-REVIEW.md WR-03 第 2 点的举例需更正（见下「事实核对」）"

tech-stack:
  added: []
  patterns:
    - "两层各自解析同一个头部时，安全性质不是「哪层更严」而是「两层是否同意」——只用『内层能否解析』做闸门挡不住『两层解析出不同值』"
    - "评审给出的可达性举例必须自己复跑一遍：本 plan 三个待验证事实里有一个（127.0.0.1:notanumber）实测不成立，真正可达的是另外两个形态"
    - "`is_client_error()` / `!is_client_error()` 这类宽松断言是「知道成功长什么样却不写下来」的形状：它们在本该抓住的回归上保持绿色"
    - "schema 声明的契约必须在提取处执行；靠被注入实现碰巧也做同样校验 = 把执行力外包给不受本文件控制的代码"
    - "上游校验一旦覆盖下游校验的输入集合，下游那条校验就失去端到端哨兵——判别点必须跟着搬家，不能留在已被截断的路径上"

key-files:
  created: []
  modified:
    - crates/prism-mcp/src/middleware.rs
    - crates/prism-mcp/tests/middleware_gate.rs
    - crates/prism-mcp/src/handler.rs
    - crates/prism-mcp/tests/trait_injection.rs
    - crates/prism-engine/tests/facade.rs

decisions:
  - "host_of 走路径 A，但形态是「一致性闸门」而非计划设想的「严格性闸门」：要求本层与 Authority::try_from 算出同一个主机名。只用『SDK 能否解析』挡不住 127.0.0.1:80@evil.com（两侧都解析成功、主机名不同）"
  - "host_of 不改用 SDK 的 Authority::host()：那会把 userinfo 静默丢掉，本层反而开始放行 evil.example.com@127.0.0.1"
  - "project_id_of 返回原值而非 trim 后的值：判空用 trim，取值不动（project id 是标识符，静默改写会让传入与查出不是同一个东西）"
  - "facade.rs 的注入判别点从「MCP 响应里出现 engine 校验文本」搬到「对同一个 Arc<dyn FeedbackSource> 直接调用」——handler 的校验集合覆盖了 engine 的，那条路径已不可达"

metrics:
  duration: ~55min
  tasks: 3
  files: 5
completed: 2026-07-29
---

# Phase 01 Plan 17: 无差别 403 契约落到实发路由 + handler 执行自己的 schema Summary

把三条「正确的守卫被不判别的断言保护着」收口：`host_of` 现在要求与 rmcp SDK 内层就同一个主机名达成一致（SDK 那两种带正文的拒绝因此不可达），A 组断言由 `is_client_error()` 收紧为「403 且空正文」并新增一条打在 `serve_loopback` 上的逐字节相等测试，`PrismHandler` 自己执行 `"required": ["projectId"]` 与 `"type": "string"`。

## rmcp 2.2 SDK 响应形态的事实核对

核对对象：`~/.cargo/registry/src/index.crates.io-*/rmcp-2.2.0/src/transport/streamable_http_server/tower.rs`（本机依赖树里的实际源码，非文档）。

| 位置 | 函数 | 实际行为 |
|---|---|---|
| 263-268 | `forbidden_response` | `StatusCode::FORBIDDEN` + `Full::new(Bytes::from(message))` —— **403 带正文** |
| 372-380 | `bad_request_response` | `StatusCode::BAD_REQUEST` + `Content-Type: text/plain; charset=utf-8` + 正文 —— **400 带正文** |
| 393、400 | `parse_host_header` | `http::uri::Authority::try_from(host_str)`，失败 → `bad_request_response("Bad Request: Invalid Host header")` |
| 405-408 | `parse_host_header` | 无 `Host` 头时回退 `uri.authority()`；仍无 → `bad_request_response("Bad Request: missing Host header")`。注释原文点名 `axum::Router::nest` 可能丢掉 hyper 合成的 `Host` |
| 423 | `validate_dns_rebinding_headers` | `forbidden_response("Forbidden: Host header is not allowed")` |
| 450、457 | `validate_origin_header` | `bad_request_response("Bad Request: Invalid Origin header")` / `forbidden_response("Forbidden: Origin header is not allowed")` |
| 270-274 | `normalize_host` | `host.trim_matches('[').trim_matches(']').to_ascii_lowercase()` |
| 117-118 | `Default` | `allowed_hosts = [localhost, 127.0.0.1, ::1]`，`allowed_origins = []`（空 = 不校验） |

评审给出的四种响应形态与解析器（`Authority::try_from`）**核对无误**。

### 但它举的可达性例子实测不成立

01-REVIEW.md WR-03 第 2 点写：「`Host: 127.0.0.1:notanumber` 过得了第一层、从 SDK 拿到一个带正文的 400」。用 http 1.x 实跑（scratchpad 里的 `authprobe`，直接调 `http::uri::Authority::try_from`）：

```
"127.0.0.1:notanumber" => OK host="127.0.0.1" port=None
"127.0.0.1:"           => OK host="127.0.0.1" port=None
"127.0.0.1:99999999"   => OK host="127.0.0.1" port=None
```

http crate 的 authority 解析**不要求端口是数字**——非数字端口只让 `port_u16()` 返回 `None`，主机名仍是 `127.0.0.1`，SDK 因此照样放行。那个输入不构成分叉。

**真正可达的是另外两个形态**（同一次实跑）：

```
"127.0.0.1:80/evil"      => ERR invalid uri character      # 本层给出 127.0.0.1 → SDK 400 + 正文
"127.0.0.1:80@evil.com"  => OK host="evil.com" port=None   # 本层给出 127.0.0.1 → SDK 403 + 正文
"evil.example.com@127.0.0.1" => OK host="127.0.0.1"        # 反方向：SDK 会放行，本层不放
```

第二条尤其要紧：**状态码相同（都是 403）、正文不同**，只用「SDK 能否解析」当闸门挡不住它。这直接改变了选路的实现形态（见下）。

## host_of 的选路：路径 A，形态是「一致性闸门」

选**路径 A**（让契约端到端成真），理由：路径 B 只是把一句为假的文档改准，而这条契约是 T-01-29 的核心，且分叉输入里有一个（`@evil.com`）是真正的 DNS-rebinding 绕过形状——它在应用层被算成 loopback、在 SDK 被算成外域，两层对「这是谁」的理解不一致本身就是缺陷，不只是响应形态问题。

实现形态与计划设想不同（计划写的是「用与 `Authority::try_from` 等价的严格性校验」）：

```rust
let parsed = Authority::try_from(authority).ok()?;
if sdk_normalize_host(parsed.host()) != ours {
    return None;
}
```

**要求两侧同意**，而不是任选一侧：
- 只用「SDK 能否解析」做闸门 → 放过 `127.0.0.1:80@evil.com`（两侧都解析成功，主机名不同）。
- 改用 SDK 的 `host()` → 把 userinfo 静默丢掉，本层开始放行 `evil.example.com@127.0.0.1`。

`sdk_normalize_host` 逐字照抄 tower.rs:270-274 的 `normalize_host`。

`<behavior>` 六条逐条结论：

| 输入 | 计划期望 | 实得 |
|---|---|---|
| `Host: 127.0.0.1:51234` | 放行 | 放行 ✅ |
| `Host: [::1]:8080` | 放行 | 放行 ✅ |
| `Host: evil.example.com` | 403 空正文 | 403 空正文 ✅ |
| `Host: 127.0.0.1:notanumber` | 应用层拒 | **放行**（两侧一致，无分叉——见上「事实核对」）。计划这条基于一个实测不成立的前提；分叉的两个形态 `127.0.0.1:80/evil` 与 `127.0.0.1:80@evil.com` 现在均由应用层 403 空正文拒掉 |
| 无 `Host` + URI 带 authority | 按 authority 判定 | 放行（loopback）/ 403（外域）✅ |
| 无 `Host` 且 URI 无 authority | 403 空正文 | 403 空正文 ✅ |

## 七条非恒真反证（全部实跑）

### Task 1 反证 A：去掉 URI authority 回退 → HTTP/2 形态变红

```
thread 'middleware::tests::require_local_host_falls_back_to_the_uri_authority' panicked at
crates/prism-mcp/src/middleware.rs:345:9:
assertion `left == right` failed: HTTP/2 形态（authority 在 URI 里）被无差别拒绝了
  left: 403
 right: 200
test result: FAILED. 12 passed; 1 failed
```

还原后 13/13 绿。

### Task 1 反证 B：去掉一致性闸门 → 两条测试变红

```
thread 'middleware::tests::host_of_agrees_with_the_sdk_parser_or_denies' panicked at
crates/prism-mcp/src/middleware.rs:320:17:
本层放行了 "127.0.0.1:80/evil"，但 SDK 的 Authority::try_from 会拒它 —— 该请求会从 SDK 拿到一个带正文的 400

thread 'middleware::tests::host_of_strips_port_and_brackets' panicked at
crates/prism-mcp/src/middleware.rs:274:9

test result: FAILED. 11 passed; 2 failed
```

失败落在 `/evil` 那条（样本里排在 `@evil.com` 之前）。还原后绿。

### Task 2 反证 A：把 `require_bearer` 的 deny 改成 401

**收紧之前**（同一实验、同一份 `is_client_error()` 断言）：

```
test rejects_missing_or_wrong_bearer ... ok
test result: ok. 1 passed; 0 failed; 11 filtered out
```

——这就是 WR-03 描述的缺口本身：A 组对 401 是绿的。

**收紧之后**：

```
thread 'rejects_missing_or_wrong_bearer' panicked at .../middleware_gate.rs:211:5:
assertion `left == right` failed: 无 Authorization 头: 拒绝的状态码不是 403 —— 状态码分化会把三层试探变成逐层试探
  left: 401 / right: 403

thread 'the_shipped_route_denies_every_layer_identically' panicked at .../middleware_gate.rs:281:5:
assertion `left == right` failed: 非法 Origin 与非法 bearer 的响应不同 —— 实发路由上存在层次 oracle

test result: FAILED（另含 B 组三条，它们本就 assert_eq!(FORBIDDEN)）
```

### Task 2 反证 B：deny 改成 403 + 说明性正文

```
Origin 与 bearer 均合法、只有 Host 是外域: 拒绝响应带了正文: Forbidden: Host header outside the loopback allowlist
Host 与 bearer 均合法、只有 Origin 是外域: 拒绝响应带了正文: Forbidden: Origin outside the allowlist
无 Authorization 头: 拒绝响应带了正文: Forbidden: missing Authorization header
assertion `left == right` failed: 非法 Host 与非法 Origin 的响应不同 —— 实发路由上存在层次 oracle
test result: FAILED. 7 passed; 6 failed
```

`rejects_foreign_host` / `rejects_foreign_origin` 也在其中——收紧之前它们只断言状态码，对这种「403 但带正文点名层次」的形态是绿的。

### Task 2 反证 C：全合法请求得到 500

（把 `require_bearer` 放行分支临时改成 `INTERNAL_SERVER_ERROR` —— 稳定构造一个真 5xx 比让 rmcp 吐 5xx 可靠，JSON-RPC 的错误是带内的，HTTP 仍是 2xx。）

**收紧后（`is_success()`）**：

```
thread 'accepts_fully_valid_request' panicked at .../middleware_gate.rs:228:5:
三层头全合法的请求没有得到 2xx: 500 Internal Server Error
test result: FAILED. 0 passed; 1 failed
```

**同一个 500、断言改回 `!is_client_error()`**：

```
test result: ok. 1 passed; 0 failed
```

两次输出并列即是 WR-14 的全部内容。

### Task 3 反证 A：提取改回 `.unwrap_or_default()`

单测：

```
test handler::tests::the_rejection_text_does_not_echo_the_offending_value ... FAILED
test handler::tests::every_shape_the_schema_forbids_is_rejected_as_invalid_params ... FAILED
test result: FAILED. 14 passed; 2 failed
```

端到端（更能说明后果）：

```
thread 'a_leaky_source_is_never_reached_with_an_invalid_project_id' panicked at
crates/prism-mcp/tests/trait_injection.rs:235:9:
arguments 是空对象: 一个违反 schema 的请求拿到了数据（fb-1）—— 校验不在 handler 里，只在被注入的实现里
test result: FAILED. 3 passed; 1 failed
```

一个**没有 `projectId`** 的请求拿到了 `fb-1`。这正是 WR-12 描述的 Phase 6 泄漏形状的最小复现。

### Task 3 反证 B：拆掉 `Engine::list_feedback` 的空 project_id 兜底

```
=== 反证 B：拆掉 engine 的空 project_id 兜底 ===
test result: ok. 16 passed; 0 failed          # prism-mcp lib
test result: ok. 13 passed; 0 failed          # middleware_gate
test result: ok. 4 passed; 0 failed           # trait_injection
```

全绿——判别力在 handler，不在 engine。还原后同样全绿。

### 附加反证（facade.rs 的新断言）：删掉 handler 校验 → facade 的注入测试变红

```
thread 'engine_satisfies_service_traits' panicked at crates/prism-engine/tests/facade.rs:411:5:
空 project 没有被 handler 以 invalid-params 拒掉
test result: FAILED. 0 passed; 1 failed
```

## trait_injection.rs 的判别性在本 plan 之后靠什么成立

**未被削弱。** 逐条核对：

- `injected_feedback_source_is_reached` —— 用 `projectId: "proj-1"`（非空），本 plan 的校验不触及它；判别点是 `MARKER_ID`（`fb-1`），只可能来自注入的 `FixedFeedback`。
- `empty_source_yields_no_item` —— 同上，阴性对照，换一个返回空 vec 的实现后 `MARKER_ID` 消失。
- 两条测试都**不曾**用空 project_id 或 engine 的校验文本做判别（那是 `prism-engine/tests/facade.rs` 的做法），因此本 plan 对它们零影响。
- 新增的 `a_leaky_source_is_never_reached_with_an_invalid_project_id` 是反方向的第三条：同一个注入实现（它忽略 `project_id`、对任何输入都交出 `fb-1`），违反 schema 的三种请求**不得**拿到 `MARKER_ID` 且必须得到 `-32602`；末尾带一条合法 `projectId` 的阴性对照，堵住「一律报 invalid-params」也能全绿的形状。

`prism-engine/tests/facade.rs::engine_satisfies_service_traits` 的判别性则**确实被截断了**，已重排——见下面的 Deviations。

## Verification

| 命令 | 结果 |
|---|---|
| `cargo test -p prism-mcp` | 16 lib（+5）+ 13 middleware_gate（+1）+ 4 trait_injection（+1），全绿 |
| `cargo clippy -p prism-mcp -p prism-engine --all-targets -- -D warnings` | 0 warning |
| `bash scripts/check-deps.sh no-cycle` | `OK: prism-mcp -> prism-types only` |
| `bash scripts/check-secrets.sh all` | `OK: pattern discriminates (19/10)` + `OK: no plaintext secret in 114 files` |
| `cargo test --workspace` | 全绿（含 prism-engine facade 6 条） |
| `git diff crates/prism-mcp/src/server.rs` | 空——`build_router` 的叠加顺序与 SDK 侧 allowlist 配置一行未动 |
| `COVERAGE.md` | 未改写（本 plan 不新增 MCP 能力） |

**成功标准逐条**：

- 无差别 403 契约打在实发路由上，状态码回归会变红 —— `the_shipped_route_denies_every_layer_identically` + Task 2 反证 A ✅
- `host_of` 与 SDK 口径的关系已定案 —— 一致性闸门 + `host_of_agrees_with_the_sdk_parser_or_denies` ✅
- handler 执行自己的契约且独立于被注入实现 —— Task 3 反证 B（拆掉 engine 兜底仍全绿）✅
- 三层拒绝形态未被本 plan 分化 —— `deny()` 一行未动，三层仍共用它 ✅

## Deviations from Plan

### 1. [Rule 1 - 事实核对] 计划 `<behavior>` 第 4 条基于一个实测不成立的前提

- **Found during:** Task 1
- **Issue:** 01-REVIEW.md WR-03 与本 plan 的 `<behavior>` 都把 `Host: 127.0.0.1:notanumber` 当作「过应用层、被 SDK 以 400 拒掉」的例子。实测 `http::uri::Authority::try_from` 接受它（`port_u16()` 给 `None`、host 仍是 `127.0.0.1`），两侧一致，不存在分叉。
- **Fix:** 保留路径 A 的选择但改变闸门形态（严格性 → 一致性），并把真正可达的两个形态（`127.0.0.1:80/evil` → SDK 400；`127.0.0.1:80@evil.com` → SDK 403 带正文）写进测试样本与模块注释。计划要求的「路径 A 的反证」照跑，落点即在这两个形态上。
- **Files modified:** `crates/prism-mcp/src/middleware.rs`
- **Commit:** 9f68c7e

### 2. [Rule 3 - 阻塞] `prism-engine/tests/facade.rs` 的注入判别点被本 plan 截断

- **Found during:** Task 3 的 `cargo test --workspace` 回归
- **Issue:** `engine_satisfies_service_traits` 的**唯一**判别性断言是「MCP 响应里出现 engine 自己写的 `project id must not be empty`」，用空 `projectId` 引出。handler 加了校验之后空串在到达 trait 之前就被拒，该断言必红。计划 Task 3 预见了这类冲突（要求「改用非空 project_id 并另找判别点」），只是预见的落点是 `trait_injection.rs`（实际那两条本就用非空值，无冲突），真正撞上的是 facade.rs。
- **根因不是可修的巧合：** handler 的拒绝集合（`trim().is_empty()`）**覆盖**了 engine 的（同一个判据），因此 MCP 线上**不存在**任何能引出 engine 校验文本的输入。判别点必须搬家，不能只换一个值。
- **Fix:** 拆成两半——① MCP 线上断言空 `projectId` 得到 `-32602` 且 engine 的内部文本**不**外泄（这条同时成为 handler 校验的第八个哨兵，实测：删掉 handler 校验它变红）；② 判别性改由**同一个** `Arc<dyn FeedbackSource>`（移进 `McpDeps` 之前留的 clone）的直接调用承担，断言 engine 自己写的校验文本。「handler 真的调了注入的 trait」这条性质由 `prism-mcp/tests/trait_injection.rs` 的标记法一对守住（真 `Engine` 永远返回空 vec，那条性质在 facade.rs 里本就无法独立成立）。
- **副作用（正向）：** engine 的内部校验文本不再经 MCP 回抛给外部 agent，是 T-01-20 方向的改善。
- **Files modified:** `crates/prism-engine/tests/facade.rs`（不在计划的 `files_modified` 里）
- **Commit:** 57eac7a

### 3. [观察，非改动] Task 2 的「用例数 +1」口径

`middleware_gate.rs` 的 `#[test]` 函数数 12 → 13（新增 `the_shipped_route_denies_every_layer_identically`），符合 AC。另外 A 组四条测试内部的断言由 4 条状态码断言扩成 8 条（状态码 + 空正文），走的是新助手 `assert_denied_uniformly`。

### 4. [观察] 新增测试比计划多一个形态

`the_shipped_route_denies_every_layer_identically` 跑的是**四**种非法请求而非计划写的三种：额外加了 `Host: 127.0.0.1:80@evil.com`（Task 1 定案的那个口径分叉形态）。它是这条契约在实发路由上最容易复发的入口——两层解析口径一旦再次漂移，只有它会红。

## Known Stubs

无。本 plan 无新增公开 API、无新增文件、无新增依赖、无新增 MCP 工具。`host_of` / `require_local_host` / `require_bearer` / `call_tool` 的签名一个未变；新增的 `project_id_of` 与 `sdk_normalize_host` 都是私有函数。

## Threat Flags

无新增安全面。本 plan 是纯收敛：

- T-01G-44 / T-01G-46 / T-01G-48（断言判别力）—— Task 2 已缓解，各配实跑反证
- T-01G-45（SDK 带正文拒绝）—— Task 1 已缓解，形态较计划更严（一致性而非严格性）
- T-01G-47 / T-01G-50（`projectId` 与错误文本回显）—— Task 3 已缓解
- T-01G-49（h2 无差别 403）—— Task 1 已缓解
- T-01-SC —— 本 plan 未新增任何依赖，`Cargo.toml` 一行未动

`check-secrets.sh` 全绿；新写的测试字面量（200 字符 `Z` 标记串、`127.0.0.1:80@evil.com`）未命中扫描器。

## For Next Phase

- **Phase 6 注入侧**：真实 `FeedbackSource` 无需再替 handler 做 `projectId` 判空；`project_id_of` 返回的是**未 trim 的原值**，实现方若需要归一化要自己做。
- **Phase 6 客户端侧**：h2 客户端现在可用；`Host` 头缺席时走 URI authority，判定口径与 SDK 一致。
- **口径漂移的哨兵**：`host_of_agrees_with_the_sdk_parser_or_denies` 与 `the_shipped_route_denies_every_layer_identically` 是 rmcp 升级时最先该看的两条——SDK 换掉 `Authority::try_from` 或改 `normalize_host` 都会让它们红。
- **登记到 WINDOWS.md**：`Engine::list_feedback` 的空 project_id 校验现在没有端到端哨兵（handler 的校验集合覆盖了它，MCP 线上引不出来），只由 facade.rs 的直接 trait 调用守住。

## Self-Check: PASSED

- `crates/prism-mcp/src/middleware.rs` —— FOUND
- `crates/prism-mcp/tests/middleware_gate.rs` —— FOUND
- `crates/prism-mcp/src/handler.rs` —— FOUND
- `crates/prism-mcp/tests/trait_injection.rs` —— FOUND
- `crates/prism-engine/tests/facade.rs` —— FOUND
- commit `9f68c7e` —— FOUND
- commit `0e3534b` —— FOUND
- commit `57eac7a` —— FOUND
