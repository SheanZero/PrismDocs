---
phase: 01-foundation-skeleton
verified: 2026-07-30T00:34:27Z
status: human_needed
score: 3/4 must-haves verified
behavior_unverified: 1
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: 3/4
  previous_verified: 2026-07-29T06:10:04Z
  gaps_closed:
    - "上轮唯一 gap：`check-secrets.sh` 关键词分支要求值带引号，未加引号的赋值整类不可见。verifier 独立复现上轮那次注入实验（往 `.github/workflows/ci.yml` 追加两行裸值密钥）—— 上轮 scan 退出 0，本轮 **退出 1 并点名两行**。上轮 8 行取样表现在 7/8 命中（第 8 行 `password = hunter2hunter2` 值仅 14 字符，落在裸值下界 16 之下，脚本第 94-100 行把它写成有理由的已知残留）"
  gaps_remaining: []
  regressions: []
  label_changes:
    - "成功标准 2 由上轮的「✓ VERIFIED（有告警）」改判为「⚠️ PRESENT_BEHAVIOR_UNVERIFIED」。**证据状态未变**（上轮同样记录了真实 WebView 证据因 01-13 加 CSP 而过期），改的是标签口径：一条断言真实 WebView 往返的行为型 truth，在其唯一端到端证据失效时不应计入 verified 分子。这不是回退，是把上轮已写在正文里的告警提到状态位上"
  warnings_closed_this_round:
    - "`concurrency.rs` 的恒真断言 `assert!(after >= 1)` → 现为 `assert_eq!(after, 2)`（01-19）"
    - "SQLite 版本下界无断言 → `MIN_SQLITE=(3,51,3)` 现由 `open.rs:125-127` 在准入路径上强制（01-18/01-19）"
    - "`check-deps.sh` 的 `cargo tree … || true` 吞掉调用失败 → 现先接住退出码再判输出（01-15）"
    - "CSP / capability 断言由 denylist 改精确相等（01-24）"
    - "前端无 lint 闸门 → ESLint flat config + `npm run lint` 接进 CI（01-26/01-27）"
    - "无 fmt 闸门 → `rustfmt.toml` + CI engine job 首步 `cargo fmt --all -- --check`（01-28）"
    - "INFRA-03 映射孤儿风险 → 已改映射 Phase 4 并加四点表下注（01-25）"
deferred:
  - truth: "INFRA-03 的「支持 Anthropic/OpenAI 兼容端点」分句（真正向端点发出请求）"
    addressed_in: "Phase 4"
    evidence: "`.planning/REQUIREMENTS.md:170` 已把 INFRA-03 映射到 Phase 4，表下注第 3 点写明依据是 ROADMAP Phase 4 goal「A3：prism-llm 传输层（流式/重试/keyring）先行交付」。本轮已核：这条不再是 Phase 1 的未闭合项，而是已完成记账迁移的跨切需求"
behavior_unverified_items:
  - truth: "成功标准 2：一条总线事件经粗粒度 Tauri event 往返前端（notify-then-fetch），一条命令经 Channel 有序流式返回"
    test: "真实 WebView 下跑 dev 冒烟页的两个入口：① 点「发一条总线事件」，事件计数与点击次数 1:1（离开页面再回来再点，不翻倍）；② 点「Channel 有序流」，读数为「seq 校验通过 · 实收 1000 条」"
    expected: "两个读数都与 01-09 那次人工验证一致。若 IPC 被 CSP 的 `connect-src 'self' ipc: http://ipc.localhost` 挡住，受影响的不是某一个命令而是全部十个——两条通路会同时断"
    why_human: "两条通路的自动化层都结构性看不见真实往返：`cargo test` 走 `mock_builder`（无 WebView），vitest 走 jsdom 且 `@tauri-apps/api/event` 被 mock。`src-tauri/tests/ipc.rs:126-131` 对 `dev_smoke_stream` 只断言 `res.is_ok()`，不校验 1000 条的到达与顺序；顺序断言在 `smoke.rs` 的**生成器**单测里，不覆盖 Channel 传输。唯一的端到端证据取自 01-09，那时 `csp` 还是 `null`——环境已变更，旧读数不得沿用"
human_verification:
  - test: "真实 WebView 下的 CSP 与 IPC 双通路（同时承载成功标准 2 的复验）。六步：① `npm run tauri dev` 起应用，窗口不是白屏；② 设置页可用，保存一个合法端点（如 `https://api.anthropic.com`）出现成功文案；③ dev 冒烟页跑三个入口（总线事件 1:1 计数 / Channel「seq 校验通过 · 实收 1000 条」/ 中文搜索「锚定引擎」>0 且「量子纠缠」=0）；④ WebView 控制台无任何 CSP 违规报告，特别留意 01-24 新加的 `form-action 'none'` 与 `frame-ancestors 'none'`；⑤ `npm run tauri build` 出 dmg，安装后对 ①–④ 重复一遍（发布形态走 `csp` 而非 `devCsp`，这是验证严格那一份的唯一路径）；⑥ 发布形态额外确认冒烟页开关不存在且四个 `dev_*` 命令不可调用"
    expected: "六步全部正常，三个入口读数与 01-09 一致。第 ④ 步的两条新指令预期不触发：Phase 1 前端不含原生 `<form>` 提交，桌面窗口也不会被嵌套"
    why_human: "CSP 只在真实 WebView 里生效——`src/lib/tauri-security.test.ts:17` 自己写明 jsdom 看不见它，`cargo test` 走 `mock_builder` 也无 WebView。出现违规时的处理：只放宽 `devCsp`，或按控制台点名的指令逐项追加到 `csp`；**禁止**把 `csp` 设回 `null`，也禁止直接删掉 01-24 新加的两条指令——若确需放宽，先在 `tauri-security.test.ts` 的精确相等断言上过一次评审（WINDOWS id=8）"
  - test: "日志 sink 真的有落点。两步：① 默认档位（不设 `RUST_LOG`）`npm run tauri dev`，在设置页把 base_url 设成一个非 loopback 的 `http://` 端点（如 `http://example.com/v1`）并保存，观察终端；② `RUST_LOG=trace npm run tauri dev`，观察终端"
    expected: "① 出现 tracing 格式的行，且 `crates/prism-store/src/settings.rs:88-91` 那条 `LLM endpoint uses plaintext http to a non-loopback host` 实际打出来（默认 info 档下就有落点，无需提档）；② 出现 01-21 的降档 warn，正文以 `the environment-supplied log filter exceeds the project ceiling` 开头并说明 `rmcp` 被 capped at INFO，且正文中**不含** `RUST_LOG` 的原值"
    why_human: "`tracing::dispatcher::has_been_set()` 只证明 dispatcher 就位，不证明日志到达终端（EnvFilter 档位与 fmt 层的输出目标都可能让它落空）。三条安全决策日志是否真有落点取决于这条端到端确认。步骤在 01-21 之后必须走默认档位：该 plan 给 env filter 加了项目天花板（`src-tauri/src/lib.rs:51` `LOG_CEILING_DIRECTIVE = \"rmcp=info\"`），原先「用 `RUST_LOG` 提档观察」既观察不到目标、也不再是 sink 有落点的证据（WINDOWS id=9）"
  - test: "CI workflow 的首次真实 GitHub Actions 运行。推分支后核对：① engine job 首步 `Format check (rustfmt default style)` 出现在步骤列表最前且为绿；② `permissions: contents: read` 下 `upload-artifact` 仍可上传；③ `concurrency` 真的收掉同 commit 的双跑；④ 两个缓存分段互不恢复"
    expected: "四项全部成立。判别力可在一个丢弃分支上注入劣化排版验证 fmt 步骤会变红并点名文件"
    why_human: "`origin/main` 停在 `4cc1347`，Phase 1 全部 28 份 plan 的产物均未推送，`gh run list` 返回 `[]`——该 workflow 至今未在 GitHub Actions 上跑过。所有 CI 闸门声称在本 phase 里都只有本机证据（WINDOWS id=14 / id=15，同一次运行可一并核对）"
---

# Phase 1: 基建骨架 Verification Report（第三轮复验）

**Phase Goal:** 可独立测试的 Rust engine workspace + Tauri 薄 shell 就绪，五项不可逆决策（单写者 SQLite + 读池、FTS5 CJK tokenizer、keyring-core 用法、prism-mcp trait 反转、notify-then-fetch）全部落地并各有验证通路
**Verified:** 2026-07-30T00:34:27Z（HEAD `270d1a2`）
**Status:** human_needed
**Re-verification:** Yes —— 复验 01-14…01-28 共 15 份 gap-closure 计划

## 与上一份报告相比，变了什么

上一份（2026-07-29T06:10:04Z）的结论是 `gaps_found`，只挂着**一条** gap：`check-secrets.sh` 的关键词分支要求值带引号，于是「配置文件里的裸值赋值」整类不可见。

**这条 gap 已关闭，且是我自己动手证的，不是读 SUMMARY 得来的：**

| 上轮的实验 | 上轮结果 | 本轮结果 |
|---|---|---|
| 往 `.github/workflows/ci.yml` 追加 `# mcp_bearer_token: <32hex>` 与 `# ANTHROPIC_AUTH_TOKEN=<32hex>`，跑 `check-secrets.sh scan` | `OK: …`，exit **0** | `FAIL: possible plaintext secret …` + 逐行打出，exit **1**（已 `git checkout` 还原） |
| 上轮 §SC-4 那张 8 行取样表 | 8/8 全 MISSED | **7/8 CAUGHT** |

第 8 行 `password = hunter2hunter2` 仍 MISSED——值只有 14 个字符，落在裸值下界 16 之下。这不是遗漏：脚本第 94-100 行专门写了这条已知残留，并把「裸值下界必须严格高于引号下界」的理由说清楚了（取值为表达式的赋值形如 `self.inner.value` 长度就在 12–20 之间，下界一降它们整片涌进来，误报会让闸门被人绕开）。它是弱人类口令而非 API key，不在本闸门要守的密钥面上。

**除此之外本轮还有一处标签改判**（不是回退，见下文 §SC-2）：成功标准 2 由「✓ VERIFIED（有告警）」改为「⚠️ PRESENT_BEHAVIOR_UNVERIFIED」。上一份报告的正文里已经写了同一件事（「真实 WebView 那一层的旧证据因 01-13 加 CSP 而**过期**」），只是把它留在了告警里而没有反映到状态位上。证据状态一字未变，改的是口径。

---

## Goal Achievement

### Observable Truths（ROADMAP Success Criteria）

| # | Truth | Status | Evidence |
|---|---|---|---|
| 1 | engine workspace 不依赖 tauri 即可 `cargo test` 全绿（D-01）；`cargo tree -d` 无重复 rusqlite/reqwest；prism-mcp 仅依赖注入的 service trait，编译期无 facade↔mcp 依赖环 | ✓ VERIFIED | 见 §SC-1（含 no-cycle 断言的注入式反证） |
| 2 | 事件总线骨架各验证一条通路：一条总线事件经粗粒度 Tauri event 往返前端（notify-then-fetch），一条命令经 Channel 有序流式返回（A1） | ⚠️ PRESENT_BEHAVIOR_UNVERIFIED | 全部接线与自动化层在位且非恒真；但两条通路断言的是**真实往返**，其唯一端到端证据取自 `csp: null` 的旧环境。见 §SC-2 |
| 3 | SQLite schema v1 落地：WAL + 单写者 + r2d2 读池（query_only=ON）并发读写正常；FTS5 中文查询返回非零结果；rusqlite_migration 迁移体系可用，bundled SQLite ≥3.51.3 | ✓ VERIFIED | verifier 自建运行期探针实测全部读数，见 §SC-3 |
| 4 | API key 经 keyring-core + apple-native-keyring-store 写入系统钥匙串并可读回，prism-llm 为唯一网络出口与唯一密钥入口，代码与配置中无明文密钥 | ✓ VERIFIED（有告警） | 三分句各自独立取证；扫描器覆盖面留一处已具名残留（WR-04），见 §SC-4 |

**Score:** 3/4 truths verified（1 present, behavior-unverified）

---

### SC-1 证据（engine 独立性 / 依赖图性质 / trait 反转）

沿用前两轮的纪律：**不拿 `cargo test --workspace` 冒充 D-01 的证据**——它会编译 shell，对「engine 不依赖 tauri」这件事恒真。

| 检查 | 命令 | 结果 |
|---|---|---|
| engine-only 测试全绿 | `cargo test -p prism-types -p prism-store -p prism-fs -p prism-parse -p prism-anchor -p prism-llm -p prism-mcp -p prism-engine` | **128 passed / 0 failed / 1 ignored**（上轮 110） |
| 唯一的 ignored | 同上，`--nocapture` 列名 | `secrets::tests::roundtrip_with_real_keychain` —— 刻意 `#[ignore]`，见 §SC-4 对它的处理 |
| 七条依赖方向断言 | `bash scripts/check-deps.sh all` | dup / tauri-free / no-cycle / single-egress / facade-egress / shell-egress / subscriber-free 全 OK，exit 0 |
| `cargo tree -d` | `cargo tree --workspace --duplicates --edges normal` | 无 `rusqlite` / `reqwest` / `libsqlite3-sys` 多版本（仅 `base64` 双版本，经 tauri 的 `swift-rs` 引入，不在 SC-1 点名的三个包内） |
| prism-mcp 的 prism-* 依赖面 | `cargo tree -p prism-mcp --edges normal --prefix none \| tail -n +2 \| grep '^prism-'` | **只有一行**：`prism-types v0.1.0`。`reqwest` 只在 `[dev-dependencies]`（`Cargo.toml:29`） |
| trait 反转的落点 | `crates/prism-mcp/src/deps.rs:26-29` | `feedback: Arc<dyn FeedbackSource>` / `comments: Arc<dyn CommentSink>`，两个 trait 都来自 `prism_types`；文件头写明为何是 `Arc<dyn …>` 而非泛型（泛型会把 prism-engine 的具体类型经 `S` 泄漏进公开签名） |

**no-cycle 断言的非恒真反证（verifier 亲手做的）。** 把 `check_no_cycle` 的 grep 目标从 `^prism-engine ` 换成 `^prism-types `（一个确实在树里的包），`bash scripts/check-deps.sh no-cycle` 当场 `FAIL` 并 exit 1；改回后 exit 0。这证明该断言读的是真实的树内容，而不是在一个空串上恒绿。`scripts/check-deps.sh` 已还原，`git diff` 干净。

---

### SC-2 证据（IPC 双通路）—— 为什么是 PRESENT_BEHAVIOR_UNVERIFIED

**在位且非恒真的部分（逐项自己看过）：**

| 层 | 证据 | 状态 |
|---|---|---|
| 接线（通路 A） | `bus_adapter.rs:18 EVENT_CHANGED="prism://changed"` → `:57 BusOutcome::Emit(ev) => emit(&app, ev)` → `:66 app.emit(EVENT_CHANGED, ev)` → `useEngineInvalidation.ts:26 listen<EngineEvent>(EVENT_CHANGED, …)` → `:36/:39 queryClient.invalidateQueries()` | ✓ WIRED |
| 接线（通路 B） | `commands.rs:142 on_event: tauri::ipc::Channel<SmokeEvent>` → `:145 spawn_blocking(… smoke::generate(total, \|ev\| on_event.send(ev)))` → `DevSmoke.tsx:51 if (ev.data.seq !== i) …` 逐位校验 | ✓ WIRED |
| 自动化（壳） | `cargo test -p prismdocs-shell --features test` → **23 passed**（21 + 2） | ✓ 回归通过 |
| 自动化（前端） | `npm run test -- --run` → **75 passed / 7 files**（上轮 34） | ✓ 回归通过 |
| 生成器顺序 | `smoke.rs:135 smoke_stream_seq_is_strictly_monotonic` —— `assert_eq!(seqs, (0..total).collect())` 加相邻对差 1，注释写明为何不能 `sort()` 后比较 | ✓ 判别力真实 |

**为什么仍不能计入 verified。** 成功标准 2 断言的是**往返**与**有序到达**——两者都是运行期行为，而两条自动化路径在结构上都看不见它：

- `cargo test` 走 `tauri::test::mock_builder`，**没有 WebView**。`src-tauri/tests/ipc.rs:126-131` 对 `dev_smoke_stream` 的断言是 `assert!(res.is_ok())`——它证明命令注册且委托走通，**不证明 1000 条 Tick 到达前端、更不证明有序**。顺序断言落在 `smoke.rs` 的**生成器**上（`generate` 喂给一个本地闭包），Channel 传输那一段没有任何自动化覆盖。
- vitest 走 jsdom，且 `@tauri-apps/api/event` 整个被 `vi.mock` 掉（`useEngineInvalidation.test.ts:10-12`）——`listen` 是一个 spy，没有真实事件系统。

唯一的端到端证据是 01-09 之后用户在真实 app 里跑冒烟页得到的读数（计数 1:1、「seq 校验通过 · 实收 1000 条」）。**那次是在 `csp: null` 的环境里取得的**，而 01-13 之后 `tauri.conf.json` 的 `connect-src 'self' ipc: http://ipc.localhost` 直接管辖 IPC 来源，01-24 又追加了 `form-action` / `frame-ancestors`。旧读数所处的环境已不存在。若这条策略在真实 WebView 下挡住 IPC，断的不是某一个命令而是全部十个——两条通路一起失效，而上面所有绿色一个都不会变红。

因此：**接线在、判别力在、行为未证**。按 verifier 的行为型 truth 规则，它既不是 FAILED（代码在位且接线正确），也不能算 VERIFIED（承重的行为无测试覆盖）→ 归入人工验证第 1 项，不计入分子。

---

### SC-3 证据（schema v1 / 并发纪律 / FTS CJK）

不采信任何 SUMMARY。verifier 在 `crates/prism-store/tests/` 下临时建了一个探针（跑完即删，`git status` 已确认干净），直接打运行期：

```
PROBE sqlite_version           = 3.53.2
PROBE MIN_SQLITE               = (3, 51, 3)
PROBE journal_mode(read pool)  = wal
PROBE query_only(read pool)    = 1
PROBE write-through-read-pool  = REJECTED: sqlite error: attempt to write a readonly database
PROBE documents_fts exists     = 1
PROBE seeded rows              = 3
PROBE search(锚定引擎)          = 1
PROBE search(量子纠缠)          = 0     ← 阴性对照
PROBE writer txn               = Ok(())
PROBE user_version             = 1
```

逐条对上成功标准 3：

| 分句 | 证据 |
|---|---|
| WAL | 读池连接上 `PRAGMA journal_mode` 返回 `wal` |
| 单写者 + 读池 `query_only=ON` | `query_only` 读到 `1`，且经读池执行 `CREATE TABLE` 被 SQLite **实际拒绝**——是机制不是约定。写者事务同时正常提交 |
| 并发读写正常 | `cargo test -p prism-store --test concurrency` 6 passed；上轮点名的恒真断言 `assert!(after >= 1)` 已改成 `assert_eq!(after, 2, "autocommit 下的第二次读应看见期间提交的那一行")`（01-19） |
| FTS5 中文查询返回非零结果 | 「锚定引擎」命中 1、「量子纠缠」命中 0。`fts_cjk.rs` 4 passed，其断言含 4 字词 / 3 字边界词 / 2 字 LIKE 回退 / 混排英文 / 阴性对照 / `%` 转义 / VACUUM 后仍命中 / project 作用域八类 |
| CJK tokenizer 在 schema v1 定案 | `migrations/001_schema_v1.sql:41-47` external-content FTS5，`content_rowid='rowid_pk'`，`tokenize = 'trigram'`，且注释写明为何刻意不声明索引粒度（降粒度会禁掉 >3 unicode 字符的查询，4 字中文词当场失效） |
| rusqlite_migration 可用 | `PRAGMA user_version = 1` |
| bundled SQLite ≥3.51.3 | 3.53.2 ≥ 3.51.3。**且这条下界现在是准入断言而不只是文档**：`open.rs:125-127` `Some(got) if got >= MIN_SQLITE => Ok(())`，否则 `Err(StoreError::SqliteTooOld)`（上轮 warning 已闭合，01-18） |

`cargo test -p prism-store` 合计 **38 passed / 0 failed**（28 lib + 6 concurrency + 4 fts_cjk）。

---

### SC-4 证据（钥匙串 / 唯一出口 / 无明文密钥）

| 分句 | 状态 | 证据 |
|---|---|---|
| API key 经 keyring-core + apple-native-keyring-store 写入系统钥匙串并可读回 | ✓ | 见下「真实钥匙串取证」 |
| prism-llm 为唯一网络出口与唯一密钥入口 | ✓ | `check-deps.sh` 的 `single-egress` / `facade-egress` / `shell-egress` 三条全绿。`facade-egress` 用的是**反向闭包**而不是整树断言（`check-deps.sh:128-157`）：它断言 `reqwest` / `keyring-core` / `apple-native-keyring-store` 到达 `prism-engine` 的路径上除 prism-llm 与 prism-engine 外无任何 prism-* crate——哪天 prism-store 悄悄加了 keyring，它会作为新名字出现在闭包里被抓住，而整树断言那时只会说「prism-engine 有密钥依赖」，与现状无法区分 |
| 代码与配置中无明文密钥 | ✓（有告警） | 状态侧与机制侧分开取证，见下 |

**真实钥匙串取证（非破坏性）。** `crates/prism-llm/src/secrets.rs:257` 的 `roundtrip_with_real_keychain` 是 SC-4 第一分句唯一的行为证明，但它 `set_api_key` → `get_api_key` → **`delete_api_key`**。verifier **刻意不运行它**：`security find-generic-password -s PrismDocs` 显示 login keychain 里存在一条真实条目

```
svce = "PrismDocs"   acct = "llm_api_key"   cdat = mdat = 20260729014700Z
```

跑那个测试会覆盖并删除它。这条条目本身就是更强的独立证据：**应用的写入路径确实到达了真实系统钥匙串**，且用的正是 `secrets.rs:21/24` 声明的 service/account 契约名；创建时间 2026-07-29T01:47Z 与 01-09 那次人工验证的窗口吻合。代码侧 `init_default_store()`（`secrets.rs:38`）用的是 `apple_native_keyring_store::keychain::Store::new()`，文件头写明为何必须是 `keychain` 模块而不是受 entitlement 约束的那个。`cargo test -p prism-llm` **10 passed / 1 ignored**（mock store 覆盖幂等写、`NoEntry → Ok(None)`、删除幂等、`trim` 归一化、`ApiKey` 无 `Display` 等）。密钥原文未读取。

**状态侧（不用项目自带正则，verifier 另拟五条更宽的检索）：**

```
A  git grep -niE '(api[_-]?key|apikey|secret|token|password|passwd|bearer|credential|authorization)[^A-Za-z0-9]{0,4}[:=][[:space:]]*[A-Za-z0-9_./+~-]{16,}'   → 0 命中
B  git grep -nE  '(sk-|sk_|ghp_|gho_|github_pat_|AKIA|AIza|xox[baprs]-|hf_|gsk_|xai-|eyJ…|-----BEGIN [A-Z ]*PRIVATE KEY)'                                    → 全部为 Cargo.lock 的 phf_* crate 名、注释、`task_failed` 标识符
C  git grep -niE 'bearer[[:space:]]+[A-Za-z0-9_./+~-]{16,}'                                                                                                  → 仅 .planning/ 内的取样与评审文本
D  git grep -nE  '["\x27][A-Fa-f0-9]{32,}["\x27]'                                                                                                            → 全部为 Cargo.lock 的 checksum
E  git ls-files | grep -iE '(^|/)\.env|\.pem$|\.key$|\.p12$|credentials|\.netrc'                                                                             → 零输出
```

**当前仓库确实没有明文密钥。** A 条特意把 `bearer` / `authorization` 也放进关键词组（即 WR-04 点名的那个洞），依然零命中。

**机制侧（两次注入式反证，都是 verifier 亲手跑的）：**

1. **裸值分支非恒真。** 把 `PATTERN` 的关键词分支值部分从 `(${QUOTE}${NOT_QUOTE}{8,}|${BARE}{16,})` 改回「引号必需」，`selftest` exit 1，且 `FAIL: positive sample not detected:` **恰好**是隔离样本那两条（`MCP_BEARER_TOKEN=<32hex>` 与 `password: abcdefghijklmnop`），其余 17 条阳性样本一条都没受影响——这正是脚本第 39-45 行自述的复现步骤，逐字成立。
2. **裸值下界非装饰。** 把 `${BARE}{16,}` 降到 `{8,}`，`selftest` exit 1，红的是阴性组的 `token = abcdefghijklmno`（15 字符）与 `secret: cfg.value`——「顺手把阈值调下去」确实有代价。

`bash scripts/check-secrets.sh all` → `OK: pattern discriminates (19 positive / 10 negative samples)` + `OK: no plaintext secret in 116 version-controlled files`，exit 0。脚本已还原，`git diff` 干净。

**告警：WR-04 的残留（我逐条实测复核过，评审说的是对的）。** 用第 101 行原样取出的 `$PATTERN` 回放：

```
MISSED : Authorization: Bearer 7f3a9c1e5b2d8f4a6c0e9b7d3f1a5c8e
MISSED : "Authorization": "Bearer 7f3a9c1e5b2d8f4a6c0e9b7d3f1a5c8e"
MISSED : curl -H 'Authorization: Bearer 7f3a…' http://127.0.0.1:1234/mcp
MISSED : prismdocs_bearer: 7f3a9c1e5b2d8f4a6c0e9b7d3f1a5c8e
MISSED : -----BEGIN RSA PRIVATE KEY-----   /   hf_…   /   gsk_…   /   xai-…
CAUGHT : MCP_BEARER_TOKEN=7f3a…   /   mcp_bearer_token: 7f3a…
```

`bearer` 不在关键词 alternation 里，且 `Authorization: Bearer <值>` 的分隔符是空格而非 `[=:]`。selftest 第 157 行那条 `Authorization: "Bearer sk-ant-api03-…"` **读起来像是覆盖了这个形态**，实际经 `sk-` 前缀分支命中——与上轮 CR-01 点名的是同一个误导形状，换了个键名活下来。

**我把它评为 WARNING 而不是 gap，理由是三条（这是本次报告里最需要被质疑的一个判断，所以摊开写）：**

1. **成功标准 4 点名的两个面都已覆盖。** 上轮记 blocker 是因为「配置」那**整半**不可见——`.env` / YAML / TOML / CI `env:` 里的裸值赋值一条都抓不到，而本项目自己的 `mcp_bearer_token` 在配置文件里恰好就是那个形状。现在这一整类已经进网（上表 CAUGHT 两行），并有隔离样本 + 反证守着。剩下的是 HTTP **头**形态，不是 SC-4 措辞里的「代码」或「配置赋值」。
2. **该形态的产出者在 Phase 1 不存在。** `crates/prism-cli/src/main.rs:24-26` 里 `headers` 明确列在 `NOT YET IMPLEMENTED (Phase 6)` 之下，`run()` 对它只返回 `UnknownSubcommand`。今天没有任何东西会产出 `Authorization: Bearer <32 hex>`。
3. **产品设计本身就禁止它落进可提交文件。** `docs/sub-prds/PRD_F4_Agent_Loop.md:240` 定的是 `headersHelper` 间接读取，原文写着「**token 本体只存钥匙串，绝不写入可提交文件**」——评审推理里「用户会把它粘进 `.mcp.json`」这一步与 D-07 的设计相反。

**但这条不能就这么放着。** Phase 6 一旦接上 `prismdocs-helper headers`，产出者出现、误导性样本还在原地，而那时没有任何东西会提醒。修法很窄（评审已给出可用形状）：把 `bearer` 加进 alternation 并单列一条 `[Bb]earer[[:space:]]+${BARE}{16,}` 分支，补一条只可能经它命中的隔离样本，顺带把 `-----BEGIN [A-Z ]*PRIVATE KEY-----`（零误报、最贵的一类泄漏）与 `hf_` / `gsk_` / `xai-` 三个前缀加上。**建议在 Phase 6 开工前完成，并登记进 deferred-items。**

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `scripts/check-secrets.sh` | 明文密钥静态检查 + 自证判别力的 selftest | ✓ VERIFIED（有告警） | 251 行；关键词分支值部分为引号串/裸值二选一（第 101 行），两个下界刻意不同且理由写在第 85-100 行；19 阳性 / 10 阴性；verifier 做了两次注入式反证。残留见 WR-04 |
| `scripts/check-deps.sh` | 七条依赖方向断言 | ✓ VERIFIED | 255 行；`check_dup` 先接住 `cargo tree` 的退出码再判输出（01-15 修，上轮 WR-11）；`no-cycle` 经 verifier 换靶反证非恒真 |
| `crates/prism-store/src/open.rs` | writer-first 六步序 + 读池 `query_only` + SQLite 版本准入 | ✓ VERIFIED | `MIN_SQLITE=(3,51,3)` 在 `:125-127` 强制；探针实测读池写入被 SQLite 拒 |
| `crates/prism-store/migrations/001_schema_v1.sql` | schema v1 + trigram FTS5 + 三触发器 | ✓ VERIFIED | `user_version=1`；`tokenize='trigram'`、`content_rowid='rowid_pk'`、三张真实表 STRICT |
| `crates/prism-store/tests/concurrency.rs` | 并发读写六测 | ✓ VERIFIED | 恒真断言已换为 `assert_eq!(after, 2)`（01-19），6 passed |
| `crates/prism-store/tests/fts_cjk.rs` | CJK 检索四测 | ✓ VERIFIED | 4 passed，八类断言含阴性对照与 VACUUM 后复检 |
| `crates/prism-store/src/settings.rs` | k/v + base_url 值侧五项校验 | ✓ VERIFIED（有告警） | `validate_base_url:52-93` scheme/host/userinfo/query/fragment；守卫长在 `set_setting` 写入路径上。告警见 WR-02 / WR-03 |
| `crates/prism-llm/src/secrets.rs` | 唯一密钥入口 + 钥匙串往返 | ✓ VERIFIED | `apple_native_keyring_store::keychain::Store::new()`；`ApiKey` 手写 `Debug`、刻意无 `Display`；10 passed / 1 ignored |
| `crates/prism-mcp/src/deps.rs` | trait 注入容器 + 可失败构造 | ✓ VERIFIED | `Arc<dyn FeedbackSource>` / `Arc<dyn CommentSink>`，均来自 prism-types；空 bearer 构造期即拒 |
| `crates/prism-mcp/src/middleware.rs` · `tests/middleware_gate.rs` | Host/Origin/bearer 三层门禁 + 无差别 403 | ✓ VERIFIED | 01-17 把 `host_of` 与 rmcp 2.2 SDK 的 `normalize_host` 口径对齐并打在实发路由上 |
| `src-tauri/src/bus_adapter.rs` · `commands.rs` · `smoke.rs` | IPC 双通路 | ✓ VERIFIED（接线层） | 见 §SC-2；行为层未证 |
| `src-tauri/tauri.conf.json` | 双份 CSP + 关闭资源协议 | ✓ VERIFIED（有告警） | `csp` / `devCsp` 各含 `object-src 'none'`、`base-uri 'self'`、`form-action 'none'`、`frame-ancestors 'none'`；`assetProtocol.enable:false`、`scope:[]`。发布 `csp` 的 `style-src` 仍带 `'unsafe-inline'`（WINDOWS id=11 / IN-02） |
| `src-tauri/capabilities/default.json` | 最小权限集 | ✓ VERIFIED | 只有 `core:event:allow-listen` / `allow-unlisten`，`windows: ["main"]` |
| `src-tauri/src/lib.rs` | `init_tracing()` + 日志天花板 + release IPC 面分叉 | ✓ VERIFIED | `LOG_CEILING_DIRECTIVE="rmcp=info"`（:51）+ 降档 warn（:58-59）；`#[cfg(not(debug_assertions))]` 那支不含四个 `dev_*` 命令（:216-224） |
| `eslint.config.js` · `rustfmt.toml` · `.github/workflows/ci.yml` · `justfile` | 前端 lint / fmt / CI 闸门 | ✓ VERIFIED（本机） | ci.yml 含 fmt(:51) / check-deps(:71) / check-secrets(:76) / clippy 双 job(:82,:151) / test 双 job(:88,:154) / 前端 lint(:177) / coverage(:180)。**GitHub Actions 上从未跑过**——见人工验证第 3 项 |
| `.planning/REQUIREMENTS.md` | INFRA-03 改映射记账 | ✓ VERIFIED（有小瑕） | `:170` INFRA-03 → Phase 4 + 四点表下注；61/61/0 覆盖不变。小瑕见 §Requirements Coverage |

### Key Link Verification

| From | To | Via | Status |
|---|---|---|---|
| `bus_adapter.rs` | `useEngineInvalidation.ts` | `app.emit("prism://changed")` → `listen<EngineEvent>(EVENT_CHANGED)` → `invalidateQueries` | ✓ WIRED（行为层见 §SC-2） |
| `commands.rs::dev_smoke_stream` | `DevSmoke.tsx` | `tauri::ipc::Channel<SmokeEvent>` → `ev.data.seq !== i` 逐位校验 | ✓ WIRED（行为层见 §SC-2） |
| `settings.rs::set_setting` | `validate_base_url` | 写入路径上调用，非调用方约定 | ✓ WIRED |
| `secrets.rs` | macOS login keychain | `apple_native_keyring_store::keychain::Store` | ✓ WIRED（真实条目存在，见 §SC-4） |
| `prism-mcp` | `prism-types` | `Arc<dyn FeedbackSource>` / `Arc<dyn CommentSink>` 注入 | ✓ WIRED |
| `prism-mcp` | `prism-engine` | （必须不存在） | ✓ 不存在（树里唯一 prism-* 是 prism-types） |
| `prismdocs-shell` | `prism-llm` / `prism-mcp` | （必须不直连） | ✓ 不存在（shell-egress 断言绿） |
| `scripts/check-secrets.sh` | `.github/workflows/ci.yml` · `justfile` | 两处都显式跑 `all` | ✓ WIRED |
| `prism-mcp::serve_loopback` | 生产启动路径 | （Phase 1 应不存在） | ✓ 仅测试调用（`trait_injection.rs` / `middleware_gate.rs`），app 不启动 MCP server |

### Data-Flow Trace（Level 4）

| Artifact | Data Variable | Source | Produces Real Data | Status |
|---|---|---|---|---|
| `src/pages/Settings.tsx` | `apiKeyStatus` / `baseUrl` | `invoke("api_key_status")` / `invoke("get_setting")` → `prism_store::settings` → SQLite/钥匙串 | 是（探针实测 settings 表可写可读、钥匙串条目真实存在） | ✓ FLOWING |
| `src/pages/DevSmoke.tsx` | `verdict` / 事件计数 | `invoke("dev_smoke_stream")` 经 Channel、`listen("prism://changed")` 经事件系统 | 接线成立；真实 WebView 下的到达未复验 | ⚠️ 见 §SC-2 |
| `src/pages/DevSmoke.tsx` | 搜索结果 | `invoke("search_documents")` → `prism_store::search` → FTS5 MATCH / LIKE 分流 | 是（探针实测「锚定引擎」=1、「量子纠缠」=0） | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|---|---|---|---|
| engine 脱离 tauri 可测 | `cargo test -p prism-types … -p prism-engine` | 128 passed / 0 failed / 1 ignored | ✓ PASS |
| 七条依赖方向断言 | `bash scripts/check-deps.sh all` | 7 × OK, exit 0 | ✓ PASS |
| no-cycle 非恒真 | grep 靶换成 `^prism-types ` → `check-deps.sh no-cycle` | `FAIL`, exit 1（已还原） | ✓ PASS |
| 扫描器看得见裸值赋值（**上轮的 FAIL**） | 注入 `mcp_bearer_token: <32hex>` + `ANTHROPIC_AUTH_TOKEN=<32hex>` 到 `ci.yml` → `check-secrets.sh scan` | `FAIL: possible plaintext secret` + 点名两行，exit **1**（已还原） | ✓ **PASS（上轮 FAIL）** |
| 裸值分支非恒真 | 把值部分改回「引号必需」→ `selftest` | exit 1，红的恰好是那两条隔离样本 | ✓ PASS |
| 裸值下界非装饰 | `${BARE}{16,}` → `{8,}` → `selftest` | exit 1，阴性组两条变红 | ✓ PASS |
| 扫描器看得见 Bearer 头形态 | 用第 101 行 `$PATTERN` 回放三种现实形态 | 全部 MISSED | ✗ **FAIL（WR-04，评为 WARNING，理由见 §SC-4）** |
| 独立宽扫（不用项目正则） | 五条 verifier 自拟检索 | A/C/E 零命中；B/D 全为 crate 名、注释、checksum | ✓ PASS |
| 真实钥匙串条目存在 | `security find-generic-password -s PrismDocs` | `svce="PrismDocs" acct="llm_api_key"`，exit 0（未读取密文，未运行破坏性测试） | ✓ PASS |
| bundled SQLite / WAL / query_only / 读池拒写 | verifier 临时探针（跑完即删） | `3.53.2` / `wal` / `1` / `attempt to write a readonly database` | ✓ PASS |
| FTS CJK 判别性 | 同探针 + `cargo test -p prism-store --test fts_cjk` | 「锚定引擎」=1、「量子纠缠」=0；4 passed | ✓ PASS |
| 壳 IPC | `cargo test -p prismdocs-shell --features test` | 23 passed | ✓ PASS |
| 前端 | `npm run test -- --run` | 75 passed / 7 files | ✓ PASS |
| 格式闸门 | `cargo fmt --all -- --check` | exit 0 | ✓ PASS |
| MCP server 生产不启动 | `git grep serve_loopback -- crates src-tauri` | 仅两个测试文件 | ✓ PASS |
| release IPC 面无 dev 命令 | `src-tauri/src/lib.rs:216-224` | 六个命令，四个 `dev_*` 不在其中 | ✓ PASS |

所有临时探针与注入均已还原；收尾 `git status --porcelain` 只剩两个本次验证之前就存在的未跟踪 `.planning/research/.cache/*.json`。

### Probe Execution

本仓库无 `scripts/*/tests/probe-*.sh` 约定探针，15 份 gap-closure PLAN / SUMMARY 亦未声明任何 probe。Step 7c 以「无 probe 可跑」记录。替代证据为上表：其中 `check-deps.sh` / `check-secrets.sh` 两个脚本按 probe 的方式对待——verifier 亲自执行、记录退出码，并各做了注入式非恒真反证。

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|---|---|---|---|---|
| INFRA-01 | 01-01/02/04/06/07/08/09/11/12/13/15/16/17/20/21/22/24/26/27/28 | Rust engine workspace（不依赖 tauri、可独立测试）+ Tauri 薄 shell + 事件总线骨架；prism-mcp 经 service trait 反转解依赖环 | ✓ SATISFIED（事件总线端到端待人工确认） | SC-1 全部证据；SC-2 接线与自动化层成立、行为层归入人工验证第 1 项 |
| INFRA-02 | 01-03/05/07/09/10/18/19 | SQLite WAL 单写者 + r2d2 读池；FTS5 CJK tokenizer；rusqlite_migration；bundled SQLite ≥3.51.3 | ✓ SATISFIED | SC-3 全部证据（探针实测 3.53.2 / wal / query_only=1 / 读池拒写 / user_version=1） |
| INFRA-03 | 01-02/04/05/07/09/10/11/13/14/23/25 | API key 存系统钥匙串；支持 Anthropic/OpenAI 兼容端点与自定义 base_url；prism-llm 为唯一网络出口与唯一密钥入口 | ➜ **已改映射 Phase 4**（本 phase 不再对它负验证责任） | Phase 1 完成的三部分（钥匙串存取 / 唯一出口入口 / base_url 存储与校验）已就地记录在 `REQUIREMENTS.md:211`；剩余的「实际向端点发请求」分句由 Phase 4 承接 |

**INFRA-03 的记账连贯性核查（本轮明确在范围内）。** 逐项核过，**连贯**：

- `REQUIREMENTS.md:91` 复选框 `[ ]` 未勾 ✓（与「未完成」一致）
- `:170` Traceability 映射为 `Phase 4 | Pending` ✓（不再是 Phase 1）
- `:210-215` 四点表下注写明改映射日期、依据（ROADMAP Phase 4 goal 明写 prism-llm 传输层先行）、Phase 1 已完成的三部分就地保留、以及三份认领它的 Phase 1 plan（01-14 / 01-23 / 01-25）为何与 Phase 4 映射不矛盾 ✓
- 覆盖计数 61/61/0 不变，**无孤儿需求** ✓
- 三份 plan 的 frontmatter 确实都声明 `requirements: [INFRA-03]` ✓

**一处记账小瑕（不阻断，建议顺手改）：** `REQUIREMENTS.md:146-147` 里 INFRA-01 / INFRA-02 的 Status 列仍是上一轮验证遗留的 `Gaps Found`，而同文件 `:89-90` 的复选框已是 `[x]`。同一份文件内部两处口径相反。本轮结论是这两条 SATISFIED，Status 列应随之更新（`Gaps Found` → `Complete` 或工作流约定的等价值）。

**Orphan 检查：** Traceability 表映射到 Phase 1 的只有 INFRA-01 / INFRA-02 两条，两条都被 plan frontmatter 认领——**无 orphaned requirement**。

### Anti-Patterns Found

`TBD` / `FIXME` / `XXX` / `TODO` / `HACK` / `PLACEHOLDER` 在 `crates/`、`src/`、`src-tauri/`、`scripts/`、`.github/`、`justfile`、`eslint.config.js`、`rustfmt.toml`、`tsconfig.json`、`package.json` 下**零命中**——无未挂钩的技术债标记，debt-marker gate 通过。

下表是第三轮 code review 的 6 条 warning + 5 条 info，**每条我都自己回到代码里核过，并逐条判定它是否动摇某条成功标准**：

| # | File | Pattern | 我的复核结论 | 是否动摇 SC |
|---|---|---|---|---|
| WR-01 | `prism-engine/src/services.rs:72-91` | `comment_id` 未受约束直接 `%` 展开进日志，旁边的 `status` 已有 `RECEIPT_STATUSES` 约束 | **成立**。同一行日志、同一条推理，一个字段修了另一个没修 | 否。`handler.rs` 无任何工具调用 `record_receipt`，`McpDeps.comments` 是死字段，且 app 不启动 MCP server（`serve_loopback` 仅测试调用）——发布形态下不可达。**Phase 6 接上评论回流的那一刻它就活了** |
| WR-02 | `Settings.tsx:43` · `settings.rs:76-80` · `commands.rs:38` | 裸 `?` / `#` 上前后端判定分叉；四条 `InvalidUrl` 理由压成同一短码 | **成立**。WHATWG 的 `search`/`hash` 在空 query 与无 query 上都返回 `""`，`url` crate 的 `query()` 看 `query_start` 是否存在 | 否。分叉方向是**前端放行、engine 拒绝**——凭据不会因此落库，SC-4 的写入侧守卫不受影响。这是 UX 缺陷（用户拿到一句与自己输入自相矛盾的文案） |
| WR-03 | `settings.rs:52,117,125` | `validate_base_url` 校验 `raw.trim()`、`tx.execute` 存原值；`key ==` 逐字节 vs `is_secret_like_key` 小写化 | **成立**。我读了 `:111-126`，确实是 `validate_base_url(value)` 后 `tx.execute(…, (key, value, now))` | 否。trim 只吃首尾空白，凭据检查作用在同一个 URL 上——空白里藏不下凭据。键名大小写那一半 Phase 1 不可达：`generate_handler!` 里没有通用 `set_setting` 命令，只有 `set_base_url`（常量键） |
| **WR-04** | `check-secrets.sh:101,157` | `bearer` 不在关键词 alternation；`Authorization: Bearer <值>` 三种现实形态全 MISSED；selftest 样本 4 读起来像覆盖了它 | **成立，我独立实测确认**（见 §SC-4 的 MISSED 清单） | **是——直接落在 SC-4 第三分句「代码与配置中无明文密钥」的执行机制上，所以我在 §SC-4 里摊开论证了为何仍判 WARNING**：SC-4 点名的两个面（代码/配置赋值）已覆盖且经两次反证；该形态的产出者 `prismdocs-helper headers` 明确列在 Phase 6（`prism-cli/src/main.rs:24-26`）；PRD F4 的 `headersHelper` 设计原文写着「token 本体只存钥匙串，绝不写入可提交文件」；A 条宽扫（含 `bearer`/`authorization` 关键词）零命中，无活的泄漏。**Phase 6 开工前必须关闭** |
| WR-05 | `useEngineInvalidation.test.ts:49-55` · `App.test.tsx:12-14` | listen 失败分支与顶层告警条**一条测试都没有** | **成立，我自己核过**：该文件里 `listenSpy` 只有 `mockReset` / `mockImplementation`（永远 resolve），全仓唯一的 `mockRejectedValueOnce` 在 `DevSmoke.test.tsx`；`App.test.tsx:13` 把 hook 桩成 `() => {}` | 否，但**贴得很近**。SC-2 要的是通路本身，成功路径有 5 条测试。这条守的是「失效链路静默失能」——与项目「0 静默丢失」的发布门槛同源，且它正是人工验证第 1 项要在真实 WebView 里排除的那类失败 |
| WR-06 | `prism-mcp/src/handler.rs:135-138` | 被注入实现的 `ServiceError` 文本原样进 MCP 响应；`Invalid`（调用方的错）被报成 `internal_error` | **成立**。`prism-types/src/service.rs:34-46` 自己写着 Phase 5/6 会加回 `Backend(String)` 并提醒不能直接 `to_string()` ——而这里正是那条约定唯一的执行点 | 否。Phase 1 不启动 MCP server，且注入实现当前不产生携带下层文本的错误 |
| IN-01 | `src-tauri/src/lib.rs:550-560` | release IPC 面断言的锚点带尾随逗号，末项无逗号时会被放过 | 成立；今天靠 `cargo fmt --check`（01-28 刚接上）间接兜底 | 否 |
| IN-02 | `tauri.conf.json:21` | 发布 CSP 的 `style-src 'unsafe-inline'` 理由（React 内联 style）不成立——客户端 React 走 CSSOM，不受 `style-src` 管辖 | 成立；仓库确实零 CSS import。已在 WINDOWS id=11 | 否；建议并入人工验证第 1 项顺手确认 |
| IN-03 | `eslint.config.js:49-56` | 基础块只声明 `languageOptions` 无 `rules`，`eslint.config.js` 与 `vite.config.ts` 落进「有 parser、无规则」的空档 | 成立 | 否 |
| IN-04 | `commands.rs:41` | `EngineError::Llm(_)` 整体映射成 `secret_error`，Phase 4 的 HTTP 失败会被说成「钥匙串不可用」 | 成立；今天 prism-llm 不发请求，潜伏 | 否；**应登记 deferred-items**，否则会随第一个真实调用一起上线 |
| IN-05 | `prism-engine/Cargo.toml:22` · `deps.rs:26-27` | 两处「已接线但无消费者」的 Phase 6 脚手架 | 成立；这正是 WR-01 目前只算 WARNING 的全部理由 | 否 |

**一条本轮观察到、评审未列的项：** `src-tauri/tests/ipc.rs:126-131` 对 `dev_smoke_stream` 只断言 `res.is_ok()`。文件头把自己称作「Channel 通路」测试，但 Channel 的**到达与顺序**在这一层没有任何断言——顺序断言在 `smoke.rs` 的生成器单测上，覆盖不到传输。这不是缺陷，是覆盖面与命名之间的落差，也是 §SC-2 判 PRESENT_BEHAVIOR_UNVERIFIED 的一部分依据。

### Human Verification Required

三项。前两项是 01-13 自述、按 `workflow.human_verify_mode: end-of-phase` 顺延至此，步骤已由 01-21 / 01-24 / 01-28 改写；第三项是 01-28 新增，与 WINDOWS id=14 / id=15 同属一次真实 CI 运行。

> **本节是这三项步骤的唯一权威文本。** `WINDOWS.md` 的 id=8 / id=9 / id=14 / id=15 只做指向；id=8 描述里的「五步」是记录当时的措辞，实际以本节为准。

#### 1. 真实 WebView 下的 CSP 与 IPC 双通路（**同时承载成功标准 2 的复验**）

**Test（六步）：**

1. `npm run tauri dev` 起应用 → 窗口**不是白屏**。
2. 打开设置页：状态行、两个输入框、两个按钮都在；保存一个合法端点（形如 `https://api.anthropic.com`），确认成功文案出现。
3. 打开 dev 冒烟页（右下角 dev 开关），跑三个验证入口：
   - **总线事件往返**：点一次，事件计数 +1；离开页面再回来再点，计数仍与点击次数 **1:1**（不翻倍）。
   - **Channel 有序流**：点一次，读数应为「seq 校验通过 · 实收 1000 条」。
   - **中文搜索**：先「写入样例文档」，搜「锚定引擎」应命中 >0；搜「量子纠缠」应为 0（阴性对照）。
4. 打开 WebView 控制台，确认**无任何 CSP 违规报告**——特别留意 01-24 新加的 `form-action 'none'` 与 `frame-ancestors 'none'` 是否在设置页或冒烟页触发违规。（顺手可确认 IN-02：把发布 `style-src` 收成 `'self'` 后是否仍无违规。）
5. `npm run tauri build` 出 dmg，安装后对 1–4 步**重复一遍**（发布形态走 `csp` 而不是 `devCsp`，这是验证严格那一份的唯一路径）。
6. 发布形态额外确认：dev 冒烟页开关不存在（`import.meta.env.DEV` 摇掉），且四个 `dev_*` 命令不可调用。

**Expected:** 六步全部正常。三个入口的读数与 01-09 一致（计数 1:1、「seq 校验通过 · 实收 1000 条」、「锚定引擎」>0 且「量子纠缠」=0）。第 4 步的两条新指令预期**不触发**：Phase 1 前端不含原生 `<form>` 提交，桌面窗口也不会被嵌套。

**Why human:** CSP 只在真实 WebView 里生效——`src/lib/tauri-security.test.ts:17` 自己写明 jsdom 看不见它，`cargo test` 走 `mock_builder`（无 WebView）。**且这一项是成功标准 2 唯一的行为证据来源**：旧读数取自 `csp: null` 的环境，而 `connect-src 'self' ipc: http://ipc.localhost` 直接管辖 IPC 来源；若它挡住 IPC，受影响的是全部十个命令，两条通路一起断。旧读数不得沿用。

**出现违规时的处理：** 只放宽 `devCsp`，或按控制台点名的指令逐项追加到 `csp`；**禁止**把 `csp` 设回 `null`，也禁止直接删掉 01-24 新加的两条指令。若确需放宽，先在 `tauri-security.test.ts` 的精确相等断言上过一次评审——那正是 01-24 让它变成精确相等的意义。

#### 2. 日志 sink 真的有落点

**Test（两步）：**

1. **默认档位**（不设 `RUST_LOG`）：`npm run tauri dev`，在设置页把 base_url 设成一个非 loopback 的 `http://` 端点（如 `http://example.com/v1`）并保存，观察终端。
2. `RUST_LOG=trace npm run tauri dev`，观察终端。

**Expected:**

1. 出现 tracing 格式的行，且 `crates/prism-store/src/settings.rs:88-91` 那条 `LLM endpoint uses plaintext http to a non-loopback host` **实际打出来**。这是默认 info 档下就有落点的那条，无需任何提档。
2. 出现 01-21 的降档 warn，正文以 `the environment-supplied log filter exceeds the project ceiling` 开头并说明 `rmcp` target 被 capped at INFO；该 warn 正文里**不含** `RUST_LOG` 的原值。

**Why human:** `tracing::dispatcher::has_been_set()` 只证明 dispatcher 就位，不证明日志到达终端（EnvFilter 档位、fmt 层的输出目标都可能让它落空）。步骤在 01-21 之后必须走默认档位：该 plan 给 env filter 加了项目天花板（`src-tauri/src/lib.rs:51`），原做法「用 `RUST_LOG` 提档观察」既观察不到目标、也不再是 sink 有落点的证据（提档被降档吃掉时，看不见日志与没有 sink 在终端上同形）。`rmcp` 是否真的转储 MCP 消息在 Phase 1 无可观测面（MCP server 尚未起），到 Phase 5/6 才有。

#### 3. CI workflow 的首次真实 GitHub Actions 运行

**Test:** 首次把本分支推到 GitHub 后核对四项：① engine job 首步 `Format check (rustfmt default style)` 在步骤列表最前且为绿；② `permissions: contents: read` 下 `upload-artifact` 仍可上传；③ `concurrency` 真的收掉同 commit 的双跑；④ 两个缓存分段互不恢复。

**Expected:** 四项全部成立。fmt 步骤的判别力可在一个丢弃分支上注入劣化排版验证——应变红并点名文件。

**Why human:** `origin/main` 停在 `4cc1347`，Phase 1 全部 28 份 plan 的产物均未推送，`gh run list` 返回 `[]`——该 workflow 至今未在 GitHub Actions 上跑过。本 phase 里所有关于 CI 闸门的声称都只有本机证据（WINDOWS id=14 / id=15）。

### Gaps Summary

**没有 gap。** 上一轮唯一的 blocker 干净关闭，且是我用与上轮同形的注入实验亲手证的：往 `ci.yml` 追加两行裸值密钥，上轮 `scan` 退出 0、本轮退出 1 并点名两行。上轮那张 8 行取样表现在 7/8 命中，第 8 行是脚本第 94-100 行写明理由的已知残留（弱人类口令、14 字符、落在裸值下界之下，而下界必须严格高于引号下界否则误报会让闸门被绕开）。我另外做了两次反证，确认这不是「把正则放宽到什么都能抓」：把值部分改回引号必需，红的**恰好**是那两条隔离样本、17 条既有阳性样本一条都不受影响；把裸值下界从 16 降到 8，阴性组当场变红。这两条正是脚本文件头自述的复现步骤，逐字成立。

四条成功标准里三条 VERIFIED。SC-1 与 SC-3 我都做了独立取证而不是复跑项目自带的断言：SC-1 把 `no-cycle` 的 grep 靶换成一个确实在树里的包，断言当场变红——它读的是真实树内容；SC-3 在 `prism-store/tests/` 下建了个临时探针直接打运行期，读到 `3.53.2` / `wal` / `query_only=1`，并让读池连接执行 `CREATE TABLE` 被 SQLite 实际拒绝——「单写者」是机制不是约定。SC-4 的钥匙串那一分句我用了一条非破坏性路径：`security find-generic-password -s PrismDocs` 显示 login keychain 里真有一条 `acct="llm_api_key"` 的条目（创建于 2026-07-29T01:47Z，与 01-09 人工验证的窗口吻合）。那条 `#[ignore]` 的真实钥匙串测试我**刻意没跑**——它会 `delete_api_key`，跑一次就把用户这条条目删了；而条目的存在本身就是比测试更强的证据。

**唯一改判的是 SC-2，而且我要说清楚这不是回退。** 上一份报告已经在正文里写了「真实 WebView 那一层的旧证据因 01-13 加 CSP 而**过期**」，只是把状态位留在了 VERIFIED。证据状态一字未变，我改的是标签口径。理由很具体：这条成功标准断言的是**往返**与**有序到达**——两个运行期行为，而两条自动化路径在结构上都看不见它。`cargo test` 走 `mock_builder`，没有 WebView；`src-tauri/tests/ipc.rs` 对 `dev_smoke_stream` 只断言 `res.is_ok()`，不校验 1000 条的到达与顺序，顺序断言落在 `smoke.rs` 的**生成器**上、覆盖不到 Channel 传输；vitest 走 jsdom 且 `@tauri-apps/api/event` 整个被 mock。唯一的端到端读数取自 `csp: null` 的环境，而新 CSP 的 `connect-src` 直接管辖 IPC 来源——若它挡住 IPC，断的是全部十个命令，上面所有绿色一个都不会变红。接线在、判别力在、行为未证：既不是 FAILED，也不该计入分子。

**我花最多时间反复质疑的是 WR-04**，因为它直接落在 SC-4 第三分句的执行机制上，而上一轮把形状相同的问题记成了 blocker。我先自己把 `$PATTERN` 从第 101 行原样抽出来回放，确认评审说的全对：`Authorization: Bearer <32hex>` 的三种现实形态全 MISSED，selftest 第 157 行那条读起来像是覆盖了它、实际经 `sk-` 前缀分支命中。但把它与上轮的 blocker 并排看，差别是实质性的：上轮失守的是「配置」那**整半**（`.env` / YAML / TOML / CI `env:` 的裸值赋值一条都抓不到），而本项目自己的 `mcp_bearer_token` 在配置文件里恰好就是那个形状——那是成功标准点名的面，且今天就能被写进去。现在那一整类进网了；剩下的是 HTTP **头**形态，它的产出者 `prismdocs-helper headers` 明确列在 `NOT YET IMPLEMENTED (Phase 6)`（`prism-cli/src/main.rs:24-26`），而 PRD F4 的 `headersHelper` 设计原文写着「token 本体只存钥匙串，**绝不写入可提交文件**」——评审推理里「用户把它粘进 `.mcp.json`」这一步与 D-07 的设计方向相反。我另外用一条把 `bearer` / `authorization` 也放进关键词组的宽扫独立确认了零命中。所以：WARNING，不是 gap。**但 Phase 6 一接上那个子命令，产出者就出现、而误导性样本还在原地**——修法很窄（单列一条 `[Bb]earer[[:space:]]+` 分支 + 一条只可能经它命中的隔离样本，顺带补 `-----BEGIN … PRIVATE KEY-----` 与 `hf_`/`gsk_`/`xai-`），建议在 Phase 6 开工前完成并登记进 deferred-items。

记账层面，INFRA-03 的改映射我逐项核过且**连贯**：复选框未勾、Traceability 指向 Phase 4、四点表下注说明了依据与三份认领它的 Phase 1 plan、覆盖计数 61/61/0 不变、无孤儿需求。一处小瑕：`REQUIREMENTS.md:146-147` 的 INFRA-01 / INFRA-02 Status 列还是上一轮遗留的 `Gaps Found`，与同文件 `:89-90` 已勾的复选框自相矛盾，本轮结论下应随之更新。

剩下的两件事都是**证据链**问题而不是代码问题，且都只能由人来做，所以状态是 `human_needed` 而不是 `passed`：真实 WebView 下的 CSP 与 IPC 双通路（同时是 SC-2 唯一的行为证据来源），以及日志 sink 是否真有落点。第三项是这个 phase 里最容易被忽略的一条——**这个 workflow 至今没在 GitHub Actions 上跑过一次**（`origin/main` 停在 `4cc1347`，`gh run list` 返回 `[]`）。fmt、clippy、check-deps、check-secrets、前端 lint、coverage 六道闸门本机全绿，但「CI 上真的会红」这件事，本 phase 里一次也没被观测过。

最后是量的问题，与上一轮的提醒方向相反、值得记一笔：上轮列的 12 条积压 warning，本轮闭合了其中的恒真断言、SQLite 版本下界、`cargo tree || true`、CSP/capability denylist、前端 lint 缺口、fmt 闸门、INFRA-03 记账七项。第三轮评审新报的 6 条 warning 全部未修，但我逐条核过，**没有一条动摇成功标准**——WR-01 / WR-06 在发布形态下不可达（MCP server 不启动、`comments` 是死字段），WR-02 / WR-03 的分叉方向不会让凭据落库，WR-05 守的是失效链路的静默失能（贴得最近，但通路本身的成功路径有测试）。WR-01 与 IN-04 有共同的时间性质：它们都会在 Phase 6 / Phase 4 的第一个真实调用点同时活过来，且届时没有任何东西会提醒——**建议连同 WR-04 一起登记进 deferred-items**，让这三条依赖关系留在代码与账本里，而不是只留在评审报告里。

---

_Verified: 2026-07-30T00:34:27Z（HEAD `270d1a2`）_
_Verifier: Claude (gsd-verifier)_
