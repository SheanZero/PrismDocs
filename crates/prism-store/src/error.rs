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

    /// bundled SQLite 低于 `MIN_SQLITE`，或版本串畸形到无法证明它够新。只带版本串。
    #[error("bundled sqlite is too old: {0}")]
    SqliteTooOld(String),

    /// `PRAGMA journal_mode=WAL` 拿回来的不是 wal。
    ///
    /// SQLite 用**返回行**而不是错误告知结果模式：WAL 起不来时（网络卷、只读目录、
    /// `-shm` 建不出来）它返回 `delete`，语句本身是成功的。只带模式串——库路径是内部事实
    /// （T-01-20）。
    ///
    /// 这个变体**不需要**新的 IPC 短码：它落在 `EngineError::Store(_)` 的兜底臂上
    /// （`src-tauri/src/commands.rs::map_err` → `"store_error"`）。理由与 01-10 记录的决策同源
    /// ——拒绝面扩张时优先复用既有短码，加短码要连带改 IPC 契约与前端 `ERROR_COPY`。
    #[error("database is not in wal mode: {0}")]
    JournalModeNotWal(String),

    /// TRUNCATE checkpoint 报告 busy：仍有连接持有 WAL，`-wal` 里的内容没能搬回主库。
    ///
    /// 后果是数据完整性风险——备份走了主库却漏掉未 checkpoint 的部分。消息只陈述这条
    /// **规则性事实**，不带路径也不带连接数（T-01-20）。短码同 [`Self::JournalModeNotWal`]，
    /// 走 `EngineError::Store(_)` 兜底臂，IPC 契约不动。
    #[error("wal checkpoint did not complete: connections still hold the wal")]
    CheckpointBusy,

    /// 写入 `settings` 的键名不被接受（当前唯一的拒绝理由是「疑似密钥」）。
    /// 消息只带键名与规则，**不带 value**——被误填进来的很可能就是密钥本身（T-01-26）。
    #[error("invalid setting: {0}")]
    InvalidSetting(String),

    /// `base_url` 不是一个可接受的 URL。同样只描述规则，不回显传入的值。
    #[error("invalid url: {0}")]
    InvalidUrl(String),

    #[error("could not resolve the platform data directory")]
    NoDataDir,
}
