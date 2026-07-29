//! D-09 的**运行期**证据：prism-mcp 经注入的 `Arc<dyn FeedbackSource>` 取数据。
//!
//! 编译期证据（依赖树里没有 prism-engine）由 `scripts/check-deps.sh no-cycle` 承担，
//! 本文件补的是另一半——「通路真的通」而不只是「能编译」。
//!
//! `injected_feedback_source_is_reached` 断言的是**假实现独有的那条数据**，
//! `empty_source_yields_no_item` 是它的阴性对照：换一个返回空 vec 的实现，
//! 同一请求的响应体里那条数据就消失。没有这条对照，一个直接返回硬编码常量、
//! 根本不调用注入 trait 的 handler 也能让第一个测试通过。

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use prism_mcp::deps::McpDeps;
use prism_mcp::server::{serve_loopback, MCP_MOUNT_PATH};
use prism_types::{CommentSink, FeedbackItem, FeedbackSource, Receipt, ServiceError};
use serde_json::json;
use tokio_util::sync::CancellationToken;

/// 32 字节 hex —— 形状与 Phase 6 的 CSPRNG token 一致，值本身是测试常量。
const TEST_BEARER: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

/// 假实现独有的标记：它只可能来自注入的 `FeedbackSource`。
const MARKER_ID: &str = "fb-1";

struct FixedFeedback(Vec<FeedbackItem>);

impl FeedbackSource for FixedFeedback {
    fn list_feedback(&self, _project_id: &str) -> Result<Vec<FeedbackItem>, ServiceError> {
        Ok(self.0.clone())
    }
}

struct NoopSink;

impl CommentSink for NoopSink {
    fn record_receipt(&self, _receipt: Receipt) -> Result<(), ServiceError> {
        Ok(())
    }
}

fn deps_returning(items: Vec<FeedbackItem>) -> McpDeps {
    McpDeps::new(
        Arc::new(FixedFeedback(items)),
        Arc::new(NoopSink),
        TEST_BEARER,
    )
    .expect("TEST_BEARER is a non-empty 32-byte hex constant")
}

fn marker_item() -> FeedbackItem {
    FeedbackItem {
        id: MARKER_ID.to_string(),
        doc_id: "doc-1".to_string(),
        body: "needs a decision".to_string(),
    }
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .expect("build test client")
}

fn endpoint(addr: &SocketAddr) -> String {
    format!("http://{addr}{MCP_MOUNT_PATH}")
}

async fn post(
    client: &reqwest::Client,
    addr: &SocketAddr,
    session: Option<&str>,
    body: serde_json::Value,
) -> reqwest::Response {
    let mut req = client
        .post(endpoint(addr))
        .header(reqwest::header::ACCEPT, "application/json, text/event-stream")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::ORIGIN, "http://127.0.0.1")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {TEST_BEARER}"),
        )
        .body(body.to_string());
    if let Some(session) = session {
        req = req.header("mcp-session-id", session);
    }
    req.send().await.expect("request reached the server")
}

/// 完整走一次 MCP 握手，再调一次 `list_feedback` 工具，返回工具响应的原始文本。
async fn call_list_feedback(addr: &SocketAddr) -> String {
    let client = client();

    let init = post(
        &client,
        addr,
        None,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": prism_mcp::protocol_version(),
                "capabilities": {},
                "clientInfo": { "name": "prism-trait-injection-test", "version": "0.0.0" }
            }
        }),
    )
    .await;
    assert!(
        init.status().is_success(),
        "initialize failed: {}",
        init.status()
    );
    let session = init
        .headers()
        .get("mcp-session-id")
        .expect("server issued a session id")
        .to_str()
        .expect("session id is ascii")
        .to_string();
    let _ = init.text().await;

    let ack = post(
        &client,
        addr,
        Some(&session),
        json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
    )
    .await;
    assert!(
        ack.status().is_success(),
        "initialized notification failed: {}",
        ack.status()
    );
    let _ = ack.text().await;

    let call = post(
        &client,
        addr,
        Some(&session),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "list_feedback",
                "arguments": { "projectId": "proj-1" }
            }
        }),
    )
    .await;
    assert!(
        call.status().is_success(),
        "tools/call failed: {}",
        call.status()
    );
    call.text().await.expect("tool response body")
}

#[tokio::test]
async fn injected_feedback_source_is_reached() {
    let ct = CancellationToken::new();
    let (addr, _task) = serve_loopback(deps_returning(vec![marker_item()]), ct.clone())
        .await
        .expect("loopback server started");

    let body = call_list_feedback(&addr).await;
    ct.cancel();

    assert!(
        body.contains(MARKER_ID),
        "response did not carry the injected implementation's own datum ({MARKER_ID}): {body}"
    );
}

/// 阴性对照：数据来自注入的实现，不是 handler 内的常量。
#[tokio::test]
async fn empty_source_yields_no_item() {
    let ct = CancellationToken::new();
    let (addr, _task) = serve_loopback(deps_returning(Vec::new()), ct.clone())
        .await
        .expect("loopback server started");

    let body = call_list_feedback(&addr).await;
    ct.cancel();

    assert!(
        !body.contains(MARKER_ID),
        "an empty FeedbackSource still produced {MARKER_ID} — the handler is not reading the injected trait: {body}"
    );
}

/// T-01-28：只绑 loopback；端口由 OS 分配（端口策略是 Phase 6 的事）。
#[tokio::test]
async fn serve_loopback_binds_to_loopback_on_an_os_assigned_port() {
    let ct = CancellationToken::new();
    let (addr, _task) = serve_loopback(deps_returning(vec![marker_item()]), ct.clone())
        .await
        .expect("loopback server started");
    ct.cancel();

    assert_eq!(addr.ip().to_string(), "127.0.0.1", "bound to {addr}");
    assert_ne!(addr.port(), 0, "OS did not assign a concrete port");
}
