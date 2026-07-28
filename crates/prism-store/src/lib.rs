//! SQLite 持有层：本 workspace 中唯一打开 SQLite 连接的 crate。
//!
//! 对外只有三件东西：sidecar 路径解析（[`data_root`] / [`default_db_path`]）、
//! 库的打开流程与并发纪律（[`open`] / [`Store`]，见 `open.rs` 的 writer-first 六步序）、
//! schema 迁移集合（[`migrations`]），以及查询层（[`search`]，按长度分流的 trigram/LIKE 搜索）。

pub mod error;
pub mod migrations;
mod open;
pub mod search;

use std::path::PathBuf;

pub use error::StoreError;
pub use open::{open, Store, BUSY_TIMEOUT_MS, MIN_SQLITE, READ_POOL_MAX_SIZE};
pub use search::{escape_fts_query, search, MIN_TRIGRAM_CHARS};

/// sidecar 数据根的目录名（D-13：`~/Library/Application Support/PrismDocs/`）。
const DATA_DIR_NAME: &str = "PrismDocs";
const DB_FILE_NAME: &str = "prismdocs.db";

/// sidecar 数据根。**必须**来自 `dirs::data_dir()`——Tauri 的等价 API 会让本 crate
/// 依赖 tauri，违反 D-01。
pub fn data_root() -> Result<PathBuf, StoreError> {
    let base = dirs::data_dir().ok_or(StoreError::NoDataDir)?;
    Ok(base.join(DATA_DIR_NAME))
}

pub fn default_db_path() -> Result<PathBuf, StoreError> {
    Ok(data_root()?.join(DB_FILE_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts(v: &str) -> Vec<u32> {
        v.split('.').filter_map(|s| s.parse().ok()).collect()
    }

    #[test]
    fn open_creates_missing_parent_directories() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let db_path = dir.path().join("nested").join("deeper").join("prismdocs.db");
        let store = Store::open(&db_path).expect("open store");
        assert!(db_path.exists(), "db file should have been created");
        drop(store);
    }

    #[test]
    fn sqlite_version_returns_three_dotted_numbers() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let store = Store::open(&dir.path().join("prismdocs.db")).expect("open store");
        let version = store.sqlite_version().expect("version query");
        let p = parts(&version);
        assert_eq!(p.len(), 3, "unexpected sqlite version string: {version}");
        assert_eq!(p[0], 3, "expected SQLite 3.x, got {version}");
    }

    #[test]
    fn data_root_ends_with_prismdocs() {
        let root = data_root().expect("data root");
        assert!(
            root.ends_with(DATA_DIR_NAME),
            "unexpected data root: {root:?}"
        );
    }

    #[test]
    fn default_db_path_lives_under_the_data_root() {
        let path = default_db_path().expect("db path");
        assert_eq!(path.file_name().unwrap(), DB_FILE_NAME);
        assert_eq!(path.parent().unwrap(), data_root().unwrap());
    }
}
