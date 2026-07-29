---
phase: 01-foundation-skeleton
plan: 14
subsystem: security-tooling
tags: [security, secret-scanning, shell, regex, evidence-chain, gap-closure]
status: complete

requires:
  - "scripts/check-secrets.sh（01-11 建立的 selftest + scan 双子命令结构）"
  - "01-VERIFICATION.md § SC-4 的 8 行取样表（本 plan 的命中率标尺）"
  - "01-REVIEW.md CR-01 / WR-09（blocker 与 warning 的原始记录）"
provides:
  - "看得见未加引号赋值的 check-secrets.sh 关键词分支（引号串{8,} 或 裸值{16,}）"
  - "github_pat_ / xox[baprs]- / AIza 三条供应商前缀 alternation"
  - "只能经关键词分支命中的两条阳性样本 + 裸值下界的三条边界阴性样本"
  - "scan 的 cwd 固定与扫描面下限断言；OK 消息新增文件计数"
affects:
  - ".github/workflows/ci.yml:36-37 与 justfile:25-26 两个调用点（行为不变，输出文案变）"
  - "将来任何 fixture / 局部变量：关键词后跟 ≥16 字符裸值也会被抓"

tech-stack:
  added: []
  patterns:
    - "检测控件的每一次扩宽都必须配一条只能经新分支命中的样本——否则旧分支会替新分支兜底，selftest 变成有误导性而非仅仅不完整"
    - "防线（cd 到仓库根）与报警器（扫描面下限）成对存在：防线失效时报警器让失效表现为红"

key-files:
  created: []
  modified:
    - scripts/check-secrets.sh
    - crates/prism-store/src/settings.rs

decisions:
  - "裸值段下界取 16、严格高于引号段的 8：引号串本身是「有人刻意写了字面量」的强信号，裸值没有这个信号"
  - "SC-4 取样表第 5 行（值仅 14 字符）作为已知残留写进源码注释，不下调阈值、不编成阴性断言"
  - "IN_QUERY fixture 的重命名从 Task 3 提前到 Task 1：Task 1 的 verify gate 在它之前无法变绿"

metrics:
  duration: ~25min
  tasks: 3
  files: 2
  completed: 2026-07-29
---

# Phase 01 Plan 14: check-secrets 裸值可见性与作用域固定 Summary

把 `scripts/check-secrets.sh` 的关键词分支从「值必须带引号」改成「引号串或裸值两者取一」，
补三条供应商前缀，并把 scan 的工作目录钉在仓库根 + 加扫描面下限断言——
成功标准 4 里「**配置**中无明文密钥」那半句的执行机制，从此看得见 `.env` / YAML / CI `env:` 形态。

## 关闭的东西

| 来源 | 编号 | 结论 |
|---|---|---|
| `01-VERIFICATION.md` frontmatter `gaps[0]` | blocker | 关闭（8 行取样表 7/8 命中，第 5 行为已记录残留，见下） |
| `01-REVIEW.md` | CR-01 | 关闭（裸值可见 + 三条新前缀 + 隔离样本 + 非恒真反证） |
| `01-REVIEW.md` | WR-09 | 关闭（cwd 固定 + 下限断言，收窄从绿变红） |

## 改了什么

### Task 1 — PATTERN 关键词分支让引号可选，补三条前缀（`6bec3cb`）

新增裸值字符类片段变量：

```bash
BARE="[A-Za-z0-9_./+~-]"
```

`PATTERN` 从四段 alternation 扩到七段，关键词分支的值部分改为两者取一：

```
…|github_pat_[A-Za-z0-9_]{20,}|xox[baprs]-[A-Za-z0-9-]{16,}|AIza[A-Za-z0-9_-]{30,}|(api[_-]?key|secret|token|password)[[:space:]]*[=:][[:space:]]*(${QUOTE}${NOT_QUOTE}{8,}|${BARE}{16,})
```

两个长度下界刻意不同、裸值那个更高（8 vs 16）。理由写进 `PATTERN` 上方的编号注释第 7 条：
引号串本身已经是一个强信号（有人刻意写了一个字面量），裸值没有这个信号，
`token = someVar` / `secret: cfg.value` 这类普通标识符赋值在低下界上会大面积误报，
而被误报烦到的人会把闸门绕开——绕开的闸门等于没有闸门。

`$PATTERN` 仍是单一定义（去掉注释行后 `grep -c '^PATTERN='` 输出 `1`）。

### Task 2 — selftest 补隔离关键词分支的样本（`56a58b8`）

阳性组新增一节（值里不含任何供应商前缀，因此只可能经关键词分支的裸值那一半命中）：

```bash
positive+=("MCP_BEARER_${tok}=7f3a9c1e5b2d8f4a6c0e9b7d3f1a5c8e")
positive+=("${pw}${word}: abcdefghijklmnop")
```

第一条是本项目自己的第二个密钥（`docs/keychain-naming.md` 的 account 名 `mcp_bearer_token`）
在配置文件里的形态；第二条恰好压在裸值下界 16 上。两条都按文件头的自扫约束由片段拼出。

阴性组新增三条把下界钉成边界：15 字符（比下界少一个）、`let token = someVar;`、`secret: cfg.value`。

样本数 14 阳性 / 7 阴性 → **19 阳性 / 10 阴性**。

文件头补了「复现非恒真反证」的四步照做说明（不是结论）。

### Task 3 — scan 固定 cwd + 扫描面下限（`58d1872`）

```bash
cd "$(git rev-parse --show-toplevel)"          # set -euo pipefail 之后第一件事
```

```bash
local floor=40
files=$(git ls-files -- ':(exclude).planning/' | wc -l | tr -d '[:space:]')
if [ "$files" -lt "$floor" ]; then
  echo "FAIL: scan surface implausibly small ($files < $floor version-controlled files) …" >&2
  return 1
fi
```

OK 消息带上计数：`OK: no plaintext secret in 114 version-controlled files`。
「扫了多少」从此是每次运行都看得见的读数。

## 四条非恒真反证（实跑输出，非转述）

### 反证 A — 把关键词分支的值部分改回「引号必需」

```
$ perl -pi -e 's/\(\$\{QUOTE\}\$\{NOT_QUOTE\}\{8,\}\|\$\{BARE\}\{16,\}\)/\${QUOTE}\${NOT_QUOTE}{8,}/ if /^PATTERN=/' scripts/check-secrets.sh
$ bash scripts/check-secrets.sh selftest
FAIL: positive sample not detected: MCP_BEARER_TOKEN=7f3a9c1e5b2d8f4a6c0e9b7d3f1a5c8e
FAIL: positive sample not detected: password: abcdefghijklmnop
CP-A exit=1
--- 还原后（cp 自备份，非 git checkout）---
OK: pattern discriminates (19 positive / 10 negative samples)
restore exit=0
```

失败行**恰好**是本 plan 新增的那两条；01-VERIFICATION.md § SC-4 对应的既有 5 条阳性样本
（`sk-ant-api03-…` 那一族）全部不在失败列表里——它们经 `sk-` 前缀分支命中，与本分支无关。
这正是 CR-01 指出的误导：旧样本读起来像在覆盖裸值赋值，实际不是。

> **过程记录（第一次反证做错了）**：首次用 `perl -0pi -e` 无行锚点做替换，
> 它命中的是**文件头注释里**那段同形字面量，`PATTERN=` 那一行一个字没动，
> selftest 照常 `exit=0`。若不看落点只看红绿，会把「反证没生效」误读成「反证不成立」。
> 加上 `if /^PATTERN=/` 行锚点后才是上面这次。这是 STATE.md 里「反证本身需要被验证」
> 那条 blocker 的第四次出现。

### 反证 B — 把裸值下界从 16 降到 8

```
$ perl -pi -e 's/\$\{BARE\}\{16,\}/\${BARE}{8,}/ if /^PATTERN=/' scripts/check-secrets.sh
$ bash scripts/check-secrets.sh selftest
FAIL: negative sample wrongly detected: token = abcdefghijklmno
FAIL: negative sample wrongly detected: secret: cfg.value
CP-B exit=1
--- 还原后 ---
OK: pattern discriminates (19 positive / 10 negative samples)
restore exit=0
```

新增的三条阴性样本里有两条变红。第三条 `let token = someVar;` 的值只有 7 个字符，
落在 8 之下所以在这个具体阈值上不响——它守的是更低的下界（≤7）。
计划 AC 预期的是「两条」，实测正是两条，只是其中一条是 `secret: cfg.value` 而非 `someVar` 那条。

### 反证 C — 删掉固定工作目录那一行（WR-09 主体）

```
$ perl -ni -e 'print unless /^cd "\$\(git rev-parse --show-toplevel\)"$/' scripts/check-secrets.sh
$ cd src && bash ../scripts/check-secrets.sh scan
FAIL: scan surface implausibly small (14 < 40 version-controlled files) — 作用域被收窄了
CP-C exit=1
--- 还原后从 src/ 再跑 ---
OK: no plaintext secret in 114 version-controlled files
restore exit=0
```

改动前这条路径是 `OK … exit=0`（verifier 实测记录在 WR-09）。现在收窄表现为红。

### 反证 D — 往 `.github/workflows/ci.yml` 追加两行裸值密钥（blocker 主体）

```
$ printf '  MCP_BEARER_TOKEN=9f2b7d1a4c6e8092b5d3f7a1c9e0b246\n  password: plaintextsecretvalue\n' >> .github/workflows/ci.yml
$ bash scripts/check-secrets.sh scan
FAIL: possible plaintext secret in version-controlled files
.github/workflows/ci.yml:104:  MCP_BEARER_TOKEN=9f2b7d1a4c6e8092b5d3f7a1c9e0b246
.github/workflows/ci.yml:105:  password: plaintextsecretvalue
CP-D exit=1
$ git checkout .github/workflows/ci.yml
--- 还原后 ---
OK: no plaintext secret in 114 version-controlled files
restore exit=0
```

同一形态在改动前是 `OK … exit=0`（01-VERIFICATION.md § SC-4 端到端复现）。blocker 关闭。

## SC-4 取样表逐行回放（Task 1 之后）

用 `source <(sed -n '/^QUOTE=/,/^PATTERN=/p' scripts/check-secrets.sh)` 取出真实 PATTERN 逐条喂：

| 取样行 | 结果 |
|---|---|
| `ANTHROPIC_API_KEY=abcdef0123456789abcdef0123456789` | HIT |
| `mcp_bearer_token: 7f3a9c1e5b2d8f4a6c0e9b7d3f1a5c8e` | HIT |
| `api_key=abcdef0123456789abcdef` | HIT |
| `export OPENAI_API_KEY=abcdefghijklmnopqrstuvwxyz012345` | HIT |
| `password = hunter2hunter2` | **MISS** —— 见下「已知残留」 |
| `token = 0123456789abcdef0123` | HIT |
| `secret: not-a-real-value-here` | HIT |
| `bearer_token=0123456789abcdefghij` | HIT |
| 误报对照 `let token = someVar;` | MISS（期望） |
| 误报对照 `secret: cfg.value` | MISS（期望） |
| `github_pat_11ABCDEFG0abcdefghijklmnopqrstuvwxyz1234567890` | HIT |
| `xoxb-123456789012-1234567890123-abcdefghijklmnopqrstuvwx` | HIT |
| `AIzaSyA-abcdefghijklmnopqrstuvwxyz12345` | HIT |

改动前这张表是 0/8（引号必需）；现在 7/8。

## 重跑 scan 后的新命中（逐条处理）

加宽后的正则在全仓库只产生两条新命中，是同一个字面量的两处副本：

| 命中 | 处理 |
|---|---|
| `crates/prism-store/src/settings.rs:246` `IN_QUERY = "…?api-key=prism-test-secret-value"` | 查询参数名 `api-key` → `deployment`，值原样保留；旁边加注释写明改名理由 |
| `scripts/check-secrets.sh` 阴性组里 `IN_QUERY` 的镜像 | 同步改名（两处必须一致，否则 selftest 与 scan 对同一字面量给出相反结论） |

改名未削弱它守的性质：`validate_base_url` 拒的是「query 非空」而不是「query 里有 api-key」，
换个参数名之后 fixture 仍精确落在 `url.query().is_some()` 分支上。
`cargo test -p prism-store --lib` **21 passed**，`settings_base_url_rejects_credential_bearing_values`
的断言语义一字未变。

`settings.rs:73` 的注释里还有一处 `?api-key=…`（解释守卫理由的散文），因结尾是省略号不构成命中，未动。

## 已知残留（不是遗漏）

`password = hunter2hunter2` 的值只有 **14 个字符**，落在裸值下界 16 之下，裸值形态不命中。
这是本改动核心权衡的直接代价，不是笔误：

- 加引号的同一条（`password = "hunter2hunter2"`）在 selftest 阳性组里，经引号段的 8 命中；
- 它是弱人类口令而非 API key / token，而本闸门守的是成功标准 4 的**密钥**面；
- 要接住它就得把裸值下界降到 ≤14，而取值为表达式的赋值（形如 `self.inner.value`）
  长度恰在 12–20 之间——下界一降它们整片涌进来。

已写进 `scripts/check-secrets.sh` 文件头 `PATTERN` 注释第 7 条，附完整算术与「不要顺手调下去」的理由。
**刻意没有**把它编成阴性断言：那会把一个已知缺口写成一条绿色的期望性质。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - 阻塞] IN_QUERY fixture 改名从 Task 3 提前到 Task 1**

- **Found during:** Task 1 首次跑 `bash scripts/check-secrets.sh selftest`
- **Issue:** Task 1 的 `<verify>` 是 selftest，而加宽后的正则立刻撞上阴性组里的 `IN_QUERY` 镜像
  （`FAIL: negative sample wrongly detected`）。计划把改名安排在 Task 3，
  于是 Task 1 与 Task 2 的 verify gate 在改名之前**结构性地无法变绿**。
- **Fix:** 按计划自述的单向约定（改 fixture 不改防线）把两处改名提前到 Task 1 一并提交。
  Task 3 因此只剩 cwd 固定与下限断言两件事。
- **Files modified:** `crates/prism-store/src/settings.rs`、`scripts/check-secrets.sh`
- **Commit:** `6bec3cb`

**2. [Rule 1 - 计划断言不成立] Task 1 AC 的第 5 行取样与计划自己规定的 {16,} 下界互斥**

- **Found during:** Task 1 acceptance 回放
- **Issue:** AC 要求 8 行取样全部命中，其中 `password = hunter2hunter2` 值长 14；
  而同一 Task 的 action 与 frontmatter backstop truth 都要求裸值下界严格高于引号下界并明确取 16。
  14 < 16，两条要求在算术上不可同时满足。
- **Fix:** 保留 {16,}（它是本改动的核心权衡且被 backstop truth 钉住），
  把这一行作为**已知残留**连同完整算术写进源码注释与本 SUMMARY，不下调阈值、不加 allowlist。
- **Commit:** `6bec3cb`

**3. [过程] 反证 A 首次执行无效（已在上文反证 A 记录）**

无行锚点的 `perl -0pi` 改到了注释而非 `PATTERN=` 行，selftest 照常绿。
加锚点重跑后才取得真实落点。记录在此以免读者把「反证没生效」当成「反证不成立」。

### 未做的放宽

- 排除集仍只有 `.planning/` 一条（`git grep` 与新增的 `git ls-files` 用同一条 pathspec）
- 零新增 allowlist、零新增整目录排除
- 既有长度阈值 `{20,}` `{20,}` `{16}` `{8,}` 一个未降；新增的是 `{20,}` `{16,}` `{30,}` `{16,}` 四条**下界**
- 未新增文件、未新增 crate、未新增依赖

## Verification

| 命令 | 结果 |
|---|---|
| `bash scripts/check-secrets.sh all` | `OK: pattern discriminates (19 positive / 10 negative samples)` + `OK: no plaintext secret in 114 version-controlled files`，exit 0 |
| `bash scripts/check-deps.sh all` | 七条断言全 OK，exit 0 |
| `cargo test -p prism-store --lib` | 21 passed / 0 failed |
| `cd src && bash ../scripts/check-secrets.sh scan` | `OK: no plaintext secret in 114 version-controlled files`，与仓库根**完全相同**的计数 |
| 自扫约束 | scan 对 `scripts/check-secrets.sh` 自身零命中（scan 整体 exit 0） |
| `grep -v '^[[:space:]]*#' … \| grep -c '^PATTERN='` | `1` |

## Known Stubs

无。本 plan 未引入桩、未跳过测试、未留下未跑的 `<verify>`。

## Threat Flags

无新增安全面。本 plan 只改一条检测控件与它的 selftest，不涉及产品代码路径；
唯一被动到的产品文件是一条测试 fixture 的常量值。

## Self-Check: PASSED

- 提交存在：`6bec3cb` FOUND、`56a58b8` FOUND、`58d1872` FOUND
- 文件存在：`scripts/check-secrets.sh` FOUND、`crates/prism-store/src/settings.rs` FOUND
- 工作树干净（除 `.planning/` 文档与既有的两个未跟踪 research cache 文件）
