# PRD-F6：Chrome 剪藏插件（PrismDocs MVP 子 PRD）

| 项目 | 内容 |
|---|---|
| 文档类型 | 子 PRD（Feature PRD）— F6 Chrome 剪藏插件 **（v0.3 决议：P1，MVP 不交付）** |
| 版本 | v0.2（标注 P1 决议；关闭 OQ-1 同步通道） |
| 日期 | 2026-07-28 |
| 上游文档 | 主 PRD《PrismDocs MVP 主产品需求文档》v0.3 §3-F6（关联 §3-F5、§3-F7、§5）；BRD v0.3 §6.4、§12-M3 |
| 作者 | Shean（起草协作：Claude） |

---

> **v0.2 状态说明**：主 PRD v0.3 决议 F6 整体移出 MVP、P1 第一批交付（BRD M3 的 Chrome 商店引流随之后置）。本文档全部需求保留不变，供 P1 排期直接使用；仅同步两处：① 同步通道开放决策 OQ-1 已定案（见 §4-REQ-6.5 与 §10）；② 与 F5/F7 的接口在 MVP 内不激活。

---

## 1. 功能概述与目标

Chrome 剪藏插件（MV3）是 PrismDocs 知识库的**外部知识入口**：把网页（Stack Overflow、GitHub、技术博客、MDN 类文档站）抓取为净化后的 Markdown **Clip（剪藏）**，显示 token 估算，存入本地知识库并可关联 **Project**。

三重定位：

1. **知识入口**：解决 BRD 痛点 #8——"为 AI 收集网页资料：HTML 噪音、手工清理、token 不透明"。Clip 是外部素材层，与 Card（理解卡片）分工明确：Clip 存原文，理解写进 Card（F5）。
2. **获客引流**：BRD §12-M3 明确"剪藏器单独上架 Chrome 商店引流"。插件须可在未深度使用桌面端时独立提供价值（净化 + token 估算即有用），成为低门槛获客入口（BRD §11 风险对策）。
3. **对北极星的间接贡献**：Clip 经 F7 打包进 Context Pack 喂给 agent，提升评论闭环质量。直接指标为 BRD §9：`clip_used_in_pack / clip_created ≥25%`。

必守产品原则（继承主 PRD，不可妥协）：

- **Clip 不可被评论**（REQ-6.7）：评论语义 = 驱动 AI 改文档，Clip 是外部素材，理解沉淀走 Card。
- **数据只在本地流转**（主 PRD §5 隐私）：插件 → 桌面端仅走本地回环通信，**不经过我方服务器**（MVP 无服务端）。
- **token 估算可见且可信**：误差 ≤10%（AC-6c）。
- **代码块净化是首要质量指标**（AC-6a，Stack Overflow 场景）。

## 2. 范围

**In Scope（P0）**：整页正文提取、选区剪藏；HTML→Markdown 净化；剪藏面板（标题/目标 Project/标签/token 估算/备注）；元数据采集；本地回环同步 + 离线暂存队列；桌面端剪藏收件箱与批量操作；`clip_created` 等埋点（opt-in）。

**In Scope（P0.5，可降级）**：手动框选元素模式（REQ-6.1）；图片可选下载。

**Out of Scope**：AI 摘要/压缩（属 F7 REQ-7.1 的"AI 压缩版"，P0.5，且在桌面端执行，插件不调用 LLM）；剪藏上的评论（REQ-6.7）；云同步/分享（开放问题 Q4，P1 前需法务审视）；Firefox/Safari；剪藏全文编辑（Clip 是素材快照，修正认知走 Card）。

**与 F5 / F7 的边界与依赖**：

| 对接功能 | 接口方向 | F 间接口约定 | 归属 |
|---|---|---|---|
| F5 理解卡片 | Clip → Card | 桌面端 Clip 阅读页选中文字"存为卡片"：选中内容进 Card 引用区（折叠、标来源链接），正文必须另写（REQ-5.2/5.4）；Card 可 `[[` 双链到 Clip（REQ-5.3） | 交互在 F5，Clip 只需提供稳定 clip_id 与锚定引用 |
| F7 Context Pack | Clip → Pack | 组装器可勾选 Clip（REQ-7.1）；Clip 的 token 数供 Pack 总量统计（AC-7b 误差 ≤10%，两端须用同一 tokenizer 口径） | F6 保证 markdown 干净紧凑 + token 元数据准确 |
| F5→F6 轻推 | 面板备注字段 | 剪藏面板"为什么剪它"备注（REQ-6.3）是原创表述的最早触点，桌面端可将其预填为 Card 草稿引子（P1 观察） | F6 |

## 3. 用户故事与关键场景

1. **整页剪 Stack Overflow**：Shean 查到一个 SQLite 并发写入的 SO 答案页，点插件图标 → 整页正文提取 → 面板显示"~3.2k tokens"，选 Project「prismdocs-app」，备注"WAL 模式下单写者即可"→ 保存。桌面端 Clips 列表即时出现，代码块可直接复制运行。
2. **选区剪藏**：一篇长博客只有中间一段基准测试有用。选中该段 → 右键菜单「剪藏选中内容到 PrismDocs」→ 面板仅含选区内容，token 从 12k 降到 800。
3. **桌面端未启动时暂存**：通勤路上用笔记本剪了 3 篇 GitHub Discussion，桌面端没开。插件提示"已离线暂存（3）"，回家启动桌面端后自动补传，剪藏收件箱出现 3 条并提示来源。
4. **剪藏后归 Project**：剪藏时没选 Project 的 Clip 落入剪藏收件箱"未归类"，周末批量勾选 5 条 → 归入对应 Project。
5. **Clip 被 Context Pack 引用**：开新一轮 agent 会话前，Shean 在 F7 组装器勾选 2 个 Clip 进 Context Pack，`clip_used_in_pack` 记录，闭环指标 +1 个引用。

## 4. 详细功能需求

### REQ-6.1 抓取模式（三种）

- REQ-6.1.1 **整页正文提取（P0，默认）**：Readability 类算法提取正文，剔除导航/侧栏/广告/评论区（站点适配可保留答案区，见 REQ-6.2.5）。提取置信度低（正文 <200 字符或文本密度异常）时不静默保存，转"失败引导选区"状态（§5）。
- REQ-6.1.2 **选区剪藏（P0）**：用户选中文字后，经右键菜单/快捷键/popup 触发，仅转换选区对应 DOM 片段。选区跨元素时取最小公共祖先内的选中部分。
- REQ-6.1.3 **手动框选元素（P0.5）**：hover 高亮 DOM 元素、点击选定（类 DevTools inspect），支持上/下键扩大缩小选择范围。降级方案：仅保留前两种模式。
- REQ-6.1.4 三种模式产物统一走 REQ-6.2 净化管线与 REQ-6.3 面板，数据结构无差别（仅 `mode` 字段不同）。

### REQ-6.2 净化转换（HTML → Markdown）

质量优先级：**代码块 > 表格 > 列表/标题 > 图片 > 其余**。

- REQ-6.2.1 **代码块（首要指标）**：`<pre><code>` 转 fenced code block；语言标注按序探测：class（`language-*`/`lang-*`/highlight.js/Prism 类名）→ 站点约定（SO 的 `s-code-block` tag 提示）→ 启发式推断，无法判定则留空不乱标；**剥离一切高亮 span/行号元素/复制按钮注入的隐藏文本**，还原纯文本源码，保留原始缩进与空行。行内 `<code>` 转反引号。
- REQ-6.2.2 **表格**：转 GFM 表格；含 rowspan/colspan 的复杂表格降级为逐行展开并在 Clip 内标注"表格结构已简化"。
- REQ-6.2.3 **图片**：默认存原始 URL（绝对化处理 lazy-load 的 `data-src`/`srcset`）；"下载图片到本地"为面板可选项（P0.5，防盗链失败见 §7）。
- REQ-6.2.4 **列表/标题/引用/链接**：保持层级；相对链接转绝对 URL；`javascript:` 等非 http(s) 链接仅保留文字。
- REQ-6.2.5 **重点适配站点清单**（主 PRD REQ-6.2 明确）及已知问题：

| 站点 | 适配要点 | 已知问题 |
|---|---|---|
| Stack Overflow | 保留问题 + 高票/已采纳答案，剥离投票栏/评论流；代码块 tag 语言提示 | 折叠的低票答案默认不抓；MathJax 公式降级为源文本 |
| GitHub README | 渲染后 DOM 提取；锚点链接绝对化 | mermaid/嵌入式 SVG 图降级为占位链接 |
| GitHub Issue/Discussion | 逐楼层保留作者/时间为引用头；隐藏折叠的 outdated 内容 | 超长线程触发懒加载，仅抓已渲染楼层（提示"可能不完整"） |
| 常见技术博客（Medium/Dev.to/个人博客） | Readability 主路径 | Medium 付费墙截断（§7）；gist 嵌入 iframe 需特殊处理（§7） |
| MDN 类文档站 | 保留兼容性表格、参数定义列表 | 交互式示例（iframe）降级为链接 |

### REQ-6.3 剪藏面板字段

保存前弹出面板：**标题**（预填 `<title>`/og:title，可改）；**目标 Project**（下拉，默认"上次选择"，可选"暂不归类"→ 入剪藏收件箱未归类）；**标签**（自由输入 + 历史联想）；**token 估算**（只读、醒目，误差 ≤10%，超长预警见 §7）；**备注**（placeholder：「一句话：为什么剪它？」，可跳过——轻推原创表述，与 F5 引导一致）。默认焦点在备注框但回车即可直接保存（不强迫）。

### REQ-6.4 元数据（自动采集，用户不可改）

URL（规范化：去 utm 等跟踪参数——**REQ-6.NEW-1，主 PRD 未覆盖，需回填**：URL 规范化规则用于重复检测口径）、站点名（域名 + og:site_name）、抓取时间、原文语言（franc 类检测）、抓取模式、token 数、插件版本。

### REQ-6.5 同步机制（本地回环，开放决策）

两个候选方案取舍分析：

| 维度 | Native Messaging | 本地端口（localhost HTTP/WS） |
|---|---|---|
| 隐私叙事 | 最强：进程间 stdio，无网络面 | 仍是回环，但需绑 127.0.0.1 + token 鉴权防本机其他进程/网页伪造请求 |
| 安装成本 | 需桌面端注册 native host manifest（安装器写注册表/plist） | 零额外安装，桌面端启动即监听 |
| 桌面端未启动 | 可由浏览器拉起 host 进程（但拉起整个桌面 app 不合理） | 连接失败即知未启动，逻辑简单 |
| 消息体积 | Chrome 单条 ≤1MB（native→extension 方向 ≤64MB），长文需分片 | 无实际限制 |
| 跨浏览器（Edge 顺带） | 每个浏览器单独注册 manifest | 天然通用 |
| 故障排查 | host 崩溃静默、日志难拿 | 标准 HTTP，易调试 |

**已定案（v0.2，关闭 OQ-1）**：本地端口方案——桌面端 axum 宿主上的 **loopback WebSocket 端点 + 桌面端生成的配对 token**（首次配对经用户在插件内一键确认），与 MCP server 共用同一 HTTP 宿主进程。零额外安装契合"引流插件独立分发"（BRD M3）、规避 Native Messaging 的 1MB 分片复杂度、天然覆盖 Edge；满足主 PRD §5"剪藏与文档不经过我方服务器"。

### REQ-6.6 暂存队列（细化主 PRD REQ-6.5 后半句）

- REQ-6.6.1 桌面端不可达时，Clip 完整落入插件本地存储（`chrome.storage.local` 或扩展 IndexedDB），面板正常可用，保存后提示"已离线暂存（n）"。
- REQ-6.6.2 **容量（REQ-6.NEW-2，主 PRD 未覆盖，需回填）**：暂存上限 50 条或 8MB（先到为准，留足 `chrome.storage.local` 10MB 配额余量）；达上限后阻止新剪藏并引导启动桌面端，**不静默丢弃**。
- REQ-6.6.3 补传：桌面端可达后按时间序自动补传，逐条确认落库后才删除本地副本；补传失败条目保留并可手动重试。剪藏收件箱对补传批次给出来源提示。

### REQ-6.7 剪藏收件箱（桌面端，细化主 PRD REQ-6.6）

- REQ-6.7.1 主导航「剪藏（Clips）」= 剪藏收件箱：默认按时间倒序，未归类 Clip 置顶分组；支持按 Project/站点/标签筛选与全文搜索（纳入主 PRD §5 性能预算：搜索 <300ms）。
- REQ-6.7.2 批量操作：多选 → 归 Project / 归档 / 删除。删除前确认；被 Card 或 Context Pack 引用的 Clip 删除时警示引用数，删除后引用方按 F5 边界规则转快照（REQ-5.2 边界）。
- REQ-6.7.3 Clip 详情页：净化后 Markdown 渲染 + 元数据 + "查看原文"外链；**无评论入口**（REQ-6.7 产品原则）；提供"存为卡片"（走 F5）与"加入 Context Pack"（走 F7）两个动作。
- REQ-6.7.4 重复 URL（规范化后同 URL）再剪 → 面板内提示"已存在（时间）"，可选「存为新版本」（同 URL 多版本并列，详情页可切换）或「打开已有 Clip」。

## 5. 交互与界面规格

**入口**：工具栏图标（popup）；右键菜单「剪藏本页 / 剪藏选中内容」（**REQ-6.NEW-3，主 PRD 未覆盖，需回填**）；快捷键默认 `Alt+Shift+S`（可在 Chrome 快捷键页改）。

**状态机**：`空闲 → 抓取中（图标 spinner，≤3s 预期）→ 面板确认 → 保存中 → 成功（toast + token 数）`；分支：`抓取失败/低置信 → 引导选区`；`桌面端不可达 → 离线暂存`。

剪藏面板（popup 内）：

```
┌──────────────────────────────────────┐
│ ✂ PrismDocs 剪藏          [整页 ▾]    │
│ 标题  [SQLite write concurrency…  ]  │
│ Project [prismdocs-app        ▾]      │
│ 标签  [sqlite] [并发] [+]            │
│ ── 预览（前 3 段）────────────────    │
│ | ```sql … ``` 代码块 ✓ 已净化       │
│ 📏 约 3,240 tokens                   │
│ 备注 [一句话：为什么剪它？(可跳过)]   │
│           [取消]   [保存 ↵]          │
└──────────────────────────────────────┘
```

失败引导选区：

```
┌──────────────────────────────────────┐
│ ⚠ 未能可靠提取本页正文               │
│ 该页面可能是动态应用或结构特殊。      │
│ [框选元素(P0.5)] [选中文字后重试]     │
│ [仍然保存原始抓取结果]               │
└──────────────────────────────────────┘
```

离线暂存：toast「桌面端未运行，已暂存（3/50）。启动 PrismDocs 后自动同步。」图标角标显示暂存数。

桌面端剪藏收件箱：左列表（未归类分组置顶、多选框），右详情（Markdown 渲染 + 元数据侧栏 + 动作条「存为卡片 | 加入 Context Pack | 归档」）。

## 6. 数据模型与技术要点（供技术设计参考）

`clip` 表（桌面端 SQLite，主 PRD §5 本地优先）：`id, project_id(nullable), title, content_md, source_url, canonical_url, site_name, lang, mode(full|selection|element), token_count, note, tags(json), clipped_at, synced_at, version_group_id, archived(bool), created_by_plugin_version`。引用关系复用主 PRD §2.2：Card 双链、Context Pack 勾选均指向 `clip.id`。

MV3 架构要点：**service worker**（生命周期短，同步/暂存逻辑须无状态可恢复，队列落 storage 而非内存）；**content script**（DOM 抓取与净化在页面侧执行，只回传净化后 Markdown + 元数据，最小化消息体积）；**offscreen document**（若 tokenizer/Turndown 需 DOM 或较重计算，置于 offscreen 规避 service worker 限制）。净化管线 = Readability 类提取 + Turndown 类转换，定制点：代码块规则（REQ-6.2.1 的语言探测与 span 剥离）、GFM 表格插件、lazy-load 图片 URL 还原、站点 adapter 层（REQ-6.2.5 清单可配置扩展）。token 估算：插件内嵌与桌面端/F7 相同口径的 tokenizer（建议 tiktoken 类 BPE 的 JS 实现）保证 AC-6c 与 AC-7b 一致。

## 7. 边界情况与异常处理

| # | 场景 | 处理 |
|---|---|---|
| 1 | 付费墙/登录墙（Medium 等） | 抓到什么存什么，检测截断特征（"Sign in to read"类标记）→ Clip 标注「可能不完整」（主 PRD 边界，继承） |
| 2 | SPA 动态内容 | 以触发时当前 DOM 为准；提取失败/低置信 → 引导选区（主 PRD 边界，继承） |
| 3 | 超长页面（>50k token） | 面板红色预警 + 建议「仅剪选区」，仍允许强制保存（主 PRD 边界，继承） |
| 4 | 重复 URL | 规范化比对 → 提示已存在，可存为新版本（REQ-6.7.4） |
| 5 | iframe 内容（gist 嵌入、交互示例） | 同源 iframe 尝试并入；跨源 iframe 无法读取 → 降级为「嵌入内容：<url>」占位链接 |
| 6 | Shadow DOM（调研提及 Reddit 等站点） | content script 递归遍历 open shadow root 并入提取；closed shadow root 不可达 → 计入低置信判定，引导选区 |
| 7 | 图片防盗链（Referer 校验） | 默认仅存 URL 不受影响；P0.5 下载失败的图片保留 URL 并标「下载失败」，不阻塞保存 |
| 8 | 桌面端保存中途崩溃/断连 | 插件未收到落库确认 → 转入暂存队列重试，靠确认后删除保证不丢（REQ-6.6.3） |
| 9 | `chrome://`、商店页等受限页面 | content script 无法注入 → popup 明示「此页面不支持剪藏」 |
| 10 | 正文含 PrismDocs 语法冲突（如 ``` 嵌套） | 转义/加长 fence，保证桌面端 Markdown 解析不破坏 |

## 8. 埋点（opt-in，继承主 PRD §6）

| 事件 | 属性 | 对应指标 |
|---|---|---|
| `clip_created` | site_domain（仅域名，不含路径/参数——隐私最小化）、mode、token_count、has_note(bool)、project_assigned(bool)、via_offline_queue(bool) | 剪藏量基线；备注率观察原创轻推效果 |
| `clip_used_in_pack` | clip_id 哈希、pack 内占比 | 北极星护航：引用率 ≥25%（BRD §9） |
| `clip_extract_failed`（REQ-6.NEW-4，主 PRD 未覆盖，需回填） | site_domain、fail_reason(low_confidence/restricted/timeout) | 站点适配优先级排序 |
| `clip_saved_as_card` | — | F5/F6 协同：素材→理解转化率 |

埋点数据同样不经过我方服务器落地前不上传（MVP 无服务端：本地暂存，内测期经授权导出）。

## 9. 验收标准与测试要点

细化主 PRD AC-6a/6b/6c：

- **AC-6a-1**：从 SO 高票答案页样本集（≥20 页，覆盖 JS/Python/SQL/Shell/多代码块混排）剪藏，代码块 100% 无 span 噪音、无行号残留、缩进与空行与原文一致，可直接复制运行。
- **AC-6a-2**：代码块语言标注正确率 ≥90%（样本集内），无法判定时留空、0 错标为无关语言。
- **AC-6b-1**：测试站点清单成功率 ≥95%：GitHub（README×10、Issue×5、Discussion×5）、技术博客（Medium、Dev.to、个人 Hugo/Hexo 博客各 5）、MDN 页面 ×10。"成功" = 正文完整、结构（标题/列表/表格）保持、无导航噪音。
- **AC-6b-2**：含 open Shadow DOM 与懒加载图片的页面各 ≥3 例纳入样本，处理符合 §7 规则。
- **AC-6c-1**：token 估算与桌面端实际 tokenizer 计数误差 ≤10%（样本 ≥30 条，覆盖中英混排与代码密集型 Clip），且与 F7 Pack 总数口径一致。
- **AC-6d-1**（新增，对应 REQ-6.6）：桌面端关闭状态连续剪藏 10 条 → 启动后 30 秒内全部补传成功，0 丢失、0 重复。
- **AC-6e-1**（新增，对应 REQ-6.7.2）：收件箱批量归 Project 50 条 ≤3 次交互完成。

## 10. 依赖与开放问题

**依赖**：桌面端本地服务与 clip 落库（F1 基础设施）；F5 的"存为卡片"交互；F7 组装器对 Clip 的勾选与 token 汇总；统一 tokenizer 口径（与 F7/主 PRD §5 成本可见共用）；BRD M3 商店上架材料（主 PRD §9 后续文档）。

**开放问题**：

| # | 问题 | 倾向 |
|---|---|---|
| OQ-1 | ~~同步通道：native messaging vs 本地端口~~ | **已关闭（v0.2）**：loopback WebSocket + 配对 token，挂在桌面端 axum 宿主上（见 REQ-6.5） |
| OQ-2 | 插件独立分发（BRD M3 引流）时未连接桌面端的形态：暂存队列可否长期充当"轻量收藏夹"并支持导出 .md？ | 倾向支持导出（引流价值），但须避免演变为独立产品；与增长策略一起决策 |
| OQ-3 | 剪藏整页存储的版权边界 | 继承主 PRD Q4：个人本地使用属合理范围；任何云同步/分享前法务审视 |
| OQ-4 | token 估算的目标 tokenizer 与用户实际所用模型（Anthropic/OpenAI 兼容端点，主 PRD §1.3）不一致时如何标注？ | 倾向固定单一口径并在 UI 注明"按 XX tokenizer 估算"；与 F7 联合决策 |
| OQ-5 | 站点 adapter 清单（REQ-6.2.5）是否开放用户自定义规则？ | MVP 内置固定清单；自定义后置 P1 |

## 11. 变更记录

| 版本 | 日期 | 变更 |
|---|---|---|
| v0.1 | 2026-07-26 | 初稿 |
| v0.2 | 2026-07-28 | 同步主 PRD v0.3：整体标注 P1（MVP 不交付，需求全文保留供 P1 排期）；REQ-6.5/OQ-1 同步通道定案为 loopback WebSocket + 配对 token（与 MCP 共用 axum 宿主） |
