---
gsd_state_version: 1.0
milestone: v0.2
milestone_name: milestone
current_phase: 01
current_phase_name: foundation-skeleton
status: executing
stopped_at: Completed 01-10-PLAN.md（gap 1 engine+前端两半均已关闭）
last_updated: "2026-07-29T04:00:21.051Z"
last_activity: 2026-07-29
last_activity_desc: Phase 01 execution started
progress:
  total_phases: 1
  completed_phases: 0
  total_plans: 13
  completed_plans: 10
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-07-28)

**Core value:** 「评论 → AI 修改 → 复核通过」的闭环：10 分钟看懂 AI 英文文档、批注两句让它接着干、评论在 AI 大规模重写下 0 静默丢失。北极星 = 每周闭环数。
**Current focus:** Phase 01 — foundation-skeleton

## Current Position

Phase: 01 (foundation-skeleton) — EXECUTING
Plan: 2 of 13
Status: Ready to execute
Last activity: 2026-07-29 — Phase 01 execution started

Progress: [████████░░] 77%

## Performance Metrics

**Velocity:**

- Total plans completed: 0
- Average duration: -
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

**Recent Trend:**

- Last 5 plans: -
- Trend: -

*Updated after each plan completion*
**Per-Plan Metrics:**

| Plan | Duration | Tasks | Files |
|------|----------|-------|-------|
| Phase 01 P01 | 39min | 3 tasks | 53 files |
| Phase 01 P02 | 8min | 2 tasks | 8 files |
| Phase 01 P03 | 68min | 2 tasks | 8 files |
| Phase 01 P04 | 7min | 2 tasks | 11 files |
| Phase 01 P05 | 38min | 2 tasks | 7 files |
| Phase 01 P06 | 26min | 2 tasks | 10 files |
| Phase 01 P07 | 31min | 2 tasks | 9 files |
| Phase 01 P08 | 11min | 2 tasks | 9 files |
| Phase 01 P09 | 81min | 3 tasks | 19 files |
| Phase 01 P10 | 10min | 2 tasks | 4 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Init]: Roadmap 采用调研文档 Phase 划分 + 修正 A1/A2/A3；关键路径 1→2→3→5→6，Phase 4 ∥ 5
- [Init]: Phase 1 承载五项不可逆决策（单写者 SQLite + 读池、FTS5 CJK tokenizer、keyring-core、prism-mcp trait 反转、notify-then-fetch）
- [Init]: INFRA-04/05 为跨切预算——自 Phase 1 起执行，Phase 8 验证级验收
- [Init]: comrak 唯一锚定真相源；MigrationResult/ChangeSet 接口在 Phase 3 冻结（TD-01 §7）
- [Phase 1]: workspace members 用 crates/* 通配 + src-tauri：新增 crate 无需改动根 Cargo.toml
- [Phase 1]: prism-mcp 的 protocol_version() 用 static 承载 rmcp ProtocolVersion::LATEST（内含 Cow，const 提升不适用）
- [Phase 1]: 暂无自然用途的依赖（prism-llm serde、prism-anchor similar/ulid、prism-mcp axum/tokio）以依赖可用性单测引用，不提前发明后续 phase 的公开 API
- [Phase 1]: INFRA-01 不在 plan 01-01 勾选：该需求横跨本 phase 的 7 个 plan，事件总线与 Channel 有序流在 01-04/01-08/01-09
- [Phase ?]: 依赖方向断言用 herestring 而非管道喂 grep：pipefail + grep -q 早退会因 SIGPIPE 让四条断言全部静默恒绿
- [Phase ?]: 覆盖率 Phase 1 只测量不设阈值（engine 85.48% / 前端 10%），Phase 2 按排除已登记人工与 ignored 路径后 >=80% 开硬闸门
- [Phase ?]: schema v1 定案方案 A：external-content FTS5 + rowid_pk 显式 INTEGER PRIMARY KEY + 三同步触发器 + STRICT 表，索引粒度保持默认全粒度（不声明该选项，降粒度会废掉 4 字中文 MATCH）
- [Phase ?]: 只读池用 SQLITE_OPEN_READ_WRITE + query_only=ON 而非 READ_ONLY flags：只读连接在崩溃后 -shm 缺失时无法重建它
- [Phase ?]: 「颠倒迁移与建池顺序」的行为反证实测不成立（六个并发测试仍全绿），改用 open.rs 内的源码顺序断言作为常驻哨兵
- [Phase ?]: prism-types 的依赖上限就是 serde + thiserror 两项：它是 prism-mcp 与 prism-engine 的共同汇点，任何新增依赖会同时压到两侧（D-09）
- [Phase ?]: service trait 一律同步（非 async）：底层 rusqlite 本就阻塞，同步 trait 天然 object-safe，consumer 用 spawn_blocking 调用
- [Phase ?]: 跨边界转发第三方错误时只保留已核实安全的 Display 文本：keyring_core::Error 的 derive Debug 会打印原始密钥字节，而 unwrap()/tracing 的 ?err 走的正是 Debug
- [Phase ?]: 持有密钥的类型手写 Debug 输出占位串，并刻意不实现 Display——缺席让 format!("{key}") 成为编译错误而非运行期泄漏
- [Phase ?]: 钥匙串 service/account 命名（PrismDocs / llm_api_key / mcp_bearer_token）是跨二进制契约，固化于 docs/keychain-naming.md；prismdocs-helper 因 D-10 必须自带字面量副本
- [Phase ?]: FTS 表在 SQL 中不能起别名：MATCH 左操作数必须是 fts5 表名，JOIN 打在 d.rowid_pk = documents_fts.rowid 上
- [Phase ?]: LIKE 回退分支补模式语言层转义（%/_/\ + ESCAPE）：与未转义 MATCH 是同一类漏洞的两个面
- [Phase ?]: settings 的 base_url 校验与密钥键名守卫都长在 set_setting 内部：放调用方是约定，放写入路径才是机制
- [Phase ?]: 01-06: prism-mcp 的 bearer 在 McpDeps 中为私有字段 + pub(crate) expose_bearer，不是 pub 字段——token 的取用点在代码搜索中唯一可见
- [Phase ?]: 01-06: 三层门禁一律返回 403 + 空正文（不给 bearer 缺失单开 401）——状态码差异本身就是逐层试探的信息源（T-01-29）
- [Phase ?]: 01-06: rmcp SDK 侧 allowed_hosts/allowed_origins 与应用层中间件配成同一份做防御纵深；代价是端到端摘层反证失效，改由 sentinel-router 隔离测试承担
- [Phase ?]: 01-07: check-deps.sh 的 single-egress 拆两条——叶子 crate 整树断言不变，prism-engine 改为「直接依赖里没有 + cargo tree --invert 反向闭包里除 prism-llm 外无 prism-*」。原断言与「密钥唯一经 prism-llm 转交」互斥（src-tauri 只依赖 prism-engine，shell 通往钥匙串必经 facade）
- [Phase ?]: 01-07: 端到端注入测试的判别性不能落在「响应里有空集」上——Phase 1 的 list_feedback 返回空 vec，空结果与「handler 根本没调注入 trait」不可区分；改落在 engine 自己写的校验文本上（并实测确认 rmcp 对该参数无兜底校验）
- [Phase ?]: 01-07: 等事件的测试一律包 timeout，且前置条件断言要移出判别性测试——前者防「反证挂住而非变红」，后者防「反证落在前置条件上而非被守的那条断言上」
- [Phase ?]: 01-07: facade 方法一律同步（spawn_blocking 归调用方）：改成 async fn 会废掉 std::sync::MutexGuard !Send 对「跨 await 持写锁」的编译期保护
- [Phase ?]: 01-08: ipc 进程内测试的来源 URL 必须等于 tauri.conf.json 的 devUrl —— http://tauri.localhost 是 Windows 形态，macOS 下 is_local=false 会让每个命令被 ACL 拒成 'not allowed. Plugin not found'（与未注册错误同含 not found）
- [Phase ?]: 01-08: check-deps.sh 补第六条 shell-egress（src-tauri 不得直接依赖 prism-llm），形态同 facade-egress；反证证明原五条对该缺口全部不敏感
- [Phase ?]: 01-08: 有序性断言用序列比较而非集合比较；命令注册断言必须配未注册命令的负对照 + 断言 Ok（而非仅'错误不像未注册'）
- [Phase ?]: 01-09: Tauri v2 的 ACL 只管插件命令——generate_handler! 注册的自有命令不过 ACL。capabilities/ 缺失时 listen() 被拒而 invoke 全部正常，表现为「点了没反应且零报错」；任何新的 @tauri-apps/api 用法都需补一行 capability，且新调用点必须自己呈现 rejection
- [Phase ?]: 01-09: 可达性是独立于路由正确性的性质——「hash 是 X 时渲染谁」全绿不代表用户到得了 X（Tauri 窗口没有地址栏）。dev-only UI 用 import.meta.env.DEV 门控并配一条生产构建断言（grep dist 产物）
- [Phase ?]: 01-09: 勾选 INFRA-01（成功标准 2 的真实 WebView 两条通路由人工验证兑现）；INFRA-03 不勾——prism-llm 只有 secrets.rs，无 chat client，「支持 Anthropic/OpenAI 兼容端点」到 Phase 4
- [Phase ?]: 01-10: 凭据守卫扩在 validate_base_url 内部而非新加调用点——set_setting 一行未动，「机制而非约定」的设计陈述完整保住（T-01-43：绕过界面直接 invoke 不改变结果）
- [Phase ?]: 01-10: 密钥容器的边界必须建在**值**上而不只是键名上——is_secret_like_key 防的是键名，而 llm.base_url 这个键名完全正常，凭据藏在值里；userinfo 与 ?api-key= 是同一个洞的两个面
- [Phase ?]: 01-10: 拒绝面扩张时不加 StoreError 变体（复用 InvalidUrl）——加变体会连带要求 commands.rs::map_err 与前端 ERROR_COPY 同步扩表，那是 IPC 短码契约的变更；「是凭据还是 scheme」的区分只在前端本地校验层做得到
- [Phase ?]: 01-10: 前端体验层校验返回错误码而非布尔（localUrlIssue 取代 looksLikeHttpUrl）；判定面与 engine 逐项对齐（scheme/userinfo/query/fragment），避免「前端放行、engine 拒绝」在正常输入上出现
- [Phase ?]: 01-10: 计划里的两条断言实测不成立须就地修正——单字符用户名 u 让「错误串不含用户名」恒假；type=text 的端点输入框必须回显用户输入，document.body.innerHTML 级的不回显断言在任何正确实现下都红。不回显的守法面是**错误文案**，不是整个 DOM

### Pending Todos

None yet.

### Blockers/Concerns

- [Phase 3 前]: TD-01 阈值 T_high/T_low 与权重待 Track B 真实 agent diff 语料标定（Phase 3 内 harness 完成）
- [Phase 3 关闭前]: A→B→A 降级锚点复活语义未定义，需修订为 TD-01 v0.2（阻塞 phase close，不阻塞 start）
- [Phase 5/6 计划前]: F3 OQ-2/OQ-3、F4 OQ-1（declined 语义）需拍板（阻塞 F4 状态机）
- [Phase 4 前]: Q1 速读区模型档位待 M0 评测定档
- ~~[Phase 1]: rmcp 2.2 feature-flag 确切名称需对照 README 核验（5 分钟检查）~~ — RESOLVED (01-01)：`rmcp = { version = "2.2", features = ["server", "transport-streamable-http-server"] }` 已在 prism-mcp 中实际编译通过
- [数据勘误]: REQUIREMENTS.md 原 Coverage 写 51 条，实际 v1 REQ-ID 为 61 条，已于 roadmap 创建时更正
- [每个 plan 执行时]: 反证本身需要被验证（01-03 后第三次出现）：01-05 的计划反证 C 实跑不成立，暴露了 LIKE 分支缺阴性对照；触发器 DELETE 路径的验证按计划写法恒真（JOIN 掩盖了陈旧索引条目）；01-06 的两条计划反证（从 build_router 摘掉 Host / Origin 中间件）实跑**全绿**——rmcp SDK 自带的 allowlist 替它拒掉了。跑反证时要看**落点**（红在哪一条断言）而非只看红绿；当被测层之上还有第三方兜底时，反证必须把被测层放进一个**没有兜底**的最小链路里（01-06 的 sentinel-router 隔离测试即此形态）。
- [Phase 6 计划时]: rmcp SDK 的 Host 拒绝响应体为 "Forbidden: Host header is not allowed"，与本项目 T-01-29 的无差别拒绝口径不一致。当前应用层在外先拒使其不可达，但若 Phase 6 调整中间件顺序或 allowlist 使两者不再等价，SDK 的正文会泄漏落点
- ~~[01-09 / Phase 2+]: 若将来给项目加 capabilities/ 目录，has_app_acl_manifest 变 true，即使本地来源也会走 ACL —— src-tauri/tests/ipc.rs 届时需加一份测试用 capability，否则集体变红~~ — 实测不成立 (01-09)：capability 已加入，`cargo test -p prismdocs-shell --features test --test ipc` 仍 2 passed。被测的十个命令都是 `generate_handler!` 注册的自有命令、不受 ACL 管辖；ACL 生效影响的是**插件**命令，ipc 测试里一个都没有。若 Phase 6 给 ipc 测试加插件命令用例，那时才需要测试用 capability
- [Phase 2+ 每次新增前端 Tauri API 用法]: 任何新的 `@tauri-apps/api` import（fs/dialog/window/webview/http…）都必须在 `src-tauri/capabilities/default.json` 补一行权限，**其缺席表现为静默无操作而非报错**（ACL 只管插件命令，自有命令不过 ACL）；且新调用点必须自己接住并呈现 rejection，否则连「是不是 capability 缺了」都无从判断。`capabilities.test.ts` 挡得住「顺手加个 fs:default」的过宽修复，挡不住忘记加
- [Phase 2+ 每次写前端交互测试]: 单测会替被测系统假设掉前置条件——jsdom 替用户完成「输入 hash」（01-09 缺陷 1：冒烟页在真实窗口不可达而路由断言全绿）、mock 替运行时完成「ACL 放行」（01-09 缺陷 2）。两者的症状都是「什么都没发生，也没有报错」。这是 01-06 / 01-08 那族问题的第三、第四个变种，共同解药只有「把被测性质放进一个没有替身的链路里跑一次」
- [01-11 完成前] INFRA-03 仍不勾：01-10 只关闭了写入侧（凭据型 base_url 不入库），静态扫描能否看见明文密钥由 01-11 关闭；且需求文本的「支持 Anthropic/OpenAI 兼容端点」半句要到 Phase 4 才有 chat client（沿用 01-09 的同一判据）

## Deferred Items

Items acknowledged and carried forward from previous milestone close:

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| *(none)* | | | |

## Session Continuity

Last session: 2026-07-29T03:59:34.615Z
Stopped at: Completed 01-10-PLAN.md（gap 1 engine+前端两半均已关闭）
Resume file: None
