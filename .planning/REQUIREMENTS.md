# Requirements: PrismDocs

**Defined:** 2026-07-27
**Core Value:** 「双层文档 + 评论回流」显著降低 vibe coder review AI 文档的成本（F1–F4 闭环，AC-4a）
**Source:** docs/PRD_PrismDocs_MVP.md v0.2（REQ 编号映射见每条括注）；docs/BRD_PrismDocs_MVP.md v0.2

> 优先级：P0 = MVP 发布必须；P0.5 = 发布前尽力、可降级并记录（PRD §7）。P0 + P0.5 归入 v1；P1 归入 v2。
> ★ 标记 = 核心闭环 / 架构脊柱。

## v1 Requirements

Requirements for the MVP release. Each maps to exactly one roadmap phase (see Traceability).

### Import — 项目与文档导入（F1）

- [ ] **IMPORT-01**: 用户可将 PrismDocs 指向本地文件夹或 Git 仓库根目录新建项目 (PRD REQ-1.1)
- [ ] **IMPORT-02**: 首次导入按可配置 glob 扫描（默认含 `docs/**/*.md`、根 `*.md`、`CLAUDE.md`、`AGENTS.md`、`.claude/**/*.md`；排除 `node_modules`/`.git`/构建产物） (PRD REQ-1.2)
- [ ] **IMPORT-03**: 导入的 AI 生成 `.html` 报告转换为 Markdown 存为 Base 层，转换损失明显时降级为附件并提示 (PRD REQ-1.3)
- [ ] **IMPORT-04**: FS watcher 实时感知新增/修改/删除/重命名，2s 防抖合并高频写入 (PRD REQ-1.4, AC-1b)
- [ ] **IMPORT-05**: 外部修改（IDE/agent/git pull）一律以磁盘为准，PrismDocs 不锁文件——磁盘是 Base 层权威副本 (PRD REQ-1.5)
- [ ] **IMPORT-06**: 用户可在 PrismDocs 内编辑 Base 层并写回磁盘（默认只读，显式解锁——见 Q2） (PRD REQ-1.6)
- [ ] **IMPORT-07**: 导入时解析文件已有 YAML frontmatter，标准六字段入库为结构化元数据并参与筛选/搜索；无 frontmatter 不强制添加，元数据存 sidecar；type 不在受控词表内归入「未登记」并提示 (PRD REQ-1.8)
- [ ] **IMPORT-08** *(P0.5 — degradable)*: Git 感知——识别 branch 与文件 git 状态，变更记录尽量关联 commit hash (PRD REQ-1.7)
- [ ] **IMPORT-09** *(P0.5 — degradable)*: 项目级开关：将文档变更历史按 OKF `log.md` 约定物化到磁盘，默认关闭 (PRD REQ-1.9)

### Anchor — ★ Block 锚定与迁移引擎（PRD §2.4，护城河 / #1 风险）

- [ ] **ANCHOR-01**: 文档解析为 Block 树（Markdown AST：标题分节，段落/代码块/表格为叶子） (PRD §2.4)
- [ ] **ANCHOR-02**: 每个 Block 有稳定 ID（内容哈希 + 位置启发式 + 标题路径/邻居指纹） (PRD §2.4)
- [ ] **ANCHOR-03**: Base 更新时用 diff 匹配算法（内容相似度 + 相对位置）迁移 Block ID，尽量保持评论与 Lens 锚点存活 (PRD §2.4)
- [ ] **ANCHOR-04**: ★ 锚点迁移置信度低于阈值时，评论降级为「文档级评论」并保留 quote 快照 + 标记「原位置已变化」，绝不静默丢失；满足计数不变式（迁移+降级=原始） (PRD §2.4, AC-3b)
- [ ] **ANCHOR-05**: Block ID 不写入用户 Markdown 源文件，存 PrismDocs 本地 sidecar（保持源文件干净） (PRD §2.4)

### Lens — 口语化投影层（F2）

- [ ] **LENS-01**: 对 Base 文档按 Block 分段生成指定语言（首发简体中文）口语投影，Lens 段落与 Base Block 一一锚定 (PRD REQ-2.1)
- [ ] **LENS-02**: 阅读视图支持三模式切换：仅 Lens（默认）/ 对照（左右分栏）/ 仅 Base (PRD REQ-2.1)
- [ ] **LENS-03**: Lens 是口语化重述 + 取舍标注（非翻译）：讲人话、压缩重复、显式标出 ⚖️取舍 / ⚠️风险 / ❓需决策 (PRD REQ-2.2)
- [ ] **LENS-04**: 文档头自动生成「速读区」：3–5 句摘要 + 需决策事项清单（链接到对应段落） (PRD REQ-2.3)
- [ ] **LENS-05**: 增量重投影——Base 变更后仅重生成受影响 Block（含 diff 决定的相邻块）的 Lens，未变更部分复用缓存 (PRD REQ-2.4, AC-2b)
- [ ] **LENS-06**: 变更高亮——自上次「已读」以来变化的 Lens 段落显示变更条，可一键「标记本文档已读」 (PRD REQ-2.5)
- [ ] **LENS-07**: 忠实度保障——每段可一键展开 Base 原文；「需要决策」段落强制附 Base 原文摘录；提供「报告失真」按钮 (PRD REQ-2.6, AC-2c)
- [ ] **LENS-08**: Lens 不可编辑（产品原则），不满通过评论表达 (PRD REQ-2.7)
- [ ] **LENS-09**: 逐段流式渲染；失败段落显示重试按钮，不阻塞其他段落 (PRD REQ-2.8)
- [ ] **LENS-10**: 成本控制——投影调用显示预估 token；项目级「自动/手动投影」设置（默认 ≤5k token 自动） (PRD REQ-2.9)

### Comment — 段落级评论（F3）

- [ ] **COMMENT-01**: 在 Lens 或 Base 任意 Block 上评论（悬停按钮/选中浮条），选中文字存为 quote (PRD REQ-3.1, AC-3c)
- [ ] **COMMENT-02**: 评论类型：💬提问 / ✏️修改要求（默认）/ ✅Approve / ❌Reject，可附言 (PRD REQ-3.2)
- [ ] **COMMENT-03**: 评论线程支持回复；状态机 `open → sent → needs-review → resolved / reopened` (PRD REQ-3.3)
- [ ] **COMMENT-04**: 评论语言自由（中文为主）；回流时由产品附英文摘要 (PRD REQ-3.4)
- [ ] **COMMENT-05**: 文档级评论区（不针对具体段落的整体意见） (PRD REQ-3.5)
- [ ] **COMMENT-06**: 评论侧栏按文档聚合、可按状态/类型筛选；Inbox 聚合跨文档 needs-review 项 (PRD REQ-3.6)
- [ ] **COMMENT-07**: 评论数据存 PrismDocs 本地库，不污染用户 Markdown 文件 (PRD REQ-3.7)

### Loop — ★ 评论回流 AI（F4，核心闭环）

- [ ] **LOOP-01**: Feedback Bundle 生成——选择评论（默认全部 open），生成结构化反馈：目标文件路径 + Block 定位（标题路径 + 原文摘录）+ 类型 + 用户中文原文 + 产品生成的英文意图摘要 + 线程上下文 + 明确执行指令头 (PRD REQ-4.1)
- [ ] **LOOP-02**: 文件协议交付（P0）——写入 `.prismdocs/feedback/<timestamp>.md`（人类可读 Markdown + YAML 元数据），复制「喂给 agent 的一句话指令」到剪贴板 (PRD REQ-4.2, AC-4c)
- [ ] **LOOP-03**: 本地 MCP Server（P0）——提供 `list_feedback` / `get_feedback` / `respond_to_comment` / `get_document_comments` 工具；提供 Claude Code hook/skill 安装引导 (PRD REQ-4.2, §4.2)
- [ ] **LOOP-04**: 回流范围控制——Bundle 仅含评论 + 被评块原文 + 必要父级标题路径；整份文档默认不进；提供「附带全文」手动开关（token ≤ 全文 30%，AC-4b） (PRD REQ-4.3)
- [ ] **LOOP-05**: 闭环回收——agent 经 MCP 回执或产品检测到 Base 变更命中被评 Block → 评论转 needs-review + Inbox 通知；用户在变更高亮+上下文中复核 → resolve（计入北极星）或 reopen (PRD REQ-4.4)
- [ ] **LOOP-06**: 「提问」类评论的 agent 文字答复显示在评论线程内 (PRD REQ-4.5)
- [ ] **LOOP-07**: Agent 贡献溯源——每次 Base 变更记录触发来源（Feedback Bundle / MCP 回执方 / 外部未知），复核界面展示「由谁、因哪条评论触发」 (PRD REQ-4.7)
- [ ] **LOOP-08** *(P0.5 — degradable)*: Bundle 历史留档可查（哪些评论、何时、是否回执） (PRD REQ-4.6)

### Card — 理解卡片（F5）

- [ ] **CARD-01**: 卡片 = 标题 + Markdown 正文 + 双链 + 标签，无文件夹层级（Zettelkasten：靠链接不靠分类） (PRD REQ-5.1)
- [ ] **CARD-02**: 原创引导——创建界面 placeholder「你会怎么向朋友解释这件事？」；从文档/剪藏「存为卡片」时选中内容进折叠引用区（标来源链接），正文必须另写；正文空或与引用高度重复时发布前柔性提醒；正文不提供 AI 代写 (PRD REQ-5.2)
- [ ] **CARD-03**: 双链——`[[` 唤起联想选择文档/卡片/剪藏；反链面板显示「谁引用了这张卡」 (PRD REQ-5.3)
- [ ] **CARD-04**: 场景入口——评论 resolve 时提示「为这个决策写张卡片吗？」（预填上下文链接）；阅读中选中文字→存为卡片 (PRD REQ-5.4, AC-5a)
- [ ] **CARD-05**: 「注入上下文」开关——卡片可标记 context-worthy，被 F7 默认拾取；建议附英文一句话版本（AI 可代译此格式化字段） (PRD REQ-5.5)
- [ ] **CARD-06**: 卡片列表 + 全文搜索，按标签/项目/链接对象筛选 (PRD REQ-5.6)

### Clip — Chrome 剪藏插件（F6，MV3）

- [ ] **CLIP-01**: 抓取模式——整页正文提取（Readability 类算法）/ 选区剪藏 (PRD REQ-6.1)
- [ ] **CLIP-02**: 净化转换 HTML→Markdown，保障代码块（语言标注、去高亮 span 噪音）、表格、列表、图片；适配 Stack Overflow / GitHub / 技术博客 / MDN (PRD REQ-6.2, AC-6a, AC-6b)
- [ ] **CLIP-03**: 剪藏面板——标题（可改）、目标项目、标签、token 估算、备注（一句话「为什么剪它」，可跳过） (PRD REQ-6.3, AC-6c)
- [ ] **CLIP-04**: 元数据——URL、站点名、抓取时间、原文语言 (PRD REQ-6.4)
- [ ] **CLIP-05**: 与桌面端同步——本地回环通信；桌面端未运行时本地暂存队列，启动后补传 (PRD REQ-6.5)
- [ ] **CLIP-06**: 剪藏收件箱——未归类剪藏入 Inbox 式列表，支持批量归项目/归档/删除 (PRD REQ-6.6)
- [ ] **CLIP-07**: 剪藏不可被评论（MVP 语义纯度约束） (PRD REQ-6.7)

### Pack — 上下文组装（F7）

- [ ] **PACK-01**: 组装器——树形勾选文档（Base 层）/ 卡片（注入行优先，正文可选）/ 剪藏；实时显示总 token 及各项占比 (PRD REQ-7.1, AC-7b)
- [ ] **PACK-02**: 输出——写入 `.prismdocs/context/<name>.md`，结构化（来源标注、分节），纯英文倾向（中文卡片附机器英译或原文保留，用户可选） (PRD REQ-7.2)
- [ ] **PACK-03**: 常用包保存为模板；文档更新后重新生成时提示内容已变化 (PRD REQ-7.3)
- [ ] **PACK-04**: context-worthy 卡片默认预勾选 (PRD REQ-7.4)
- [ ] **PACK-05**: 与 MCP 打通——agent 可通过 `get_context_pack` 直接拉取 (PRD REQ-7.5, §4.2)
- [ ] **PACK-06** *(P0.5 — degradable)*: 导出 OKF Bundle——勾选内容导出为合规 OKF bundle 目录（sidecar 元数据物化为 frontmatter，自动生成 `index.md`），可被任何 OKF 消费方读取 (PRD REQ-7.6)

### Agent — Agent 集成协议（§4，跨功能）

- [ ] **AGENT-01**: `.prismdocs/` 目录约定（`feedback/`、`context/`、自动生成的英文 `README.md` 解释协议 + 声明遵循 OKF v0.1） (PRD §4.1)
- [ ] **AGENT-02**: 项目初始化时提示将 `.prismdocs/` 加入 `.gitignore`（默认建议）；提供可一键追加到 CLAUDE.md/AGENTS.md 的英文协议说明 (PRD §4.1)
- [ ] **AGENT-03**: MCP 安全——仅本地回环、仅暴露当前 Workspace、写操作限于评论回执（agent 不能创建/删除评论与卡片）；per-install token + Origin 校验 (PRD §4.2)
- [ ] **AGENT-04**: 兼容矩阵——Claude Code（MCP + hook，一级支持）；Cursor（MCP + 文件协议）；其他 agent 纯文件兜底 (PRD §4.3)

### NFR — 非功能需求（§5）

- [ ] **NFR-01**: 性能——500 文档 / 2000 卡片规模下：全文搜索 <300ms、文档打开 <500ms、FS 变更呈现 <10s (PRD §5)
- [ ] **NFR-02**: 本地优先——断网可读/可评/可写卡（LLM 功能除外）；数据库单目录可备份（SQLite + 文件） (PRD §5)
- [ ] **NFR-03**: 隐私——文档内容仅发送到用户配置的 LLM 端点；无遥测默认开启（埋点 opt-in）；剪藏与文档不经我方服务器 (PRD §5)
- [ ] **NFR-04**: 密钥——API key 存系统钥匙串；支持自定义 base_url（兼容代理/本地模型/OpenAI 兼容端点） (PRD §5)
- [ ] **NFR-05**: 成本可见——全局设置页显示本月各类 LLM 调用 token 消耗统计 (PRD §5)
- [ ] **NFR-06**: 可靠性——LLM 调用全部可重试、失败不损坏数据；投影缓存持久化，重启不重算 (PRD §5)
- [ ] **NFR-07**: 埋点（opt-in）——采集北极星与护栏指标：`loop_closed` / `first_loop_closed` / `lens_generated` / `lens_fidelity_report` / `card_created` / `clip_created` / `clip_used_in_pack` / `feedback_bundle_sent` / 周活跃 (PRD §6)

## v2 Requirements

Deferred (PRD 标注 P1）。Tracked but not in current MVP roadmap.

### Deferred (P1)

- **V2-01**: 多语言 Lens（日文 / 英文）
- **V2-02**: 文档间图谱视图
- **V2-03**: 决策日志自动抽取
- **V2-04**: 多人评论 / 团队协作与权限体系
- **V2-05**: GitHub PR 集成
- **V2-06**: 卡片间隔回顾（spaced repetition）
- **V2-07**: 入库冲突检测（新文档/剪藏与已有决策卡片矛盾时提示，借鉴 CoWiki「校验」阶段）
- **V2-08**: MCP `export_okf_bundle` 工具（REQ-7.6 的 MCP 形态）
- **V2-09**: MCP `list_cards(filter)` 工具（检索 context-worthy 卡片）
- **V2-10**: 卡片发布后 AI「复述质检」（指出可能理解偏了的点）
- **V2-11**: 剪藏手动框选元素 / 剪藏「AI 压缩版」英文要点摘要
- **V2-12**: Windows 桌面端 / Edge 扩展一级支持

## Out of Scope

Explicitly excluded (BRD §7.3 / PRD)。Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| IDE / 代码编辑器 / 代码生成 / 内置编码 agent | 只对接编码 agent，与其共生而非竞争；巨头/IDE 厂商赛道 |
| 通用笔记应用 | 不与 Obsidian 抢「人生笔记」，只做工程语境 |
| 对外发布的文档站 | 不与 Mintlify/GitBook 竞争 |
| Lens 层直接编辑 | 产品原则——防双层漂移（单向投影），非技术妥协 |
| 服务端 / 云同步 | MVP 本地优先，无我方服务器，文档不经我方（云同步 P1+，需法务审视版权边界 Q4） |
| 剪藏被评论 | 剪藏是外部素材，理解写进卡片，保持「评论=驱动 AI 改文档」语义纯度 |
| 多分支文档视图 | MVP 不做（IMPORT-08 仅识别当前 branch 状态） |

## Traceability

Populated during roadmap creation (`gsd-roadmapper`). Each v1 requirement maps to exactly one phase.

| Requirement | Phase | Status |
|-------------|-------|--------|
| _(pending roadmap)_ | — | Pending |

**Coverage:**
- v1 requirements: 69 total（IMPORT 9 · ANCHOR 5 · LENS 10 · COMMENT 7 · LOOP 8 · CARD 6 · CLIP 7 · PACK 6 · AGENT 4 · NFR 7）
- Mapped to phases: 0（roadmapper 待填）
- Unmapped: TBD ⚠️

---
*Requirements defined: 2026-07-27*
*Last updated: 2026-07-27 after initialization*
