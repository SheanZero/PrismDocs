---
gsd_state_version: 1.0
milestone: v0.2
milestone_name: milestone
current_phase: 01
current_phase_name: foundation-skeleton
status: executing
stopped_at: Completed 01-24-PLAN.md
last_updated: "2026-07-29T13:32:20.485Z"
last_activity: 2026-07-29
last_activity_desc: 01-16 完成：prism-mcp 门禁层三条缺口关闭（WR-05/06/07）
progress:
  total_phases: 1
  completed_phases: 0
  total_plans: 28
  completed_plans: 22
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-07-28)

**Core value:** 「评论 → AI 修改 → 复核通过」的闭环：10 分钟看懂 AI 英文文档、批注两句让它接着干、评论在 AI 大规模重写下 0 静默丢失。北极星 = 每周闭环数。
**Current focus:** Phase 01 — foundation-skeleton

## Current Position

Phase: 01 (foundation-skeleton) — EXECUTING
Plan: 22 of 28（01-01..01-16 已执行；下一份 01-17）
Status: Ready to execute
Last activity: 2026-07-29 — 01-16 完成：prism-mcp 门禁层三条缺口关闭（WR-05/06/07）

Progress: [████████░░] 79%

**为什么 Phase 1 尚未 Complete：** ~~`01-VERIFICATION.md` 记 3/4 成功标准通过。唯一 blocker 是成功标准 4
的自动化证据链——`scripts/check-secrets.sh` 的关键词分支要求值带引号，未加引号的赋值（.env / YAML /
TOML / CI `env:`）整类不可见~~ — **01-14 已关闭**：关键词分支的值改为「引号串{8,} 或 裸值{16,}」
两者取一，补 `github_pat_` / `xox[baprs]-` / `AIza` 三条前缀；`scan` 固定 cwd 到仓库根并加扫描面
下限断言（WR-09）。四条非恒真反证实跑（见 01-14-SUMMARY.md）。**已知残留**：SC-4 取样表第 5 行
`password = hunter2hunter2` 值仅 14 字符，落在裸值下界之下——已写进源码注释，不下调阈值。
其余 14 份处理两轮评审累积的 36 条 warning/info（用户定的范围：全部清干净再进 Phase 2）。

## Performance Metrics

**Velocity:**

- Total plans completed: 0
- Average duration: -
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

**Recent Trend:**

- Last 5 plans: -
- Trend: -

*Updated after each plan completion*
**Per-Plan Metrics:**

| Plan | Duration | Tasks | Files |
|------|----------|-------|-------|
| Phase 01 P01 | 39min | 3 tasks | 53 files |
| Phase 01 P02 | 8min | 2 tasks | 8 files |
| Phase 01 P03 | 68min | 2 tasks | 8 files |
| Phase 01 P04 | 7min | 2 tasks | 11 files |
| Phase 01 P05 | 38min | 2 tasks | 7 files |
| Phase 01 P06 | 26min | 2 tasks | 10 files |
| Phase 01 P07 | 31min | 2 tasks | 9 files |
| Phase 01 P08 | 11min | 2 tasks | 9 files |
| Phase 01 P09 | 81min | 3 tasks | 19 files |
| Phase 01 P10 | 10min | 2 tasks | 4 files |
| Phase 01 P11 | 15min | 2 tasks | 5 files |
| Phase 01 P13 | 14min | 3 tasks | 7 files |
| Phase 01 P12 | 6min | 2 tasks | 6 files |
| Phase 01 P14 | 25min | 3 tasks | 2 files |
| Phase 01 P15 | ~20min | 2 tasks | 2 files |
| Phase 01 P16 | ~35min | 3 tasks | 3 files |
| Phase 01 P18 | 15min | 3 tasks | 2 files |
| Phase 01 P20 | ~20min | 2 tasks | 2 files |
| Phase 01 P21 | ~40min | 3 tasks | 3 files |
| Phase 01 P22 | ~25min | 2 tasks | 4 files |
| Phase 01 P23 | 9min | 3 tasks | 5 files |
| Phase 01 P24 | ~25min | 3 tasks | 3 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Init]: Roadmap 采用调研文档 Phase 划分 + 修正 A1/A2/A3；关键路径 1→2→3→5→6，Phase 4 ∥ 5
- [Init]: Phase 1 承载五项不可逆决策（单写者 SQLite + 读池、FTS5 CJK tokenizer、keyring-core、prism-mcp trait 反转、notify-then-fetch）
- [Init]: INFRA-04/05 为跨切预算——自 Phase 1 起执行，Phase 8 验证级验收
- [Init]: comrak 唯一锚定真相源；MigrationResult/ChangeSet 接口在 Phase 3 冻结（TD-01 §7）
- [Phase 1]: workspace members 用 crates/* 通配 + src-tauri：新增 crate 无需改动根 Cargo.toml
- [Phase 1]: prism-mcp 的 protocol_version() 用 static 承载 rmcp ProtocolVersion::LATEST（内含 Cow，const 提升不适用）
- [Phase 1]: 暂无自然用途的依赖（prism-llm serde、prism-anchor similar/ulid、prism-mcp axum/tokio）以依赖可用性单测引用，不提前发明后续 phase 的公开 API
- [Phase 1]: INFRA-01 不在 plan 01-01 勾选：该需求横跨本 phase 的 7 个 plan，事件总线与 Channel 有序流在 01-04/01-08/01-09
- [Phase ?]: 依赖方向断言用 herestring 而非管道喂 grep：pipefail + grep -q 早退会因 SIGPIPE 让四条断言全部静默恒绿
- [Phase ?]: 覆盖率 Phase 1 只测量不设阈值（engine 85.48% / 前端 10%），Phase 2 按排除已登记人工与 ignored 路径后 >=80% 开硬闸门
- [Phase ?]: schema v1 定案方案 A：external-content FTS5 + rowid_pk 显式 INTEGER PRIMARY KEY + 三同步触发器 + STRICT 表，索引粒度保持默认全粒度（不声明该选项，降粒度会废掉 4 字中文 MATCH）
- [Phase ?]: 只读池用 SQLITE_OPEN_READ_WRITE + query_only=ON 而非 READ_ONLY flags：只读连接在崩溃后 -shm 缺失时无法重建它
- [Phase ?]: 「颠倒迁移与建池顺序」的行为反证实测不成立（六个并发测试仍全绿），改用 open.rs 内的源码顺序断言作为常驻哨兵
- [Phase ?]: prism-types 的依赖上限就是 serde + thiserror 两项：它是 prism-mcp 与 prism-engine 的共同汇点，任何新增依赖会同时压到两侧（D-09）
- [Phase ?]: service trait 一律同步（非 async）：底层 rusqlite 本就阻塞，同步 trait 天然 object-safe，consumer 用 spawn_blocking 调用
- [Phase ?]: 跨边界转发第三方错误时只保留已核实安全的 Display 文本：keyring_core::Error 的 derive Debug 会打印原始密钥字节，而 unwrap()/tracing 的 ?err 走的正是 Debug
- [Phase ?]: 持有密钥的类型手写 Debug 输出占位串，并刻意不实现 Display——缺席让 format!("{key}") 成为编译错误而非运行期泄漏
- [Phase ?]: 钥匙串 service/account 命名（PrismDocs / llm_api_key / mcp_bearer_token）是跨二进制契约，固化于 docs/keychain-naming.md；prismdocs-helper 因 D-10 必须自带字面量副本
- [Phase ?]: FTS 表在 SQL 中不能起别名：MATCH 左操作数必须是 fts5 表名，JOIN 打在 d.rowid_pk = documents_fts.rowid 上
- [Phase ?]: LIKE 回退分支补模式语言层转义（%/_/\ + ESCAPE）：与未转义 MATCH 是同一类漏洞的两个面
- [Phase ?]: settings 的 base_url 校验与密钥键名守卫都长在 set_setting 内部：放调用方是约定，放写入路径才是机制
- [Phase ?]: 01-06: prism-mcp 的 bearer 在 McpDeps 中为私有字段 + pub(crate) expose_bearer，不是 pub 字段——token 的取用点在代码搜索中唯一可见
- [Phase ?]: 01-06: 三层门禁一律返回 403 + 空正文（不给 bearer 缺失单开 401）——状态码差异本身就是逐层试探的信息源（T-01-29）
- [Phase ?]: 01-06: rmcp SDK 侧 allowed_hosts/allowed_origins 与应用层中间件配成同一份做防御纵深；代价是端到端摘层反证失效，改由 sentinel-router 隔离测试承担
- [Phase ?]: 01-07: check-deps.sh 的 single-egress 拆两条——叶子 crate 整树断言不变，prism-engine 改为「直接依赖里没有 + cargo tree --invert 反向闭包里除 prism-llm 外无 prism-*」。原断言与「密钥唯一经 prism-llm 转交」互斥（src-tauri 只依赖 prism-engine，shell 通往钥匙串必经 facade）
- [Phase ?]: 01-07: 端到端注入测试的判别性不能落在「响应里有空集」上——Phase 1 的 list_feedback 返回空 vec，空结果与「handler 根本没调注入 trait」不可区分；改落在 engine 自己写的校验文本上（并实测确认 rmcp 对该参数无兜底校验）
- [Phase ?]: 01-07: 等事件的测试一律包 timeout，且前置条件断言要移出判别性测试——前者防「反证挂住而非变红」，后者防「反证落在前置条件上而非被守的那条断言上」
- [Phase ?]: 01-07: facade 方法一律同步（spawn_blocking 归调用方）：改成 async fn 会废掉 std::sync::MutexGuard !Send 对「跨 await 持写锁」的编译期保护
- [Phase ?]: 01-08: ipc 进程内测试的来源 URL 必须等于 tauri.conf.json 的 devUrl —— http://tauri.localhost 是 Windows 形态，macOS 下 is_local=false 会让每个命令被 ACL 拒成 'not allowed. Plugin not found'（与未注册错误同含 not found）
- [Phase ?]: 01-08: check-deps.sh 补第六条 shell-egress（src-tauri 不得直接依赖 prism-llm），形态同 facade-egress；反证证明原五条对该缺口全部不敏感
- [Phase ?]: 01-08: 有序性断言用序列比较而非集合比较；命令注册断言必须配未注册命令的负对照 + 断言 Ok（而非仅'错误不像未注册'）
- [Phase ?]: 01-09: Tauri v2 的 ACL 只管插件命令——generate_handler! 注册的自有命令不过 ACL。capabilities/ 缺失时 listen() 被拒而 invoke 全部正常，表现为「点了没反应且零报错」；任何新的 @tauri-apps/api 用法都需补一行 capability，且新调用点必须自己呈现 rejection
- [Phase ?]: 01-09: 可达性是独立于路由正确性的性质——「hash 是 X 时渲染谁」全绿不代表用户到得了 X（Tauri 窗口没有地址栏）。dev-only UI 用 import.meta.env.DEV 门控并配一条生产构建断言（grep dist 产物）
- [Phase ?]: 01-09: 勾选 INFRA-01（成功标准 2 的真实 WebView 两条通路由人工验证兑现）；INFRA-03 不勾——prism-llm 只有 secrets.rs，无 chat client，「支持 Anthropic/OpenAI 兼容端点」到 Phase 4
- [Phase ?]: 01-10: 凭据守卫扩在 validate_base_url 内部而非新加调用点——set_setting 一行未动，「机制而非约定」的设计陈述完整保住（T-01-43：绕过界面直接 invoke 不改变结果）
- [Phase ?]: 01-10: 密钥容器的边界必须建在**值**上而不只是键名上——is_secret_like_key 防的是键名，而 llm.base_url 这个键名完全正常，凭据藏在值里；userinfo 与 ?api-key= 是同一个洞的两个面
- [Phase ?]: 01-10: 拒绝面扩张时不加 StoreError 变体（复用 InvalidUrl）——加变体会连带要求 commands.rs::map_err 与前端 ERROR_COPY 同步扩表，那是 IPC 短码契约的变更；「是凭据还是 scheme」的区分只在前端本地校验层做得到
- [Phase ?]: 01-10: 前端体验层校验返回错误码而非布尔（localUrlIssue 取代 looksLikeHttpUrl）；判定面与 engine 逐项对齐（scheme/userinfo/query/fragment），避免「前端放行、engine 拒绝」在正常输入上出现
- [Phase ?]: 01-10: 计划里的两条断言实测不成立须就地修正——单字符用户名 u 让「错误串不含用户名」恒假；type=text 的端点输入框必须回显用户输入，document.body.innerHTML 级的不回显断言在任何正确实现下都红。不回显的守法面是**错误文案**，不是整个 DOM
- [Phase ?]: 01-11: sk- 段的字符类扩到 [A-Za-z0-9_-] 与长度下界 16→20 是同一个改动的两半——只放宽字符类会把普通连字符标识符扫进来，用误报换漏报。反证 F 与 H1/H2 的落点互补，证明两者是独立的判别维度
- [Phase ?]: 01-11: 检测控件必须自证判别力——失明的扫描器与干净的仓库退出码相同。selftest（14 阳性 / 7 阴性，与 scan 共用同一个 $PATTERN 变量，源码断言只赋值一次）把这件事变成每次 CI 重跑的断言
- [Phase ?]: 01-11: 扫描器不排除自身，阳性样本一律由片段拼出——scan 退出 0 即为「源码里没有命中自身正则的字面量」的可执行证明；docs/ 整目录排除按实测（零命中）取消，排除集只剩 .planning/
- [Phase ?]: 01-11: fixture 撞上扫描器时改 fixture 不改防线（放宽正则 / 加 allowlist / 加整目录排除 / 降阈值四者都算放宽）。两处新命中以改名解决（secret→fixture_bearer、token→configured），allowlist 零新增
- [Phase ?]: 01-11: 闸门在 justfile 与 CI 两处都显式写 all 而非靠无参数默认值——默认值哪天改回 scan-only，CI 会静默失去 selftest 那一半并照常绿；all 的顺序是 selftest 先、scan 后（先红的应当是原因而不是后果）
- [Phase ?]: 01-11: INFRA-03 仍不勾——证据侧（静态扫描能看见明文密钥）与写入侧（01-10）两半均已关闭，但需求文本的「支持 Anthropic/OpenAI 兼容端点」半句要到 Phase 4 才有 chat client（沿用 01-09/01-10 同一判据）
- [Phase ?]: 01-13: CSP 做成 csp / devCsp 两份——两处放宽（script-src 的 'unsafe-inline'、connect-src 的 Vite HMR 来源）只进 dev 那一份，发布形态一个字不改。Tauri v2 在 devCsp 缺席时退回用 csp，所以这一份不是可选项
- [Phase ?]: 01-13: assetProtocol 的配置侧与 cargo protocol-asset feature 是配套的两半，只关一半等于没关；cargo build 移除 feature 后仍绿即为「无代码路径依赖资源协议」的证明。csp 刻意不含 asset:
- [Phase ?]: 01-13: init_tracing 用 try_init 而非 init（后者在 dispatcher 已就位时 panic，「装日志把应用弄崩」是最不该发生的失败模式）；返回 bool 区分「这次装上了」与「早就装好了」；返回值在 run() 显式丢弃，与钥匙串失败不阻断启动同口径
- [Phase ?]: 01-13: 默认档取 info 而非 01-REVIEW 建议的 info,prism_mcp=debug——核对 middleware.rs 后确认 deny 的 reason 是编译期常量且为 warn!，info 已覆盖。少开一个 target 的 debug 就少一份「日志里会出现什么」的不确定性（T-01-58：sink 本身是新增外泄面）
- [Phase ?]: 01-13: 源码序断言的锚点必须取完整语句而非裸名字——include_str! 的匹配面同时含代码与注释，一条提到 tauri::Builder 的解释性注释就让断言在实现正确时变红（实测撞上）。这是 open.rs 范式的补充；失败方向是假红（安全）而非静默恒绿
- [Phase ?]: 01-13: subscriber-free 受检集合用 TAURI_FREE_CRATES（含 prism-cli，将来 externalBin 单独公证）且只看 --edges normal（dev-deps 里装 subscriber 合理）；反证必须注入 [dependencies] 而非 [dev-dependencies]，否则反证成功地什么都没证明
- [Phase ?]: 01-13: 不给 subscriber-free 加 justfile recipe 与 CI 步骤——它已纳入 all，而两处调用点跑的都是 check-deps.sh all，零调用点改动即成为闸门；同时避开与 01-11 的文件冲突。这是决定不是遗漏
- [Phase ?]: 01-13: 包合法性闸门不可自动放行（即使 auto_advance 为真）——缺失的审计行是执行器无法自行确立的事实，不是人可以橡皮图章的验证步骤。tracing-subscriber 已人工核对（tokio-rs/tracing 同仓库、2019 首发、~523M 下载、MIT、0.3.23）并写回 RESEARCH 审计表
- [Phase ?]: McpDeps::new 改为可失败构造：空/纯空白 bearer 在构造期即被拒（trim().is_empty()），McpDeps 一旦存在其 bearer 保证非空
- [Phase ?]: McpError::EmptyBearer 文案零插值——Phase 6 被拒的值可能是真 token 的畸形前缀，错误只陈述规则不回显值（T-01-29 同源）
- [Phase ?]: 比较层只加一条空 expected 早退，不做 WR-15 整体重写：绑定两件事会让「哪一层挡住了」的反证落点不再唯一
- [Phase ?]: 被单测钉住的 fail-open 行为用反转断言而非删除断言修复——被删掉的形态就是没人看着的形态
- [Phase ?]: 01-14: 裸值段下界取 16 且严格高于引号段的 8——引号串本身是「有人刻意写了字面量」的强信号，裸值没有；低下界会让 token = self.inner.value 这类表达式赋值整片误报，而被误报烦到的人会绕开闸门
- [Phase ?]: 01-14: SC-4 取样表第 5 行（password = hunter2hunter2，值仅 14 字符）作为已知残留写进源码注释，不下调阈值也不编成阴性断言——把已知缺口写成绿色的期望性质比留着缺口更坏
- [Phase ?]: 01-14: scan 的 cwd 固定（防线）与扫描面下限断言（报警器）成对存在——单有防线时它被删掉仍表现为绿，下限让作用域收窄从 OK/exit 0 变成 FAIL/exit 1
- [Phase ?]: 01-14: 检测控件每次扩宽都必须配一条只能经新分支命中的样本——否则旧分支替新分支兜底，selftest 从「不完整」变成「有误导性」（CR-01 的形态）
- [Phase ?]: 01-14: INFRA-03 仍不勾——证据侧（扫描器现在看得见裸值配置形态）与写入侧（01-10）两半均已关闭，但需求文本的「支持 Anthropic/OpenAI 兼容端点」半句要到 Phase 4 才有 chat client（沿用 01-09/01-10/01-11 同一判据）
- [Phase ?]: check_dup 的 FAIL 消息拆两行：退出码 + 明说「这不是发现了重复，是断言本身没跑起来」——两件事在退出码上不可区分，只能靠文案分
- [Phase ?]: 01-15: no-cycle 侧裸前缀实测方向与 REVIEW IN-04 散文相反（过敏而非逃逸），仍做尾随空格锚定但源码注释写实测方向，并写明将来 facade 层新 crate 必须显式加进 check_no_cycle
- [Phase ?]: constant_time_eq 只删不可达分支不重写折叠结构：绑定两件事会让「哪一层挡住了」的反证落点不再唯一
- [Phase ?]: 源码断言锚点定为完整语句 `if expected.is_empty() {`——反证实测证明注释里的词不足以钉住守卫，且死分支会稀释同一条断言的判别力
- [Phase ?]: bearer 归一化分两处只做一次：配置侧在 McpDeps::new trim，呈递侧只 trim_start（尾随 OWS 归 HTTP 解析层）
- [Phase ?]: 01-18: 新 StoreError 变体（JournalModeNotWal / CheckpointBusy）走 EngineError::Store(_) 兜底臂，不扩 IPC 短码表（沿用 01-10 决策）
- [Phase ?]: 01-18: 畸形 SQLite 版本串复用 SqliteTooOld 而非加第三个变体——「无法证明够新」与「确实太旧」同一处置
- [Phase ?]: 01-18: 断言落在解析结果而非下游布尔上——计划给的六条版本用例在当前 MIN_SQLITE 下恒真，真正的放行口是 3.x.53 → (3,53,0)
- [Phase ?]: 01-18: busy 复现测试留下（计划要求删除）——否则 close() 读 busy 列的三行再无行为面保护；代价是 lib 测试从 0.03s 变 5.5s
- [Phase ?]: 01-20: 回执 status 受控取值集合放 prism-engine 侧常量，不改 Receipt.status 类型——避免与 01-16/01-17 抢 dto.rs 与 prism-mcp 测试；上移条件是 Phase 5 COMMENT-03 定下真实状态机
- [Phase ?]: 01-20: status 精确匹配不做大小写折叠（Applied 被拒）——线协议值应当是确定的小写 token
- [Phase ?]: 01-20: 移除零构造点的 ServiceError::Backend（Phase 5 在关键路径下游，不属于「很快落地」）；重新引入检查点与文本约束就地写在文档注释里
- [Phase ?]: 01-21: EnvFilter 按特异性而非书写顺序取指令 —— 追加 rmcp=info 压得住 RUST_LOG=trace，也替换掉 RUST_LOG=rmcp=trace（实测两种形态）
- [Phase ?]: 01-21: #[serial] 只挡并发挡不住顺序 —— 依赖「我是第一个」的断言必须按前置状态参数化，标 serial 不够（实测 5/5 红）
- [Phase ?]: 01-21: has_been_set() 被 with_default 的线程局部 dispatcher 永久置真，不能用它证明全局 subscriber 装上了
- [Phase ?]: 01-21: 源码序断言的锚点须跨行 —— 单行锚点会先命中测试自己的字符串字面量，使 expect 在实现缺失时仍走过
- [Phase ?]: 冒烟流上界夹紧点放在不依赖 tauri 的 smoke::generate 内部（经 clamp_total），命令层只在文档注释里指向它——两处各夹一次会让「哪一个承重」含糊
- [Phase ?]: 「上界必须高于冒烟页默认值」写成 const _: () = assert!(...) 编译期断言而非单测：clippy assertions_on_constants 拒绝常量间的运行期 assert，且编译期形态更强
- [Phase ?]: spawn_blocking 落点用函数体切片型源码断言而非 Handle::try_current() 行为探针：探针答案取决于 tauri::async_runtime 的后端实现，钉住它等于埋一颗随上游升级而红的雷
- [Phase ?]: 01-23: errorCopy 的查找表建在 Object.create(null) 之上（机制），而不是靠每个查找点写 Object.hasOwn（约定）
- [Phase ?]: 01-23: 前端 URL 判定一律先解析后看结构，不做字节级前缀比较；与 engine validate_base_url 的一致性用临时跨语言对照测试取证（跑完即删）
- [Phase ?]: 01-23: API key 在前端 submitKey 与 prism_llm::secrets::set_api_key 两端走同一份 trim（与 01-16 McpDeps::new 同源）
- [Phase ?]: 01-24: 断言最小权限/最小面用精确相等而非 denylist 过滤——denylist 的强度恰好等于写它的人当时的想象力，实测 CSP 侧 5 条、capability 侧 2 条本该红的削弱形态在 denylist 下全绿
- [Phase ?]: 01-24: 一条 not.toContain 在目标串本来就合法含该 token 时是不可满足的断言，改写成「含该 token 的项集合精确等于 [合法的那一项]」——可满足且强度更高
- [Phase ?]: 01-24: CSP 白名单必须同时钉住指令名集合——script-src-elem 优先于 script-src 生效，只钉六条来源列表对它一条都不会红

### Pending Todos

None yet.

### Blockers/Concerns

- [Phase 3 前]: TD-01 阈值 T_high/T_low 与权重待 Track B 真实 agent diff 语料标定（Phase 3 内 harness 完成）
- [Phase 3 关闭前]: A→B→A 降级锚点复活语义未定义，需修订为 TD-01 v0.2（阻塞 phase close，不阻塞 start）
- [Phase 5/6 计划前]: F3 OQ-2/OQ-3、F4 OQ-1（declined 语义）需拍板（阻塞 F4 状态机）
- [Phase 4 前]: Q1 速读区模型档位待 M0 评测定档
- ~~[Phase 1]: rmcp 2.2 feature-flag 确切名称需对照 README 核验（5 分钟检查）~~ — RESOLVED (01-01)：`rmcp = { version = "2.2", features = ["server", "transport-streamable-http-server"] }` 已在 prism-mcp 中实际编译通过
- [数据勘误]: REQUIREMENTS.md 原 Coverage 写 51 条，实际 v1 REQ-ID 为 61 条，已于 roadmap 创建时更正
- [每个 plan 执行时]: 反证本身需要被验证（01-03 后第三次出现）：01-05 的计划反证 C 实跑不成立，暴露了 LIKE 分支缺阴性对照；触发器 DELETE 路径的验证按计划写法恒真（JOIN 掩盖了陈旧索引条目）；01-06 的两条计划反证（从 build_router 摘掉 Host / Origin 中间件）实跑**全绿**——rmcp SDK 自带的 allowlist 替它拒掉了。跑反证时要看**落点**（红在哪一条断言）而非只看红绿；当被测层之上还有第三方兜底时，反证必须把被测层放进一个**没有兜底**的最小链路里（01-06 的 sentinel-router 隔离测试即此形态）。
- [Phase 6 计划时]: rmcp SDK 的 Host 拒绝响应体为 "Forbidden: Host header is not allowed"，与本项目 T-01-29 的无差别拒绝口径不一致。当前应用层在外先拒使其不可达，但若 Phase 6 调整中间件顺序或 allowlist 使两者不再等价，SDK 的正文会泄漏落点
- ~~[01-09 / Phase 2+]: 若将来给项目加 capabilities/ 目录，has_app_acl_manifest 变 true，即使本地来源也会走 ACL —— src-tauri/tests/ipc.rs 届时需加一份测试用 capability，否则集体变红~~ — 实测不成立 (01-09)：capability 已加入，`cargo test -p prismdocs-shell --features test --test ipc` 仍 2 passed。被测的十个命令都是 `generate_handler!` 注册的自有命令、不受 ACL 管辖；ACL 生效影响的是**插件**命令，ipc 测试里一个都没有。若 Phase 6 给 ipc 测试加插件命令用例，那时才需要测试用 capability
- [Phase 2+ 每次新增前端 Tauri API 用法]: 任何新的 `@tauri-apps/api` import（fs/dialog/window/webview/http…）都必须在 `src-tauri/capabilities/default.json` 补一行权限，**其缺席表现为静默无操作而非报错**（ACL 只管插件命令，自有命令不过 ACL）；且新调用点必须自己接住并呈现 rejection，否则连「是不是 capability 缺了」都无从判断。`capabilities.test.ts` 挡得住「顺手加个 fs:default」的过宽修复，挡不住忘记加
- [Phase 2+ 每次写前端交互测试]: 单测会替被测系统假设掉前置条件——jsdom 替用户完成「输入 hash」（01-09 缺陷 1：冒烟页在真实窗口不可达而路由断言全绿）、mock 替运行时完成「ACL 放行」（01-09 缺陷 2）。两者的症状都是「什么都没发生，也没有报错」。这是 01-06 / 01-08 那族问题的第三、第四个变种，共同解药只有「把被测性质放进一个没有替身的链路里跑一次」
- [Phase 4 前] INFRA-03 仍不勾：~~01-10 只关闭了写入侧（凭据型 base_url 不入库），静态扫描能否看见明文密钥由 01-11 关闭~~ — 两半均已关闭（01-10 写入侧 / 01-11 证据侧，扫描器对 01-VERIFICATION.md § SC-4 取样表命中率 1/5 → 5/5）。剩余阻塞只有需求文本的「支持 Anthropic/OpenAI 兼容端点」半句——要到 Phase 4 才有 chat client（沿用 01-09 的同一判据）
- [Phase 2+ 每次新增 fixture / 测试局部变量]: 名字像密钥的标识符后跟一个引号串会被 `scripts/check-secrets.sh` 抓住，这是它该抓的形状。撞车时改 fixture 的名字或值，**不动扫描器**——放宽正则 / 加 allowlist / 加整目录排除 / 降长度阈值四者都算放宽。判断标准：若某个改动会让 selftest 的某条阴性样本被误命中、或某条阳性样本不再命中，那就是在放宽防线
- [Phase 1 收尾人工验证]: 01-13 Task 1 的 <human-check> 五步未执行（human_verify_mode: end-of-phase）。CSP 只在真实 WebView 里生效，jsdom 与 cargo test 都看不见它——npm run tauri dev 非白屏 / 设置页完整 / 冒烟页三入口 / Console 无 CSP 违规 / tauri build 的 dmg 重复验证（发布形态走 csp 而非 devCsp，是验证严格那一份的唯一路径）。顺带确认 tracing sink 非空：base_url 设成非 loopback 的 http 端点，终端应出现 settings.rs 的明文 http 告警。出现违规时只放宽 devCsp 或按报告点名的指令逐项追加，禁止设回 null
- Phase 6 注入 MCP bearer 时不得 unwrap McpDeps::new 的 Err：按 D-06 须降级为「MCP 服务不启动 + 一条 warn」，否则「token 没配」会从开着的门变成启动崩溃（T-01-54）
- INFRA-03 concurrency 探针未覆盖（flagged assumption）：keyring_core::set_default_store 是进程全局，并发写同一 keychain account 的行为无任何断言；secrets.rs 全部测试 #[serial] 是在回避而非回答。Phase 4 接上真实 chat client 时需补一条覆盖它的测试。

## Deferred Items

Items acknowledged and carried forward from previous milestone close:

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| *(none)* | | | |

## Session Continuity

Last session: 2026-07-29T13:32:20.476Z
Stopped at: Completed 01-24-PLAN.md
Resume file: None
