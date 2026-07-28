# Phase 1: 基建骨架 - Context

**Gathered:** 2026-07-28
**Status:** Ready for planning

<domain>
## Phase Boundary

可独立测试的 Rust engine workspace（facade + 6 domain crates + prism-types + CLI helper 占位）+ Tauri 薄 shell + React 壳就绪。五项不可逆决策全部落地并各有验证通路：单写者 SQLite + r2d2 读池（query_only）、FTS5 CJK tokenizer（本讨论定案为 trigram）、keyring-core 用法、prism-mcp trait 反转、notify-then-fetch + Channel 双 IPC 通路（A1）。

Requirements: INFRA-01, INFRA-02, INFRA-03。不含任何 F1–F8 功能——导入/同步是 Phase 2，锚定是 Phase 3。INFRA-04/05（性能/数据安全）自本阶段起作为设计约束执行，验证级验收在 Phase 8。

</domain>

<decisions>
## Implementation Decisions

### FTS5 CJK tokenizer（schema v1 定案项）
- **D-01:** FTS5 用 **trigram 统一单索引**——一张 FTS 表、一套查询逻辑，中英文都走 substring 匹配，天然覆盖 CJK 混排。不做 unicode61/trigram 双索引。索引体积 ~3× 与英文无词干提取是已接受的代价（500 文档/2000 卡片规模下 <300ms 预算轻松达标）。 — **Reversibility:** one-way — INFRA-02 将 tokenizer 钉在 schema v1；事后更换 = 全量重建 FTS 索引 + 查询层重写，且 Phase 2+ 的搜索行为契约（AC-1a 中文搜索）已依赖它。
- **D-02:** 短查询（<3 字符，典型为 2 字中文词如「评论」「锚点」）**自动降级 LIKE '%xx%' 线性扫描**；≥3 字符走 trigram MATCH。查询层按长度分流，用户无感。500 文档规模下全扫远在 300ms 预算内。

### Schema v1 覆盖范围
- **D-03:** **最小骨架 + 逐 phase 增量迁移**：schema v1 只建 Phase 1 验证所需，后续每个 phase 用 rusqlite_migration 追加自己的表。不做全量领域 schema 一次落地——comments/cards 等表在真实需求出现前不做推测式设计。迁移体系本身就是 Phase 1 要验证的能力。
- **D-04:** 最小集边界 = **projects、documents（含内容列供 FTS 验证）、FTS 表、settings**。document_versions/blocks 等留给 Phase 2/3 自己的迁移（快照保留策略、引用不淘汰等字段需求那时才明确）。
- **D-05:** 非密钥配置（base_url、模型标识、项目阈值等）存 **SQLite settings 表**（k/v）——随库整体备份（INFRA-05「数据库单目录整体备份可恢复」）、单一真相源、事务一致。密钥仍走钥匙串，绝不入库。

### 薄 shell 前端程度
- **D-06:** 前端交付两块：**settings 页**（API key 写钥匙串 + base_url，可跳过——无 key 应用照常启动）+ **隐藏 dev 冒烟页**承载验证按钮（触发总线事件往返、Channel 有序流式、FTS 中文查询）。冒烟页是脚手架，后续 phase 逐步替换，不投机建正式布局/文档树（Phase 2 才有导入功能，现在排布局是推测式设计）。
- **D-07:** **Phase 1 即引入 TanStack Query**，冒烟页的总线事件往返直接用「coarse event → invalidateQueries → refetch」实现——A1 要验证的就是这个最终模式，后续所有 phase 沿用，不留「临时写法后换真基建」的返工。 — **Reversibility:** costly — 前端数据层惯例被后续所有 phase 的 UI 复用，中途更换牵动全部查询调用点。

### Crate 骨架完整度
- **D-08:** **全 crate 空骨架一次定型**：prism-engine（facade）+ prism-store/fs/parse/anchor/llm/mcp 全部建好；未到 phase 的 crate 只有 lib.rs + 依赖声明 + 最小编译单元。这使 `cargo tree -d` 检查覆盖真实依赖树（rusqlite/reqwest 全部在场），版本 pin 冲突 Phase 1 就暴露而非 Phase 4 才发现。
- **D-09:** service trait（FeedbackSource / CommentSink 等）定义在**独立 prism-types 小 crate**（零依赖）；prism-mcp 与 prism-engine 都依赖它，编译期无 facade↔mcp 依赖环。后续 F7 注册 get_context_pack 等新 trait 时不动 prism-mcp。 — **Reversibility:** costly — trait 落点决定 workspace 依赖方向，Phase 6/7 的 MCP 工具注册都建在其上。
- **D-10:** CLI helper（headersHelper + check-feedback hook）**Phase 1 建空占位 binary**（只依赖 keyring + reqwest，不链任何 engine crate）。成本近零，workspace 形状一次定型，其依赖也进入 cargo tree -d 检查范围（externalBin 签名公证雷区的依赖面早可见）。

### Claude's Discretion
- 冒烟页 Channel 流式验证用什么样例命令（假数据流即可，不必真功能）
- FTS 表的具体建法（external content table 与否、大小写折叠、remove_diacritics 等 trigram 选项）
- settings 页字段细节与校验
- rusqlite_migration 的具体组织方式（M::up 列表结构、迁移测试写法）
- STATE.md 已记录的待办：rmcp 2.2 feature-flag 确切名称需对照 README 核验（5 分钟检查，计划阶段做）

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 划分与退出标准
- `docs/调研_技术基建与开发Phase.md` §4 — Phase 1 内容清单（7 crate + settings 页）与退出标准；§2.1 架构形状；§2.3 单写者/回声抑制原语
- `.planning/ROADMAP.md` Phase 1 节 — 4 条成功标准（本阶段的验收权威）

### 架构与模式
- `.planning/research/ARCHITECTURE.md` — A1 修正（事件总线骨架 + IPC 双通路进 Phase 1）；Pattern 1（notify-then-fetch vs Channel 的选择依据）；Pattern 2（单写者纪律，writer 为 facade 持有的显式句柄）；trait 反转两方案对比（§The one dependency-graph trap）；Anti-Pattern 1–3
- `.planning/research/PITFALLS.md` — Phase 1 相关坑（如有）

### 技术栈与版本 pin
- `.planning/research/STACK.md` — 全部 crate 版本 pin 与验证依据；keyring 4 clarification（keyring-core + apple-native-keyring-store 具体用法）；FTS5 trigram 注记；rusqlite bundled 含 FTS5 的验证；Vite 8 修正案
- `.claude/CLAUDE.md`（项目级）— Technology Stack 表（同 STACK 的浓缩版，含 What-NOT-to-use）

### 非功能约束
- `docs/PRD_PrismDocs_MVP.md` §5 — 性能/隐私/密钥非功能需求（INFRA-04/05 设计约束的原始出处）

</canonical_refs>

<code_context>
## Existing Code Insights

绿地项目——repo 内目前只有 docs/ 与 .planning/，无任何代码。无可复用资产与既有模式；本阶段建立的就是后续所有 phase 的模式基线：

### Integration Points（本阶段建立、供后续消费）
- workspace 依赖图：shell → prism-engine（唯一入口）；prism-mcp → prism-types ← prism-engine
- 单写者句柄：facade 持有，Phase 2+ 所有写路径必须经它（不允许「随手拿池连接写」）
- 事件总线（tokio broadcast）→ shell adapter → coarse Tauri event → TanStack Query invalidate：Phase 2 watcher 事件直接接入
- rusqlite_migration 序列：Phase 2/3/5/7 各自追加迁移

</code_context>

<specifics>
## Specific Ideas

- 验证通路要「直观可点」：dev 冒烟页上每条不可逆决策一个可触发的验证入口，配合 cargo test 双保险
- cargo tree -d 无重复 rusqlite/reqwest 是显式检查项，应进 CI/justfile 而非口头约定

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 1-基建骨架*
*Context gathered: 2026-07-28*
