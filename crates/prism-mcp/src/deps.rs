//! 注入容器：D-09 的 trait 反转在运行期的落点。
//!
//! ## 为什么是 `Arc<dyn …>` 而不是泛型参数
//!
//! 泛型 `S: FeedbackSource` 会把 `prism-engine` 的具体类型经 `S` 泄漏进
//! `prism-mcp` 的公开签名（乃至 axum service 的类型），并让
//! `StreamableHttpService::new` 的 `'static` factory 闭包难写；`Arc<dyn …>`
//! 之下 factory 只是 clone 一个 Arc，本 crate 的公开类型保持单一具体形态。
//!
//! ## bearer token 的归属
//!
//! **prism-mcp 既不生成也不存储 token**——它只持有调用方注入的那一份。
//! Phase 1 由测试注入；Phase 6 由 `prism-engine` 从钥匙串
//! （`docs/keychain-naming.md` 的 `mcp_bearer_token`）读出后注入。
//! 密钥的唯一入口仍是 `prism-llm`，本 crate 不引入任何 keyring 依赖。

use std::sync::Arc;

use prism_types::{CommentSink, FeedbackSource};

/// MCP handler 可用的全部能力面 + 本次安装的 bearer token。
#[derive(Clone)]
pub struct McpDeps {
    pub feedback: Arc<dyn FeedbackSource>,
    pub comments: Arc<dyn CommentSink>,
    bearer: Arc<str>,
}

impl McpDeps {
    pub fn new(
        feedback: Arc<dyn FeedbackSource>,
        comments: Arc<dyn CommentSink>,
        bearer: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            feedback,
            comments,
            bearer: bearer.into(),
        }
    }

    /// token 原文的唯一取用口。
    ///
    /// 刻意不是 `pub` 字段也刻意不叫 `bearer()`：调用点在代码搜索中可见，
    /// 且中间件之外没有第二个消费者。返回 `&str` 而非 `String` 是为了不制造副本。
    pub(crate) fn expose_bearer(&self) -> &str {
        &self.bearer
    }
}

/// 手写 `Debug`：token 绝不能经 `?deps` / `{:?}` 进日志或错误文本（T-01-29 同源要求）。
/// 与 `prism_llm::secrets::ApiKey` 同一口径——**刻意不实现 `Display`**，
/// 让 `format!("{deps}")` 成为编译错误而不是运行期泄漏。
impl std::fmt::Debug for McpDeps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpDeps")
            .field("feedback", &"<dyn FeedbackSource>")
            .field("comments", &"<dyn CommentSink>")
            .field("bearer", &"<redacted>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism_types::{FeedbackItem, Receipt, ServiceError};

    struct Empty;

    impl FeedbackSource for Empty {
        fn list_feedback(&self, _project_id: &str) -> Result<Vec<FeedbackItem>, ServiceError> {
            Ok(Vec::new())
        }
    }

    impl CommentSink for Empty {
        fn record_receipt(&self, _receipt: Receipt) -> Result<(), ServiceError> {
            Ok(())
        }
    }

    fn deps(bearer: &str) -> McpDeps {
        McpDeps::new(Arc::new(Empty), Arc::new(Empty), bearer)
    }

    #[test]
    fn debug_does_not_reveal_the_bearer_token() {
        // 绑定名刻意不叫 `secret`：`scripts/check-secrets.sh` 抓的正是「名字像密钥的
        // 标识符后面跟一个引号串」这个形状，它没抓错。与 `prism-llm` 的 `FIXTURE_SECRET`
        // 和前端的 `FAKE_KEY` 同源——扫描器是防线，不该为了迁就测试局部变量而放宽。
        let fixture_bearer = "s3cr3t-token-value";
        let rendered = format!("{:?}", deps(fixture_bearer));
        assert!(
            !rendered.contains(fixture_bearer),
            "McpDeps Debug leaked the bearer token: {rendered}"
        );
        assert!(rendered.contains("<redacted>"), "{rendered}");
    }

    #[test]
    fn deps_are_cheap_to_clone_and_keep_the_same_token() {
        let d = deps("abc");
        let cloned = d.clone();
        assert_eq!(cloned.expose_bearer(), "abc");
    }
}
