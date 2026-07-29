---
schema_version: 1
open_count: 13
waived_count: 0
fixed_count: 1
total_count: 14
last_updated: 2026-07-29T23:39:44.371Z
---

# Broken Windows Ledger

> Cross-phase defect register. `/gsd-ship` blocks while `open_count > 0`.
> Waive with `gsd-tools windows waive <id> "<reason>"` (reason required).
> Mark fixed with `gsd-tools windows fixed <id>`.

| id | phase | kind | file | line | description | status | reason | recorded_at | resolved_at |
|----|-------|------|------|------|-------------|--------|--------|-------------|-------------|
| 1 | 01 | deviation | crates/prism-store/src/open.rs |  | 迁移先于建池的顺序无行为面哨兵：计划设想的反证实测不成立，目前只由源码顺序断言守住 | open |  | 2026-07-28T23:13:51.042Z |  |
| 2 | 01 | deviation | crates/prism-mcp/tests/middleware_gate.rs |  | 计划的端到端摘层反证（摘 require_local_host / require_origin_allowlist 后对应测试变红）实跑不成立：rmcp SDK 自带的 allowed_hosts/allowed_origins 会替它拒掉。落点唯一的反证改由 B 组 sentinel-router 隔离测试承担 | open |  | 2026-07-28T23:59:36.146Z |  |
| 3 | 01 | deviation | scripts/check-deps.sh |  | 计划 01-07 的两条验收项在原 check-deps.sh 下互斥（facade 必须依赖 prism-llm 才能转交密钥，但 prism-engine 在 PURE_CRATES 的整树断言里）；已拆成叶子整树断言 + facade 反向闭包断言 | open |  | 2026-07-29T00:24:01.614Z |  |
| 4 | 01 | deviation | src-tauri/tests/ipc.rs |  | 研究文档示例的 ipc 请求 URL (http://tauri.localhost) 在 macOS 上非本地来源，会让每个命令被 ACL 拒成 'not allowed. Plugin not found'——其错误串与真正的未注册错误同含 'not found'，无负对照时测试会绿着什么都没测 | open |  | 2026-07-29T00:42:43.308Z |  |
| 5 | 01 | deviation | src/App.tsx |  | 冒烟页在真实 Tauri 窗口里不可达（窗口无地址栏，计划的「靠地址栏进入 #/dev」不可执行）；而「hash 是 #/dev 时渲染谁」的路由断言在冒烟页永远够不着的世界里同样全绿——路由正确性不蕴含可达性 | open |  | 2026-07-29T02:21:00.297Z |  |
| 6 | 01 | deviation | src-tauri/capabilities/default.json |  | listen() 被 Tauri v2 ACL 拒绝且 rejection 被吞：capabilities/ 缺失时 ACL 集合编译成 {}；ACL 只管插件命令，generate_handler! 的自有命令不过 ACL——于是 invoke 全部正常、listen 从未注册、计数停在 0 且零报错。vitest 里 event 模块被 mock，这一整类失败在单测中结构上不可见 | open |  | 2026-07-29T02:21:00.347Z |  |
| 7 | 01 | deviation | scripts/check-secrets.sh |  | INFRA-03 未勾：需求文本「支持 Anthropic/OpenAI 兼容端点」半句需 Phase 4 chat client；扫描器与写入侧两半已关闭 | open |  | 2026-07-29T04:11:54.094Z |  |
| 8 | 01 | unrun-verify | src-tauri/tauri.conf.json |  | 01-13 Task 1 的 <human-check> 五步（真实 WebView + dmg 两形态的 CSP 验证）未执行，顺延至 end-of-phase 人工验证 | open |  | 2026-07-29T05:34:36.523Z |  |
| 9 | 01 | unrun-verify | src-tauri/src/lib.rs |  | 01-13 Task 3 的行为断言未执行：tauri dev 终端是否真的打出 settings.rs 的明文 http 告警（sink 非空的端到端证据） | open |  | 2026-07-29T05:34:36.571Z |  |
| 10 | 01 | deviation | src/pages/Settings.tsx |  | Settings.tsx 的成功通知不在任何 live region 里（NoticeLine 的 ok 分支只有颜色无 role），读屏对「已保存」完全静默——同一条 IN-06 推理，本 plan 范围外未动 | fixed |  | 2026-07-29T13:10:05.615Z | 2026-07-29T13:22:54.127Z |
| 11 | 01 | deviation | src-tauri/tauri.conf.json |  | 发布 csp 的 style-src 仍带 'unsafe-inline'（React 内联 style 属性），使 01-24 plan 要求的「csp 整串不含 unsafe-inline」不可满足；已改写为「含该 token 的指令集合精确等于 [style-src ...]」，但这条发布形态的放宽本身未被消除 | open |  | 2026-07-29T13:32:35.552Z |  |
| 12 | 01 | deviation | crates/prism-engine/src/services.rs |  | Engine::list_feedback 的空 project_id 校验无端到端哨兵：01-17 在 PrismHandler 加的 projectId 校验集合覆盖了它（同为 trim().is_empty()），MCP 线上不存在能引出 engine 校验文本的输入；判别性已搬到 facade.rs 对 Arc<dyn FeedbackSource> 的直接调用 | open |  | 2026-07-29T14:00:06.507Z |  |
| 13 | 01 | deviation | .planning/phases/01-foundation-skeleton/01-REVIEW.md |  | 01-REVIEW.md WR-03 第 2 点举的可达性例子 Host: 127.0.0.1:notanumber 实测不成立（http 1.x 的 Authority::try_from 接受非数字端口）；真正分叉的是 127.0.0.1:80/evil 与 127.0.0.1:80@evil.com，已在 01-17 更正并落测 | open |  | 2026-07-29T14:00:06.558Z |  |
| 14 | 01 | unrun-verify | .github/workflows/ci.yml |  | 本 plan 加的三项 workflow 级配置（permissions: contents: read 下 upload-artifact 是否仍可上传、concurrency 是否真的收掉同 commit 的双跑、两个缓存分段是否互不恢复）只有 YAML 可解析 + 推理这一层证据：gh run list 返回 []、origin/main 停在 4cc1347，该 workflow 至今未在 GitHub Actions 上跑过。步骤级断言已全部本机实证 | open |  | 2026-07-29T23:39:44.371Z |  |

````json
[
  {
    "id": 1,
    "kind": "deviation",
    "phase": "01",
    "file": "crates/prism-store/src/open.rs",
    "line": null,
    "description": "迁移先于建池的顺序无行为面哨兵：计划设想的反证实测不成立，目前只由源码顺序断言守住",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-07-28T23:13:51.042Z",
    "resolved_at": null
  },
  {
    "id": 2,
    "kind": "deviation",
    "phase": "01",
    "file": "crates/prism-mcp/tests/middleware_gate.rs",
    "line": null,
    "description": "计划的端到端摘层反证（摘 require_local_host / require_origin_allowlist 后对应测试变红）实跑不成立：rmcp SDK 自带的 allowed_hosts/allowed_origins 会替它拒掉。落点唯一的反证改由 B 组 sentinel-router 隔离测试承担",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-07-28T23:59:36.146Z",
    "resolved_at": null
  },
  {
    "id": 3,
    "kind": "deviation",
    "phase": "01",
    "file": "scripts/check-deps.sh",
    "line": null,
    "description": "计划 01-07 的两条验收项在原 check-deps.sh 下互斥（facade 必须依赖 prism-llm 才能转交密钥，但 prism-engine 在 PURE_CRATES 的整树断言里）；已拆成叶子整树断言 + facade 反向闭包断言",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-07-29T00:24:01.614Z",
    "resolved_at": null
  },
  {
    "id": 4,
    "kind": "deviation",
    "phase": "01",
    "file": "src-tauri/tests/ipc.rs",
    "line": null,
    "description": "研究文档示例的 ipc 请求 URL (http://tauri.localhost) 在 macOS 上非本地来源，会让每个命令被 ACL 拒成 'not allowed. Plugin not found'——其错误串与真正的未注册错误同含 'not found'，无负对照时测试会绿着什么都没测",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-07-29T00:42:43.308Z",
    "resolved_at": null
  },
  {
    "id": 5,
    "kind": "deviation",
    "phase": "01",
    "file": "src/App.tsx",
    "line": null,
    "description": "冒烟页在真实 Tauri 窗口里不可达（窗口无地址栏，计划的「靠地址栏进入 #/dev」不可执行）；而「hash 是 #/dev 时渲染谁」的路由断言在冒烟页永远够不着的世界里同样全绿——路由正确性不蕴含可达性",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-07-29T02:21:00.297Z",
    "resolved_at": null
  },
  {
    "id": 6,
    "kind": "deviation",
    "phase": "01",
    "file": "src-tauri/capabilities/default.json",
    "line": null,
    "description": "listen() 被 Tauri v2 ACL 拒绝且 rejection 被吞：capabilities/ 缺失时 ACL 集合编译成 {}；ACL 只管插件命令，generate_handler! 的自有命令不过 ACL——于是 invoke 全部正常、listen 从未注册、计数停在 0 且零报错。vitest 里 event 模块被 mock，这一整类失败在单测中结构上不可见",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-07-29T02:21:00.347Z",
    "resolved_at": null
  },
  {
    "id": 7,
    "kind": "deviation",
    "phase": "01",
    "file": "scripts/check-secrets.sh",
    "line": null,
    "description": "INFRA-03 未勾：需求文本「支持 Anthropic/OpenAI 兼容端点」半句需 Phase 4 chat client；扫描器与写入侧两半已关闭",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-07-29T04:11:54.094Z",
    "resolved_at": null
  },
  {
    "id": 8,
    "kind": "unrun-verify",
    "phase": "01",
    "file": "src-tauri/tauri.conf.json",
    "line": null,
    "description": "01-13 Task 1 的 <human-check> 五步（真实 WebView + dmg 两形态的 CSP 验证）未执行，顺延至 end-of-phase 人工验证",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-07-29T05:34:36.523Z",
    "resolved_at": null
  },
  {
    "id": 9,
    "kind": "unrun-verify",
    "phase": "01",
    "file": "src-tauri/src/lib.rs",
    "line": null,
    "description": "01-13 Task 3 的行为断言未执行：tauri dev 终端是否真的打出 settings.rs 的明文 http 告警（sink 非空的端到端证据）",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-07-29T05:34:36.571Z",
    "resolved_at": null
  },
  {
    "id": 10,
    "kind": "deviation",
    "phase": "01",
    "file": "src/pages/Settings.tsx",
    "line": null,
    "description": "Settings.tsx 的成功通知不在任何 live region 里（NoticeLine 的 ok 分支只有颜色无 role），读屏对「已保存」完全静默——同一条 IN-06 推理，本 plan 范围外未动",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-07-29T13:10:05.615Z",
    "resolved_at": "2026-07-29T13:22:54.127Z"
  },
  {
    "id": 11,
    "kind": "deviation",
    "phase": "01",
    "file": "src-tauri/tauri.conf.json",
    "line": null,
    "description": "发布 csp 的 style-src 仍带 'unsafe-inline'（React 内联 style 属性），使 01-24 plan 要求的「csp 整串不含 unsafe-inline」不可满足；已改写为「含该 token 的指令集合精确等于 [style-src ...]」，但这条发布形态的放宽本身未被消除",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-07-29T13:32:35.552Z",
    "resolved_at": null
  },
  {
    "id": 12,
    "kind": "deviation",
    "phase": "01",
    "file": "crates/prism-engine/src/services.rs",
    "line": null,
    "description": "Engine::list_feedback 的空 project_id 校验无端到端哨兵：01-17 在 PrismHandler 加的 projectId 校验集合覆盖了它（同为 trim().is_empty()），MCP 线上不存在能引出 engine 校验文本的输入；判别性已搬到 facade.rs 对 Arc<dyn FeedbackSource> 的直接调用",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-07-29T14:00:06.507Z",
    "resolved_at": null
  },
  {
    "id": 13,
    "kind": "deviation",
    "phase": "01",
    "file": ".planning/phases/01-foundation-skeleton/01-REVIEW.md",
    "line": null,
    "description": "01-REVIEW.md WR-03 第 2 点举的可达性例子 Host: 127.0.0.1:notanumber 实测不成立（http 1.x 的 Authority::try_from 接受非数字端口）；真正分叉的是 127.0.0.1:80/evil 与 127.0.0.1:80@evil.com，已在 01-17 更正并落测",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-07-29T14:00:06.558Z",
    "resolved_at": null
  },
  {
    "id": 14,
    "kind": "unrun-verify",
    "phase": "01",
    "file": ".github/workflows/ci.yml",
    "line": null,
    "description": "本 plan 加的三项 workflow 级配置（permissions: contents: read 下 upload-artifact 是否仍可上传、concurrency 是否真的收掉同 commit 的双跑、两个缓存分段是否互不恢复）只有 YAML 可解析 + 推理这一层证据：gh run list 返回 []、origin/main 停在 4cc1347，该 workflow 至今未在 GitHub Actions 上跑过。步骤级断言已全部本机实证",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-07-29T23:39:44.371Z",
    "resolved_at": null
  }
]
````
