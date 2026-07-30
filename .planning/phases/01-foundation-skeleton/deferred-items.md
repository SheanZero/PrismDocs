# Phase 1 — Deferred Items

执行期发现、但落在当前 plan 范围之外的事项。**不在本 phase 修**，登记在此供后续 plan 或
`/gsd-verify-work` 决定去留。

本文件的两条候选（01-03 的 rustfmt、01-05 的 `[Phase ?]` 前缀）已于 Phase 1 收尾的 gap-closure
轮次（plan 01-25）逐条定案，去向见各条下方的同名小节；本文件此后只承接**新**发现的范围外事项。

## 发现于 plan 01-03

### 1. workspace 未通过 `cargo fmt --check`，且 CI 无 fmt 闸门

`cargo fmt --all -- --check` 在 `prism-anchor/src/lib.rs`、`prism-cli/src/main.rs`、
`prism-store/src/lib.rs` 等多处报差异（全部为 rustfmt 的换行偏好，非正确性问题）。
`.github/workflows/ci.yml` 只跑 `check-deps` / `check-secrets` / `clippy` / `test`，
**没有 fmt 步骤**，所以这些差异不会让 CI 变红。

- 差异早于 plan 01-03（plan 01-01 建立的文件里就有），属既有状态，按 scope boundary 不在本 plan 修
- 要么补一个 `cargo fmt --all -- --check` 的 CI 步骤并一次性格式化全仓，要么明确记为「不采用 rustfmt 默认风格」
- 建议在 plan 01-09（CI 收尾）或 `/gsd-verify-work` 时一次性拍板，避免逐 plan 各自格式化造成风格拉锯

#### 去向（Phase 1 收尾定案）

**已排进本轮 gap-closure 的 `01-28-PLAN.md`**（wave 6，本轮最后一份），形态是一个
`checkpoint:decision`（该 plan 的 Task 1，也是本轮唯一的 `checkpoint:decision`）：

- **选项 A**：补一个 `cargo fmt --all -- --check` 的 CI 步骤（engine job 最前）并一次性格式化全仓，
  同时创建 `rustfmt.toml`（空文件即代表显式采用 rustfmt 默认风格）与 `justfile` 的 `fmt-check` recipe。
- **选项 B**：明确记为「本项目不采用 rustfmt 默认风格」，不加闸门；决定记录（含日期与理由）写在
  `justfile` 文件头，仓库里不放 `rustfmt.toml`。

**可逆性是 `costly` 而非 one-way**：两个方向都只是一次全仓 diff，没有已发布契约被破坏、不需要数据迁移；
但选项 A 会产生一个触及每一个 Rust 文件的提交，且此后每个 phase 都受它约束。

**为什么必须现在拍板**：逐 plan 各自格式化会造成风格拉锯，而每多一个 phase 就多一批文件——成本随时间
单调上升。选项 A 另有一个已知风险需在执行时看住：本仓库多条 `include_str!` 源码序断言靠字符串片段定位
（`open.rs` / `lib.rs` / `commands.rs` / `services.rs` / `middleware.rs`），rustfmt 的换行偏好可能把某个
锚点拆到两行，使断言在实现完全正确时变红；届时修的是**锚点**，不是断言本身。

`01-28-PLAN.md` 的 Task 2 会把最终结论回填到本小节。本小节只做指向，不做决策。
（记账澄清：rustfmt 决策不在 01-27——01-27 只做 CI 闸门接线，其 objective 亦写明二选一定案在 01-28。）

##### 定案：选项 A（2026-07-30，01-28 Task 1）

**上面那句「本小节只做指向，不做决策」写于计划期，到此为止**——它承诺的回填就是本小节。
上面的选项 A / B 原文一字未改地留着：它们是这次决策的备选记录，不是待办。

**用户在 `01-28-PLAN.md` 的 `checkpoint:decision` 上选定 `option-a`**：采用 rustfmt 默认风格，
一次性格式化全仓并加 CI 闸门。呈给用户的决策材料是本机实跑读数而非印象描述：
`cargo fmt --all -- --check` 报 **38 个 hunk，覆盖 40 个受版本控制 `.rs` 文件中的 23 个**，
落地后的真实 diff 为 **+142 / −82 行**，全部为换行与缩进偏好，无一处改变语义。

落地形态（三件套缺任何一件，这个决定就只是一句声明）：

- `rustfmt.toml`（仓库根，新建）——**不含任何设置项；空本身就是「显式采用默认风格」这个决定**。
  文件头写明了这一点与做出决定的日期，使下一个读者不会把它当成「没人配置过」。
- `.github/workflows/ci.yml` engine job 的**第一条实质步骤** `cargo fmt --all -- --check`，
  且 `dtolnay/rust-toolchain@stable` 的 `components` 里**显式列出 `rustfmt`**（不靠默认 profile 捎带）。
- `justfile` 的 `fmt-check` recipe——单行委托，与 CI 那步逐字等价。

**已知风险的实测结论：全部 7 处 `include_str!` 源码序断言在格式化后逐条重跑仍绿，0 处锚点被修改。**
唯一真正被格式化触及的重叠点是 `src-tauri/src/lib.rs:167`（`tauri::Builder::default()` 与
`.setup(` 被并到同一行），而该处锚点取的是 `tauri::Builder::default()` 这个**完整语句片段**，
并行不影响子串匹配——这是 01-21 把锚点从裸名字收窄为完整语句时顺带买到的韧性。

**仍未证明的那一半（不要当成已验证）**：本条的证据止于「本机 `cargo fmt --all -- --check` 退出 0，
且注入劣化排版后退出 1 并点名文件」。这条闸门**在 GitHub Actions 上是否真的会红，与 WINDOWS id=14
一样尚未观测**——该 workflow 至今未在 CI 上跑过。首次真实 CI 运行时应与 id=14 一并核对。

## 01-05 登记

- **`state add-decision` 写出的行前缀是 `[Phase ?]` 而非 `[Phase 1]`**（.planning/STATE.md 第 82–94 行）。
  自 01-02 起每个 plan 追加的决策都带这个占位符，01-01 手写的四条则是 `[Phase 1]`。
  属于 gsd-tools 侧的行为，非本仓库代码；范围外未修，留待 milestone 收尾时统一整理。

#### 去向（Phase 1 收尾定案）

**继续顺延。** 理由：它属于 gsd-tools 侧的行为、**不是本仓库的代码**——本轮 gap-closure 的「全部清干净」
范围是本仓库的代码与配置，改不了的东西不能靠本轮的计划关闭（写一份 plan 去改它只会产出一份必然落空的
计划）。

**具体去处**（二者不互斥）：

- milestone 收尾时统一整理 `.planning/STATE.md` 的决策前缀，把 `[Phase ?]` 批量归位为实际的 Phase 编号；
- 或向 gsd-tools 上游反馈 `state add-decision` 未继承当前 phase 编号这一行为。

**实际影响是记账可读性，不是正确性**：决策内容本身完整无损，只有 Phase 编号是占位符；任何按内容检索
决策的用法都不受影响，受影响的只有按 Phase 分组阅读 STATE.md 的场景。

## 01-18 登记

- **`prism-engine/src/lib.rs` 的版本串仍是 `filter_map` 惯用法**（上轮 IN-03 点名的四处之一）。
  01-18 只改了 `open.rs` 那一处（唯一进入准入判定路径的），01-19 覆盖 `lib.rs` 的 `parts`
  与 `tests/concurrency.rs` 的 `version_tuple`，**`prism-engine` 那一处两个 plan 都没覆盖**。
  它是展示用途（不参与任何准入或比较判定），按 scope boundary 未修；建议在 `/gsd-verify-work`
  或下一轮评审时一并处理。
- **计划文本里的 `3.x.51` 反例不具判别力**：它塌缩成 `(3,51,0)`，恰好也低于 `MIN_SQLITE=(3,51,3)`，
  于是准入结果与正确实现一致。真正的放行口是 `3.x.53` → `(3,53,0)`。已在 01-18 的测试用例与
  `parse_sqlite_version` 注释里更正；若后续 plan 引用了同一反例，需一并更正。
