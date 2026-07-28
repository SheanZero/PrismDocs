# PrismDocs MVP 需求文档集索引

| 层级 | 文档 | 文件 | 版本 |
|---|---|---|---|
| L1 商业 | BRD：面向 Vibe Coding 的双层文档管理应用 | `BRD_PrismDocs_MVP.md` | **v0.3** |
| L2 产品总纲 | 主 PRD（信息架构 / OKF 兼容约定 / F1–F8 需求总述 / agent 协议 / 非功能 / 发布标准） | `PRD_PrismDocs_MVP.md` | **v0.3** |
| L2 调研 | 补充调研：CoWiki 与 OKF 对 BRD/PRD 的影响（v0.2 变更依据） | `调研补充_CoWiki与OKF对BRD_PRD的影响.md` | v0.1 |
| L2 调研 | 技术基建与开发 Phase 划分（含 MVP′ 速读区修订，v0.3 变更依据） | `调研_技术基建与开发Phase.md` | v0.2 |
| L2 调研 | 整体构想 v2：多项目知识层（F8 与 F6 降级的 v0.3 变更依据） | `调研_整体构想v2_多项目知识层.md` | v0.1 |
| L3 功能 | PRD-F1 项目与文档导入 | `sub-prds/PRD_F1_Import.md` | **v0.2** |
| L3 功能 | PRD-F2 速读区生成（MVP）/ 全文 Lens（P1） | `sub-prds/PRD_F2_Lens.md` | **v0.2** |
| L3 功能 | PRD-F3 段落级评论 | `sub-prds/PRD_F3_Comments.md` | v0.1 |
| L3 功能 | PRD-F4 评论回流 AI ★ 核心闭环（含 §4 agent 协议细化） | `sub-prds/PRD_F4_Agent_Loop.md` | **v0.3** |
| L3 功能 | PRD-F5 理解卡片 | `sub-prds/PRD_F5_Cards.md` | **v0.2** |
| L3 功能 | PRD-F6 Chrome 剪藏插件（P1，MVP 不交付） | `sub-prds/PRD_F6_Clipper.md` | **v0.2** |
| L3 功能 | PRD-F7 上下文组装 Context Pack | `sub-prds/PRD_F7_Context_Pack.md` | **v0.3** |
| L3 功能 | PRD-F8 跨项目知识层 ★ 防偏差 | `sub-prds/PRD_F8_Cross_Project.md` | **v0.1** |
| L4 技术设计 | TD-01 Block 锚定与迁移契约 ★ 接口冻结点（四消费方） | `技术设计_Block锚定与迁移契约.md` | **v0.1**（阈值待 M0 标定） |

**v0.2 变更范围**：合并 CoWiki/OKF 补充调研（BRD B1–B6、主 PRD P1–P8）。新增内容：主 PRD §2.5 OKF 兼容约定（受控 type 词表）、REQ-1.8/1.9（frontmatter 解析、log.md 物化）、REQ-4.7（Agent 贡献溯源）、REQ-7.6（导出 OKF Bundle）。F2/F3/F5/F6 不受本次变更影响，保持 v0.1。

**v0.3 变更范围（2026-07-28，构想 v2）**：①F2 修订为速读区（全文 Lens 降 P1，回归条件见 BRD §6.1）；②新增 F8 跨项目知识层（契约订阅、漂移检测→闭环，主 PRD §3-F8）；③F6 剪藏整体降 P1；④新增 REQ-4.8 变更时间线；⑤MCP 传输定为 D-07（app 内嵌 loopback streamable HTTP，作废子 PRD-F4 REQ-4.NEW-1 的 stdio 方案）。**子 PRD 同步状态（2026-07-28）**：v0.3 变更已全部同步——F8 新建 v0.1；F2 v0.2（§0 速读区/全文 Lens P1 状态映射 + 速读区细化）；F4 v0.3（D-07 传输、复核界面、REQ-4.8 立项、承载 F8 核对 Bundle）；F5 v0.2（跨项目双链、存卡入口改 Base，关闭 OQ-5.2/5.3）；F6 v0.2（P1 标注，关闭 OQ-1 为 loopback WebSocket）；F7 v0.3（Workspace 作用域、F8 联动）。**F3 保持 v0.1**：其 Lens 入口相关条目（REQ-3.1.1/3.1.3、AC-3a-1、E11 等）按主 PRD v0.3 视为 P1 范畴，MVP 内评论入口仅 Base 视图，无结构性冲突，待 F3 下次实质修订时一并同步；F8 的 REQ-8.NEW-1 需要 F3 增加 comment `origin` 字段（见 PRD-F8 OQ-1），届时同步。

## 编号约定

- 需求：主 PRD 定义 REQ-x.y；子 PRD 细化为 REQ-x.y.z；子 PRD 新增且主 PRD 未覆盖的记为 REQ-x.NEW-n。
- 验收：主 PRD 定义 AC-xN；子 PRD 细化为 AC-xN-1。
- 术语与状态机以主 PRD §2 为准；冲突时主 PRD 优先，异议走各文档「开放问题」。

## 待回填主 PRD 的新增需求（v0.3 时合并）

各子 PRD 共提出 23 条 REQ-x.NEW（F1×4、F2×3、F3×3、F4×3、F5×3、F6×4、F7×3），均已在各自文档第 4 节标注「主 PRD 未覆盖，需回填」。**v0.2 仅合并了 CoWiki/OKF 变更，这 23 条的逐条评审顺延至 v0.3**：采纳者收编为正式 REQ 编号，不采纳者在子 PRD 中移除。

## 阅读顺序建议

评审产品方向 → BRD；评审整体方案 → 主 PRD；进入设计/开发 → 对应子 PRD（F4 建议最先精读，它是闭环核心且定义了对外协议）。
