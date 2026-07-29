import { describe, expect, it } from "vitest";

/// capability 文件是**源码**（`src-tauri/gen/schemas/` 才是 gitignore 掉的生成物）。
///
/// 这条断言存在的唯一理由：它缺席时应用不会报错，只是安静地失能。Tauri v2 的 ACL 只管
/// 插件命令，`generate_handler!` 注册的自有命令不过 ACL——于是 `invoke('dev_emit_bus_event')`
/// 照常成功，而 `listen()`（走 `plugin:event|listen`）被拒。发射方一切正常、监听方从未注册，
/// 表现就是「点了没反应、也没有任何错误」。
///
/// 用静态 import 而不是 `fs.existsSync` 读：既省掉一个 `@types/node` 依赖，
/// 也让「文件被删」同时炸在 `tsc --noEmit` 和 vitest 两处，而不只是一条运行时断言。
import capability from "../../src-tauri/capabilities/default.json";

describe("tauri capabilities", () => {
  it("grants the main window event listen/unlisten and nothing beyond least privilege", () => {
    // 窗口标签：tauri.conf.json 的 windows[0] 未写 label，Tauri 默认给 "main"
    // （tauri-utils config.rs::default_window_label）。授权挂错标签等于没授权，
    // 而症状与文件整个缺失完全相同。
    expect(capability.windows).toContain("main");

    // 最小权限：**精确相等**，期望值逐字取自 `src-tauri/capabilities/default.json`。
    //
    // 这里原来是一个六前缀的 denylist（`fs` / `shell` / `http` / `dialog` /
    // `core:webview` / `core:window`），注释自称断言最小权限。它做不到：denylist 的强度
    // 恰好等于写它的人当时想到的那六个前缀，之外的一切**无声通过**。实测确认下面两条
    // 在 denylist 下全绿：
    //   · `core:event:allow-emit` —— 它让 WebView 里的脚本能伪造 `prism://changed`
    //     事件直接注进失效管线，是本项目里最不该被放过的那一条，而 `core:event:` 恰好
    //     不在六个前缀里（前缀表只列了 `core:webview` 与 `core:window`）。
    //   · `core:app:default` —— 同理，后续 phase 会想加的每一个 `core:app:*` /
    //     `core:path:*` / `plugin:*` 都在 denylist 之外。
    //
    // 精确相等把「加一条权限」变成「必须编辑这一行」，而那正是 denylist 想近似、
    // 却近似不到的那个评审检查点。当前授予集只有两条，这个写法的成本是零。
    //
    // 用 `toEqual` 直接比数组而不是先排序：顺序在 ACL 语义上无意义，但保持顺序敏感是免费的
    // ——它顺带钉住了文件里的书写顺序，而重排两条权限没有任何正当理由，出现即值得看一眼。
    //
    // 原来那两条 `toContain("core:event:allow-listen"/"allow-unlisten")` **并入**了这条
    // 精确相等，而不是与它并存：`toEqual` 在缺项时给出的 diff 已经直接指出少了哪一条，
    // 两条 toContain 只是同一件事的更弱形式，留着会让「授予集是什么」有两个真相源。
    // 保留它们原本承载的理由：listen 与 unlisten 是两条独立的插件命令（@tauri-apps/api 的
    // event.js 分别 invoke `plugin:event|listen` 与 `plugin:event|unlisten`），
    // 少任一条都留下一个 rejection。
    expect(capability.permissions).toEqual([
      "core:event:allow-listen",
      "core:event:allow-unlisten",
    ]);
  });
});
