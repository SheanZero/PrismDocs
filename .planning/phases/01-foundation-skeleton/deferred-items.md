# Phase 1 — Deferred Items

执行期发现、但落在当前 plan 范围之外的事项。**不在本 phase 修**，登记在此供后续 plan 或
`/gsd-verify-work` 决定去留。

## 发现于 plan 01-03

### 1. workspace 未通过 `cargo fmt --check`，且 CI 无 fmt 闸门

`cargo fmt --all -- --check` 在 `prism-anchor/src/lib.rs`、`prism-cli/src/main.rs`、
`prism-store/src/lib.rs` 等多处报差异（全部为 rustfmt 的换行偏好，非正确性问题）。
`.github/workflows/ci.yml` 只跑 `check-deps` / `check-secrets` / `clippy` / `test`，
**没有 fmt 步骤**，所以这些差异不会让 CI 变红。

- 差异早于 plan 01-03（plan 01-01 建立的文件里就有），属既有状态，按 scope boundary 不在本 plan 修
- 要么补一个 `cargo fmt --all -- --check` 的 CI 步骤并一次性格式化全仓，要么明确记为「不采用 rustfmt 默认风格」
- 建议在 plan 01-09（CI 收尾）或 `/gsd-verify-work` 时一次性拍板，避免逐 plan 各自格式化造成风格拉锯

## 01-05 登记

- **`state add-decision` 写出的行前缀是 `[Phase ?]` 而非 `[Phase 1]`**（.planning/STATE.md 第 82–94 行）。
  自 01-02 起每个 plan 追加的决策都带这个占位符，01-01 手写的四条则是 `[Phase 1]`。
  属于 gsd-tools 侧的行为，非本仓库代码；范围外未修，留待 milestone 收尾时统一整理。

## 01-18 登记

- **`prism-engine/src/lib.rs` 的版本串仍是 `filter_map` 惯用法**（上轮 IN-03 点名的四处之一）。
  01-18 只改了 `open.rs` 那一处（唯一进入准入判定路径的），01-19 覆盖 `lib.rs` 的 `parts`
  与 `tests/concurrency.rs` 的 `version_tuple`，**`prism-engine` 那一处两个 plan 都没覆盖**。
  它是展示用途（不参与任何准入或比较判定），按 scope boundary 未修；建议在 `/gsd-verify-work`
  或下一轮评审时一并处理。
- **计划文本里的 `3.x.51` 反例不具判别力**：它塌缩成 `(3,51,0)`，恰好也低于 `MIN_SQLITE=(3,51,3)`，
  于是准入结果与正确实现一致。真正的放行口是 `3.x.53` → `(3,53,0)`。已在 01-18 的测试用例与
  `parse_sqlite_version` 注释里更正；若后续 plan 引用了同一反例，需一并更正。
