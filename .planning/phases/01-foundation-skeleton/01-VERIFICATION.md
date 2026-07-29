---
phase: 01-foundation-skeleton
verified: 2026-07-29T06:10:04Z
status: gaps_found
score: 3/4 must-haves verified
behavior_unverified: 0
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: 3/4
  previous_verified: 2026-07-29T02:43:11Z
  gaps_closed:
    - "成功标准 4 第三分句：代码与配置中无明文密钥 —— 凭据型 base_url 写入路径（gap 1）。verifier 用与上次同形的探针复现，7 个凭据型 URL 全部 write_ok=false / stored=None，settings 表 0 行"
    - "扫描器对 Anthropic `sk-ant-api03-…` 形态失明（gap 2 的核心失效）。verifier 把该形态注入受版本控制文件后 `check-secrets.sh scan` 退出 1；还原后退出 0"
  gaps_remaining:
    - "gap 2 的另一半：`check-secrets.sh` 关键词分支要求**值带引号**，未加引号的赋值（.env / YAML / TOML / CI env:）整类看不见——正是成功标准 4 里「配置」那一半，也正是本项目自己的 `mcp_bearer_token` 的形状"
  regressions: []
gaps:
  - truth: "成功标准 4 第三分句的执行机制：明文密钥静态扫描能真的看见明文密钥（配置文件那一半）"
    status: partial
    reason: >-
      本次 gap-closure 关掉了上次记录的主漏洞（`sk-` 字符类含连字符、长度阈值 16→20、
      ghp_/AKIA 前缀、docs/ 整目录排除取消、脚本自扫、selftest 进 CI 与 justfile），
      这些 verifier 都独立跑过并做了非恒真反证。但关键词分支
      `(api[_-]?key|secret|token|password)[[:space:]]*[=:][[:space:]]*["'][^"']{8,}`
      里的引号是**必需**的，于是任何**未加引号**的赋值整类命不中。
      成功标准 4 写的是「代码与**配置**中无明文密钥」——源码里的密钥是带引号的字符串
      字面量（能抓），配置文件（.env / YAML / TOML / GitHub Actions `env:` / shell）里的
      赋值常态是裸值（抓不到）。而本项目的第二个密钥 `mcp_bearer_token`
      （docs/keychain-naming.md 命名，Phase 6 注入 McpDeps）是一串没有任何供应商前缀的
      随机值——它在配置文件里的形态恰好落在这个洞里，三条前缀分支一条也接不住。
      verifier 实测：往 `.github/workflows/ci.yml` 追加
      `# mcp_bearer_token: <32 hex>` 与 `# ANTHROPIC_AUTH_TOKEN=<32 hex>` 两行后
      `bash scripts/check-secrets.sh scan` 仍打印 OK 并退出 0。
      selftest 让这件事从「不完整」变成「有误导性」：阳性样本 3
      （`ANTHROPIC_API_KEY=sk-ant-api03-…`，第 92 行）读起来像是在覆盖「未加引号的
      env 赋值」，实际是经 `sk-` 前缀分支命中的——把 `sk-` 去掉它就不再命中。
      整个 selftest 里没有任何一条样本单独隔离关键词分支去打一个裸值，
      而这正是本次改动扩宽的那条分支。脚本文件头自述的设计目标
      （「正则的判别力从此是每次 CI 都重新证明一遍的断言」）对该分支不成立。
      注：这是**证据缺口，不是现行泄漏**。verifier 用比项目自带正则宽得多的两轮扫描
      （关键词裸值 + 七种供应商前缀 + 追踪中的 .env/.pem/.key 文件）独立确认：
      当前仓库零明文密钥，命中的全是 fixture、注释与标识符。
    artifacts:
      - path: "scripts/check-secrets.sh"
        issue: "第 52 行 PATTERN 的关键词分支强制要求 `[\"']`；第 89-109 行的阳性组无一条隔离该分支打裸值，阳性样本 3 经 `sk-` 分支命中却读作「裸值 env 赋值已覆盖」"
      - path: ".planning/phases/01-foundation-skeleton/01-11-PLAN.md"
        issue: "must-have 第 3 条把该分支写成「后接 `=` 或 `:` 再接引号串」——引号前提在计划期就被写死，执行完全忠实于计划；这是计划层欠范围，不是执行失败"
    missing:
      - "PATTERN 的关键词分支让引号可选：`(${QUOTE}${NOT_QUOTE}{8,}|[A-Za-z0-9_./+~-]{16,})`（裸值段的长度下界须高于引号段，否则 `token = someVar` 之类会大面积误报）"
      - "补 selftest 阳性样本，值里不含任何供应商前缀，使其只可能经关键词分支命中——例如 `MCP_BEARER_TOKEN=<32 hex>` 与 `password: hunter2hunter2hunter`；删掉「引号可选」那半个改动后这两条必须立刻变红，而既有 5 行取样不会"
      - "顺带补三个未覆盖前缀：`github_pat_`、`xox[baprs]-`、`AIza`（后者是 Google 端点的形态，与 base_url 自定义端点同一使用面）"
      - "widen 后重跑 scan，撞车的 fixture 按脚本自述的单向约定改 fixture 不改防线（预期 `settings.rs:246` 的 `IN_QUERY` 与其在 `check-secrets.sh:117` 的镜像需改查询参数名）"
deferred:
  - truth: "INFRA-03 的「支持 Anthropic/OpenAI 兼容端点」分句（真正向端点发请求）"
    addressed_in: "Phase 4"
    evidence: "Phase 4 goal: 「A3：prism-llm 传输层（流式/重试/keyring）先行交付，速读区功能其后，使 4→6 边只依赖『传输层完成』」——Phase 1 只有 base_url 的存储与校验面，没有 chat client，01-09..01-13 各 SUMMARY 与 STATE.md 记录的顺延理由成立"
human_verification:
  - test: "`npm run tauri dev` 起应用，确认窗口不是白屏；打开设置页与 dev 冒烟页；跑冒烟页三个验证入口（总线事件往返 / Channel 1000 条有序流 / 中文搜索命中与阴性对照）；打开 WebView 控制台确认无 CSP 违规报告；随后 `npm run tauri build` 出 dmg 并对装出来的 app 重复一遍"
    expected: "五步全部正常。三个验证入口的读数与 01-09 人工验证时一致（事件计数 1:1、seq 校验通过 · 实收 1000 条、「锚定引擎」命中 >0 且「量子纠缠」= 0）"
    why_human: "CSP 只在真实 WebView 里生效——jsdom 与 `cargo test` 都看不见它。这是 01-13 Task 1 的 `<human-check>`，按 workflow.human_verify_mode=end-of-phase 顺延至此。**且这一项现在同时承载成功标准 2 的复验**：SC-2 的真实 WebView 确认由用户在 01-09 之后完成，而 01-13 在那之后给 WebView 加了 CSP（`connect-src 'self' ipc: http://ipc.localhost`）。若 IPC 来源在真实 WebView 下被这条策略挡住，所有命令一起失效——SC-2 的端到端证据因环境变更而失效，必须重跑，不能沿用旧读数"
  - test: "在 `npm run tauri dev` 的终端里把 base_url 设成一个非 loopback 的 `http://` 端点，观察终端"
    expected: "终端出现 tracing 格式的行，且 `settings.rs` 那条明文 http 告警（`LLM endpoint uses plaintext http to a non-loopback host`）实际打出来"
    why_human: "`tracing::dispatcher::has_been_set()` 只证明 dispatcher 就位，不证明日志真的到达终端。01-13 Task 3 自述这条端到端确认尚未做——WR-04 点名的三条安全决策日志是否真有落点，取决于它"
---

# Phase 1: 基建骨架 Verification Report（re-verification）

**Phase Goal:** 可独立测试的 Rust engine workspace + Tauri 薄 shell 就绪，五项不可逆决策（单写者 SQLite + 读池、FTS5 CJK tokenizer、keyring-core 用法、prism-mcp trait 反转、notify-then-fetch）全部落地并各有验证通路
**Verified:** 2026-07-29T06:10:04Z（HEAD `b738038`）
**Status:** gaps_found
**Re-verification:** Yes — 复验 01-10 / 01-11 / 01-12 / 01-13 四份 gap-closure 计划

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | engine workspace 不依赖 tauri 即可 `cargo test` 全绿（D-01）；`cargo tree -d` 无重复 rusqlite/reqwest；prism-mcp 仅依赖注入的 service trait，编译期无 facade↔mcp 依赖环 | ✓ VERIFIED | 回归通过，且新增第七条断言（见 §SC-1） |
| 2 | 事件总线骨架各验证一条通路：一条总线事件经粗粒度 Tauri event 往返前端（notify-then-fetch），一条命令经 Channel 有序流式返回（A1） | ✓ VERIFIED（有告警） | 自动化三层全绿；真实 WebView 那一层的旧证据因 01-13 加 CSP 而**过期**，已并入人工验证第 1 项 |
| 3 | SQLite schema v1 落地：WAL + 单写者 + r2d2 读池（query_only=ON）并发读写正常；FTS5 中文查询返回非零结果；rusqlite_migration 迁移体系可用，bundled SQLite ≥3.51.3 | ✓ VERIFIED | 回归通过 + verifier 自行探针实测（见 §SC-3） |
| 4 | API key 经 keyring-core + apple-native-keyring-store 写入系统钥匙串并可读回，prism-llm 为唯一网络出口与唯一密钥入口，**代码与配置中无明文密钥** | ✗ PARTIAL | 前两分句 ✓；第三分句：**状态**为真（独立宽扫零命中），**执行机制**在「配置」那一半仍有洞——见 §SC-4 与 frontmatter `gaps` |

**Score:** 3/4 truths verified（0 present, behavior-unverified）

---

### 上次两条 gap 的复验结论

#### gap 1（凭据型 base_url 写入 settings 表）→ **已关闭**

不采信 01-10-SUMMARY 的说法，用与上次同形的临时探针（`crates/prism-store/tests/` 下建、跑完即删、`git status` 干净）直接打写入路径：

```
PROBE "https://user:sk-verifier-probe-0000000000000000@api.vendor.com/v1" -> write_ok=false stored=None
PROBE "https://user@api.vendor.com/v1"                                    -> write_ok=false stored=None
PROBE "https://api.vendor.com/v1?api-key=verifier-probe-0000"             -> write_ok=false stored=None
PROBE "https://api.vendor.com/v1#verifier-probe-0000"                     -> write_ok=false stored=None
PROBE "HTTPS://user:pw@api.vendor.com/v1"                                 -> write_ok=false stored=None
PROBE "https://api.vendor.com/v1/?"                                       -> write_ok=false stored=None
PROBE "https://api.vendor.com/v1#"                                        -> write_ok=false stored=None
PROBE settings rows = 0
```

上次那一行被持久化的凭据（`stored = Some("https://user:sk-…@api.vendor.com/v1")`）现在是 `None`，表 0 行。守卫落在 `validate_base_url`（`settings.rs:68-80`，userinfo 与 query/fragment 两个分支），而 `set_setting`（111-119）在 `tx.execute` 之前调用它——是机制不是约定。前端 `localUrlIssue`（`Settings.tsx:23-35`）逐项对齐 scheme / userinfo / query / fragment 并返回错误码而非布尔。`settings_base_url_rejects_credential_bearing_values` 的六组断言各自钉住一个具体的静默失败模式（含「只有用户名没有密码」与「一律拒绝」两条阴性对照）。

#### gap 2（扫描器看不见明文密钥）→ **部分关闭**

**已关闭的那一半**（verifier 亲手跑的，不是转述）：

- `bash scripts/check-secrets.sh all` → `OK: pattern discriminates (14 positive / 7 negative samples)` + `OK: no plaintext secret in version-controlled files`，退出 0。
- 上次那张 5 行取样表，现在 5/5 命中（用 selftest 的正则副本逐条回放确认）。
- **非恒真反证**：把一个 `sk-ant-api03-…` 形态的假串追加进 `src/lib/ipc.ts` → `scan` 打印 `FAIL: … src/lib/ipc.ts:131` 并退出 1；`git checkout` 还原后退出 0。
- `docs/` 整目录排除已取消（排除集只剩 `.planning/`），脚本自身进入受检集合，selftest 与 scan 共用同一个 `$PATTERN` 变量，`justfile` 与 `.github/workflows/ci.yml:36-37` 两个调用点都显式跑 `all`。

**仍开着的那一半**（见 §SC-4 与 frontmatter `gaps`）：关键词分支要求值带引号，未加引号的赋值整类看不见。

---

### SC-1 证据（engine 独立性 / 依赖图性质）

沿用上次的纪律：不拿 `cargo test --workspace` 冒充 D-01 的证据（它会编译 shell，恒真）。

| 检查 | 命令 | 结果 |
|---|---|---|
| engine-only 测试全绿 | `cargo test -p prism-types -p prism-store -p prism-fs -p prism-parse -p prism-anchor -p prism-llm -p prism-mcp -p prism-engine` | **110 passed / 0 failed / 1 ignored** |
| 六条既有依赖方向断言 | `bash scripts/check-deps.sh all` | dup / tauri-free / no-cycle / single-egress / facade-egress / shell-egress 全 OK |
| **新增第七条** | 同上，`subscriber-free` | `OK: all checked crates are tracing-subscriber-free (engine set + CLI helper)` |
| 新断言非恒真（verifier 亲手做的） | 往 `crates/prism-store/Cargo.toml` 的 `[dependencies]` 注入 `tracing-subscriber = "0.3"` → `bash scripts/check-deps.sh subscriber-free` | `FAIL: prism-store depends on tracing-subscriber`，exit **1**；已还原（`Cargo.lock` 一并 `git checkout`，`git status` 干净） |
| subscriber 只属于壳 | `src-tauri/Cargo.toml:31` | 唯一一处 `tracing-subscriber = { workspace = true }`，旁边写明「任何 engine crate 都不加这一行」 |
| 全量参考 | `cargo test --workspace` / `cargo clippy --workspace --all-targets -- -D warnings` / `npx tsc --noEmit` | **127 passed（1 ignored）** / 0 warning / 0 error |

01-12 把 `McpDeps::new` 改成可失败构造后，四处调用点（三个 mcp 测试文件 + `prism-engine/tests/facade.rs`）全部适配且 workspace 全绿——这条跨 crate 的签名变更没有把 SC-1 的依赖方向弄脏（`check-deps.sh no-cycle` 仍是 `OK: prism-mcp -> prism-types only`）。

### SC-2 证据（IPC 双通路）

| 层 | 证据 | 本次状态 |
|---|---|---|
| 自动化（壳） | `cargo test -p prismdocs-shell --features test --test ipc` → **2 passed**；`cargo test --workspace` 里 shell 的 `map_recv` 三分支单测与事件名契约全绿 | ✓ 回归通过 |
| 自动化（前端） | `npm run test -- --run` → **34 passed / 7 files**（上次 32/6，增量为 `tauri-security.test.ts`） | ✓ 回归通过 |
| 接线 | `bus_adapter.rs:18 EVENT_CHANGED="prism://changed"` → `bus_adapter.rs:66 app.emit(EVENT_CHANGED, ev)` → `useEngineInvalidation.ts:26 listen<EngineEvent>(EVENT_CHANGED, …)` → `invalidateQueries()` | ✓ WIRED |
| 真实 WebView（人工） | 01-09 之后由用户完成：计数 1:1、离开页面再回来不翻倍；「seq 校验通过 · 实收 1000 条」 | ⚠️ **证据过期** |

**为什么标过期。** 01-13 给 WebView 装了 CSP，其中 `connect-src 'self' ipc: http://ipc.localhost` 直接管辖 IPC 的来源。用户做那次人工验证时 `csp` 还是 `null`——那次绿色是在一个已经不存在的环境里取得的。若这条策略在真实 WebView 下挡住 IPC，受影响的不是某一个命令而是全部十个，SC-2 的两条通路一起断。`tauri-security.test.ts` 守的是策略的**形态**（它自己在第 17 行写明「jsdom 看不见 CSP」），`cargo test` 走的是 `mock_builder`（没有 WebView）——两条自动化路径都结构性地看不见这个风险。因此 SC-2 保留 VERIFIED（其自身的验证通路仍在且非恒真），但把真实 WebView 的复验并入人工验证第 1 项，且**不得沿用旧读数**。

### SC-3 证据（schema v1 / 并发纪律 / FTS CJK）

| 检查 | 命令 / 位置 | 结果 |
|---|---|---|
| 并发纪律六测 | `cargo test -p prism-store --test concurrency` | **6 passed** |
| FTS CJK 四测 | `cargo test -p prism-store --test fts_cjk` | **4 passed**（两条分支各有阴性对照） |
| 迁移体系 + settings | `cargo test -p prism-store --lib` | **21 passed**（上次 20，增量为 01-10 新增的凭据型对照） |
| 运行期实测（verifier 临时探针，跑完即删） | `sqlite_version()` / `PRAGMA journal_mode` / `PRAGMA query_only` / `documents_fts` 存在性 | **3.53.2**（≥3.51.3 ✓）/ **wal** ✓ / **1** ✓ / **1** ✓ |
| tokenizer 定案 | `migrations/001_schema_v1.sql:41-47` | external-content FTS5，`content_rowid='rowid_pk'`，`tokenize = 'trigram'` |

### SC-4 证据（钥匙串 / 唯一出口 / 无明文密钥）

| 分句 | 状态 | 证据 |
|---|---|---|
| API key 经 keyring-core + apple-native-keyring-store 写入并可读回 | ✓ | `secrets.rs:38` `apple_native_keyring_store::keychain::Store::new()`；`cargo test -p prism-llm` **9 passed / 1 ignored**；真实钥匙串往返由用户在 01-09 完成，本轮四份 gap-closure 未触碰 `prism-llm`（`git diff c231656..HEAD --stat` 无该 crate） |
| prism-llm 为唯一网络出口与唯一密钥入口 | ✓ | `check-deps.sh` 的 `single-egress` / `facade-egress` / `shell-egress` 三条全绿；`src-tauri/Cargo.toml` 只依赖 prism-engine / prism-types / prism-store，不直连 prism-llm |
| **代码与配置中无明文密钥** | ✗ PARTIAL | 状态为真、机制有洞——见下 |

**状态侧（独立确认为真）。** 不采信项目自带的扫描器，verifier 用宽得多的两条正则自行扫了一遍受版本控制的文件：

```
git grep -niE '(api[_-]?key|apikey|secret|token|password|passwd|bearer|credential)[[:space:]]*[=:][[:space:]]*[^[:space:],;)}]{8,}'
git grep -nE  '(sk-|sk_|ghp_|gho_|github_pat_|AKIA|AIza|xox[baprs]-|eyJ[A-Za-z0-9_-]{10,})'
git ls-files | grep -iE '(^|/)\.env|\.pem$|\.key$|credentials'
```

命中共 19 行，逐行看过：全部是 fixture（`FIXTURE_SECRET` / `FAKE_KEY` / `s3cr3t-token-value` / `IN_QUERY`）、注释里对格式的讨论、或普通标识符（`bearer: Arc<str>`、`task_failed`）。第三条命令零输出——没有任何 `.env` / `.pem` / `.key` / credentials 文件在版本控制里。**当前仓库确无明文密钥。**

**机制侧（实测仍有洞）。** 关键词分支的引号是必需的：

```
PATTERN 关键词段 = (api[_-]?key|secret|token|password)[[:space:]]*[=:][[:space:]]*["'][^"']{8,}
```

verifier 用第 52 行原样取出的 PATTERN 逐条回放：

| 取样行 | 是否被扫描器看见 |
|---|---|
| `ANTHROPIC_API_KEY=abcdef0123456789abcdef0123456789` | ✗ |
| `mcp_bearer_token: 7f3a9c1e5b2d8f4a6c0e9b7d3f1a5c8e` | ✗ |
| `api_key=abcdef0123456789abcdef` | ✗ |
| `export OPENAI_API_KEY=abcdefghijklmnopqrstuvwxyz012345` | ✗ |
| `password = hunter2hunter2` | ✗ |
| `token = 0123456789abcdef0123` | ✗ |
| `secret: not-a-real-value-here` | ✗ |
| `bearer_token=0123456789abcdefghij` | ✗ |

同一批值加上引号就全部命中——差别只在引号。端到端复现：往 `.github/workflows/ci.yml` 追加

```
# mcp_bearer_token: 7f3a9c1e5b2d8f4a6c0e9b7d3f1a5c8e
# ANTHROPIC_AUTH_TOKEN=9f2b7d1a4c6e8092b5d3f7a1c9e0b246
```

之后 `bash scripts/check-secrets.sh scan` 仍打印 `OK: no plaintext secret in version-controlled files` 并退出 **0**（已还原）。

三点使这一条不能当作已关闭：

1. **成功标准写的是「代码与配置」。** 源码里的密钥是带引号的字符串字面量（抓得到）；配置文件（.env / YAML / TOML / GitHub Actions `env:` / shell / justfile）里的赋值常态是裸值（抓不到）。被漏掉的恰好是成功标准点名的那一半。
2. **落进洞里的是本项目自己的第二个密钥。** `mcp_bearer_token` 由 `docs/keychain-naming.md` 命名、Phase 6 注入 `McpDeps`，是一串没有任何供应商前缀的随机值——三条前缀分支（`sk-` / `ghp_` / `AKIA`）一条也接不住它，只剩关键词分支，而关键词分支要引号。
3. **selftest 让它从「不完整」变成「有误导性」。** 阳性样本 3（第 92 行）`ANTHROPIC_API_KEY=sk-ant-api03-xyz…` 是一条未加引号的 env 赋值，读起来像是在证明裸值赋值已覆盖——它实际经 `sk-` 前缀分支命中，把 `sk-` 去掉就完全不命中。整个 14 条阳性组里没有一条隔离关键词分支去打裸值，而那正是本次扩宽的分支。脚本文件头写着「正则的判别力从此是每次 CI 都重新证明一遍的断言」，对这条分支不成立。

**Chesterton's Fence 检查。** 引号前提不是执行期的疏漏：01-11-PLAN.md 的 must-have 第 3 条原文就是「后接 `=` 或 `:` 再**接引号串**」，执行完全忠实于计划；01-11-SUMMARY.md 里能检索到的相关权衡只有「加 `-i` 不引入额外误报」与「字符类放宽须配长度阈值提高」，**没有**任何关于裸值形态的讨论。所以这是计划层欠范围（与上次 gap 的 `missing` 第一条「`=` 与 `:` 两种赋值形态」相比，计划把它窄化成了引号形态），不是执行失败——闭环计划应当只针对扫描器，不必回溯追责执行。也因此不建议用 override 吸收：这不是被权衡后接受的取舍。

**与 01-REVIEW.md CR-01 的对账。** 我先独立跑完上表与端到端注入实验，之后才读 CR-01。结论一致：事实层面完全吻合（引号必需、样本 3 经前缀分支命中、`mcp_bearer_token` 是本项目自己的密钥、当前无实际泄漏）。分级上我与它一致（CR-01 记为 BLOCKER），但把范围收得更窄：**状态断言为真，失守的只是执行机制在配置文件形态上的覆盖面**——闭环计划因此只需改 `check-secrets.sh` 一个文件加两条 selftest 样本，不涉及任何产品代码。

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `crates/prism-store/src/settings.rs` | k/v + base_url 值侧校验（scheme/host/userinfo/query/fragment） | ✓ VERIFIED | 338 行；`validate_base_url:68-80` 两个新分支；探针实测 7 种凭据形态全拒 |
| `src/pages/Settings.tsx` | 设置页 + 提交前本地凭据检查 | ✓ VERIFIED | `localUrlIssue:23-35` 与 engine 逐项对齐，返回错误码而非布尔 |
| `src/lib/ipc.ts` | 命令封装 + 错误码→中文文案 | ✓ VERIFIED | 新增 `invalid_url_credentials` 文案键 |
| `scripts/check-secrets.sh` | 明文密钥静态检查 + selftest | ⚠️ **INCOMPLETE** | 主漏洞已关（sk- 字符类 / 阈值 / 前缀 / 排除集 / 自扫 / 闸门）；关键词分支要求引号，裸值整类漏（gap） |
| `scripts/check-deps.sh` | 七条依赖方向断言 | ✓ VERIFIED | 新增 `subscriber-free` 并纳入 `all`；已由 verifier 注入反证其非恒真 |
| `crates/prism-mcp/src/deps.rs` | 可失败构造，空 bearer 即拒 | ✓ VERIFIED | `new:47-58` 返回 `Err(McpError::EmptyBearer)`（含 `trim()`，纯空白也拒）；`an_empty_bearer_is_refused_at_construction` 通过 |
| `crates/prism-mcp/src/middleware.rs` | 三层门禁 + 空 expected 早退 | ✓ VERIFIED | `constant_time_eq:151` 空 expected 早退；第 225/229 行原先被钉住的 fail-open 断言已**反转**为 `assert!(!constant_time_eq("", ""))` |
| `crates/prism-mcp/tests/middleware_gate.rs` | A/B 两组门禁测试 | ✓ VERIFIED | **12 passed**，含 `an_empty_presented_token_is_denied_by_the_bearer_layer_alone`（CR-03 描述的现实攻击形态）与 `an_empty_configured_bearer_cannot_be_constructed_in_the_first_place` |
| `src-tauri/tauri.conf.json` | 双份 CSP + 关闭资源协议 | ✓ VERIFIED（有告警） | `csp` / `devCsp` 各一份；`assetProtocol.enable:false`、`scope:[]`；真实 WebView 效果待人工验证 |
| `src-tauri/Cargo.toml` | 移除 `protocol-asset` feature | ✓ VERIFIED | feature 列表已空，注释写明「两半是配套的，只关一半等于没关」 |
| `src/lib/tauri-security.test.ts` | 把 CSP 形态钉成断言 | ✓ VERIFIED（有告警） | 79 行；verifier 把 `csp` 改回 `null` → 测试当场变红（`Expected: "string" / Received: "object"`），已还原。断言③的强度见 WR-A |
| `src-tauri/src/lib.rs` | `init_tracing()` 在 `run()` 第一步 | ✓ VERIFIED | `try_init()` 非 `init()`；三条测试（安装 / `has_been_set()` / 幂等）+ 一条源码序断言 |
| `.github/workflows/ci.yml` · `justfile` | 两个调用点都跑到 selftest | ✓ VERIFIED | ci.yml:36-37 与 justfile:25-30 都显式写 `all`，注释写明「不靠无参数默认值」 |
| （上一轮已验证且本轮未改动的 artifact） | `open.rs` / `migrations.rs` / `001_schema_v1.sql` / `search.rs` / `service.rs` / `secrets.rs` / `bus_adapter.rs` / `commands.rs` / `useEngineInvalidation.ts` / `DevSmoke.tsx` / `keychain-naming.md` | ✓ VERIFIED（回归） | `git diff c231656..HEAD --stat` 确认未触碰；对应测试本轮全部重跑通过 |

### Key Link Verification

| From | To | Via | Status |
|---|---|---|---|
| `crates/prism-store/src/settings.rs` | `url::Url` | `url.username()` / `url.password()` / `url.query()` / `url.fragment()` 逐项检查 | ✓ WIRED（上轮 PARTIAL → 本轮修复） |
| `src/pages/Settings.tsx` | `src/lib/ipc.ts` | `localUrlIssue` 产出 `invalid_url_credentials` 错误码，文案只在 `ERROR_COPY` 生成 | ✓ WIRED |
| `crates/prism-mcp/src/middleware.rs` | `crates/prism-mcp/src/deps.rs` | `require_bearer` 经 `expose_bearer` 取配置值，该值现在保证非空 | ✓ WIRED |
| `crates/prism-mcp/src/deps.rs` | `crates/prism-mcp/src/lib.rs` | 构造失败返回 `McpError::EmptyBearer` | ✓ WIRED |
| `crates/prism-engine/tests/facade.rs` | `crates/prism-mcp/src/deps.rs` | 注入点已适配可失败构造 | ✓ WIRED |
| `src/lib/tauri-security.test.ts` | `src-tauri/tauri.conf.json` | 静态 `import config from "../../src-tauri/tauri.conf.json"` | ✓ WIRED（改 null 即红，已反证） |
| `scripts/check-deps.sh` | `src-tauri/Cargo.toml` | `subscriber-free` 断言 | ✓ WIRED（注入即红，已反证） |
| `.github/workflows/ci.yml` · `justfile` | `scripts/check-secrets.sh` | 两处都调 `all` | ✓ WIRED |
| `src-tauri/src/lib.rs` | 全 workspace 的 `tracing::` 发射点 | `init_tracing()` 装全局 dispatcher | ⚠️ PARTIAL — dispatcher 就位已断言；日志真的落到终端待人工确认 |
| `src-tauri/src/bus_adapter.rs` | 前端 `listen('prism://changed')` | `AppHandle::emit` | ⚠️ PARTIAL — 接线与自动化层成立；真实 WebView 在新 CSP 下未复验 |
| `prism-mcp` | `prism-engine` | （必须不存在） | ✓ 不存在 |
| `prismdocs-shell` | `prism-mcp` / `prism-llm` | （必须不直连） | ✓ 不存在 |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|---|---|---|---|
| engine 脱离 tauri 可测 | `cargo test -p prism-types … -p prism-engine` | 110 passed / 0 failed / 1 ignored | ✓ PASS |
| 七条依赖方向断言 | `bash scripts/check-deps.sh all` | 7 × OK, exit 0 | ✓ PASS |
| 新断言非恒真 | 注入 `tracing-subscriber` 到 prism-store → `check-deps.sh subscriber-free` | `FAIL: prism-store depends on tracing-subscriber`, exit 1 | ✓ PASS |
| 凭据型 base_url 被拒 | 临时探针：7 种形态过 `set_setting` | 全部 `write_ok=false stored=None`，表 0 行 | ✓ PASS（上轮 FAIL） |
| 扫描器看得见 Anthropic 形态 | 注入 `sk-ant-api03-…` 到 `src/lib/ipc.ts` → `check-secrets.sh scan` | exit 1 + 命中行 | ✓ PASS（上轮 FAIL） |
| selftest 自证判别力 | `bash scripts/check-secrets.sh selftest` | `pattern discriminates (14 positive / 7 negative)` | ✓ PASS |
| **扫描器看得见裸值赋值** | 注入 `mcp_bearer_token: <32hex>` + `ANTHROPIC_AUTH_TOKEN=<32hex>` 到 `ci.yml` → `check-secrets.sh scan` | `OK: …`, exit **0** | ✗ **FAIL** |
| 独立宽扫（不用项目正则） | 三条 verifier 自拟的 `git grep` / `git ls-files` | 19 行命中全为 fixture / 注释 / 标识符；零 `.env`/`.pem`/`.key` | ✓ PASS |
| 空 bearer 构造期即拒 | `cargo test -p prism-mcp` | 11 + 12 + 3 passed，含两条空 bearer 用例 | ✓ PASS |
| CSP 钉子非恒真 | 把 `tauri.conf.json` 的 `csp` 改回 `null` → `npm run test -- --run src/lib/tauri-security.test.ts` | 1 failed（`Expected: "string" / Received: "object"`） | ✓ PASS |
| bundled SQLite / WAL / query_only | 临时探针 | `3.53.2` / `wal` / `1` | ✓ PASS |
| FTS CJK 判别性 | `cargo test -p prism-store --test fts_cjk` | 4 passed | ✓ PASS |
| IPC 命令可达 | `cargo test -p prismdocs-shell --features test --test ipc` | 2 passed | ✓ PASS |
| 前端 | `npm run test -- --run` / `npx tsc --noEmit` | 34 passed / 7 files；0 error | ✓ PASS |
| 全量闸门 | `cargo test --workspace` / `cargo clippy --workspace --all-targets -- -D warnings` | 127 passed（1 ignored）/ 0 warning | ✓ PASS |

所有临时探针与注入均已还原，收尾 `git status --porcelain` 只剩两个未跟踪的 `.planning/research/.cache/*.json`（本次验证之前就存在）。

### Probe Execution

本仓库无 `scripts/*/tests/probe-*.sh` 约定探针；四份 gap-closure PLAN / SUMMARY 亦未声明任何 probe。Step 7c 以「无 probe 可跑」记录，替代证据为上表的 behavioral spot-checks——其中 `check-deps.sh` / `check-secrets.sh` 两个脚本按 probe 的方式对待：verifier 亲自执行、记录退出码，并对各自做了注入式非恒真反证。

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|---|---|---|---|---|
| INFRA-01 | 01-01…01-09, 01-11, 01-12, 01-13 | Rust engine workspace + Tauri 薄 shell + 事件总线骨架；prism-mcp 经 service trait 反转解依赖环 | ✓ SATISFIED | SC-1 + SC-2 全部证据；01-12 的签名变更未污染依赖方向；01-13 的 subscriber 只落在壳 |
| INFRA-02 | 01-03, 01-05, 01-07, 01-09, 01-10 | SQLite WAL 单写者 + r2d2 读池；FTS5 CJK tokenizer；rusqlite_migration；bundled SQLite ≥3.51.3 | ✓ SATISFIED | SC-3 全部证据（探针实测 3.53.2 / wal / query_only=1） |
| INFRA-03 | 01-02, 01-04, 01-05, 01-07, 01-09, 01-10, 01-11, 01-13 | API key 存系统钥匙串；支持 Anthropic/OpenAI 兼容端点与自定义 base_url；prism-llm 为唯一网络出口与唯一密钥入口 | ⚠️ **PARTIAL** | 钥匙串 ✓、唯一出口/入口 ✓、自定义 base_url（存储与校验面）✓ 且值侧守卫已补齐；「支持 Anthropic/OpenAI 兼容端点」→ **deferred to Phase 4**；扫描器覆盖面 → 见 gap |

**INFRA-03 的顺延判断。** 01-09 以来每份计划都不勾选 INFRA-03，理由是「支持 Anthropic/OpenAI 兼容端点」需要 Phase 4 才存在的 chat client。这个理由**成立**：Phase 4 的 goal 明写「A3：prism-llm 传输层（流式/重试/keyring）先行交付」，是这条分句的确定归属地，因此按 Step 9b 记为 deferred 而非 gap。

但记账口子要补：`.planning/REQUIREMENTS.md:148` 的 Traceability 表仍把 INFRA-03 映射到 **Phase 1**，而 Phase 1 即将收尾。若不动它，这条需求在 Phase 1 关闭后就没有任何 phase 负责推进——GSD 的「每条需求映射且仅映射一个 Phase」规则会把它变成孤儿。建议二选一（不阻断本 phase）：把 INFRA-03 改映射到 Phase 4；或按 INFRA-04/05 的既有先例拆成「Phase 1 完成的密钥/出口部分」与「Phase 4 完成的端点部分」并在表下加注。

**Orphan 检查：** Traceability 表把且仅把 INFRA-01 / INFRA-02 / INFRA-03 映射到 Phase 1，三条全部被 plan frontmatter 认领——**无 orphaned requirement**。

### Anti-Patterns Found

`TBD` / `FIXME` / `XXX` / `TODO` / `HACK` / `PLACEHOLDER` 在 `crates/`、`src/`、`src-tauri/`、`scripts/`、`.github/`、`justfile` 下**零命中**——无未挂钩的技术债标记。

下表只列本轮 gap-closure 引入或仍然成立的问题（每条都自己看过代码或跑过命令）：

| File | Line | Pattern | Severity | Impact |
|---|---|---|---|---|
| `scripts/check-secrets.sh` | 52, 89-109 | 关键词分支要求引号，裸值整类漏；selftest 无样本隔离该分支 | 🛑 Blocker | 本轮唯一 gap，见 §SC-4 |
| `src/lib/tauri-security.test.ts` | 49-59 | 断言③是四项 denylist，不是白名单 | ⚠️ Warning（WR-A） | 我复算过：`script-src 'self' 'unsafe-inline'` 与 `script-src 'self' https://cdn.evil.example` 都能通过。**当前生产 CSP 是 `script-src 'self'`，是紧的**——这是测试强度问题，不是配置问题。建议改成白名单（只允许 `'self'` 与 `'none'`） |
| `src-tauri/src/lib.rs` | 96-125 | `tracing_init_installs_a_global_subscriber_and_is_idempotent` 依赖全局进程状态 | ⚠️ Warning | 同一测试二进制里若有第二处先装了 subscriber，第一条 `assert!(init_tracing())` 会红。目前壳只有这一处，`cargo test --workspace` 实测绿；测试并行度或新增测试改变时会成为不稳定源 |
| `crates/prism-store/src/lib.rs` | 53-60 | `sqlite_version_returns_three_dotted_numbers` 只断言 major==3 | ⚠️ Warning | SC-3 写的是「≥3.51.3」，自动化断言没有把这个下界钉住——本机实测 3.53.2 满足，但 bundled 版本回退不会有任何东西变红 |
| `crates/prism-store/tests/concurrency.rs` | 43-68 | `assert!(after >= 1)` 恒真；注释声称的快照隔离未实现 | ⚠️ Warning | WR-01（上轮）仍然成立且未修。承重断言仍在，SC-3 不受影响 |
| `crates/prism-store/src/open.rs` | 53-59, 147-153 | `PRAGMA journal_mode=WAL` 返回行被丢弃；`close()` 丢弃 checkpoint busy 标志 | ⚠️ Warning | WR-02 / WR-03（上轮）仍然成立且未修 |
| `crates/prism-mcp/tests/middleware_gate.rs` | 阳性对照 | `!is_client_error()` | ⚠️ Warning | WR-14（上轮）仍然成立且未修 |
| `src/lib/ipc.ts` · `src-tauri/src/lib.rs` · `src-tauri/src/commands.rs` · `scripts/check-deps.sh` · `crates/prism-mcp/src/handler.rs` · `crates/prism-engine/src/services.rs` · `.github/workflows/ci.yml` · `src/lib/capabilities.test.ts` · `src/pages/Settings.tsx` | — | 原型链解析 / dev 命令进 release IPC 面 / `total: u32` 未夹紧 / `cargo tree \|\| true` / schema-handler 不一致 / status 未约束入日志 / clippy 与 ESLint 覆盖缺口 / 能力断言是 denylist / 读失败渲染为「未配置」 | ⚠️ Warning | WR-05 … WR-16（上轮）**全部仍然成立且本轮未修**。它们不阻断成功标准，但数量已积到 12 条，建议在进 Phase 2 前集中处理一轮 |

**关于 WR-04（无 tracing-subscriber）：已关闭。** `src-tauri/Cargo.toml:31` 引入 subscriber，`init_tracing()` 在 `run()` 第一步安装（源码序断言看住），`check-deps.sh subscriber-free` 保证它不扩散进 engine。剩余的只是「日志真的打到终端了吗」这条端到端确认，已列入人工验证第 2 项。

### Human Verification Required

两项，均为 01-13 自述的 Outstanding Human Verification，按 `workflow.human_verify_mode: end-of-phase` 顺延至此。

#### 1. 真实 WebView 下的 CSP 与 IPC 双通路（同时承载 SC-2 的复验）

**Test:** `npm run tauri dev` 起应用 → 窗口不是白屏 → 打开设置页与 dev 冒烟页 → 跑冒烟页三个验证入口 → 打开 WebView 控制台确认无 CSP 违规报告 → 随后 `npm run tauri build` 出 dmg，对装出来的 app 重复一遍。
**Expected:** 五步全部正常；三个入口的读数与 01-09 人工验证一致（事件计数 1:1、「seq 校验通过 · 实收 1000 条」、「锚定引擎」命中 >0 且「量子纠缠」= 0）。
**Why human:** CSP 只在真实 WebView 里生效，jsdom（`tauri-security.test.ts` 第 17 行自己写明）与 `cargo test`（走 `mock_builder`，无 WebView）都结构性地看不见它。**且这一项现在同时是 SC-2 的复验**——SC-2 的真实 WebView 证据取自 `csp: null` 的环境，而 `connect-src 'self' ipc: http://ipc.localhost` 直接管辖 IPC 来源；若它在真实 WebView 下挡住 IPC，十个命令一起失效。旧读数不得沿用。

#### 2. 日志 sink 真的有落点

**Test:** 在 `npm run tauri dev` 的终端里，把 base_url 设成一个非 loopback 的 `http://` 端点，观察终端输出。
**Expected:** 出现 tracing 格式的行，且 `settings.rs:85-88` 那条 `LLM endpoint uses plaintext http to a non-loopback host` 告警实际打出来。
**Why human:** `tracing::dispatcher::has_been_set()` 只证明 dispatcher 就位，不证明日志到达终端（EnvFilter 档位、fmt 层的输出目标都可能让它落空）。WR-04 点名的三条安全决策日志是否真有落点，取决于这条端到端确认。

### Gaps Summary

上轮的两条 blocker，一条**干净地关掉了**，一条**关掉了主体但留了一半**。

干净关掉的是 gap 1。我没看 SUMMARY 就先放了探针：上次那行被持久化的 `https://user:sk-…@host/v1` 现在返回 `write_ok=false / stored=None`，settings 表 0 行；连「只有用户名没有密码」「`?api-key=` 在 query 里」「fragment 里」「大写 scheme」四种变体一起拒。守卫长在 `set_setting` 的写入路径上而不是调用方，前端 `localUrlIssue` 与 engine 逐项对齐。六组断言里有阴性对照（干净 URL 仍能写入）和幂等对照（第二次仍拒），不是「一律拒绝」式的假修复。这是本轮做得最扎实的一件事。

留了一半的是 gap 2。主体确实关了——`sk-ant-api03-…` 现在命中，我把它注进 `src/lib/ipc.ts` 后 `scan` 当场退出 1；长度阈值补了成对边界样本；`docs/` 整目录排除取消；脚本自己进受检集合；selftest 进了 CI 与 justfile 两个调用点。这些我都亲手跑过，不是转述。

但关键词分支的引号是**必需**的，于是 `ANTHROPIC_API_KEY=…`、`mcp_bearer_token: …`、`password = …` 这类**未加引号**的赋值整类看不见。我把两行裸值密钥追加进 `.github/workflows/ci.yml`，扫描器照样打印 OK 并退出 0。这不是一个抽象的覆盖面问题：成功标准 4 写的是「代码与**配置**中无明文密钥」——源码里的密钥是带引号的字面量（抓得到），配置文件里的赋值常态是裸值（抓不到），漏掉的正好是成功标准点名的那一半；而 `mcp_bearer_token`——本项目自己的第二个密钥、一串没有任何供应商前缀的随机值——恰好只能靠关键词分支接住。

真正让我把它记成 blocker 而不是 warning 的是第三点：selftest 的阳性样本 3 是一条未加引号的 `ANTHROPIC_API_KEY=` 赋值，**读起来像是在证明裸值形态已覆盖**，实际是经 `sk-` 前缀分支命中的——把 `sk-` 去掉它就不再命中。14 条阳性样本里没有一条隔离关键词分支去打裸值。这与上轮 gap 2 是同一个失效形状，只是下沉了一层：一条看不见目标格式的正则照样退出 0，而现在还多了一个让人以为它看得见的样本。脚本文件头自己写着「正则的判别力从此是每次 CI 都重新证明一遍的断言」——对刚刚扩宽的那条分支，这句话不成立。

需要说清楚的是**范围**：这是证据缺口，不是现行泄漏。我不用项目自带的正则，另拟三条更宽的检索独立扫了一遍（关键词裸值、七种供应商前缀、追踪中的 `.env`/`.pem`/`.key` 文件），19 行命中逐行看过，全是 fixture、注释和普通标识符，第三条零输出。**当前仓库确实没有明文密钥。** 01-REVIEW.md 的 CR-01 用另一组检索得到同样结论。所以闭环计划只需要改 `scripts/check-secrets.sh` 一个文件：让引号可选、给裸值段配一个更高的长度下界、补两条只可能经关键词分支命中的阳性样本，顺带把 `github_pat_` / `xox[baprs]-` / `AIza` 三个前缀补上。不涉及任何产品代码。

Chesterton's Fence：引号前提不是执行期疏漏，01-11-PLAN.md 的 must-have 原文就写着「再接引号串」，执行完全忠实于计划，SUMMARY 里也检索不到任何关于裸值形态的权衡讨论。这是计划层把上轮 gap 的 `missing` 第一条（「`=` 与 `:` 两种赋值形态」）窄化成了引号形态。因此不建议用 override 吸收——它不是被想过之后接受的取舍。

01-12（空 bearer）与 01-13（CSP + subscriber）两条 CR 也都实测关闭：`McpDeps::new("")` 返回 `Err(McpError::EmptyBearer)`（含 `trim()`），比较层的 `constant_time_eq("", "")` 由 true 改判 false 且原先钉住 fail-open 的那行断言是被**反转**而非删除，端到端还多了一条「向合法门禁呈递空 token 应 403」；CSP 双份就位、`assetProtocol` 配置侧与 cargo feature 两半一起关，我把 `csp` 改回 `null` 后钉子测试当场变红；subscriber 落在壳里且由一条注入即红的新依赖断言看住。

两件事必须由人在真实 app 里做，不能自动判定，都已列进 Human Verification：CSP 只在真实 WebView 生效（这一项同时承载 SC-2 的复验——它的旧证据是在 `csp: null` 的环境里取得的，而新 CSP 的 `connect-src` 直接管辖 IPC 来源，一旦挡住就是十个命令一起失效）；以及「日志真的打到终端了吗」——`has_been_set()` 只证明 dispatcher 就位。

最后一件不阻断但会烂掉的记账：`REQUIREMENTS.md:148` 仍把 INFRA-03 映射到 Phase 1，而它的「支持 Anthropic/OpenAI 兼容端点」分句按各 SUMMARY 的判断顺延到 Phase 4（这个判断我核过，Phase 4 的 goal 确实明写 prism-llm 传输层先行，成立）。Phase 1 一关，这条需求就没有 phase 负责推进了。改映射或按 INFRA-04/05 的先例拆条加注，二选一。

同时提醒一句量的问题：上轮记录的 15 条 warning 里，本轮修掉的只有 WR-04，其余 12 条原封不动，本轮又新增 3 条（CSP 测试的 denylist、tracing 测试的全局态依赖、SQLite 版本下界无断言）。单条都不阻断，但都是「现在零成本、以后很贵」那一类。

---

_Verified: 2026-07-29T06:10:04Z（HEAD `b738038`）_
_Verifier: Claude (gsd-verifier)_
