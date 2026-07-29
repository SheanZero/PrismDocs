---
phase: 01-foundation-skeleton
plan: 25
subsystem: requirements-traceability-and-deferred-items-bookkeeping
tags: [bookkeeping, traceability, requirements, deferred-items, gap-closure]
status: complete

requires:
  - ".planning/REQUIREMENTS.md 的 Traceability 表与 INFRA-04/05 既有注（本次加注沿用其形态）"
  - ".planning/phases/01-foundation-skeleton/01-VERIFICATION.md § Requirements Coverage 与「INFRA-03 的顺延判断」段（记录了 Phase 1 已完成的三部分）"
  - ".planning/ROADMAP.md Phase 4 的 goal「A3：prism-llm 传输层（流式/重试/keyring）先行交付」——改映射的依据"
  - "01-14 / 01-23 两份 gap-closure plan 的 requirements: [INFRA-03] 声明"
provides:
  - "INFRA-03 在 Phase 1 关闭后仍有 Phase 4 负责推进——不再是孤儿需求"
  - "表下注就地保留 Phase 1 已完成并验证的三部分（钥匙串 / 唯一出口入口 / base_url 值侧校验），改映射不抹掉既有成果"
  - "deferred-items.md 两条候选各有明确去向：rustfmt 指向 01-28 的 checkpoint:decision，[Phase ?] 前缀记为顺延并写明理由与去处"
  - "deferred-items.md 顶部的新契约：此后只承接新发现的范围外事项"
affects:
  - "Phase 4 规划时：INFRA-03 会出现在该 phase 的需求集里，其待做部分只剩「实际向 Anthropic/OpenAI 兼容端点发请求」——表下注已把 Phase 1 完成面写死，避免重复实现"
  - "01-28 执行时：Task 2 需回填 deferred-items.md 里 rustfmt 那条的「去向」小节（本 plan 已建好该小节，留待写入最终结论）"
  - "/gsd-verify-work 的 orphan 检查：Phase 1 的映射集缩为 INFRA-01 / INFRA-02 两条"

tech-stack:
  added: []
  patterns:
    - "跨切需求改映射时用「改映射 + 表下注」而非拆条：GSD 的一需求一 Phase 是硬规则，拆条会绕过它；注是记录既有成果的正确位置"
    - "顺延项的正确关闭形态有两种——「排进一份具体的 plan」与「写明为何本轮改不了 + 具体去处」；两者都比继续悬着强，而「悬着」在文件里与「已接受的取舍」长得一样"
    - "记账文件的改动用 Edit 局部替换而非 Write 整份：60+ 行的表整文件写入会静默丢掉与本次改动无关的条目"

key-files:
  created:
    - .planning/phases/01-foundation-skeleton/01-25-SUMMARY.md
  modified:
    - .planning/REQUIREMENTS.md
    - .planning/phases/01-foundation-skeleton/deferred-items.md

decisions:
  - "INFRA-03 行**移动**到 Phase 4 分组（DIGEST-08 之后）而非留在原位改值：表是按 Phase 分组的，留在 Phase 1 段里写 Phase 4 会让分组失真；移动经 diff 逐行核对未连带删改相邻行"
  - "表下注写成四点编号列表而非单段：注要承载「哪三部分已完成」「哪三份 plan 认领它」「剩余分句归属」「为何不拆条」四件独立的事，编号让后续读者能逐条核对"
  - "[Phase ?] 前缀一条判为顺延而非关闭：它属 gsd-tools 侧行为，本轮范围是本仓库的代码与配置——写一份 plan 去改它只会产出一份必然落空的计划"
  - "deferred-items.md 顶部说明改用「同名小节」指代而非复述小节标题：plan 的 Task 2 自动化判据是 grep -c '去向（Phase 1 收尾定案）' == 2，顶部复述标题会让它读到 3"

metrics:
  duration: ~20min
  completed: 2026-07-29
---

# Phase 1 Plan 25: 需求追溯与顺延清单记账收口 Summary

把 INFRA-03 由 Phase 1 改映射至 Phase 4 并加表下注（Phase 1 已完成的钥匙串 / 唯一出口入口 / base_url 值侧校验三部分就地记录），同时给 `deferred-items.md` 悬了整个 phase 的两条候选各自定案。

## What Was Built

### Task 1 — INFRA-03 改映射到 Phase 4 并加表下注

**Commit:** `c8b25c2`

`01-VERIFICATION.md` 点出的记账口子是：`REQUIREMENTS.md` 的 Traceability 表仍把 INFRA-03 映射到 Phase 1，而 Phase 1 即将收尾——GSD 的「每条需求映射且仅映射一个 Phase」规则会让它在 Phase 1 关闭的那一刻变成一条无人推进的孤儿需求。

三处改动：

1. **INFRA-03 行的 Phase 列 `Phase 1` → `Phase 4`**，Status 保持 `Pending`（它在 Phase 4 才验收）。该行从 Phase 1 分组移动到 Phase 4 分组末尾（DIGEST-08 之后），保住表的分组形态。
2. **表下新增一条 INFRA-03 的注**，形态照抄第 208 行 INFRA-04/05 的既有先例，含四个要点：
   - Phase 1 已完成并通过 `/gsd-verify-work` 验证的三部分：钥匙串存取（keyring-core + apple-native-keyring-store）、prism-llm 为唯一网络出口与唯一密钥入口（`scripts/check-deps.sh` 的 single-egress / facade-egress / shell-egress 三条断言）、自定义 base_url 的存储与值侧校验（`crates/prism-store/src/settings.rs::validate_base_url` 的 scheme / host / userinfo / query / fragment 五项）；
   - 认领本需求的**三份** Phase 1 gap-closure plan：`01-14`（`scripts/check-secrets.sh` 的静态检查面）、`01-23`（密钥输入路径归一化）、`01-25`（本次改映射本身——它改的是映射记录，因此认领它而非实现它）；
   - 剩余分句「支持 Anthropic/OpenAI 兼容端点」（**实际发请求**）归 Phase 4，依据 ROADMAP Phase 4 goal 的「A3：prism-llm 传输层（流式/重试/keyring）先行交付」；
   - 本次改映射的目的即守住一需求一 Phase 规则。
3. **`Last updated` 行**更新为 2026-07-29，注明改动原因与 Coverage 不变。

### Task 2 — `deferred-items.md` 两条候选各自定案

**Commit:** `e8c85f9`

**第一条（rustfmt）的去向节全文：**

> #### 去向（Phase 1 收尾定案）
>
> **已排进本轮 gap-closure 的 `01-28-PLAN.md`**（wave 6，本轮最后一份），形态是一个 `checkpoint:decision`（该 plan 的 Task 1，也是本轮唯一的 `checkpoint:decision`）：
>
> - **选项 A**：补一个 `cargo fmt --all -- --check` 的 CI 步骤（engine job 最前）并一次性格式化全仓，同时创建 `rustfmt.toml`（空文件即代表显式采用 rustfmt 默认风格）与 `justfile` 的 `fmt-check` recipe。
> - **选项 B**：明确记为「本项目不采用 rustfmt 默认风格」，不加闸门；决定记录（含日期与理由）写在 `justfile` 文件头，仓库里不放 `rustfmt.toml`。
>
> **可逆性是 `costly` 而非 one-way**：两个方向都只是一次全仓 diff，没有已发布契约被破坏、不需要数据迁移；但选项 A 会产生一个触及每一个 Rust 文件的提交，且此后每个 phase 都受它约束。
>
> **为什么必须现在拍板**：逐 plan 各自格式化会造成风格拉锯，而每多一个 phase 就多一批文件——成本随时间单调上升。选项 A 另有一个已知风险需在执行时看住：本仓库多条 `include_str!` 源码序断言靠字符串片段定位（`open.rs` / `lib.rs` / `commands.rs` / `services.rs` / `middleware.rs`），rustfmt 的换行偏好可能把某个锚点拆到两行，使断言在实现完全正确时变红；届时修的是**锚点**，不是断言本身。
>
> `01-28-PLAN.md` 的 Task 2 会把最终结论回填到本小节。本小节只做指向，不做决策。
> （记账澄清：rustfmt 决策不在 01-27——01-27 只做 CI 闸门接线，其 objective 亦写明二选一定案在 01-28。）

**第二条（`[Phase ?]` 前缀）**记为**继续顺延**，理由是它属 gsd-tools 侧行为、**非本仓库代码**——本轮「全部清干净」的范围是本仓库的代码与配置，改不了的东西不能靠本轮的计划关闭。给出两个不互斥的去处：milestone 收尾时统一整理 `.planning/STATE.md` 的决策前缀；或向 gsd-tools 上游反馈。并写明其实际影响是**记账可读性而非正确性**（决策内容完整，只有 Phase 编号是占位符）。

**文件顶部**补上：两条候选已于 Phase 1 收尾的 gap-closure 轮次（plan 01-25）逐条定案，本文件此后只承接**新**发现的范围外事项。

## Verification Evidence

### Task 1 验收判据逐条

| 判据 | 读数 |
|---|---|
| INFRA-03 行 Phase 列为 `Phase 4` | `170:\| INFRA-03 \| Phase 4 \| Pending \|` ✓ |
| 表里 INFRA-03 只出现一次 | `grep -c '^\| INFRA-03 '` → `1` ✓ |
| INFRA-01/02 仍 Phase 1，06 仍 Phase 7，04/05/07/08 仍 Phase 8 | 146/147 Phase 1；202 Phase 7；203-206 Phase 8 ✓ 未连带改动 |
| **表总行数改动前 = 改动后** | `grep -c '^\| [A-Z]'`：改动前 **65** → 改动后 **65** ✓ |
| Coverage 三行不变 | `61 total / Mapped 61 / Unmapped 0 ✓` ✓ |
| plan 的 `<verify>` 命令 | `grep -c '^\| INFRA-0[1-8]'` → `8`（八条 INFRA 全在表内） |

**`git diff --stat .planning/REQUIREMENTS.md`：** `1 file changed, 9 insertions(+), 2 deletions(-)`

逐行 diff 只有三处（已核对，无第四处）：

```
-| INFRA-03 | Phase 1 | Pending |          ← 移出 Phase 1 分组
+| INFRA-03 | Phase 4 | Pending |          ← 移入 Phase 4 分组
+注：INFRA-03 为跨切需求…（含四点编号说明，共 6 行）
-*Last updated: 2026-07-28 after roadmap creation…*
+*Last updated: 2026-07-29 — Phase 1 收尾（plan 01-25）…*
```

2 条删除 = 移动的 INFRA-03 行 + 被替换的 `Last updated` 行。**没有任何其他条目被触及。**

### Task 2 验收判据逐条

| 判据 | 读数 |
|---|---|
| `grep -c '去向（Phase 1 收尾定案）'` | `2` ✓（两条候选各一节） |
| `grep -c '01-28-PLAN.md'` | `2`（≥1 ✓） |
| 第一条去向节含 `01-28-PLAN.md` / `checkpoint:decision` / 选项 A / 选项 B | 上文全文抄录，四者齐 ✓ |
| 该节里作为 rustfmt 决策所在地被点名的 plan 编号只有 01-28 | ✓ —— 01-27 出现一次，但是作为**否定**（「rustfmt 决策不在 01-27」），非决策所在地 |
| 第二条去向节含「属 gsd-tools 侧行为、非本仓库代码」与具体去处 | ✓ 两者齐（milestone 收尾整理 / 上游反馈） |
| **原有登记内容零删除** | `git diff --stat`：`1 file changed, 38 insertions(+)`；删除行计数 **0** ✓ |
| 顶部补上「此后只承接新发现的范围外事项」 | ✓ |

### 回归（纯记账 plan 不该影响任何测试）

```
cargo test --workspace   → 全部 test result: ok；合计 134 passed, 0 failed, 1 ignored
npm run test -- --run    → Test Files 7 passed (7) · Tests 75 passed (75)
```

两者与本 plan 执行前一致。本 plan 未触碰任何 `crates/` `src/` `src-tauri/` `scripts/` `.github/` 下的文件——`git diff` 涉及的两个文件都在 `.planning/` 下。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] 顶部说明段复述小节标题导致 Task 2 的自动化判据读到 3 而非 2**

- **Found during:** Task 2
- **Issue:** Task 2 的两条要求在字面上冲突：一条要求顶部说明段落存在（其自然写法会引用小节名），另一条要求 `grep -c '去向（Phase 1 收尾定案）'` 输出恰为 `2`。首版顶部写成「去向见各条下方的『去向（Phase 1 收尾定案）』小节」，grep 读到 `3`。
- **Fix:** 顶部改用「去向见各条下方的**同名小节**」指代，不复述标题。语义不变（读者仍能定位），判据回到 `2`。
- **Files modified:** `.planning/phases/01-foundation-skeleton/deferred-items.md`
- **Commit:** `e8c85f9`（在同一次提交内修正，未产生中间提交）

### 记账口径的两点说明（非偏离，供后续读者核对）

- **INFRA-03 行选择「移动」而非「原地改值」**：plan 明确给了二选一（「留在原位」或「移到 Phase 4 的其余条目附近」）。选移动是因为表按 Phase 分组，留在 Phase 1 段里写 `Phase 4` 会让分组形态失真。移动的两次 Edit 各自锚定了上下相邻行，diff 已逐行核对未连带删改。
- **注中点名 01-27 是刻意的**：Task 2 的判据只正向断言 `01-28` 被点名，并明确允许「补一句『此前误记为 01-27』」。本 plan 的 `read_first` 特别提醒「**不要**写成 01-27」——把这条澄清写进文件本身，让后续读者不必再去翻 plan 才知道两者的分工（01-27 只做 CI 闸门接线，01-28 才是决策所在地）。

## Known Stubs

无。本 plan 不产生任何代码，两个改动文件都是记账文档，内容完整无占位。

## Threat Flags

无。本 plan 未引入任何网络端点、鉴权路径、文件访问模式或信任边界处的 schema 变更——两个改动文件都在 `.planning/` 下，不进入产品运行时。

计划 `<threat_model>` 的三条 mitigate 已各自落实：

| Threat ID | 落实证据 |
|---|---|
| T-01G-41（INFRA-03 成孤儿） | Task 1：改映射 Phase 4 + 表下注；一需求一 Phase 未被破坏 |
| T-01G-42（顺延项无限期悬置） | Task 2：一条排进 01-28 的决策 checkpoint，一条写明顺延理由与两个去处 |
| T-01G-43（`Write` 覆盖丢表外条目） | 全程用 `Edit` 局部替换；表总行数 65 → 65，diff 仅 9 增 2 删 |

## Self-Check: PASSED

**文件存在：**

```
FOUND: .planning/REQUIREMENTS.md
FOUND: .planning/phases/01-foundation-skeleton/deferred-items.md
FOUND: .planning/phases/01-foundation-skeleton/01-25-SUMMARY.md
```

**提交存在：**

```
FOUND: c8b25c2  docs(01-25): remap INFRA-03 to Phase 4 with cross-cutting note
FOUND: e8c85f9  docs(01-25): resolve both deferred-items candidates
```

## 给后续 phase 的三条

1. **Phase 4 规划时**：INFRA-03 已在你的需求集里，但**只剩一半要做**——钥匙串、唯一出口入口、base_url 值侧校验三部分已在 Phase 1 完成并验证，表下注写死了它们的证据落点。要做的只有「实际向 Anthropic/OpenAI 兼容端点发出请求」的 chat client。另注意 STATE.md 里一条 flagged assumption：`keyring_core::set_default_store` 是进程全局，并发写同一 keychain account 无任何断言（`secrets.rs` 全部测试 `#[serial]` 是回避而非回答），接真实 chat client 时需补测试。
2. **01-28 执行时**：`deferred-items.md` 里 rustfmt 那条的「去向（Phase 1 收尾定案）」小节已建好，最后一句明写「Task 2 会把最终结论回填到本小节」——回填时替换那句指向语，保留选项 A/B 的原文作为决策记录。
3. **`deferred-items.md` 的新契约**：顶部已声明此后只承接**新**发现的范围外事项。01-18 登记的两条（`prism-engine/src/lib.rs` 的 `filter_map` 版本串、`3.x.51` 反例）本 plan 范围外未动——它们是本文件里现存的、尚无「去向」小节的条目，下一轮评审需给它们同样的待遇。
