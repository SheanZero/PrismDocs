---
phase: 01-foundation-skeleton
plan: 02
subsystem: infra
tags: [cargo-tree, dependency-assertions, ci, github-actions, coverage, secret-scan]

# Dependency graph
requires:
  - "01-01：九个 engine crate + prismdocs-shell + prism-cli 的包名与依赖面（断言的受检对象）"
provides:
  - "scripts/check-deps.sh：四条依赖方向断言（dup / tauri-free / no-cycle / single-egress / all）的唯一实现"
  - "scripts/check-secrets.sh：受版本控制文件的明文密钥静态检查"
  - "justfile：八条委托型 recipe（check-* 五条 + check-all + test-engine + coverage）"
  - ".github/workflows/ci.yml：macOS runner 上的 engine / shell / frontend 三 job 工作流"
  - "两侧覆盖率工具链就位（cargo-llvm-cov + @vitest/coverage-v8），数字进 CI step summary"
affects: [01-03, 01-04, 01-06, 01-08, 01-09, phase-2-覆盖率硬闸门]

# Tech tracking
tech-stack:
  added:
    - "@vitest/coverage-v8 4.1.10（前端覆盖率 provider）"
    - "cargo-llvm-cov 0.8.7 + rustup component llvm-tools-preview（engine 覆盖率，本机与 CI 各自安装）"
    - "GitHub Actions: actions/checkout@v4, actions/cache@v4, actions/setup-node@v4, actions/upload-artifact@v4, dtolnay/rust-toolchain@stable, taiki-e/install-action@cargo-llvm-cov"
  patterns:
    - "断言逻辑单点实现：脚本是主形式，justfile 与 CI 都只是调用者——两份实现必然漂移"
    - "cargo tree 断言用 herestring 而非管道喂 grep：`cmd | grep -q` 在 pipefail 下会因 SIGPIPE 产生假阴性"
    - "每条断言都配一个反证：注入违规依赖后必须变红，否则断言可能恒真"
    - "覆盖率 include 显式钉住分母（src/**），否则未被测试导入的文件静默消失、数字虚高到 100%"

key-files:
  created:
    - scripts/check-deps.sh
    - scripts/check-secrets.sh
    - justfile
    - .github/workflows/ci.yml
  modified:
    - package.json
    - package-lock.json
    - vite.config.ts
    - .gitignore

key-decisions:
  - "断言用 herestring（`grep -q <<<\"$out\"`）而不是管道：pipefail + grep -q 早退会让 cargo tree 收到 SIGPIPE，管道状态变 141，if 判定为「无命中」——这是一个会让四条断言全部静默失效的假阴性"
  - "vitest coverage 加 include: src/**：不加时分母只含被测试导入过的文件，首跑给出 100%（1/1 statement）的无意义数字；加上后是诚实的 10%"
  - "CI 用 `cargo llvm-cov --no-report` + 两次 `report`：一次插桩运行同时产出摘要与 lcov，避免为两种输出格式跑两遍测试"
  - "覆盖率 Phase 1 只测量不设阈值，Phase 2 开硬闸门（口径见下文交接段）"

patterns-established:
  - "任何新增的架构约束，交付形态是「脚本断言 + 反证」而不是文档段落"
  - "CI 中凡是 cfg-gated 的测试目标，命令必须带上对应 feature flag——否则 job 绿灯但零测试"

requirements-completed: []

coverage:
  - id: D1
    description: "无重复 rusqlite / reqwest / libsqlite3-sys（同进程不链接两份 SQLite/HTTP 栈）"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "bash scripts/check-deps.sh dup → exit 0, `OK: no duplicate rusqlite/reqwest/libsqlite3-sys`"
        status: pass
    human_judgment: false
  - id: D2
    description: "D-01：八个 engine crate + prism-cli 的 normal+build 依赖树中均无 tauri"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "bash scripts/check-deps.sh tauri-free → exit 0；反证：prism-cli 注入 tauri 后 exit 1 + `FAIL: prism-cli depends on tauri`"
        status: pass
    human_judgment: false
  - id: D3
    description: "D-09：prism-mcp 的 normal 依赖树中无 prism-engine（dev 边被显式排除）"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "bash scripts/check-deps.sh no-cycle → exit 0, `OK: prism-mcp -> prism-types only`"
        status: pass
    human_judgment: false
  - id: D4
    description: "NFR-03：六个纯 crate 无 reqwest / keyring-core / apple-native-keyring-store"
    requirement: "INFRA-03"
    verification:
      - kind: integration
        ref: "bash scripts/check-deps.sh single-egress → exit 0；反证：prism-store 注入 reqwest 后 exit 1 + `FAIL: prism-store has network/secret dependency`"
        status: pass
    human_judgment: false
  - id: D5
    description: "T-01-03a：受版本控制文件中无明文密钥"
    requirement: "INFRA-03"
    verification:
      - kind: integration
        ref: "bash scripts/check-secrets.sh → 无输出 exit 0；反证：commands.rs 注入 sk-<24 位> 后 exit 1 并打印命中行"
        status: pass
    human_judgment: false
  - id: D6
    description: "四条断言 + 密钥检查在 CI（macOS runner）上随每次 push 自动执行"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: ".github/workflows/ci.yml 通过八项结构断言（macos / 无 shell 子目录缓存路径 / 无 cargo test --workspace / 两个脚本调用 / shell job 带 --features test / 两侧覆盖率步骤）+ PyYAML 解析出 jobs 恰为 engine,shell,frontend"
        status: pass
      - kind: manual_procedural
        ref: "工作流在真实 GitHub Actions runner 上的首跑（仓库尚无 remote，本 plan 内无法执行）"
        status: deferred
    human_judgment: true
    rationale: "YAML 结构与命令正确性可静态断言且已断言；runner 上的实际执行要等仓库推上远端后才有第一次反馈"
  - id: D7
    description: "两侧覆盖率被真实测量并输出数字"
    requirement: "INFRA-01"
    verification:
      - kind: integration
        ref: "cargo llvm-cov --workspace --summary-only → TOTAL 85.48% lines；npm run test -- --run --coverage → 10% statements"
        status: pass
    human_judgment: false

# Metrics
duration: 8min
completed: 2026-07-28
status: complete
---

# Phase 1 Plan 02: 依赖方向断言与 CI 骨架 Summary

**D-01 / D-09 / NFR-03 三条依赖图性质从口头约定变成两个脚本 + 一个 macOS 工作流：四条 `cargo tree` 断言全绿，且每条都用注入违规依赖的反证证明了它不是恒真的。**

## Performance

- **Duration:** ≈8 min
- **Started:** 2026-07-28T14:26:36Z
- **Completed:** 2026-07-28T14:34:xxZ
- **Tasks:** 2
- **Files modified:** 8（4 新建 + 4 修改）

## Accomplishments

- **四条断言 + 密钥检查全部可执行、全部绿**，并且**三条反证都真的跑了**：prism-cli 注入 `tauri` → tauri-free 红；prism-store 注入 `reqwest` → single-egress 红；`commands.rs` 注入 `sk-` 长串 → check-secrets 红；三次 `git checkout --` 撤销后各自复绿。断言不是恒真的，这一点有执行证据而不是断言。
- **断言逻辑只有一份实现。** `justfile` 八条 recipe 里的五条 check-* 全是单行 `bash scripts/…` 委托；CI 也直接调脚本、不经 `just`（runner 不预装）。本机同样没装 `just`——主形式是脚本这件事因此不是纸面约定，而是唯一跑得通的形式。
- **CI 把三个已知的静默失败模式堵上了**：shell job 带 `--features test`（否则 `tests/ipc.rs` 因 `#![cfg(feature = "test")]` 编译为零测试、绿灯却什么都没测）；engine job 用 8 个 `-p` 而非 `--workspace`（后者会编译 shell，证明不了 D-01）；缓存指向仓库根 `target/`（单一 workspace 下 shell 子目录的构建目录已不存在，写错会每次全量重编）。
- **两侧覆盖率数字第一次落地**：engine 85.48% lines / 87.01% regions / 83.12% functions；前端 10% statements。

## Task Commits

1. **Task 1: 四条依赖方向断言 + 委托型 justfile** — `1f98e6d` (feat)
   `scripts/check-deps.sh`（105 行）+ `justfile`（8 recipe）
2. **Task 2: 明文密钥静态检查 + macOS CI 工作流** — `d10bb3b` (feat)
   `scripts/check-secrets.sh`、`.github/workflows/ci.yml`、`package.json`/`package-lock.json`/`vite.config.ts`/`.gitignore` 增量

**Plan metadata:** 见本 commit（docs: complete plan）

## Files Created/Modified

- `scripts/check-deps.sh` — 断言唯一实现。三个集合变量：`ENGINE_CRATES`（8 个，也是 `test-engine` 的选择集）、`TAURI_FREE_CRATES`（= engine + `prism-cli`）、`PURE_CRATES`（6 个，排除被允许触网/触密钥的 prism-llm 与 prism-cli）。子命令 `dup` / `tauri-free` / `no-cycle` / `single-egress` / `all`，无参等同 `all`。
- `scripts/check-secrets.sh` — `git grep -nE` 两类形态（`sk-[A-Za-z0-9]{16,}`；`api[_-]?key` 后跟引号串赋值），排除 `.planning/`、`docs/` 与脚本自身。
- `justfile` — 八条 recipe：`check-dup` / `check-tauri-free` / `check-no-cycle` / `check-single-egress` / `check-secrets` / `check-all` / `test-engine` / `coverage`。`coverage` 上方注释写明两个前置安装。
- `.github/workflows/ci.yml` — `on: [push, pull_request]`，三个 job 全在 `macos-latest`。
- `package.json` / `package-lock.json` — devDependencies 增 `@vitest/coverage-v8 ^4.1.10`。
- `vite.config.ts` — vitest `coverage`: provider v8、reporter `["text","lcov"]`、`include: ["src/**/*.{ts,tsx}"]`。
- `.gitignore` — 增 `coverage/` 与 `lcov.info`（覆盖率产物，否则每次跑完留一堆未跟踪文件）。

## Decisions Made

1. **断言用 herestring 而非管道喂 grep。** RESEARCH 给的示例形态是 `cargo tree … | grep -Eq …`。在 `set -euo pipefail` 下这有一个会让**四条断言全部静默失效**的假阴性：`grep -q` 命中后立即退出，`cargo tree` 收到 SIGPIPE 以 141 结束，`pipefail` 让整条管道状态变成 141，`if` 于是判定为「无命中」→ 有违规也报 OK。改成先 `out=$(cargo tree …)` 再 `grep -Eq … <<<"$out"`，没有管道就没有这个问题。三条反证同时也是对这个改动的验证——如果假阴性还在，反证不会变红。
2. **vitest `coverage.include` 钉住分母。** 计划只要求配 provider 与 reporter。照配后首跑输出 `Statements: 100% (1/1)`、文件表为空——v8 默认只统计被测试导入过的文件，`App.tsx` / `main.tsx` 直接从分母里消失。一个恒为 100% 的数字比没有数字更坏（它会让人以为前端测够了）。加上 `include: ["src/**/*.{ts,tsx}"]` 后是 10%，诚实。
3. **CI 覆盖率用 `--no-report` + 两次 `report`。** 摘要与 lcov 两种输出如果各跑一次 `cargo llvm-cov --workspace`，等于插桩编译加跑全套测试两遍。`cargo llvm-cov --no-report --workspace` 跑一次，再 `report --summary-only` 与 `report --lcov` 各出一份。本机验证用的仍是 `justfile` 里那条 `cargo llvm-cov --workspace --summary-only`（单条命令、给人用）。
4. **`on: [push, pull_request]` 用流式序列写法。** 块式写法下 `on:` 里的 `push:` 与 job 名同为两格缩进，任何按缩进抽 job 名的结构断言都会把 `push` 误判成第四个 job。流式序列消除这个歧义（PyYAML 解析确认 jobs 恰为 `engine,shell,frontend`）。
5. **`.gitignore` 增 `coverage/` 与 `lcov.info`。** 覆盖率产物，不加会在每次跑完后留下未跟踪文件。

## 覆盖率：Phase 1 只测量不设阈值，Phase 2 开硬闸门

这是本 plan 必须原样交接给 Phase 2 的一条决定，写在这里是为了让「Phase 2 再开」不会变成「永远不开」。

**全局规则是 80% 硬下限**（`~/.claude/rules/common/testing.md`）。Phase 1 不设阈值，理由具体到本阶段三块可核验的事实：

1. **生成式样板计入分母但没有可测行为。** 本次实测：`src-tauri/src/lib.rs` 0%（30 regions）、`src-tauri/src/main.rs` 0%（3 regions）、`src-tauri/src/commands.rs` 0%（8 regions）。commands.rs 的 0% 尤其说明问题——它的测试在 `tests/ipc.rs` 里，而该文件由 plan 01-08 加了 `#![cfg(feature = "test")]`，`cargo llvm-cov --workspace` 与 `cargo test --workspace` 一样在无 feature 下把它编译成**零个测试**。这两项由 CI 的 `shell` job（带 `--features test`）单独覆盖，但不进覆盖率数字。
2. **真实钥匙串往返路径按设计是 `#[ignore]` 的**（`01-VALIDATION.md` § Manual-Only Verifications：CI/headless 无解锁钥匙串，`keyring_core::set_default_store` 还是进程全局的），永远不会被 CI 覆盖。
3. **经真实 WebView 的事件到达与 Channel 有序性同样按设计是人工验证**——`tauri::test` 的 mock runtime 没有 WebView。

在这三块之上压 80% 硬闸门，唯一的产出是为凑数字而写的空测试，那比没有闸门更糟。

**交接给 Phase 2 的具体动作：** 在 `.github/workflows/ci.yml` 的 Coverage 步骤加 `--fail-under-lines`（engine 侧）与 `coverage.thresholds`（前端侧），口径是**「排除已登记的人工/`#[ignore]` 路径后 ≥80%」**——那时 Phase 1 已经把人工验证面在 `01-VALIDATION.md` § Manual-Only Verifications 里界定清楚，排除集是可枚举的，不是拍脑袋。

**本次实测的两个数字（Phase 2 定阈值时的基线）：**

| 侧 | 命令 | 结果 |
|----|------|------|
| engine | `cargo llvm-cov --workspace --summary-only` | lines **85.48%** (310 行 / 45 未覆盖)、regions 87.01%、functions 83.12% |
| 前端 | `npm run test -- --run --coverage` | statements **10%** (1/10)、branches 0% (0/8)、functions 33.33% (1/3) |

**两条本机前置条件**（都不随 rustup / 仓库自带，已写进 `justfile` 中 `coverage` recipe 上方的注释）：

- `cargo install cargo-llvm-cov`（本次装的是 0.8.7）**加** `rustup component add llvm-tools-preview`——第二条计划里没提，但没有它 `cargo llvm-cov` 直接报错。CI 侧由 `taiki-e/install-action@cargo-llvm-cov` + `dtolnay/rust-toolchain@stable` 的 `components: llvm-tools-preview` 覆盖。
- `npm ci`（装上 `@vitest/coverage-v8`）。

## Deviations from Plan

### 自动修复 / 补齐

**1. [Rule 1 - Bug] 断言的管道形态在 pipefail 下有假阴性，改为 herestring**

- **Found during:** Task 1
- **Issue:** RESEARCH 示例的 `cargo tree … | grep -Eq …` 在 `set -euo pipefail` 下，grep 早退触发 SIGPIPE，管道退出码变 141，`if` 判为无命中——四条断言可能全部恒绿。
- **Fix:** 先把 `cargo tree` 输出存入变量，再用 herestring 喂 grep。
- **Files modified:** `scripts/check-deps.sh`
- **Commit:** `1f98e6d`

**2. [Rule 2 - 缺失的正确性要求] vitest coverage 补 `include`**

- **Found during:** Task 2
- **Issue:** 按计划字面配置后覆盖率恒为 100%（分母 1 个 statement），是个假指标。
- **Fix:** `include: ["src/**/*.{ts,tsx}"]`。
- **Files modified:** `vite.config.ts`
- **Commit:** `d10bb3b`

**3. [Rule 3 - 阻塞] `rustup component add llvm-tools-preview`**

- **Found during:** Task 2
- **Issue:** 计划只写了 `cargo install cargo-llvm-cov`；装完后 `rustup component list --installed | grep llvm-tools` 为空，`cargo llvm-cov` 无法运行。
- **Fix:** 本机 `rustup component add llvm-tools-preview`；CI 侧在 `dtolnay/rust-toolchain@stable` 的 `components` 里加 `llvm-tools-preview`。
- **Files modified:** `.github/workflows/ci.yml`（+ 本机 toolchain）
- **Commit:** `d10bb3b`

**4. [Rule 2] `.gitignore` 增覆盖率产物条目** — `coverage/`、`lcov.info`。Commit `d10bb3b`。

### 计划文字与实现的两处措辞调整

- **CI 注释不能出现 shell 子目录的构建目录字面量。** 计划的验收门 `node -e` 断言整个 `ci.yml` 不匹配 `/src-tauri\/target/`——包括注释。首版注释里写了那个路径解释「为什么不缓存它」，门直接红。改写成不含该字面量的表述。这不是偏离，是验收门比计划正文严格一格。
- **`tauri-free` 的成功消息不含任何 `prism-` 名。** 验收门要求成功时 stdout 不出现 `prism-` 开头的 crate 名（失败时才打印违规者），所以消息是 `OK: all checked crates are tauri-free (engine set + CLI helper)` 而不是列出集合。

## Known Stubs

**None.** 本 plan 的产物是断言与工作流，无占位实现。

`.planning/WINDOWS.md` 不存在，本 plan 也无需追加条目——四条断言与密钥检查各有反证证明有效，无 stub / 跳过的测试 / 未跑的 verify。

唯一一项**未在本 plan 内闭环**的是 D6 的第二条：工作流在真实 GitHub Actions runner 上的首跑。仓库当前没有 remote，无法触发。YAML 结构与全部命令已静态断言通过；首次推远端后应确认三个 job 全绿（尤其 `shell` job 的测试计数不为 0——那正是 `--features test` 要防的失败模式）。

## Threat Flags

无新增安全面。本 plan 的四条断言 + 密钥检查正是 `<threat_model>` 中 T-01-03a / T-01-09a / T-01-16 / T-01-17 / T-01-18 五项的 `mitigate` 落地，全部已实现并有反证。

## Issues Encountered

- **`cargo llvm-cov` 首次运行报缺 llvm-tools**（见 Deviation 3），装组件后正常。
- **前端覆盖率首跑给出无意义的 100%**（见 Decision 2），加 `include` 后修正。
- 除此之外无阻塞。

## Verification Evidence

```
bash -n scripts/check-deps.sh                                   → exit 0
bash -n scripts/check-secrets.sh                                → exit 0

bash scripts/check-deps.sh dup            → OK: no duplicate rusqlite/reqwest/libsqlite3-sys
bash scripts/check-deps.sh tauri-free     → OK: all checked crates are tauri-free (engine set + CLI helper)
bash scripts/check-deps.sh no-cycle       → OK: prism-mcp -> prism-types only
bash scripts/check-deps.sh single-egress  → OK: prism-llm is the sole network+secret crate among engine crates
bash scripts/check-deps.sh all            → 四行 OK，exit 0
bash scripts/check-secrets.sh             → 无输出，exit 0

# 反证（断言非恒真）
crates/prism-cli/Cargo.toml   += tauri    → tauri-free    exit 1, "FAIL: prism-cli depends on tauri"
crates/prism-store/Cargo.toml += reqwest  → single-egress exit 1, "FAIL: prism-store has network/secret dependency"
src-tauri/src/commands.rs     += sk-<24>  → check-secrets exit 1, 打印 commands.rs:10 命中行
三次 git checkout -- <file> 撤销后         → 各自重新 exit 0

# 结构断言
grep -v '^#' scripts/check-deps.sh | grep -c 'edges normal'                        → 4  (≥4)
sed 's/#.*//' scripts/check-deps.sh | grep 'TAURI_FREE_CRATES=' | grep -c prism-cli → 1  (≥1)
sed 's/#.*//' scripts/check-deps.sh | grep 'PURE_CRATES='      | grep -c prism-cli → 0
bash scripts/check-deps.sh tauri-free | grep -c 'prism-'                           → 0
justfile test-engine 行的 '-p ' 计数 / 'workspace' 计数                             → 8 / 0
justfile recipe 名                → check-dup check-tauri-free check-no-cycle check-single-egress
                                     check-secrets check-all test-engine coverage（恰 8 条）
check-* recipe 实现体含 'scripts/' 的行数                                          → 6

# ci.yml 八项门（Task 2 <verify>）
macos ✓ / src-tauri-target ✗ / check-deps.sh ✓ / check-secrets.sh ✓ /
cargo test --workspace ✗ / cargo test -p prismdocs-shell --features test ✓ /
cargo llvm-cov ✓ / npm run test -- --run --coverage ✓                              → "ci.yml OK"
python3 yaml.safe_load(ci.yml) → jobs == ['engine','shell','frontend'], on == ['push','pull_request']

# 覆盖率
cargo llvm-cov --workspace --summary-only  → TOTAL lines 85.48% / regions 87.01% / functions 83.12%
npm run test -- --run --coverage           → 3 passed；Statements 10% (1/10)，exit 0

git diff --diff-filter=D --name-only HEAD~2 HEAD  → 空（两个 commit 未删除任何被跟踪文件）
git status --short                                → 无本 plan 产生的未跟踪/未提交残留
```

## User Setup Required

本 plan 在本机装了两样东西（其他机器上重现需要同样两条）：

```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview
```

`npm ci` 已覆盖前端侧的 `@vitest/coverage-v8`。CI runner 由工作流自行安装，无需人工。

**推远端后的一次性确认**：CI 首跑三个 job 是否全绿，且 `shell` job 的测试计数不为 0。

## Next Phase Readiness

- **每个后续 plan 的 per-wave 门禁已经就位**：`bash scripts/check-deps.sh all && bash scripts/check-secrets.sh`。plan 01-03 起新增任何依赖，违反 D-01 / D-09 / NFR-03 会立刻红。
- **01-04（service trait + 事件总线）** 需注意：给 `prism-mcp` 加 `prism-engine` 的 **dev**-dependency 也会被 `no-cycle` 挡住的**只是普通边**——dev 边本就是允许的逃逸口，断言用 `--edges normal` 显式排除它正是为了让「dev 环合法但普通环非法」这条规则可执行。要验证 trait 反转本身，靠的是 `crates/prism-mcp/tests/trait_injection.rs`，不是这条断言。
- **01-08（`tests/ipc.rs` cfg 门）** 与本 plan 的 CI `shell` job 是配套的：那个 plan 加 `#![cfg(feature = "test")]`，本 plan 的 job 加 `--features test`。两边缺一边就会出现「绿灯零测试」。
- **Phase 2 第一件事**：按上文交接段给两侧加阈值。

---
*Phase: 01-foundation-skeleton*
*Completed: 2026-07-28*

## Self-Check

**PASSED**

- 4 个新建文件全部存在：`scripts/check-deps.sh`、`scripts/check-secrets.sh`、`justfile`、`.github/workflows/ci.yml`
- 2 个 commit 均可在 `git log` 中找到：`1f98e6d`、`d10bb3b`
- `git diff --diff-filter=D --name-only HEAD~2 HEAD` 为空——未删除任何被跟踪文件
- 全部断言在写 SUMMARY 前重跑一次确认仍绿（`check-deps.sh all` exit 0、`check-secrets.sh` exit 0）
