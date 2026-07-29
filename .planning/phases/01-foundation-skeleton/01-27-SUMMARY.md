---
phase: 01-foundation-skeleton
plan: 27
subsystem: infra
tags: [ci, github-actions, clippy, coverage, eslint, tsconfig, cache, permissions, concurrency]

requires:
  - phase: 01-foundation-skeleton
    provides: "01-14..01-24 的十二份缺口修复（clippy 可扩面的前提：受检面本身已无告警）"
  - phase: 01-foundation-skeleton
    provides: "01-26 的 eslint.config.js 与 package.json 的 lint script（本 plan 只做 CI 接线）"
provides:
  - "clippy -D warnings 覆盖仓库里全部 Rust：八个 engine crate + prism-cli（engine job）+ prismdocs-shell --all-targets --features test（shell job）"
  - "prism-cli 进入 engine job 的 cargo test 步骤，成为声明的闸门而非 coverage 的副作用"
  - "coverage 范围收窄到八个 engine crate，与 `### Engine coverage` 标题同范围；前置 `cargo llvm-cov clean --workspace` 使范围断言成立"
  - "CI workflow 顶层 permissions: contents: read 与 concurrency 分组（head_ref || ref_name）"
  - "engine / shell 两个 job 的缓存 key 与 restore-keys 互不为前缀"
  - "frontend job 的 npm run lint 步骤，排在测试之前；lint script 加 --max-warnings 0"
  - "tsconfig.json 移除与真实设置相反的 vitest/globals 类型声明"
  - "01-VERIFICATION.md 的两项人工验证步骤按 01-21 / 01-24 的交接改写（CSP 五步→七步、日志 sink 两步重写）"
affects: [01-28, phase-02, ci]

tech-stack:
  added: []
  patterns:
    - "CI 步骤的范围必须与它自己的标题是同一个集合（否则 Phase 2 的硬闸门基线会挂在错的读数上）"
    - "`cargo llvm-cov report` 汇总它在 target/ 里找到的全部对象，不是本次 --no-report 点名的那些——缓存 target/ 的 CI 必须先 clean，否则范围断言不成立"
    - "concurrency 分组用 `head_ref || ref_name`（不是 github.ref）才能真的合并 push 与 pull_request 两个事件"
    - "每个 CI job 的 feature 面与它的缓存分段一一对应；跨 feature 面的步骤要放进对应 job，不要塞进邻居"

key-files:
  created: []
  modified:
    - .github/workflows/ci.yml
    - justfile
    - tsconfig.json
    - package.json
    - .planning/phases/01-foundation-skeleton/01-VERIFICATION.md

key-decisions:
  - "壳的 clippy 落在 shell job 而不是 plan 指定的 engine job：放进 engine job 会让它编译壳的 test feature 面，与同一 plan Task 2 要建立的「两个 job 编译不同 feature 集」前提直接矛盾"
  - "coverage 范围 = 八个 engine crate，prism-cli 刻意不进（按 D-10 它不是 engine crate），它的测试由声明的 cargo test 步骤看住"
  - "coverage 步骤前置 `cargo llvm-cov clean --workspace`：不 clean 时缓存 target/ 里的残留对象会让 src-tauri/src/* 以 0.00% 混进 TOTAL（本机实测）"
  - "concurrency group 用 `github.head_ref || github.ref_name`，不用 plan 字面写的 `github.ref`——后者在 push 与 pull_request 两个事件上永不相等，消不掉双跑"
  - "`--max-warnings 0` 加在 package.json 的 lint script 而不是 CI 命令行：断言只有一处实现，本机与 CI 同强度"
  - "rustfmt 一字未动（决策在 01-28）；`just` 未安装，scripts/ 仍是每条断言的唯一实现"

patterns-established:
  - "非恒真反证要同时跑「旧命令」与「新命令」：同一处注入下旧命令退出 0、新命令退出非零，才证明扩面本身是那条差别"
  - "多 flag 的闸门要逐 flag 反证：`--all-targets` 与 `--features test` 四种组合里只有全带的那一种能抓到 tests/ipc.rs 的告警"

requirements-completed: [INFRA-01]

coverage:
  - id: D1
    description: "clippy -D warnings 覆盖 prism-cli"
    requirement: "INFRA-01"
    verification:
      - kind: automated
        ref: "cargo clippy -p prism-types … -p prism-engine -p prism-cli -- -D warnings（注入 needless_return 后退出 101；旧的八 crate 命令对同一注入退出 0）"
        status: pass
    human_judgment: false
  - id: D2
    description: "clippy -D warnings 覆盖 prismdocs-shell 的 tests/ipc.rs（--all-targets --features test）"
    requirement: "INFRA-01"
    verification:
      - kind: automated
        ref: "cargo clippy -p prismdocs-shell --all-targets --features test -- -D warnings（四组合反证：只有全带 flag 的那一种退出 101）"
        status: pass
    human_judgment: false
  - id: D3
    description: "prism-cli 进入 engine job 的 cargo test 步骤"
    requirement: "INFRA-01"
    verification:
      - kind: unit
        ref: "cargo test -p prism-cli（4 passed）"
        status: pass
    human_judgment: false
  - id: D4
    description: "coverage 范围与 `### Engine coverage` 标题一致（八个 engine crate，clean 前置）"
    requirement: "INFRA-01"
    verification:
      - kind: automated
        ref: "cargo llvm-cov clean --workspace && cargo llvm-cov --no-report -p ×8 && report --summary-only → 22 个文件全部在 crates/ 下，src-tauri 出现 0 次"
        status: pass
    human_judgment: false
  - id: D5
    description: "frontend job 跑 npm run lint 且排在测试之前"
    requirement: "INFRA-01"
    verification:
      - kind: automated
        ref: "npm run lint（注入 console.log 后退出 1；同一注入下 npx tsc --noEmit 退出 0，证明它与 build 的 tsc 闸门不重复）"
        status: pass
    human_judgment: false
  - id: D6
    description: "tsconfig.json 不再声明 vitest globals"
    requirement: "INFRA-01"
    verification:
      - kind: automated
        ref: "npx tsc --noEmit（0 error）+ npm run test -- --run（75 passed）+ npm run build（0）"
        status: pass
    human_judgment: false
  - id: D7
    description: "workflow 卫生：permissions / concurrency / 缓存 key 不互为前缀"
    requirement: "INFRA-01"
    verification:
      - kind: other
        ref: "python3 yaml.safe_load 退出 0；grep -n 'cargo-' 逐行核对"
        status: pass
    human_judgment: true
    rationale: "这三项的**运行期**行为在本机不可观测：upload-artifact 在 contents: read 下是否仍能上传、concurrency 是否真的取消掉同 commit 的第二次跑、两个缓存分段是否真的互不恢复——只有在 GitHub Actions 上实跑一次才有证据。而 `gh run list` 返回空、origin/main 停在 4cc1347（整个 Phase 1 尚未推送），这个 workflow 至今一次都没跑过"
  - id: D8
    description: "01-21 / 01-24 交接的两项人工验证步骤改写"
    verification: []
    human_judgment: true
    rationale: "改写的是人工验证步骤本身；步骤是否可执行、以及 CSP 与日志 sink 的真实行为，只能在真实 WebView / tauri dev 终端里由人确认（WINDOWS id=8 / id=9）"

duration: 76min
completed: 2026-07-30
status: complete
---

# Phase 01 Plan 27: CI 闸门收口 Summary

**把前十二份 gap-closure plan 的修复钉进每次 push 会重跑的闸门：仓库里再没有无 clippy 闸门的 Rust、也没有无 lint 闸门的前端；coverage 的范围第一次与它自己的标题是同一个集合。**

## Performance

- **Duration:** 约 76 min
- **Started:** 2026-07-30T22:19Z
- **Completed:** 2026-07-30T23:35Z
- **Tasks:** 3 完成（plan 定义 2 个 + 汇总任务 1 个）
- **Files modified:** 5

## Accomplishments

- **clippy 扩面到全部 Rust。** engine job 的 clippy 与 test 两步各加 `-p prism-cli`；shell job 新增一条 `cargo clippy -p prismdocs-shell --all-targets --features test -- -D warnings` 并给它的 toolchain 补 `components: clippy`。此前 `src-tauri/src/` 下四个文件与 `crates/prism-cli/src/main.rs`——承载 IPC 边界与未来 `externalBin` 的那两个 crate——是仓库里唯一没有 lint 闸门的 Rust。
- **coverage 的范围与标题第一次对齐**，并顺带发现范围断言还差一步（`clean`，见下）。
- **workflow 卫生三项**：顶层 `permissions: contents: read`、`concurrency` 分组、engine 缓存分段。
- **前端 lint 进闸门**，排在测试之前；`lint` script 加 `--max-warnings 0`（01-26 的交接项）。
- **tsconfig 的死配置移除**：`types` 里的 `vitest/globals` 声明的是与真实设置**相反**的事。
- **三份 plan 的交接全部落地**（01-21 的日志步骤、01-24 的 CSP 五步→七步、01-26 的 lint 接线）。

## Task Commits

1. **Task 1: clippy 与 test 覆盖全部 Rust，coverage 范围与标题对齐** — `c56c974` (ci)
2. **Task 2: workflow 卫生、tsconfig 死配置，并接上前端 lint** — `6e8702e` (ci)
3. **Task 3（汇总任务）: 01-21 / 01-24 交接的两项人工验证步骤改写** — `e6db6c4` (docs)

## Files Created/Modified

- `.github/workflows/ci.yml` — clippy 扩面（两处）、test 扩面、coverage 收窄 + clean 前置、顶层 `permissions` / `concurrency`、engine 缓存分段、frontend lint 步骤、shell toolchain 补 clippy 组件
- `justfile` — `test-engine` 加 `prism-cli`；`coverage` 同步收窄并加 `clean`；新增 `clippy-all`
- `tsconfig.json` — `types` 移除 `vitest/globals`，并写明为什么刻意不要它
- `package.json` — `lint` script 加 `--max-warnings 0`
- `.planning/phases/01-foundation-skeleton/01-VERIFICATION.md` — 两项人工验证步骤改写（frontmatter + 正文两处同步）

## 新的 coverage 基线

改后的 coverage 步骤（**逐字**，plan 要求贴出的「改后的两行」在此，范围行与标题行）：

```
cargo llvm-cov --no-report -p prism-types -p prism-store -p prism-fs -p prism-parse -p prism-anchor -p prism-llm -p prism-mcp -p prism-engine
            echo '### Engine coverage'
```

本机实测读数（`cargo llvm-cov clean --workspace` 之后跑上面这条，再 `report --summary-only`）：

| 范围 | Region | Function | Line | 报告里的文件数 | 含 src-tauri？ |
|---|---|---|---|---|---|
| **新（八个 engine crate）** | **93.30%** | **91.76%** | **94.19%** | 22（全部在 `crates/` 下） | 否（grep 计数 0） |
| 旧（`--workspace`，同样 clean 后） | 88.59% | 78.96% | 88.47% | 26 | 是（`commands.rs` 39.61%、`main.rs` 0.00%） |

**与 STATE.md 里 85.48% 的差异有两个来源，缺一不可：**

1. **范围变了。** 旧步骤是 `--workspace`，把 Tauri 壳一并测量。壳里覆盖最低的 `src-tauri/src/commands.rs`（39.61% region）与 `main.rs`（0.00%）被剔出分母后，读数自然上抬。这是本 plan 造成的那一半。
2. **被测代码也变了。** 85.48% 是 01-02 时期记下的读数，之后本轮十二份 gap-closure plan 往 engine 侧加了大量测试。同一条旧命令今天跑出来是 88.59%，不是 85.48%——所以 85.48% → 93.30% 里**只有 88.59% → 93.30% 那一段是范围变化带来的**，85.48% → 88.59% 那一段是这一轮补的测试。

**给 Phase 2 的交接：硬闸门的基线取 93.30% region / 94.19% line，范围是八个 engine crate（不含 `prismdocs-shell`、不含 `prism-cli`）。** 阈值仍未设（`--fail-under` 未加），本 plan 只改范围不改阈值。

### 意外发现：收窄 `-p` 不足以让范围成立（已修）

第一次跑收窄后的命令时，报告里**仍然出现** `src-tauri/src/commands.rs` / `lib.rs` / `main.rs`（各 0.00%）并计入 TOTAL（当时 92.10%）：

```
src-tauri/src/commands.rs      8   8   0.00%   4   4   0.00%   6   6   0.00%
src-tauri/src/lib.rs          30  30   0.00%   4   4   0.00%  19  19   0.00%
src-tauri/src/main.rs          3   3   0.00%   1   1   0.00%   3   3   0.00%
TOTAL                       3355 265  92.10% 326  34  89.57% 1933 142  92.65%
```

根因：`cargo llvm-cov report` 汇总的是它在 `target/llvm-cov-target` 里找到的**全部**对象，不是本次 `--no-report` 点名的那些。而 CI 把 `./target` 整个放进缓存，所以上一次运行（或上一个范围）的对象会跨 run 存活。跑 `cargo llvm-cov clean --workspace` 后 src-tauri 归零、TOTAL 变成 93.30%。

这意味着**光收窄 `-p` 并不能让「范围 == 标题」成立**——在一个缓存 `target/` 的 CI 上，范围等于历史累积。`clean` 因此进了 CI 步骤与 `just coverage`，并在两处都写明它是范围断言的一部分而不是保险动作。

## 两条（实际三条）非恒真反证的实际输出

### 反证 1：clippy 扩面（往 `crates/prism-cli/src/main.rs` 插 `needless_return`）

注入（`user_agent()`，第 62 行）：`format!(…)` → `return format!(…);`

```
$ cargo clippy -p prism-types -p prism-store -p prism-fs -p prism-parse \
    -p prism-anchor -p prism-llm -p prism-mcp -p prism-engine -- -D warnings   # 改动前的 CI
OLD exit=0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.11s

$ cargo clippy … -p prism-engine -p prism-cli -- -D warnings                    # 改动后的 CI
NEW exit=101
error: unneeded `return` statement
  --> crates/prism-cli/src/main.rs:62:5
   |
62 |     return format!("PrismDocs-helper/{CRATE_VERSION}");
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: `-D clippy::needless-return` implied by `-D warnings`
error: could not compile `prism-cli` (bin "prismdocs-helper") due to 1 previous error
```

还原后：`engine+cli exit=0`。

（记一笔口径：cargo 的失败退出码是 **101**，不是 plan 文字里写的 1。CI 判的是「非零」，两者等效，但抄读数时别按 1 去核对。）

### 反证 2：前端 lint（往 `src/App.tsx` 插 `console.log`）

```
$ npm run lint
lint exit=1
/Users/xinz/Development/PrismDocs/src/App.tsx
  11:3  error  Unexpected console statement  no-console
✖ 1 problem (1 error, 0 warnings)

$ npx tsc --noEmit          # 同一注入下，既有的 build 闸门看不见它
tsc exit=0
```

还原后：`lint exit=0`。第二行是顺手补的判别力证据——它证明新加的 lint 步骤不是 `npm run build` 里 `tsc --noEmit` 的重复，而是覆盖了一整类 tsc 结构性看不见的问题。

### 反证 3（plan 未要求，补做）：壳 clippy 的两个 flag 逐个都是承载性的

必要性：must-have 只写了「后者带 `--features test`」，但那句话本身也可能是恒真的——如果 `tests/ipc.rs` 根本不进 clippy 的受检面，带不带 feature 都一样绿。往该文件的 `payload()` 插同一处 `needless_return`，跑四种组合：

| 命令 | 退出码 |
|---|---|
| `cargo clippy -p prismdocs-shell -- -D warnings` | 0 |
| `cargo clippy -p prismdocs-shell --all-targets -- -D warnings` | 0 |
| `cargo clippy -p prismdocs-shell --features test -- -D warnings` | 0 |
| **`cargo clippy -p prismdocs-shell --all-targets --features test -- -D warnings`** | **101** |

```
error: unneeded `return` statement
   --> src-tauri/tests/ipc.rs:114:5
```

两个 flag 缺任何一个，这个文件都完全不进受检面。还原后：`shell exit=0`。

## `justfile` 三条 recipe 与 CI 的逐条对照

| justfile recipe | 命令 | 对应 CI 步骤 | 等价？ |
|---|---|---|---|
| `check-all` | `bash scripts/check-deps.sh all` + `bash scripts/check-secrets.sh all` | engine job 的 `Dependency-direction assertions` + `Plaintext secret scan` | ✓ 逐字相同（本 plan 未动） |
| `test-engine` | `cargo test -p …×8 -p prism-cli` | engine job 的 `Test (engine selection set + CLI helper)` | ✓ 本 plan 同步加了 `-p prism-cli` |
| `coverage` | `cargo llvm-cov clean --workspace` → `--no-report -p …×8` → `report --summary-only`，再 `npm run test -- --run --coverage` | engine job 的 `Coverage (engine)` + frontend job 的 `Test with coverage` | ✓ engine 侧逐字相同（含 `clean`）；前端侧相同。CI 另有 `report --lcov` + upload-artifact，本机不需要产出 lcov 文件 |
| `clippy-all`（**新**） | `cargo clippy -p …×8 -p prism-cli -- -D warnings` + `cargo clippy -p prismdocs-shell --all-targets --features test -- -D warnings` | engine job 的 `Clippy (engine selection set + CLI helper)` + shell job 的 `Clippy (Tauri shell, …)` | ✓ 两条命令逐字相同 |

未加前端 lint 的 recipe：`npm run lint` 本身已是单命令，justfile 的既有约定是给 `bash scripts/…` 做简写，包一层只增加一处会漂移的重复。

**`just` 仍未安装。** 本 plan 全程用 `bash scripts/…` / `cargo` / `npm` 直呼，`scripts/` 仍是每条断言的唯一实现；justfile 只是本机装了 just 时的等价简写，这个不变量没被动过。

## 五条既有闸门逐条点名（一条未删）

| 闸门 | 位置 | 状态 |
|---|---|---|
| `bash scripts/check-deps.sh all` | ci.yml:55 | 原样保留 |
| `bash scripts/check-secrets.sh all`（显式 `all`，不靠默认值） | ci.yml:60 | 原样保留 |
| engine 选择集的 clippy | ci.yml:66 | 保留并**扩面**（+`prism-cli`） |
| engine 选择集的 test | ci.yml:72 | 保留并**扩面**（+`prism-cli`） |
| shell job 的 `cargo test -p prismdocs-shell --features test` | ci.yml:138 | 原样保留 |

## 缓存 key 的逐行核对

```
$ grep -n 'cargo-' .github/workflows/ci.yml
51:          key: ${{ runner.os }}-cargo-engine-${{ hashFiles('**/Cargo.lock') }}
52:          restore-keys: ${{ runner.os }}-cargo-engine-
74:      - uses: taiki-e/install-action@cargo-llvm-cov
125:          key: ${{ runner.os }}-cargo-shell-${{ hashFiles('**/Cargo.lock') }}
126:          restore-keys: ${{ runner.os }}-cargo-shell-
```

`<os>-cargo-engine-` 与 `<os>-cargo-shell-`：两个字符串互不为前缀（第 12 个字符起分叉），两个 job 从此不会互相恢复 `target/`。改动前 engine 的 restore-keys 是 `<os>-cargo-`，正是 shell key 的前缀。

## 三份 plan 的交接落地情况

| 来源 | 交接内容 | 落点 |
|---|---|---|
| **01-26** | `npm run lint` 已就绪待 CI 调用；刻意未碰 ci.yml、未把 lint 折进 `build`；`--max-warnings 0` 「加上无害」 | frontend job 的 `Lint (frontend)` 步骤（`npm ci` 之后、测试之前）；`--max-warnings 0` 加在 `package.json` 的 script 里而**不是** CI 命令行——放 CI 会让本机与 CI 不同强度，那正是这个仓库反复警告的两份实现漂移 |
| **01-24** | 人工验证第 1 项五步→七步（确认 `form-action` / `frame-ancestors` 不在设置页/冒烟页触发违规），**明确不新开 WINDOWS 条目** | `01-VERIFICATION.md` 第 1 项改成显式编号七步，第 5/6 步为两条新指令，并写明「若第 5 步真的红了」的修法；WINDOWS id=8 未动 |
| **01-21** | 人工验证第 2 项不能再用 `RUST_LOG` 提档观察（该 plan 给 env filter 加了项目天花板），改为默认档位观察 + 确认降档 warn 出现且不回显原值 | `01-VERIFICATION.md` 第 2 项整条重写为两步，降档 warn 正文逐字贴出（已与 `src-tauri/src/lib.rs:57-59` 的 `LOG_CEILING_WARNING` 核对，Rust 的 `\` 续行会吃掉换行与前导空白，拼出的串与贴出的一致）；WINDOWS id=9 未动 |

两条 WINDOWS 条目都保持 `open`。`01-VERIFICATION.md` 的 `### Human Verification Required` 一节现在自述为这两项步骤的**唯一权威文本**，并点明 id=8 描述里的「五步」是记录当时的措辞、实际步数以该节为准——ledger 的 description 字段没有编辑动词（只有 append / waive / fixed），硬改生成文件的表格 + JSON 两处不如留一条指向。

## Decisions Made

1. **壳的 clippy 放 shell job，不放 plan 指定的 engine job。** 详见下方 Deviation 1。
2. **coverage 范围 = 八个 engine crate，`prism-cli` 不进。** 标题是 `### Engine coverage`，而 `prism-cli` 按 D-10 明令依赖树里不许出现任何 `prism-*`——它不是 engine crate。把它算进「Engine coverage」等于让标题重新说谎（只是方向反了）。它的测试由那条**声明的** `cargo test` 步骤看住，这正是 plan 要它进 test 步骤的理由。
3. **`clean` 进 coverage 步骤。** 见上方「意外发现」。
4. **`concurrency` group 用 `head_ref || ref_name`。** 详见下方 Deviation 2。
5. **`--max-warnings 0` 落在 package.json。** 单一实现处，本机与 CI 同强度。
6. **rustfmt 一字未动**，`cargo fmt` 一次未跑，`rustfmt.toml` 未创建，CI 无 fmt 步骤——那片地留给 01-28。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] 按 plan 把壳的 clippy 放进 engine job，会与同一 plan Task 2 的前提自相矛盾**

- **Found during:** Task 1
- **Issue:** plan 的 `<action>` 与 `<artifacts_this_phase_produces>` 表都把 `prismdocs-shell --features test` 的 clippy 步骤放在 **engine job**。但同一份 plan 的 must-have 第 4 条要求「engine 与 shell 两个 job 的缓存 key 不再互相前缀覆盖」，理由原文是「两个 job 编译不同的 feature 集」。把壳的 clippy 放进 engine job 会让 engine job 也编译 `prismdocs-shell/test` 的 feature 面——那条理由当场不成立，而缓存分段（`-cargo-engine-`）正是照它做的。附带成本：coverage 收窄之后 engine job 本来完全不再编译 Tauri 壳（含 wry / webkit 那一大坨），塞回去等于把它请回来，并与 shell job 重复编译一遍。
- **Fix:** 壳的 clippy 落在 **shell job**，排在它的 test 步骤之前；该 job 的 `dtolnay/rust-toolchain@stable` 补 `components: clippy`。engine job 的 clippy 只扩到 `prism-cli`（它与八个 engine crate feature 面相同、且按 D-10 同样 tauri-free，因此并进同一次调用）。两处都在注释里写明为什么这样分。
- **must-have 仍然满足：** 「clippy 的 `-D warnings` 覆盖 `prism-cli` 与 `prismdocs-shell`（后者带 `--features test`）」是一条关于**覆盖面**的断言，不是关于 job 归属的；覆盖面逐字达成，且比 plan 要求的多了一条 `--all-targets` 的反证。
- **Files modified:** `.github/workflows/ci.yml`
- **Verification:** 两条 clippy 命令各自退出 0；反证 3 的四组合表证明该步骤非恒真
- **Committed in:** `c56c974`

**2. [Rule 1 - Bug] plan 给的 `concurrency` 分组键消不掉它自己点名的双跑**

- **Found during:** Task 2
- **Issue:** plan 写「`group` 用 workflow 名加 ref，`cancel-in-progress` 为真，消掉同仓库 PR 的双跑」。但 `github.ref` 在两个事件上取值不同：push 是 `refs/heads/X`，pull_request 是 `refs/pull/N/merge`。两个字符串永不相等 → 两次跑落进**不同**的组 → 双跑一次不少。照 plan 字面实现会得到一个看起来解决了问题、实际什么都没合并的配置——而这类「装饰性配置」正是本轮在清理的东西。
- **Fix:** `group: ${{ github.workflow }}-${{ github.head_ref || github.ref_name }}`。`head_ref` 在 pull_request 事件下是源分支名 `X`，push 下为空则回落到 `ref_name`（push 下正是 `X`）——同一分支的两个事件取值相同，才真的落进一个组并被 `cancel-in-progress` 收掉一次。注释里写明为什么不能用 `github.ref`。
- **未一并收窄 `on:`（scope boundary）：** 另一种消双跑的做法是 `on: push: branches: [main]`，但那会让「没开 PR 的功能分支推送不再跑 CI」，是行为收缩而非卫生修复，plan 也没要求。`on: [push, pull_request]` 保持原样。
- **Files modified:** `.github/workflows/ci.yml`
- **Verification:** YAML 可解析（`yaml.safe_load` 退出 0）。**运行期效果本机不可验证**，已登记（见 Known Gaps）
- **Committed in:** `6e8702e`

**3. [Rule 2 - Missing] 收窄 `-p` 不足以让「范围 == 标题」成立，缺 `clean`**

- **Found during:** Task 1（第一次跑收窄后的 coverage）
- **Issue:** plan 的两个选项都只谈 `--no-report` 的 `-p` 列表，隐含假设「点名什么就测量什么」。实测证伪：`report` 汇总 `target/` 里找到的全部 llvm-cov 对象，收窄后的报告里 `src-tauri/src/*` 仍以 0.00% 出现并计入 TOTAL（92.10%）。而 CI 把 `./target` 整个放进缓存，这些对象会跨 run 存活——也就是说 plan 想关的那个缺口（范围与标题不符）在只改 `-p` 的世界里会以另一种形态复活，而且更难查（本机第一次跑就会 clean，CI 才有缓存）。
- **Fix:** coverage 步骤前置 `cargo llvm-cov clean --workspace`，`just coverage` 同步，两处注释都写明它是范围断言的一部分而不是保险动作。加 `clean` 后 TOTAL 93.30%、报告里 22 个文件全在 `crates/` 下、`src-tauri` grep 计数 0。
- **Files modified:** `.github/workflows/ci.yml`, `justfile`
- **Verification:** `report --summary-only | grep -c src-tauri` → 0；`grep -c '^prism-'` → 22
- **Committed in:** `c56c974`

**4. [Rule 2 - Missing] `--max-warnings 0` 与它的落点（plan 未提，01-26 交接项）**

- **Found during:** Task 2
- **Issue:** 01-26-SUMMARY 明确把这一项交给本 plan（「加上无害且能防住以后有人把规则降成 warn」），plan 的 tasks 里没有它。而 01-26 的 config 注释自己论证过这条的重要性：eslint 对 warn 级别命中**仍然退出 0**，于是「规则确实生效」与「规则根本没装」在退出码上同形——判别力归零。
- **Fix:** `package.json` 的 `lint` script 改成 `eslint . --max-warnings 0`。刻意**不**写成 CI 里的 `npm run lint -- --max-warnings 0`：那会让 CI 比本机严，断言分裂成两处实现，正是这个仓库反复警告的漂移形态。
- **`package.json` 不在 Task 2 的 `<files>` 里**，但它是本轮交接明确指名的动作，且改动是一个 flag。
- **Files modified:** `package.json`
- **Verification:** `npm run lint` 退出 0（当前 0 warning，所以这条 flag 今天不改变结果——这正是 01-26 说的「不依赖它」）
- **Committed in:** `6e8702e`

**5. [Rule 2 - Missing] 两项人工验证步骤的改写（plan 未列为 task）**

- **Found during:** Task 3（本轮汇总职责）
- **Issue:** 01-21 与 01-24 两份 SUMMARY 各留了一节「（供 01-27 汇总）」，内容是 `01-VERIFICATION.md` 里两项人工验证步骤的必要改写。plan 的 `<tasks>` 与 `files_modified` 都没有这个文件。若不落地，两条已知失效的步骤会以现状进入 Phase 收尾人工验证——第 2 项的原步骤在 01-21 之后**已经不可执行**（`RUST_LOG` 提档被天花板吃掉），照它做的人会看不到目标日志并误判成「sink 没有落点」。
- **Fix:** `01-VERIFICATION.md` 的 frontmatter `human_verification` 两条 + 正文 `### Human Verification Required` 一节同步改写；第 1 项显式编号七步、第 2 项重写为两步并逐字贴出降档 warn 正文。两条 WINDOWS 条目（id=8 / id=9）按 01-24 的交代**未新开、未改状态**，改由该节自述为唯一权威文本并点明 id=8 描述里的「五步」以它为准。
- **Files modified:** `.planning/phases/01-foundation-skeleton/01-VERIFICATION.md`
- **Verification:** frontmatter `yaml.safe_load` 退出 0 且两条 `expected` 里无转义残留（首版写成 `` \\` `` 留下了字面反斜杠，已改掉）；降档 warn 正文与 `src-tauri/src/lib.rs:57-59` 逐字核对
- **Committed in:** `e6db6c4`

---

**Total deviations:** 5 auto-fixed（Rule 1 ×2、Rule 2 ×3）
**Impact on plan:** 两条 Rule 1 都是 plan 字面写法会产出装饰性配置的地方（壳 clippy 的 job 归属会拆掉自己 Task 2 的前提；`github.ref` 分组消不掉双跑），修的是手段不是目标，四条 must-have truth 全部逐字达成。三条 Rule 2 中两条来自本轮 plan 之间的交接（`--max-warnings 0`、人工验证步骤），一条是实测发现的范围断言缺口（`clean`）。无 scope creep：未新增任何包、未跑 `cargo fmt`、未碰 `rustfmt.toml`、未动同轮其他 plan 的源码文件。

## Issues Encountered

- **`tsconfig.json` 加注释的安全性。** 想把「为什么刻意不要 `vitest/globals`」写进文件本身，但该文件此前无注释，而它是每条前端闸门的硬依赖（tsc / eslint 的 projectService / vite / vitest 四方都读它）。做法是加完注释后把四条命令全跑一遍：`npx tsc --noEmit` 0、`npm run lint` 0、`npm run test -- --run` 75 passed、`npm run build` 0。TypeScript 官方支持 tsconfig 里的注释，四方工具链实测都吃得下，注释保留。
- **`cargo llvm-cov` 的退出码与 plan 文字不符。** plan 的验收写「退出 1」，cargo 实际是 101。CI 判非零，等效；已在 SUMMARY 里标出口径，免得后来者按 1 去核对而以为反证失败。
- **frontmatter 里嵌套代码引号的转义坑。** 第一版把降档 warn 正文写进 YAML 双引号串时用了 `` \\` ``，`yaml.safe_load` 出来带字面反斜杠。改成不在代码 span 内再嵌 span，正文逐字版只放在 markdown 正文的围栏块里。

## Known Gaps

**这个 CI workflow 至今一次都没在 GitHub Actions 上跑过。** `gh run list` 返回 `[]`，`origin/main` 停在 `4cc1347 update arch docs`——整个 Phase 1 的提交尚未推送。因此本 plan 加的三项 workflow 级配置只有「YAML 可解析 + 推理」这一层证据，运行期行为未验证：

1. `permissions: contents: read` 下 `actions/upload-artifact@v4` 是否仍能上传（依据是它走 `ACTIONS_RUNTIME_TOKEN` 而非 `GITHUB_TOKEN`，推理可靠但未实证）；
2. `concurrency` 是否真的把同 commit 的两次跑收成一次；
3. 两个缓存分段是否真的互不恢复。

**步骤级的断言全部本机实证过**（十条验证命令 + 三条反证），这是 `scripts/` 作为唯一实现换来的性质：闸门内容与 CI 是否跑起来解耦。但 workflow 这一层没有本机等价物。已按 `unrun-verify` 登记进 WINDOWS ledger，首次 push 后核对一次即可关闭。

## Known Stubs

无。本 plan 未写任何产品代码，未留占位实现、未跳过任何测试。三处临时注入（`prism-cli/src/main.rs` 的 `needless_return`、`src-tauri/tests/ipc.rs` 的 `needless_return`、`src/App.tsx` 的 `console.log`）均为反证探针，全部已还原并逐条重跑确认退出 0；`git status --short` 除两个先于本 plan 存在的 `.planning/research/.cache/` 未跟踪文件外干净。

## Threat Flags

无新增安全相关面。本 plan 的四条 `mitigate` 全部落地：

| Threat ID | Disposition | 落点 |
|---|---|---|
| T-01G-58（无 `permissions` 块） | mitigate | 顶层 `permissions: contents: read`（ci.yml:13-14） |
| T-01G-59（engine 可恢复 shell 缓存） | mitigate | `-cargo-engine-` / `-cargo-shell-` 两段互不为前缀 |
| T-01G-60（`prism-cli` / 壳无 clippy 闸门） | mitigate | 两条 clippy 步骤 + 反证 1、反证 3 |
| T-01G-61（coverage 范围与标题不符） | mitigate | 范围收窄 + `clean` + 新基线记录 |
| T-01G-62（前端 lint 未进 CI） | mitigate | `Lint (frontend)` 步骤 + 反证 2 |
| T-01-SC（包安装） | accept | 未新增任何 npm / cargo 包（`Cargo.lock` / `package-lock.json` 无新条目） |

## User Setup Required

None — 无外部服务配置。

## Verification Evidence

`<verification>` 的十条命令全部实跑：

```
$ cargo clippy -p prism-types -p prism-store -p prism-fs -p prism-parse \
    -p prism-anchor -p prism-llm -p prism-mcp -p prism-engine -p prism-cli -- -D warnings
exit=0
$ cargo clippy -p prismdocs-shell --all-targets --features test -- -D warnings
exit=0
$ cargo test -p prism-cli
exit=0        test result: ok. 4 passed; 0 failed
$ cargo test --workspace
exit=0
$ cargo test -p prismdocs-shell --features test
exit=0        21 passed (lib) / 2 passed (tests/ipc.rs) / 0 + 0（另两个空 target）
$ bash scripts/check-deps.sh all
exit=0
$ bash scripts/check-secrets.sh all
exit=0
$ npm run lint
exit=0
$ npx tsc --noEmit
exit=0
$ npm run test -- --run
exit=0        Test Files 7 passed (7) / Tests 75 passed (75)
$ npm run build
exit=0        ✓ built in 52ms
$ python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"
exit=0
```

`cargo test -p prismdocs-shell --features test` 的 2 passed 是 `tests/ipc.rs`——它同时确认新加的壳 clippy 步骤指向的确实是一个非空的受检面。

## Self-Check: PASSED

- 声明修改的 5 个文件全部存在于磁盘：`.github/workflows/ci.yml`、`justfile`、`tsconfig.json`、`package.json`、`.planning/phases/01-foundation-skeleton/01-VERIFICATION.md`
- 三个 task commit 全部可在 `git log` 中找到：`c56c974`、`6e8702e`、`e6db6c4`
- 工作树除 `.planning/` 元数据与两个先于本 plan 存在的 `.planning/research/.cache/` 未跟踪文件外干净（三处反证探针已全部还原）
