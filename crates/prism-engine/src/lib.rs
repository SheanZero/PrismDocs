//! 编排层（facade）：shell 与 MCP 之外的一切编排都发生在这里。
//!
//! **不依赖 tauri**（D-01）——这是本 crate 存在的理由：Tauri shell 只是薄壳，
//! 所有编排在这一层，因此这一层必须能在没有 tauri 的情况下被完整测试
//! （`cargo test -p prism-engine` 的输出里不出现 `Compiling tauri v`）。
//!
//! 三个模块各守一条边界：
//!
//! * [`bus`] —— 事件总线（engine 唯一的订阅点）
//! * [`facade`] —— [`Engine`]：单写者句柄的持有者与三条门面路径的编排
//! * [`services`] —— `impl FeedbackSource / CommentSink for Engine`（D-09 的注入面）

pub mod bus;
pub mod error;
pub mod facade;
pub mod services;

pub use bus::{EventBus, BUS_CAPACITY};
pub use error::EngineError;
pub use facade::Engine;

#[cfg(test)]
mod tests {
    use super::*;
    use prism_store::Store;
    use std::sync::Arc;

    fn engine_on_temp_db() -> (tempfile::TempDir, Engine) {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let store = Store::open(&dir.path().join("prismdocs.db")).expect("open store");
        (dir, Engine::new(Arc::new(store)))
    }

    #[test]
    fn ping_delegates_to_the_store_rather_than_returning_a_constant() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let store = Arc::new(Store::open(&dir.path().join("prismdocs.db")).expect("open store"));
        let expected = store.sqlite_version().expect("store version");
        let engine = Engine::new(Arc::clone(&store));
        assert_eq!(engine.ping().expect("ping"), expected);
    }

    #[test]
    fn ping_returns_a_real_sqlite_version_string() {
        let (_dir, engine) = engine_on_temp_db();
        let version = engine.ping().expect("ping");
        let parts: Vec<u32> = version.split('.').filter_map(|s| s.parse().ok()).collect();
        assert_eq!(parts.len(), 3, "unexpected version string: {version}");
    }

    #[test]
    fn engine_exposes_the_shared_types_crate_version() {
        let (_dir, engine) = engine_on_temp_db();
        assert!(!engine.types_crate_version().is_empty());
    }
}
