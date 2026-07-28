# Phase 1: Foundation & Core Engine Skeleton - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-28
**Phase:** 1-Foundation & Core Engine Skeleton
**Areas discussed:** 应用整体架构, 运行时架构 ADR, MCP 传输与回环边界, DB 位置与备份模型, 首次运行引导 + provider

**灰区选择：** 提供了 4 个候选区，用户全选并追加了「应用整体架构」作为统摄区，共 5 个区。

---

## 应用整体架构

### 问题 1：core 骨架的模块分解怎么切？

| Option | Description | Selected |
|--------|-------------|----------|
| Cargo workspace 多 crate | 拆成 prism-store / prism-llm / prism-mcp 等子 crate，src-tauri 只做薄 command 层。边界硬、单测快、anchor 引擎能被 CI/CLI 单独驱动。代价是前期多一点 workspace 配置 | ✓ |
| 单 core crate + 模块 | 一个 prism-core crate，内部用 mod 分。更简单，但模块边界靠约定而非编译器强制，后期拆分成本高 | |
| 你拍板 | 按 mvp「骨架」最小原则自定 | |

**User's choice:** Cargo workspace 多 crate
**Notes:** 推荐理由被接受 —— Phase 3 锚定引擎（护城河，带 AC-3b 对抗语料 CI 门）早晚要能脱离 shell 被单测驱动。

### 问题 2：两个入口（Tauri IPC / MCP）怎么接进 core？

| Option | Description | Selected |
|--------|-------------|----------|
| 单一 Engine facade | core 暴露一个 Engine facade（共享 DB handle），Tauri command 层和 MCP server 都只是薄包装；权限差异（MCP 只读+回执）在 facade 上用不同方法面限定 | ✓ |
| 各自直连 store | 两边各自拿 store handle 写逻辑。更松，但容易行为漂移、权限面难统一 | |
| 你拍板 | — | |

**User's choice:** 单一 Engine facade
**Notes:** facade 同时成为 AGENT-03「MCP 无写面除评论回执」的结构性执行点 —— 写操作只在暴露给 MCP 的子集里，约束落在类型层而非运行时判断。

---

## 运行时架构 ADR

### 问题：MCP + LLM 两个子系统放 Rust 原生还是 Node sidecar？

| Option | Description | Selected |
|--------|-------------|----------|
| 全 Rust 原生 | rmcp 2.2 + async-openai + 薄 reqwest 手写 Anthropic SSE。零 Node runtime → 最小包体、单工具链、密钥不出 Rust 进程。代价：手写 Anthropic 流式 | |
| Node sidecar | 一个 sidecar 同托 @modelcontextprotocol/sdk + @anthropic-ai/sdk + openai。SSE/精确 token 计数开箱即用、与 Claude Code 同源。代价：+40–80MB runtime + 回环 IPC | |
| 先 Rust，难处再降级 | 先全 Rust 原生，Anthropic 客户端隔离在 trait 后；若手写 SSE 真痛苦再局部引 sidecar。保留退路但不先付双套成本 | ✓ |

**User's choice:** 先 Rust，难处再降级
**Notes:** 这是 SC-5 要求落章的那条 ADR 的实质内容。ADR 文件需记录被拒的 Node sidecar 方案，以及触发降级的具体条件。

---

## MCP 传输与回环边界

### 问题 1：MCP 传输走 stdio 还是回环 HTTP？

| Option | Description | Selected |
|--------|-------------|----------|
| App 自托回环 HTTP | resident app 在 127.0.0.1:\<port\> 起 rmcp streamable-HTTP，per-install token + Origin 校验——字面命中 AGENT-03。app 即 MCP server 直接接 Engine facade，无子进程/无二次回环 IPC；剪藏 WS bridge 合用同一回环服务 | ✓ |
| stdio 子进程 + 回环 IPC | agent 起个薄 rmcp-stdio 代理转发给 resident app（CLAUDE.md 字面写法）。stdio 兼容面最广，但多一层进程且 Origin/token 语义套不上 | |
| 你拍板 | — | |

**User's choice:** App 自托回环 HTTP
**Notes:** 讨论中识别出一处规格冲突 —— AGENT-03 的「127.0.0.1 + token + Origin」措辞指向 HTTP 回环，而 `.claude/CLAUDE.md` 技术栈文档写的是 rmcp over stdio。本决策覆盖后者，已在 CONTEXT.md 中显式标注供 planner 处理。

### 问题 2：per-install token + port 怎么交给 agent？

| Option | Description | Selected |
|--------|-------------|----------|
| 固定端口 + 回写配置 | 默认固定端口，被占则向上扫；onboarding 把实际 port+token 写进项目 .mcp.json / 协议片段。token 本体 per-install 存钥匙串，配置里只放引用 | ✓ |
| 每次启动临时端口 | 随机端口写到 well-known 文件，agent 配置指向 wrapper 读取。更难被碰端口，但接入多一层间接 | |
| 你拍板 | — | |

**User's choice:** 固定端口 + 回写配置
**Notes:** 与 F4「一键生成 CLAUDE.md/AGENTS.md 协议片段」一体，也最好调试。token 不进 git（.prismdocs 建议 gitignore）。server 随 app 常驻这点无异议。

---

## DB 位置与备份模型

### 问题：sidecar SQLite 放哪？

| Option | Description | Selected |
|--------|-------------|----------|
| App Support 按路径 keyed | DB 存 ~/Library/Application Support/PrismDocs/\<项目路径 hash\>/。源仓库全程干净、零 git 噪声、不泄露私人评论。NFR-02「单目录可备份」靠内置备份/导出命令满足 | ✓ |
| 项目内 .prismdocs/db | DB 跟项目走，天然单目录可备份/可携带。但与 CLAUDE.md "What NOT to Use" 相悖（git 噪声、误提交私人评论风险） | |
| 你拍板 | — | |

**User's choice:** App Support 按路径 keyed
**Notes:** 可携带性由显式的「备份/导出本项目数据」功能补上。项目移动/重命名后的 re-key 策略交由 planner 处理（倾向项目侧留 project-id 标记映射到 app-support 目录）。

---

## 首次运行引导 + provider

### 问题 1：Phase 1 的 LLM 边界做到多深？

| Option | Description | Selected |
|--------|-------------|----------|
| 双族 + 连接测试 | 首发接 Anthropic + OpenAI 兼容（base_url），onboarding 做一次轻量真实连接测试。把 钥匙串→reqwest→用户端点 整条路提前跑通，字面验证 SC-1/SC-3/NFR-04 | ✓ |
| 先只做 config 管道 | 只存 endpoint+base_url+key、校验格式，不发真请求；首次真实调用推到 Phase 4。更小，但 LLM 边界风险推后 | |
| 你拍板 | — | |

**User's choice:** 双族 + 连接测试
**Notes:** LLM 边界是隐性风险点（SSE、错误处理、base_url 兼容性），骨架期用一次廉价测试摧实优于推到 Phase 4 才爆。

### 问题 2：首次运行引导要多长？

| Option | Description | Selected |
|--------|-------------|----------|
| 只做 LLM 配置 | Phase 1 = 选 provider + base_url + key + 连接测试即收手。workspace 注册/.prismdocs 初始化跟导入走（P2），MCP 协议片段跟 F4 走（P5） | |
| 完整四步引导 | LLM 配置 + workspace 注册 + .prismdocs 初始化 + MCP 协议片段一次做完 | ✓ |
| 你拍板 | — | |

**User's choice:** 完整四步引导
**Notes:** 与前面决策自洽 —— 既然 MCP 走固定端口 + 回写配置（D-10），协议片段必须在 Phase 1 存在，否则 SC-4 的回环端点无从验证。

### 问题 3（边界澄清）：四步引导里的「workspace 注册」在 Phase 1 做到哪？

| Option | Description | Selected |
|--------|-------------|----------|
| 只选目录 + 建记录 | 选项目根 → 写 workspace 记录 → 按路径 key 开 DB → 生成 .prismdocs 骨架 + 协议片段。不扫任何 .md | |
| 顺手扫一遍文档 | 除上述外，再扫一遍默认 glob 列出找到的 MD（不解析、不监听）做可见反馈 | ✓ |
| 你拍板 | — | |

**User's choice:** 顺手扫一遍文档
**Notes:** 追加了一条硬边界写进 CONTEXT（D-18）：**只枚举，不解析 frontmatter、不建 FS watcher、不写入 documents 表**。glob 配置化、.html 转换、异常处理、增量同步、重命名识别全部仍归 Phase 2 F1，本阶段不得实现，避免与 F1 重叠返工。

---

## Claude's Discretion

用户明确交给我和下游 agent 拍板的部分（在初始灰区选择时即声明「骨架范围/schema 广度等技术细节我会自己拍板」）：

- SQLite schema 广度（骨架期建多少表 / 哪些 stub）、migration 框架细节
- typed IPC 契约的具体形式（命令命名、序列化、错误类型）
- 错误处理与重试语义、数据完整性保障
- 测试策略与 TDD 在 Rust + Tauri 上的落地形式、CI 门的构成
- 项目 re-key（移动/重命名存活）的具体机制

## Deferred Ideas

讨论全程未出现范围蔓延 —— 用户的每个选择都落在 Phase 1 边界内。以下是讨论中显式划走给后续阶段的内容：

- 剪藏 WS bridge 的实际协议实现 → Phase 6（F6）；Phase 1 只保证回环服务能容纳它
- MCP 各工具的具体实现 → Phase 5 / 6 / 7；Phase 1 只立传输层与安全边界骨架
- 文档导入全部真实逻辑（glob 配置化、.html 转换、frontmatter round-trip、FS watcher + 2s debounce、内容哈希识别重命名）→ Phase 2 F1
- `.prismdocs/feedback/` 与 `context/` 的内容语义 → 分别 Phase 5 / Phase 7
- Q1 Lens 模型路由 → Phase 4 决议；Phase 1 的 LLM trait 设计应不阻碍多模型路由
- Q3 评论/卡片 git 同步「便携模式」→ P1；MVP 已按 sidecar + 导出备份定案
