//! D-07 的宿主形态：rmcp `StreamableHttpService` 挂在 loopback axum 0.8 上。
//!
//! 挂载形态逐行照 rmcp 官方示例 `examples/servers/src/counter_streamhttp.rs`：
//! `nest_service` + `LocalSessionManager` + `StreamableHttpServerConfig`。
//! 会话管理与 SSE 帧一律交给 SDK，**不手写**。
//!
//! 三层应用级中间件（Host / Origin / bearer）由 `crate::middleware` 叠加。

use std::net::SocketAddr;

use axum::Router;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::deps::McpDeps;
use crate::handler::PrismHandler;
use crate::McpError;

/// MCP 端点挂载路径。Phase 6 的端口发现契约会引用它。
pub const MCP_MOUNT_PATH: &str = "/mcp";

/// **只绑 127.0.0.1**（T-01-28）。端口 0 = 由 OS 分配；端口策略是 Phase 6 的事。
pub const LOOPBACK_BIND: &str = "127.0.0.1:0";

/// 组装路由：rmcp service（三层中间件在 plan 01-06 Task 2 叠上）。
pub fn build_router(deps: McpDeps, ct: CancellationToken) -> Router {
    let service = StreamableHttpService::new(
        move || Ok(PrismHandler::new(deps.clone())),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default().with_cancellation_token(ct.child_token()),
    );

    Router::new().nest_service(MCP_MOUNT_PATH, service)
}

/// 在 loopback 上起服务，返回 OS 实际分配到的地址与后台任务句柄。
///
/// `ct` 取消时 axum 优雅关停，SDK 侧的会话也随 child token 一并终止（T-01-30）。
pub async fn serve_loopback(
    deps: McpDeps,
    ct: CancellationToken,
) -> Result<(SocketAddr, JoinHandle<()>), McpError> {
    let listener = tokio::net::TcpListener::bind(LOOPBACK_BIND).await?;
    let addr = listener.local_addr()?;
    let router = build_router(deps, ct.clone());

    let task = tokio::spawn(async move {
        let shutdown = async move { ct.cancelled().await };
        if let Err(err) = axum::serve(listener, router)
            .with_graceful_shutdown(shutdown)
            .await
        {
            tracing::warn!(error = %err, "loopback MCP server stopped");
        }
    });

    Ok((addr, task))
}
