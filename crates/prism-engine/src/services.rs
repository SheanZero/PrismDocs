//! D-09 依赖倒置的落地点：`Engine` 实现 `prism-types` 的服务 trait。
//!
//! `prism-mcp` 只认 trait，具体实现在这一侧。于是 **Phase 6/7 注册新 MCP 工具时
//! 只需要在 `prism-types` 加 trait、在本文件加 impl，不动 `prism-mcp`**——
//! 依赖方向 `prism-engine ──▶ prism-mcp ──▶ prism-types` 保持单向，
//! 编译期不可能出现 facade↔mcp 环。
//!
//! # 两条纪律
//!
//! ## 同步实现，实现体内不得出现 `.await`
//!
//! trait 本身是同步的（底层 rusqlite 本就阻塞）。消费侧的 `prism-mcp` handler
//! 已经用 `tokio::task::spawn_blocking` 把调用包好了，所以这里做阻塞 IO 是安全的；
//! 反过来，一旦这里写出 `.await`，trait 就得变 async，`Arc<dyn …>` 的 object-safety
//! 随之失效——D-09 的注入形态就没了。
//!
//! ## 错误文本不回显调用参数
//!
//! 这些错误会经 MCP 响应回抛给**外部 agent**（T-01-04 / T-01-20）。
//! `ServiceError::Invalid` 携带的必须是描述**规则**的固定文本，不是违规的值本身。

use prism_types::{CommentSink, FeedbackItem, FeedbackSource, Receipt, ServiceError};

use crate::facade::Engine;

impl FeedbackSource for Engine {
    /// 列出某个项目待处理的反馈。
    ///
    /// **Phase 1 返回空集合，而且是 `Ok(vec![])` 不是 `Err(NotFound)`。**
    /// schema v1 按 D-04 的最小集边界还没有 comments 表（那是 Phase 5 的迁移），
    /// 但「这个项目还没有待处理反馈」在任何 phase 都是**正常状态**而非错误——
    /// 现在返回 `Err` 会让 Phase 6 的 agent 侧写出一条把 NotFound 当空集处理的
    /// 补偿分支，然后在真实实现上线那天变成死代码。
    ///
    /// 边界校验现在就位：它与"有没有 comments 表"无关，是这个方法**永远**要做的事，
    /// 也让 Phase 6 的真实实现有一个现成的落点。
    fn list_feedback(&self, project_id: &str) -> Result<Vec<FeedbackItem>, ServiceError> {
        if project_id.trim().is_empty() {
            // 只描述规则，不回显传入的值。
            return Err(ServiceError::Invalid("project id must not be empty".into()));
        }
        Ok(Vec::new())
    }
}

impl CommentSink for Engine {
    /// 记录 agent 对一条反馈的处理回执。
    ///
    /// Phase 1 只记日志（真实落库属于 Phase 6 的评论回流）。
    ///
    /// **日志里只有 comment_id 与 status，没有正文**（T-01-33）：回执正文来自外部
    /// agent，可能整段引用用户文档；一条 `?receipt` 就把它写进了本地日志文件。
    fn record_receipt(&self, receipt: Receipt) -> Result<(), ServiceError> {
        if receipt.comment_id.trim().is_empty() {
            return Err(ServiceError::Invalid("comment id must not be empty".into()));
        }
        tracing::info!(
            comment_id = %receipt.comment_id,
            status = %receipt.status,
            "recorded an agent receipt"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facade::Engine;
    use prism_store::Store;
    use std::sync::Arc;

    fn engine() -> (tempfile::TempDir, Engine) {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let store = Store::open(&dir.path().join("prismdocs.db")).expect("open store");
        (dir, Engine::new(Arc::new(store)))
    }

    #[test]
    fn no_pending_feedback_is_ok_not_an_error() {
        let (_dir, engine) = engine();
        let items = engine
            .list_feedback("p1")
            .expect("「还没有反馈」是正常状态，不该是 Err");
        assert!(items.is_empty(), "Phase 1 的 schema 还没有 comments 表");
    }

    /// 阴性对照：不做这一条，一个「无条件 `Ok(vec![])`」的实现也能让上面全绿，
    /// 而端到端注入测试就再也没有 engine 独有的数据可断言了。
    #[test]
    fn an_empty_project_id_is_rejected() {
        let (_dir, engine) = engine();
        let rejected = engine.list_feedback("");
        assert!(
            matches!(rejected, Err(ServiceError::Invalid(_))),
            "空 project_id 应被拒绝，实得 {rejected:?}"
        );

        // 全空白与空串同义——否则 "   " 会绕过校验。
        assert!(matches!(
            engine.list_feedback("   "),
            Err(ServiceError::Invalid(_))
        ));
    }

    /// 错误文本不得回显调用参数（T-01-20）：它会经 MCP 响应回抛给外部 agent。
    #[test]
    fn rejection_text_does_not_echo_the_caller_argument() {
        let (_dir, engine) = engine();
        let probe = "   ";
        let msg = engine
            .list_feedback(probe)
            .expect_err("should be rejected")
            .to_string();
        assert_eq!(
            msg, "invalid request: project id must not be empty",
            "校验文本漂移了——端到端注入测试正是靠这段固定文本判别 engine 是否真的被调用"
        );
    }

    #[test]
    fn record_receipt_accepts_a_well_formed_receipt() {
        let (_dir, engine) = engine();
        let receipt = Receipt {
            comment_id: "c-1".to_string(),
            status: "applied".to_string(),
        };
        assert!(engine.record_receipt(receipt).is_ok());
    }

    #[test]
    fn record_receipt_rejects_a_receipt_without_a_comment_id() {
        let (_dir, engine) = engine();
        let receipt = Receipt {
            comment_id: String::new(),
            status: "applied".to_string(),
        };
        assert!(matches!(
            engine.record_receipt(receipt),
            Err(ServiceError::Invalid(_))
        ));
    }

    /// 两个 impl 都必须是同步的（纪律 1）。
    ///
    /// `.await` 一旦出现，trait 就得变成 async，`Arc<dyn FeedbackSource>` 的
    /// object-safety 随之失效——D-09 的整个注入形态建立在它之上。
    #[test]
    fn the_service_impls_contain_no_await() {
        let source = include_str!("services.rs");
        let cut = source.find("#[cfg(test)]").unwrap_or(source.len());
        let production: String = source[..cut]
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            !production.contains(".await"),
            "service impl 里出现了 .await —— 同步 trait 的前提被破坏"
        );
        assert_eq!(
            production.matches("impl FeedbackSource for Engine").count(),
            1
        );
        assert_eq!(production.matches("impl CommentSink for Engine").count(), 1);
    }
}
