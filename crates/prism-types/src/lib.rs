//! 零第三方依赖（仅 serde + thiserror）的契约 crate —— **D-09 的 trait 落点**。
//!
//! 依赖方向（编译期强制，见 `scripts/check-deps.sh no-cycle`）：
//!
//! ```text
//! prism-engine ──▶ prism-mcp ──▶ prism-types
//!       │                            ▲
//!       └────────────────────────────┘
//! ```
//!
//! 本 crate 是依赖图的汇点，`prism-mcp` 与 `prism-engine` 都指向它，因此
//! **任何新增依赖都会同时压到两侧**。`Cargo.toml` 的 `[dependencies]` 只允许
//! serde 与 thiserror；`async-trait` / `tokio` / `anyhow` / 任何 engine crate 都不得进入。

pub mod dto;
pub mod event;
pub mod service;

pub use dto::{FeedbackItem, Receipt, SearchHit};
pub use event::EngineEvent;
pub use service::{CommentSink, FeedbackSource, ServiceError};

pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// 编译期的 object-safety 证明：若哪天有人给 trait 加了泛型方法或
    /// `async fn`，这个 `Arc<dyn …>` 会直接编译失败，而不是等到 Phase 6
    /// 在 rmcp handler 里才发现 trait 不能被 dyn 化。
    struct FakeSource;

    impl FeedbackSource for FakeSource {
        fn list_feedback(&self, _project_id: &str) -> Result<Vec<FeedbackItem>, ServiceError> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn feedback_source_can_be_used_behind_arc_dyn() {
        let source: Arc<dyn FeedbackSource> = Arc::new(FakeSource);
        assert!(source
            .list_feedback("any")
            .expect("fake never fails")
            .is_empty());
    }

    #[test]
    fn crate_version_is_populated() {
        assert!(!CRATE_VERSION.is_empty());
    }
}
