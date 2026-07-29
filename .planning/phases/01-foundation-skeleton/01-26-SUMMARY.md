---
phase: 01-foundation-skeleton
plan: 26
subsystem: tooling
tags: [eslint, linting, frontend, react-hooks, typescript-eslint, package-legitimacy, supply-chain]

requires:
  - phase: 01-foundation-skeleton
    provides: "01-22 / 01-23 / 01-24 落地的前端实现与测试（DevSmoke / Settings / ipc / capabilities），本 plan 的 lint 闸门首跑正打在它们上"
  - phase: 01-foundation-skeleton
    provides: "01-13 立下的「包合法性闸门不可自动放行」纪律，本 plan 的 Task 1 沿用"
provides:
  - "`npm run lint` —— 前端第一条 lint 闸门，当前退出 0"
  - "`eslint.config.js` —— flat config，type-aware 块限定 `src/**/*.{ts,tsx}`"
  - "四个 ESLint 相关 npm 包的合法性审计结论（写回 01-RESEARCH.md）"
  - "三条实跑反证证明的判别力：rules-of-hooks / exhaustive-deps / no-floating-promises"
affects: [01-27, ci, frontend]

tech-stack:
  added:
    - "eslint 10.8.0"
    - "typescript-eslint 8.65.0"
    - "eslint-plugin-react-hooks 7.1.1"
    - "globals 17.8.0"
  patterns:
    - "type-aware lint 作用域限定在 src/**，仓库根配置文件只走非类型感知基础块"
    - "规则豁免必须带实测依据的就地注释；产品代码的命中一律改代码而非放宽规则"
    - "async handler 交给 onClick 时写 `() => void handleX()`，与 `void queryClient.invalidateQueries()` 同一约定"

key-files:
  created:
    - eslint.config.js
  modified:
    - package.json
    - package-lock.json
    - .planning/phases/01-foundation-skeleton/01-RESEARCH.md
    - src/lib/ipc.ts
    - src/lib/useEngineInvalidation.ts
    - src/pages/DevSmoke.tsx

key-decisions:
  - "不 extends eslint-plugin-react-hooks 的 recommended-latest：它把 exhaustive-deps 配成 warn，而 eslint 对 warn 退出 0，会让反证 ② 判别力归零；它另带 15 条 React Compiler 规则，超出本 plan 范围"
  - "不引入 @eslint/js：eslint 10 已不再把它装进 node_modules，引入等于往依赖树加一个未经审计的直接依赖"
  - "测试文件定向豁免三条规则（no-unsafe-return / no-unnecessary-type-assertion / require-await），每条均有首跑实测依据，非为了让 lint 变绿"
  - "no-console 不为 console.error 开例外：本代码库把每个错误路由过 errorCopy 并渲染进 live region，桌面 WebView 控制台用户不会打开"
  - "选 eslint-plugin-react-hooks 7.x 而非更轻的 6.x：人工明确拒绝 6.x，接受 7.x 的 babel/hermes/zod 传递依赖为已知成本"

patterns-established:
  - "反证样本一律不用未使用变量：该规则已按 tsconfig 覆盖并在 ESLint 侧关闭，拿它做反证必然退出 0"
  - "规则与 tsc 冲突时以 tsc 为权威，并把实测报错原文抄进 eslint.config.js 注释"

requirements-completed: [INFRA-01]

coverage:
  - id: D1
    description: "npm run lint 存在且在当前代码库上退出 0"
    requirement: "INFRA-01"
    verification:
      - kind: automated
        ref: "npm run lint"
        status: pass
    human_judgment: false
  - id: D2
    description: "lint 闸门能抓条件分支里的 hook 调用（react-hooks/rules-of-hooks）"
    requirement: "INFRA-01"
    verification:
      - kind: automated
        ref: "反证①：DevSmoke.tsx 的 useState 挪进三元条件 → exit 1, 1 problem, 点名 react-hooks/rules-of-hooks"
        status: pass
    human_judgment: false
  - id: D3
    description: "lint 闸门能抓缺失的 hook 依赖（react-hooks/exhaustive-deps）"
    requirement: "INFRA-01"
    verification:
      - kind: automated
        ref: "反证②：useEngineInvalidation.ts 去掉 [queryClient] → exit 1, 1 problem, 点名 react-hooks/exhaustive-deps"
        status: pass
    human_judgment: false
  - id: D4
    description: "lint 闸门能抓悬空 promise（@typescript-eslint/no-floating-promises，WR-16 点名的那条）"
    requirement: "INFRA-01"
    verification:
      - kind: automated
        ref: "反证③：DevSmoke.tsx handleSearch 插入未 await 的 searchDocuments() → exit 1, 1 problem, 点名 @typescript-eslint/no-floating-promises"
        status: pass
    human_judgment: false
  - id: D5
    description: "四个新增 npm 包在安装前经人工合法性核对，结论写回 01-RESEARCH.md § Package Legitimacy Audit"
    requirement: "INFRA-01"
    verification:
      - kind: manual_procedural
        ref: "人工 2026-07-29 在 npm registry 逐包核对后回复 approved"
        status: pass
    human_judgment: true
    rationale: "包合法性是供应链信任判断，形近名与恶意 postinstall 无法由自动化替人做决定；沿用 01-13 立下的不可自动放行纪律"

duration: 18min
completed: 2026-07-30
status: complete
---

# Phase 01 Plan 26: 前端 ESLint 闸门 Summary

**前端从「零 lint 闸门」到有一条能抓条件 hook 调用、缺失 hook 依赖与悬空 promise 的闸门；装上后首跑立刻在产品代码里揪出 6 处真实问题——其中两处正是 `useEngineInvalidation.ts` 自己注释里写着「不能发生」的悬空 promise。**

## Performance

- **Duration:** 约 18 min（本续跑 agent 部分；不含前一 agent 的闸门审计）
- **Tasks:** 2/2
- **Files created/modified:** 7

## Accomplishments

- **`npm run lint` 落地并退出 0。** flat config（ESM），type-aware 块限定 `src/**/*.{ts,tsx}`，从未收窄过。
- **闸门装上第一跑就抓到 6 处真实产品代码问题，全部改代码解决，无一靠放宽规则消掉。** 最有说服力的两处在 `src/lib/useEngineInvalidation.ts`：该文件第 15-19 行的注释写着「这一类失败不能被丢进未处理的 Promise」，而它自己第 31 / 34 行的 `queryClient.invalidateQueries()` 正是两个悬空 promise——`Settings.tsx` / `DevSmoke.tsx` 的四个同类调用点本来都写了 `void`，只有这个文件漏了。上轮 WR-16 点名这条规则时的判断是准的。
- **三条反证全部实跑通过**，每条都是 exit 1 + 恰好 1 个 problem + 点名目标规则。
- **两条测试文件豁免都有实测依据，不是「为了变绿」。** 其中 `no-unnecessary-type-assertion` 被实测证明其 autofix **不健全**（与 tsc 直接矛盾）。
- **自查中发现并修掉一个空洞的绿**：`eslint .` 原本在 lint `target/` 下 18 个 Tauri 生成物。

## Task Commits

1. **Task 1: 包合法性闸门 + 安装** - `f04f4ae` (chore)
2. **Task 2: ESLint flat config 与 npm run lint** - `8587e4d` (feat)
3. **自查发现的忽略集缺口** - `cf28b31` (fix)

## Task 1：包合法性核对的实际读数

人工于 **2026-07-29** 在 npm registry 上逐包核对后回复 `approved`。执行者本轮复核了 registry 事实（`npm view <pkg> repository.url license hasInstallScript`），与人工核对结论一致：

| 包名 | 源仓库（registry `repository.url` 实读） | 最新稳定版 / 发版 | 周下载量 | 许可 | install/postinstall | deprecated | 判定 |
|---|---|---|---|---|---|---|---|
| `eslint` | github.com/eslint/eslint | 10.8.0 / 2026-07-24 | 152.4M/wk | MIT | 无 | 否 | OK |
| `typescript-eslint` | github.com/typescript-eslint/typescript-eslint | 8.65.0 / 2026-07-20 | 84.5M/wk | MIT | 无 | 否 | OK |
| `eslint-plugin-react-hooks` | github.com/facebook/react | 7.1.1 / 2026-04-17 | 93.6M/wk | MIT | 无 | 否 | OK |
| `globals` | github.com/sindresorhus/globals | 17.8.0 / 2026-07-26 | 262.0M/wk | MIT | 无（`prepare` 只在 git 源/本地开发时跑） | 否 | OK |

补充事实：

- 四个包名全部通过 registry `repository.url` **逐字**解析到上表官方仓库，未到达任何形近包。
- `typescript-eslint`（flat config 统一入口）与旧的 `@typescript-eslint/parser` + `@typescript-eslint/eslint-plugin` 确实是不同的东西，不是形近名；它依赖那两个 scoped 包且 pin 在同一 `8.65.0`。
- `typescript-eslint` 带 **SLSA v1 provenance attestation**（GitHub Actions trusted publisher / OIDC），四者中供应链证据最强。`eslint` 由 OpenJS Foundation（`eslintbot`）发布，无 attestation。
- peer 兼容：`typescript-eslint@8.65.0` 要求 `typescript >=4.8.4 <6.1.0`，本仓库 `typescript@^5.9.0` 在区间内；`eslint-plugin-react-hooks@7.1.1` 接受 `eslint ^10.0.0`。三者同处 eslint 10 线。
- **人工明确接受的代价**：`eslint-plugin-react-hooks@7.1.1` 拉入 `@babel/core ^7.24.4` / `@babel/parser` / `hermes-parser ^0.25.1` / `zod` / `zod-validation-error`。6.x 曾作为更轻方案提出并被**明确拒绝**，7.x 的更重依赖树是已接受的成本。

最终 pin 与实装（`npm ls`，eslint 在树中 deduped 为单一副本）：

```
prismdocs@0.1.0 /Users/xinz/Development/PrismDocs
├── eslint-plugin-react-hooks@7.1.1
├── eslint@10.8.0
├── globals@17.8.0
└── typescript-eslint@8.65.0
```

## Task 2：首跑的 6 处产品代码命中（全部改代码）

首跑共 36 个 error。**产品代码 6 处，全部改代码解决**；其余 29 处全在测试文件，见下一节。

| # | 位置 | 规则 | 处理 |
|---|---|---|---|
| 1-2 | `src/lib/useEngineInvalidation.ts:31,34` | `no-floating-promises` | 两处 `queryClient.invalidateQueries()` 补 `void`，与 `Settings.tsx` / `DevSmoke.tsx` 已有的四个调用点同形 |
| 3-6 | `src/pages/DevSmoke.tsx:175,191,213,233` | `no-misused-promises` | `onClick={handleX}` → `onClick={() => void handleX()}`；四个 handler 各有完整内部 try/catch，`void` 是准确的意思表达 |
| 7 | `src/lib/ipc.ts:49` | `no-unsafe-assignment` | 断言收窄到 `Object.create(null) as Record<string, string>`。**运行期语义分毫未变**——容器仍然没有原型链，01-23 的自有属性判定完好 |

第 1-2 处值得单独记一笔：这是闸门自证价值最强的一条证据。该文件的注释明确写着这类失败不能被丢进未处理的 Promise，而漏 `void` 的恰好就是它自己，且四个同类调用点里只有它漏。这正是「注释是约定、lint 才是机制」的实例。

## Task 2：测试文件的三条定向豁免（均有实测依据）

29 处测试文件命中集中在三个规则族。豁免只作用于 `src/**/*.test.{ts,tsx}`，产品代码里三条照常生效。

**① `@typescript-eslint/no-unnecessary-type-assertion`（11 处）—— 实测其 autofix 不健全。**
Testing Library 的 `findByLabelText` 签名是 `<T extends HTMLElement = HTMLElement>`，TS 会从断言目标**反推** `T`，于是断言在规则看来「没有改变类型」。实测跑 `npx eslint . --fix` 后 tsc 立刻报：

```
src/pages/Settings.test.tsx(77,18): error TS2339: Property 'type' does not exist on type 'HTMLElement'.
src/pages/Settings.test.tsx(83,38): error TS2339: Property 'value' does not exist on type 'HTMLElement'.
```

规则与 `tsc --noEmit` 直接矛盾，而 tsc 是权威（build 绿是本 plan 的 must_have）。已 `git checkout` 还原。

**② `@typescript-eslint/require-await`（6 处）—— 实测 `async` 承载语义。**
`listen` 返回 `Promise<UnlistenFn>`，产品代码对它做 `pending.catch(...)` / `pending.then(...)`；mock 上的 `async` 正是把返回值塑造成 Promise 的东西。实测把 `useEngineInvalidation.test.ts:49` 的 `async` 去掉后该文件 **5 条测试全部失败**（`.catch` 打在非 Promise 上）。已还原，5/5 恢复通过。

**③ `@typescript-eslint/no-unsafe-return`（12 处）—— 源自 `vi.fn()` 的 `any`。**
`vi.fn()` 不带类型参数时返回 `Mock<(...args: any[]) => any>`，mock 工厂体 `() => spy(...)` 的返回值静态类型即 `any`。这些工厂**刻意**写成「体内只引用不解引用 spy」以绕开 `vi.mock` 提升导致的 TDZ（01-22 起已验证的写法）。补全泛型是对本轮刚落地测试的纯噪声改写。

**没有豁免的**：`no-floating-promises` / `no-misused-promises` / 两条 hook 规则 / `no-console` 在测试文件里全部保持 error。

## 三条非恒真反证（全部实跑，逐条 `git checkout` 还原）

三条均从**已提交的干净基线** `8587e4d` 出发，样本一律不用未使用变量（该规则已按 tsconfig 覆盖并在 ESLint 侧关闭）。

### 反证 ① `react-hooks/rules-of-hooks`

样本（`src/pages/DevSmoke.tsx`）—— 把一次 `useState` 挪进条件分支：

```diff
-  const [streaming, setStreaming] = useState(false);
+  const [streaming, setStreaming] =
+    projectId !== "" ? useState(false) : [false, () => {}];
```

```
/Users/xinz/Development/PrismDocs/src/pages/DevSmoke.tsx
  75:24  error  React Hook "useState" is called conditionally. React Hooks must be called in the exact same order in every component render  react-hooks/rules-of-hooks

✖ 1 problem (1 error, 0 warnings)
exit=1
```

### 反证 ② `react-hooks/exhaustive-deps`

样本（`src/lib/useEngineInvalidation.ts`）—— 去掉一条必需依赖：

```diff
-  }, [queryClient]);
+  }, []);
```

```
/Users/xinz/Development/PrismDocs/src/lib/useEngineInvalidation.ts
  54:6  error  React Hook useEffect has a missing dependency: 'queryClient'. Either include it or remove the dependency array  react-hooks/exhaustive-deps

✖ 1 problem (1 error, 0 warnings)
exit=1
```

这条同时证明了「配成 error 而非沿用 preset 的 warn」是必要的：preset 的 warn 级别下 eslint 仍退出 0，这条反证会在规则确实生效时也退出 0。

### 反证 ③ `@typescript-eslint/no-floating-promises`

样本（`src/pages/DevSmoke.tsx` 的 `handleSearch`）—— 插入一个未 `await` 也未 `void` 的 promise 调用：

```diff
   async function handleSearch() {
     setNotice(null);
+    searchDocuments(projectId, query);
```

```
/Users/xinz/Development/PrismDocs/src/pages/DevSmoke.tsx
  150:5  error  Promises must be awaited, end with a call to .catch, end with a call to .then with a rejection handler or be explicitly marked as ignored with the `void` operator  @typescript-eslint/no-floating-promises

✖ 1 problem (1 error, 0 warnings)
exit=1
```

**type-aware 块的最终 `files` glob：`["src/**/*.{ts,tsx}"]` —— 全程从未收窄。** 反证 ③ 的样本文件 `src/pages/DevSmoke.tsx` 落在该 glob 内，规则未落空。首跑无性能问题，`<action>` 允许的收窄退让未被动用。

## 本轮其他 plan 的实现语义未被 lint 驱动改动

`git diff` 涉及的其他 plan 文件逐条说明：

- `src/lib/ipc.ts`（01-23）—— 唯一改动是把 `as Record<string, string>` 断言从赋值处收窄到 `Object.create(null)` 上。容器仍是 `Object.create(null)`，**没有原型链这一性质完好**，`errorCopy` 的自有属性判定与兜底行为一字未动。测试 75/75 全绿，`ipc.test.ts` 针对 `"toString"` / `"__proto__"` / `"constructor"` 的断言全部照常通过。
- `src/pages/DevSmoke.tsx`（01-22）—— 四处 `onClick` 包一层 `() => void handleX()`。handler 本体与内部 try/catch 一字未动；点击行为等价（原本 React 也丢弃返回值）。
- `src/lib/useEngineInvalidation.ts` —— 两处补 `void`，语义等价（原本也没等它）。
- `src/lib/tauri-security.test.ts` / `src/lib/capabilities.test.ts` / `src/pages/Settings.tsx`（01-23 / 01-24）—— **零改动**。01-24 的精确相等断言未被触碰。

## Files Created/Modified

- `eslint.config.js`（新）—— flat config；文件头注释说明闸门存在的理由与它守的四类问题，形态参照 `scripts/check-secrets.sh`
- `package.json` —— 新增 `lint` script 与四个 devDependencies；`build` **未**被塞入 lint（CI 接线归 01-27）
- `package-lock.json` —— 85 个包（含传递依赖）
- `.planning/phases/01-foundation-skeleton/01-RESEARCH.md` —— 审计表追加四行 + 一段人工确认说明
- `src/lib/ipc.ts` / `src/lib/useEngineInvalidation.ts` / `src/pages/DevSmoke.tsx` —— 首跑命中的修复

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - 缺失的关键配置] `eslint .` 在 lint cargo 的 `target/` 生成物**

- **Found during:** Task 2 自查（用 `--format json` 数 eslint 实际走过的文件）
- **Issue:** 计划给出的忽略集是 `dist/` / `node_modules/` / `src-tauri/` / `coverage/`，漏了仓库根的 `target/`（cargo 输出，`.gitignore` 里是 `/target`）。eslint 实际在 lint 它下面 18 个 Tauri 生成的 `__global-api-script.js`。它们今天 0 报错，但那是**空洞的绿**——生成物只落进不定义任何规则的基础块；一旦某次生成物带了会命中规则的写法，`npm run lint` 就会因为一个没人写过的文件而失败。
- **Fix:** 忽略集加 `target/`，并就地注释写明理由。受检文件从 34 降到 16（`eslint.config.js` + 14 个 `src` 文件 + `vite.config.ts`），全部是有意覆盖的目标。
- **Verification:** `npx eslint . --format json` 逐文件核对；反证 ③ 在改动后重跑仍 exit 1 并点名目标规则。
- **Committed in:** `cf28b31`

**2. [Rule 3 - 计划假设与工具链现状不符] 不引入 `@eslint/js`**

- **Found during:** Task 2（探测配置 API 面）
- **Issue:** 常规 flat config 写法会 `import js from "@eslint/js"` 取 `js.configs.recommended`。实测 `node_modules/@eslint/js` **不存在**——eslint 10 已不再把它作为依赖装进来。补装它等于往依赖树加一个 Task 1 未审计的直接依赖，与本 plan「未经核对不得进入依赖树」的 prohibition 直接冲突。
- **Fix:** 不引入。规则骨干取 `typescript-eslint` 的 `recommendedTypeChecked`，需要的 eslint 核心规则（`no-console`）按名字直接开——核心规则内置于 eslint，无需任何 import。理由写进 `eslint.config.js` 文件头。
- **Verification:** `npm run lint` 退出 0；三条反证全部命中。
- **Committed in:** `8587e4d`

---

**Total deviations:** 2 auto-fixed（1× Rule 2 缺失配置，1× Rule 3 计划假设与工具链现状不符）
**Impact on plan:** 两条都收紧而非放宽了闸门，无 scope creep。计划的核心约束全部守住：type-aware glob 未收窄、`no-floating-promises` 未退档、产品代码命中全部改代码解决。

## Issues Encountered

**执行过程中的一次自身失误（已修正，值得记录）：** 反证 ① 还原时对 `src/pages/DevSmoke.tsx` 跑 `git checkout --`，把同一文件里**尚未提交**的四处 `void handleX()` 修复一并冲掉了，导致反证 ② 的输出里混进 4 条无关的 `no-misused-promises`。已重新施加修复，并改为**先提交 Task 2、再从干净基线跑三条反证**——记录在案的三条反证输出全部来自已提交基线 `8587e4d`，每条恰好 1 个 problem。教训：注入式反证必须从已提交状态出发，否则 `git checkout` 的还原范围会超出注入范围。

## Known Stubs

无。

## 交接给 01-27 / 01-28

- `npm run lint` 已就绪可被 CI 调用。**本 plan 刻意没有碰 `.github/workflows/ci.yml`**，frontend job 的 lint 步骤归 01-27。
- `build` 里**没有**塞 lint（`build` 的职责是构建）。若 01-27 需要单命令闸门，请在 CI 里显式并列调用。
- 若 01-27 打算加 `--max-warnings 0`：本 config 目前**不依赖**它（两条 hook 规则已是 error），加上无害且能防住以后有人把规则降成 warn。

## User Setup Required

None - no external service configuration required.

## Self-Check: PASSED

- `eslint.config.js` 存在
- `01-26-SUMMARY.md` 存在
- 三个 task 提交均在 git 历史中：`f04f4ae` / `8587e4d` / `cf28b31`
- `package.json` 第 9 行含 `"lint": "eslint ."`
- `01-RESEARCH.md` 含 4 行「人工确认于 01-26 执行期」审计条目
- 收尾复跑：`npm run lint` = 0、`npm run build` = 0、`npm run test -- --run` = 75/75
