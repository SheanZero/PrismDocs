# PRD：PrismDocs MVP 主产品需求文档

| 项目 | 内容 |
|---|---|
| 产品代号 | PrismDocs（暂定） |
| 文档类型 | 主 PRD（Master PRD）— MVP 版本 |
| 版本 | v0.3（构想 v2 合并：速读区 MVP、新增 F8 跨项目知识层、F6 降级 P1、变更时间线，变更记录见 §9；各子 PRD 的 REQ-x.NEW 逐条 triage 仍待进行，其中 schema 级三条已实质采纳，见 §9） |
| 日期 | 2026-07-28 |
| 上游文档 | 《BRD：面向 Vibe Coding 的双层文档管理应用（MVP）》v0.3 |
| 作者 | Shean（起草协作：Claude） |

---

## 1. 概述

### 1.1 产品一句话

PrismDocs 是 vibe coder 的多项目工程知识工作台：集中管理编码 agent 在各仓库生成的英文技术文档（Base 层），产品为每份文档生成中文速读区（讲清内容、取舍与待决策点；全文口语化投影 Lens 为 P1）；用户在文档上写评论驱动 AI 迭代，每次变更可追溯到驱动它的评论；项目间文档相互引用、契约可订阅，上游变更自动警示下游核对；用户用自己的语言写卡片沉淀理解。（Chrome 剪藏插件 v0.3 起为 P1。）

### 1.2 本 PRD 的范围

覆盖 BRD 中 P0 功能 F1–F5、F7、F8 的完整产品需求，包括交互、状态、边界情况、验收标准，以及支撑它们的信息架构、数据模型和 agent 集成协议。P1 功能（全文 Lens、F6 剪藏、语义漂移检测、多语言速读区、图谱视图、多人协作与团队版因果追溯等）不在本文范围，仅在架构上预留；F2 的全文 Lens 需求与 F6 全部需求保留于对应子 PRD，标注 P1。

### 1.3 产品形态

- **桌面应用**（macOS 优先，本地优先架构：文档、评论、卡片全部落在本地，不强制上云）
- **Chrome 扩展**（剪藏器，MV3）——v0.3 起为 P1，MVP 不交付
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
| **Workspace** | 顶层容器，对应一个用户的全部知识库。MVP 单 Workspace。v0.3 起为实体而非壳：跨项目搜索、跨项目 Inbox 聚合、跨项目引用（Xref）的作用域 |
| **Project** | 对应一个代码仓库/工作目录。含文档、卡片、剪藏的关联 |
| **Document（Base 层）** | 唯一真相源。英文 Markdown 文件，与磁盘上的真实文件双向同步（`docs/**/*.md`、`CLAUDE.md`、`AGENTS.md` 等） |
| **Digest（速读区，v0.3 修订）** | Base 文档的中文头部摘要（3–5 句摘要 + ❓ 决策清单 + 变更摘要）。派生数据，不可编辑，随 Base 变更整体重生成。**全文 Lens（按 Block 投影）为 P1**，对象定义保留 |
| **Block** | 文档的最小锚定单元（标题、段落、列表、代码块、表格）。评论与 Lens 段落都锚定到 Block |
| **Comment** | 挂在 Block 上的评论线程。类型：提问 / 修改要求 / 决策（approve · reject） |
| **Card（理解卡片）** | 用户手写的原子笔记，用自己的语言。可双链到 Document / Clip / Card |
| **Clip（剪藏）** | Chrome 插件抓取的网页内容，净化后的 Markdown + 元数据（v0.3 起随 F6 为 P1，数据模型保留） |
| **Xref（跨项目引用，v0.3 新增）** | Workspace 级引用关系：`src(project, doc, block?) → target(project, doc, block?)`，`link_type ∈ references / depends-on / contract-of`；存 sidecar，不污染源文件 |
| **Contract（契约标记，v0.3 新增）** | 任一 Document 可标记为契约（API spec、数据模型、协议约定）；下游项目按文档或 Block 粒度订阅，变更命中订阅即触发漂移警示 |
| **Context Pack** | 用户勾选文档/卡片/剪藏后组装的、给 AI 的紧凑上下文包 |
| **Feedback Bundle** | 一次「回流」导出的结构化评论包，供 agent 消费 |

### 2.2 对象关系

```
Workspace ─ 1:N ─ Project ─ 1:N ─ Document ─ 1:N ─ Block ─ 1:N ─ Comment
    │                │                │
    │                │                └─ 1:1 ─ Digest（速读区；全文 Lens 为 P1）
    │                ├─ 1:N ─ Card ──双链──> Document / Clip / Card（候选跨项目）
    │                ├─ 1:N ─ Clip（P1）
    │                └─ 1:N ─ Context Pack / Feedback Bundle
    └─ 1:N ─ Xref（跨项目引用/订阅：Document/Block × 任意两个 Project）
```

### 2.3 主导航（桌面端）

左侧栏三个一级入口 + 项目切换器：

1. **文档（Docs）**：项目文档树（镜像磁盘目录结构），徽标显示未读变更数、未解决评论数
2. **卡片（Cards）**：理解卡片列表/搜索
3. **剪藏（Clips)**：剪藏收件箱（v0.3 起随 F6 为 P1）
4. 顶部全局：**待你处理（Inbox）**——**跨项目聚合**（v0.3 修订）"新变更待 review、AI 已修改待复核、评论被回应、上游契约变更（v0.3 新增）"四类事项。**Inbox 是日常主入口**，产品的节奏是"打开 → 清 Inbox → 关掉"。

### 2.4 Block 锚定机制（关键技术约定）

- 文档解析为 Block 树（基于 Markdown AST，标题分节，段落/代码块/表格为叶子）。
- 每个 Block 有稳定 ID：内容哈希 + 位置启发式。Base 更新时用 diff 匹配算法（内容相似度 + 相对位置）迁移 Block ID，尽量保持评论与 Lens 锚点存活。
- 锚点迁移置信度低于阈值时，评论降级为「文档级评论」并标记"原位置已变化，请确认"，**绝不静默丢失**。
- Block ID 不写入用户的 Markdown 源文件（保持文件干净），存在 PrismDocs 本地库中（sidecar 存储）。
- 锚定引擎的一次 Block 级 diff 计算供四方消费（v0.3 明确）：① 评论锚点迁移（F3）；② 被评 Block 命中判定（F4 兜底回收）；③ 变更高亮与速读区重生成触发（F2）；④ 跨项目订阅命中判定（F8）。

### 2.5 OKF 兼容约定（v0.2 新增，架构级决策）

Base 层的存储与导出遵循 [Google Open Knowledge Format v0.1](https://github.com/GoogleCloudPlatform/knowledge-catalog) 的核心约定，使 PrismDocs 知识库可被任何 OKF 消费方直接读取（无锁定承诺，见 BRD §5 定位语）：

- **文件即概念**：一个 Markdown 文件一个概念，文件路径即身份——与现有 Document/Card/Clip 模型天然同构。
- **YAML frontmatter**：识别并管理 `type`（必填）、`title`、`description`、`resource`、`tags`、`timestamp` 六个标准字段；PrismDocs 内部维护**受控 type 词表**（吸取社区对 OKF"词表未注册"的批评）：`Spec` / `Plan` / `Architecture` / `Decision` / `Runbook` / `Contract`（v0.3 新增） / `Card` / `Clip` / `ContextPack` / `Doc`（缺省），可扩展但需在设置中登记。
- **保留文件名**：`index.md`（目录导航）、`log.md`（变更编年史）按 OKF 语义处理。
- **不污染原则不变**：用户源文件没有 frontmatter 时**不强制写入**；PrismDocs 的元数据存 sidecar，仅在「导出 OKF Bundle」（REQ-7.6）时物化为合规 frontmatter。用户文件已有 frontmatter 时解析入库并保持往返一致（round-trip 不破坏）。
- 跨文档引用使用标准 Markdown 链接（已是现状），导出后目录即可被 OKF 工具解析为图。跨项目引用（Xref）导出时重写为 bundle 间相对链接，`link_type` 物化为扩展字段 `x-link-type`（对 OKF"链接无类型"批评的产品级补齐）；Workspace 导出 = bundle-of-bundles，每项目一个 bundle（v0.3 新增）。

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

### F2 · 速读区生成（中文理解层，v0.3 修订）

> **v0.3 范围修订**：MVP 交付**速读区**（文档级中文摘要层）；按 Block 的全文口语化投影（Lens）连同三视图、增量重投影、逐段失真报告整体降为 **P1**——原需求（REQ-2.1/2.4 及 REQ-2.6/2.8 的逐段部分）保留于子 PRD-F2 并标注 P1，回归条件见 BRD §6.1。本节 REQ 编号沿用 v0.2。

**用户故事**：作为读英文技术文档吃力的开发者，我打开任何一份 AI 写的文档，头部的中文速读区告诉我"这文档讲了什么、AI 做了什么取舍、哪里需要我拍板、上次看后改了什么"；我按决策清单定位到具体段落（附英文原文摘录）做判断，基本不用通读英文全文。

**需求细则**

- REQ-2.2（适配保留）：速读区不是翻译，是「口语化提炼 + 取舍标注」。生成要求：讲人话（受过教育但非本领域读者能懂）；**显式标出**：⚖️ 取舍决策、⚠️ 风险/待确认、❓ 需要用户决策的点；专有名词保留英文；❓ 宁缺毋滥（误报稀释注意力，对齐 BRD 警报疲劳约束）。
- REQ-2.3（P0 核心）：速读区 = 3–5 句话摘要 + ❓ 需决策清单（每项链接跳转对应 Base Block）+ **自上次已读以来的变更摘要**（基于锚定 diff 生成，列出变更 Block 与一句话说明，v0.3 新增）。清单为空时显示"本次无需你决策"。
- REQ-2.5（P0，v0.3 改挂 Base 视图）：以用户上次「标记已读」为基线，之后变化的 **Base Block** 显示变更条（新增/修改/删除；数据源为锚定引擎 diff，不依赖投影）；「标记本文档已读」一键清除并推进基线；未读变更数上报文档树徽标与 Inbox。
- REQ-2.6（P0 部分保留）：❓ 决策清单项**强制**附 Base 原文摘录，不可隐藏（BRD 风险对策）；速读区整体提供「报告失真」入口（文档级；逐段版随全文 Lens P1）。
- REQ-2.7（不变）：速读区只读，不可编辑；用户对内容的一切意见通过评论表达。
- REQ-2.8（适配）：单次调用流式渲染；失败可重试、不损坏数据；同一文档同一时刻至多一个生成任务，重复触发合并。
- REQ-2.9（适配）：生成前显示预估 token；项目级设置"自动生成 / 手动触发"（默认：≤5k token 的文档自动，超过则提示）；消耗计入全局 token 统计。
- REQ-2.10（v0.3 新增）：缓存键 = 文档内容哈希 + prompt 版本 + 目标语言 + 模型标识；持久化，重启不重算（§5 可靠性）。

**边界情况**

- Base 本身是中文或混合语言 → 跳过口语化提炼，仅生成决策清单与变更摘要。
- 代码块不进入摘要重述，其意图由摘要文字覆盖。
- API key 未配置/额度耗尽 → 速读区显示引导卡片，Base 层阅读、评论、卡片不受影响。
- 生成进行中 Base 再次变更 → 取消进行中任务，以最新版本重新调度。

**验收标准**

- AC-2a（v0.3 口径）：内测用户对"速读区 + 原文定位足以完成 review 决策"的认同率 ≥70%。
- AC-2b（v0.3 口径）：Base 变更后速读区在防抖窗口后自动重生成；重启应用打开已生成文档 0 次 LLM 调用（缓存持久化）；变更条与锚定 diff 逐块一致。
- AC-2c（不变）：所有 ❓ 决策清单项均带 Base 原文摘录，无一例外。
- AC-2d（适配）：失真报告率 <5%（内测期）；❓ 精确率 ≥90%（M0 评测集）。

---

### F3 · 段落级评论

**用户故事**：作为 reviewer，我从速读区的决策清单跳到 Base 对应段落，圈出一段写下质疑或修改要求，就像在 Google Docs 上评论一样自然；评论精确锚定该 Block，AI 大改后也不丢。（全文 Lens 回归后，评论亦可在 Lens 段落上创建，经锚定映射到 Base。）

**需求细则**

- REQ-3.1 评论入口：在 Base 视图的任意 Block 上（悬停出现评论按钮 / 选中文字后浮条）；选中文字作为 quote 存入评论。（Lens 视图入口随全文 Lens P1。）
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

- AC-3a（v0.3 口径）：从速读区决策清单跳转后创建的评论，100% 锚定到正确的 Base Block。（对照视图验收随全文 Lens P1。）
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
- REQ-4.7 Agent 贡献溯源（v0.2 新增，P0）：每次 Base 变更记录触发来源（关联的 Feedback Bundle / MCP 回执方 / 外部未知变更），needs-review 复核界面展示"这次修改由谁、因哪条评论触发"；为 OKF v0.2 的 provenance 方向预留数据结构。借鉴 CoWiki 的问题意识：防止错误来源不明的内容进入共享上下文后被 agent 放大。（v0.3 注：复核界面为 **Base diff + 评论并排**，Lens 重投影列随全文 Lens P1。）
- REQ-4.8 变更时间线与 Block 溯源（v0.3 新增，P0）：文档变更历史升级为**时间线视图**——每个版本节点展示 diff + 驱动它的评论线程 + agent 回执 + 执行者（REQ-4.7 数据的呈现层）；任意 Block 提供「这段为什么是这样」入口，展示该 Block 历次变更及各次的驱动评论/决策。约束：external-unknown 变更如实标注"来源不明"，不得造成因果链完备的错觉；被溯源记录引用的版本快照不参与存储淘汰（扩展 REQ-1.NEW-2 的 anchored 语义）。团队版扩展（多人身份、git 便携共享时间线）为 P1。

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
- REQ-5.3 双链：`[[` 唤起联想选择文档/卡片/剪藏，候选覆盖全 Workspace（跨项目，v0.3 落实子 PRD-F5 OQ-5.2 倾向）；反链面板显示"谁引用了这张卡"。
- REQ-5.4 场景入口：评论 resolve 时提示"要为这个决策写张卡片吗？"（预填上下文链接）；文档（Base 视图）阅读中选中文字 → 存为卡片，引用区存 Base 原文（剪藏入口随 F6 P1）。
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

### F6 · Chrome 剪藏插件（v0.3 起为 P1，MVP 不交付）

> **v0.3 决议**：F6 整体移出 MVP——重述后的三个核心 job（集中管理理解、评论透明、跨项目防偏差）均不涉及外部网页素材，且它是唯一可完整剥离的独立工程线。需求全文保留于本节与子 PRD-F6，P1 第一批交付；BRD M3 的 Chrome 商店引流随之后置。

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

- REQ-7.1 组装器：树形勾选文档（Base 层）/ 卡片（注入行优先，正文可选）/ 剪藏（随 F6 P1，含"AI 压缩版"）；**作用域为 Workspace 级（v0.3 修订）**：可勾选其他项目的文档，典型用法是 client 的 Pack 带上 server 的契约文档；实时显示总 token 及各项占比。
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

### F8 · 跨项目知识层（v0.3 新增）★ 防偏差

**用户故事**：作为同时维护 server 和 client 两个仓库的开发者，我把 server 的 API spec 标记为契约、在 client 项目订阅它；server 侧 agent 改了接口文档后，client 的 Inbox 立即提醒我"上游契约变更"，我一键生成核对反馈交给 client 侧 agent，两个仓库不再悄悄漂移。

**需求细则**

- REQ-8.1 跨项目引用（Xref）：建立方式——① 应用内显式建链（文档/Block 上「引用其他项目文档」动作）；② 卡片 `[[` 双链跨项目候选（同 REQ-5.3）；③ 自动发现（P0.5）：正文中指向另一已导入项目文件的相对路径 / repo URL 链接，提示确认后收编为 Xref。引用关系存 sidecar（`link_type ∈ references / depends-on / contract-of`），不污染源文件；Block 级引用复用 §2.4 锚定机制，随锚点迁移，置信度低时显式降级警示（与评论同一"0 静默丢失"契约）。
- REQ-8.2 契约标记与订阅：任一 Document 可标记为契约（type 词表 `Contract` 或标志位）；下游项目显式订阅，粒度可到 Block（如仅订阅某一接口章节）。**订阅制是警报疲劳的第一道闸：仅订阅命中的变更才产生跨项目警示。**非 Markdown 契约（openapi.yaml、proto 等）以附件级纳入：文件级订阅，不做 Block 级。
- REQ-8.3 漂移警示：上游契约变更且锚定 diff 命中被订阅 Block → 下游项目 Inbox「上游契约变更」事项：上游 diff 摘录 + 变更溯源（复用 REQ-4.7：哪个 agent / Bundle 触发）+ 受影响的下游引用方清单；同一契约的多处变更聚合为一条（对齐 REQ-1.5.2 批量模式精神）。MVP 只做结构信号；语义漂移检测（LLM 比对上下游断言矛盾）为 P1。
- REQ-8.4 一键下游核对反馈：从警示一键在**下游项目**生成 Feedback Bundle——内容 = 上游变更 diff 摘录 + 指令头（"上游契约 X 节已变更，核对本项目文档与实现是否需要跟进，逐条回执"）——走 F4 既有双通道交付与三信号回收；复核 resolve 计入北极星闭环。**漂移修复复用评论闭环，agent 侧零新协议。**
- REQ-8.5 OKF 对齐：Xref 导出时重写为 bundle 间相对链接，`link_type` 物化为 `x-link-type` 扩展字段（见 §2.5）。

**边界情况**

- 上游契约文档被删除/重命名 → 订阅随 F1 身份识别迁移；删除时下游警示"契约源已删除"，订阅转失效态（保留记录，不静默消失）。
- 订阅的 Block 被拆分/合并 → 随锚点迁移；置信度低 → 警示"订阅段落已大幅变化，请重新确认订阅范围"。
- 上游变更由同一用户在上游的评论回流触发 → 警示仍产生，但标注"由你在上游的评论触发"，可一键静默。
- 同一契约被多个下游订阅 → 各下游独立警示、独立核对 Bundle。
- 循环引用（A 订 B、B 订 A）→ 允许；核对 Bundle 不自动级联生成（防警示风暴，人工逐环确认）。

**验收标准**

- AC-8a：server/client 双仓真实场景跑通"上游契约变更 → 下游警示（FS 呈现预算 10s 内）→ 一键核对 Bundle → 下游 agent 处理 → 复核 resolve"全链路。
- AC-8b：未订阅的上游变更 0 跨项目警示（订阅制过滤有效）；警示误报率 <10%（内测观察）。
- AC-8c：上游文档被 agent 大规模重写后，订阅与 Xref 随锚点 ≥90% 正确迁移或显式降级，0 静默丢失（复用 AC-3b 测试集）。

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
| `list_dependencies(path)` / `get_upstream_contracts()` | agent→读 | 本项目的上游契约与跨项目引用（P1，v0.3 新增；MVP 用 Context Pack 承载跨项目上下文） |

传输（v0.3 明确，决策 D-07）：MCP server 由桌面应用自身托管——loopback streamable HTTP（`127.0.0.1:<port>`），per-install bearer token（存系统钥匙串）+ 非空 Origin allowlist，无子进程；配套一个轻量 CLI helper 承担 Claude Code `headersHelper`（从钥匙串读 token）与 SessionStart hook 的 check-feedback 提示。**子 PRD-F4 中早期的 stdio 代理方案（REQ-4.NEW-1）作废，以本条为准。**

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
| 国际化 | UI 首发中文；速读区/Lens 目标语言架构上可扩展（P1 加日/英） |
| 平台 | macOS（Apple Silicon）首发；Windows P1；Chrome 扩展 MV3（Edge 兼容顺带，随 F6 为 P1） |

---

## 6. 埋点与指标对应（opt-in）

| 事件 | 对应指标 |
|---|---|
| `loop_closed`（评论 resolve 且此前经过 sent/needs-review） | 北极星：闭环数/周 |
| `first_loop_closed`（注册后首次） | 激活：7 日内 ≥40% |
| `digest_generated` / `digest_fidelity_report`（v0.3 改速读区口径） | 失真率 <5%；❓ 精确率评估 |
| `xref_created` / `contract_subscribed` / `drift_alert_shown` / `drift_alert_resolved` / `drift_feedback_sent`（v0.3 新增） | 漂移警示处理率 ≥60%、误报率 <10%；跨项目采用度 |
| `timeline_viewed` / `block_provenance_viewed`（v0.3 新增） | 变更时间线使用率（透明价值验证） |
| `card_created`（区分入口） | 卡片/人/周 ≥3 |
| `clip_created` / `clip_used_in_pack`（随 F6 为 P1） | 剪藏引用率 ≥25%（P1 生效） |
| `feedback_bundle_sent` / `bundle_no_response_48h` | 闭环漏斗诊断 |
| 周活跃（打开且有 ≥1 次实质操作） | W4 留存 ≥30% |

---

## 7. MVP 发布标准（Release Criteria）

1. F1–F5、F7、F8 全部 P0 REQ 完成，全部 AC 通过（P0.5 项允许降级并记录；F2 按 v0.3 速读区口径；F6 不在发布标准内）。
2. 端到端闭环（AC-4a）在 3 个真实项目、Claude Code 与 Cursor 两种 agent 上验证通过；跨项目链路（AC-8a）在 server/client 双仓场景验证通过。
3. 锚点迁移专项测试：AI 大规模重写场景下评论 0 静默丢失（AC-3b）。
4. 20 名内测用户完成 M2 内测，北极星与护栏指标数据可采集。
5. 崩溃率 <1%，无数据丢失类 P0 bug。

---

## 8. 开放问题（需在设计/开发前决议）

| # | 问题 | 倾向 |
|---|---|---|
| Q1 | 速读区生成用哪档模型？（成本 vs 口语质量；v0.3 自"Lens 投影"缩围） | 快速档单调用起步；M0 用 ❓ 精确率 / 忠实度 / 成本三维评测定档 |
| Q2 | Base 层允许在 PrismDocs 内编辑（REQ-1.6）是否与"评论驱动"原则打架？ | 保留但入口弱化（默认只读，显式解锁），观察内测行为 |
| Q3 | 评论/卡片存 sidecar（当前方案）还是同步进 git？（换机器/团队场景） | MVP sidecar + 导出备份；git 同步作为 P1"便携模式" |
| Q4 | 剪藏原文的版权边界（整页存储） | 个人本地使用属合理范围；P1 云同步/分享前需法务审视 |
| Q5 | 产品名 | **已定名 PrismDocs**（2026-07-26 选定，取"棱镜"隐喻：一份 Base 层折射出 Lens、卡片等多个谱层）。遗留动作：上线前完成域名/商标终检——已知弱碰撞：Prism Software 商标邻近、同名个人项目、arXiv 的 DocPrism 论文 |
| Q6 | 受控 type 词表的范围与扩展策略（v0.2 新增）：九个内置 type 是否足够？用户自定义 type 如何避免重蹈 OKF"词表失控"？ | 内置词表 + 设置内登记制起步；观察内测 |
| Q7 | 是否提供"frontmatter 直写入源文件"模式（v0.2 新增）：与"不污染用户文件"原则的权衡；agent 可能更希望源文件自带元数据 | 默认 sidecar；提供项目级 opt-in 开关的方案进设计评审 |
| Q8 | 契约订阅的默认粒度（整文档 vs 引导选 Block）与警示聚合窗口（v0.3 新增） | 默认整文档订阅 + 引导标注关键章节；聚合窗口对齐 F1 批量模式；M0 走查校准 |
| Q9 | 团队版共享机制：评论/溯源 sidecar 走 git 便携模式还是同步服务？（v0.3 新增，关联 Q3） | 团队版第一步走 git 便携模式（零服务端，repo 权限即评论权限，变更时间线随 git 同步）；同步服务后置 |

---

## 9. 变更记录

| 版本 | 日期 | 变更 |
|---|---|---|
| v0.1 | 2026-07-26 | 初稿 |
| v0.2 | 2026-07-26 | 合并《补充调研：CoWiki 与 OKF》P1–P8：新增 §2.5 OKF 兼容约定（含受控 type 词表）；F1 新增 REQ-1.8 frontmatter 解析、REQ-1.9 log.md 物化；F4 新增 REQ-4.7 Agent 贡献溯源；F7 新增 REQ-7.6 导出 OKF Bundle；§4 协议声明 OKF、MCP 增 export_okf_bundle（P1）；P1 功能池增入库冲突检测；§8 新增 Q6/Q7。**注**：七份子 PRD 的 23 条 REQ-x.NEW 回填评审留待 v0.3 |
| v0.2.1 | 2026-07-26 | 产品定名 PrismDocs（关闭 Q5），全文档集占位名 VibeDocs → PrismDocs，协议目录 `.vibedocs/` → `.prismdocs/` |
| v0.3 | 2026-07-28 | 构想 v2 合并（上游 BRD v0.3；依据《调研_整体构想v2_多项目知识层》《调研_技术基建与开发Phase》v0.2）：①F2 修订为速读区（REQ-2.2/2.3/2.5/2.6/2.7/2.8/2.9 适配保留、新增 REQ-2.10 缓存；REQ-2.1/2.4 及三视图/逐段流式/逐段失真降 P1；AC-2a/2b/2d 换口径）；②新增 §3-F8 跨项目知识层（REQ-8.1～8.5、AC-8a/8b/8c），领域对象增 Xref/Contract，type 词表增 `Contract`，§2.4 明确锚定四消费方；③F6 整体降 P1（§1.3/§2.3/§3-F6/§5/§7）；④新增 REQ-4.8 变更时间线与 Block 溯源，REQ-4.7 复核界面改 Base diff + 评论并排；⑤F7 升级 Workspace 作用域；⑥F5 双链跨项目化、存卡入口改 Base 视图；⑦§4.2 明确 MCP 传输 D-07（作废子 PRD-F4 REQ-4.NEW-1 的 stdio 方案）、工具面预留 `list_dependencies`/`get_upstream_contracts`（P1）；⑧§6 埋点、§7 发布标准同步；⑨Q1 缩围，新增 Q8/Q9。注：本版已实质依赖子 PRD 的 REQ-1.NEW-1/2（文档身份、版本快照）与 REQ-7.NEW-1（token 预算），视为已采纳；其余 REQ-x.NEW 逐条 triage 仍待进行 |

---

## 10. 后续文档

- 子 PRD-F8（跨项目知识层，按主 PRD §3-F8 展开）；子 PRD-F2 v0.2（速读区拆分与全文 Lens P1 标注）；子 PRD-F4 v0.3（MCP 传输改 D-07、复核界面修订）
- 设计规格（UI/UX Spec）：Inbox（跨项目聚合）、速读区 + Base 阅读视图、评论交互、变更时间线、复核界面（Base diff + 评论并排）
- 技术设计文档：Block 锚定迁移契约（四消费方接口，第一优先）、MCP server（D-07）、速读区生成调度
- M0 概念验证方案（BRD 里程碑）：三赛道方案 + 5 用户访谈提纲（含多仓开发者筛选）
- 团队版方向文档（P1）：因果可追溯——共享变更时间线、身份体系、git 便携模式（关联 Q9）
- Chrome 扩展单独的商店上架材料（随 F6 P1）
