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
///
/// 返回**状态码与响应体**（形态照 B 组的 `oneshot`）。只返回状态码的版本无法表达
/// T-01-29 的一半——「403 且**空正文**」里的正文那一半（01-REVIEW.md WR-03）。
async fn probe(
    addr: &std::net::SocketAddr,
    host: Option<&str>,
    origin: Option<&str>,
    authorization: Option<&str>,
) -> (StatusCode, String) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .expect("build test client");

    let mut req = client
        .post(format!("http://{addr}{MCP_MOUNT_PATH}"))
        .header(
            reqwest::header::ACCEPT,
            "application/json, text/event-stream",
        )
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

    let response = req.send().await.expect("request reached the server");
    let status = response.status();
    let body = response.text().await.expect("read response body");
    (status, body)
}

fn good_auth() -> String {
    format!("Bearer {GOOD_BEARER}")
}

/// T-01-29 的拒绝形态：403 **且空正文**。两半都要断言。
///
/// 只断言 `is_client_error()` 会让 401 / 400 / 404 全部算通过，而状态码分化正是
/// 本契约禁止的那件事（01-REVIEW.md WR-03）；只断言状态码会漏掉 SDK 那种
/// 「403 但带正文点名层次」的形态。
#[track_caller]
fn assert_denied_uniformly(observed: &(StatusCode, String), what: &str) {
    assert_eq!(
        observed.0,
        StatusCode::FORBIDDEN,
        "{what}: 拒绝的状态码不是 403 —— 状态码分化会把三层试探变成逐层试探"
    );
    assert!(
        observed.1.is_empty(),
        "{what}: 拒绝响应带了正文: {}",
        observed.1
    );
}

#[tokio::test]
async fn rejects_foreign_host() {
    let (addr, ct) = serve().await;
    let observed = probe(
        &addr,
        Some("evil.example.com"),
        Some("http://127.0.0.1"),
        Some(&good_auth()),
    )
    .await;
    ct.cancel();

    assert_denied_uniformly(&observed, "Origin 与 bearer 均合法、只有 Host 是外域");
}

#[tokio::test]
async fn rejects_foreign_origin() {
    let (addr, ct) = serve().await;
    let observed = probe(
        &addr,
        None,
        Some("https://evil.example.com"),
        Some(&good_auth()),
    )
    .await;
    ct.cancel();

    assert_denied_uniformly(&observed, "Host 与 bearer 均合法、只有 Origin 是外域");
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

    // 收紧自 `is_client_error()`：那条断言对 401 是绿的，而「bearer 缺失单独用 401」
    // 正是 T-01-29 刻意不做的事——它告诉攻击者前两层已经过了。
    assert_denied_uniformly(&missing, "无 Authorization 头");
    assert_denied_uniformly(&wrong_same_len, "等长错误 token");
    assert_denied_uniformly(&wrong_short, "短错误 token");
    assert_denied_uniformly(&not_bearer, "非 Bearer scheme");
}

/// A 组的阴性对照：三层不是把所有请求都拒了。
///
/// 断言 `is_success()` 而不是 `!is_client_error()`：后者对 500 / 502 同样是绿的，
/// 而这条测试**知道成功长什么样**，没有理由用宽松形态（01-REVIEW-prior.md WR-14）。
/// 收紧之后它同时成为「请求真的到达了 mcp service」的证据——三层放行不等于下游能跑。
#[tokio::test]
async fn accepts_fully_valid_request() {
    let (addr, ct) = serve().await;
    let (status, _) = probe(&addr, None, Some("http://127.0.0.1"), Some(&good_auth())).await;
    ct.cancel();

    assert!(
        status.is_success(),
        "三层头全合法的请求没有得到 2xx: {status}"
    );
}

/// `rejections_do_not_disclose_which_layer_denied` 的**实发路由**版本。
///
/// 那一条跑的是三个只在测试里存在的 sentinel router，因此它守的是「每一层各自的
/// 拒绝形态」；它**碰不到** `serve_loopback` 实际组装出来的链路，也就碰不到 rmcp
/// SDK 内层那套「403 带正文 / 400 带正文」的拒绝（01-REVIEW.md WR-03）。两者互补：
/// 那一条证明每层自己的形态，这一条证明发货形态下三种非法请求**逐字节同形**。
#[tokio::test]
async fn the_shipped_route_denies_every_layer_identically() {
    let (addr, ct) = serve().await;

    // 每次只让一项非法，其余两项合法 —— 这样被拒的确实是不同的那一层。
    let host_denied = probe(
        &addr,
        Some("evil.example.com"),
        Some("http://127.0.0.1"),
        Some(&good_auth()),
    )
    .await;
    let origin_denied = probe(
        &addr,
        None,
        Some("https://evil.example.com"),
        Some(&good_auth()),
    )
    .await;
    let bearer_denied = probe(
        &addr,
        None,
        Some("http://127.0.0.1"),
        Some(&format!("Bearer {WRONG_SAME_LEN}")),
    )
    .await;
    // 口径分叉的那个形态（01-17 Task 1）：本层与 SDK 曾经对它算出不同主机名，
    // SDK 会用一个**带正文的 403** 拒它。它必须与上面三条逐字节相同。
    let split_parse_denied = probe(
        &addr,
        Some("127.0.0.1:80@evil.com"),
        Some("http://127.0.0.1"),
        Some(&good_auth()),
    )
    .await;
    ct.cancel();

    assert_eq!(
        host_denied, origin_denied,
        "非法 Host 与非法 Origin 的响应不同 —— 实发路由上存在层次 oracle"
    );
    assert_eq!(
        origin_denied, bearer_denied,
        "非法 Origin 与非法 bearer 的响应不同 —— 实发路由上存在层次 oracle"
    );
    assert_eq!(
        bearer_denied, split_parse_denied,
        "解析口径分叉的 Host 拿到了与其余三条不同的响应 —— SDK 内层的带正文拒绝可达了"
    );
    assert_denied_uniformly(&host_denied, "实发路由");
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
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "require_local_host 未拦下外域 Host"
    );
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

    // 阴性对照表（缺了它，「把 require_bearer 改成有 Authorization 头就放行」也能让
    // 上面的拒绝表全绿——那不是门禁，是一块写着门禁的牌子）。
    //
    // 三行都是 RFC 合规客户端的真实形态：RFC 7235 §2.1 定义 auth-scheme 为**大小写
    // 不敏感**的 token，RFC 6750 的客户端合法地发 `bearer` / `BEARER`；同一条语法
    // 里 scheme 与 credentials 之间允许 `1*SP`。Phase 6 的 MCP 客户端（Claude Code
    // 与其他 agent）不受本项目控制，而本门禁刻意无诊断——一个合规客户端与一次攻击
    // 在它面前完全同形，误拒的诊断成本因此极高（01-REVIEW.md WR-07）。
    let accepted = [
        (
            format!("bearer {GOOD_BEARER}"),
            "小写 scheme —— RFC 7235 §2.1 的 auth-scheme 大小写不敏感",
        ),
        (
            format!("BEARER {GOOD_BEARER}"),
            "大写 scheme —— 同上，比较不得是字节精确的",
        ),
        (
            format!("Bearer   {GOOD_BEARER}"),
            "scheme 与 credentials 之间多个空格 —— RFC 7235 允许 1*SP",
        ),
    ];

    for (authorization, label) in accepted {
        let guarded = sentinel_router().layer(from_fn_with_state(deps(), require_bearer));
        let (status, body) = oneshot(
            guarded,
            request("127.0.0.1:51234", None, Some(&authorization)),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{label}: 合规客户端被误拒");
        assert!(body.contains(SENTINEL), "{label}: 请求没能到达 handler");
    }
}

/// CR-03 的现实攻击形态：向一个**合法配置**的门禁发 `Authorization: Bearer `
/// （Bearer 之后是空 token）。解析层对它给出 scheme = `Bearer`、credentials = 空串
/// （旧形态的 `strip_prefix("Bearer ")` 同样给出 `Some("")`），
/// 因此空呈递值不会在解析阶段被挡掉，一路走到比较层——那里必须拒。
#[tokio::test]
async fn an_empty_presented_token_is_denied_by_the_bearer_layer_alone() {
    const EMPTY_PRESENTED: &str = "Bearer ";

    let guarded = sentinel_router().layer(from_fn_with_state(deps(), require_bearer));
    let (status, body) = oneshot(
        guarded,
        request("127.0.0.1:51234", None, Some(EMPTY_PRESENTED)),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "空呈递 token 未被 require_bearer 拒绝"
    );
    assert!(!body.contains(SENTINEL), "请求仍到达了 handler: {body}");
    // 拒绝形态与其余两层完全一致（T-01-29）：403 且空正文。
    assert!(body.is_empty(), "拒绝响应带了正文: {body}");

    // 反证（落点唯一）：摘掉这一层，同一请求直达 sentinel。
    let bare = sentinel_router();
    let (status, body) = oneshot(
        bare,
        request("127.0.0.1:51234", None, Some(EMPTY_PRESENTED)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains(SENTINEL),
        "没挂中间件时请求也没到 handler —— 上面的 403 可能另有来源"
    );

    // 合法 token 放行。
    let guarded = sentinel_router().layer(from_fn_with_state(deps(), require_bearer));
    let (status, body) = oneshot(
        guarded,
        request("127.0.0.1:51234", None, Some(&good_auth())),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(SENTINEL));
}

/// 纵深的另一半（CR-03）：「配置为空的门禁」这个对象在本 crate 之外根本造不出来。
///
/// 刻意**不经** `deps()` 助手——它带 `.expect(...)`，断言会落在 panic 上而不是返回值上。
#[test]
fn an_empty_configured_bearer_cannot_be_constructed_in_the_first_place() {
    assert!(
        McpDeps::new(Arc::new(EmptySource), Arc::new(EmptySource), "").is_err(),
        "空配置构造出了一个门禁"
    );
    assert!(
        McpDeps::new(Arc::new(EmptySource), Arc::new(EmptySource), "   ").is_err(),
        "纯空白配置构造出了一个门禁"
    );

    // 阴性对照：守卫不是「一律拒绝构造」。
    assert!(
        McpDeps::new(Arc::new(EmptySource), Arc::new(EmptySource), GOOD_BEARER).is_ok(),
        "合法配置也被拒绝构造了"
    );
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
        // 小写 scheme 走的是**放行**分支，也不得回显 token。
        Some(format!("bearer {GOOD_BEARER}")),
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
