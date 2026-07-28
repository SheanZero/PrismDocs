//! SQLite 持有层：本 workspace 中唯一打开 SQLite 连接的 crate。
//!
//! Phase 1 plan 01（tracer）只建立「能打开一个库并问它版本」这条最薄的真路；
//! WAL / pragma 套餐 / 迁移 / 只读池是 plan 03 的 writer-first 六步序，
//! 此处刻意留空位，不预先猜测其形态。

pub mod error;
pub mod migrations;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;

pub use error::StoreError;

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

pub struct Store {
    writer: Mutex<Connection>,
}

impl Store {
    /// 打开（必要时创建）单写者连接。
    ///
    /// tracer 阶段刻意不设 WAL / pragma / 迁移 / 只读池——那是 plan 03 的
    /// writer-first 六步序，顺序有语义，不能在这里提前猜。
    pub fn open(db_path: &Path) -> Result<Store, StoreError> {
        if let Some(dir) = db_path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let writer = Connection::open(db_path)?;
        Ok(Store {
            writer: Mutex::new(writer),
        })
    }

    pub fn sqlite_version(&self) -> Result<String, StoreError> {
        let conn = self.writer.lock().unwrap_or_else(|e| e.into_inner());
        let version = conn.query_row("SELECT sqlite_version()", [], |row| row.get(0))?;
        Ok(version)
    }
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
        assert!(root.ends_with(DATA_DIR_NAME), "unexpected data root: {root:?}");
    }

    #[test]
    fn default_db_path_lives_under_the_data_root() {
        let path = default_db_path().expect("db path");
        assert_eq!(path.file_name().unwrap(), DB_FILE_NAME);
        assert_eq!(path.parent().unwrap(), data_root().unwrap());
    }
}
