---
schema_version: 1
open_count: 6
waived_count: 0
fixed_count: 0
total_count: 6
last_updated: 2026-07-29T02:21:00.347Z
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
  }
]
````
