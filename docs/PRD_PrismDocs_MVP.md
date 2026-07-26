# PRD：PrismDocs MVP 主产品需求文档

| 项目 | 内容 |
|---|---|
| 产品代号 | PrismDocs（暂定） |
| 文档类型 | 主 PRD（Master PRD）— MVP 版本 |
| 版本 | v0.2（合并《补充调研：CoWiki 与 OKF》变更 P1–P8，变更记录见 §9；各子 PRD 的 REQ-x.NEW 回填评审留待 v0.3） |
| 日期 | 2026-07-26 |
| 上游文档 | 《BRD：面向 Vibe Coding 的双层文档管理应用（MVP）》v0.2 |
| 作者 | Shean（起草协作：Claude） |

---

## 1. 概述

### 1.1 产品一句话

PrismDocs 是 vibe coder 的工程文档工作台：AI 维护紧凑的英文技术文档（Base 层），产品自动投影为用户母语的口语化版本（Lens 层）；用户在文档上写评论驱动 AI 迭代，用自己的语言写卡片沉淀理解，用 Chrome 插件把网页资料剪进知识库。

### 1.2 本 PRD 的范围

覆盖 BRD 中 P0 功能 F1–F7 的完整产品需求，包括交互、状态、边界情况、验收标准，以及支撑它们的信息架构、数据模型和 agent 集成协议。P1 功能（多语言 Lens、图谱视图、多人协作等）不在本文范围，仅在架构上预留。

### 1.3 产品形态

- **桌面应用**（macOS 优先，本地优先架构：文档、评论、卡片全部落在本地，不强制上云）
- **Chrome 扩展**（剪藏器，MV3）
- **本地 MCP Server + 文件协议**（与 Claude Code / Cursor 等编码 agent 的接口）
- LLM 能力：用户自备 API key（MVP 首发，支持 Anthropic / OpenAI 兼容端点），订阅制内置额度后置。

### 1.4 目标用户（摘自 BRD）

P1 独立 vibe coder（中文母语，用 Claude Code / Cursor 开发，首要）；P2 产品型创始人 / 半技术用户（次要）。MVP 的一切取舍以 P1 为准。

### 1.5 MVP 成功标准（摘自 BRD）

北极星：每周完成的「评论 → AI 修改 → 复核通过」闭环数。激活目标：新用户 7 日内完成首个闭环 ≥40%。

---

## 2. 核心概念与信息架构

### 2.1 领域对象

| 对象 | 说明 |
|---|---|
| **Workspace** | 顶层容器，对应一个用户的全部知识库。MVP 单 Workspace |
| **Project** | 对应一个代码仓库/工作目录。含文档、卡片、剪藏的关联 |
| **Document（Base 层）** | 唯一真相源。英文 Markdown 文件，与磁盘上的真实文件双向同步（`docs/**/*.md`、`CLAUDE.md`、`AGENTS.md` 等） |
| **Lens** | Base 文档的口语化投影（MVP：简体中文）。派生数据，不可编辑，随 Base 增量重建 |
| **Block** | 文档的最小锚定单元（标题、段落、列表、代码块、表格）。评论与 Lens 段落都锚定到 Block |
| **Comment** | 挂在 Block 上的评论线程。类型：提问 / 修改要求 / 决策（approve · reject） |
| **Card（理解卡片）** | 用户手写的原子笔记，用自己的语言。可双链到 Document / Clip / Card |
| **Clip（剪藏）** | Chrome 插件抓取的网页内容，净化后的 Markdown + 元数据 |
| **Context Pack** | 用户勾选文档/卡片/剪藏后组装的、给 AI 的紧凑上下文包 |
| **Feedback Bundle** | 一次「回流」导出的结构化评论包，供 agent 消费 |

### 2.2 对象关系

```
Workspace ─ 1:N ─ Project ─ 1:N ─ Document ─ 1:N ─ Block ─ 1:N ─ Comment
                     │                │
                     │                └─ 1:1 ─ Lens（按 Block 分段投影）
                     ├─ 1:N ─ Card ──双链──> Document / Clip / Card
                     ├─ 1:N ─ Clip
                     └─ 1:N ─ Context Pack / Feedback Bundle
```

### 2.3 主导航（桌面端）

左侧栏三个一级入口 + 项目切换器：

1. **文档（Docs）**：项目文档树（镜像磁盘目录结构），徽标显示未读变更数、未解决评论数
2. **卡片（Cards）**：理解卡片列表/搜索
3. **剪藏（Clips)**：剪藏收件箱
4. 顶部全局：**待你处理（Inbox）**——聚合"新变更待 review、AI 已修改待复核、评论被回应"三类事项。**Inbox 是日常主入口**，产品的节奏是"打开 → 清 Inbox → 关掉"。

### 2.4 Block 锚定机制（关键技术约定）

- 文档解析为 Block 树（基于 Markdown AST，标题分节，段落/代码块/表格为叶子）。
- 每个 Block 有稳定 ID：内容哈希 + 位置启发式。Base 更新时用 diff 匹配算法（内容相似度 + 相对位置）迁移 Block ID，尽量保持评论与 Lens 锚点存活。
- 锚点迁移置信度低于阈值时，评论降级为「文档级评论」并标记"原位置已变化，请确认"，**绝不静默丢失**。
- Block ID 不写入用户的 Markdown 源文件（保持文件干净），存在 PrismDocs 本地库中（sidecar 存储）。

### 2.5 OKF 兼容约定（v0.2 新增，架构级决策）

Base 层的存储与导出遵循 [Google Open Knowledge Format v0.1](https://github.com/GoogleCloudPlatform/knowledge-catalog) 的核心约定，使 PrismDocs 知识库可被任何 OKF 消费方直接读取（无锁定承诺，见 BRD §5 定位语）：

- **文件即概念**：一个 Markdown 文件一个概念，文件路径即身份——与现有 Document/Card/Clip 模型天然同构。
- **YAML frontmatter**：识别并管理 `type`（必填）、`title`、`description`、`resource`、`tags`、`timestamp` 六个标准字段；PrismDocs 内部维护**受控 type 词表**（吸取社区对 OKF"词表未注册"的批评）：`Spec` / `Plan` / `Architecture` / `Decision` / `Runbook` / `Card` / `Clip` / `ContextPack` / `Doc`（缺省），可扩展但需在设置中登记。
- **保留文件名**：`index.md`（目录导航）、`log.md`（变更编年史）按 OKF 语义处理。
- **不污染原则不变**：用户源文件没有 frontmatter 时**不强制写入**；PrismDocs 的元数据存 sidecar，仅在「导出 OKF Bundle」（REQ-7.6）时物化为合规 frontmatter。用户文件已有 frontmatter 时解析入库并保持往返一致（round-trip 不破坏）。
- 跨文档引用使用标准 Markdown 链接（已是现状），导出后目录即可被 OKF 工具解析为图。

---

## 3. 功能需求

以下每个功能包含：用户故事、需求细则（REQ 编号）、边界情况、验收标准（AC）。优先级 P0 = MVP 发布必须；P0.5 = MVP 发布前尽力，可降级。

---

### F1 · 项目与文档导入

**用户故事**：作为 vibe coder，我把 PrismDocs 指向我的项目文件夹后，它自动识别并持续同步 AI 写的文档，我不需要改变 Claude Code 的任何使用习惯。

**需求细则**

- REQ-1.1 新建项目 = 选择本地文件夹（或已 clone 的 Git 仓库根目录）。
- REQ-1.2 首次导入扫描规则（可配置的 glob）：默认包含 `docs/**/*.md`、`*.md`（根目录）、`CLAUDE.md`、`AGENTS.md`、`.claude/**/*.md`；默认排除 `node_modules`、`.git`、构建产物目录。
- REQ-1.3 HTML 文档（AI 生成的 .html 报告）导入时转换为 Markdown 存为 Base 层，保留原文件引用；转换损失明显（复杂脚本页面）时提示"以附件方式保存，不参与双层结构"。
- REQ-1.4 文件监听：基于 FS watcher 实时感知新增/修改/删除/重命名，防抖 2s 合并高频写入（agent 常连续写盘）。
- REQ-1.5 外部修改（用户在 IDE 里改文档、agent 写文档、git pull）一律以磁盘为准——**磁盘文件是 Base 层的权威副本**，PrismDocs 不锁文件。
- REQ-1.6 在 PrismDocs 内编辑 Base 层（允许，面向能读英文的用户）即写回磁盘文件。
- REQ-1.7 Git 感知（P0.5）：识别当前 branch 与文件的 git 状态，变更记录尽量关联 commit hash；MVP 不做多分支文档视图。
- REQ-1.8 frontmatter 解析（v0.2 新增，P0）：导入时解析文件已有的 YAML frontmatter，标准六字段（§2.5）入库为结构化元数据并参与筛选/搜索；无 frontmatter 的文件不强制添加，元数据存 sidecar。type 值不在受控词表内时归入"未登记"并提示。
- REQ-1.9 `log.md` 物化（v0.2 新增，P0.5）：项目级开关，将文档变更历史按 OKF `log.md` 约定物化到磁盘，供外部 agent 读取文档演化史；默认关闭。

**边界情况**

- 文件在 PrismDocs 打开期间被外部删除 → 文档标记 archived，评论与卡片引用保留（显示"源文件已删除"）。
- 重命名/移动 → 通过内容哈希识别为同一文档，锚点随迁。
- 单文件 >1MB 或非 UTF-8 → 跳过并在项目设置中列出。
- 同一文件夹被重复添加为项目 → 阻止并跳转到已有项目。

**验收标准**

- AC-1a：导入含 20 份 .md 的仓库，全部文档 60 秒内可浏览。
- AC-1b：Claude Code 在会话中连续修改 5 个文档，PrismDocs 在 10 秒内全部呈现最新内容且无重复变更记录。
- AC-1c：外部重命名文件后，其上的评论 100% 保留。

---

### F2 · Lens 层生成（口语化投影）

**用户故事**：作为读英文技术文档吃力的开发者，我打开任何一份 AI 写的文档，默认看到的是中文口语版，它告诉我"这文档讲了什么、AI 做了什么取舍、哪里需要我拍板"，我基本不用读英文原文。

**需求细则**

- REQ-2.1 投影粒度：按 Block 分段生成，Lens 段落与 Base Block 一一锚定；阅读视图支持三种模式切换：**仅 Lens（默认）/ 对照（左右分栏）/ 仅 Base**。
- REQ-2.2 Lens 不是翻译，是「口语化重述 + 取舍标注」。生成 prompt 的产品要求：讲人话（目标：受过教育但非本领域的读者能懂）；压缩重复内容；**显式标出**：⚖️ 取舍决策、⚠️ 风险/待确认、❓ 需要用户决策的点。
- REQ-2.3 文档头部自动生成「速读区」：3-5 句话摘要 + 需要决策的事项清单（链接到对应段落）。
- REQ-2.4 增量重投影：Base 变更后，仅重新生成受影响 Block 的 Lens（含受上下文影响的相邻块，由 diff 决定），未变更部分复用缓存。
- REQ-2.5 变更高亮：自上次用户「已读」标记以来发生变化的 Lens 段落显示变更条（新增/修改/删除），一键"标记本文档已读"。
- REQ-2.6 忠实度保障：每个 Lens 段落可一键展开对应 Base 原文；含"需要决策"标记的段落**强制**附带 Base 原文摘录（BRD 风险对策）；Lens 段落提供「报告失真」按钮（数据回流用于 prompt 迭代，也是北极星护栏指标数据源）。
- REQ-2.7 Lens 不可编辑（产品原则）。用户对 Lens 的一切不满通过评论表达。
- REQ-2.8 生成状态：逐段流式渲染；失败段落显示重试按钮，不阻塞其他段落。
- REQ-2.9 成本控制：投影调用显示预估 token；项目级设置"自动投影"或"手动触发投影"（默认：≤5k token 的文档自动，超过则提示）。

**边界情况**

- Base 本身是中文或混合语言 → 语言检测，中文内容跳过投影仅做速读区。
- 代码块不投影，原样呈现，但其前后的解释文字要覆盖代码块的意图。
- 表格投影保持表格结构，仅口语化单元格措辞。
- API key 未配置/额度耗尽 → Lens 区显示引导卡片，Base 层阅读不受影响。
- Base 在投影进行中再次变更 → 取消进行中任务，以最新版本重新调度。

**验收标准**

- AC-2a：内测用户对"只看 Lens 能理解文档 80% 内容"的认同率 ≥70%。
- AC-2b：修改 Base 中 1 个段落，重投影只调用受影响段落（可通过日志验证），10 秒内呈现。
- AC-2c：所有"需要决策"段落均带 Base 原文摘录，无一例外。
- AC-2d：失真报告率 <5%（内测期）。

---

### F3 · 段落级评论

**用户故事**：作为 reviewer，我在中文 Lens 上圈出一段写下质疑或修改要求，就像在 Google Docs 上评论一样自然；这条评论会精确地落在英文 Base 的对应位置上。

**需求细则**

- REQ-3.1 评论入口：在 Lens 或 Base 的任意 Block 上（悬停出现评论按钮 / 选中文字后浮条）；选中文字作为 quote 存入评论。
- REQ-3.2 评论类型（创建时选择，默认"修改要求"）：
  - 💬 **提问**：期待 AI 回答而非改文档
  - ✏️ **修改要求**：期待 AI 修改 Base
  - ✅ **Approve** / ❌ **Reject**：对该 Block（或整份文档）的决策标记，可附言
- REQ-3.3 评论线程：支持回复；状态机 `open → sent（已回流）→ needs-review（AI 已响应待复核）→ resolved / reopened`。
- REQ-3.4 评论语言自由（中文为主）；回流时由产品附带英文摘要（见 F4）。
- REQ-3.5 文档级评论区：不针对具体段落的整体意见。
- REQ-3.6 评论侧栏：按文档聚合，可筛选（状态/类型），Inbox 聚合跨文档的 needs-review 项。
- REQ-3.7 评论数据存 PrismDocs 本地库（不污染用户 Markdown 文件）。

**边界情况**

- 评论后 Base 该段被 AI 大改 → 锚点迁移；置信度低时评论标"原文已变化"并展示评论时的 quote 快照。
- 评论的 Block 被删除 → 评论降级为文档级，保留 quote 快照。
- 同一 Block 多条评论 → 全部保留，回流时打包。

**验收标准**

- AC-3a：在 Lens 段落上创建的评论，在对照视图中正确显示在 Base 对应 Block 旁。
- AC-3b：AI 重写文档 50% 内容后，≥90% 的评论锚点正确迁移或显式降级，0 静默丢失。
- AC-3c：评论创建 ≤2 次点击 + 输入。

---

### F4 · 评论回流 AI（Comment-to-Agent Loop）★ 核心闭环

**用户故事**：作为 vibe coder，我批注完后点一下「发给 AI」，回到 Claude Code 说一句"处理 PrismDocs 反馈"（或由 hook 自动触发），agent 就能拿到结构化的反馈精确干活；它改完后，PrismDocs 自动提醒我复核。

**需求细则**

- REQ-4.1 **Feedback Bundle 生成**：用户在文档或项目级点「回流」，选择包含的评论（默认全部 open 状态），产品生成结构化反馈：
  - 每条评论包含：目标文件路径 + Block 定位（标题路径 + 原文摘录）+ 评论类型 + 用户原文（中文）+ **产品生成的英文意图摘要** + 线程上下文
  - 明确的执行指令头：要求 agent 逐条处理、修改对应文件、不得改动未涉及部分、完成后逐条回执
- REQ-4.2 双通道交付：
  - **文件协议（P0）**：写入项目根 `.prismdocs/feedback/<timestamp>.md`（人类可读的 Markdown，附 YAML 元数据），并复制"喂给 agent 的一句话指令"到剪贴板
  - **MCP Server（P0）**：本地 MCP 提供 `list_feedback` / `get_feedback` / `respond_to_comment`（回执：逐条标记已处理+说明）/ `get_document_comments` 工具；提供 Claude Code hook/skill 安装引导（一键生成配置片段）
- REQ-4.3 回流范围控制（token 约束，BRD 设计约束）：Bundle 只含评论 + 被评块原文 + 必要的父级标题路径；整份文档默认不进 Bundle；提供"附带全文"手动开关。
- REQ-4.4 闭环回收：agent 通过 MCP 回执，或产品检测到 Base 变更命中被评论 Block → 评论状态转 needs-review，Inbox 通知；用户在变更高亮 + 评论上下文中复核 → resolve（计入北极星闭环数）或 reopen（可追评）。
- REQ-4.5 「提问」类评论的回答：agent 经 MCP respond 的文字答复显示在评论线程内。
- REQ-4.6 Bundle 历史：每次回流留档可查（哪些评论、何时、agent 是否回执）。
- REQ-4.7 Agent 贡献溯源（v0.2 新增，P0）：每次 Base 变更记录触发来源（关联的 Feedback Bundle / MCP 回执方 / 外部未知变更），needs-review 复核界面展示"这次修改由谁、因哪条评论触发"；为 OKF v0.2 的 provenance 方向预留数据结构。借鉴 CoWiki 的问题意识：防止错误来源不明的内容进入共享上下文后被 agent 放大。

**边界情况**

- agent 改了文件但没回执（用户没装 MCP/hook）→ 靠 REQ-4.4 的文件变更检测兜底闭环。
- agent 只改了部分评论涉及内容 → 命中的转 needs-review，未命中的保持 sent，48 小时后提示"AI 似乎没有处理这几条"。
- 用户在 Bundle 未处理时又发新 Bundle → 允许，新旧独立；同一评论不重复进入两个 Bundle。
- Reject/Approve 类评论回流时明确告知 agent 决策结果及其含义（reject = 换方案重来，需先在文档中提出新方案）。

**验收标准**

- AC-4a：在真实 Claude Code 项目中完整跑通：Lens 上写 3 条评论 → 回流 → Claude Code 修改 Base → Lens 重投影 + needs-review 提醒 → 复核 resolve，全程无手工拷贝粘贴（除触发 agent 的一句话）。
- AC-4b：Feedback Bundle 的 token 数 ≤ 被评论文档全文的 30%（典型场景）。
- AC-4c：未装 MCP 的用户仅凭文件协议也能完成闭环（降级路径可用）。

---

### F5 · 理解卡片

**用户故事**：作为想真正掌握自己项目的开发者，我复核完一个决策后，顺手用自己的话写一张卡片记下"为什么这么定"；这些卡片成为项目里最可信的记忆，也能喂给 AI 防止它忘记。

**需求细则**

- REQ-5.1 卡片 = 标题 + 正文（Markdown）+ 双链 + 标签。刻意保持简单，无文件夹层级（Zettelkasten 原则：靠链接不靠分类）。
- REQ-5.2 原创引导（产品化的"用自己的话"）：
  - 创建界面 placeholder：「你会怎么向朋友解释这件事？」
  - 从文档/剪藏选中文字"存为卡片"时，选中内容进入**引用区（折叠展示，标注来源链接）**，正文必须另写；正文为空或与引用高度重复时发布前柔性提醒（不强制阻止）
  - 卡片正文不提供 AI 代写按钮（刻意缺失；AI 仅可在发布后提供"复述质检"：指出你可能理解偏了的点——P0.5）
- REQ-5.3 双链：`[[` 唤起联想选择文档/卡片/剪藏；反链面板显示"谁引用了这张卡"。
- REQ-5.4 场景入口：评论 resolve 时提示"要为这个决策写张卡片吗？"（预填上下文链接）；文档/剪藏阅读中选中文字 → 存为卡片。
- REQ-5.5 「注入上下文」开关：卡片可标记为 context-worthy，被 F7 上下文组装默认拾取；此类卡片建议附英文一句话版本（AI 可代译此格式化字段——与"正文不代写"不冲突，正文是给人的，注入行是给 AI 的）。
- REQ-5.6 列表与全文搜索、按标签/项目/链接对象筛选。

**边界情况**

- 删除被卡片引用的文档/剪藏 → 卡片引用变为快照（保留 quote），标注源已删。
- 卡片互链成环 → 允许（Zettelkasten 常态）。

**验收标准**

- AC-5a：从"评论 resolve"入口 30 秒内可完成一张卡片。
- AC-5b：引用内容与正文在数据层分离，导出时可区分。
- AC-5c：内测期人均卡片 ≥3 张/周（观察指标）。

---

### F6 · Chrome 剪藏插件

**用户故事**：作为开发者，我看到一篇讲 SQLite 并发的好文章，点一下插件就把它存成干净的 Markdown 进了知识库，代码块完好，还看得到它值多少 token；下次让 AI 参考它时不用再手工清理。

**需求细则**

- REQ-6.1 抓取模式：整页正文提取（Readability 类算法）/ 选区剪藏 / 手动框选元素（P0.5）。
- REQ-6.2 净化转换：HTML → Markdown，重点保障：代码块（语言标注、去除高亮 span 噪音）、表格、列表、图片（存 URL，可选下载）；重点适配站点：Stack Overflow、GitHub（README/Issue/Discussion）、常见技术博客、MDN 类文档站（调研明确的痛点站点）。
- REQ-6.3 剪藏时弹出面板：标题（可改）、目标项目、标签、**token 估算**、备注（一句话"为什么剪它"——轻推原创表述，可跳过）。
- REQ-6.4 元数据：URL、站点名、抓取时间、原文语言。
- REQ-6.5 与桌面端同步：本地回环通信（native messaging 或本地端口）；桌面端未运行时插件本地暂存队列，启动后补传。
- REQ-6.6 剪藏收件箱：未归类剪藏统一入 Inbox 式列表，支持批量归项目/归档/删除。
- REQ-6.7 剪藏可被评论？——**不可**（MVP）。剪藏是外部素材，理解写进卡片，保持"评论 = 驱动 AI 改文档"的语义纯度。

**边界情况**

- 付费墙/登录墙页面 → 抓到什么存什么，标注"可能不完整"。
- SPA 动态内容 → 以当前 DOM 为准；抓取失败给出"选区剪藏"引导。
- 超长页面（>50k token）→ 提示并提供"仅剪选区"。
- 重复剪藏同一 URL → 提示已存在，可存为新版本。

**验收标准**

- AC-6a：Stack Overflow 答案页剪藏后，代码块在桌面端 100% 可正确复制运行（无 span 噪音）。
- AC-6b：GitHub README、技术博客、MDN 三类页面剪藏成功率 ≥95%（内测样本）。
- AC-6c：token 估算与实际 tokenizer 计数误差 ≤10%。

---

### F7 · 上下文组装（Context Pack）

**用户故事**：作为要开新一轮 AI 会话的开发者，我勾选 3 份文档、5 张卡片和 2 个剪藏，生成一个紧凑的英文上下文包，看着总 token 数心里有数，然后一句话让 Claude Code 引用它。

**需求细则**

- REQ-7.1 组装器：树形勾选文档（Base 层）/ 卡片（注入行优先，正文可选）/ 剪藏（可选"AI 压缩版"——生成英文要点摘要以省 token，P0.5）；实时显示总 token 及各项占比。
- REQ-7.2 输出：写入 `.prismdocs/context/<name>.md`，结构化（来源标注、分节），纯英文倾向（中文卡片正文附机器英译或原文保留，用户可选）。
- REQ-7.3 常用包可保存为模板（如"架构决策包"），文档更新后重新生成时提示内容已变化。
- REQ-7.4 context-worthy 卡片默认预勾选。
- REQ-7.5 与 MCP 打通：agent 可通过 `get_context_pack` 直接拉取（免文件引用）。
- REQ-7.6 导出 OKF Bundle（v0.2 新增，P0.5）：将勾选的文档/卡片/剪藏导出为合规 OKF bundle 目录——sidecar 元数据物化为 frontmatter（Card → `type: Card`（含注入行）、Clip → `type: Clip`（来源 URL 写入 `resource`）），自动生成 `index.md`；导出物可被任何 OKF 消费方（含 Google Knowledge Catalog）读取。

**边界情况**

- 勾选内容超过用户设定的 token 预算 → 高亮超支项，建议压缩或剔除。
- 引用的文档已删除 → 生成时跳过并警示。

**验收标准**

- AC-7a：生成的 Pack 被 Claude Code 通过文件引用或 MCP 消费，实际可用。
- AC-7b：token 显示误差 ≤10%。
- AC-7c：从打开组装器到生成完成 ≤60 秒（不含可选 AI 压缩）。

---

## 4. Agent 集成协议（跨功能规格）

### 4.1 `.prismdocs/` 目录约定（项目根）

```
.prismdocs/
  feedback/     # F4 Feedback Bundle（agent 读，处理后可留回执文件）
  context/      # F7 Context Pack（agent 读）
  README.md     # 自动生成：向 agent 解释本目录协议（英文）
```

- 产品在项目初始化时提示将 `.prismdocs/` 加入 `.gitignore`（默认建议加入；用户想让团队共享 Bundle 时可不加）。
- 提供一段可一键追加到 CLAUDE.md / AGENTS.md 的协议说明（英文，~10 行）：告诉 agent feedback 的位置、处理规范、回执方式。
- `.prismdocs/README.md` 声明本目录输出（feedback / context / 导出物）遵循 OKF v0.1 约定（v0.2 新增）。

### 4.2 本地 MCP Server 工具面

| 工具 | 方向 | 说明 |
|---|---|---|
| `list_feedback()` | agent→读 | 未处理 Bundle 列表 |
| `get_feedback(id)` | agent→读 | Bundle 全文 |
| `respond_to_comment(comment_id, action, note)` | agent→写 | 回执：done / answered / declined + 说明 |
| `get_document_comments(path)` | agent→读 | 某文档当前评论 |
| `get_context_pack(name)` | agent→读 | 拉取上下文包 |
| `list_cards(filter)` | agent→读 | 检索 context-worthy 卡片（P0.5） |
| `export_okf_bundle(selection)` | agent→读 | 导出 OKF bundle（P1，v0.2 新增，同 REQ-7.6 的 MCP 形态） |

安全：MCP 仅本地回环、仅暴露当前 Workspace 数据、写操作限于评论回执（agent 不能创建/删除评论与卡片）。

### 4.3 兼容目标

MVP 验证矩阵：Claude Code（MCP + hook，一级支持）；Cursor（MCP + 文件协议）；其他 agent（纯文件协议兜底）。

---

## 5. 非功能需求

| 类别 | 需求 |
|---|---|
| 性能 | 500 文档 / 2000 卡片规模下：全文搜索 <300ms；文档打开 <500ms；FS 变更呈现 <10s |
| 本地优先 | 断网可读可评可写卡（LLM 功能除外）；数据库为单目录可备份（SQLite + 文件） |
| 隐私 | 文档内容仅发送到用户配置的 LLM 端点；无遥测默认开启（埋点 opt-in，内测期请求授权）；剪藏与文档不经过我方服务器（MVP 无服务端） |
| 密钥 | API key 存系统钥匙串；支持自定义 base_url（兼容代理/本地模型） |
| 成本可见 | 全局设置页显示本月投影/摘要等各类 LLM 调用的 token 消耗统计 |
| 可靠性 | LLM 调用全部可重试、失败不损坏数据；投影缓存持久化，重启不重算 |
| 国际化 | UI 首发中文；Lens 目标语言架构上可扩展（P1 加日/英） |
| 平台 | macOS（Apple Silicon）首发；Windows P1；Chrome 扩展 MV3（Edge 兼容顺带） |

---

## 6. 埋点与指标对应（opt-in）

| 事件 | 对应指标 |
|---|---|
| `loop_closed`（评论 resolve 且此前经过 sent/needs-review） | 北极星：闭环数/周 |
| `first_loop_closed`（注册后首次） | 激活：7 日内 ≥40% |
| `lens_generated` / `lens_fidelity_report` | 失真率 <5% |
| `card_created`（区分入口） | 卡片/人/周 ≥3 |
| `clip_created` / `clip_used_in_pack` | 剪藏引用率 ≥25% |
| `feedback_bundle_sent` / `bundle_no_response_48h` | 闭环漏斗诊断 |
| 周活跃（打开且有 ≥1 次实质操作） | W4 留存 ≥30% |

---

## 7. MVP 发布标准（Release Criteria）

1. F1–F7 全部 P0 REQ 完成，全部 AC 通过（P0.5 项允许降级并记录）。
2. 端到端闭环（AC-4a）在 3 个真实项目、Claude Code 与 Cursor 两种 agent 上验证通过。
3. 锚点迁移专项测试：AI 大规模重写场景下评论 0 静默丢失（AC-3b）。
4. 20 名内测用户完成 M2 内测，北极星与护栏指标数据可采集。
5. 崩溃率 <1%，无数据丢失类 P0 bug。

---

## 8. 开放问题（需在设计/开发前决议）

| # | 问题 | 倾向 |
|---|---|---|
| Q1 | Lens 投影用哪档模型？（成本 vs 口语质量） | 快速模型打底 + "需要决策"段落用强模型复核；M0 阶段 A/B |
| Q2 | Base 层允许在 PrismDocs 内编辑（REQ-1.6）是否与"评论驱动"原则打架？ | 保留但入口弱化（默认只读，显式解锁），观察内测行为 |
| Q3 | 评论/卡片存 sidecar（当前方案）还是同步进 git？（换机器/团队场景） | MVP sidecar + 导出备份；git 同步作为 P1"便携模式" |
| Q4 | 剪藏原文的版权边界（整页存储） | 个人本地使用属合理范围；P1 云同步/分享前需法务审视 |
| Q5 | 产品名 | **已定名 PrismDocs**（2026-07-26 选定，取"棱镜"隐喻：一份 Base 层折射出 Lens、卡片等多个谱层）。遗留动作：上线前完成域名/商标终检——已知弱碰撞：Prism Software 商标邻近、同名个人项目、arXiv 的 DocPrism 论文 |
| Q6 | 受控 type 词表的范围与扩展策略（v0.2 新增）：九个内置 type 是否足够？用户自定义 type 如何避免重蹈 OKF"词表失控"？ | 内置词表 + 设置内登记制起步；观察内测 |
| Q7 | 是否提供"frontmatter 直写入源文件"模式（v0.2 新增）：与"不污染用户文件"原则的权衡；agent 可能更希望源文件自带元数据 | 默认 sidecar；提供项目级 opt-in 开关的方案进设计评审 |

---

## 9. 变更记录

| 版本 | 日期 | 变更 |
|---|---|---|
| v0.1 | 2026-07-26 | 初稿 |
| v0.2 | 2026-07-26 | 合并《补充调研：CoWiki 与 OKF》P1–P8：新增 §2.5 OKF 兼容约定（含受控 type 词表）；F1 新增 REQ-1.8 frontmatter 解析、REQ-1.9 log.md 物化；F4 新增 REQ-4.7 Agent 贡献溯源；F7 新增 REQ-7.6 导出 OKF Bundle；§4 协议声明 OKF、MCP 增 export_okf_bundle（P1）；P1 功能池增入库冲突检测；§8 新增 Q6/Q7。**注**：七份子 PRD 的 23 条 REQ-x.NEW 回填评审留待 v0.3 |
| v0.2.1 | 2026-07-26 | 产品定名 PrismDocs（关闭 Q5），全文档集占位名 VibeDocs → PrismDocs，协议目录 `.vibedocs/` → `.prismdocs/` |

---

## 10. 后续文档

- 设计规格（UI/UX Spec）：Inbox、阅读三视图、评论交互的高保真设计
- 技术设计文档：Block 锚定迁移算法、增量投影调度、MCP server
- M0 概念验证方案（BRD 里程碑）：手工流程脚本 + 5 用户访谈提纲
- Chrome 扩展单独的商店上架材料
