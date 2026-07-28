# PrismDocs

## What This Is

PrismDocs 是 vibe coder 的多项目工程知识工作台（macOS 桌面应用，Tauri v2）：集中管理 Claude Code / Codex 等编码 agent 在各仓库生成的英文技术文档（Base 层，唯一真相源、OKF 兼容、紧凑省 token），为每份文档生成中文速读区（3–5 句摘要 + ❓ 需决策清单 + 变更摘要）；用户在文档上写段落级评论，评论结构化回流给 AI 驱动下一轮迭代，每次变更可追溯到驱动它的评论；项目间文档相互引用、契约可订阅——上游（如 server 的 API spec）变更时自动警示下游核对，让独立开发的多个仓库不产生偏差。

面向 P1 独立 vibe coder（中文母语、英文技术阅读能力中等，用 Claude Code / Cursor 开发）；P2 产品型创始人为次要人群（随全文 Lens P1 回归再覆盖）。

## Core Value

**「评论 → AI 修改 → 复核通过」的闭环**：用户能在 10 分钟内看懂 AI 写的英文文档、批注两句让它接着干、且评论在 AI 大规模重写下 0 静默丢失。北极星指标 = 每周闭环数（跨项目闭环计入）。

## Business Context

- **Customer**: 独立 vibe coder（中文母语，已为 AI 编码工具付 $20–200/月）
- **Revenue model**: 免费层（单项目）+ Pro ~$15/月（不限项目、解锁 F8 跨项目知识层、内置模型额度）
- **Success metric**: 北极星 = 闭环数/周；激活 = 新用户 7 日内首闭环 ≥40%；W4 留存 ≥30%
- **Strategy notes**: docs/BRD_PrismDocs_MVP.md §9–10（指标与商业模式）、§11（风险）

## Requirements

### Validated

(None yet — ship to validate)

### Active

MVP 范围 = 主 PRD v0.3 的 F1–F5、F7、F8 全部 P0 REQ（详见 .planning/REQUIREMENTS.md 与 docs/ 各子 PRD）：

- [ ] F1 项目与文档导入：本地文件夹/Git 仓库导入 .md 为 Base 层，FS watcher 实时同步（磁盘权威），文档身份识别与版本快照，frontmatter 解析
- [ ] F2′ 速读区：每文档一次 LLM 调用生成中文速读区（摘要 + ❓ 决策清单强制附原文摘录 + 变更摘要），Base 变更后自动重生成，缓存持久化；Base 视图 Block 级变更条与已读基线
- [ ] F3 段落级评论：Block 锚定评论（提问/修改要求/approve/reject），线程与状态机，锚点迁移 0 静默丢失（AC-3b，发布门槛）
- [ ] F4 评论回流 AI ★ 核心闭环：Feedback Bundle 双通道交付（文件协议 + MCP server D-07），三信号闭环回收，agent 贡献溯源（REQ-4.7），变更时间线与 Block 溯源（REQ-4.8），复核界面（Base diff + 评论并排）
- [ ] F5 理解卡片：手写原子卡片 + 原创引导，双链跨项目候选，注入上下文开关
- [ ] F7 上下文组装：Workspace 级勾选文档/卡片生成紧凑英文 Context Pack，token 实时显示，MCP `get_context_pack`
- [ ] F8 跨项目知识层 ★ 防偏差：跨项目引用（Xref sidecar）、契约标记与订阅（可到 Block 粒度）、结构漂移警示 → 一键下游核对反馈（复用 F4 闭环，agent 侧零新协议）
- [ ] Block 锚定引擎（护城河，TD-01 已冻结接口）：comrak 唯一真相源，一次计算四消费方（F3 锚点迁移 / F4 命中判定 / F2 变更条 / F8 订阅命中），置信度三档行为契约

### Out of Scope

- F6 Chrome 剪藏插件 — v0.3 决议降 P1：三个核心 job 均不涉及外部网页素材，是唯一可完整剥离的独立工程线（需求全文保留于子 PRD-F6）
- 全文 Lens（按 Block 口语化投影、三视图、增量重投影）— 降 P1，回归条件数据驱动预先写死（BRD §6.1 / 调研_技术基建 §3.5）；MVP′ 以速读区验证「中文理解层是否够用」
- 语义级漂移检测（LLM 比对上下游断言矛盾）— P1；MVP 只做结构信号（订阅 Block 命中）
- IDE / 代码编辑器 / 代码生成 / 内置编码 agent — 定位为 agent 之外面向人的知识层，只对接不替代
- 通用笔记应用 — 不与 Obsidian 抢"人生笔记"，只做工程语境
- 对外发布文档站 — 不与 Mintlify/GitBook 竞争
- 团队协作与权限体系 — MVP 单人；团队版（共享变更时间线、git 便携模式）为 P1 方向
- 理解层直接编辑 — 产品原则：速读区/Lens 只读，意见通过评论表达，从根上防双层漂移
- Windows / Mac App Store — macOS (Apple Silicon) 直发公证 DMG 首发，不进 App Sandbox；Windows P1、MAS P2
- 服务端 / 云同步 — MVP 本地优先零服务端，文档不经过我方服务器

## Context

**文档资产（docs/，15 份，随版本演进持续同步）**：

- L1: BRD v0.3（商业论证、竞品、痛点 Top 9、里程碑 M0–M3）
- L2: 主 PRD v0.3（信息架构、OKF 兼容约定 §2.5、F1–F8 需求总述、agent 协议 §4、非功能 §5、发布标准 §7、开放问题 Q1–Q9）
- L2 调研 ×3：CoWiki/OKF 影响、技术基建与开发 Phase 划分 v0.2（含 Phase 0–9 表）、整体构想 v2 多项目知识层
- L3: 子 PRD F1–F8（F4 v0.3 为闭环核心与对外协议定义，建议最先精读）
- L4: TD-01 Block 锚定与迁移契约 v0.1 ★ 全系统接口冻结点（阈值 T_high/T_low 待 M0 Track B 标定后 v0.2 定稿）

**领域对象**：Workspace → Project → Document(Base) → Block → Comment；Document 1:1 Digest（速读区）；Card / Clip(P1)；Xref（跨项目引用 sidecar）；Contract（契约订阅）；Context Pack / Feedback Bundle。

**开发 Phase 划分**（已决定采用《调研_技术基建与开发Phase》v0.2 §4 + 构想 v2 §4 修订，作为 roadmap 基础）：

1. 基建骨架（Cargo workspace + Tauri 薄 shell + schema v1 + keyring）
2. F1 导入与同步
3. 锚定引擎 ★（TD-01 实现 + 标定 harness；接口冻结点）
4. F2′ 速读区（可与 5 并行；含 prompt 评测 harness）
5. F3 评论（可与 4 并行）
6. F4 回流闭环 ★★（关键路径终点，AC-4a 全闭环可验证）
7. F5 卡片 + F7 Context Pack + F8 跨项目知识层
8. 发布准备（压测、埋点、签名公证、内测支撑；原 Phase 9）

关键路径 1→2→3→5→6；Phase 4 与 5 可并行。M0 三赛道中用户走查线下进行；锚点标定与 prompt 评测的可编码基建并入 Phase 3/4 交付。

**M0 待标定/待拍板项（进入对应 Phase 前解决）**：Q1 速读区模型档位（M0 评测定档）；TD-01 阈值与权重（Track B 标定）；F3 OQ-2/OQ-3、F4 OQ-1（declined 语义）、F7 Q3（staleness 标注）——均有倾向意见，阻塞 F4 状态机的需在 Phase 5/6 计划前拍板。

## Constraints

- **Tech stack**（调研 2026-07-28 对照 crates.io 验证 pin）: Tauri v2 + React 19 + Vite 7（前端仅渲染）；纯 Rust Engine Facade workspace（不依赖 tauri，可独立测试，D-01）：prism-store（rusqlite 0.40 + r2d2 + FTS5 + rusqlite_migration）、prism-fs（notify 8 + debouncer-full）、prism-parse（comrak 0.54 sourcepos）、prism-anchor（blake3 + similar 3.1）、prism-llm（reqwest 0.13 + async-openai + eventsource-stream + keyring 4.1）、prism-mcp（rmcp 2.2 StreamableHttpService + axum 0.8）
- **锚定真相源**: comrak 是唯一 Block 边界真相源，前端 react-markdown 仅渲染——两个 parser 各自分块必然锚点漂移（What-NOT-to-use 首条）
- **不污染原则**: Block ID / 评论 / Xref / 元数据全部存 sidecar（~/Library/Application Support/PrismDocs/，按 project-id 索引，D-13）；用户 repo 内仅 .prismdocs/ 协议产物；frontmatter 写回按字节保留原块只替换正文（round-trip `git diff` 0 变化）
- **MCP 传输 (D-07)**: app 自身托管 loopback streamable HTTP（127.0.0.1）+ per-install bearer token（钥匙串）+ Origin allowlist，无子进程；配套轻量 CLI helper（headersHelper + SessionStart hook check-feedback）；子 PRD-F4 早期 stdio 方案已作废
- **性能**: 500 文档/2000 卡片下搜索 <300ms、文档打开 <500ms、FS 变更呈现 <10s；单文档锚点迁移 P95 <300ms
- **隐私/密钥**: 本地优先、断网可读可评可写卡；API key 存系统钥匙串（keyring 直连，不用 stronghold）；文档内容仅发送到用户配置的 LLM 端点；埋点 opt-in 本地暂存
- **发布门槛**: 锚点 0 静默丢失（AC-3b）是 MVP 发布标准，非普通验收项
- **Timeline 参照**: BRD M1 = 6–8 周（对应 Phase 1–6）；MVP′ 缩围后基本可达

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| 全文 Lens 降级为速读区（MVP′） | Lens 占 F2 复杂度 ~3/4 且是最大不确定性中心；速读区是价值密度最高部件；回归条件数据驱动预先写死 | — Pending（内测验证） |
| F8 跨项目知识层进 MVP | 构想 v2 第三 job；四个依赖引擎全现成，工程量小（≈1–1.5 周）；第二条护城河 | — Pending |
| F6 剪藏降 P1 | 三个核心 job 均不涉网页素材；唯一可完整剥离的工程线 | — Pending |
| D-07: MCP = app 内嵌 loopback streamable HTTP | 无子进程、token 走钥匙串；作废 stdio 代理方案（不留两个版本） | — Pending |
| comrak 唯一锚定真相源；Block ID = 不透明 ULID + 属性匹配迁移 | ID 由内容派生则内容一改 ID 即变，锚定无从谈起（TD-01 §2） | — Pending |
| 锚定引擎一次计算四消费方，输出接口最先冻结 | F3/F4/F2/F8 共用，是全系统心脏（TD-01 §7 已冻结） | — Pending |
| 置信度非对称原则：静默错迁代价 ≫ 多余降级 | 标定硬约束 silent-wrong = 0，T_high 宁高勿低 | — Pending |
| OKF 兼容（受控 type 词表 + sidecar 物化导出） | 无锁定信任卖点；CoWiki 已对齐 OKF，不跟进失分 | — Pending |
| macOS 直发公证 DMG，不进 App Sandbox | 省掉安全书签等一整类沙盒问题；MAS 列 P2 | — Pending |
| Roadmap 采用调研文档 Phase 划分（Phase 1–9 全量，M0 走查线下） | 依赖关系与退出标准已在文档中论证；GSD 覆盖全部可编码交付 | — Pending（2026-07-28 初始化时用户拍板） |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-07-28 after initialization*
