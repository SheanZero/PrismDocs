# Phase 1: 基建骨架 - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-28
**Phase:** 1-基建骨架
**Areas discussed:** FTS5 CJK tokenizer 方案, Schema v1 覆盖范围, 薄 shell 前端做到什么程度, Crate 骨架完整度

---

## FTS5 CJK tokenizer 方案

| Option | Description | Selected |
|--------|-------------|----------|
| trigram 统一单索引（Recommended） | 一张 FTS 表、一套查询逻辑，中英文都走 substring 匹配，天然处理 CJK 混排；代价：索引体积 ~3×、英文无词干提取——500 文档/2000 卡片规模下 <300ms 预算轻松达标，简单性胜出 | ✓ |
| 双索引（unicode61 英文 + trigram 中文） | 英文 Base 文档用 unicode61 词级匹配（索引小、相关性略好），中文列用 trigram；代价：两张 FTS 表、查询 fan-out + 合并排序，英文文档里的 CJK 混排段落仍会漏 | |

**User's choice:** trigram 统一单索引

| Option | Description | Selected |
|--------|-------------|----------|
| LIKE 回退全扫（Recommended） | 查询层判长度：≥3 字符走 trigram MATCH，<3 字符自动降级 LIKE '%xx%' 线性扫。500 文档规模下全扫也远在 300ms 预算内，用户无感，实现约十行 | ✓ |
| 接受 ≥3 字限制 | 搜索框提示「至少输入 3 字符」，2 字词搜不到。实现最简但中文体验明显受损 | |
| 你决定 | 交给实现时权衡 | |

**User's choice:** LIKE 回退全扫

---

## Schema v1 覆盖范围

| Option | Description | Selected |
|--------|-------------|----------|
| 最小骨架 + 逐 phase 增量迁移（Recommended） | v1 只建 Phase 1 验证所需，后续每个 phase 用 rusqlite_migration 加自己的表；避免在 comments/cards 真实需求出现前做推测式设计 | ✓ |
| 全量领域 schema 一次落地 | v1 就把 Document/Block/Comment/Card/Xref 全部建好；风险是没见过真实用法就定表结构，大概率仍要改 | |

**User's choice:** 最小骨架 + 逐 phase 增量迁移

| Option | Description | Selected |
|--------|-------------|----------|
| 严格最小（Recommended） | 只建 projects、documents（含内容列供 FTS 验证）、FTS 表、settings；versions/blocks 留给 Phase 2/3 自己的迁移 | ✓ |
| 预建 versions/blocks 空骨架 | 把 Phase 2/3 肯定要的表先建空壳；同全量方案的缩小版——推测式设计 | |

**User's choice:** 严格最小

| Option | Description | Selected |
|--------|-------------|----------|
| SQLite settings 表（Recommended） | k/v 表存主库；INFRA-05 要求数据库单目录整体备份可恢复——配置随库备份，单一真相源，事务一致 | ✓ |
| App Support 下 JSON 文件 | 人类可直接编辑，坏库时配置仍在；代价：多一套读写路径，备份需覆盖两处 | |
| 你决定 | 交给实现时权衡 | |

**User's choice:** SQLite settings 表

---

## 薄 shell 前端做到什么程度

| Option | Description | Selected |
|--------|-------------|----------|
| settings 页 + dev 冒烟页（Recommended） | 真实交付物只有 settings 页（API key 写钥匙串 + base_url，可跳过）；另建隐藏 dev 冒烟页承载验证按钮（总线事件往返、Channel 流式、FTS 中文查询），后续 phase 逐步替换 | ✓ |
| 纯 cargo test + 空窗口 | 前端几乎为零；但 IPC 双通路的「经 WebView 往返」难以真正被证明 | |
| 最小应用壳 | settings + 文档树空壳 + 主布局框架；在没有导入功能时排布局属推测式设计 | |

**User's choice:** settings 页 + dev 冒烟页

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 1 立 TanStack Query 模式（Recommended） | 冒烟页的总线事件往返直接用「coarse event → invalidateQueries → refetch」实现——A1 要验证的就是这个最终模式，后续 phase 沿用 | ✓ |
| 冒烟页手写 fetch | Phase 1 不引依赖，手写 invoke+setState；更轻但验证的不是最终模式 | |
| 你决定 | 交给实现时权衡 | |

**User's choice:** Phase 1 立 TanStack Query 模式

---

## Crate 骨架完整度

| Option | Description | Selected |
|--------|-------------|----------|
| 全 crate 空骨架（Recommended） | facade + store/fs/parse/anchor/llm/mcp 全部建好（未到 phase 的只有 lib.rs + 依赖声明 + 最小编译单元）；cargo tree -d 覆盖真实依赖树，版本 pin 冲突 Phase 1 就暴露 | ✓ |
| 只建用到的 + prism-mcp | store/engine/shell + prism-mcp；更精简，但依赖树不全时 cargo tree -d 结论不稳，reqwest 重复风险推迟到 Phase 4 暴露 | |

**User's choice:** 全 crate 空骨架

| Option | Description | Selected |
|--------|-------------|----------|
| 独立 prism-types 小 crate（Recommended） | 只含 trait + 共享类型的零依赖 crate，prism-mcp 与 prism-engine 都依赖它；依赖方向清晰，F7 注册新 trait 时不动 prism-mcp | ✓ |
| trait 定义在 prism-mcp 内 | 少一个 crate，但 engine 为实现 trait 必须依赖 prism-mcp，协议层变成被依赖方 | |
| 你决定 | 交给实现时权衡 | |

**User's choice:** 独立 prism-types 小 crate

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 1 建空占位 binary（Recommended） | 只依赖 keyring + reqwest 的空 main；成本近零，workspace 形状一次定型，externalBin 雷区的依赖面早可见 | ✓ |
| Phase 6 再建 | 到 MCP server 上线时一并建；少一个空壳但 workspace 成员列表 Phase 6 还要动 | |
| 你决定 | 交给实现时权衡 | |

**User's choice:** Phase 1 建空占位 binary

---

## Claude's Discretion

- 冒烟页 Channel 流式验证的样例命令选型（假数据流即可）
- FTS 表具体建法（external content、大小写折叠、trigram 选项）
- settings 页字段细节与校验
- rusqlite_migration 组织方式与迁移测试写法
- rmcp 2.2 feature-flag 确切名称核验（计划阶段 5 分钟检查）

## Deferred Ideas

None — discussion stayed within phase scope
