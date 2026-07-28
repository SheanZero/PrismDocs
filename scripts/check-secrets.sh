#!/usr/bin/env bash
#
# 明文密钥静态检查（成功标准 4 / T-01-03a）。
#
# 只扫受版本控制的文件（git grep）——提交进仓库的密钥会随源码与构建产物一并流出。
# 排除 .planning/ 与 docs/（规划与研究文档会引用这些正则本身），以及本脚本自身。
#
# 用法: bash scripts/check-secrets.sh
#       有命中时把命中行打印到 stderr 并 exit 1；无命中则静默 exit 0。

set -euo pipefail

# 拆成两段拼接，避免在源码里直接写出成串的引号字面量。
QUOTE="[\"']"
NOT_QUOTE="[^\"']"

# 两类形态：OpenAI 风格长令牌前缀；api_key / api-key / apikey 的引号串赋值。
PATTERN="sk-[A-Za-z0-9]{16,}|api[_-]?key[[:space:]]*=[[:space:]]*${QUOTE}${NOT_QUOTE}{8,}"

hits=$(git grep -nE "$PATTERN" -- \
  ':(exclude).planning/' \
  ':(exclude)docs/' \
  ':(exclude)scripts/check-secrets.sh' || true)

if [ -n "$hits" ]; then
  echo "FAIL: possible plaintext secret in version-controlled files" >&2
  printf '%s\n' "$hits" >&2
  exit 1
fi

exit 0
