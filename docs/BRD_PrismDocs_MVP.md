# BRD：面向 Vibe Coding 的双层文档管理应用（MVP）

| 项目 | 内容 |
|---|---|
| 产品代号 | PrismDocs（暂定） |
| 文档类型 | 商业需求文档（BRD）— MVP 版本 |
| 版本 | v0.2（合并《补充调研：CoWiki 与 OKF》变更 B1–B6，变更记录见文末） |
| 日期 | 2026-07-26 |
| 作者 | Shean（调研与起草协作：Claude） |

---

## 1. 一句话定义

**PrismDocs 是一个面向 vibe coder 的工程文档工作台：AI 用英文写详细技术文档（供 AI 持续操作、省 token），产品同步生成用户母语的口语化理解层（供人快速 review）；用户在文档上写评论，评论结构化回流给 AI 驱动下一轮迭代；文档以 LLM Wiki + 卡片笔记（Zettelkasten）方式组织，配套 Chrome 插件把网页资料剪藏进知识库。**

核心信念：vibe coding 的瓶颈已经从"让 AI 多产出"转移到"帮人理解、验证、记住 AI 的产出"。人的工作不是通读，而是决策。

---

## 2. 背景与市场机会

### 2.1 宏观背景

2025–2026 年，AI 辅助开发（vibe coding）从尝鲜走向日常，随之出现三个结构性变化：

第一，**瓶颈从"写"转移到"读和验证"**。AI 生成代码与文档的速度远超人类阅读速度，PR 审查时间激增（有分析称增长 91%），社区出现"3 分钟批准 300+ 行 diff"的"凭感觉合并"现象；日本一项 322 名工程师的调查显示 86.3% 在审查 AI 代码时遇到问题。

第二，**"外部记忆"成为刚需但缺乏治理**。Claude Code / Cursor 用户普遍把项目决策外置到 CLAUDE.md、AGENTS.md、rules 文件中，但这些文件持续膨胀、模型反而越不遵守；上下文压缩（/compact）后 AI"忘记"已定的约定，决策蒸发。Spec 驱动开发工具（AWS Kiro、GitHub Spec Kit 等）在 2025-2026 爆发，但公认痛点是生成的 spec 冗长重复、审阅乏味、与代码漂移。

第三，**"理解债"（Comprehension Debt）被正式提出**。Addy Osmani 引用的研究显示 AI 辅助组对自己代码的理解测验得分低 17%；日本调查中 49.5% 的工程师承认"无法充分解释 AI 生成的源代码"。认知科学的对策明确：只有**主动用自己的话重新组织**，理解才会留存——这正是卡片笔记法（Zettelkasten）的核心方法论。

### 2.2 被忽视的人群：非英语母语的 vibe coder

现有工具（DeepWiki、Kiro、Swimm、markupmarkdown 等）的隐含用户全部是"能流畅阅读英文技术文档的工程师"。但 vibe coding 的真实主力中有大量中日韩开发者、产品型创始人和半技术用户，他们的典型状态是："英文、太长、没有图"——读不动 AI 吐出的英文架构文档，只能再把文档丢回给 AI 用母语提问。他们想要的形态很具体：**母语口语层用于人类理解与决策 + 英文技术层保留给 AI 精确操作**。目前没有任何产品提供这种双层结构。

### 2.3 机会窗口

与本产品重叠度最高的两个产品（markupmarkdown：Markdown 版 Google Docs，AI agent 作为一等评审者；Plannotator：批注 AI 计划并结构化回流给 Claude Code/Codex）都是 2026 年刚起步的开源项目，均为单层英文文档、面向专业开发团队，无口语层、无卡片知识库、无剪藏。HackMD 正在向"AI 写文档、人类审阅"的治理层转型，是中期最大威胁。窗口期真实存在，但不会太长。

---

## 3. 竞品分析

### 3.1 五个相邻赛道的代表玩家

| 赛道 | 代表产品 | 定位与核心能力 | 定价参考 | 与本产品的差异 |
|---|---|---|---|---|
| AI 代码库文档 / LLM Wiki | DeepWiki（Cognition）、**Google Code Wiki**（2025-11）、Mintlify、Swimm、Komment | 从仓库自动生成架构 Wiki / 对外文档站；Code Wiki 随代码变更同步 + Gemini 问答，标志该赛道已巨头化 | DeepWiki 公开仓库免费；Mintlify Pro $250/月 | "生成后浏览"的只读模式，无评论回流迭代，无人类口语层；巨头入场进一步说明不应正面竞争"自动生成" |
| LLM Wiki 产品化（团队知识） | **CoWiki**（cowiki.ai） | 自称"LLM Wiki 团队版"：AI 编译管线（收录→编译→校验→人工审核）、Git 驱动可审计回退、多 agent 协同与冲突检测、agent 贡献审计；开源、中文团队、宣布对齐 OKF | 早期 waitlist，开源 | 团队通用知识沉淀 vs 本产品的工程开发闭环；无双层/口语层、无评论回流编码 agent、无"用自己的话"卡片；其"校验防错误传播"与"agent 溯源"机制值得借鉴 |
| Spec 驱动开发 | AWS Kiro、GitHub Spec Kit、OpenSpec、Tessl、CodeGuide | requirements→design→tasks 式 spec 工作流；spec 即源码范式 | Kiro Pro $20–200/月；Spec Kit 免费开源 | spec 锁在 IDE/CLI 内，冗长难审、迭代笨重；全部是英文技术文档 |
| 评论驱动 AI 迭代 | **markupmarkdown**、**Plannotator**、HackMD、CodeRabbit | .md 段落级批注 + AI agent 评审 / 批注回流编码 agent / AI PR 评审 | 多为开源或 $24–40/人/月 | **最接近的竞品**，但单层英文、面向开发团队，无双语层、无知识库结构、无剪藏 |
| 卡片笔记 / PKM | Obsidian（+ Copilot/Web Clipper）、Heptabase、Logseq、Tana | "用自己的话写卡片"方法论成熟，AI 插件生态活跃 | Obsidian 免费；Heptabase ~$12/月 | 通用笔记工具，不理解"工程文档→代码→AI 迭代"语境，与编码 agent 无闭环 |
| 网页剪藏 | Obsidian Web Clipper、Notion Clipper、Readwise Reader、Fabric | 剪藏 + AI 摘要已是红海 | 免费～$10/月 | 剪进通用笔记库；"剪藏→喂给编码 agent 的规格素材"无专门产品，代码块清理、token 成本不透明等工程向问题无人解决 |

### 3.2 市场空白判断

单项能力都有强者，但**"AI 文档审阅 + 评论驱动 AI 迭代 + 双层（AI 英文层 / 人类母语口语层）文档 + 卡片化知识组织 + 剪藏入口"的组合目前无人占位**。其中最空白、也最难被快速模仿的一格是**双层文档的双向同步**：口语层上的评论如何精确映射回英文技术层、再结构化回流给 AI——这是本产品的护城河，也是核心技术难点。

行业已接受"给 AI 和给人两套文本"的理念，且在 2026 年 6 月得到比 llms.txt 更强的背书：**Google 发布 Open Knowledge Format（OKF）**，把民间 LLM Wiki 模式（Markdown 目录 + YAML frontmatter + index.md/log.md 约定）标准化为厂商中立的知识包格式——与 llms.txt 一样，它服务的都是"AI 那一层"。OKF 明确不覆盖"人怎么看懂、怎么批注、怎么回流"，等于替本产品把地基（AI 层格式）标准化了，而把地基之上的人类理解与决策层留成空白。本产品的 Base 层与 OKF 天然同构，以极小成本即可 OKF 兼容，换取"知识库无锁定、可被任何 OKF 消费方读取"的信任卖点（CoWiki 已宣布对齐 OKF，不跟进将在对比中失分）。

---

## 4. 目标用户与痛点

### 4.1 用户画像

**P1 · 独立 vibe coder / 独立开发者（首要）**
使用 Claude Code / Cursor / Codex 构建产品的个人开发者，中文（或日文）为母语，英文技术阅读能力中等。项目里已有一堆 AI 生成的 .md（plan、spec、architecture、CLAUDE.md），但"写的时候没细看，回头找不到、读不懂、不敢信"。付费意愿参照：已为 AI 编码工具付 $20–200/月。

**P2 · 产品型创始人 / 半技术用户（次要）**
能描述需求、能看懂口语化说明，但读不动英文架构文档。需要一个"我能看懂并拍板"的层，来指挥 AI 干活。

**P3 · 小团队技术负责人（观察）**
需要治理团队的 AI 生成文档，MVP 阶段不主攻，但架构上不排斥。

### 4.2 痛点排序（调研结论 Top 8）

| # | 痛点 | 严重度 | 本产品的回应 |
|---|---|---|---|
| 1 | 审查疲劳："AI 写得比我读得快"，验证成本反超写码成本 | 极高 | 口语层先读摘要、再下钻；结构化批注代替通读 |
| 2 | AI 忘记决策：/compact 后约定蒸发，被迫自建外部记忆 | 极高 | 原子化决策卡片 = 可按需注入的外部记忆 |
| 3 | CLAUDE.md / rules 膨胀且失效 | 高 | 卡片化拆分 + 按需组装上下文，替代巨型单文件 |
| 4 | AI 生成的 spec 冗长、审阅乏味、AI 还不遵守 | 高 | 评论驱动局部迭代："不重写整份 spec，只在这段上批注" |
| 5 | 理解债：解释不了自己的代码库 | 高（上升中） | "用自己的话写卡片"机制，有认知科学依据 |
| 6 | 文档漂移：spec 与代码脱节 | 高 | 文档是持续被评论、被更新的活物；版本与状态标记 |
| 7 | 非母语者读不动英文长文档 | 东亚市场极高 | 双层双语结构（本产品独有） |
| 8 | 为 AI 收集网页资料：HTML 噪音、手工清理、token 不透明 | 中高 | Chrome 剪藏器：净化为 Markdown、标注 token 量、入卡片库 |

（另有两条作为设计约束：token 成本焦虑 → 英文技术层保持紧凑、上下文按需组装；警报疲劳 → 本产品的 AI 反馈必须少而准，由用户评论校准。）

---

## 5. 产品定位与价值主张

**定位**：AI 编码工具（Claude Code / Cursor / Codex）之外的、面向人的工程知识工作台。不做 IDE，不做代码生成，与编码 agent 是共生关系——agent 写代码和技术文档，PrismDocs 负责让人**看得懂、批得动、记得住**。

**价值主张（按用户视角）**：

- 给 P1 独立开发者：「10 分钟看懂 AI 昨晚写的东西，批注两句，让它接着干。」
- 给 P2 创始人：「不懂架构也能拍板——用你的母语，在你能看懂的那一层。」
- 对 token 焦虑：「AI 读的那层永远是紧凑英文，你读的那层不花它的 token。」

**差异化护城河**（按防御力排序）：双层文档双向同步机制 > 口语层 + 卡片化的"理解留存"设计 > 评论→AI 结构化回流协议 > 剪藏入口。

**对外定位语（v0.2 新增）**：PrismDocs = **OKF 之上的人类理解与决策层**——数据层兼容开放标准（无锁定），产品价值全部在标准之外（Lens、评论闭环、理解卡片）。

---

## 6. 产品概念：四个核心机制

### 6.1 双层文档（Two-Layer Docs）

每份文档存在两层，同源但不同语域：

- **Base 层（AI 层）**：英文、技术上完备详细、结构紧凑（省 token）。是唯一真相源（source of truth），AI 读写的对象。存储为纯 Markdown，与用户 repo 中的文件（如 `docs/`、CLAUDE.md）可双向同步。
- **Lens 层（人类层）**：由产品基于 Base 层自动生成的指定语言口语化版本，讲人话、有取舍、突出"改了什么、为什么、需要你决策什么"。Lens 层是**投影而非副本**——不可独立编辑，只能通过评论作用于 Base 层，从根上避免双层漂移。

每个 Lens 段落锚定到 Base 层的对应区块（block-level anchor）。Base 层更新后，仅重新生成受影响段落的 Lens，并标记"自上次 review 以来的变化"。

### 6.2 评论驱动迭代（Comment-to-Agent Loop）

用户在任一层（通常是 Lens 层）对段落写评论：质疑、修改要求、决策（approve / reject / 换方案）。评论经锚点映射到 Base 层区块，打包成结构化的反馈文件（如 `feedback.md` / MCP 输出），供 Claude Code 等 agent 读取执行。agent 完成后更新 Base 层，Lens 层重新投影，评论线程标记为 resolved，形成闭环：**AI 写 → 人在口语层批 → AI 改 → 人复核**。

设计约束：评论回流必须"少而准"——只传用户评论 + 被评论区块 + 必要上下文，不整份文档转发，控制 token。

### 6.3 LLM Wiki + 卡片笔记结构

知识库分两类节点：

- **Wiki 文档**：AI 生成的工程文档（architecture、spec、plan、decision log），双层结构，互相链接，构成项目的 LLM Wiki。
- **理解卡片（用户手写）**：用户**用自己的语言**写的原子笔记——对某个概念/决策/网页资料的个人重述。产品明确不鼓励复制粘贴：卡片创建界面以"你会怎么向朋友解释这个？"引导原创表述，可引用（链接）Wiki 文档和剪藏，但引用不等于内容。这是 Zettelkasten 方法论的产品化：**阅读靠 Lens 层，理解留存靠自己写卡片**。

卡片和文档通过双链关联；卡片可被标记为"注入上下文"，按需组装进给 AI 的 context（替代巨型 CLAUDE.md）。

### 6.4 Chrome 剪藏插件

抓取网页内容（文档、博客、Stack Overflow、GitHub 讨论）→ 净化为干净 Markdown（正确处理代码块）→ 显示 token 估算 → 存入知识库并可关联到项目/卡片。剪藏件可作为参考资料打包进给 AI 的上下文。

---

## 7. MVP 范围

### 7.1 MVP 目标

用最小闭环验证核心假设：**"双层文档 + 评论回流"能显著降低 vibe coder review AI 文档的成本，并且用户愿意为此改变工作流。**

### 7.2 功能清单

**P0（MVP 必须有）**

| 编号 | 功能 | 说明 | 验收要点 |
|---|---|---|---|
| F1 | 项目与文档导入 | 连接本地文件夹 / Git 仓库，导入 MD（HTML 转 MD）文档为 Base 层；文件变更监听同步 | 导入含 20 份 .md 的 repo ≤1 分钟可读 |
| F2 | Lens 层生成 | 对 Base 文档生成指定语言（首发：简体中文）口语化投影，段落级锚定；Base 变更后增量重投影并高亮变化 | 用户评价"看 Lens 能懂 80%"；重投影只触及受影响段落 |
| F3 | 段落级评论 | 在 Lens/Base 任意区块评论；评论类型：提问 / 修改要求 / 决策（approve/reject）；线程与状态（open/resolved） | 评论精确锚定，Base 更新后锚点不丢 |
| F4 | 评论回流 AI | 一键导出结构化反馈（feedback 文件 + MCP server），Claude Code / Cursor 可直接消费；agent 更新 Base 后自动重投影并把关联评论标记待复核 | 在真实 Claude Code 项目中跑通"批注→AI 修改→复核"全闭环 |
| F5 | 理解卡片 | 手写原子卡片（强引导原创表述），双链到文档/剪藏；卡片列表与基础搜索 | 卡片创建流畅；引用不复制内容 |
| F6 | Chrome 剪藏插件 | 网页→净化 Markdown（代码块正确）、token 估算、存入知识库、可关联项目 | Stack Overflow / GitHub / 博客三类页面剪藏可用 |
| F7 | 上下文组装 | 勾选文档/卡片/剪藏，生成紧凑的 AI 上下文包（英文 Base 层内容），显示总 token 数 | 生成的包可直接被 Claude Code 引用 |

**P1（MVP 后第一批）**：日文/英文 Lens、文档间图谱视图、决策日志自动抽取、多人评论、GitHub PR 集成、卡片间隔回顾、入库冲突检测（v0.2 新增：新文档/剪藏与已有决策卡片矛盾时提示，借鉴 CoWiki"校验"阶段）、MCP `export_okf_bundle` 工具（v0.2 新增）。

### 7.3 明确不做（Out of Scope）

- 不做 IDE / 代码编辑器，不做代码生成，不内置编码 agent（只对接）。
- 不做通用笔记应用（不与 Obsidian 抢"人生笔记"，只做工程语境）。
- 不做对外发布的文档站（不与 Mintlify/GitBook 竞争）。
- MVP 不做团队协作与权限体系（P2/P3 人群后置）。
- Lens 层不可直接编辑（防双层漂移的产品原则，非技术妥协）。

### 7.4 形态与技术前提（供 PRD 细化）

桌面应用（本地优先，文档不强制上云）+ Chrome 扩展；LLM 能力走用户自备 API key 或订阅内置额度；与 agent 的接口优先 MCP + 文件协议（feedback.md），兼容 Claude Code、Cursor。

---

## 8. 核心用户旅程（MVP 主流程）

1. Shean 用 Claude Code 开发项目，agent 按约定把 plan/spec/architecture 写进 `docs/`（英文、详细）。
2. PrismDocs 监听到新文档，生成中文口语 Lens，推送摘要："本次新增 2 份文档，架构有 1 个需要你决策的取舍。"
3. Shean 花 10 分钟读 Lens 层，在"为什么选 SQLite 而不是 Postgres"一段评论："并发写入会不会有问题？如果用户超过 1 万怎么办"，并对另外两段点 approve。
4. 评论结构化回流 Claude Code；agent 补充分析、修改 Base 文档；Lens 重投影，该段高亮"已根据你的评论更新"。
5. Shean 复核通过，顺手写一张卡片：「SQLite 在我们场景够用，因为写入都走单队列——超过 1 万用户再迁移，迁移点在 storage 层已隔离。」这张卡片被标记注入上下文，此后 AI 不再"忘记"这个决策。
6. 期间 Shean 剪藏了一篇 SQLite 并发写入的博客，关联到该卡片，作为下次给 AI 的参考资料。

---

## 9. 成功指标

**北极星指标：每周完成的"评论→AI 修改→复核通过"闭环数（Closed Loops / week）。**

| 类别 | 指标 | MVP 目标（上线 3 个月） |
|---|---|---|
| 验证核心假设 | 用户自评"review AI 文档的时间下降" | ≥50% 用户报告下降一半以上 |
| 激活 | 新用户 7 日内完成首个闭环 | ≥40% |
| 留存 | 周活跃留存（W4） | ≥30% |
| 理解留存 | 人均理解卡片数/周 | ≥3（观察指标，不强推） |
| 剪藏 | 剪藏→被引用进上下文的比例 | ≥25% |
| 商业信号 | 愿付费访谈确认（$10–20/月档） | ≥20 个明确付费意向 |

**反指标（护栏）**：Lens 层与 Base 层的投影投诉率（"翻译不准/漏了关键信息"）<5%；评论回流后 agent 误改率可控。

---

## 10. 商业模式（初步）

免费层：单项目、有限 Lens 生成额度（自备 API key 可放宽）。
Pro：$15/月左右（锚定 Heptabase $12、CodeGuide $24、Kiro $20），不限项目、内置模型额度、优先支持新语言 Lens。
后续：团队版（P3 人群）按席位收费。
成本注意：Lens 投影是主要模型成本，增量重投影（只重生成受影响段落）既是体验设计也是成本设计。
信任卖点：OKF 兼容 = "你的知识库随时可以完整带走、可被任何 OKF 消费方读取"，作为免费层与获客话术，降低采用阻力。

---

## 11. 风险与对策

| 风险 | 等级 | 对策 |
|---|---|---|
| IDE 厂商（Kiro/Cursor/Claude Code）补齐 spec 审阅 UI | 高 | 定位错开：IDE 外、面向人的知识层 + 母语口语层；东亚市场先行 |
| 双层同步的技术难度（锚点漂移、投影失真） | 高 | Lens 不可编辑的单向投影原则；block-anchor + 增量重投影作为第一优先级攻坚 |
| markupmarkdown / Plannotator / HackMD 加速 | 中 | 它们面向英文开发团队；以双语双层 + 卡片理解层建立差异；保持对其开源协议的兼容（能读它们的格式） |
| 用户不愿改变工作流（惯性用 IDE 内对话） | 中 | 接入成本压到最低：监听文件夹即用、不要求改变 agent 使用习惯；剪藏器作为低门槛获客入口 |
| Lens 翻译失真导致错误决策 | 中 | Lens 段落一键对照 Base 原文；关键决策段强制展示原文摘录 |
| 模型成本失控 | 低 | 自备 key 选项 + 增量投影 + 紧凑 Base 层 |
| Google 沿 OKF + Code Wiki 向工具层延伸，从"格式 + 自动 Wiki"两头逼近 | 中 | 兼容其格式而非对抗；占住 OKF 明确不覆盖的口语层与评论闭环；东亚市场先行 |
| CoWiki 等中文开源 LLM Wiki 产品分流中文早期用户 | 低-中 | 定位差异明确（团队知识沉淀 vs 工程开发闭环）；同为 OKF 兼容，可互操作而非互斥 |

---

## 12. 里程碑建议

| 阶段 | 周期 | 内容 |
|---|---|---|
| M0 概念验证 | 2–3 周 | 手工流程验证：用现有工具拼出"英文 doc→中文 Lens→评论→回流 Claude Code"，找 5 个目标用户走一遍，验证 Lens 可读性与闭环意愿 |
| M1 MVP 开发 | 6–8 周 | F1–F4（双层 + 评论闭环）为先，F5–F7 随后 |
| M2 内测 | 4 周 | 20–30 名种子用户（中文 vibe coder 社区招募），盯北极星指标 |
| M3 公测 | — | Product Hunt / 即刻 / V2EX / 掘金发布；剪藏器单独上架 Chrome 商店引流 |

---

## 13. 变更记录

| 版本 | 日期 | 变更 |
|---|---|---|
| v0.1 | 2026-07-26 | 初稿 |
| v0.2 | 2026-07-26 | 合并《补充调研：CoWiki 与 OKF》B1–B6：竞品表新增 Google Code Wiki 与 CoWiki（§3.1）；市场空白论据由 llms.txt 升级为 OKF（§3.2）；新增对外定位语"OKF 之上的人类理解与决策层"（§5）；商业模式补 OKF 无锁定卖点（§10）；风险表新增 Google 两头逼近、CoWiki 分流两条（§11） |
| v0.2.1 | 2026-07-26 | 产品定名 **PrismDocs**（棱镜隐喻：一份 Base 层折射出 Lens、卡片等多个谱层），替换占位名 VibeDocs；上线前需完成域名/商标终检 |

---

## 14. 附录：主要调研来源

**竞品**：[CoWiki](https://cowiki.ai/)（[V2EX 发布帖](https://www.v2ex.com/t/1228349)） · [Google OKF](https://cloud.google.com/blog/products/data-analytics/how-the-open-knowledge-format-can-improve-data-sharing)（[spec repo](https://github.com/GoogleCloudPlatform/knowledge-catalog)） · [Google Code Wiki](https://ai-bot.cn/code-wiki/) · [DeepWiki](https://codersera.com/blog/how-to-use-deepwiki/) · [Mintlify 定价](https://www.featurebase.app/blog/mintlify-pricing) · [Kiro 定价](https://kiro.dev/pricing/) · [SDD 工具对比](https://github.com/cameronsjo/spec-compare) · [Tessl](https://tessl.io/blog/tessl-launches-spec-driven-framework-and-registry/) · [CodeGuide](https://www.codeguide.dev/) · [markupmarkdown](https://metavert.io/markupmarkdown) · [Plannotator](https://plannotator.ai/)（[GitHub](https://github.com/backnotprop/plannotator)） · [HackMD agent 治理方向](https://homepage.hackmd.io/blog/2026/04/22/AI-writes-your-docs-hackmd) · [AI 代码评审工具综述](https://dev.to/heraldofsolace/the-best-ai-code-review-tools-of-2026-2mb3) · [Obsidian Web Clipper](https://www.geeky-gadgets.com/obsidian-web-clipper-browser-extension/) · [llms.txt](https://buildwithfern.com/post/optimizing-api-docs-ai-agents-llms-txt-guide)

**痛点**：[HN：审查疲劳讨论](https://news.ycombinator.com/item?id=48272984) · [Blind：AI coding is exhausting](https://www.teamblind.com/post/ai-coding-is-surprisingly-exhausting-27k1c4e6) · [Stop Vibe Merging](https://shmulc.substack.com/p/stop-vibe-merging) · [日本 322 名工程师调查](https://techtarget.itmedia.co.jp/tt/news/2605/05/news02.html) · [Claude /compact 失忆](https://golev.com/post/claude-saves-tokens-forgets-everything/) · [Context rot 与 CLAUDE.md 膨胀](https://www.mindstudio.ai/blog/context-rot-claude-code-skills-bloated-files) · [Thoughtworks：SDD 工具实测](https://www.martinfowler.com/articles/exploring-gen-ai/sdd-3-tools.html) · [Kiro 使用体验](https://dev.to/aws-builders/what-i-learned-using-specification-driven-development-with-kiro-pdj) · [Addy Osmani：Comprehension Debt](https://addyosmani.com/blog/comprehension-debt/) · [日本开发者：英文文档之痛](https://note.com/sakamototakuma/n/n4461dc34fa68) · [Web2MD：剪藏喂 AI 之痛](https://web2md.org/blog/obsidian-web-clipper-companion-for-ai-workflow) · [AI 文档没人读](https://dev.to/ujjavala/ai-confluence-docs-and-readmes-why-ai-written-docs-end-up-unread-31i8)
