---
phase: 01-foundation-skeleton
reviewed: 2026-07-30T00:20:58Z
depth: standard
round: 3
scope: 全 phase 复评（diff base b71023a^，HEAD 1976a44）
files_reviewed: 70
files_reviewed_list:
  - .github/workflows/ci.yml
  - crates/prism-anchor/src/lib.rs
  - crates/prism-cli/src/main.rs
  - crates/prism-engine/Cargo.toml
  - crates/prism-engine/src/bus.rs
  - crates/prism-engine/src/error.rs
  - crates/prism-engine/src/facade.rs
  - crates/prism-engine/src/lib.rs
  - crates/prism-engine/src/services.rs
  - crates/prism-engine/tests/facade.rs
  - crates/prism-fs/src/lib.rs
  - crates/prism-llm/Cargo.toml
  - crates/prism-llm/src/lib.rs
  - crates/prism-llm/src/secrets.rs
  - crates/prism-mcp/Cargo.toml
  - crates/prism-mcp/src/deps.rs
  - crates/prism-mcp/src/handler.rs
  - crates/prism-mcp/src/lib.rs
  - crates/prism-mcp/src/middleware.rs
  - crates/prism-mcp/src/server.rs
  - crates/prism-mcp/tests/middleware_gate.rs
  - crates/prism-mcp/tests/trait_injection.rs
  - crates/prism-parse/src/lib.rs
  - crates/prism-store/Cargo.toml
  - crates/prism-store/migrations/001_schema_v1.sql
  - crates/prism-store/src/error.rs
  - crates/prism-store/src/lib.rs
  - crates/prism-store/src/migrations.rs
  - crates/prism-store/src/open.rs
  - crates/prism-store/src/search.rs
  - crates/prism-store/src/seed.rs
  - crates/prism-store/src/settings.rs
  - crates/prism-store/tests/concurrency.rs
  - crates/prism-store/tests/fts_cjk.rs
  - crates/prism-types/Cargo.toml
  - crates/prism-types/src/dto.rs
  - crates/prism-types/src/event.rs
  - crates/prism-types/src/lib.rs
  - crates/prism-types/src/service.rs
  - crates/prism-types/tests/contract.rs
  - docs/keychain-naming.md
  - eslint.config.js
  - justfile
  - package.json
  - rustfmt.toml
  - scripts/check-deps.sh
  - scripts/check-secrets.sh
  - src-tauri/Cargo.toml
  - src-tauri/capabilities/default.json
  - src-tauri/src/bus_adapter.rs
  - src-tauri/src/commands.rs
  - src-tauri/src/lib.rs
  - src-tauri/src/smoke.rs
  - src-tauri/tauri.conf.json
  - src-tauri/tests/ipc.rs
  - src/App.test.tsx
  - src/App.tsx
  - src/lib/capabilities.test.ts
  - src/lib/ipc.test.ts
  - src/lib/ipc.ts
  - src/lib/queryClient.ts
  - src/lib/tauri-security.test.ts
  - src/lib/useEngineInvalidation.test.ts
  - src/lib/useEngineInvalidation.ts
  - src/main.tsx
  - src/pages/DevSmoke.test.tsx
  - src/pages/DevSmoke.tsx
  - src/pages/Settings.test.tsx
  - src/pages/Settings.tsx
  - tsconfig.json
findings:
  critical: 0
  warning: 6
  info: 5
  total: 11
status: issues_found
---

# Phase 01 第三轮：Code Review Report

**Reviewed:** 2026-07-30T00:20:58Z
**Depth:** standard
**Files Reviewed:** 70
**Status:** issues_found

## Summary

这是 phase 01 的第三轮评审。我先读了 `01-REVIEW.md`（第二轮）与 `01-REVIEW-prior.md`（第一轮），
再逐文件读了 70 个源文件，**没有把任何一条前两轮的结论当作已关闭来接受**——凡是我在本轮
声称「已关闭」的，都是自己重新推过一遍的。

**前两轮findings的复核结论（逐条自查，不是复述 SUMMARY）：**

- 第二轮 CR-01（`check-secrets.sh` 裸值不可见）**已关闭**。我把 `scripts/check-secrets.sh:101`
  的 `$PATTERN` 原样抽出来重跑了那张 MISSED 清单：`ANTHROPIC_API_KEY=…`、`MCP_BEARER_TOKEN=<32 hex>`、
  `//registry.npmjs.org/:_authToken=…` 现在全部 CAUGHT。隔离样本（`check-secrets.sh:186-187`）
  确实只可能经关键词分支的裸值那一半命中——我逐条验过它们不含任何供应商前缀。
  **但同一个失效形状在另一个键名上仍然开着**，见 WR-04。
- 第二轮 WR-01（CSP 断言是 denylist）**已关闭且强度显著超出建议**：`tauri-security.test.ts:114-124`
  的 ④″ 指令名集合精确相等，把「换一个指令名重新开面」这条 denylist 与逐指令相等都盖不住的
  路径也钉住了。
- 第二轮 WR-03（403 契约在实发路由上不成立）**已关闭**。`host_of` 的一致性闸门
  （`middleware.rs:106-110`）是正确的选择——我核对了 rmcp 2.2 `tower.rs` 的
  `normalize_host` / `parse_host_header` / `parse_origin_value`，`sdk_normalize_host`
  是逐字照抄。我另外沿 **Origin** 方向重新推了一遍两侧口径（本层 `split_once("://")` + `Authority`
  vs SDK 的 `http::Uri`）：`user@host`、`host?x`、`host/x`、`null`、大写 scheme、超范围端口
  六种形态两侧一致，Origin 侧没有残留的层次 oracle。
- 第二轮 WR-05 / WR-06 / WR-07（`constant_time_eq` 死分支、`McpDeps::new` 归一化、scheme
  大小写）**均已关闭**。我重跑了 `the_comparison_is_not_a_plain_equality` 的切片逻辑：
  `split("fn constant_time_eq").nth(1)` 会先命中定义处而不是同前缀的测试函数名，
  `find("\n}\n")` 收在函数的零缩进右括号上，三条断言的落点都真实。
- 第二轮 WR-04 / WR-08 / WR-09 / WR-10 / WR-11 与 IN-01..IN-04 **均已关闭**，各自的
  非恒真反证我都看过落点。

**本轮的新发现集中在同一个主题的两个方向上，两个都是「已经修过一次的缺陷，在它的兄弟身上没修」：**

1. `record_receipt` 把外部 agent 控制的 `status` 收进了受控取值集合（上轮 WR-13 的修复），
   却把**同一行日志里的另一个外部字段 `comment_id`** 留在只判空的状态（WR-01）。
2. 「用 `trim()` 判定、存原值」这个形状在本轮被修了两次（`McpDeps::new` / `set_api_key`），
   第三处 `prism_store::settings::set_setting` 原样留着（WR-03）。

另外三条是**闸门覆盖面**的问题：前后端 URL 判定在 `?` / `#` 空串上确实分叉且错误文案自相矛盾
（WR-02，已用 `url` crate 与 node 双侧实测确认）；`Authorization: Bearer <token>` ——
本项目自己的第二个密钥、也是 Phase 6 `prismdocs-helper headers` 的输出形态——扫描器在三种
现实提交形态下全部看不见（WR-04，已实测）；`useEngineInvalidation` 的 listen 失败分支与
`App.tsx` 的告警条**一条测试都没有**（WR-05）。

**没有 BLOCKER。** 这不是客气：Phase 1 的发布形态里 MCP loopback 服务**根本没有被启动**
（全仓库只有测试调用 `serve_loopback`），`record_receipt` 没有任何 MCP 工具通到它
（`handler.rs` 只读 `deps.feedback`，`deps.comments` 是死字段），四条 `dev_*` 命令已在
`#[cfg(not(debug_assertions))]` 分支外，CSP 与 capability 都是收紧且被精确相等钉住的。
我另外做了两次独立取证：全仓库 bearer 形态与 ≥32 位 hex/base64 字面量的扫描**零命中**，
因此 WR-04 是证据链问题而不是活的泄漏。

---

## Narrative Findings (AI reviewer)

### Warnings

#### WR-01: `record_receipt` 把一个未受约束的外部字段 `comment_id` 直接写进日志——上轮 WR-13 只修了它旁边的 `status`

**Classification:** WARNING
**File:** `crates/prism-engine/src/services.rs:72-91`

**Issue:**

```rust
fn record_receipt(&self, receipt: Receipt) -> Result<(), ServiceError> {
    if receipt.comment_id.trim().is_empty() { ... }          // ← 只判空
    if !RECEIPT_STATUSES.contains(&receipt.status.as_str()) { ... }  // ← 受控取值集合
    tracing::info!(
        comment_id = %receipt.comment_id,   // ← 无界、未校验的外部字符串
        status = %receipt.status,
        "recorded an agent receipt"
    );
```

`Receipt` 的两个字段都是从 MCP 线上直接反序列化的 `String`（`prism-types/src/dto.rs:19-24`），
两个都完全不可信。本轮的 01-20 给 `status` 加了 `RECEIPT_STATUSES` 受控集合，理由写在
文件里：*「一个不受约束的外部字段可以把整段用户文档、或含嵌入换行的伪造日志行写进本地
日志文件（T-01G-18 / T-01G-19）」*（`services.rs:26-31`）。

**这句话逐字适用于 `comment_id`，而 `comment_id` 才是这行日志里第一个被 `%` 展开的字段。**
`record_receipt_rejects_a_long_multiline_status`（`services.rs:228`）构造的那个
`"applied\n" + "user document body ".repeat(1024) + "\nINFO forged log line: ..."` 攻击载荷，
原样换到 `comment_id` 上就会**全部落进日志**——没有任何断言会红。同一个函数、同一行日志、
同一条推理，一个字段修了，另一个没修。

拦截它的东西目前只有一条：`handler.rs` 没有任何工具调用 `record_receipt`，`McpDeps.comments`
是 Phase 1 的死字段（见 IN-05）。也就是说这条路径在发布形态里**不可达**——所以是 WARNING
而不是 BLOCKER。但 Phase 6 接上评论回流工具的那一刻它就活了，而届时没有任何东西会提醒。

顺带：`the_service_impls_contain_no_await`（`services.rs:284-293`）用源码序断言钉住了
「status 校验必须排在 `tracing::info!` 之前」。`comment_id` 没有对应的校验，因此也没有
对应的顺序哨兵——缺口在结构上是对称的。

**Fix:**

```rust
/// comment_id 是 ULID（Phase 5 的 COMMENT-03 定形）。在此之前，最低限度是：
/// 长度上界 + 拒绝控制字符——后者挡的是伪造日志行，前者挡的是整段文档。
const MAX_COMMENT_ID_LEN: usize = 64;

fn record_receipt(&self, receipt: Receipt) -> Result<(), ServiceError> {
    let comment_id = receipt.comment_id.trim();
    if comment_id.is_empty()
        || comment_id.len() > MAX_COMMENT_ID_LEN
        || comment_id.chars().any(|c| c.is_control())
    {
        // 只陈述规则，不回显值（与 status 那条同一口径）。
        return Err(ServiceError::Invalid(
            "comment id is not a well-formed identifier".into(),
        ));
    }
    ...
}
```

并把 `record_receipt_rejects_a_long_multiline_status` 复制成 `..._comment_id` 版本
（同一个 forged 载荷换字段），再给 `the_service_impls_contain_no_await` 加一条
`guard_comment_id < log` 的源码序断言——否则「校验写在日志之后」的实现照样会绿。

---

#### WR-02: `?` / `#` 空串上前后端判定分叉，而 `map_err` 把四条不同的 `InvalidUrl` 理由压成同一个短码，用户拿到一句自相矛盾的话

**Classification:** WARNING
**File:** `src/pages/Settings.tsx:43`（前端判定）、`crates/prism-store/src/settings.rs:76-80`（engine 判定）、`src-tauri/src/commands.rs:38`（短码映射）、`src/lib/ipc.ts:57`（文案）

**Issue:** 两个独立的缺陷叠在一起，结果比任何一个单独看都糟。

**(a) 判定分叉。** `Settings.tsx` 的文件头注释把「逐项对齐」写成契约，并说明反方向分歧
（前端拒绝、engine 接受）代价更高。实测两侧在 `?` / `#` **空串**上不一致：

```
# rust url 2.5.8（engine 侧）
https://api.example.com/v1?     query=Some("")   → validate_base_url 拒绝
https://api.example.com/v1#     frag =Some("")   → validate_base_url 拒绝

# WHATWG URL（node，前端侧）
"https://api.example.com/v1?"   search=""  hash=""  → localUrlIssue 返回 null（放行）
"https://api.example.com/v1#"   search=""  hash=""  → localUrlIssue 返回 null（放行）
```

WHATWG 规范明确规定 `search` / `hash` 在 query/fragment 为 null **或空串**时都返回 `""`，
而 `url` crate 的 `query()`（`url-2.5.8/src/lib.rs:1471`）看的是 `query_start` 是否存在。
`Settings.test.tsx` 的 `URL_CASES` 表覆盖了 `?deployment=…` 与 `#…`，**没有覆盖裸 `?` / `#`**
——恰好是两侧唯一分叉的那个边界。

**(b) 短码坍缩。** `map_err` 把 `StoreError::InvalidUrl(_)` 全部映射成 `"invalid_url"`，
而 `ERROR_COPY.invalid_url` 是「链接必须以 http:// 或 https:// 开头，并带有主机名。」。
于是 engine 侧**四条**规则（scheme / host / userinfo / query+fragment）的拒绝共用一句只描述
前两条的文案。`invalid_url_credentials` 这个码 **engine 永远产生不出来**，它只由前端自造。

把 (a) 和 (b) 合起来：用户输入 `https://api.example.com/v1?` —— 一个确实以 `https://` 开头、
确实带主机名的链接 —— 前端放行、engine 拒绝，界面告诉他「链接必须以 http:// 或 https://
开头，并带有主机名」。这正是 `Settings.tsx:23-25` 写下的那句「他没有任何办法照做」，
只是方向反了过来。

**Fix:** 两半都要补。

前端与 engine 对齐（`Settings.tsx:43`）：

```ts
// WHATWG 的 search/hash 在「空 query」与「无 query」上都返回 ""，而 engine 侧的
// url crate 区分二者。判定改看原串里有没有分隔符，两侧口径才真的一致。
const afterOrigin = raw.trim().slice(raw.trim().indexOf(url.host) + url.host.length);
if (afterOrigin.includes("?") || afterOrigin.includes("#")) return "invalid_url_credentials";
```

（或更简单：engine 侧把 `url.query().is_some_and(|q| !q.is_empty())` 放宽为忽略空 query
——但那会让 `?` 原样落进 `settings.value`，不推荐。）

短码分化（`commands.rs:38`）：给 `StoreError` 加一个 `InvalidUrlCredentials` 变体
（或在 `map_err` 里按 `InvalidUrl` 的载荷文本分流），映射到已存在的
`"invalid_url_credentials"`，前端文案一个字都不用改。

`URL_CASES` 补两行：`{ input: "https://api.example.com/v1?", verdict: ... }` 与 `#` 版本。

---

#### WR-03: `set_setting` 用 `value.trim()` 校验、却把**原值**写进表——本轮修过两次的同一个形状，第三处原样留着

**Classification:** WARNING
**File:** `crates/prism-store/src/settings.rs:52`（校验）、`crates/prism-store/src/settings.rs:117`（分派）、`crates/prism-store/src/settings.rs:125`（写入）

**Issue:**

```rust
pub fn validate_base_url(raw: &str) -> Result<Url, StoreError> {
    let url = Url::parse(raw.trim())          // ← 校验的是 trim 后的字节
    ...
pub fn set_setting(tx: &Transaction, key: &str, value: &str) -> Result<(), StoreError> {
    ...
    if key == SETTING_BASE_URL { validate_base_url(value)?; }
    ...
    tx.execute(SQL_UPSERT, (key, value, now))?;   // ← 存的是原值
```

`prism-mcp/src/deps.rs:51-59` 把这个形状写成了本仓库的一条明文纪律：*「归一化与判空必须落在
**同一份字节**上」*；`prism-llm/src/secrets.rs:45-49` 又为同一条纪律加了第二道防御。
`set_setting` 是这条纪律的第三个落点，而它没有被改。

模块头自己给出的理由说明了为什么这次不能只靠前端：*「放在调用方就等于『每个调用点都记得』，
那是约定；放在这里它才是机制——绕过界面直接 invoke 也改变不了结果」*（`settings.rs:107-110`）。
今天 UI 路径确实先 `trim()` 了（`Settings.tsx:121`），所以这是约定在替机制干活——正是这段
注释要否定的状态。一次 `invoke("set_base_url", { url: " https://x/v1 \n" })` 会让
`settings.value` 里躺一个带首尾空白的值，`get_setting` 原样读回、`Settings.tsx:204` 原样渲染，
而它与被校验的那个字符串不是同一份字节。

**同一处还有第二个不对称**：分派条件 `key == SETTING_BASE_URL` 是**逐字节**比较，而紧挨着
它的 `is_secret_like_key` 是 `to_lowercase()` 后比较。也就是说一个 `" llm.base_url"` 或
`"LLM.BASE_URL"` 的键会**完全跳过** `validate_base_url`，同时仍受密钥键名守卫约束。
Phase 1 没有通用的 `set_setting` IPC 命令，所以这一半目前不可达；但两条守卫在同一个函数里
用两种口径判定同一个 `key`，本身就是下一个调用点的陷阱。

**Fix:**

```rust
pub fn set_setting(tx: &Transaction, key: &str, value: &str) -> Result<(), StoreError> {
    // 归一化一次，此后校验与写入看的是同一份字节（同 McpDeps::new / set_api_key）。
    let key = key.trim();
    let value = value.trim();

    if is_secret_like_key(key) { ... }
    if key.eq_ignore_ascii_case(SETTING_BASE_URL) {
        validate_base_url(value)?;
    }
    tx.execute(SQL_UPSERT, (key, value, now))?;
    Ok(())
}
```

`settings_base_url_validation` 加一条：写 `"  https://api.example.com/v1  "` 之后
`get_setting` 读回的必须**逐字等于** `"https://api.example.com/v1"`；再加一条阴性对照，
证明裁剪只碰首尾（值内部的空白原样保留），否则「把所有空白都删掉」的实现也会绿。

---

#### WR-04: `check-secrets.sh` 看不见 `Authorization: Bearer <token>`——本项目自己的第二个密钥、也是 Phase 6 helper 的输出形态；而 selftest 里那条样本读起来像是覆盖了它

**Classification:** WARNING
**File:** `scripts/check-secrets.sh:101`（PATTERN）、`scripts/check-secrets.sh:157`（误导性阳性样本）

**Issue:** 关键词 alternation 是 `(api[_-]?key|secret|token|password)`。**`bearer` 不在里面。**
把 `$PATTERN` 从第 101 行原样抽出来实测：

```
MISSED : Authorization: Bearer 7f3a9c1e5b2d8f4a6c0e9b7d3f1a5c8e
MISSED : "Authorization": "Bearer 7f3a9c1e5b2d8f4a6c0e9b7d3f1a5c8e"
MISSED : curl -H 'Authorization: Bearer 7f3a9c1e5b2d8f4a6c0e9b7d3f1a5c8e' http://127.0.0.1:1234/mcp
MISSED : prismdocs_bearer: 7f3a9c1e5b2d8f4a6c0e9b7d3f1a5c8e
CAUGHT : MCP_BEARER_TOKEN=7f3a9c1e5b2d8f4a6c0e9b7d3f1a5c8e
```

只有**赋值形态**能命中（因为账户名 `mcp_bearer_token` 恰好含 `token`）。而这个 token 在现实里
最常出现的形态不是赋值——是 **HTTP 头**：D-07 规划的 `prismdocs-helper headers` 子命令
（`crates/prism-cli/src/main.rs:25`）的全部作用就是把 `Authorization: Bearer <32 hex>` 打到
标准输出，供用户粘进 agent 的配置文件（`.mcp.json` / `.claude/settings.json`）或 runbook ——
两者都是会被提交的文件。

**真正让这条值得报的是 selftest。** 阳性样本第 4 条是

```bash
positive+=("Authorization: ${q}Bearer ${sk}${dash}ant-api03-abcdefghijklmnop${q}")
```

它绿，于是**读起来像**「Authorization 头这一形态覆盖了」。并没有：它经 `sk-` 前缀分支命中，
把 `sk-ant-…` 换成 32 位 hex（也就是本项目自己的 token 形态）立刻 MISSED。这与第二轮 CR-01
点名的那条 `ANTHROPIC_API_KEY=${sk}${dash}ant-…` **是同一个误导形状**，换了一个键名活了下来
——CR-01 修的是「裸值那一半」，没有修「样本经前缀分支兜底」这个成因。

我做了独立取证，确认**当前没有东西藏在这个洞里**：
`git grep -niE '(bearer|authorization)[^A-Za-z0-9]{0,4}[:=][[:space:]]*[A-Za-z0-9_./+~-]{16,}'`
零命中，≥32 位 hex/base64 字面量扫描也只剩 blake3/lock 噪声。所以这是证据链问题，不是活的泄漏
——但成功标准 4 的自动化证据对这个供应商形态不成立。

顺带（同一处修复顺手做掉）：`hf_`（HuggingFace）、`gsk_`（Groq）、`xai-`、
`-----BEGIN … PRIVATE KEY-----` 全部未覆盖，实测均 MISSED。

**Fix:** 把 `bearer` 加进关键词 alternation，并补一条**只可能经它命中**的隔离样本：

```bash
PATTERN="…|(api[_-]?key|secret|token|password|bearer)[[:space:]]*[=:][[:space:]]*(${QUOTE}${NOT_QUOTE}{8,}|${BARE}{16,})"
```

```bash
# 必须由 bearer 关键词命中：值里没有任何供应商前缀，键名里也没有 token/secret/key。
# 删掉 alternation 里的 `bearer` → 这两条立刻变红，其余样本一条都不受影响。
positive+=("Authorization: Bearer 7f3a9c1e5b2d8f4a6c0e9b7d3f1a5c8e")
positive+=("prismdocs_${bear}: 7f3a9c1e5b2d8f4a6c0e9b7d3f1a5c8e")
```

（注意 `Authorization: Bearer …` 里 `Bearer` 与值之间是空格不是 `[=:]`，所以真正命中的是
`bearer` 后面那个空格 + 值——需要把值部分的分隔符从 `[=:]` 放宽为 `[=:[:space:]]`，
或者单列一条 `[Bb]earer[[:space:]]+${BARE}{16,}` 分支。后者更窄、误报更少，推荐。）

同时把 `-----BEGIN [A-Z ]*PRIVATE KEY-----` 加成独立分支——它零误报且是最贵的那类泄漏。

---

#### WR-05: `useEngineInvalidation` 的 listen 失败分支与 `App.tsx` 的告警条**一条测试都没有**；`App.test.tsx` 把这个 hook 整个桩掉

**Classification:** WARNING
**File:** `src/lib/useEngineInvalidation.ts:42-44`、`src/App.tsx:57,61-65`、`src/App.test.tsx:12-14`、`src/lib/useEngineInvalidation.test.ts:49-55`

**Issue:** 这条链路的存在理由写得非常清楚（`useEngineInvalidation.ts:15-19`）：

> *`listen` 走插件命令 `plugin:event|listen`，受 Tauri ACL 管辖；capability 缺失时它 reject，
> 而同一页面上 `invoke` 的自有命令不过 ACL、照常成功。……对一个以「0 静默丢失」为发布门槛的
> 项目，这一类失败不能被丢进未处理的 Promise。*

**这段推理没有任何自动化证据。** 逐条核实：

- `useEngineInvalidation.test.ts` 的 `beforeEach`（第 49-55 行）把 `listenSpy` 固定成
  `async (…) => { … }`，**永远 resolve**；五条测试没有一条覆写成 reject。
- 五条测试都写成 `renderHook(() => useEngineInvalidation(), { wrapper })` 并丢弃返回值——
  hook 的返回值（失败文案）**从未被读取**。
- `App.test.tsx:12-14` 把整个 hook 换成 `useEngineInvalidation: () => {}`（返回 `undefined`），
  所以 `App.tsx:61-65` 的 `{invalidationFailure && <p role="alert">…}` 在任何测试里都走不到。
- 全仓库唯一一条 `listenSpy.mockRejectedValueOnce` 在 `DevSmoke.test.tsx:152`——那守的是
  **冒烟页自己那条独立的 listen**（`DevSmoke.tsx:92-94`），与本 hook 是两段各自独立的代码。

结论：把 `useEngineInvalidation.ts:42-44` 的 `pending.catch(...)` 整段删掉、再把 `App.tsx`
的告警条删掉，`npm run test -- --run` 仍然 75/75 全绿，`npm run lint` 也绿
（`no-floating-promises` 看的是 `pending` 这个已被 `.then(…, …)` 消费的绑定，删掉 `.catch`
不会让它变红）。而这两段代码守的正是本项目定义为最贵的那种失败：**失效链路静默失能，
表现与「数据本来就没变」完全同形**。

这与冒烟页那条测试的注释（`DevSmoke.test.tsx:150`：*「计数为 0 不足以作断言：正常状态下它也是 0」*）
是同一条推理——只是它在冒烟页被落实了，在顶层 hook 与 `App` 上没有。

**Fix:** 两条测试，各守一半。

`useEngineInvalidation.test.ts`：

```ts
it("returns the listen-failure copy instead of swallowing the rejection", async () => {
  listenSpy.mockRejectedValueOnce(
    "event.listen not allowed. Permissions associated with this command: core:event:allow-listen",
  );
  const { wrapper } = setup();
  const view = renderHook(() => useEngineInvalidation(), { wrapper });
  await flush();

  expect(view.result.current).toContain("事件通道");
  // 原始 ACL 文本是内部细节，不得原样出场（与 DevSmoke 同一条规矩）。
  expect(view.result.current).not.toContain("not allowed");
});

// 阴性对照：listen 成功时必须是 null，否则「一律返回文案」的实现也会绿。
it("returns null while the listener is healthy", async () => { … });
```

`App.test.tsx`：把那个 `vi.mock` 改成可切换的桩（`useEngineInvalidation: () => failureStub`），
加一条「hook 返回文案时顶层渲染出一个 `role="alert"`、且文案出现在里面」的测试，
再加一条「返回 null 时 `queryByRole("alert")` 为 null」的阴性对照。

---

#### WR-06: `handler.rs` 把被注入实现的 `ServiceError` 文本原样送进 MCP 响应——01-17 为「校验」关掉的外包，在「信息披露」上还开着

**Classification:** WARNING
**File:** `crates/prism-mcp/src/handler.rs:135-138`

**Issue:**

```rust
let items = tokio::task::spawn_blocking(move || source.list_feedback(&project_id))
    .await
    .map_err(|_| ErrorData::internal_error("feedback lookup task failed", None))?
    .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;   // ← 原样透传
```

`project_id_of` 的文档注释（`handler.rs:29-42`）把 01-17 的整条推理写得很到位：
*「声明了却不执行，等于把执行力外包给**被注入的实现**……缺陷会在别处引入、在这里生效。」*
那条推理被完整落实在**参数校验**上，但**没有**落实在同一个函数里的**错误文本**上：
`err.to_string()` 的内容完全由被注入的 `FeedbackSource` 决定，而它直接跨到外部 agent。

这不是假设的未来。`prism-types/src/service.rs:34-46` 明文写着 `ServiceError::Backend(String)`
会在 **Phase 5/6 第一个真实会失败的调用方出现时就地加回来**，并在同一段里提醒
*「rusqlite / io 的原始错误串可能带路径与 SQL 片段，不能直接 `to_string()` 塞进去」*
——那句提醒是写给**实现方**的约定，而这里正是那条约定唯一的执行点，它没有执行。
一个 `Backend(rusqlite_err.to_string())` 落地的那天，SQLite 语句片段与 sidecar 路径会经
`internal_error` 直达外部 agent，`handler.rs` 一行都不用改。

同一处还有一个更小的类别错误：被注入实现返回的 `ServiceError::Invalid`（**调用方的错**）
被报成 `internal_error`（服务端故障），与 `project_id_of` 刻意选 `invalid_params` 的理由
（`handler.rs:44-47`：*「报成 internal error 会让调用方去重试而不是去修请求」*）直接矛盾。

**Fix:** 在边界上把错误映射成**本 crate 写死的**规则形文本，按类别分流：

```rust
.map_err(|err| match err {
    ServiceError::Invalid(_) => {
        ErrorData::invalid_params("the request was rejected by the feedback source", None)
    }
    ServiceError::NotFound => ErrorData::invalid_params("no such project", None),
    // #[non_exhaustive]：新变体默认落到最粗的类别，且**不携带**下层文本。
    _ => ErrorData::internal_error("feedback lookup failed", None),
})?;
```

配一条注入测试：让 `FixedFeedback` 返回一个携带可识别长标记的 `ServiceError`，断言
`tools/call` 的响应体里**不含**那个标记（形态照抄
`handler.rs::the_rejection_text_does_not_echo_the_offending_value`）。缺了这条断言，
「边界上不透传」就仍然只是一句注释。

---

### Info

#### IN-01: `the_release_ipc_surface_excludes_the_dev_commands` 的锚点带尾随逗号——列表最后一项没有逗号时会被放过

**File:** `src-tauri/src/lib.rs:550-560`

断言循环用的是 `"commands::dev_ping,"` 等四个**带逗号**的字面量。切出来的 `arm`
（从 `#[cfg(not(debug_assertions))]` 到 `]);`）里不含任何注释，所以裸名字在这里不会误命中——
也就是说逗号是多余的，而它引入了一个缺口：一条作为**列表末项**追加、没有尾随逗号的
`commands::dev_seed_sample_docs` 会让四条断言全绿。今天靠 `cargo fmt --check`（01-28 刚接上）
把 `generate_handler![…]` 排成竖版并补齐尾随逗号来兜底，但那是一条间接的依赖。

**Fix:** 锚点改成 `"commands::dev_"` 一条，语义更准（「release 面上不许有任何 dev 命令」）
且不依赖格式化器：

```rust
assert!(!arm.contains("commands::dev_"), "release 的 IPC 面上仍注册着 dev 命令: {arm}");
```

---

#### IN-02: 发布 CSP 的 `style-src 'unsafe-inline'` 的理由不成立，很可能可以直接去掉

**File:** `src-tauri/tauri.conf.json:21`、`src/lib/tauri-security.test.ts:98-108`

测试注释说 `'unsafe-inline'` 是「React 的内联 style 属性需要它」。客户端 React（`createRoot`，
无 SSR）设置 `style` prop 走的是 **CSSOM**（`node.style.setProperty`），而 CSP 的 `style-src` /
`style-src-attr` **不管辖 CSSOM 操作**——只管辖 HTML 里的 `style=` 属性与 `<style>` 元素。
本项目全仓库没有任何 CSS import（`main.tsx` / `App.tsx` 都没有），`vite build` 因此不会注入
`<style>`。也就是说发布形态大概率一条内联样式都不需要。

**Fix:** 先把发布 CSP 的 `style-src` 收成 `'self'`，跑一次 `npm run tauri build` 打开的窗口，
确认控制台无 CSP 违规报告（这条已经在 01-13-PLAN 的 `<human-check>` 里）；确认后同步改
`tauri-security.test.ts` 的 ④′ 断言为 `expect(csp).not.toContain("unsafe-inline")`。
devCsp 保持不变（Vite 的引导脚本确实需要它）。

---

#### IN-03: eslint 的基础块只声明 `languageOptions`、不声明任何 `rules`，仓库根的两个配置文件被解析后一条规则都不过

**File:** `eslint.config.js:49-56`

`files: ["*.js", "*.ts", "vite.config.ts"]` 这一块的目的是避开 `projectService` 的 parser 缺口，
说明写得很清楚。但它没有 `rules`，而类型感知块的作用域是 `src/**`，于是 `eslint.config.js`
与 `vite.config.ts` 落在一个「有 parser、无规则」的空档里——`npm run lint` 会读它们、
然后什么都不检查。文件头把这份配置称作「前端唯一的 lint 闸门」，而闸门有两个文件不在其内。

**Fix:** 给基础块补上不需要类型信息的那几条核心规则（`no-console`、`no-debugger`、
`no-unused-labels`、`eqeqeq`），或者显式在注释里写明「这两个文件刻意只做语法解析」——
两者都行，重要的是这件事在文件里可读，而不是要从块结构推出来。

---

#### IN-04: `EngineError::Llm(_)` 整体映射成 `secret_error`，Phase 4 的 HTTP 失败会被说成「钥匙串不可用」

**File:** `src-tauri/src/commands.rs:41`、`crates/prism-llm/src/lib.rs:15`、`src/lib/ipc.ts:62`

`LlmError` 有两个变体：`Http(reqwest::Error)` 与 `Keychain(String)`，`map_err` 把两者一起
映射成 `"secret_error"`，文案是「系统钥匙串当前不可用，密钥没有保存。」。Phase 4 接上真实
LLM 调用之后，一次网络超时会告诉用户去检查钥匙串。今天 `prism-llm` 不发任何请求，所以是潜伏的。

**Fix:** `EngineError::Llm(LlmError::Http(_)) => "network_error"`（新短码 + 新文案），
`EngineError::Llm(_) => "secret_error"` 保留为兜底。`error_codes_are_stable_short_strings`
补一行。放在 Phase 4 开工时做也可以，但要记进 deferred-items，否则它会随第一个真实调用一起上线。

---

#### IN-05: Phase 1 的两处「已接线但无消费者」——`prism-mcp` 作为 `prism-engine` 的普通依赖无生产使用点；`McpDeps.comments` 从未被读

**File:** `crates/prism-engine/Cargo.toml:22`、`crates/prism-mcp/src/deps.rs:26-27`、`crates/prism-mcp/src/handler.rs:134`

`prism-engine` 的 `src/` 里没有任何 `prism_mcp::` 使用点（只有 `tests/facade.rs` 用它，
那是 dev 边）；`McpDeps.comments: Arc<dyn CommentSink>` 被注入后没有任何 handler 读它。
两者都是刻意的 Phase 6 脚手架，本身不是缺陷——记在这里是因为**它们是 WR-01 目前只算 WARNING
的全部理由**：`record_receipt` 的 `comment_id` 缺口不可达，靠的就是 `deps.comments` 这条死线。
Phase 6 接上评论回流工具的那一次 commit，会在同一时刻让 WR-01 变成活的。

**Fix:** 无需改代码。建议在 `deps.rs` 的 `comments` 字段旁写一行「Phase 6 接上它之前，
`record_receipt` 的入参校验必须先补齐（见 01-REVIEW WR-01）」，让这条依赖关系留在代码里而不是
只留在评审报告里。

---

## 覆盖面声明

70 个文件全部逐行读过，没有只扫标题的。按 `<scope_note>` 的要求，注意力配比如下：

- **重点核实（含独立取证 / 交叉查阅上游源码）**：`crates/prism-mcp/src/middleware.rs`
  （对照 `~/.cargo/registry/.../rmcp-2.2.0/src/transport/streamable_http_server/tower.rs`
  逐函数比对 Host 与 Origin 两条路径）、`crates/prism-mcp/src/{handler,deps,server}.rs`、
  `crates/prism-store/src/{open,settings,search}.rs`、`crates/prism-llm/src/secrets.rs`、
  `scripts/check-*.sh`（把 `$PATTERN` 抽出来实跑取样）、`src-tauri/{tauri.conf.json,capabilities/default.json,src/lib.rs}`、
  `.github/workflows/ci.yml`、`eslint.config.js`、`src/pages/Settings.tsx` +
  `crates/prism-store/src/settings.rs`（用 `url` crate 与 node 双侧实跑对齐口径）。
- **常规通读并核对断言判别力**：全部 `tests/` 与 `*.test.ts(x)`、`crates/prism-engine/**`、
  `src-tauri/src/{commands,bus_adapter,smoke}.rs`、`src/lib/**`、`src/pages/DevSmoke.tsx`。
- **通读但未深挖**（骨架 crate，Phase 1 只声明依赖，逻辑面极小）：`crates/prism-fs/src/lib.rs`、
  `crates/prism-parse/src/lib.rs`、`crates/prism-anchor/src/lib.rs`、`crates/prism-cli/src/main.rs`。
  这四个文件我读完了，没有发现问题，但它们的真实判别力要到 Phase 2/3/6 才有对象可测。
- **不在本次 review 文件清单内、因此未审**：`vite.config.ts`（`eslint.config.js` 与
  `tsconfig.json` 都引用它，测试环境/覆盖率配置在其中）、`src-tauri/src/main.rs`、
  根 `Cargo.toml` 的 `[workspace.dependencies]`。如果下一轮想把「75 个前端测试确实跑在
  jsdom 下、覆盖率确实在收集」这件事也纳入证据，`vite.config.ts` 需要进清单。

## 已复核为「仍然关闭」的前两轮 findings

第一轮 CR-01/CR-02/CR-03、WR-01..WR-16、IN-01..IN-06 与第二轮 CR-01、WR-01..WR-11、
IN-01..IN-04 我都逐条回到代码里确认过关闭状态，未发现回退。本轮**没有**重复报告其中任何一条；
上面 WR-01 / WR-03 / WR-04 虽然与第一轮 WR-13、第二轮 WR-06、第二轮 CR-01 属于同一缺陷**类别**，
但落点是不同的字段 / 不同的函数 / 不同的键名，是那些修复**未覆盖到的兄弟位置**，不是回退。

---

_Reviewed: 2026-07-30T00:20:58Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard · Round 3_
