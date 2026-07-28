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
use crate::middleware::{
    require_bearer, require_local_host, require_origin_allowlist, ALLOWED_HOSTS, ALLOWED_ORIGINS,
};
use crate::McpError;

/// MCP 端点挂载路径。Phase 6 的端口发现契约会引用它。
pub const MCP_MOUNT_PATH: &str = "/mcp";

/// **只绑 127.0.0.1**（T-01-28）。端口 0 = 由 OS 分配；端口策略是 Phase 6 的事。
pub const LOOPBACK_BIND: &str = "127.0.0.1:0";

/// 组装路由：rmcp service + Host / Origin / bearer 三层中间件。
///
/// ## 中间件顺序（RESEARCH Assumptions Log A5，已实测）
///
/// axum 的 `.layer()` 由内向外叠加——**后 `.layer()` 的先执行**。下面的写法下请求
/// 依次经过 `require_local_host` → `require_origin_allowlist` → `require_bearer`
/// → mcp service：先做 DNS-rebinding 防护，再做鉴权。
/// 该方向由 `middleware_gate.rs::layers_run_outermost_first_meaning_last_added_runs_first`
/// 实测锁死，而不是靠读文档推断。
///
/// 顺序只影响「先被哪一层拒」，**不影响安全性**——三层各自独立成立，
/// `tests/middleware_gate.rs` 的 B 组对每层都有一条落点唯一的隔离反证。
///
/// ## 与 SDK 自带校验的关系（防御纵深，不是冗余）
///
/// rmcp 2.2 的 `StreamableHttpServerConfig` 自己也有 `allowed_hosts` / `allowed_origins`
/// （GHSA-89vp-x53w-74fx 的上游修复；其中 `allowed_origins` 默认为空 = 不校验）。
/// 这里把两处配成同一份 allowlist：应用层在外先拒（403 空正文，不透露是哪一层，
/// T-01-29），SDK 层在内兜底。**SDK 的修复不替代应用层 allowlist**——它可被一行
/// `disable_allowed_hosts()` 关掉，且 SDK 大版本变更时应用层这三条不动。
pub fn build_router(deps: McpDeps, ct: CancellationToken) -> Router {
    let factory_deps = deps.clone();
    let service = StreamableHttpService::new(
        move || Ok(PrismHandler::new(factory_deps.clone())),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default()
            .with_cancellation_token(ct.child_token())
            .with_allowed_hosts(ALLOWED_HOSTS)
            .with_allowed_origins(ALLOWED_ORIGINS),
    );

    Router::new()
        .nest_service(MCP_MOUNT_PATH, service)
        .layer(axum::middleware::from_fn_with_state(deps, require_bearer))
        .layer(axum::middleware::from_fn(require_origin_allowlist))
        .layer(axum::middleware::from_fn(require_local_host))
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
