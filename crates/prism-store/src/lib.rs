//! SQLite 持有层：本 workspace 中唯一打开 SQLite 连接的 crate。
//!
//! 对外只有三件东西：sidecar 路径解析（[`data_root`] / [`default_db_path`]）、
//! 库的打开流程与并发纪律（[`open`] / [`Store`]，见 `open.rs` 的 writer-first 六步序）、
//! schema 迁移集合（[`migrations`]），以及查询层（[`search`]，按长度分流的 trigram/LIKE 搜索）。

pub mod error;
pub mod migrations;
mod open;
pub mod search;
pub mod seed;
pub mod settings;

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

    /// `major.minor.patch` **按位**解析——与 `open.rs::parse_sqlite_version` 同源。
    ///
    /// 那一份在 `open()` 的准入判定路径上且是 `open` 模块私有的（本模块访问不到，
    /// 因此这里是副本而不是复用）；这一份只服务测试。两处都必须按位解析：
    /// `filter_map(|s| s.parse().ok())` 会丢掉不可解析的分量并把后面每一位左移一格，
    /// 于是 `3.x.53` 塌缩成 `(3, 53, 0)` —— 一个碰巧够新的元组（上轮 IN-03）。
    fn version_tuple(v: &str) -> Option<(u32, u32, u32)> {
        let mut parts = v.split('.').map(str::parse::<u32>);
        match (parts.next(), parts.next(), parts.next()) {
            (Some(Ok(major)), Some(Ok(minor)), Some(Ok(patch))) => Some((major, minor, patch)),
            _ => None,
        }
    }

    #[test]
    fn open_creates_missing_parent_directories() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let db_path = dir.path().join("nested").join("deeper").join("prismdocs.db");
        let store = Store::open(&db_path).expect("open store");
        assert!(db_path.exists(), "db file should have been created");
        drop(store);
    }

    /// 成功标准 3 的「bundled SQLite ≥3.51.3」在**运行期**的落点。
    ///
    /// 两条断言守两条不同的性质：形态（三段点分数字）与下界（≥ [`MIN_SQLITE`]）。
    /// 下界必须比到 patch 位——只判 `p[0] == 3` 时，pin 回退到 3.50.x 不会有任何东西变红
    /// （`01-VERIFICATION.md` 的第三条 warning）。期望值取自 `MIN_SQLITE` 而不是字面量：
    /// 这个数字在仓库里只有 `open.rs` 那一个来源，改 pin 时两条测试自动跟上。
    #[test]
    fn sqlite_version_meets_the_pinned_minimum() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let store = Store::open(&dir.path().join("prismdocs.db")).expect("open store");
        let version = store.sqlite_version().expect("version query");
        assert_eq!(
            version.split('.').count(),
            3,
            "unexpected sqlite version string: {version}"
        );
        let got = version_tuple(&version)
            .unwrap_or_else(|| panic!("unparsable sqlite version: {version}"));
        assert!(
            got >= MIN_SQLITE,
            "bundled SQLite {version} is older than the pinned minimum {MIN_SQLITE:?}"
        );
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
