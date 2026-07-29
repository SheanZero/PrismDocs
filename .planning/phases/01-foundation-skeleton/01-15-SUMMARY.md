---
phase: 01-foundation-skeleton
plan: 15
subsystem: dependency-assertions
tags: [dependency-assertions, shell, evidence-chain, gap-closure]
status: complete

requires:
  - "scripts/check-deps.sh（01-05 / 01-07 / 01-09 累积建立的七条断言）"
  - "01-REVIEW.md WR-04（verifier 用 PATH 上的 cargo 桩做过的端到端复现）"
  - "01-REVIEW.md IN-04 第一条（prism-engine-core / prism-mcp2 两个逃逸形态）"
provides:
  - "区分「命令没跑起来」与「跑起来了没发现问题」的 check_dup"
  - "三处按 `<crate-name> ` 尾随空格锚定的 crate 名匹配"
  - "check-secrets.sh 里那条把本缺陷当在世反面教材的注释，已同步为「已关闭」"
affects:
  - ".github/workflows/ci.yml:31-32 与 justfile:33-35 两个调用点（调用方式与输出文案均不变）"
  - "将来任何 facade 层新 crate：不再因名字前缀相同被 no-cycle 顺带看住，必须显式加进去"

tech-stack:
  added: []
  patterns:
    - "`|| true` 没有「一律可用」或「一律禁用」的规则——必须逐处判断被吞掉的是**干净**还是**失败**；两者在这一行内形态完全相同"
    - "包名匹配必须锚定到右边界（`--prefix none` 下就是那个尾随空格）；裸前缀在两个方向上都错：过敏（吸收同前缀 crate）或截断（真违规折叠成合法邻居）"

key-files:
  created: []
  modified:
    - scripts/check-deps.sh
    - scripts/check-secrets.sh

decisions:
  - "check_dup 的 FAIL 消息拆两行：第一行给退出码，第二行明说「这不是发现了重复，是断言本身没跑起来」——两件事在退出码上不可区分，只能靠文案分"
  - "no-cycle 侧的裸前缀方向与计划散文描述相反（实测是过敏而非逃逸），仍按 frontmatter truth 做尾随空格锚定，但源码注释写实测方向而非计划散文"
  - "同时写明覆盖代价：将来的 prism-engine-core 必须显式加进 check_no_cycle，不依赖前缀巧合"

metrics:
  duration: ~20min
  tasks: 2
  files: 2
  completed: 2026-07-29
---

# Phase 01 Plan 15: check-deps 断言判别力 Summary

把 `scripts/check-deps.sh` 的 `check_dup` 从「吞掉 `cargo tree` 的失败后在空串上找不到东西即报 OK」
改成先接住退出码再判输出，并把三处裸前缀的 crate 名匹配锚定到包名右边界——
成功标准 1-b / 1-c 与 NFR-03 的自动化证据，不再在它什么都没学到的时候报告成功。

## 关闭的东西

| 来源 | 编号 | 结论 |
|---|---|---|
| `01-REVIEW.md` | WR-04（≡ 上轮 WR-11） | 关闭（PATH 桩下从 `OK exit=0` 变为 `FAIL exit=1`） |
| `01-REVIEW.md` | IN-04 第一条 | 关闭（三处锚定 + 两组表达式对照反证） |

## 改了什么

### Task 1 — `check_dup` 区分「跑不起来」与「没有重复」（`4737e7e`）

```bash
check_dup() {
  local out rc=0
  # 命令替换失败时不让 set -e 直接掐断函数——把退出码记进 rc 自己判。
  out=$(cargo tree --workspace --duplicates --edges normal) || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "FAIL: \`cargo tree --workspace --duplicates\` could not run (exit $rc)" >&2
    echo "      —— 这不是「发现了重复依赖」，是这条断言本身没跑起来，未提供任何证据。" >&2
    return 1
  fi
  …
```

函数上方补了 `|| true` 的逐处判断理由：`check-secrets.sh` 的 `scan` 里那个是**承重**的
（`git grep` 无命中时退出 1，吞掉它才能把「干净」与「失败」分开），而这里被吞掉的恰好是失败本身。
两者形态相同、性质相反，所以不存在「`|| true` 一律可用/一律禁用」的规则。

`all` 模式下 `check_dup` 仍是第一条，`set -e` 让它的非 0 返回直接终止整个运行——没有加任何兜底。

顺带同步了 `scripts/check-secrets.sh:127-130` 的注释（见 Deviations）：它把本缺陷当作
「在世的承重 `|| true` 反面教材」写在源码里，缺陷关闭后那段引用不再成立。

### Task 2 — 三处 crate 名匹配锚定到包名右边界（`658b683`）

| 位置 | 改动前 | 改动后 |
|---|---|---|
| `check_no_cycle` | `grep -q '^prism-engine'` | `grep -q '^prism-engine '` |
| `check_facade_egress` offenders | `grep -oE '^prism-[a-z]+'` | `grep -oE '^prism-[a-z0-9-]+ ' \| sed 's/ $//'` |
| `check_shell_egress` offenders | 同上 | 同上 |

去尾随空格必须在 allowlist 过滤**之前**——否则 `prism-llm `（带空格）与 `^prism-llm$` 不等，
合法者会被当成违规。这一条写进了注释，也在下面的实跑里验过。

三处各补了注释说明尾随空格就是 `--prefix none` 输出里包名的右边界。

## 两组非恒真反证（实跑输出，非转述）

### 反证 A — PATH 上放一个恒失败的 `cargo` 桩（WR-04 主体）

桩内容：打一句话到 stderr 后 `exit 1`。

```
--- 改动前 ---
$ PATH=$stub:$PATH bash scripts/check-deps.sh dup
error: failed to parse manifest (stub)
OK: no duplicate rusqlite/reqwest/libsqlite3-sys
exit=0

$ PATH=$stub:$PATH bash scripts/check-deps.sh all
error: failed to parse manifest (stub)
OK: no duplicate rusqlite/reqwest/libsqlite3-sys      <-- check_dup 报绿
error: failed to parse manifest (stub)
exit=1                                                <-- 非 0 来自 check_tauri_free，与 dup 无关

--- 改动后 ---
$ PATH=$stub:$PATH bash scripts/check-deps.sh dup
error: failed to parse manifest (stub)
FAIL: `cargo tree --workspace --duplicates` could not run (exit 1)
      —— 这不是「发现了重复依赖」，是这条断言本身没跑起来，未提供任何证据。
exit=1

$ PATH=$stub:$PATH bash scripts/check-deps.sh all
error: failed to parse manifest (stub)
FAIL: `cargo tree --workspace --duplicates` could not run (exit 1)
      —— 这不是「发现了重复依赖」，是这条断言本身没跑起来，未提供任何证据。
exit=1                                                <-- 第一条即中止，OK 行消失

--- 移走桩（真实 cargo）---
$ bash scripts/check-deps.sh dup
OK: no duplicate rusqlite/reqwest/libsqlite3-sys
exit=0
```

`all` 那一对尤其要看：改动前它也是 exit=1，但那个 1 来自**下一条**断言，
`check_dup` 自己照常印了 OK——只看整体退出码会把这个 fail-open 完全看漏。

### 反证 B — 两组 grep 表达式在同一输入上的对照

输入是 `cargo tree -p prism-mcp --edges normal --prefix none | tail -n +2` 的真实输出，末尾手工追加一行：

```
追加行=[prism-engine v0.1.0]        改动前(裸前缀)=HIT   改动后(尾随空格)=HIT
追加行=[prism-engine-core v0.1.0]   改动前(裸前缀)=HIT   改动后(尾随空格)=MISS
```

第一行说明**没有放宽**：真正的 `prism-engine` 照样命中。第二行说明锚定生效：两次结果不同。

offenders 抽取，输入含一行 `prism-mcp2 v0.1.0`：

```
--- 输入 ---
reqwest v0.13.4
prism-mcp2 v0.1.0
prism-llm v0.1.0
prism-engine v0.1.0

--- 改动前 grep -oE '^prism-[a-z]+' | sort -u | grep -v allowlist ---
prism-mcp

--- 改动后 grep -oE '^prism-[a-z0-9-]+ ' | sed 's/ $//' | sort -u | grep -v allowlist ---
prism-mcp2
```

两次都会报违规（都不在 allowlist 里），但改动前**报出来的名字指向一个无辜的 crate**：
真正的违规者 `prism-mcp2` 被折叠成了合法邻居 `prism-mcp` 的名字。

### 活路径确认（offenders 那两行不是死代码）

三个 egress 包在 `prism-engine` 树内都存在，所以 `continue` 不生效、offenders 行每次都执行：

```
### reqwest 在 prism-engine 树内 —— offenders 行会被执行
  反向闭包里的 prism-* 原始抽取: prism-engine prism-llm
  过 allowlist 后 offenders: []
### keyring-core 在 prism-engine 树内 —— offenders 行会被执行
  反向闭包里的 prism-* 原始抽取: prism-engine prism-llm
  过 allowlist 后 offenders: []
### apple-native-keyring-store 在 prism-engine 树内 —— offenders 行会被执行
  反向闭包里的 prism-* 原始抽取: prism-engine prism-llm
  过 allowlist 后 offenders: []
```

`prism-llm` / `prism-engine` 去掉尾随空格后被 allowlist 正确过滤 → offenders 为空 → OK。
这同时验证了「去空格必须先于 allowlist 过滤」那条注释。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - 计划散文与实测方向相反] `check_no_cycle` 侧裸前缀是「过敏」而不是「逃逸」**

- **Found during:** Task 2，动手改之前先跑反证 B
- **Issue:** 计划 action 与 01-REVIEW.md IN-04 都写「裸前缀会让 `prism-engine-core`
  **满足** no-cycle 检查（它真的是个环）」。但 `check_no_cycle` 的逻辑是 `grep -q` **命中即 FAIL**，
  所以裸前缀 `^prism-engine` 匹配上 `prism-engine-core` 的后果是**误报一个环**，
  而不是放过一个环。实测（反证 B 第二行）：改动前 = HIT（FAIL），改动后 = MISS（OK）——
  方向与计划散文正好相反。若把计划的散文原样抄进源码注释，就是往代码库里写一句假话，
  而这恰是本轮 gap-closure 在治的那类问题。
- **Fix:** 机械改动照 frontmatter truth 做（三处一律尾随空格锚定，口径与同文件
  第 51/78/103/138/180 行对齐，且消除了 `prism-engineering` 这类误报）；
  但源码注释写**实测方向**：本处是过敏、下面两处是截断、根因同一个（包名未锚定到边界）。
  同时把覆盖代价写明：将来若真拆出 facade 层新 crate（如 `prism-engine-core`），
  它不会再因前缀相同被这一条顺带看住，**必须显式加进 `check_no_cycle`**，不依赖前缀巧合。
- **Files modified:** `scripts/check-deps.sh`
- **Commit:** `658b683`

**2. [Rule 2 - 证据链准确性] 同步 `check-secrets.sh` 里指向本缺陷的注释**

- **Found during:** Task 1（计划 read_first 已预告「改完后该段引用仍需成立或同步更新」）
- **Issue:** `scripts/check-secrets.sh:127-130` 把本缺陷（记作 WR-11）当作**在世的**
  「承重 `|| true` 反面教材」写在源码里。缺陷关闭后，这段注释在陈述一个已不存在的事实。
- **Fix:** 改为过去时并标注关闭位置：「…记作 WR-11 / WR-04，已在 01-15 改为先接住退出码再判输出」。
  对照的教学价值（形态相同、性质相反）保留，未删。
- **Files modified:** `scripts/check-secrets.sh`（超出计划 frontmatter 的 `files_modified`，仅注释）
- **Commit:** `4737e7e`

### 未做的放宽

- 四个受检集合变量逐字未变（`git show HEAD~2:scripts/check-deps.sh` 与现文件 diff 为空）：

  ```
  ENGINE_CRATES="prism-types prism-store prism-fs prism-parse prism-anchor prism-llm prism-mcp prism-engine"
  TAURI_FREE_CRATES="$ENGINE_CRATES prism-cli"
  PURE_CRATES="prism-types prism-store prism-fs prism-parse prism-anchor"
  EGRESS_CRATES='reqwest|keyring-core|apple-native-keyring-store'
  ```

- 七条断言的 OK 输出文本与改动前逐字相同
- 子命令一个不增不减：`usage: scripts/check-deps.sh [dup|tauri-free|no-cycle|single-egress|facade-egress|shell-egress|subscriber-free|all]`
- 零新增文件、零新增依赖、零新增 allowlist；未给 `check_dup` 加任何兜底

## Verification

| 命令 | 结果 |
|---|---|
| `bash scripts/check-deps.sh all` | 七条全 OK，exit 0 |
| `bash scripts/check-deps.sh dup` | `OK: no duplicate rusqlite/reqwest/libsqlite3-sys`，exit 0 |
| `PATH=$stub:$PATH bash scripts/check-deps.sh dup` | 新 FAIL 消息（含 exit 1），exit 1 |
| `PATH=$stub:$PATH bash scripts/check-deps.sh all` | 第一条即中止，exit 1 |
| `bash scripts/check-secrets.sh all` | selftest 19/10 OK + scan 114 文件 OK，exit 0 |
| `bash scripts/check-deps.sh bogus` | usage 原文，exit 2（子命令集未变） |
| 四个受检集合变量 diff | 空 |

## Known Stubs

无。本 plan 未引入桩、未跳过测试、未留下未跑的 `<verify>`。
反证用的 `cargo` 桩是临时目录里的一次性文件，不在仓库内、不进版本控制。

## Threat Flags

无新增安全面。本 plan 只改两条断言脚本，不触及产品代码。
计划 threat_model 的 T-01G-06 / T-01G-07 / T-01G-08 三条 mitigate 全部落实并各配了实跑反证。

## Self-Check: PASSED

- 提交存在：`4737e7e` FOUND、`658b683` FOUND
- 文件存在：`scripts/check-deps.sh` FOUND、`scripts/check-secrets.sh` FOUND
- 两次提交均无文件删除（`git diff --diff-filter=D HEAD~2 HEAD` 为空）
- 工作树干净（除 `.planning/` 文档与既有的两个未跟踪 research cache 文件）
