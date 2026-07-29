//! 门面错误类型。
//!
//! **威胁模型 T-01-20a（Information Disclosure）**：`EngineError` 会经命令层
//! （plan 01-08）一路回传到前端，因此 `Display` 中**不得**出现数据库文件的绝对路径、
//! 密钥原文或用户文档片段。
//!
//! 两个下层错误都已在各自 crate 里被收紧过，所以这里 `#[error(transparent)]`
//! 转发是安全的——但这是**继承来的**性质，不是自动成立的：
//!
//! * `StoreError::Io` 刻意不携带 `PathBuf`（只转发 `std::io::Error` 的 errno 描述）
//! * `LlmError::Keychain` 只保留 `keyring_core::Error` 的 Display 文本，
//!   不转发错误值本身（后者的 derive `Debug` 会打印原始密钥字节）
//!
//! 新增变体时必须重新核对这一条：`transparent` 会把下层 Display 原样放行。

use prism_llm::LlmError;
use prism_store::StoreError;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EngineError {
    #[error(transparent)]
    Store(#[from] StoreError),

    #[error(transparent)]
    Llm(#[from] LlmError),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 数据库路径不得经错误文本外泄（T-01-20a）。
    ///
    /// 断言的形态是「路径根本进不去」：`StoreError::Io` 由 `std::io::Error` 转来，
    /// 而后者的 Display 只有 errno 描述。若哪天有人给下层变体塞回一个 `PathBuf`，
    /// 这条会红。
    #[test]
    fn error_display_does_not_carry_a_filesystem_path() {
        let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission denied");
        let err = EngineError::from(StoreError::from(io));

        let shown = err.to_string();
        assert!(
            !shown.contains("PrismDocs/") && !shown.contains(".db"),
            "EngineError 的 Display 泄漏了库文件路径: {shown}"
        );
        assert!(
            shown.contains("Permission denied"),
            "错误类别本身仍应可读: {shown}"
        );
    }

    /// `transparent` 转发不得在下层文本之外**追加**任何东西。
    ///
    /// 这条守的是「有人给变体加了 `#[error("engine failed on {path}: {0}")]`」这类改动——
    /// 那正是路径与密钥重新溜进错误文本的典型方式。
    #[test]
    fn variants_forward_the_lower_layer_text_verbatim() {
        let store = StoreError::InvalidUrl("scheme must be one of [\"http\", \"https\"]".into());
        let expected = store.to_string();
        assert_eq!(EngineError::from(store).to_string(), expected);

        let llm = LlmError::Keychain("Password data is not valid UTF-8".into());
        let expected = llm.to_string();
        assert_eq!(EngineError::from(llm).to_string(), expected);
    }
}
