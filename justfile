# PrismDocs — 常用命令
#
# 主形式是 `bash scripts/…`（本机与 CI 都不假定 `just` 已安装，见 01-RESEARCH.md
# § Environment Availability）；本文件的 check-* recipe 是装了 just 时的等价简写，
# 每条只做单行委托、不重复实现断言逻辑——两份实现必然漂移。

# 格式闸门：本项目显式采用 rustfmt 默认风格（决定记在 rustfmt.toml 的文件头）。
# 与 CI engine job 的第一步逐字等价。
fmt-check:
    cargo fmt --all -- --check

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

# D-01 的证据形态：engine 选择集单独可测（不是 --workspace，那会编译 shell）。
# `prism-cli` 一并在内，与 CI 的 `Test (engine selection set + CLI helper)` 等价——
# 它按 D-10 同样 tauri-free，且此前它的测试只作为 --workspace 覆盖率的副作用被跑到。
test-engine:
    cargo test -p prism-types -p prism-store -p prism-fs -p prism-parse -p prism-anchor -p prism-llm -p prism-mcp -p prism-engine -p prism-cli

# 全部 Rust 的 lint 闸门，与 CI 的两条 clippy 步骤逐字等价（engine job 一条、shell job 一条）。
# 分两次调用不是风格选择：壳必须带 `--features test`（否则 tests/ipc.rs 编译成零个测试）
# 与 `--all-targets`（否则 tests/ 根本不进受检面），而 engine 侧不需要也不应该带这个 feature。
clippy-all:
    cargo clippy -p prism-types -p prism-store -p prism-fs -p prism-parse -p prism-anchor -p prism-llm -p prism-mcp -p prism-engine -p prism-cli -- -D warnings
    cargo clippy -p prismdocs-shell --all-targets --features test -- -D warnings

# 覆盖率：Phase 1 只测量与呈报，不设阈值（理由见 01-02-SUMMARY.md）。
# 范围与 CI 的 `Coverage (engine)` 一致：八个 engine crate，不是 --workspace。
# `clean` 是范围的一部分——`report` 汇总它在 target/ 里找到的全部 llvm-cov 对象，
# 不 clean 时上一次跑过的壳/CLI 对象会混进 TOTAL（本机实测过）。
# 前置条件（都不随 rustup / 仓库自带）：
#   cargo install cargo-llvm-cov     # engine 侧；CI 由 taiki-e/install-action 装
#   npm ci                           # 前端侧，装上 @vitest/coverage-v8
coverage:
    cargo llvm-cov clean --workspace
    cargo llvm-cov --no-report -p prism-types -p prism-store -p prism-fs -p prism-parse -p prism-anchor -p prism-llm -p prism-mcp -p prism-engine
    cargo llvm-cov report --summary-only
    npm run test -- --run --coverage
