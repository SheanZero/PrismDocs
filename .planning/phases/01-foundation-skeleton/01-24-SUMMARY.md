---
phase: 01-foundation-skeleton
plan: 24
subsystem: webview-security-policy-and-acl-assertions
tags: [security, csp, capabilities, allowlist, xss, gap-closure]
status: complete

requires:
  - "src-tauri/tauri.conf.json 的 csp / devCsp 双份策略（01-13 确立的分工：放宽只进 devCsp）"
  - "src/lib/tauri-security.test.ts 里 01-13 写下的 directiveSources 助手（签名未变，本 plan 复用）"
  - "src-tauri/capabilities/default.json 的两条 event 权限（01-09 确立的最小授予集）"
  - "01-REVIEW.md WR-01 / WR-02；01-REVIEW-prior.md WR-09"
provides:
  - "csp / devCsp 两份都关掉了 form-action 与 frame-ancestors —— 两条没有 default-src 兜底的指令"
  - "CSP 回归测试的判别力：六条承重指令逐条精确相等 + 指令名集合精确相等 + 'unsafe-inline' 落点精确相等"
  - "capability 授予集的精确相等断言：追加任何一条权限（含 core:event:allow-emit）都变红"
  - "12 + 5 条实测反证记录，含改动前/改动后的对照（改动前 7 条本该红的形态是绿的）"
affects:
  - "Phase 2+：新增任何 Tauri 前端 API 用法时，必须同时编辑 capabilities.test.ts 的期望数组——这就是评审检查点"
  - "Phase 3+：渲染外部 agent 写的 Markdown 时，一次「顺手加 'unsafe-inline'」的 CSP 削弱会当场变红"
  - "Phase 收尾人工验证第 1 项（真实 WebView 的 CSP）步骤需更新——详见文末，由 01-27 汇总"

tech-stack:
  added: []
  patterns:
    - "断言最小权限/最小面时用**精确相等**而非 denylist 过滤：denylist 的强度恰好等于写它的人当时的想象力，而要防的失败模式正是「没想到的那一种」"
    - "整串负向检查（not.toContain）与逐项精确相等是**互补**的：前者守「换个指令名把面重新打开」，后者守已知指令的取值；单用任一条都有结构性盲区"
    - "一条 not.toContain 在目标串本来就合法地含有该 token 时是永远失败的断言——改写成「含该 token 的项集合精确等于 [合法的那一项]」，可满足且强度更高"
    - "反证驱动器写成独立脚本（程序化改配置 → 跑测试 → git checkout 还原），避免手改配置时漏还原"

key-files:
  created: []
  modified:
    - src-tauri/tauri.conf.json
    - src/lib/tauri-security.test.ts
    - src/lib/capabilities.test.ts

decisions:
  - "plan 要求的 `expect(csp).not.toContain(\"unsafe-inline\")` 不可满足（发布 csp 的 style-src 合法地带着它，而 Task 1 又明令 style-src 逐字不动），改写成「带 'unsafe-inline' 的指令集合精确等于 [style-src 'self' 'unsafe-inline']」——可满足且顺带钉住 style-src 自身取值"
  - "额外新增指令名集合精确相等（④\"）：六条来源列表守不住 `script-src-elem https://evil` 这类换名绕过，而它在 CSP 里优先于 script-src 生效"
  - "额外新增 devCsp 的 form-action / frame-ancestors 精确相等（⑥'）：dev 形态是开发者每天面对的那一个，只在 build 后存在的边界等于在最需要观察它的窗口期里不存在"
  - "capabilities.test.ts 的两条 toContain **并入**精确相等而非并存（plan 给的二选一）：toEqual 的缺项 diff 已直接指出少了哪条，留着会让「授予集是什么」有两个真相源"
  - "capability 用顺序敏感的 toEqual 而不是先排序：顺序在 ACL 语义上无意义，但顺序敏感是免费的，且重排两条权限没有正当理由，出现即值得看一眼"

metrics:
  duration: ~25min
  tasks: 3
  files: 3
completed: 2026-07-29
---

# Phase 01 Plan 24: CSP 补齐无兜底指令 + 两份 denylist 断言改精确相等 Summary

给 `csp` 与 `devCsp` 各补上 `form-action 'none'` 与 `frame-ancestors 'none'`（这两条没有 `default-src` 兜底，不写就等于没设），并把守着它们的两份测试从 denylist 过滤改成逐项精确相等——改动前实测有 7 种本该变红的削弱形态是全绿的，改动后 17 条反证全部变红。

## What Was Built

### Task 1 —— 两份 CSP 各补两条无兜底指令（commit `424c42a`）

`src-tauri/tauri.conf.json` 的 `csp` 与 `devCsp` 末尾各追加 `form-action 'none'; frame-ancestors 'none'`。

改动前后的指令列表 diff（按 `; ` 切分逐条比对，脚本输出）：

```
csp:    added=["form-action 'none'","frame-ancestors 'none'"]  removed=[]  prefix-identical: true
devCsp: added=["form-action 'none'","frame-ancestors 'none'"]  removed=[]  prefix-identical: true
```

`prefix-identical: true` 的含义是：改动后列表的前 7 项与改动前的 7 项**逐条逐字相同且顺序相同**，两条新指令纯追加在末尾。没有任何既有来源列表被改动，也就没有任何放宽。

最终形态：

| # | csp | devCsp |
|---|---|---|
| 0 | `default-src 'self'` | `default-src 'self'` |
| 1 | `script-src 'self'` | `script-src 'self' 'unsafe-inline'` |
| 2 | `style-src 'self' 'unsafe-inline'` | `style-src 'self' 'unsafe-inline'` |
| 3 | `img-src 'self' data:` | `img-src 'self' data:` |
| 4 | `connect-src 'self' ipc: http://ipc.localhost` | `connect-src 'self' ipc: http://ipc.localhost ws://localhost:1420 http://localhost:1420` |
| 5 | `object-src 'none'` | `object-src 'none'` |
| 6 | `base-uri 'self'` | `base-uri 'self'` |
| 7 | **`form-action 'none'`**（新） | **`form-action 'none'`**（新） |
| 8 | **`frame-ancestors 'none'`**（新） | **`frame-ancestors 'none'`**（新） |

`form-action` 关掉的是「注入一个 `<form action="https://evil.example" method="POST">` 加自动提交」这条外泄通道——它绕开 `connect-src` / `img-src` / `script-src` 的全部收口。Phase 1 前端没有任何原生 `<form>` 提交（设置页是按钮 + `onClick`），`'none'` 不影响现有功能。`frame-ancestors` 是桌面 WebView 不该被任何东西嵌套的边界（clickjacking）。

### Task 2 —— CSP 回归测试改逐指令精确相等（commit `f3dff83`）

原断言③（四项 denylist 循环，第 44-55 行）整段替换。新形态：

**六条承重指令逐条 `toEqual`**，期望值逐字取自 `tauri.conf.json`：

| 指令 | 测试里的期望值 | `tauri.conf.json` 里的原文 |
|---|---|---|
| `script-src` | `["'self'"]` | `script-src 'self'` |
| `connect-src` | `["'self'", "ipc:", "http://ipc.localhost"]` | `connect-src 'self' ipc: http://ipc.localhost` |
| `object-src` | `["'none'"]` | `object-src 'none'` |
| `base-uri` | `["'self'"]` | `base-uri 'self'` |
| `form-action` | `["'none'"]` | `form-action 'none'` |
| `frame-ancestors` | `["'none'"]` | `frame-ancestors 'none'` |

**两条互补的整串级断言**：

- ④ `expect(csp).not.toContain("unsafe-eval")`（既有，未动）
- ④′ 带 `'unsafe-inline'` 的指令集合精确等于 `["style-src 'self' 'unsafe-inline'"]`（plan 要求的形式不可满足，改写理由见 Deviations 1）

**两条本 plan 加强的断言**（超出 plan 要求，理由见 Deviations 2 / 3）：

- ④″ `directiveNames(csp)` 精确等于九条已知指令名
- ⑥′ `devCsp` 的 `form-action` / `frame-ancestors` 同样精确等于 `["'none'"]`

新增 `directiveNames()` 助手；`directiveSources()` 签名与实现未动，只补了注释说明「取首个匹配」与浏览器语义一致（同名指令重复时浏览器只认第一条）。既有断言 ①②⑤⑥ **一条未删**。文件顶部注记保留，并补两段：一段说明「只加一个词的 diff 比 `csp: null` 更难在评审里被看见」，一段说明新补的两条指令同样只在真实 WebView 里有效果、Phase 收尾人工验证要覆盖。

### Task 3 —— capability 权限断言改精确相等（commit `c6d57a9`）

第 26-32 行的六前缀 denylist（`/^(fs|shell|http|dialog|core:webview|core:window):/`）替换成：

```ts
expect(capability.permissions).toEqual([
  "core:event:allow-listen",
  "core:event:allow-unlisten",
]);
```

期望值与 `src-tauri/capabilities/default.json` 的 `"permissions": ["core:event:allow-listen", "core:event:allow-unlisten"]` 逐字一致（含顺序）。原第 23-24 行的两条 `toContain` 并入这一条（plan 给的二选一，选择与理由写进注释）；`windows` 含 `main` 的断言与其上方解释原样保留。

## 十七条非恒真反证（全部实跑）

反证驱动器是两个临时脚本（程序化改配置 → 跑 vitest → `git checkout` 还原），置于 scratchpad，不进仓库。

### CSP：改动前 / 改动后对照

| # | 削弱形态 | 改动前（denylist） | 改动后（精确相等） | 落点断言 |
|---|---|---|---|---|
| 1 | `script-src` 追加 `'unsafe-inline'` | **exit=0 全绿** ❌ | exit=1 红 ✅ | `expected [ "'self'", "'unsafe-inline'" ] to deeply equal [ "'self'" ]` |
| 2 | `script-src` 追加 `https://cdn.evil.example` | **exit=0 全绿** ❌ | exit=1 红 ✅ | `expected [ "'self'", …(1) ] to deeply equal [ "'self'" ]` |
| 3 | `connect-src` → `*` | **exit=0 全绿** ❌ | exit=1 红 ✅ | `expected [ '*' ] to deeply equal [ "'self'", 'ipc:', …(1) ]` |
| 4 | `object-src` → `'self'` | **exit=0 全绿** ❌ | exit=1 红 ✅ | `expected [ "'self'" ] to deeply equal [ "'none'" ]` |
| 5 | `base-uri` → `*` | **exit=0 全绿** ❌ | exit=1 红 ✅ | `expected [ '*' ] to deeply equal [ "'self'" ]` |
| 6 | 删掉 `form-action` | n/a（改动前不存在这条指令） | exit=1 红 ✅ | `expected [] to deeply equal [ "'none'" ]` |
| 7 | 删掉 `frame-ancestors` | n/a（同上） | exit=1 红 ✅ | `expected [] to deeply equal [ "'none'" ]` |
| 8 | `csp` → `null`（既有断言①回归对照） | exit=1 红 | exit=1 红 ✅ | `expected 'object' to be 'string'` |

plan 预言改动前有 3 条全绿；实测是 **5 条**（`object-src 'self'` 与 `base-uri *` 也一样通过——denylist 只循环 `script-src` 的来源，其余四条承重指令根本没被检查）。

本 plan 额外新增的断言各配一条反证：

| # | 削弱形态 | 结果 | 落点断言 |
|---|---|---|---|
| 9 | 追加 `script-src-elem 'self' https://cdn.evil.example` | exit=1 红 ✅ | `expected [ 'default-src', 'script-src', …(8) ] to deeply equal [ …(7) ]`（④″） |
| 10 | `default-src` 追加 `'unsafe-inline'` | exit=1 红 ✅ | `expected [ …(2) ] to deeply equal [ Array(1) ]`（④′） |
| 11 | `devCsp` 删掉 `form-action` | exit=1 红 ✅ | `expected [] to deeply equal [ "'none'" ]`（⑥′） |
| 12 | `devCsp` 追加 `'unsafe-eval'` | exit=1 红 ✅ | `expected '…' not to contain 'unsafe-eval'`（既有⑥回归对照） |

反证 9 是 ④″ 存在的全部理由：`script-src-elem` 在 CSP 里优先于 `script-src` 生效，而六条来源列表的精确相等对它**一条都不会红**。

### capability：改动前 / 改动后对照

| # | 改动形态 | 改动前（六前缀 denylist） | 改动后（精确相等） | 落点断言 |
|---|---|---|---|---|
| 1 | 追加 `core:event:allow-emit` | **exit=0 全绿** ❌ | exit=1 红 ✅ | `expected [ 'core:event:allow-listen', …(2) ] to deeply equal [ …(1) ]` |
| 2 | 追加 `core:app:default` | **exit=0 全绿** ❌ | exit=1 红 ✅ | 同上形状 |
| 3 | 追加 `fs:default` | exit=1 红 | exit=1 红 ✅ | 同上形状（denylist 时代的回归对照） |
| 4 | 删掉 `core:event:allow-unlisten` | exit=1 红 | exit=1 红 ✅ | `expected [ 'core:event:allow-listen' ] to deeply equal [ …(1) ]` |
| 5 | `windows` → `["not-main"]` | exit=1 红 | exit=1 红 ✅ | `expected [ 'not-main' ] to include 'main'`（既有断言，保留） |

反证 1 就是上轮 WR-09 点名的那条：`core:event:allow-emit` 让 WebView 里的脚本能伪造 `prism://changed` 事件直接注进失效管线，而 `core:event:` 恰好不在六个前缀里（前缀表只列了 `core:webview` 与 `core:window`）。

全部 17 条反证还原后，`git status --porcelain` 中不含 `src-tauri/tauri.conf.json` 与 `src-tauri/capabilities/default.json`。

## Verification

```
$ npx vitest run
 Test Files  7 passed (7)
      Tests  75 passed (75)

$ npx tsc --noEmit
tsc: 0 error

$ node -e "require('./src-tauri/tauri.conf.json')"
JSON parse ok

$ cargo test -p prismdocs-shell --features test
test result: ok. 21 passed; 0 failed        (lib)
test result: ok. 2 passed; 0 failed         (tests/ipc.rs)

$ git status --porcelain
（只剩两个先于本 plan 存在的 .planning/research/.cache/ 未跟踪文件）
```

`cargo test -p prismdocs-shell` 是 CSP 变更的回归对照：进程内 IPC 测试不读 `tauri.conf.json`，两条新指令确实没有波及它（21 + 2 全绿，与 01-21 收尾时的计数一致）。

## CSP 人工验证步骤更新（供 01-27 汇总）

本 plan 给两份 CSP 各加了 `form-action 'none'` 与 `frame-ancestors 'none'`。Phase 收尾**人工验证第 1 项**（真实 WebView 下的 CSP 与 IPC 双通路）在既有五步之外需补一步：

- 打开 WebView 控制台确认无 CSP 违规报告时，**额外确认这两条新指令没有在设置页或冒烟页触发违规**。预期是不触发——Phase 1 前端不含任何原生 `<form>` 提交，桌面窗口也不会被嵌套。若设置页出现 `Refused to send form data ... form-action` 一类报告，说明某处存在未被识别的原生表单提交，属于本 plan 引入的回归，需要把该指令放宽到 `'self'` 并同步改 `tauri-security.test.ts` 的期望值（那次改动过一遍评审，正是这条断言存在的意义）。

这条与 WINDOWS ledger 里 id=8（01-13 的 `<human-check>` 五步未执行）是同一项人工验证，本 plan 只是把它的步骤扩到七步，未新开一条。

## Deviations from Plan

### 1. [Rule 1 - Bug] plan 要求的 `expect(csp).not.toContain("unsafe-inline")` 是一条不可满足的断言

- **Found during:** Task 2
- **Issue:** plan 的 `<action>` 写「额外保留两条整串级断言：`csp` 不含 `unsafe-eval`；`csp` 不含 `unsafe-inline`（新增——发布形态一个都不许）」。但发布 `csp` 的 `style-src 'self' 'unsafe-inline'` 合法地带着这个 token（React 的内联 style 属性需要它），而同一 plan 的 Task 1 又明令「`style-src` 的取值逐字保持」。两条要求互斥：照 plan 字面写下去，这条断言在**当前**配置下就是红的，而唯一能让它变绿的做法是改 `style-src`——那正是 Task 1 禁止的。
- **Fix:** 改写成「`csp` 里含 `'unsafe-inline'` 的指令集合精确等于 `["style-src 'self' 'unsafe-inline'"]`」。它保住了 plan 的原意（守住「有人换了个写法把它塞进别的指令」），可满足，且强度更高——它顺带钉住了 `style-src` 自身的完整取值。反证 10（`default-src` 追加 `'unsafe-inline'`）实测变红，证明改写没有丢掉判别力。注释里写明为什么不能写成 `not.toContain`。
- **Files modified:** `src/lib/tauri-security.test.ts`
- **Commit:** `f3dff83`

### 2. [Rule 2 - Missing] 六条来源列表的精确相等守不住「换个指令名把面重新打开」

- **Found during:** Task 2
- **Issue:** plan 的六条精确相等覆盖了六个**已知**指令名。CSP 里 `script-src-elem` / `script-src-attr` / `style-src-elem` 这些指令在存在时**优先于**对应的 `script-src` / `style-src` 生效，而追加一条 `script-src-elem 'self' https://cdn.evil.example` 不会让六条精确相等里的任何一条变红——`directiveSources(csp, "script-src")` 用 `=== name || startsWith(name + " ")` 匹配，不会误命中它。这是本 plan 想关掉的那个失败模式（「denylist 的强度等于写它的人当时的想象力」）在白名单侧的同构版本：白名单只列了六条，第七条无声通过。
- **Fix:** 新增断言 ④″：`directiveNames(csp)` 精确等于九条已知指令名的有序数组。新增一条合法指令时必须编辑这个数组，那正是想要的评审检查点。配反证 9 实测变红。
- **Files modified:** `src/lib/tauri-security.test.ts`
- **Commit:** `f3dff83`

### 3. [Rule 2 - Missing] `devCsp` 的两条新指令原本无人看守

- **Found during:** Task 2
- **Issue:** Task 1 要求 `csp` 与 `devCsp` **两份**都补上 `form-action` / `frame-ancestors`，但 plan 的 Task 2 只要求 `csp` 侧的精确相等。于是 `devCsp` 的这两条一旦被删，两个 Task 的验收都不会红——只有 Task 1 那条一次性的 `node -e` 检查覆盖它，而那条不进测试套件。
- **Fix:** 新增断言 ⑥′：`devCsp` 的 `form-action` / `frame-ancestors` 同样精确等于 `["'none'"]`。理由写进注释：dev 形态是开发者每天面对的那一个，一条只在 build 后才存在的边界，等于在最需要观察它的窗口期里不存在。配反证 11 实测变红。
- **Files modified:** `src/lib/tauri-security.test.ts`
- **Commit:** `f3dff83`

### 4. [Rule 2 - Missing] plan 预言的「改动前 3 条全绿」实测为 5 条

- **Found during:** Task 2 的改动前基线测量
- **Issue:** plan（转述 01-REVIEW.md WR-01）说 verifier 复算出三种通过 denylist 的削弱形态。实测把 `<behavior>` 里全部 5 种在改动前可施加的形态都跑了一遍，结果是 5 条全绿——`object-src 'self'` 与 `base-uri *` 也一样通过。根因比 WR-01 描述的更彻底：denylist 循环只遍历 `script-src` 的来源列表，`connect-src` / `object-src` / `base-uri` 三条指令从头到尾**根本没被读取**。
- **Fix:** 无需修改实现（精确相等本来就覆盖这五条）；把实测出的更大缺口写进 SUMMARY 的对照表与测试注释，替代 plan 里的「三种」说法。
- **Files modified:** `src/lib/tauri-security.test.ts`（注释）
- **Commit:** `f3dff83`

### 未做的事（scope boundary）

- **未跑 `cargo fmt`**：本仓库有早于本 plan 的 rustfmt 漂移且无 CI fmt 闸门（`deferred-items.md` 已登记，由 01-28 关闭）。本 plan 未新增任何 Rust 代码。
- **未触碰同波次其他 plan 的文件**：`src-tauri/src/lib.rs`（01-21）、`src/pages/Settings.tsx`（01-23）一字未改。
- **未改 `src-tauri/capabilities/default.json` 的授予集**：本 plan 只加强守着它的断言，一条权限未增未减。
- **未验证 CSP 在真实 WebView 里的效果**：jsdom 与 `cargo test` 的 `mock_builder` 都结构性看不见 CSP，这是 Phase 收尾人工验证的范围（WINDOWS id=8）。

## Known Stubs

无。本 plan 未引入任何硬编码空值、占位文本或未接数据源的组件。

## Threat Flags

无。本 plan 只**收紧**了 WebView 的加载面（两条新增的 `'none'` 指令），并加强了守着它的断言，未引入新的网络端点、认证路径、文件访问模式或信任边界上的 schema 变更。plan 的 `<threat_model>` 里 T-01G-36 至 T-01G-40 五条 `mitigate` 全部落地并各配实测反证。

## Self-Check: PASSED

```
$ for f in src-tauri/tauri.conf.json src/lib/tauri-security.test.ts src/lib/capabilities.test.ts; do
    [ -f "$f" ] && echo "FOUND: $f" || echo "MISSING: $f"; done
FOUND: src-tauri/tauri.conf.json
FOUND: src/lib/tauri-security.test.ts
FOUND: src/lib/capabilities.test.ts

$ for h in 424c42a f3dff83 c6d57a9; do
    git log --oneline --all | grep -q "$h" && echo "FOUND: $h" || echo "MISSING: $h"; done
FOUND: 424c42a
FOUND: f3dff83
FOUND: c6d57a9
```
