---
phase: 01-foundation-skeleton
plan: 11
subsystem: infrastructure
tags: [secret-scanning, ci-gate, selftest, counterproof, gap-closure, detection-control, T-01-44, T-01-45, T-01-46, T-01-47, T-01-48, T-01-49]
status: complete

# Dependency graph
requires:
  - "01-02：`scripts/check-deps.sh` 的结构范式（`set -euo pipefail`、长文件头说明「为什么这条断言存在 + 本文件是唯一实现」、具名函数 + `main()` 分派 `${1:-all}` + `usage:` exit 2）与 justfile / CI 双调用点约定"
  - "01-04 / 01-09：`crates/prism-llm/src/secrets.rs` 的 `FIXTURE_SECRET` 与 `src/pages/Settings.test.tsx` 的 `FAKE_KEY` 两处 fixture 命名约定的原文——「扫描器是防线，不该为了让 fixture 通过而放宽它」"
  - "01-06：`crates/prism-mcp/src/{deps,middleware}.rs` 的两个测试（`debug_does_not_reveal_the_bearer_token` / `constant_time_eq_agrees_with_equality_on_every_shape`）与它们的断言集合"
  - "01-10：新增的四条凭据型 URL fixture（userinfo / query / fragment），本 plan 把它们钉进 selftest 的阴性样本组"
  - "01-VERIFICATION.md § SC-4 的 5 行真实形态取样表——本 plan 的验收清单就是那张表"
provides:
  - "`scripts/check-secrets.sh` 的四段 alternation PATTERN（`sk-[A-Za-z0-9_-]{20,}` / `ghp_` / `AKIA` / 关键词赋值），Anthropic 形态从失明变为命中"
  - "`selftest` 子命令：14 条阳性 / 7 条阴性样本，与 `scan` 共用同一个 `$PATTERN` 变量"
  - "`scan` 子命令：受检集合含脚本自身，排除 pathspec 只剩 `.planning/` 一条"
  - "`all` 子命令（无参数默认）：selftest 先跑、scan 后跑"
  - "`just check-secrets-selftest` —— 改正则时的快速回路（不碰 git）"
  - "CI 步骤 `Plaintext secret scan (pattern selftest + repo scan)` —— 显式跑 `all`，不依赖默认值"
affects: [phase-2-起每次新增-fixture, phase-4-LLM-chat-client, phase-6-MCP-bearer-配置, 全部后续-phase-的-CI-闸门]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "检测控件必须自证判别力：一个看不见目标格式的扫描器与一个干净的仓库，退出码完全相同。selftest 把「正则有判别力」从声明变成每次 CI 都重跑一遍的断言"
    - "selftest 与 scan 共用同一个 `$PATTERN` 变量（源码断言：`PATTERN=` 只出现一次）——两份正则可以静默漂移，那时测的是副本"
    - "扫描器不排除自身：所有阳性样本由片段拼出，`scan` 退出 0 即为「源码里没有命中自身正则的字面量」这条性质的可执行证明"
    - "fixture 撞上扫描器时改 fixture，不改防线。放宽正则 / 加 allowlist / 加整目录排除 / 降长度阈值——四者都算放宽"
    - "承重的 `|| true` 与吞掉失败的 `|| true` 在一行内不可区分，只能靠配套控件区分：`git grep` 无命中即退出 1（承重），而一次被吞掉的调用失败会让 selftest 的阳性样本当场不再命中"
    - "阈值类参数要有**成对**边界样本（20 命中 / 19 不命中），否则那个数字是没人验证过的"

key-files:
  created: []
  modified:
    - scripts/check-secrets.sh
    - crates/prism-mcp/src/deps.rs
    - crates/prism-mcp/src/middleware.rs
    - justfile
    - .github/workflows/ci.yml

key-decisions:
  - "01-11: `sk-` 段的字符类扩到 `[A-Za-z0-9_-]` 的同时长度下界必须从 16 提到 20——放宽字符类后 16 会把普通的连字符标识符串扫进来。两者是同一个改动的两半，只做前一半会用误报换漏报"
  - "01-11: 排除集收窄到只剩 `.planning/`。`docs/` 的整目录排除按实测取消（零命中），脚本自排除取消；将来确需引用正则的文档只排除那一个文件路径并在脚本里写明理由"
  - "01-11: 两处新命中以**改名**解决（`secret`→`fixture_bearer`、`token`→`configured`），一条 allowlist 都不新增——放宽不需要迁移，却会让此后每一次「加个 allowlist 就好了」看起来合理，而那正是本 gap 的成因"
  - "01-11: justfile 与 CI 都显式写 `all` 而不是靠无参数默认值——默认值哪天改回 scan-only，闸门会静默失去 selftest 那一半并照常绿"
  - "01-11: `all` 的顺序是 selftest 先、scan 后：正则失去判别力时，先红的应当是那个原因而不是它的后果"
  - "01-11: selftest 逐条喂样本一律用 herestring，不得用管道——`pipefail` + `grep -q` 早退的 SIGPIPE 会让断言静默恒绿（本 phase 在 check-deps.sh 上踩过一次）"

patterns-established:
  - "检测控件的 selftest 形态：阳性组 / 阴性组两组样本 + 共用同一个判别变量 + 逐条 herestring + 失败累积（`failed=1` 而非早退，让所有落点一次看清）"
  - "样本溯源写进文件头：阳性组前五条逐条对应 01-VERIFICATION.md § SC-4 的取样表，阴性组前五条是既有 fixture 的真实源码行——样本不是编的，是有出处的"
  - "阳性样本片段拼接：`local sk=\"sk\" dash=\"-\" ghp=\"ghp\" us=\"_\" aws=\"AKI\" q='\"'`，使脚本源码不含命中自身正则的连续字面量"

requirements-completed: [INFRA-01, INFRA-03]

coverage:
  - id: D1
    description: "widen 后的 PATTERN 对 01-VERIFICATION.md § SC-4 的 5 行取样全部命中（旧正则 1/5），对既有 fixture 全部不命中，阈值边界有成对样本"
    requirement: INFRA-03
    verification:
      - kind: unit
        ref: "bash scripts/check-secrets.sh selftest → exit 0，`OK: pattern discriminates (14 positive / 7 negative samples)`，耗时 0.039s"
        status: pass
      - kind: other
        ref: "反证 F（`sk-` 段改回 `sk-[A-Za-z0-9]{16,}`）→ exit 1，落点为三条 Anthropic 形态阳性样本 + 19 字符阴性样本"
        status: pass
      - kind: other
        ref: "反证 G（PATTERN 改成 `.+`）→ exit 1，落点为阴性组全部 7 条"
        status: pass
      - kind: other
        ref: "反证 H1（下界 `{21,}`）→ exit 1 落点为 20 字符阳性样本；反证 H2（下界 `{19,}`）→ exit 1 落点为 19 字符阴性样本"
        status: pass
      - kind: other
        ref: "源码断言：`grep -c '^PATTERN=' scripts/check-secrets.sh` = 1；`grep -n '|[[:space:]]*grep'` 零命中（无管道喂 grep）"
        status: pass
      - kind: other
        ref: "bash scripts/check-secrets.sh nonsense-subcommand → 打印 usage 到 stderr 并 exit 2"
        status: pass
    human_judgment: false
  - id: D2
    description: "widen 后全仓重新变绿：两处新命中按既有 fixture 约定改名解决，防线一寸未放宽，两个既有测试仍在仍绿且断言条数不变"
    requirement: INFRA-03
    verification:
      - kind: other
        ref: "Task 1 结束时 `scan` exit 1（两处命中）→ Task 2 改名后 exit 0。红→绿的转变本身即 widen 真的生效过的证据"
        status: pass
      - kind: unit
        ref: "cargo test -p prism-mcp → 10(lib) + 10(middleware_gate) + 3(trait_injection) passed；两个被改名的测试各自在列且绿"
        status: pass
      - kind: unit
        ref: "cargo test --workspace → exit 0，122 passed / 1 ignored（改名前为 121 passed，差额来自 01-10 新增用例，非本 plan）"
        status: pass
      - kind: other
        ref: "cargo clippy -p prism-mcp -- -D warnings → exit 0；cargo clippy -p prism-mcp --all-targets -- -D warnings → exit 0"
        status: pass
      - kind: other
        ref: "排除 pathspec 只剩 `.planning/` 一条（源码断言：`grep -n exclude` 单行命中）；`git grep` 受检集合含 scripts/check-secrets.sh 自身"
        status: pass
    human_judgment: false
  - id: D3
    description: "闸门在两个调用点都真的被更新，且不是恒真的"
    requirement: INFRA-01
    verification:
      - kind: other
        ref: "反证 I（往 `src/lib/ipc.ts` 追加 `sk-ant-api03-…` 形态假串）→ scan exit 1 并把该行打到 stderr；撤销后 exit 0"
        status: pass
      - kind: other
        ref: "反证 J（往 `scripts/check-secrets.sh` 自身追加同样的串）→ scan exit 1；撤销后 exit 0"
        status: pass
      - kind: other
        ref: "反证 K（往阳性组塞一条不可能命中的样本）→ CI 步骤命令与 justfile `check-all` 的 recipe 体均 exit 1；撤销后 exit 0"
        status: pass
      - kind: other
        ref: "行为断言：justfile 中三条 check-secrets 相关 recipe 全为单行委托，无断言逻辑副本"
        status: pass
      - kind: other
        ref: "bash scripts/check-deps.sh all → exit 0（本 plan 不动依赖图）"
        status: pass
    human_judgment: false

# Metrics
duration: 15min
completed: 2026-07-29
---

# Phase 01 Plan 11: 明文密钥扫描器的失明修复与自证 Summary

把 `scripts/check-secrets.sh` 从「看不见本项目一等端点密钥格式」的状态修好，并加上一个每次 CI 都重跑的 selftest——一次绿色的扫描从此真的意味着「受版本控制的文件里没有那几种明文密钥」，而这句话本身由 14 条阳性 / 7 条阴性样本重新证明一遍。

## 关闭的缺口

01-VERIFICATION.md gap 2（Blocker）。旧 PATTERN 要求 `sk-` 后紧跟 ≥16 个**连续**字母数字，而 Anthropic 的 `sk-ant-api03-…` 在第三个字符的连字符处就断开。verifier 用 5 行真实形态取样，只命中 1 行。

这不是覆盖率不足，是**证据失效**。成功标准 4 第三分句的主要自动化证据就是这个脚本退出 0；一个失明的正则与一个干净的仓库，退出码完全相同。而 CLAUDE.md 点名 Anthropic Messages API 是一等端点、`Settings.tsx:149` 的 placeholder 就写着 `https://api.anthropic.com`——最可能被泄漏的那个供应商的密钥，恰好是唯一看不见的。

本 plan 与 01-10 是同一条决策（D-05：非密钥配置存 settings 表，密钥走钥匙串、绝不入库）的两处失守：01-10 补的是**写入路径**，本 plan 补的是**静态扫描**。缺任一半，D-05 都只是注释。

## Accomplishments

### Task 1 — widen PATTERN、加 selftest、收窄排除集

`scripts/check-secrets.sh` 按 `scripts/check-deps.sh` 的结构重写（31 行 → 165 行）：长文件头说明这条断言为什么存在、本文件是唯一实现、justfile 与 CI 只是调用者；`set -euo pipefail`；`scan()` / `selftest()` 两个具名函数 + `main()` 分派 `${1:-all}`，未知参数打 `usage:` 并 `exit 2`。

**PATTERN 的四段 alternation**（唯一定义，两个子命令共用；源码断言 `PATTERN=` 只出现一次）：

| 段 | 修复了什么 |
|---|---|
| `sk-[A-Za-z0-9_-]{20,}` | 字符类加入连字符与下划线是本 gap 的核心修复；下界 16→20 是它的必要配套 |
| `ghp_[A-Za-z0-9]{20,}` | GitHub PAT 前缀（新增） |
| `AKIA[A-Z0-9]{16}` | AWS access key id 前缀（新增） |
| `(api[_-]?key\|secret\|token\|password)[[:space:]]*[=:][[:space:]]*${QUOTE}${NOT_QUOTE}{8,}` | 在旧的 `api_key=` 形态上扩了三个关键词与冒号赋值形态 |

`git grep` 加 `-i`，使 `API_KEY=` / `apiKey:` 两种大小写都进网（实测不引入任何额外误报）。

**排除集从三条收窄到一条**：

- 保留 `.planning/`——规划与验证文档按设计要引用取样密钥，01-VERIFICATION.md § SC-4 的 5 行取样表就是活证据。理由写进注释。
- 去掉 `docs/` 整目录排除——执行时实测 `git grep -niE "$PATTERN" -- 'docs/'` 零命中，整目录排除已无存在理由。
- 去掉脚本对自身的排除——脚本被自己扫这件事，是「源码里没有命中自己正则的字面量」这条性质的可执行证明。为此**所有阳性样本由片段拼出**（`local sk="sk" dash="-" ghp="ghp" us="_" aws="AKI" q='"'`），阴性样本反而整串直写（它们本来就不该命中，直写正是要断言的事）。

**`selftest()`**：14 条阳性 / 7 条阴性，逐条用 herestring 喂 `grep -qiE "$PATTERN"`，失败累积（`failed=1` 而非早退，让所有落点一次看清）。样本溯源写进文件头。

阳性组 14 条：01-VERIFICATION.md 取样 5 条 + 关键词赋值 6 种形态 + `ghp_` / `AKIA` 各 1 条 + `sk-` 恰好 20 字符 1 条。
阴性组 7 条：`FIXTURE_SECRET` / `FAKE_KEY` 两处既有 fixture 的**真实源码行** + 01-10 的三条凭据型 URL fixture + `sk-` 恰好 19 字符 + 一条 17 字符的（阈值从 16 提到 20 挡下的那类）。

### Task 2 — 全仓重新变绿 + 闸门接进两个调用点

widen 后全仓恰好两处新命中，都在 prism-mcp 测试里，都是「局部变量名叫 secret / token，右边跟着一个引号串」的形态——扫描器抓的就是这个形状，**它没抓错**。按 `FIXTURE_SECRET` / `FAKE_KEY` 已确立的约定处理：

| 文件 | 测试 | 改名 | 字面量 |
|---|---|---|---|
| `crates/prism-mcp/src/deps.rs:89` | `debug_does_not_reveal_the_bearer_token` | `secret` → `fixture_bearer` | 不动（已是刻意混淆过的形态） |
| `crates/prism-mcp/src/middleware.rs:196` | `constant_time_eq_agrees_with_equality_on_every_shape` | `token` → `configured` | 不动 |

两处各补三行注释说明改名理由与两处既有 fixture 同源。`configured` 同时语义更准：它是「配置侧的那个值」，与 `constant_time_eq` 签名里的 `expected` 参数同义但不同作用域。两个测试的断言集合**一条未动**（`constant_time_eq("", "")` 那条留给 plan 01-12）。

**两个调用点**（否则 selftest 只是约定，不是闸门）：

- `justfile`：`check-secrets` 与 `check-all` 里的那行改为显式 `all`；新增 `check-secrets-selftest` 单行委托到 `selftest`（改正则时的快速回路，不碰 git）。
- `.github/workflows/ci.yml`：步骤改名为 `Plaintext secret scan (pattern selftest + repo scan)`，`run` 改为显式 `all`。

两处都显式写 `all` 而不是靠无参数默认值——理由写在注释里：默认值哪天改回 scan-only，闸门会静默失去 selftest 那一半并照常绿。

## Task Verification

| Task | Type | Commit | Verify |
|------|------|--------|--------|
| 1 | auto | `1b4ef33` | `bash scripts/check-secrets.sh selftest` exit 0（0.039s，14 阳性 / 7 阴性）；`nonsense-subcommand` exit 2；`scan` **exit 1**（预期，两处待处理命中） |
| 2 | auto | `e371283` | `bash scripts/check-secrets.sh all` exit 0；`cargo test -p prism-mcp` 全绿；`cargo clippy -p prism-mcp -- -D warnings` exit 0；`cargo test --workspace` exit 0（122 passed / 1 ignored） |

**红 → 绿的转变**：Task 1 结束时 `scan` 打印两行命中并 exit 1；Task 2 改名后 exit 0。这个转变本身就是 widen 真的生效过的证据——一次也没有绿着走完全程。

## 与 01-VERIFICATION.md § SC-4 取样表的逐行对照

| 取样行 | 旧正则 | 新正则 |
|---|---|---|
| `const k = "sk-ant-api03-AbCdEf…";` | ✗ | ✓ |
| `const apiKey: "sk-openai-realkeyvaluehere1234";` | ✗ | ✓ |
| `ANTHROPIC_API_KEY=sk-ant-api03-xyz…` | ✗ | ✓ |
| `Authorization: "Bearer sk-ant-api03-abcdefghijklmnop"` | ✗ | ✓ |
| `const k = "sk-AbCdEfGhIjKlMnOpQrStUvWxYz0123456789";` | ✓ | ✓ |

**1/5 → 5/5。** 五条全部作为阳性组前五条常驻在 selftest 里。

## 反证（落点逐条核对）

| 反证 | 掏空的东西 | 结果 | 落点 |
|------|-----------|------|------|
| F | `sk-` 段改回 `sk-[A-Za-z0-9]{16,}`（恢复旧行为） | selftest exit 1 | 阳性组三条 Anthropic 形态（取样第 1/3/4 行）**+** 阴性组 19 字符那条被误命中——窄字符类下 19 个连续字母数字 ≥ 16 |
| G | PATTERN 改成 `.+`（匹配任意非空行） | selftest exit 1 | 阴性组**全部 7 条** |
| H1 | 长度下界改成 `{21,}` | selftest exit 1 | 阳性组「`sk-` + 恰好 20」那条，仅此一条 |
| H2 | 长度下界改成 `{19,}` | selftest exit 1 | 阴性组「`sk-` + 恰好 19」那条，仅此一条 |
| I | 往 `src/lib/ipc.ts` 追加一行 `sk-ant-api03-…` 形态假串 | scan exit 1 | `src/lib/ipc.ts:130` 打到 stderr。**这正是 01-VERIFICATION.md 判定「绿色扫描等于没扫」的那个场景** |
| J | 往 `scripts/check-secrets.sh` 自身追加同样的串 | scan exit 1 | `scripts/check-secrets.sh:166` 打到 stderr——脚本不再是盲区 |
| K | 往阳性组塞一条不可能命中的样本 | 两个调用点均 exit 1 | CI 步骤命令与 justfile `check-all` recipe 体都红在 `FAIL: positive sample not detected` |

七条全部落在被守的那条样本上。F 与 H1/H2 的落点互补，共同说明字符类与长度下界是**两个独立的**判别维度——只改其一都会被抓住。

反证 F 的额外落点（阴性组 19 字符那条被旧正则误命中）值得记一笔：它说明「窄字符类 + 低阈值」与「宽字符类 + 高阈值」不是简单的松紧关系，两者的判定面互有出入。这也是 `sk-` 段的字符类与长度下界必须同时改的原因。

**`just` 未安装于本机**（justfile 文件头本就声明「本机与 CI 都不假定 just 已安装」），反证 K 的 justfile 侧是直接跑 `check-all` 的两行 recipe 体验证的，与 `just check-all` 等价。

## 威胁登记的处置

| Threat ID | 处置 | 落地形态 |
|---|---|---|
| T-01-44（Anthropic 形态失明，high） | mitigate | 字符类 + 下界双改；selftest 阳性组前五条钉住取样表，命中率 5/5 |
| T-01-45（控件失效仍报绿，high） | mitigate | selftest 每次 CI 执行；反证 F/G/H 三组；`PATTERN=` 源码断言只出现一次 |
| T-01-46（脚本自排除成盲区，medium） | mitigate | 自排除取消；阳性样本片段拼接；反证 J 证明脚本自身也会被抓 |
| T-01-47（为迁就 fixture 放宽正则，medium） | mitigate | 阴性组钉住 5 条既有 fixture；两处新命中以改名解决，allowlist 零新增；prohibition 已登记 |
| T-01-48（docs/ 整目录排除，low） | mitigate | 实测零命中后取消；单文件排除的替代路径写进注释 |
| T-01-49（`\|\| true` 吞掉调用失败，medium） | accept + compensate | `\|\| true` 保留（承重）；补偿控件是 selftest，注释里逐句区分两种 `\|\| true` 与 check-deps.sh 的 WR-11 |

## Deviations from Plan

**None — plan executed exactly as written.**

计划阶段跑过的 21 条样本实测表在执行时逐条复现，无一条与计划不符（这是本 phase 少见的一次：01-05 / 01-06 / 01-10 都出现过计划反证实跑不成立）。计划预言的「widen 后全仓恰好两处新命中、docs 下零命中」也逐条兑现。

三处按计划留出的自由度做了具体选择，均在计划授权范围内、不构成偏离：

1. **阳性组做到 14 条**（计划要求 ≥9）：关键词赋值形态展开成 6 条（`apiKey:` / `api_key=` / `api-key:` / `secret:` / `token =` / `password =`），使 truths 里「各有一条命中样本」逐项可指认。
2. **阴性组做到 7 条**（计划要求 ≥5）：除计划点名的 5 条外，补了一条 17 字符样本，用来说明阈值为什么是 20 而不是 16——否则「16→20」这个改动在样本组里没有任何证据。
3. **阴性组用既有 fixture 的真实源码行而非裸值**：`const FIXTURE_SECRET: &str = "prism-test-secret-value";` 而不是 `prism-test-secret-value`。裸值不会命中是显然的，真实源码行才复现了 scan 实际看到的上下文（`SECRET:` 后面跟着 `&str = "…"`，与关键词赋值段只差一个 `&`）。

## Notes for Next Phase

- **`bash scripts/check-secrets.sh` 的调用形态向后兼容且自动变强**：无参数默认从 scan-only 变成 `all`。任何历史文档里写着裸调用的地方都不必改。
- **新增 fixture 时的规矩已成机制**：撞上扫描器就改 fixture 的名字或值，不动 `scripts/check-secrets.sh`。判断标准很简单——若某个改动会让 selftest 的某条阴性样本被误命中或某条阳性样本不再命中，那就是在放宽防线。
- **Phase 4 引入 chat client 时**：`prism-llm` 会第一次出现真实的 Authorization 头构造代码。阳性组第 4 条（`Authorization: "Bearer sk-ant-api03-…"`）就是为那一刻准备的——届时任何把真实 token 写进测试的做法都会当场变红。
- **INFRA-03 的证据侧至此解除阻塞**，但需求文本的「支持 Anthropic / OpenAI 兼容端点」半句仍要到 Phase 4 才有 chat client（沿用 01-09 / 01-10 的同一判据）。
- **`docs/` 若将来要引用正则**：只排除那一个文件路径，并在 `scan()` 的注释里写明是哪篇、为什么——注释里已经预留了这句话的位置。

## Self-Check: PASSED

五个被改文件与 SUMMARY 全部存在；三个 commit（`1b4ef33` / `e371283` / `b4c8bca`）全部可在 `git log --all` 中找到；三个 commit 均无文件删除。收尾复跑：`bash scripts/check-secrets.sh all` exit 0、`bash scripts/check-deps.sh all` exit 0。
