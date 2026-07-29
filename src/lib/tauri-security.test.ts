import { describe, expect, it } from "vitest";

/// WebView 的内容安全策略与资源协议开关——两条都只存在于 `tauri.conf.json` 里，
/// 两条都在被关掉时**不会让任何东西报错**。
///
/// 这条断言存在的唯一理由：`"csp": null` 与 `"assetProtocol": { "enable": true }` 的
/// 应用跑起来跟设了策略的一模一样——窗口照开、页面照渲、命令照调。差别只在被注入的
/// 那一天才显现，而那时它已经是主要利用路径而不是加固缺口（Phase 3+ 渲染外部 agent 写的
/// Markdown，Phase 6 把 LLM 放进链路）。一个把 `csp` 改回 `null` 的 diff 是一次代码评审
/// 最容易放过的形状：它只有一行，而且删掉的是一串看不出用途的字符。
///
/// 同一句话逐字适用于一个**只加一个词**的 diff——`script-src 'self' 'unsafe-inline'`
/// 比 `"csp": null` 更难在评审里被看见，而它打开的是同一条路径。这就是断言 ③ 从
/// denylist 改成逐指令精确相等的理由（见 ③ 上方的实测证据）。
///
/// 用静态 import 而不是 `fs.readFileSync` 读：与 `capabilities.test.ts` 同一手法——
/// 既省掉 `@types/node`，也让「文件被删」同时炸在 `tsc --noEmit` 与 vitest 两处。
///
/// 注意 jsdom 看不见 CSP：这条测试守的是**策略的形态**，不是它在真实 WebView 里的效果。
/// 后者只有 `npm run tauri dev` / `npm run tauri build` 能验证，已记在 01-13-PLAN.md 的
/// `<human-check>` 里。本轮补的两条指令（`form-action` / `frame-ancestors`）同样只在真实
/// WebView 里有效果——Phase 收尾的人工验证要覆盖它们：确认设置页与冒烟页都不触发这两条的违规报告。
import config from "../../src-tauri/tauri.conf.json";

/// 从策略串里取出某条指令的来源列表。`script-src 'self' 'unsafe-inline'` → `["'self'", "'unsafe-inline'"]`。
///
/// 取**首个**匹配而不是全部：CSP 里同名指令重复时浏览器只认第一条、其余整条忽略，
/// 所以「首个」就是真实生效的那一条。指令名用 `=== name` 或 `startsWith(name + " ")` 匹配，
/// 因此查 `script-src` 不会误命中 `script-src-elem`（那个由 ④″ 的指令名集合断言负责）。
function directiveSources(policy: string, name: string): string[] {
  const directive = policy
    .split(";")
    .map((part) => part.trim())
    .find((part) => part === name || part.startsWith(`${name} `));
  if (directive === undefined) {
    return [];
  }
  return directive.split(/\s+/).slice(1);
}

/// 策略串里出现的全部指令名，按书写顺序。
function directiveNames(policy: string): string[] {
  return policy
    .split(";")
    .map((part) => part.trim())
    .filter((part) => part.length > 0)
    .map((part) => part.split(/\s+/)[0]);
}

describe("tauri webview security", () => {
  it("pins the content security policy and keeps the asset protocol closed", () => {
    const { csp, devCsp, assetProtocol } = config.app.security;

    // ① 「不许回到 null」的钉子。类型上 csp 可以是 null，运行期必须不是。
    expect(typeof csp).toBe("string");
    expect(csp.length).toBeGreaterThan(0);

    // ② 发布形态的两条底线：默认来源与脚本来源都收在自身。
    //    ③ 是它的加强而不是替代——这两条 contains 在失败时给出的是更好读的消息。
    expect(csp).toContain("default-src 'self'");
    expect(csp).toContain("script-src 'self'");

    // ③ 六条**承重指令**逐条精确相等，期望值逐字取自 `src-tauri/tauri.conf.json`。
    //
    //    这里原来是一个四项 denylist（来源不得为 `*` / `http:` / `https:` / `data:`，
    //    也不得以 `*.` 开头），自称把 ② 从「字面量在」升级成「面没被扩宽」。它没有：
    //    denylist 的强度恰好等于写它的人当时想到的那几种形态，而最小权限断言要防的
    //    失败模式正是「没想到的那一种」。这不是推测，以下削弱形态在 denylist 下**实测全绿**：
    //      · `script-src 'self' 'unsafe-inline'` —— 最可能发生的那次回归。它是 CSP 挡住
    //        某个东西时被顺手加上的一项，而它完整地把这条测试想关掉的 XSS 路径装了回去。
    //      · `script-src 'self' https://cdn.evil.example` —— 一个具体远端主机既不是 `*`
    //        也不是裸 `https:`，五条黑名单项一条都不沾。
    //      · `connect-src *` —— 一条不受约束的外泄通道，而 denylist 只看 script-src，
    //        connect-src 根本没被检查。
    //      · `object-src 'self'` 与 `base-uri *` —— 同理，两条指令都不在被检查的范围里。
    //
    //    精确相等把上面每一种都变红：改动这六条里任何一条的来源，都必须同时编辑下面这几行。
    //    「必须编辑这一行」正是 denylist 试图近似、却近似不到的那个评审检查点。
    expect(directiveSources(csp, "script-src")).toEqual(["'self'"]);
    expect(directiveSources(csp, "connect-src")).toEqual([
      "'self'",
      "ipc:",
      "http://ipc.localhost",
    ]);
    expect(directiveSources(csp, "object-src")).toEqual(["'none'"]);
    expect(directiveSources(csp, "base-uri")).toEqual(["'self'"]);
    // `form-action` 与 `frame-ancestors` 是少数**没有 `default-src` 兜底**的指令：
    // 不显式写就等于没设。前者关掉「注入一个 <form action="https://evil" method="POST">
    // 加自动提交」这条外泄通道——它绕开 connect-src / img-src / script-src 的全部收口；
    // 后者是桌面 WebView 不该被任何东西嵌套的边界（clickjacking）。
    expect(directiveSources(csp, "form-action")).toEqual(["'none'"]);
    expect(directiveSources(csp, "frame-ancestors")).toEqual(["'none'"]);

    // ④ 不得放行字符串求值。`'unsafe-eval'` / `'wasm-unsafe-eval'` 把「一段字符串
    //    进了 DOM」重新变回「一段字符串被执行了」，等于把 ② 的收口撤销。
    expect(csp).not.toContain("unsafe-eval");

    // ④′ `'unsafe-inline'` 在发布形态里只允许出现在 style-src 这一处（React 的内联
    //     style 属性需要它），别处一律不许——放宽只允许发生在 devCsp 里。
    //
    //     刻意不写成 `expect(csp).not.toContain("unsafe-inline")`：发布 csp 的 style-src
    //     本来就带着它，那样写是一条永远失败的断言。改为断言「带 'unsafe-inline' 的指令
    //     集合精确等于 style-src 这一条」，强度反而更高——它同时钉住了 style-src 自己的取值。
    const inlineBearingDirectives = csp
      .split(";")
      .map((part) => part.trim())
      .filter((part) => part.includes("'unsafe-inline'"));
    expect(inlineBearingDirectives).toEqual(["style-src 'self' 'unsafe-inline'"]);

    // ④″ 指令名集合本身也精确相等。③ 守的是六条已知指令各自的来源列表，守不住
    //     「换一个指令名把面重新打开」——`script-src-elem https://cdn.evil.example`
    //     在 CSP 里优先于 script-src 生效，却不会让 ③ 的任何一条变红。
    //     新增一条合法指令时必须编辑这个数组，那正是它想要的评审检查点。
    expect(directiveNames(csp)).toEqual([
      "default-src",
      "script-src",
      "style-src",
      "img-src",
      "connect-src",
      "object-src",
      "base-uri",
      "form-action",
      "frame-ancestors",
    ]);

    // ⑤ 资源协议在配置侧关闭，scope 空。cargo feature 那一半在
    //    `src-tauri/Cargo.toml` 里同步移除（两半是配套的，只关一半等于没关）。
    expect(assetProtocol.enable).toBe(false);
    expect(assetProtocol.scope).toEqual([]);

    // ⑥ 开发形态另有一份，可以更宽（Vite 的内联引导脚本与 HMR websocket），
    //    但不能是敞开的——放宽只允许发生在 devCsp 里，csp 一个字不改。
    expect(typeof devCsp).toBe("string");
    expect(devCsp.length).toBeGreaterThan(0);
    expect(devCsp).toContain("default-src 'self'");
    expect(devCsp).not.toContain("unsafe-eval");
    expect(devCsp).not.toBe(csp);

    // ⑥′ 两条无兜底的指令在 devCsp 里也必须关着。dev 形态是开发者每天面对的那一个，
    //     一条只在 build 后才存在的边界，等于在最需要观察它的窗口期里不存在。
    expect(directiveSources(devCsp, "form-action")).toEqual(["'none'"]);
    expect(directiveSources(devCsp, "frame-ancestors")).toEqual(["'none'"]);
  });
});
