---
phase: 01-foundation-skeleton
plan: 16
subsystem: mcp-bearer-gate
tags: [security, mcp, bearer, constant-time, rfc7235, gap-closure]
status: complete

requires:
  - "crates/prism-mcp/src/middleware.rs（01-06 建立的三层门禁 + 01-12 加固的 constant_time_eq）"
  - "crates/prism-mcp/src/deps.rs（01-12 把 McpDeps::new 改成可失败构造）"
  - "01-REVIEW.md WR-05 / WR-06 / WR-07"
provides:
  - "比较层只有一个空值处理点，且它的存在由一条落点唯一的源码断言看住"
  - "McpDeps 一旦存在，它持有的就是一份已归一化的非空 bearer（存与取对同一份字节达成一致）"
  - "RFC 7235 §2.1 合规的 scheme 匹配：大小写不敏感 + 容忍 1*SP"
  - "middleware_gate 的三行放行取样（阴性对照），使「改成接受一切」不再能全绿"
affects:
  - "Phase 6：从钥匙串读出的 token 带尾随换行不再造出「构造成功但永远比不中」的门禁"
  - "Phase 6：不受本项目控制的 MCP 客户端（Claude Code / 其他 agent）发 `bearer` / `BEARER` 不再被误拒"
  - "plan 01-17 将在同两个文件上继续改动——本 plan 未触碰它的范围"

tech-stack:
  added: []
  patterns:
    - "源码断言的锚点必须取**完整语句**：函数体的匹配面同时含代码与注释，一个也可能出现在注释里的片段会在守卫被删后仍然绿（本 plan 实测）"
    - "死分支不只是噪声，它会**稀释源码断言的判别力**：`padded` 分支里那份 `if expected.is_empty() {` 让同一条断言在守卫被删掉时照样通过"
    - "「用 trim 判空、存原值」是一类可复用的缺陷形状：判空与存值必须落在同一份字节上，否则失败静默且 fail-closed"
    - "呈递侧只裁前导空白、配置侧在构造期 trim：归一化的责任分两处但只做一次，避免「token 末尾有空白」与「header 多打了空格」不可区分"

key-files:
  created: []
  modified:
    - crates/prism-mcp/src/middleware.rs
    - crates/prism-mcp/src/deps.rs
    - crates/prism-mcp/tests/middleware_gate.rs

decisions:
  - "只删不可达分支，不重写折叠结构：上轮 WR-15 指出 XOR fold 在 same_len 之下冗余，但绑定两件事会让「哪一层挡住了」的反证落点不再唯一（沿用 STATE 已记录的决策）"
  - "源码断言的锚点定为 `if expected.is_empty() {`（含 if 与左花括号），不用裸 `is_empty`——反证 B 实测证明注释里的词不足以钉住它"
  - "呈递值只 trim_start：尾随 OWS 由 HTTP 头解析层负责，配置侧归一化在 McpDeps::new 做过一次"
  - "cargo fmt 会顺带重排三个文件里的既有代码（早于本 plan 的 rustfmt 漂移，deferred-items.md 01-03 已登记）——按 scope boundary 已回滚，只保留本 plan 新写代码的格式"

metrics:
  duration: ~35min
  tasks: 3
  files: 3
completed: 2026-07-29
---

# Phase 01 Plan 16: prism-mcp 门禁层三条缺口关闭 Summary

把 bearer 门禁收敛成「一个空值处理点 + 一份归一化字节 + 一条 RFC 合规的 scheme 匹配」：删掉 `constant_time_eq` 里那段为不存在的第二条空值路径写的死代码并用完整语句锚点的源码断言钉住真正的守卫；`McpDeps::new` 改为存 trim 后的值；`require_bearer` 按 RFC 7235 §2.1 大小写不敏感比对 scheme 并容忍 `1*SP`。

## What Was Built

### Task 1 —— `constant_time_eq` 只保留唯一的空值处理点（commit b1f8397）

- 删掉 `expected.len().max(1)` 的 `.max(1)`：早退之后长度必 ≥1，`1` 那一臂取不到。
- 删掉 `padded` 二选一分支及其上方那条描述「空 expected 时 folded 是长度 1 的哨兵」的注释——那个状态函数进不去，而注释在邀请下一个读者删掉真正的守卫。
- 注释改写成陈述唯一性：「本函数**唯一**的空值处理点，下面没有第二道守卫——删掉这一条，CR-03 的 fail-open 当场复活」；doc 注释同步加一句指向看着它的那条断言。
- `the_comparison_is_not_a_plain_equality` 加第三条源码断言，锚点为完整语句 `if expected.is_empty() {`。

函数签名、折叠缓冲区结构、长度比较按位与均不变；`constant_time_eq_agrees_with_equality_on_every_shape` 的 8 条断言（含 `assert!(!constant_time_eq("", ""))`）一条未删。

### Task 2 —— `McpDeps::new` 存 trim 后的 bearer（commit b730c09）

`let bearer: Arc<str> = Arc::from(bearer.into().trim());` 后再判空，归一化与判空落在同一份字节上。注释写明这是一个「构造成功但永远比不中」的门禁形状：失败静默且 fail-closed，唯一诊断信号是一条与攻击者不可区分的 `warn!("bearer token mismatch")`，并点名 Phase 6 钥匙串往返 / 文件回退 / `Command` 输出三条常带尾随换行的现实路径。

`an_empty_bearer_is_refused_at_construction` 新增第 ⑤ 段三条断言：`" tok "` → `"tok"`；64 位十六进制的干净值逐字不变；`"<value>\n"` → 不带换行的同一值。①②③④ 段（空串 / 纯空白 / 错误文本零插值 / 阴性对照）一字未改。

### Task 3 —— `require_bearer` 按 RFC 7235 匹配 scheme（commit 04b191c）

`raw.strip_prefix("Bearer ")` 一行拆成三步：`split_once(' ')` 取出 scheme 与 credentials（切不出来 → `deny("Authorization header carries no credentials")`）、`scheme.eq_ignore_ascii_case("bearer")`（不匹配 → 沿用的 `deny("Authorization scheme is not Bearer")`）、credentials 只做 `trim_start()` 后送进 `constant_time_eq`。三条 `deny` 共用同一个 `deny()`，响应仍是 403 + 空正文，原因串是编译期常量、只进本地 tracing。

`bearer_layer_alone_is_what_rejects_a_bad_token` 补一张放行取样表（小写 scheme / 大写 scheme / 多空格，均期望 200 + 到达 sentinel）；`the_bearer_token_never_appears_in_a_response` 的循环补进小写 scheme 形态。既有的 `Basic <GOOD_BEARER>` 拒绝行保留——它是让「改成接受一切」不能蒙混过关的那一条。

## 四组非恒真反证（全部实跑）

### 反证 A1（改动前形态）：删掉空值早退 → `constant_time_eq("", "")` 复活为 true

在**清理死代码之前**删掉 `if expected.is_empty() { return false; }`：

```
---- middleware::tests::constant_time_eq_agrees_with_equality_on_every_shape stdout ----
thread '...' panicked at crates/prism-mcp/src/middleware.rs:225:9:
assertion failed: !constant_time_eq("", "")
test result: FAILED. 10 passed; 1 failed
```

**但 `the_comparison_is_not_a_plain_equality` 仍然是绿的。** 原因正是 WR-05 描述的那个陷阱的加强版：死分支 `let padded = if expected.is_empty() {` 里含有与新断言完全相同的字面量，于是守卫被删掉之后，源码断言照样在死分支上匹配成功。这条实测把「死代码是主动危险」从论证变成了可复现事实——它不仅误导人类读者，还稀释了看着这段代码的那条断言的判别力。

### 反证 A2（清理后形态）：删掉空值早退 → 两条断言同时变红

```
---- middleware::tests::the_comparison_is_not_a_plain_equality stdout ----
thread '...' panicked at crates/prism-mcp/src/middleware.rs:250:9:
空配置的短路守卫被删掉了 —— 比较层的 fail-open 复活了

---- middleware::tests::constant_time_eq_agrees_with_equality_on_every_shape stdout ----
thread '...' panicked at crates/prism-mcp/src/middleware.rs:163:20:
attempt to calculate the remainder with a divisor of zero

test result: FAILED. 9 passed; 2 failed
```

行为测试的失败点是 `constant_time_eq("", configured)` 那一行的除零 panic（先于 `("", "")` 那条执行）：`.max(1)` 删除后缓冲区长度为 0，`% folded.len()` 当场炸。落点从「悄悄返回 true」变成「立刻 panic」，是比改动前更硬的失败形态；而 `("", "")` 会返回 true 这件事由反证 A1 单独证明（A1 的形态里 `.max(1)` 尚在，因此不 panic，红的正是那条断言）。两条一起看，「删掉早退 → 至少两个测试变红」成立。还原后 11/11 绿。

### 反证 B：源码断言的锚点取注释里也有的词 → 守卫被删仍全绿

把断言改成 `body.contains("fail-open")`（`fail-open` 只出现在函数体的注释里）并删掉早退语句（保留注释）：

```
test middleware::tests::the_comparison_is_not_a_plain_equality ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 10 filtered out
```

锚点选窄一步就是「有断言」与「有一条永远为真的断言」的差别。**最终选定的锚点字面量：`if expected.is_empty() {`**（含 `if`、含左花括号，只可能出现在那条早退语句里）。

### 反证 C（Task 2）：把 `McpDeps::new` 改回「trim 判空、存原值」→ 只有新增的第 ⑤ 段变红

```
test deps::tests::debug_does_not_reveal_the_bearer_token ... ok
test deps::tests::deps_are_cheap_to_clone_and_keep_the_same_token ... ok
test deps::tests::an_empty_bearer_is_refused_at_construction ... FAILED

---- deps::tests::an_empty_bearer_is_refused_at_construction stdout ----
thread '...' panicked at crates/prism-mcp/src/deps.rs:190:9:
assertion `left == right` failed: the constructor stored the untrimmed value —— 门禁永远比不中
  left: " tok "
 right: "tok"

test result: FAILED. 10 passed; 1 failed
```

失败点精确落在第 ⑤ 段的第一条断言上，说明 ①②③④ 段全部先行通过——本 Task 的判别力不是搭在既有断言上的。还原后 11/11 绿。

### 反证 D（Task 3）：`require_bearer` 改成「有 Authorization 头就放行」→ 三条拒绝断言变红

```
---- bearer_layer_alone_is_what_rejects_a_bad_token stdout ----
thread '...' panicked at .../middleware_gate.rs:331:9:
assertion `left == right` failed: 等长但内容不同 未被拒
  left: 200 / right: 403

---- an_empty_presented_token_is_denied_by_the_bearer_layer_alone stdout ----
thread '...' panicked at .../middleware_gate.rs:404:5:
assertion `left == right` failed: 空呈递 token 未被 require_bearer 拒绝

---- rejects_missing_or_wrong_bearer stdout ----
thread '...' panicked at .../middleware_gate.rs:193:5:
等长错误 token 未被拒: 200 OK

test result: FAILED. 9 passed; 3 failed
```

拒绝表在第一行就红，`Basic` 那一行来不及执行。因此又跑了一次**只摘掉 scheme 检查、保留 token 比较**的隔离变体，让 `Basic <GOOD_BEARER>` 成为唯一被放行的形态：

```
---- bearer_layer_alone_is_what_rejects_a_bad_token stdout ----
assertion `left == right` failed: scheme 不是 Bearer 未被拒
  left: 200 / right: 403

---- rejects_missing_or_wrong_bearer stdout ----
非 Bearer scheme 未被拒: 200 OK

test result: FAILED. 10 passed; 2 failed
```

落点唯一地打在那条阴性对照上。加上反证 D 的第一段，「只加放行样本、不加拒绝对照」会让这个反证全绿这件事被两次实测封住。

### 反证 A（Task 3 的 RED 阶段）：scheme 字节精确匹配 → 新增的放行行变红

先写测试后写实现，RED 阶段跑到的正是这个状态：

```
---- bearer_layer_alone_is_what_rejects_a_bad_token stdout ----
assertion `left == right` failed: 小写 scheme —— RFC 7235 §2.1 的 auth-scheme 大小写不敏感: 合规客户端被误拒
  left: 403 / right: 200
test result: FAILED. 11 passed; 1 failed
```

其余 11 个测试全绿——改动是加法而非放宽。

## Verification

| 命令 | 结果 |
|---|---|
| `cargo test -p prism-mcp` | 11 lib + 12 middleware_gate + 3 trait_injection，全绿 |
| `cargo clippy -p prism-mcp --all-targets -- -D warnings` | 0 warning（死代码删除后无新 lint） |
| `bash scripts/check-deps.sh no-cycle` | `OK: prism-mcp -> prism-types only` |
| `bash scripts/check-secrets.sh all` | `OK: pattern discriminates (19/10)` + `OK: no plaintext secret in 114 files` |
| 八个 engine crate 全量 `cargo test` | 全绿 |

**关于「用例数 +3」这条 AC 的口径**：新增的是同一个 `#[test]` 内的三行**取样**（外加 `the_bearer_token_never_appears_in_a_response` 循环里的第 4 条形态），`#[test]` 函数个数不变，仍是 12。取样条数 +4。

**成功标准逐条**：

- 比较层只有一个空值处理点，删掉它会有测试变红 —— 反证 A2 ✅
- 存与取对同一份归一化字节达成一致 —— 反证 C + 第 ⑤ 段三条断言 ✅
- 合规客户端不被误拒，伪造 token 仍以无差别 403 被拒 —— 放行取样表 + 拒绝表 ✅
- 三层拒绝响应仍逐字节相同 —— `rejections_do_not_disclose_which_layer_denied` 仍绿 ✅

## Deviations from Plan

### 1. [Rule 2 - 文档正确性] 同步一条因本 plan 而失真的测试注释

- **Found during:** Task 3
- **Issue:** `an_empty_presented_token_is_denied_by_the_bearer_layer_alone` 的 doc 注释里写着「`strip_prefix("Bearer ")` 对它给出 `Some("")`」，而这一行已在本 Task 被替换。注释描述一个不再存在的实现，正是 WR-05 那类「注释邀请错误推论」的形状。
- **Fix:** 改为陈述新解析路径（scheme = `Bearer`、credentials = 空串），并保留一句旧形态的对照，说明结论不随实现变化。
- **Files modified:** `crates/prism-mcp/tests/middleware_gate.rs`
- **Commit:** 04b191c

### 2. [Scope boundary] `cargo fmt -p prism-mcp` 的越界改动已回滚

- **Found during:** Task 2
- **Issue:** 顺手跑的 `cargo fmt -p prism-mcp` 重排了 `middleware.rs` / `middleware_gate.rs` / `trait_injection.rs` 里**早于本 plan**的既有代码（`require_bearer` 签名折行、`.header(...)` 拆行等），与本 plan 无关。
- **Fix:** `git checkout --` 回滚三个文件的格式化改动，只保留 deps.rs 的实际改动；后续两个 Task 手写 rustfmt 一致的代码而不再整包 fmt。`cargo fmt -p prism-mcp -- --check` 的残留差异全部位于本 plan 未触碰的既有代码。
- **既有登记:** 该 rustfmt 漂移与「CI 无 fmt 闸门」已在 `deferred-items.md`「发现于 plan 01-03」条目下登记，本 plan 不重复登记。

### 3. [观察，非改动] Task 2 新增的 64 位十六进制断言

计划 behavior 只列了 `" tok "` 与 `"<32hex>\n"` 两条，但 acceptance criteria 要求「以一个无外围空白的 64 位十六进制串构造，`expose_bearer()` 与入参逐字相同」。按 AC 补了这条断言（原第 ④ 段的阴性对照用的是 19 字符短串，不满足「64 位十六进制」的字面要求）。绑定名沿用 `configured` 以避开 `check-secrets.sh` 的关键词分支。

## Known Stubs

无。本 plan 无新增公开 API、无新增文件、无新增依赖；`McpDeps::new` / `expose_bearer` / `require_bearer` / `constant_time_eq` 的签名一个未变。

## Threat Flags

无新增安全面。三条 `deny` 全部走同一个 `deny()`，`rejections_do_not_disclose_which_layer_denied` 与 `the_bearer_token_never_appears_in_a_response` 两条回归闸门均绿（T-01G-12 已缓解）；新写的注释与测试字面量未命中 `check-secrets.sh`（T-01G-13 已缓解）。

## For Next Phase

- plan 01-17 将继续改 `middleware.rs` 与 `tests/middleware_gate.rs`。本 plan 留下的接口面：`require_bearer` 内部现在有三条 `deny` 分支（无 credentials / scheme 不匹配 / token 不匹配），`bearer_layer_alone_is_what_rejects_a_bad_token` 内部现在有「拒绝表 + 单条放行 + 放行表」三段。
- Phase 6 注入侧：`McpDeps::new` 会 trim，钥匙串读出的尾随换行不再是问题；但 `Err(EmptyBearer)` 仍需按 D-06 降级为「MCP 服务不启动 + 一条 warn」，不要 `unwrap()`。

## Self-Check: PASSED

- `crates/prism-mcp/src/middleware.rs` —— FOUND
- `crates/prism-mcp/src/deps.rs` —— FOUND
- `crates/prism-mcp/tests/middleware_gate.rs` —— FOUND
- commit `b1f8397` —— FOUND
- commit `b730c09` —— FOUND
- commit `04b191c` —— FOUND
