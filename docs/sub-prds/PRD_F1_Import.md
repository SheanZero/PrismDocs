# PRD-F1：项目与文档导入（PrismDocs MVP 子 PRD）

| 项目 | 内容 |
|---|---|
| 文档类型 | 功能子 PRD |
| 版本 | v0.2（同步主 PRD v0.2 的 OKF 兼容变更） |
| 日期 | 2026-07-26 |
| 上游文档 | 主 PRD《PrismDocs MVP 主产品需求文档》v0.2 §3-F1（另参照 §2 信息架构（含 §2.5 OKF 兼容约定）、§4 agent 协议、§5 非功能需求）；BRD v0.1 |
| 作者 | Shean（起草协作：Claude） |

---

## 1. 功能概述与目标

F1 是 PrismDocs 的入口功能：用户将产品指向本地项目文件夹（Project = 一个代码仓库/工作目录），PrismDocs 扫描并导入 Markdown / HTML 文档为 Base 层（Document），此后通过 FS watcher 持续与磁盘同步。核心产品原则：**磁盘文件是 Base 层的权威副本**——PrismDocs 不锁文件、不要求用户改变 Claude Code / Cursor 的任何使用习惯（BRD §11「接入成本压到最低」）。

**对北极星指标（闭环数/周）的贡献路径**：F1 不直接产生闭环，但决定闭环的起点质量与激活率。路径为：导入顺畅（AC-1a：20 份 .md ≤60s 可读）→ 新用户尽快看到自己项目的文档进入 Inbox → 触发首次 Lens 阅读与评论（F2/F3）→ 7 日内首个闭环（激活 ≥40%）。同步可靠性（防抖合并、身份识别）保证「agent 改完 → needs-review 提醒」这半环不丢事件；文档身份与锚点存活（AC-1c）保证评论不因文件改名而失效，是「0 静默丢失」承诺的地基。

---

## 2. 范围

### In Scope

- 新建 Project 的三步 onboarding（选文件夹 → 扫描预览 → 确认导入）
- glob 包含/排除规则的设置 UI 与实时预览（REQ-1.2）
- HTML→Markdown 转换管线与转换质量分级（REQ-1.3）
- FS watcher：新增/修改/删除/重命名感知，防抖与事件合并（REQ-1.4）
- 文档身份识别（内容哈希 + 路径启发式）与重命名/移动追踪
- Base 层的磁盘权威同步（REQ-1.5）与应用内编辑写回（REQ-1.6）
- Git 感知（REQ-1.7，P0.5）
- 变更记录（version snapshot）保存与存储上限
- `.prismdocs/` 目录初始化与 .gitignore 提示（主 PRD §4.1，操作发生在 F1 项目创建时）
- 文件系统异常处理（权限、符号链接、非 UTF-8、超大文件等）

### Out of Scope

- Lens 生成（F2）、评论（F3）、回流（F4）、卡片（F5）、剪藏（F6）、Context Pack（F7）的自身逻辑
- 远程 Git 托管平台集成（clone、PR）——用户须先自行 clone 到本地
- 多分支文档视图（主 PRD REQ-1.7 明确 MVP 不做）
- 云同步、多 Workspace（MVP 单 Workspace）

### 与其他 F 的接口

| 对接功能 | F1 提供 | F1 依赖 |
|---|---|---|
| F2 Lens | 文档导入/变更事件（含变更 Block 集合），触发投影与增量重投影 | 无 |
| F3 评论 | Block 树与 Block ID 迁移结果（§2.4 锚定机制的输入数据）；文档 archived 状态 | 无 |
| F4 回流 | `.prismdocs/` 目录初始化；Base 变更检测事件（兜底闭环 REQ-4.4） | `.prismdocs/feedback/` 写入不得触发自导入（见 REQ-1.2.3） |
| F5 卡片 | 文档存在性状态（源已删 → 卡片引用变快照） | 无 |
| F7 Context Pack | 文档树与 Base 内容 | `.prismdocs/context/` 同样排除在导入之外 |

---

## 3. 用户故事与关键场景

**US-1 新用户首次导入（onboarding）**：Shean 首次打开 PrismDocs → 点「新建项目」→ 选择 `~/dev/myapp` 文件夹 → 看到扫描预览（37 个候选文件，含/排规则可调，2 个 HTML 将转换、1 个文件将跳过并说明原因）→ 确认导入 → 进度页逐个显示导入完成 → 跳转文档树，同时弹出 `.prismdocs/` 初始化与 .gitignore 建议。

**US-2 agent 连续写盘**：Shean 在 Claude Code 里让 agent 重构文档，agent 在 30 秒内对 `docs/architecture.md` 写盘 8 次并新建 2 个文件。PrismDocs 防抖合并后只产生每文件一条变更记录，10 秒内呈现最新内容，Inbox 出现「3 份文档有新变更」。

**US-3 重命名/移动**：agent 执行 `git mv docs/plan.md docs/roadmap.md`。PrismDocs 通过内容哈希识别为同一 Document，路径更新，其上 5 条 Comment 与 Lens 锚点全部随迁，不产生「删除+新增」的假事件。

**US-4 HTML 报告导入**：agent 生成了 `report.html`。扫描判定其为「可转换」级，转换为 Markdown 存为 Base 层并保留原文件引用；另一份含大量脚本图表的 `dashboard.html` 判定「降级」级，提示「以附件方式保存，不参与双层结构」。

**US-5 外部删除**：Shean 在 IDE 里删除了一份已有评论的文档。PrismDocs 将其标记 archived，评论与 Card 引用保留并显示「源文件已删除」；文件若在垃圾恢复后回到原路径且哈希匹配，可一键取消 archived。

---

## 4. 详细功能需求

### REQ-1.1 新建项目（细化）

- REQ-1.1.1 入口：空态引导页「新建项目」按钮 + 菜单栏；调用系统文件夹选择器，只接受目录。
- REQ-1.1.2 校验：所选目录被重复添加（与已有 Project 根路径相同或互为父子）→ 阻止并跳转/提示已有项目（继承主 PRD 边界情况；父子重叠为细化：提示选择其一）。
- REQ-1.1.3 三步 onboarding：①选文件夹 → ②扫描预览（见 REQ-1.2.4）→ ③确认导入。确认前不落库、不写任何文件；用户可在②返回①换目录。
- REQ-1.1.4 确认导入后：批量导入（并行解析，进度可见）；完成后初始化 `.prismdocs/` 目录（`feedback/`、`context/`、`README.md`，按主 PRD §4.1），并弹出 .gitignore 提示：检测项目根 `.gitignore`，若未含 `.prismdocs/` 则提供「一键追加」（默认建议加入；提示文案说明团队共享 Bundle 场景可不加）。无 `.gitignore` 且非 git 仓库时不弹此提示。
- REQ-1.1.5 同时提供「一键追加 agent 协议说明到 CLAUDE.md / AGENTS.md」入口（主 PRD §4.1 第二条），可跳过，可稍后在项目设置中补做。

### REQ-1.2 扫描规则与 glob 配置（细化）

- REQ-1.2.1 默认包含：`docs/**/*.md`、根目录 `*.md`、`CLAUDE.md`、`AGENTS.md`、`.claude/**/*.md`；默认排除：`node_modules/**`、`.git/**`、常见构建产物目录（`dist/**`、`build/**`、`out/**`、`target/**`、`.next/**` 等预置清单）。
- REQ-1.2.2 HTML 纳入：默认包含 `docs/**/*.html` 与根目录 `*.html`（走 REQ-1.3 转换管线）。【主 PRD 未明确 HTML 的默认 glob 范围，此为细化默认值，见开放问题 OQ-1】
- REQ-1.2.3 强制排除（用户不可解除）：`.prismdocs/**`，防止 Feedback Bundle / Context Pack 被回导成 Document 形成回路。
- REQ-1.2.4 设置 UI 与实时预览：包含/排除规则各为一个可增删的 glob 列表（文本行编辑），每次编辑后 ≤500ms 内刷新右侧预览树——命中文件高亮、被排除文件灰显并标注命中的排除规则；顶部显示「将导入 N 个文件（约 X MB）」。onboarding 第②步与项目设置页复用同一组件。
- REQ-1.2.5 规则修改在既有项目上生效时：新命中的文件增量导入；不再命中的文档转 archived（评论/引用保留），不物理删除任何数据。
- REQ-1.2.6（v0.2 新增）保留文件名的特殊处理：`index.md` 与 `log.md` 两个保留文件名按主 PRD §2.5 的 OKF 语义处理——`index.md` 参与导航（作为所在目录的导航入口展示）；`log.md` 不作为普通 Document 进入评论/Lens 流程（在文档树中标注为「变更编年史」，只读查看）。

### REQ-1.3 HTML→Markdown 转换与质量分级（细化）

- REQ-1.3.1 管线：HTML 解析 → 正文提取（去 nav/footer/script/style）→ 转 Markdown（保代码块语言标注、表格、列表、图片 URL）→ 产出 Base 层 Markdown；Document 记录 `source_format=html` 与原文件引用，原 .html 不删除不修改。
- REQ-1.3.2 质量分级（两级，对齐主 PRD「可转/降级」语义）：
  - **可转换**：转换后文本损失可忽略 → 存为 Base 层，参与双层结构。
  - **降级为附件**：判定信号命中任一即降级——正文提取后文本量 < 原始可见文本的 60%；脚本渲染型页面（body 几乎为空、依赖 JS）；表单/画布/iframe 为主体。降级文件在文档树中以附件图标展示，可打开原 HTML，不生成 Lens、不可评论，提示语沿用主 PRD：「以附件方式保存，不参与双层结构」。
  - 判定结果在扫描预览中提前展示，用户可对单个文件手动改判（附件→强制转换，反之亦然）。
- REQ-1.3.3 源 .html 后续变更：重跑管线，转换结果作为该 Document 的 Base 新版本进入变更记录；用户在 PrismDocs 内编辑此类 Document（REQ-1.6）时警示「源为 HTML，编辑内容不会写回 .html」并写入 sidecar 副本。【主 PRD 未覆盖 HTML 源文档的写回语义，标记 REQ-1.NEW-3，见 §10】

### REQ-1.4 FS watcher：防抖与事件合并（细化）

- REQ-1.4.1 监听项目根目录递归 watch，按 glob 过滤后进入事件队列。
- REQ-1.4.2 防抖：单文件 2s 静默窗口（继承主 PRD 防抖 2s）——收到该文件事件后启动/重置 2s 计时器，窗口内后续事件合并；另设单文件 10s 上限（即使持续写入，最迟 10s 强制处理一次），保证 §5 非功能「FS 变更呈现 <10s」。
- REQ-1.4.3 事件合并语义（同一防抖窗口内）：修改×N → 1 次变更；新增+修改 → 1 次新增；新增+删除 → 忽略（临时文件，如编辑器 swap）；修改+删除 → 删除；删除+同哈希新增（可跨路径）→ 重命名/移动（见 REQ-1.NEW-1）。
- REQ-1.4.4 处理产物：每次合并处理产生至多 1 条变更记录（对应 AC-1b「无重复变更记录」），并向 F2/F3 发布变更 Block 集合。
- REQ-1.4.5 watcher 失效兜底：应用启动、窗口重新聚焦、以及每 5 分钟做一次轻量全量对账（mtime+size 快扫，差异文件再算哈希），修复漏事件。

### REQ-1.5 磁盘权威同步（细化）

- REQ-1.5.1 一切外部修改（IDE、agent、git pull/checkout）以磁盘为准，PrismDocs 无条件接受，不产生冲突弹窗、不锁文件、不写临时锁文件。
- REQ-1.5.2 git 批量变更（pull/checkout 触发大量文件事件）：进入「批量模式」——聚合为一条「同步了 N 个文件」的汇总记录 + 每文件变更明细，Inbox 只推一条汇总，避免警报疲劳（BRD 设计约束）。
- REQ-1.5.3 读取失败（文件被独占、瞬时不可读）：指数退避重试 3 次，仍失败则保留上一版本并在文档上标「同步暂停，点击重试」。

### REQ-1.6 应用内编辑 Base 层（细化）

- REQ-1.6.1 默认只读，显式解锁进入编辑（对齐主 PRD 开放问题 Q2 倾向「入口弱化」）。
- REQ-1.6.2 保存 = 直接写回磁盘文件（原子写：临时文件 + rename），写回产生的 FS 事件须被识别为自身操作，不重复生成变更记录、不触发 Inbox 通知。
- REQ-1.6.3 编辑期间磁盘文件被外部修改：保存前检测 mtime/哈希漂移 → 提示「磁盘已更新」，提供「以磁盘为准放弃我的编辑」或「查看差异后覆盖」；不做自动合并（磁盘权威原则下，覆盖是用户显式决定）。

### REQ-1.7 Git 感知（P0.5，细化）

- REQ-1.7.1 识别项目根是否 git 仓库；是则显示当前 branch；切换 branch 走 REQ-1.5.2 批量模式。
- REQ-1.7.2 变更记录尽量关联 commit hash：处理变更时若文件在 git 中且 HEAD 变动与该变更时间接近，则在变更记录上附 commit hash（尽力而为，不保证）。
- REQ-1.7.3 降级：非 git 仓库一切功能可用，仅无 git 元信息。MVP 不做多分支文档视图（继承）。

### REQ-1.8 frontmatter 解析（v0.2 新增，P0，细化）

细化主 PRD REQ-1.8 与 §2.5 OKF 兼容约定：

- REQ-1.8.1 解析时机：导入时（含扫描预览阶段的轻量探测）与文件变更时（REQ-1.4 每次合并处理后）解析文件头部的 YAML frontmatter；无 frontmatter 的文件正常导入，不强制添加（不污染原则不变）。
- REQ-1.8.2 标准字段入库：识别 §2.5 的六个标准字段 `type` / `title` / `description` / `resource` / `tags` / `timestamp`，入库为 sidecar 结构化元数据，参与文档树筛选与全文搜索；非标准扩展字段原样保留、不入结构化列（随原文快照留存）。
- REQ-1.8.3 受控 type 词表校验：`type` 值对照主 PRD §2.5 受控词表（`Spec`/`Plan`/`Architecture`/`Decision`/`Runbook`/`Card`/`Clip`/`ContextPack`/`Doc` 及用户已登记扩展）校验；不在词表内 → 归入「未登记」并在文档元数据处提示，可一键跳转设置登记（对应主 PRD 开放问题 Q6）。
- REQ-1.8.4 往返一致性（round-trip）：用户文件已有的 frontmatter 在 PrismDocs 的一切写回路径（REQ-1.6 应用内编辑保存）中原样保留——不重排字段、不改写格式、不增删注释；元数据的应用内修改只落 sidecar，不写回源文件（frontmatter 直写入源文件的 opt-in 模式属主 PRD 开放问题 Q7，MVP 不做）。
- REQ-1.8.5 语法错误容错：frontmatter YAML 解析失败时按「无 frontmatter」处理（整个文件含分隔符视为正文导入，不阻塞导入），并在项目设置的跳过/异常清单中列出「frontmatter 语法错误」及原因，供用户修复。

### REQ-1.9 `log.md` 物化（v0.2 新增，P0.5，细化）

细化主 PRD REQ-1.9：

- REQ-1.9.1 项目级开关，默认关闭；开启时在设置中说明用途（供外部 agent 读取文档演化史）与写入位置。
- REQ-1.9.2 写入格式遵循 OKF `log.md` 约定：变更编年史按时间倒序排列，每条目含时间戳 + 变更摘要（涉及文件与变更类型）+ 来源（origin：external / app_edit / import / bulk_sync，git commit hash 可得时附带）。
- REQ-1.9.3 与 REQ-1.NEW-2 变更记录（version snapshot）的关系：`log.md` 是快照历史的**对外物化视图**，由 sidecar 中的 `document_version` 数据派生生成，不是另一套独立数据；开关开启后追加写入，关闭后停止写入但不删除已有文件。
- REQ-1.9.4 自触发防护：`log.md` 自身被 watcher 忽略（保留文件名排除 + 自身写盘「写前登记 hash」回声识别双保险），避免「物化写盘 → 触发变更 → 再物化」的循环。

### 主 PRD 未覆盖的新需求

- **REQ-1.NEW-1 文档身份识别与重命名追踪**（主 PRD 边界情况提及「内容哈希识别」，机制细节未覆盖，需回填）：Document 身份 = 内容哈希（归一化后正文的 SHA-256）为主 + 路径启发式为辅。删除事件后 30s 内出现同哈希新增 → 判定移动/重命名，Document ID 不变、路径更新、评论/Lens/Card 引用全部随迁。内容与路径同时变化（改名且改内容）：用相似度匹配（哈希不中时对候选文件做内容相似度 ≥85% + 目录/文件名相似加权）判定；置信度不足时按「删除+新增」处理并在旧文档（archived）上提示「疑似移动到 X，手动合并？」——与锚点「绝不静默丢失」原则一致。
- **REQ-1.NEW-2 变更记录（version snapshot）与存储上限**（主 PRD 未覆盖，需回填）：每条变更记录保存该版本全文快照（压缩存储于 sidecar 本地库，不写用户目录），供 F2 diff 定位受影响 Block、F3 quote 快照、变更历史查看。保存策略：每次合并处理存 1 版；同一文档相邻版本 ≤60s 且无用户交互（未读、无评论触达）时只保留最新。存储上限：单文档默认保留最近 50 版 + 全部「有评论/已读标记锚定」的版本；项目快照总量默认上限 500MB，超限时从最旧的无锚定版本开始淘汰，设置页可调并显示当前占用。
- **REQ-1.NEW-3 HTML 源文档编辑写回语义**：见 REQ-1.3.3（主 PRD 未覆盖，需回填）。
- **REQ-1.NEW-4 文件系统异常处理**（主 PRD 未覆盖，需回填）：详见 §7——符号链接、权限、大小写、外置卷等。

---

## 5. 交互与界面规格

### 页面清单

1. 空态引导页（无项目时）
2. 新建项目向导（三步）
3. 导入进度页
4. 项目设置页（扫描规则、跳过文件清单、快照存储、git 信息、agent 协议安装）
5. 文档树侧栏（主导航「文档（Docs）」入口，主 PRD §2.3）
6. 文档变更历史面板

### 关键界面状态

| 界面 | 默认 | 空态 | 加载 | 错误 |
|---|---|---|---|---|
| 空态引导页 | — | 「新建项目」大按钮 + 一句话价值说明 | — | — |
| 向导②扫描预览 | 规则列表 + 预览树 + 统计条 | 「0 个文件命中，检查包含规则」+ 常见目录建议 | 扫描中骨架树 + 已扫描文件计数 | 目录不可读：错误横幅 + 「重新授权/换目录」 |
| 导入进度页 | 逐文件进度 + 完成计数 | — | 进行中可取消（已导入的保留） | 失败文件汇总入「跳过清单」，不阻塞整体 |
| 文档树 | 镜像磁盘目录结构 + 未读变更/未解决评论徽标 | 「暂无文档，去检查扫描规则」 | 增量导入时新文件占位闪烁 | archived 文档灰显 +「源文件已删除」；同步暂停图标 |
| 项目设置-规则 | 同向导②组件 | — | 预览刷新 ≤500ms 内联 loading | glob 语法错误：行内红字提示，不生效 |

### ASCII 线框：向导②扫描预览

```
┌─ 新建项目 (2/3)：扫描预览 ───────────────────────────────┐
│ 项目：~/dev/myapp                                        │
│ ┌─ 包含规则 ──────────┐ ┌─ 预览（37 个文件，1.8MB）────┐ │
│ │ docs/**/*.md    [x] │ │ ▾ docs/                      │ │
│ │ *.md            [x] │ │   architecture.md        ✓   │ │
│ │ CLAUDE.md       [x] │ │   report.html      ✓ 可转换  │ │
│ │ .claude/**/*.md [x] │ │   dashboard.html   ▲ 附件    │ │
│ │ + 添加规则          │ │   old-spec.md.bak   灰(未命中)│ │
│ ├─ 排除规则 ──────────┤ │ ▾ .claude/                   │ │
│ │ node_modules/** [x] │ │   commands.md            ✓   │ │
│ │ dist/**         [x] │ │ CLAUDE.md                ✓   │ │
│ │ + 添加规则          │ │ big-log.md  ⚠ 跳过(>1MB)     │ │
│ └─────────────────────┘ └──────────────────────────────┘ │
│ 将导入 35 份，转换 1 份，附件 1 份，跳过 1 份             │
│                       [上一步]        [确认导入 →]       │
└──────────────────────────────────────────────────────────┘
```

### ASCII 线框：`.prismdocs/` 初始化提示（导入完成后）

```
┌─ 项目已就绪 ────────────────────────────────┐
│ ✓ 已初始化 .prismdocs/（feedback/ context/） │
│ 建议把 .prismdocs/ 加入 .gitignore           │
│ （若想与团队共享反馈文件可跳过）            │
│ [一键追加到 .gitignore]  [跳过]             │
│ ─────────────────────────────────────────── │
│ 可选：把 agent 协议说明追加到 CLAUDE.md     │
│ [追加]  [稍后在项目设置中操作]              │
└─────────────────────────────────────────────┘
```

---

## 6. 数据模型与技术要点（供技术设计参考，非最终 schema）

- `project`：id、root_path、created_at、include_globs(JSON)、exclude_globs(JSON)、snapshot_quota_mb、is_git、current_branch
- `document`：id、project_id、rel_path、content_hash、source_format(md/html)、status(active/archived/skipped/attachment)、skip_reason、mtime、size、updated_at
- 元数据字段（v0.2 新增，REQ-1.8）：`type`、`title`、`description`、`resource`、`tags(JSON)`、`timestamp`、`type_registered(bool)`、`meta_source(frontmatter/user/none)`——存 sidecar 本地库（可为 `document` 表列或独立 `document_meta` 表，技术设计定）；**不写入用户源文件**；用户已有 frontmatter 原文随快照保留以保障 round-trip 与扩展字段不丢失
- `document_version`（snapshot）：id、document_id、content_hash、content(压缩)、created_at、origin(external/app_edit/import/bulk_sync)、git_commit(nullable)、anchored(bool，被评论/已读标记引用)
- `fs_event_log`：debug 用环形日志，记录原始事件与合并决策（AC 验证与故障排查）
- 存储位置：全部在 Workspace 的 sidecar 本地库（SQLite + 文件，主 PRD §5），不写用户项目目录（`.prismdocs/` 内仅 F4/F7 产物与 README）。
- 技术要点：watcher 建议用平台原生 API（macOS FSEvents）封装库；归一化哈希 = 换行统一 LF、去尾随空白后 SHA-256；原子写回（temp + rename）；自身写盘用「写前登记 hash」识别回声事件。

---

## 7. 边界情况与异常处理

| # | 情况 | 处理 |
|---|---|---|
| 1 | 文件被外部删除 | Document 转 archived，评论/卡片引用保留，显示「源文件已删除」（继承主 PRD） |
| 2 | 重命名/移动 | 内容哈希识别同一 Document，锚点随迁（继承；机制见 REQ-1.NEW-1） |
| 3 | 单文件 >1MB 或非 UTF-8 | 跳过，项目设置「跳过清单」列出原因（继承）；UTF-8 BOM 可读，UTF-16 视为非 UTF-8 跳过 |
| 4 | 重复添加同一文件夹 | 阻止并跳转已有项目（继承）；父子目录重叠同样阻止 |
| 5 | 符号链接（文件） | 解析目标在项目根内 → 按目标导入一次（去重）；指向根外 → 默认跳过并入跳过清单（可在设置放开） |
| 6 | 符号链接（目录）/ 链接成环 | 默认不跟随目录符号链接，杜绝环形扫描 |
| 7 | 无读权限的文件/子目录 | 跳过清单标「权限不足」；项目根整体失去权限（如 macOS 完全磁盘访问变更）→ 全局横幅引导重新授权 |
| 8 | 磁盘卷卸载/外置盘拔出 | 项目转「离线」态：已导入内容与快照可读、评论可写，watcher 暂停；卷恢复后自动对账 |
| 9 | 大小写改名（macOS 不敏感 FS） | 按 REQ-1.NEW-1 视为同一文档路径更新 |
| 10 | 空文件 / 仅 frontmatter | 正常导入；F2 侧自行决定是否投影 |
| 11 | 项目根被整体删除/移动 | 项目转 archived，数据保留；提供「重新定位文件夹」（按各文档哈希对账恢复） |
| 12 | 导入中途退出应用 | 已导入部分持久化，重启后从对账（REQ-1.4.5）继续，不重复导入 |
| 13 | agent 写入 `.prismdocs/` | 强制排除，不产生 Document（REQ-1.2.3） |
| 14 | HTML 转换崩溃/超时 | 该文件降级为附件并入跳过清单可重试，不阻塞批次 |
| 15 | 同一内容多个文件（哈希碰撞语义） | 各自独立为 Document（路径不同即不同文档）；仅「删除+新增」配对时才用哈希判移动 |
| 16 | frontmatter 与 sidecar 元数据冲突（源文件 frontmatter 变更，而用户已在应用内显式修改过同一字段） | 以 sidecar 中用户显式修改为准，并在文档元数据处提示「源文件 frontmatter 已变化」，提供一键「采用源文件值」（REQ-1.8） |
| 17 | `type` 值为未登记词表项 | 正常导入，归入「未登记」并提示登记入口，不阻塞任何流程（REQ-1.8.3） |
| 18 | `log.md` 开关开启后被用户手工编辑 | 磁盘权威原则下接受用户编辑，不回滚；下次物化时仅追加新条目、不重写既有内容；无法安全追加（文件结构被破坏）时暂停物化并提示（REQ-1.9） |

---

## 8. 埋点（opt-in，对齐主 PRD §6）

主 PRD §6 未定义 F1 专属事件，以下为细化新增（标注：需回填主 PRD 埋点表），服务激活漏斗（`first_loop_closed` 的前置）：

| 事件 | 属性 | 用途 |
|---|---|---|
| `project_created` | doc_count、html_count、skipped_count、scan_ms、glob_customized(bool) | 激活漏斗第一步；AC-1a 线上验证 |
| `import_completed` | duration_ms、converted_count、attachment_count | 导入性能与转换质量监控 |
| `doc_sync_batch` | files、origin(watcher/bulk/reconcile)、debounce_merged_events | AC-1b 合并效果；watcher 健康度 |
| `doc_identity_migrated` | confidence、method(hash/similarity)、manual(bool) | 重命名追踪成功率（支撑 AC-1c） |
| `gitignore_prompt_action` | action(accept/skip) | `.prismdocs/` 协议采纳率 |
| `import_error` | reason(permission/encoding/oversize/convert_fail/symlink) | 异常分布 |

---

## 9. 验收标准与测试要点

继承主 PRD AC-1a/1b/1c，细化为：

- AC-1a-1：含 20 份 .md（总 ≤2MB）的仓库，从确认导入到全部可浏览 ≤60s；其中前 5 份 ≤10s 可读（渐进可用）。
- AC-1a-2：500 文档规模导入不崩溃、UI 不冻结（对齐 §5 性能规模）。
- AC-1b-1：脚本模拟 agent 在 30s 内对 5 个文件各写盘 5 次：每文件恰好 1 条变更记录，全部内容在最后一次写盘后 ≤10s 呈现。
- AC-1b-2：`git checkout` 切换涉及 30 个文件：Inbox 仅 1 条汇总通知。
- AC-1c-1：`git mv` 改名后，该文档全部评论保留且锚定正确（100%）。
- AC-1c-2：改名且同时修改 ≤15% 内容：识别为同一文档；识别失败时必须走「疑似移动」显式提示，0 静默丢失。
- AC-1d-1（新）：HTML 三类样本集（静态报告/富样式页/脚本渲染页）分级正确率 ≥90%，降级文件无一进入 Lens 管线。
- AC-1e-1（新）：应用内编辑保存不产生自触发变更记录；保存时外部已改动的文件 100% 触发漂移提示。
- AC-1f-1（新）：快照总量达到上限后继续写入，被评论锚定的版本 0 淘汰。
- AC-1g-1（v0.2 新增，frontmatter 解析；对应主 PRD 层面的 AC-1d）：含 frontmatter 的样本文件导入后，六个标准字段的元数据在文档树/搜索中可筛选；type 未登记项正确标注。
- AC-1g-2（v0.2 新增，往返一致性）：对含 frontmatter 的文件在应用内编辑正文并保存，写回后 `git diff` 中 frontmatter 部分 0 变化（不重排、不改写）。
- 测试要点：watcher 平台差异（FSEvents 事件粗粒度）、编辑器原子保存模式（写临时文件+rename）、大小写不敏感 FS、断电/强杀后的对账恢复、符号链接环。

---

## 10. 依赖与开放问题

**依赖**：F2 消费变更事件的接口约定（变更 Block 集合格式）；F3 的 Block 锚定迁移算法（主 PRD §2.4，属跨功能技术设计）；`.prismdocs/README.md` 的英文协议文案由 F4 定稿、F1 负责落盘时机；macOS 沙盒/文件访问授权方案（安全书签持久化）。

**开放问题**

| # | 问题 | 说明与倾向 |
|---|---|---|
| OQ-1 | HTML 的默认 glob 范围主 PRD 未定义（REQ-1.2.2 为细化默认值） | 倾向默认仅 `docs/` 与根目录，避免扫到构建产物 HTML；需回填主 PRD |
| OQ-2 | REQ-1.NEW-2 的快照上限默认值（50 版/500MB）无上游依据 | 内测观察 agent 写盘频率后调整；需回填主 PRD |
| OQ-3 | 主 PRD 称 Document 与磁盘「双向同步」，但 Lens 不可编辑 + 磁盘权威下，PrismDocs→磁盘方向仅 REQ-1.6 应用内编辑一种。「双向」措辞是否会被误解为产品会主动改写用户文件？ | 倾向文案统一为「磁盘权威 + 可选写回」；不改设计，仅澄清表述 |
| OQ-4 | 非 git 项目是否也提示忽略 `.prismdocs/`（无 .gitignore 可写） | 倾向不提示（REQ-1.1.4 现行为），等待确认 |
| OQ-5 | 根目录 `*.md` 是否含 README.md？README 常为面向人的仓库门面而非 AI 工程文档 | 倾向默认包含（符合「AI 写的 .md 都进来」直觉），用户可排除 |

---

## 11. 变更记录

| 版本 | 日期 | 变更 |
|---|---|---|
| v0.1 | 2026-07-26 | 初稿 |
| v0.2 | 2026-07-26 | 同步主 PRD v0.2 OKF 兼容变更：细化 REQ-1.8 frontmatter 解析（REQ-1.8.1～1.8.5）与 REQ-1.9 log.md 物化（REQ-1.9.1～1.9.4）；新增 REQ-1.2.6 保留文件名（index.md/log.md）处理；边界情况表新增 #16–18（frontmatter/sidecar 冲突、未登记 type、log.md 手工编辑）；验收标准新增 AC-1g-1/1g-2（frontmatter 解析与往返一致性）；§6 数据模型补充元数据字段的 sidecar 存储说明 |
