---
status: testing
phase: 01-foundation-skeleton
source: [01-VERIFICATION.md]
started: 2026-07-30T00:40:00Z
updated: 2026-07-30T00:40:00Z
---

## Current Test

number: 1
name: 真实 WebView 下的 CSP 与 IPC 双通路（同时承载成功标准 2 的复验）
expected: |
  六步全部正常，三个入口读数与 01-09 一致。第 ④ 步的两条新指令预期不触发：
  Phase 1 前端不含原生 `<form>` 提交，桌面窗口也不会被嵌套。
awaiting: user response

## Tests

### 1. 真实 WebView 下的 CSP 与 IPC 双通路（同时承载成功标准 2 的复验）

步骤（六步）:
1. `npm run tauri dev` 起应用，窗口不是白屏
2. 设置页可用，保存一个合法端点（如 `https://api.anthropic.com`）出现成功文案
3. dev 冒烟页跑三个入口：总线事件 1:1 计数 / Channel「seq 校验通过 · 实收 1000 条」/ 中文搜索「锚定引擎」>0 且「量子纠缠」=0
4. WebView 控制台无任何 CSP 违规报告，特别留意 01-24 新加的 `form-action 'none'` 与 `frame-ancestors 'none'`
5. `npm run tauri build` 出 dmg，安装后对 ①–④ 重复一遍（发布形态走 `csp` 而非 `devCsp`，这是验证严格那一份的唯一路径）
6. 发布形态额外确认冒烟页开关不存在且四个 `dev_*` 命令不可调用

expected: 六步全部正常，三个入口读数与 01-09 一致。第 ④ 步的两条新指令预期不触发。

why_human: CSP 只在真实 WebView 里生效——`src/lib/tauri-security.test.ts:17` 自己写明 jsdom 看不见它，`cargo test` 走 `mock_builder` 也无 WebView。

⚠️ 出现违规时的处理：只放宽 `devCsp`，或按控制台点名的指令逐项追加到 `csp`；**禁止**把 `csp` 设回 `null`，也**禁止**直接删掉 01-24 新加的两条指令——若确需放宽，先在 `tauri-security.test.ts` 的精确相等断言上过一次评审（WINDOWS id=8）。

result: [pending]

### 2. 日志 sink 真的有落点

步骤（两步）:
1. 默认档位（**不设** `RUST_LOG`）`npm run tauri dev`，在设置页把 base_url 设成一个非 loopback 的 `http://` 端点（如 `http://example.com/v1`）并保存，观察终端
2. `RUST_LOG=trace npm run tauri dev`，观察终端

expected:
- 步骤 1：出现 tracing 格式的行，且 `crates/prism-store/src/settings.rs:88-91` 那条 `LLM endpoint uses plaintext http to a non-loopback host` 实际打出来（默认 info 档下就有落点，无需提档）
- 步骤 2：出现 01-21 的降档 warn，正文以 `the environment-supplied log filter exceeds the project ceiling` 开头并说明 `rmcp` 被 capped at INFO，且正文中**不含** `RUST_LOG` 的原值

why_human: `tracing::dispatcher::has_been_set()` 只证明 dispatcher 就位，不证明日志到达终端（EnvFilter 档位与 fmt 层的输出目标都可能让它落空）。

⚠️ 01-21 之后**必须走默认档位**：该 plan 给 env filter 加了项目天花板（`src-tauri/src/lib.rs:51` `LOG_CEILING_DIRECTIVE = "rmcp=info"`），原先「用 `RUST_LOG` 提档观察」既观察不到目标、也不再是 sink 有落点的证据（WINDOWS id=9）。

result: [pending]

### 3. CI workflow 的首次真实 GitHub Actions 运行

推分支后核对四项:
1. engine job 首步 `Format check (rustfmt default style)` 出现在步骤列表最前且为绿
2. `permissions: contents: read` 下 `upload-artifact` 仍可上传
3. `concurrency` 真的收掉同 commit 的双跑
4. 两个缓存分段互不恢复

expected: 四项全部成立。判别力可在一个丢弃分支上注入劣化排版验证 fmt 步骤会变红并点名文件。

why_human: `origin/main` 停在 `4cc1347`，Phase 1 全部 28 份 plan 的产物均未推送，`gh run list` 返回 `[]`——该 workflow 至今未在 GitHub Actions 上跑过。所有 CI 闸门声称在本 phase 里都只有本机证据（WINDOWS id=14 / id=15，同一次运行可一并核对）。

result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
