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

    // listen 与 unlisten 是两条独立的插件命令（@tauri-apps/api 的 event.js 分别 invoke
    // `plugin:event|listen` 与 `plugin:event|unlisten`），少任一条都留下一个 rejection。
    expect(capability.permissions).toContain("core:event:allow-listen");
    expect(capability.permissions).toContain("core:event:allow-unlisten");

    // 最小权限：Phase 1 的前端只用 event + 自有命令 invoke + Channel（后两者不过 ACL）。
    // 任何文件系统 / shell / http 授权在这里都是越权，不是「以后会用到」。
    const forbidden = capability.permissions.filter((p) =>
      /^(fs|shell|http|dialog|core:webview|core:window):/.test(p),
    );
    expect(forbidden).toEqual([]);
  });
});
