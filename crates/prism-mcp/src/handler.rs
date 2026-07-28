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
                "projectId": { "type": "string", "description": "PrismDocs project id" }
            },
            "required": ["projectId"]
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

        let project_id = request
            .arguments
            .as_ref()
            .and_then(|args| args.get("projectId"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_owned();

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
