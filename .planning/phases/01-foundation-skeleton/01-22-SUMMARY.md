---
phase: 01-foundation-skeleton
plan: 22
subsystem: dev-smoke-path-input-bounds-and-a11y
tags: [input-validation, async-runtime, accessibility, dev-only, gap-closure]
status: complete

requires:
  - "src-tauri/src/commands.rs 模块头第 47-51 行确立的 `delegate` / `spawn_blocking` 纪律（01-07）"
  - "src-tauri/src/smoke.rs 的 `generate` / `collect` 与它的逐位序列比较断言（01-08）"
  - "src/pages/Settings.tsx 的 `Notice` 类型与 `NoticeLine`（01-11）——本 plan 采用它的形状"
  - "src/lib/ipc.ts 的 `LISTEN_FAILED` 语义（「本页从此收不到任何事件」）"
  - "01-REVIEW-prior.md WR-08 / IN-06"
provides:
  - "`SMOKE_MAX_TOTAL = 10_000` —— 冒烟流的硬上界，单一定义在 smoke.rs，`generate` 与 `collect` 共用一个 `clamp_total`"
  - "上界与冒烟页默认值的大小关系由 `const _: () = assert!(...)` 在编译期看住，而不是靠单测"
  - "`dev_smoke_stream` 经 `spawn_blocking` 落到阻塞线程池，与本文件既有的 `delegate` 同口径"
  - "commands.rs 新增一条**函数体切片**型源码断言：切出 `dev_smoke_stream` 的体，要求它含 spawn_blocking 的完整调用语句"
  - "冒烟页的 `Notice`/`NoticeLine` —— 与 Settings.tsx 逐字同形的第二份，error → alert / ok → status"
  - "一对正反断言：listen 失败必须在 alert 里，成功必须不在 alert 里"
affects:
  - "Phase 收尾人工验证：冒烟页「实收 1000 条」读数不变（1000 < 10_000，正常路径未被夹紧）"
  - "后续任何新页面的通知形状：第三种形状会与两处 `type Notice` 直接对不上"
  - "后续 Channel 型命令：`dev_smoke_stream` 不再是本文件唯一破例的那个，纪律回到全覆盖"

tech-stack:
  added: []
  patterns:
    - "夹紧点放在**不依赖 tauri 的模块**里（`smoke::clamp_total`）而不是命令层：上界因此不需要真实 Channel、不需要 mock runtime 就能被断言，与 smoke.rs 模块头写下的设计理由一致"
    - "「上界必须高于默认值」写成 `const _: () = assert!(...)` 而非单测：clippy 的 `assertions_on_constants` 会把常量间的 `assert!` 挡在测试里，而编译期断言恰好是它建议的形态——一次误编辑直接编译不过"
    - "源码断言取**函数体切片**而非整文件 `contains`：`split_once(签名).1.split(\"\\n#[\").next()` 把范围锁在这一个函数上，邻居的 `spawn_blocking` 冒充不了它"
    - "阴性对照的判别力不由「它绿」证明，由「破坏性改动让它红」证明——本 plan 的 `NoticeLine` 无条件 alert 反证给出了这个证据"

key-files:
  created: []
  modified:
    - src-tauri/src/smoke.rs
    - src-tauri/src/commands.rs
    - src/pages/DevSmoke.tsx
    - src/pages/DevSmoke.test.tsx

decisions:
  - "夹紧实现在 `smoke::generate` 内部（经 `clamp_total`）而不是 plan `<action>` 字面写的「`dev_smoke_stream` 先取小」：plan 同时要求 smoke.rs 的 `mod tests` 里有一条「上界+1 产出等于上界」的断言，而那条断言只有在夹紧发生于 `generate` 时才观测得到。两处各夹一次会让「哪一个是承重的」变得含糊，故只留一处，命令侧改为文档注释指向它"
  - "`Started` / `Finished` 携带的是**夹紧后**的 total，不是原始入参：否则一条完好的流会让前端 `实收 N 条 == Started 报的 total` 这条校验假红"
  - "上界与默认值的关系用编译期断言而非单测：clippy 直接拒绝常量间的运行期 `assert!`，而这里编译期形态本就更强"
  - "`spawn_blocking` 的落点用**源码断言**而非 `Handle::try_current()` 行为探针（plan 给的二选一里选后者）：探针的答案取决于 `tauri::async_runtime` 当前选的后端，那是实现细节，钉住它等于给自己埋一颗随上游升级而红的雷"
  - "冒烟页的 ok tone 加了 `role=\"status\"`，Settings.tsx 的 ok 分支则没有 role——两处 `NoticeLine` 因此不逐字相同。plan 的验收项只要求 `Notice` **类型**逐字相同（已满足），而 must_haves 明确要求成功通知落在 `role=\"status\"`，故按后者实现"
  - "不新增 IPC 短码：join 失败复用 `task_failed`、发送失败复用 `channel_send_failed`。新增会连带要求前端 `ERROR_COPY` 扩表，那是 IPC 短码契约变更"

metrics:
  duration: ~25min
  tasks: 2
  files: 4
completed: 2026-07-29
---

# Phase 01 Plan 22: 冒烟通路的输入上界、执行落点与失败可听见性 Summary

给 `dev_smoke_stream` 装上一个 WebView 里的脚本打不破的上界（`SMOKE_MAX_TOTAL = 10_000`，夹紧点在不依赖 tauri 的 `smoke` 模块里因此可脱离 Channel 断言），把它那个一次都不让出的同步循环从 IPC executor 上挪进阻塞线程池，并让冒烟页的失败通知走 `role="alert"`——成功仍走 `role="status"`，这条区分由一对正反断言看住。

## What Was Built

### Task 1 —— 上界 + offload（RED `116e860` / GREEN `f51f116`）

**夹紧（T-01G-27 / T-01G-29）。** `smoke.rs` 新增 `SMOKE_MAX_TOTAL: u32 = 10_000` 与私有的 `clamp_total`，`generate` 与 `collect` 都经它——上界的数字于是只有一处（`grep -c '10_000'` 在 commands.rs / smoke.rs 合计为 **1**）。

夹紧放在 `generate` 内部而不是命令层，理由是 smoke.rs 模块头自己写下的那条：本模块刻意不依赖 tauri，把上界放进来，它就和有序性、空输入两条断言一样不需要真实 Channel 就能验证。`Started` / `Finished` 携带的也是夹紧后的值——否则一条完好的流会让前端 `verifySmokeStream` 的「实收 N 条 == 期望 total」假红。`collect` 的 `Vec::with_capacity` 同样按 `clamp_total` 走：未校验的 `total` 直进 `with_capacity` 意味着一次 `u32::MAX` 调用先申请十几 GB，而循环连一条都还没产出。

「上界必须严格高于冒烟页固定使用的 1000」这条关系写成 **编译期断言**：

```rust
const _: () = assert!(SMOKE_DEFAULT_TOTAL < SMOKE_MAX_TOTAL);
```

起因是 clippy 的 `assertions_on_constants` 拒绝了单测里的运行期版本，而它建议的 const 形态在这里本就更强——把上界误调到 1000 以下会直接编译不过，而不是等某条测试来报。这一条守的是 plan `must_haves` 里那条 backstop：夹紧正常路径会让「实收 1000 条」这个人工验证判据静默失效。

**offload（T-01G-28）。** `dev_smoke_stream` 改为：

```rust
tauri::async_runtime::spawn_blocking(move || smoke::generate(total, |ev| on_event.send(ev)))
    .await
    .map_err(|_| ERR_TASK.to_string())?
    .map_err(|_| ERR_CHANNEL.to_string())
```

`tauri::ipc::Channel<SmokeEvent>` 直接 move 进闭包即满足 `Send + 'static`，无需先 clone（plan 预留的那条退路未启用）。两条短码都是既有的，未新增。

新增两条 Rust 断言：

| 测试 | 钉住的性质 |
|---|---|
| `smoke::tests::smoke_stream_clamps_a_total_above_the_ceiling` | `collect(MAX+1)` 的 tick 数等于 `MAX`，且首末事件报的也是夹紧后的值 |
| `smoke::tests::the_default_total_passes_through_unclamped` | 与上一条**成对**：单看上一条，`total.min(1)` 也能绿 |
| `commands::tests::dev_smoke_stream_hands_the_loop_to_the_blocking_pool` | `dev_smoke_stream` **函数体切片**里含 `spawn_blocking(move \|\| smoke::generate(` |

第三条取的是函数体切片（`split_once(签名).1` 再切到下一个 `\n#[`），不是整文件 `contains`——后者会被文件里别处的 `delegate` 冒充。

### Task 2 —— 失败通知走 alert（RED `d0fb6a2` / GREEN `baee043`）

冒烟页的 `notice` 从裸字符串换成与 `Settings.tsx` **逐字同形**的 `{ tone, text }`，并新增一份 `NoticeLine`：error tone → `role="alert"` 且红，ok tone → `role="status"` 且绿，无通知时不渲染任何节点。四个 handler 的 catch 分支与 listen 失败分支统一写 error tone，`handleSeed` 的成功文案写 ok tone。

两个页面各写一份 `NoticeLine` 是本 phase 刻意接受的重复（D-06 禁的是投机建共享布局层），冒烟页那份的注释里点名它与 Settings.tsx 同源。

测试侧：listen-失败断言从 `findByRole("status")` 改为 `findByRole("alert")`；新增阴性对照 `keeps a successful notice out of the alert region`——成功通知必须出现在 `status` 里且 `queryByRole("alert")` 为 null。

## Verification

```
cargo test -p prismdocs-shell --features test
  → lib 21 passed / tests/ipc.rs 2 passed（含 dev_smoke_stream 的可达性断言）/ 0 failed
cargo clippy -p prismdocs-shell --all-targets --features test -- -D warnings
  → Finished，0 warning
npm run test -- --run
  → Test Files 7 passed (7) / Tests 35 passed (35)
npx tsc --noEmit
  → 0 error
grep -c '10_000' src-tauri/src/commands.rs src-tauri/src/smoke.rs
  → commands.rs:0  smoke.rs:1（合计 1）
grep -h 'type Notice = ' src/pages/DevSmoke.tsx src/pages/Settings.tsx | sort -u | wc -l
  → 1（两处逐字相同）
```

### 四条非恒真反证（全部实跑）

**A —— 去掉 `generate` 里的夹紧那一步**

```
test smoke::tests::smoke_stream_clamps_a_total_above_the_ceiling ... FAILED
panicked at src-tauri/src/smoke.rs:100:9:
  left: 10001
 right: 10000
test smoke::tests::the_default_total_passes_through_unclamped ... ok
test result: FAILED. 20 passed; 1 failed
```

配对的默认值断言在同一次运行里保持绿——夹紧断言红的原因是「没夹」，不是「夹过头」。还原后 21 passed。

**B —— 去掉 `spawn_blocking`，改回直接调用**

```
test commands::tests::dev_smoke_stream_hands_the_loop_to_the_blocking_pool ... FAILED
panicked at src-tauri/src/commands.rs:197:9:
dev_smoke_stream 没有把 smoke::generate 交给 spawn_blocking —— 这个循环会跑在 IPC executor 上:
( on_event: tauri::ipc::Channel<SmokeEvent>, total: u32, ) -> Result<(), String> {
smoke::generate(total, |ev| on_event.send(ev)).map_err(|_| ERR_CHANNEL.to_string()) }
test result: FAILED. 20 passed; 1 failed
```

失败信息里回显的切片正好只有 `dev_smoke_stream` 一个函数——切片范围本身也被这次输出验证了。还原后绿。

**C —— 把 listen 失败分支的 tone 改回 ok**

```
× surfaces a rejected listen instead of silently sitting at zero
TestingLibraryElementError: Unable to find role="alert"
Tests  1 failed | 10 passed (11)
```

**D —— `NoticeLine` 无条件渲染 `role="alert"`**

```
✓ surfaces a rejected listen instead of silently sitting at zero 16ms
× keeps a successful notice out of the alert region 1044ms
FAIL  keeps a successful notice out of the alert region
Tests  1 failed | 10 passed (11)
```

这一条是本 Task 的关键证据：一个「把所有通知都改成 alert」的实现**能**让 listen-失败断言绿，而阴性对照会红。两条断言合起来看住的是「区分」，不是「有没有 alert」。

## Deviations from Plan

### 1. [实现选择] 夹紧点落在 `smoke::generate` 内部，而非 `dev_smoke_stream` 命令体

- **发生在:** Task 1
- **情况:** plan 的 `<action>` 写「`dev_smoke_stream` 先把 `total` 与上界取小」，但同一 Task 又要求 `smoke.rs` 的 `mod tests` 里有一条「`generate` 在上界+1 下产出等于上界」的断言。后者只有在夹紧发生于 `generate` 时才观测得到。
- **处理:** 只在 `generate`（经 `clamp_total`）夹一次，命令侧改为在文档注释里指向它。两处各夹一次会让「哪一个是承重的」变得含糊，而 plan 明确要求「保持单一定义」。
- **对 must_haves 的影响:** 无。「`dev_smoke_stream` 传入 `u32::MAX` 时实际生成的 tick 数等于上界」照样成立，且反证 A 直接证明了它非恒真。
- **提交:** `f51f116`

### 2. [Rule 3 - 阻塞] clippy `assertions_on_constants` 拒绝了单测里的大小关系断言

- **发生在:** Task 1，`cargo clippy -- -D warnings` 编译失败
- **情况:** 最初把「`SMOKE_DEFAULT_TOTAL < SMOKE_MAX_TOTAL`」写成测试里的 `assert!`，clippy 报 `-D clippy::assertions-on-constants` 并建议 `const { assert!(..) }`。
- **处理:** 提为生产代码里的 `const _: () = assert!(...)`，测试那条相应瘦身为「默认路径读数不被夹紧」。结果比原方案强：误编辑直接编译不过。
- **提交:** `f51f116`

### 3. [文档一致性] `SMOKE_MAX_TOTAL` 的文档注释不再复述字面量 `10_000`

- **发生在:** Task 1
- **情况:** 验收项要求 `grep -c '10_000'` 合计为 1，而文档注释里「取 10_000 而不是……」也算一处命中。
- **处理:** 注释改为描述关系（「留在 `SMOKE_DEFAULT_TOTAL` 之上一个数量级」）而不复述数字。plain grep 现在也是 1，不必退到「以常量名 grep」的备选口径。
- **提交:** `f51f116`

### 4. [形状取舍] 冒烟页 ok tone 带 `role="status"`，Settings.tsx 的 ok 分支不带

- **发生在:** Task 2
- **情况:** plan 要求「采用 `Settings.tsx` 同一形状」，但 must_haves 同时要求「成功通知仍在 `role="status"`」；Settings.tsx 的 ok 分支实际上没有 role。
- **处理:** 按 must_haves 实现——`Notice` **类型**与 Settings.tsx 逐字相同（验收项要求的正是类型），`NoticeLine` 的 ok 分支多一个 `role="status"`。这也是阴性对照能成立的前提。
- **遗留:** Settings.tsx 的成功通知目前不在任何 live region 里，读屏对它完全静默。这不在本 plan 的 `files_modified` 范围内，已记入下方 Deferred。
- **提交:** `baee043`

## Deferred Issues

- **Settings.tsx 的成功通知不在任何 live region 里**（`src/pages/Settings.tsx` `NoticeLine` 的 ok 分支只有颜色，无 `role`）。同一条 IN-06 推理适用：读屏用户拿不到「已保存」的反馈。本 plan 的 `files_modified` 不含该文件、且 01-23 正在改它，避免冲突，未动。
- 仓库存在 rustfmt 漂移且无 CI fmt 门（`deferred-items.md` 已登记，01-28 正在关闭）。本 plan 按约定**未**运行 `cargo fmt`，新增代码按 rustfmt 口径手写。

## Known Stubs

无。本 plan 未引入任何占位实现、硬编码空值或未接数据源的组件。

## Threat Flags

无。本 plan 未引入新的网络端点、认证路径、文件访问模式或信任边界处的 schema 变更；`threat_model` 里 T-01G-27 / 28 / 29 / 30 四条 `mitigate` 全部落地，T-01-SC（依赖安装）保持 accept——未新增任何依赖。
