# Pitfalls Research

**Domain:** 本地优先桌面知识工作台(Tauri v2 + Rust;Markdown Block 锚定、FS-watch 同步 agent 写入文件、流式 LLM 速读区、内嵌 MCP server、跨项目契约订阅)
**Researched:** 2026-07-28
**Confidence:** MEDIUM(多源交叉验证的条目标 HIGH;单一 web 来源标 LOW;官方 advisory/spec 为 HIGH)

> Phase 编号采用 PROJECT.md 口径:0=M0 概念验证、1=基建骨架、2=F1 导入同步、3=锚定引擎、4=F2′ 速读区、5=F3 评论、6=F4 回流闭环、7=F5/F7/F8、8=发布准备。
>
> 本文以项目自身风险清单(TD-01 §9、调研 §2.3)为基线,只收录**验证后确认真实**或**清单尚未覆盖**的坑;已被 TD-01 完整处理的边界(如 setext↔ATX、重复块对齐)不重复。

## Critical Pitfalls

### Pitfall 1: 双 parser 各自分块 → 锚点系统性漂移(护城河直接失效)

**What goes wrong:**
comrak 与前端(react-markdown / CodeMirror)对同一文档给出不同的 Block 边界或坐标,评论高亮渐进性错位;用户看到评论标在错误段落上,信任崩塌。

**Why it happens:**
不是"决定用哪个 parser"的决策错误(D 级决策已锁 comrak),而是**执行期偷懒**:前端为了渲染方便自己重新解析并计算偏移,或用 DOM 位置反推 Block;两套边界规则在 GFM 边缘(懒惰续行、HTML block、表格)必然分歧。

**How to avoid:**
- 前端一律消费后端下发的 `Block[]`(含 sourcepos span),渲染层只做"按 span 切片 + 渲染",**永远不做边界判定**。
- 在 IPC 契约上把这一点结构化:前端拿不到全文-only 接口,只能拿 Block 化接口,让偷懒在类型上不可能。
- CI 加"边界一致性"冒烟:随机文档,前端高亮区间 = 后端 span,逐字节比对。

**Warning signs:**
前端代码里出现第二个 markdown parser 的依赖;高亮偏移在含表格/HTML block 的文档上出现 ±1 行误差。

**Phase to address:** Phase 3(接口冻结时把"前端零解析"写进消费方契约)+ Phase 5(评论渲染实现审查)。

---

### Pitfall 2: "orphan 判定自身失败"——降级路径静默失效(Hypothesis 教训)

**What goes wrong:**
系统承诺"绝不静默丢失",但真正丢失的方式往往不是迁移算法错,而是**降级/orphan 通道自己有 bug**:锚定失败却没进入 Degraded 状态,评论直接从 UI 消失。Hypothesis 生产环境实测 27% 注释已 orphan、61% 处于风险中,且存在"anchor 失败却未报 orphan"的已确认缺陷类([product-backlog#954](https://github.com/hypothesis/product-backlog/issues/954))——最成熟的 web 注释系统也栽在这条路径上。

**Why it happens:**
测试精力集中在"迁移正确率"上,降级路径被当成 else 分支;消费方(F3/F8)丢弃了引擎产出的 entry 却无人察觉——引擎完备性 ≠ 端到端 0 丢失。

**How to avoid:**
- TD-01 §8-1 完备性不变式做**属性测试**(随机变异下 entries 覆盖全部旧 ID)——这只保证引擎层。
- 消费方层加对账脚本:`评论表中 live 锚点 ∪ 降级锚点` 必须与 `migration_log` 逐条对得上(AC-3b-2 已有此意,须落成自动化,不是人工核对)。
- UI 层验收:Degraded 评论必须可见(文档级列表 + quote 快照),写进 AC-3b 测试而非只测引擎输出。

**Warning signs:**
评论总数在迁移前后不守恒;migration_log 行数 ≠ 旧树 live block 数;用户报告"评论不见了"而日志无 anchor_degraded 事件。

**Phase to address:** Phase 3(不变式属性测试)+ Phase 5(消费方对账与 UI 降级可见性)+ Phase 8(发布前锚点专项复测)。

---

### Pitfall 3: 标定集用合成 diff 而非真实 agent diff → 阈值失真,静默错迁上线才暴露

**What goes wrong:**
T_high/T_low 在人工构造的"改写样本"上标定得很好,上线后真实 agent(Claude Code 常整文件 Write 重写、Cursor 局部 patch)的改写分布完全不同,静默错迁在生产出现——而这恰是硬约束 silent-wrong=0 要防的。

**Why it happens:**
攒真实语料麻烦;合成样本倾向"人类式小编辑",而 agent 改写呈双峰分布(要么几乎不动、要么全文重排+重述),Step 3 相似度评分在后者上行为完全不同。CJK 混排时 similar 词级 ratio 依赖空格分词,中文内容得分系统性偏低(TD-01 §12-2 已识别为 spike 项,不可跳过)。

**How to avoid:**
- M0 Track B 的 ≥50 对样本必须来自**真实 Claude Code / Cursor 会话 diff**,覆盖 AC-3b-1 四类场景,且含中英混排样本;holdout 20% 复验(TD-01 §6 已设计,此处强调"真实来源"是不可妥协项)。
- Phase 3 的标定 CLI 保留为回归工具,内测期持续把真实迁移样本回灌语料库。

**Warning signs:**
标定集里没有"全文重写后 >50% 块降级"的样本;中文段落的 s_content 分布与英文明显分层。

**Phase to address:** Phase 0(M0 语料采集)+ Phase 3(标定 harness 与 CJK 分词 spike)。

---

### Pitfall 4: 编辑器/agent 原子保存(temp+rename)被误判为"删除+新建"→ 文档身份断裂,评论全体 orphan

**What goes wrong:**
VS Code、vim、多数 agent 工具链保存文件 = 写 temp 文件 + rename 覆盖,一次逻辑保存产生 Create(temp)+Rename+Write 至少 3 个事件;macOS FSEvents **不关联 rename 的新旧两侧**,只在两个路径各给一个 ItemRenamed 标志。若 F1 把这串事件解读成"原文件删除",走 archived 流程,该文档全部评论/订阅降级——一次保存清空一份文档的所有锚点。

**Why it happens:**
FSEvents 会合并、乱序、丢失甚至多发事件([watchexec 文档](https://watchexec.github.io/docs/macos-fsevents.html));notify-debouncer-full 虽会为 rename 前的事件改写路径,但"temp 文件模式识别"与"删除延迟确认"是应用层责任,debouncer 不提供。

**How to avoid:**
- 删除事件**永不立即生效**:进入 grace window(如 2–5s),窗口内同目录出现内容哈希匹配的新文件 → 判为同一文档(REQ-1.NEW-1 的内容哈希+路径启发式正是为此,须明确覆盖 rename-over 模式)。
- 对账(5 分钟 + 项目打开时)以磁盘为准,**按内容哈希对账而非 mtime**——FSEvents 有"文件未关闭前不投递事件"的已知行为([notify#240](https://github.com/notify-rs/notify/issues/240)),mtime 会漏。
- 测试矩阵写死:VS Code 保存、vim `:w`(backupcopy 默认)、Claude Code Write tool、`git checkout`,四种写入模式各自跑通"评论 100% 保留"(AC-1c 扩展)。

**Warning signs:**
日志中 archived→re-import 成对出现;同一文档 Document ID 在一次保存后变化。

**Phase to address:** Phase 2(F1 事件语义与身份识别;这是 Phase 2 最重要的单条验收)。

---

### Pitfall 5: 自写盘回声环——写回/物化触发自己的 watcher,形成迁移与重生成风暴

**What goes wrong:**
app 写 frontmatter、物化 log.md、写 `.prismdocs/feedback|context` → watcher 收到事件 → 当成外部变更 → 新版本 + 迁移 + 速读区重生成(LLM 调用!)→ 若处理又触发写盘,形成循环;轻则版本链污染与 token 浪费,重则死循环。

**Why it happens:**
调研 §2.3-2 已识别"写前登记 hash"统一原语,坑在执行:三处写盘(应用内编辑、log.md、feedback/context)只要有一处绕过原语直接 `fs::write`,回声就回来了。且**按时间窗过滤不可靠**(防抖 2s 窗口 + 编辑器原子保存的延迟事件会穿透时间窗),必须按内容哈希识别。

**How to avoid:**
- prism-fs 暴露唯一写盘入口 `write_registered(path, bytes)`(登记归一化哈希 → 原子写 temp+rename+fsync → 事件到达时哈希匹配即吞掉);crate 内不导出裸写函数,clippy lint/代码评审禁止直接 `std::fs::write` 用户 repo 路径。
- 回声吞掉的事件也写 debug 日志(区别于静默),便于发现漏网。

**Warning signs:**
document_version 表出现内容哈希与上一版本相同的相邻版本;速读区在用户未改文档时重生成;token 消耗曲线出现无操作尖峰。

**Phase to address:** Phase 2(原语落地)+ Phase 6(feedback/context 写入强制走原语)。

---

### Pitfall 6: git 瞬态状态(checkout/rebase/stash)被当作真实编辑 → 降级风暴 + LLM 费用风暴

**What goes wrong:**
agent 和用户频繁 `git checkout`/rebase:一次分支切换 = 数十上百文档瞬间"全文重写"。后果三连:① 大量 Block 进入 Degraded(切回原分支后 **ID 不复用 + Degraded 无自动复活路径**,评论永久降级——TD-01 §9-12 只保证不跳版,未定义 A→B→A 的锚点恢复);② 每份变更文档触发速读区重生成 → 一次 checkout 烧掉几十次 LLM 调用;③ 版本快照表被瞬态状态灌满。

**Why it happens:**
FS watcher 无法区分"agent 改了文档"与"git 换了工作树";这是所有 watch-git-repo 工具(watchexec、各类 dev server)公认的难题,而本项目的代价被锚定引擎放大。

**How to avoid:**
- watch `.git/HEAD` 与 `.git/index`:检测到 git 操作时进入"批量模式"——延长防抖、暂停速读区自动重生成(改为标记 stale,用户进入文档时再生成或手动批量)。
- 为 A→B→A 定义**降级复活规则**:新版本内容哈希与某历史版本一致时,恢复该版本的锚点状态(等价于 diff_versions(基线,当前) 的净零路径,但须显式覆盖"已 Degraded 的锚点可复活"——目前 TD-01 未定义,建议作为 v0.2 议题,阻塞 Phase 3 收尾)。
- 速读区重生成设全局并发上限 + 单次事件批量超过 N 文档时改为需确认(复用 REQ-2.9 成本闸门)。

**Warning signs:**
demo 项目里 `git checkout main && git checkout -` 后评论侧栏出现降级警示;一次分支切换后 token 消耗跳变。

**Phase to address:** Phase 2(git 操作检测与批量模式)+ Phase 3(A→B→A 复活语义进契约)+ Phase 4(重生成成本闸门)。

---

### Pitfall 7: SQLite 多连接写/checkpoint 并发 → 损坏或 SQLITE_BUSY;r2d2 池"人人可写"是反模式

**What goes wrong:**
WAL 只提升读写并发,**写与写仍互斥**;r2d2 池里每个连接都能发起写时,迁移事务、FTS 索引、评论写入并发碰撞 → SQLITE_BUSY 级联;更糟的是 WAL-reset 类 bug 在"两个连接同时写/checkpoint"时可致 `database disk image is malformed`(SQLite 3.51.3 已修复该实例)。长驻读连接(如 UI 常开查询)阻塞 checkpoint,-wal 无限膨胀。

**Why it happens:**
"WAL + 连接池"被误读为"随便并发";桌面单进程让人低估了多线程连接的互踩。

**How to avoid:**
- **单写者架构**:一个专用写连接(或写任务串行队列),r2d2 池只发只读连接(`PRAGMA query_only=ON` 防呆)。
- 启动 PRAGMA 套餐:`journal_mode=WAL; synchronous=NORMAL; busy_timeout=5000; foreign_keys=ON`;退出前 `wal_checkpoint(TRUNCATE)`。
- rusqlite bundled SQLite 版本 pin ≥3.51.3;读连接用完即还,禁止跨事件循环长持有。
- 迁移落库(version + block_instance + migration_log)保持单事务(TD-01 §5.1 已要求)——crash 时磁盘权威 + 对账可重建,DB 不会出现半迁移状态。

**Warning signs:**
日志出现 SQLITE_BUSY;-wal 文件超过 DB 主文件大小;`cargo tree` 出现两个 rusqlite 版本(链接两份 SQLite 是另一个经典损坏源,Phase 1 退出标准已含此检查)。

**Phase to address:** Phase 1(连接架构与 PRAGMA 定死,后改成本极高)。

---

### Pitfall 8: FTS5 unicode61 对 CJK 静默零结果——中文搜索直接失效且无报错

**What goes wrong:**
unicode61 tokenizer 把连续中文当作**单个 token**(CJK 无词边界),`MATCH '锚定'` 在含"锚定引擎设计"的文档上返回 0 行——不报错、不降级,搜索功能对中文内容整体失效。本项目中文速读区、中文评论、中文卡片全是 FTS 目标,命中率为零 = "500 文档搜索 <300ms"的性能指标毫无意义。

**Why it happens:**
FTS5 默认配置对英文开箱即用,英文测试全绿,中文问题要到真实数据才暴露;而 **tokenizer 是建表参数,后改 = 全量重建索引**。

**How to avoid:**
- Phase 1 schema v1 就选定 CJK 方案:首选 `tokenize='trigram'`(内置、substring 匹配、无需扩展,代价是索引偏大)或 simple/jieba 类外部 tokenizer(需编译扩展);英文列与中文列可分表分 tokenizer。
- 验收用真实中文查询:速读区文本、中文评论各准备查询样例,命中率进 Phase 2 退出标准。

**Warning signs:**
FTS 测试用例全是英文;中文查询返回 0 行但 LIKE 能命中。

**Phase to address:** Phase 1(schema/tokenizer 决策)+ Phase 2(FTS 落地验收)。

---

### Pitfall 9: MCP loopback server 的 DNS rebinding / Host 校验缺失(rmcp 有真实 CVE)

**What goes wrong:**
恶意网页通过 DNS rebinding 向 127.0.0.1 上的 MCP server 发认证外请求,枚举并调用全部 tool——对本项目意味着读取文档内容、伪造 feedback 回执、拉取 Context Pack。**rmcp 自身在 <1.4.0 就有此漏洞**([GHSA-89vp-x53w-74fx](https://github.com/modelcontextprotocol/rust-sdk/security/advisories/GHSA-89vp-x53w-74fx):Streamable HTTP transport 未校验 Host header)。

**Why it happens:**
"只绑 127.0.0.1 就安全"是误解——rebinding 攻击正是从浏览器发起的 loopback 请求;bearer token 若通过可预测方式交付也会被绕过。

**How to avoid:**
- rmcp pin ≥1.4.0 并在升级时复查 advisory;axum 层显式校验 Host ∈ {127.0.0.1, localhost} + Origin allowlist(D-07 已含)+ bearer token 三层齐备,任何一层缺失都 403。
- MCP spec 明文要求 Origin 校验(HIGH,官方 spec)——把"无 Origin 的非浏览器请求放行、有 Origin 的必须命中 allowlist"逻辑写测试。
- 同一 axum 实例未来承载剪藏 WS 端点(P1)时,同等防护自动继承——中间件做在实例级而非路由级。

**Warning signs:**
集成测试没有"伪造 Origin 请求被拒"用例;rmcp 版本浮动(`^`)而非 pin。

**Phase to address:** Phase 6(MCP server 实现;安全用例进 AC)。

---

### Pitfall 10: Feedback Bundle 被 agent 忽略或半执行——闭环回收失败

**What goes wrong:**
结构化反馈文件写得再好,agent 可能:不读、读了不逐条执行、执行了不写回执。社区大量实证:指令文件过长时 LLM **整体忽略而非选择性过滤**;Claude Code 存在"读了 CLAUDE.md 仍违反其中规则"的已知问题。闭环回收失败 = 北极星指标(闭环数/周)直接归零。

**Why it happens:**
把 agent 当确定性消费方设计协议;Bundle 若是大 JSON 嵌套结构,agent 解析成本高、执行动机弱;回执协议("处理完写 receipt 文件")对 LLM 是额外负担,最容易被丢弃。

**How to avoid:**
- Bundle 格式偏向 agent 友好:紧凑 Markdown、每条评论一个明确的可执行指令 + 原文引用 + 期望动作,而非深嵌套 JSON;总长度设上限(过长自动分批)。
- **三信号回收(MCP 回执 / Receipt 文件 / FS 变更兜底)是本坑的正解,已在 F4 设计中**——执行期的坑是把 FS 兜底当"低优先级";实际上它是三信号中唯一不依赖 agent 配合的,必须与前两信号同等强度实现(AC-4c 无 MCP 也闭环)。
- SessionStart hook 注入 check-feedback 时输出必须极短(几行指针,不是 Bundle 全文)——hook 输出过长本身会挤占 agent context,降低执行率。
- M0 与 Phase 6 都用**真实 Claude Code 与 Cursor 会话**验证执行率,不用手工模拟(AC-4a 已要求双 agent,此处强调"执行率"要有量化观测:发出 N 条修改要求,统计被正确处理的比例)。

**Warning signs:**
内测中 needs-review 长期挂起、FS 兜底信号占比远高于回执信号(说明 agent 没在配合协议);Bundle 平均体积持续增长。

**Phase to address:** Phase 0(M0 手工跑通即观测执行率)+ Phase 6(协议与三信号等强度实现)。

---

### Pitfall 11: 用户 repo 处于 iCloud/Dropbox 同步下——自写文件被回滚、dataless 文件读挂

**What goes wrong:**
用户把项目放在 iCloud Drive / Dropbox / OneDrive 里(vibe coder 常态)。两类事故:① app 写入 frontmatter/`.prismdocs/` 后数秒内被同步服务用云端旧版**静默回滚**(Claude Code 自身有此已确认 issue:同步目录下编辑静默失败);② iCloud "优化存储"把文件 evict 成 dataless,读取阻塞触发下载或拿到空内容,watcher 收到的事件序列也被同步服务的影子文件污染。

**Why it happens:**
本地优先设计默认"磁盘写入即持久",而同步服务是另一个并发写者;测试环境没人把 repo 放 iCloud。

**How to avoid:**
- 所有自写盘走原子写(temp+rename+fsync)后**读回验证**,不一致即报错给用户(绝不静默)。
- 导入向导检测路径是否在已知同步目录(`~/Library/Mobile Documents`、`~/Dropbox`、`~/Library/CloudStorage/*`),给出明确警示与建议。
- sidecar 数据库放 Application Support(已定 D-13)天然避开;此坑只影响用户 repo 内的写入——也是"少写用户 repo"原则的又一论据。

**Warning signs:**
frontmatter 写回后 checksum 校验失败;watcher 收到大量 `.icloud` 占位或冲突副本文件事件。

**Phase to address:** Phase 2(导入检测 + 写回验证)+ Phase 6(feedback 写入同样验证)。

---

### Pitfall 12: LLM 流中断的部分输出被当作完整结果缓存

**What goes wrong:**
速读区流式生成到一半连接断开/进程被杀,半截摘要落了缓存(缓存键匹配 → 永不重生成),用户长期看到残缺速读区;或文档在生成期间又变了,过期的生成结果覆盖新状态。

**Why it happens:**
SSE 在 HTTP 200 之后出错只能以流内事件表达,客户端若不区分"流正常终结(finish_reason)"与"连接断开",部分输出与完整输出不可分辨——这是流式 LLM 的公认结构性问题(无行业标准解法,需应用层自处理)。

**How to avoid:**
- 缓存写入唯一条件 = 收到正常终结信号(finish_reason/`[DONE]`)且结构校验通过(速读区三段结构完整、❓ 项均带原文摘录);中断 → 丢弃 + 标记重试。
- 生成任务与文档版本绑定:新 document_version 到达即 cancel 在途生成(CancellationToken 贯穿 reqwest 流),完成时校验版本仍是最新才落缓存。
- 重试对 429/5xx 指数退避,且**输入 token 已计费**——重试计数进成本统计,避免账单静默翻倍。

**Warning signs:**
缓存表中出现结构不完整的速读区;同一文档缓存命中却显示旧版内容。

**Phase to address:** Phase 4(prism-llm 通道与缓存语义)。

---

### Pitfall 13: frontmatter 写回走 parse→serialize 往返 → 用户 repo git diff 污染

**What goes wrong:**
YAML 重新序列化改变引号风格/键序/缩进,用户 `git diff` 出现非本意变更,"不污染原则"破产;agent 下次读取也会把这些噪音 diff 带进上下文。

**Why it happens:**
gray_matter 解析无损,但任何"解析后再写出"的实现天然丢失字节级格式;这是调研 §2.2 已标的风险,坑在实现者图省事。

**How to avoid:**
写回策略写死:定位原 frontmatter 字节区间,**只替换正文字节,frontmatter 块原样保留**;若需改 frontmatter 字段,做键级字节替换而非整块重序列化。AC-1g-2(round-trip `git diff` 0 变化)必须覆盖:CRLF 文件、无尾换行文件、含非常规 YAML(锚点、多行字符串)的文件。

**Warning signs:**
测试语料全是"干净"YAML;写回实现里出现 `serde_yaml::to_string`。

**Phase to address:** Phase 2(实现即验收)。

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| r2d2 池所有连接可写,靠 busy_timeout 硬扛 | 少写一层写队列 | SQLITE_BUSY 级联、潜在 WAL 损坏,后期改架构牵动全部 crate | Never(Phase 1 就定单写者) |
| FTS5 先用默认 unicode61,"以后再换 tokenizer" | schema 简单 | tokenizer 是建表参数,换 = 全量重建索引 + 数据迁移 | Never(中文是核心场景) |
| 前端自己解析 markdown 算高亮偏移 | 渲染实现快 | 双真相源锚点漂移,护城河失效 | Never |
| 时间窗过滤自写盘回声(不做 hash 登记) | 实现 10 分钟 | 原子保存延迟事件穿透时间窗,回声环上线才暴露 | Never |
| 迁移日志只记失败不记全量 entry | 省存储 | AC-3b-2"逐条核对"失去数据面,静默丢失不可归因 | Never |
| 速读区重生成无并发/成本闸门,"先跑通再说" | Phase 4 提前完成 | git checkout 一次烧几十次调用,内测期用户账单惊吓 → 卸载 | 仅 Phase 4 开发分支,进 Phase 6 前必须补 |
| 签名公证留到 Phase 8 首次尝试 | 前期无 CI 负担 | CLI helper(externalBin)签名、hardened runtime、entitlements 首次必爆,阻塞发布 1–2 周 | 可延后,但 Phase 6(CLI helper 成形)后须做一次端到端公证冒烟 |
| 版本快照无淘汰策略先全留 | 逻辑简单 | 500 文档 × agent 高频重写 → sidecar 膨胀数 GB | MVP 可接受,Phase 8 压测时补(REQ-1.NEW-2 anchored 语义已预留) |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Claude Code(MCP 客户端) | 假设 token 交付只有一种方式 | Claude Code 用 headersHelper 从 Keychain 读 token(已定);SessionStart hook 输出保持极短 |
| Cursor(MCP 客户端) | 直接把 bearer token 明文写进项目内 `.cursor/mcp.json` → token 进 git 历史 | Cursor 无 headersHelper 等价物:token 走用户级(非项目级)配置或环境变量插值;安装向导按 agent 分路径;`.prismdocs/` 与任何含 token 文件强制 gitignore 校验 |
| rmcp / MCP spec | 只靠绑定 127.0.0.1 防护 | Host 校验 + Origin allowlist + bearer token 三层;rmcp pin ≥1.4.0(GHSA-89vp-x53w-74fx) |
| notify (FSEvents) | 信任事件序列完整、把 rename 当成对出现 | 事件只当"脏标记",真相靠内容哈希对账;删除延迟确认;测试覆盖 VS Code/vim/agent Write/git 四种写入模式 |
| OpenAI 兼容端点(用户自配 base_url) | 假设所有端点 SSE 行为一致(finish_reason、429 头、error 事件格式) | 用户可配任意兼容端点(OpenRouter、本地代理),流解析对缺失 finish_reason/非标 error 事件做防御;429 无 Retry-After 时用指数退避 |
| keyring / macOS Keychain | dev 期每次 rebuild 触发 Keychain 授权弹窗(签名身份变化),团队误以为 bug 而绕开 Keychain | dev build 用固定 ad-hoc 签名身份或环境变量 fallback;release 签名稳定后弹窗仅首次;绝不因此回退到明文存储 |
| Tauri updater | 混淆公证与 updater 签名 | 两者独立:notarization 不影响 minisign updater 签名;私钥丢失 = 永久无法推更新,密钥备份进发布 checklist |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| MigrationResult/Block 树全量走默认 JSON IPC 广播 | 大文档保存后 UI 卡顿数百 ms | 事件只发轻量通知(doc_id + ChangeSet 摘要),前端按需拉取;大 payload 用 Tauri raw request/Channel | 单文档 >200 Block 或批量变更 >20 文档 |
| 500-Block 文档整树渲染进 DOM | WKWebView 内存爬升、滚动掉帧 | Block 列表虚拟滚动;diff 视图懒渲染 | 文档 >1000 Block 或长会话数小时后 |
| 每个 FS 事件独立触发完整管线(解析+迁移+FTS+速读区) | git 操作后 CPU 打满、事件积压 | 批量模式(Pitfall 6)+ 管线按文档去重合并 | 一次 git checkout 触及 >50 文档 |
| 长驻只读连接跨事件循环持有 | -wal 文件持续膨胀 | 读连接即用即还(池化使重开廉价) | 运行数天的长会话 |
| Step 3 相似度矩阵在全文重写时 O(u×v) 爆炸 | 大文档全文重写迁移超 P95 300ms | TD-01 已限定 Step 3 仅作用残差;加残差规模上限,超限直接批量降级(宁降级不超时) | 500-Block 文档 90% 块未匹配时 |
| 速读区对每次 ChangeSet 立即重生成 | token 消耗与文档保存频率线性挂钩(agent 保存极频繁) | 防抖已复用 F1 节奏;再加"文档静默 N 秒后才生成"+ 可见性优先(只即时生成用户打开的文档) | agent 连续工作会话(每分钟多次保存) |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| rmcp 版本浮动、Host/Origin 校验依赖框架默认 | DNS rebinding → 本地 tool 全暴露(读文档、伪造回执) | pin ≥1.4.0;三层校验显式实现 + 恶意 Origin 测试用例 |
| token/敏感产物落入用户 repo(Cursor 配置、Bundle 内嵌密钥) | token 随 git push 泄漏 | token 只存 Keychain;写入用户 repo 的所有产物过"无密钥"断言;gitignore 校验 |
| 文档内容经 prompt 注入操纵速读区/❓ 清单 | agent 写的恶意/被污染文档诱导用户批准错误决策 | 速读区 prompt 加防注入约束;❓ 项强制附原文摘录(已设计)正是人工核验防线,不可为省 token 砍掉 |
| 埋点导出包含文档内容片段 | 违反"文档不经过我方服务器"承诺 | 埋点 schema 白名单(仅计数/时长/状态),导出前人工可读审查 |
| MCP tool 返回值包含全文档而无授权分级 | 任何能连上 MCP 的进程可拉全部知识库 | tool 粒度最小化;Context Pack 拉取记审计日志 |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| 降级警示逐条弹出(F8 上游高频变更 × 下游多订阅) | 警示疲劳 → 用户关掉订阅功能,第二护城河失效 | 按上游变更批次聚合为一条摘要警示;Inbox 分组;"一键下游核对"已是正解,入口做在聚合层 |
| token 消耗事后才可见 | 账单惊吓 → 信任崩塌(付费用户红线) | 生成前显示预估(REQ-2.9 已设计);批量操作(git 后重生成)必须前置确认 |
| Degraded 评论藏在折叠区 | 用户以为评论丢了(即使系统没丢) | 降级评论保持一等公民可见性:文档级列表 + quote 快照 + 醒目重锚入口 |
| MCP 安装向导假设 agent 已在 PATH/已配置 | 首次闭环失败在安装环节,激活漏斗断裂 | 向导逐 agent 检测 + 复制粘贴级指引 + 连通性自检按钮;7 日首闭环 ≥40% 的激活指标直接依赖此处 |
| 速读区静默过期(文档变了、缓存没更新标记) | 用户基于旧摘要做决策 | 速读区头部显式 stale 标记 + 生成时间;stale 时❓清单置灰 |

## "Looks Done But Isn't" Checklist

- [ ] **锚定引擎:** 引擎完备性测试全绿 ≠ 端到端 0 丢失——验证消费方对账脚本(评论表 ↔ migration_log)与 Degraded UI 可见性
- [ ] **FS 同步:** 手动改文件跑通 ≠ 同步可靠——验证 VS Code 原子保存、vim、Claude Code Write、git checkout 四种模式 + 断网/睡眠唤醒后对账
- [ ] **FTS 搜索:** 英文查询命中 ≠ 搜索可用——验证中文查询命中(速读区、中文评论、卡片)
- [ ] **速读区缓存:** 重启不重算 ≠ 缓存正确——验证中断流不落缓存、版本竞争不落旧缓存、prompt 版本升级使旧缓存失效
- [ ] **MCP server:** Claude Code 连通 ≠ 双 agent 可用——Cursor 的 token 交付路径独立验证;伪造 Origin 请求被 403
- [ ] **回流闭环:** 演示会话跑通 ≠ 闭环可靠——统计真实会话中 Bundle 逐条执行率与三信号各自触发占比;FS 兜底单独断网测试
- [ ] **frontmatter 写回:** 干净 YAML 往返一致 ≠ round-trip 安全——CRLF、无尾换行、YAML 锚点/多行字符串样本各过一遍 `git diff` 0 变化
- [ ] **签名发布:** 本机能跑 ≠ 可分发——干净机器(无 dev 证书)下载 DMG:Gatekeeper 放行、CLI helper 可执行、updater 完整走一轮
- [ ] **sidecar 数据:** 功能正常 ≠ 数据安全——kill -9 后重启数据完好;提供导出;DB 损坏时的恢复路径(至少 OKF 物化导出可再导入)

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| 静默丢锚上线后发现 | HIGH | migration_log 全量回放定位丢失点;quote 快照兜底人工重锚;发布门槛(AC-3b)存在的意义就是不让走到这一步 |
| 文档身份断裂(评论 orphan 成批) | MEDIUM | 按内容哈希在历史版本中重认亲(document_version 保留使之可行);提供批量重绑定工具 |
| SQLite 损坏 | MEDIUM | `.recover` + 从磁盘 md 全量重导入;评论/卡片依赖 sidecar 备份快照(Phase 8 交付定期快照) |
| 阈值失真致降级率过高(非静默错) | LOW | 阈值是非冻结参数:回灌真实语料重标定,发参数更新即可(TD-01 分层冻结的红利) |
| comrak 升级边界漂移 | MEDIUM | parse_options_version 已预留;金标语料回归 → 受控全量重建(TD-01 §12-5) |
| updater 私钥丢失 | HIGH | 无法热修复,只能引导用户手动重装;预防:私钥离线备份进发布 checklist |
| 回声环上线后发现 | LOW | 哈希登记原语补漏点;版本链污染用"相邻同哈希版本合并"清理脚本 |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| 1 双 parser 漂移 | Phase 3 冻结 + Phase 5 实现 | 前端零 parser 依赖审查;高亮 span 逐字节一致冒烟 |
| 2 orphan 判定自身失败 | Phase 3 + Phase 5 | 完备性属性测试;消费方对账脚本;Degraded UI 用例 |
| 3 标定集失真 | Phase 0 + Phase 3 | 语料来源审查(真实 agent diff);holdout 复验;CJK 样本占比 |
| 4 原子保存误判删除 | Phase 2 | 四种写入模式 × 评论 100% 保留(AC-1c 扩展) |
| 5 自写盘回声环 | Phase 2(原语)+ Phase 6(强制) | 无操作期 version 表零增长;回声吞噬 debug 日志抽查 |
| 6 git 瞬态风暴 | Phase 2 + Phase 3(复活语义)+ Phase 4(闸门) | checkout 往返后评论零降级 + token 零消耗测试 |
| 7 SQLite 并发写损坏 | Phase 1 | 单写者架构评审;并发压测无 BUSY;`cargo tree -d` 清洁 |
| 8 FTS5 CJK 零结果 | Phase 1(tokenizer)+ Phase 2 | 中文查询命中率用例 |
| 9 MCP DNS rebinding | Phase 6 | rmcp 版本 pin 审查;恶意 Origin/Host 403 用例 |
| 10 Bundle 被 agent 忽略 | Phase 0 + Phase 6 | 双 agent 真实会话执行率统计;三信号占比监控 |
| 11 同步服务回滚写入 | Phase 2 + Phase 6 | 写后读回验证;同步目录检测警示用例 |
| 12 部分流当完整缓存 | Phase 4 | 中断注入测试;版本竞争测试;结构校验 |
| 13 frontmatter 往返污染 | Phase 2 | AC-1g-2 扩展样本(CRLF/无尾换行/YAML 边缘) |
| 签名公证末期爆雷 | Phase 6 后冒烟 + Phase 8 | 干净机器端到端安装验证 |
| F8 警示疲劳 | Phase 7 | 聚合警示交互走查 |

## Sources

**锚定/注释系统(HIGH——一手系统与实证研究):**
- [Hypothesis: Fuzzy Anchoring](https://web.hypothes.is/blog/fuzzy-anchoring/)(diff-match-patch 多选择器方案)
- [Quantifying Orphaned Annotations in Hypothes.is](https://www.researchgate.net/publication/283646490_Quantifying_Orphaned_Annotations_in_Hypothesis)(27% orphaned / 61% at-risk 实证)
- [hypothesis/product-backlog#954](https://github.com/hypothesis/product-backlog/issues/954)(anchor 失败却未报 orphan——静默丢失路径实锤)

**FS watcher(HIGH——官方文档与 issue 交叉验证):**
- [Watchexec: Mac FSEvents limitations](https://watchexec.github.io/docs/macos-fsevents.html)(批量/乱序/丢失/多发)
- [notify-rs#240](https://github.com/notify-rs/notify/issues/240)(文件关闭前事件不投递)
- [notify-rs#371](https://github.com/notify-rs/notify/pull/371)、[notify_debouncer_full docs](https://docs.rs/notify-debouncer-full)(rename 处理边界)

**Tauri(MEDIUM):**
- [Tauri IPC 性能讨论 #11915](https://github.com/tauri-apps/tauri/discussions/11915)(raw request 与平台差异)
- [Tauri#11992](https://github.com/tauri-apps/tauri/issues/11992)(externalBin 公证失败——与本项目 CLI helper 直接相关)
- [Tauri v2 macOS Code Signing](https://v2.tauri.app/distribute/sign/macos/)、[updater 签名与公证独立性讨论 #7703](https://github.com/tauri-apps/tauri/discussions/7703)

**SQLite(HIGH——官方文档/论坛):**
- [SQLite WAL 官方文档](https://www.sqlite.org/wal.html)、[WAL checkpoint 损坏机理论坛帖](https://sqlite.org/forum/info/47107ab818977549?t=h)(多连接并发写/checkpoint 损坏,3.51.3 修复)
- [FTS5 unicode61 CJK 讨论(sqlite-users)](https://sqlite-users.sqlite.narkive.com/N5MOmskp/why-sqlite-fts5-unicode61-tokenizer-does-not-support-cjk-chinese-japanese-krean)、[FTS5 中文 bigram 实践](https://dev.to/foxck016077/sqlite-fts5-wont-tokenize-chinese-heres-the-7-line-bigram-fix-that-did-4fcc)

**MCP 安全(HIGH——官方 advisory 与 spec):**
- [GHSA-89vp-x53w-74fx: rmcp Streamable HTTP DNS rebinding](https://github.com/modelcontextprotocol/rust-sdk/security/advisories/GHSA-89vp-x53w-74fx)
- [MCP Transports spec(Origin 校验 MUST)](https://modelcontextprotocol.io/specification/2025-03-26/basic/transports)
- [rust-sdk#822: Origin 校验 defense-in-depth](https://github.com/modelcontextprotocol/rust-sdk/issues/822)

**LLM 流式(MEDIUM):**
- [Streaming LLM responses without breaking your backend](https://www.firsttoken.dev/p/streaming-llm-responses-without-breaking-your-backend)(中断/部分输出/非幂等)
- [Redis: Streaming LLM Responses](https://redis.io/blog/streaming-llm-responses/)

**Agent 指令执行(MEDIUM——多 issue 交叉):**
- [claude-code#27032](https://github.com/anthropics/claude-code/issues/27032)、[claude-code#7777](https://github.com/anthropics/claude-code/issues/7777)(读了指令文件仍违反)
- [Agent instruction 长度与忽略行为分析](https://www.wordman.dev/blog/agent-instructions)

**本地优先/同步冲突(MEDIUM):**
- [claude-code#52493](https://github.com/anthropics/claude-code/issues/52493)(同步目录下编辑被静默回滚——静默数据丢失实锤)
- [Obsidian sync 冲突机制](https://deepwiki.com/obsidianmd/obsidian-help/2.3-synchronization-and-conflict-resolution)

**项目内部基线:**
- TD-01 §9 边界情况、§12 开放项;《调研_技术基建与开发Phase》§1.2 问题清单、§2.3 三个工程决策

---
*Pitfalls research for: PrismDocs(本地优先 Markdown 知识工作台,Block 锚定 + agent 闭环)*
*Researched: 2026-07-28*
