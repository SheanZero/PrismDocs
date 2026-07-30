---
phase: 01-foundation-skeleton
plan: 28
subsystem: infra
tags: [rustfmt, toolchain, decision, phase-closure, ci, github-actions]

requires:
  - phase: 01-foundation-skeleton
    provides: "01-27 的 CI 闸门接线（本 plan 只在 engine job 最前插一步，其余一律不动）"
  - phase: 01-foundation-skeleton
    provides: "01-16 / 01-20 / 01-21 把源码序断言的锚点从裸名字收窄为完整语句——本 plan 的 0 锚点改动直接受益于它"
provides:
  - "rustfmt 取向的最终定案：显式采用默认风格（option-a），从 plan 01-03 悬置至此的候选关闭"
  - "rustfmt.toml（仓库根）——不含任何设置项；空本身即决定，文件头记录日期与理由"
  - "CI engine job 的第一条实质步骤 `cargo fmt --all -- --check`，且 components 显式列出 rustfmt"
  - "justfile 的 fmt-check recipe（单行委托，与 CI 那步逐字等价）"
  - "全仓一次性格式化：23 个受版本控制 .rs 文件，+142/−82"
  - "deferred-items.md 里 01-03 那条的最终结论（纯追加，删除计数 0）"
  - "Phase 1 收尾的两项人工验证清单（原样转录，供 /gsd-verify-work 承接）"
affects: [phase-02, ci, all-rust-files]

tech-stack:
  added: []
  patterns:
    - "一个空的配置文件可以是一份决定：没有 rustfmt.toml 时「用默认风格」与「没人配置过」不可区分，有它则下一个改动者知道自己在推翻决定而不是填遗漏"
    - "CI 里显式列出 toolchain component 而不是依赖 profile 默认值——默认值变化时的失败形态（找不到工具）与真正的闸门失败（格式不合）都是红，但修的方向相反"
    - "源码序断言的锚点取完整语句而非裸标识符，除了避开注释误命中，还顺带买到对 rustfmt 换行的韧性（本 plan 实测：0 处锚点需要改）"
    - "`git checkout -- <file>` 恢复到的是 HEAD，不是未提交的工作状态——在「注入→反证→还原」这类实验里它会连带撤销本轮的合法改动"

key-files:
  created:
    - rustfmt.toml
  modified:
    - .github/workflows/ci.yml
    - justfile
    - .planning/phases/01-foundation-skeleton/deferred-items.md
    - "23 个 .rs 文件（纯格式化，无语义改动）"

key-decisions:
  - "用户在 checkpoint:decision 上选定 option-a：采用 rustfmt 默认风格 + CI 闸门 + 一次性格式化全仓"
  - "rustfmt.toml 刻意不含任何设置项（连 edition 也不写）——`cargo fmt` 从 Cargo.toml 取 edition，写进来只会制造第二处真相源"
  - "fmt 步骤排在 actions/cache 之前：它不需要 target/，而失败时也没有任何编译产物值得缓存"
  - "components 里显式补 rustfmt，不靠 dtolnay/rust-toolchain@stable 的默认 profile"
  - "deferred-items.md 采用纯追加式回填（删除计数 0），过期的指向句由新小节首行就地声明作废，而不是删改它"

metrics:
  duration: "约 40 分钟（含 checkpoint 前的决策材料采集）"
  completed: 2026-07-30
  tasks_completed: 2
  files_changed: 27
status: complete
---

# Phase 01 Plan 28: rustfmt 取向定案 Summary

用户选定 option-a：全仓一次性 `cargo fmt`（23 文件 / +142−82，纯换行偏好），配 `rustfmt.toml`（空 = 决定本身）+ CI engine job 首步 fmt 闸门 + `just fmt-check`；7 处 `include_str!` 源码序断言逐条重跑全绿、0 处锚点被改。

---

## Task 1：rustfmt 取向二选一（checkpoint:decision，gate="blocking"）

### 决策材料（本机实跑，非印象描述）

| 项 | 读数 |
|---|---|
| `cargo fmt --all -- --check` hunk 数 | 38 |
| 受影响文件 | 40 个受版本控制 `.rs` 中的 **23 个** |
| 落地后真实 diff | **+142 / −82** 行 |
| 差异性质 | 全部为换行 / 缩进偏好，**无一处改变语义** |
| 源码序断言交叉核对 | 7 处 `include_str!` 站点，其中与格式化区域**真正重叠的只有 1 处**：`src-tauri/src/lib.rs:167` |

### 决策结论

**`option-a`** —— 采用 rustfmt 默认风格并加 CI 闸门（一次性格式化全仓）。

用户是在看过上表、看过 `src-tauri/src/lib.rs:167` 这个唯一重叠点、并且看过下面这条继承自 01-27
的诚实限制之后做的选择：

> 本 plan 对 fmt 闸门的证据止于「本机 `cargo fmt --all -- --check` 退出 0，且注入劣化排版后退出
> 非零并点名文件」。「CI 上这一步真的会红」**未经验证**，与 WINDOWS id=14 并列——该 workflow 至今
> 未在 GitHub Actions 上跑过。本 SUMMARY 不把它写成已证明的事。

### 理由

三条，按分量排序：

1. **成本随时间单调上升。** 现在做只触及本 phase 建立的文件；每晚一个 phase，被卷进这次提交的
   文件就多一批。
2. **与项目基调一致。** 本仓库其余每一条纪律都有断言看住（依赖方向、密钥、CSP、源码序）；靠约定
   维持风格会是唯一一处例外。
3. **风险被实测证伪了大半。** 计划期担心的「锚点被拆行导致断言假红」在落地后一处也没发生——理由
   见下。

### deferred-items.md 回填确认

`.planning/phases/01-foundation-skeleton/deferred-items.md` 的「去向（Phase 1 收尾定案）」小节已加上
`##### 定案：选项 A（2026-07-30，01-28 Task 1）` 子节。

- **删除计数 0**：`git diff --numstat` 对该文件为 `27  0`。
- **选项 A / B 原文一字未改**保留在上方——它们是这次决策的备选记录，不是待办。
- 计划期写下的指向句「本小节只做指向，不做决策」**未被删改**，改由新小节首行就地声明它到此为止。
  这是为了同时满足两条要求（「替换掉过期指向句」与「删除计数为 0」）——两者字面上冲突，采用的解法
  是**加法式作废**而非行内修改。

---

## Task 2：按决策落地（commit `7a21f3b`）

### 1. `rustfmt.toml`（新建，仓库根）

**不含任何设置项**——连 `edition` 也没写（`cargo fmt` 从 `Cargo.toml` 取，写进来只会制造第二处
真相源）。文件里只有注释，核心一句：

> 一个被想过之后接受的取舍，与一个没人注意到的空洞，在仓库里长得一样。

没有这个文件时，「采用默认风格」与「没人配置过 rustfmt」在仓库里不可区分；有了它，下一个想加
`max_width` 的人知道自己是在推翻一个决定。

实测确认它被 rustfmt 正常读取且**无 unknown-config-key 告警**。

### 2. `cargo fmt --all`

```
.rs 文件：23 files changed, 142 insertions(+), 82 deletions(-)
连同 ci.yml / justfile / deferred-items.md：27 files changed, 211 insertions(+), 83 deletions(-)
```

落地后的读数与 checkpoint 上呈给用户的预估**完全一致**。

### 3. 七处源码序断言逐条重跑 —— 全绿，0 处锚点被修改

每条都是**单独** `--exact` 跑的，不是靠一次全量绿倒推：

| # | 站点 | 测试 | 锚点 | 结果 |
|---|---|---|---|---|
| 1 | `crates/prism-store/src/open.rs:308` | `migration_runs_before_the_read_pool_is_built` | `to_latest(&mut writer)` / `Pool::builder()` | **ok**（EXIT=0） |
| 2 | `crates/prism-mcp/src/middleware.rs:448` | `the_comparison_is_not_a_plain_equality` | `ct_eq` / `expected == presented` / `if expected.is_empty() {` | **ok**（EXIT=0） |
| 3 | `crates/prism-engine/src/facade.rs:158` | `no_public_method_hands_out_a_connection` | `production_source()` + 四个 `-> Connection` 族 | **ok**（EXIT=0） |
| 4 | `crates/prism-engine/src/services.rs:260` | `the_service_impls_contain_no_await` | `!RECEIPT_STATUSES.contains(&receipt.status.as_str())` / `"recorded an agent receipt"` | **ok**（EXIT=0） |
| 5 | `src-tauri/src/lib.rs:489` | `run_installs_tracing_before_it_builds_the_app` | `let _ = init_tracing();` / `tauri::Builder::default()` | **ok**（EXIT=0） |
| 6 | `src-tauri/src/lib.rs:529` | `the_release_ipc_surface_excludes_the_dev_commands` | `RELEASE_ARM_ANCHOR`（跨行，含 `\n    ` 四空格缩进） | **ok**（EXIT=0） |
| 7a | `src-tauri/src/commands.rs:155` | `commands_carry_no_business_logic` | `production_source()` + `Connection`/`prepare`/`query_row`/`keyring` | **ok**（EXIT=0） |
| 7b | `src-tauri/src/commands.rs:155` | `dev_smoke_stream_hands_the_loop_to_the_blocking_pool` | `tauri::async_runtime::spawn_blocking(move || smoke::generate(` | **ok**（EXIT=0） |

**被修改的锚点：无。旧值/新值/理由三栏因此为空。**

#### 唯一真正的重叠点：`src-tauri/src/lib.rs:167`（按实测报告，不是推断）

计划期与 checkpoint 上都把这一处标为「最可能变红」。格式化确实动了它：

```diff
-    let builder = tauri::Builder::default()
-        .setup(|app| {
-            use tauri::Manager;
+    let builder = tauri::Builder::default().setup(|app| {
+        use tauri::Manager;
```

**站点 5 仍然绿**，原因是锚点取的是 `tauri::Builder::default()` 这个**完整语句片段**——`.setup(`
被并到同一行不影响子串匹配。这条韧性是 01-21 把锚点从裸的 `"tauri::Builder"` 收窄为完整语句时
顺带买到的（当时的动机是避开解释性注释里的同名字样）。同一个收窄动作挡住了两类完全不同的失败。

站点 6 的 `RELEASE_ARM_ANCHOR` 含字面的 `\n    `（四空格），是本轮最脆的一个锚点。实测
`.github` 之外的 release arm 区域（`src-tauri/src/lib.rs:215-216`）**未被 fmt 触及**——
`git diff -U0` 的 hunk 头分别落在 57 / 170 / 174 / 176 / 183 / 301 / 361 行，无一覆盖 215。
加上该测试自带 `expect()`（锚点找不到会 panic 而非静默通过），绿即证明锚点仍命中。

### 4. `check-secrets.sh` 匹配面复核

换行变化可能让字面量跨行（漏检）或并行（新命中）。实测**两者都没发生**：

```
OK: pattern discriminates (19 positive / 10 negative samples)
OK: no plaintext secret in 115 version-controlled files
>>> EXIT=0
```

未产生新命中，因此**没有触发**「改 fixture 不改防线」的单向约定。

### 5. CI fmt 步骤

engine job 步骤顺序（YAML 实测可解析）：

```
0. actions/checkout@v4
1. dtolnay/rust-toolchain@stable          ← components: clippy, llvm-tools-preview, rustfmt
2. Format check (rustfmt default style)   ← 新增，第一条实质步骤
3. actions/cache@v4
4. Dependency-direction assertions
5. Plaintext secret scan
6. Clippy (engine selection set + CLI helper)
7. Test (engine selection set + CLI helper)
8. taiki-e/install-action@cargo-llvm-cov
9. Coverage (engine)
10. actions/upload-artifact@v4
```

两个偏离字面计划的实现选择，各有理由：

- **`components` 里显式写 `rustfmt`。** 计划说「默认带 rustfmt；若不带再补」。这里选择无条件写出：
  默认 profile 今天确实带它，但那是一个**默认值**而不是承诺。它哪天变了，这一步会以「找不到
  rustfmt」的形态失败——与「格式不合」同样是红，却要往相反方向修。写出来它就不再取决于默认值。
- **fmt 排在 `actions/cache` 之前。** 它不读 `target/`，而它失败时也没有任何编译产物值得缓存。

**01-27 的改动一处未动**（逐条确认）：`cargo llvm-cov clean --workspace`、shell job 的 clippy、
engine job 的 `-p prism-cli`、顶层 `permissions` / `concurrency`、两段互不为前缀的缓存 key、
frontend job 的 lint 步骤——全部保持原样。

### 6. `just fmt-check`

单行委托，与 CI 那步逐字等价，遵循 justfile 文件头既有约定（`scripts/` 与 `cargo` 自身是断言的
唯一实现，`just` 只是本机简写）。`just` 未安装于本机，故该 recipe 未实跑——这与 justfile 里
其余每一条 recipe 的处境相同，不是本 plan 引入的新缺口。

---

## fmt 闸门的非恒真反证（实跑输出）

**[0] 基线**

```
$ cargo fmt --all -- --check
>>> EXIT=0
```

**[1] 注入劣化排版**（`crates/prism-anchor/src/lib.rs`，只改缩进与空格，语义不变）

```rust
pub fn content_fingerprint(s: &str)   ->   String {
        blake3::hash(s.as_bytes())
            .to_hex()
                .to_string()
}
```

**[2] 闸门退出非零并点名文件**

```
$ cargo fmt --all -- --check
>>> EXIT=1
Diff in /Users/xinz/Development/PrismDocs/crates/prism-anchor/src/lib.rs:12:
-pub fn content_fingerprint(s: &str)   ->   String {
-        blake3::hash(s.as_bytes())
-            .to_hex()
-                .to_string()
+pub fn content_fingerprint(s: &str) -> String {
+    blake3::hash(s.as_bytes()).to_hex().to_string()
```

**[3] 还原 → [5] 退回 0**

```
$ cargo fmt --all -- --check
>>> EXIT=0
```

### 反证过程中的一处真实教训（不是顺利完成的，如实记）

还原用的是 `git checkout -- crates/prism-anchor/src/lib.rs`——它恢复到的是 **HEAD**，也就是
**格式化之前**的状态，于是把本轮对该文件的合法格式化一并撤销了。结果是「还原后 `--check` 应当
退出 0」这一步**当场变红**，且红的原因与被反证的那件事无关。

处理：重跑 `cargo fmt --all` 把状态拉回，随后 `--check` 退出 0，并复核 `.rs` 部分的 diff 规模
仍是 `23 files, +142/−82`（与还原前逐字相同），确认工作树没有被这次误操作污染。

这条值得写下来：在「注入 → 反证 → 还原」这类实验里，`git checkout --` 的还原目标是 HEAD 而不是
未提交的工作状态。同一轮里既有合法未提交改动、又要临时劣化某个文件时，它会连带撤销前者。

---

## 全部闸门重跑（每条带实测 exit code）

| 闸门 | 结果 |
|---|---|
| `cargo build --workspace` | Finished，EXIT=0 |
| `cargo fmt --all -- --check` | **EXIT=0** |
| `cargo test -p prism-types … -p prism-cli`（engine 选择集 + CLI，CI 逐字） | 全部 `ok`，0 failed，EXIT=0 |
| `cargo test -p prismdocs-shell --features test` | 全部 `ok`，0 failed，EXIT=0 |
| `cargo clippy -p prism-types … -p prism-cli -- -D warnings`（CI 逐字） | 0 warning，EXIT=0 |
| `cargo clippy -p prismdocs-shell --all-targets --features test -- -D warnings`（CI 逐字） | 0 warning，EXIT=0 |
| `bash scripts/check-secrets.sh all` | 19 正 / 10 负样本判别通过；115 文件无明文密钥，EXIT=0 |
| `bash scripts/check-deps.sh all` | 六条依赖方向断言全 OK，EXIT=0 |
| `npm run lint` | EXIT=0（`--max-warnings 0`） |
| `npx tsc --noEmit` | 0 error，EXIT=0 |
| `npm run test -- --run` | 7 files / **75 tests passed**，EXIT=0 |
| `npm run build` | built in 55ms，EXIT=0 |

反证完成后又整轮复跑了 fmt-check / engine tests / shell tests / check-secrets，全部 EXIT=0。

---

## Deviations from Plan

### 1. [Rule 3 - 计划内部冲突] `deferred-items.md` 采用加法式作废而非行内替换

- **Found during:** Task 2 最后一步
- **Issue:** 交接指令同时要求「replace the trailing pointer sentence」与「deletion count 0」。
  任何对既有行的修改在 `git diff --numstat` 里都计为删除，两者字面上不可兼得。
- **Fix:** 保留指向句原文，由新增小节的首行就地声明它到此为止（「上面那句……写于计划期，到此为止」）。
  效果上指向句不再是最后一句话，而删除计数保持 0。
- **Files modified:** `.planning/phases/01-foundation-skeleton/deferred-items.md`（`27  0`）
- **Commit:** `7a21f3b`

### 2. [Rule 2 - 缺失的关键性] `components` 显式列出 `rustfmt`

- **Found during:** Task 2 第 5 步
- **Issue:** 计划写「默认带 rustfmt；若不带在 components 里补上」。依赖默认 profile 会让闸门的
  可用性取决于一个上游默认值。
- **Fix:** 无条件写进 `components`。理由记在 ci.yml 的行内注释里。
- **Commit:** `7a21f3b`

### 3. fmt 步骤置于 `actions/cache` 之前

计划只说「engine job 最前」。实现取字面义放在 checkout + toolchain 之后的第一条实质步骤，即
cache 之前。它不读 `target/`，失败时也无产物值得缓存。

---

## Known Stubs

无。本 plan 未新增任何生产代码路径。

---

## Threat Flags

无。本 plan 不新增网络端点、认证路径、文件访问模式或信任边界上的 schema 改动；23 个 `.rs`
文件的改动全部为排版，无新符号。

威胁登记表的三条本轮处置：

| Threat ID | 处置结果 |
|---|---|
| T-01G-63（格式化打断源码序断言→假红→被顺手删掉） | **未触发**：7 处逐条重跑全绿，0 处锚点被改，因此不存在「被顺手删掉」的机会 |
| T-01G-64（格式化后 check-secrets 字面量跨行漏检） | **未触发**：`check-secrets.sh all` 退出 0，判别力自证 19/10 通过，无新命中 |
| T-01G-65（决定未被记录，下一个读者当成遗漏） | **已缓解**：`rustfmt.toml` 文件头 + `deferred-items.md` 定案小节，两处独立记录 |
| T-01-SC（包引入） | **accept 成立**：本 plan 零新增包；rustfmt 是 rustup 组件 |

---

## 未验证的部分（不要读成已证明）

1. **fmt 闸门在真实 CI 上的行为。** 证据止于本机：`--check` 退出 0，注入劣化后退出 1 并点名文件，
   YAML 可解析且步骤顺序符合预期。「GitHub Actions 上这一步真的会红」**未观测**——该 workflow
   至今未在 CI 上跑过（WINDOWS id=14 记录的同一事实）。首次真实 CI 运行时应与 id=14 一并核对。
2. **`just fmt-check` 未实跑**（本机无 `just`）。与 justfile 里其余每一条 recipe 同处境。

---

## Outstanding Human Verification

> 以下两项**不可自动判定**，不是任务。本 plan 作为 Phase 1 gap-closure 的最后一份，把它们汇总为
> 收尾的人工验证清单。以下内容自 `01-28-PLAN.md` 原样转录，供 `/gsd-verify-work` 承接。步骤已按
> 本轮改动更新：01-24 给两份 CSP 加了 `form-action` / `frame-ancestors`；01-21 给 `RUST_LOG` 加了
> 项目上限与降档 warn，并把四个 `dev_*` 命令移出 release 的 `generate_handler!`。

### 1. 真实 WebView 下的 CSP 与 IPC 双通路（同时承载成功标准 2 的复验）

**Test:**
1. `npm run tauri dev` 起应用 → 窗口**不是白屏**
2. 打开设置页：状态行、两个输入框、两个按钮都在；随手保存一个合法端点（形如 `https://api.anthropic.com`），确认成功文案出现
3. 打开 dev 冒烟页（右下角 dev 开关），跑三个验证入口：
   - **总线事件往返**：点一次，事件计数 +1；离开页面再回来再点，计数仍与点击次数 **1:1**（不翻倍）
   - **Channel 有序流**：点一次，读数应为「seq 校验通过 · 实收 1000 条」
   - **中文搜索**：先「写入样例文档」，再搜「锚定引擎」应命中 >0；搜「量子纠缠」应为 0（阴性对照）
4. 打开 WebView 控制台，确认**无任何 CSP 违规报告**——本轮新加了 `form-action 'none'` 与 `frame-ancestors 'none'`，要特别留意这两条是否在设置页或冒烟页触发违规
5. `npm run tauri build` 出 dmg，安装后对装出来的 app **重复第 1–4 步**（发布形态走 `csp` 而不是 `devCsp`，这是验证严格那一份的唯一路径）
6. **发布形态额外确认**（01-21 新增）：dev 冒烟页的开关按钮不存在（`import.meta.env.DEV` 摇掉），且四个 `dev_*` 命令已移出 release 的 `generate_handler!`——若发布形态下仍能触达冒烟页且命令仍能调用，说明分叉未生效

**Expected:** 六步全部正常。三个入口的读数与 01-09 人工验证时一致（事件计数 1:1、「seq 校验通过 · 实收 1000 条」、「锚定引擎」命中 >0 且「量子纠缠」= 0）。

**Why human:** CSP 只在真实 WebView 里生效——jsdom（`tauri-security.test.ts` 第 17 行自己写明）与 `cargo test`（走 `mock_builder`，无 WebView）都结构性看不见它。**且这一项同时是 SC-2 的复验**：SC-2 的真实 WebView 证据取自 `csp: null` 的环境，而 `connect-src 'self' ipc: http://ipc.localhost` 直接管辖 IPC 来源；若它在真实 WebView 下挡住 IPC，受影响的不是某一个命令而是**全部十个**。旧读数不得沿用。

**出现违规时的处理：** 只放宽 `devCsp`，或按控制台报告点名的指令逐项追加到 `csp`；**禁止**把 `csp` 设回 `null`，也禁止直接删掉本轮新加的两条指令。若确需放宽，先在 `src/lib/tauri-security.test.ts` 的精确相等断言上过一次评审——那正是 01-24 让它变成精确相等的意义。

### 2. 日志 sink 真的有落点

**Test:**
1. 在 `npm run tauri dev` 的终端里，打开设置页把 base_url 设成一个**非 loopback 的 `http://`** 端点（例如 `http://example.com/v1`），点保存
2. 观察终端
3. **本轮新增的第二半**（因 01-21 的 `RUST_LOG` 上限而增加）：用 `RUST_LOG=trace npm run tauri dev` 再起一次，观察终端

**Expected:**
- 第 2 步：出现 tracing 格式的行，且 `crates/prism-store/src/settings.rs` 那条明文 http 告警（`LLM endpoint uses plaintext http to a non-loopback host`）**实际打出来**
- 第 3 步：出现 01-21 新增的**降档 warn**（说明环境提供的档位超过项目上限）；终端里**没有**大量 `rmcp` target 的 trace 行（Phase 1 尚未启动 MCP server，因此更实际的判据是「降档 warn 出现」）

**Why human:** `tracing::dispatcher::has_been_set()` 只证明 dispatcher 就位，不证明日志到达终端（EnvFilter 档位、fmt 层的输出目标都可能让它落空）。三条安全决策日志（无差别 403 的真实原因、明文 http 告警、钥匙串不可用降级提示）是否真有落点，取决于这条端到端确认。

### 3.（本 plan 新增）fmt 闸门的首次真实 CI 运行

**Test:** 首次把本分支推到 GitHub 后，看 engine job 的 `Format check (rustfmt default style)` 步骤。

**Expected:** 该步骤出现在步骤列表最前，且在格式合规时为绿。若要验证它的判别力，可在一个丢弃分支上
注入本 SUMMARY「非恒真反证」那段的劣化排版，确认该步骤在 CI 上变红并点名文件。

**Why human:** 需要一次真实的 GitHub Actions 运行——本机跑不出它。与 WINDOWS id=14 是同一次运行
可以一并核对的三项 workflow 级配置并列。

---

## Self-Check: PASSED

| 声称 | 复核方式 | 结果 |
|---|---|---|
| `rustfmt.toml` 存在 | `[ -f ]` | FOUND |
| `01-28-SUMMARY.md` 存在 | `[ -f ]` | FOUND |
| commit `7a21f3b` 存在 | `git log --oneline --all \| grep` | FOUND |
| `cargo fmt --all -- --check` 退出 0 | 复跑 | EXIT=0 |
| `justfile` 有 `fmt-check` recipe | `grep -c "^fmt-check:"` | 1 |
| CI 有 `Format check (rustfmt default style)` 步骤 | `grep -c` | 1 |
| `components` 含 `rustfmt` | `grep -c "components: clippy, llvm-tools-preview, rustfmt"` | 1 |
| `deferred-items.md` 有定案小节 | `grep -c "定案：选项 A"` | 1 |
| 选项 A / B 原文保留 | `grep -c "^- \*\*选项 [AB]\*\*"` | 2 |
| `deferred-items.md` 删除计数为 0 | `git diff --numstat`（提交前） | `27  0` |

无缺失项。
