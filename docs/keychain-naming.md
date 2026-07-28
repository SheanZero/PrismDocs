# Keychain 命名契约

**建立于:** Phase 1 / plan 01-04（INFRA-03）
**状态:** 生效中——**改名是破坏性变更**，见文末警示

PrismDocs 的密钥全部存在 macOS 登录钥匙串（`keyring-core` + `apple-native-keyring-store`
的 `keychain` 模块）。钥匙串条目由 `(service, account)` 两元组定位，这两个字符串是
**跨 crate 与跨二进制的隐式接口**：桌面 app 写、CLI helper 读，两侧编译单元不同，
拼错不会有编译错误，只会在运行期表现为「读不到 key」。因此把它们写在这里。

## 契约

| service | account | 内容 | 写入方 | 读取方 |
|---------|---------|------|--------|--------|
| `PrismDocs` | `llm_api_key` | 用户配置的 LLM 端点 API key | `prism-llm::secrets::set_api_key`（经 settings 页） | `prism-llm::secrets::get_api_key` |
| `PrismDocs` | `mcp_bearer_token` | 本机 MCP loopback 宿主的 per-install bearer token | Phase 6：app 首次启动时以 CSPRNG 生成并写入 | Phase 6：app 的鉴权中间件；`prismdocs-helper headers` 子命令 |

常量的唯一定义处是 `crates/prism-llm/src/secrets.rs`：

```rust
pub const SERVICE: &str = "PrismDocs";
pub const ACCOUNT_LLM_KEY: &str = "llm_api_key";
pub const ACCOUNT_MCP_TOKEN: &str = "mcp_bearer_token";
```

`prismdocs-helper`（`crates/prism-cli`）**不链接任何 `prism-*` engine crate**（D-10，
由 `scripts/check-deps.sh` 断言），所以它无法 `use` 上面的常量——它必须自己写一份
同值的字面量。这正是本文件存在的理由：两份字面量之间没有编译期约束，只有这张表。

## 为什么是 `keychain` 模块而不是另一个

`apple-native-keyring-store` 提供两个后端模块，crate 默认一个都不启用，必须显式选 feature。
PrismDocs 用 `features = ["keychain"]` + `apple_native_keyring_store::keychain::Store::new()`。

另一个模块要求 app 由 provisioning profile 签名。PrismDocs 是**直发公证 DMG、不进
App Sandbox、无 provisioning profile**，选错会编译通过但在运行期报
`-34018 A required entitlement isn't present`——一个只在真机运行时才出现的失败。

## 不变量

1. **密钥的唯一存放地是系统钥匙串。** 不进 SQLite（含 `settings` 表）、不进仓库内文件、
   不进 `tauri.conf.json`、不进日志。非密钥配置（`base_url`、模型标识等）走
   `settings` 表（D-05），两者的存放地刻意不同：`settings` 会被「数据库单目录整体备份」
   带走，密钥不能。
2. **钥匙串失败时显式冒泡，不回退到明文来源。** 唯一被映射为「没有」的错误是
   `NoEntry`（→ `Ok(None)`，D-06 要求无 key 也能启动）。环境变量 / dotfile
   回退路径一律不存在——有一条就等于「密钥只在钥匙串」的承诺是假的。
3. **密钥不经 `Debug` / `Display` / 日志泄漏。** `ApiKey` 手写 `Debug` 输出
   `ApiKey(<redacted>)` 且刻意不实现 `Display`；`LlmError::Keychain` 只保留
   `keyring_core::Error` 的 Display 文本，不转发会打印原始字节的错误值。

## 更名警示

`service` 或 `account` 任一改名，**已安装用户钥匙串里的旧条目会变得不可见**——
app 会表现为「key 凭空消失」，而旧条目仍留在钥匙串里成为孤儿。

若将来确有更名需要，必须配套一次迁移：用旧名读出 → 用新名写入 → 删除旧条目，
并且要同时更新 `prismdocs-helper` 内的字面量副本，否则 CLI 侧会静默停留在旧名上。

## 手动验证真实钥匙串往返

自动化测试用 `keyring_core::mock::Store` 覆盖逻辑；真实钥匙串往返标了 `#[ignore]`，
因为 dev 期签名身份变化会反复触发系统授权弹窗，且 CI / headless 环境没有已解锁的钥匙串。

```bash
cargo test -p prism-llm -- --ignored roundtrip_with_real_keychain
```

弹出系统授权框时选「允许」。测试自带清理（`delete_api_key`），跑完可在
「钥匙串访问」中确认 `PrismDocs` 条目已被删除。
