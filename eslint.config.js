// ESLint flat config —— 前端唯一的 lint 闸门。
//
// 为什么这份配置存在：`npm run build` 里的 `tsc --noEmit` 是本仓库前端此前唯一的自动闸门，
// 而它有一整类结构性看不见的问题。Rust 侧有 `clippy -D warnings`，前端侧在 01-26 之前
// 什么都没有——上轮评审 WR-16 点名的正是这个缺口。tsc 看不见但会在运行期咬人的四类：
//
//   1. 条件分支里的 hook 调用 —— 类型完全合法，运行期 hook 顺序错乱。
//   2. 缺依赖的 `useEffect` —— 类型完全合法，表现是「数据一直是旧的」。
//   3. 悬空 promise —— 类型完全合法。`src/lib/useEngineInvalidation.ts` 第 15-19 行的注释
//      写着「这一类失败不能被丢进未处理的 Promise」；在此之前没有任何自动化的东西守这句话。
//      对一个以「锚点 0 静默丢失」为发布门槛的项目，静默失败是最贵的失败形态。
//   4. `console.*` 直写 —— 本代码库刻意把每个错误都路由过 `errorCopy`（`src/lib/ipc.ts`），
//      绕过它直接 console 等于让一条错误既不进 DOM 也不进用户视野。
//
// 规则集取向：**最小但有牙**，且刻意只取 tsc 结构性看不见的那一类。
// 与 `tsconfig.json` 重复的规则一律关掉（见下方 `no-unused-vars` 一条）——同一件事被
// tsc 与 ESLint 各报一次只制造噪声，而噪声会让人开始整体忽略 lint 输出。
//
// 依赖面：本文件只 import Task 1 经人工合法性核对批准的四个包
// （`typescript-eslint` / `eslint-plugin-react-hooks` / `globals`，加 eslint 本体）。
// **刻意不 import `@eslint/js`**：eslint 10 不再把它作为依赖装进 node_modules，
// 引入它等于往依赖树里加一个未经审计的直接依赖，而审计闸门是 01-26 Task 1 的全部意义。
// 需要的 eslint 核心规则按名字直接开（核心规则内置于 eslint，无需任何 import）。
//
// 本文件是 lint 断言的唯一实现；`package.json` 的 `lint` script 与 CI 都只是调用者。
// CI 接线（frontend job 的 lint 步骤）由 01-27 完成，不在本文件职责内。

import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import tseslint from "typescript-eslint";

export default tseslint.config(
  {
    ignores: ["dist/", "node_modules/", "src-tauri/", "coverage/"],
  },

  // ── 基础块（非类型感知）：仓库根的配置文件 ────────────────────────────────
  //
  // `eslint.config.js` 与 `vite.config.ts` 只走这一块，**不进类型感知块**。
  // 理由是一个不显眼的坑：`tsconfig.json` 的 `include` 是 `["src", "vite.config.ts"]`,
  // 不含仓库根的 `eslint.config.js`，而 `lint` script 是 `eslint .`。若类型感知配置
  // 作用于全部文件，eslint 会在本文件上报出 **parser 错误**（「该文件不在任何 tsconfig
  // 的 project 里」）而不是 lint 错误——报错形态会让人以为是规则写错了，而不是作用域问题。
  {
    files: ["*.js", "*.ts", "vite.config.ts"],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: "module",
      globals: globals.node,
    },
  },

  // ── 类型感知块：`src/` 下的全部 TS/TSX ────────────────────────────────────
  //
  // 作用域限定在 `src/**` 上，这**同时**满足两件事：避开上面那个 parser 缺口，
  // 又让反证 ③ 的目标规则（`no-floating-promises`，结构性依赖 type-aware linting）
  // 打在真正的产品代码上。`tsconfig.json` 的 `include: ["src"]` 覆盖 `src/` 全部文件
  // （含 `*.test.tsx`），所以 projectService 能为其中每个文件找到 project。
  {
    files: ["src/**/*.{ts,tsx}"],
    extends: [tseslint.configs.recommendedTypeChecked],
    languageOptions: {
      globals: globals.browser,
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    plugins: {
      "react-hooks": reactHooks,
    },
    rules: {
      // ── hook 规则：两条都必须是 error ─────────────────────────────────
      //
      // **刻意不 extends `reactHooks.configs['recommended-latest']`。** 两个理由：
      //
      // (a) 那个 preset 把 `exhaustive-deps` 配成 **warn**，而 eslint 对 warn 级别的
      //     命中仍然 **退出 0**（除非加 `--max-warnings 0`）。于是「规则确实生效」与
      //     「规则根本没装」在退出码上完全同形——判别力归零。本 plan 三条反证的判据
      //     正是 `npm run lint` 退出 1，所以这两条必须是 error。
      // (b) 那个 preset 还带 15 条 React Compiler 相关规则（`purity` / `immutability` /
      //     `set-state-in-effect` 等）。它们不在 01-26 的范围内，且会对本轮 01-22/01-23
      //     刚落地的实现语义提出改动要求——而本 plan 明确禁止 lint 驱动改动那些语义。
      //     要引入它们应当是一个独立的、有自己反证的决策。
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "error",

      // ── 悬空 promise：本 plan 的硬约束 ─────────────────────────────────
      //
      // `recommendedTypeChecked` 已把它配成 error，这里显式重申是为了让「它是硬约束」
      // 这件事在文件里可读——它是上轮 WR-16 点名的那条，也是
      // `useEngineInvalidation.ts` 注释里那句话的自动化形态。
      "@typescript-eslint/no-floating-promises": "error",

      // ── console 直写 ──────────────────────────────────────────────────
      //
      // **不为 `console.error` 开例外。** 本代码库把每个错误都路由过 `errorCopy`
      // 并渲染进 `role="alert"` live region（`src/lib/ipc.ts` 的文件头注释说明了
      // 为什么错误短码不能原样进 DOM）。桌面应用的 WebView 控制台用户根本不会打开，
      // 往那里写等于把错误写进没人看的地方——那正是 `errorCopy` 这套设计要消灭的失败形态。
      // 当前 `src/` 下 console 用量为 0，所以这条规则落地时没有任何既有代码需要改动。
      "no-console": "error",

      // ── 关掉与 tsconfig.json 重复的规则 ────────────────────────────────
      //
      // `noUnusedLocals` / `noUnusedParameters` 已在 tsconfig.json 里开启，未使用变量
      // 由 `tsc --noEmit` 报。同一件事报两次只制造噪声。
      //
      // 连带的后果写在这里以免以后有人误用：本 plan 的三条反证**一律不用未使用变量做样本**。
      // `lint` script 是 `eslint .`、不含 tsc，拿已关掉的规则做反证必然退出 0——
      // 那种「反证」证明不了 ESLint 在跑。三条反证的目标规则全部取 ESLint 独有的那一类。
      "@typescript-eslint/no-unused-vars": "off",
    },
  },

  // ── 测试文件：三条规则的定向豁免 ──────────────────────────────────────────
  //
  // 下面三条**只在测试文件里**关掉，每一条都是首跑实测出的规则/工具链冲突，
  // 不是「为了让 lint 变绿」。产品代码里三条全部照常生效。
  // 首跑在产品代码里报的 6 处（`useEngineInvalidation.ts` 两处悬空 promise、
  // `DevSmoke.tsx` 四处 `onClick` 交出 Promise、`ipc.ts` 一处 any 赋值）
  // **全部是改代码解决的**，没有一条是靠放宽规则消掉的。
  //
  // 同样重要的是这里**没有**关掉什么：`no-floating-promises` / `no-misused-promises`
  // / 两条 hook 规则 / `no-console` 在测试文件里全部保持 error。漏掉的 await 是
  // 测试假绿的主要来源之一，正是要在测试里守的东西。
  {
    files: ["src/**/*.test.{ts,tsx}"],
    rules: {
      // 冲突：`vi.fn()` 不带类型参数时返回 `Mock<(...args: any[]) => any>`，
      // 于是 mock 工厂体 `() => spy(...)` 的返回值静态类型是 `any`。
      // 而这些工厂**刻意**写成「体内只引用不解引用 spy」的形态——那是为了绕开
      // `vi.mock` 提升到静态 import 之上导致的 TDZ（见 ipc.test.ts 第 6 行注释，
      // 01-22 起就是本仓库已验证的写法）。要消掉这条得给每个 `vi.fn()` 补全泛型，
      // 那是对本轮 01-22/01-23/01-24 刚落地的测试做一次纯噪声改写。
      "@typescript-eslint/no-unsafe-return": "off",

      // 冲突（**实测**）：这条规则与 `tsc --noEmit` 直接矛盾，而 tsc 是权威。
      // Testing Library 的 `findByLabelText` 签名是 `<T extends HTMLElement = HTMLElement>`,
      // TS 会**从断言目标反推** `T`，于是 `(await findByLabelText(...)) as HTMLInputElement`
      // 在规则看来「没有改变类型」。但把断言删掉后 `T` 退回默认的 `HTMLElement`，
      // 首跑实测 `npx eslint . --fix` 之后 tsc 立刻报：
      //   src/pages/Settings.test.tsx(77,18): error TS2339: Property 'type' does not exist on type 'HTMLElement'.
      //   src/pages/Settings.test.tsx(83,38): error TS2339: Property 'value' does not exist on type 'HTMLElement'.
      // 也就是说这条规则的 autofix 在这个签名下是**不健全**的。
      "@typescript-eslint/no-unnecessary-type-assertion": "off",

      // 冲突（**实测**）：这些 `async` 是**承载语义**的，不是可省的装饰。
      // `listen` 返回 `Promise<UnlistenFn>`，产品代码对它做 `pending.catch(...)` /
      // `pending.then(...)`；mock 上的 `async` 正是把返回值塑造成 Promise 的那个东西。
      // 实测把 `useEngineInvalidation.test.ts` 第 49 行的 `async` 去掉后，
      // 该文件 5 条测试全部失败（`.catch` 打在一个非 Promise 上）。
      "@typescript-eslint/require-await": "off",
    },
  },
);
