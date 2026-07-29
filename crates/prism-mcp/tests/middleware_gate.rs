//! D-07 的三层门禁：Host（DNS-rebinding 防护）/ Origin allowlist / bearer。
//!
//! ## 为什么有两组测试
//!
//! **A 组（端到端）** 打真实的 `serve_loopback` 路由，证明部署形态下三种非法请求都被拒、
//! 全合法请求能到达 mcp service。
//!
//! **B 组（隔离）** 才是每一层各自的反证。原因是 rmcp 2.2 的
//! `StreamableHttpService` **自己也校验 Host**（GHSA-89vp-x53w-74fx 的上游修复，
//! 默认 `allowed_hosts = [localhost, 127.0.0.1, ::1]`），本项目又刻意把 SDK 侧
//! 的 allowlist 与应用层配成同一份做防御纵深。因此在 A 组里把
//! `require_local_host` 从 `build_router` 摘掉，`rejects_foreign_host` **不会变红**
//! ——SDK 会替它拒掉。那样的「反证」证明的是「有人拒了」，不是「这一层拒的」。
//!
//! B 组把每一层单独叠在一个 sentinel handler 上（链路里没有 rmcp），
//! 并在同一个测试里对比「挂了这层」与「没挂这层」：没挂时请求直达 sentinel。
//! 这样断言失败的落点唯一——只可能是被测的那一层没生效。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::{from_fn, from_fn_with_state, Next};
use axum::Router;
use prism_mcp::deps::McpDeps;
use prism_mcp::middleware::{require_bearer, require_local_host, require_origin_allowlist};
use prism_mcp::server::{serve_loopback, MCP_MOUNT_PATH};
use prism_types::{CommentSink, FeedbackItem, FeedbackSource, Receipt, ServiceError};
use serde_json::json;
use tokio_util::sync::CancellationToken;
use tower::ServiceExt;

const GOOD_BEARER: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
/// 与 `GOOD_BEARER` **等长**、仅末位不同：比较不得因长度短路而放行。
const WRONG_SAME_LEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdee";
/// 长度不同的错误 token。
const WRONG_SHORT: &str = "0123456789abcdef";

const SENTINEL: &str = "sentinel-reached";

struct EmptySource;

impl FeedbackSource for EmptySource {
    fn list_feedback(&self, _project_id: &str) -> Result<Vec<FeedbackItem>, ServiceError> {
        Ok(Vec::new())
    }
}

impl CommentSink for EmptySource {
    fn record_receipt(&self, _receipt: Receipt) -> Result<(), ServiceError> {
        Ok(())
    }
}

fn deps() -> McpDeps {
    McpDeps::new(Arc::new(EmptySource), Arc::new(EmptySource), GOOD_BEARER)
        .expect("GOOD_BEARER is a non-empty 32-byte hex constant")
}

// ---------------------------------------------------------------------------
// A 组：端到端，打真实的 serve_loopback 路由
// ---------------------------------------------------------------------------

async fn serve() -> (std::net::SocketAddr, CancellationToken) {
    let ct = CancellationToken::new();
    let (addr, _task) = serve_loopback(deps(), ct.clone())
        .await
        .expect("loopback server started");
    (addr, ct)
}

/// 一次合法的 `initialize` 请求体——三层全过时它应当到达 mcp service。
fn init_body() -> String {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": prism_mcp::protocol_version(),
            "capabilities": {},
            "clientInfo": { "name": "prism-middleware-gate-test", "version": "0.0.0" }
        }
    })
    .to_string()
}

/// 发一次请求；`host` / `origin` / `authorization` 三项按需替换成非法值。
async fn probe(
    addr: &std::net::SocketAddr,
    host: Option<&str>,
    origin: Option<&str>,
    authorization: Option<&str>,
) -> StatusCode {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .expect("build test client");

    let mut req = client
        .post(format!("http://{addr}{MCP_MOUNT_PATH}"))
        .header(reqwest::header::ACCEPT, "application/json, text/event-stream")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(init_body());

    // 不传 host 时用 reqwest 自动填的 `127.0.0.1:<port>`（合法）。
    if let Some(host) = host {
        req = req.header(reqwest::header::HOST, host);
    }
    if let Some(origin) = origin {
        req = req.header(reqwest::header::ORIGIN, origin);
    }
    if let Some(authorization) = authorization {
        req = req.header(reqwest::header::AUTHORIZATION, authorization);
    }

    req.send().await.expect("request reached the server").status()
}

fn good_auth() -> String {
    format!("Bearer {GOOD_BEARER}")
}

#[tokio::test]
async fn rejects_foreign_host() {
    let (addr, ct) = serve().await;
    let status = probe(
        &addr,
        Some("evil.example.com"),
        Some("http://127.0.0.1"),
        Some(&good_auth()),
    )
    .await;
    ct.cancel();

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "Origin 与 bearer 均合法、只有 Host 是外域，却没有 403"
    );
}

#[tokio::test]
async fn rejects_foreign_origin() {
    let (addr, ct) = serve().await;
    let status = probe(
        &addr,
        None,
        Some("https://evil.example.com"),
        Some(&good_auth()),
    )
    .await;
    ct.cancel();

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "Host 与 bearer 均合法、只有 Origin 是外域，却没有 403"
    );
}

#[tokio::test]
async fn rejects_missing_or_wrong_bearer() {
    let (addr, ct) = serve().await;

    let missing = probe(&addr, None, Some("http://127.0.0.1"), None).await;
    let wrong_same_len = probe(
        &addr,
        None,
        Some("http://127.0.0.1"),
        Some(&format!("Bearer {WRONG_SAME_LEN}")),
    )
    .await;
    let wrong_short = probe(
        &addr,
        None,
        Some("http://127.0.0.1"),
        Some(&format!("Bearer {WRONG_SHORT}")),
    )
    .await;
    let not_bearer = probe(
        &addr,
        None,
        Some("http://127.0.0.1"),
        Some(&format!("Basic {GOOD_BEARER}")),
    )
    .await;
    ct.cancel();

    assert!(missing.is_client_error(), "无 Authorization 头却未被拒: {missing}");
    assert!(
        wrong_same_len.is_client_error(),
        "等长错误 token 未被拒: {wrong_same_len}"
    );
    assert!(
        wrong_short.is_client_error(),
        "短错误 token 未被拒: {wrong_short}"
    );
    assert!(
        not_bearer.is_client_error(),
        "非 Bearer scheme 未被拒: {not_bearer}"
    );
}

/// A 组的阴性对照：三层不是把所有请求都拒了。
#[tokio::test]
async fn accepts_fully_valid_request() {
    let (addr, ct) = serve().await;
    let status = probe(&addr, None, Some("http://127.0.0.1"), Some(&good_auth())).await;
    ct.cancel();

    assert!(
        !status.is_client_error(),
        "三层头全合法的请求仍被 4xx 拒绝: {status}"
    );
}

// ---------------------------------------------------------------------------
// B 组：隔离反证。链路里只有 sentinel handler 与被测的那一层。
// ---------------------------------------------------------------------------

fn sentinel_router() -> Router {
    Router::new().route(MCP_MOUNT_PATH, axum::routing::post(|| async { SENTINEL }))
}

fn request(host: &str, origin: Option<&str>, authorization: Option<&str>) -> Request<Body> {
    let mut builder = axum::http::Request::builder()
        .method("POST")
        .uri(MCP_MOUNT_PATH)
        .header(axum::http::header::HOST, host);
    if let Some(origin) = origin {
        builder = builder.header(axum::http::header::ORIGIN, origin);
    }
    if let Some(authorization) = authorization {
        builder = builder.header(axum::http::header::AUTHORIZATION, authorization);
    }
    builder.body(Body::empty()).expect("valid test request")
}

async fn oneshot(router: Router, req: Request<Body>) -> (StatusCode, String) {
    let response = router.oneshot(req).await.expect("router is infallible");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("read response body");
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

#[tokio::test]
async fn host_layer_alone_is_what_rejects_a_foreign_host() {
    let guarded = sentinel_router().layer(from_fn(require_local_host));
    let (status, body) = oneshot(guarded, request("evil.example.com", None, None)).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "require_local_host 未拦下外域 Host");
    assert!(!body.contains(SENTINEL), "请求仍到达了 handler: {body}");

    // 反证（落点唯一）：摘掉这一层，同一请求直达 sentinel。
    let bare = sentinel_router();
    let (status, body) = oneshot(bare, request("evil.example.com", None, None)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains(SENTINEL),
        "没挂中间件时请求也没到 handler —— 上面的 403 可能另有来源"
    );

    // 合法 Host 放行。
    let guarded = sentinel_router().layer(from_fn(require_local_host));
    let (status, body) = oneshot(guarded, request("127.0.0.1:51234", None, None)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(SENTINEL));
}

#[tokio::test]
async fn origin_layer_alone_is_what_rejects_a_foreign_origin() {
    let guarded = sentinel_router().layer(from_fn(require_origin_allowlist));
    let (status, body) = oneshot(
        guarded,
        request("127.0.0.1:51234", Some("https://evil.example.com"), None),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "require_origin_allowlist 未拦下外域 Origin"
    );
    assert!(!body.contains(SENTINEL), "请求仍到达了 handler: {body}");

    let bare = sentinel_router();
    let (status, body) = oneshot(
        bare,
        request("127.0.0.1:51234", Some("https://evil.example.com"), None),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(SENTINEL));

    // 无 Origin 头（非浏览器客户端，如 Claude Code）放行到下一层。
    let guarded = sentinel_router().layer(from_fn(require_origin_allowlist));
    let (status, body) = oneshot(guarded, request("127.0.0.1:51234", None, None)).await;
    assert_eq!(status, StatusCode::OK, "无 Origin 的非浏览器客户端被误拒");
    assert!(body.contains(SENTINEL));

    // Tauri WebView 的 Origin 在 allowlist 内。
    let guarded = sentinel_router().layer(from_fn(require_origin_allowlist));
    let (status, _) = oneshot(
        guarded,
        request("127.0.0.1:51234", Some("tauri://localhost"), None),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn bearer_layer_alone_is_what_rejects_a_bad_token() {
    let cases = [
        (None, "缺失 Authorization"),
        (Some(format!("Bearer {WRONG_SAME_LEN}")), "等长但内容不同"),
        (Some(format!("Bearer {WRONG_SHORT}")), "长度不同"),
        (Some(format!("Bearer {GOOD_BEARER}extra")), "正确前缀加后缀"),
        (Some(format!("Basic {GOOD_BEARER}")), "scheme 不是 Bearer"),
    ];

    for (authorization, label) in cases {
        let guarded = sentinel_router().layer(from_fn_with_state(deps(), require_bearer));
        let (status, body) = oneshot(
            guarded,
            request("127.0.0.1:51234", None, authorization.as_deref()),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{label} 未被拒");
        assert!(!body.contains(SENTINEL), "{label}: 请求仍到达了 handler");

        // 反证：同一请求在没有这一层时直达 sentinel。
        let bare = sentinel_router();
        let (status, body) = oneshot(
            bare,
            request("127.0.0.1:51234", None, authorization.as_deref()),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{label}");
        assert!(body.contains(SENTINEL), "{label}");
    }

    // 正确 token 放行。
    let guarded = sentinel_router().layer(from_fn_with_state(deps(), require_bearer));
    let (status, body) = oneshot(
        guarded,
        request("127.0.0.1:51234", None, Some(&good_auth())),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(SENTINEL));
}

/// 拒绝响应体不得透露「是哪一层挂了」（T-01-29）。
#[tokio::test]
async fn rejections_do_not_disclose_which_layer_denied() {
    let host_denied = oneshot(
        sentinel_router().layer(from_fn(require_local_host)),
        request("evil.example.com", None, None),
    )
    .await;
    let origin_denied = oneshot(
        sentinel_router().layer(from_fn(require_origin_allowlist)),
        request("127.0.0.1:1", Some("https://evil.example.com"), None),
    )
    .await;
    let bearer_denied = oneshot(
        sentinel_router().layer(from_fn_with_state(deps(), require_bearer)),
        request("127.0.0.1:1", None, None),
    )
    .await;

    assert_eq!(host_denied, origin_denied);
    assert_eq!(origin_denied, bearer_denied);
    assert!(
        host_denied.1.is_empty(),
        "拒绝响应带了正文: {}",
        host_denied.1
    );
}

/// token 绝不出现在响应里（即使 token 本身是正确的）。
#[tokio::test]
async fn the_bearer_token_never_appears_in_a_response() {
    for authorization in [
        None,
        Some(good_auth()),
        Some(format!("Bearer {WRONG_SAME_LEN}")),
    ] {
        let guarded = sentinel_router().layer(from_fn_with_state(deps(), require_bearer));
        let (_, body) = oneshot(
            guarded,
            request("127.0.0.1:1", None, authorization.as_deref()),
        )
        .await;
        assert!(
            !body.contains(GOOD_BEARER) && !body.contains(WRONG_SAME_LEN),
            "响应体回显了 token: {body}"
        );
    }
}

/// 关闭 RESEARCH Assumptions Log 的 A5：`.layer()` 后加的先执行（由内向外叠加）。
#[tokio::test]
async fn layers_run_outermost_first_meaning_last_added_runs_first() {
    let inner_ran = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&inner_ran);

    let router = sentinel_router()
        // 先加 => 内层
        .layer(from_fn(move |req: Request, next: Next| {
            let flag = Arc::clone(&flag);
            async move {
                flag.store(true, Ordering::SeqCst);
                next.run(req).await
            }
        }))
        // 后加 => 外层，应当先执行并直接拒掉
        .layer(from_fn(require_local_host));

    let (status, _) = oneshot(router, request("evil.example.com", None, None)).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(
        !inner_ran.load(Ordering::SeqCst),
        "先 .layer() 的那层被执行了 —— 叠加方向与 A5 的假设相反，build_router 的注释需修正"
    );
}
