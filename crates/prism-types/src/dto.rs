//! 跨边界共享的数据载体。
//!
//! Phase 1 只承载 plan 01-06 假数据 handler 所需的最小字段；字段扩充留给
//! Phase 6（真实的评论回流与工具面）。**先窄后宽**是有意的：过早铺开字段
//! 等于替 Phase 6 做推测式设计。

use serde::{Deserialize, Serialize};

/// 一条待 agent 处理的反馈（评论 / 需决策项）。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackItem {
    pub id: String,
    pub doc_id: String,
    pub body: String,
}

/// agent 对一条反馈的处理回执。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    pub comment_id: String,
    pub status: String,
}

/// 一条搜索命中。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub doc_id: String,
    pub title: Option<String>,
    pub rel_path: String,
}
