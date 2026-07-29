#!/usr/bin/env bash
#
# 依赖方向断言 —— 成功标准 1（依赖图性质）与成功标准 4（唯一出口）的可执行形式。
#
# 这四条约束（D-01 / D-09 / NFR-03 与「无重复 SQLite/HTTP 栈」）都是**依赖图性质**：
# 代码评审看不住，只有 cargo tree 断言能看住。本文件是断言的唯一实现，
# justfile 与 .github/workflows/ci.yml 都只是调用者。
#
# 用法: bash scripts/check-deps.sh [dup|tauri-free|no-cycle|single-egress|subscriber-free|all]
#       无参数等同 all。失败时打印违规 crate 名并 exit 1。

set -euo pipefail

# 八个 engine crate —— 也是 `just test-engine` 的选择集。
ENGINE_CRATES="prism-types prism-store prism-fs prism-parse prism-anchor prism-llm prism-mcp prism-engine"

# tauri-free 的受检集合：engine 八个 + CLI helper。
# 多出来的那一个未来要作为 externalBin 单独签名公证，一旦悄悄链上 tauri，
# 公证与体积问题会到 Phase 6 才炸；纳入这条断言的成本是零。
TAURI_FREE_CRATES="$ENGINE_CRATES prism-cli"

# single-egress 的受检集合：**叶子** engine crate。它们的整棵普通依赖树里都不得出现
# 网络/密钥包——这几个 crate 没有任何理由碰到它们。
#
# 排除三个：prism-llm 与 prism-cli 是被允许触网/触密钥的（NFR-03 的那个「唯一出口」
# 就是 prism-llm）；prism-engine 另有一条更精确的断言，见 check_facade_egress。
PURE_CRATES="prism-types prism-store prism-fs prism-parse prism-anchor"

# 网络/密钥包的名字。两条断言共用。
EGRESS_CRATES='reqwest|keyring-core|apple-native-keyring-store'

# 成功标准 1-b：同一进程不得链接两份 SQLite / HTTP 栈。
# --duplicates 只列多版本共存的包，命中即意味着 WAL 状态可能分叉。
#
# 这里刻意**不用** `|| true`。`check-secrets.sh` 的 scan 里那个 `|| true` 是**承重**的：
# `git grep` 无命中时退出 1，吞掉它才能把「干净」与「失败」分开。而这一行被吞掉的恰好
# 是失败本身——命令没跑起来 → `out` 为空串 → `grep` 什么都找不到 → 函数打 OK 返回 0，
# 断言在它什么都没学到的时候报告成功。两者形态相同、性质相反，所以只能逐处判断，
# 不存在「`|| true` 一律可用」或「一律禁用」的规则。
#
# 于是先接住退出码再看输出：`rc != 0` 说的是**这条断言没跑起来**，与「发现了重复」
# 是两件完全不同的事，消息必须让读的人一眼分清（未来 cargo 改掉 --duplicates 这个
# flag 名时，走的就是这条分支）。
check_dup() {
  local out rc=0
  # 命令替换失败时不让 set -e 直接掐断函数——把退出码记进 rc 自己判。
  out=$(cargo tree --workspace --duplicates --edges normal) || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "FAIL: \`cargo tree --workspace --duplicates\` could not run (exit $rc)" >&2
    echo "      —— 这不是「发现了重复依赖」，是这条断言本身没跑起来，未提供任何证据。" >&2
    return 1
  fi
  if grep -Eq '^(rusqlite|reqwest|libsqlite3-sys) v' <<<"$out"; then
    echo "FAIL: duplicate critical crate in dependency tree" >&2
    grep -E '^(rusqlite|reqwest|libsqlite3-sys) v' <<<"$out" >&2
    return 1
  fi
  echo "OK: no duplicate rusqlite/reqwest/libsqlite3-sys"
}

# 成功标准 1-a（D-01）：engine 与 CLI helper 的普通+构建依赖树中不得出现 tauri。
# 用 normal,build 而不是只 normal —— build script 依赖同样会把 tauri 拖进构建图。
check_tauri_free() {
  local c out
  for c in $TAURI_FREE_CRATES; do
    out=$(cargo tree -p "$c" --edges normal,build --prefix none)
    if grep -Eq '^tauri( |$)' <<<"$out"; then
      echo "FAIL: $c depends on tauri" >&2
      return 1
    fi
  done
  echo "OK: all checked crates are tauri-free (engine set + CLI helper)"
}

# 成功标准 1-c（D-09）：MCP 服务端不得反向依赖 facade。
# 必须是 --edges normal：cargo 允许「A→B 普通 + B→A dev」这种环存在且普通编译不报错，
# dev 边是这条约束唯一的逃逸口，断言里显式排除它。
#
# 为什么是 `^prism-engine ` 带一个尾随空格而不是裸前缀：`--prefix none` 的每行形如
# `<name> v<version>`，包名与版本之间必然有一个空格，所以尾随空格就是包名的右边界。
# 裸前缀 `^prism-engine` 会把任何同前缀的**别的** crate 一并吸收（`prism-engine-core`、
# `prism-engineering`）——本处实测方向是**过敏**：追加一行 `prism-engine-core v0.1.0`
# 后裸前缀命中（把一个别的 crate 报成环），尾随空格不命中。下面两处 offenders 抽取
# 的裸前缀则是**截断**（`prism-mcp2` 被折成 `prism-mcp`），方向相反、根因同一个：
# 包名没有锚定到边界。
#
# 代价要写明：将来若真的拆出一个 facade 层的新 crate（比如 prism-engine-core），
# 它不会再因为名字前缀相同而被这一条顺带看住——**必须显式加进这里**，
# 不要依赖前缀巧合来提供覆盖。
check_no_cycle() {
  local out body
  out=$(cargo tree -p prism-mcp --edges normal --prefix none)
  body=$(printf '%s\n' "$out" | tail -n +2)
  if grep -q '^prism-engine ' <<<"$body"; then
    echo "FAIL: prism-mcp depends on prism-engine" >&2
    return 1
  fi
  echo "OK: prism-mcp -> prism-types only"
}

# 成功标准 4（NFR-03）第一半：叶子 engine crate 的整棵普通依赖树里没有网络/密钥包。
check_single_egress() {
  local c out
  for c in $PURE_CRATES; do
    out=$(cargo tree -p "$c" --edges normal --prefix none)
    if grep -Eq "^($EGRESS_CRATES) " <<<"$out"; then
      echo "FAIL: $c has network/secret dependency" >&2
      return 1
    fi
  done
  echo "OK: leaf engine crates carry no network/secret dependency"
}

# 成功标准 4（NFR-03）第二半：facade 可以**转交**密钥，但不得自己开第二个出口。
#
# 为什么 prism-engine 不能沿用上面那条整棵树的断言：src-tauri 只依赖 prism-engine，
# 所以 shell 通往钥匙串的唯一路线必然经过 facade。要么 facade 依赖 prism-llm，
# 要么把 prism-llm 直接塞给 shell——后者才是真正破坏「唯一入口」的那个选项。
# 于是对 facade 的正确断言不是「树里没有」，而是**「只能经 prism-llm 进来」**：
#
#   (a) prism-engine 自己的直接依赖里没有网络/密钥包；
#   (b) 在 prism-engine 的依赖树内，这些包的反向依赖闭包中除 prism-llm 与
#       prism-engine 自身外，不含任何 prism-* crate。
#
# (b) 是真正有牙的一条：哪天 prism-store 或 prism-anchor 悄悄加了 keyring，
# 它会作为一个新的 prism-* 名字出现在反向闭包里而被抓住——而整棵树的断言
# 那时只会说「prism-engine 有密钥依赖」，与现状无法区分。
check_facade_egress() {
  local direct inverted offenders c
  direct=$(cargo tree -p prism-engine --edges normal --depth 1 --prefix none | tail -n +2)
  if grep -Eq "^($EGRESS_CRATES) " <<<"$direct"; then
    echo "FAIL: prism-engine declares a network/secret crate as a direct dependency" >&2
    return 1
  fi

  for c in reqwest keyring-core apple-native-keyring-store; do
    # 该包不在树里就没什么可查的（--invert 对不存在的包会报错）。
    if ! cargo tree -p prism-engine --edges normal --prefix none \
         | grep -Eq "^$c "; then
      continue
    fi
    inverted=$(cargo tree -p prism-engine --edges normal --invert "$c" --prefix none)
    # 抽到尾随空格为止（字符类同时含数字与连字符），再去掉空格进 sort：
    # 裸前缀 `^prism-[a-z]+` 会把 `prism-mcp2 v0.1.0` **截断**成 `prism-mcp`——
    # 一个真违规被折叠成一个合法邻居的名字，报出来的名字指向一个无辜的 crate。
    # 去空格必须在 allowlist 过滤**之前**，否则 `prism-llm ` 带着空格与 `^prism-llm$` 不等，
    # 合法者会被当成违规。
    offenders=$(grep -oE '^prism-[a-z0-9-]+ ' <<<"$inverted" \
                | sed 's/ $//' | sort -u | grep -vE '^(prism-llm|prism-engine)$' || true)
    if [ -n "$offenders" ]; then
      echo "FAIL: $c reaches prism-engine through a crate other than prism-llm:" >&2
      echo "$offenders" >&2
      return 1
    fi
  done
  echo "OK: prism-engine only ever reaches network/secrets through prism-llm"
}

# 成功标准 4（NFR-03）第三半：shell 通往钥匙串的路线只有「经 facade」这一条。
#
# 01-07 把 facade 的形态守住了，但受检集合里**没有 prismdocs-shell**。
# 现状是 shell → prism-engine → prism-llm → keyring，即 NFR-03 要的单一入口；
# 而 `src-tauri/Cargo.toml` 里加一行 `prism-llm = ...` 就能让它变成两条路，
# 且上面五条断言没有一条会红（shell 不在任何一个受检集合里）。
#
# 形态与 check_facade_egress 同构（直接依赖 + 反向闭包），只是允许名单里多了
# prism-engine——它在这条链上是**合法的中间跳**，而不是第二个出口。
check_shell_egress() {
  local direct inverted offenders c
  direct=$(cargo tree -p prismdocs-shell --edges normal --depth 1 --prefix none | tail -n +2)
  if grep -Eq "^($EGRESS_CRATES) " <<<"$direct"; then
    echo "FAIL: prismdocs-shell declares a network/secret crate as a direct dependency" >&2
    return 1
  fi
  if grep -Eq '^prism-llm ' <<<"$direct"; then
    echo "FAIL: prismdocs-shell depends on prism-llm directly —— 密钥入口不再唯一" >&2
    return 1
  fi

  for c in reqwest keyring-core apple-native-keyring-store; do
    if ! cargo tree -p prismdocs-shell --edges normal --prefix none \
         | grep -Eq "^$c "; then
      continue
    fi
    inverted=$(cargo tree -p prismdocs-shell --edges normal --invert "$c" --prefix none)
    # 与 check_facade_egress 同一口径：抽到尾随空格为止再去空格，
    # 否则 `prism-mcp2` 会被截断成 allowlist 邻域里的 `prism-mcp`。见上一处的完整理由。
    offenders=$(grep -oE '^prism-[a-z0-9-]+ ' <<<"$inverted" \
                | sed 's/ $//' | sort -u | grep -vE '^(prism-llm|prism-engine)$' || true)
    if [ -n "$offenders" ]; then
      echo "FAIL: $c reaches prismdocs-shell through a crate other than prism-llm:" >&2
      echo "$offenders" >&2
      return 1
    fi
  done
  echo "OK: prismdocs-shell only ever reaches network/secrets through prism-engine -> prism-llm"
}

# D-01 的延伸：日志 **subscriber** 是壳的职责，engine crate 只发不装。
#
# 守的是什么：workspace 里 7 处 `tracing` 发射点是 engine 侧的，而安装全局 dispatcher
# 这件事只属于 `src-tauri`。一旦 subscriber 进了任一 engine crate 的普通依赖树，
# 「engine 可脱离壳独立测试」这条性质就开始漏——而且它**不会以编译错误的形式表现出来**：
# 多一个依赖照常编译、照常跑测试，只是 engine 从此拖着一整套日志栈。
#
# 受检集合沿用 TAURI_FREE_CRATES 而不是 ENGINE_CRATES：prism-cli 将来要作为 externalBin
# 单独签名公证，它悄悄链上一个日志栈同样是要到 Phase 6 才炸的那类问题，纳入成本为零。
#
# 只看 --edges normal：dev-dependencies 里出现 subscriber 是合理的（测试可以自己装一个），
# 这条断言守的是**普通**依赖边。
check_subscriber_free() {
  local c out
  for c in $TAURI_FREE_CRATES; do
    out=$(cargo tree -p "$c" --edges normal --prefix none)
    if grep -Eq '^tracing-subscriber( |$)' <<<"$out"; then
      echo "FAIL: $c depends on tracing-subscriber" >&2
      return 1
    fi
  done
  echo "OK: all checked crates are tracing-subscriber-free (engine set + CLI helper)"
}

main() {
  case "${1:-all}" in
    dup)            check_dup ;;
    tauri-free)     check_tauri_free ;;
    no-cycle)       check_no_cycle ;;
    # single-egress 跑两条：NFR-03 是一条约束，拆成两条断言只是因为 facade 与叶子
    # crate 的正确形态不同。子命令保持单一，调用方（justfile / CI / 计划验收项）不必知道这件事。
    single-egress)
      check_single_egress
      check_facade_egress
      check_shell_egress
      ;;
    facade-egress)  check_facade_egress ;;
    shell-egress)   check_shell_egress ;;
    subscriber-free) check_subscriber_free ;;
    all)
      check_dup
      check_tauri_free
      check_no_cycle
      check_single_egress
      check_facade_egress
      check_shell_egress
      check_subscriber_free
      ;;
    *)
      echo "usage: $0 [dup|tauri-free|no-cycle|single-egress|facade-egress|shell-egress|subscriber-free|all]" >&2
      exit 2
      ;;
  esac
}

main "$@"
