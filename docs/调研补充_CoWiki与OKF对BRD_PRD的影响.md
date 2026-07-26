# 补充调研：CoWiki 与 Google Open Knowledge Format（OKF）对 BRD / PRD 的影响分析

| 项目 | 内容 |
|---|---|
| 文档类型 | 补充调研与变更建议 |
| 版本 | v0.1 |
| 日期 | 2026-07-26 |
| 作用对象 | BRD v0.1、主 PRD v0.1（建议合并进各自 v0.2） |

---

## 1. CoWiki 调研结论

**是什么**：[CoWiki](https://cowiki.ai/)（cowiki.ai）——"人与 AI 共建的下一代团队 Wiki"，中文团队开发（创始人：V2EX 用户"微扰"），代码开源，当前处于早期 waitlist 阶段。自我定位就是 **"LLM Wiki 的团队版"**（[V2EX 发布帖](https://www.v2ex.com/t/1228349)）。

**核心机制**：

- **AI 编译管线**：自动收录 → 编译 → 校验 → 人工审核，四阶段把散落信息（链接导入、会议转写、云端收集）沉淀为 Wiki 条目
- **Git 驱动**：AI 行为可审计、可追溯、可回退；借用 Git 协作模型但隐藏工程复杂度
- **多 Agent 协同**：多智能体 24/7 主动收集信息、自动冲突检测与解决；核心问题意识是"错误信息进入共享上下文后会被 agent 放大"（错误传播）
- **Agent 身份与贡献审计**：验证 agent 身份与贡献历史
- **本地优先 + 明确宣布对齐 OKF**（数据可移植性）
- 中英双语界面

**与 PrismDocs 的关系判断**：

| 维度 | CoWiki | PrismDocs |
|---|---|---|
| 核心对象 | 团队通用知识（信息沉淀为 Wiki） | 单个项目的工程文档（spec/plan/architecture）与代码开发闭环 |
| 主循环 | 信息流入 → AI 编译 → 人审入库 | AI 写文档 → 人在口语层批注 → 评论回流编码 agent → 复核 |
| 人的角色 | 知识库的审核者 | 开发方向的决策者 |
| 双层/双语文档 | 无（双语只是 UI） | 核心差异化 |
| 与编码 agent（Claude Code/Cursor）闭环 | 无 | 核心功能（F4） |
| 卡片"用自己的话" | 无 | 核心机制（F5） |

**结论**：CoWiki 不是正面竞品，但它是**同一趋势（LLM Wiki 产品化）里离中文市场最近的邻居**，且"开源 + 中文 + OKF 对齐"三点与我们的目标用户重叠。应列入 BRD 竞品表与风险表。同时它验证了两件事：LLM Wiki 概念在中文开发者社区有真实热度；"人工审核门禁防止错误进入 AI 上下文"的问题意识与我们的评论复核机制同源——这反过来强化了 BRD 的立论。

另注：调研中同时发现 **Google Code Wiki**（[codewiki.google](https://codewiki.google/)，2025 年 11 月推出）——自动扫描仓库生成结构化 Wiki、随代码变更同步、Gemini 问答。它挤压的是 DeepWiki 所在的"自动代码 Wiki"赛道，进一步验证"只做生成不做审阅闭环"的赛道已巨头化，我们不应与之正面竞争（与 BRD 现有定位一致，但竞品表应更新）。

---

## 2. Google OKF 调研结论

**是什么**：[Open Knowledge Format](https://cloud.google.com/blog/products/data-analytics/how-the-open-knowledge-format-can-improve-data-sharing)（2026-06-12 发布，v0.1，[开源 repo](https://github.com/GoogleCloudPlatform/knowledge-catalog)）——Google Cloud 提出的厂商中立规范，把民间自发的 "LLM Wiki 模式"（Karpathy 的 LLM Wiki gist、Obsidian vault 接 agent、AGENTS.md/CLAUDE.md、index.md+log.md 仓库）**标准化为可移植的知识包格式**。

**规范要点**（对我们有直接工程意义）：

- **Bundle = 一个 Markdown 文件目录**；文件路径即概念身份，一个概念一个文件
- **每个文件 = YAML frontmatter + Markdown 正文**；frontmatter 仅 `type` 必填，标准字段还有 `title` / `description` / `resource` / `tags` / `timestamp`，允许自由扩展字段
- **保留文件名**：`index.md`（渐进式导航）、`log.md`(变更编年史)
- **跨链接用标准 Markdown 链接**，目录因此成为可查询的图
- 纯文本、无运行时、无 SDK、git/tarball 即可分发；Apache 2.0
- Google Knowledge Catalog 已支持摄取与服务 OKF；生态明确期待 IDE、搜索、agent 等消费方

**社区批评**（[Marc Bara 的分析](https://medium.com/@marc.bara.iniesta/googles-new-format-for-agent-context-a-standard-or-just-a-folder-82fb21d92041)）：OKF 只解决了"容器标准化"，没解决语义互操作——`type` 值没有注册词表（一家写 `BigQuery Table` 另一家写 `table`）、链接无类型、无信任/溯源、无检索、无权限。结论是"共享上下文存储，而非共享理解方式"。

**对我们的战略含义**：

1. **方向被验证**：BRD 的核心叙事（"文档结构结合 LLM Wiki""给 AI 的层保持紧凑 Markdown"）现在有了 Google 背书的行业标准语言。BRD §3.2 原来引 llms.txt 作为"双套文本已被接受"的证据，OKF 是更强的证据。
2. **免费的互操作红利**：PrismDocs 的 Base 层数据模型与 OKF 几乎天然同构（Markdown 文件、目录即结构、frontmatter 元数据）。**以极小成本做到 "OKF 兼容"，即可宣称：你的 PrismDocs 知识库可被任何 OKF 消费方（包括 Google 生态的 agent）直接读取，无锁定**。这直接回应用户的数据主权焦虑，也是市场话术（"OKF-compatible"）。CoWiki 已宣布对齐 OKF——我们不跟进会在对比中吃亏。
3. **OKF 没做的正是我们的产品**：OKF 是格式，不管"人怎么看得懂、怎么批注、怎么回流"。Lens 层、评论闭环、理解卡片全部在 OKF 规范之外。**OKF 把地基标准化了，等于帮我们把"地基之上"的价值切得更清楚**。
4. **风险**：OKF v0.2+ 计划加入 trust/provenance 与 agentic metadata；Google 若沿此向上做工具层（结合 Code Wiki），会从两头（格式 + 自动 Wiki）逼近。应对仍是 BRD 既定策略：占住"面向人的理解与决策层"。

---

## 3. 对 BRD 的补充建议（合并进 BRD v0.2）

| # | 位置 | 变更建议 | 优先级 |
|---|---|---|---|
| B1 | §3.1 竞品表 · 赛道一 | 增加 Google Code Wiki（自动代码 Wiki 赛道已巨头化的证据）；差异说明照旧："只生成不闭环" | 高 |
| B2 | §3.1 竞品表 · 新增行 | 增加 CoWiki：LLM Wiki 团队版，开源、中文、OKF 对齐、AI 编译管线+人工审核；差异：团队知识沉淀 vs 工程开发闭环，无双层/无评论回流编码 agent | 高 |
| B3 | §3.2 市场空白 | 将 llms.txt 论据升级为 OKF：Google 已把 "LLM Wiki + Markdown + frontmatter" 标准化，"给 AI 与给人两套文本"的行业共识进一步坐实；同时明确 OKF 不覆盖人类理解层——空白依旧成立且边界更清晰 | 高 |
| B4 | §5 差异化护城河 | 增加一条定位语："PrismDocs = OKF 之上的人类理解与决策层"（格式兼容开放标准，价值在标准之外） | 中 |
| B5 | §11 风险表 | 新增：Google 沿 OKF+Code Wiki 向工具层延伸（等级：中；对策：兼容其格式、占住口语层与闭环）；CoWiki 等中文开源 LLM Wiki 产品分流中文早期用户（等级：低-中；对策：定位差异明确，且可互操作而非互斥） | 中 |
| B6 | §10 商业模式 | 补一句：OKF 兼容 = 无锁定承诺，作为免费层/信任卖点，降低采用阻力 | 低 |

## 4. 对主 PRD 的补充建议（合并进主 PRD v0.2）

| # | 位置 | 变更建议 | 优先级 |
|---|---|---|---|
| P1 | §2.1 领域对象 / 新增 REQ | **Base 层存储采用 OKF 兼容约定**：文档保留/识别 YAML frontmatter（`type`/`title`/`description`/`tags`/`timestamp`，内部定义受控 type 词表——吸取"词表未注册"批评，如 `Spec`/`Plan`/`Architecture`/`Decision`/`Card`/`Clip`）；`index.md`/`log.md` 按 OKF 保留名处理 | **高（架构决策，宜在开发前定）** |
| P2 | F1（REQ-1.NEW） | 导入时解析已有 frontmatter 并入库为结构化元数据；无 frontmatter 的文档不强制添加（不污染用户文件的原则不变），元数据存 sidecar，仅在导出时物化 | 高 |
| P3 | F7（REQ-7.NEW） | **「导出为 OKF Bundle」**：把勾选的文档/卡片/剪藏导出为合规 OKF bundle（卡片 type: Card 含注入行、剪藏 type: Clip 含来源 URL 于 `resource` 字段）；Context Pack 的分节结构参照 OKF 概念文件组织 | 高 |
| P4 | F1 变更记录 | 文档级变更历史可选物化为 OKF `log.md` 约定（供外部 agent 读取项目文档演化史） | 中 |
| P5 | §4 Agent 协议 | `.prismdocs/README.md` 中声明本目录输出遵循 OKF 约定；MCP 增加 `export_okf_bundle` 工具（P1 排期即可） | 中 |
| P6 | F4 / 新增 REQ（借鉴 CoWiki） | **Agent 贡献溯源**：每次 Base 变更记录来源（哪个 agent/会话/Feedback Bundle 触发），复核界面展示；呼应 OKF v0.2 的 provenance 方向，也是"错误传播"的防线 | 中 |
| P7 | P1 功能池（借鉴 CoWiki） | 新增候选：**入库冲突检测**——新文档/剪藏与已有决策卡片矛盾时提示（CoWiki"校验"阶段的个人版；防止错误/过期信息进入 Context Pack） | 低（P1 评估） |
| P8 | §8 开放问题 | 新增 Q6：type 词表的受控范围与扩展策略；Q7：是否允许用户开启"frontmatter 直写入源文件"模式（与"不污染文件"原则的权衡） | 中 |

## 5. 一句话总结

CoWiki 和 OKF 都没有做 PrismDocs 要做的事，但它们把 PrismDocs 脚下的地基（LLM Wiki 形态 + Markdown 知识包格式）变成了行业共识：**BRD 的立论被增强，PRD 的数据层应当以低成本换取 OKF 兼容，把"开放格式之上的人类理解与决策层"写进产品定位。**

---

### 来源

[CoWiki 官网](https://cowiki.ai/) · [CoWiki V2EX 发布帖](https://www.v2ex.com/t/1228349) · [CoWiki GitHub](https://github.com/eoneed/cowiki) · [Google Cloud OKF 官方博客](https://cloud.google.com/blog/products/data-analytics/how-the-open-knowledge-format-can-improve-data-sharing) · [OKF spec repo](https://github.com/GoogleCloudPlatform/knowledge-catalog) · [MarkTechPost：OKF 发布报道](https://www.marktechpost.com/2026/06/16/google-cloud-introduces-open-knowledge-format-okf-a-vendor-neutral-markdown-spec-for-giving-ai-agents-curated-context/) · [Marc Bara：A Standard, or Just a Folder?](https://medium.com/@marc.bara.iniesta/googles-new-format-for-agent-context-a-standard-or-just-a-folder-82fb21d92041) · [Google Code Wiki 介绍](https://ai-bot.cn/code-wiki/) · [SEJ：OKF 报道](https://www.searchenginejournal.com/google-cloud-announces-the-open-knowledge-format/579253/)
