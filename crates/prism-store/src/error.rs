//! `prism-store` 的错误类型。
//!
//! **威胁模型 T-01-20（Information Disclosure）**：这些错误会经 facade 与命令层回传前端，
//! 因此 `Display` 中**不得**出现数据库文件的绝对路径或任何用户文档片段——只带类别与
//! 底层错误码。需要路径的场合由调用方在本地日志侧自行拼接，不走错误类型。

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StoreError {
    /// 准备数据目录 / 打开库文件时的 IO 失败。
    ///
    /// `std::io::Error` 的 `Display` 只给出 errno 描述（如 `Permission denied (os error 13)`），
    /// 不含路径——这正是 T-01-20 要的。**不要**把 `PathBuf` 塞进这个变体。
    #[error("filesystem error while preparing the data directory: {0}")]
    Io(#[from] std::io::Error),

    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// r2d2 只读池的建池 / 取用失败。
    #[error("read pool error: {0}")]
    Pool(#[from] r2d2::Error),

    /// `rusqlite_migration` 的迁移失败。携带的是迁移 SQL 与版本号（schema 事实），非用户内容。
    #[error("schema migration failed: {0}")]
    Migration(#[from] rusqlite_migration::Error),

    /// bundled SQLite 低于 `MIN_SQLITE`。只带版本串。
    #[error("bundled sqlite is too old: {0}")]
    SqliteTooOld(String),

    #[error("could not resolve the platform data directory")]
    NoDataDir,
}
