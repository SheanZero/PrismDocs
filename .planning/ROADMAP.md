# Roadmap: PrismDocs

## Overview

从零构建 vibe coder 的多项目工程知识工作台（macOS / Tauri v2）：先落地承载五项不可逆决策的 Rust engine 骨架，再打通坑最密集的文件导入与同步层；在此之上实现全系统心脏——Block 锚定引擎（TD-01，接口冻结点，四消费方共用）；随后并行交付中文速读区与段落评论，两者在 F4 回流闭环汇合——这是核心价值「评论 → AI 修改 → 复核通过」首次可端到端验证的关键路径终点；接着组装卡片、Context Pack 与跨项目知识层（复用 F4 闭环通道，agent 侧零新协议）；最后以压测、埋点、签名公证与锚点 0 静默丢失发布门槛复测收口。关键路径 1→2→3→5→6；Phase 4 与 Phase 5 可并行。

采用《调研_技术基建与开发Phase》v0.2 §4 + 构想 v2 §4 修订的 Phase 划分，并折入调研修正 A1（Phase 1 含事件总线骨架 + IPC 通路证明）、A2（锚定纯核 + 标定 harness 可与 Phase 2 并行启动）、A3（Phase 4 内 prism-llm 传输层先行）。

**跨切预算**：INFRA-04（性能）与 INFRA-05（本地优先/数据安全）自 Phase 1 起作为设计约束在各阶段执行，于 Phase 8 做验证级验收（两条需求映射至 Phase 8）。

## Phases

**Phase Numbering:**

- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: 基建骨架** - Cargo workspace + Tauri 薄 shell + schema v1 + keyring + 事件总线/IPC 通路证明，五项不可逆决策全部落地
- [ ] **Phase 2: F1 导入与同步** - 导入向导、watcher 管线、文档身份与版本快照、回声抑制、frontmatter 字节保真回写（坑最密集阶段）
- [ ] **Phase 3: 锚定引擎 ★** - TD-01 实现与真实 agent diff 标定，MigrationResult/ChangeSet 接口冻结（四消费方），TD-01 v0.2 修订
- [ ] **Phase 4: F2′ 速读区** - prism-llm 传输层先行，中文摘要 + ❓ 决策清单 + 变更条 + 已读基线 + GFM/Mermaid 渲染（可与 Phase 5 并行）
- [ ] **Phase 5: F3 评论** - Block 锚定评论 CRUD/线程/状态机/侧栏/降级可见性/Inbox + 消费方对账（可与 Phase 4 并行）
- [ ] **Phase 6: F4 回流闭环 ★★** - Feedback Bundle 双通道 + MCP server (D-07) + 三信号回收 + 溯源与时间线 + 复核界面，AC-4a 全闭环在此可验证
- [ ] **Phase 7: F5 卡片 + F7 Context Pack + F8 跨项目知识层** - 卡片与原创引导、token 计量的上下文组装、契约订阅与漂移警示、一键下游核对（复用 F4 通道）
- [ ] **Phase 8: 发布准备** - 500 文档/2000 卡片压测、埋点、签名公证全量、锚点 0 静默丢失发布门槛复测、kill -9 数据安全

## Phase Details

### Phase 1: 基建骨架

**Goal**: 可独立测试的 Rust engine workspace + Tauri 薄 shell 就绪，五项不可逆决策（单写者 SQLite + 读池、FTS5 CJK tokenizer、keyring-core 用法、prism-mcp trait 反转、notify-then-fetch）全部落地并各有验证通路
**Depends on**: Nothing (first phase)
**Requirements**: INFRA-01, INFRA-02, INFRA-03
**Success Criteria** (what must be TRUE):

  1. engine workspace 不依赖 tauri 即可 `cargo test` 全绿（D-01）；`cargo tree -d` 无重复 rusqlite/reqwest；prism-mcp 仅依赖注入的 service trait，编译期无 facade↔mcp 依赖环
  2. 事件总线骨架各验证一条通路：一条总线事件经粗粒度 Tauri event 往返前端（notify-then-fetch），一条命令经 Channel 有序流式返回（A1）
  3. SQLite schema v1 落地：WAL + 单写者 + r2d2 读池（query_only=ON）并发读写正常；FTS5 中文查询返回非零结果（CJK tokenizer 在 schema v1 定案）；rusqlite_migration 迁移体系可用，bundled SQLite ≥3.51.3
  4. API key 经 keyring-core + apple-native-keyring-store 写入系统钥匙串并可读回，prism-llm 为唯一网络出口与唯一密钥入口，代码与配置中无明文密钥

**Plans**: 11/13 plans executed

Plans:
**Wave 1**

- [x] 01-01-PLAN.md — 工作区骨架与端到端 tracer（React → Tauri → engine → store → SQLite）+ 全 crate 骨架

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 01-02-PLAN.md — 四条依赖方向断言、明文密钥静态检查与 macOS CI 门禁

**Wave 3** *(blocked on Wave 2 completion)*

- [x] 01-03-PLAN.md — prism-store：迁移体系、schema v1（含 FTS 形态 checkpoint）与 writer-first 连接纪律
- [x] 01-04-PLAN.md — prism-types 服务契约（同步 trait + EngineEvent）与 prism-llm 钥匙串密钥入口

**Wave 4** *(blocked on Wave 3 completion)*

- [x] 01-05-PLAN.md — prism-store 查询层：FTS 长度分流与语法转义、settings 与 base_url 校验
- [x] 01-06-PLAN.md — prism-mcp trait 反转骨架与 Host/Origin/bearer 三层门禁

**Wave 5** *(blocked on Wave 4 completion)*

- [x] 01-07-PLAN.md — prism-engine facade：事件总线与服务 trait 实现

**Wave 6** *(blocked on Wave 5 completion)*

- [x] 01-08-PLAN.md — Tauri shell IPC 双通路：coarse event（含 Lagged→Resync）与 Channel 有序流

**Wave 7** *(blocked on Wave 6 completion)*

- [x] 01-09-PLAN.md — 前端 settings 页与 dev 冒烟页（TanStack Query 失效模式）

**Gap-closure Wave 1** *(波次相对本缺口修补集重新起算，不接续上面的七波)*

- [x] 01-10-PLAN.md — 【Blocker】凭据型 base_url 在 engine 写入路径与前端两道守卫上被拒（gap 1）
- [x] 01-11-PLAN.md — 【Blocker】明文密钥扫描器看得见 `sk-ant-…` 形态 + selftest 自证非恒真（gap 2）
- [ ] 01-13-PLAN.md — WebView 内容安全策略 + 关闭无消费方的资源协议（CR-02）、tracing subscriber 落地（WR-04）

**Gap-closure Wave 2** *(blocked on 01-11 — 同为 prism-mcp 文件)*

- [ ] 01-12-PLAN.md — prism-mcp 空 bearer 由 fail-open 改为构造期即拒 + 比较层纵深（CR-03）

### Phase 2: F1 导入与同步

**Goal**: 磁盘权威的 Base 层导入与实时同步管线可靠运转——文档身份在重命名/外部重写/git 批量操作下保持稳定，为下游所有消费方提供身份与版本层（调研认定的坑最密集阶段，预算按此配置；A2：锚定纯核 + 标定 harness 可在本阶段并行启动）
**Depends on**: Phase 1
**Requirements**: IMPORT-01, IMPORT-02, IMPORT-03, IMPORT-04, IMPORT-05, IMPORT-06, IMPORT-07, IMPORT-08, IMPORT-09
**Success Criteria** (what must be TRUE):

  1. 用户选择本地文件夹/Git 仓库新建项目，glob 可配置扫描导入，20 份 .md 60 秒内可浏览、中文全文搜索可用（AC-1a）
  2. 四种写入模式（VS Code 保存 / vim `:w` / agent Write / `git checkout`）下外部变更 10s 内呈现、无重复变更记录、已有评论数据 100% 保留——原子保存（temp+rename）不被误判为删除（AC-1b，本阶段第一验收）
  3. 重命名/移动经内容哈希识别为同一文档（Document ID 不变），版本快照持久化且被评论/订阅/溯源引用的版本不受保留策略淘汰（AC-1c）
  4. frontmatter 解析入库后写回按字节保留原块，round-trip `git diff` 0 变化（含 CRLF/无尾换行/YAML anchor 样例）（AC-1g-2）
  5. 应用内显式解锁编辑 Base 并写回磁盘不触发回声（write_registered 原语 + hash 登记）；HTML 导入转 Markdown、损失明显时降级附件并提示；git 感知（branch/commit 关联）作为 P0.5 达成或记录降级

**Plans**: TBD
**UI hint**: yes

Plans:

- [ ] TBD

### Phase 3: 锚定引擎 ★

**Goal**: 全系统心脏就位——TD-01 三步迁移管线在真实 agent diff 上标定达标，MigrationResult/ChangeSet 接口冻结供四消费方（F3/F4/F2′/F8）消费；阶段关闭前完成 A→B→A 降级锚点复活语义的 TD-01 v0.2 修订
**Depends on**: Phase 2
**Requirements**: ANCHOR-01, ANCHOR-02, ANCHOR-03, ANCHOR-04, ANCHOR-05
**Success Criteria** (what must be TRUE):

  1. comrak 为唯一 Block 边界真相源解析 Block 树，解析选项 pin 且版本化；前端仅消费 engine 输出的 Block spans，IPC 契约中无全文自行分块入口（TD-01 §3）
  2. AI 重写 50% 内容后，Block ULID 经三步迁移管线（精确对齐/移动检测/相似度匹配）正确迁移或按置信度三档显式降级，标定语料上 silent-wrong = 0（AC-3b 引擎级）
  3. MigrationResult/ChangeSet 输出接口冻结并写入四消费方契约，diff_versions 基线对比入口可用（TD-01 §7）
  4. 标定 harness（参数网格扫描 CLI）在真实 agent diff 语料（Claude Code/Cursor 会话产物，含 CJK 混排）上完成扫描，T_high/T_low 与权重定稿，连同 A→B→A 降级锚点复活语义写入 TD-01 v0.2（TD-01 §6/§11）
  5. 完备性/决定性不变式属性测试通过；每次迁移产生 migration_log 逐条事件日志（TD-01 §8，AC-3b-2）

**Plans**: TBD

Plans:

- [ ] TBD

### Phase 4: F2′ 速读区

**Goal**: 用户 10 分钟内看懂 AI 写的英文文档——每份 Base 生成中文速读区（摘要 + ❓ 决策清单 + 变更摘要），配合 Block 级变更条与已读基线；A3：prism-llm 传输层（流式/重试/keyring）先行交付，速读区功能其后，使 4→6 边只依赖「传输层完成」（可与 Phase 5 并行）
**Depends on**: Phase 3 (可与 Phase 5 并行)
**Requirements**: DIGEST-01, DIGEST-02, DIGEST-03, DIGEST-04, DIGEST-05, DIGEST-06, DIGEST-07, DIGEST-08
**Success Criteria** (what must be TRUE):

  1. 打开英文 Base 文档可见流式生成的中文速读区（3–5 句口语化摘要 + ❓ 需决策清单 + 自上次已读以来的变更摘要）；Base 为中文时跳过提炼仅生成清单与变更摘要；速读区只读，文档级「报告失真」入口可用
  2. ❓ 清单每项 100% 可跳转对应 Base Block 且强制附英文原文摘录不可隐藏；清单为空显示「本次无需你决策」（AC-2c）
  3. Base 变更后防抖自动重生成，同一文档同一时刻至多一个任务，生成中再变更则取消重调度；重启后打开已生成文档 0 次 LLM 调用（缓存键=内容哈希+prompt 版本+语言+模型），异常中断的部分流不入缓存（AC-2b）
  4. 生成前显示预估 token，超项目阈值（默认 ≤5k 自动）转手动确认，消耗计入全局 token 统计
  5. Base 视图完整渲染 GFM + Mermaid；Block 级变更条（新增/修改/删除，数据源为锚定 diff 基线对比）可见，「标记已读」推进基线并更新文档树徽标与 Inbox 未读数

**Plans**: TBD
**UI hint**: yes

Plans:

- [ ] TBD

### Phase 5: F3 评论

**Goal**: 用户批注两句就能表达意见——Block 锚定评论在 AI 大规模重写下 0 静默丢失，降级评论 UI 一等可见，为 F4 回流提供评论源（可与 Phase 4 并行）
**Depends on**: Phase 3 (可与 Phase 4 并行)
**Requirements**: COMMENT-01, COMMENT-02, COMMENT-03, COMMENT-04, COMMENT-05, COMMENT-06
**Success Criteria** (what must be TRUE):

  1. 用户在任意 Block 悬停/选中即可创建评论（≤2 次点击+输入），选中文字作为 quote 存档；四类型（提问/修改要求/approve/reject）与线程回复可用（AC-3c）
  2. AI 重写 50% 内容后 ≥90% 评论锚点正确迁移或显式降级（降级=文档级警示 + quote 快照 + 手动重锚入口，UI 一等可见），0 静默丢失；评论表↔migration_log 消费方对账自动化（AC-3b 消费方级）
  3. 评论状态机 open→sent→needs-review→resolved/reopened 为唯一权威定义，非法状态转移被拒绝
  4. 评论侧栏按文档聚合、按状态/类型可筛选；文档级评论区可用；Inbox 聚合跨文档 needs-review
  5. 评论数据全存本地 sidecar 库，1000 条评论后所有源文件 checksum 0 变化（AC-3c-4）

**Plans**: TBD
**UI hint**: yes

Plans:

- [ ] TBD

### Phase 6: F4 回流闭环 ★★

**Goal**: 核心价值首次端到端可验证——「评论 → AI 修改 → 复核通过」全闭环在真实 Claude Code 与 Cursor 会话中跑通（AC-4a），关键路径终点；阶段关闭后在干净机器上完成签名公证冒烟（CLI helper externalBin 为已知雷区）
**Depends on**: Phase 4, Phase 5
**Requirements**: LOOP-01, LOOP-02, LOOP-03, LOOP-04, LOOP-05, LOOP-06, LOOP-07, LOOP-08, LOOP-09
**Success Criteria** (what must be TRUE):

  1. 一键「回流」生成 Feedback Bundle（每评论含文件路径 + Block 定位 + 类型 + 中文原文 + 英文意图摘要 + 线程上下文，附执行指令头，token ≤全文 30%）原子写入 `.prismdocs/feedback/<timestamp>.md`，「喂给 agent 的一句话」入剪贴板；未装 MCP 仅凭文件协议即可完成闭环（AC-4c）；Bundle 历史留档
  2. 内嵌 MCP server（D-07：loopback streamable HTTP + per-install bearer token 存钥匙串 + Host/Origin 校验）提供 list_feedback/get_feedback/respond_to_comment/get_document_comments，恶意 Origin 请求 403（自动化测试验证）
  3. 三信号（MCP 回执 / 回执文件 / FS 变更命中被评 Block 兜底，等强实现）任一发生 → 评论转 needs-review + Inbox 通知，部分处理 48h 后提示；真实 Claude Code 与 Cursor 会话各完成一次「评论→AI 修改→复核 resolve」全闭环并计入北极星（AC-4a）
  4. 复核界面 Base diff + 评论并排，展示「由谁、因哪条评论触发」（Bundle/MCP 回执方/external-unknown 如实标注）；变更时间线（版本节点=diff+驱动评论+回执+执行者）与任意 Block「这段为什么是这样」历次溯源可用，被引用版本快照不淘汰
  5. Claude Code（headersHelper 读钥匙串 + SessionStart hook）与 Cursor（token 不落 git）安装向导一键生成配置片段与协议说明；阶段关闭后干净机器签名公证冒烟通过

**Plans**: TBD
**UI hint**: yes

Plans:

- [ ] TBD

### Phase 7: F5 卡片 + F7 Context Pack + F8 跨项目知识层

**Goal**: 知识资产层与第二条护城河就位——手写卡片沉淀理解、Workspace 级上下文组装喂给 agent、跨项目契约订阅让独立仓库不产生偏差（F8 复用 F4 的 Bundle 通道，agent 侧零新协议——必须在 Phase 6 之后）
**Depends on**: Phase 6
**Requirements**: CARD-01, CARD-02, CARD-03, CARD-04, CARD-05, CARD-06, PACK-01, PACK-02, PACK-03, PACK-04, PACK-05, XPROJ-01, XPROJ-02, XPROJ-03, XPROJ-04, XPROJ-05, INFRA-06
**Success Criteria** (what must be TRUE):

  1. 评论 resolve 时提示写卡（预填上下文链接）30 秒内完成一张（AC-5a）；卡片列表、CJK 全文搜索、按标签/项目/链接对象筛选可用（AC-5b）；`[[` 双链候选覆盖全 Workspace（跨项目），反链面板显示「谁引用了这张卡」
  2. 原创引导生效：选中存卡时选中内容入引用区（折叠+来源链接）、正文必须另写、高度重复柔性提醒、正文无 AI 代写按钮；「注入上下文」开关 + 英文注入行（可 AI 代译）可配置
  3. Workspace 级树形勾选文档/卡片（可跨项目勾选契约文档）实时显示总 token 与各项占比、超预算高亮，输出结构化 `.prismdocs/context/<name>.md`；常用包存模板、内容变化重生成提示、context-worthy 默认预勾选；MCP 注册 get_context_pack/list_cards；OKF Bundle 导出作为 P0.5 达成或记录降级
  4. 上游契约变更命中被订阅 Block → 下游 Inbox「上游契约变更」警示（上游 diff 摘录+变更溯源+受影响引用方清单+同契约多变更聚合），未订阅变更 0 警示（AC-8b）；一键生成下游核对 Bundle 走 F4 双通道与三信号回收，复核 resolve 计入北极星，agent 侧零新协议
  5. 订阅与 Xref 随锚点迁移 ≥90% 正确或显式降级、0 静默丢失，契约源删除转失效态不静默消失（AC-8c）；needs-review 与漂移警示可经 macOS 系统通知触达（P0.5）

**Plans**: TBD
**UI hint**: yes

Plans:

- [ ] TBD

### Phase 8: 发布准备

**Goal**: 内测可发布——性能与数据安全预算（自 Phase 1 起执行的 INFRA-04/05）验证达标，锚点 0 静默丢失发布门槛端到端复测通过，签名公证 DMG 全量交付
**Depends on**: Phase 7
**Requirements**: INFRA-04, INFRA-05, INFRA-07, INFRA-08
**Success Criteria** (what must be TRUE):

  1. 500 文档/2000 卡片压测通过：全文搜索 <300ms、文档打开 <500ms、FS 变更呈现 <10s、单文档锚点迁移 P95 <300ms（§5，TD-01 §10）
  2. 断网状态可读文档、可写评论、可写卡片（仅 LLM 功能除外）；kill -9 后重启 0 数据丢失；数据库单目录整体备份可恢复
  3. 锚点 0 静默丢失发布门槛（AC-3b）端到端复测通过——全链路（含 UI 降级呈现与对账）而非 engine-only
  4. 签名公证 DMG 全量通过（含 CLI helper externalBin），干净机器下载安装即用；崩溃率 <1%、无数据丢失类 P0 bug（§7 发布标准）
  5. opt-in 本地埋点覆盖北极星漏斗全事件（loop_closed/first_loop_closed/digest_*/xref_*/drift_*/timeline_*/card_created/bundle_*），本地暂存 + 授权导出可验证（§6）

**Plans**: TBD

Plans:

- [ ] TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 ∥ 5 → 6 → 7 → 8（关键路径 1→2→3→5→6；Phase 4 与 Phase 5 可并行）

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. 基建骨架 | 11/13 | In Progress|  |
| 2. F1 导入与同步 | 0/TBD | Not started | - |
| 3. 锚定引擎 ★ | 0/TBD | Not started | - |
| 4. F2′ 速读区 | 0/TBD | Not started | - |
| 5. F3 评论 | 0/TBD | Not started | - |
| 6. F4 回流闭环 ★★ | 0/TBD | Not started | - |
| 7. F5 卡片 + F7 Context Pack + F8 跨项目知识层 | 0/TBD | Not started | - |
| 8. 发布准备 | 0/TBD | Not started | - |

---
*Roadmap created: 2026-07-28*
