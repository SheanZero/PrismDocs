//! 引擎事件：notify-then-fetch 的**粗粒度失效信号**载荷契约。
//!
//! 载荷保持小是这条通路相对 push-payload 的全部价值所在——只带 ID 与计数，
//! 绝不带文档正文。前端收到事件后自己回查 SQLite（唯一真相源），因此
//! 「事件丢了」可以用一次全量失效无损补偿，而 push-payload 丢的是不可恢复的数据。

use serde::{Deserialize, Serialize};

/// 引擎向壳层广播的失效信号。
///
/// 序列化形态是跨 IPC 边界的契约：内部标签字段名为 `kind`，变体名与字段名
/// 统一 camelCase，与前端 TypeScript 的判别联合一一对应。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum EngineEvent {
    /// 某个文档的内容或元数据发生变化。
    DocChanged { project_id: String, doc_id: String },

    /// 某个项目的收件箱未读数变化。
    InboxUpdated { project_id: String, unread: u32 },

    /// 全量失效。
    ///
    /// 专供总线 `broadcast::error::RecvError::Lagged` 时使用：慢接收者丢掉了
    /// 未知数量的消息，此时唯一安全的补偿是让前端把全部查询都作废重取
    /// （plan 01-08 的 shell adapter 消费这一变体）。
    Resync,
}
