# 调研:技术基建与开发 Phase 划分

| 项目 | 内容 |
|---|---|
| 文档类型 | 技术调研与开发计划建议 |
| 版本 | v0.2(新增 §3 范围修订:Lens 降级为速读区 MVP′,并同步修订 Phase 划分与 M0) |
| 日期 | 2026-07-28 |
| 作用对象 | BRD v0.2、主 PRD v0.2、七份子 PRD(评审输入);后续技术设计文档与排期(输出) |
| 上游文档 | 全部 11 份需求文档(BRD / 主 PRD / 调研补充 / sub-prds F1–F7) |
| 作者 | Shean(调研与起草协作:Claude) |

---

## 1. 文档集评审结论

### 1.1 总体判断

文档集可以进入技术设计阶段。突出优点:

- **闭环叙事一致**:每份子 PRD 第 1 节都回答「本功能对北极星(闭环数/周)的贡献路径」,可直接用于排期取舍。
- **状态机有唯一权威定义**(F3 REQ-3.3.2 迁移表);F4 三信号幂等回收(MCP 回执 / Receipt 文件 / FS 变更兜底)设计完整,降级路径(AC-4c 无 MCP 也能闭环)明确。
- **产品原则落到了数据层**:Lens 不可编辑、引用与正文分表(AC-5b)、Block ID 不写源文件、`.prismdocs/**` 强制排除防回导。
- 「锚点 0 静默丢失」被正确设为发布门槛(主 PRD §7-3)。

### 1.2 问题清单(按严重度)

**P0 — 开发前必须解决**

| # | 问题 | 说明与建议 |
|---|---|---|
| 1 | **MCP 传输方案文档漂移** | F4 子 PRD REQ-4.NEW-1 与 §5.3 写的是「stdio 轻量代理优先 + HTTP/SSE 兼容,端口 127.0.0.1:23816,`.prismdocs/mcp.json` 端口发现」;已定决策 **D-07** 是「app 自身托管 rmcp `StreamableHttpService`,loopback streamable HTTP + per-install bearer token + Origin allowlist,无子进程」。两者的 `.mcp.json` 形态、hook 命令、桌面端未运行时的错误路径都不同。**建议以 D-07 为准回改 F4 子 PRD**。注意 D-07 下仍需一个小 CLI helper(Claude Code `headersHelper` 从 Keychain 读 token + hook 的 check-feedback),职责需在技术设计中明确。 |
| 2 | **Block 锚定迁移算法无承载文档** | 它是护城河、是 AC-3b「0 静默丢失」的实现体,目前只散落在主 PRD §2.4 与 F3 §6 行为契约;阈值 T_high/T_low 待 M0 真实数据标定(F3 OQ-1)。F1/F2/F3/F4 共用这套契约(F2 §10 要求「接口需先冻结」)。**这是第一份要写的技术设计文档。**(v0.2 注:MVP′ 下锚定引擎一行不少——评论的 AC-3b 与变更高亮都依赖它,与是否有全文 Lens 无关。) |
| 3 | **23 条 REQ-x.NEW 悬而未决** | 其中 REQ-1.NEW-1(文档身份识别)、REQ-1.NEW-2(版本快照与存储上限)、REQ-7.NEW-1(token 预算)是 schema 级输入,不能等 v0.3。建议先做一次 triage:影响 schema/接口的先批,纯交互类留 v0.3。 |

**P1 — 影响排期与验收**

| # | 问题 | 说明与建议 |
|---|---|---|
| 4 | Lens 质量验收需要评测基建 | AC-2a-2(30 段双人 5 级量表)、AC-2a-3(20 份金标集)需要金标集 + 评测脚本 + prompt 版本管理。**v0.2 修订:MVP′ 下缩减为速读区评测(摘要忠实度 + ❓ 决策点精确率),全文 Lens 金标集随其回归 P1 一并延后。** |
| 5 | UI/UX Spec 缺位 | Inbox 是日常主入口(主 PRD §2.3),复核 diff 并排是交互最重的部分,目前只有 ASCII 线框。建议 F3/F4 开发前补轻量设计规格。(v0.2 注:三视图/对照连线随全文 Lens 延后,设计规格范围相应缩小。) |
| 6 | 5 个阻塞性开放问题 | Q1 模型分档(v0.2 修订:范围缩为「速读区用哪档」,M0 解决)、F3 OQ-2/OQ-3(reopened 默认入包?Approve 轻量路径?——阻塞 F4 状态机)、F4 OQ-1(declined 语义)、F7 Q3(MCP 拉取 staleness 标注)。均有倾向意见,拍板即可。 |

**P2 — 措辞与低风险**

| # | 问题 | 建议 |
|---|---|---|
| 7 | 「双向同步」措辞(F1 OQ-3) | 统一为「磁盘权威 + 可选写回」。 |
| 8 | F6 OQ-1(native messaging vs 本地端口) | stack 决策已选 loopback WebSocket + 配对 token,与子 PRD 倾向一致,建议正式关闭该 OQ。 |
| 9 | 埋点无服务端的落地方式 | 仅 F6 §8 提到「本地暂存、内测期经授权导出」,建议提升为主 PRD 级约定。 |

---

## 2. 技术基建

版本 pin 沿用项目 CLAUDE.md 的 Technology Stack(2026-07-28 对照 crates.io 验证),本节不复述版本表,只记录架构综合、需求覆盖核对与工程决策。

### 2.1 总体架构

```
┌─ Chrome 扩展 (WXT + @mozilla/readability + turndown + gpt-tokenizer) ─┐
│         loopback WebSocket + 配对 token(离线队列在扩展侧)              │
└──────────────────────────────┬────────────────────────────────────────┘
                               │
┌─ PrismDocs.app (Tauri v2) ───▼────────────────────────────────────────┐
│  React 19 + Vite 7 WebView(CodeMirror 6 编 Base;                     │
│  react-markdown 渲 Base/速读区——仅渲染,不做锚定真相源)                 │
│  ────────── Tauri IPC(薄 shell:路径注入 / 窗口 / 托盘,不含业务) ──── │
│  Engine Facade(纯 Rust workspace,不依赖 tauri,可独立测试;D-01)      │
│   ├─ prism-store   rusqlite 0.40 + r2d2 池 + FTS5 + rusqlite_migration │
│   ├─ prism-fs      notify 8 + debouncer-full(2s 窗口/10s 上限)+ 对账 │
│   ├─ prism-parse   comrak 0.54(sourcepos)→ Block 树                  │
│   ├─ prism-anchor  blake3 内容哈希 + similar 3.1 迁移  ★护城河          │
│   ├─ prism-llm     reqwest 0.13 + async-openai + eventsource-stream    │
│   │                + keyring 4.1(唯一网络出口 + 唯一密钥入口,NFR-03) │
│   └─ prism-mcp     rmcp 2.2 StreamableHttpService,挂 axum 0.8          │
│                    (127.0.0.1 + bearer token + Origin allowlist,D-07)│
│  同一 axum 实例同时承载扩展的 clip WebSocket 端点                        │
└───────────────────────────────────────────────────────────────────────┘
  sidecar 数据:~/Library/Application Support/PrismDocs/(dirs::data_dir
  计算,非 app_data_dir),按 project-id 索引(D-13,不按路径);
  用户 repo 内仅 .prismdocs/ 协议产物(feedback/ context/ README.md)
```

### 2.2 PRD 需求 → 组件覆盖核对(关键行)

| PRD 需求 | 承载组件 | 核对结论 |
|---|---|---|
| §2.4 Block 锚定 + AC-3b | comrak(sourcepos)+ blake3 + similar + 位置启发式 | 覆盖。**comrak 是唯一锚定真相源**,前端 remark 仅渲染——两个 parser 各自分块必然锚点漂移(What-NOT-to-use 首条)。 |
| REQ-1.4 防抖 2s/上限 10s + 事件合并语义 | notify-debouncer-full | debouncer 只给合并原语;REQ-1.4.3 语义表(新增+删除=忽略等)与 REQ-1.4.5 的 5 分钟对账均为自建逻辑。 |
| REQ-1.8 frontmatter 往返一致(AC-1g-2:`git diff` 0 变化) | gray_matter | **有风险**:解析没问题,但写回必须按字节保留原 frontmatter 块、只替换正文——不能 parse 后再 serialize。技术设计写死该策略。 |
| REQ-2.8 逐段流式 | eventsource-stream + 自建按 Block 任务队列 | v0.2 修订:逐段流式随全文 Lens 移入 P1;MVP′ 下 eventsource-stream 仍用于速读区单次调用的流式渲染与 F4 意图摘要,按 Block 任务队列延后。 |
| REQ-4.2 MCP 双通道 | rmcp + axum(D-07) | 覆盖,但 F4 子 PRD 需按 §1.2-#1 回改。 |
| REQ-6.5 扩展桥接 + 离线队列 | axum WS 端点 + chrome.storage 队列 | 覆盖;MV3 service worker 空闲被杀 → 唤醒重连是扩展侧主要工程点。 |
| AC-6c/AC-7b token 口径一致(≤10%) | 扩展 gpt-tokenizer / 桌面 tiktoken-rs,同为 o200k_base | 口径一致;UI 注明「按 o200k 口径估算」,Claude 精确值走 `messages/count_tokens`。 |
| §5 性能(500 文档搜索 <300ms) | SQLite FTS5 | 轻松达标,无额外服务。 |
| §5 密钥 | keyring 4.1 直连(service=PrismDocs;account:LLM key、MCP token) | 覆盖;不用 tauri-plugin-stronghold(v3 弃用)与 tauri-plugin-keyring。 |

### 2.3 技术设计阶段需钉死的三个工程决策

1. **锚定计算「一次计算三用途」**(F4 §7.3 已点出):FS 变更后的 Block 级 diff 同时产出 ① 锚点迁移结果(F3)② 被评 Block 命中判定(F4 兜底回收)③ 受影响 Block 集(MVP′ 下供给 Base 视图变更条与速读区重生成触发;全文 Lens 回归后供给增量重投影)。prism-anchor 的输出接口是全系统心脏,最先冻结。
2. **自写盘「回声识别」统一原语**:应用内编辑(REQ-1.6.2)、log.md 物化(REQ-1.9.4)、feedback/context 写入,三处自写盘都要「写前登记 hash」防自触发——应在 prism-fs 做成统一原语,不是三处各写一套。
3. **macOS 文件访问策略**:MVP 明确直发(公证 DMG),不进 App Sandbox,普通文件选择器授权即可;F1 §10 的「安全书签持久化」仅 Mac App Store 需要,MAS 列 P2,省掉一整类沙盒问题。

---

## 3. 范围修订:Lens 降级为速读区(MVP′)★ v0.2 新增

### 3.1 决策与理由

2026-07-28 讨论决定:**MVP 不做按 Block 的全文 Lens 投影,保留并强化「中文速读区」**。修订后:

> **MVP′ = F1(导入同步)+ F2′(中文速读区 + 变更高亮)+ F3(评论,在 Base 上)+ F4(回流闭环)+ F5/F6/F7(不变)**

理由链:

1. **全文 Lens 是最大成本与不确定性中心**:逐块调用、增量重投影调度、逐块缓存、三视图对照、忠实度评测基建(金标集/双人量表/prompt 版本管理)——占 F2 复杂度约 3/4。
2. **速读区是 Lens 价值密度最高的部件**:「AI 改了什么、需要你拍板什么」直接回应痛点 #1(审查疲劳)与 #7 的大半;每文档一次调用,成本结构简单,token 焦虑叙事更好讲。
3. **差异化不塌**:对比 markupmarkdown/Plannotator(单层英文、无中文层),「中文速读区 + 决策清单 + 中文评论回流」仍是独有组合;P1 画像是「英文阅读中等」而非「不能读」,速读区 + 定位跳转正好补在能力线上。代价:P2(创始人)人群暂时放弃,待全文 Lens 回归再覆盖。
4. **护城河照建**:锚点迁移引擎是评论(AC-3b)与变更高亮的依赖,与全文 Lens 无关,一行不少。
5. **MVP′ 本身是一个实验**:内测数据将直接回答「全文 Lens 是否必需」(见 §3.5 回归条件),避免在未验证前投入最贵的部件。

**明确否决的两个更激进方案**:① 纯文件管理(F1 为核心)——F1 是激活管线不是价值,免费替代品遍地,无付费市场;② 完全去除中文层只留评论回流——退化为与 markupmarkdown/Plannotator 正面竞争,BRD 立论的核心人群痛点(#7)不再被解决。

### 3.2 保留 / 砍掉 / 延后对照

| 类别 | 内容 |
|---|---|
| **保留(MVP′)** | 速读区(原 REQ-2.3,升级为独立功能 F2′);变更高亮与已读基线(原 REQ-2.5,改挂 **Base 视图**,数据源为锚定 diff,不依赖投影);❓ 决策清单项强制附 Base 原文摘录(继承 REQ-2.6.2 精神);成本控制/token 预估(REQ-2.9 适配为速读区口径);生成失败重试、缓存持久化重启不重算 |
| **延后至 P1(随全文 Lens 回归)** | 按 Block 全文投影与 1:1 锚定(REQ-2.1)、增量重投影与逐块缓存(REQ-2.4)、三视图/对照分栏(REQ-2.1.2)、逐段流式与任务队列(REQ-2.8、REQ-2.NEW-1)、逐段「报告失真」与展开原文(REQ-2.6.1/2.6.3,速读区保留轻量版)、AC-2a 全文理解 80% 指标及其金标集评测 |
| **不变** | Lens 不可编辑原则(速读区同样只读,不满通过评论表达);F3 评论直接锚定 Base Block(本就无需坐标换算);F4 全部;F5/F6/F7 全部;锚定引擎全部 |

### 3.3 速读区(F2′)规格要点

- 内容 = 3–5 句中文摘要 + ❓ 需决策清单(每项链接跳转 Base Block,**强制附原文摘录**)+ 「自上次已读以来的变更摘要」(基于锚定 diff 生成,列出变更 Block 与一句话说明)。
- 生成粒度:每文档一次 LLM 调用;Base 变更后防抖重生成(复用 F1 的 2s 防抖节奏);流式渲染;缓存键 = 文档内容哈希 + prompt 版本 + 模型标识,持久化。
- 阅读视图:仅 Base(带速读区头部 + Block 级变更条);中文评论交互不变。
- 成本策略:沿用 REQ-2.9 框架(≤5k token 自动、超过提示确认;消耗计入全局统计)。
- 轻量忠实度防线:速读区整体提供「报告失真」入口(不做逐段);❓ 清单项的原文摘录是主要防线。

### 3.4 对上游文档的影响(需回写清单)

| 文档 | 需要的修订 |
|---|---|
| BRD §6.1/§9 | 双层文档描述加「MVP 分两步:速读区先行,全文 Lens P1」;成功指标「看 Lens 能懂 80%」替换为速读区口径(如「速读区 + 原文足以完成 review 决策」认同率);北极星不变 |
| 主 PRD §3-F2 | 拆分:REQ-2.3/2.5/2.6.2/2.9 保留并重编号为 F2′;其余标注 P1;§7 发布标准同步 |
| 子 PRD F2 | 出 v0.2:按 §3.2 对照表拆分;AC-2a 系列延后,新增速读区验收(见 §4 Phase 4 退出标准) |
| 子 PRD F4 | 复核界面(§6.3)改为「Base diff + 评论并排」,去掉 Lens 重投影列;其余不变 |
| 子 PRD F5 | REQ-5.4.2「Lens 上选中存卡」改为 Base 上选中(引用区仅存 Base 原文,OQ-5.3 随之消解) |

### 3.5 全文 Lens 的回归条件(数据驱动,预先写死)

满足任一即启动全文 Lens(按原 F2 子 PRD,P1 第一优先):

1. M0 走查或 M2 内测中,≥1/3 用户明确反馈「速读区不够,正文读不动」且该反馈与闭环流失点吻合;
2. 速读区版的激活指标(7 日首闭环 ≥40%)未达标,且漏斗诊断显示流失发生在「打开文档 → 创建首条评论」之间;
3. P2 人群出现明确付费拉力(访谈或 waitlist 信号)。

反之,若内测无上述信号,全文 Lens 持续后置,节省的成本投向多语言速读区(日文)或图谱视图。

---

## 4. 开发 Phase 划分(v0.2 修订)

关键路径:**Phase 1 → 2 → 3 → 5 → 6**;Phase 4(F2′ 速读区)规模缩为原 F2 的约 1/4,建 prism-llm 通道后即可与 Phase 5 并行,仅需先于 Phase 6(F4 意图摘要复用 LLM 通道)。Phase 6 完成即核心假设可验证(AC-4a 全闭环)。F6 扩展全程可并行;F5/F7 在 Phase 5 后可与 Phase 6 并行。

| # | Phase | 内容 | 前置 | 退出标准(锚定 AC) |
|---|---|---|---|---|
| 0 | **M0 概念验证**(1–2 周,可与 Phase 1 并行) | 手工跑通「英文 doc→中文速读区→评论→回流 Claude Code」;速读区 prompt 原型 + ❓ 决策点精确率评测(Q1 缩围拍板);用真实 Claude Code diff 攒锚点标定集;**走查同时收集「全文 Lens 是否必需」证据(§3.5)** | 无 | Q1(速读区档位)拍板;T_high/T_low 初值;5 个目标用户走查(BRD M0) |
| 1 | **基建骨架** | Cargo workspace 7 crate + Tauri 薄 shell + React 壳;schema v1 + 迁移 + WAL/池;keyring 双 account;settings 页(API key/base_url,可跳过) | — | app 启动、迁移执行、钥匙串往返、`cargo tree -d` 无重复 rusqlite |
| 2 | **F1 导入与同步** | 三步向导、glob 预览、HTML 分级转换(htmd)、watcher+防抖+对账、文档身份识别(REQ-1.NEW-1)、版本快照(REQ-1.NEW-2)、frontmatter 解析、FTS5、`.prismdocs/` 初始化 | 1 | AC-1a/1b/1c + AC-1g-2 |
| 3 | **锚定引擎 ★** | comrak Block 树 + blake3 ID + similar 迁移 + 置信度三档行为 + 事件日志;对 F3/F4/变更条 的三路输出接口冻结 | 2 | AC-3b-1 四类重写场景 ×20 例:≥90% 正确迁移或显式降级,静默丢失 = 0 |
| 4 | **F2′ 速读区**(v0.2 缩围) | prism-llm 通道(流式/重试/429 降速)、速读区 prompt 与缓存、❓ 决策清单 + 强制原文摘录、Base 视图变更条与已读基线、成本控制 | 3(可与 5 并行) | 速读区生成与缓存重启 0 调用;❓ 清单 100% 附原文摘录(AC-2c 精神);变更条与锚定 diff 一致;❓ 精确率 ≥90%(M0 评测集) |
| 5 | **F3 评论** | 评论 CRUD/quote 快照/四类型/线程/状态机、侧栏与筛选、降级警示+手动重锚、Inbox 框架 | 3(可与 4 并行) | AC-3a(Base 视图锚定正确)、AC-3c-1~4(含 1000 条评论后源文件 checksum 0 变化) |
| 6 | **F4 回流闭环 ★★** | Bundle 生成+意图摘要(可降级)、文件协议+原子写、MCP server(D-07 形态)+ 安装向导、三信号回收、溯源(REQ-4.7)、**复核界面(Base diff + 评论并排)**、Bundle 历史/重发/撤回、48h 提醒 | 4+5 | **AC-4a-1/4a-2 真实项目、Claude Code 与 Cursor 双 agent 跑通**;AC-4b/4c/4d |
| 7 | **F5 卡片 + F7 Context Pack** | 卡片 CRUD/双链/反链/原创引导/注入行(引用区存 Base 原文);组装器/token 条/模板/预算;`get_context_pack`+`list_cards` 注册进已有 MCP;OKF 导出(P0.5) | 5;6(MCP 注册) | AC-5a/5b/5d、AC-7a/7b/7c |
| 8 | **F6 Chrome 扩展** | WXT 骨架、Readability+turndown 净化(代码块自定义规则)、站点 adapter、token 估算、WS 桥接+配对、离线队列;桌面端剪藏收件箱 | 桥接契约 + clip schema;**其余全程可并行** | AC-6a-1(SO 代码块 100% 无噪音)、AC-6b/6c、AC-6d-1 |
| 9 | **发布准备(M2 前)** | 500 文档/2000 卡片压测、埋点(本地暂存+授权导出)、签名/公证/打包、扩展商店材料、内测运营支撑 | 6–8 | 主 PRD §7 发布标准五条(F2 相关项按 F2′ 口径),崩溃率 <1%、锚点专项复测 |
| P1 | **全文 Lens 回归**(条件触发,见 §3.5) | 原 F2 子 PRD 全量:逐块投影、增量重投影、三视图、逐段失真报告、金标集评测 | 9 + §3.5 任一条件成立 | 原 AC-2a/2b/2c/2d 全套 |

**排期影响**:相比 v0.1,关键路径上 Phase 4 从「原 F2 全量」缩为 1/4 规模,MVP′ 预计提前 2–3 周;BRD M1 的 6–8 周(对应 Phase 1–6)从偏乐观变为基本可达。

---

## 5. 开工前决断清单(按顺序)

1. 回改 F4 子 PRD 的 MCP 传输节,与 D-07 对齐(或推翻 D-07——二选一,不留两个版本)。
2. 按 §3.4 清单回写 MVP′ 范围修订(BRD、主 PRD、子 PRD F2/F4/F5)。
3. Triage 23 条 REQ-x.NEW,先批 REQ-1.NEW-1/2、REQ-7.NEW-1 等 schema 级条目。
4. 拍板 4 个阻塞性 OQ(Q1 已缩围为速读区档位、F3 OQ-2/3、F4 OQ-1、F7 Q3)。
5. 写第一份技术设计文档:**Block 锚定与迁移契约**(输入输出接口、置信度语义、三路消费方)——Phase 3 的输入,也是全系统接口冻结点。

---

## 6. 变更记录

| 版本 | 日期 | 变更 |
|---|---|---|
| v0.1 | 2026-07-28 | 初稿:文档集评审、技术基建覆盖核对、9 Phase 划分、开工前决断清单 |
| v0.2 | 2026-07-28 | 新增 §3 范围修订(Lens 降级为速读区 MVP′):决策理由、保留/延后对照、F2′ 规格要点、上游文档回写清单、全文 Lens 回归条件;Phase 表按 MVP′ 修订(Phase 4 缩围、新增 P1 回归行);§1.2 #4/#5/#6、§2.2 REQ-2.8 行、§2.3-1、决断清单同步标注 |
