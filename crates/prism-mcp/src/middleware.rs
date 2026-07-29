//! D-07 的三层门禁：Host / Origin allowlist / bearer，缺一即拒。
//!
//! 三层**各自独立成立**，不靠执行顺序保证安全性——顺序只决定「先被哪一层拒」。
//! `tests/middleware_gate.rs` 的 B 组对每层都有一条落点唯一的隔离反证。
//!
//! ## 统一的拒绝形态（T-01-29）
//!
//! 三层一律返回 **403 且空正文**。刻意不给 bearer 缺失单独用 401：状态码或正文的
//! 任何差异都会告诉攻击者「你已经过了哪几层」，把三层试探变成逐层试探。
//! 真实原因只进本地 `tracing`，不进响应。
//!
//! ### 这句话为什么在**实发路由**上也成立（01-REVIEW.md WR-03）
//!
//! `build_router` 刻意把 rmcp SDK 侧的 allowlist 配成同一份做防御纵深，而 **SDK 的
//! 拒绝不是无差别的**：rmcp 2.2 `transport/streamable_http_server/tower.rs` 会回
//! `forbidden_response("Forbidden: Host header is not allowed")`（403 **带正文**，
//! 423 行）与 `bad_request_response("Bad Request: Invalid Host header")`（**400**
//! 带正文，400 行）。两层都可达的成因是**解析口径不同**——SDK 用
//! `http::uri::Authority::try_from`（RFC 3986，认 userinfo），本层曾经只取第一个
//! `:` 之前的东西。两个实测可达的输入：
//!
//! - `Host: 127.0.0.1:80/evil` —— 本层给出 `127.0.0.1`（放行），SDK 解析失败 → **400 + 正文**
//! - `Host: 127.0.0.1:80@evil.com` —— 本层给出 `127.0.0.1`（放行），SDK 按 userinfo
//!   取到 `evil.com` → **403 + 正文**（状态码相同、正文不同，同样是层次 oracle）
//!
//! 因此 [`host_of`] 现在要求**两侧对同一个主机名达成一致**（01-17 选定的路径 A）：
//! 不一致即在本层以 403 空正文拒掉，SDK 那两种带正文的形态在门禁面上不再可达，
//! 上面那句话端到端为真。该性质由 `host_of_agrees_with_the_sdk_parser_or_denies`
//! （源码级）与 `middleware_gate.rs::the_shipped_route_denies_every_layer_identically`
//! （实发路由级）两条测试钉住。
//!
//! **范围**：本契约覆盖的是**三层门禁**的拒绝。SDK 的协议级错误（Accept 头不合规、
//! 未知 session id 等）形态不同，但它们只在**通过 bearer 之后**可达，不构成未鉴权
//! 的层次 oracle。

use axum::extract::{Request, State};
use axum::http::uri::Authority;
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use subtle::ConstantTimeEq;

use crate::deps::McpDeps;

/// Host 头允许的主机名（端口不限——loopback 端口由 OS 分配）。
///
/// `[::1]` 形式的 Host 在比较前会被剥掉方括号，因此这里存无括号形式。
pub const ALLOWED_HOSTS: [&str; 3] = ["127.0.0.1", "localhost", "::1"];

/// Origin 允许的 (scheme, host) 组合，**端口不限**。
///
/// `tauri://localhost` 是 Tauri WebView 的 Origin；其余三项是本机浏览器/工具。
pub const ALLOWED_ORIGINS: [&str; 4] = [
    "http://127.0.0.1",
    "http://localhost",
    "http://[::1]",
    "tauri://localhost",
];

/// 无差别拒绝：403 + 空正文。
fn deny(reason: &'static str) -> Response {
    tracing::warn!(reason, "rejected an MCP request at the loopback gate");
    StatusCode::FORBIDDEN.into_response()
}

/// 从 `Host` / Origin 的 authority 中取出主机名（剥端口、剥 IPv6 方括号），小写化。
///
/// **本层的判定必须与 rmcp SDK 内层就同一个主机名达成一致**（模块头 T-01-29 那一节）：
/// 两侧算出不同主机名、或 SDK 那侧根本解析不出 authority 的输入，一律在这里拒掉。
/// 少了这道闸门，就存在「过了本层、被 SDK 以另一种形态拒掉」的输入，而 SDK 的拒绝
/// 带正文且状态码分化——那正是 T-01-29 禁止的可观测差异。
fn host_of(authority: &str) -> Option<String> {
    let authority = authority.trim();
    if authority.is_empty() {
        return None;
    }

    let ours = if let Some(rest) = authority.strip_prefix('[') {
        // IPv6 字面量：`[::1]:8080` → `::1`
        let end = rest.find(']')?;
        let host = &rest[..end];
        if host.is_empty() {
            return None;
        }
        host.to_ascii_lowercase()
    } else {
        let host = authority.split(':').next()?;
        if host.is_empty() {
            return None;
        }
        host.to_ascii_lowercase()
    };

    // 一致性闸门。**不是**「改用 SDK 的解析器」而是「要求两者同意」——两个方向都要防：
    //
    // - SDK 解析不出来：`127.0.0.1:80/evil` 在本层给出 `127.0.0.1`（allowlist 内），
    //   在 SDK 那边是 `Authority::try_from` 失败 → 400 + `Bad Request: Invalid Host
    //   header`（rmcp 2.2 tower.rs:400）。
    // - 两者算出不同主机：`127.0.0.1:80@evil.com` 在本层给出 `127.0.0.1`，SDK 按
    //   RFC 3986 把前半段当 userinfo、host 取到 `evil.com` → 403 + `Forbidden: Host
    //   header is not allowed`（tower.rs:423）。**状态码相同但正文不同**，一样是
    //   层次 oracle；只用「SDK 能否解析」做闸门挡不住这一条。
    //
    // 反过来只用 SDK 的 `host()` 也不行：那等于把 userinfo 静默丢掉，本层会开始
    // 放行 `evil.example.com@127.0.0.1`。要的是**两侧同意**，不是任选一侧。
    let parsed = Authority::try_from(authority).ok()?;
    if sdk_normalize_host(parsed.host()) != ours {
        return None;
    }
    Some(ours)
}

/// 逐字照抄 rmcp 2.2 `tower.rs:270-274` 的 `normalize_host`，用于上面的一致性比较。
fn sdk_normalize_host(host: &str) -> String {
    host.trim_matches('[').trim_matches(']').to_ascii_lowercase()
}

/// 拆 Origin 为 (scheme, host)，端口丢弃。形如 `scheme://host[:port]`，不接受路径。
fn origin_tuple(origin: &str) -> Option<(String, String)> {
    let (scheme, rest) = origin.trim().split_once("://")?;
    if scheme.is_empty() || rest.contains('/') {
        return None;
    }
    Some((scheme.to_ascii_lowercase(), host_of(rest)?))
}

/// ① DNS-rebinding 防护（T-01-05）。
///
/// rmcp <1.4.0 有过真实 CVE（GHSA-89vp-x53w-74fx）；本项目 pin 的 2.2 已含上游修复，
/// 且 `build_router` 把 SDK 侧的 allowlist 配成同一份——但 **SDK 的修复不替代这一层**：
/// 它可被一行 `disable_allowed_hosts()` 关掉，且 SDK 大版本变更时这一层不动。
pub async fn require_local_host(request: Request, next: Next) -> Response {
    let host = {
        // HTTP/1.1 走 `Host` 头；HTTP/2 把 authority 放在 `:authority` 伪头里，
        // hyper 不为它合成 `Host`（rmcp 自己也做同一个回退，tower.rs:403-409，
        // 那条注释还点名 `axum::Router::nest` 可能丢掉合成头）。只认字面 `Host`
        // 会让任何 h2 客户端拿到一个无从诊断的 403——本层刻意无诊断，误拒的代价
        // 全由用户承担（同 WR-07 的推理）。
        let raw = match request.headers().get(header::HOST) {
            Some(raw) => match raw.to_str() {
                Ok(raw) => raw,
                Err(_) => return deny("non-ascii Host header"),
            },
            None => match request.uri().authority() {
                Some(authority) => authority.as_str(),
                None => return deny("missing Host header"),
            },
        };
        host_of(raw)
    };
    let Some(host) = host else {
        return deny("malformed Host header");
    };
    if !ALLOWED_HOSTS.iter().any(|allowed| *allowed == host) {
        return deny("Host header outside the loopback allowlist");
    }
    next.run(request).await
}

/// ② Origin allowlist。
///
/// 无 `Origin` 头的请求放行到下一层——非浏览器客户端（Claude Code 等）本就不发它，
/// 而它们的鉴权由第三层的 bearer 承担。有 `Origin` 的一律必须在 allowlist 内。
pub async fn require_origin_allowlist(request: Request, next: Next) -> Response {
    let Some(raw) = request.headers().get(header::ORIGIN) else {
        return next.run(request).await;
    };
    let Ok(raw) = raw.to_str() else {
        return deny("non-ascii Origin header");
    };
    let Some(origin) = origin_tuple(raw) else {
        return deny("malformed Origin header");
    };
    let allowed = ALLOWED_ORIGINS
        .iter()
        .filter_map(|entry| origin_tuple(entry))
        .any(|entry| entry == origin);
    if !allowed {
        return deny("Origin outside the allowlist");
    }
    next.run(request).await
}

/// ③ bearer token，**常数时间比较**（Security Domain V2 / T-01-06）。
///
/// scheme 按 RFC 7235 §2.1 大小写不敏感匹配，且容忍 scheme 与 credentials 之间的
/// `1*SP`：RFC 6750 的客户端合法地发 `bearer` / `BEARER`，而 Phase 6 的 MCP 客户端
/// （Claude Code 与其他 agent）不受本项目控制。本层刻意无诊断——一个合规客户端与
/// 一次攻击在它面前完全同形，把合规形态判成攻击的代价只由用户承担（WR-07）。
///
/// 三条 `deny` 的原因串都是编译期常量、只进本地 tracing；响应仍一律 403 + 空正文，
/// 本层不引入任何可观测的差异化（T-01-29）。
pub async fn require_bearer(
    State(deps): State<McpDeps>,
    request: Request,
    next: Next,
) -> Response {
    let Some(raw) = request.headers().get(header::AUTHORIZATION) else {
        return deny("missing Authorization header");
    };
    let Ok(raw) = raw.to_str() else {
        return deny("non-ascii Authorization header");
    };
    let Some((scheme, presented)) = raw.split_once(' ') else {
        return deny("Authorization header carries no credentials");
    };
    if !scheme.eq_ignore_ascii_case("bearer") {
        return deny("Authorization scheme is not Bearer");
    }
    // 只裁**前导**空白（RFC 7235 的 `1*SP`）。尾随 OWS 由 HTTP 头解析层负责，在这里
    // 再 trim 一次会让「token 本身末尾带空白」与「header 里多打了空格」不可区分；
    // 配置侧的归一化已在 `McpDeps::new` 做过一次，两侧因此对同一份字节达成一致。
    let presented = presented.trim_start();
    if !constant_time_eq(deps.expose_bearer(), presented) {
        return deny("bearer token mismatch");
    }
    next.run(request).await
}

/// 常数时间比较。**不得写成 `expected == presented`**——`==` 在首个不同字节处短路，
/// 逐字节猜测 token 的时序侧信道由此成立（T-01-06）。
///
/// 空 `expected` 直接返回 false，且是在进入常数时间比较**之前**短路。这与下一段的
/// 「长度不等时也不提前返回」不矛盾：那条规则守的是「不得因比较**结果**而提前返回」，
/// 而空 expected 是**配置错误**，不是比较结果——它不随呈递值变化，因此不构成侧信道。
/// 这里也没有可泄漏的秘密（配置本身就是空的）；真正的泄漏是放行。
/// 这是 CR-03 纵深的第二层，第一层在 `McpDeps::new`：即便有人绕过构造器造出空配置，
/// 这里仍拒；即便有人放宽这里，构造器仍拒。两层各有自己的测试。
/// **本函数内只有这一个空值处理点**，函数体里不存在第二道兜底——
/// `the_comparison_is_not_a_plain_equality` 的第三条断言看着它。
///
/// 长度不等时也不提前返回：把 presented 折进一个与 expected 等长的缓冲区
/// （超出部分参与折叠而非被丢弃），再与长度比较结果按位与。
fn constant_time_eq(expected: &str, presented: &str) -> bool {
    // 本函数**唯一**的空值处理点。下面没有第二道守卫——删掉这一条，两个空串会一路
    // 走到底并让长度与内容双双成立，CR-03 的 fail-open 当场复活。
    // （历史形态里下面还有一个「空 expected 时取零长切片」的分支，它从来进不去，
    //   却让读者以为空值在下面还被兜了一次；01-REVIEW.md WR-05 记录的正是这个陷阱。）
    if expected.is_empty() {
        return false;
    }

    let expected = expected.as_bytes();
    let presented = presented.as_bytes();

    // 早退之后 expected 必然非空，因此缓冲区长度恒 ≥ 1，下面的 `% folded.len()` 安全。
    let mut folded = vec![0u8; expected.len()];
    for (i, byte) in presented.iter().enumerate() {
        let slot = i % folded.len();
        folded[slot] ^= byte;
    }

    let same_len = (expected.len() as u64).ct_eq(&(presented.len() as u64));
    let same_bytes = expected.ct_eq(&folded[..]);
    (same_len & same_bytes).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_of_strips_port_and_brackets() {
        assert_eq!(host_of("127.0.0.1:51234").as_deref(), Some("127.0.0.1"));
        assert_eq!(host_of("LocalHost").as_deref(), Some("localhost"));
        assert_eq!(host_of("[::1]:8080").as_deref(), Some("::1"));
        assert_eq!(host_of("[::1]").as_deref(), Some("::1"));
        assert_eq!(host_of(""), None);
        assert_eq!(host_of(":8080"), None);
        assert_eq!(host_of("[::1"), None);
        // 端口不是数字是**合法** authority（实测：`Authority::try_from` 接受它、
        // `port_u16()` 给 None、host 仍是 `127.0.0.1`），两侧一致 → 放行。
        // 01-REVIEW.md WR-03 把它当作「过本层、被 SDK 以 400 拒掉」的例子，实测不成立；
        // 真正可达的两个形态见下面 `host_of_agrees_with_the_sdk_parser_or_denies`。
        assert_eq!(host_of("127.0.0.1:notanumber").as_deref(), Some("127.0.0.1"));
        assert_eq!(host_of("127.0.0.1:").as_deref(), Some("127.0.0.1"));
        // 两侧口径分叉的两个实测形态，现在一律在本层拒：
        assert_eq!(host_of("127.0.0.1:80/evil"), None); // SDK 侧解析失败 → 400 + 正文
        assert_eq!(host_of("127.0.0.1:80@evil.com"), None); // SDK 侧 host = evil.com → 403 + 正文
        assert_eq!(host_of("evil.example.com@127.0.0.1"), None); // 反方向：SDK 会放行，本层不放
    }

    /// 让 SDK 那两种带正文的拒绝不可达的**那条性质**：本层与 SDK 对主机名达成一致。
    ///
    /// rmcp 2.2 `parse_host_header`（tower.rs:382-410）对 `Host` 头做
    /// `http::uri::Authority::try_from`，失败即 `bad_request_response("Bad Request:
    /// Invalid Host header")`（400 带正文）；解析成功但主机名不在 allowlist 里则是
    /// `forbidden_response("Forbidden: Host header is not allowed")`（403 带正文）。
    /// 本层放行的每一个 authority 都必须让 SDK 算出**同一个**主机名，否则就存在一个
    /// 「过了应用层、被 SDK 以另一种形态拒掉」的输入——T-01-29 禁止的可观测差异。
    #[test]
    fn host_of_agrees_with_the_sdk_parser_or_denies() {
        // 覆盖面刻意含**会被放行**的形态（阴性对照）：若本条只喂非法样本，
        // 一个「一律返回 None」的 host_of 也能让它全绿。
        let sample = [
            "127.0.0.1:51234",
            "localhost",
            "LocalHost",
            "[::1]:8080",
            "[::1]",
            "evil.example.com",
            "evil.example.com:8080",
            "127.0.0.1:notanumber",
            "127.0.0.1:",
            "127.0.0.1:99999999",
            "127.0.0.1:0080",
            "127.0.0.1:80/evil",
            "127.0.0.1:80@evil.com",
            "evil.example.com@127.0.0.1",
            "127.0.0.1 51234",
            "127.0.0.1?x",
            "[::1",
            ":8080",
            "",
        ];

        let mut accepted = 0usize;
        for authority in sample {
            let Some(ours) = host_of(authority) else {
                continue;
            };
            accepted += 1;
            let parsed = Authority::try_from(authority.trim()).unwrap_or_else(|_| {
                panic!(
                    "本层放行了 {authority:?}，但 SDK 的 Authority::try_from 会拒它 —— \
                     该请求会从 SDK 拿到一个带正文的 400"
                )
            });
            assert_eq!(
                sdk_normalize_host(parsed.host()),
                ours,
                "本层与 SDK 对 {authority:?} 算出了不同的主机名 —— \
                 SDK 会用一个带正文的 403 拒它，三层的无差别拒绝就此破裂"
            );
        }
        assert!(
            accepted >= 5,
            "样本里没有足够的放行形态，这条断言退化成恒真（accepted={accepted}）"
        );
    }

    /// HTTP/2 把 authority 放在 `:authority` 伪头里，hyper 不为它合成 `Host`
    /// （rmcp 自己也这么处理：tower.rs:403-409 的注释点名 `axum::Router::nest`
    /// 可能丢掉合成头）。要求字面 `Host` 头会让任何 h2 客户端拿到一个无从诊断的 403。
    #[tokio::test]
    async fn require_local_host_falls_back_to_the_uri_authority() {
        // 无 Host 头 + URI 带 loopback authority（HTTP/2 形态）→ 放行。
        assert_eq!(
            host_layer_status("http://127.0.0.1:51234/mcp", None).await,
            StatusCode::OK,
            "HTTP/2 形态（authority 在 URI 里）被无差别拒绝了"
        );

        // 同一形态换成外域 authority → 仍拒。回退不是放行。
        assert_eq!(
            host_layer_status("http://evil.example.com/mcp", None).await,
            StatusCode::FORBIDDEN,
            "URI authority 回退把外域也放行了"
        );

        // 既无 Host 头也无 URI authority → 拒（既有行为）。
        assert_eq!(
            host_layer_status("/mcp", None).await,
            StatusCode::FORBIDDEN,
            "两个来源都没有 authority 时应当拒绝"
        );

        // Host 头存在时它优先（既有行为，阴性对照：回退不得夺走 Host 头的判定权）。
        assert_eq!(
            host_layer_status("http://127.0.0.1:51234/mcp", Some("evil.example.com")).await,
            StatusCode::FORBIDDEN,
            "URI authority 合法就放行了一个外域 Host 头"
        );
        assert_eq!(
            host_layer_status("/mcp", Some("127.0.0.1:51234")).await,
            StatusCode::OK,
        );
    }

    /// 把一个请求过一遍**只挂 `require_local_host`** 的路由，返回状态码。
    async fn host_layer_status(uri: &str, host_header: Option<&str>) -> StatusCode {
        use tower::ServiceExt;

        let mut builder = axum::http::Request::builder().method("POST").uri(uri);
        if let Some(host) = host_header {
            builder = builder.header(header::HOST, host);
        }
        let request = builder
            .body(axum::body::Body::empty())
            .expect("valid test request");

        let router = axum::Router::new()
            .route("/mcp", axum::routing::post(|| async { "sentinel" }))
            .layer(axum::middleware::from_fn(require_local_host));

        router
            .oneshot(request)
            .await
            .expect("router is infallible")
            .status()
    }

    #[test]
    fn origin_tuple_drops_the_port_and_rejects_paths() {
        assert_eq!(
            origin_tuple("http://127.0.0.1:1420"),
            Some(("http".into(), "127.0.0.1".into()))
        );
        assert_eq!(
            origin_tuple("TAURI://localhost"),
            Some(("tauri".into(), "localhost".into()))
        );
        assert_eq!(origin_tuple("http://evil.example.com/x"), None);
        assert_eq!(origin_tuple("evil.example.com"), None);
        assert_eq!(origin_tuple("://localhost"), None);
    }

    #[test]
    fn constant_time_eq_agrees_with_equality_on_every_shape() {
        // 绑定名刻意不叫 `token`：`scripts/check-secrets.sh` 抓的正是「名字像密钥的
        // 标识符后面跟一个引号串」这个形状，它没抓错。与 `prism-llm` 的 `FIXTURE_SECRET`
        // 和前端的 `FAKE_KEY` 同源——扫描器是防线，不该为了迁就测试局部变量而放宽。
        // `configured` 同时更准：它是「配置侧的那个值」，与函数签名的 `expected` 同义。
        let configured = "0123456789abcdef";
        assert!(constant_time_eq(configured, configured));
        // 等长、仅末位不同
        assert!(!constant_time_eq(configured, "0123456789abcdee"));
        // 前缀（更短）
        assert!(!constant_time_eq(configured, "0123456789abcde"));
        // 正确值加后缀：折叠必须不让它等价于正确值
        assert!(!constant_time_eq(configured, "0123456789abcdefx"));
        // 恰好长一轮：折叠的 slot 复用不得制造碰撞
        assert!(!constant_time_eq(
            configured,
            "0123456789abcdef0123456789abcdef"
        ));
        assert!(!constant_time_eq(configured, ""));
        assert!(!constant_time_eq("", configured));
        // 配置为空的门禁不得放行任何人——包括呈递空 token 的人（CR-03）。
        // 这条 case 刻意**不删**：被删掉的形态就是没人看着的形态。它曾经把
        // fail-open 钉成预期，现在钉的是相反的结论。
        assert!(!constant_time_eq("", ""));
    }

    /// 源码层面的守卫：这一层永远不能退回 `==`。
    #[test]
    fn the_comparison_is_not_a_plain_equality() {
        let source = include_str!("middleware.rs");
        let body = source
            .split("fn constant_time_eq")
            .nth(1)
            .expect("constant_time_eq exists");
        let body = &body[..body.find("\n}\n").expect("function body ends")];
        assert!(body.contains("ct_eq"), "常数时间比较被换掉了");
        assert!(
            !body.contains("expected == presented"),
            "退回了短路比较 `==`"
        );
        // 第三条：空配置的短路守卫仍在函数体内。它是本函数唯一的空值处理点，
        // 删掉它 CR-03 的 fail-open 当场复活（01-REVIEW.md WR-05 逐行推过一遍）。
        //
        // 锚点刻意取**完整语句**而不是裸的 `is_empty`：这里的匹配面同时含代码与
        // 函数体内的解释性注释，一个也可能出现在注释里的片段会让这条断言在守卫被
        // 删掉之后仍然绿。本 phase 已有实测教训（`src-tauri/src/lib.rs` 的源码序
        // 断言撞上注释而假红，方向相反、成因相同）。
        assert!(
            body.contains("if expected.is_empty() {"),
            "空配置的短路守卫被删掉了 —— 比较层的 fail-open 复活了"
        );
    }
}
