# PRD-F3：段落级评论（PrismDocs MVP 子 PRD）

| 项目 | 内容 |
|---|---|
| 文档类型 | 子 PRD（Feature PRD）— F3 段落级评论 |
| 版本 | v0.1 草案 |
| 日期 | 2026-07-26 |
| 上游文档 | 主 PRD《PrismDocs MVP 主产品需求文档》v0.1 §3-F3（另参照 §2.4 Block 锚定机制、§3-F4 评论回流、§2.3 Inbox） |
| 作者 | Shean（起草协作：Claude） |

---

## 1. 功能概述与目标

段落级评论（Comment）是用户作用于文档的唯一通道：Lens 不可编辑、Base 默认由 agent 维护，用户的质疑、修改要求与决策全部以 Comment 形式挂在 Block 上，经 F4 打包为 Feedback Bundle 回流给 agent。

**对北极星指标「闭环数/周」的贡献路径**：评论是闭环的起点。每一个 `loop_closed`（闭环）都始于一条 Comment 的创建，途经 `open → sent → needs-review → resolved`。F3 的产品目标即优化这条漏斗的头两步：

1. **降低创建成本**：≤2 次点击 + 输入即可发出评论（AC-3c），让"随手批注"成为习惯；
2. **保障锚点可靠**：评论精确落在 Base 对应 Block 上、Base 被 AI 大改后 0 静默丢失（AC-3b），用户才敢把决策交给评论而不是回到 IDE 对话框。

不做的事：F3 不负责回流打包与 agent 通信（F4），不负责 Lens 生成（F2），只负责"评论的创建、锚定、线程、状态与展示"。

---

## 2. 范围

**In Scope**
- Lens / Base 任意 Block 上的评论创建（悬停按钮 / 选中浮条），quote 快照
- 四种评论类型：💬 提问、✏️ 修改要求、✅ Approve、❌ Reject
- 评论线程（回复）与状态机 `open → sent → needs-review → resolved / reopened`
- 文档级评论区；评论侧栏与筛选；向 Inbox 供给 needs-review 事项
- 评论的 sidecar 存储（本地库，不写入用户 Markdown）
- 锚点迁移失败时的显式降级（文档级评论 + quote 快照）

**Out of Scope**
- 多人评论 / @提及 / 通知他人（P1，见主 PRD §7.2）
- 对 Clip 的评论（主 PRD REQ-6.7 明确不可）
- 锚点迁移算法本身的实现（属技术设计文档；F3 只定义其行为契约与阈值语义）
- Feedback Bundle 的生成、英文意图摘要、MCP 回执（F4）

**与相邻功能的边界与依赖**

| 相邻功能 | 方向 | F 间接口 |
|---|---|---|
| F2 Lens | F3 依赖 F2 | 评论锚定到 Block；Lens 段落与 Base Block 一一锚定，故 Lens 上创建的评论直接落在 Base Block ID 上，无需坐标换算。Lens 重投影不改变评论锚点 |
| F1 导入/同步 | F3 依赖 F1 | Base 变更（磁盘为准）触发 §2.4 锚点迁移；文件删除→文档 archived，评论保留 |
| F4 回流 | F4 消费 F3 | F4 读取 open 评论打包 Bundle 并将状态置 sent；agent 回执或 Base 变更命中被评 Block 时由 F4 将状态置 needs-review；resolve/reopen 由用户在 F3 界面完成，resolve 事件回传 F4 计入闭环 |
| F5 卡片 | F3 触发 F5 | 评论 resolve 时提示"要为这个决策写张卡片吗？"（主 PRD REQ-5.4）——F3 只负责在 resolve 成功后调起该入口并预填上下文链接，卡片创建属 F5 |
| Inbox（§2.3） | F3 供给 | needs-review 评论以卡片形式聚合进 Inbox"AI 已修改待复核"分区 |

---

## 3. 用户故事与关键场景

**US-1 在 Lens 上写修改要求（主路径）**
Shean 读中文 Lens，看到"选 SQLite 而不是 Postgres"一段。悬停该段 → 右侧浮现评论按钮 → 点击 → 气泡默认类型 ✏️ 修改要求 → 输入"并发写入会不会有问题？超过 1 万用户怎么办" → Enter 发出。评论状态 open，锚定该 Lens 段落对应的 Base Block；切到对照视图，评论同时显示在 Base Block 旁（AC-3a）。

**US-2 在 Base 上选中文字 Approve**
能读英文的用户在 Base 视图选中一句话 → 浮条出现 → 选 ✅ Approve → 可附言可不附 → 发出。选中文字作为 quote 存入评论。

**US-3 AI 改后复核并 reopen**
评论经 F4 回流（→ sent），Claude Code 修改 Base，评论转 needs-review 并进入 Inbox。Shean 从 Inbox 卡片点入，看到变更高亮 + 原评论线程；认为 AI 只改了一半 → 点 Reopen 并追评"1 万并发的数字依据呢？" → 状态 reopened，等待下次回流。若满意则点 Resolve → 触发 `loop_closed` → 弹出 F5 卡片入口提示。

**US-4 锚点降级（0 静默丢失原则）**
Shean 评论过的 Block 被 AI 整段重写，迁移置信度低于阈值。评论不消失：降级为文档级评论，显示警示条"原位置已变化，请确认"，并展示评论时的 quote 快照。Shean 可将其手动重新锚定到新 Block，或就地 resolve/reopen。

**US-5 文档级整体意见**
读完速读区后，Shean 对整份文档有一句总体意见（"这份 spec 整体太乐观了"），在文档级评论区直接发出 💬 提问，不锚定任何 Block。

---

## 4. 详细功能需求

继承主 PRD REQ-3.1～3.7，细化编号 REQ-3.y.z；新增项标注回填。

**REQ-3.1 评论入口**
- REQ-3.1.1 悬停任意 Block（Lens 或 Base 视图）时，Block 右缘出现评论按钮；点击即打开该 Block 的评论气泡（无 quote）。
- REQ-3.1.2 选中 Block 内文字时出现浮条（评论 + 四类型快捷入口 + "存为卡片"F5 入口）；经浮条创建的评论把选中文字存为 quote。quote 永远保存创建时刻的文本快照，不随 Base 更新而变。
- REQ-3.1.3 在 Lens 上创建的评论锚定到该 Lens 段落对应的 Base Block ID（§2.4 锚定机制）；对照视图中同一评论在两侧同步显示（AC-3a）。
- REQ-3.1.4 选区跨多个 Block 时，评论锚定到选区起始 Block，quote 保留完整选区文本（见 §7 边界）。
- REQ-3.1.5 快捷键：选中文字后 `Cmd+Shift+M` 打开评论气泡；气泡内 `Cmd+Enter` 发出、`Esc` 取消；数字键 `1-4` 切换类型（💬/✏️/✅/❌）。**REQ-3.NEW-1（主 PRD 未覆盖，需回填）**

**REQ-3.2 评论类型**
- REQ-3.2.1 创建时四选一，默认 ✏️ 修改要求；💬 提问期待 agent 文字答复（经 F4 REQ-4.5 显示在线程内）而非改文档；✏️ 期待 agent 修改 Base。
- REQ-3.2.2 ✅ Approve / ❌ Reject 是对该 Block 或整份文档的决策标记，可附言、可不附言（唯一允许空正文的类型）。回流时 F4 向 agent 明确决策语义（reject = 换方案重来）。
- REQ-3.2.3 类型创建后不可更改（保证回流语义稳定）；发错则删除重发（仅 open 状态可删，见 REQ-3.3.4）。**REQ-3.NEW-2（主 PRD 未覆盖，需回填）**

**REQ-3.3 线程与状态机**
- REQ-3.3.1 每条评论是一个线程根，支持用户回复与 agent 回执/答复（经 F4 写入）；线程共享同一状态与锚点。
- REQ-3.3.2 状态机迁移表（唯一权威定义）：

| 迁移 | 触发者 | 触发条件 |
|---|---|---|
| （创建）→ open | 用户 | 发出评论 |
| open → sent | 系统（F4） | 评论被打包进 Feedback Bundle 并导出 |
| sent → needs-review | 系统（F4） | agent MCP 回执命中该评论，或检测到 Base 变更命中被评 Block（REQ-4.4） |
| needs-review → resolved | 仅用户 | 复核通过，点 Resolve（计入北极星闭环） |
| needs-review → reopened | 仅用户 | 复核不通过，点 Reopen，可追评 |
| reopened → sent | 系统（F4） | 随下一个 Bundle 再次回流 |
| open → resolved | 仅用户 | 允许直接 resolve（如自问自答），但不经过 sent/needs-review 的 resolve **不计入** `loop_closed`（对齐主 PRD §6） |

- REQ-3.3.3 resolve/reopen 权限仅属于用户；agent 通过 MCP 只能回执（needs-review 化），不能 resolve 评论、不能创建/删除评论（对齐主 PRD §4.2 安全约束）。
- REQ-3.3.4 删除评论：仅 open 状态可删（硬删）；sent 及之后仅可 resolve（留痕），防止已回流的评论在 agent 侧与产品侧状态漂移。**REQ-3.NEW-3（主 PRD 未覆盖，需回填）**
- REQ-3.3.5 resolve 成功后调起 F5 入口提示（可关闭，不阻塞）。

**REQ-3.4 评论语言**：正文语言自由（中文为主），F3 不做翻译；英文意图摘要在回流时由 F4 生成，非 F3 职责。

**REQ-3.5 文档级评论区**：文档底部（及侧栏顶部固定入口）提供不锚定 Block 的整体评论；锚点降级的评论也归入此区并带警示标记。

**REQ-3.6 评论侧栏与筛选**
- REQ-3.6.1 侧栏按文档聚合全部评论，按 Block 在文中的顺序排列；筛选器：状态（open/sent/needs-review/resolved/reopened）× 类型（四类）。
- REQ-3.6.2 Inbox 聚合跨文档 needs-review 项（见 §5 卡片规格）；resolved 默认折叠。

**REQ-3.7 存储**：评论、线程、锚点、quote 快照全部存 PrismDocs 本地库（sidecar，SQLite），绝不写入用户 Markdown 源文件；断网可评（主 PRD §5 本地优先）。

---

## 5. 交互与界面规格

**Block 悬停 / 选中浮条（Lens 视图）**

```
│ ⚖️ 为什么选 SQLite：写入走单队列，够用……   [💬2]│ ← 已有评论角标
│ （悬停时右缘浮现） ────────────────── [ + 评论 ]│
│ 选中文字浮条： [💬 提问][✏️ 修改][✅][❌][📇 存为卡片]│
```

**评论气泡（创建态）**：类型选择（默认 ✏️）→ quote 预览（若有，灰底折叠）→ 输入框 → `Cmd+Enter 发送`。

**评论侧栏（含状态角标）**

```
┌ 评论 · architecture.md ── 筛选:[状态 ▾][类型 ▾] ┐
│ ✏️ ● open        "并发写入会不会有问题…"        │
│ 💬 ◐ sent        "为什么不用 ORM？"             │
│ ✏️ ◉ needs-review "1 万并发的依据…" [复核]      │
│ ⚠️ 文档级（已降级）"原位置已变化，请确认"        │
│ ▸ 已解决 (12)                                   │
└─────────────────────────────────────────────────┘
```

**Inbox 中的 needs-review 卡片**

```
┌ AI 已修改待复核 ────────────────────────────────┐
│ architecture.md · ✏️ 修改要求                    │
│ 你的评论："并发写入会不会有问题…"                │
│ AI 回执：Added WAL-mode analysis (2 blocks)     │
│ [ 查看变更 ]  [ Resolve ✓ ]  [ Reopen ↺ ]       │
└─────────────────────────────────────────────────┘
```

**状态与空态**
- 状态角标：open ●（蓝）/ sent ◐（灰蓝）/ needs-review ◉（橙，Inbox 同色）/ resolved ✓（绿，折叠）/ reopened ↺（红）。
- 空态：侧栏无评论时显示"选中任意段落即可评论——你的评论会驱动 AI 修改文档"。
- 降级警示态：降级评论顶部橙色警示条"⚠️ 原位置已变化，请确认"+ quote 快照 + `[重新锚定]` 按钮（进入点选 Block 模式）。
- Lens 段落上的评论气泡在对照视图中与 Base 侧连线对齐显示（AC-3a 的可视化承载）。

---

## 6. 数据模型与技术要点（供技术设计参考，非实现规约）

```
comment_thread: id, project_id, doc_id, anchor_id(nullable→文档级),
  type(question|change_request|approve|reject),
  status(open|sent|needs-review|resolved|reopened),
  quote_snapshot(text, 创建时不可变), created_at, resolved_at,
  degraded(bool), degraded_reason, last_bundle_id(F4 关联)
comment_message: id, thread_id, author(user|agent), body, created_at
anchor: id, doc_id, block_id, block_hash, heading_path,
  confidence(最近一次迁移置信度), migrated_at
```

- 锚点迁移置信度行为契约（阈值具体数值属技术设计，行为属产品定义）：`confidence ≥ T_high` 静默随迁；`T_low ≤ c < T_high` 随迁但在评论上标"原文可能已变化"（弱提示）；`c < T_low` **显式降级**为文档级评论（degraded=true）+ 警示条 + quote 快照，绝不静默丢失（§2.4）。
- quote 快照与 anchor 分离存储：降级后 anchor 置空，quote 仍在。
- 状态机迁移全部写事件日志（迁移、触发者、时间），供 AC-3b 验证与埋点复用。
- heading_path（标题路径）冗余存储，供 F4 Bundle 定位（REQ-4.1）与降级后人工重锚参考。

---

## 7. 边界情况与异常处理

| # | 场景 | 行为 |
|---|---|---|
| E1 | 被评 Block 被 AI 大改（置信度低） | 按 §6 契约降级为文档级 + 警示 + quote 快照；若该评论处于 sent，同时转 needs-review（Base 变更命中，REQ-4.4） |
| E2 | 被评 Block 被删除 | 降级为文档级，保留 quote 快照，警示"原段落已删除" |
| E3 | Block 合并/拆分 | 迁移算法取相似度最高的目标 Block；置信度不足则按 E1 降级；拆分时不复制评论（一条评论只锚一个 Block） |
| E4 | 跨 Block 选区评论 | 锚定起始 Block，quote 保留完整选区（REQ-3.1.4）；起始 Block 后被删则按 E2 |
| E5 | 同一 Block 多条评论 | 全部独立保留，侧栏按创建时间排列，角标显示计数；回流时由 F4 打包（主 PRD 边界） |
| E6 | 评论进行中（气泡未发出）Base 变更 | 草稿不丢：发出时按最新 Block 状态锚定；若目标 Block 已消失，提示改为文档级或取消 |
| E7 | 评论处于 sent 期间文件被外部大改（git pull / 用户 IDE 编辑，非 agent） | 与 agent 修改同等处理：命中被评 Block → needs-review（磁盘为准，REQ-1.5；系统不区分改动者） |
| E8 | 文档被外部删除 | 文档 archived（F1 边界），评论全部保留并可在侧栏查看，状态冻结不可回流 |
| E9 | 文档重命名/移动 | 内容哈希识别为同一文档（F1），锚点与评论随迁，无感知 |
| E10 | resolved 评论所在 Block 再次变更 | 不重开、不提示（闭环已完成）；用户可手动 reopen |
| E11 | Lens 重投影措辞变化但 Base Block 未变 | 锚点是 Base Block ID，评论不受影响，不触发任何状态变化 |
| E12 | 对同一 Block 既有 Approve 又有新的修改要求 | 允许共存（时序上后者覆盖语义由 agent/用户判断）；侧栏按时间排列呈现矛盾信号 |

---

## 8. 埋点（opt-in，对齐主 PRD §6）

| 事件 | 属性 | 对应指标 |
|---|---|---|
| `comment_created` | type、layer(lens/base)、有无 quote、入口(hover/浮条/快捷键) | 闭环漏斗头部；类型分布 |
| `comment_sent`（F4 触发，F3 记录状态迁移） | bundle_id | 漏斗：open→sent 转化 |
| `comment_needs_review` | 触发方式(MCP 回执/文件变更检测) | 漏斗诊断 |
| `comment_resolved` | 是否经过 sent/needs-review（是→同时发 `loop_closed`） | **北极星：闭环数/周** |
| `comment_reopened` | reopen 次数 | agent 误改率护栏（BRD §9 反指标） |
| `anchor_migrated` | confidence 区间 | 锚点健康度 |
| `anchor_degraded` | 原因(低置信/Block 删除) | **锚点降级率**（AC-3b 线上镜像；目标 <10%） |
| `comment_rebound`（降级后手动重锚） | — | 降级恢复率 |

---

## 9. 验收标准与测试要点

细化主 PRD AC-3a/3b/3c：

- **AC-3a-1**：Lens 段落上创建的评论，切换到对照视图后显示在 Base 对应 Block 旁，100% 对应正确（抽样 50 条）。
- **AC-3a-2**：Base 视图创建的评论在仅 Lens 视图中显示于对应 Lens 段落旁。
- **AC-3b-1（锚点迁移专项）**：构造 4 类重写场景各 ≥20 例——(a) 段内局部改写（≥90% 应静默随迁）；(b) 段落重排序（应随迁）；(c) 整段重写语义保留（随迁或弱提示）；(d) 整段删除/替换为新内容（应显式降级）。总体：≥90% 正确迁移或显式降级，静默丢失 = 0（发布门槛，主 PRD §7-3）。
- **AC-3b-2**：AI 重写文档 50% 内容（真实 Claude Code 会话产生的 diff，非合成），逐条核对事件日志：每条评论必有 `anchor_migrated` 或 `anchor_degraded` 记录，无遗漏。
- **AC-3b-3**：降级评论 100% 保留 quote 快照且警示条可见；`[重新锚定]` 可用。
- **AC-3c-1**：悬停入口路径：hover → 点按钮 → 输入 → Cmd+Enter，全程 2 次点击 + 输入。
- **AC-3c-2**：状态机非法迁移（如 agent 尝试 resolve、open 直接 needs-review）被拒绝并记录。
- **AC-3c-3**：评论全部存于 sidecar 库；对用户 Markdown 文件做前后 checksum 比对，创建/解决 1000 条评论后文件 0 字节变化。
- **AC-3c-4**：断网状态下创建、回复、resolve 评论全部可用。

---

## 10. 依赖与开放问题

**依赖**：F1 文件监听与文档身份识别（内容哈希）；F2 Block 锚定与对照视图（§2.4 为共同契约）；F4 状态机中 sent/needs-review 的系统侧触发与 agent 回执写入；F5 resolve 后卡片入口；Inbox 框架（§2.3）。

**开放问题**

| # | 问题 | 倾向 |
|---|---|---|
| OQ-1 | 置信度阈值 T_high/T_low 的具体数值与算法归属技术设计，但需 M0 用真实 agent diff 标定，否则 AC-3b 的 90% 无法承诺 | M0 数据标定，技术设计文档定稿 |
| OQ-2 | reopened 评论是否自动进入下一个 Bundle，还是需用户再次显式勾选？主 PRD REQ-4.1 默认"全部 open 状态"未提 reopened | 倾向 reopened 视同 open 默认入包；需与 F4 子 PRD 对齐并回填主 PRD |
| OQ-3 | ✅ Approve 的评论是否应有独立的轻量状态（approve 后无需 AI 动作，sent→resolved 无 needs-review 环节）？当前统一走五态状态机略重 | 倾向 Approve 回流后由系统直接置 resolved（agent 确认收到即可）；待 F4 联合决议 |
| OQ-4 | E12 的 Approve 与后续修改要求共存，是否需要"撤销 Approve"操作？ | MVP 不做撤销，靠时序呈现；观察内测 |
| OQ-5 | 主 PRD Q3（sidecar vs git 同步）若 P1 转向 git 便携模式，评论表结构需预留导出格式 | 本 PRD §6 模型以可序列化为约束设计 |
