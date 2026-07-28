# PRD-F8:跨项目知识层(PrismDocs MVP 子 PRD)

| 项目 | 内容 |
|---|---|
| 文档类型 | 子 PRD — F8 跨项目知识层 ★ 防偏差 |
| 版本 | v0.1 |
| 日期 | 2026-07-28 |
| 上游文档 | 主 PRD《PrismDocs MVP 主产品需求文档》v0.3 §3-F8(另参照 §2.1/2.2 Xref·Contract 对象、§2.4 锚定机制四消费方、§2.5 OKF 兼容约定、§3-F4 回流通道、§4 agent 协议);BRD v0.3 §6.5、§4.2 痛点 #9;《调研_整体构想v2_多项目知识层》v0.1 |
| 作者 | Shean(起草协作:Claude) |

---

## 1. 功能概述与目标

F8 把 Workspace 从「项目的集合」升级为「知识图」:项目间文档可相互引用(**Xref**),关键文档可标记为**契约**(Contract)并被下游项目**订阅**;上游契约被 agent 修改且命中订阅时,下游项目收到**漂移警示**,一键生成**核对反馈**(Feedback Bundle)交给下游的编码 agent 处理。典型场景:server 与 client 两个仓库独立开发,API spec、数据模型、协议约定不再悄悄脱节。

**设计基线(不可违背)**:

1. **组合而非新引擎**:F8 完全构建在既有四个引擎之上——F1 多项目 watch、锚定引擎(§2.4,F8 是其第四消费方)、F4 回流闭环、Inbox。agent 侧**零新协议**。
2. **不污染源文件**:全部引用/订阅关系存 sidecar;源文件中已有的跨项目链接只被"识别",不被改写。
3. **订阅制防警报疲劳**(BRD 设计约束):只有显式订阅命中的变更才产生跨项目警示;警示按契约聚合,不逐 Block 轰炸;无 48 小时追杀提醒。
4. **0 静默丢失契约延伸**:Block 级引用与订阅随锚点迁移,置信度低时显式降级警示,与评论同一契约(§2.4)。

**对北极星指标(闭环数/周)的贡献路径**:漂移警示 → 核对反馈 → 下游 agent 处理 → 复核 resolve,是闭环的一种(跨项目闭环计入 `loop_closed`)。同时 F8 是多仓用户的留存理由与 Pro 卖点(BRD §10:免费层单项目天然不含 F8),其护航指标为漂移警示处理率 ≥60%、误报率 <10%(BRD §9)。

**MVP 只做结构信号**:命中判定基于锚定 diff(哪个被订阅 Block 变了),不做语义比对;语义漂移检测(LLM 比对上下游文档断言矛盾)为 P1。

---

## 2. 范围

### In Scope(P0,除标注外)

- Xref 建立:应用内显式建链、卡片 `[[` 跨项目双链的存储侧、自动发现(**P0.5**)
- 契约标记(文档级标志位)与订阅(文档级/Block 级粒度)
- 漂移警示:命中判定、警示内容(diff 摘录 + 溯源 + 受影响引用清单)、聚合、自触发标注与静默
- 一键下游核对反馈(经 F4 通道交付与回收,核对评论机制见 REQ-8.NEW-1)
- 订阅健康度管理(正常 / 源已删 / 需重新确认)
- 非 Markdown 契约(openapi.yaml、proto 等)的文件级订阅
- Xref 的 OKF 导出对齐(`x-link-type`)
- 上游侧「谁订阅了我」视图(**P0.5**)

### Out of Scope(明确不做 / P1)

- 语义漂移检测(LLM 比对上下游断言矛盾)→ P1
- MCP `list_dependencies(path)` / `get_upstream_contracts()` → P1(MVP 用 F7 Context Pack 承载跨项目上下文)
- 核对 Bundle 的自动级联生成(A 警示→自动触发 B 警示→…)→ **永不做**(防警示风暴,人工逐环确认)
- 契约版本协商 / 审批流 / API 治理 → 不做(PrismDocs 不是 API 治理工具;代码级契约由 OpenAPI/契约测试等工具负责,见 BRD §3.2 v0.3 补充)
- 团队多人共享订阅与警示 → P1 团队版(关联主 PRD Q9)
- 跨 Workspace 引用 → 不存在(MVP 单 Workspace)

### 与相邻功能的边界与依赖

| 相邻功能 | 方向 | 接口内容 | 边界约定 |
|---|---|---|---|
| F1 导入/同步 | F1 → F8 | 多项目根注册、文档身份识别(重命名/移动随迁)、变更事件(防抖后)、批量模式信号(REQ-1.5.2) | F8 不自己 watch;非 Markdown 契约需用户在 F1 扫描规则中显式加入 glob |
| 锚定引擎(§2.4) | 引擎 → F8 | Block 级 diff 与迁移结果(第四消费方):命中订阅 Block 判定、Xref/订阅锚点随迁与置信度 | F8 不重复计算 diff;同一次计算四用途 |
| F3 评论 | F8 → F3 | 核对反馈以「核对评论」为载体进入 F3 状态机(REQ-8.NEW-1) | 漂移警示本身**不是**评论;F3 状态机权限需为系统创建的核对评论开例外(OQ-1) |
| F4 回流 | F8 → F4 | 核对 Bundle 走 F4 双通道交付、三信号回收、溯源展示;frontmatter 加 `drift_check` 标记 | F8 不新增 agent 侧协议;Bundle 格式对 agent 与普通反馈一致 |
| F5 卡片 | F5 ↔ F8 | 卡片 `[[` 跨项目候选的交互属 F5(REQ-5.3),链接落 xref 表属 F8 | 卡片跨项目链接是 references 型 Xref,不产生警示 |
| F7 Context Pack | F8 → F7 | 组装器 Workspace 作用域(REQ-7.1);警示详情提供「把上游契约加入 Pack」快捷入口 | Pack 内容与勾选逻辑属 F7 |
| Inbox(§2.3) | F8 → Inbox | 「上游契约变更」第四类事项 | 警示聚合规则由 F8 定,Inbox 只呈现 |

---

## 3. 用户故事与关键场景

**US-1 标记契约并订阅(初始化)**
Shean 的 Workspace 里有 `myapp-server` 和 `myapp-web` 两个项目。在 server 项目中,他打开 `docs/api-spec.md`,点「标记为契约」;切到 client 项目,在「上游契约」面板看到 Workspace 内全部契约文档,订阅 `api-spec.md` 并勾选只订阅 `## Endpoints` 一章。完成后 client 项目设置页显示 1 条活跃订阅。

**US-2 漂移警示 → 核对闭环(主路径)**
两周后,server 侧 Claude Code 处理一个 Feedback Bundle 时修改了 `POST /orders` 的返回结构。PrismDocs 的锚定 diff 命中被订阅 Block → `myapp-web` 的 Inbox 出现「上游契约变更」:diff 摘录(旧/新对照)、溯源(「由 server 项目的评论回流 fb-0726-a1b2 触发」)、受影响引用清单(client 的 `docs/api-client.md` 引用了该章节)。Shean 点「生成核对反馈」,确认目标文档为 `docs/api-client.md` → 核对 Bundle 写入 `myapp-web/.prismdocs/feedback/`,他对 client 侧 agent 说一句话;agent 对照更新了 client 文档与调用代码并回执 → needs-review → Shean 复核 resolve。跨项目闭环 +1,两个仓库没有漂移。

**US-3 自动发现既有链接(P0.5)**
client 的 `docs/architecture.md` 里本来就有一个指向 `../myapp-server/docs/api-spec.md` 的相对链接。导入时 PrismDocs 识别出目标是另一个已导入项目的文件,提示「识别到 1 个跨项目引用,确认收编?」;Shean 确认后该链接成为 references 型 Xref,出现在两侧的引用面板中。

**US-4 上游大规模重写后的订阅迁移**
server 侧 agent 把 `api-spec.md` 整体重构,`## Endpoints` 被拆成三章。订阅的 Block 锚点迁移置信度低 → client 收到警示「订阅段落已大幅变化,请重新确认订阅范围」,附旧订阅快照与新文档结构;Shean 重新勾选三个新章节。订阅从未静默失效。

**US-5 自触发静默**
Shean 自己在 server 项目的 Lens 决策清单上发起评论、回流、agent 改了契约。client 侧警示照常产生,但带「由你在上游的评论触发」标注;他确认无需下游跟进,点「静默」——警示留痕但不占未读。

---

## 4. 详细功能需求

继承主 PRD REQ-8.1~8.5,细化编号 REQ-8.y.z;新增项标注回填。

### REQ-8.1 跨项目引用(Xref)

- REQ-8.1.1 显式建链:文档头部动作菜单与 Block 悬停菜单提供「引用其他项目文档…」;选择器三级:项目 → 文档(树形,复用 F1 文档树组件)→ 可选 Block(按标题路径选择)。创建的 Xref 为 `references` 型。
- REQ-8.1.2 卡片双链跨项目:`[[` 联想候选覆盖全 Workspace(交互属 F5 REQ-5.3);候选排序:当前项目优先,跨项目项带项目名徽标;选中后链接落 xref 表(`created_by=user`)。
- REQ-8.1.3 自动发现(P0.5):导入与变更解析时,正文中的相对路径 / repo URL 链接若解析到另一已导入项目的文件 → 文档上出现「识别到 N 个跨项目引用」提示,逐条确认收编(`created_by=auto`)或忽略;**不自动收编**,忽略后同一链接不重复提示。
- REQ-8.1.4 link_type 语义:`references`(一般引用,双链/显式建链产生)、`contract-of`(订阅产生,系统写入)、`depends-on`(预留,MVP 不暴露 UI)。
- REQ-8.1.5 锚点联动:Block 级 Xref 挂 Block ID,随锚点迁移;迁移置信度低 → 显式降级为文档级引用并在引用面板标注「原段落已变化」,绝不静默丢失(同 F3 §6 契约)。
- REQ-8.1.6 引用面板:文档详情页显示「引用了谁 / 被谁引用」(含跨项目,带项目徽标);是无文件夹结构下跨项目导航的主要方式(与 F5 反链面板同构)。

### REQ-8.2 契约标记与订阅

- REQ-8.2.1 标记契约:文档动作「标记为契约」→ sidecar 记录 `is_contract=true`;契约是**独立标志位**,不强改文档已有的受控 type(一份 `type: Spec` 的文档可同时是契约,见 OQ-3);文档未设 type 时建议登记为 `Contract`(§2.5 词表)。取消标记时若存在活跃订阅 → 确认框列出订阅方,确认后订阅转失效态(留痕)。
- REQ-8.2.2 订阅入口:下游项目的「上游契约」面板(项目设置内,见 §5)列出 Workspace 内全部契约文档(排除本项目自己的);订阅粒度:**文档级(默认)**或 Block 级(按标题章节勾选,对应主 PRD Q8 倾向:默认整文档 + 引导标注关键章节)。
- REQ-8.2.3 非 Markdown 契约:`openapi.yaml`、`.proto` 等经用户在 F1 扫描规则显式加入后以附件级纳入;仅支持**文件级**订阅,变更即警示(无 Block diff,警示显示 mtime/size 变化与 git commit——可得时)。
- REQ-8.2.4 订阅管理:面板显示每条订阅的健康度——`active`(正常)/ `source_deleted`(契约源已删,警示后保留记录)/ `needs_reconfirm`(订阅段落大幅变化,待用户重新确认范围);退订留痕(历史警示不删)。
- REQ-8.2.5 单项目空态:Workspace 仅一个项目时,「上游契约」面板显示引导空态(「导入第二个项目后,可在项目间订阅契约文档」),F8 其余入口隐藏。

### REQ-8.3 漂移警示

- REQ-8.3.1 触发管线:F1 变更事件(防抖后)→ 锚定引擎 Block 级 diff → 命中判定(文档级订阅 = 任何 Block 实质变化;Block 级订阅 = 命中被订阅 Block 或其迁移后代)→ 为每个订阅方项目生成警示,推入其 Inbox。全链路在 FS 呈现预算内(<10s,§5 非功能)。
- REQ-8.3.2 警示内容:①上游 diff 摘录(变更 Block 旧/新对照,单 Block 超 120 词按 F4 quote 截断规则);②变更溯源(复用 REQ-4.7 数据:actor / trigger / confidence,如「由 server 的 Bundle fb-xxx · 评论 c-101 触发」或「external-unknown,来源不明」);③受影响引用清单(该契约在本项目的 Xref 反查,含引用位置)。
- REQ-8.3.3 聚合:同一契约、同一防抖批次的多 Block 变更合并为一条警示(逐 Block 明细在详情内);git 批量模式(REQ-1.5.2)下,多契约警示并入「同步了 N 个文件」的批量汇总,Inbox 只多一行「其中 M 个上游契约有变更」。
- REQ-8.3.4 自触发标注:上游变更溯源为 `feedback-triggered`(由本 Workspace 用户的评论回流触发)→ 警示带「由你在上游的评论触发」标注 + 一键「静默」(状态转 dismissed,留痕不占未读);`mcp-attributed` / `external-unknown` 正常警示,后者按 REQ-4.7.5 加"来源不明"提示。
- REQ-8.3.5 警示状态机:`open →(生成核对反馈)→ handled →(核对评论 resolve)→ closed`;`open →(忽略)→ dismissed`(留痕可查,可重新激活);**无 48h 追杀提醒**(与 F4 的 48h 机制刻意不同——警示是信息不是任务,防疲劳)。核对评论被 reopen 时警示回 handled。

### REQ-8.4 一键下游核对反馈

- REQ-8.4.1 生成入口:警示详情「生成核对反馈」→ 面板预填:目标文档(默认 = 受影响引用清单中的下游文档,可改;无引用文档时用户手选,见 §7-6)、核对指令预览、token 估算。确认后:①在目标文档创建一条**核对评论**(REQ-8.NEW-1);②生成 Bundle 写入**下游项目**的 `.prismdocs/feedback/`,frontmatter 增加 `drift_check: true`、`upstream_contract: <项目>/<路径>`、`alert_id`;③复制一句话指令到剪贴板(同 F4 REQ-4.1.6 模式)。
- REQ-8.4.2 核对 Bundle 内容:上游契约变更 diff 摘录(含前后对照与标题路径)+ 核对评论(含下游受影响引用清单)+ 指令头。指令头英文草案(接续 F4 REQ-4.1.5 风格,写死在 Bundle):

> This is a **drift check**: an upstream contract document this project depends on has changed. Review the diff below, then check whether THIS project's documents and implementation need to follow up. If changes are needed: update the referenced document sections (and code if applicable), then send a receipt via `respond_to_comment(comment_id, "done", note)`. If no change is needed: respond with `respond_to_comment(comment_id, "declined", reason)` explaining why this project is unaffected. Do not edit the upstream project's files.

- REQ-8.4.3 回收与闭环:核对评论走 F3 标准状态机(`open→sent→needs-review→resolved/reopened`)与 F4 三信号回收;resolve 上报 `loop_closed`(属性 `drift=true`),警示转 closed。declined 回执按 F4 OQ-1 决议同样转 needs-review,由人裁决"确实无需跟进"后 resolve。
- REQ-8.4.4 不级联:核对反馈导致下游文档变更时,若该下游文档本身也被第三个项目订阅 → 正常产生新警示,但**不自动生成**下一环核对 Bundle(人工逐环确认,防风暴)。

### REQ-8.5 OKF 导出对齐

- REQ-8.5.1 F7 导出 OKF Bundle 时(REQ-7.6),Xref 重写为 bundle 间相对链接(目标同批导出时)或降级为纯文本 + 原路径注记(目标未导出时,列入导出报告);`link_type` 物化为 frontmatter 扩展字段 `x-link-type`。
- REQ-8.5.2 契约标志导出:`is_contract=true` 的文档,导出 frontmatter 增加 `x-contract: true`(type 保持其受控词表值)。

### 新增需求(主 PRD 未覆盖,需回填)

- **REQ-8.NEW-1 核对评论机制**(回填主 PRD §3-F8/§3-F3):核对反馈的回收载体是一条**系统预生成、用户确认后创建**的文档级评论(类型 ✏️ 修改要求,正文 = 核对指令中文摘要 + 上游变更引用,`origin=drift_check` 标记,创建者记为用户)。它进入 F3 标准状态机与 F4 回收,从而复用全部既有机制与北极星计数。F3 的「评论仅用户创建」约束不变——核对评论在用户点击确认时以用户身份创建,系统只负责预填。
- **REQ-8.NEW-2 上游侧「谁订阅了我」视图**(P0.5,回填主 PRD):契约文档详情页显示订阅方清单(项目、粒度、订阅时间);编辑契约前用户可预估影响面。
- **REQ-8.NEW-3 警示留痕与历史**(回填主 PRD):dismissed/closed 警示在项目「漂移历史」列表可查(时间、契约、diff 摘要、处理方式);为误报率指标(BRD §9)提供数据面。

---

## 5. 交互与界面规格

### 「上游契约」面板(下游项目设置内)

```
┌─ 上游契约 · myapp-web ──────────────────────────────┐
│ 已订阅 (2)                                          │
│ ● myapp-server/docs/api-spec.md   [## Endpoints]    │
│   状态:正常 · 上次变更 2 小时前 · [调整范围][退订]   │
│ ⚠ myapp-server/docs/data-model.md [整文档]          │
│   状态:订阅段落已大幅变化,请重新确认 [重新确认]      │
│ ─────────────────────────────────────────────────── │
│ Workspace 内其他契约 (1)                             │
│ ○ shared-libs/docs/auth-protocol.md    [订阅…]      │
└─────────────────────────────────────────────────────┘
```

### Inbox 漂移警示卡片

```
┌ 上游契约变更 ────────────────────────────────────────┐
│ myapp-server/docs/api-spec.md · ## Endpoints · 2 处   │
│ 由 server 的评论回流 fb-0726-a1b2 触发                │
│ - POST /orders: response now returns `order_id` (was  │
│   `id`), adds `status` field …                        │
│ 本项目受影响引用:docs/api-client.md(1 处)           │
│ [ 查看 diff ]  [ 生成核对反馈 ]  [ 忽略 ]             │
└──────────────────────────────────────────────────────┘
```

自触发变体:标题行下追加灰字「由你在上游的评论触发」,操作区多一个 [静默]。

### 核对反馈生成面板

```
┌─ 生成核对反馈 → myapp-web 的 agent ──────────────────┐
│ 上游变更:api-spec.md · ## Endpoints(2 Block,diff 附)│
│ 目标文档:[docs/api-client.md        ▾](受影响引用)  │
│ 核对指令预览:                                        │
│  「上游契约 Endpoints 章节已变更(见 diff)。核对本项目 │
│   文档与实现是否需要跟进;需要则修改并回执,不需要则   │
│   declined 说明理由。」                               │
│ Bundle ≈ 640 tok            [取消]  [生成并发送 →]   │
└──────────────────────────────────────────────────────┘
```

发送成功态同 F4 §6.1(文件路径 + 剪贴板指令 + hook 提示);警示状态即转 handled。

### 状态一览

- 订阅健康度:`active`(●绿)/ `needs_reconfirm`(⚠橙,点入重新勾选范围)/ `source_deleted`(灰,仅留痕)。
- 警示状态:open(Inbox 未读)/ handled(带核对评论状态角标,联动 F3)/ dismissed(漂移历史可查)/ closed(✓)。
- 空态:单项目 Workspace 见 REQ-8.2.5;多项目但无契约 → 「把 server 的 API 文档标记为契约,client 就能订阅它」引导。

---

## 6. 数据模型与技术要点(供技术设计参考,非最终 schema)

```
xref: id, src_project_id, src_doc_id, src_block_id(nullable),
      target_project_id, target_doc_id, target_block_id(nullable),
      link_type(references|depends-on|contract-of),
      created_by(user|auto|system), confidence, degraded(bool), created_at
contract: doc_id, is_contract(bool), marked_at
contract_subscription: id, contract_doc_id, subscriber_project_id,
      scope(document|blocks), block_ids(JSON), status(active|source_deleted|needs_reconfirm),
      baseline_version_id(订阅时刻版本,首个基线), created_at
drift_alert: id, subscription_id, upstream_version_from, upstream_version_to,
      hit_block_ids(JSON), provenance_snapshot(JSON: actor/trigger/confidence),
      status(open|handled|dismissed|closed), check_comment_id(nullable),
      bundle_id(nullable), created_at
```

- 命中判定复用锚定引擎输出(第四消费方,§2.4):不重复解析、不重复 diff;警示生成在 F1 合并处理的同一事务尾部完成,保住 <10s 预算。
- 订阅基线:以订阅时刻的 `document_version` 为基线,**不回溯**订阅前的历史变更(§7-10)。
- 自触发识别规则:上游变更 `provenance.trigger` 关联到本 Workspace 的 Bundle → `feedback-triggered` → 标注可静默;无关联(external-unknown / mcp-attributed 无 Bundle)→ 正常警示。
- 溯源快照:警示存 provenance **快照**而非外键引用,上游 Bundle 被清理后警示历史仍完整(与 F4 REQ-4.6.3 的文件保留策略解耦)。
- 净零变更过滤:防抖批次内改了又改回(前后内容哈希一致)→ 无实质变化,不产生警示。
- 版本快照保留联动:被 `drift_alert` 引用的 `document_version` 计入不淘汰集合(扩展 REQ-1.NEW-2 的 anchored 语义,同 REQ-4.8 约束)。

---

## 7. 边界情况与异常处理

| # | 场景 | 处理 |
|---|---|---|
| 1 | 上游契约文档被删除 | 订阅转 `source_deleted`,下游警示「契约源已删除」一次;记录保留,不静默消失(继承主 PRD) |
| 2 | 上游契约重命名/移动 | F1 内容哈希识别同一文档,订阅与 Xref 无感随迁(继承) |
| 3 | 订阅的 Block 被拆分/合并/大改 | 随锚点迁移;置信度低 → 订阅转 `needs_reconfirm`,警示「请重新确认订阅范围」附旧快照与新结构(继承,US-4) |
| 4 | 上游变更由本 Workspace 用户的上游评论回流触发 | 警示照常产生 + 「由你在上游的评论触发」标注 + 一键静默(继承,REQ-8.3.4) |
| 5 | 同一契约被多个下游订阅 | 各下游独立警示、独立核对 Bundle,互不可见(继承) |
| 6 | 生成核对反馈时下游无受影响引用文档 | 目标文档下拉列出下游全部文档由用户手选;取消则警示保持 open |
| 7 | 循环订阅(A 订 B 的契约,B 订 A 的契约) | 允许;核对反馈不自动级联(REQ-8.4.4),每一环都需人工确认(继承) |
| 8 | 上游项目被整体移除/archived | 其契约的全部订阅转 `source_deleted`;警示历史保留 |
| 9 | 非 Markdown 契约变更 | 文件级警示:无 Block diff,显示 mtime/size 与 git commit(可得时);无法做 Block 级订阅(REQ-8.2.3) |
| 10 | 订阅创建前的历史变更 | 不回溯:以订阅时刻版本为基线,只警示此后的变更 |
| 11 | git pull 批量变更命中多个契约 | 并入 F1 批量汇总(REQ-8.3.3),不逐条轰炸;溯源标 bulk_sync |
| 12 | 核对评论被 reopen | 走 F4 reopened 流程再次回流;警示回 handled,随评论终态联动 |
| 13 | 防抖批次内改了又改回(净零 diff) | 不警示(§6 净零过滤);跨批次改回 → 两次警示,如实反映(第二次 diff 显示恢复) |
| 14 | 警示未处理时上游同一契约再次变更 | 新警示与旧警示合并(diff 基线取旧警示的 from 版本),Inbox 仍一条,详情显示"累计 2 次变更" |
| 15 | 下游项目删除了核对评论所在文档 | 核对评论按 F3-E8 冻结;警示保持 handled 并提示"目标文档已删除,可重新生成" |
| 16 | Xref 目标 Block 被删除 | 降级为文档级引用 + 「原段落已删除」标注,quote 无(Xref 不存 quote,跳转落文档顶部) |

---

## 8. 埋点(opt-in,对齐主 PRD §6)

| 事件 | 属性 | 对应指标 |
|---|---|---|
| `xref_created` | method(explicit/wikilink/auto_confirm)、link_type、cross_block(bool) | 跨项目引用数/Workspace(BRD §9 采用度) |
| `contract_marked` / `contract_subscribed` | scope(document/blocks)、block_count | 契约功能渗透;Q8 粒度校准数据 |
| `drift_alert_shown` | hit_blocks、provenance(feedback-triggered/mcp/external)、aggregated(bool)、self_triggered(bool) | 警示量基线;误报分析分母 |
| `drift_alert_action` | action(handled/dismissed/muted_self)、latency_min | **处理率 ≥60%**;dismissed 率为误报率 <10% 的代理指标 |
| `drift_feedback_sent` | bundle_id、target_doc_manual(bool) | 警示→核对转化 |
| `loop_closed`(F4 事件,属性扩展) | `drift=true` | **北极星**:跨项目闭环占比 |
| `subscription_needs_reconfirm` | trigger(low_confidence/source_deleted) | 订阅健康度;锚定引擎跨项目表现 |

---

## 9. 验收标准与测试要点

继承主 PRD AC-8a/8b/8c,细化:

- **AC-8a-1(端到端主路径)**:server/client 双仓真实场景——标记契约、Block 级订阅 → server 侧经 Claude Code 回流修改契约 → client Inbox 警示在 FS 事件后 10s 内出现,含正确 diff 摘录与 `feedback-triggered` 溯源 → 生成核对反馈 → client 侧 agent 处理并回执 → 复核 resolve → `loop_closed(drift=true)` 上报,警示转 closed。
- **AC-8a-2(非 Markdown 契约)**:订阅 `openapi.yaml`,外部修改后文件级警示出现,含 mtime 变化与 commit hash(git 仓库内)。
- **AC-8b-1(订阅制过滤)**:上游 20 次未命中订阅范围的变更(其他章节/未订阅文档),下游 0 警示。
- **AC-8b-2(聚合)**:一次 git pull 改动 3 个契约共 12 个 Block,下游 Inbox 恰好 1 条批量汇总(内含 3 契约明细),无逐条轰炸。
- **AC-8c-1(迁移)**:复用 AC-3b 四类重写测试集作用于被订阅契约:订阅与 Xref ≥90% 正确迁移或显式转 `needs_reconfirm`,静默失效 = 0。
- **AC-8d-1(自触发)**:由本 Workspace 用户上游评论触发的变更,警示 100% 带自触发标注且可静默;external-unknown 变更 100% 带"来源不明"提示。
- **AC-8e-1(不级联)**:构造 A→B→C 订阅链,A 的变更经核对反馈改了 B,C 收到对 B 的新警示但**无**自动生成的核对 Bundle。
- 测试要点:警示与核对评论状态联动(§7-12/15);净零过滤与跨批次恢复(§7-13);警示合并(§7-14);单项目空态与第二项目导入后的入口出现;`loop_closed` 双属性(drift + closed_via)归因正确。

---

## 10. 依赖与开放问题

**依赖**:F1 多项目与文档身份(前置);锚定引擎四消费方接口(前置,技术设计文档《Block 锚定与迁移契约》定稿);F3 评论状态机(核对评论载体);F4 Bundle 通道与溯源数据(REQ-4.7);Inbox 框架;F7 Workspace 组装(警示→Pack 快捷入口)。

**开放问题**

| # | 问题 | 倾向 |
|---|---|---|
| OQ-1 | 核对评论以用户身份创建(系统预填)是否足够,还是 F3 需要正式的 `origin=drift_check` 评论子类型?影响埋点归因与 UI 徽标 | 倾向:F3 的 comment 表加 `origin` 字段(default=manual),不加子类型;需与 F3 子 PRD 对齐回填 |
| OQ-2 | 订阅默认粒度(承接主 PRD Q8):整文档订阅在高频变更契约上可能吵 | 默认整文档 + 首次警示时提示"可缩小到章节";M0 走查校准 |
| OQ-3 | 契约标志位与 `type: Contract` 的关系:一份 `type: Spec` 的契约导出时 type 取什么? | 倾向:标志位独立,type 不变,导出加 `x-contract: true`(REQ-8.5.2);`Contract` 词表值仅用于原本无 type 的文档 |
| OQ-4 | 免费层单项目不含 F8(BRD §10),免费用户如何感知该价值? | 「上游契约」面板在免费层显示为带说明的锁定态;与增长策略一起定,非本 PRD 范围 |
| OQ-5 | 警示是否需要任何形式的跟进提醒(当前刻意无 48h 机制) | MVP 不做,靠 Inbox 未读常驻;若内测 open 警示大量沉积再议 |
| OQ-6 | 自动发现(REQ-8.1.3)对 repo URL 的匹配规则(同名仓库、fork、路径变体) | MVP 仅匹配本地相对路径与 remote URL 精确匹配;宽松匹配后置 |

---

## 11. 变更记录

| 版本 | 日期 | 变更 |
|---|---|---|
| v0.1 | 2026-07-28 | 初稿:细化主 PRD v0.3 §3-F8(REQ-8.1.1~8.5.2);新增 REQ-8.NEW-1 核对评论机制、REQ-8.NEW-2 订阅方视图、REQ-8.NEW-3 警示留痕;警示状态机、数据模型、16 条边界情况、埋点与 AC 细化 |
