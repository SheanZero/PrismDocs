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
/// 用静态 import 而不是 `fs.readFileSync` 读：与 `capabilities.test.ts` 同一手法——
/// 既省掉 `@types/node`，也让「文件被删」同时炸在 `tsc --noEmit` 与 vitest 两处。
///
/// 注意 jsdom 看不见 CSP：这条测试守的是**策略的形态**，不是它在真实 WebView 里的效果。
/// 后者只有 `npm run tauri dev` / `npm run tauri build` 能验证，已记在 01-13-PLAN.md 的
/// `<human-check>` 里。
import config from "../../src-tauri/tauri.conf.json";

/// 从策略串里取出某条指令的来源列表。`script-src 'self' 'unsafe-inline'` → `["'self'", "'unsafe-inline'"]`。
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

describe("tauri webview security", () => {
  it("pins the content security policy and keeps the asset protocol closed", () => {
    const { csp, devCsp, assetProtocol } = config.app.security;

    // ① 「不许回到 null」的钉子。类型上 csp 可以是 null，运行期必须不是。
    expect(typeof csp).toBe("string");
    expect(csp.length).toBeGreaterThan(0);

    // ② 发布形态的两条底线：默认来源与脚本来源都收在自身。
    expect(csp).toContain("default-src 'self'");
    expect(csp).toContain("script-src 'self'");

    // ③ 脚本来源里不得出现通配符或整协议放行——`script-src 'self' https:` 与
    //    `script-src *` 在语法上都合法，都能让 ② 那两条 contains 继续为真，
    //    而 WebView 已经敞开了。这条把 ② 从「字面量在」升级成「面没被扩宽」。
    const scriptSources = directiveSources(csp, "script-src");
    expect(scriptSources).not.toHaveLength(0);
    for (const source of scriptSources) {
      expect(source).not.toBe("*");
      expect(source).not.toBe("http:");
      expect(source).not.toBe("https:");
      expect(source).not.toBe("data:");
      expect(source.startsWith("*.")).toBe(false);
    }

    // ④ 不得放行字符串求值。`'unsafe-eval'` / `'wasm-unsafe-eval'` 把「一段字符串
    //    进了 DOM」重新变回「一段字符串被执行了」，等于把 ② 的收口撤销。
    expect(csp).not.toContain("unsafe-eval");

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
  });
});
