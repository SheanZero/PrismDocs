---
schema_version: 1
open_count: 2
waived_count: 0
fixed_count: 0
total_count: 2
last_updated: 2026-07-28T23:59:36.146Z
---

# Broken Windows Ledger

> Cross-phase defect register. `/gsd-ship` blocks while `open_count > 0`.
> Waive with `gsd-tools windows waive <id> "<reason>"` (reason required).
> Mark fixed with `gsd-tools windows fixed <id>`.

| id | phase | kind | file | line | description | status | reason | recorded_at | resolved_at |
|----|-------|------|------|------|-------------|--------|--------|-------------|-------------|
| 1 | 01 | deviation | crates/prism-store/src/open.rs |  | 迁移先于建池的顺序无行为面哨兵：计划设想的反证实测不成立，目前只由源码顺序断言守住 | open |  | 2026-07-28T23:13:51.042Z |  |
| 2 | 01 | deviation | crates/prism-mcp/tests/middleware_gate.rs |  | 计划的端到端摘层反证（摘 require_local_host / require_origin_allowlist 后对应测试变红）实跑不成立：rmcp SDK 自带的 allowed_hosts/allowed_origins 会替它拒掉。落点唯一的反证改由 B 组 sentinel-router 隔离测试承担 | open |  | 2026-07-28T23:59:36.146Z |  |

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
  }
]
````
