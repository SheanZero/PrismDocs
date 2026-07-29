---
phase: 01-foundation-skeleton
verified: 2026-07-29T02:43:11Z
status: gaps_found
score: 3/4 must-haves verified
behavior_unverified: 0
overrides_applied: 0
gaps:
  - truth: "成功标准 4 第三分句：代码与配置中无明文密钥"
    status: failed
    reason: >-
      写入路径只防「疑似密钥的键名」，不防「携带凭据的值」。
      validate_base_url 检查 scheme 与 host 但不检查 userinfo，
      于是 https://user:sk-…@host/v1 通过校验并被 set_setting 原样写进
      SQLite settings 表——docs/keychain-naming.md 不变量 1 与 plan 01-04 的
      privacy prohibition 都明文规定那里绝不能有密钥，且理由正是「settings
      会被数据库单目录整体备份带走」。verifier 已实测复现（见下方证据）。
    artifacts:
      - path: "crates/prism-store/src/settings.rs"
        issue: "validate_base_url (48-71) 不检查 url.username() / url.password()；set_setting (88-104) 写入的是未经净化的原始字符串"
      - path: "src/pages/Settings.tsx"
        issue: "前端 looksLikeHttpUrl (17-20) 同样放行带 userinfo 的 URL，两道守卫都不挡"
      - path: "crates/prism-llm/src/secrets.rs"
        issue: "模块头声明的不变量 1（密钥绝不进 settings 表）在上述路径下不成立"
    missing:
      - "validate_base_url 中拒绝非空 username / 非 None password（错误文案保持 rule-shaped、不回显 value，符合 T-01-26）"
      - "同时考虑拒绝非空 query / fragment（部分网关用 ?api-key=… 形式）"
      - "settings_base_url_validation 增加带凭据 URL 的阴性对照"
      - "src/pages/Settings.tsx 的 looksLikeHttpUrl 同步收紧，并补一条前端测试"
  - truth: "成功标准 4 第三分句的执行机制：明文密钥静态扫描能真的看见明文密钥"
    status: failed
    reason: >-
      scripts/check-secrets.sh 退出 0 是本条成功标准的主要自动化证据，
      但其正则对本项目最可能泄漏的密钥格式完全失明。verifier 用 5 行真实形态的
      明文密钥测试该正则，只有 1 行被命中；四行 Anthropic 格式（sk-ant-api03-…）
      全部漏过——而 Anthropic Messages API 是 CLAUDE.md 点名的一等端点、
      Settings.tsx:149 的 placeholder 就是 https://api.anthropic.com。
      一次绿色扫描因此不足以支撑「无明文密钥」这条断言。
    artifacts:
      - path: "scripts/check-secrets.sh"
        issue: "PATTERN 要求 sk- 后紧跟 ≥16 个连续字母数字，sk-ant-… 在第一个连字符处即断开；仅匹配 api_key 形态的 = 赋值，apiKey: \"…\" / token = / Authorization: Bearer / .env 中的 ANTHROPIC_API_KEY= 全部漏过；docs/ 被整目录排除"
    missing:
      - "扩宽 PATTERN 覆盖 sk-[A-Za-z0-9_-]{20,}（含连字符）、(api[_-]?key|secret|token|password) 的 = 与 : 两种赋值形态、ghp_ / AKIA 前缀"
      - "把 ':(exclude)docs/' 收窄为排除具体引用了正则的那几篇文档"
      - "补一条 fixture 反证：一个 sk-ant-api03-… 形态的假串必须让脚本退出非 0（现有 FIXTURE_SECRET / FAKE_KEY 仍应保持不命中）"
deferred: []
---

# Phase 1: 基建骨架 Verification Report

**Phase Goal:** 可独立测试的 Rust engine workspace + Tauri 薄 shell 就绪，五项不可逆决策（单写者 SQLite + 读池、FTS5 CJK tokenizer、keyring-core 用法、prism-mcp trait 反转、notify-then-fetch）全部落地并各有验证通路
**Verified:** 2026-07-29T02:43:11Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | engine workspace 不依赖 tauri 即可 `cargo test` 全绿（D-01）；`cargo tree -d` 无重复 rusqlite/reqwest；prism-mcp 仅依赖注入的 service trait，编译期无 facade↔mcp 依赖环 | ✓ VERIFIED | 见 §SC-1 证据 |
| 2 | 事件总线骨架各验证一条通路：一条总线事件经粗粒度 Tauri event 往返前端（notify-then-fetch），一条命令经 Channel 有序流式返回（A1） | ✓ VERIFIED | 见 §SC-2 证据（含真实 WebView 人工验证） |
| 3 | SQLite schema v1 落地：WAL + 单写者 + r2d2 读池（query_only=ON）并发读写正常；FTS5 中文查询返回非零结果；rusqlite_migration 迁移体系可用，bundled SQLite ≥3.51.3 | ✓ VERIFIED | 见 §SC-3 证据 |
| 4 | API key 经 keyring-core + apple-native-keyring-store 写入系统钥匙串并可读回，prism-llm 为唯一网络出口与唯一密钥入口，**代码与配置中无明文密钥** | ✗ FAILED | 前两分句成立；第三分句实测被推翻——见 §SC-4 与 frontmatter `gaps` |

**Score:** 3/4 truths verified (0 present, behavior-unverified)

---

### SC-1 证据（engine 独立性 / 依赖图性质）

01-01 的 transparency prohibition 明确禁止拿 `cargo test --workspace` 冒充 D-01 的证据（它会编译 shell，恒真）。因此下列证据全部走 `-p` 选择集与 `cargo tree` 断言。

| 检查 | 命令 | 结果 |
|---|---|---|
| engine-only 测试全绿 | `cargo test -p prism-types -p prism-store -p prism-fs -p prism-parse -p prism-anchor -p prism-llm -p prism-mcp -p prism-engine` | **106 passed / 0 failed** |
| 九个 crate 全部 tauri-free | 对 8 engine crate + prism-cli 逐个 `cargo tree -p <c> --edges normal,build \| grep -c '^tauri'` | 每个都是 **0** |
| 无重复 SQLite / HTTP 栈 | `cargo tree --workspace --duplicates --edges normal \| grep -E '^(rusqlite\|reqwest\|libsqlite3-sys) v'` | **无命中**；树中唯一版本为 rusqlite 0.40.1 / reqwest 0.13.4 / libsqlite3-sys 0.38.1 |
| 无 facade↔mcp 环 | `cargo tree -p prism-mcp --edges normal` 及**含 dev 边**的默认形式 | prism-* 中只出现 prism-mcp 自身与 **prism-types**；dev 边同样干净 |
| 断言进 CI 而非口头约定 | `.github/workflows/ci.yml:31-41`（macos-latest）| `bash scripts/check-deps.sh all` + `check-secrets.sh` + clippy/test 均随 push 执行 |
| 全量参考 | `cargo test --workspace` / `cargo clippy --workspace --all-targets -- -D warnings` / `npx tsc --noEmit` | 121 passed（1 ignored）/ 0 / 0 |

**非恒真反证（verifier 亲手做的）：** 在 `crates/prism-store/Cargo.toml` 的 `[dependencies]` 注入一行 `reqwest = { workspace = true }` 后，`bash scripts/check-deps.sh single-egress` 打印 `FAIL: prism-store has network/secret dependency` 并退出 **1**；随后已还原（`git status` 干净）。第一次尝试把该行追加到文件末尾落进了 `[dev-dependencies]`，断言照常通过——这是 `--edges normal` 的正确行为（dev 依赖不进产物），一并记录以免后人误读。

### SC-2 证据（IPC 双通路）

这条成功标准是行为型的（事件往返的 1:1 语义、流的顺序不变量），presence 检查看不见。三层证据齐备：

| 层 | 证据 |
|---|---|
| 真实 WebView（人工） | 事件往返：计数 1:1、离开页面再回来不翻倍；Channel 流：「seq 校验通过 · 实收 1000 条」（本次验证前由用户在真实 app 完成） |
| 自动化 | `cargo test -p prismdocs-shell` **11 passed**（含 `map_recv` 的 Emit/Lagged→Resync/Closed→Stop 与事件名契约）；`cargo test -p prismdocs-shell --features test --test ipc` **2 passed**；`npm run test -- --run` **32 passed / 6 files**（含 `useEngineInvalidation` 的 cleanup 与 resync 分支） |
| 接线 | `bus_adapter.rs:18 EVENT_CHANGED="prism://changed"` → `useEngineInvalidation.ts:26 listen<EngineEvent>(EVENT_CHANGED, …)` → `invalidateQueries()`（resync 走无参全量失效，line 31）；`DevSmoke.tsx:47` 逐位断言 `ev.data.seq !== i` 而非「这些 seq 都出现过」 |

`--features test` 那条命令不可省：`src-tauri/tests/ipc.rs` 首行 `#![cfg(feature = "test")]`，不开 feature 时编译为 0 个测试且照样退出 0（本次 `cargo test --workspace` 的输出里 `Running tests/ipc.rs … 0 passed` 即是该形态）。CI 的 shell job（ci.yml:82-83）显式带了它。

### SC-3 证据（schema v1 / 并发纪律 / FTS CJK）

| 检查 | 命令 / 位置 | 结果 |
|---|---|---|
| 并发纪律六测 | `cargo test -p prism-store --test concurrency` | **6 passed**，函数名与计划逐一对应 |
| FTS CJK 四测 | `cargo test -p prism-store --test fts_cjk` | **4 passed** |
| 迁移体系 + settings | `cargo test -p prism-store`（unit） | **20 passed**（含 `migrations_are_valid`、`settings_*`） |
| tokenizer 定案 | `migrations/001_schema_v1.sql:41-47` | external-content FTS5，`content_rowid='rowid_pk'`，`tokenize = 'trigram'`，三触发器齐全 |
| query_only 注入点 | `open.rs:74-82` | `with_init` 中 `query_only=ON` 为最后一条 pragma |
| 迁移先于建池 | `open.rs:64` `to_latest` 早于 `open.rs:83` `Pool::builder()` | 另有 `migration_runs_before_the_read_pool_is_built` 源码序断言 |
| 运行期实测（verifier 临时探针） | `sqlite_version()` / `PRAGMA journal_mode` | **3.53.2**（≥3.51.3 ✓）/ **wal** ✓ — 探针文件已删除 |
| 真实 app（人工） | 冒烟页中文搜索 | 「锚定引擎」命中 >0，阴性对照「量子纠缠」= 0 |

**这组测试不是恒真的：** `fts_cjk` 对两条分支各配一条阴性对照（MATCH 分支「量子纠缠」→0、LIKE 回退分支「量子」→0），另有 `%` 通配符转义与 FTS5 语法注入两条；`pooled_connection_cannot_write` / `every_pooled_connection_is_query_only` 用 `expect_err`，实现被掏空即变红；`wal_truncated_on_close` 先断言 `-wal` 文件已增长（>0）再断言 close 后为 0——顺带证明 WAL 真的生效了。

### SC-4 证据（钥匙串 / 唯一出口 / 无明文密钥）

| 分句 | 状态 | 证据 |
|---|---|---|
| API key 经 keyring-core + apple-native-keyring-store 写入并可读回 | ✓ | `secrets.rs:38` `apple_native_keyring_store::keychain::Store::new()`（非 protected 模块，符合无 entitlement 的直发公证形态）；`cargo test -p prism-llm` 9 passed（mock store 往返、无 key → `Ok(None)`、幂等、Debug 脱敏、错误扁平化）；真实钥匙串往返由用户在真实 app 中完成（写入 + 读回状态 + 删除） |
| prism-llm 为唯一网络出口与唯一密钥入口 | ✓ | `check-deps.sh` 的 `single-egress` / `facade-egress` / `shell-egress` 三条全绿，且已用 reqwest 注入反证其非恒真；`grep -rnE "env::var\|std::env\|dotenv\|\.env"` 在 prism-llm / prism-engine / src-tauri 中**零命中**——不存在环境变量或 dotfile 降级路径；`src-tauri/Cargo.toml` 只依赖 prism-engine / prism-types / prism-store，不直连 prism-llm |
| **代码与配置中无明文密钥** | ✗ | 见下 |

**推翻第三分句的实测。** verifier 在 `crates/prism-store/tests/` 下临时放了一个探针测试（运行后已删除，`git status` 干净）：

```
PROBE validate_base_url -> true
PROBE set_setting -> true
PROBE stored value = Some("https://user:sk-verifier-probe-0000@api.vendor.com/v1")
PROBE plaintext secret persisted in sqlite = true
```

`is_secret_like_key("llm.base_url")` 为 false（标记只有 key/token/secret），`validate_base_url` 只看 scheme 与 host，于是整串（含 password）落进 `settings.value`。`docs/keychain-naming.md` 不变量 1 与 `prism-llm/src/secrets.rs` 模块头都写着「不进 SQLite（含 settings 表）」，理由正是 settings 会被整目录备份带走——这条路径把该理由变成了现实。plan 01-04 把这条 prohibition 标为 `status: resolved`，实测不成立。

**第二条证据：执行机制本身看不见目标格式。** 用 `scripts/check-secrets.sh` 的 PATTERN 对 5 行真实形态的明文密钥取样，只命中 1 行：

| 取样行 | 是否被扫描器看见 |
|---|---|
| `const k = "sk-ant-api03-AbCdEf…";` | ✗ |
| `const apiKey: "sk-openai-realkeyvaluehere1234";` | ✗ |
| `ANTHROPIC_API_KEY=sk-ant-api03-xyz…` | ✗ |
| `Authorization: "Bearer sk-ant-api03-abcdefghijklmnop"` | ✗ |
| `const k = "sk-AbCdEfGhIjKlMnOpQrStUvWxYz0123456789";` | ✓ |

`sk-` 后必须紧跟 ≥16 个**连续**字母数字，Anthropic 的 `sk-ant-api03-…` 在第三个字符处的连字符即断开——而 Anthropic 正是本项目点名的一等端点（`Settings.tsx:149` 的 placeholder 就是 `https://api.anthropic.com`）。

**Chesterton's Fence 检查：** 在 9 份 SUMMARY、01-VALIDATION.md 与 `docs/keychain-naming.md` 中检索 userinfo / 凭据 / `@host` 等表述，**零命中**——这不是一个被权衡后接受的取舍，是没被想到的口子。因此不建议用 override 吸收。

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `Cargo.toml` | workspace members + 版本 pin | ✓ VERIFIED | 单一 workspace，产物在仓库根 `target/`，无 `src-tauri/target/` |
| `crates/prism-store/src/open.rs` | writer-first 六步序 + 闭包式 write/read/close | ✓ VERIFIED | 182 行；`write` 用 `std::sync::Mutex`（guard `!Send`，跨 await 持锁是编译错误） |
| `crates/prism-store/src/migrations.rs` | LazyLock<Migrations> + `migrations_are_valid` | ✓ VERIFIED | `include_str!` 把 SQL 编进二进制；`validate()` 单测通过 |
| `crates/prism-store/migrations/001_schema_v1.sql` | projects/documents/documents_fts/settings + 三触发器 | ✓ VERIFIED | trigram、`content_rowid='rowid_pk'`、STRICT 表 |
| `crates/prism-store/src/search.rs` | 长度分流 + FTS5 转义 | ✓ VERIFIED | 两条分支各有阴性对照 |
| `crates/prism-store/src/settings.rs` | k/v + base_url 校验 + 密钥键名拒绝 | ⚠️ **INCOMPLETE** | 键名侧守卫成立；**值侧 userinfo 无守卫**（gap 1） |
| `crates/prism-types/src/service.rs` | 同步 object-safe service trait | ✓ VERIFIED | `cargo tree -p prism-types` 只有 serde/proc-macro 系；`contract.rs` 7 passed |
| `crates/prism-llm/src/secrets.rs` | SERVICE/ACCOUNT 常量 + init_default_store + set/get/delete + ApiKey | ✓ VERIFIED | 240 行；`ApiKey` 手写 Debug、刻意不实现 Display |
| `docs/keychain-naming.md` | 跨二进制命名契约 | ✓ VERIFIED | 含 `mcp_bearer_token`；不变量 1 与实现存在缺口（gap 1） |
| `crates/prism-mcp/src/{deps,middleware,server,handler}.rs` | 注入容器 + 三层中间件 + StreamableHttpService 挂载 | ✓ VERIFIED（有告警） | 10 + 10 + 3 测试全绿；空 bearer 兜底见 Warnings |
| `crates/prism-engine/src/{bus,facade,services}.rs` | 事件总线 + 门面 + trait 实现 | ✓ VERIFIED | unit 17 + `tests/facade.rs` 6 passed |
| `src-tauri/src/bus_adapter.rs` | broadcast → coarse event（含 Lagged→Resync） | ✓ VERIFIED | 纯函数 `map_recv` 三分支各有单测 |
| `src-tauri/src/commands.rs` | 全部 `#[tauri::command]`，单行委托 | ✓ VERIFIED（有告警） | `dev_smoke_stream` 是唯一不走 `delegate` 的命令（WR-08） |
| `src-tauri/tests/ipc.rs` | mock_builder 命令注册测试 | ✓ VERIFIED | `--features test` 下 2 passed；未开 feature 时 0 测试（已知代价，CI 已显式补齐） |
| `src/lib/useEngineInvalidation.ts` | listen → invalidate，含 cleanup 与 resync | ✓ VERIFIED | 两参 `.then(un=>un(), ()=>{})` 形式，listen 失败不落未处理 rejection |
| `src/pages/Settings.tsx` | API key 写钥匙串 + base_url（可跳过） | ⚠️ **INCOMPLETE** | `looksLikeHttpUrl` 同样放行 userinfo（gap 1）；读失败渲染为「未配置」（WR-06） |
| `src/pages/DevSmoke.tsx` | 三个验证入口 | ✓ VERIFIED | 逐位 seq 断言，非「集合包含」式弱断言 |
| `scripts/check-deps.sh` | 六条依赖方向断言 | ✓ VERIFIED（有告警） | 已反证非恒真；`check_dup` 的 `\|\| true` 见 WR-11 |
| `scripts/check-secrets.sh` | 明文密钥静态检查 | ✗ **INADEQUATE** | 对目标格式失明（gap 2） |
| `.github/workflows/ci.yml` | macOS runner 串起 per-wave 命令 | ✓ VERIFIED（有告警） | clippy 未覆盖 prism-cli / prismdocs-shell；无前端 linter（WR-16） |

### Key Link Verification

| From | To | Via | Status |
|---|---|---|---|
| `src/App.tsx` / `src/lib/ipc.ts` | `src-tauri/src/commands.rs` | `invoke('dev_ping')` 等 10 条命令封装 | ✓ WIRED |
| `src-tauri/src/commands.rs` | `crates/prism-engine/src/facade.rs` | `state.engine` 单行委托 | ✓ WIRED |
| `crates/prism-engine` | `crates/prism-store/src/open.rs` | `Arc<Store>`，写路径经 `store.write` 闭包 | ✓ WIRED |
| `crates/prism-engine` | `crates/prism-llm/src/secrets.rs` | 密钥读写唯一经 `prism_llm::secrets` | ✓ WIRED |
| `crates/prism-engine/src/services.rs` | `prism_types::FeedbackSource` | `impl FeedbackSource for Engine` | ✓ WIRED |
| `crates/prism-mcp/src/server.rs` | `rmcp::…::StreamableHttpService` | `nest_service` 挂载 + `from_fn` 三层中间件 | ✓ WIRED（仅测试消费） |
| `src-tauri/src/bus_adapter.rs` | 前端 `listen('prism://changed')` | `AppHandle::emit` | ✓ WIRED |
| `crates/prism-store/src/search.rs` | `001_schema_v1.sql` | `JOIN … ON documents.rowid_pk = documents_fts.rowid` | ✓ WIRED |
| `crates/prism-store/src/settings.rs` | `url::Url` | `Url::parse` 后限定 scheme | ⚠️ PARTIAL — scheme/host 已限，userinfo 未限 |
| `prism-mcp` | `prism-engine` | （必须不存在） | ✓ 不存在，normal 与 dev 边均无 |
| `prismdocs-shell` | `prism-mcp` / `prism-llm` | （必须不直连） | ✓ 不存在——MCP server 在 Phase 1 无运行期消费方，仅测试拉起 |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|---|---|---|---|
| engine 脱离 tauri 可测 | `cargo test -p prism-types … -p prism-engine` | 106 passed / 0 failed | ✓ PASS |
| 无重复关键 crate | `cargo tree --workspace --duplicates --edges normal \| grep -E '^(rusqlite\|reqwest\|libsqlite3-sys) v'` | 无命中 | ✓ PASS |
| 无 facade↔mcp 环 | `cargo tree -p prism-mcp`（含 dev 边） | 仅 prism-types | ✓ PASS |
| 依赖断言非恒真 | 注入 reqwest 到 prism-store `[dependencies]` → `check-deps.sh single-egress` | exit 1, `FAIL: prism-store has network/secret dependency` | ✓ PASS |
| bundled SQLite / WAL | 临时探针：`sqlite_version()` / `PRAGMA journal_mode` | `3.53.2` / `wal` | ✓ PASS |
| FTS CJK 判别性 | `cargo test -p prism-store --test fts_cjk` | 4 passed（含 2 条分支各自的阴性对照） | ✓ PASS |
| IPC 命令可达 | `cargo test -p prismdocs-shell --features test --test ipc` | 2 passed | ✓ PASS |
| 前端 | `npm run test -- --run` / `npx tsc --noEmit` / `npm run build` | 32 passed / 0 / 0 | ✓ PASS |
| 全量闸门 | `cargo test --workspace` / `cargo clippy --workspace --all-targets -- -D warnings` | 121 passed（1 ignored）/ 0 | ✓ PASS |
| 密钥可落进 settings 表 | 临时探针：`set_setting("llm.base_url", "https://user:sk-…@host/v1")` | 明文凭据被持久化 | ✗ **FAIL** |
| 密钥扫描器可见性 | 5 行真实形态密钥过 `check-secrets.sh` 的 PATTERN | 命中 1 / 5 | ✗ **FAIL** |

### Probe Execution

本仓库无 `scripts/*/tests/probe-*.sh` 约定探针；PLAN / SUMMARY 亦未声明任何 probe。Step 7c 以「无 probe 可跑」记录，替代证据为上表的 behavioral spot-checks。

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|---|---|---|---|---|
| INFRA-01 | 01-01, 01-02, 01-04, 01-06, 01-07, 01-08, 01-09 | Rust engine workspace（不依赖 tauri、可独立测试）+ Tauri 薄 shell + 事件总线骨架；prism-mcp 经 service trait 反转解依赖环 | ✓ SATISFIED | SC-1 + SC-2 全部证据 |
| INFRA-02 | 01-03, 01-05, 01-07, 01-09 | SQLite WAL 单写者 + r2d2 读池；FTS5 CJK tokenizer 定于 schema v1；rusqlite_migration；bundled SQLite ≥3.51.3 | ✓ SATISFIED | SC-3 全部证据（实测 3.53.2 / wal / trigram） |
| INFRA-03 | 01-02, 01-04, 01-05, 01-07, 01-09 | API key 存系统钥匙串；自定义 base_url；prism-llm 为唯一网络出口与唯一密钥入口 | ✗ **BLOCKED** | 钥匙串与唯一出口成立；「密钥不进 settings 表」这条同源不变量被 gap 1 推翻，base_url 校验面是同一段代码 |

**Orphan 检查：** `.planning/REQUIREMENTS.md` 的 Traceability 表把且仅把 INFRA-01 / INFRA-02 / INFRA-03 映射到 Phase 1，三条全部被 plan frontmatter 认领——**无 orphaned requirement**。表中 INFRA-01/02 已预标 Complete、INFRA-03 仍为 Pending，与本报告结论一致。

### Anti-Patterns Found

`TBD` / `FIXME` / `XXX` / `TODO` / `HACK` / `PLACEHOLDER` 在 `crates/`、`src/`、`src-tauri/`、`scripts/` 下**零命中**——无未挂钩的技术债标记。

下表是本次验证独立确认的问题（与 01-REVIEW.md 交叉核对；每条我都自己看过代码或跑过命令，不是转述）：

| File | Line | Pattern | Severity | Impact |
|---|---|---|---|---|
| `crates/prism-store/src/settings.rs` | 48-71, 88-104 | 值侧守卫缺失（userinfo） | 🛑 Blocker | gap 1 — 明文凭据进 sidecar 库 |
| `scripts/check-secrets.sh` | 18-23 | 扫描器对目标格式失明 | 🛑 Blocker | gap 2 — 成功标准 4 的主要自动化证据失效 |
| `src-tauri/tauri.conf.json` | 20-26 | `"csp": null` + `assetProtocol.enable: true`（scope 空、无消费方） | ⚠️ Warning | CR-02。`bundle.active: true` / `targets: "dmg"` 已开，而 Phase 3+ 要渲染外部 agent 写的 Markdown、Phase 6 引入 LLM——现在设策略成本为零 |
| `crates/prism-mcp/src/deps.rs` · `middleware.rs` | 30-40 · 143-162, 208 | 空 bearer fail-open，且被单测钉为「预期行为」 | ⚠️ Warning | CR-03。Phase 1 无运行期消费方（shell 不依赖 prism-mcp），故未破 SC；Phase 6 从钥匙串读 token 时空值可信路径成立，届时修复成本上升 |
| workspace 全域 | — | 无 `tracing-subscriber`（`grep` 零命中） | ⚠️ Warning | WR-04。三条安全决策（T-01-29 的 uniform-403 真实原因、T-01-33 的 agent 回执审计、settings 的明文 http 告警）与「钥匙串不可用」启动降级提示全部写向空 sink |
| `crates/prism-store/tests/concurrency.rs` | 43-68 | `assert!(after >= 1)` 恒真；注释声称的快照隔离 `Store::read` 并未实现 | ⚠️ Warning | WR-01。该测试**仍非恒真**——`.expect("writer is not blocked by an open reader")` 与 `assert_eq!(total, 2)` 是真承重断言，SC-3 因此仍成立；但名字/注释/断言三者不一致 |
| `crates/prism-store/src/open.rs` | 53-59 | `PRAGMA journal_mode=WAL` 的返回行被丢弃 | ⚠️ Warning | WR-02。本机实测确为 `wal`，但网络文件系统 / 只读目录下会静默退化为 rollback journal，到 Phase 2+ 表现为偶发 SQLITE_BUSY |
| `crates/prism-store/src/open.rs` | 147-153 | `close()` 丢弃 checkpoint 的 busy 标志 | ⚠️ Warning | WR-03。文档注释精准描述了这个失败模式，代码随即用 `\|_\| Ok(())` 把它扔掉 |
| `crates/prism-mcp/tests/middleware_gate.rs` | 207-217 | 阳性对照断言 `!is_client_error()` | ⚠️ Warning | WR-14。500 也算通过——阳性对照恰恰是知道成功长什么样的那一个 |
| `src/pages/Settings.tsx` + `Settings.test.tsx` | 100-102, 142 | 读失败渲染为「未配置」；测试无 `mockRejectedValue` 覆盖 read 路径 | ⚠️ Warning | WR-06。与 WR-04 叠加后，钥匙串被锁对用户完全静默 |
| `src/lib/ipc.ts` | 41-68 | `ERROR_COPY[code] ?? …` 会解析原型链成员 | ⚠️ Warning | WR-05。`errorCopy("toString")` 返回函数（类型标注为 string），流进 JSX 会被 React 拒绝；目前是潜伏态 |
| `src-tauri/src/lib.rs` | 48-59 | 四个 `dev_*` 命令无条件注册进 release IPC 面 | ⚠️ Warning | WR-07。`dev_seed_sample_docs` 会往用户真实库写 `smoke-project` 夹带数据；前端只藏了页面没藏命令 |
| `src-tauri/src/commands.rs` | 131-137 | `total: u32` 未夹紧且不走 `spawn_blocking` | ⚠️ Warning | WR-08。全文件唯一不遵守本模块自述纪律的命令 |
| `scripts/check-deps.sh` | 34-43 | `cargo tree … \|\| true` 吞掉失败后 grep 空串即报 OK | ⚠️ Warning | WR-11。同一文件另五条检查都让 `set -e` 直接炸；本次已由 verifier 独立跑 `cargo tree` 确认当前树真的干净 |
| `crates/prism-mcp/src/handler.rs` | 36-53, 85-91 | schema 声明 `required: ["projectId"]`，handler 用 `unwrap_or_default()` | ⚠️ Warning | WR-12。今日靠 `Engine::list_feedback` 兜底；未来某个把 `""` 当「全部 project」的实现即成跨项目泄漏 |
| `crates/prism-engine/src/services.rs` | 53-63 | agent 提供的 `status` 未约束即原样入日志 | ⚠️ Warning | WR-13。注释写明「只记 comment_id 与 status」的理由，恰好没推广到 status 自身 |
| `.github/workflows/ci.yml` | 37-38, 85-101 | clippy 不覆盖 prism-cli / prismdocs-shell；无 ESLint | ⚠️ Warning | WR-16。承载 IPC 边界与未来 externalBin 的两个 crate 是全仓唯一无 lint 闸门的 Rust |
| `src/lib/capabilities.test.ts` | 26-32 | 「最小权限」断言实为六前缀 denylist | ⚠️ Warning | WR-09。`core:event:allow-emit` 之类可静默加入——前端脚本据此可伪造 `prism://changed` |
| `crates/prism-mcp/src/middleware.rs` | 143-162 | `constant_time_eq` 的 XOR 折叠在 `same_len` 之后不可达 | ℹ️ Info | WR-15。行为正确（已追过），但认证比较里的手写复杂度是未来引 bug 的地方 |
| `src-tauri/tests/ipc.rs` · `crates/prism-store/src/seed.rs` · `open.rs` · `middleware.rs` · `service.rs` · `DevSmoke.tsx` | — | 注释计数过时 / 返回常量而非行数 / 版本串解析静默重排 / h2 `:authority` / `ServiceError::Backend` 无构造点 / `role="status"` 承载错误 | ℹ️ Info | IN-01 … IN-06，均已复核为真 |

### Human Verification Required

无新增待办。本次验证前，用户已在真实 app 中跑完 01-09 Task 3 的全部六项（总线事件往返 1:1、Channel 1000 条 seq 校验、中文搜索命中与阴性对照、真实钥匙串写入/读回/删除、`file:///etc/passwd` 被中文文案拒绝、无 key 时应用照常启动可用）。

关于 `#[ignore]` 的 `roundtrip_with_real_keychain`（`crates/prism-llm/src/secrets.rs:226`）：**视为已覆盖，不再单列。** 该测试与人工验证第 4 项走的是同一段代码——UI 路径经 `Engine::init_secrets()` → `secrets::init_default_store()` 注册 `apple_native_keyring_store::keychain::Store`，再经同样的 `set_api_key` / `get_api_key` / `delete_api_key`，且是在真实 macOS 登录钥匙串上完成的。保留 `#[ignore]` 是对的（它会弹授权框，不该进 CI）。

需要留意的是：用户对第 2–6 项给的是通过判定，只有第 1 项报了具体读数（「计数正常跳动，1:1」）。这不构成重验理由，但记录在案。

### Gaps Summary

四条成功标准里三条硬扎实——依赖图性质（SC-1）、IPC 双通路（SC-2）、SQLite/FTS 骨架（SC-3）都有能被掏空即变红的证据，而且我自己动手做了非恒真反证：注入一条 reqwest 依赖，依赖方向断言当场变红。这个 phase 反复自查「测试是否恒真」的做法是有效的，`fts_cjk` 给两条分支各配阴性对照那一手尤其到位。

失守的是成功标准 4 的最后一个分句，而且是两处独立的失守指向同一件事：**密钥容器的边界建在了键名上，没建在值上，而看守这条边界的扫描器又看不见本项目最可能出现的那种密钥。**

前一处是机制缺口。`is_secret_like_key` 被刻意写成「宽进严出」来防键名，`validate_base_url` 却只管 scheme 和 host——于是 `https://user:sk-…@host/v1` 这种若干 OpenAI 兼容网关明文写在文档里的形式，会把整串凭据写进 `settings.value`。而 `docs/keychain-naming.md` 给出的不变量理由是「settings 会被数据库单目录整体备份带走」，那正是这条路径的后果。前端 `looksLikeHttpUrl` 是同样的形状，两道守卫在同一处一起漏。我跑探针复现了，不是推演。

后一处是证据缺口。这条成功标准的自动化证据就是 `check-secrets.sh` 退出 0，而它对 `sk-ant-api03-…` 完全失明——`sk-` 后要求 ≥16 个连续字母数字，Anthropic 的格式在第三个字符处就断了。CLAUDE.md 点名 Anthropic Messages API 是一等端点，Settings 页的 placeholder 就写着 `api.anthropic.com`。也就是说：最可能被泄漏的那个供应商的密钥，恰好是扫描器唯一看不见的。绿色扫描在这里等于没扫。

两处的修法都很小（一个 `if` + 一次正则扩宽 + 各一条阴性对照），但不该拖到 Phase 4 真正发请求时才补——那时 `settings` 表里可能已经躺着凭据了。

不阻断但值得在进 Phase 2 前一并处理的三件事，因为它们都属于「现在零成本、以后很贵」：`csp: null` 配着已开启的 dmg 打包（CR-02）；空 bearer fail-open 且被单测钉成预期行为（CR-03，Phase 6 从钥匙串读 token 时正是空值可信的场景）；全 workspace 没有 `tracing-subscriber`，于是三条安全决策所依赖的日志、以及「钥匙串不可用」这条启动降级提示，全都写向一个不存在的 sink（WR-04）——它和 WR-06「读失败渲染为未配置」叠在一起，会让钥匙串被锁这件事对用户完全静默。

---

_Verified: 2026-07-29T02:43:11Z_
_Verifier: Claude (gsd-verifier)_
