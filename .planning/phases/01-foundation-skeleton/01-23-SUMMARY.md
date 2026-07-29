---
phase: 01-foundation-skeleton
plan: 23
subsystem: settings-page-failure-surface
tags: [frontend, error-handling, prototype-pollution, url-validation, keychain, whitespace, accessibility, gap-closure]
status: complete

requires:
  - "src/lib/ipc.ts 的 `errorCopy` 契约（「无法识别时给通用兜底，而不是 String(err)」）与 `LISTEN_FAILED` 语义"
  - "crates/prism-store/src/settings.rs::validate_base_url —— 端点判定面的权威（scheme / host / userinfo / query / fragment）"
  - "crates/prism-mcp 的 `McpDeps::new` 对 bearer token 的裁剪（01-16）—— 本 plan 在另一条密钥路径上复用同一条推理"
  - "src/pages/DevSmoke.tsx 的 error → alert / ok → status 形状（01-22）"
  - "scripts/check-secrets.sh 的关键词分支（01-14 收紧后：引号串{8,} 或 裸值{16,}）"
provides:
  - "`ERROR_COPY` 建在 `Object.create(null)` 之上 —— 任何查找都够不着 `Object.prototype`，`errorCopy` 的声明类型与运行期行为一致"
  - "`localUrlIssue` 先解析后判 `protocol`/`hostname`/userinfo/query/fragment，与 engine 侧 `validate_base_url` 在十条取样上逐条一致"
  - "API key 在**两端**被同一份归一化处理：`submitKey` 判空与提交用同一份 `trim()` 值；`prism_llm::secrets::set_api_key` 侧防御性裁剪"
  - "设置页两个 useQuery 的显式四态（pending / error / 有值 / 无值）+ 两条 `role=\"alert\"` 通知"
  - "`NoticeLine` 的 ok 分支现在落在 `role=\"status\"` —— 与 DevSmoke.tsx 同形（关闭 WINDOWS #10）"
  - "src/lib/ipc.test.ts 的 errorCopy 表驱动：9 正常短码（阴性对照）+ 8 原型链成员 + 6 非字符串输入"
affects:
  - "后续任何新增 IPC 短码：加进 `ERROR_COPY` 的方式不变（对象字面量作为 `Object.assign` 的第二参），但查找不再有原型链风险"
  - "Phase 4 的 chat client：钥匙串里不会再出现带尾随空白的凭据，401 排查面收窄"
  - "后续任何读取型 query 的渲染：设置页现在是「读失败必须与空状态可区分」的样板"

tech-stack:
  added: []
  patterns:
    - "查找表建在 `Object.create(null)` 之上而不是靠每个查找点记得写 `Object.hasOwn`：前者是机制（第二个查找点加进来也安全），后者是约定"
    - "跨语言判定面对齐用**临时对照测试**取证而不是靠读代码比对：十条输入喂给 Rust 侧 `validate_base_url` 打印结论，跑完即删，`git status` 收尾干净"
    - "URL 判定一律「先解析再看结构」，不做字节级前缀比较——scheme 大小写不敏感这件事在两侧的 parser 里都成立，而 `startsWith` 不成立"
    - "非恒真反证成对出现：一条证明「修复前会红」，另一条证明「偷懒实现（无条件兜底 / 无条件拒绝 / 无条件报错）会让阴性对照红」"

key-files:
  created: []
  modified:
    - src/lib/ipc.ts
    - src/lib/ipc.test.ts
    - src/pages/Settings.tsx
    - src/pages/Settings.test.tsx
    - crates/prism-llm/src/secrets.rs

decisions:
  - "`ERROR_COPY` 选「无原型容器」而不是「`Object.hasOwn` 判定」（plan 给的二选一）：后者要求每一个查找点都记得那样写，那是约定；容器没有原型链则是机制"
  - "原型链取样从 plan 列的 5 个扩到 8 个（补 `isPrototypeOf` / `propertyIsEnumerable` / `toLocaleString`）：它们与列出的五个是同一类，成本为零而覆盖面完整"
  - "`localUrlIssue` 判定后**不**改写提交值——仍把 `urlDraft.trim()` 交给 engine，而不是 `url.href`。归一化是 engine 的事，前端多做一层会让「界面显示的值」与「用户输入的值」悄悄分叉，且不在本 plan 范围内"
  - "两条 query 错误态的测试用 `Error` 对象而不是短码串拒绝：既有九个短码的文案全是**写入**口吻（「密钥没有保存」「写入本地数据库失败」），套到读失败上是半句假话。通用兜底在这里更诚实，且断言「alert 文本等于 errorCopy 产物」的判别力不受影响。不新增读向短码（plan 明令不扩短码表）"
  - "`submitKey` 的局部变量取名 `trimmed` 而不是 `secret`：`check-secrets.sh` 的关键词分支会把 `secret = <16 字符以上裸值>` 判成明文密钥。撞车时改代码不改防线（该脚本文件头的单向约定）"
  - "顺手关掉 WINDOWS #10（Settings.tsx 成功通知无 live region）：该文件在本 plan 的 `files_modified` 里，且 Task 3 的 error → alert 分支正是靠 ok/error 的 region 区分才有判别力——两者是同一件事的两半"

metrics:
  duration: ~9min
  tasks: 3
  files: 5
completed: 2026-07-29
---

# Phase 01 Plan 23: 设置页失败面的四条缺口 Summary

把 `errorCopy` 的查找表从对象字面量搬到 `Object.create(null)` 之上（`errorCopy("toString")` 不再返回函数、不再让设置页整页白掉），把 `localUrlIssue` 从字节级前缀比较改成「先解析再看结构」（`HTTPS://` 不再被误拒），让 API key 在前端与 `prism-llm` 两端走同一份 `trim()`，并给两个 `useQuery` 装上显式 `isError` 分支——钥匙串读失败不再被说成「未配置」，而真的空钥匙串仍然长得像空钥匙串。

## What Was Built

### Task 1 —— `errorCopy` 不再读到 `Object.prototype`（RED `a44bc89` / GREEN `23ac643`）

`ERROR_COPY` 改为 `Object.assign(Object.create(null), { … })`。plan 给的二选一里选无原型容器而不是 `Object.hasOwn` 判定，理由写在源码注释里：后者要求**每一个**查找点都记得那样写（约定），容器没有原型链则对第二个查找点同样成立（机制）。

`errorCopy` 的文档注释补了 plan 要求的那句——`Record<string, string>` 在这里不提供编译期保护：`ERROR_COPY[code]` 的静态类型是 `string`，而在对象字面量上它的运行期值可能是一个函数；这个值一路流进 `setKeyNotice({ text })` 并被 `NoticeLine` 渲染成 `{notice.text}`，React 对函数子节点抛错，设置页整页卸载成空白。

`src/lib/ipc.test.ts` **已存在**（01-01 建的 `devPing` 三条），本 plan 在其后追加 errorCopy 段（见 Deviations 1）。表驱动共 23 条：

| 组 | 条数 | 作用 |
|---|---|---|
| 九个正常短码 | 9 | **阴性对照**——一个「一律返回兜底」的实现会让原型链那批全绿，只有这九条照得出来 |
| `Object.prototype` 成员名 | 8 | plan 列的 5 个 + `isPrototypeOf` / `propertyIsEnumerable` / `toLocaleString` |
| 非字符串输入 | 6 | number / null / undefined / Error / object / array |

每条都先断言 `typeof === "string"` 再断言等于预期文案；末尾一条把三组合起来再扫一遍。

### Task 2 —— 判定面对齐 + 密钥两端裁剪（RED `7e6f2e4` / GREEN `be4f39b`）

`localUrlIssue` 去掉了 `startsWith("http://") / startsWith("https://")` 那一步，改为 `new URL(raw.trim())` 解析后看 `protocol`（已小写化，与 engine 的 `url.scheme()` 同口径）、`hostname`、`username`/`password`、`search`/`hash`。函数头的不变量注释改写成双向陈述：前端放过 engine 拒绝只是多一次 IPC 往返，而前端拒绝 engine 会接受会告诉用户一句他没法照做的话。

`submitKey` 的判空与提交改用同一份 `secretDraft.trim()`；`prism_llm::secrets::set_api_key` 里 `set_password(secret.trim())`，文档注释点名它与 `McpDeps::new`（01-16）的对称关系与「防御性第二道」的定位（绕过界面直接 invoke 走的也是这个函数）。

### Task 3 —— 两个 query 的显式错误态（RED `ab58a0c` / GREEN `2894c90`）

密钥状态与端点当前值都改成四态（pending / error / 有值 / 无值），两处各配一条复用既有 `NoticeLine` 的 `role="alert"`，文案走 `errorCopy(query.error)`。渲染点旁的注释写明这条区分为什么重要：用户对「未配置」的反应是重新输入密钥，那次保存也会失败（钥匙串本来就不可用），且他仍不知道原因。

### 附带 —— 关闭 WINDOWS #10（`c01fd44`）

`NoticeLine` 的 ok 分支加 `role="status"`，与 `DevSmoke.tsx`（01-22）同形。这条不是顺手做的装饰：Task 3 的 error → alert 断言之所以有判别力，正是因为 ok 落在**另一个** region 上——两者是同一件事的两半。配一对正反断言（成功落在 status 且 alert 为 null）。

## Verification

```
npm run test -- --run
  → Test Files 7 passed (7) / Tests 75 passed (75)
npx tsc --noEmit
  → 0 error
cargo test -p prism-llm
  → 10 passed / 0 failed / 1 ignored（真实钥匙串那条仍需手动跑）
cargo test --workspace
  → 全部 test result: ok，0 failed
cargo clippy -p prism-llm --all-targets -- -D warnings
  → Finished，0 warning
bash scripts/check-secrets.sh all
  → OK: pattern discriminates (19 positive / 10 negative samples)
  → OK: no plaintext secret in 114 version-controlled files（exit 0）
git status --porcelain
  → 仅两条本 plan 之前就存在的 .planning/research/.cache 未跟踪文件；临时对照测试已删净
```

### 跨语言判定面对照表（十条取样）

临时 Rust 测试 `crates/prism-store/src/settings.rs::tmp_crosscheck`（跑完即删，未提交）把与前端表驱动**同一批**输入喂给 `validate_base_url`：

| 输入 | 前端 `localUrlIssue` | engine `validate_base_url` | 一致 |
|---|---|---|---|
| `https://api.example.com/v1` | null（接受） | accepted | ✓ |
| `HTTPS://api.example.com/v1` | null（接受） | accepted | ✓ |
| `HTTP://localhost:8080` | null（接受） | accepted | ✓ |
| `ftp://api.example.com` | `invalid_url` | rejected: scheme must be one of ["http", "https"] | ✓ |
| `not a url` | `invalid_url` | rejected: could not be parsed as a URL: relative URL without a base | ✓ |
| `https://` | `invalid_url` | rejected: could not be parsed as a URL: empty host | ✓ |
| `https://prism-test-user:prism-test-value@api.vendor.com/v1` | `invalid_url_credentials` | rejected: must not carry credentials in the userinfo component | ✓ |
| `https://prism-test-user@api.vendor.com/v1` | `invalid_url_credentials` | rejected: must not carry credentials in the userinfo component | ✓ |
| `https://api.vendor.com/v1?deployment=prism-test-value` | `invalid_url_credentials` | rejected: must not carry a query string or fragment | ✓ |
| `https://api.vendor.com/v1#prism-test-value` | `invalid_url_credentials` | rejected: must not carry a query string or fragment | ✓ |

十条逐条一致。修复前 `HTTPS://` 与 `HTTP://` 两行是分歧行（前端拒绝、engine 接受）。

### 八条非恒真反证（全部实跑）

**Task 1-A —— `ERROR_COPY` 改回对象字面量**

```
× returns the generic fallback string for the Object.prototype member toString
× ... constructor  × ... valueOf  × ... hasOwnProperty  × ... __proto__
× ... isPrototypeOf  × ... propertyIsEnumerable  × ... toLocaleString
× never returns a non-string, for any sampled input
  AssertionError: expected 'function' to be 'string'
  AssertionError: expected 'object' to be 'string'   ← __proto__ 那条
Tests  9 failed | 19 passed (28)
```

**Task 1-B —— `errorCopy` 无条件返回兜底**（阴性对照的判别力）

```
× translates the invalid_url short code
× translates the invalid_url_credentials short code
× translates the invalid_setting short code
× translates the store_error short code
× translates the secret_error short code
× translates the task_failed short code
× translates the channel_send_failed short code
× translates the engine_error short code
× translates the listen_failed short code
Tests  9 failed | 19 passed (28)
```

九条正常码红、八条原型链断言与合并断言全绿——这证明本 Task 的断言不是「只要是字符串就算过」。

**Task 2-A —— `localUrlIssue` 改回 `startsWith` 判定**

```
× judges 'HTTPS://api.example.com/v1' as 'accepted', matching the engine
× judges 'HTTP://localhost:8080' as 'accepted', matching the engine
Tests  2 failed | 17 passed (19)
```

恰好只有两条大写 scheme 变红，其余八条判定不受影响——反证锁定的正是被修的那件事。

**Task 2-B —— `localUrlIssue` 无条件返回 null**

```
× rejects a credential-bearing endpoint before it ever reaches the engine（既有测试）
× judges 'ftp://api.example.com' as 'invalid_url'
× judges 'not a url' as 'invalid_url'
× judges 'https://' as 'invalid_url'
× judges 'https://prism-test-user:prism-test-va…' as 'invalid_url_credentials'
× judges 'https://prism-test-user@api.vendor.co…' as 'invalid_url_credentials'
× judges 'https://api.vendor.com/v1?deployment=…' as 'invalid_url_credentials'
× judges 'https://api.vendor.com/v1#prism-test-…' as 'invalid_url_credentials'
Tests  8 failed | 11 passed (19)
```

七条拒绝断言（+1 条既有的凭据测试）红，而三条接受断言——含两条大写 scheme——仍绿。

**Task 2-C —— `submitKey` 改回 `mutate(secretDraft)`**

```
× trims surrounding whitespace off the key before it reaches the keychain
Tests  1 failed | 18 passed (19)
```

**Task 2-D —— `set_api_key` 去掉 `.trim()`**

```
test secrets::tests::set_api_key_trims_surrounding_whitespace ... FAILED
  assertion `left == right` failed: 存进钥匙串的密钥仍带首尾空白
    left: Some("  prism-test-secret-value\n")
   right: Some("prism-test-secret-value")
test result: FAILED. 9 passed; 1 failed; 1 ignored
```

（这一条是 RED 阶段的实测输出，与 plan 要求的「改回去 → 变红」等价。）

**Task 3-A —— 去掉两个 `isError` 分支**

```
× says 读取失败 rather than 未配置 when the key status query is rejected
× does not claim 未设置 when the endpoint query is rejected
Tests  2 failed | 20 passed (22)
```

**Task 3-B —— 端点渲染改成无条件「读取失败」**（阴性对照的判别力）

```
× still says （未设置） when the endpoint query resolves to null
Tests  1 failed | 21 passed (22)
```

阴性对照红而被拒断言仍绿——两条断言合起来看住的是「读失败与空状态可区分」，不是「有没有出现读取失败四个字」。

**附带 —— `NoticeLine` 的 ok 分支改成 `role="alert"`**

```
× announces a successful save in a status region, not an alert region
× judges 'https://api.example.com/v1' as 'accepted', matching the engine
× judges 'HTTPS://api.example.com/v1' as 'accepted', matching the engine
× judges 'HTTP://localhost:8080' as 'accepted', matching the engine
Tests  4 failed | 19 passed (23)
```

三条 accepted 断言里的 `queryByRole("alert") === null` 一并变红——它们本来就在替这条区分把关，只是此前没人指出来。

## Deviations from Plan

### 1. [事实修正] `src/lib/ipc.test.ts` 不是新文件，是追加

- **发生在:** Task 1
- **情况:** plan 的 `<action>` 与 `<artifacts_this_phase_produces>` 都写「新建 `src/lib/ipc.test.ts`」，但该文件在 01-01 就已存在（`devPing` 的三条断言）。
- **处理:** 在既有内容之后追加 errorCopy 段，未改动既有三条。SUMMARY 的 `key-files.created` 相应为空。
- **对 must_haves 的影响:** 无。

### 2. [Rule 3 - 阻塞] `check-secrets.sh` 把 `const secret = secretDraft.trim();` 判成明文密钥

- **发生在:** Task 2，`bash scripts/check-secrets.sh all` 退出 1
- **情况:** 01-14 收紧后的关键词分支认「`secret` = 裸值{16,}」，而 `secretDraft.trim();` 恰好 19 字符。
- **处理:** 局部变量改名 `trimmed`，并在原处留注释说明撞车原因与「改代码不改防线」的单向约定（该脚本文件头已写明）。没有为迁就这一行去放宽扫描器。
- **提交:** `be4f39b`

### 3. [取舍] 两条 query 错误态的测试用 `Error` 对象拒绝，而不是既有短码串

- **发生在:** Task 3
- **情况:** plan 的验收项要求「alert 文本等于 `errorCopy` 的产物（通用兜底或已知短码文案）」，两者都合规。但既有九个短码的文案全是**写入**口吻（`secret_error` = 「系统钥匙串当前不可用，密钥没有保存。」、`store_error` = 「写入本地数据库失败，请重试。」），套到一次**读取**失败上后半句是假的。
- **处理:** 用 `new Error("keychain is locked")` / `new Error("db is unreadable")` 拒绝，断言 alert 文本**逐字等于**通用兜底且不含原始细节——判别力不减（不泄漏 `String(error)` 这条反而被直接验证），且不引入一句半假的文案。未新增读向短码（plan 明令不扩短码表）。
- **对 must_haves 的影响:** 无。「`apiKeyStatus` 查询被拒时显示「读取失败」而不是「未配置」」逐字成立。

### 4. [范围外补齐] 关闭 WINDOWS #10 —— Settings.tsx 成功通知加 `role="status"`

- **发生在:** Task 3 之后
- **情况:** 01-22 登记的开放窗口：`NoticeLine` 的 ok 分支只有颜色无 `role`，读屏对「已保存」完全静默。该文件在本 plan 的 `files_modified` 里，且 01-22 的备注明确写「01-23 正在改它」。
- **处理:** 加 `role="status"`（与 `DevSmoke.tsx` 同形）+ 一对正反断言。这与 Task 3 是同一件事的两半：error → alert 的判别力来自 ok 落在另一个 region 上。
- **提交:** `c01fd44` —— WINDOWS #10 标记为 fixed。

### 5. [状态一致性] INFRA-03 未勾选 —— `requirements.mark-complete` 的写入已回退

- **发生在:** 收尾的 state 更新
- **情况:** plan frontmatter 的 `requirements: [INFRA-03]` 触发 `requirements.mark-complete INFRA-03`，把 `REQUIREMENTS.md` 里那条改成了 `[x]` / `Complete`。但 WINDOWS #7 已明确登记「INFRA-03 未勾：需求文本『支持 Anthropic/OpenAI 兼容端点』半句需 Phase 4 chat client；扫描器与写入侧两半已关闭」。
- **处理:** `git checkout -- .planning/REQUIREMENTS.md` 回退，INFRA-03 保持 `[ ]` / `Pending`。本 plan 关的是密钥入口那一半的四条缺口，不是那半句需求。WINDOWS #7 保持 open。

### 6. [范围克制] 十条 URL 判定后仍提交 `urlDraft.trim()` 而非 `url.href`

- **发生在:** Task 2
- **情况:** `HTTPS://api.example.com/v1` 通过判定后，存进 `settings` 表的仍是大写 scheme 的原串（engine 侧 `set_setting` 存的是传入 value，不是解析后的 `Url`）。
- **处理:** 保持原样。归一化写回是 engine 的职责，前端多做一层会让「界面显示的值」与「用户输入的值」悄悄分叉，且不在本 plan 的 `<behavior>` 里。已在 decisions 记录，供后续 plan 判断是否要在 engine 侧规范化写入值。

## Deferred Issues

- **INFRA-03 concurrency 探针 —— 显式 flagged assumption（未覆盖）。** 原文：*「`keyring_core::set_default_store` 是进程全局（01-RESEARCH.md § Pitfall 1），并发写同一 account 的行为未被任何测试断言；Phase 4 接上真实 chat client 时需要一条覆盖它的测试。本 plan 不新增。」* 理由：Phase 1 没有并发密钥写入路径，且 `secrets.rs` 的既有测试全部 `#[serial]`——串行化本身就是在回避这个问题，不是在回答它。**需写进 `.planning/STATE.md` 的 Blockers/Concerns：Phase 4 接上真实 chat client 时补一条并发写同一 keychain account 的测试。**
- INFRA-03 idempotency 探针 —— **已覆盖**，证据位置：`crates/prism-llm/src/secrets.rs::set_and_delete_are_idempotent`（重复写入不产生第二条条目、重复删除是 no-op）与 `crates/prism-store/src/settings.rs::settings_roundtrip`（`set_setting` 是 upsert，覆盖写不产生第二行）。本 plan 的裁剪改动不破坏其中任何一条——`set_api_key_trims_surrounding_whitespace` 与既有幂等测试在同一次 `cargo test -p prism-llm` 里同时绿。
- 仓库仍存在 rustfmt 漂移且无 CI fmt 门（`deferred-items.md` 已登记，01-28 正在关闭）。本 plan 按约定**未**运行 `cargo fmt`，新增 Rust 代码按 rustfmt 口径手写。
- `secrets::tests::roundtrip_with_real_keychain` 仍是 `#[ignore]`（触碰真实登录钥匙串），未在本次跑动中执行——与既有状态一致，非本 plan 引入。

## Known Stubs

无。本 plan 未引入任何占位实现、硬编码空值或未接数据源的组件。

## Threat Flags

无新增。`threat_model` 里五条 `mitigate` 全部落地：

| Threat ID | 落点 |
|---|---|
| T-01G-31 | `ERROR_COPY` 无原型容器 + 反证 1-A/1-B |
| T-01G-32 | 每条 errorCopy 断言含 `typeof === "string"`（23 条） |
| T-01G-33 | `localUrlIssue` 先解析后判 protocol + 十条跨语言对照表 |
| T-01G-34 | `submitKey` 与 `set_api_key` 两端裁剪 + 反证 2-C/2-D |
| T-01G-35 | 两个 query 的 `isError` 分支 + `role="alert"` + 阴性对照 3-B |

T-01-SC（依赖安装）保持 accept —— 本 plan 未新增任何依赖。

## Self-Check: PASSED

五个源文件与本 SUMMARY 均在磁盘上；六个提交（`a44bc89` / `23ac643` / `7e6f2e4` / `be4f39b` / `ab58a0c` / `2894c90` / `c01fd44`）均在 git 历史中。
