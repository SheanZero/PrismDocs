//! 服务 trait：D-09 的 trait 反转落点。
//!
//! `prism-mcp` 只依赖本 crate，`prism-engine` 实现这些 trait 并以 `Arc<dyn …>`
//! 注入——依赖方向因此是单向的，编译期不可能出现 facade↔mcp 环。
//!
//! **Phase 6/7 新增 MCP 工具时，在此文件追加 trait，不动 prism-mcp。**
//!
//! ## 为什么是同步 trait
//!
//! 底层 `prism-store` 基于 rusqlite，本来就是阻塞的；async 在这里是伪需求。
//! 同步 trait 天然 object-safe（`Arc<dyn FeedbackSource>` 直接可用），不触碰
//! AFIT 的 dyn-safety 问题，也让本 crate 不需要 `async-trait` 依赖。
//! rmcp 的 async handler 在自己的上下文里用 `tokio::task::spawn_blocking` 调用即可。

use crate::dto::{FeedbackItem, Receipt};

/// 读取某个项目待处理的反馈。
pub trait FeedbackSource: Send + Sync + 'static {
    fn list_feedback(&self, project_id: &str) -> Result<Vec<FeedbackItem>, ServiceError>;
}

/// 记录 agent 对一条反馈的处理回执。
pub trait CommentSink: Send + Sync + 'static {
    fn record_receipt(&self, receipt: Receipt) -> Result<(), ServiceError>;
}

/// 服务层错误。
///
/// **威胁模型 T-01-04 / T-01-20（Information Disclosure）**：这些错误会经 MCP 响应
/// 回抛给外部 agent，因此 `Display` **不得回显调用方传入的原始参数**（project_id、
/// 查询串、文档片段等）——否则就成了一条把内部标识与用户内容外送的隐蔽通道。
/// `Invalid` 携带的必须是实现方自己写死的说明文本，不是调用参数。
///
/// ## 后端失败的变体：等第一个真实调用方
///
/// 这里曾有一个 `Backend(String)` 变体，供存储 / 解析 / 文件系统失败之用，
/// 但工作区里没有任何构造点——两个 `FeedbackSource` / `CommentSink` 实现与全部测试
/// 都不构造它。一个既没人造也没人接的变体不是「预留」，是死代码：它让 `match` 多一条
/// 永不触发的臂，也让「这层会返回哪些错误」这件事失真。这与 `prismdocs-helper doctor`
/// 当初「给 `HelperError` 的变体一个构造点以便 `clippy -D warnings` 通过」是同一条
/// 推理的另一半——要么给它构造点，要么不要它。
///
/// **重新引入的条件**：Phase 5（评论落库）或 Phase 6（评论回流）里第一个真实会失败的
/// 调用方出现时，就地加回来。届时其文本仍须由实现方写死、不含调用参数——rusqlite /
/// io 的原始错误串可能带路径与 SQL 片段，不能直接 `to_string()` 塞进去。
///
/// 本枚举是 `#[non_exhaustive]` 的，加回变体对 crate 外的 `match` 无影响。
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ServiceError {
    /// 请求的对象不存在。刻意不携带任何标识——「不存在」本身已是全部信息。
    #[error("requested resource was not found")]
    NotFound,

    /// 请求本身不合法。文本描述**哪条规则**被违反，不回显违规的值。
    #[error("invalid request: {0}")]
    Invalid(String),
}
