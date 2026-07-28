# 技术设计 TD-01:Block 锚定与迁移契约

| 项目 | 内容 |
|---|---|
| 文档类型 | 技术设计(TD-01)——**全系统接口冻结点** |
| 版本 | v0.1(接口冻结候选;阈值与权重为初始值,待 M0 Track B 标定后在 v0.2 定稿) |
| 日期 | 2026-07-28 |
| 上游文档 | 主 PRD v0.3 §2.4(锚定机制四消费方)、§3-F3 及子 PRD-F3 §6(置信度行为契约)、子 PRD-F1 REQ-1.NEW-1/2(文档身份与版本快照)、子 PRD-F4 §7.3(命中检测)、子 PRD-F8 REQ-8.1.5/8.3(跨项目订阅);《调研_技术基建与开发Phase》v0.2 §2.3 |
| 实现载体 | `prism-anchor` crate(依赖 `prism-parse`);comrak 0.54 + blake3 1.x + similar 3.1 |
| 作者 | Shean(起草协作:Claude) |

---

## 1. 目的与冻结范围

本文定义 PrismDocs 的 **Block 锚定引擎**:文档如何切分为 Block、Block 如何获得稳定身份、文档被(agent 大规模)重写后身份如何迁移、置信度如何分档、以及四个消费方各自消费什么。它是「评论 0 静默丢失」(AC-3b,发布门槛)的实现体,也是护城河所在。

**冻结的部分**(消费方可以依赖、变更需走本文档修订):

- §3 Block 模型(类型集合、边界规则、comrak 选项 pin、归一化与哈希规则)
- §4 两级身份体系与 Block 属性
- §6 置信度三档语义(行为契约,阈值数值除外)
- §7 输出类型契约与四消费方契约
- §8 不变式与事件日志义务

**不冻结的部分**(实现自由度,标定期可调):阈值数值(T_high/T_low)、评分权重、相似度算法内部选择(Myers/patience、词级/行级)、匹配的贪心策略细节——只要满足 §8 不变式与 §11 验收。

---

## 2. 两级身份体系(术语澄清)

系统中存在两级身份,分属两个引擎,不可混淆:

| 层级 | 身份 | 归属 | 机制 |
|---|---|---|---|
| 文档级 | Document ID | F1(prism-fs / prism-store) | 归一化全文 SHA-256 + 路径启发式;重命名/移动识别(REQ-1.NEW-1) |
| Block 级 | **Block ID** | 本契约(prism-anchor) | 不透明稳定 ID + 内容哈希/位置/标题路径三属性做**迁移匹配** |

关键澄清(主 PRD §2.4 的"内容哈希 + 位置启发式"易被误读):**Block ID 本身是一个不透明的稳定标识符**(ULID,首次见到该 Block 时分配,永不复用);内容哈希、标题路径、序号是它的**属性**,用于版本间的迁移匹配——ID 不由内容派生,否则内容一改 ID 即变,锚定无从谈起。

文档重命名/移动由 F1 在上游解决(Document ID 不变),本引擎始终在同一 Document ID 的版本序列内工作。

---

## 3. Block 模型

### 3.1 解析器与选项 pin(决定性前提)

**comrak 0.54 是唯一的 Block 边界真相源**;前端 react-markdown 仅渲染,任何锚定/diff 不得依赖它(What-NOT-to-use 首条)。comrak 选项**必须全局唯一且版本化**(选项变化 = Block 边界可能变化 = 全量重建风险,升级需迁移方案):

```rust
// prism-parse: PARSE_OPTIONS_V1 —— 变更此常量即触发 parse_options_version 递增
ComrakOptions {
    extension: ComrakExtensionOptions {
        table: true, strikethrough: true, tasklist: true, autolink: true,
        footnotes: false,                       // GFM 无 footnotes,保持关闭
        front_matter_delimiter: Some("---"),    // frontmatter 剥离,不进 Block 树(归 F1 REQ-1.8)
        ..default()
    },
    parse: ComrakParseOptions { smart: false, ..default() },  // 绝不改写源语义
    // render 选项与本引擎无关(只 parse 不 render)
}
```

### 3.2 Block 边界规则

Block = 文档 AST 中**根级(剥离 frontmatter 后)的每个顶层节点**;容器只取最外层:

| BlockKind | 说明 |
|---|---|
| `Heading{level}` | 标题自身是 Block(可被评论),同时贡献 heading_path |
| `Paragraph` | 段落 |
| `CodeBlock{lang}` | 围栏/缩进代码块 |
| `Table` | GFM 表格(整表一个 Block) |
| `List{ordered}` | **整个列表一个 Block**(不下钻到 list item——粒度权衡见 §9-7) |
| `Blockquote` | 整个引用块一个 Block(内部不再切分) |
| `Html` | 根级 HTML 块 |
| `ThematicBreak` | 分隔线(可锚定但通常无评论价值) |

嵌套内容(如 blockquote 里的代码块)属于其最外层容器 Block,不独立成块。

### 3.3 归一化与内容哈希

`content_hash = blake3(normalize(source_slice))`,其中 `source_slice` 按 comrak `sourcepos` 从源文件字节切出,`normalize` 规则(**与 F1 文档级哈希的归一化规则保持一致口径**):

1. 换行统一 `LF`;
2. 每行去尾随空白;
3. 去 Block 首尾空行;
4. **Heading 特例(规范化)**:哈希输入为规范形式 `#{level} {inline_text}`——使 setext↔ATX 风格切换不破坏标题的精确匹配(标题是 heading_path 的支点,稳定性优先);
5. 其余 Block 不做语义规范化(列表 `-`↔`*` 换标记等属源变更,交给相似度匹配兜住——预期得分 >T_high,静默随迁,见 §9-5);
6. **不做 Unicode NFC 归一化**(v0.1 决定;CJK 场景若出现问题在 v0.2 重议,见 §12)。

推论:纯尾随空白/换行风格的编辑不扰动任何锚点(精确匹配,c=1.0)。

### 3.4 Block 属性(迁移匹配的输入)

```rust
pub struct Block {
    pub block_id: BlockId,            // ULID,稳定不透明
    pub kind: BlockKind,
    pub span: SourceSpan,             // 字节区间 + 行区间(源文件坐标,含 frontmatter 偏移)
    pub content_hash: Blake3Hash,     // §3.3
    pub heading_path: Vec<String>,    // 由根到近的各级标题 inline 文本(规范化后)
    pub ordinal: u32,                 // 文档内顺序号(0-based)
    pub char_len: u32,                // 归一化后字符数(短块守卫用,§5.3)
}
```

---

## 4. 存储模型(sidecar,不写用户文件)

```
block_identity(block_id PK, doc_id, created_version_id, status ∈ live|degraded|retired)
block_instance(version_id, ordinal, block_id FK, kind, span, content_hash, heading_path JSON, char_len)
  -- 每个文档版本一套实例行;version_id 来自 F1 的 document_version(REQ-1.NEW-2)
migration_log(id, doc_id, from_version_id, to_version_id, block_id, outcome, confidence,
              detail JSON, created_at)   -- §8 事件日志,审计与 AC-3b-2 的数据面
```

版本快照的保留/淘汰由 F1 策略管理;**被锚定(评论/订阅/溯源/警示)引用的版本不淘汰**(REQ-1.NEW-2 anchored 语义,已由主 PRD REQ-4.8 与 F8 扩展)。block_instance 随其版本同生命周期。

---

## 5. 迁移算法

### 5.1 触发与执行位置(一次计算四用途)

F1 防抖合并处理产生新 `document_version` → 同一处理事务尾部**同步**执行迁移(旧树从 `block_instance` 读取,不重解析旧版):

```
parse(new_content) → new_tree(无 ID)
migrate(old_tree, new_tree) → MigrationResult
持久化 new block_instance(继承/新分配 block_id)+ migration_log
→ 事件总线发布 MigrationResult,四消费方各自消费(§7)
```

全链路计入 FS 呈现预算(<10s);本引擎自身预算见 §10。

### 5.2 匹配管线(三步)

**Step 1 · 精确对齐**:对新旧两个 `content_hash` 序列跑序列 diff(`similar`,算法选择属实现自由),得到 LCS 对齐——未变动与整体平移的 Block 直接继承 ID,`c = 1.0`。重复内容块(同哈希 ×N)由序列对齐天然按相对顺序配对,不歧义。

**Step 2 · 移动检测**:Step 1 的"删除侧"与"新增侧"中哈希相同的条目按出现顺序配对 → `Moved`(内容未变、位置变),继承 ID,`c = 1.0`(内容层面无变化,但进入 ChangeSet.moved,见 §7.1)。

**Step 3 · 相似度匹配**(仅对剩余未匹配的旧块 × 新块):

评分:

```
c = k_kind × (w_c·s_content + w_p·s_path + w_o·s_pos)

s_content:相似度比率(similar TextDiff ratio)。散文类词级;CodeBlock 行级
s_path:   heading_path 相似(全等=1;仅叶级不同=按共同前缀比例;完全不同=0)
s_pos:    1 − |old_ordinal/old_total − new_ordinal/new_total|
k_kind:   同类=1.0;散文类互转(Paragraph↔List↔Blockquote)=0.9;
          涉及 CodeBlock 或 Table 的跨类=0(硬排除——代码/表格不与散文互认)
初始权重: w_c=0.7, w_p=0.2, w_o=0.1(标定项)
短块守卫: char_len < 40 时 s_content 不可靠,权重切换为 w_c=0.3, w_p=0.4, w_o=0.3(标定项)
```

指派:候选对按 `(score desc, old_ordinal asc, new_ordinal asc)` 排序**贪心指派**,每块至多配一次——排序规则即决定性保证(同分不随机)。

**拆分/合并**:

- 拆分(旧 A → 新 B1..Bn):A 只配得分最高的 Bi(F3-E3:一条评论只锚一个 Block),其余片段为 `Added`;
- 合并(旧 A1、A2 → 新 B):得分最高者继承为 B 的 block_id,其余旧块产出 `Merged{into}`——其锚点**重指向幸存块**,消费侧按弱提示档处理(仍有有效落点,0 丢失且不在树中造出重复 ID)。

未被任何步骤匹配的旧块 → `Degraded{reason: Deleted}`;新块 → `Added`(分配新 ULID)。

### 5.3 置信度汇总

| 来源 | c |
|---|---|
| Step 1/2 精确/移动 | 1.0 |
| Step 3 匹配 | 评分值 |
| Merged | 其评分值,且**封顶于 T_high 之下**(强制至少弱提示档——合并必然丢失了部分原文语境) |
| Deleted / 未匹配 | — (直接降级) |

---

## 6. 置信度三档行为契约(冻结;数值待标定)

继承子 PRD-F3 §6,数值为 v0.1 初始值:

| 档位 | 条件 | 引擎产出 | 消费方义务(以 F3 为例) |
|---|---|---|---|
| 静默随迁 | `c ≥ T_high`(初始 **0.85**) | `Migrated{tier: Silent}` | 锚点更新,无 UI 提示 |
| 弱提示随迁 | `T_low ≤ c < T_high`(T_low 初始 **0.55**) | `Migrated{tier: WeakHint}` | 锚点更新 + "原文可能已变化"弱提示 |
| 显式降级 | `c < T_low` 或 Deleted | `Degraded{...}` | 降级为文档级 + 警示条 + quote 快照 + 手动重锚入口;**绝不静默丢失** |

**非对称原则(标定时的目标函数)**:静默错迁的代价 ≫ 多余降级的代价。标定约束:标注集上**静默错迁 = 0**(硬约束),在此前提下最小化降级率;T_high 宁高勿低。

标定程序(M0 Track B,详见《调研_技术基建与开发Phase》M0 设计):≥50 对真实 agent 改写前后样本(覆盖 AC-3b-1 四类场景),人工标注金标映射;对 (T_high, T_low, w_*, 短块守卫参数) 网格扫描,按上述目标函数选工作点;保留 20% holdout 复验后写入 v0.2 定稿。

---

## 7. 输出契约与四消费方

### 7.1 输出类型(冻结)

```rust
pub struct MigrationResult {
    pub doc_id: DocId,
    pub from_version: VersionId,
    pub to_version: VersionId,
    pub entries: Vec<MigrationEntry>,   // 不变式:旧树每个 live block_id 恰好出现一次
    pub changes: ChangeSet,
}

pub enum MigrationEntry {
    Unchanged { block_id: BlockId },                                  // c=1.0,位置内容均未变
    Moved     { block_id: BlockId, old_ordinal: u32, new_ordinal: u32 },
    Migrated  { block_id: BlockId, confidence: f32, tier: Tier },     // 内容有变,身份保留
    Merged    { block_id: BlockId, into: BlockId, confidence: f32 },  // 锚点重指向 into
    Degraded  { block_id: BlockId, reason: DegradeReason,             // LowConfidence | Deleted
                last_seen: (VersionId, SourceSpan) },                 // 指向快照,供 quote/追溯
    Added     { block_id: BlockId },                                  // 新块,新 ULID
}

pub struct ChangeSet {                  // 面向"什么变了"的消费视角
    pub modified: Vec<BlockId>,         // 内容哈希变化且身份保留(Migrated)
    pub added:    Vec<BlockId>,
    pub removed:  Vec<BlockId>,         // Degraded(Deleted)
    pub moved:    Vec<BlockId>,         // 内容未变仅移位
    pub merged:   Vec<(BlockId, BlockId)>,
}
```

另一冻结入口:`diff_versions(doc_id, from, to) -> MigrationResult`——对任意两个已存版本按同一算法计算(树从 block_instance 读取)。**F2 的已读基线 diff 必须走此接口直接对比基线↔当前**,不得累加逐次 ChangeSet(A→B→A 的净零变更会被累加法误报)。

### 7.2 四消费方契约

| 消费方 | 消费内容 | 行为契约 |
|---|---|---|
| **F3 评论** | 按锚定 block_id 查 entries | 三档行为(§6);Merged 按弱提示档重锚到 `into`;Degraded 走显式降级(quote 快照来自评论创建时,不依赖本引擎);每次消费写 F3 侧 `anchor_migrated`/`anchor_degraded` 事件(AC-3b-2) |
| **F4 命中检测** | 被评(sent 态)Block ∈ `modified ∪ removed ∪ merged 参与方` | 命中 → needs-review(时间窗与本地编辑过滤属 F4 §7.3);confidence 随 entry 附带供复核界面展示 |
| **F2 变更条/速读区** | `ChangeSet`(事件流)+ `diff_versions(基线, 当前)`(展示) | modified/added/removed 出变更条;moved 不出变更条;任何非空 ChangeSet 触发速读区重生成调度(REQ-2.3.3);变更摘要文本由 F2 基于 diff_versions 生成 |
| **F8 订阅命中** | 订阅的 block_ids ∩ `modified ∪ removed ∪ merged 参与方`;文档级订阅 = ChangeSet 非空(moved 除外) | 命中 → 漂移警示(REQ-8.3);订阅锚点自身随迁,低置信 → `needs_reconfirm`(REQ-8.2.4);净零变更天然无事件(§5.1 无新版本或 ChangeSet 为空) |

---

## 8. 不变式与事件日志(冻结)

1. **完备性**:旧树中每个 live 状态的 block_id 在 `entries` 中恰好出现一次(六种 entry 之一)。这是"0 静默丢失"的机械保证——任何消费方丢锚点都可归因到自身,不可能归因到引擎漏报。
2. **决定性**:同一 (old_tree, new_tree, 参数版本) 输入,输出逐字节相同(排序化贪心 + 无随机源)。
3. **ID 不复用**:Degraded/retired 的 block_id 永不分配给新块。
4. **单真相源**:Block 边界只由 pin 过的 comrak 选项决定;`parse_options_version` 与 `algo_params_version` 随结果持久化,升级时可识别跨版本边界差异并触发受控重建。
5. **日志义务**:每次迁移运行写 `migration_log`(每个 entry 一行,含 outcome/confidence/detail);引擎级日志与消费方级日志(F3 的 anchor_* 事件等)两层齐备,支撑 AC-3b-2 的"逐条核对无遗漏"。

---

## 9. 边界情况

| # | 情况 | 处理 |
|---|---|---|
| 1 | 空文档 / 仅 frontmatter | 空树;全部旧块 Degraded(Deleted)(文档删除本身由 F1 archived 流程先行拦截) |
| 2 | 重复内容块(同哈希 ×N) | Step 1 序列对齐按相对顺序配对;不做内容之外的猜测 |
| 3 | 超大文档 | F1 已拦 >1MB;引擎不再设限 |
| 4 | 全文重写(相似度普遍低) | 大量 Degraded——正确行为(AC-3b-1 类 d 场景),宁降级不错迁 |
| 5 | 列表换标记 / 围栏字符变化等纯风格变更 | 精确匹配失败 → Step 3 高分(≈1)静默随迁;已知非精确路径,标定集应含此类样本 |
| 6 | setext ↔ ATX 标题风格切换 | §3.3-4 标题规范化,精确匹配,c=1.0 |
| 7 | 列表内部单项编辑 | 整列表为一个 Block → 表现为该 Block modified(粒度权衡:锚点数可控 > 项级精度;评论 quote 仍可精确到选区) |
| 8 | 代码块仅改语言标注 | 源变更 → Step 3;行级 ratio 高,静默随迁 |
| 9 | 短块(char_len < 40) | 短块守卫权重(§5.2);预期降级率偏高,接受(信息量不足时宁降级) |
| 10 | heading_path 因上级标题改名而整体变化 | s_path 受损但 s_content 主导;标题自身作为 Block 先行匹配,其迁移结果可在实现中反哺子块的 path 相似度(实现自由度,不冻结) |
| 11 | 文档编码/换行风格整体变化(CRLF→LF) | 归一化吸收,全部精确匹配 |
| 12 | 并发:迁移进行中文档再变更 | F1 防抖保证串行;若强制处理窗口(10s 上限)内又有新版本,逐版本顺序迁移,不跳版(身份链不断裂) |

---

## 10. 性能预算与缓存

- 单文档迁移(解析新版 + 三步匹配 + 落库):**P95 < 300ms**(500-Block 文档、M1 级硬件);blake3 与序列 diff 为线性,Step 3 仅作用于未匹配残差(典型 agent 编辑 <10% 块),O(u×v) 可控。
- 旧树零解析(读 block_instance);新树单次解析单次哈希。
- 全项目对账(F1 REQ-1.4.5)复用同一入口,仅对哈希变化的文档触发迁移。

---

## 11. 测试与验收映射

| 验收 | 本引擎的承接 |
|---|---|
| AC-3b-1(四类 ×20 例,≥90% 正确迁移或显式降级,静默丢失=0) | 金标语料 + 指标 harness(silent-wrong 率 / 正确迁移率 / 降级率 / 弱提示精确率);语料来自 M0 Track B 真实 agent diff |
| AC-3b-2(事件日志无遗漏) | §8-1 完备性不变式的属性测试 + migration_log 逐条核对脚本 |
| AC-1c(重命名后评论 100% 保留) | F1 身份层测试;本引擎保证同 doc_id 下版本链不断 |
| AC-8c(订阅迁移复用 AC-3b 测试集) | 同一 harness,消费方换 F8 |
| 属性测试 | 恒等(无变更→全 Unchanged)、纯重排(→全 Moved)、决定性(重复运行 byte-equal)、完备性(随机变异下 entries 覆盖全部旧 ID) |
| 标定工具 | 参数网格扫描 CLI(输入语料+标注,输出 silent-wrong/降级率曲面),M0 与后续回归共用 |

---

## 12. 开放项(不阻塞接口冻结)

| # | 项 | 去向 |
|---|---|---|
| 1 | T_high/T_low/权重/短块守卫数值 | M0 Track B 标定 → v0.2 定稿 |
| 2 | similar 算法细选(Myers vs patience;词级 ratio 的分词口径,CJK 混排) | Phase 3 spike,标定集上比较 |
| 3 | Unicode NFC 归一化 | v0.1 不做;CJK 样本若现问题 v0.2 重议(需全量重哈希迁移方案) |
| 4 | 全文 Lens 回归后的「受上下文影响的相邻块」判定(Q-F2-4) | P1,随全文 Lens 设计;本契约的 ChangeSet 已预留 modified 邻接信息可推导 |
| 5 | comrak 大版本升级的边界漂移应对 | `parse_options_version` 已预留;升级前用金标语料回归边界一致性 |

---

## 13. 变更记录

| 版本 | 日期 | 变更 |
|---|---|---|
| v0.1 | 2026-07-28 | 初稿:两级身份澄清、Block 模型与 comrak 选项 pin、归一化/哈希规则(含标题规范化)、三步迁移管线与拆分/合并语义、置信度三档契约(初始阈值)、输出类型与四消费方契约、完备性/决定性不变式、边界情况、性能预算、验收映射与标定程序 |
