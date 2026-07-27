# PrismDocs

## What This Is

PrismDocs 是面向 vibe coder 的工程文档工作台：AI 用英文维护紧凑、技术完备的技术文档（Base 层，唯一真相源、省 token），产品自动投影出用户母语（首发简体中文）的口语化理解层（Lens 层，供人快速 review）。用户在文档上写评论，评论结构化回流给 Claude Code / Cursor 等编码 agent 驱动下一轮迭代；文档以 LLM Wiki + 卡片笔记（Zettelkasten）组织，配套 Chrome 插件把网页资料剪藏进知识库。

主力用户是使用 AI 编码工具、中文（或日文）为母语、英文技术阅读中等的独立开发者（P1）和产品型创始人（P2）。定位是 IDE 之外、面向人的工程知识层——「OKF 之上的人类理解与决策层」。

## Core Value

**「双层文档 + 评论回流」显著降低 vibe coder review AI 文档的成本** —— 若一切从简，必须跑通的是：AI 写英文 Base → 产品投影中文 Lens → 人在 Lens 上批注 → 结构化回流 Claude Code → AI 改 Base → Lens 重投影提醒复核（F1–F4 闭环，AC-4a）。

## Business Context

- **Customer**: 独立 vibe coder / 产品型创始人（中日韩非英语母语为主），已为 AI 编码工具付 $20–200/月
- **Revenue model**: 免费层（单项目 + 有限 Lens 额度，自备 API key 可放宽）→ Pro ~$15/月（不限项目、内置额度、优先新语言）→ 后续团队版按席位
- **Success metric**: 北极星 = 每周完成的「评论→AI 修改→复核通过」闭环数（Closed Loops / week）；激活 = 新用户 7 日内完成首个闭环 ≥40%
- **Strategy notes**: 见 docs/BRD_PrismDocs_MVP.md（v0.2）、docs/PRD_PrismDocs_MVP.md（v0.2）、docs/调研补充_CoWiki与OKF对BRD_PRD的影响.md

## Requirements

### Validated

(None yet — ship to validate)

### Active

<!-- MVP P0 功能 F1–F7。详细 REQ 编号与验收标准见 .planning/REQUIREMENTS.md -->

- [ ] F1 · 项目与文档导入（连接本地文件夹/Git 仓库，导入 MD 为 Base 层，FS 监听同步，frontmatter 解析）
- [ ] F2 · Lens 层生成（Base 文档投影为中文口语版，Block 级锚定，增量重投影，变更高亮，忠实度保障）
- [ ] F3 · 段落级评论（Lens/Base 任意 Block 评论；提问/修改/决策类型；线程状态机；锚点迁移 0 静默丢失）
- [ ] F4 · 评论回流 AI（Feedback Bundle 双通道交付：文件协议 + 本地 MCP Server；闭环回收；agent 贡献溯源）★ 核心闭环
- [ ] F5 · 理解卡片（手写原子卡片，强引导原创表述，双链，注入上下文开关）
- [ ] F6 · Chrome 剪藏插件（网页→净化 Markdown，代码块正确，token 估算，存入知识库）
- [ ] F7 · 上下文组装（勾选文档/卡片/剪藏生成紧凑英文 Context Pack，显示 token，MCP 打通，导出 OKF Bundle）

### Out of Scope

- IDE / 代码编辑器 / 代码生成 / 内置编码 agent — 只对接编码 agent，与其共生而非竞争
- 通用笔记应用 — 不与 Obsidian 抢「人生笔记」，只做工程语境
- 对外发布的文档站 — 不与 Mintlify/GitBook 竞争
- 团队协作与权限体系 — P2/P3 人群后置，MVP 单 Workspace 单用户
- Lens 层直接编辑 — 产品原则（防双层漂移），一切不满通过评论表达
- 服务端 — MVP 本地优先，无我方服务器，文档不经过我方
- 剪藏被评论 — 剪藏是外部素材，理解写进卡片，保持「评论=驱动 AI 改文档」语义纯度

## Context

- **生态背景**：2025–2026 vibe coding 从「让 AI 多产出」转向「帮人理解/验证/记住 AI 产出」。瓶颈从「写」转到「读和验证」；CLAUDE.md/rules 膨胀失效、/compact 后决策蒸发；「理解债」被正式提出（Zettelkasten「用自己的话」是认知科学对策）。
- **被忽视人群**：非英语母语的 vibe coder（中日韩），现有工具（DeepWiki、Kiro、Swimm、markupmarkdown 等）隐含用户全是能流畅读英文的工程师。
- **最接近竞品**：markupmarkdown、Plannotator（2026 年刚起步、单层英文、面向开发团队，无口语层/无卡片/无剪藏）。中期最大威胁：HackMD 向「AI 写、人审」治理层转型。巨头信号：Google Code Wiki（自动代码 Wiki 赛道已巨头化，不正面竞争）。
- **OKF 战略**：Google Open Knowledge Format（2026-06，Markdown 目录 + YAML frontmatter + index.md/log.md）标准化了「AI 那一层」，但不覆盖人类理解/批注/回流层——正好留白给本产品。PrismDocs Base 层与 OKF 天然同构，低成本做到 OKF 兼容 = 无锁定信任卖点。CoWiki（中文开源 LLM Wiki 团队版）已宣布对齐 OKF，其「校验防错误传播」「agent 溯源」机制值得借鉴。
- **核心技术难点/护城河**（按防御力）：双层文档双向同步（block-anchor + 增量重投影，锚点漂移是首要攻坚）> 口语层+卡片的「理解留存」设计 > 评论→AI 结构化回流协议 > 剪藏入口。

## Constraints

- **平台**: macOS（Apple Silicon）首发桌面应用 + Chrome 扩展（MV3，Edge 兼容顺带）；Windows 列为 P1 — 主力用户在 Mac，聚焦单平台降低 MVP 成本
- **架构**: 本地优先，文档/评论/卡片全落本地，不强制上云；单目录可备份（SQLite + 文件）— 数据主权 + 无服务端成本
- **真相源**: 磁盘 Markdown 文件是 Base 层权威副本，PrismDocs 不锁文件；Block ID / 评论 / 卡片存 sidecar 不污染用户源文件 — 与用户 IDE/agent/git 工作流零冲突
- **Agent 接口**: 优先 MCP + 文件协议（`.prismdocs/feedback/*.md`），一级支持 Claude Code，兼容 Cursor，其他 agent 纯文件兜底 — 接入成本压到最低
- **LLM**: 用户自备 API key（Anthropic / OpenAI 兼容端点，支持自定义 base_url），key 存系统钥匙串；订阅制内置额度后置 — 成本转嫁 + 隐私
- **成本**: Lens 投影是主要模型成本，增量重投影（只重生成受影响段落）+ 紧凑 Base 层既是体验也是成本设计；投影调用显示预估 token
- **数据格式**: Base 层存储/导出遵循 OKF v0.1 核心约定（文件即概念、frontmatter 六字段、受控 type 词表、index.md/log.md 保留名）— 换取无锁定互操作
- **性能**: 500 文档 / 2000 卡片规模下全文搜索 <300ms、文档打开 <500ms、FS 变更呈现 <10s

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| 双层文档 = Base(英文,真相源) + Lens(母语,不可编辑投影) | Lens 单向投影从根上避免双层漂移；护城河所在 | — Pending |
| Block 锚定 = 内容哈希+位置启发式，存 sidecar 不写源文件 | 保持用户 Markdown 干净；锚点迁移置信度低时降级为文档级评论，绝不静默丢失 | — Pending |
| 回流双通道 = 文件协议(P0) + 本地 MCP Server(P0) | 未装 MCP 的用户靠文件变更检测兜底闭环（降级路径可用，AC-4c） | — Pending |
| OKF 兼容为架构级决策（开发前定） | 低成本换无锁定信任卖点；CoWiki 已对齐，不跟进会失分 | — Pending |
| 卡片正文不提供 AI 代写（刻意缺失） | Zettelkasten「用自己的话」是理解留存的机制，AI 代写会破坏之 | — Pending |
| 产品定名 PrismDocs（棱镜：一份 Base 折射多个谱层） | 已选定 2026-07-26；遗留：上线前域名/商标终检（Prism Software 商标邻近等弱碰撞） | — Pending |
| MVP 里程碑：F1–F4（双层+评论闭环）先行，F5–F7 随后 | 核心假设验证优先于周边能力 | — Pending |

## Open Questions

<!-- 摘自 PRD §8，需在设计/开发前决议 -->

- Q1 Lens 投影模型选型（成本 vs 口语质量）：倾向快速模型打底 + 「需要决策」段落用强模型复核，M0 阶段 A/B
- Q2 Base 层在 PrismDocs 内可编辑（REQ-1.6）是否与「评论驱动」原则打架：倾向保留但入口弱化（默认只读，显式解锁），观察内测
- Q3 评论/卡片存 sidecar vs 同步进 git：倾向 MVP sidecar + 导出备份，git 同步作为 P1「便携模式」
- Q6 受控 type 词表范围与扩展策略：倾向内置九词表 + 设置内登记制起步
- Q7 是否提供「frontmatter 直写入源文件」模式：倾向默认 sidecar + 项目级 opt-in，进设计评审

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
*Last updated: 2026-07-27 after initialization*
