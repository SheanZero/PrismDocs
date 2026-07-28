//! LLM 调用层：**本 workspace 中唯一被允许声明 reqwest / keyring 依赖的 engine crate**。
//!
//! NFR-03：API key 的唯一入口在这里，密钥只存系统钥匙串（keyring 直连，不用 stronghold）。
//! Phase 1 plan 01（D-08）只建立骨架与真实依赖声明——密钥读写、端点配置与
//! SSE 流式解析是 plan 04 及后续 phase 的内容。

pub mod secrets;

/// crate 版本，供 UA 与诊断输出使用。
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LlmError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// 钥匙串操作失败。
    ///
    /// **威胁模型 T-01-04（Information Disclosure）**：这里刻意只保留
    /// `keyring_core::Error` 的 **Display 文本**，而不是错误值本身。该类型的
    /// `Display` 已核实不含密钥（`BadEncoding` 只写 "not valid UTF-8"），但它
    /// derive 出来的 `Debug` 会打印 `BadEncoding(Vec<u8>)` / `BadDataFormat(Vec<u8>, _)`
    /// 携带的**原始密钥字节**——而 `unwrap()`、`expect()` 与 `tracing` 的 `?err`
    /// 走的都是 `Debug`。带 payload 转发等于留一条泄漏通道。
    #[error("keychain error: {0}")]
    Keychain(String),
}

impl From<keyring_core::Error> for LlmError {
    fn from(err: keyring_core::Error) -> Self {
        Self::Keychain(err.to_string())
    }
}

/// 发往用户配置的 LLM 端点时使用的 User-Agent。
pub fn user_agent() -> String {
    format!("PrismDocs/{CRATE_VERSION} (prism-llm)")
}

/// macOS Keychain 后端的类型名。
///
/// 选 `keychain` 模块而非 `protected`：后者需要 provisioning profile，
/// 直发公证 DMG 的签名形态下会得到 `PlatformError -34018`（RESEARCH Pitfall 2）。
pub fn keychain_backend_name() -> &'static str {
    std::any::type_name::<apple_native_keyring_store::keychain::Store>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_agent_carries_the_crate_version() {
        let ua = user_agent();
        assert!(ua.starts_with("PrismDocs/"), "unexpected UA: {ua}");
        assert!(ua.contains(CRATE_VERSION), "unexpected UA: {ua}");
    }

    #[test]
    fn keychain_backend_is_the_apple_native_keychain_store() {
        let name = keychain_backend_name();
        assert!(
            name.starts_with("apple_native_keyring_store::keychain::"),
            "unexpected keychain backend: {name}"
        );
    }

    /// 依赖真的可用：构造 HTTP 客户端会初始化 TLS 后端（不发任何请求）。
    #[test]
    fn an_http_client_can_be_built_with_our_user_agent() {
        let client = reqwest::Client::builder()
            .user_agent(user_agent())
            .build()
            .map_err(LlmError::from)
            .expect("build http client");
        drop(client);
    }

    /// 依赖真的可用：serde derive 在本 crate 的 feature 解析下可编译。
    #[test]
    fn serde_derive_is_available_for_future_payloads() {
        #[derive(serde::Serialize, serde::Deserialize)]
        struct Probe {
            model: String,
        }

        let probe = Probe {
            model: "probe".to_string(),
        };
        assert_eq!(probe.model, "probe");
    }
}
