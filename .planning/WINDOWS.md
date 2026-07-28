---
schema_version: 1
open_count: 1
waived_count: 0
fixed_count: 0
total_count: 1
last_updated: 2026-07-28T23:13:51.042Z
---

# Broken Windows Ledger

> Cross-phase defect register. `/gsd-ship` blocks while `open_count > 0`.
> Waive with `gsd-tools windows waive <id> "<reason>"` (reason required).
> Mark fixed with `gsd-tools windows fixed <id>`.

| id | phase | kind | file | line | description | status | reason | recorded_at | resolved_at |
|----|-------|------|------|------|-------------|--------|--------|-------------|-------------|
| 1 | 01 | deviation | crates/prism-store/src/open.rs |  | 迁移先于建池的顺序无行为面哨兵：计划设想的反证实测不成立，目前只由源码顺序断言守住 | open |  | 2026-07-28T23:13:51.042Z |  |

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
  }
]
````
