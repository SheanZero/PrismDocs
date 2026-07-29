# Requirements: PrismDocs

**Defined:** 2026-07-28
**Core Value:** 「评论 → AI 修改 → 复核通过」的闭环：用户能在 10 分钟内看懂 AI 写的英文文档、批注两句让它接着干、且评论在 AI 大规模重写下 0 静默丢失。

来源映射：本清单由主 PRD v0.3（docs/PRD_PrismDocs_MVP.md）F1–F5/F7/F8 的 P0/P0.5 REQ 直接映射（括号内保留 PRD 编号与 AC 溯源），另含 2 条调研新增（标 ★new，见 .planning/research/FEATURES.md）。标 (P0.5) 的条目为发布前尽力项，允许降级并记录。

## v1 Requirements

### 锚定引擎 ANCHOR（TD-01，护城河）

- [ ] **ANCHOR-01**: 文档解析为 Block 树，comrak 为唯一 Block 边界真相源，解析选项 pin 且版本化（TD-01 §3）
- [ ] **ANCHOR-02**: Block 身份为稳定 ULID，经三步迁移管线（精确对齐/移动检测/相似度匹配）在重写后迁移，置信度三档行为（静默随迁/弱提示/显式降级），静默丢失=0（TD-01 §5–6，AC-3b）
- [ ] **ANCHOR-03**: 一次迁移计算供四消费方（F3 锚点迁移/F4 命中判定/F2 变更条/F8 订阅命中），MigrationResult/ChangeSet 输出接口冻结，含 diff_versions 基线对比入口（TD-01 §7）
- [ ] **ANCHOR-04**: 标定 harness（参数网格扫描 CLI + 真实 agent diff 语料，含 CJK 混排）标定 T_high/T_low 与权重，硬约束 silent-wrong=0，产出 TD-01 v0.2（TD-01 §6/§11）
- [ ] **ANCHOR-05**: 完备性/决定性不变式属性测试 + migration_log 逐条事件日志（TD-01 §8，AC-3b-2）

### 导入与同步 IMPORT（F1）

- [ ] **IMPORT-01**: 用户选择本地文件夹/Git 仓库新建项目，glob 可配置扫描（默认 docs/**、根 *.md、CLAUDE.md、AGENTS.md 等）导入为 Base 层，20 份 .md 60 秒内可浏览（REQ-1.1/1.2，AC-1a）
- [ ] **IMPORT-02**: FS watcher 实时感知增删改移，2s 防抖合并，外部变更 10s 内呈现且无重复变更记录；磁盘为 Base 层权威副本，不锁文件（REQ-1.4/1.5，AC-1b）
- [ ] **IMPORT-03**: 重命名/移动经内容哈希识别为同一文档（Document ID 不变），其上评论 100% 保留（REQ-1.NEW-1，AC-1c）
- [ ] **IMPORT-04**: 文档版本快照持久化并受保留策略管理；被评论/订阅/溯源引用的版本不淘汰（REQ-1.NEW-2）
- [ ] **IMPORT-05**: 已有 YAML frontmatter 解析入库为结构化元数据（受控 type 词表）；写回按字节保留原 frontmatter 块，round-trip 后 `git diff` 0 变化（REQ-1.8，AC-1g-2）
- [ ] **IMPORT-06**: HTML 文档导入转换为 Markdown；转换损失明显时降级为附件并提示（REQ-1.3）
- [ ] **IMPORT-07**: 自写盘回声抑制统一原语（写前登记 hash）；git 批量操作（checkout/rebase）检测；编辑器原子保存（temp+rename）不误判为删除——四种写入模式（VS Code/vim/agent Write/git）下评论 100% 保留（调研 PITFALLS #3/#4/#5）
- [ ] **IMPORT-08**: 应用内可编辑 Base 层（默认只读、显式解锁）并写回磁盘（REQ-1.6）
- [ ] **IMPORT-09** (P0.5): Git 感知（识别 branch、变更关联 commit hash）；项目级 log.md 物化开关（默认关）（REQ-1.7/1.9）

### 速读区 DIGEST（F2′）

- [ ] **DIGEST-01**: 每份 Base 文档生成中文速读区：3–5 句口语化摘要 + ❓ 需决策清单 + 自上次已读以来的变更摘要；Base 为中文时跳过提炼仅生成清单与变更摘要（REQ-2.2/2.3）
- [ ] **DIGEST-02**: ❓ 决策清单项 100% 链接跳转对应 Base Block 并强制附英文原文摘录，不可隐藏；清单为空显示「本次无需你决策」（REQ-2.3/2.6，AC-2c）
- [ ] **DIGEST-03**: Base 变更后防抖自动重生成；同一文档同一时刻至多一个生成任务；流式渲染；失败可重试不损坏数据；生成中再变更则取消重调度（REQ-2.8）
- [ ] **DIGEST-04**: 生成缓存键=内容哈希+prompt 版本+目标语言+模型标识，持久化，重启打开已生成文档 0 次 LLM 调用；不缓存异常中断的部分流（REQ-2.10，AC-2b，调研）
- [ ] **DIGEST-05**: Base 视图 Block 级变更条（新增/修改/删除，数据源为锚定 diff 的基线对比）；「标记已读」推进基线；未读变更数上报文档树徽标与 Inbox（REQ-2.5）
- [ ] **DIGEST-06**: 生成前显示预估 token；项目级自动/手动策略（默认 ≤5k 自动，超限提示）；消耗计入全局 token 统计（REQ-2.9）
- [ ] **DIGEST-07**: 速读区只读，意见通过评论表达；文档级「报告失真」入口（REQ-2.6/2.7）
- [ ] **DIGEST-08** ★new: Base 视图完整 GFM + Mermaid 图渲染（调研：agent 文档大量含 Mermaid，类目可信度 table stakes）

### 段落评论 COMMENT（F3）

- [ ] **COMMENT-01**: 用户可在 Base 视图任意 Block 上评论（悬停按钮/选中浮条），选中文字作为 quote 存档，创建 ≤2 次点击+输入（REQ-3.1，AC-3c）
- [ ] **COMMENT-02**: 评论四类型：提问/修改要求（默认）/approve/reject（可附言），支持线程回复（REQ-3.2/3.3）
- [ ] **COMMENT-03**: 评论状态机 open→sent→needs-review→resolved/reopened，唯一权威定义（REQ-3.3）
- [ ] **COMMENT-04**: AI 重写 50% 内容后 ≥90% 评论锚点正确迁移或显式降级（降级=文档级+警示+quote 快照+手动重锚入口），0 静默丢失；消费方对账（评论表↔migration_log）自动化（AC-3b，调研 PITFALLS #2）
- [ ] **COMMENT-05**: 评论侧栏按文档聚合可筛选（状态/类型）；文档级评论区；Inbox 聚合跨文档 needs-review（REQ-3.5/3.6）
- [ ] **COMMENT-06**: 评论数据存本地 sidecar 库，1000 条评论后源文件 checksum 0 变化（REQ-3.7，AC-3c-4）

### 回流闭环 LOOP（F4 ★核心闭环）

- [ ] **LOOP-01**: 用户一键「回流」生成 Feedback Bundle：每条评论含目标文件路径+Block 定位（标题路径+原文摘录）+类型+中文原文+产品生成的英文意图摘要+线程上下文，附执行指令头（逐条处理/不改无关部分/逐条回执）（REQ-4.1）
- [ ] **LOOP-02**: 文件协议交付：写入项目根 `.prismdocs/feedback/<timestamp>.md`（原子写），复制「喂给 agent 的一句话」到剪贴板；未装 MCP 仅凭文件协议可完成闭环（REQ-4.2，AC-4c）
- [ ] **LOOP-03**: 内嵌本地 MCP server（D-07：app 托管 loopback streamable HTTP + per-install bearer token 存钥匙串 + Host/Origin 校验，恶意 Origin 403 测试）提供 list_feedback/get_feedback/respond_to_comment/get_document_comments（REQ-4.2，§4.2，调研 PITFALLS #6）
- [ ] **LOOP-04**: 三信号闭环回收等强实现（MCP 回执/回执文件/FS 变更命中被评 Block 兜底）→ 评论转 needs-review + Inbox 通知；部分处理 48h 后提示（REQ-4.4，边界）
- [ ] **LOOP-05**: 复核界面：Base diff + 评论并排；复核 resolve 计入北极星闭环，reopen 可追评（REQ-4.4/4.7）
- [ ] **LOOP-06**: Agent 贡献溯源：每次 Base 变更记录触发来源（Bundle/MCP 回执方/external-unknown 如实标注），复核界面展示「由谁、因哪条评论触发」（REQ-4.7）
- [ ] **LOOP-07**: 变更时间线视图（版本节点=diff+驱动评论+回执+执行者）+ 任意 Block「这段为什么是这样」历次变更溯源；被引用版本快照不淘汰（REQ-4.8）
- [ ] **LOOP-08**: Bundle 范围控制（只含评论+被评块+父级标题路径，token ≤全文 30%，可选附全文）；Bundle 历史留档（哪些评论/何时/是否回执）（REQ-4.3/4.6，AC-4b）
- [ ] **LOOP-09**: Claude Code（headersHelper 从钥匙串读 token + SessionStart hook）与 Cursor（token 交付不落 git）双 agent 安装向导，一键生成配置片段与 CLAUDE.md/AGENTS.md 协议说明（REQ-4.2，§4.1/4.3）

### 理解卡片 CARD（F5）

- [ ] **CARD-01**: 卡片=标题+Markdown 正文+双链+标签，无文件夹层级（REQ-5.1）
- [ ] **CARD-02**: 原创引导：「你会怎么向朋友解释」placeholder；选中存卡时选中内容入引用区（折叠+来源链接）正文必须另写，高度重复时柔性提醒；正文无 AI 代写按钮（REQ-5.2）
- [ ] **CARD-03**: `[[` 双链联想候选覆盖全 Workspace（跨项目），反链面板显示「谁引用了这张卡」（REQ-5.3）
- [ ] **CARD-04**: 场景入口：评论 resolve 时提示写卡（预填上下文链接）；Base 视图选中文字存为卡片（REQ-5.4，AC-5a：resolve 入口 30 秒完成一张）
- [ ] **CARD-05**: 「注入上下文」（context-worthy）开关 + 建议附英文一句话注入行（此字段可 AI 代译）（REQ-5.5）
- [ ] **CARD-06**: 卡片列表、全文搜索（CJK 可用）、按标签/项目/链接对象筛选；引用与正文数据层分离（REQ-5.6，AC-5b）

### 上下文组装 PACK（F7）

- [ ] **PACK-01**: Workspace 级树形勾选文档（Base）/卡片（注入行优先），可跨项目勾选契约文档；实时显示总 token 与各项占比，超预算高亮（REQ-7.1，REQ-7.NEW-1）
- [ ] **PACK-02**: 输出写入 `.prismdocs/context/<name>.md`，结构化分节、来源标注、英文倾向（REQ-7.2）
- [ ] **PACK-03**: 常用包存为模板；内容已变化时重生成提示；context-worthy 卡片默认预勾选（REQ-7.3/7.4）
- [ ] **PACK-04**: MCP 提供 `get_context_pack` 与 `list_cards`（注册进 F4 的 MCP server）（REQ-7.5）
- [ ] **PACK-05** (P0.5): 导出合规 OKF Bundle（sidecar 元数据物化 frontmatter + 自动 index.md；Xref 重写为 bundle 间链接 + x-link-type）（REQ-7.6，§2.5）

### 跨项目知识层 XPROJ（F8 ★防偏差）

- [ ] **XPROJ-01**: 跨项目引用 Xref 存 sidecar（link_type ∈ references/depends-on/contract-of）：应用内显式建链 + 卡片双链跨项目候选；正文链接自动发现为 P0.5（REQ-8.1）
- [ ] **XPROJ-02**: 任一文档可标记为契约（type `Contract`/标志位）；下游项目显式订阅，粒度可到 Block；非 Markdown 契约文件级订阅（REQ-8.2）
- [ ] **XPROJ-03**: 上游契约变更命中被订阅 Block → 下游 Inbox「上游契约变更」警示（上游 diff 摘录+变更溯源+受影响引用方清单+同契约多变更聚合）；未订阅变更 0 警示（REQ-8.3，AC-8b）
- [ ] **XPROJ-04**: 一键在下游生成核对反馈 Bundle（上游 diff 摘录+核对指令头），走 F4 双通道与三信号回收，复核 resolve 计入北极星；agent 侧零新协议（REQ-8.4）
- [ ] **XPROJ-05**: 订阅与 Xref 随锚点迁移 ≥90% 正确或显式降级，0 静默丢失；契约源删除转失效态不静默消失（REQ-8.1.5，AC-8c）

### 平台与基建 INFRA

- [ ] **INFRA-01**: Rust engine workspace（不依赖 tauri、可独立测试）+ Tauri 薄 shell + 事件总线骨架（notify-then-fetch 粗粒度事件 + Channel 有序流各验证一条通路）；prism-mcp 经 service trait 反转解依赖环（D-01，调研 A1/ARCHITECTURE）
- [ ] **INFRA-02**: SQLite WAL 单写者+r2d2 读池（query_only）架构；FTS5 含 CJK 可用 tokenizer（schema v1 时定）；rusqlite_migration 迁移体系；bundled SQLite ≥3.51.3（调研 Phase 1 不可逆决策）
- [ ] **INFRA-03**: API key 存系统钥匙串（keyring-core + apple-native-keyring-store）；支持 Anthropic/OpenAI 兼容端点与自定义 base_url；prism-llm 为唯一网络出口与唯一密钥入口（§5，调研 STACK）
- [ ] **INFRA-04**: 性能达标：500 文档/2000 卡片下全文搜索 <300ms、文档打开 <500ms、FS 变更呈现 <10s、单文档锚点迁移 P95 <300ms（§5，TD-01 §10）
- [ ] **INFRA-05**: 本地优先：断网可读可评可写卡（LLM 功能除外）；数据库单目录可备份；kill -9 不丢数据（§5）
- [ ] **INFRA-06** ★new (P0.5): macOS 系统通知承载 needs-review 与漂移警示（Inbox 之外的 OS 级触达）（调研 FEATURES）
- [ ] **INFRA-07**: 签名公证 DMG（Phase 6 后冒烟、发布前全量）；崩溃率 <1%；无数据丢失类 P0 bug（§7 发布标准）
- [ ] **INFRA-08**: opt-in 本地埋点（本地暂存+授权导出）覆盖北极星漏斗：loop_closed/first_loop_closed/digest_*/xref_*/drift_*/timeline_*/card_created/bundle_*（§6）

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### 全文 Lens（回归条件预先写死，见 BRD §6.1）

- **LENS-01**: 按 Block 全文口语化投影 + 1:1 锚定 + 三视图对照
- **LENS-02**: 增量重投影与逐块缓存、逐段流式任务队列
- **LENS-03**: 逐段「报告失真」与金标集评测基建

### F6 Chrome 剪藏（P1 第一批）

- **CLIP-01**: 整页/选区/框选抓取，净化为 Markdown（代码块无噪音）
- **CLIP-02**: token 估算面板 + 剪藏收件箱 + loopback WebSocket 桥接与离线队列

### 其他

- **MISC-01**: 语义级漂移检测（LLM 比对上下游断言矛盾）
- **MISC-02**: 日文/英文速读区
- **MISC-03**: 文档间图谱视图
- **MISC-04**: 团队版因果追溯（共享变更时间线 + git 便携模式，PRD Q9）
- **MISC-05**: GitHub PR 集成；决策日志自动抽取；卡片间隔回顾
- **MISC-06**: MCP `export_okf_bundle` / `list_dependencies` / `get_upstream_contracts`
- **MISC-07**: Windows 平台；Mac App Store（P2）

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| IDE/代码编辑器/代码生成/内置编码 agent | 定位为 agent 之外面向人的知识层，只对接不替代 |
| 通用笔记应用 | 不与 Obsidian 抢"人生笔记"，只做工程语境 |
| 对外发布文档站 | 不与 Mintlify/GitBook 竞争 |
| 团队协作与权限体系（MVP） | P2/P3 人群后置；架构上不排斥 |
| 理解层（速读区/Lens）直接编辑 | 产品原则：投影只读，意见走评论，从根上防双层漂移 |
| 文档问答 RAG 面板 | 与用户自己的编码 agent 重复，且与「提问评论」路由冲突（调研反功能） |
| 每次修订 AI 自动评审 | 单人场景警报疲劳陷阱（调研反功能，对齐 BRD 护栏） |
| AI 代写卡片正文 | 理解留存的方法论根基，刻意缺失 |
| 服务端/云同步（MVP） | 本地优先零服务端；文档不经过我方服务器 |
| 第二个 Markdown parser 参与锚定 | comrak 唯一真相源，双 parser 必然锚点漂移 |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| INFRA-01 | Phase 1 | Gaps Found |
| INFRA-02 | Phase 1 | Gaps Found |
| INFRA-03 | Phase 1 | Pending |
| IMPORT-01 | Phase 2 | Pending |
| IMPORT-02 | Phase 2 | Pending |
| IMPORT-03 | Phase 2 | Pending |
| IMPORT-04 | Phase 2 | Pending |
| IMPORT-05 | Phase 2 | Pending |
| IMPORT-06 | Phase 2 | Pending |
| IMPORT-07 | Phase 2 | Pending |
| IMPORT-08 | Phase 2 | Pending |
| IMPORT-09 | Phase 2 | Pending |
| ANCHOR-01 | Phase 3 | Pending |
| ANCHOR-02 | Phase 3 | Pending |
| ANCHOR-03 | Phase 3 | Pending |
| ANCHOR-04 | Phase 3 | Pending |
| ANCHOR-05 | Phase 3 | Pending |
| DIGEST-01 | Phase 4 | Pending |
| DIGEST-02 | Phase 4 | Pending |
| DIGEST-03 | Phase 4 | Pending |
| DIGEST-04 | Phase 4 | Pending |
| DIGEST-05 | Phase 4 | Pending |
| DIGEST-06 | Phase 4 | Pending |
| DIGEST-07 | Phase 4 | Pending |
| DIGEST-08 | Phase 4 | Pending |
| COMMENT-01 | Phase 5 | Pending |
| COMMENT-02 | Phase 5 | Pending |
| COMMENT-03 | Phase 5 | Pending |
| COMMENT-04 | Phase 5 | Pending |
| COMMENT-05 | Phase 5 | Pending |
| COMMENT-06 | Phase 5 | Pending |
| LOOP-01 | Phase 6 | Pending |
| LOOP-02 | Phase 6 | Pending |
| LOOP-03 | Phase 6 | Pending |
| LOOP-04 | Phase 6 | Pending |
| LOOP-05 | Phase 6 | Pending |
| LOOP-06 | Phase 6 | Pending |
| LOOP-07 | Phase 6 | Pending |
| LOOP-08 | Phase 6 | Pending |
| LOOP-09 | Phase 6 | Pending |
| CARD-01 | Phase 7 | Pending |
| CARD-02 | Phase 7 | Pending |
| CARD-03 | Phase 7 | Pending |
| CARD-04 | Phase 7 | Pending |
| CARD-05 | Phase 7 | Pending |
| CARD-06 | Phase 7 | Pending |
| PACK-01 | Phase 7 | Pending |
| PACK-02 | Phase 7 | Pending |
| PACK-03 | Phase 7 | Pending |
| PACK-04 | Phase 7 | Pending |
| PACK-05 | Phase 7 | Pending |
| XPROJ-01 | Phase 7 | Pending |
| XPROJ-02 | Phase 7 | Pending |
| XPROJ-03 | Phase 7 | Pending |
| XPROJ-04 | Phase 7 | Pending |
| XPROJ-05 | Phase 7 | Pending |
| INFRA-06 | Phase 7 | Pending |
| INFRA-04 | Phase 8 | Pending |
| INFRA-05 | Phase 8 | Pending |
| INFRA-07 | Phase 8 | Pending |
| INFRA-08 | Phase 8 | Pending |

注：INFRA-04/05 为跨切预算，自 Phase 1 起作为设计约束执行，映射至完成验证的 Phase 8（GSD 规则：每条需求映射且仅映射一个 Phase）。

**Coverage:**

- v1 requirements: 61 total（勘误：初稿 Coverage 误记为 51，实际 REQ-ID 计数为 61，2026-07-28 roadmap 创建时更正）
- Mapped to phases: 61
- Unmapped: 0 ✓

---
*Requirements defined: 2026-07-28*
*Last updated: 2026-07-28 after roadmap creation (traceability filled, coverage corrected 51→61)*
