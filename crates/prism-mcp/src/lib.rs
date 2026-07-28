//! MCP 工具面：rmcp StreamableHttpService 挂在 loopback axum 上（D-07）。
//!
//! **D-09：本 crate 只依赖 prism-types，绝不依赖 prism-engine。**
//! 编排能力经由 prism-types 中的 service trait 以 `Arc<dyn Trait>` 注入，
//! 因此编译期不存在 facade↔mcp 依赖环——该性质由
//! `cargo tree -p prism-mcp --edges normal` 中不出现 prism-engine 断言。
//!
//! Phase 1 plan 01（D-08）建立骨架，plan 01-06 填入注入容器、最小 handler、
//! loopback 宿主与三层鉴权中间件；工具面本身是 Phase 6 的内容。

pub mod deps;
pub mod handler;
pub mod server;

/// 后续 plan 会在此扩展（绑定失败、token 缺失等）。
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum McpError {
    #[error("failed to bind the loopback MCP listener: {0}")]
    Bind(#[from] std::io::Error),
}

/// 本 SDK 版本声明的最新 MCP 协议版本。
pub fn protocol_version() -> &'static str {
    // `ProtocolVersion` 内含 `Cow<'static, str>`，用 static 承载即可拿到 'static 借用。
    static LATEST: rmcp::model::ProtocolVersion = rmcp::model::ProtocolVersion::LATEST;
    LATEST.as_str()
}

/// 与 prism-types 的连接点（D-09 的依赖方向由此固定）。
pub fn shared_types_version() -> &'static str {
    prism_types::CRATE_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_version_is_a_dated_spec_revision() {
        let v = protocol_version();
        let parts: Vec<&str> = v.split('-').collect();
        assert_eq!(parts.len(), 3, "unexpected protocol version: {v}");
        assert!(
            parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())),
            "unexpected protocol version: {v}"
        );
    }

    #[test]
    fn protocol_version_is_known_to_the_sdk() {
        assert!(
            rmcp::model::ProtocolVersion::KNOWN_VERSIONS
                .iter()
                .any(|v| v.as_str() == protocol_version()),
            "protocol version drifted out of the SDK's known set"
        );
    }

    #[test]
    fn shared_types_are_reachable_from_here() {
        assert!(!shared_types_version().is_empty());
    }

    /// 依赖真的可用：D-07 的宿主是 axum，路由在 plan 01-06 才被填满。
    #[tokio::test]
    async fn an_axum_router_can_be_constructed_on_the_runtime() {
        let router: axum::Router = axum::Router::new();
        drop(router);
    }
}
