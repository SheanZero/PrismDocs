#!/usr/bin/env bash
#
# 明文密钥静态检查 —— 成功标准 4 第三分句「代码与配置中无明文密钥」的可执行形式。
#
# 为什么这条断言存在：提交进仓库的密钥会随源码与构建产物一并流出，而 git 历史不可撤销。
# 代码评审看不住这件事，只有一条每次 push 都跑的扫描能看住。
#
# 为什么它还必须**自证判别力**：这条成功标准的自动化证据就是本脚本退出 0。
# 一个看不见目标格式的正则同样退出 0——证据链断在这里而没有任何人会察觉。
# 01-VERIFICATION.md 记录的正是这个失效：旧正则要求 `sk-` 后紧跟 ≥16 个**连续**字母数字，
# 而 Anthropic 的密钥在第三个字符的连字符处就断开，5 行真实形态取样只命中 1 行。
# 本项目点名 Anthropic Messages API 是一等端点（CLAUDE.md，Settings 页 placeholder 即
# api.anthropic.com）——最可能被泄漏的那个供应商的密钥，恰好是扫描器唯一看不见的。
# 于是有了 selftest：正则的判别力从此是每次 CI 都重新证明一遍的断言，而不是一句声明。
#
# 本文件是断言的唯一实现；justfile 与 .github/workflows/ci.yml 都只是调用者。
#
# 用法: bash scripts/check-secrets.sh [scan|selftest|all]
#       无参数等同 all（先 selftest 后 scan）。未知子命令打 usage 到 stderr 并 exit 2。
#
#   scan     —— 对受版本控制的文件跑正则，有命中即把命中行打到 stderr 并 exit 1。
#   selftest —— 两组样本：阳性组每条必须命中、阴性组每条必须不命中。
#
# 样本溯源：
#   阳性组前五条逐条对应 .planning/phases/01-foundation-skeleton/01-VERIFICATION.md
#   § SC-4 的 5 行真实形态取样表。旧正则在那张表上命中 1/5，widen 后必须 5/5。
#   阴性组前两条是既有 fixture 的值：prism-llm 的 FIXTURE_SECRET（secrets.rs）与
#   前端的 FAKE_KEY（Settings.test.tsx），第三到五条是 plan 01-10 新增的凭据型 URL fixture。
#   它们在这里被钉成断言，方向是单向的：**fixture 撞上扫描器时改 fixture，不改扫描器**。
#   放宽正则、加 allowlist、加整目录排除、降长度阈值——四者都算放宽，都不允许。
#   （这条约定已由 secrets.rs 与 Settings.test.tsx 两处注释确立；本文件把它变成可执行断言。）
#
# 自扫约束：本脚本自身在 scan 的受检集合里。因此**任何新增的阳性样本都必须由片段拼出**，
# 使源码里不出现命中自身正则的连续字面量。scan 退出 0 这件事本身就是该性质的证明。

set -euo pipefail

# 拆成两段拼接，避免在源码里直接写出成串的引号字面量。
QUOTE="[\"']"
NOT_QUOTE="[^\"']"

# 四段 alternation。两个子命令共用这一个变量——不得复制第二份，
# 否则 selftest 测的是正则的副本，与 scan 实际用的那条可以静默漂移。
#
#   1. sk-…      字符类含连字符与下划线是本次修复的核心：Anthropic 的 sk-ant-api03-… 形态
#                在连字符处被旧的 [A-Za-z0-9] 截断。长度下界同时从 16 提到 20——
#                放宽字符类后 16 会把普通的连字符标识符串扫进来。
#   2. ghp_…     GitHub personal access token 前缀。
#   3. AKIA…     AWS access key id 前缀，长度固定 16。
#   4. 关键词赋值 在旧的 api_key 等号形态上扩了 secret / token / password 三个关键词
#                与冒号赋值形态；git grep 带 -i，故 API_KEY / apiKey 两种大小写都进网。
PATTERN="sk-[A-Za-z0-9_-]{20,}|ghp_[A-Za-z0-9]{20,}|AKIA[A-Z0-9]{16}|(api[_-]?key|secret|token|password)[[:space:]]*[=:][[:space:]]*${QUOTE}${NOT_QUOTE}{8,}"

# 对受版本控制的文件跑一遍正则。
scan() {
  local hits

  # 排除集只剩一条。.planning/ 按设计要引用取样密钥——01-VERIFICATION.md § SC-4 的
  # 5 行取样表就是活证据，规划与验证文档不引用真实形态就没法讨论这个问题。
  #
  # docs/ 的整目录排除已**取消**：执行本次修改时实测确认 docs 下零命中，
  # 一片扫不到的地方不该因为「可能会有文档引用正则」这种假设而存在。
  # 将来某篇文档确需引用时，只排除那一个文件路径，并在这里写明是哪篇、为什么。
  #
  # 本脚本自身的排除也已取消——一个把自己排除在外的检查器就是个人人可写的盲区。
  #
  # `|| true` 在这里是**承重**的：git grep 无命中时退出 1，不吞掉它 set -e 会把「干净」
  # 当成失败。它与 check-deps.sh 的 WR-11（`cargo tree … || true` 吞掉真实调用失败后
  # grep 空串即报 OK）形态相同、性质相反：那里被吞的是失败，这里被吞的是干净。
  # 两者无法在这一行内区分，所以配套证明放在 selftest：一次被吞掉的调用失败会让 scan
  # 看到空串而报 OK，而 selftest 的阳性样本会当场不再命中而变红。
  hits=$(git grep -niE "$PATTERN" -- ':(exclude).planning/' || true)

  if [ -n "$hits" ]; then
    echo "FAIL: possible plaintext secret in version-controlled files" >&2
    printf '%s\n' "$hits" >&2
    return 1
  fi
  echo "OK: no plaintext secret in version-controlled files"
}

# 证明上面那条正则真的有判别力。不碰 git，纯字符串匹配。
selftest() {
  # 阳性样本一律由片段拼出（见文件头的自扫约束）。阴性样本反而整串直写——
  # 它们本来就不该命中，直写正是要断言的那件事。
  local sk="sk" dash="-" ghp="ghp" us="_" aws="AKI" q='"'
  local positive=() negative=() s failed=0

  # ——阳性组前五条：01-VERIFICATION.md § SC-4 的 5 行取样，旧正则只命中最后一条。
  positive+=("const k = ${q}${sk}${dash}ant-api03-AbCdEfGhIjKlMnOpQrStUvWx${q};")
  positive+=("const apiKey: ${q}${sk}${dash}openai-realkeyvaluehere1234${q};")
  positive+=("ANTHROPIC_API_KEY=${sk}${dash}ant-api03-xyz0123456789abcdefghij")
  positive+=("Authorization: ${q}Bearer ${sk}${dash}ant-api03-abcdefghijklmnop${q}")
  positive+=("const k = ${q}${sk}${dash}AbCdEfGhIjKlMnOpQrStUvWxYz0123456789${q};")

  # ——阳性组：关键词赋值的六种形态（三种关键词拼写 × 等号/冒号 × 大小写）。
  positive+=("apiKey: ${q}realkeyvaluehere1234${q}")
  positive+=("api_key=${q}realkeyvaluehere1234${q}")
  positive+=("api-key: ${q}realkeyvaluehere1234${q}")
  positive+=("secret: ${q}not-a-real-value-here${q}")
  positive+=("let token = ${q}0123456789abcdef${q};")
  positive+=("password = ${q}hunter2hunter2${q}")

  # ——阳性组：另两种前缀。
  positive+=("${ghp}${us}0123456789abcdefghijklmnopqr")
  positive+=("${aws}AIOSFODNN7EXAMPLE")

  # ——阳性组：长度阈值的下界。与阴性组同名的那条构成成对边界样本。
  positive+=("const k = ${q}${sk}${dash}abcdefghijklmnopqrst${q};")

  # ——阴性组前两条：既有 fixture，扫描器不得为了迁就它们而放宽（反之亦然）。
  negative+=("const FIXTURE_SECRET: &str = \"prism-test-secret-value\";")
  negative+=("const FAKE_KEY = \"fixture-not-a-real-credential\";")

  # ——阴性组：plan 01-10 新增的凭据型 URL fixture（userinfo / query / fragment 三面）。
  negative+=("const CREDENTIAL_URL: &str = \"https://prism-test-user:prism-test-secret-value@api.vendor.com/v1\";")
  negative+=("const IN_QUERY: &str = \"https://api.vendor.com/v1?api-key=prism-test-secret-value\";")
  negative+=("const IN_FRAGMENT: &str = \"https://api.vendor.com/v1#prism-test-secret-value\";")

  # ——阴性组：长度阈值的下界之下（恰好 19，比上面那条阳性样本少一个字符）。
  negative+=("const k = \"sk-abcdefghijklmnopqrs\";")

  # ——阴性组：阈值从 16 提到 20 挡下的那类东西。放宽字符类后它在 16 下会命中。
  negative+=("const mode = \"sk-dev-mode-fallback\";")

  # 逐条喂样本一律用 herestring，**不得用管道**：pipefail 下 `grep -q` 命中即早退，
  # 写端拿到 SIGPIPE 后管道整体的退出码会掩盖判定结果，断言静默恒绿。
  # 本 phase 已经在 check-deps.sh 的依赖方向断言上踩过一次这个坑。
  for s in "${positive[@]}"; do
    if ! grep -qiE "$PATTERN" <<<"$s"; then
      echo "FAIL: positive sample not detected: $s" >&2
      failed=1
    fi
  done

  for s in "${negative[@]}"; do
    if grep -qiE "$PATTERN" <<<"$s"; then
      echo "FAIL: negative sample wrongly detected: $s" >&2
      failed=1
    fi
  done

  if [ "$failed" -ne 0 ]; then
    return 1
  fi
  echo "OK: pattern discriminates (${#positive[@]} positive / ${#negative[@]} negative samples)"
}

main() {
  case "${1:-all}" in
    scan)     scan ;;
    selftest) selftest ;;
    # 先 selftest 后 scan：正则失去判别力时，先红的应当是那个原因而不是它的后果。
    all)
      selftest
      scan
      ;;
    *)
      echo "usage: $0 [scan|selftest|all]" >&2
      exit 2
      ;;
  esac
}

main "$@"
