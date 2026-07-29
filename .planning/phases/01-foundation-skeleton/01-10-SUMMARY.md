---
phase: 01-foundation-skeleton
plan: 10
subsystem: security
tags: [credential-leak, url-validation, userinfo, settings, sqlite, tdd, counterproof, gap-closure, T-01-39, T-01-43]
status: complete

# Dependency graph
requires:
  - "01-04：`docs/keychain-naming.md` 不变量 1（密钥绝不进 settings 表）与它给出的理由——sidecar 按单目录整体备份"
  - "01-05：`validate_base_url` / `set_setting` / `is_secret_like_key` 三者的既有形态，以及「守卫长在写入路径而非调用方」的设计陈述"
  - "01-08：IPC 错误短码契约（`invalid_url` / `invalid_setting` / …）与 `map_err` 映射表"
  - "01-09：`src/lib/ipc.ts` 的 `ERROR_COPY` 表、`src/pages/Settings.tsx` 与 `Settings.test.tsx` 的既有 harness"
provides:
  - "`validate_base_url` 的**值侧**守卫：拒绝 userinfo（username 非空 或 password 存在）、拒绝 query 或 fragment"
  - "`settings_base_url_rejects_credential_bearing_values` —— 六组断言的非恒真用例（纯函数拒绝 / 写入路径拒绝 / 未留痕 / 不回显 / 幂等 / 阴性对照）"
  - "`localUrlIssue(raw): \"invalid_url\" | \"invalid_url_credentials\" | null` —— 取代返回布尔的 `looksLikeHttpUrl`，判定面与 engine 逐项对齐"
  - "`ERROR_COPY[\"invalid_url_credentials\"]` —— 凭据型端点的专属中文文案（rule-shaped，不回显输入）"
  - "前端用例 `rejects a credential-bearing endpoint before it ever reaches the engine`（四条断言含阴性对照）"
affects: [phase-4-LLM设置页与真实请求, phase-6-MCP设置页, 01-11-明文密钥静态扫描]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "密钥容器的边界必须建在**值**上而不只是键名上：`is_secret_like_key` 防键名，`llm.base_url` 这个键名完全正常，凭据藏在值里"
    - "同一个洞的两个面：userinfo（`user:key@host`）与 query/fragment（`?api-key=…`）是同一类凭据载体，守卫必须同时覆盖，且两段早退可各自被掏空反证"
    - "拒绝面扩张时**不加 error 变体**：`InvalidUrl` 语义已够用，加变体会连带要求 `map_err` 与 `ERROR_COPY` 同步扩表——那是 IPC 短码契约的变更"
    - "前端体验层校验返回**错误码**而非布尔：布尔无法区分「scheme 不对」与「带凭据」，而文案必须只在 `ERROR_COPY` 里生成"
    - "反证要看**落点**：五条反证各自使对应测试变红，且失败断言落在被守的那条上而非前置条件上"

key-files:
  created: []
  modified:
    - crates/prism-store/src/settings.rs
    - src/lib/ipc.ts
    - src/pages/Settings.tsx
    - src/pages/Settings.test.tsx

key-decisions:
  - "01-10: 凭据守卫扩在 `validate_base_url` 内部而非新加调用点——`set_setting` 一行未动，「机制而非约定」的设计陈述完整保住"
  - "01-10: query/fragment 用 `is_some()` 而非「非空」判定：base_url 本身没有携带二者的正当用途，真正的 query 由 Phase 4 发请求时自拼"
  - "01-10: `StoreError` 不加变体，engine 侧对两类拒绝一律回 `invalid_url` 短码；「是凭据还是 scheme」的区分只在前端本地校验层做得到（那一层在 IPC 往返之前）"
  - "01-10: 前端 ③ 的不回显断言范围收到 `role=\"alert\"` 文案上——端点输入框是 `type=\"text\"`，它必须显示用户键入的内容，`document.body.innerHTML` 必然含有该串（实测确认）"
  - "01-10: URL fixture 的密码位复用非 `sk-` 的既有 fixture 值（Rust 侧 `prism-test-secret-value`、前端侧 `FAKE_KEY`），扫描器 allowlist 一条不新增"

patterns-established:
  - "值侧凭据守卫：解析后逐项检查 userinfo / query / fragment，消息 rule-shaped 且禁止插值 raw / username() / password()（T-01-26）"
  - "两道守卫的判定面对齐：前端 `localUrlIssue` 与 engine `validate_base_url` 判定同一组四类，避免「前端放行、engine 拒绝」在正常输入上出现"
  - "掏空式反证成对：一个守卫有两段早退时，两段各配一条反证，证明它们各自独立成立而非互相兜底"

requirements-completed: [INFRA-02, INFRA-03]

coverage:
  - id: D1
    description: "engine 写入路径拒绝携带凭据的 base_url 值（userinfo / query / fragment），被拒的值不进 settings 表，错误消息不回显用户名或密码"
    requirement: INFRA-03
    verification:
      - kind: unit
        ref: "cargo test -p prism-store settings_base_url_rejects_credential_bearing_values"
        status: pass
      - kind: unit
        ref: "cargo test -p prism-store（31 tests，settings / concurrency / fts_cjk / migrations 全绿）"
        status: pass
      - kind: other
        ref: "反证 A（删 userinfo 早退）红在 ①第一条；反证 B（删 query/fragment 早退）红在 ①第三条；反证 C（一律拒绝）红在 ⑥"
        status: pass
      - kind: other
        ref: "verifier 原始探针复刻：PROBE validate_base_url=false / PROBE set_setting=false / PROBE stored value=None"
        status: pass
    human_judgment: false
  - id: D2
    description: "前端在提交前本地拒绝携带凭据的端点，给出专属中文文案且一次 IPC 都不发出；干净端点仍正常保存"
    requirement: INFRA-03
    verification:
      - kind: unit
        ref: "npm run test -- --run src/pages/Settings.test.tsx（8 cases，含 rejects a credential-bearing endpoint before it ever reaches the engine）"
        status: pass
      - kind: unit
        ref: "npm run test -- --run（6 files / 33 tests 全绿，改动前为绿的用例无一转红）"
        status: pass
      - kind: other
        ref: "反证 D（删前端 userinfo/query/fragment 判定）红在 ①；反证 E（一律拒绝）红在 ④"
        status: pass
      - kind: other
        ref: "npx tsc --noEmit 退出 0；npm run build 退出 0"
        status: pass
    human_judgment: false
  - id: D3
    description: "两道守卫的判定面对齐（scheme / userinfo / query / fragment 同集），不引入依赖图变更"
    requirement: INFRA-02
    verification:
      - kind: other
        ref: "bash scripts/check-deps.sh all 退出 0（六条断言全 OK）"
        status: pass
      - kind: other
        ref: "源码断言：validate_base_url 体内出现 url.username() 与 url.password()，且二者不出现在任何 format! / 消息构造里"
        status: pass
      - kind: other
        ref: "cargo clippy --workspace --all-targets -- -D warnings 退出 0"
        status: pass
    human_judgment: false

# Metrics
duration: 10min
completed: 2026-07-29
---

# Phase 01 Plan 10: 凭据型 base_url 的值侧守卫 Summary

把密钥容器的边界从**键名**扩到**值**——`validate_base_url` 现在拒绝 userinfo、query 与 fragment，凭据型 `base_url` 在 engine 写入路径上被拒且不留痕；前端本地校验同步收紧并改为返回错误码，两侧判定面逐项对齐。

## 关闭的缺口

01-VERIFICATION.md gap 1（Blocker）：`https://user:密钥@host/v1` 此前通过 `validate_base_url`（它只看 scheme 与 host）并被 `set_setting` 原样写进 SQLite `settings` 表，推翻了 `docs/keychain-naming.md` 不变量 1 与 plan 01-04 标为 `resolved` 的 privacy prohibition。verifier 已用探针实测复现——本 plan 用同一探针形态确认复现已消失。

守卫的边界此前建错了地方：`is_secret_like_key` 防的是键名，而 `llm.base_url` 这个键名完全正常。前端 `looksLikeHttpUrl` 是同一个形状的洞，两道守卫在同一处一起漏。

## Accomplishments

### Task 1 — engine 值侧守卫（tracer, TDD）

`crates/prism-store/src/settings.rs`：在 `validate_base_url` 的 host 检查之后、明文 http 告警之前加入两段早退，形状与既有两段 `return Err(StoreError::InvalidUrl(...))` 完全一致。

- **凭据位**：`!url.username().is_empty() || url.password().is_some()`。两个条件都要——`https://user@host/v1` 里 `password()` 是 `None`，只看 password 会漏掉只带用户名的形态。
- **query / fragment**：`url.query().is_some() || url.fragment().is_some()`。部分 OpenAI 兼容网关用 `?api-key=…` 传凭据，那是同一个洞的另一面；base_url 本身没有携带二者的正当用途。
- 两条消息都 rule-shaped，不插值 `raw` / `url.username()` / `url.password()`（T-01-26）。
- `set_setting` **一行未动**——它本来就在 `key == SETTING_BASE_URL` 时调用 `validate_base_url`，缺的只是守卫的覆盖面。改动落在被调用的校验函数里而不是新加调用点，「守卫是机制而非约定」这条设计陈述因此完整保住（T-01-43：绕过界面直接 invoke 不改变结果）。

新测试 `settings_base_url_rejects_credential_bearing_values`，六组断言，复用既有 `fixture()` / `count()` 助手：

| # | 断言 | 删掉之后重新可能的静默失败 |
|---|---|---|
| ① | 四类输入 `validate_base_url` 均返回 `Err(InvalidUrl)` | 守卫的某一段被掏空，表现为「保存成功」而无任何报错 |
| ② | `store.write(\|tx\| set_setting(...))` 返回 `Err(InvalidUrl)` | 守卫退化成「调用方约定」，绕过界面即可写入 |
| ③ | `count(&store) == 0` | 「先 INSERT 再报错」的实现也会绿，而备份带走的是行不是返回值 |
| ④ | 错误串不含用户名子串也不含密码子串 | 被误填的密钥随错误消息进入本地日志与前端 DOM |
| ⑤ | 同一 URL 连写两次都被拒、行数始终 0 | 「只在首次写入时校验」的实现也会绿 |
| ⑥ | 干净的 `https://api.example.com/v1` 仍被接受且恰好写入 1 行 | 「一律拒绝」式的假修复会让上面五条全绿 |

### Task 2 — 前端本地校验同步收紧（TDD）

- `src/lib/ipc.ts`：`ERROR_COPY` 新增 `invalid_url_credentials`，文案说清两件事（链接不能带 `user:pass@host` / 不能带 `?…` 或 `#…`；密钥请填 API key 栏，它只进钥匙串不入库）。`errorCopy` 函数体**零行变更**（其原型链问题 WR-05 不在本次范围）。
- `src/pages/Settings.tsx`：`looksLikeHttpUrl`（布尔）→ `localUrlIssue`（错误码 | `null`）。判定顺序 trim → scheme 前缀 → `new URL()` try/catch → userinfo → query/fragment，与 engine 侧四类逐项对齐。「**这不是安全边界**」那段注释保留并补了一句：多认一种形态是为了让用户在按下保存之前就知道，不是因为它变成了防线。
- `src/pages/Settings.test.tsx`：新增一条用例，四条断言（专属文案 / `setBaseUrl` 零次调用 / 文案不回显输入 / 阴性对照 `setBaseUrl` 恰好被调用一次）。凭据 URL 由既有 `FAKE_KEY` 拼出。

## Task Verification

| Task | Type | Commits | Verify |
|------|------|---------|--------|
| 1 | tracer / tdd | `be3cd35`(RED) `bcd7c27`(GREEN) | `cargo test -p prism-store` 31 passed；`cargo clippy -p prism-store -- -D warnings` exit 0 |
| 2 | auto / tdd | `dfa1524`(RED) `4f1bc13`(GREEN) | `npm run test -- --run` 6 files / 33 passed；`npx tsc --noEmit` exit 0；`npm run build` exit 0 |

**Tracer feedback gate**：Task 1 提交后重跑其 `<verify>` 端到端通过（含 workspace 级 `cargo test --workspace` 与 `clippy --workspace --all-targets` 全绿），据此展开 Task 2。`workflow.human_verify_mode = end-of-phase`（config.json），mid-flight 不停机；tracer 的人工确认项并入 phase 末批次。

## 反证（落点逐条核对）

| 反证 | 掏空的东西 | 结果 | 落点 |
|------|-----------|------|------|
| A | engine userinfo 早退 | 红 | ① 第一条（`Ok(Url{ username: "prism-test-user", password: Some(...) })`） |
| B | engine query/fragment 早退 | 红 | ① 第三条（`Ok(Url{ query: Some("api-key=...") })`） |
| C | engine 一律拒绝 | 红 | ⑥「干净的 https 端点仍应被接受」 |
| D | 前端 userinfo/query/fragment 判定 | 红 | ①（`findByRole("alert")` 超时——请求走通并显示「端点已保存」） |
| E | 前端一律拒绝 | 红 | ④（`Settings.test.tsx:159` 阴性对照的 `setBaseUrl` 调用断言） |

五条全部落在被守的那条断言上，无一落在前置条件上。反证 E 另外带红两条既有 base_url 用例——那是「一律拒绝」的应有后果，不影响落点判定。

## 与 01-VERIFICATION.md § SC-4 探针的逐行对照

用 verifier 的原始探针形态跑了一次一次性测试（跑完即删，未留在仓库里）：

```
PROBE validate_base_url ok = false
PROBE set_setting ok = false
PROBE stored value = None
```

三行与 gap 1 要求的目标状态完全一致。

## TDD Gate Compliance

四个 commit 构成两轮完整的 RED → GREEN：`be3cd35`(test) → `bcd7c27`(feat)、`dfa1524`(test) → `4f1bc13`(feat)。两轮的 RED 均实跑失败并留下失败输出（engine 侧 `Ok(Url{ password: Some(...) })`；前端侧 alert 未出现、页面显示「端点已保存。」），不存在「测试意外通过」的跳过。两轮均无需 REFACTOR 阶段——改动本身就是最小实现。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Task 1 fixture 的用户名从 `u` 改为 `prism-test-user`**

- **Found during:** Task 1（写 RED 测试时）
- **Issue:** 计划的 ①第一条 fixture 是 `https://u:prism-test-secret-value@api.vendor.com/v1`，用户名为单字符 `u`。而 ④ 要求断言「错误消息里不含用户名子串」——任何一句英文错误消息都必然含字母 `u`（`must`、`must not`…），这条断言无法成立，只能恒假。
- **Fix:** 用户名改为 `prism-test-user`，与密码 `prism-test-secret-value` 一样是可判别的长子串，④ 因此变成一条真断言。
- **Files modified:** `crates/prism-store/src/settings.rs`
- **Commit:** `be3cd35`

**2. [Rule 1 - Bug] Task 2 的 ③ 不回显断言范围从 `document.body.innerHTML` 收到 `role="alert"` 文案**

- **Found during:** Task 2（RED 实跑）
- **Issue:** 计划写的是「`document.body.innerHTML` 中不含 `FAKE_KEY` 那段子串」。实测 RED 输出显示端点输入框在 DOM 里带 `value="https://u:fixture-not-a-real-credential@api.vendor.com/v1"` 属性——它是 `type="text"` 的普通输入框，**必须**显示用户刚键入的内容。该断言在任何正确实现下都无法通过，写进去只会得到一条永远红的假断言。
- **Fix:** ③ 断言改为 `alert.textContent` 不含 `FAKE_KEY`，另加一条 `alert.textContent` 不含 `invalid_url_credentials` 码串。这两条守的正是这一层真正的泄漏面：**错误文案**不得回显输入（API key 栏是 `type="password"` 且提交后清空，那一面由 01-09 既有的 `never echoes the typed key back into the DOM` 守着）。
- **Files modified:** `src/pages/Settings.test.tsx`
- **Commit:** `dfa1524`

**3. [Rule 2 - 缺失的关键说明] 三处文档注释同步扩写**

- **Found during:** Task 1
- **Issue:** `validate_base_url` 与 `set_setting` 的文档注释都写着「`base_url` 一律先过 **scheme** 校验」，改动后与实际拒绝面（四类）不符。注释与机制脱节正是这个 gap 的成因之一。
- **Fix:** 两处注释改写为四类拒绝面，并在 `set_setting` 处补上「绕过界面直接 invoke 也改变不了结果」（T-01-43）。`validate_base_url` 处补上「值侧 vs 键名侧」的对照。`Settings.tsx` 的「这不是安全边界」注释按计划保留并补句。
- **Files modified:** `crates/prism-store/src/settings.rs`, `src/pages/Settings.tsx`
- **Commit:** `bcd7c27` / `4f1bc13`

### 未采纳的计划项

无。计划的两个 task、五条反证、全部源码断言均已执行。

## Known Stubs

无。本 plan 未引入任何占位实现、跳过的测试或未跑的 `<verify>`。

## 已知毛刺（不构成缺口，登记备查）

`https://api.example.com/v1?`（尾随空 `?`）与 `…/v1#`（尾随空 `#`）两种形态上，两侧判定有一根头发丝的偏差：Rust `url` crate 的 `query()` 对尾随 `?` 返回 `Some("")`，而 JS `URL` 会把空 `?` 规范化掉使 `search === ""`。后果是前端放行、engine 拒绝，用户看到的是通用 `invalid_url` 文案而不是凭据专属文案。**不是安全缺口**：值仍被 engine 拒绝、不入表，失败模式是文案不够精确而非放行。对应 T-01-42（severity: low）。若 Phase 4 要消除它，做法是前端改判 `trimmed` 里是否出现过 `?` / `#` 而不是看规范化后的 `search` / `hash`。

## Threat Flags

无。本 plan 未引入新的网络端点、认证路径、文件访问形态或信任边界处的 schema 变更——改动纯粹是既有写入路径上拒绝面的收窄。

## Prohibitions 复核

| Requirement | Statement | 状态 | 证据 |
|---|---|---|---|
| INFRA-03 | MUST NOT 让任何携带凭据的**值**进入 settings 表 | 成立 | 测试 ②③⑤ + 探针 `PROBE stored value = None`；反证 A/B 证明非恒真 |
| INFRA-03 | MUST NOT 在新增的凭据拒绝路径上回显被拒的值或其任何片段 | 成立 | 测试 ④（Rust 错误串）+ 前端 ③（alert 文案）；源码断言确认 `username()` / `password()` 不出现在任何 `format!` / 消息构造里 |

## Requirements Completed

- **INFRA-02**（SQLite/FTS 骨架）：本 plan 只间接触及（settings 写入事务），依赖图未变（`check-deps.sh all` 六条全 OK）。
- **INFRA-03**（密钥不入库）：**写入侧**恢复成立。另一半——静态扫描能否看见明文密钥——由 plan 01-11 关闭；在 01-11 完成前，INFRA-03 整体不应被当作已解除阻塞。

## Success Criteria

成功标准 4 第三分句「代码与配置中无明文密钥」的**写入侧**恢复成立：密钥容器的边界从键名扩到值，`docs/keychain-naming.md` 不变量 1 在 `llm.base_url` 这条路径上重新为真。

## Self-Check: PASSED

- `crates/prism-store/src/settings.rs` FOUND（含 `settings_base_url_rejects_credential_bearing_values`）
- `src/lib/ipc.ts` FOUND（含 `invalid_url_credentials`）
- `src/pages/Settings.tsx` FOUND（含 `localUrlIssue`，`looksLikeHttpUrl` 出现次数 0）
- `src/pages/Settings.test.tsx` FOUND（8 cases）
- commits `be3cd35` / `bcd7c27` / `dfa1524` / `4f1bc13` 全部存在于 `git log`
