#!/usr/bin/env bash
#
# 依赖方向断言 —— 成功标准 1（依赖图性质）与成功标准 4（唯一出口）的可执行形式。
#
# 这四条约束（D-01 / D-09 / NFR-03 与「无重复 SQLite/HTTP 栈」）都是**依赖图性质**：
# 代码评审看不住，只有 cargo tree 断言能看住。本文件是断言的唯一实现，
# justfile 与 .github/workflows/ci.yml 都只是调用者。
#
# 用法: bash scripts/check-deps.sh [dup|tauri-free|no-cycle|single-egress|all]
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
check_dup() {
  local out
  out=$(cargo tree --workspace --duplicates --edges normal || true)
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
check_no_cycle() {
  local out body
  out=$(cargo tree -p prism-mcp --edges normal --prefix none)
  body=$(printf '%s\n' "$out" | tail -n +2)
  if grep -q '^prism-engine' <<<"$body"; then
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
    offenders=$(grep -oE '^prism-[a-z]+' <<<"$inverted" \
                | sort -u | grep -vE '^(prism-llm|prism-engine)$' || true)
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
    offenders=$(grep -oE '^prism-[a-z]+' <<<"$inverted" \
                | sort -u | grep -vE '^(prism-llm|prism-engine)$' || true)
    if [ -n "$offenders" ]; then
      echo "FAIL: $c reaches prismdocs-shell through a crate other than prism-llm:" >&2
      echo "$offenders" >&2
      return 1
    fi
  done
  echo "OK: prismdocs-shell only ever reaches network/secrets through prism-engine -> prism-llm"
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
    all)
      check_dup
      check_tauri_free
      check_no_cycle
      check_single_egress
      check_facade_egress
      check_shell_egress
      ;;
    *)
      echo "usage: $0 [dup|tauri-free|no-cycle|single-egress|facade-egress|shell-egress|all]" >&2
      exit 2
      ;;
  esac
}

main "$@"
