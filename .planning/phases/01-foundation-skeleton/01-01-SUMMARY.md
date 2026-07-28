---
phase: 01-foundation-skeleton
plan: 01
subsystem: infra
tags: [cargo-workspace, tauri-v2, rusqlite, react-19, vite, comrak, rmcp, axum, blake3, notify, keyring]

# Dependency graph
requires: []
provides:
  - "单一 cargo workspace（9 个 engine crate + prismdocs-shell），产物落在仓库根 target/"
  - "根 Cargo.toml 的 [workspace.dependencies]：Phase 1–8 全部版本 pin 与 feature 串的单点来源"
  - "一条可点可见的端到端通路 dev_ping：React → invoke → Tauri command → Engine::ping → Store::sqlite_version → SQLite"
  - "prism-store 的 sidecar 根解析（dirs::data_dir，禁用 Tauri app_data_dir）与 Store::open/sqlite_version"
  - "prism-engine facade 的 new/ping 形态（不依赖 tauri）"
  - "prismdocs-helper 占位 binary（externalBin 的依赖面已定型）"
  - "编译期依赖方向证据：engine crates 无 tauri、prism-mcp 无 prism-engine、prism-cli 无任何 prism-*"
affects: [01-02-依赖方向断言脚本, 01-03-writer-first启动序列与migration001, 01-04-service-trait与事件总线, 01-06-MCP-loopback宿主, 01-08-冒烟命令, 01-09-冒烟页, phase-2-文件监视, phase-3-锚点迁移]

# Tech tracking
tech-stack:
  added:
    - "rusqlite 0.40 (bundled, FTS5) + r2d2/r2d2_sqlite + rusqlite_migration 2.6"
    - "tauri 2.11 / tauri-build 2（仅 src-tauri）"
    - "react 19 + vite 8 + vitest + @tanstack/react-query 5"
    - "comrak 0.54 / notify 8 + notify-debouncer-full 0.7 / blake3 1.8 / similar 3.1 / ulid 1"
    - "rmcp 2.2 (server + transport-streamable-http-server) + axum 0.8 + tokio 1"
    - "reqwest 0.13 + keyring-core 1.0 + apple-native-keyring-store 1.0 (keychain)"
    - "dirs 6 / serde / serde_json / thiserror 2 / tracing / url / subtle 2 / tempfile / serial_test"
  patterns:
    - "薄 shell：#[tauri::command] 体是对 facade 的单行委托，不含任何业务逻辑或 SQLite 句柄"
    - "单一 workspace + `-p` 选择集：D-01 的证据形态是 cargo tree 断言，不是 --workspace 构建"
    - "engine crate 的最小编译单元必须真实调用其主依赖，使 cargo tree/-d 覆盖真实依赖面（D-08）"
    - "sidecar 路径唯一来源 dirs::data_dir()，源码中禁止出现 app_data_dir（D-13）"
    - "prism-mcp 只依赖 prism-types，编译期无 facade↔mcp 环（D-09）"
    - "prism-cli 不依赖任何 prism-* engine crate，为 externalBin 单独签名公证留出小依赖面（D-10）"

key-files:
  created:
    - Cargo.toml
    - crates/prism-types/src/lib.rs
    - crates/prism-store/src/lib.rs
    - crates/prism-engine/src/lib.rs
    - crates/prism-fs/src/lib.rs
    - crates/prism-parse/src/lib.rs
    - crates/prism-anchor/src/lib.rs
    - crates/prism-llm/src/lib.rs
    - crates/prism-mcp/src/lib.rs
    - crates/prism-cli/src/main.rs
    - src-tauri/src/lib.rs
    - src-tauri/src/commands.rs
    - src-tauri/tauri.conf.json
    - src/lib/ipc.ts
    - src/App.tsx
  modified:
    - .gitignore

key-decisions:
  - "workspace members 用 `crates/*` 通配 + `src-tauri`：Task 3 新增六个 crate 无需改动根 Cargo.toml"
  - "prism-mcp 的 protocol_version() 用 `static LATEST: ProtocolVersion` 承载 rmcp 的 const，以取得 &'static str（ProtocolVersion 内含 Cow，const 提升不适用）"
  - "无自然使用场景的依赖（prism-llm 的 serde、prism-anchor 的 similar/ulid、prism-mcp 的 axum/tokio）以「依赖可用性」单测引用，而不是提前发明 Phase 2/3/4 的公开 API"
  - "本 plan 不勾选 INFRA-01：该需求跨 7 个 plan（事件总线与 Channel 有序流在 01-04/01-08/01-09），提前标完成会是虚假信号"

patterns-established:
  - "错误类型形态：每个 engine crate 从 Phase 1 起就是 thiserror 枚举，后续 plan 只加变体不改形态"
  - "骨架 crate 的注释必须写明「哪一 plan/phase 来填」，避免后来者把占位当遗漏"
  - "依赖方向断言以 `cargo tree -p <crate> --edges normal --prefix none | grep -c` 的可复现命令形式给出"

requirements-completed: []

coverage:
  - id: D1
    description: "单一 cargo workspace 成立：9 个 engine crate + prismdocs-shell 全部编译，产物落在仓库根 target/（不存在 src-tauri/target/）"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "cargo build --workspace（exit 0）+ `ls src-tauri/target` → No such file or directory"
        status: pass
    human_judgment: false
  - id: D2
    description: "端到端 dev_ping 通路：点击按钮后页面显示由 SQLite 一路取回的真实版本串"
    requirement: "INFRA-01"
    verification:
      - kind: unit
        ref: "crates/prism-engine/src/lib.rs#ping_delegates_to_the_store_rather_than_returning_a_constant"
        status: pass
      - kind: unit
        ref: "src/lib/ipc.test.ts（devPing 断言 invoke 以 'dev_ping' 被调用）"
        status: pass
      - kind: manual_procedural
        ref: "npm run tauri dev → 点击 ping 按钮 → 页面显示三段点分版本串（用户已确认）"
        status: pass
    human_judgment: true
    rationale: "真实 WebView 中的渲染结果只有人眼能确认；自动化只能覆盖到 IPC 边界两侧，跨不过 WebView"
  - id: D3
    description: "D-01 依赖方向：9 个 engine crate 的 normal 依赖树中均不出现 tauri"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "cargo tree -p <每个 crate> --edges normal --prefix none | grep -c '^tauri ' → 全部 0"
        status: pass
    human_judgment: false
  - id: D4
    description: "D-09 无环：prism-mcp 的依赖树中不出现 prism-engine（只依赖 prism-types）"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "cargo tree -p prism-mcp --edges normal --prefix none | grep -c '^prism-engine' → 0"
        status: pass
    human_judgment: false
  - id: D5
    description: "D-10 小依赖面：prismdocs-helper 编译通过且依赖树中无任何 prism-* engine crate"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "cargo tree -p prism-cli --edges normal --prefix none | tail -n +2 | grep -c '^prism-' → 0；ls target/debug/prismdocs-helper 存在"
        status: pass
      - kind: unit
        ref: "crates/prism-cli/src/main.rs#doctor_reports_both_backends_without_touching_the_network"
        status: pass
    human_judgment: false
  - id: D6
    description: "D-08 依赖真进树：六个骨架 crate 的主依赖被真实链接，而非空声明"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "cargo tree -p prism-parse|grep comrak =1；prism-anchor|grep blake3 =1；prism-mcp|grep rmcp =1"
        status: pass
      - kind: unit
        ref: "cargo test -p prism-fs -p prism-parse -p prism-anchor -p prism-llm -p prism-mcp（20 passed）"
        status: pass
    human_judgment: false

# Metrics
duration: 39min
completed: 2026-07-28
status: complete
---

# Phase 1 Plan 01: 单一 workspace 骨架与 dev_ping tracer Summary

**单一 cargo workspace（9 engine crate + Tauri 薄 shell + prismdocs-helper）就位，并用一条可点可见的 `dev_ping` 通路把「shell 薄 / engine 厚 / 依赖单向」这一形状钉死在编译期。**

## Performance

- **Duration:** ≈39 min agent 时间（22:42→23:21），另有两次人工确认门（Task 1 供应链、Task 2 tracer）的等待时间未计入
- **Started:** 2026-07-28T13:42:36Z
- **Completed:** 2026-07-28T14:21:01Z
- **Tasks:** 3
- **Files modified:** 53

## Accomplishments

- **一条真的端到端路，不是原型**：点击按钮 → `invoke('dev_ping')` → `#[tauri::command]` 单行委托 → `Engine::ping` → `Store::sqlite_version` → bundled SQLite → 版本串回到页面。用户在真实 WebView 中确认了三段点分版本串。
- **五项不可逆决策的形状在第一个 commit 就被编译期证据固定**：D-01（engine 无 tauri）、D-08（全 crate 一次定型）、D-09（mcp 无 facade 环）、D-10（CLI helper 小依赖面）、D-13（sidecar 根来自 `dirs::data_dir`）。
- **`[workspace.dependencies]` 一次写全**：Phase 1–8 会用到的 27 个 pin 与 feature 串集中在根 Cargo.toml，`cargo tree -d` 未发现重复 rusqlite / reqwest / libsqlite3-sys。
- **骨架 crate 不是空壳**：六个未到 phase 的 crate 各有一个真正调用主依赖的函数（comrak 数根块、blake3 指纹、rmcp 协议版本、notify 后端名、keychain 后端名），依赖真的进了树，20 个单测覆盖。

## Task Commits

1. **Task 1: npm 依赖合法性确认（供应链门禁）** — 无产物（`checkpoint:human-verify`，`gate="blocking-human"`）。用户核对后批准 `@tanstack/react-query`（TanStack 官方组织、千万级周下载、无 postinstall、5.x）与 `subtle`（dalek-cryptography、千万级下载、2.x）。
2. **Task 2: 端到端 `dev_ping` tracer** — TDD 三段：
   - `4d7c67a` (test) — RED：workspace 骨架 + 三个 crate 的失败单测 + `src/lib/ipc.test.ts`
   - `5460649` (feat) — GREEN：`Store`/`Engine`/`dev_ping`/前端实现，全绿
   - `bc57bf5` (chore) — 把 tauri 生成的 capability schema 移出版本控制
3. **Task 3: 其余六个 crate 骨架与 CLI helper 占位 binary** — `59cb368` (feat)

**Plan metadata:** 见本 commit（docs: complete plan）

## Files Created/Modified

### Task 3 新增（本次会话）

- `crates/prism-fs/{Cargo.toml,src/lib.rs}` — notify + debouncer-full 落点；`watcher_backend_name()` / `debounce_cache_name()` / `FsError`
- `crates/prism-parse/{Cargo.toml,src/lib.rs}` — **comrak 是 Block 边界唯一真相源**；`root_block_count()` / `ParseError`
- `crates/prism-anchor/{Cargo.toml,src/lib.rs}` — `content_fingerprint()`（blake3 hex）/ `AnchorError`；similar + ulid 由单测证明可用
- `crates/prism-llm/{Cargo.toml,src/lib.rs}` — **本 workspace 唯一可声明 reqwest/keyring 的 engine crate**；`user_agent()` / `keychain_backend_name()` / `LlmError`
- `crates/prism-mcp/{Cargo.toml,src/lib.rs}` — 只依赖 prism-types（D-09）；`protocol_version()` / `shared_types_version()` / `McpError`
- `crates/prism-cli/{Cargo.toml,src/main.rs}` — `prismdocs-helper` binary：`help` 打印用法退出 0，`doctor` 做不触网的本机自检
- `Cargo.lock` — 六个 crate 的依赖解析（`crates/*` 通配已覆盖 members，根 Cargo.toml 无需改动）

### Task 2 建立（前序会话，此处仅索引）

`Cargo.toml`、`.gitignore`、`package.json`、`vite.config.ts`、`tsconfig.json`、`index.html`、`src/{main.tsx,App.tsx,lib/ipc.ts,lib/ipc.test.ts}`、`src-tauri/{Cargo.toml,tauri.conf.json,build.rs,src/{main.rs,lib.rs,commands.rs}}`、`crates/prism-{types,store,engine}/`

## Decisions Made

1. **`crates/*` 通配已足够，Task 3 未改根 Cargo.toml。** 计划里 Task 3 的 `<files>` 含 `Cargo.toml`，实际确认通配生效后无需改动 —— 少一次 lockfile 之外的根文件变更。
2. **`protocol_version()` 用 `static` 而非 `const` 承载 `rmcp::model::ProtocolVersion::LATEST`。** 该类型内含 `Cow<'static, str>`（需要 Drop），`const X: &T = &T::CONST` 的常量提升对它不适用；`static` 从不 drop，`as_str(&'static self)` 经生命周期省略即得 `&'static str`。
3. **对暂无自然用途的依赖，用「依赖可用性」单测引用，不提前发明 API。** `prism-llm` 的 serde、`prism-anchor` 的 similar/ulid、`prism-mcp` 的 axum/tokio 都只在 `#[cfg(test)]` 中被触碰。这既满足 D-08「依赖真进树、pin 冲突 Phase 1 暴露」的目的，又不会与 plan 04 / Phase 3 的 API 设计撞车。
4. **不勾选 INFRA-01。** 该需求横跨本 phase 的 7 个 plan（事件总线在 01-04，Channel 有序流在 01-08/01-09），仅凭 plan 01 标记完成会向后续规划发出虚假信号。`requirements-completed: []`，REQUIREMENTS.md 保持 Pending。
5. **`prismdocs-helper` 给了 `doctor` 子命令而非纯打印。** 计划只要求"打印用法后退出 0"（已满足，无参/`help` 即走该路径）；`doctor` 让 `HelperError` 的 reqwest/keyring 变体有真实构造点，否则 `clippy -D warnings` 会因 dead_code 失败。副作用为零：只构造 client、只取类型名，不发请求、不读密钥。

## Deviations from Plan

**None** —— 计划照写执行，无需触发任何 deviation rule。上面「Decisions Made」第 1/3/5 条是计划留给执行者的自由度内的具体选择，不是对计划的偏离。

## Known Stubs

以下六处为 **D-08 明确要求的有意骨架**，不是遗漏，各有指定的填充 plan：

| 位置 | 现状 | 由谁填满 |
|------|------|---------|
| `crates/prism-fs/src/lib.rs` | 只有后端名查询与 `FsError::Watch` | Phase 2（REQ-1.4.3 合并语义、`.prismdocs/` 忽略、10s 呈现预算） |
| `crates/prism-parse/src/lib.rs` | 只有 `root_block_count` | Phase 3（sourcepos 字节区间、frontmatter 边界、Block 切分） |
| `crates/prism-anchor/src/lib.rs` | 只有 `content_fingerprint` | Phase 3（TD-01 三步迁移算法） |
| `crates/prism-llm/src/lib.rs` | 只有 UA 与后端名 | plan 01-04（Keychain 往返）、Phase 4（端点与 SSE） |
| `crates/prism-mcp/src/lib.rs` | 只有协议版本 | plan 01-06（loopback 宿主 + bearer 中间件）、Phase 6（工具注册） |
| `crates/prism-cli/src/main.rs` | `help` / `doctor` 两个子命令 | Phase 6（`headers`、`check-feedback`） |

`prism-store` 刻意未加 WAL / pragma / 迁移 / 只读池 —— 那是 plan 01-03 的 writer-first 六步序，顺序有语义，不能在 tracer 阶段提前猜。

**未记入 `.planning/WINDOWS.md`**：该文件不存在，且以上均为计划内的架构占位（有指定填充 plan、有解释性注释），不是缺陷。把它们写进 ship 门禁会产生 6 条永不关闭的噪声条目。

## Issues Encountered

**None.** Task 3 一次编译通过，一次 clippy 通过，20 个新单测首跑全绿，5 条依赖方向断言全部符合预期。

前序会话（Task 2）的 tracer 人工确认门已由用户以 "verified" 关闭。

## Verification Evidence

```
cargo build --workspace                                     → exit 0
cargo clippy --workspace --all-targets -- -D warnings       → exit 0
cargo test -p prism-types -p prism-store -p prism-engine    → 9 passed（选择集不含 tauri）
cargo test -p prism-fs -p prism-parse -p prism-anchor \
           -p prism-llm -p prism-mcp                        → 20 passed
cargo test -p prism-cli                                     → 4 passed
npm run test -- --run                                       → 3 passed
npm run build                                               → dist/index.html 生成
ls src-tauri/target                                         → No such file or directory（产物在根 target/）
git status --porcelain | grep -E 'target/|node_modules/'    → 空

# 依赖方向（D-01 / D-09 / D-10）—— 不用恒真的 --workspace 构建冒充证据
cargo tree -p <9 个 engine crate> --edges normal | grep -c '^tauri '        → 全部 0
cargo tree -p prism-mcp  --edges normal | grep -c '^prism-engine'           → 0
cargo tree -p prism-cli  --edges normal | tail -n +2 | grep -c '^prism-'    → 0
cargo tree -d --edges normal | grep -E '^(rusqlite|reqwest|libsqlite3-sys)' → 空（无重复）

# D-08 依赖真进树
cargo tree -p prism-parse  | grep -c '^comrak '  → 1
cargo tree -p prism-anchor | grep -c '^blake3 '  → 1
cargo tree -p prism-mcp    | grep -c '^rmcp '    → 1

./target/debug/prismdocs-helper          → 打印用法，exit 0
./target/debug/prismdocs-helper doctor   → http backend: ok / keychain backend: apple_native_keyring_store::keychain::Store
```

## Self-Check

见文末 `## Self-Check` 段。

## User Setup Required

None —— 本 plan 未引入需要外部服务配置的依赖。API key 与钥匙串写入是 plan 01-04 的内容。

## Next Phase Readiness

**已就绪，可并行开工的下游 plan：**

- **01-02（依赖方向断言脚本 / justfile + CI）** —— 本 plan 已把四条断言的**命令形态**跑通并记录在上面的 Verification Evidence 中，01-02 只需把它们固化成 `check-deps.sh` 并接进 CI。
- **01-03（writer-first 启动序列 + migration 001）** —— `Store::open` 留的空位正对着六步序；FTS5 由 `rusqlite` 的 `bundled` 保证（bundled SQLite 3.53.2 ≥ 硬要求 3.51.3）。注意 migration 001 与 `STRICT` / external-content 的 `checkpoint:decision` 仍待用户拍板。
- **01-04（service trait + 事件总线）** —— `prism-types` 目前只有 `CRATE_VERSION` + `CrateInfo`，是 trait 反转的干净落点；`prism-mcp` 已经只依赖它。
- **01-06（MCP loopback 宿主）** —— rmcp 2.2 的 `server` + `transport-streamable-http-server` feature 串已核实可解析、可编译；`subtle` 已在 workspace pin 中且经 Task 1 人工确认。

**需要注意的两点：**

1. `~/Library/Application Support/PrismDocs/app.db`（含 `-wal`/`-shm`）是已归档的 Phase 1 旧尝试遗留，**不是本 plan 的库**。本 plan 的 sidecar 库是同目录下的 `prismdocs.db`。不要从 `app.db` 读或迁移。
2. INFRA-01 仍是 Pending —— 本 plan 只交付了它的骨架部分，事件总线与 Channel 有序流两条通路要到 01-04/01-08/01-09 才补齐。

---
*Phase: 01-foundation-skeleton*
*Completed: 2026-07-28*

## Self-Check

**PASSED**

- 六个新 crate 的 12 个文件全部存在于工作树（`crates/prism-{fs,parse,anchor,llm,mcp,cli}/`）
- 四个 commit 全部可在 `git log` 中找到：`4d7c67a`、`5460649`、`bc57bf5`、`59cb368`
- `git diff --diff-filter=D HEAD~1 HEAD` 为空 —— Task 3 的 commit 未删除任何被跟踪文件
