---
phase: 01-foundation-skeleton
plan: 09
subsystem: frontend
tags: [tanstack-query, notify-then-fetch, tauri-acl, capability, listen-cleanup, hash-route, keychain-ui, fts-cjk, seed-data, human-verification]

# Dependency graph
requires:
  - "01-01：src-tauri 薄壳、`src/lib/ipc.ts` 的 devPing、vitest 基座"
  - "01-04：prism-types 的 EngineEvent（serde tag=kind + camelCase）"
  - "01-05：prism-store 的 settings / search（FTS5 trigram，4 字中文可命中）"
  - "01-07：Engine facade（search / get_setting / set_base_url / set_api_key / api_key_status / delete_api_key / publish）"
  - "01-08：八个命令、`prism://changed` 事件名、Channel 有序流 `dev_smoke_stream`、错误短码"
provides:
  - "queryClient —— TanStack Query 单例（staleTime 0 + refetchOnWindowFocus false），D-07 的前端数据层惯例起点"
  - "useEngineInvalidation —— listen('prism://changed') → invalidateQueries；resync 走无参全量失效；**返回失败文案**（listen 被 ACL 拒时不再静默）"
  - "src/lib/ipc.ts —— 十个命令的类型化封装 + EngineEvent / SmokeEvent / SearchHit 契约类型 + errorCopy（短码 → 中文，未知码走通用兜底）"
  - "SettingsPage —— 密钥只显示「已配置/未配置」+ 删除；base_url 前端提示 + engine 权威校验"
  - "DevSmokePage —— 三个验证入口（事件 1:1 计数 / Channel total=1000 逐位校验 / 中文 FTS + 阴性对照 + 一键播种）与纯函数 verifySmokeStream"
  - "src-tauri/capabilities/default.json —— 最小权限 capability（只放行 core:event listen/unlisten）"
  - "prism_store::seed —— 幂等样例数据（SAMPLE_PROJECT_ID / SAMPLE_DOCS / insert_samples）"
  - "命令 delete_api_key / dev_seed_sample_docs（八个 → 十个）"
affects: [phase-2-文档树与导入UI, phase-3-锚点UI, phase-4-LLM设置页, phase-6-MCP设置页]

# Tech tracking
tech-stack:
  added:
    - "@tanstack/react-query（前端数据层，D-07 的最终模式，Phase 1 即引入不留返工）"
    - "src-tauri/capabilities/（Tauri v2 ACL manifest，本 plan 首次引入到本项目）"
  patterns:
    - "`listen()` 的 Promise **两个方向都要接住**：`pending.catch(...)` 报错 + cleanup 用两参 `then(ok, noop)`——单参 `.then(un => un())` 在 listen 失败时自己也是未处理的 rejection"
    - "Tauri v2 的 ACL **只管插件命令**：`generate_handler!` 注册的自有命令不过 ACL。于是「invoke 全部正常 + listen 全部被拒」是一个真实且高度误导的状态"
    - "可达性是一条独立于路由逻辑的性质：「hash 是 X 时渲染谁」全绿，不代表用户到得了 X"
    - "`import.meta.env.DEV` 门控 dev-only UI：vite build 下替换为字面 false，整块被摇掉——是「不存在」而不是「藏起来」，可用 grep dist 产物证伪"
    - "前端只渲染译文，**从不把错误短码放进 DOM**；未知码走通用兜底而不是 `String(err)`"
    - "空库上的搜索断言恒为 0：中文 FTS 的验收必须先有样例数据当分母，且样例内容刻意不含阴性对照词"

key-files:
  created:
    - src/lib/queryClient.ts
    - src/lib/useEngineInvalidation.ts
    - src/lib/useEngineInvalidation.test.ts
    - src/lib/capabilities.test.ts
    - src/pages/Settings.tsx
    - src/pages/Settings.test.tsx
    - src/pages/DevSmoke.tsx
    - src/pages/DevSmoke.test.tsx
    - src/App.test.tsx
    - src-tauri/capabilities/default.json
    - crates/prism-store/src/seed.rs
  modified:
    - src/App.tsx
    - src/main.tsx
    - src/lib/ipc.ts
    - src-tauri/src/commands.rs
    - src-tauri/src/lib.rs
    - src-tauri/tests/ipc.rs
    - crates/prism-engine/src/facade.rs
    - crates/prism-store/src/lib.rs

key-decisions:
  - "capability 文件是**源码**并纳入版本控制（`src-tauri/gen/schemas/` 才是 gitignore 的生成物）；只授 `core:event:allow-listen` + `allow-unlisten`，无 fs/shell/http——最小权限在这里不是洁癖，capability 一旦放宽，越权的表现同样是「没有任何症状」"
  - "`listen` 的失败必须冒到界面：`useEngineInvalidation` 从 `void` 改为返回 `string | null`，由 App 渲染成一行 `role=alert`。失效链路死掉的表现是「数据一直是旧的」，与「数据本来就没变」在界面上完全同形"
  - "为 ACL 拒绝自造一个前端短码 `listen_failed` 而不是走 `errorCopy` 的通用兜底：兜底文案「操作失败，请重试」会把「本页从此收不到任何事件」说成一次可重试的操作失败"
  - "dev 路由开关用 `import.meta.env.DEV` 而不是任何运行期开关：D-06 的「不放导航入口」是**正式产品外观**的承诺，不是「不可达」——摇掉比藏起来强"
  - "capability 的存在性断言用**静态 import** 而不是 `fs.existsSync`：省掉 `@types/node`，且文件被删时同时炸在 `tsc --noEmit` 与 vitest 两处"
  - "样例 project id 由 `dev_seed_sample_docs` 的**返回值**交给前端，前端不另抄常量（两份必然漂移）"
  - "播种走 `ON CONFLICT DO UPDATE` 而非先 DELETE 再 INSERT：前者触发 `documents_au` 的删增两半、FTS 照常跟上"
  - "**勾选 INFRA-01、不勾选 INFRA-03**：前者的最后一块（成功标准 2 的真实 WebView 两条通路）在本 plan 由人工验证兑现；后者的「支持 Anthropic/OpenAI 兼容端点」在 Phase 4——`crates/prism-llm/src/` 目前只有 `lib.rs` 与 `secrets.rs`，没有任何 chat client。见 Requirements 一节"

patterns-established:
  - "任何新的 `@tauri-apps/api` 用法（fs / dialog / window / webview / http…）都需要在 `capabilities/default.json` 里补一行，否则**静默无操作**"
  - "新增命令仍是三处清单（commands.rs / lib.rs generate_handler! / tests/ipc.rs），本 plan 已按此加了两条"
  - "前端新增查询：query key 用 `['docs', projectId]` / `['settings', key]` / `['apiKeyStatus']`；失效由事件驱动，不写 refetchInterval"

requirements-completed: [INFRA-01]

coverage:
  - id: D1
    description: "notify-then-fetch 前端半边：coarse event → invalidateQueries；resync 走无参全量失效；cleanup 在 StrictMode 双执行下仍只留一个 listener"
    requirement: "INFRA-01"
    verification:
      - kind: unit
        ref: "`npm run test -- --run useEngineInvalidation` → 5 passed（事件名契约 / docChanged 精确失效 / resync 无参 / 卸载后不再失效 / 双执行后 listener 恰好 1 个）"
        status: pass
      - kind: other
        ref: "反证：把 effect 的返回值改成 undefined → 落在两条 `handlers.size` 断言上（1 → 2），其余三条保持绿；恢复后逐字节一致"
        status: pass
    human_judgment: false
  - id: D2
    description: "总线事件在真实 WebView 中 1:1 往返（成功标准 2 的人工半边之一）"
    requirement: "INFRA-01"
    verification:
      - kind: manual
        ref: "用户重启 dev 会话后实测并报告：「计数正常跳动，1:1」——每次点击计数恰好 +1；离开该页再回来后计数不翻倍，说明 cleanup 在真实 runtime 下也生效。（这是六项人工验证中**唯一**带具体描述的一项）"
        status: pass
      - kind: unit
        ref: "`DevSmokePage > counts one bus event per click, 1:1`（mock runtime 侧的同口径断言）"
        status: pass
    human_judgment: true
    rationale: "mock runtime 没有 WebView，也没有 ACL——这一项的两个真实失败模式（事件根本没出界、listen 被 ACL 拒）都只在真实应用里存在。事实证明：第二个失败模式确实发生了（见「人工验证抓到的两个缺陷」）。"
  - id: D3
    description: "Channel 有序流在真实 WebView 中 total=1000 严格单调无缺口（成功标准 2 的人工半边之二）"
    requirement: "INFRA-01"
    verification:
      - kind: unit
        ref: "`verifySmokeStream` 四条纯函数断言（正例 / 缺口并指出位置 / 乱序但集合完整 / 末条不是 Finished）+ `DevSmokePage > streams with total=1000 and reports the seq verdict`"
        status: pass
      - kind: manual
        ref: "用户在真实应用中执行并报告通过。**记录在案的是用户的通过判定，不是冒烟页显示的确切文案与实收条数**——计划要求记录后者，用户未报"
        status: pass
    human_judgment: true
  - id: D4
    description: "中文 FTS 在真实应用中可用：4 字中文词命中 > 0，阴性对照 0 命中"
    requirement: "INFRA-02"
    verification:
      - kind: unit
        ref: "`prism-store::seed::tests` 三条：「锚定引擎」命中 > 0、「量子纠缠」0 命中、重复播种不产生重复文档；`DevSmokePage > reports hit counts for a Chinese query and zero for the negative control`"
        status: pass
      - kind: manual
        ref: "用户在真实应用中执行并报告通过。**记录在案的是用户的通过判定，不是两个查询各自的命中条数**——计划要求记录后者，用户未报"
        status: pass
    human_judgment: true
  - id: D5
    description: "settings 页可写入 API key 到系统钥匙串并读回状态，可删除；密钥绝不回显"
    requirement: "INFRA-03"
    verification:
      - kind: unit
        ref: "`Settings.test.tsx` 五条：已配置/未配置文案、输入 `type=password` 且渲染输出不含被输入的密钥串、钥匙串失败译成中文"
        status: pass
      - kind: manual
        ref: "用户在真实应用与真实钥匙串上执行并报告通过。**记录在案的是用户的通过判定，不是所见的状态文案，也不是钥匙串条目的 service/account 字符串**——计划要求记录后者，用户未报"
        status: pass
    human_judgment: true
    rationale: "计划原文：「记『已通过』三个字不算，那正是这个检查点要防的东西」。用户确实执行了验证，但回报的是判定而非读数——如实记为「通过，读数未捕获」，不把预期值当作观测值写进来。"
  - id: D6
    description: "非法 base_url（`file:///etc/passwd`）被拒且给出可读中文提示，不回显短码"
    requirement: "INFRA-03"
    verification:
      - kind: unit
        ref: "`Settings.test.tsx > renders the engine's rejection as Chinese copy and no success notice`（同时断言成功提示未出现）"
        status: pass
      - kind: manual
        ref: "用户执行并报告通过。**未捕获所见的确切错误文案**——同 D5 的读数完整性口径"
        status: pass
    human_judgment: true
  - id: D7
    description: "无 API key 时应用照常启动并可用（D-06 / D-16a 的可跳过要求）"
    requirement: "INFRA-03"
    verification:
      - kind: unit
        ref: "`Settings.test.tsx > stays fully usable with no key configured`"
        status: pass
      - kind: manual
        ref: "用户在未配置任何密钥的状态下启动应用并报告通过（应用正常启动、冒烟项可用）"
        status: pass
    human_judgment: true
  - id: D8
    description: "冒烟页在真实 Tauri 窗口中可达（无地址栏）"
    requirement: "INFRA-01"
    verification:
      - kind: unit
        ref: "`App.test.tsx` 四条：默认路由下存在可点入口且点击后到达 #/dev、冒烟页上存在返回控件、生产构建下两个控件都不存在"
        status: pass
      - kind: build
        ref: "`npm run build` 后 `grep -c 'dev 冒烟页\\|dev-route-entry\\|dev-route-back' dist/assets/*.js` → 0（三处标识串在正式产物中均不存在）"
        status: pass
      - kind: manual
        ref: "用户在真实应用里进入了冒烟页并完成 1:1 计数验证（D2）——可达性因此在真实 runtime 下也成立"
        status: pass
    human_judgment: true
  - id: D9
    description: "Tauri v2 ACL：主窗口获授 core:event 的 listen/unlisten，且不越权"
    requirement: "INFRA-01"
    verification:
      - kind: unit
        ref: "`capabilities.test.ts`：windows 含 `main`、permissions 含两条 `core:event:allow-*`、无 `fs|shell|http|dialog|core:webview|core:window` 前缀的授权"
        status: pass
      - kind: other
        ref: "反证：删掉 `src-tauri/capabilities/default.json` → `npx tsc --noEmit` 与 vitest **同时**变红（落在静态 import 的模块解析）"
        status: pass
      - kind: integration
        ref: "`cargo test -p prismdocs-shell --features test --test ipc` → 2 passed —— 引入 app ACL manifest 后 ipc 测试**未**变红（01-08 交接的第 3 点担忧在本次配置下未兑现）"
        status: pass
    human_judgment: false
  - id: D10
    description: "listen 的 rejection 不再被吞：两处 listen 站点都把失败呈现为中文文案"
    requirement: "INFRA-01"
    verification:
      - kind: unit
        ref: "`DevSmokePage > surfaces a rejected listen instead of silently sitting at zero`（断言页面出现中文提示且不含原始 ACL 英文文本）"
        status: pass
      - kind: other
        ref: "反证（修复前的实际状态）：该测试落在 `findByRole('status')` 找不到元素，vitest 另行报出 Unhandled Rejection"
        status: pass
    human_judgment: false

# Metrics
duration: 81min（含人工验证等待；agent 净时间约 16min）
completed: 2026-07-29
status: complete
---

# Phase 01 Plan 09: settings 页与 dev 冒烟页 Summary

**D-07 的 TanStack Query 最终模式（coarse event → invalidateQueries → refetch）与 D-06 的两块前端落地；而本 plan 最有价值的产出不是这些代码——是人工验证抓到的两个缺陷：冒烟页在真实 Tauri 窗口里根本到不了，以及 `listen()` 被 Tauri v2 ACL 拒绝且拒绝被完全吞掉。两者在 32 项前端单测 + 121 项 Rust 测试全绿的情况下都不可见。**

## Performance

- **Duration:** 约 81 min 挂钟时间（09:48 → 11:09），其中 agent 净工作约 16 min，其余是两轮人工验证与其间的等待
- **Tasks:** 2 个自动 task（各按 TDD 走 RED→GREEN）+ 1 个人工检查点（六项全通过）+ 2 轮由检查点驱动的修复
- **Files created/modified:** 19（新建 11，修改 8）
- **测试增量:** 前端 3 → **32 passed**（6 files）；workspace 118 → **121 passed**

## 人工验证结果（Task 3）

**六项全部由用户在真实应用中执行并报告通过。Task 3 完成。**

| # | 验证项 | 状态 |
|---|--------|------|
| 1 | 总线事件往返，计数 1:1 | **通过** — 用户报「计数正常跳动，1:1」：每次点击计数恰好 +1；离开该页再回来后计数不翻倍，cleanup 在真实 runtime 下成立 |
| 2 | Channel 有序流「seq 校验通过 · 实收 1000 条」 | **通过** |
| 3 | 插入样例文档 → 「锚定引擎」命中 > 0、「量子纠缠」= 0 | **通过** |
| 4 | API key 往返 + 钥匙串条目 + 删除 | **通过** |
| 5 | 非法端点 `file:///etc/passwd` 被拒并给出中文提示（非短码原文） | **通过** |
| 6 | 无 API key 时应用照常启动并可用（D-16a / D-06） | **通过** |

### 关于证据的一处如实说明

计划的检查点原文要求记录**看到的具体数字与文案**（计数器读数、实收条数、命中条数、钥匙串条目的 service/account），并写着「记『已通过』三个字不算，那正是这个检查点要防的东西」。

实际情况是：用户确实执行了六项并给出通过判定，但**除第 1 项外没有回报具体读数**。所以本表记录在案的是**用户的通过判定**，不是这些读数。计划的预期值（「seq 校验通过 · 实收 1000 条」「service `PrismDocs` / account `llm_api_key`」等）**刻意不写进上表**——把预期值当作观测值抄进来，正是这个检查点要防的那件事，只是换了个更隐蔽的形式。

自动化侧的对应断言是齐的（见 coverage D3–D7 的 unit 条目：`verifySmokeStream` 四条纯函数、seed 三条含阴性对照、Settings 五条含「密钥不回显 DOM」与「引擎拒绝译成中文且成功提示未出现」）。人工那一半覆盖的是自动化证明不了的部分：真实 WebView 与真实登录钥匙串。

## 人工验证抓到的两个缺陷

这是本 plan 最该被记住的部分。两个缺陷都不是「测试没写够」——它们所在的性质，**当时的测试形态在结构上就看不见**。

### 缺陷 1：冒烟页在真实 Tauri 窗口里完全到不了

- **被谁抓到：** 用户执行 Task 3 第 1 步时报告的阻塞。
- **当时的自动化状态：** 全绿。`App.test.tsx` 之前的两条断言是「hash 等于 `#/dev` 时渲染 DevSmokePage」「默认 hash 渲染 SettingsPage」——**路由逻辑本身没有任何问题**，这两条断言在冒烟页永远够不着的世界里同样全绿。
- **真实原因：** 计划的检查点指令写着「靠地址栏进入」，而 **Tauri 桌面窗口没有地址栏**。jsdom 里测试可以直接 `window.location.hash = '#/dev'`，用户不能。
- **这暴露的性质差别：** 「hash 是 X 时渲染谁」是**路由正确性**；「用户到得了 X 吗」是**可达性**。前者绿不蕴含后者。这类缺口的通用形状是：*测试替用户完成了用户其实做不到的那一步*。
- **修复：** `App.tsx` 挂一个 `import.meta.env.DEV` 门控的角落开关（设置页上是「dev 冒烟页」，冒烟页上是「← 设置」），点击改 `location.hash` 走原有 hashchange 通路，不引入第二套路由。D-06 的「不放导航入口」是正式外观的承诺——vite build 下 `import.meta.env.DEV` 被替换成字面 `false`，整块被摇掉，`dist/` 中三处标识串 grep 计数为 0，是「不存在」而非「藏起来」。
- **新增断言：** `App.test.tsx` 四条——默认路由下存在可点入口且点击后到达 `#/dev`、冒烟页上有可点返回控件、生产构建（`DEV=false`）下两个控件都不存在。
- **Commits:** `2bb765f`（test）/ `4108d44`（fix）

### 缺陷 2：`listen()` 被 Tauri v2 ACL 拒绝，而拒绝被完全吞掉

- **被谁抓到：** 用户进入冒烟页后点「触发总线事件」，**计数停在 0，且界面上没有任何错误**。
- **当时的自动化状态：** 全绿，且**结构上不可能发现**：vitest 里 `@tauri-apps/api/event` 是被 mock 的，mock 不会有 ACL。
- **真实原因（两层，缺一不可）：**
  1. `src-tauri/capabilities/` 目录**根本不存在**，于是 `gen/schemas/capabilities.json` 编译成 `{}`。Tauri v2 的 ACL **只管插件命令**：`generate_handler!` 注册的自有命令（`dev_ping` / `dev_emit_bus_event` / …）不过 ACL，照常成功且不报错；而 `listen()` 走 `plugin:event|listen`，被拒。**发射方一切正常、监听方从未注册。**
  2. 两处 `listen` 站点的 rejection 都无人接手——`const pending = listen(...)` 没有 `.catch`，cleanup 的 `pending.then(un => un())` 自己也会 reject。整类失败落进未处理的 Promise，界面零变化。
- **为什么这个组合特别恶劣：** 症状是「点了没反应，也没有任何错误」，而「计数为 0」在一切正常的状态下同样成立——**这个观测量指不出问题**。对一个把「锚点 0 静默丢失」写进发布门槛的项目，这正是最不该存在的失败形态。
- **修复（两条独立的线）：**
  1. 新增 `src-tauri/capabilities/default.json`，**只**授 `core:event:allow-listen` + `core:event:allow-unlisten`（前端对 `@tauri-apps/api` 的全部用法只有 event 的 listen/unlisten、自有命令 invoke 与 Channel，后两者不过 ACL）。无 fs/shell/http/dialog 授权。窗口标签 `main` 是 Tauri 默认值——**授权挂错标签等于没授权，而症状与文件整个缺失完全相同**。
  2. 两处 listen 站点显式接住 rejection：`useEngineInvalidation` 从 `void` 改为返回 `string | null`，由 `App` 渲染成一行 `role=alert`；`DevSmoke` 复用既有 notice。cleanup 一律改两参 `then(ok, noop)`——单参写法在 listen 失败时自己也是一个未处理的 rejection。
  3. `ipc.ts` 新增前端自造短码 `listen_failed` 与中文文案。ACL 拒绝时 reject 出来的是一段原始英文 ACL 文本（`event.listen not allowed. Permissions associated with this command: …`），它既不该进 DOM，也不该走 `errorCopy` 的通用兜底——兜底文案「操作失败，请重试」会把「本页从此收不到任何事件」说成一次可重试的操作失败。
- **新增断言与各自的反证：**
  - `capabilities.test.ts`（静态 import capability JSON）→ 反证：删掉该文件后 `tsc --noEmit` 与 vitest **同时**变红，落在模块解析这一步。用静态 import 而不是 `fs.existsSync` 正是为了拿到这个「两处同时红」。
  - `DevSmokePage > surfaces a rejected listen instead of silently sitting at zero` → 反证（即修复前的实际状态）：落在 `findByRole('status')` 找不到元素，vitest 另行报出 Unhandled Rejection。
- **Commit:** `2fc002a`

### 两个缺陷的共同形状

它们都不是逻辑错误，而是**测试替被测系统假设掉了一个前置条件**：缺陷 1 里 jsdom 替用户完成了「输入 hash」，缺陷 2 里 mock 替运行时完成了「ACL 放行」。单测越是把环境抽象掉，这类缺口越隐形——而它们的症状恰好都是「什么都没发生，也没有报错」。

这与 01-06 那条教训（「被测层之上若有第三方 backstop，反证会被掩盖」）和 01-08 那条（「平行的拒绝路径与被测失败模式共享错误文本」）是同一族问题的第三、第四个变种。共同的解药只有一个：**把被测性质放进一个没有替身的链路里跑一次**——这次是人。

## Task Commits

1. **Task 1: TanStack Query 客户端与 useEngineInvalidation hook** — TDD 两段：
   - `4aab092` (test) — RED：五条断言（事件名契约 / docChanged 精确失效 / resync 无参全量 / 卸载后不再失效 / StrictMode 式双执行后 listener 恰好 1 个）。落点：`Failed to resolve import "./useEngineInvalidation"`
   - `f6ce795` (feat) — GREEN：`queryClient.ts` / `useEngineInvalidation.ts` / `ipc.ts` 八命令封装与三个契约类型 / `main.tsx` 改用共享 client，5 passed。反证（去掉 cleanup）落在两条 `handlers.size` 断言上
2. **Task 2a: 样例数据播种与密钥删除命令（Rust 侧前置）** — TDD 两段：
   - `832466e` (test) — RED：seed 三条 + ipc.rs 三处清单各加两条。落点：`E0432 unresolved imports super::{insert_samples, SAMPLE_DOCS, SAMPLE_PROJECT_ID}`；`E0433 cannot find __cmd__dev_seed_sample_docs`
   - `3220738` (feat) — GREEN：`prism-store::seed` / `Engine::seed_sample_docs`（返回样例 project id，写后广播 Resync）/ 两个命令，workspace 118 → 121 passed
3. **Task 2b: settings 页与 dev 冒烟页** — TDD 两段：
   - `6a335dd` (test) — RED：Settings 七条 + DevSmoke 九条。落点：`Failed to resolve import "./Settings" / "./DevSmoke"`
   - `1b02b2c` (feat) — GREEN：两个页面 + `errorCopy` + hash 路由 + 顶层挂一次 `useEngineInvalidation`，24 passed / tsc 0 / build 0
4. **Task 3 人工检查点驱动的两轮修复：**
   - `2bb765f` (test) + `4108d44` (fix) — 缺陷 1：冒烟页可达性
   - `2fc002a` (fix) — 缺陷 2：ACL capability + listen 失败的显式呈现

**Plan metadata:** 见本 commit（docs: complete plan）

## Files Created/Modified

### 前端

- `src/lib/queryClient.ts`（新，15 行）— `QueryClient` 单例，`staleTime: 0` + `refetchOnWindowFocus: false`
- `src/lib/useEngineInvalidation.ts`（新，52 行）— listen → invalidateQueries；resync 无参全量；返回失败文案
- `src/lib/ipc.ts`（改，127 行）— 十个命令封装、`EngineEvent` / `SmokeEvent` / `SearchHit`、`EVENT_CHANGED`、`SETTING_BASE_URL`、`ERROR_COPY` / `errorCopy` / `LISTEN_FAILED`
- `src/pages/Settings.tsx`（新，195 行）— 密钥状态（「已配置 / 未配置」，读取中显示「读取中…」）、`type="password"` 输入且保存后清空、删除密钥、base_url（前端 http/https 提示 + engine 权威校验）
- `src/pages/DevSmoke.tsx`（新，268 行）— 三个验证入口 + 纯函数 `verifySmokeStream`（逐位比较，`SMOKE_TOTAL = 1000`）
- `src/App.tsx`（改，88 行）— hash 路由、顶层挂一次 `useEngineInvalidation`、失败 `role=alert` 条、dev-only `DevRouteToggle`
- `src/main.tsx`（改，13 行）— 改用共享 `queryClient`
- 测试：`useEngineInvalidation.test.ts`(124) / `capabilities.test.ts`(33) / `Settings.test.tsx`(141) / `DevSmoke.test.tsx`(235) / `App.test.tsx`(95)

### src-tauri

- `capabilities/default.json`（新，7 行）— 最小权限 capability
- `src/commands.rs`（改）— `delete_api_key` / `dev_seed_sample_docs` 两行单行委托
- `src/lib.rs`（改）— `generate_handler!` 八条 → 十条
- `tests/ipc.rs`（改）— 三处清单各加两条

### engine

- `crates/prism-store/src/seed.rs`（新，145 行）— `SAMPLE_PROJECT_ID` / `SAMPLE_DOCS` / `insert_samples`（幂等）；3 个单测
- `crates/prism-store/src/lib.rs`（改）— 挂 `seed` 模块
- `crates/prism-engine/src/facade.rs`（改）— `seed_sample_docs`（返回样例 project id，写后 publish Resync）

## Decisions Made

1. **capability 文件纳入版本控制，且只授两条权限。** `src-tauri/gen/schemas/` 是 gitignore 的生成物，capability 是源码。放宽授权的代价与缺失授权对称——两者的症状都是「没有任何症状」，所以最小权限在这里是可验证性的一部分，不是洁癖。
2. **`useEngineInvalidation` 的签名从 `void` 改成 `string | null`。** 这是本 plan 唯一一处对 01-RESEARCH 给定实现的偏离，理由写在 D10：失效链路死掉的界面表现与「数据本来就没变」完全同形。
3. **为 ACL 拒绝自造前端短码 `listen_failed`。** 它不是引擎返回的短码，所以不在 `map_err` 的码表里；但它必须有自己的文案，否则会掉进通用兜底并被描述成「可重试的操作失败」。
4. **dev 开关用 `import.meta.env.DEV`。** 见缺陷 1。可用 grep `dist/` 产物证伪，这一点比任何运行期条件都硬。
5. **capability 断言用静态 import。** 换来「文件被删时 tsc 与 vitest 同时红」，而 `fs.existsSync` 只会红一处，还得拖一个 `@types/node`。
6. **样例 project id 走命令返回值。** 前端不另抄常量。冒烟页的输入框默认值虽然写着 `smoke-project`，但播种后采用的是引擎返回值——有一条单测专门盯这个（`adopts the project id the engine returned`）。
7. **不勾选任何 requirement。** 见下一节。

## Requirements

01-01 至 01-08 全部刻意留了 `requirements-completed: []`，理由一律是「这条需求横跨多个 plan，人工半边在 01-09」。本 plan 是 phase 最后一个 plan，所以逐条重新核过——**核的是需求原文是否兑现，不是 phase 是否收尾**。

**勾选 INFRA-01。** 需求原文：「Rust engine workspace（不依赖 tauri、可独立测试）+ Tauri 薄 shell + 事件总线骨架（notify-then-fetch 粗粒度事件 + Channel 有序流**各验证一条通路**）；prism-mcp 经 service trait 反转解依赖环」。四个组成部分逐一对照：

| 组成 | 状态 | 证据 |
|------|------|------|
| engine workspace 不依赖 tauri、可独立测试 | 成立 | `check-deps.sh` 的 tauri-free 断言（六条之一）；engine crates 的测试不需要 tauri runtime |
| Tauri 薄 shell | 成立 | 01-01 / 01-08：十个命令全部单行委托，`commands_carry_no_business_logic` 常驻断言 |
| notify-then-fetch 粗粒度事件**一条通路** | 成立 | Rust 侧 `bus_adapter` 4 passed + 反证 CP-1（01-08）；前端 5 passed + 去掉 cleanup 的反证；**真实 WebView 中用户实测 1:1**（人工项 1） |
| Channel 有序流**一条通路** | 成立 | Rust 侧 `smoke_stream_seq_is_strictly_monotonic`（total=1000，序列比较）；前端 `verifySmokeStream` 四条；**真实 WebView 中用户实测通过**（人工项 2） |
| prism-mcp service trait 反转解依赖环 | 成立 | 01-06 / 01-07；`check-deps.sh` 的 no-cycle 与 `prism-mcp -> prism-types only` 两条断言 |

这条需求缺的一直就是最后两行的「真实 WebView」那半边——它在本 plan 兑现了，所以勾。

**不勾 INFRA-03。** 需求原文：「API key 存系统钥匙串（keyring-core + apple-native-keyring-store）；**支持 Anthropic/OpenAI 兼容端点与自定义 base_url**；prism-llm 为唯一网络出口与唯一密钥入口」。第一项与第三项成立（人工项 4/5 通过；`check-deps.sh` 的 facade-egress + shell-egress 两条断言），但**中间那项在 Phase 1 尚未存在**：

```
crates/prism-llm/src/  →  lib.rs  secrets.rs        （只有这两个文件）
crates/prism-llm/src/lib.rs 的全部公开函数：user_agent() / keychain_backend_name()
```

没有任何 chat client，没有对 Anthropic Messages API 或 OpenAI 兼容端点的调用。目前成立的只是「base_url 这个**设置项**可写可校验」，那与「支持这些端点」不是一回事。这条要到 Phase 4 才可能完成，Phase 1 勾它就是把设置项当成了能力。

**INFRA-02** 已在 01-05 勾为 Complete，本 plan 只是给它补了真实应用里的可见性（样例数据 + 搜索入口 + 人工项 3 通过），状态不变。

## Deviations from Plan

### 检查点驱动的修复（不是 Rule 1–3 的自动修复，是人工验证的产出）

两条见上面「人工验证抓到的两个缺陷」。它们在流程上属于 Task 3 检查点的正常结果——检查点发现问题、返回、修复、再验证。记在这里是为了让它们不被 Deviations 一节的沉默掩盖。

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] `listen` 的 rejection 在两处站点都被吞掉**

- **Found during:** 缺陷 2 的调查过程
- **Issue:** 即使 capability 补齐，「listen 失败」这一整类故障仍会静默——没有 `.catch`，cleanup 的单参 `.then` 自己也 reject。
- **Fix:** 两处站点显式接住并呈现中文文案；cleanup 一律两参 `then(ok, noop)`。
- **为什么算 Rule 2 而不是缺陷 2 的一部分:** capability 是**这一次**的具体原因；吞掉 rejection 是让**任何一次** listen 失败都不可见的机制。前者修好后后者依然是个洞。
- **Commit:** `2fc002a`

**2. [Rule 3 - Blocking] 冒烟页不可达，Task 3 无法执行**

- **Found during:** Task 3 第 1 步
- **Fix:** 见缺陷 1。
- **Commit:** `4108d44`

### 01-08 交接的第 3 点担忧：本次未兑现

01-08 的 Next Phase Readiness 第 3 条写着「将来若真给项目加 `capabilities/` 目录，`has_app_acl_manifest` 变 true，即使本地来源也会走 ACL —— `src-tauri/tests/ipc.rs` 届时需加一份测试用 capability，否则集体变红」。

本 plan 正是那个「将来」。实测：**加了 capability 之后 `cargo test -p prismdocs-shell --features test --test ipc` 仍是 2 passed**，未变红。原因是被测的十个命令都是 `generate_handler!` 注册的自有命令，不受 ACL 管辖——而 ACL 生效后受影响的是**插件**命令，ipc 测试里一个都没有。这条担忧的形状是对的（ACL 确实开始生效了），但它落在了错误的对象上。STATE.md 中对应的 blocker 条目据此更新。

---

**Total deviations:** 2 auto-fixed（1 个 Rule 2 - 缺失的关键性，1 个 Rule 3 - 阻塞），均由人工检查点触发
**Impact on plan:** 计划的 `must_haves.truths` 八条全部成立——自动化可证的部分见 Verification Evidence，真实 WebView 与真实钥匙串的部分见「人工验证结果」（六项全通过，读数未捕获一节如实记录）。`artifacts` 四个文件全部存在且超过 `min_lines`；`key_links` 四条各自可 grep；`prohibitions` 为空。

## Known Stubs

**None（按「阻碍本 plan 目标达成」的口径）。**

`DevSmokePage` 与 `prism_store::seed` 是 D-06 明写的脚手架（「冒烟页是脚手架，后续 phase 逐步替换」），不是占位——两者都有真实实现与真实断言。样例文档的内容是假的，但中文 FTS 的验证本来就不需要真文档，且样例内容**刻意不含**阴性对照词「量子纠缠」，让「搜什么都命中」的实现能被看出来。

`Settings.tsx` 里没有 LLM 模型选择、没有端点连通性测试——那是 Phase 4 的内容，Phase 1 的 settings 页按 D-06 就是「可跳过」的最小面。

已向 `.planning/WINDOWS.md` 登记两条 `deviation`（两个由人工验证抓到的缺陷各一条）。

## Threat Flags

None——本 plan 未引入计划 `<threat_model>` 之外的安全面。四条处置：

| Threat | 处置 | 证据 |
|--------|------|------|
| T-01-04c（settings 页回显 API key 原文） | mitigate | 只展示布尔状态；输入 `type="password"` 且保存后清空；`never echoes the typed key back into the DOM` 断言渲染输出不含被输入的密钥串；无任何命令返回密钥（01-08）；`check-secrets.sh` exit 0 |
| T-01-36（只在前端校验 base_url） | mitigate | 前端 `isHttpLike` 是**提示**；engine 侧 `validate_base_url`（01-05）是权威边界；`renders the engine's rejection as Chinese copy` 断言走的是引擎返回的短码路径 |
| T-01-37（listen 未清理导致失效风暴） | mitigate | effect 返回清理函数；StrictMode 双执行断言（listener 恰好 1 个）+ 去掉 cleanup 即红的反证；**真实 WebView 中用户实测计数 1:1、离开再回来不翻倍** |
| T-01-38（dev 冒烟页在正式构建中可经地址栏访问） | accept → **收紧** | 计划的处置是 accept（冒烟页只有只读查询与无害触发）。本 plan 因缺陷 1 加了 dev 开关，同时把它门控在 `import.meta.env.DEV`——正式产物里入口**不存在**（grep dist 为 0），而 hash 路由本身仍在。残余风险与原判断一致，且比原状态更小 |

**额外收紧（不在计划的威胁模型内）：** 新增 `capabilities/default.json` 把主窗口的 Tauri 插件权限从「编译成 `{}`（因缺失而全拒）」变为「显式的两条」。这不是放宽——ACL manifest 缺失时是全拒，而一旦有人为了排错随手加一个宽授权（如 `core:default` 或 `fs:default`），越权同样没有症状。`capabilities.test.ts` 的 forbidden 前缀断言就是盯着这个。

## Issues Encountered

**两个真缺陷（见专节），均由人工检查点抓到，均已修复并配了各自的反证。**

其余顺利：三个 RED 均按预期以「符号/模块不存在」失败；三个 GREEN 各一次通过。

值得记一笔的是**发现顺序**：缺陷 1（不可达）挡住了缺陷 2（ACL）的暴露。如果冒烟页从一开始就可达，缺陷 2 会在第一轮人工验证就被看到；反过来，如果没有人工检查点，两个缺陷都会带着「32 项前端单测全绿」进入 Phase 2，而缺陷 2 的表现届时会是「Phase 2 的 watcher 事件好像没生效」——那时排查成本要高一个量级，且第一嫌疑人会是 watcher 而不是 ACL。

## Verification Evidence

```
npm run test -- --run                → Test Files 6 passed (6) / Tests 32 passed (32)
npx tsc --noEmit                     → exit 0
npm run build                        → exit 0，dist/index.html 0.32 kB + dist/assets/index-*.js 239.10 kB
cargo test --workspace               → 121 passed（118 → 121，seed 三条）
cargo test -p prismdocs-shell --features test --test ipc
                                     → test result: ok. 2 passed   ← 引入 ACL manifest 后未变红
cargo clippy --workspace --all-targets -- -D warnings  → exit 0
bash scripts/check-deps.sh           → 六条全 OK
bash scripts/check-secrets.sh        → exit 0

# 六个测试文件的分布
src/lib/capabilities.test.ts          1
src/lib/ipc.test.ts                   3   （01-01 建立）
src/lib/useEngineInvalidation.test.ts 5
src/App.test.tsx                      6   （2 条路由 + 4 条可达性）
src/pages/Settings.test.tsx           7
src/pages/DevSmoke.test.tsx          10   （4 条纯函数 + 6 条组件）

# 缺陷 1 的证伪证据（正式构建里入口不存在，而不是藏起来）
npm run build && grep -c 'dev 冒烟页\|dev-route-entry\|dev-route-back' dist/assets/*.js  → 0

# capability 的实际内容（最小权限）
src-tauri/capabilities/default.json:
  windows: ["main"]
  permissions: ["core:event:allow-listen", "core:event:allow-unlisten"]
  （无 fs / shell / http / dialog / core:webview / core:window）

# must_haves.artifacts 的行数（全部 ≥ min_lines）
src/lib/useEngineInvalidation.ts   52  ≥ 20
src/pages/Settings.tsx            195  ≥ 50
src/pages/DevSmoke.tsx            268  ≥ 70
src/lib/queryClient.ts             15  （未设下限）

# must_haves.key_links 四条各自可 grep
useEngineInvalidation.ts → 'prism://changed'   经 ipc.ts 的 EVENT_CHANGED（bus_adapter.rs 的同名常量）
useEngineInvalidation.ts → 'invalidateQueries' ×2（无参全量 + 按 key）
DevSmoke.tsx / ipc.ts    → 'new Channel'       ×1（在 devSmokeStream 内）
Settings.tsx / ipc.ts    → "set_api_key"       ×1

# 三条反证的落点
去掉 useEffect 的 cleanup      → 两条 handlers.size 断言（1 → 2），其余三条保持绿 ✔ 落点隔离
删掉 capabilities/default.json → tsc --noEmit 与 vitest **同时**红，落在静态 import 的模块解析 ✔
listen 被 mock 成 reject（= 修复前的真实状态）
                              → findByRole('status') 找不到元素 + vitest 报 Unhandled Rejection ✔

# 提交未删除任何被跟踪文件
git diff --diff-filter=D --name-only 323b0f5..HEAD  → 空
```

## User Setup Required

None - no external service configuration required.

若需复现人工验证：`npm run tauri dev` → 点右下角「dev 冒烟页」→ 三个按钮依次可点；设置页在默认路由。应用不配置任何密钥即可正常启动与使用（D-06 / D-16a，已由人工项 6 确认）。

## Next Phase Readiness

**已就绪，可开工：**

- **Phase 2（导入与文件监视）** — 前端数据层惯例已定形：新查询用 `useQuery` + query key `['docs', projectId]`，失效**不用自己写**——watcher 侧 `engine.publish(...)` 即可，`useEngineInvalidation` 在 App 顶层已挂。
- **Phase 4（LLM 设置页）** — `Settings.tsx` 是可扩展的起点；新错误码在 `ipc.ts` 的 `ERROR_COPY` 加一行即可，前端从不渲染码串本身。
- **Phase 2+ 的任何新页面** — hash 路由与 `App.tsx` 的结构是刻意最小的；D-06 禁止的推测式布局到 Phase 2 有导入功能后才解禁。

**必须注意的四点：**

1. **任何新的 `@tauri-apps/api` import 都需要在 `capabilities/default.json` 里补一行对应权限，而它的缺席会表现为「静默无操作」，不是报错。** 这是本 plan 最贵的一课：Tauri v2 的 ACL 只管插件命令，自有命令（`generate_handler!` 注册的）不过 ACL——于是「invoke 全都正常、某个 API 完全不工作且不报错」是一个真实可达的状态。**并且**：新调用点必须像现在这两处一样**自己把 rejection 呈现出来**，否则 capability 补齐与否都无从判断。`capabilities.test.ts` 的 forbidden 前缀断言会挡住「顺手加个 `fs:default`」这类过宽修复，但它挡不住忘记加。
2. **`import.meta.env.DEV` 门控的 UI 必须配一条生产构建断言。** `App.test.tsx` 的两条（`DEV=false` 下两个控件都不存在）+ grep `dist/` 是配套的：前者证明逻辑，后者证明摇树真的发生了。
3. **INFRA-03 仍未完成，缺的是「支持 Anthropic/OpenAI 兼容端点」那一项**（见 Requirements）。Phase 4 补上 chat client 后才能勾——base_url 可写可校验不等于端点被支持。另：本 plan 六项人工验证虽全部通过，但记录在案的是通过判定而非读数；若 Phase 验证阶段需要读数级证据，需重跑一次并当场记录。
4. **01-08 交接的 ipc-测试-会因 capabilities-变红 那条担忧已实测不成立**（本 plan 加了 capability，ipc 仍 2 passed），原因是被测命令都不过 ACL。若 Phase 6 给 ipc 测试加**插件**命令的用例，那时才需要测试用 capability。

---
*Phase: 01-foundation-skeleton*
*Completed: 2026-07-29*

## Self-Check: PASSED

- 11 个新建文件全部在盘上（`queryClient.ts` / `useEngineInvalidation.ts` / `capabilities.test.ts` / `Settings.tsx` / `Settings.test.tsx` / `DevSmoke.tsx` / `DevSmoke.test.tsx` / `App.test.tsx` / `useEngineInvalidation.test.ts` / `capabilities/default.json` / `seed.rs`）
- 9 个 commit 全部可在 `git log` 中找到：`4aab092` / `f6ce795` / `832466e` / `3220738` / `6a335dd` / `1b02b2c` / `2bb765f` / `4108d44` / `2fc002a`
- `git diff --diff-filter=D --name-only 323b0f5..HEAD` 为空——未删除任何被跟踪文件
- `must_haves.artifacts` 四个文件的行数全部 ≥ `min_lines`（52/195/268，`queryClient.ts` 未设下限）
- `must_haves.key_links` 四条各自可 grep
- 阶段闸门实跑全绿：vitest 32 passed (6 files) / `tsc --noEmit` 0 / `npm run build` 0 / `cargo test --workspace` 121 passed / `cargo test -p prismdocs-shell --features test --test ipc` 2 passed / clippy 0 / `check-deps.sh` 六条 OK / `check-secrets.sh` 0
- 三条反证全部实跑并确认落点（cleanup 去除 → `handlers.size`；capability 删除 → tsc 与 vitest 同时红；listen reject → `findByRole('status')` 找不到）
- 人工验证六项由用户执行并全部通过；读数未捕获一事已在「关于证据的一处如实说明」中记录，未以预期值代替观测值
- `requirements-completed: [INFRA-01]` 逐条核对过组成部分；INFRA-03 明确不勾并给出理由
</content>
</invoke>
