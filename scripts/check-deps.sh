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

# single-egress 的受检集合：排除被允许触网/触密钥的两个包（LLM 客户端与 CLI helper）。
PURE_CRATES="prism-types prism-store prism-fs prism-parse prism-anchor prism-engine"

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

# 成功标准 4（NFR-03）：网络与密钥只有一个出口。
check_single_egress() {
  local c out
  for c in $PURE_CRATES; do
    out=$(cargo tree -p "$c" --edges normal --prefix none)
    if grep -Eq '^(reqwest|keyring-core|apple-native-keyring-store) ' <<<"$out"; then
      echo "FAIL: $c has network/secret dependency" >&2
      return 1
    fi
  done
  echo "OK: prism-llm is the sole network+secret crate among engine crates"
}

main() {
  case "${1:-all}" in
    dup)            check_dup ;;
    tauri-free)     check_tauri_free ;;
    no-cycle)       check_no_cycle ;;
    single-egress)  check_single_egress ;;
    all)
      check_dup
      check_tauri_free
      check_no_cycle
      check_single_egress
      ;;
    *)
      echo "usage: $0 [dup|tauri-free|no-cycle|single-egress|all]" >&2
      exit 2
      ;;
  esac
}

main "$@"
