# PrismDocs — 常用命令
#
# 主形式是 `bash scripts/…`（本机与 CI 都不假定 `just` 已安装，见 01-RESEARCH.md
# § Environment Availability）；本文件的 check-* recipe 是装了 just 时的等价简写，
# 每条只做单行委托、不重复实现断言逻辑——两份实现必然漂移。

# 成功标准 1-b：无重复 rusqlite / reqwest / libsqlite3-sys
check-dup:
    bash scripts/check-deps.sh dup

# 成功标准 1-a（D-01）：engine crate 与 CLI helper 的依赖树中无 tauri
check-tauri-free:
    bash scripts/check-deps.sh tauri-free

# 成功标准 1-c（D-09）：prism-mcp 无 facade 依赖（普通边）
check-no-cycle:
    bash scripts/check-deps.sh no-cycle

# 成功标准 4（NFR-03）：网络与密钥只有一个出口
check-single-egress:
    bash scripts/check-deps.sh single-egress

# 成功标准 4：正则的判别力自证（selftest）+ 受版本控制的文件里无明文密钥（scan）。
# 显式写 all 而不是靠无参数默认值：默认值哪天改回 scan-only，闸门会静默失去 selftest 那一半。
check-secrets:
    bash scripts/check-secrets.sh all

# 只跑判别力自证那一半（改正则时的快速回路，不碰 git）
check-secrets-selftest:
    bash scripts/check-secrets.sh selftest

# 六条依赖方向断言 + 密钥静态检查两半（CI 的 engine job 跑的就是这两条）
check-all:
    bash scripts/check-deps.sh all
    bash scripts/check-secrets.sh all

# D-01 的证据形态：engine 选择集单独可测（不是 --workspace，那会编译 shell）
test-engine:
    cargo test -p prism-types -p prism-store -p prism-fs -p prism-parse -p prism-anchor -p prism-llm -p prism-mcp -p prism-engine

# 覆盖率：Phase 1 只测量与呈报，不设阈值（理由见 01-02-SUMMARY.md）。
# 前置条件（都不随 rustup / 仓库自带）：
#   cargo install cargo-llvm-cov     # engine 侧；CI 由 taiki-e/install-action 装
#   npm ci                           # 前端侧，装上 @vitest/coverage-v8
coverage:
    cargo llvm-cov --workspace --summary-only
    npm run test -- --run --coverage
