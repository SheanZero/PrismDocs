---
phase: 01-foundation-skeleton
plan: 21
subsystem: shell-logging-and-ipc-surface
tags: [security, logging, tracing, ipc-surface, dev-only, release-hardening, gap-closure]
status: complete

requires:
  - "src-tauri/src/lib.rs（01-13 装起来的日志 sink 与 DEFAULT_LOG_FILTER 的克制注释）"
  - "src-tauri/tests/ipc.rs（01-09 建立的命令可达性测试与它的负对照）"
  - "根 Cargo.toml [workspace.dependencies] 里已有的 serial_test = \"3\""
  - "01-REVIEW.md WR-10 / IN-02；01-REVIEW-prior.md WR-07 / IN-01"
provides:
  - "日志 sink 有一个 RUST_LOG 打不破的天花板：rmcp target 被钉在 info，且降档这件事会发一条只陈述规则的 warn"
  - "上限是针对性的：prism_* 在 RUST_LOG=trace 下仍可达 TRACE，开发者的调试能力未被剥夺"
  - "tracing 测试不再依赖「本二进制里我是第一个装 subscriber 的」这条会随时间失效的前提"
  - "release IPC 面上不存在四条 dev_* 命令（编译期分叉），由一条源码断言看住"
  - "log_filter_probe! —— 直接向某个 filter 提问的探针，绕开全局 dispatcher 与 callsite 兴趣缓存"
affects:
  - "Phase 5/6：rmcp 起 streamable http server 之后，评论正文与文档摘录不会因一条环境变量落进本地日志"
  - "Phase 2：用户真实库里不会再被 WebView 里的脚本塞进 smoke-project 的 fixture 行"
  - "Phase 收尾人工验证第 2 项（日志 sink 落点）的步骤需更新——详见本文末尾，由 01-27 汇总"
  - "本 crate 新增一条测试约定：任何装 subscriber 或改 RUST_LOG 的测试必须进 #[serial] 组"

tech-stack:
  added: []
  patterns:
    - "EnvFilter 按**特异性**（target 长度 / 字段数）而非书写顺序挑选生效指令：追加一条带 target 的 `rmcp=info` 压得住 `RUST_LOG=trace`，也会直接替换掉 `RUST_LOG=rmcp=trace`（本 plan 实测两种形态）"
    - "问「filter 放不放行」要自造 callsite + Metadata 直接调 `Subscriber::enabled`，不要走 `event_enabled!`：后者问的是全局 dispatcher，答案还会经 callsite 兴趣缓存跨 subscriber 复用"
    - "`#[serial]` 只挡**并发**，挡不住**顺序**：依赖「我是第一个」的断言必须改成按前置状态**参数化**，标 serial 是不够的（本 plan 实测 5/5 红）"
    - "`tracing::dispatcher::has_been_set()` 被 `with_default` 的线程局部 dispatcher 永久置真，不能用它证明「全局 subscriber 装上了」"
    - "编译期 `#[cfg]` 分叉而非运行期 `if`：运行期分支意味着命令仍在二进制里；`strings` 检索命令名是这条性质的直接实证"

key-files:
  created: []
  modified:
    - src-tauri/src/lib.rs
    - src-tauri/Cargo.toml
    - src-tauri/tests/ipc.rs

decisions:
  - "上限只钉 `rmcp` 一个 target，不把所有 target 压回 info：后者会让「开发者要调试的能力不被剥夺」这句话失效，单测里专门有一条 prism_mcp@TRACE 仍放行的断言守着这个区别"
  - "走「环境优先 + 追加封顶」而不是 plan 里给的备选「超限就整体回落到 DEFAULT_LOG_FILTER」——实测确认追加的指令确实压得住，粗路径没必要"
  - "降档 warn 排在 `try_init()` 之后：subscriber 没装上之前发的 warn 写向虚无"
  - "IN-02 的修法从 plan 写的「标 #[serial]」改成「#[serial] + 断言 ① 按前置状态参数化」——实测证明只标 serial 修不好（见反证 2），详见 Deviations"
  - "断言 ② 补一条 ②′ 而不是替换：`has_been_set()` 那条一字未删，只是它在本文件出现 `with_default` 之后已不足以证明全局装上了"
  - "ipc.rs 三处数词直接删掉而不是改对（上轮 IN-01 建议的两条路径里选后者）：长度由 `[&str; N]` 自己声明，散文不再有可漂移的量"

metrics:
  duration: ~40min
  tasks: 3
  files: 3
completed: 2026-07-29
---

# Phase 01 Plan 21: 日志天花板 + tracing 测试去全局前提 + release IPC 面收口 Summary

给日志 sink 装上一个 `RUST_LOG` 打不破的天花板（`rmcp` 钉在 `info`，降档时发一条不回显环境变量原值的 warn），把 tracing 测试对「本二进制里我是第一个装 subscriber 的」这条隐式前提拆掉，并把 `generate_handler!` 按 `debug_assertions` 分成两份命令列表——release 那一份里不再有能往用户真实数据库写 fixture 行的 `dev_seed_sample_docs`。

## What Was Built

### Task 1 —— 环境提供的日志过滤器加项目上限（commit `70cee58`）

`init_tracing()` 里那一行 `EnvFilter::try_from_default_env()` 拆成「读环境 → 探针 → 追加封顶」三步：

- 新增 `LOG_CEILING_DIRECTIVE = "rmcp=info"`，由 `build_log_filter()` 追加到环境提供的 filter 之上。`DEFAULT_LOG_FILTER` 上方那段注释扩写成两半——原来只解释「默认档为什么是 info」，现在补上「为什么这个 sink 还需要一个环境变量打不破的天花板」，点名 `rmcp-2.2.0` 的 `tracing::trace!(?message)` 与 T-01-58 / T-01-33。
- 新增 `log_filter_probe!` 宏：自造 `Callsite` + `Metadata` 常量，直接调 `Subscriber::enabled` 向手上这一个 filter 提问。刻意不走 `tracing::event_enabled!`——那个宏问的是进程当前的全局 dispatcher，且答案会经 callsite 兴趣缓存跨 subscriber 复用，判定结果于是取决于「这个测试二进制里此前装过什么」，正是本 plan 要拆掉的那类前置依赖。
- `build_log_filter()` 返回 `(EnvFilter, bool)`，第二位报告是否发生降档。探针那一份 filter 从环境重新构建（`EnvFilter` 不实现 `Clone`）。
- 降档 warn 的正文提为常量 `LOG_CEILING_WARNING`，**只陈述规则**，不回显 `RUST_LOG` 原值（T-01G-26）；调用点排在 `try_init()` 之后。

四条新单测（全部 `#[serial]`，用 `Drop` 型 `RustLogGuard` 设置/还原环境变量）：

| 测试 | 钉住的性质 |
|---|---|
| `the_project_ceiling_holds_against_a_global_rust_log_trace` | `RUST_LOG=trace` 下 rmcp@DEBUG 被拒、rmcp@INFO 仍放行、prism_mcp@TRACE 仍放行 |
| `the_project_ceiling_replaces_a_target_specific_rust_log_directive` | `RUST_LOG=rmcp=trace` 这条绕过全局档位的最短路径同样压得住 |
| `without_rust_log_the_filter_is_equivalent_to_the_default_filter` | 无 `RUST_LOG` 时行为等价 `info`，且不报降档 |
| `the_downgrade_warning_does_not_echo_the_rust_log_value` | 捕获实际日志输出，断言含 `capped at INFO`（负对照）且不含 `RUST_LOG` 里那个可识别 marker |

**实现选择**：plan 允许在「追加指令」与「超限整体回落到 `DEFAULT_LOG_FILTER`」之间按实测二选一。实测确认追加的那条压得住——`EnvFilter` 的 `Directives` 按**特异性**（target 长度 / 字段数）而非书写顺序排序取首个匹配，带 target 的 `rmcp=info` 比无 target 的全局 `trace` 更特异；同 target 同字段的指令则被 `binary_search` 直接替换。两种形态各有一条单测钉住，粗路径未启用。

`init_tracing()` 仍返回 bool、仍用 `try_init()`、仍是 `run()` 第一步；`run_installs_tracing_before_it_builds_the_app` 的两个源码序锚点未动。

### Task 2 —— tracing 测试去掉对进程全局前置条件的依赖（commit `9bc1fdf`）

`serial_test` 进 `src-tauri` 的 `[dev-dependencies]`（版本继承根 workspace，不引入新外部包），随 Task 1 一起落地——Task 1 那四条测试碰进程级 `RUST_LOG`，本身就必须串行（见 Deviations）。

`tracing_init_installs_a_global_subscriber_and_is_idempotent` 标 `#[serial]`，断言 ① 改成按前置状态参数化：

```rust
let was_installed_before = a_real_subscriber_is_in_place();
assert_eq!(init_tracing(), !was_installed_before, "...");
```

三条断言一条未删，另补一条 ②′。`a_real_subscriber_is_in_place()` 问的是「此刻眼前这个 dispatcher 不是 `NoSubscriber`」——不能用 `has_been_set()` 代替，那个标志由 `set_global_default` 与 `with_default` 共同置真且**永不复位**，而本文件 Task 1 那条捕获日志的测试正好用了 `with_default`（这是实跑中真实撞上的，见 Deviations）。

测试上方那段注释扩写成完整推理链：原注释解释「为什么不写前置断言」，现在补上「断言 ① 本身就是一条隐式前置条件」、「`#[serial]` 只挡并发挡不住顺序（实测五次全红）」、以及为什么 `#[serial]` 仍然要留（`has_been_set()` 与紧随的 `init_tracing()` 之间有读-改窗口，且 Task 1 那四条会改 `RUST_LOG`——`init_tracing()` 读它）。末尾写下约定：**本 crate 里任何会装 subscriber 或改 `RUST_LOG` 的测试都必须进这个串行组。**

### Task 3 —— dev 命令按构建形态分叉，修正 ipc.rs 陈旧计数（commit `ecf7fd0`）

`run()` 里的 `invoke_handler` 拆成两份，用 `#[cfg]` 作用在同一个 `builder` 绑定上：debug 那份十条不变，release 那份只有六条生产命令。分叉上方的注释写明为什么不能只靠前端门控（`App.tsx` 的 `import.meta.env.DEV` 摇掉的是 dev **按钮**不是 dev **命令**）、`dev_seed_sample_docs` 写进用户真实库的那些行 project id 是硬编码的 `smoke-project`、以及为什么不用运行期 `if`（那意味着命令仍在二进制里）。

新增源码断言 `the_release_ipc_surface_excludes_the_dev_commands`：从 `pub fn run()` 起切片，用**跨行**锚点定位 release 那一支，收在该语句的 `]);` 上，断言四条 `dev_*` 均不在其中，并配两条「生产命令确实在这一支里」的负对照。

`src-tauri/tests/ipc.rs`：三处数词（「六个」对 `[&str; 7]`、「两个」对 `[&str; 3]`、「八个」对 `[&str; 10]`）按上轮 IN-01 建议的第二条路径**直接删掉**——长度由 `[&str; N]` 自己声明，散文不再有可漂移的量。`COMMANDS` 上方的注释同时写明：本文件的 `mock_app()` 自建 handler 列表、不经 `run()`，两种构建形态下都是同一份；release 那一支由 `lib.rs` 的源码断言看住。

## 四条非恒真反证（全部实跑）

### 反证 1：去掉 filter 封顶 → 两条上限断言变红

Task 1 的 RED 阶段就是这条反证：探针机制与降档检测全部就位、`build_log_filter()` 里只差 `.add_directive(...)` 那一步。

```
test tests::the_project_ceiling_holds_against_a_global_rust_log_trace ... FAILED
test tests::the_project_ceiling_replaces_a_target_specific_rust_log_directive ... FAILED

---- tests::the_project_ceiling_holds_against_a_global_rust_log_trace stdout ----
thread '...' panicked at src-tauri/src/lib.rs:260:9:
RUST_LOG=trace 把 rmcp 放行到了 DEBUG —— 整条 MCP 消息会被倒进本地日志

---- tests::the_project_ceiling_replaces_a_target_specific_rust_log_directive stdout ----
thread '...' panicked at src-tauri/src/lib.rs:283:9:
环境里的 rmcp=trace 覆盖掉了项目上限指令

test result: FAILED. 15 passed; 2 failed
```

其余三条（默认档等价、warn 不回显）保持绿——落点精确在被守的那条性质上。加回 `.add_directive(...)` 后 `17 passed; 0 failed`。

### 反证 2：加一条装 subscriber 的测试 → 原断言 ① 五次全红，且 `#[serial]` 修不好它

在 `src-tauri/src/lib.rs` **本文件的** `mod tests` 里临时加一条名字排在被测测试之前、第一行调 `init_tracing()` 的测试，`--test-threads=4` 连跑五次：

**未标 `#[serial]`：**

```
run 1..5: test tests::tracing_init_installs_a_global_subscriber_and_is_idempotent ... FAILED
          test result: FAILED. 17 passed; 1 failed   (五次全红)
```

**补上 `#[serial]` 之后（plan 预期这里转绿）：**

```
run 1..5: test tests::tracing_init_installs_a_global_subscriber_and_is_idempotent ... FAILED
          test result: FAILED. 17 passed; 1 failed   (仍然五次全红)

thread '...' panicked at src-tauri/src/lib.rs:396:9:
the first init_tracing() should install
```

失败消息「the first init_tracing() should install」指着 `init_tracing`，而问题在那条临时测试——正是 IN-02 预言的「失败消息指向错误的代码」。**`#[serial]` 只挡并发、挡不住顺序**：那条测试字母序在前，先跑完就把 dispatcher 装上了，串行与否都改变不了这一点。修法因此改为「`#[serial]` + 断言 ① 按前置状态参数化」（见 Deviations）。

**修法落地后，racer 标不标 `#[serial]` 都是五次全绿：**

```
=== racer WITH #[serial] ===     run 1..5: ... ok | 18 passed; 0 failed
=== racer WITHOUT #[serial] ===  run 1..5: ... ok | 18 passed; 0 failed
```

两组都绿说明前置依赖是真的被拆掉了，而不是被串行化掩盖住。临时测试已删除，`git status --porcelain` 收尾干净。

### 反证 3：把 `dev_seed_sample_docs` 加回 release 那一支 → 源码断言变红

```
---- tests::the_release_ipc_surface_excludes_the_dev_commands stdout ----
thread '...' panicked at src-tauri/src/lib.rs:551:13:
release 的 IPC 面上仍注册着 commands::dev_seed_sample_docs, —— WebView 里任何脚本都能 invoke 它

test result: FAILED. 0 passed; 1 failed; 17 filtered out
```

还原后绿（`18 passed; 0 failed`）。

这条断言的锚点选取本身也实测过一轮：**起初只用单行 `#[cfg(not(debug_assertions))]` 作锚点，在实现根本没写分叉时也能走过 `expect`**——因为 `find` 先命中了本测试自己的字符串字面量。当时是那两条「生产命令确实在这一支里」的负对照把它捞了回来：

```
thread '...' panicked at src-tauri/src/lib.rs:524:13:
release 的命令列表里没有生产命令 commands::search_documents, —— 切片锚点已失效
```

最终锚点改成**跨行**的完整语句片段 `"#[cfg(not(debug_assertions))]\n    let builder = builder.invoke_handler("`——它在本文件源码里只有一处能命中，因为本测试自己那份是转义后的字面量（`\n` 是两个字符），与 `run()` 里真正的换行不同形。

### 反证 4：release 产物里检索命令名

`generate_handler!` 会把命令名作为字符串写进二进制，所以这条检索是分叉是否生效的直接实证。两次构建对照：

**分叉未生效的形态**（反证 3 那次临时把 `dev_seed_sample_docs` 加回 release 那一支时顺带构建）：

```
$ cargo build --release -p prismdocs-shell
    Finished `release` profile [optimized] target(s) in 46.04s
$ strings target/release/prismdocs | grep -c 'dev_seed_sample_docs'
1
```

命中处是 tauri 命令名表：`...set_api_keystatesecretprojectIdqset_base_urlurldelete_api_keydev_seed_sample_docsapi_key_status...`

**分叉生效的形态**（还原后重新构建）：

```
$ cargo build --release -p prismdocs-shell
    Finished `release` profile [optimized] target(s) in 9.94s
$ for c in dev_ping dev_emit_bus_event dev_smoke_stream dev_seed_sample_docs; do
    echo "$c: $(strings target/release/prismdocs | grep -c "$c")"; done
dev_ping: 0
dev_emit_bus_event: 0
dev_smoke_stream: 0
dev_seed_sample_docs: 0

$ for c in search_documents set_api_key api_key_status delete_api_key get_setting set_base_url; do
    echo "$c: $(strings target/release/prismdocs | grep -c "$c")"; done
search_documents: 1
set_api_key: 1
api_key_status: 1
delete_api_key: 1
get_setting: 1
set_base_url: 1
```

四条 dev 命令名 0 命中、六条生产命令名各 1 命中——负对照成立（检索本身有效，不是 `strings` 什么都找不到）。

## Verification

```
$ cargo test -p prismdocs-shell --features test
test result: ok. 18 passed; 0 failed        (lib)
test result: ok. 2 passed; 0 failed         (tests/ipc.rs)

$ cargo clippy -p prismdocs-shell --all-targets --features test -- -D warnings
    Finished `dev` profile   (0 warning)

$ cargo build --release -p prismdocs-shell
    Finished `release` profile [optimized]

$ bash scripts/check-deps.sh all
OK: no duplicate rusqlite/reqwest/libsqlite3-sys
OK: all checked crates are tauri-free (engine set + CLI helper)
OK: prism-mcp -> prism-types only
OK: leaf engine crates carry no network/secret dependency
OK: prism-engine only ever reaches network/secrets through prism-llm
OK: prismdocs-shell only ever reaches network/secrets through prism-engine -> prism-llm
OK: all checked crates are tracing-subscriber-free (engine set + CLI helper)

$ cargo test --workspace
（27 个 test binary 全绿，0 failed）

$ grep -nE '[一二三四五六七八九十]个命令' src-tauri/tests/ipc.rs
（无输出——三处数词已清）
```

## 日志人工验证步骤更新（供 01-27 汇总）

本 plan 改动了 Phase 收尾人工验证**第 2 项（日志 sink 是否真有落点）**的可行做法。原步骤若建议「用 `RUST_LOG` 提档观察」，现在必须改成：

1. **默认档位下观察**（不设 `RUST_LOG`）：在设置页填一个 `http://` 开头的 base url，确认终端出现 `settings.rs` 那条明文 http 告警。这是默认 `info` 档下就有落点的那条。
2. **额外确认天花板生效**：`RUST_LOG=trace npm run tauri dev`（或直接跑 release 之外的构建产物），确认：
   - 终端出现降档 warn：`the environment-supplied log filter exceeds the project ceiling; the \`rmcp\` target was capped at INFO because raising it dumps whole MCP message bodies into the local log sink`；
   - 该 warn 正文里**不含** `RUST_LOG` 的原值；
   - `rmcp` 没有开始转储 MCP 消息（Phase 1 尚未起 MCP server，此项在 Phase 5/6 才有实际可观测面；Phase 1 收尾只需确认前两条 + `prism_*` 的 trace 确实被放行，以证明上限是针对性的而非全局压制）。

## Deviations from Plan

### 1. [Rule 3 - Blocking] `serial_test` 的 dev-dependency 随 Task 1 落地，而不是 Task 2

- **Found during:** Task 1
- **Issue:** plan 把 `src-tauri/Cargo.toml` 划给 Task 2，但 Task 1 的 acceptance criteria 要求它新增的两条 filter 测试标 `#[serial]`（它们改进程级 `RUST_LOG`）——没有这个 dev-dependency，Task 1 自己的测试编译不过。
- **Fix:** `serial_test = { workspace = true }` 随 Task 1 的 commit 一起进 `[dev-dependencies]`，并在该行上方写明理由。Task 2 的其余内容（给既有 tracing 测试标 `#[serial]` + 扩写注释）不变。
- **Files modified:** `src-tauri/Cargo.toml`
- **Commit:** `70cee58`
- **不触发包合法性闸门：** 版本已在根 workspace 定义、`crates/prism-engine/tests/facade.rs` 已在用、`01-RESEARCH.md` § Package Legitimacy Audit 表里 verdict OK / Approved。未新增任何外部包（`Cargo.lock` 无新条目，仅路径依赖图更新）。

### 2. [Rule 1 - Bug] plan 为 IN-02 规定的修法（只标 `#[serial]`）修不好它

- **Found during:** Task 2
- **Issue:** plan 的 `behavior` 要求「断言 ① 保留，但不再依赖『本二进制里没有第二处装 subscriber』」，`action` 与 acceptance criteria 给的手段是标 `#[serial]`，并预期「给临时测试补上 `#[serial]` 之后五次全绿」。实跑证伪：`#[serial]` 序列化的是**执行**，不是**顺序**——字母序在前的那条测试无论标不标 serial 都先跑完并装上 dispatcher，原断言 ①（「首次调用返回 true」）五次全红（反证 2 的两组输出）。按 plan 原样收工会留下一条仍然会闪红、且失败消息仍指向错误代码的测试。
- **Fix:** 断言 ① 改成按前置状态参数化 —— `assert_eq!(init_tracing(), !was_installed_before)`。这条不依赖调用顺序，判别力也没丢（恒返回 true 或恒返回 false 的桩实现在两种世界里各有一种会被逮住，②②′③ 兜住其余）。`#[serial]` 仍保留，理由改为「挡 `has_been_set()`/`init_tracing()` 之间的读-改窗口，以及与 Task 1 那四条共享 `RUST_LOG`」，并把这条推理写进注释。三条原断言一条未删。
- **Files modified:** `src-tauri/src/lib.rs`
- **Commit:** `9bc1fdf`

### 3. [Rule 1 - Bug] `has_been_set()` 被 `with_default` 永久置真，断言 ② 的判别力被 Task 1 稀释

- **Found during:** Task 2（删掉临时 racer 后单跑，测试反而红了）
- **Issue:** 上一条修法的第一版用 `tracing::dispatcher::has_been_set()` 当「调用前是否已装」的判据，结果 `18 passed` 的并行跑法下绿、单独跑却红：
  ```
  assertion `left == right` failed: init_tracing() 的返回值与「调用前全局 dispatcher 是否已就位」不一致
    left: true
   right: false
  ```
  根因是 `has_been_set()` 由 `set_global_default` 与 `with_default` **共同**置真且永不复位。Task 1 那条捕获日志的测试用了 `with_default`（线程局部），从此该标志恒为真，哪怕全局槽位仍然空着。这同时意味着既有断言 ②（`assert!(has_been_set())`）的判别力被本 plan 的新测试**稀释**了——一个什么都不做的 `init_tracing` 桩现在也能让它绿。
- **Fix:** 新增 `a_real_subscriber_is_in_place()`（`get_default(|d| !d.is::<NoSubscriber>())`）作为准确判据，断言 ① 改用它；断言 ② 原样保留、另补一条 ②′ 用同一判据把 ② 想说的那件事拉回原本的强度。`a_real_subscriber_is_in_place` 的注释写明为什么不能用 `has_been_set()`。
- **Files modified:** `src-tauri/src/lib.rs`
- **Commit:** `9bc1fdf`

### 4. [Rule 2 - Missing] 源码断言的锚点从单行改为跨行完整语句

- **Found during:** Task 3
- **Issue:** plan 要求「锚点取完整语句片段而非裸名字」，理由是裸名字会命中解释性注释。实跑暴露出同一类问题的另一个形态：单行锚点 `"#[cfg(not(debug_assertions))]"` 会先命中**本测试自己的字符串字面量**，于是在实现根本没写分叉时也能走过 `expect`，判别力全靠负对照捞。
- **Fix:** 锚点改成跨行的 `"#[cfg(not(debug_assertions))]\n    let builder = builder.invoke_handler("`——本测试自己那份是转义后的字面量（`\n` 是两个字符），与源码里真正的换行不同形，因此全文只有 `run()` 里一处能命中。理由写进测试的 doc 注释，作为本 phase 源码序断言经验的第三条。
- **Files modified:** `src-tauri/src/lib.rs`
- **Commit:** `ecf7fd0`

### 未做的事（scope boundary）

- **未跑 `cargo fmt`**：本仓库有早于本 plan 的 rustfmt 漂移且无 CI fmt 闸门（`deferred-items.md` 已登记，由 01-28 关闭）。本 plan 新写的代码按 rustfmt 风格手写。
- **未触碰同波次其他 plan 的文件**：`src-tauri/src/commands.rs` / `src-tauri/src/smoke.rs`（01-22）、`src-tauri/tauri.conf.json`（01-24）一字未改。反证 2 的临时测试按 plan 要求放在 `lib.rs` 本文件的 `mod tests` 里。
- **INFRA-01 edge probe 的另一半未新增证据**：事件总线与 Channel 有序流在并发下的保证，仍由既有的 `bus_adapter` Lagged→Resync 三分支单测与 `smoke` 逐位序列比较承担（plan 的 backstop 条目已声明本轮不新增）。

## Known Stubs

无。本 plan 未引入任何硬编码空值、占位文本或未接数据源的组件。

## Threat Flags

无。本 plan 只**缩小**了两个既有面（日志 sink 的档位上界、release IPC 面的命令数），未引入新的网络端点、认证路径、文件访问模式或信任边界上的 schema 变更。

## Self-Check: PASSED

```
$ for f in src-tauri/src/lib.rs src-tauri/Cargo.toml src-tauri/tests/ipc.rs; do
    [ -f "$f" ] && echo "FOUND: $f" || echo "MISSING: $f"; done
FOUND: src-tauri/src/lib.rs
FOUND: src-tauri/Cargo.toml
FOUND: src-tauri/tests/ipc.rs

$ for h in 70cee58 9bc1fdf ecf7fd0; do
    git log --oneline --all | grep -q "$h" && echo "FOUND: $h" || echo "MISSING: $h"; done
FOUND: 70cee58
FOUND: 9bc1fdf
FOUND: ecf7fd0
```
