---
phase: 01
slug: foundation-skeleton
status: secured
# threats_open = count of OPEN threats at or above workflow.security_block_on severity (the blocking gate)
threats_open: 0
asvs_level: 1
block_on: high
threats_total: 149
threats_closed: 149
register_authored_at_plan_time: true
created: 2026-07-30
---

# Phase 01 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

首次 security audit。威胁册**在计划期就写好**（28 份 PLAN 全部带 `<threat_model>` 块），
因此本次是「验证缓解措施存在」而非 retroactive-STRIDE 构册。

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| WebView ↔ Tauri shell | React 前端经 IPC 调 10 个 `#[tauri::command]`；CSP 限制 `connect-src 'self' ipc: http://ipc.localhost` | 设置值、搜索词、事件计数；**从不跨越**：API key 本体（`api_key_status` 只回 `bool`） |
| Tauri shell ↔ engine workspace | 薄 shell 只做 `map_err` 短码转换，无业务逻辑（`commands.rs` 中 Connection/prepare/query_row/keyring 各 0 次） | DTO；错误一律降为短码，不透传 rusqlite/io 原文 |
| engine ↔ SQLite sidecar | 单写者 `Mutex<Connection>` + r2d2 读池（`query_only=ON`）；库落在 `dirs::data_dir()` 而非用户 repo | 文档/设置/评论；WAL 模式 |
| engine ↔ 系统钥匙串 | `keyring-core` + `apple-native-keyring-store`；`prism-llm` 是**唯一密钥入口** | API key；`ApiKey` 手写 Debug 脱敏且刻意不实现 Display |
| MCP loopback ↔ 外部 agent | `127.0.0.1:0` + Host/Origin allowlist + 常数时间 bearer 比较 | **今天不带电** —— `serve_loopback` 在发布形态零调用点，见 § Reachable-in-Future |
| 版本库 ↔ 外部 | `scripts/check-secrets.sh` 每次 CI 扫全部受控文件（排除集仅 `.planning/`） | 源码与配置；闸门自证判别力（22 阳性 / 13 阴性） |
| 依赖树 ↔ 上游 registry | `blocking-human` 包合法性闸门 + 锁文件 integrity | 三方代码；见 § Accepted Risks 的 T-01-SC 条目 |

---

## Threat Register

149 条威胁，跨 28 份 PLAN。**149 closed / 0 open。**
逐条落点见 `01-SECURITY-audit-2026-07-30.md` 的完整表（由 gsd-security-auditor 产出，
每条以 `file:line` 或实跑命令为证据）。此处只记结构与关键项。

| 分布 | critical | high | medium | low | 合计 |
|---|---|---|---|---|---|
| 数量 | 1 | 68 | 60 | 18 | 147\* |
| disposition | mitigate 126 / accept 21 | | | | |

\* 另 2 条（`T-01-49` @01-11、`T-01G-06` @01-15）因表格 cell 含管道符未进机器解析，由 auditor 从 PLAN 原文直读，均判 CLOSED。

### 本次审计期间关闭的唯一 OPEN

| Threat ID | Category | Component | Severity | Disposition | Mitigation | Status |
|-----------|----------|-----------|----------|-------------|------------|--------|
| T-01-03a | Information Disclosure | `scripts/check-secrets.sh` | high | mitigate | `$PATTERN` 新增 `bearer[[:space:]]+[A-Za-z0-9._~+/-]{20,}` 分支 + selftest 三条隔离阳性样本 + 一条边界阴性 + 两条散文阴性 | **closed** |

**它为什么曾经 open**：威胁陈述里有两个名词（「明文 API key **/ bearer token**」），
而 `$PATTERN` 只覆盖前一个。`Authorization: Bearer <32 hex>` 的三种现实提交写法全部不命中，
而那恰好是 D-07 的 `prismdocs-helper headers` 输出形态。selftest 阳性组里
`Authorization: "Bearer sk-ant-api03-…"` 那条读起来像已覆盖，实际经 `sk-` 前缀分支命中
——控件看着有、实际没有（≡ `01-REVIEW.md` 第三轮 WR-04，≡ 第二轮 CR-01 的同形态换键名）。

**收口证据（三条非恒真反证，全部实跑）**：

| 反证 | 操作 | 结果 |
|---|---|---|
| A | 从 `$PATTERN` 删掉 bearer 分支 | selftest exit 1，红的**恰好**是新增三条隔离样本；其余 19 条阳性一条不受影响 |
| B | bearer 下界 20 → 8 | 边界阴性样本 `Bearer abcdefghijklmnopqrs`（19 字符）当场变红 |
| C | 把 headers helper 输出粘进 `crates/prism-mcp/src/` 下的受控文件 | `scan` exit 1 并点名行号；删除后 exit 0 |

收口后：`check-secrets.sh all` → 22 阳性 / 13 阴性全绿，116 个受控文件零命中，exit 0。
下界取 20 而非更低，理由与裸值下界同源：bearer token 是机器生成高熵串（本项目
`mcp_bearer_token` 是 32 位十六进制），20 圈得住它同时避开 `Bearer <short-word>` 这类散文；
字符集取 RFC 6750 的 b64token 语法。

---

## Reachable-in-Future

**下列 25 条威胁的缓解措施代码与测试均已就位且可复跑，但在今天的发布形态里走不到。**
它们不是「已解决」，是「已布防、尚未带电」。**Phase 6 规划必须把这一节读完。**

`serve_loopback` / `build_router` / `McpDeps::new` 在 `crates/prism-engine/src/` 与
`src-tauri/src/` 中**零调用点**（grep 验证）——MCP loopback 服务在发布形态从未启动，
只有测试调用它。

- **MCP 门禁面（21 条）**：`T-01-05` `T-01-06` `T-01-28` `T-01-29` `T-01-30` `T-01-50`
  `T-01-51` `T-01-52` `T-01-53` `T-01G-09` `T-01G-10` `T-01G-11` `T-01G-12` `T-01G-13`
  `T-01G-44` `T-01G-45` `T-01G-46` `T-01G-47` `T-01G-48` `T-01G-49` `T-01G-50`
- **`record_receipt` 面（4 条）**：`T-01-33` `T-01G-18` `T-01G-19` `T-01G-20`
  —— `McpDeps.comments` 在 `handler.rs` 中无消费方，是死字段。

**Phase 6 接线当天，这 25 条同时带电。** 接线前应重跑 `/gsd-secure-phase`。

---

## Accepted Risks Log

21 条 `accept` 处置。**逐条转录在此是有意的**：不转录的话下一轮审计会重新把它们提为 OPEN，
该 disposition 即失效。

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-01 | T-01-10 (01-03, DoS, low) | WAL 无限膨胀（长驻读连接阻塞 checkpoint）。补偿控件：`open.rs:155-161` 闭包式 `read()` 不留长驻连接、`:179` `wal_checkpoint(TRUNCATE)`。量化复核推迟到 Phase 8 压测 | plan 01-03 | 2026-07-29 |
| AR-02 | T-01-24 (01-04, EoP, low) | prism-types 引入三方依赖会同时污染 prism-mcp 与 prism-engine。补偿控件：`prism-types/Cargo.toml` 依赖仅 serde + thiserror，且 `check-deps.sh` 的 `PURE_CRATES` 断言看住五个叶子 crate | plan 01-04 | 2026-07-29 |
| AR-03 | T-01-30 (01-06, DoS, low) | rmcp session 未随应用关闭清理。补偿控件：`server.rs:57` `with_cancellation_token(ct.child_token())` + `:80-88` graceful shutdown | plan 01-06 | 2026-07-29 |
| AR-04 | T-01-32 (01-07, DoS, low) | 慢订阅者塞满 broadcast 环形缓冲。设计上 broadcast 丢最旧而不阻塞 publish 端，由 `Lagged→Resync` 补偿（`bus_adapter.rs:44`）；`bus.rs:22` `BUS_CAPACITY=256` | plan 01-07 | 2026-07-29 |
| AR-05 | T-01-38 (01-09, InfoDisc, low) | 隐藏 dev 冒烟页在正式构建中仍可经地址栏访问。**已被 01-21 收紧至超出接受范围**：`App.tsx:28` `if (!import.meta.env.DEV) return null`（vite build 整块摇掉）+ `lib.rs:201-223` `#[cfg(debug_assertions)]` 编译期分叉，`strings target/release/prismdocs` 实测四条 dev 命令名 0 命中 | plan 01-09 → 01-21 收紧 | 2026-07-29 |
| AR-06 | T-01-49 (01-11, 见 PLAN 原文) | `check-secrets.sh` 中 `\|\| true` 的双义性（吞失败 vs 吞干净）无法在单行内区分。补偿控件：`:127-133` 注释逐句区分两种形态 + selftest 作为配套证明（被吞的调用失败会让阳性样本变红） | plan 01-11 | 2026-07-29 |
| AR-07 | T-01-54 (01-12, DoS, low) | 构造期改为可失败后，Phase 6 注入路径若把 `Err` 处理成 `unwrap()` 会把配置错误变成崩溃。补偿控件：`deps.rs:39-42` 与 `01-16-SUMMARY.md` For Next Phase 均写明 Phase 6 须降级为 warn 而非 panic | plan 01-12 | 2026-07-29 |
| AR-08 | T-01G-21 (01-20, Repudiation, low) | 移除未构造的 `ServiceError::Backend` 变体后失去一类错误的表达力。补偿控件：`prism-types/src/service.rs:34-46` 在原位写明重新引入的条件与届时的文本约束（含「不能直接 `to_string()`」） | plan 01-20 | 2026-07-29 |
| AR-09 … AR-21 | T-01-SC ×14 (01-14/15/16/17/18/19/20/21/22/23/24/25/27/28, Tampering, high) | 各 gap-closure plan 的「本 plan 不新增依赖」声明。**证据**：`git log --name-only -- Cargo.lock package-lock.json` 显示 01-14…01-28 区间内仅两次锁文件变动 —— `70cee58`（01-21，`serial_test`，已在 workspace 且有审计行）与 `f04f4ae`（01-26，经 blocking-human 闸门的四个 eslint 包）。其余 plan 声明成立 | 各 plan | 2026-07-29 |

---

## Unregistered Flags

新出现的攻击面，**无威胁映射**。WARNING 级，不计入 `threats_open`。
它们不是本 phase 的失败，是下一轮威胁建模的输入。

| id | 落点 | 说明 | 建议归属 |
|---|---|---|---|
| UF-1 | `package-lock.json`（`a95043b`） | `@emnapi/core` / `@emnapi/runtime` **@2.0.0-alpha.3** 由 phase 收尾后的 CI 修复提交带入，落在所有 `<threat_model>` 之外。**已于 2026-07-30 补录审计行**（`01-RESEARCH.md`），但**未走 blocking-human 闸门**。实际风险面低：两包在锁文件中为 `dev+optional+peer`，`node_modules/@emnapi/` 在有原生 rolldown 绑定的平台整个目录不存在 —— 有记录、无执行。**未解决**：若将来某 runner 缺原生绑定，这两个 alpha 包会被真正安装并执行，届时必须补正式审计 | Phase 2 威胁建模 |
| UF-2 | `crates/prism-mcp/src/handler.rs:138` | `internal_error(err.to_string())` 把被注入实现的 `ServiceError` Display 逐字回抛给外部 agent。今天无害（变体只剩 `NotFound` + `Invalid(String)`，文本由实现方写死），但 `service.rs:42-46` 已预告 Phase 5/6 加回携带 rusqlite/io 文本的 `Backend` 变体 —— 届时这一行即外泄通道。**现有防线只是一句注释，不是机制** | Phase 5/6（与 AR-08 同一处） |
| UF-3 | `crates/prism-engine/src/services.rs:73,86` | `comment_id` 与受控取值的 `status` 同处一条日志行，但只做 `trim().is_empty()`；嵌入换行 / 整段正文可经它进日志。今天不可达（`deps.comments` 无消费方） | Phase 6（≡ 01-REVIEW WR-01） |
| UF-4 | `crates/prism-store/src/settings.rs:118 vs :125` | 「校验 `trim()` 后的串、存原值」形态本轮在 `McpDeps::new` 与 `set_api_key` 两处已修，`set_setting` 未修。auditor 追了 L2 数据流后确认**不可构造凭据绕过**（`str::trim` 与 WHATWG URL 归一化对凭据判定等价），属一致性缺陷 | Phase 2（≡ 01-REVIEW WR-03） |
| UF-5 | `crates/prism-llm/src/secrets.rs` | `keyring_core::set_default_store` 是进程级全局；对同一 keychain account 的并发写**无任何断言**。全部测试 `#[serial]` 是回避该问题而非回答它 | Phase 4（接真实 chat client 时） |
| UF-6 | `src-tauri/tauri.conf.json`（WINDOWS id=11） | 发布 CSP 保留 `style-src 'self' 'unsafe-inline'`（React 内联 style 需要）。已被精确相等断言钉住且有记录，但放宽本身未被消除 | 前端重构时 |
| UF-7 | `scripts/check-deps.sh` | T-01-17 的第二道控件（`cargo tree --edges dev`）只是 plan 期一次性验收项，**未进 `all` / CI**。今天干净（auditor 复跑为 0），未来回归无人看守 | 下一轮闸门收口 |
| UF-8 | `docs/keychain-naming.md` ↔ `secrets.rs:21-27` | 跨二进制命名契约无自动漂移哨兵。改名不会让任何测试变红，破裂点在 Phase 6 的 `prismdocs-helper` | Phase 6 |

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-07-30 | 149 | 148 | 1 (T-01-03a) | gsd-security-auditor |
| 2026-07-30 | 149 | 149 | 0 | orchestrator（收口 T-01-03a，三条非恒真反证实跑） |

**auditor 超出 ASVS L1 要求所做的额外核查**（L1 只要求 grep 级存在性）：

1. **T-01-43 / WR-03 是否可被绕过** —— 尝试构造「通过 `validate_base_url(trim)` 校验、却把凭据
   留在存储原值里」的输入。Rust `str::trim` 只剥 Unicode White_Space，`url` crate 按 WHATWG
   先移除全部 tab/LF/CR，两者归一化后的字节对凭据判定完全等价，**无法构造绕过**。
2. **T-01-03b 的键名启发式** —— `is_secret_like_key` 是 `contains` 匹配，插零宽字符可规避；
   但该威胁语义是「误入」而非「刻意藏入」，且 01-10 已把真正的边界建在**值**上（T-01-39/40）。
3. **T-01-17 的 dev 逃逸口** —— 复跑 `cargo tree -p prism-mcp --edges dev` 得 0，
   prism-mcp 的 `[dev-dependencies]` 只有 tokio/reqwest/tower。残余见 UF-7。

**auditor 独立复跑的闸门**（非引用文档，本人执行）：

```
bash scripts/check-secrets.sh all  → exit 0
bash scripts/check-deps.sh all     → exit 0（七条断言全 OK）
cargo test -p prism-llm ×3         → 10 passed / 1 ignored ×3（T-01-23 的 backstop）
cargo tree -p prism-mcp --edges dev --prefix none | grep -c '^prism-engine ' → 0
grep -rn "serve_loopback|build_router|McpDeps::new" crates/prism-engine/src src-tauri/src → 无匹配
```

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log（21 条逐条转录）
- [x] `threats_open: 0` confirmed
- [x] Reachable-in-future 威胁显式标出，未被静默计入已关闭（25 条）
- [x] Unregistered flags 记录并给出归属建议（8 条）
