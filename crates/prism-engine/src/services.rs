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

/// 回执 `status` 的受控取值集合。
///
/// `Receipt.status` 是从 MCP 线上直接反序列化的 `String`——完全不可信的外部输入。
/// 01-13 给进程装上 subscriber 之后，`record_receipt` 的那行日志是**真的会写出去**的；
/// 一个不受约束的外部字段可以把整段用户文档、或含嵌入换行的伪造日志行写进本地日志
/// 文件（T-01G-18 / T-01G-19）。受控集合让这两类输入在抵达日志之前就被拒。
///
/// **精确匹配，不做大小写折叠**：线协议值应当是确定的小写 token，`Applied` 与
/// `applied` 之间的宽容只会把「哪些值合法」这件事重新变成不确定的。
///
/// **与 Phase 5 的衔接点**：真实的评论状态机要到 Phase 5（COMMENT-03）才定案。
/// 届时把取值集合上移到 `prism-types`、把 `Receipt.status` 做成 enum 是自然的下一步。
/// 现在不做，是因为那会连带改 `prism-types/src/dto.rs` 的 serde 形态与 `prism-mcp`
/// 三个测试文件的构造点；上移的条件是 COMMENT-03 定下真实状态机。
const RECEIPT_STATUSES: [&str; 3] = ["applied", "rejected", "deferred"];

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
    ///
    /// 同一条推理逐字适用于 `status` 本身——它也是外部字段——所以它在被记录之前
    /// 必须落在 [`RECEIPT_STATUSES`] 里。
    fn record_receipt(&self, receipt: Receipt) -> Result<(), ServiceError> {
        if receipt.comment_id.trim().is_empty() {
            return Err(ServiceError::Invalid("comment id must not be empty".into()));
        }
        // 顺序不可调换：这条校验必须排在下面的 tracing::info! 之前。写在日志之后的
        // 实现在功能上「也拒了」，但被拒的那个值已经进了日志——那正是本条要防的事。
        // 顺序由 `the_service_impls_contain_no_await` 里的源码断言看住。
        if !RECEIPT_STATUSES.contains(&receipt.status.as_str()) {
            // 只描述规则，不回显传入的值（文件头第二条纪律 / T-01-20）。
            return Err(ServiceError::Invalid(
                "status is not a recognised value".into(),
            ));
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

    /// 受控取值集合的两端：三个 token 全部放行，空串不放行。
    ///
    /// 刻意写死字面量而不是遍历 `RECEIPT_STATUSES`——遍历常量的版本在常量被清空时
    /// 依然全绿（零次迭代），那正是本条要防的退化。
    #[test]
    fn every_controlled_receipt_status_is_accepted_and_an_empty_one_is_not() {
        let (_dir, engine) = engine();
        for status in ["applied", "rejected", "deferred"] {
            let receipt = Receipt {
                comment_id: "c-1".to_string(),
                status: status.to_string(),
            };
            assert!(
                engine.record_receipt(receipt).is_ok(),
                "受控取值 {status} 被拒了"
            );
        }

        let empty = Receipt {
            comment_id: "c-1".to_string(),
            status: String::new(),
        };
        assert!(matches!(
            engine.record_receipt(empty),
            Err(ServiceError::Invalid(_))
        ));
    }

    /// 拒绝文本逐字固定，且不回显传入的 status（T-01G-20）。
    ///
    /// 形态照抄 `rejection_text_does_not_echo_the_caller_argument`：这段文本会经
    /// MCP 响应回抛给外部 agent，回显被拒的值等于把外泄面从日志挪到响应，而不是关掉它。
    /// 探针取大小写不符的 `Applied`——受控集合是精确匹配，不做大小写折叠。
    #[test]
    fn status_rejection_text_does_not_echo_the_caller_argument() {
        let (_dir, engine) = engine();
        let probe = "Applied";
        let msg = engine
            .record_receipt(Receipt {
                comment_id: "c-1".to_string(),
                status: probe.to_string(),
            })
            .expect_err("大小写不符的 status 应被拒——线协议值是确定的小写 token")
            .to_string();
        assert_eq!(
            msg, "invalid request: status is not a recognised value",
            "校验文本漂移了——拒绝文本必须只陈述规则"
        );
        assert!(!msg.contains(probe), "拒绝文本回显了传入的 status: {msg}");
    }

    /// 含嵌入换行的超长 status 在抵达日志行之前就被拒（T-01G-18 / T-01G-19）。
    ///
    /// 这是 WR-13 的原始攻击面：`Receipt.status` 直接从 MCP 线上反序列化，
    /// 一个外部 agent 可以把整段用户文档、或伪造的额外日志行塞进这个字段。
    #[test]
    fn record_receipt_rejects_a_long_multiline_status() {
        let (_dir, engine) = engine();
        let forged = format!(
            "applied\n{}\nINFO forged log line: {}",
            "user document body ".repeat(1024),
            "x".repeat(4096)
        );
        let msg = engine
            .record_receipt(Receipt {
                comment_id: "c-1".to_string(),
                status: forged,
            })
            .expect_err("含换行的超长 status 应被拒")
            .to_string();
        assert_eq!(msg, "invalid request: status is not a recognised value");
        assert!(!msg.contains('\n'), "拒绝文本里出现了换行: {msg}");
        assert!(
            !msg.contains("user document body"),
            "拒绝文本回显了被拒的正文: {msg}"
        );
        assert!(
            !msg.contains("forged log line"),
            "拒绝文本回显了伪造的日志行: {msg}"
        );
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

        // status 校验必须排在回执日志行之前。一个把校验写在日志之后的实现在功能上
        // 「也拒了」——上面那几条行为测试全都仍然会绿——但被拒的那个值已经进了日志，
        // 那正是 WR-13 要防的事。行为测试对顺序没有判别力，所以顺序由这里看住。
        //
        // 锚点取完整语句而非裸标识符：`RECEIPT_STATUSES` 单独出现在常量声明处，
        // 用它做锚点会锚到声明而不是校验点。
        let guard = production
            .find("!RECEIPT_STATUSES.contains(&receipt.status.as_str())")
            .expect("status 受控取值校验不见了——外部字段会直接进日志");
        let log = production
            .find("\"recorded an agent receipt\"")
            .expect("回执日志行不见了");
        assert!(
            guard < log,
            "status 校验被排到了 tracing::info! 之后：被拒的值仍会先写进日志"
        );
    }
}
