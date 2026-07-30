//! INFRA-03 的**唯一密钥入口**：系统钥匙串的读写往返。
//!
//! ## 不变量
//!
//! 1. 密钥的唯一存放地是系统钥匙串。**绝不**写入 SQLite（含 `settings` 表）、
//!    仓库内文件、`tauri.conf.json` 或任何会被「数据库单目录整体备份」带走的位置。
//! 2. 钥匙串不可用或读取失败时**显式冒泡**，不回退到环境变量、dotfile 或任何
//!    明文来源。一条看不见的降级路径会让「密钥只在钥匙串」的承诺变成假的（T-01-21）。
//! 3. 持有密钥的类型不经 `Debug` / `Display` / 日志泄漏原文（T-01-04）。
//!
//! ## 命名契约
//!
//! `SERVICE` 与两个 `ACCOUNT_*` 是**跨二进制契约**：Phase 6 的 `prismdocs-helper`
//! 必须用同一套名字读同一条目，写错要到 Phase 6 才暴露。改名等于让已安装用户的
//! 密钥不可见。完整说明见 `docs/keychain-naming.md`。

use crate::LlmError;
use keyring_core::{Entry, Error};

/// 钥匙串条目的 service 名。全应用（含 CLI helper）唯一。
pub const SERVICE: &str = "PrismDocs";

/// LLM API key 的 account 名。
pub const ACCOUNT_LLM_KEY: &str = "llm_api_key";

/// 本机 MCP loopback 宿主的 per-install bearer token 的 account 名（Phase 6 使用）。
pub const ACCOUNT_MCP_TOKEN: &str = "mcp_bearer_token";

/// 注册进程级默认 credential store（macOS 真实钥匙串）。
///
/// **应用启动时调用一次，且必须在创建任何 Entry 之前。**
///
/// 必须用 `keychain` 模块：PrismDocs 是直发公证 DMG、不进 App Sandbox、
/// 无 provisioning profile，用另一个受 entitlement 约束的模块会在运行期报
/// `-34018 A required entitlement isn't present`（RESEARCH Pitfall 2）。
#[cfg(target_os = "macos")]
pub fn init_default_store() -> Result<(), LlmError> {
    keyring_core::set_default_store(apple_native_keyring_store::keychain::Store::new()?);
    Ok(())
}

/// 写入 LLM API key。对同一 (service, account) 重复写入是幂等的——
/// 底层 store 更新既有条目而不是追加一条。
///
/// 入参先裁剪首尾空白。这是**防御性**的第二道——设置页在提交前已裁剪一次——
/// 但绕过界面直接 invoke 走的也是这个函数。与 `McpDeps::new` 对 bearer token 的
/// 归一化同源（01-16）：只用 `trim` 判空却存原值，会造出一个看起来配置好了
/// 但永远用不了的凭据（`api_key_status()` 报「已配置」，每次调用返回 401，
/// 而本地没有任何信号指向那个尾随换行）。裁剪只碰首尾，内部空白原样保留。
pub fn set_api_key(secret: &str) -> Result<(), LlmError> {
    Entry::new(SERVICE, ACCOUNT_LLM_KEY)?.set_password(secret.trim())?;
    Ok(())
}

/// 读取 LLM API key。
///
/// 无条目时返回 `Ok(None)` 而不是 `Err`——D-06 要求「API key 可跳过，
/// 无 key 应用照常启动」。其余错误一律冒泡：钥匙串被锁、权限被拒等情况
/// 必须让调用方看见，不得被当成「没有 key」吞掉。
pub fn get_api_key() -> Result<Option<String>, LlmError> {
    // `keyring_core::Error` 是 #[non_exhaustive]，match 必须带 `_` 臂。
    match Entry::new(SERVICE, ACCOUNT_LLM_KEY)?.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(Error::NoEntry) => Ok(None),
        Err(other) => Err(other.into()),
    }
}

/// 删除 LLM API key。已不存在视为已删除，返回 `Ok(())`（幂等）。
pub fn delete_api_key() -> Result<(), LlmError> {
    match Entry::new(SERVICE, ACCOUNT_LLM_KEY)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(Error::NoEntry) => Ok(()),
        Err(other) => Err(other.into()),
    }
}

/// 内存中持有 API key 的 newtype。
///
/// **手写 `Debug` 输出固定占位串**，且**刻意不实现 `Display`**——后者的缺席
/// 让 `format!("{key}")`、`println!("{key}")` 与 `tracing` 的 `%key` 直接编译失败，
/// 而不是在运行期把密钥打进日志（Security Domain V7 / T-01-04）。
/// 取原文的唯一途径是 [`ApiKey::expose`]，调用点因此在代码搜索中可见。
pub struct ApiKey(String);

impl ApiKey {
    pub fn new(secret: impl Into<String>) -> Self {
        Self(secret.into())
    }

    /// 取出密钥原文。仅用于构造出站请求头，取出后不得再存放到别处。
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ApiKey(<redacted>)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::collections::HashMap;

    /// 测试用的假密钥。刻意不用 `sk-` 开头的长串，避免与
    /// `scripts/check-secrets.sh` 的明文密钥正则撞车——那个扫描器是防线，
    /// 不该为了让 fixture 通过而放宽它。
    const FIXTURE_SECRET: &str = "prism-test-secret-value";
    const OTHER_FIXTURE_SECRET: &str = "prism-test-rotated-value";

    /// 装上进程级 mock store。
    ///
    /// 先 unset 再 set：前一个测试若在收尾前 panic，默认 store 会残留下来，
    /// 而 `set_default_store` 只是覆盖——先清一次让每个测试的起点确定。
    fn install_mock_store() {
        let _ = keyring_core::unset_default_store();
        keyring_core::set_default_store(
            keyring_core::mock::Store::new().expect("mock store is constructible"),
        );
    }

    /// 当前 (SERVICE, ACCOUNT_LLM_KEY) 下的条目数。
    fn llm_key_entry_count() -> usize {
        let spec = HashMap::from([("service", SERVICE), ("user", ACCOUNT_LLM_KEY)]);
        keyring_core::Entry::search(&spec)
            .expect("search the default store")
            .len()
    }

    #[test]
    #[serial]
    fn roundtrip_with_mock_store() {
        install_mock_store();

        assert!(get_api_key().expect("read empty store").is_none());

        set_api_key(FIXTURE_SECRET).expect("write the key");
        assert_eq!(
            get_api_key().expect("read back").as_deref(),
            Some(FIXTURE_SECRET)
        );

        delete_api_key().expect("delete the key");
        assert!(get_api_key().expect("read after delete").is_none());

        let _ = keyring_core::unset_default_store();
    }

    #[test]
    #[serial]
    fn no_key_is_not_an_error() {
        install_mock_store();

        // D-06：API key 可跳过，无 key 时应用照常启动。所以「没有条目」
        // 必须是 Ok(None) 而不是 Err——后者会让启动路径不得不吞错。
        let read = get_api_key();
        assert!(read.is_ok(), "missing key surfaced as an error");
        assert!(read.expect("checked above").is_none());

        let _ = keyring_core::unset_default_store();
    }

    #[test]
    #[serial]
    fn set_and_delete_are_idempotent() {
        install_mock_store();

        set_api_key(FIXTURE_SECRET).expect("first write");
        set_api_key(FIXTURE_SECRET).expect("second write of the same value");
        assert_eq!(
            get_api_key().expect("read back").as_deref(),
            Some(FIXTURE_SECRET),
            "repeated writes changed the stored value"
        );
        assert_eq!(
            llm_key_entry_count(),
            1,
            "repeated writes created a duplicate keychain entry"
        );

        // 覆写为另一个值同样只应留下一条条目（轮换 key 的路径）。
        set_api_key(OTHER_FIXTURE_SECRET).expect("rotate the key");
        assert_eq!(
            get_api_key().expect("read back").as_deref(),
            Some(OTHER_FIXTURE_SECRET)
        );
        assert_eq!(llm_key_entry_count(), 1, "rotation created a second entry");

        delete_api_key().expect("first delete");
        delete_api_key().expect("second delete must be a no-op, not an error");
        assert!(get_api_key().expect("read after delete").is_none());

        let _ = keyring_core::unset_default_store();
    }

    /// 从供应商控制台复制来的密钥常带一个尾随换行或空格。前端已在提交前裁剪，
    /// 这里是**第二道**：绕过界面直接 invoke 走的也是这个函数，而一个带空白的凭据
    /// 会让 `api_key_status()` 报「已配置」、让 Phase 4 的每次调用返回 401，
    /// 本地没有任何信号指向空白。
    #[test]
    #[serial]
    fn set_api_key_trims_surrounding_whitespace() {
        install_mock_store();

        set_api_key(&format!("  {FIXTURE_SECRET}\n")).expect("write a padded key");
        assert_eq!(
            get_api_key().expect("read back").as_deref(),
            Some(FIXTURE_SECRET),
            "存进钥匙串的密钥仍带首尾空白"
        );

        // 阴性对照：裁剪只碰首尾。一个「把所有空白都删掉」的实现会让上一条也绿，
        // 而那会悄悄改坏一个内部含空格的合法凭据。
        let inner = "prism-test value";
        set_api_key(inner).expect("write a key with an inner space");
        assert_eq!(get_api_key().expect("read back").as_deref(), Some(inner));

        delete_api_key().expect("clean up");
        let _ = keyring_core::unset_default_store();
    }

    #[test]
    fn apikey_debug_is_redacted() {
        let key = ApiKey::new(FIXTURE_SECRET);
        let shown = format!("{key:?}");

        assert!(
            !shown.contains(FIXTURE_SECRET),
            "ApiKey leaked its secret through Debug: {shown}"
        );
        // 反向确认：`expose()` 才是取原文的唯一途径。
        assert_eq!(key.expose(), FIXTURE_SECRET);
    }

    #[test]
    fn keychain_errors_are_flattened_to_their_display_text() {
        // `keyring_core::Error` 的 derive Debug 会打印 BadEncoding 携带的原始字节。
        // LlmError 必须只保留 Display 文本，否则 `unwrap()` / `tracing` 的 `?err`
        // 就是一条把密钥打进日志的通道（T-01-04）。
        let raw = keyring_core::Error::BadEncoding(FIXTURE_SECRET.as_bytes().to_vec());
        let flattened = LlmError::from(raw);

        assert_eq!(
            format!("{flattened:?}"),
            r#"Keychain("Password data is not valid UTF-8")"#,
            "LlmError carried the raw keyring payload"
        );
        assert!(!format!("{flattened}").contains(FIXTURE_SECRET));
    }

    #[test]
    #[serial]
    #[ignore = "touches the real login keychain; run manually: cargo test -p prism-llm -- --ignored roundtrip_with_real_keychain"]
    fn roundtrip_with_real_keychain() {
        let _ = keyring_core::unset_default_store();
        init_default_store().expect("register the macOS keychain store");

        set_api_key(FIXTURE_SECRET).expect("write to the real keychain");
        assert_eq!(
            get_api_key()
                .expect("read back from the real keychain")
                .as_deref(),
            Some(FIXTURE_SECRET)
        );
        delete_api_key().expect("clean up the real keychain entry");
        assert!(get_api_key().expect("read after delete").is_none());

        let _ = keyring_core::unset_default_store();
    }
}
