# Phase 1: Foundation & Core Engine Skeleton - Context

**Gathered:** 2026-07-28
**Status:** Ready for planning

<domain>
## Phase Boundary

立起每个后续阶段都依赖的横切引擎骨架，并把唯一一条 load-bearing 架构决策落成 ADR：

- shell-agnostic Rust `core`（不依赖 Tauri）+ 薄 Tauri shell
- 本地优先 sidecar 存储（SQLite WAL + migrations）
- 安全 LLM 客户端边界（系统钥匙串 + 自定义 base_url）
- 回环 MCP 骨架（127.0.0.1 + per-install token + Origin 校验 + workspace 作用域）
- typed IPC 契约
- 首次运行引导（四步）

**明确不在本阶段：** 真正的文档导入解析（glob 配置化、frontmatter 解析、FS watcher、增量同步）属 Phase 2 F1；Block 锚定属 Phase 3；Lens 投影与评论属 Phase 4；Feedback Bundle 与回流语义属 Phase 5。

</domain>

<decisions>
## Implementation Decisions

### 整体架构分层

- **D-01:** `core` 拆成 **Cargo workspace 多 crate**（如 `prism-store` / `prism-llm` / `prism-mcp`，为 Phase 3/4 的 `prism-anchor` / `prism-lens` 预留位置），`src-tauri` 只做薄 command 层。理由：边界由编译器强制而非约定；Phase 3 的锚定引擎（护城河 + AC-3b CI 门）必须能脱离 shell 被对抗语料单测/CLI 驱动。
- **D-02:** `core` 对外暴露**单一 Engine facade**，持有共享 DB handle；Tauri command 层与 MCP server 都只是它的薄包装，不各自直连 store。理由：一个引擎两扇门，业务逻辑不重写、两边行为不漂移。
- **D-03:** **AGENT-03 的写面限制由 facade 结构性执行** —— 写操作只在 facade 暴露给 MCP 的那个子集里（评论回执）。agent 不能创建/删除评论与卡片这条约束落在类型/接口层，而非运行时 if 判断。

### 运行时架构 ADR（SC-5 必须落章）

- **D-04:** **先全 Rust 原生**：`rmcp` 做 MCP、`async-openai` 做 OpenAI 兼容端点、薄 `reqwest` 手写 Anthropic SSE。零 Node runtime → 最小包体、单工具链、密钥不出 Rust 进程。
- **D-05:** **Anthropic 客户端隔离在 trait 后**，保留 Node sidecar 降级退路。若手写 SSE 实际痛苦（边缘 case、精确 token 计数），只换实现不动上层。不预先支付双套成本。
- **D-06:** SC-5 要求这个决策**记录成正式 ADR 文件**（不是只写在 CONTEXT 里）。ADR 需说明 Rust-native 选择、被拒的 Node sidecar 方案及其触发降级的条件。

### MCP 传输与回环边界

- **D-07:** **App 自托回环 HTTP**：resident app 自己在 `127.0.0.1:<port>` 起 `rmcp` streamable-HTTP 服务，per-install token + Origin 校验。理由：字面命中 AGENT-03 的「127.0.0.1 + token + Origin」措辞（stdio 套不上 Origin 语义），且 app 本身即 MCP server，直接接 Engine facade —— 无子进程、无二次回环 IPC。
  - **注意：** 这与 `.claude/CLAUDE.md` 技术栈文档里「rmcp over **stdio**」的写法相冲突，本决策**覆盖**该处描述。planner 应据此更新技术栈文档或在 ADR 中标注。
- **D-08:** **剪藏 WS bridge（Phase 6 F6）合用同一回环服务**，走不同路由。一个回环监听面同时服务两个需求。Phase 1 只立骨架，不实现剪藏协议。
- **D-09:** MCP server **随 app 常驻**（app 起即起）。
- **D-10:** **固定默认端口，被占则向上扫**；onboarding 把实际 port + token 回写进项目的 `.mcp.json` / 协议片段。token 本体 per-install 存**系统钥匙串**，不进 git（`.prismdocs/` 建议 gitignore）。理由：与 F4「一键生成 CLAUDE.md/AGENTS.md 协议片段」一体，也最好调试。

### 存储位置与备份模型

- **D-11:** sidecar SQLite 存 **`~/Library/Application Support/PrismDocs/<项目路径 keyed>/`**，不放进用户仓库。理由：源仓库全程干净、零 git 噪声、不泄露私人评论（与 CLAUDE.md "What NOT to Use" 一致）。
- **D-12:** NFR-02「单目录可备份」靠**显式的「备份/导出本项目数据」功能**满足（把 app-support 那份打包），而非靠把 DB 放进项目里。
- **D-13:** 项目被移动/重命名后靠 **project-id 映射存活**（项目侧留标记 → app-support 目录），不因路径变化丢数据。具体形式由 planner 决定。
- **D-14:** SQLite 开 **WAL 模式**（MCP reader 与 app writer 不互相阻塞）。

### 首次运行引导与 LLM provider

- **D-15:** **双族 provider 首发都要**：Anthropic + OpenAI 兼容（自定义 `base_url`，`async-openai` 覆盖代理/本地模型/长尾兼容端点）。
- **D-16:** onboarding 做**一次轻量真实连接测试**（如 count_tokens / models / 极小 completion），把「钥匙串 → reqwest → 用户端点」整条路提前跑通。理由：LLM 边界是隐性风险点（SSE、错误处理、base_url 兼容性），骨架期用一次廉价测试摧实，比推到 Phase 4 才爆强。这同时字面验证 SC-1 / SC-3 / NFR-04。
- **D-16a（2026-07-28，用户在 01-02 执行中下达，覆盖 D-16 的强制性）：** LLM 配置**不是**进入应用的必要条件，onboarding 第一步必须可 skip。D-16 的连接测试保留为**可用能力**，不再是通过 onboarding 的**门禁**。理由：用户不应为了看一眼产品而先去搞一把 API key。
  - 保留：连接测试本身、双族 provider、keychain 落盘、base_url 归一化——全部不动。
  - 变更：`StepLlmConfig` 增加 skip 出口；跳过后 onboarding 继续走第 2–4 步（workspace / `.prismdocs/` / MCP 协议片段都不依赖 LLM，SC-4 路径不受影响）。
  - 代价（已知并接受）：D-16 原本要在骨架期摧毁的活链路风险（真实 base_url 兼容性、TLS、macOS Keychain、SSE）在用户实际配置 provider 之前不会被证伪。自动化套件覆盖了除**活调用**以外的每一环，所以残留缺口窄但真实——它降级为 phase 级 UAT 项，不再阻塞 phase 执行。
- **D-17:** **完整四步引导**：LLM 配置 → workspace 注册 → `.prismdocs/` 初始化 → MCP 协议片段。理由自洽于 D-07/D-10：既然 MCP 走固定端口 + 回写配置，协议片段必须在 Phase 1 存在，否则回环端点无从验证 SC-4。**（受 D-16a 修正：第一步可跳过，四步结构不变。）**
- **D-18:** 引导第二步的 workspace 注册**顺手只读枚举一遍默认 glob 下的 MD 文件**做可见反馈。**硬边界：只枚举，不解析 frontmatter、不建 FS watcher、不写入 documents 表。** glob 配置化、`.html` 转换、异常处理、增量同步、重命名识别全部仍归 Phase 2 F1 —— 本阶段不得实现，避免返工。
- **D-19:** `.prismdocs/` 初始化包含骨架目录 + 自动生成的英文 `README.md`（向 agent 解释协议）+ `.gitignore` 建议（PRD §4.1）。feedback/context 的实际内容语义分别归 Phase 5 / Phase 7。

### Claude's Discretion

用户明确交给我和下游 agent 拍板的部分：

- SQLite schema 广度（骨架期建多少表 / 哪些 stub）、migration 框架细节
- typed IPC 契约的具体形式（命令命名、序列化、错误类型）
- 错误处理与重试语义、数据完整性保障（NFR 可靠性）
- 测试策略与 TDD 在 Rust + Tauri 上的落地形式、CI 门的构成
- 项目 re-key（移动/重命名存活）的具体机制

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### 需求与验收
- `.planning/REQUIREMENTS.md` — AGENT-03 / NFR-02 / NFR-03 / NFR-04 的原文与验收口径（第 100、106–108 行）
- `.planning/ROADMAP.md` § Phase 1 — 五条 Success Criteria 的权威表述
- `.planning/PROJECT.md` — Constraints / Key Decisions / Open Questions（Q1、Q3 与本阶段相关）

### 产品规格
- `docs/PRD_PrismDocs_MVP.md` §4.1 — `.prismdocs/` 目录约定（feedback / context / README，gitignore 建议，CLAUDE.md/AGENTS.md 协议片段）
- `docs/PRD_PrismDocs_MVP.md` §4.2 — 本地 MCP Server 完整工具面与安全约束（本阶段只立骨架，工具实现分散在 P5/P6/P7）
- `docs/PRD_PrismDocs_MVP.md` §4.3 — 兼容目标（Claude Code 一级、Cursor、其他 agent 文件兜底）
- `docs/PRD_PrismDocs_MVP.md` §5 — 非功能需求表（本地优先 / 隐私 / 密钥 / 可靠性 / 平台）
- `docs/PRD_PrismDocs_MVP.md` §2.5 — OKF 兼容约定（架构级决策，影响 schema 设计）
- `docs/BRD_PrismDocs_MVP.md` — 商业上下文与成功指标
- `docs/sub-prds/INDEX_文档集索引.md` — 子 PRD 索引（F1–F7 各自的详细规格）

### 技术栈
- `.claude/CLAUDE.md` § Technology Stack — Tauri 2.10 / rusqlite bundled / rmcp 2.2（协议 pin **2025-11-25** stable）/ keyring / reqwest / async-openai / tokio 的版本与选型理由
  - **冲突提示：** 该文档写 rmcp over **stdio**；本 CONTEXT 的 **D-07 覆盖之**（改为回环 HTTP）。planner 需处理这处不一致。

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

**无 —— 这是 greenfield。** 仓库当前只有 `.planning/`、`docs/`、`LICENSE`、`.claude/`，没有任何源码、`package.json`、`Cargo.toml` 或 `src-tauri/`。Phase 1 从零 scaffold。

### Established Patterns

代码层面尚无既成模式。约束全部来自文档：

- **技术栈已锁**（`.claude/CLAUDE.md`）：Tauri v2 + Rust-centric core，rusqlite（bundled + WAL），rusqlite_migration，rmcp 2.2，keyring crate（**不用** tauri-plugin-stronghold，v3 将废弃），reqwest + async-openai，tokio
- **sidecar 纪律**（PROJECT.md Constraints）：Block ID / 评论 / 卡片一律不写进用户源 `.md`；frontmatter 只在 OKF 导出时物化
- **单一解析器权威**（CLAUDE.md）：未来 comrak（Rust）是 Block 边界唯一真相源，remark/react-markdown 只做渲染 —— Phase 1 的 crate 划分应为此预留

### Integration Points

本阶段建立的、后续阶段将接入的边界：

- **Engine facade** ← Phase 2 的 import/watch、Phase 3 的 anchor、Phase 4 的 lens/comments 全部挂在它上面
- **SQLite schema + migrations** ← Phase 2 起每阶段追加表
- **回环 HTTP 服务** ← Phase 5 挂 MCP 工具实现，Phase 6 挂剪藏 WS 路由
- **LLM 客户端 trait** ← Phase 4 的 Lens 投影调用（含 SSE 流式、token 计数）

</code_context>

<specifics>
## Specific Ideas

- **ADR 是交付物本身**，不只是决策记录：SC-5 要求 Tauri-vs-sidecar ADR 落成文件。需包含被拒方案与「什么条件下降级到 Node sidecar」的触发条件（D-05）。
- **连接测试是骨架期的风险摧毁工具**，不是可选打磨：宁可在 Phase 1 用一次廉价真实调用暴露 base_url / SSE / 错误处理问题，也不推到 Phase 4。
- **AGENT-03 的安全面要能被验证，不只是被声明**：SC-4 说的「拒绝缺 token 或带外来 Origin 的请求」应有对应的可执行验证手段。

</specifics>

<deferred>
## Deferred Ideas

- **剪藏 WS bridge 的实际协议实现** — Phase 6（F6）。Phase 1 只保证回环服务能容纳它（D-08）。
- **MCP 工具的具体实现**（`list_feedback` / `get_feedback` / `respond_to_comment` / `get_document_comments` / `get_context_pack` / `list_cards` / `export_okf_bundle`）— 分散在 Phase 5 / Phase 6 / Phase 7。Phase 1 只立传输层与安全边界骨架。
- **文档导入的全部真实逻辑**（glob 配置化、`.html` → Markdown、frontmatter 解析与 round-trip、FS watcher + 2s debounce、重命名按内容哈希识别）— Phase 2 F1。Phase 1 的枚举严格只读（D-18）。
- **`.prismdocs/feedback/` 与 `context/` 的内容语义** — 分别 Phase 5 / Phase 7。Phase 1 只建目录骨架与 README（D-19）。
- **Q1 Lens 模型路由**（快速模型打底 + 需决策段落强模型复核）— Phase 4 决议。Phase 1 的 LLM trait 设计应不阻碍多模型路由。
- **Q3 评论/卡片 git 同步「便携模式」** — 标记为 P1，MVP 走 sidecar + 导出备份（D-11/D-12 已按此定）。

</deferred>

---

*Phase: 1-Foundation & Core Engine Skeleton*
*Context gathered: 2026-07-28*
