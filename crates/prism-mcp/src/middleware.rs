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

use axum::extract::{Request, State};
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
fn host_of(authority: &str) -> Option<String> {
    let authority = authority.trim();
    if authority.is_empty() {
        return None;
    }
    if let Some(rest) = authority.strip_prefix('[') {
        // IPv6 字面量：`[::1]:8080` → `::1`
        let end = rest.find(']')?;
        let host = &rest[..end];
        if host.is_empty() {
            return None;
        }
        return Some(host.to_ascii_lowercase());
    }
    let host = authority.split(':').next()?;
    if host.is_empty() {
        return None;
    }
    Some(host.to_ascii_lowercase())
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
    let Some(raw) = request.headers().get(header::HOST) else {
        return deny("missing Host header");
    };
    let Ok(raw) = raw.to_str() else {
        return deny("non-ascii Host header");
    };
    let Some(host) = host_of(raw) else {
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
