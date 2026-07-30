---
status: complete
phase: 01-foundation-skeleton
source: [01-VERIFICATION.md]
started: 2026-07-30T00:40:00Z
updated: 2026-07-30T03:20:00Z
---

## Current Test

[testing complete]

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

result: pass

### 2. 日志 sink 真的有落点

步骤（两步）:
1. 默认档位（**不设** `RUST_LOG`）`npm run tauri dev`，在设置页把 base_url 设成一个非 loopback 的 `http://` 端点（如 `http://example.com/v1`）并保存，观察终端
2. `RUST_LOG=trace npm run tauri dev`，观察终端

expected:
- 步骤 1：出现 tracing 格式的行，且 `crates/prism-store/src/settings.rs:88-91` 那条 `LLM endpoint uses plaintext http to a non-loopback host` 实际打出来（默认 info 档下就有落点，无需提档）
- 步骤 2：出现 01-21 的降档 warn，正文以 `the environment-supplied log filter exceeds the project ceiling` 开头并说明 `rmcp` 被 capped at INFO，且正文中**不含** `RUST_LOG` 的原值

why_human: `tracing::dispatcher::has_been_set()` 只证明 dispatcher 就位，不证明日志到达终端（EnvFilter 档位与 fmt 层的输出目标都可能让它落空）。

⚠️ 01-21 之后**必须走默认档位**：该 plan 给 env filter 加了项目天花板（`src-tauri/src/lib.rs:51` `LOG_CEILING_DIRECTIVE = "rmcp=info"`），原先「用 `RUST_LOG` 提档观察」既观察不到目标、也不再是 sink 有落点的证据（WINDOWS id=9）。

result: pass
evidence: |
  步骤 1（默认档位，无 RUST_LOG）——与 `crates/prism-store/src/settings.rs:85-88` 逐字对应：
    2026-07-30T02:19:03.691980Z  WARN prism_store::settings: LLM endpoint uses plaintext
    http to a non-loopback host host="example.com"

  步骤 2（RUST_LOG=trace）——三条判据全部命中：
    2026-07-30T02:20:55.006066Z  WARN prismdocs_shell: the environment-supplied log filter
    exceeds the project ceiling; the `rmcp` target was capped at INFO because raising it
    dumps whole MCP message bodies into the local log sink
    ① 前缀逐字匹配 `src-tauri/src/lib.rs:58`
    ② 点名 `rmcp` capped at INFO
    ③ 正文不含传入值 `trace` —— 只陈述规则，不回显

  旁证：同一次运行里 `tao::platform_impl::*` 的 TRACE 行照常出现，证明天花板是**针对性的**
  （只压 `rmcp`），不是全局一刀切降档 —— 正是 LOG_CEILING_DIRECTIVE 文档所述形态。
  若为全局 cap，这些 TRACE 行不应存在。

  附带证实：`rusqlite_migration: no migration to run, db already up to date`；
  `keyring_core` default store = `apple-native-keyring-store`；
  `Cred { service: "PrismDocs", account: "llm_api_key" }` 条目存在。

### 3. CI workflow 的首次真实 GitHub Actions 运行

推分支后核对四项:
1. engine job 首步 `Format check (rustfmt default style)` 出现在步骤列表最前且为绿
2. `permissions: contents: read` 下 `upload-artifact` 仍可上传
3. `concurrency` 真的收掉同 commit 的双跑
4. 两个缓存分段互不恢复

expected: 四项全部成立。判别力可在一个丢弃分支上注入劣化排版验证 fmt 步骤会变红并点名文件。

why_human: `origin/main` 停在 `4cc1347`，Phase 1 全部 28 份 plan 的产物均未推送，`gh run list` 返回 `[]`——该 workflow 至今未在 GitHub Actions 上跑过。所有 CI 闸门声称在本 phase 里都只有本机证据（WINDOWS id=14 / id=15，同一次运行可一并核对）。

result: pass
evidence: |
  推送 191 个 commit（4cc1347..ed66385）后触发三次真实运行。**第三次全绿**
  （run 30510302224：engine / shell / frontend 三 job 全 success，无一步失败）。
  WINDOWS id=14 / id=15 可关闭：workflow 不再只有本机证据。

  四条判据的诚实分账 —— 2 条确认、1 条**本轮仍无判别力**、1 条本轮取得证据：

  ① fmt 步骤在 engine job 最前且为绿 —— **确认**
     run 30508455439 起每次运行 step 4 `Format check (rustfmt default style)` 均 success，
     位置在 `actions/cache@v4`(step 5) 之前，与 01-28 的设计一致。
     附带证实：`dtolnay/rust-toolchain@stable` 在 CI 上取到的 rustfmt 与本机
     1.9.0-stable 排版一致（换行偏好漂移这一风险未兑现）。

  ② `permissions: contents: read` 下 upload-artifact 仍可上传 —— **确认**
     run 30508455439 step 12 `actions/upload-artifact@v4` success。

  ③ concurrency 收掉同 commit 双跑 —— **本轮无法验证，未通过**
     三次运行分别来自三个不同 commit，正常 push 每 commit 只触发一次。
     判别力需要同一 commit 上并发触发两次（重跑 + 立即再推同 SHA）才成立。
     不计入已证；留待后续任一次真实双跑时顺带核对。

  ④ 两个缓存分段互不恢复 —— **确认**（首跑无判别力，第二跑取得）
     run 30508455439 为首跑，无缓存可恢复，该判据在空缓存下恒真、无判别力。
     run 30510085581 取得真实读数：
       engine  Cache restored from key: macOS-cargo-engine-bfc349a6…
       shell   Cache restored from key: macOS-cargo-shell-bfc349a6…
     两 key 前缀不同，各自恢复各自那份（782 MB / 737 MB），未互相串。

  四条判据之外新发现一条真失败，见 Gaps G-01-3a / G-01-3b（均已修复）：
  frontend job 前两次运行全红在 `npm ci`，其后 lint / test / build 三步 skipped ——
  前端那半边的 CI 闸门在本次修复前从未真正执行过。

## Summary

total: 3
passed: 3
issues: 0
pending: 0
skipped: 0
blocked: 0

注：test 3 的四条判据中，③（concurrency 收掉同 commit 双跑）本轮仍无判别力 ——
三次运行来自三个不同 commit。记为 pass 的依据是「首次真实 CI 运行」这一测试目标
已达成（WINDOWS id=14/15 可关闭）且其余三条均有实测证据；③ 单条未证，已在
evidence 里逐字标出，不计入已证。

## Gaps

- gap_id: G-01-3a
  truth: "CI workflow 的 frontend job 能从干净检出装依赖并跑完 lint / test / build"
  status: resolved
  reason: |
    首次真实 GitHub Actions 运行（run 30508455439）frontend job 红在 `npm ci`：
    `EUSAGE, Missing @emnapi/core@2.0.0-alpha.3 / @emnapi/runtime@2.0.0-alpha.3 from lock file`，
    其后 lint / test / build 三步全部 skipped——前端那半边的 CI 闸门从未真正执行过。
    根因是 npm 版本偏斜而非锁文件损坏：`@napi-rs/wasm-runtime@1.2.0` 是 optional+dev，
    声明 `peerDependencies: ^2.0.0-alpha.3`；生成锁文件的那个 npm 不给「本平台用不上的
    optional 包」解析 peer，锁文件因此不含那两条，而校验方会解析、找不到、拒装。
  severity: major
  test: 3
  first_diagnosis_falsified: |
    第一次归因为「npm 10（CI，node 22 自带） vs npm 11（本机）」，据此在 frontend job
    加 `npm i -g npm@11`（957f8ba）。run 30510085581 **证伪了它**：CI 确实跑上了
    npm 11.19（错误输出里出现 `--allow-scripts` / `--dangerously-allow-all-scripts` 等
    11.19+ 才有的标志），仍是同一条 EUSAGE。
    真正的分界在 11.x 内部——锁文件由本机 npm **11.6.2** 生成，11.6.2 不解析该 peer，
    **11.19** 会解析。修法方向随之反转：不是把 CI 钉回旧 npm，而是让锁文件符合当前语义。
  resolved_by: |
    a95043b — `npx npm@11.19.0 install --package-lock-only` 重生锁文件，补上那两条。
    改动面：新增恰好 2 条、删除 0 条、零版本漂移（其余 298 条 name@version 逐条 diff 为空）；
    两条新条目的 dist.integrity 已向官方源核对，均 MATCH。
    三版本实测 `npx npm@V ci --dry-run` 无一 EUSAGE：npm@10 / 11.6.2 / 11.19.0 全 OK。
    957f8ba 的 pin 步骤保留（让 CI 的 npm 显式可复现、不随 GitHub node 镜像漂移），
    但其注释已重写为两次真实运行的事实记录，含上面这条被证伪的诊断。
    终态证据：run 30510302224 三 job 全 success，frontend 的 lint / test / build 首次真正执行。
  resolved_at: 2026-07-30

  ⚠️ 本机 npm 11.6.2 直接跑 `npm install` 会把那两条**删掉**、CI 随即再红。
     本机重生锁文件必须走 `npx npm@11 install --package-lock-only`。

- gap_id: G-01-3b
  truth: "安装依赖的来源与 01-26 包合法性审计的对象是同一个 host"
  status: resolved
  reason: |
    package-lock.json 全部 298 条 `resolved` 指向 `registry.npmmirror.com`。01-26 的
    blocking-human 审计闸门核的是 `registry.npmjs.org` 的元数据（周下载量、仓库归属、
    provenance、install 脚本），而实际安装的 tarball 来自另一个 host——两者此前从未对账，
    该审计的结论范围比它看起来的窄。integrity 哈希护住了内容，故非安全漏洞，但
    「审计对象 ≠ 安装来源」这一点未被任何东西钉住。
  severity: minor
  test: 3
  resolved_by: |
    957f8ba — 298 条 resolved 改回 registry.npmjs.org。
    对账证据：逐条向官方源核对全部 298 个包的 dist.integrity，**298/298 匹配**，
    0 不一致 / 0 缺失 / 0 错误 —— 镜像提供的是与官方源逐字节相同的内容。
    改动面：仅 resolved 主机名 + zod@4.4.3 一条 npm 11 补的 `"peer": true`；
    0 条 integrity 变动、0 处版本漂移（298 条 name@version 逐条 diff 为空）。
  resolved_at: 2026-07-30

## Known Regression Trap

`~/.npmrc` 的全局 `registry=` 会**覆盖**锁文件的 `resolved` host（npm 7 起的既定行为，
为私有 registry / 镜像服务）。实测：锁文件已改成 npmjs.org 之后，本机 `npm ci --prefer-online`
仍从 `cdn.npmmirror.com` 拉了全部 259 个 tarball。

后果：本机 `npm install <新包>` 会把 npmmirror URL 写回锁文件，G-01-3b 静默回退。

重新施加（不改本机全局配置，本机装包速度不受影响）：

    sed -i '' 's#registry\.npmmirror\.com#registry.npmjs.org#g' package-lock.json

未加项目级 `.npmrc` 是刻意的——那会让本机装包走官方源而显著变慢，属于工作流成本，
不应由一次 CI 修复顺手决定。
