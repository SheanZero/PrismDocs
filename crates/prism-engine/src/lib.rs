//! Facade：shell 与 MCP 之外的一切编排都发生在这里。
//!
//! **不依赖 tauri**（D-01）。Phase 1 plan 01（tracer）只提供 `new` / `ping`，
//! 用来证明 shell → facade → store → SQLite 这条方向是真的委托而非硬编码。

use std::sync::Arc;

use prism_store::{Store, StoreError};

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    Store(#[from] StoreError),
}

pub struct Engine {
    store: Arc<Store>,
}

impl Engine {
    pub fn new(store: Arc<Store>) -> Engine {
        Engine { store }
    }

    /// 端到端探针：把 store 报告的 SQLite 版本原样送回调用方。
    pub fn ping(&self) -> Result<String, EngineError> {
        Ok(self.store.sqlite_version()?)
    }

    pub fn types_crate_version(&self) -> &'static str {
        prism_types::CRATE_VERSION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
