# PrismDocs MVP 需求文档集索引

| 层级 | 文档 | 文件 | 版本 |
|---|---|---|---|
| L1 商业 | BRD：面向 Vibe Coding 的双层文档管理应用 | `BRD_PrismDocs_MVP.md` | **v0.3** |
| L2 产品总纲 | 主 PRD（信息架构 / OKF 兼容约定 / F1–F8 需求总述 / agent 协议 / 非功能 / 发布标准） | `PRD_PrismDocs_MVP.md` | **v0.3** |
| L2 调研 | 补充调研：CoWiki 与 OKF 对 BRD/PRD 的影响（v0.2 变更依据） | `调研补充_CoWiki与OKF对BRD_PRD的影响.md` | v0.1 |
| L2 调研 | 技术基建与开发 Phase 划分（含 MVP′ 速读区修订，v0.3 变更依据） | `调研_技术基建与开发Phase.md` | v0.2 |
| L2 调研 | 整体构想 v2：多项目知识层（F8 与 F6 降级的 v0.3 变更依据） | `调研_整体构想v2_多项目知识层.md` | v0.1 |
| L3 功能 | PRD-F1 项目与文档导入 | `sub-prds/PRD_F1_Import.md` | **v0.2** |
| L3 功能 | PRD-F2 Lens 层生成（口语化投影） | `sub-prds/PRD_F2_Lens.md` | v0.1 |
| L3 功能 | PRD-F3 段落级评论 | `sub-prds/PRD_F3_Comments.md` | v0.1 |
| L3 功能 | PRD-F4 评论回流 AI ★ 核心闭环（含 §4 agent 协议细化） | `sub-prds/PRD_F4_Agent_Loop.md` | **v0.2** |
| L3 功能 | PRD-F5 理解卡片 | `sub-prds/PRD_F5_Cards.md` | v0.1 |
| L3 功能 | PRD-F6 Chrome 剪藏插件 | `sub-prds/PRD_F6_Clipper.md` | v0.1 |
| L3 功能 | PRD-F7 上下文组装 Context Pack | `sub-prds/PRD_F7_Context_Pack.md` | **v0.2** |

**v0.2 变更范围**：合并 CoWiki/OKF 补充调研（BRD B1–B6、主 PRD P1–P8）。新增内容：主 PRD §2.5 OKF 兼容约定（受控 type 词表）、REQ-1.8/1.9（frontmatter 解析、log.md 物化）、REQ-4.7（Agent 贡献溯源）、REQ-7.6（导出 OKF Bundle）。F2/F3/F5/F6 不受本次变更影响，保持 v0.1。

**v0.3 变更范围（2026-07-28，构想 v2）**：①F2 修订为速读区（全文 Lens 降 P1，回归条件见 BRD §6.1）；②新增 F8 跨项目知识层（契约订阅、漂移检测→闭环，主 PRD §3-F8）；③F6 剪藏整体降 P1；④新增 REQ-4.8 变更时间线；⑤MCP 传输定为 D-07（app 内嵌 loopback streamable HTTP，作废子 PRD-F4 REQ-4.NEW-1 的 stdio 方案）。**子 PRD 尚未同步 v0.3**，待更新：F2（拆分速读区）、F4（传输与复核界面）、F5（存卡入口/跨项目双链）、F6（P1 标注）、F7（Workspace 作用域）；待新建：PRD-F8。

## 编号约定

- 需求：主 PRD 定义 REQ-x.y；子 PRD 细化为 REQ-x.y.z；子 PRD 新增且主 PRD 未覆盖的记为 REQ-x.NEW-n。
- 验收：主 PRD 定义 AC-xN；子 PRD 细化为 AC-xN-1。
- 术语与状态机以主 PRD §2 为准；冲突时主 PRD 优先，异议走各文档「开放问题」。

## 待回填主 PRD 的新增需求（v0.3 时合并）

各子 PRD 共提出 23 条 REQ-x.NEW（F1×4、F2×3、F3×3、F4×3、F5×3、F6×4、F7×3），均已在各自文档第 4 节标注「主 PRD 未覆盖，需回填」。**v0.2 仅合并了 CoWiki/OKF 变更，这 23 条的逐条评审顺延至 v0.3**：采纳者收编为正式 REQ 编号，不采纳者在子 PRD 中移除。

## 阅读顺序建议

评审产品方向 → BRD；评审整体方案 → 主 PRD；进入设计/开发 → 对应子 PRD（F4 建议最先精读，它是闭环核心且定义了对外协议）。
