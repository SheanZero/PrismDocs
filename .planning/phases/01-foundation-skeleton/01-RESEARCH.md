# Phase 1: 基建骨架 - Research

**Researched:** 2026-07-28
**Domain:** Rust workspace 架构（Tauri v2 薄 shell + tauri-free engine crates）、SQLite WAL 单写者并发模型、FTS5 trigram CJK 索引、macOS Keychain（keyring-core 1.0）、rmcp 2.2 trait 反转、Tauri IPC 双通路
**Confidence:** HIGH（五项不可逆决策的关键事实全部来自官方文档 / 官方示例源码 / crates.io registry，本 session 直接核验）

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**FTS5 CJK tokenizer（schema v1 定案项）**
- **D-01:** FTS5 用 **trigram 统一单索引**——一张 FTS 表、一套查询逻辑，中英文都走 substring 匹配，天然覆盖 CJK 混排。不做 unicode61/trigram 双索引。索引体积 ~3× 与英文无词干提取是已接受的代价（500 文档/2000 卡片规模下 <300ms 预算轻松达标）。 — **Reversibility:** one-way — INFRA-02 将 tokenizer 钉在 schema v1；事后更换 = 全量重建 FTS 索引 + 查询层重写，且 Phase 2+ 的搜索行为契约（AC-1a 中文搜索）已依赖它。
- **D-02:** 短查询（<3 字符，典型为 2 字中文词如「评论」「锚点」）**自动降级 LIKE '%xx%' 线性扫描**；≥3 字符走 trigram MATCH。查询层按长度分流，用户无感。500 文档规模下全扫远在 300ms 预算内。

**Schema v1 覆盖范围**
- **D-03:** **最小骨架 + 逐 phase 增量迁移**：schema v1 只建 Phase 1 验证所需，后续每个 phase 用 rusqlite_migration 追加自己的表。不做全量领域 schema 一次落地——comments/cards 等表在真实需求出现前不做推测式设计。迁移体系本身就是 Phase 1 要验证的能力。
- **D-04:** 最小集边界 = **projects、documents（含内容列供 FTS 验证）、FTS 表、settings**。document_versions/blocks 等留给 Phase 2/3 自己的迁移（快照保留策略、引用不淘汰等字段需求那时才明确）。
- **D-05:** 非密钥配置（base_url、模型标识、项目阈值等）存 **SQLite settings 表**（k/v）——随库整体备份（INFRA-05「数据库单目录整体备份可恢复」）、单一真相源、事务一致。密钥仍走钥匙串，绝不入库。

**薄 shell 前端程度**
- **D-06:** 前端交付两块：**settings 页**（API key 写钥匙串 + base_url，可跳过——无 key 应用照常启动）+ **隐藏 dev 冒烟页**承载验证按钮（触发总线事件往返、Channel 有序流式、FTS 中文查询）。冒烟页是脚手架，后续 phase 逐步替换，不投机建正式布局/文档树（Phase 2 才有导入功能，现在排布局是推测式设计）。
- **D-07:** **Phase 1 即引入 TanStack Query**，冒烟页的总线事件往返直接用「coarse event → invalidateQueries → refetch」实现——A1 要验证的就是这个最终模式，后续所有 phase 沿用，不留「临时写法后换真基建」的返工。 — **Reversibility:** costly — 前端数据层惯例被后续所有 phase 的 UI 复用，中途更换牵动全部查询调用点。

**Crate 骨架完整度**
- **D-08:** **全 crate 空骨架一次定型**：prism-engine（facade）+ prism-store/fs/parse/anchor/llm/mcp 全部建好；未到 phase 的 crate 只有 lib.rs + 依赖声明 + 最小编译单元。这使 `cargo tree -d` 检查覆盖真实依赖树（rusqlite/reqwest 全部在场），版本 pin 冲突 Phase 1 就暴露而非 Phase 4 才发现。
- **D-09:** service trait（FeedbackSource / CommentSink 等）定义在**独立 prism-types 小 crate**（零依赖）；prism-mcp 与 prism-engine 都依赖它，编译期无 facade↔mcp 依赖环。后续 F7 注册 get_context_pack 等新 trait 时不动 prism-mcp。 — **Reversibility:** costly — trait 落点决定 workspace 依赖方向，Phase 6/7 的 MCP 工具注册都建在其上。
- **D-10:** CLI helper（headersHelper + check-feedback hook）**Phase 1 建空占位 binary**（只依赖 keyring + reqwest，不链任何 engine crate）。成本近零，workspace 形状一次定型，其依赖也进入 cargo tree -d 检查范围（externalBin 签名公证雷区的依赖面早可见）。

### Claude's Discretion

- 冒烟页 Channel 流式验证用什么样例命令（假数据流即可，不必真功能）
- FTS 表的具体建法（external content table 与否、大小写折叠、remove_diacritics 等 trigram 选项）
- settings 页字段细节与校验
- rusqlite_migration 的具体组织方式（M::up 列表结构、迁移测试写法）
- STATE.md 已记录的待办：rmcp 2.2 feature-flag 确切名称需对照 README 核验（5 分钟检查，计划阶段做）

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope

### 本研究对 Discretion 项的处置

| Discretion 项 | 本研究给出的建议 | 依据 |
|---------------|------------------|------|
| Channel 样例命令 | `dev_smoke_stream(Channel<SmokeEvent>, total=1000)`，前端断言 `seq[i]===i` 无缺口 | Pattern 6；小 total 证明不了乱序 |
| FTS 表建法 | external content + `content_rowid='rowid_pk'`（显式 INTEGER PRIMARY KEY）+ 三触发器；`detail` 保持默认 `full`；`case_sensitive`/`remove_diacritics` 均保持默认 0 | Pattern 3，全部有 sqlite.org 原文支撑 |
| settings 字段与校验 | `key/value/updated_at` k/v 表；`base_url` 用 `url::Url` 解析且 scheme ∈ {http,https} | Security Domain V5 |
| rusqlite_migration 组织 | `LazyLock<Migrations>` + `include_str!` 每个迁移一个 .sql 文件；单测调 `validate()` | Code Examples |
| **rmcp feature-flag 核验（STATE.md 待办）** | **已完成，无需再查**：`features = ["server", "transport-streamable-http-server"]` | crates.io `/api/v1/crates/rmcp/2.2.0` manifest 直读 |
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| INFRA-01 | Rust engine workspace（不依赖 tauri、可独立测试）+ Tauri 薄 shell + 事件总线骨架（notify-then-fetch 粗粒度事件 + Channel 有序流各验证一条通路）；prism-mcp 经 service trait 反转解依赖环 | Recommended Project Structure（单 workspace + `-p` 选择性测试）；Pattern 4（prism-types 同步 trait + `Arc<dyn>` + rmcp/axum 挂载形态）；Pattern 5（broadcast → coarse Tauri event，含 `Lagged`→`Resync`）；Pattern 6（`tauri::ipc::Channel` 有序流）；justfile 四条 `cargo tree` 断言把 D-01/D-09 变为 CI 可执行检查；`tauri::test::mock_builder` 提供非脆弱的 IPC 自动化测试 |
| INFRA-02 | SQLite WAL 单写者 + r2d2 读池（query_only）；FTS5 含 CJK 可用 tokenizer（schema v1 时定）；rusqlite_migration 迁移体系；bundled SQLite ≥3.51.3 | Pattern 1（writer-first 六步启动序列 + pragma 持久性/每连接性的官方依据 + `with_init` 注入点）；Pattern 2（`std::sync::Mutex<Connection>` 让"跨 await 持锁"成为编译错误）；Pattern 3（trigram + `detail=full` + external content + 显式 INTEGER PRIMARY KEY 防 VACUUM rowid 错位 + 查询层长度分流）；Code Examples 的迁移 `validate()` 单测与并发/中文命中集成测试；rusqlite 0.40.1 bundled = **SQLite 3.53.2**，已满足 ≥3.51.3 |
| INFRA-03 | API key 存系统钥匙串（keyring-core + apple-native-keyring-store）；支持 Anthropic/OpenAI 兼容端点与自定义 base_url；prism-llm 为唯一网络出口与唯一密钥入口 | Standard Stack 的 `apple-native-keyring-store` **必须启用 `keychain` feature**（直发公证 DMG 无 provisioning profile，用 `protected` 会 `-34018`）；Code Examples 给出 `set_default_store` / `Entry` / `NoEntry→Ok(None)` 的完整往返与 mock/真实双路径测试；Pitfall 1 指出 `set_default_store` 进程全局与并行测试的冲突及两种解法；`just check-single-egress` 用 `cargo tree` 断言只有 prism-llm 持有 reqwest/keyring 依赖；base_url 存 settings 表（D-05）并做 scheme 校验 |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

`./.claude/CLAUDE.md` 的强制指令，planner 必须逐条满足：

| 指令 | 来源 | 对本阶段的约束 |
|------|------|----------------|
| **GSD Workflow Enforcement**：Edit/Write 前必须经 GSD 命令入口（`/gsd-execute-phase` 等），不得在 GSD 流程外直接改仓库 | CLAUDE.md § GSD Workflow Enforcement | 所有代码创建任务必须在 `/gsd-execute-phase` 内进行 |
| **版本 pin 表为既定输入**（Technology Stack 全表，2026-07-28 crates.io 验证） | CLAUDE.md § Technology Stack | 计划不得重新选型；本研究只补 feature flag 与用法 |
| **comrak 是唯一 Block 边界真相源**，前端 react-markdown 仅渲染 | CLAUDE.md § Constraints / What NOT to Use #1 | Phase 1 不做锚定，但 prism-parse 空骨架不得引入第二 parser；前端不得引入任何 markdown AST 处理 |
| **不污染原则**：Block ID / 评论 / Xref / 元数据全部存 sidecar（`~/Library/Application Support/PrismDocs/`，按 project-id 索引，D-13） | CLAUDE.md § Constraints | 用 `dirs::data_dir()`，**禁止** Tauri `app_data_dir`（后者会让 prism-store 依赖 tauri，违反 D-01） |
| **MCP 传输 D-07**：loopback streamable HTTP + per-install bearer（钥匙串）+ Origin allowlist，无子进程；配套轻量 CLI helper | CLAUDE.md § Constraints | Pattern 4 的挂载与中间件形态；Phase 1 只建骨架 |
| **密钥**：API key 存系统钥匙串（keyring 直连，**不用 stronghold**）；文档内容仅发往用户配置端点 | CLAUDE.md § Constraints / What NOT to Use | 禁用 `tauri-plugin-stronghold` 与 `tauri-plugin-keyring` |
| **禁用清单**：`bundled-full`/系统 SQLite、`tokio-rusqlite`/`sqlx`、`serde_yaml`、stdio MCP proxy、额外 SSE crate、react-markdown 作锚定源 | CLAUDE.md § What NOT to Use | 计划中出现任一即为违规 |
| **Immutability / 文件组织 / 错误处理 / 输入校验**（全局 rules/common/coding-style.md） | 用户级 CLAUDE.md | 200–400 行/文件典型、800 上限；边界处校验；不静默吞错 |
| **测试**：最低 80% 覆盖率，TDD（RED→GREEN→REFACTOR） | 用户级 rules/common/testing.md | Wave 0 先建测试骨架再实现——与本文件 Validation Architecture 的 Wave 0 清单一致 |
| **安全**：提交前无硬编码密钥、输入校验、参数化查询、错误信息不泄漏敏感数据 | 用户级 rules/common/security.md | 见 Security Domain；`git grep` 明文密钥检查已列入测试映射 |
| **Git**：conventional commits（`feat:`/`fix:`/`chore:` …），attribution 已全局关闭 | 用户级 rules/common/git-workflow.md | 提交信息格式 |
| **回复语言默认中文**；代码默认不写注释（WHY 非显然时一行） | 用户级 CLAUDE.md § 风格偏好 | 文档/交流用中文；代码注释克制 |

## Summary

本阶段的技术风险不在"选什么库"——项目 CLAUDE.md 的 2026-07-28 pin 审计已经解决了版本选型，本次核验全部 pin 依然是 crates.io 当前版本（rmcp 2.2.0 仍为 max_stable，3.0.0-beta.3 未稳定）。风险在**用法层**：五项不可逆决策每一项都有一个"看起来能跑、但在 Phase 3+ 才暴露"的错误用法，而它们全部落在 schema / 连接架构 / workspace 依赖方向这三类"改一次牵动全局"的位置。

本次调研把五项决策各自钉死到可写进任务的粒度，并找出四个原 CONTEXT 未覆盖但会静默毁掉验收的细节：**(1)** FTS5 `detail=none/column` 会禁止长度 >3 字符的 MATCH 查询——trigram 表必须留在默认 `detail=full`，否则 D-01 的中文搜索在 4 字词上直接报错；**(2)** external content FTS 表若绑定隐式 rowid，`VACUUM` 会重编号 rowid 导致索引与内容表静默错位——`documents` 必须显式声明 `INTEGER PRIMARY KEY`；**(3)** `tokio::sync::broadcast` 在接收端落后时丢消息（`RecvError::Lagged`），shell adapter 必须把 Lagged 翻译成一次全量 invalidate，否则 notify-then-fetch 会静默漏更新；**(4)** `keyring_core::set_default_store` 是**进程级全局**，Rust 默认多线程并行跑测试会让 mock store 与真实 Keychain 互相污染。

依赖方向上有一个可以让 `prism-types` 保持零依赖的关键简化：**service trait 应设计为同步（blocking）trait**，而非 async trait。prism-store 基于 rusqlite 本来就是阻塞的，rmcp handler 在自己的 async 上下文里用 `spawn_blocking` 调用即可。这样 trait 天然 object-safe，`Arc<dyn FeedbackSource>` 直接可用，prism-types 不需要 `async-trait` 依赖，也不需要 AFIT/`trait_variant` 的 dyn-safety 变通。

**Primary recommendation:** 按「writer-first 启动序列 → 迁移 → 只读池」建 prism-store；FTS5 用 `tokenize='trigram'` + `detail=full` + external content + 三触发器 + `documents.doc_id INTEGER PRIMARY KEY`；prism-types 只放同步 trait 与共享类型；engine 事件总线用 `tokio::sync::broadcast` 且 shell adapter 必须处理 `Lagged`；用 `cargo tree -e normal` 的三条断言（无重复 rusqlite/reqwest/libsqlite3-sys、engine crates 树中无 tauri、prism-mcp 树中无 prism-engine）把 D-01/D-09 变成 CI 可执行的检查而非口头约定。

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| SQLite schema / 迁移 / 连接纪律 | prism-store（engine） | — | 唯一持有 SQLite 连接的 crate；单写者句柄由 facade 持有并转交 |
| FTS5 索引与查询分流（trigram MATCH / LIKE 回退） | prism-store（engine） | — | 索引与查询逻辑必须同源，否则 tokenizer 语义与查询层漂移 |
| 事件总线（typed broadcast） | prism-engine（facade） | — | 唯一订阅点，D-01 要求它不依赖 tauri |
| 总线 → Tauri event 适配（coarse invalidate） | Tauri shell（src-tauri） | — | 唯一允许 link tauri 的地方（Anti-Pattern 1） |
| Channel 有序流式命令 | Tauri shell（src-tauri） | prism-engine 提供数据源 | `tauri::ipc::Channel<T>` 是 tauri 类型，不能进 engine |
| 前端失效与重取（TanStack Query） | React WebView | — | 前端只持 view-model，SQLite 是唯一真相源 |
| API key 读写 Keychain | prism-llm（engine） | Tauri shell 仅转发命令 | NFR-03：唯一密钥入口 |
| 非密钥配置（base_url 等） | prism-store `settings` 表 | — | D-05：随库备份、事务一致 |
| MCP 工具面 | prism-mcp（engine） | prism-types 提供注入 trait | D-09：编译期无 facade↔mcp 环 |
| sidecar 数据根路径解析 | prism-store（`dirs::data_dir()`） | — | D-13 明确禁用 Tauri `app_data_dir`（那会让 store 依赖 tauri） |
| CLI helper（headersHelper / check-feedback） | 独立 workspace binary | — | D-10：只依赖 keyring + reqwest，不 link 任何 engine crate |

## Standard Stack

> 版本 pin 由项目 CLAUDE.md 的 2026-07-28 审计确定，**不在此重新论证**。下表只列本阶段实际要写进 `[workspace.dependencies]` 的条目，并补充 pin 审计未覆盖的 **feature flag** 与**具体子 crate**——这些才是 Phase 1 会踩的地方。

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rusqlite` | 0.40（`features = ["bundled"]`） | SQLite 绑定 | `bundled` 编译期含 `-DSQLITE_ENABLE_FTS5`；0.40.1 **bundled SQLite = 3.53.2**，满足 ≥3.51.3 硬要求 [VERIFIED: github.com/rusqlite/rusqlite releases v0.40.1 "Bump bundled SQLite version to 3.53.2"] |
| `r2d2` + `r2d2_sqlite` | 0.8 / 0.35 | 只读连接池 | `SqliteConnectionManager::file(p).with_init(f)` 是官方文档指定的 per-connection PRAGMA 注入点 [CITED: docs.rs/r2d2_sqlite SqliteConnectionManager] |
| `rusqlite_migration` | 2.6 | 迁移体系 | 用 SQLite `user_version` 追踪版本；`validate()` 在内存库跑全套迁移，是标准单测形态 [VERIFIED: docs.rs/rusqlite_migration] |
| `keyring-core` | 1.0.0 | 密钥 API 门面 | v4 拆分后的正确落点；内置 `mock::Store` 供测试 [VERIFIED: docs.rs/keyring-core] |
| `apple-native-keyring-store` | 1.0.1（**`features = ["keychain"]`**） | macOS Keychain 后端 | `keychain` 模块用于**非 provisioning-profile 签名**的 app（= 直发公证 DMG）；`protected` 模块需要 profile，否则 `PlatformError -34018` [VERIFIED: 官方 README + examples/instantiation.rs] |
| `rmcp` | 2.2（**`features = ["server", "transport-streamable-http-server"]`**） | MCP 协议层 | feature 名本 session 从 crates.io 2.2.0 manifest 直读核验 [VERIFIED: crates.io api /crates/rmcp/2.2.0] |
| `axum` | 0.8 | loopback HTTP 宿主 | rmcp 官方示例即 `Router::new().nest_service("/mcp", service)` [VERIFIED: rust-sdk examples/servers/src/counter_streamhttp.rs] |
| `tokio` | 1（`rt-multi-thread`, `sync`, `macros`） | 运行时 + `broadcast` 总线 | rmcp/axum/reqwest 已强制在树中 |
| `dirs` | 6 | sidecar 根路径 | macOS 返回 `$HOME/Library/Application Support` [VERIFIED: docs.rs/dirs data_dir] |
| `tauri` | 2（shell crate 独有；**测试加 `features = ["test"]`**） | 桌面壳 | `tauri::test::mock_builder` + `get_ipc_response` 是官方进程内 IPC 测试路径 [VERIFIED: docs.rs/tauri/test] |
| `@tanstack/react-query` | 5.101.4 | 前端失效/重取（D-07） | coarse event → `invalidateQueries` → refetch 即 A1 要验证的最终模式 [VERIFIED: npm registry 2026-07-28] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `ulid` | 1 | projects/documents 主键 | Phase 1 起统一 ID 形态，Phase 3 Block ID 复用 |
| `blake3` | 1.8 | `documents.content_hash` | Phase 1 只存不算差异；Phase 2/3 复用 |
| `serde` / `serde_json` / `thiserror` / `tracing` | current | 通用管线 | workspace-wide |
| `tempfile` | 3 | 测试隔离的 DB 目录与 sidecar 根 | 所有 prism-store 集成测试 |
| `serial_test` | 3 | 串行化 keyring 全局 store 测试 | 仅在坚持用 `cargo test` 时需要（用 `cargo-nextest` 可免，见下） |
| `cargo-nextest`（dev tool） | latest | 每个测试独立进程 | 直接消解 `set_default_store` 进程全局冲突；代价是不跑 doctest |
| `vitest` + `@testing-library/react` | latest | 冒烟页/adapter hook 单测 | 前端仅需最小覆盖（冒烟页是脚手架） |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `std::sync::Mutex<Connection>` 单写者 | 专用 writer task + `mpsc` 命令队列 | 队列方案让 TD-01 §5.1 的「version+migrate+log 单事务尾」变成一个消息类型，事务组合能力丧失；Mutex 方案用闭包 `write(\|tx\| …)` 天然组合。**选 Mutex** |
| external content FTS 表 | 普通 FTS 表（内容双份存储） | 普通表让 trigram 索引之外再存一份全文（trigram 索引本已 ~3×）；且写路径要手工同步两处。**选 external content + 触发器** |
| external content FTS 表 | contentless（`content=''`） | contentless **不支持 `rebuild` 命令** [CITED: sqlite.org/fts5.html]，而 Phase 2+ 重建索引是可预期需求。**排除** |
| 同步 service trait | `#[async_trait]` async trait | async_trait 引入 boxing + 一个 prism-types 的外部依赖；而 prism-store 本就是阻塞 rusqlite，async 是伪需求。**选同步 trait + `spawn_blocking`** |
| `Arc<dyn Trait>` 注入 | 泛型参数 `S: FeedbackSource` | 泛型会让 prism-mcp 的公开类型（以及 axum service 类型）被 engine 具体类型污染，`StreamableHttpService::new` 的 `'static` factory 闭包更难写。**选 dyn + Arc** |
| `cargo-nextest` | `serial_test::serial` 标注 | nextest 进程隔离更彻底但不跑 doctest；serial_test 零工具链要求但需人工记得标注。**两者择一，计划阶段拍板** |
| 单一 workspace（engine + src-tauri） | 双 workspace | 双 workspace 让 `cargo tree -d` 无法一次覆盖真实依赖树（D-08 的全部理由）。**选单一 workspace，用 `-p` 选择性测试** |

**Installation:**

```toml
# 根 Cargo.toml —— [workspace.dependencies]，Phase 1 新增/明确的条目
rusqlite            = { version = "0.40", features = ["bundled"] }
r2d2                = "0.8"
r2d2_sqlite         = "0.35"
rusqlite_migration  = "2.6"
keyring-core        = "1.0"
apple-native-keyring-store = { version = "1.0", features = ["keychain"] }
rmcp                = { version = "2.2", features = ["server", "transport-streamable-http-server"] }
axum                = "0.8"
tokio               = { version = "1", features = ["rt-multi-thread", "sync", "macros"] }
dirs                = "6"
ulid                = "1"
blake3              = "1.8"
tempfile            = "3"
serial_test         = "3"
```

```bash
npm install @tanstack/react-query@5 react@19 react-dom@19
npm install -D vite@8 @vitejs/plugin-react @tauri-apps/cli@2 typescript vitest
```

**Version verification（本 session 直查 registry）:** rusqlite 0.40.1 / r2d2_sqlite 0.35.0 / rusqlite_migration 2.6.0 / keyring-core 1.0.0 / apple-native-keyring-store 1.0.1 / rmcp 2.2.0（max_stable；3.0.0-beta.3 为 pre-release）/ axum 0.8.9 / tauri 2.11.5 / @tanstack/react-query 5.101.4 / vite 8.1.5 / react 19.2.8。全部与 CLAUDE.md pin 一致，无需修订。[VERIFIED: crates.io api + npm registry, 2026-07-28]

## Package Legitimacy Audit

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| keyring-core | crates.io | 首发 2025-06 | 64k/wk | open-source-cooperative/keyring-core | OK | Approved |
| apple-native-keyring-store | crates.io | 首发 2025-08 | 38k/wk | open-source-cooperative/apple-native-keyring-store | OK | Approved |
| r2d2 | crates.io | 2014 | 359k/wk | sfackler/r2d2 | OK | Approved |
| r2d2_sqlite | crates.io | 2015 | 91k/wk | ivanceras/r2d2-sqlite | OK | Approved |
| rusqlite_migration | crates.io | 2020 | 202k/wk | cljoly/rusqlite_migration | OK | Approved |
| dirs | crates.io | 2015 | 5.1M/wk | soc/dirs-rs | OK | Approved |
| ulid | crates.io | 2017 | 759k/wk | dylanhart/ulid-rs | OK | Approved |
| rmcp | crates.io | 2025-03 | 689k/wk | modelcontextprotocol/rust-sdk | OK | Approved |
| axum | crates.io | 2021 | 7.9M/wk | tokio-rs/axum | OK | Approved |
| tempfile | crates.io | 2015 | 12.2M/wk | Stebalien/tempfile | OK | Approved |
| serial_test | crates.io | 2018 | 1.86M/wk | palfrey/serial_test | OK | Approved |
| @tanstack/react-query | npm | 最新版 2026-07-21 | 61.4M/wk | TanStack/query | SUS（近期发版触发） | Approved — 见下 |
| @tauri-apps/api | npm | 2026-06-17 | 2.1M/wk | tauri-apps/tauri | OK | Approved |
| @tauri-apps/cli | npm | 2026-06-28 | 1.8M/wk | tauri-apps/tauri | SUS（近期发版触发） | Approved — 见下 |
| @vitejs/plugin-react | npm | 2026-07-22 | 75.5M/wk | vitejs/vite-plugin-react | SUS（近期发版触发） | Approved — 见下 |

**Packages removed due to [SLOP] verdict:** none

**Packages flagged as suspicious [SUS]:** `@tanstack/react-query`、`@tauri-apps/cli`、`@vitejs/plugin-react`

**关于三条 SUS 的判定说明（planner 请读）:** 触发信号是 seam 用"最新版本发布时间"近似包龄，三者最近一次发版都在两周内。反向信号是压倒性的：官方组织 GitHub 仓库、周下载 1.8M–75M、无 `postinstall` 脚本、`@tauri-apps/*` 与 `@vitejs/plugin-react` 本就是项目 CLAUDE.md 已锁定技术栈的组成部分。判定为**推荐启发式的假阳性，不建议为其插入 `checkpoint:human-verify` 任务**。若 planner 依 GSD 硬规则必须插入，建议合并为一个针对 `@tanstack/react-query`（本阶段唯一新引入的 npm 依赖，来自 D-07）的确认点，其余两个跳过。

## Architecture Patterns

### System Architecture Diagram

Phase 1 交付的是骨架 + 四条验证通路。下图按数据流绘制（不是文件清单）：

```
                    ┌──────────────── React 19 WebView ────────────────┐
                    │  settings 页          dev 冒烟页（隐藏路由）      │
                    │      │                  │      │        │        │
                    │      │        ┌─────────┘      │        │        │
                    │      │        │ listen()       │ invoke │ invoke │
                    │      │        ▼                │ +Chan  │ query  │
                    │      │  TanStack Query         │        │        │
                    │      │  invalidateQueries      │        │        │
                    │      │        │ refetch        │        │        │
                    └──────┼────────┼────────────────┼────────┼────────┘
     ═══════════════ Tauri IPC 边界 ═══════════════════════════════════
       ①命令        │        ②coarse event      ③Channel<T>   ④命令
       set_api_key  │        "prism://changed"   有序流        search
                    ▼        ▲                   ▲             ▼
                    ┌────────┴───────────────────┴──────────────────┐
                    │  src-tauri（唯一 link tauri 的 crate）        │
                    │  commands = 单行委托     bus→event adapter    │
                    │                            └ Lagged ⇒ resync  │
                    └────────┬───────────────────────────────────────┘
                             ▼  直接调用 + 订阅
       ┌─────────────────── prism-engine (facade) ──────────────────┐
       │  持有：Store 句柄（含单写者）、broadcast::Sender<EngineEvent>│
       │  实现：prism-types 的 service traits                        │
       └──┬──────────────┬───────────────┬────────────────┬─────────┘
          │              │               │                │ 注入 Arc<dyn …>
          ▼              ▼               ▼                ▼
     prism-store    prism-llm       prism-parse      prism-mcp
     ┌──────────┐   ┌─────────┐     prism-anchor     ┌──────────┐
     │writer    │   │keyring  │     prism-fs         │axum 0.8  │
     │Mutex<Conn│   │ ↕       │     （Phase 1 空骨架）│127.0.0.1 │
     │  ↓ WAL   │   │Keychain │                      │bearer +  │
     │reader池  │   │ ↕       │                      │Origin MW │
     │query_only│   │reqwest  │                      │  ↓       │
     └────┬─────┘   └────┬────┘                      └────┬─────┘
          │              │                                │
          ▼              ▼                                ▼
    ~/Library/Application Support/PrismDocs/     用户 LLM 端点      agent (Phase 6)
    prismdocs.db (+ -wal, -shm)                                          ▲
          ▲                                                              │
          └──────── prism-types (零依赖：同步 trait + 共享类型) ──────────┘
                    ↑ prism-mcp 只依赖它，绝不依赖 prism-engine
```

四条验证通路对应四条成功标准：② = 标准 2 前半，③ = 标准 2 后半，`prism-store` 的 writer/reader/FTS = 标准 3，`prism-llm` 的 keyring 往返 = 标准 4；整个依赖方向图 = 标准 1。

### Recommended Project Structure

```
PrismDocs/
├── Cargo.toml                 # [workspace] members + [workspace.dependencies]
├── Cargo.lock                 # 提交
├── justfile                   # 依赖方向断言 + 常用命令（.specifics 要求进 CI）
├── crates/
│   ├── prism-types/           # 零依赖：service traits + EngineEvent + 共享 DTO
│   ├── prism-store/           # rusqlite/r2d2/FTS5/migrations —— 唯一开 SQLite 的 crate
│   ├── prism-fs/              # Phase 1 空骨架（lib.rs + 依赖声明）
│   ├── prism-parse/           # 空骨架
│   ├── prism-anchor/          # 空骨架
│   ├── prism-llm/             # keyring-core + reqwest —— 唯一密钥入口/网络出口
│   ├── prism-mcp/             # rmcp + axum；只依赖 prism-types
│   ├── prism-engine/          # facade：总线 + 编排 + trait 实现
│   └── prism-cli/             # D-10 空占位 binary（keyring + reqwest，不 link engine）
├── src-tauri/                 # workspace member；唯一 link tauri
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/{lib.rs, commands.rs, bus_adapter.rs}
├── src/                       # React 19 + Vite 8
│   ├── pages/{Settings.tsx, DevSmoke.tsx}
│   └── lib/{queryClient.ts, useEngineInvalidation.ts}
├── package.json
└── vite.config.ts
```

**单一 workspace 的关键取舍：** Tauri 官方文档承认项目可以位于更大的 workspace 中（`.taurignore` 的说明明确提到 "cargo workspace root folder"），但**没有给出集成指引** [CITED: v2.tauri.app/develop]。实践后果需要计划阶段显式处理：

- `target/` 移到 workspace 根，`src-tauri/target/` 不再存在——`.gitignore` 与任何路径假设要按根 `target/` 写
- 官方"提交 `src-tauri/Cargo.lock`"的建议在 workspace 下变成"提交根 `Cargo.lock`"（workspace 只有一个 lockfile）
- D-01 的"engine 不依赖 tauri 即可 `cargo test`"在单 workspace 下由 **`-p` 选择性构建**保证：`cargo test -p prism-store -p prism-engine …` 只编译被选包及其依赖，不会拉 tauri。**不要用 `cargo test --workspace` 作为 D-01 的证据**——那会编译 shell，证明不了任何事

### Pattern 1: writer-first 启动序列（单写者 + 只读池）

**What:** 连接建立顺序是有语义的，不能并行、不能颠倒。

**顺序（每一步都是必要的）:**

1. `fs::create_dir_all(data_root)`；**先**打开 writer 连接（读写 flags），让它创建 DB 文件
2. writer 上执行 `PRAGMA journal_mode=WAL`。**必须用 `execute_batch` 或 `query_row`，不能用 `execute`** —— `journal_mode` 返回一行结果，`Connection::execute` 遇到返回行会报 `ExecuteReturnedResults`
3. writer 上执行 per-connection PRAGMA 套餐：`busy_timeout`、`synchronous=NORMAL`、`foreign_keys=ON`
4. `migrations.to_latest(&mut writer_conn)`（签名要求 `&mut Connection`）
5. **迁移完成后**才建只读池，`with_init` 里按序执行 `busy_timeout` → `foreign_keys=ON` → `query_only=ON`（`query_only` 放最后）
6. 关闭时 writer 上 `PRAGMA wal_checkpoint(TRUNCATE)`

**Why 顺序不可变:** `journal_mode=WAL` 是**持久设置**——"The WAL journaling mode is persistent; after being set it stays in effect across multiple database connections and after closing and reopening the database" [CITED: sqlite.org/pragma.html]。其余全部是**每连接**设置：`synchronous`、`busy_timeout`、`query_only`、`foreign_keys`、`cache_size`、`wal_autocheckpoint`、`temp_store`、`mmap_size` 无一持久 [CITED: sqlite.org/pragma.html]。这直接推出两条工程结论：(a) WAL 只需在 writer 上设一次，(b) **只读池的每个连接都必须重新设 `query_only`**，`with_init` 是唯一正确的注入点 [CITED: docs.rs/r2d2_sqlite]。r2d2 `Pool::new` 会立即预热 `min_idle` 条连接，若池先于 writer 建立，池会自己创建一个空 DB 文件，迁移随后跑在错误的文件上——这就是"writer-first"不是风格偏好的原因。

**Example:**

```rust
// crates/prism-store/src/open.rs
use rusqlite::{Connection, OpenFlags};
use r2d2_sqlite::SqliteConnectionManager;

const BUSY_TIMEOUT_MS: u32 = 5_000;
const MIN_SQLITE: (u32, u32, u32) = (3, 51, 3);

pub struct Store {
    writer: std::sync::Mutex<Connection>,      // 见 Pattern 2
    readers: r2d2::Pool<SqliteConnectionManager>,
}

pub fn open(db_path: &std::path::Path) -> Result<Store, StoreError> {
    if let Some(dir) = db_path.parent() { std::fs::create_dir_all(dir)?; }

    // 1–3. writer 先行
    let mut writer = Connection::open(db_path)?;
    assert_sqlite_version(&writer)?;                       // 成功标准 3 的一部分
    writer.execute_batch(&format!(
        "PRAGMA journal_mode=WAL;\
         PRAGMA synchronous=NORMAL;\
         PRAGMA busy_timeout={BUSY_TIMEOUT_MS};\
         PRAGMA foreign_keys=ON;"
    ))?;                                                    // execute_batch 容忍返回行

    // 4. 迁移（需 &mut）
    crate::migrations::migrations().to_latest(&mut writer)?;

    // 5. 只读池 —— 迁移之后
    let manager = SqliteConnectionManager::file(db_path)
        .with_flags(OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_URI)
        .with_init(|c| c.execute_batch(&format!(
            "PRAGMA busy_timeout={BUSY_TIMEOUT_MS};\
             PRAGMA foreign_keys=ON;\
             PRAGMA query_only=ON;"       // 最后一条：之后该连接无法写
        )));
    let readers = r2d2::Pool::builder().max_size(4).build(manager)?;

    Ok(Store { writer: std::sync::Mutex::new(writer), readers })
}

fn assert_sqlite_version(c: &Connection) -> Result<(), StoreError> {
    let v: String = c.query_row("SELECT sqlite_version()", [], |r| r.get(0))?;
    let p: Vec<u32> = v.split('.').filter_map(|s| s.parse().ok()).collect();
    let got = (p[0], *p.get(1).unwrap_or(&0), *p.get(2).unwrap_or(&0));
    if got < MIN_SQLITE { return Err(StoreError::SqliteTooOld(v)); }
    Ok(())
}
```

**注意只读池用 `SQLITE_OPEN_READ_WRITE` 而非 `SQLITE_OPEN_READ_ONLY`：** 只读 flags 的连接无法在 `-shm` 文件缺失时创建它（例如崩溃后首次打开），会拿到 `SQLITE_CANTOPEN`。用读写 flags 打开 + `query_only=ON` 防呆，既保留 WAL 恢复能力又保证写被拒——这正是 PITFALLS Pitfall 7 里"防呆"的准确含义。 [ASSUMED — SQLITE_OPEN_READ_ONLY 的 -shm 行为来自训练知识，未在本 session 直查文档；`query_only` 方案本身由 PITFALLS 与 pragma 文档共同支持]

### Pattern 2: `std::sync::Mutex<Connection>` 作为单写者句柄

**What:** writer 用 `std::sync::Mutex`（**不是** `tokio::sync::Mutex`），对外暴露闭包式事务 API：

```rust
impl Store {
    /// 阻塞调用。facade 侧必须包在 tokio::task::spawn_blocking 里。
    pub fn write<T>(&self, f: impl FnOnce(&rusqlite::Transaction) -> Result<T, StoreError>)
        -> Result<T, StoreError>
    {
        let mut guard = self.writer.lock().unwrap_or_else(|e| e.into_inner());
        let tx = guard.transaction()?;
        let out = f(&tx)?;
        tx.commit()?;
        Ok(out)
    }

    pub fn read<T>(&self, f: impl FnOnce(&rusqlite::Connection) -> Result<T, StoreError>)
        -> Result<T, StoreError>
    {
        let conn = self.readers.get()?;
        f(&conn)      // conn 出作用域即归还，禁止跨事件循环长持有
    }
}
```

**Why 选 `std::sync::Mutex`（这是特性不是妥协）:** `std::sync::MutexGuard` 是 `!Send`，因此"在 `.await` 上持有 writer 锁"会变成**编译错误**而不是运行期长事务。这把 Pattern 2 的纪律交给了编译器。同理，闭包式 API 让 TD-01 §5.1 要求的「version + block_instance + migration_log 单事务尾」是一个自然的 `write(|tx| { … })` 调用，而写队列方案要为每种事务定义一个消息类型。

**Why 不是 async:** rusqlite 是阻塞 API；项目 CLAUDE.md 的 What-NOT-to-use 已明确排除 `tokio-rusqlite`/`sqlx`。正确形态是「阻塞 store + facade 侧 `spawn_blocking`」，这也顺带让 service trait 可以是同步的（见 Pattern 4）。

### Pattern 3: FTS5 trigram schema v1（不可逆，本节每条都进 migration 001）

**验证过的 trigram 事实**（全部来自 sqlite.org/fts5.html [CITED: sqlite.org/fts5.html]）：

| 事实 | 原文/结论 | 对本项目的影响 |
|------|-----------|----------------|
| 最短匹配长度 | "Substrings consisting of fewer than 3 unicode characters do not match any rows when used with a full-text query." | 直接证成 D-02 的 <3 字符降级 |
| 选项 | `case_sensitive`（默认 0）、`remove_diacritics`（默认 0，且仅当 `case_sensitive=0` 时可设 1） | **两者都保持默认** |
| LIKE/GLOB 索引化 | "Unless the remove_diacritics option is set, FTS5 tables that use the trigram tokenizer also support indexed GLOB and LIKE"；`case_sensitive=1` 时只能索引 GLOB | 不设 `remove_diacritics` = 保留未来把 LIKE 也走索引的余地 |
| LIKE 回退条件 | 模式中若没有 ≥3 个连续非通配符字符，FTS5 退化为全表线性扫 | 2 字中文的 `LIKE '%评论%'` **必然全扫**，D-02 的成本判断正确 |
| **`detail=` 陷阱** | "If the FTS5 table is created with the detail=none or detail=column option specified, full-text queries may not contain any tokens longer than 3 unicode characters." | **必须留在 `detail=full`（默认）**。若为省索引体积改成 `detail=none`，`MATCH '锚定引擎'`（4 字）直接失效 —— 这会在中文验收上炸掉 |
| contentless 限制 | "`rebuild` … is not available with contentless tables." | 排除 contentless |
| external content 同步 | "It is still the responsibility of the user to ensure that the contents … are kept up to date … One way to do this is with triggers." | 用三触发器（ai/ad/au），不靠调用方记得维护 |

**rowid 陷阱（本次调研新发现，CONTEXT 未覆盖）:** external content 表通过 `content_rowid` 绑定内容表的 rowid。SQLite 文档："The VACUUM command may change the ROWIDs of entries in any tables that do not have an explicit INTEGER PRIMARY KEY." [CITED: sqlite.org/lang_vacuum.html] 若 `documents` 只有 `id TEXT PRIMARY KEY`（ULID），它的 rowid 是隐式的，一次 `VACUUM` 就会让 FTS 索引与内容表**静默错位**——搜索返回错误文档，且不报错。**修复：`documents` 显式声明一个 `INTEGER PRIMARY KEY` 代理列，ULID 作为 `UNIQUE TEXT` 并存。**

**schema v1（D-04 严格最小集）:**

```sql
-- migration 001
CREATE TABLE projects (
  rowid_pk    INTEGER PRIMARY KEY,          -- 显式 rowid，VACUUM 安全
  id          TEXT NOT NULL UNIQUE,         -- ULID，D-13 的 project-id
  name        TEXT NOT NULL,
  root_path   TEXT NOT NULL,
  created_at  INTEGER NOT NULL
) STRICT;

CREATE TABLE documents (
  rowid_pk    INTEGER PRIMARY KEY,          -- ← FTS content_rowid 绑定这一列
  id          TEXT NOT NULL UNIQUE,         -- ULID
  project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  rel_path    TEXT NOT NULL,
  title       TEXT,
  content     TEXT NOT NULL,                -- D-04：含内容列供 FTS 验证
  content_hash TEXT NOT NULL,               -- blake3
  updated_at  INTEGER NOT NULL,
  UNIQUE(project_id, rel_path)
) STRICT;

CREATE VIRTUAL TABLE documents_fts USING fts5(
  title,
  content,
  content        = 'documents',
  content_rowid  = 'rowid_pk',
  tokenize       = 'trigram'                -- detail 保持默认 full
);

CREATE TRIGGER documents_ai AFTER INSERT ON documents BEGIN
  INSERT INTO documents_fts(rowid, title, content) VALUES (new.rowid_pk, new.title, new.content);
END;
CREATE TRIGGER documents_ad AFTER DELETE ON documents BEGIN
  INSERT INTO documents_fts(documents_fts, rowid, title, content)
    VALUES ('delete', old.rowid_pk, old.title, old.content);
END;
CREATE TRIGGER documents_au AFTER UPDATE ON documents BEGIN
  INSERT INTO documents_fts(documents_fts, rowid, title, content)
    VALUES ('delete', old.rowid_pk, old.title, old.content);
  INSERT INTO documents_fts(rowid, title, content) VALUES (new.rowid_pk, new.title, new.content);
END;

CREATE TABLE settings (
  key         TEXT PRIMARY KEY,             -- D-05：非密钥配置（base_url、模型标识…）
  value       TEXT NOT NULL,
  updated_at  INTEGER NOT NULL
) STRICT;
```

**查询层分流（D-02）:**

```rust
pub fn search(conn: &Connection, project_id: &str, q: &str) -> Result<Vec<Hit>, StoreError> {
    if q.chars().count() >= 3 {
        // trigram MATCH —— 走索引
        conn.prepare("SELECT d.id, d.title FROM documents_fts f \
                      JOIN documents d ON d.rowid_pk = f.rowid \
                      WHERE documents_fts MATCH ?1 AND d.project_id = ?2 \
                      ORDER BY rank")?
            /* … */
    } else {
        // <3 字符：对 documents 表直接 LIKE，不经 FTS
        conn.prepare("SELECT id, title FROM documents \
                      WHERE project_id = ?1 AND (content LIKE '%'||?2||'%' OR title LIKE '%'||?2||'%')")?
            /* … */
    }
}
```

回退分支**打在 `documents` 表而非 FTS 表**：语义等价、路径更短，且不依赖 FTS 的 LIKE 索引化（那条路在 <3 字符时本来也会全扫）。同时它把 FTS 的 LIKE 能力完整保留为 Phase 2+ 的余地。

**MATCH 查询串必须转义:** 用户输入直接塞进 `MATCH` 会被解析为 FTS5 查询语法（`"`、`*`、`AND`/`OR`/`NOT`、`:`、`^` 等有特殊含义），既是功能 bug 也是注入面。标准做法是把整个查询包成一个双引号短语并把内部 `"` 加倍：`format!("\"{}\"", q.replace('"', "\"\""))`。参数化只保护 SQL 层，保护不了 FTS 查询语法层——这是两层不同的东西。

### Pattern 4: prism-types 同步 trait 反转（D-09）

**依赖方向（编译期强制）:**

```
prism-engine ──▶ prism-mcp ──▶ prism-types
      │                            ▲
      └────────────────────────────┘
```

`prism-engine` 依赖 `prism-mcp`（它构造并托管 server）。因此**一旦 `prism-mcp` 声明对 `prism-engine` 的普通依赖，就构成 cargo 硬错误 `cyclic package dependency`**——环在编译期不可能存在。唯一的逃逸口是 **dev-dependency 环**（cargo 允许 A→B 普通依赖 + B→A dev 依赖），所以 CI 断言必须显式排除 dev 边。 [VERIFIED: Cargo 语义 + rust-lang/cargo issue 讨论]

**trait 设计（同步，不用 async_trait）:**

```rust
// crates/prism-types/src/lib.rs —— 零第三方依赖（除 serde derive）
pub trait FeedbackSource: Send + Sync + 'static {
    fn list_feedback(&self, project_id: &str) -> Result<Vec<FeedbackItem>, ServiceError>;
}

pub trait CommentSink: Send + Sync + 'static {
    fn record_receipt(&self, receipt: Receipt) -> Result<(), ServiceError>;
}

/// 引擎事件（coarse，只带 ID/计数——Pattern 5 的载荷契约）
#[derive(Clone, Debug, serde::Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum EngineEvent {
    DocChanged { project_id: String, doc_id: String },
    InboxUpdated { project_id: String, unread: u32 },
    Resync,                       // 总线 Lagged 时发出：前端做全量失效
}
```

**Why 同步而非 async:** 底层 prism-store 是阻塞 rusqlite。同步 trait 天然 object-safe（`Arc<dyn FeedbackSource>` 直接可用），prism-types 不需要 `async-trait` 依赖，也不触碰 AFIT 的 dyn-safety 问题。prism-mcp 的 rmcp handler 是 async 的，在其中这样调用：

```rust
let svc = self.feedback.clone();                       // Arc<dyn FeedbackSource>
let pid = project_id.clone();
let items = tokio::task::spawn_blocking(move || svc.list_feedback(&pid))
    .await
    .map_err(|e| /* join error */)??;
```

**Why `dyn` + `Arc` 而非泛型:** `StreamableHttpService::new(service_factory: impl Fn() -> Result<S, Error> + Send + Sync + 'static, session_manager: Arc<M>, config)` [VERIFIED: docs.rs/rmcp StreamableHttpService] 的 factory 每 session 调一次。用 `Arc<dyn …>` 时 factory 只是 clone 一个 Arc，prism-mcp 的公开类型保持单一具体类型；用泛型则会把 engine 具体类型经 `S` 泄漏进 axum service 类型签名，且 workspace 里每个消费方都要重新单态化。

**rmcp 2.2 + axum 0.8 挂载形态**（官方示例逐行照搬 [VERIFIED: modelcontextprotocol/rust-sdk examples/servers/src/counter_streamhttp.rs]）:

```rust
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};

pub fn build_router(deps: McpDeps, ct: tokio_util::sync::CancellationToken) -> axum::Router {
    let svc = StreamableHttpService::new(
        move || Ok(PrismHandler::new(deps.clone())),          // 每 session 一个 handler
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default().with_cancellation_token(ct.child_token()),
    );
    axum::Router::new()
        .nest_service("/mcp", svc)
        .layer(axum::middleware::from_fn(require_bearer))     // ③ bearer
        .layer(axum::middleware::from_fn(require_local_host)) // ② Host + Origin allowlist
}
```

**中间件顺序（Phase 1 只建骨架，但形态现在定死）:** axum `.layer()` 是**由内向外**叠加——后 `.layer()` 的先执行。上面写法下请求顺序是 `require_local_host` → `require_bearer` → mcp service，即先做 DNS-rebinding 防护再做鉴权，符合 PITFALLS Pitfall 9 的三层要求（Host ∈ {127.0.0.1, localhost} + Origin allowlist + bearer，缺一即 403）。绑定用 `TcpListener::bind("127.0.0.1:0")`（Phase 1 用 0 让 OS 分配，端口策略是 Phase 6 的事）。

**Phase 1 交付边界：** prism-mcp 只需要一个能编译、能起 axum、能通过一次注入 trait 返回假数据的最小 handler。工具面、端口发现、CLI helper 契约全部是 Phase 6。

### Pattern 5: notify-then-fetch（engine bus → Tauri event → TanStack Query）

**Tauri 官方原文（这是 A1 的依据）:** "Event listeners are called in the order they are registered, but if a listener is async and the event emitter sends multiple events in rapid succession, the listeners may process events out of order." 以及 "For ordered, high-throughput data delivery, consider using Channels instead of the event system." 事件载荷"always JSON strings"，且缺少命令具备的 capability 安全控制。 [CITED: v2.tauri.app/develop/calling-frontend]

**engine 侧（tauri-free）:**

```rust
// prism-engine
pub struct Engine { bus: tokio::sync::broadcast::Sender<EngineEvent>, /* … */ }
impl Engine {
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<EngineEvent> { self.bus.subscribe() }
    pub(crate) fn publish(&self, ev: EngineEvent) { let _ = self.bus.send(ev); }
}
```

**shell adapter（唯一 link tauri 的地方）—— `Lagged` 分支是必须的:**

```rust
// src-tauri/src/bus_adapter.rs
use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast::error::RecvError;

pub fn spawn(app: AppHandle, mut rx: tokio::sync::broadcast::Receiver<EngineEvent>) {
    tauri::async_runtime::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(ev)                 => { let _ = app.emit("prism://changed", ev); }
                Err(RecvError::Lagged(_)) => {
                    // broadcast 在接收端落后时会丢弃消息。粗粒度失效语义下丢弃无害，
                    // 但必须补一次全量失效，否则前端会静默停留在旧数据上。
                    let _ = app.emit("prism://changed", EngineEvent::Resync);
                }
                Err(RecvError::Closed) => break,
            }
        }
    });
}
```

`RecvError::Lagged` 是 `tokio::sync::broadcast` 的定义行为（有界环形缓冲，慢接收者丢最旧消息）。这条分支是"notify-then-fetch 相对于 push-payload 的全部价值所在"——载荷是粗粒度失效信号，所以丢消息可以用一次 `Resync` 无损补偿；若走 push-payload（Anti-Pattern 2）则丢的是不可恢复的业务数据。 [ASSUMED — broadcast 的 Lagged 语义来自训练知识，未在本 session 直查 tokio 文档；但它是该 API 广为人知的核心语义，且不影响设计正确性：即使不 Lagged，Resync 分支也是无害冗余]

**前端（D-07 的最终模式）:**

```ts
// src/lib/useEngineInvalidation.ts
export function useEngineInvalidation() {
  const qc = useQueryClient();
  useEffect(() => {
    const p = listen<EngineEvent>('prism://changed', (e) => {
      const ev = e.payload;
      if (ev.kind === 'resync') { qc.invalidateQueries(); return; }
      qc.invalidateQueries({ queryKey: ['docs', ev.projectId] });
    });
    return () => { p.then((un) => un()); };   // listen 返回 Promise<UnlistenFn>，必须清理
  }, [qc]);
}
```

React 19 StrictMode 下 effect 会执行两次 → 会注册两个 listener。上面的 cleanup 写法正确处理了这一点（第一次的 unlisten 在第二次注册前被调用）；**遗漏 cleanup 是这个 pattern 最常见的 bug**，表现为每次热更新后失效次数翻倍。

### Pattern 6: Channel 有序流（A1 的第二条通路）

Rust 侧（官方示例形态 [CITED: v2.tauri.app/develop/calling-frontend]）：

```rust
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
enum SmokeEvent { Started { total: u32 }, Tick { seq: u32 }, Finished { total: u32 } }

#[tauri::command]
async fn dev_smoke_stream(on_event: tauri::ipc::Channel<SmokeEvent>, total: u32) -> Result<(), String> {
    on_event.send(SmokeEvent::Started { total }).map_err(|e| e.to_string())?;
    for seq in 0..total {
        on_event.send(SmokeEvent::Tick { seq }).map_err(|e| e.to_string())?;
    }
    on_event.send(SmokeEvent::Finished { total }).map_err(|e| e.to_string())?;
    Ok(())
}
```

Channel 由**前端**创建并作为命令参数传入（`new Channel<SmokeEvent>()` from `@tauri-apps/api/core`），因此只适合请求作用域的流；引擎主动推送没有常驻 channel，这正是 FS 驱动流程必须走 Pattern 5 的原因。 [CITED: v2.tauri.app/develop/calling-frontend]

**可观测的有序性断言（成功标准 2 后半的证据）:** 冒烟页收集 `seq`，断言严格递增且无缺口（`seq[i] === i`，count === total），并断言 `Started` 首、`Finished` 末。假数据流足够（D-06 已允许）；建议 `total` 取 500 以上，让"快速连发"真正发生——`total=3` 证明不了排序。

### Anti-Patterns to Avoid

- **在 migration 里写 PRAGMA：** `rusqlite_migration` 的 `M::up` 文档明确"PRAGMA statements are generally discouraged" [CITED: docs.rs/rusqlite_migration]。`journal_mode` 属于连接打开流程，不属于 schema 版本；放进迁移会让 `validate()` 的内存库行为与真实库分叉。
- **用 `Connection::execute` 跑 `PRAGMA journal_mode=WAL`：** 该 pragma 返回一行，`execute` 会报 `ExecuteReturnedResults`。用 `execute_batch` 或 `query_row`。
- **`detail=none` 省索引：** 会禁掉 >3 字符的 MATCH 查询——中文四字词直接失效。
- **external content 表绑隐式 rowid：** `VACUUM` 后索引静默错位。
- **`cargo test --workspace` 当作 D-01 的证据：** 它会编译 shell，什么都证明不了。用 `-p` 选择集。
- **业务逻辑写进 `#[tauri::command]` 体：** ARCHITECTURE Anti-Pattern 1；每个命令应是对 facade 的单行委托。
- **在 `.await` 上持有 writer 锁：** 用 `std::sync::Mutex` 让它成为编译错误，而不是靠代码评审。
- **把 API key 写进 `settings` 表：** D-05 明确密钥只进钥匙串。建议用类型区分（`Setting` vs `SecretRef`）而非注释约定。
- **在测试里对真实登录钥匙串读写且不加 `#[ignore]`：** PITFALLS 记录 dev 期每次 rebuild 会触发 Keychain 授权弹窗；CI/headless 无解锁钥匙串，测试会挂。

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| FTS 索引与内容表同步 | 在每个写路径手工 `INSERT INTO fts` | external content + 三触发器（ai/ad/au） | 触发器由数据库强制，Phase 2/3/5 新增的写路径不可能"忘记"同步；官方文档给出的正是这套触发器 |
| schema 版本追踪 | 自建 `schema_version` 表 + 手写升级分支 | `rusqlite_migration`（用 `user_version`） | 已 pin；`validate()` 直接给出迁移单测；`user_version` 是 SQLite 原生字段，开库零开销 |
| macOS Keychain 调用 | 直接调 `security-framework` / `SecItemAdd` | `keyring-core` + `apple-native-keyring-store` | 已 pin；`keychain` vs `protected` 的签名前提差异是踩过才知道的坑（`-34018`） |
| SSE / MCP 传输 | 手写 streamable HTTP 会话管理 | rmcp `StreamableHttpService` + `LocalSessionManager` | session 生命周期、resume-by-event-id、SSE 重放全在 SDK 里；且 rmcp 会跟进 MCP spec revision |
| 只读连接的 PRAGMA 注入 | 每次 `pool.get()` 后手动设 pragma | `SqliteConnectionManager::with_init` | 手动方案必然有遗漏点；`with_init` 是连接创建时的唯一入口 |
| 测试用密钥后端 | 自建 trait + 假实现 | `keyring_core::mock::Store` | 内置，且支持 `mock.set_error(..)` 注入一次性错误来测失败分支 |
| Tauri 命令的进程内测试 | WebDriver / tauri-driver E2E | `tauri::test::mock_builder` + `get_ipc_response`（feature `test`） | 官方进程内 mock runtime，无需真实 WebView，不脆弱 |
| 并发读写的正确性论证 | 代码评审 + "应该没问题" | 真实临时目录 DB 的并发集成测试 | 见 Validation Architecture；这是成功标准 3 的唯一可信证据 |

**Key insight:** 本阶段每一个"自己写会更简单"的地方，恰好都是**错误会静默、且要到 Phase 3+ 才暴露**的地方——FTS 漏同步表现为搜不到而非报错，rowid 错位表现为搜到错文档，pragma 遗漏表现为偶发 `SQLITE_BUSY`。把这些交给数据库/库去强制，是本阶段最高杠杆的决策。

## Runtime State Inventory

> 绿地项目、无迁移/重命名工作。按协议逐项显式回答。

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | **None** — repo 内只有 `docs/` 与 `.planning/`，无任何代码、无既有数据库（`git log` 5 条提交全为文档） | 无 |
| Live service config | **None** — 尚无 n8n / Datadog / Cloudflare 等外部服务接入；MCP server 本阶段才首次创建 | 无 |
| OS-registered state | **None** — 无 launchd plist、无 pm2、无 Task Scheduler 注册项 | 无 |
| Secrets/env vars | **None 已存在**；本阶段**首次创建** Keychain 条目 `service="PrismDocs"`, `account ∈ {llm_api_key, mcp_bearer_token}`（NFR-03 双 account） | 新建，非迁移。需在 `docs/` 记录 service/account 命名，Phase 6 的 CLI helper 依赖同一对命名 |
| Build artifacts | **None** — 无 `target/`、无 `node_modules/`、无已安装包 | 无 |

**结论：本阶段无任何数据迁移任务，全部为新建。** 唯一需要在计划中固化的是 Keychain 的 `service`/`account` 命名契约——它是跨 crate（prism-llm）与跨二进制（prism-cli，Phase 6）的隐式接口，写错在 Phase 6 才暴露。

## Common Pitfalls

### Pitfall 1: `keyring_core::set_default_store` 是进程全局，与并行测试冲突

**What goes wrong:** `set_default_store` 文档："Sets the credential store used by default to create entries… This will block waiting for all other threads currently creating entries to complete… It is recommended to call this at startup before creating any entries." [CITED: docs.rs/keyring-core] Rust 的 `cargo test` 默认在**同一进程**内多线程并行跑测试。于是"设 mock store 的测试"与"设真实 Keychain 的测试"会互相覆盖全局，表现为随机失败——重跑可能通过，最容易被误判为 flaky 而加 retry。

**Why it happens:** 全局单例 + 并行测试是 Rust 测试的经典盲区，而这里的失败模式是"另一个测试的数据"而非崩溃。

**How to avoid:** 二选一（计划阶段拍板）——
(a) `cargo-nextest`：默认**每个测试一个进程**，全局天然隔离，代价是不跑 doctest（本项目 doctest 需求极低）；
(b) `serial_test::serial` 标注所有触碰默认 store 的测试 + 在测试内 `unset_default_store()` 收尾。
无论哪种，**真实 Keychain 的往返测试必须 `#[ignore]`**，由人工 `cargo test -- --ignored` 或冒烟页触发（PITFALLS 已记录 dev 期签名身份变化会反复弹授权框）。

**Warning signs:** 单跑绿、全跑红；`Error::NoEntry` 在明明刚 `set_password` 之后出现。

### Pitfall 2: `apple-native-keyring-store` 选错模块 → `PlatformError -34018`

**What goes wrong:** 用了 `protected` 模块但 app 未由 provisioning profile 签名，报 `-34018 A required entitlement isn't present`。

**Why it happens:** 两个模块名字都像"正确选择"，且 crate 默认不启用任何一个——必须显式指定 feature。

**How to avoid:** PrismDocs 是**直发公证 DMG、不进 App Sandbox、无 provisioning profile**（调研 §2.3-3 已定），因此用 `features = ["keychain"]` + `apple_native_keyring_store::keychain::Store::new()?`。README 原文："If you are writing client apps that are not code-signed by a provisioning profile… you should use the `keychain` module." [VERIFIED: 官方 README]

**Warning signs:** 编译过但运行期 `PlatformFailure`，错误码 `-34018`。

### Pitfall 3: FTS 触发器与单写者纪律的交互

**What goes wrong:** 触发器在 `documents` 的 INSERT/UPDATE/DELETE 上写 `documents_fts`。若任何写路径绕过单写者（例如 Phase 2 某处"随手拿池连接写"），先撞上的是 `query_only=ON` 的 readonly 错误——这是好事。但若有人为了"图省事"把某个池连接的 `query_only` 去掉，触发器写入就会与 writer 事务并发，回到 Pitfall 7 的 `SQLITE_BUSY` / WAL 损坏路径。

**How to avoid:** `with_init` 里的 `query_only=ON` 不设开关、不加配置项；用一个测试锁死它（见 Validation Architecture V3-b）。

**Warning signs:** 日志出现 `SQLITE_BUSY`；`-wal` 文件超过主库大小。

### Pitfall 4: 单一 workspace 下 `target/` 位置变化打破路径假设

**What goes wrong:** `src-tauri` 成为 workspace member 后，构建产物落在**根 `target/`**，`src-tauri/target/` 不再存在。任何写死 `src-tauri/target/...` 的脚本、`.gitignore` 条目、CI 缓存 key 都会失效或缓存 miss。

**How to avoid:** `.gitignore` 用根级 `/target`；CI 缓存 `~/.cargo` + `./target`；`justfile` 里的产物路径统一从 `cargo metadata --format-version 1 | jq -r .target_directory` 取。

**Warning signs:** CI 每次全量重编；`tauri build` 后找不到 `.app`。

### Pitfall 5: 前端 `listen` 未清理导致失效风暴

**What goes wrong:** `listen()` 返回 `Promise<UnlistenFn>`；React 19 StrictMode 下 effect 双执行。忘记 cleanup → 每次挂载多一个 listener → 一条 coarse event 触发 N 次 `invalidateQueries` → N 次 refetch。

**How to avoid:** effect 返回 `() => { p.then(un => un()); }`。在冒烟页加一个"收到事件次数"计数器，反复挂载/卸载后计数应保持 1:1。

**Warning signs:** devtools 网络/IPC 面板里一次变更引发多次 query；热更新后现象加剧。

### Pitfall 6: `MATCH` 参数未做 FTS5 查询语法转义

**What goes wrong:** `WHERE documents_fts MATCH ?1` 用参数绑定挡住了 SQL 注入，但**没挡住 FTS5 查询语法**。用户搜 `a"b` 或 `foo NOT bar` 会得到语法错误或意外语义。

**How to avoid:** 统一包成引号短语：`format!("\"{}\"", q.replace('"', "\"\""))`，并在测试里覆盖含 `"`、`*`、`^`、`:`、`NOT` 的输入。

**Warning signs:** 用户搜含标点的中文串时报 `fts5: syntax error near …`。

### Pitfall 7: 迁移用 `&mut Connection`，与 writer Mutex 的取用时机

**What goes wrong:** `Migrations::to_latest(&self, conn: &mut Connection)` 需要独占可变引用 [VERIFIED: docs.rs/rusqlite_migration]。若把 writer 先包进 `Mutex` 再迁移，需要 `lock()` 后取 `&mut *guard`，且此时**只读池尚不存在**——顺序搞反（先建池后迁移）会让池里的连接看到未迁移的 schema，并且它们 `query_only=ON` 也修不了。

**How to avoid:** 严格按 Pattern 1 的六步序；迁移在 `Mutex::new(writer)` **之前**用裸 `Connection` 完成。

**Warning signs:** 首次启动时读查询报 `no such table`。

## Code Examples

### 迁移定义与单测（rusqlite_migration 2.6）

```rust
// crates/prism-store/src/migrations.rs
use rusqlite_migration::{Migrations, M};
use std::sync::LazyLock;

static MIGRATIONS: LazyLock<Migrations<'static>> = LazyLock::new(|| {
    Migrations::new(vec![
        M::up(include_str!("../migrations/001_schema_v1.sql")),
        // Phase 2/3/5/7 各自 append，绝不修改已发布的迁移
    ])
});

pub fn migrations() -> &'static Migrations<'static> { &MIGRATIONS }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migrations_are_valid() {
        // 官方推荐：在内存库跑完整套迁移
        migrations().validate().expect("migration set is invalid");
    }
}
```

### keyring 往返（真实 Keychain / mock 两条路径）

```rust
// crates/prism-llm/src/secrets.rs
use keyring_core::{Entry, Error};

pub const SERVICE: &str = "PrismDocs";
pub const ACCOUNT_LLM_KEY: &str = "llm_api_key";
pub const ACCOUNT_MCP_TOKEN: &str = "mcp_bearer_token";

/// 应用启动时调用一次（macOS 真实 Keychain）。
#[cfg(target_os = "macos")]
pub fn init_default_store() -> Result<(), Error> {
    keyring_core::set_default_store(apple_native_keyring_store::keychain::Store::new()?);
    Ok(())
}

pub fn set_api_key(secret: &str) -> Result<(), Error> {
    Entry::new(SERVICE, ACCOUNT_LLM_KEY)?.set_password(secret)
}

/// 无 key 时返回 Ok(None) —— D-06 要求"可跳过，无 key 应用照常启动"
pub fn get_api_key() -> Result<Option<String>, Error> {
    match Entry::new(SERVICE, ACCOUNT_LLM_KEY)?.get_password() {
        Ok(s)               => Ok(Some(s)),
        Err(Error::NoEntry) => Ok(None),
        Err(e)              => Err(e),
    }
}

pub fn delete_api_key() -> Result<(), Error> {
    match Entry::new(SERVICE, ACCOUNT_LLM_KEY)?.delete_credential() {
        Ok(())              => Ok(()),
        Err(Error::NoEntry) => Ok(()),
        Err(e)              => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]                     // 用 cargo-nextest 时此标注可去掉
    fn roundtrip_with_mock_store() {
        keyring_core::set_default_store(keyring_core::mock::Store::new().unwrap());
        assert!(get_api_key().unwrap().is_none());
        set_api_key("sk-test-123").unwrap();
        assert_eq!(get_api_key().unwrap().as_deref(), Some("sk-test-123"));
        delete_api_key().unwrap();
        assert!(get_api_key().unwrap().is_none());
        keyring_core::unset_default_store();
    }

    #[test]
    #[serial]
    #[ignore = "touches the real login keychain; run manually: cargo test -- --ignored"]
    fn roundtrip_with_real_keychain() {
        init_default_store().unwrap();
        set_api_key("sk-real-smoke").unwrap();
        assert_eq!(get_api_key().unwrap().as_deref(), Some("sk-real-smoke"));
        delete_api_key().unwrap();
        keyring_core::unset_default_store();
    }
}
```

`Error::NoEntry` 是 `keyring_core::Error` 的具名变体（枚举为 `#[non_exhaustive]`，match 必须带 `_` 臂）[VERIFIED: docs.rs/keyring-core error::Error]。

### 并发读写集成测试（成功标准 3 的核心证据）

```rust
// crates/prism-store/tests/concurrency.rs
#[test]
fn reader_snapshot_is_isolated_from_concurrent_write() {
    let dir = tempfile::tempdir().unwrap();
    let store = std::sync::Arc::new(prism_store::open(&dir.path().join("t.db")).unwrap());

    store.write(|tx| { tx.execute("INSERT INTO projects(id,name,root_path,created_at) \
                                   VALUES('p1','P','/tmp',0)", [])?; Ok(()) }).unwrap();

    let (tx_started, rx_started) = std::sync::mpsc::channel();
    let (tx_go, rx_go) = std::sync::mpsc::channel();
    let s2 = store.clone();
    let reader = std::thread::spawn(move || {
        s2.read(|c| {
            let mut n: i64 = c.query_row("SELECT count(*) FROM projects", [], |r| r.get(0))?;
            tx_started.send(()).unwrap();
            rx_go.recv().unwrap();                      // 写在此期间发生
            n = c.query_row("SELECT count(*) FROM projects", [], |r| r.get(0))?;
            Ok(n)
        }).unwrap()
    });

    rx_started.recv().unwrap();
    // 写者不被读者阻塞（WAL）：这一步必须成功且不超时
    store.write(|tx| { tx.execute("INSERT INTO projects(id,name,root_path,created_at) \
                                   VALUES('p2','Q','/tmp',0)", [])?; Ok(()) }).unwrap();
    tx_go.send(()).unwrap();
    assert!(reader.join().unwrap() >= 1);               // 读者全程未报 SQLITE_BUSY
    assert_eq!(store.read(|c| Ok(c.query_row("SELECT count(*) FROM projects", [], |r| r.get::<_,i64>(0))?)).unwrap(), 2);
}

#[test]
fn pooled_connection_cannot_write() {
    let dir = tempfile::tempdir().unwrap();
    let store = prism_store::open(&dir.path().join("t.db")).unwrap();
    let err = store.read(|c| Ok(c.execute("INSERT INTO settings(key,value,updated_at) \
                                           VALUES('x','y',0)", [])?)).unwrap_err();
    assert!(format!("{err}").contains("readonly"), "query_only=ON not enforced: {err}");
}

#[test]
fn bundled_sqlite_meets_minimum() {
    let dir = tempfile::tempdir().unwrap();
    let store = prism_store::open(&dir.path().join("t.db")).unwrap();
    let v: String = store.read(|c| Ok(c.query_row("SELECT sqlite_version()", [], |r| r.get(0))?)).unwrap();
    // rusqlite 0.40.1 bundled = 3.53.2
    assert!(version_tuple(&v) >= (3, 51, 3), "bundled SQLite too old: {v}");
}
```

### FTS5 中文命中测试（成功标准 3 的第二半）

```rust
// crates/prism-store/tests/fts_cjk.rs
#[test]
fn chinese_query_returns_nonzero_rows() {
    let dir = tempfile::tempdir().unwrap();
    let store = prism_store::open(&dir.path().join("t.db")).unwrap();
    store.write(|tx| {
        tx.execute("INSERT INTO projects(id,name,root_path,created_at) VALUES('p1','P','/tmp',0)", [])?;
        tx.execute("INSERT INTO documents(id,project_id,rel_path,title,content,content_hash,updated_at) \
                    VALUES('d1','p1','a.md','锚定引擎设计', \
                           '本文描述 Block 锚定引擎的设计与迁移契约，覆盖 CJK 混排 mixed English content。','h',0)", [])?;
        Ok(())
    }).unwrap();

    // ① ≥3 字符：走 trigram MATCH
    assert_eq!(store.read(|c| search(c, "p1", "锚定引擎")).unwrap().len(), 1);
    // ② 3 字符边界
    assert_eq!(store.read(|c| search(c, "p1", "迁移契")).unwrap().len(), 1);
    // ③ <3 字符：走 LIKE 回退（D-02）—— unicode61 下这条必然为 0，是 tokenizer 决策的判别性用例
    assert_eq!(store.read(|c| search(c, "p1", "锚定")).unwrap().len(), 1);
    // ④ 中英混排里的英文子串
    assert_eq!(store.read(|c| search(c, "p1", "mixed")).unwrap().len(), 1);
    // ⑤ 阴性对照：不存在的词返回 0（证明不是"总是返回全部"）
    assert_eq!(store.read(|c| search(c, "p1", "量子纠缠")).unwrap().len(), 0);
    // ⑥ FTS 查询语法转义
    assert_eq!(store.read(|c| search(c, "p1", "设计\" OR 1=1")).unwrap().len(), 0);
}

#[test]
fn fts_index_follows_update_and_delete() {
    // 触发器正确性：UPDATE 后旧词搜不到、新词搜得到；DELETE 后归零
}
```

用例 ③ 是**判别性**的：若有人把 tokenizer 换回 `unicode61`，① ② ③ 全会变成 0 行；若忘了实现 D-02 的 LIKE 分流，只有 ③ 会红。用例 ⑤ 防止"回退分支写成永远匹配"。

### Tauri 命令的进程内测试（成功标准 2 的自动化半边）

```rust
// src-tauri/tests/ipc.rs   —— 需要 tauri = { version = "2", features = ["test"] }
use tauri::test::{mock_builder, INVOKE_KEY};

#[test]
fn smoke_stream_command_is_registered_and_returns_ok() {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![crate::commands::dev_smoke_stream])
        .build(tauri::generate_context!("tauri.conf.json"))
        .expect("failed to build mock app");
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();

    let res = tauri::test::get_ipc_response(&webview, tauri::webview::InvokeRequest {
        cmd: "dev_smoke_stream".into(),
        callback: tauri::ipc::CallbackFn(0),
        error:    tauri::ipc::CallbackFn(1),
        url:      "http://tauri.localhost".parse().unwrap(),
        body:     tauri::ipc::InvokeBody::default(),
        headers:  Default::default(),
        invoke_key: INVOKE_KEY.to_string(),
    });
    assert!(res.is_ok());
}
```

[VERIFIED: docs.rs/tauri/test —— `mock_builder` / `get_ipc_response` / `INVOKE_KEY` 与该示例形态直接来自官方模块文档]

**边界说明:** mock runtime 没有真实 WebView，**证明不了"事件真的到达 JS"**。因此成功标准 2 的完整证据是「进程内命令测试（自动）+ 冒烟页人工点击（手动）」两层——这也正是 D-06 设置冒烟页的原因。不要试图用 tauri-driver E2E 补这一块（脆弱且本阶段收益为零）。

### 依赖方向断言（justfile，进 CI —— `<specifics>` 明确要求）

```make
ENGINE_CRATES := "prism-types prism-store prism-fs prism-parse prism-anchor prism-llm prism-mcp prism-engine"

# 成功标准 1-b：无重复 rusqlite / reqwest / libsqlite3-sys
check-dup:
    #!/usr/bin/env bash
    set -euo pipefail
    out=$(cargo tree --workspace --duplicates --edges normal || true)
    if echo "$out" | grep -Eq '^(rusqlite|reqwest|libsqlite3-sys) v'; then
        echo "FAIL: duplicate critical crate in dependency tree"; echo "$out"; exit 1
    fi
    echo "OK: no duplicate rusqlite/reqwest/libsqlite3-sys"

# 成功标准 1-a：engine crates 的依赖树中不出现 tauri（D-01）
check-tauri-free:
    #!/usr/bin/env bash
    set -euo pipefail
    for c in {{ENGINE_CRATES}}; do
      if cargo tree -p "$c" --edges normal,build --prefix none | grep -Eq '^tauri( |$)'; then
        echo "FAIL: $c depends on tauri"; exit 1
      fi
    done
    echo "OK: all engine crates are tauri-free"

# 成功标准 1-c：prism-mcp 不依赖 prism-engine（D-09，含排除 dev 边的逃逸口）
check-no-cycle:
    #!/usr/bin/env bash
    set -euo pipefail
    if cargo tree -p prism-mcp --edges normal --prefix none | grep -q '^prism-engine'; then
      echo "FAIL: prism-mcp depends on prism-engine"; exit 1
    fi
    echo "OK: prism-mcp -> prism-types only"

# 成功标准 4：只有 prism-llm 与 prism-cli 可触网/触密钥
check-single-egress:
    #!/usr/bin/env bash
    set -euo pipefail
    for c in prism-types prism-store prism-fs prism-parse prism-anchor prism-engine; do
      if cargo tree -p "$c" --edges normal --prefix none | grep -Eq '^(reqwest|keyring-core|apple-native-keyring-store) '; then
        echo "FAIL: $c has network/secret dependency"; exit 1
      fi
    done
    echo "OK: prism-llm is the sole network+secret crate among engine crates"

# D-01：engine 可独立测试（注意：不是 --workspace）
test-engine:
    cargo test {{ replace(ENGINE_CRATES, " ", " -p ") }} --  # 展开为 -p prism-types -p ...
```

`cargo tree --edges` 的取值集包含 `normal`/`build`/`dev`/`no-dev` 等，`--duplicates` 只显示多版本共存的包 [VERIFIED: 本机 `cargo tree --help`, cargo 1.95.0]。`--edges normal` 显式排除 dev-dependency——这是必要的，因为 cargo **允许** dev 边构成的环（A→B 普通 + B→A dev），普通编译不会报错 [VERIFIED: cargo 语义 + 社区文档]。

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `keyring` v3 单 crate 的 `Entry::new` | `keyring-core` 1.0 + 平台 store crate，启动时 `set_default_store` | keyring 4.x（keyring-core 1.0.0，2026-04-21） | Phase 1 必须新增一次显式启动注册；测试可用内置 `mock::Store` |
| MCP stdio 子进程代理 | rmcp `StreamableHttpService` on axum loopback + bearer | rmcp 2.x（D-07 已决） | 无子进程、无 `.prismdocs/mcp.json`；发现机制走 CLI helper（Phase 6） |
| Tauri v1 `emit_all` / 无 Channel | Tauri v2 `Emitter` trait（`emit`/`emit_to`/`emit_filter`）+ `tauri::ipc::Channel<T>` | Tauri 2.0（`tauri::ipc` 模块新增 Channel） | 有序流必须走 Channel；事件仅用于粗粒度失效 |
| Vite 7 (esbuild/rollup) | Vite 8（Rolldown 统一打包器） | Vite 8 稳定 2026-03-12 | CLAUDE.md 已修订；本机 Node 22.18 满足 Vite 8 的 `^20.19.0 \|\| >=22.12.0` |
| SQLite 系统库 / `bundled-full` | `rusqlite features=["bundled"]`（3.53.2） | rusqlite 0.40.1（2026-06-06） | FTS5 无需额外 feature；≥3.51.3 的 WAL-reset 修复已包含 |

**Deprecated/outdated:**
- `tauri-plugin-stronghold`（v3 弃用）、`tauri-plugin-keyring`（滞后上游且违反 D-01）— 见 CLAUDE.md What-NOT-to-use
- `keyring` v4 默认 features（不启 `v1` 或 `cli` 时不暴露任何有用 API）
- rmcp <1.4.0 存在 Streamable HTTP 未校验 Host header 的 DNS-rebinding 漏洞 [GHSA-89vp-x53w-74fx]；本项目 pin 2.2 已远超该门槛，但**仍需自建 Host/Origin 中间件**（SDK 修复不替代应用层 allowlist）

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `SQLITE_OPEN_READ_ONLY` 的连接在 `-shm` 缺失时会失败，故只读池应用读写 flags + `query_only=ON` | Pattern 1 | 若不成立，改用 `SQLITE_OPEN_READ_ONLY` 会更强的防呆；影响仅为 flags 一行，不影响架构。计划阶段可用一个"删掉 -shm 后开库"的测试证伪 |
| A2 | `tokio::sync::broadcast` 在慢接收端会返回 `RecvError::Lagged(n)` 并丢弃最旧消息 | Pattern 5 | 若不成立，`Resync` 分支只是无害冗余；不影响正确性 |
| A3 | `cargo-nextest` 默认每个测试独立进程，可消解 `set_default_store` 全局冲突 | Pitfall 1 | 若不成立，退回 `serial_test` 方案（已作为备选写入）；两方案都在计划里，风险已对冲 |
| A4 | Vite 8 + Tauri v2 模板无摩擦（CLAUDE.md 已给出降级到 Vite 7 的 fallback） | Standard Stack | 前端是 render-only 薄层，降级成本近零；已有书面 fallback |
| A5 | axum 0.8 `.layer()` 语义为"后加的先执行"（由内向外） | Pattern 4 | 若顺序理解反了，Host 校验会在 bearer 之后执行——两层都在、都会 403，安全性不变，仅错误码语义顺序不同。Phase 6 落地时以实测为准 |
| A6 | Keychain service/account 命名 `PrismDocs` / `llm_api_key` / `mcp_bearer_token` | Runtime State Inventory | 命名是本阶段自定的契约，无外部约束；但必须在 Phase 1 写进文档，否则 Phase 6 CLI helper 会用另一套名字 |

## Open Questions

1. **只读池 `max_size` 取值**
   - What we know: MVP 单用户、500 文档；WAL 下读不互斥；长驻读连接会阻塞 checkpoint 导致 `-wal` 膨胀（PITFALLS Pitfall 7）
   - What's unclear: 4 是拍脑袋值；真正的约束是"读连接用完即还"而非池大小
   - Recommendation: Phase 1 用 4，把「禁止跨事件循环长持有读连接」写进 `Store::read` 的闭包式 API（已在 Pattern 2 体现，API 形态本身就禁止长持有）。数值调优留到 Phase 8 压测

2. **`cargo-nextest` vs `serial_test`**
   - What we know: 两者都能解决 Pitfall 1；nextest 隔离更彻底但引入工具链依赖且不跑 doctest
   - What's unclear: 项目是否会依赖 doctest 作为文档正确性保障
   - Recommendation: 本项目文档以 `.planning/` + `docs/` 为主、doctest 需求低 → **建议 nextest**，并在 CI 里额外跑一次 `cargo test --doc` 兜底。计划阶段拍板即可，成本对称

3. **`STRICT` 表关键字**
   - What we know: schema 示例用了 `STRICT`（SQLite 3.37+，bundled 3.53.2 支持）；它把类型错误从静默转换变成运行期错误
   - What's unclear: 是否会与 Phase 2/3 某些动态列用法冲突
   - Recommendation: 用 `STRICT`。本项目所有列类型都明确，收益（类型错误早暴露）大于风险；若 Phase 3 确有需求，去掉 `STRICT` 是一次 `ALTER`-free 的表重建，代价可接受

4. **冒烟页 Channel 样例的 `total`**
   - What we know: D-06 允许假数据流；有序性只在"快速连发"下才可能被违反
   - Recommendation: `total = 1000`，前端断言 `seq[i] === i` 且无缺口。这是 Claude's Discretion 范围内的选择

5. **`prism-fs`/`prism-parse`/`prism-anchor` 空骨架的"最小编译单元"内容**
   - What we know: D-08 要求它们建好且依赖声明齐全（让 `cargo tree -d` 覆盖真实依赖树）
   - What's unclear: 只写 `pub fn version() -> &'static str` 会不会因未使用依赖触发 `unused_crate_dependencies` lint
   - Recommendation: 每个 crate 写一个引用其主依赖的最小函数（如 prism-parse 写 `pub fn parse_smoke(md: &str) -> usize` 调 comrak 数一下 root 子节点），既保证依赖真的被链接进树，又给 Phase 2/3 一个起点

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | 全部 engine crates | ✓ | rustc/cargo 1.95.0 | — |
| `cargo tree` | 依赖方向断言（成功标准 1） | ✓ | 内置于 cargo 1.95.0，支持 `--edges`/`--duplicates` | — |
| Xcode Command Line Tools | Tauri macOS 构建、libsqlite3-sys 编译 bundled SQLite | ✓ | `/Applications/Xcode.app/Contents/Developer` | — |
| Node.js | Vite 8 / Tauri CLI | ✓ | v22.18.0（满足 Vite 8 的 `^20.19.0 \|\| >=22.12.0`） | — |
| npm | 前端依赖 | ✓ | 11.6.2 | — |
| pkg-config | 部分 native 依赖 | ✓ | 2.5.1 | — |
| macOS 登录钥匙串 | 真实 keyring 往返（成功标准 4） | ✓ | 系统自带 | mock store 覆盖自动化路径；真实往返 `#[ignore]` + 冒烟页人工验证 |
| `~/Library/Application Support` | sidecar 根（`dirs::data_dir()`） | ✓ | 存在 | 测试用 `tempfile::TempDir` 注入隔离根 |
| `@tauri-apps/cli` | `tauri dev` / `tauri build` | ✗（未安装） | — | `npm install -D @tauri-apps/cli@2`（Wave 0 任务） |
| `just` | justfile 断言脚本 | ✗ | — | 断言改写为 `scripts/check-deps.sh` + npm script；**不阻塞** |
| `cargo-nextest` | 测试进程隔离（Pitfall 1 方案 a） | ✗ | — | `serial_test` crate（方案 b），零工具链要求 |
| `cargo-deny` | 可选的依赖策略强制 | ✗ | — | `cargo tree` 断言已覆盖本阶段全部需求；**不需要** |
| Apple Developer 签名身份 | 公证 DMG | 未检测（Phase 8 才需要） | — | Phase 1 用 ad-hoc 签名；PITFALLS 提示 dev 期签名身份变化会反复触发 Keychain 弹窗，建议固定 ad-hoc 身份 |

**Missing dependencies with no fallback:** 无。

**Missing dependencies with fallback:** `@tauri-apps/cli`（npm 安装，属正常项目初始化）；`just`（可用 shell 脚本替代）；`cargo-nextest`（可用 serial_test 替代）。三者均不阻塞执行。

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework (Rust) | `cargo test` 内置 harness（+ 可选 `cargo-nextest` 做进程隔离）+ `tempfile` 3 + `serial_test` 3 |
| Framework (前端) | Vitest（冒烟页/hook 单测，最小覆盖） |
| Config file | **none — 全部由 Wave 0 创建**（无 `Cargo.toml`、无 `vite.config.ts`、无 `package.json`；repo 目前只有 `docs/` + `.planning/`） |
| Quick run command | `cargo test -p prism-types -p prism-store -p prism-llm -p prism-mcp -p prism-engine` |
| Full suite command | `just check-dup && just check-tauri-free && just check-no-cycle && just check-single-egress && cargo test --workspace && npm run test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| INFRA-01 | engine workspace 不依赖 tauri 即可测试（D-01） | 依赖断言 | `just check-tauri-free` | ❌ Wave 0 |
| INFRA-01 | engine-only 测试全绿 | integration | `cargo test -p prism-types -p prism-store -p prism-llm -p prism-mcp -p prism-engine` | ❌ Wave 0 |
| INFRA-01 | `cargo tree -d` 无重复 rusqlite/reqwest/libsqlite3-sys | 依赖断言 | `just check-dup` | ❌ Wave 0 |
| INFRA-01 | prism-mcp 无 facade 依赖（D-09） | 依赖断言 | `just check-no-cycle` | ❌ Wave 0 |
| INFRA-01 | prism-mcp 经注入 trait 返回数据 | unit | `cargo test -p prism-mcp trait_injection` | ❌ Wave 0 |
| INFRA-01 | 总线事件 → coarse 载荷映射（含 Lagged→Resync） | unit | `cargo test -p prismdocs-shell bus_adapter` | ❌ Wave 0 |
| INFRA-01 | Channel 命令可调用且返回 Ok | integration（`tauri::test`） | `cargo test -p prismdocs-shell --features test ipc` | ❌ Wave 0 |
| INFRA-01 | 事件往返 & Channel 有序（经真实 WebView） | manual-only | 冒烟页：点击 → 事件计数 1:1；流式 `seq` 严格递增无缺口（total=1000） | ❌ Wave 0（mock runtime 无真实 WebView，此路径无法自动化） |
| INFRA-02 | 迁移集合有效 | unit | `cargo test -p prism-store migrations_are_valid` | ❌ Wave 0 |
| INFRA-02 | WAL 下并发读写不阻塞、无 BUSY | integration | `cargo test -p prism-store --test concurrency reader_snapshot_is_isolated` | ❌ Wave 0 |
| INFRA-02 | 池连接不可写（`query_only=ON`） | integration | `cargo test -p prism-store --test concurrency pooled_connection_cannot_write` | ❌ Wave 0 |
| INFRA-02 | bundled SQLite ≥3.51.3 | integration | `cargo test -p prism-store --test concurrency bundled_sqlite_meets_minimum` | ❌ Wave 0 |
| INFRA-02 | 中文查询返回非零结果（trigram） | integration | `cargo test -p prism-store --test fts_cjk chinese_query_returns_nonzero_rows` | ❌ Wave 0 |
| INFRA-02 | FTS 索引随 UPDATE/DELETE 同步（触发器） | integration | `cargo test -p prism-store --test fts_cjk fts_index_follows_update_and_delete` | ❌ Wave 0 |
| INFRA-02 | 关闭时 `wal_checkpoint(TRUNCATE)` 生效 | integration | `cargo test -p prism-store --test concurrency wal_truncated_on_close` | ❌ Wave 0 |
| INFRA-03 | 密钥往返（mock store） | unit | `cargo test -p prism-llm roundtrip_with_mock_store` | ❌ Wave 0 |
| INFRA-03 | 密钥往返（真实 Keychain） | manual-only | `cargo test -p prism-llm -- --ignored roundtrip_with_real_keychain`（或 settings 页人工操作） | ❌ Wave 0（CI/headless 无解锁钥匙串） |
| INFRA-03 | 无 key 时应用照常启动（`NoEntry` → `Ok(None)`） | unit | `cargo test -p prism-llm no_key_is_not_an_error` | ❌ Wave 0 |
| INFRA-03 | 仅 prism-llm 持有网络/密钥依赖 | 依赖断言 | `just check-single-egress` | ❌ Wave 0 |
| INFRA-03 | 代码/配置无明文密钥 | 静态检查 | `git grep -nE '(sk-[A-Za-z0-9]{16,}\|api[_-]?key\s*=\s*["\x27][^"\x27]{8,})' -- ':!*.planning/*'` 无输出 | ❌ Wave 0 |
| INFRA-03 | base_url 校验（仅 http/https，拒绝其他 scheme） | unit | `cargo test -p prism-store settings_base_url_validation` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p <被改动的 crate>` + `cargo clippy -p <crate> -- -D warnings`
- **Per wave merge:** `just check-dup && just check-tauri-free && just check-no-cycle && just check-single-egress && cargo test --workspace`
- **Phase gate:** 上述全套 + `npm run test` + 冒烟页四项人工验证（事件往返 / Channel 有序 / 中文搜索 / 真实钥匙串往返）全部通过后才进 `/gsd-verify-work`

### Wave 0 Gaps

本项目为绿地，**测试基础设施 100% 缺失**，Wave 0 必须建立：

- [ ] 根 `Cargo.toml`（`[workspace] members` + `[workspace.dependencies]`）与 9 个 crate 的 `Cargo.toml`
- [ ] `src-tauri/Cargo.toml`（含 `[features] test = ["tauri/test"]`）与 `tauri.conf.json`
- [ ] `package.json` + `vite.config.ts` + `vitest` 配置
- [ ] `justfile`（4 条依赖方向断言 + `test-engine`）或等价 `scripts/check-deps.sh`
- [ ] `crates/prism-store/tests/concurrency.rs` — 覆盖 INFRA-02 的 WAL/query_only/版本三项
- [ ] `crates/prism-store/tests/fts_cjk.rs` — 覆盖 INFRA-02 的中文命中与触发器同步
- [ ] `crates/prism-store/src/migrations.rs` 内的 `migrations_are_valid` 单测
- [ ] `crates/prism-llm/src/secrets.rs` 内的 mock/真实 keyring 测试（真实路径 `#[ignore]`）
- [ ] `crates/prism-mcp/tests/trait_injection.rs` — 用假 `FeedbackSource` 实现验证注入路径
- [ ] `src-tauri/tests/ipc.rs` — `tauri::test::mock_builder` 命令注册测试
- [ ] `src-tauri/src/bus_adapter.rs` 的纯函数映射单测（含 Lagged→Resync）
- [ ] 测试隔离约定：所有 store 测试用 `tempfile::TempDir` 注入 data root，**不得**触碰真实 `~/Library/Application Support/PrismDocs/`
- [ ] 框架安装：Rust harness 内置无需安装；`npm install -D vitest`；（可选）`cargo install cargo-nextest`
- [ ] CI 工作流（GitHub Actions，macOS runner）串起 Per-wave 命令

## Security Domain

**ASVS Level 1；`security_enforcement: true`。** 本阶段建立的是安全边界骨架——Phase 6 的 MCP 攻击面、Phase 2 的文件系统攻击面都建在这些边界上。

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | 部分（骨架） | MCP bearer token 生成与存储：`getrandom`/`rand` 生成 ≥256-bit，存 Keychain `account="mcp_bearer_token"`；中间件用**常数时间比较**（`subtle::ConstantTimeEq`），不用 `==` |
| V3 Session Management | 否（Phase 6） | rmcp `LocalSessionManager` 负责；Phase 1 只需 `nest_service` 挂载形态正确 |
| V4 Access Control | 是 | ① 只读池 `query_only=ON` = 数据库层最小权限；② MCP 中间件 Host∈{127.0.0.1, localhost} + Origin allowlist + bearer，三层缺一即 403；③ loopback-only 绑定 |
| V5 Input Validation | 是 | ① 所有 SQL 用 rusqlite 参数绑定，**禁止 `format!` 拼 SQL**；② FTS5 `MATCH` 串独立做**查询语法**转义（参数绑定不覆盖这层，见 Pitfall 6）；③ settings 的 `base_url` 用 `url::Url` 解析并限定 scheme ∈ {http, https}；④ 项目 root_path 必须 `canonicalize()` 后校验 |
| V6 Cryptography | 是 | 密钥存储全权交给 macOS Keychain（`keyring-core` + `apple-native-keyring-store`），**绝不自建加密存储**；TLS 由 reqwest 提供；blake3 仅作内容指纹，**不作为安全原语使用**（非 MAC、非密码哈希） |
| V7 Error Handling & Logging | 是 | `tracing` 输出中禁止出现密钥：为持有密钥的类型手写 `Debug` 打印 `<redacted>`；LLM 请求日志只记 endpoint host 与状态码，不记 header |
| V12 File Resources | 部分 | sidecar 根固定为 `dirs::data_dir()/PrismDocs/`；所有 sidecar 路径拼接后必须校验仍在根内（防止 project-id 里的 `../`） |
| V14 Configuration | 是 | 无明文密钥进 repo / `settings` 表 / `tauri.conf.json`；`.gitignore` 覆盖 `/target`、`node_modules`、`*.db`、`*.db-wal`、`*.db-shm`、`.env*` |

### Known Threat Patterns for Tauri v2 + Rust engine + loopback MCP

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| DNS rebinding 到 127.0.0.1 的 MCP server（rmcp <1.4.0 有真实 CVE GHSA-89vp-x53w-74fx） | Spoofing | Host header 校验 + Origin allowlist + bearer，三层；pin rmcp 2.2 已含上游修复但不替代应用层校验 |
| bearer token 被可预测方式生成/交付 | Spoofing | CSPRNG 生成、Keychain 存储、常数时间比较；不写入 repo 内任何文件（D-07 已废弃 `.prismdocs/mcp.json`） |
| API key 经日志/错误信息泄漏 | Information Disclosure | 自定义 `Debug`；错误类型不携带密钥原文；`tracing` 过滤 |
| API key 误入 SQLite（可被整目录备份带走） | Information Disclosure | D-05 类型层隔离：`settings` 表只接受 `Setting`，密钥走 `SecretRef`；加一条测试断言 `settings` 表内不存在含 `key`/`token`/`secret` 的键名 |
| SQL 注入（含 FTS5 查询语法注入） | Tampering | 参数绑定 + FTS 查询串转义，两层都要 |
| 用户配置的 `base_url` 指向 http 明文或内网地址（SSRF 雏形） | Information Disclosure / Tampering | 强制 scheme ∈ {http, https}；非 localhost 的 http 给出明确警告；Phase 4 落地实际请求时复核 |
| 路径穿越经 project-id 写出 sidecar 根之外 | Tampering | project-id 限定为 ULID 字符集；拼接后 `starts_with(data_root)` 校验 |
| 多连接并发写 / checkpoint 导致 `database disk image is malformed` | Denial of Service / Tampering | 单写者 + `query_only` 读池 + bundled SQLite 3.53.2（含 3.51.3 的修复）+ `cargo tree -d` 保证只链接一份 SQLite |
| WAL 无限膨胀（长驻读连接阻塞 checkpoint） | Denial of Service | 闭包式 `read()` API 强制用完即还；退出前 `wal_checkpoint(TRUNCATE)` |

## Sources

### Primary (HIGH confidence)

- **sqlite.org/fts5.html** — trigram tokenizer 选项与最短匹配长度、`detail=` 的查询长度限制、external content/contentless 差异、同步触发器、`rebuild` 命令（原文引用）
- **sqlite.org/pragma.html** — `journal_mode` 持久性 vs 其余 pragma 的每连接性、`wal_checkpoint(TRUNCATE)` 语义
- **sqlite.org/lang_vacuum.html** — "VACUUM may change the ROWIDs of entries in any tables that do not have an explicit INTEGER PRIMARY KEY"
- **docs.rs/keyring-core**（via Context7） — `set_default_store`、`Entry::new`、`Error` 全变体、`mock::Store` + `mock::Cred::set_error`
- **github.com/open-source-cooperative/apple-native-keyring-store** — README（keychain vs protected 的签名前提、`-34018`）+ `examples/instantiation.rs`（`keychain::Store::new()` 完整用法）
- **crates.io API `/crates/rmcp/2.2.0`** — feature flag 精确名称：`server`、`transport-streamable-http-server`、`transport-streamable-http-server-session`、`server-side-http`（**解除 STATE.md 的 Phase 1 待办**）
- **github.com/modelcontextprotocol/rust-sdk `examples/servers/src/counter_streamhttp.rs`** — `StreamableHttpService::new` + `LocalSessionManager` + `Router::nest_service("/mcp", …)` + graceful shutdown 的官方形态
- **docs.rs/rmcp StreamableHttpService** — 构造函数签名（service_factory / session_manager / config）
- **docs.rs/rusqlite_migration**（via Context7） — `Migrations::new`、`M::up`（"PRAGMA statements are generally discouraged"）、`to_latest(&mut Connection)`、`validate()`
- **docs.rs/r2d2_sqlite SqliteConnectionManager** — `file`/`memory`/`with_flags`/`with_init` 完整方法集
- **docs.rs/tauri/test** — `test` feature、`mock_builder`/`mock_app`/`get_ipc_response`/`INVOKE_KEY` 与完整示例
- **v2.tauri.app/develop/calling-frontend** — 事件乱序警告原文、"consider using Channels" 指引、`Emitter` 三方法、Channel 前端创建模式
- **docs.rs/dirs data_dir** — macOS 返回 `$HOME/Library/Application Support`
- **github.com/rusqlite/rusqlite releases v0.40.1** — "Bump bundled SQLite version to 3.53.2"
- **crates.io registry + npm registry**（2026-07-28 直查） — 全部版本与包合法性信号
- **本机环境探测** — rustc/cargo 1.95.0、node v22.18.0、npm 11.6.2、Xcode CLT、`cargo tree --help` 的 `--edges`/`--duplicates` 取值

### Secondary (MEDIUM confidence)

- v2.tauri.app/develop — `.taurignore` 与 cargo workspace root 的关系（官方唯一提及 workspace 的位置，但无集成指引）
- rust-lang/cargo 社区讨论 — 普通依赖环为硬错误、dev-dependency 环被允许
- SQLite user forum — trigram LIKE/GLOB 优化需 ≥3 连续非通配符字符，否则线性扫

### Tertiary (LOW confidence)

- `SQLITE_OPEN_READ_ONLY` 与 `-shm` 创建的交互（A1）— 训练知识，未直查；已给出证伪方法
- `tokio::sync::broadcast` `RecvError::Lagged` 语义（A2）— 训练知识，未直查；错判无害
- axum 0.8 `.layer()` 叠加方向（A5）— 训练知识；错判不影响安全性

### 项目内已有研究（本次未重复调研）

`.planning/research/{STACK,ARCHITECTURE,PITFALLS}.md`、`.claude/CLAUDE.md` Technology Stack、`docs/调研_技术基建与开发Phase.md` §2.1/§2.3/§4、`.planning/ROADMAP.md` Phase 1、`.planning/phases/01-foundation-skeleton/01-CONTEXT.md`

## Metadata

**Confidence breakdown:**

- Standard stack: **HIGH** — 全部 pin 本 session 对 crates.io/npm 复核，与 CLAUDE.md 审计一致；新增的 feature flag（`rmcp` 三项、`apple-native-keyring-store` 的 `keychain`）来自 registry manifest 与官方 README 直读
- Architecture（五项决策的具体形态）: **HIGH** — 每项都有官方文档或官方示例源码支撑；仅 A5（axum layer 顺序）为推断且不影响正确性
- Pitfalls: **HIGH** — 六条中五条有官方文档直接支撑（`detail=` 限制、VACUUM rowid、`set_default_store` 全局性、`-34018`、`to_latest` 签名），一条（`execute` vs `execute_batch`）为 rusqlite 广知行为
- Validation Architecture: **HIGH（结构）/ MEDIUM（命令细节）** — 测试框架与命令形态确定，但因项目为绿地、文件全部不存在，具体测试函数名与路径由计划阶段最终确定
- Security domain: **MEDIUM-HIGH** — ASVS 映射与威胁模式基于本阶段实际组件推导；MCP 相关控制的落地验证在 Phase 6

**Research date:** 2026-07-28
**Valid until:** 2026-08-27（30 天）—— 唯一的快速变动项是 rmcp（2.x → 3.0 beta 在途）；若 Phase 6 开工时 rmcp 3.0 已稳定，需复核 `StreamableHttpService` API 与 feature 名称。SQLite/keyring-core/rusqlite 系列稳定，无需复查。
