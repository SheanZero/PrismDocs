//! Phase 1 的最小 MCP handler：证明「数据经注入的 service trait 取得」这条通路通。
//!
//! 工具面的真实内容（`get_context_pack`、评论回流等）是 Phase 6 的事，本文件只有
//! 一个 `list_feedback`。它的意义不在功能，而在于 `tests/trait_injection.rs`
//! 可以断言响应里出现**假实现独有的那条数据**。
//!
//! **本文件不得出现任何 prism-engine 类型。** 该性质由 `scripts/check-deps.sh no-cycle`
//! 在依赖树层面守住（编译期），此处只是提醒。

use std::borrow::Cow;
use std::sync::Arc;

use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestMethod, CallToolRequestParams, CallToolResult, ContentBlock, ErrorData,
    ListToolsResult, PaginatedRequestParams, ProtocolVersion, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::RequestContext;
use rmcp::RoleServer;

use crate::deps::McpDeps;

/// Phase 1 唯一的工具名。Phase 6 在此追加，不改本文件的调用形态。
pub const TOOL_LIST_FEEDBACK: &str = "list_feedback";

/// `arguments` 里的 `projectId` 键名。与 [`PrismHandler::tool_descriptor`] 的 schema 同源。
const ARG_PROJECT_ID: &str = "projectId";

/// 按工具描述符声明的契约提取 `projectId`：**必须存在、必须是字符串、去空白后非空**。
///
/// ## 为什么校验必须长在这里
///
/// 这三条正是 `tool_descriptor()` 的 schema 已经写下的（`"required": ["projectId"]`
/// 与 `"type": "string"`）。声明了却不执行，等于把执行力外包给**被注入的实现**：
/// 今天 `Engine::list_feedback` 恰好也拒绝空 project id，于是缺失 / 非字符串 /
/// 纯空白一律被 `.unwrap_or_default()` 静默强转成空串之后，仍然被 engine 挡住。
///
/// 那份兜底不属于本文件，也不受本文件控制。Phase 6 的真实 `FeedbackSource` 若把
/// 空串当作「所有项目」（一个再自然不过的实现），这里当场变成一条跨项目数据泄漏，
/// 而 `handler.rs` 一行代码都不用改——缺陷会在别处引入、在这里生效。
/// `tests/trait_injection.rs::a_leaky_source_is_never_reached_with_an_invalid_project_id`
/// 把这条推理钉成可执行的：它注入的正是那样一个「空串照样给数据」的实现。
///
/// ## 错误类别
///
/// 返回 `invalid_params` 而不是 `internal_error`：一个格式错误的请求不是服务端故障，
/// 报成 internal error 会让调用方去重试而不是去修请求。
///
/// 消息是**规则形**的固定文本，不回显传入的值（T-01-04 / T-01-20）——`projectId`
/// 来自外部 agent，可能很长、也可能整段引用用户文档。
fn project_id_of(request: &CallToolRequestParams) -> Result<&str, ErrorData> {
    fn reject() -> ErrorData {
        ErrorData::invalid_params(
            "tool argument `projectId` must be a non-empty string",
            None,
        )
    }

    let value = request
        .arguments
        .as_ref()
        .and_then(|arguments| arguments.get(ARG_PROJECT_ID))
        .ok_or_else(reject)?;
    // `as_str()` 同时挡住数字 / null / 数组 / 对象——JSON 里除字符串外的一切。
    let project_id = value.as_str().ok_or_else(reject)?;
    if project_id.trim().is_empty() {
        return Err(reject());
    }
    // 返回**原值**而不是 trim 后的值：project id 是标识符，静默改写它会让
    // 「传进来的」与「查出来的」不是同一个东西。判空用 trim，取值不动。
    Ok(project_id)
}

/// 每个 MCP session 一个实例（由 `StreamableHttpService` 的 factory 闭包构造）。
pub struct PrismHandler {
    deps: McpDeps,
}

impl PrismHandler {
    pub fn new(deps: McpDeps) -> Self {
        Self { deps }
    }

    fn tool_descriptor() -> Tool {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                ARG_PROJECT_ID: { "type": "string", "description": "PrismDocs project id" }
            },
            "required": [ARG_PROJECT_ID]
        });
        let schema = match schema {
            serde_json::Value::Object(map) => map,
            _ => unreachable!("the literal above is an object"),
        };
        Tool::new(
            Cow::Borrowed(TOOL_LIST_FEEDBACK),
            Cow::Borrowed("List the pending feedback items of a PrismDocs project."),
            Arc::new(schema),
        )
    }
}

impl ServerHandler for PrismHandler {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.protocol_version = ProtocolVersion::LATEST;
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info.name = "prismdocs".into();
        info.server_info.version = env!("CARGO_PKG_VERSION").into();
        info
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(vec![
            Self::tool_descriptor(),
        ]))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        if request.name != TOOL_LIST_FEEDBACK {
            return Err(ErrorData::method_not_found::<CallToolRequestMethod>());
        }

        let project_id = project_id_of(&request)?.to_owned();

        // service trait 是**同步**的（prism-store 本就是阻塞 rusqlite），而这里是 async
        // 上下文——必须先 clone 出 Arc 再 spawn_blocking，不得在 async 里直接阻塞调用。
        let source = Arc::clone(&self.deps.feedback);
        let items = tokio::task::spawn_blocking(move || source.list_feedback(&project_id))
            .await
            .map_err(|_| ErrorData::internal_error("feedback lookup task failed", None))?
            .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;

        let payload = serde_json::to_string(&items)
            .map_err(|_| ErrorData::internal_error("failed to encode feedback items", None))?;

        Ok(CallToolResult::success(vec![ContentBlock::text(payload)]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::ErrorCode;

    /// 照**线上形态**构造：从 JSON 反序列化，而不是填结构体字段。
    /// `CallToolRequestParams` 还有 `_meta` / `task` 等字段，逐个填会把测试绑死在
    /// SDK 的字段集合上；而真实调用方本来就是从 JSON 来的。
    fn params(raw: serde_json::Value) -> CallToolRequestParams {
        serde_json::from_value(raw).expect("valid tools/call params")
    }

    fn with_arguments(arguments: serde_json::Value) -> CallToolRequestParams {
        params(serde_json::json!({
            "name": TOOL_LIST_FEEDBACK,
            "arguments": arguments,
        }))
    }

    #[test]
    fn a_non_empty_string_project_id_is_accepted() {
        let request = with_arguments(serde_json::json!({ "projectId": "p1" }));
        assert_eq!(project_id_of(&request).expect("accepted"), "p1");
    }

    /// schema 声明 `"required": ["projectId"]` 与 `"type": "string"`。
    /// 六种违反声明的输入必须在**提取处**被拒，且类别是 invalid-params。
    #[test]
    fn every_shape_the_schema_forbids_is_rejected_as_invalid_params() {
        let cases = [
            (
                params(serde_json::json!({ "name": TOOL_LIST_FEEDBACK })),
                "完全没有 arguments",
            ),
            (with_arguments(serde_json::json!({})), "arguments 是空对象"),
            (
                with_arguments(serde_json::json!({ "project_id": "p1" })),
                "拼错的键名（schema 里的键是 projectId）",
            ),
            (
                with_arguments(serde_json::json!({ "projectId": 42 })),
                "projectId 是数字",
            ),
            (
                with_arguments(serde_json::json!({ "projectId": null })),
                "projectId 是 null",
            ),
            (
                with_arguments(serde_json::json!({ "projectId": "   " })),
                "projectId 是纯空白",
            ),
        ];

        for (request, label) in cases {
            let error = project_id_of(&request).expect_err(label);
            assert_eq!(
                error.code,
                ErrorCode::INVALID_PARAMS,
                "{label}: 一个格式错误的请求被报成了别的类别（{:?}）",
                error.code
            );
        }
    }

    /// T-01-04 / T-01-20：错误文本只陈述规则，不回显传入的值。
    #[test]
    fn the_rejection_text_does_not_echo_the_offending_value() {
        // 可识别的长标记：`projectId` 来自外部 agent，可能整段引用用户文档。
        let marker = "Z".repeat(200);
        // 两种**必然被拒**且把标记带在值里的形状。
        let cases = [
            with_arguments(serde_json::json!({ "projectId": [marker.clone()] })),
            with_arguments(serde_json::json!({ "projectId": { "value": marker.clone() } })),
        ];

        for request in cases {
            let error = project_id_of(&request).expect_err("非字符串的 projectId");
            assert!(
                !error.message.contains(&marker),
                "错误文本回显了传入的值: {}",
                error.message
            );
            assert!(
                error
                    .data
                    .as_ref()
                    .is_none_or(|data| !data.to_string().contains(&marker)),
                "错误的 data 字段回显了传入的值"
            );
        }
    }
}
