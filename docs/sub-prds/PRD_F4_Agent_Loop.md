# PRD-F4：评论回流 AI（Comment-to-Agent Loop）（PrismDocs MVP 子 PRD）

| 项目 | 内容 |
|---|---|
| 文档类型 | 子 PRD — F4 评论回流 AI ★ 核心闭环 |
| 版本 | v0.2（同步主 PRD v0.2：Agent 贡献溯源 + OKF 协议声明） |
| 日期 | 2026-07-26 |
| 上游文档 | 主 PRD v0.2 §3-F4、§4 Agent 集成协议（整节细化）、§3-F3（上游评论）、§6 埋点；BRD v0.2 §6.2 |
| 作者 | Shean（起草协作：Claude） |

---

## 1. 功能概述与目标

F4 把 F3 产生的评论打包为结构化 **Feedback Bundle**，通过**文件协议 + 本地 MCP Server 双通道**交付给编码 agent（Claude Code / Cursor 等）；agent 修改 Base 层文档并回执后，产品把评论推进到 needs-review，用户复核 resolve 即完成一个闭环。**北极星指标「闭环数/周」的每一次计数都发生在本功能内**——F4 是产品价值假设的直接载体。

设计原则（继承 BRD/主 PRD，不可违背）：
1. 回流「少而准」：Bundle 只含评论 + 被评 Block 原文 + 必要上下文，token ≤ 被评文档全文的 30%（AC-4b）。
2. 双通道：文件协议是 P0 兜底，MCP 是 P0 增强；**未装 MCP 也必须能闭环**（靠文件变更检测兜底回收，AC-4c）。
3. MCP 仅本地回环，写操作限于评论回执；**磁盘文件是 Base 层的权威副本**，F4 自身从不改用户文档。

### 闭环状态流转图（Comment 状态机，继承 REQ-3.3）

```
                    ┌────────────────────────────────────────────┐
                    │                (reopen，可追评)             │
                    ▼                                            │
 [open] ──用户点「回流」──> [sent] ──agent 回执(MCP respond_to_comment)
   ▲                          │      或 Base 变更命中被评 Block──> [needs-review]
   │                          │                                      │
   │                          └─48h 无响应→提醒(状态不变，可重发)      │ 用户复核
   │                                                                ├─通过──> [resolved] ✓ loop_closed
   └────────────── (新评论) ────────────────────────────────────────┴─不通过─> [reopened] ──再次回流─> [sent]
```

resolved 且此前经过 sent/needs-review → 上报 `loop_closed`（§9）。

---

## 2. 范围

**In Scope**：Bundle 生成与格式；回流面板交互；文件协议交付 + 剪贴板指令；本地 MCP Server 全部工具面（含 F7 的 `get_context_pack`、F5 的 `list_cards`——工具宿主在 F4，内容由对应功能供给）；Claude Code hook/skill 一键安装；回执匹配与文件变更兜底检测；needs-review 复核界面；Bundle 历史；48h 无响应提醒；`.prismdocs/` 目录与 CLAUDE.md 协议文案。

**Out of Scope**：评论创建与锚点迁移（F3）；Lens 重投影本身（F2，F4 只触发）；Context Pack 组装 UI（F7）；agent 侧行为质量（不控 agent 是否改得好，只保证输入结构化、输出可复核）；多人协作回流、GitHub PR 集成（P1）。

**与相邻功能的接口**

| 相邻功能 | 方向 | 接口内容 |
|---|---|---|
| F3 段落级评论 | F3 → F4 | 提供 open 评论集合（类型/quote/线程/Block 锚点）；F4 回写状态 sent/needs-review |
| F2 Lens 生成 | F4 → F2 | Base 变更兜底检测确认命中后，触发受影响 Block 增量重投影与变更高亮 |
| F1 文件同步 | F1 → F4 | FS watcher 的 Base 变更事件（防抖 2s 后）是兜底回收的信号源 |
| F7 上下文组装 | F7 → F4 | `get_context_pack` 由 F4 的 MCP Server 暴露，内容由 F7 生成 |
| F5 理解卡片 | F5 → F4 | `list_cards`（P0.5）暴露 context-worthy 卡片；resolve 时的「写卡片」入口跳转 F5 |
| Inbox（§2.3） | F4 → Inbox | needs-review 事项、48h 提醒推入 Inbox |

---

## 3. 用户故事与关键场景

**S1 · 全自动路径（Claude Code + MCP + hook，一级支持）**
Shean 在 Lens 上写完 3 条评论，点「发给 AI」。PrismDocs 写入 Bundle 并经 MCP 可见；Claude Code 的 hook 在下次会话提示 agent 有未处理反馈，agent 调 `get_feedback` 逐条处理、改 Base、逐条 `respond_to_comment` 回执。PrismDocs 收到回执 + 检测到文件变更 → 评论转 needs-review，Inbox 弹出「AI 已修改 2 处待复核」。Shean 在 diff+评论并排视图复核，resolve。全程除在 Claude Code 里说一句话（或 hook 自动）外零手工拷贝。

**S2 · 纯文件协议降级路径（未装 MCP，AC-4c）**
Cursor 用户小李点「回流」，产品写入 `.prismdocs/feedback/2026-07-26T1030-a1b2.md` 并复制一句话指令到剪贴板；他粘贴到 Cursor 对话框。agent 读文件、改 Base、无任何回执。PrismDocs 的 FS watcher 检测到被评 Block 所在文件变更且 diff 命中该 Block → 评论自动转 needs-review。闭环达成，只是「未命中」判定精度低于 MCP 路径。

**S3 · agent 部分处理**
5 条评论回流，agent 只处理了 3 条（回执 done×2、declined×1，其余 2 条既无回执文件也未变更）。命中的 3 条转 needs-review（declined 附 agent 说明，供用户决定 resolve 或 reopen）；2 条保持 sent，48 小时后 Inbox 提示「AI 似乎没有处理这 2 条」，提供「重新回流」按钮。

**S4 · 提问类评论问答（REQ-4.5）**
Shean 在「为什么选 SQLite」段落发了一条 💬 提问。agent 不改文档，调 `respond_to_comment(id, "answered", "SQLite is fine because all writes go through a single queue...")`，答复以 agent 身份显示在评论线程内、状态转 needs-review；Shean 看完 resolve（同样计入闭环），并顺手从线程入口写了张卡片。

**S5 · Reject 决策回流**
Shean 对一段方案点 ❌ Reject 附言「换方案」。Bundle 中该条被明确标注 decision=reject，指令头告知 agent：reject 意味着推倒重来、**须先在文档中提出新方案而非直接改代码**。agent 在 Base 中新增备选方案节，评论转 needs-review 供拍板。

---

## 4. 详细功能需求

### REQ-4.1 Feedback Bundle 生成

- **REQ-4.1.1** 入口：文档级（评论侧栏顶部「回流本文档评论」）与项目级（Inbox /文档树顶部「回流全部」）。打开回流面板（§6），默认勾选全部 open 评论；sent/needs-review 状态评论不可勾选（防重复，见 REQ-4.NEW-2 重发例外）。
- **REQ-4.1.2** 每条评论在 Bundle 中必含：目标文件相对路径、Block 定位（标题路径 + Base 原文摘录 quote）、评论类型、用户原文（中文）、**英文意图摘要**（REQ-4.1.4）、线程上下文（有回复时按时间序附带）、comment_id（回执匹配键）。
- **REQ-4.1.3** 格式 = YAML frontmatter + 人类可读 Markdown 正文，**完整格式示例**：

````markdown
---
prismdocs_bundle: v1
bundle_id: fb-2026-07-26T1030-a1b2
project: my-saas-app
created_at: 2026-07-26T10:30:00+08:00
comment_count: 3
documents: ["docs/architecture.md"]
respond_via: "MCP tool respond_to_comment, or edit the Receipt section below"
---

# PrismDocs Feedback — 3 comments on 1 document

<!-- INSTRUCTION HEADER: see full text in REQ-4.1.5 -->
You are receiving structured review feedback from the human reviewer...

## [c-101] ✏️ CHANGE REQUEST — docs/architecture.md
**Location**: `## Data Layer > ### Why SQLite`
**Quoted base text**:
> SQLite is chosen for its simplicity and zero-ops deployment.
**Reviewer comment (zh)**: 并发写入会不会有问题？如果用户超过 1 万怎么办？请补充分析和迁移预案。
**Intent (en)**: Analyze SQLite concurrent-write limits; add a >10k-users migration plan to this section.

## [c-102] 💬 QUESTION — docs/architecture.md
**Location**: `## Sync > ### Debounce`
**Quoted base text**:
> File events are debounced by 2 seconds.
**Reviewer comment (zh)**: 2 秒是怎么定的？agent 连续写盘会不会漏事件？
**Intent (en)**: Question only — explain the 2s debounce rationale; do NOT edit the document. Answer via respond_to_comment.

## [c-103] ❌ REJECT — docs/architecture.md
**Location**: `## Auth > ### Session Strategy`
**Reviewer comment (zh)**: 不要用自签 JWT，换托管方案。
**Intent (en)**: DECISION: rejected. Do not implement. Propose an alternative (managed auth) IN THE DOCUMENT first, then wait for re-review.

## Receipt (file-protocol fallback)
<!-- If MCP is unavailable, mark each line: [x] done | answered: <text> | declined: <reason> -->
- [ ] c-101
- [ ] c-102
- [ ] c-103
````

- **REQ-4.1.4** 英文意图摘要生成要求：由产品用快速档模型生成，1–2 句祈使句、面向 agent 可执行；必须保真不加戏（不得引入用户未提的要求）；提问类须显式含 "do NOT edit the document"；决策类须显式含 approve/reject 语义；生成后在回流面板可见、可手动改（改动不回写评论原文）；LLM 不可用时降级为机翻或原文直传并标注 `Intent (en): (auto-summary unavailable, see zh above)`，不阻塞回流。
- **REQ-4.1.5** 执行指令头完整英文文案（草案，写死在 Bundle 头部）：

> You are receiving structured review feedback from the human reviewer of this project, exported by PrismDocs. Process EVERY item below, one by one. For ✏️ CHANGE REQUEST items: edit the referenced file at the referenced section only; do not modify unrelated parts of any document. For 💬 QUESTION items: answer via the `respond_to_comment` MCP tool (or the Receipt section at the bottom); do not edit documents. For ✅ APPROVE items: no edit needed; acknowledge with a receipt. For ❌ REJECT items: the reviewer has rejected this approach — propose an alternative in the document first; do not implement the rejected approach. After handling each item, send a receipt via `respond_to_comment(comment_id, action, note)` with action = done / answered / declined; if MCP is unavailable, fill in the Receipt checklist at the bottom of this file instead. Keep documents in English and compact.

- **REQ-4.1.6** 剪贴板一句话指令文案（回流成功即复制，模板）：
  `Process the PrismDocs feedback in .prismdocs/feedback/<filename>.md — handle every item and leave receipts as instructed in the file.`

### REQ-4.2 双通道交付

- **REQ-4.2.1** 文件协议（P0 兜底）：Bundle 写入 `.prismdocs/feedback/<timestamp>-<shortid>.md`；文件名可字典序排序；写入采用临时文件 + rename 原子落盘（防 agent 读到半截）。
- **REQ-4.2.2** MCP Server（P0）：随桌面端启动的本地 server，工具面与 schema 见 §5.2；传输方式与端口策略见 REQ-4.NEW-1。
- **REQ-4.2.3** 双通道内容一致：MCP `get_feedback` 返回的正文与文件字节一致（同一生成物），frontmatter 即结构化元数据来源。
- **REQ-4.2.4** Claude Code hook/skill 一键安装引导（细化见 §5.3）。

### REQ-4.3 回流范围控制（token 约束）

- **REQ-4.3.1** Bundle 内容白名单：指令头 + 每条评论的定位/quote/原文/意图摘要/线程。quote 截断规则：单 Block 超过 120 词时保留首尾各 60 词并以 `[...]` 标记（agent 可自行读全文件）。
- **REQ-4.3.2** 回流面板实时显示 Bundle 预估 token 与「被评文档全文 token」的比值；超 30% 时黄色警示并提示缩减勾选（不硬阻止——AC-4b 面向典型场景）。
- **REQ-4.3.3** 「附带全文」手动开关：默认关；开启后逐文档确认，全文以附录节加入并计入 token 显示。

### REQ-4.4 闭环回收（双信号）

- **REQ-4.4.1** MCP 回执信号：收到合法 `respond_to_comment` → 对应评论立即转 needs-review，记录 action/note/时间。
- **REQ-4.4.2** 文件回执信号（降级）：watcher 检测到 Bundle 文件自身被修改 → 解析 Receipt 勾选区，等效回执。
- **REQ-4.4.3** 文件变更兜底信号：Base 文档变更（F1 事件）且 diff 命中被评 Block（算法见 §7.3）→ sent 评论转 needs-review，标注「由文件变更检测触发（无 agent 回执）」。三种信号幂等合并，先到先转，后到补充信息。
- **REQ-4.4.4** 转 needs-review 时：触发 F2 增量重投影；推 Inbox 通知（聚合同一 Bundle 的多条）。
- **REQ-4.4.5** 复核界面（§6.3）中 resolve → 上报 `loop_closed`，并按 REQ-5.4 提示写卡片；reopen → 状态 reopened，可追评后再次回流。

### REQ-4.5 提问类评论的回答

- **REQ-4.5.1** `respond_to_comment(action="answered", note=...)` 的 note 以「agent 回复」样式（带 agent 图标与 bundle 来源）插入评论线程；note 上限 2000 字符，超长截断并提示查看 Bundle 历史详情。
- **REQ-4.5.2** 提问类评论若 agent 误改了文档（兜底信号命中）同样转 needs-review，复核界面提示「此条为提问，但检测到文档变更」。

### REQ-4.6 Bundle 历史

- **REQ-4.6.1** 每个 Bundle 留档：包含评论清单、创建时间、交付方式、逐条回执状态（无/done/answered/declined/文件变更命中）、关联的 Base 变更摘要。
- **REQ-4.6.2** 历史列表入口：项目侧栏「回流历史」；每项可展开查看 Bundle 原文（只读）。
- **REQ-4.6.3** Bundle 文件保留策略：默认保留最近 20 个，更早的仅归档进本地库、删除磁盘文件（`.prismdocs/` 保持干净）；设置可调。

### REQ-4.7 Agent 贡献溯源（主 PRD v0.2 新增，P0）

继承主 PRD REQ-4.7 与 CoWiki 的问题意识（防止来源不明的错误内容进入共享上下文后被 agent 放大），对每次命中被评 Block 的 Base 变更记录并展示触发来源。

- **REQ-4.7.1 变更来源分类**：每条 Base 变更记录归入三类之一：
  - `feedback-triggered`：由回流触发（关联 Bundle ID + 评论 ID）；
  - `mcp-attributed`：由 MCP 回执显式归因（记录回执方标识，即 MCP client 信息中的 agent_kind）；
  - `external-unknown`：watcher 检测到变更但无法归因（无回执、无时间窗内关联 Bundle，如手工编辑、git pull、未接入协议的 agent）。
- **REQ-4.7.2 归因方法**：优先级顺序——① MCP 回执显式关联（`respond_to_comment` 携带的 comment_id 反查 Bundle）；② 启发式归因：Bundle 发出后时间窗内命中被评 Block 的变更，归为 `feedback-triggered` 但标注为**推断（inferred）**，UI 中与显式归因区分显示（如「推断」灰色徽标）；③ 均不满足 → `external-unknown`。
- **REQ-4.7.3 复核界面展示**：needs-review 复核界面（§6.3）顶部展示溯源信息：「本次修改由 <来源> 因 <评论摘要> 触发」；来源为推断时附「（推断）」标注，`external-unknown` 时按 REQ-4.7.5 提示。
- **REQ-4.7.4 数据结构预留**：每条变更记录持久化 `actor`（agent_kind / user / unknown）、`trigger`（bundle_id + comment_id / 空）、`timestamp`、`confidence`（explicit / inferred / unknown）四字段，为 OKF v0.2 provenance 方向预留（届时可物化导出，本版不导出）。
- **REQ-4.7.5 external-unknown 特殊提示**：`external-unknown` 变更命中被评论 Block 时，复核界面显著提示「此修改并非来自你的评论回流，请留意」——防止来源不明内容被误 resolve 后进入上下文传播。

### 新增需求（主 PRD 未覆盖，需回填）

- **REQ-4.NEW-1**（回填 §4.2）：MCP 传输与冲突策略——首选 stdio（由 agent 配置以命令方式拉起轻量代理连接桌面端），兼容本地回环 HTTP/SSE；HTTP 端口默认 127.0.0.1:23816，占用时自动递增并把实际端口写入 `.prismdocs/mcp.json` 供代理发现；多项目共用单一 server，按调用方项目根路径路由数据（见 §8）。
- **REQ-4.NEW-2**（回填 §3-F4）：「重新回流」——48h 无响应或用户主动时，允许把 sent 评论重新打入新 Bundle；新 Bundle frontmatter 带 `resend_of: <bundle_id>`，指令头追加一句 "Some items were re-sent because no response was recorded."。
- **REQ-4.NEW-3**（回填 §3-F4）：Bundle 撤回——发出后未收到任何信号前可「撤回」：删除 feedback 文件、MCP 列表中移除、评论回退 open。已产生任一回执信号后不可撤回。

---

## 5. Agent 集成协议细化（主 PRD §4 整节）

### 5.1 `.prismdocs/` 目录结构与 README

```
.prismdocs/
  feedback/                      # F4 Bundle；agent 读，可在文件内 Receipt 区回执
    2026-07-26T1030-a1b2.md
  context/                       # F7 Context Pack；agent 只读
  mcp.json                       # server 发现信息（端口/版本），产品维护
  README.md                      # 英文协议说明，产品生成与更新
```

项目初始化时提示将 `.prismdocs/` 加入 `.gitignore`（默认建议加入；团队共享 Bundle 场景可不加）。**README.md 内容草案（英文，随产品版本更新）**：

> **This directory is managed by PrismDocs.** `feedback/` contains review feedback bundles from the human reviewer — when asked to process PrismDocs feedback, read the newest file there and follow its embedded instructions. `context/` contains curated context packs you may be asked to reference. If the PrismDocs MCP server is configured, prefer `list_feedback` / `get_feedback` / `respond_to_comment` over raw files. Never edit files under `context/`; only `feedback/*.md` Receipt sections may be edited as receipts. Outputs in this directory follow the Open Knowledge Format (OKF) v0.1 conventions. Do not delete this directory.

### 5.2 MCP 工具 schema（仅本地回环；写操作限于回执）

**`list_feedback()`** → 未完成回执的 Bundle 列表：

```json
{ "bundles": [ { "bundle_id": "fb-2026-07-26T1030-a1b2",
    "project_root": "/Users/shean/my-saas-app",
    "created_at": "2026-07-26T10:30:00+08:00",
    "comment_count": 3, "pending_count": 2,
    "documents": ["docs/architecture.md"] } ] }
```

**`get_feedback(bundle_id)`** → `{ "bundle_id": "...", "markdown": "<Bundle 全文>", "comments": [ { "comment_id": "c-101", "type": "change_request|question|approve|reject", "file": "docs/architecture.md", "heading_path": ["Data Layer","Why SQLite"], "quote": "...", "comment_zh": "...", "intent_en": "...", "status": "sent" } ] }`（缺省/非法 id → error）。

**`respond_to_comment(comment_id, action, note)`**：`action ∈ done | answered | declined`，`note` 必填（answered/declined）/选填（done），≤2000 字符。返回 `{ "ok": true, "comment_status": "needs-review" }`；对非 sent 状态评论返回 `{ "ok": false, "error": "comment not awaiting response" }`。**这是唯一写操作。**

**`get_document_comments(path)`** → 该文档当前全部评论（含状态与锚点信息），供 agent 改文档前自查上下文；path 为项目根相对路径。

**`get_context_pack(name)`** → `{ "name": "...", "markdown": "...", "token_estimate": 4210 }`（F7 供给）。

**`list_cards(filter)`**（P0.5）→ context-worthy 卡片元数据与注入行（F5 供给）。

**`export_okf_bundle(selection)`**（P1，主 PRD v0.2 新增）→ 导出 OKF bundle，同主 PRD REQ-7.6 的 MCP 形态；schema 本文档不展开，见主 PRD。

安全边界（继承 §4.2）：仅 127.0.0.1；仅暴露当前 Workspace；agent 不能创建/删除/修改评论内容与卡片；所有调用写审计日志（Bundle 历史可见「agent 何时读了什么」）。

### 5.3 Claude Code 一键安装流程

设置页「连接 Claude Code」向导，三步、每步展示将写入的片段并征求确认：

1. **MCP 注册**：生成命令供用户执行或直接代写项目级 `.mcp.json`：

```json
{ "mcpServers": { "prismdocs": {
    "command": "prismdocs-mcp", "args": ["--project", "."] } } }
```

2. **Hook（SessionStart 提醒）**：追加到 `.claude/settings.json`，会话启动时提示未处理反馈：

```json
{ "hooks": { "SessionStart": [ { "hooks": [ { "type": "command",
  "command": "prismdocs-mcp --check-feedback --project . 2>/dev/null || true" } ] } ] } }
```

（命令在有未处理 Bundle 时输出一行英文提示注入上下文，无则静默；工具不存在时 `|| true` 保证不破坏用户会话。）

3. **CLAUDE.md 协议说明**（一键追加，英文草案 ~10 行）：

```markdown
## PrismDocs Review Protocol
This project is reviewed in PrismDocs. Human feedback arrives as bundles in
`.prismdocs/feedback/*.md` and via the `prismdocs` MCP server.
- When asked to "process PrismDocs feedback" (or notified by a hook), call
  `list_feedback` then `get_feedback`, or read the newest feedback file.
- Handle every item; edit only the referenced sections of the referenced files.
- QUESTION items: answer via `respond_to_comment`, do not edit documents.
- REJECT items: propose an alternative in the document first; never implement
  a rejected approach.
- Always leave a receipt per item: `respond_to_comment(comment_id, action, note)`
  (action: done/answered/declined), or tick the Receipt list in the bundle file.
```

### 5.4 Cursor 与其他 agent 兼容

- **Cursor（MCP + 文件协议）**：向导生成 `.cursor/mcp.json`（同 5.3-1 结构）与 AGENTS.md/rules 追加片段（同 5.3-3 文案）；无 hook 机制 → 依赖剪贴板一句话指令触发，回收依赖 MCP 回执或兜底检测。
- **其他 agent（纯文件协议）**：README + Bundle 内嵌指令头自解释；Receipt 勾选区作为无 MCP 的回执通道；兜底检测保证闭环。验证矩阵维持主 PRD §4.3：Claude Code 一级、Cursor 二级、其余兜底。

---

## 6. 交互与界面规格

### 6.1 回流面板（文档级示例）

```
┌─ 回流评论 → AI ──────────────────────────────────────┐
│ docs/architecture.md · 4 条 open 评论                 │
│ [✓] ✏️ c-101 「并发写入会不会有问题…」                 │
│      Intent(en): Analyze SQLite concurrent-write… ✎  │
│ [✓] 💬 c-102 「2 秒是怎么定的…」                      │
│ [✓] ❌ c-103 「不要用自签 JWT…」                      │
│ [ ] ✏️ c-104 「这段图挂了」（已在其他 Bundle · 禁选）  │
│ ─────────────────────────────────────────────────── │
│ □ 附带全文（不推荐）    Bundle ≈ 830 tok / 全文 24%   │
│                      [取消]   [生成并发送 →]         │
└──────────────────────────────────────────────────────┘
```

发送成功态：`✅ 已写入 .prismdocs/feedback/…md · 指令已复制到剪贴板` + 「喂给 agent」引导：检测到已装 hook → 提示「下次 Claude Code 会话将自动提醒 agent」；未装 → 提示「把剪贴板指令粘贴到 agent 对话框」+「连接 Claude Code」入口。评论状态即时变 sent（灰色徽标）。

### 6.2 Bundle 历史列表

每行：时间 · 文档数/评论数 · 回执进度（如 `2/3 已回执 · 1 无响应`）· 状态色点（全回执绿 / 部分黄 / 无响应且超 48h 红）；展开看逐条回执与 Bundle 原文；行内操作：重新回流未响应项（REQ-4.NEW-2）、撤回（REQ-4.NEW-3，仅无信号时）。

### 6.3 needs-review 复核界面（Inbox 点入）

```
┌ 复核：docs/architecture.md · c-101 ────────────────────────────┐
│ 你的评论(✏️): 并发写入会不会有问题？如果用户超过1万怎么办        │
│ agent 回执: done — "Added WAL-mode analysis and a migration    │
│            plan section." · 2026-07-26 11:02                   │
│ ┌─ Base 变更 diff ─────────────┬─ Lens（重投影后）────────────┐ │
│ │ - SQLite is chosen for…      │ 这段更新了：AI 补充了 WAL     │ │
│ │ + SQLite (WAL mode) is…      │ 模式下的并发分析，并新增了    │ │
│ │ + ### Migration plan (>10k)  │ 「超过 1 万用户的迁移预案」…  │ │
│ └──────────────────────────────┴──────────────────────────────┘ │
│        [✓ 通过 (resolve)]  [↩ 不行，重开 (reopen) + 追评]       │
└────────────────────────────────────────────────────────────────┘
```

要点：diff 与评论上下文**并排**；resolve 后浮层「要为这个决策写张卡片吗？」（REQ-5.4）；同 Bundle 多条支持「下一条」连续复核。

### 6.4 48 小时未响应提醒

Bundle 发出 48h 后仍有 sent 评论无任何信号 → Inbox 卡片「AI 似乎没有处理这 N 条」，操作：重新回流 / 撤回转 open / 忽略（再 48h 后不再提醒）；同时上报 `bundle_no_response_48h`。

---

## 7. 数据模型与技术要点（供技术设计参考，非约束实现）

### 7.1 表（SQLite，sidecar 库）

- `feedback_bundle(id, project_id, created_at, file_path, markdown, token_estimate, doc_token_total, delivered_via, resend_of, retracted_at)`
- `bundle_comment(bundle_id, comment_id, intent_en, quote_snapshot, status_at_send)`
- `comment_receipt(id, comment_id, bundle_id, source ∈ mcp|receipt_file|fs_change, action ∈ done|answered|declined|null, note, created_at)`
- `mcp_audit(id, tool, args_digest, caller_project_root, created_at)`

### 7.2 回执匹配逻辑

comment_id 为唯一匹配键（Bundle 与 MCP 返回中均携带）；Receipt 文件解析按行匹配 `- [x] c-xxx` 与 `answered:/declined:` 后缀；同一评论多信号幂等：状态转换只发生一次，后续信号仅追加 `comment_receipt` 记录供历史展示。

### 7.3 「Base 变更命中被评 Block」检测算法要点

1. F1 事件（防抖后）触发对变更文件跑 Block 级 diff（复用 §2.4 的锚点迁移管线，同一次计算双用途：迁移锚点 + 判命中）。
2. 命中判定：被评 Block 的新旧内容哈希不一致，或 Block 被删除/拆分/合并（迁移置信度信息附带）。
3. 时间窗过滤：仅当该评论处于 sent 且变更发生在所属 Bundle 创建之后，才视为兜底回收信号；open 状态评论的 Base 变更走 F3 的「原文已变化」逻辑，不误触发 needs-review。
4. 用户自己在 PrismDocs 内编辑 Base（REQ-1.6）产生的变更打本地编辑标记，不作为兜底信号（防自我误闭环）。

---

## 8. 边界情况与异常处理（穷举）

| # | 场景 | 处理 |
|---|---|---|
| 1 | agent 改了文件但无任何回执（未装 MCP/hook） | REQ-4.4.3 兜底检测转 needs-review（继承主 PRD） |
| 2 | agent 只处理部分评论 | 命中/回执的转 needs-review；其余保持 sent，48h 提醒 + 可重发（S3） |
| 3 | Bundle 未处理时用户又发新 Bundle | 允许，新旧独立；sent 评论在回流面板禁选，不重复入包（REQ-4.1.1） |
| 4 | agent 改错文件（变更未命中任何被评 Block） | 评论不转状态；变更走 F1/F2 正常「新变更待 review」通道；48h 后按无响应提醒 |
| 5 | agent 改了被评 Block 但改的与评论无关 | 产品无法判意图 → 仍转 needs-review，由复核界面的 diff+评论并排交给人判断（reopen 兜底） |
| 6 | 提问类评论 agent 却改了文档 | 转 needs-review 并提示「此条为提问但检测到变更」（REQ-4.5.2） |
| 7 | `respond_to_comment` 携带非法/已 resolve 的 comment_id | 返回 error，不改状态，记审计日志 |
| 8 | MCP 端口被占用 | 自动递增端口并更新 `.prismdocs/mcp.json`；stdio 代理不受影响（REQ-4.NEW-1） |
| 9 | 多项目并发（两个项目同时回流） | 单 server 按项目根路由；`list_feedback` 仅返回调用方项目的 Bundle |
| 10 | 桌面端未运行时 agent 调 MCP | stdio 代理返回明确错误文案，指引 agent 改读 `.prismdocs/feedback/` 文件 |
| 11 | agent 直接删除/篡改 Bundle 文件正文 | 本地库存有权威副本；仅 Receipt 区变更被解析，其余篡改忽略并在历史标注 |
| 12 | 回流后用户在 PrismDocs 编辑了同一 Block | 本地编辑不触发兜底（§7.3-4）；评论 quote 保持发送时快照 |
| 13 | Bundle 中文件随后被重命名/删除 | F1 哈希识别随迁：历史与回收逻辑跟随新路径；删除则评论按 F3 降级，Bundle 历史标注「目标已删除」 |
| 14 | 意图摘要生成失败（无 key/超额） | REQ-4.1.4 降级直传，不阻塞回流 |
| 15 | `.prismdocs/` 写入失败（只读盘/权限） | 回流失败明确报错；MCP 通道仍可用时提示「仅 MCP 交付」并降级发送 |
| 16 | git 操作（pull/checkout）批量改动命中被评 Block | 无法与 agent 修改区分 → 仍转 needs-review，复核界面显示变更来源提示（P0.5：关联 REQ-1.7 的 commit 信息辅助判断） |
| 17 | 多个 Bundle 时间窗重叠，同一变更可归因到多个 Bundle | 归因歧义 → 全部标 inferred（REQ-4.7.2），溯源信息列出全部候选 Bundle/评论，交由复核界面人工判断 |
| 18 | agent 回执的 comment_id 与实际变更的 Block 不符（回执 c-101 但变更命中 c-102 的 Block） | 以回执为准建立关联，但 confidence 降级标注（不按 explicit 显示），复核界面提示回执与变更不一致 |

---

## 9. 埋点（opt-in，继承主 PRD §6）

| 事件 | 属性 | 说明 |
|---|---|---|
| `feedback_bundle_sent` | bundle_id、comment_count、按类型计数、token_estimate、token_ratio（/全文）、delivered_via（file/mcp/both）、has_fulltext、is_resend | 漏斗起点 |
| `bundle_first_response` | bundle_id、latency_min、source（mcp/receipt_file/fs_change） | 通道有效性与响应时延 |
| `comment_needs_review` | comment_id、source、action | 回收粒度 |
| `loop_closed` | comment_id、bundle_id、loop_duration_min、agent_kind（claude-code/cursor/unknown，据 MCP client 信息或缺省）、closed_via（mcp/fallback） | **北极星**；首个上报同时发 `first_loop_closed` |
| `comment_reopened` | comment_id、reopen_count | 护栏：agent 误改率代理指标 |
| `bundle_no_response_48h` | bundle_id、pending_count | 漏斗漏损点 |
| `mcp_installed` / `hook_installed` | agent_kind | 集成渗透率 |
| 变更来源分布（`comment_needs_review` 附带 provenance ∈ feedback-triggered/inferred/external-unknown） | 三类占比 | external-unknown 占比高 = 闭环渗透不足，产品健康度信号 |

**闭环漏斗定义**：`feedback_bundle_sent → bundle_first_response → comment_needs_review → loop_closed`，按 bundle 归因；北极星 = 周内 `loop_closed` 计数；诊断视图按 delivered_via 分层对比降级路径转化率。

---

## 10. 验收标准与测试要点

继承 AC-4a/4b/4c，细化端到端脚本：

**AC-4a-1（Claude Code 全自动路径）**：真实项目装好 MCP+hook+CLAUDE.md 片段 → Lens 上创建 3 条评论（✏️×1、💬×1、❌×1）→ 回流 → 新开 Claude Code 会话，hook 提示后 agent 自行完成 `get_feedback`→改 Base→3 条 `respond_to_comment` → 验证：3 条均转 needs-review 且提问答复在线程内；Lens 受影响段重投影+高亮；逐条复核 resolve → 3 条 `loop_closed` 上报。全程唯一人工输入为评论与复核点击。

**AC-4a-2（Cursor / 文件协议路径）**：不装 MCP → 回流 2 条 ✏️ 评论 → 粘贴剪贴板指令到 Cursor → agent 改 Base（无回执）→ 验证：兜底检测在 FS 事件后 10s 内将 2 条转 needs-review（source=fs_change）→ 复核 1 条 resolve、1 条 reopen 追评再回流，新 Bundle 不含已 resolve 项。

**AC-4b-1**：对 3 份典型文档（2k/5k/10k token）各回流 3–5 条评论，Bundle token ≤ 全文 30%；quote 截断规则生效于超长 Block。
**AC-4c-1**：全新用户不进任何设置向导，仅凭回流面板 + 剪贴板指令完成一次完整闭环。
**AC-4-新**：撤回后 feedback 文件消失、评论回 open；48h 提醒在 mock 时钟下准时触发且上报事件；端口占用场景 MCP 仍可连通。

**AC-4d（溯源）**：AC-4a 的端到端用例中，needs-review 复核界面正确显示触发评论（「本次修改由 <来源> 因 <评论摘要> 触发」）；在同一项目手工修改一处未回流关联的内容命中被评 Block，该变更被标为 external-unknown 并出现 REQ-4.7.5 的特殊提示。

**测试要点**：三信号幂等（同一评论 MCP+文件变更先后到达只转一次状态）；§8 表逐项覆盖；MCP 安全用例（越权写、跨项目读均被拒）。

---

## 11. 依赖与开放问题

**依赖**：F3 评论与状态机（前置）；F1 FS watcher 与 Block diff 管线（兜底检测复用 §2.4 锚点迁移计算）；F2 增量重投影触发接口；F7/F5 供给 `get_context_pack`/`list_cards` 内容；LLM 通道（意图摘要，可降级）。

**开放问题**

| # | 问题 | 倾向 |
|---|---|---|
| OQ-1 | 主 PRD respond_to_comment 的 action 含 declined，但 declined 后评论应转 needs-review（现方案）还是新增终态？ | 转 needs-review 由人裁决，不加状态；写回主 PRD 澄清 |
| OQ-2 | Approve 类评论是否需要回流？（无需 agent 动作） | 默认不勾选但可选；回流仅为让 agent 知晓决策上下文，回执 done 即转 needs-review 一键 resolve——待 M0 观察是否多余 |
| OQ-3 | hook 采用 SessionStart 单点还是加 Stop hook 提醒回执？ | MVP 仅 SessionStart，避免打扰；视内测 agent 漏回执率决定 |
| OQ-4 | 兜底信号与 git pull 混淆（§8-16）是否需要 commit 作者启发式区分？ | P0.5 结合 REQ-1.7；MVP 接受由人复核裁决 |
| OQ-5 | 意图摘要用哪档模型、是否与 Lens 投影共用配置？ | 共用快速档；主 PRD Q1 决议时一并定 |

---

## 12. 变更记录

| 版本 | 日期 | 变更 |
|---|---|---|
| v0.1 | 2026-07-26 | 初稿 |
| v0.2 | 2026-07-26 | 同步主 PRD v0.2：新增 REQ-4.7 Agent 贡献溯源细化（REQ-4.7.1～4.7.5：来源分类/归因方法/复核展示/OKF provenance 数据结构预留/external-unknown 提示）；§5.1 README 草案声明 OKF v0.1 约定、§5.2 MCP 工具面追加 export_okf_bundle（P1）；§8 边界情况追加 #17/#18（归因歧义、回执与变更不符）；§9 埋点追加变更来源分布；§10 追加 AC-4d（溯源） |
