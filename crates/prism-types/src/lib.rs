//! 零依赖共享类型层（除 serde derive）。
//!
//! Phase 1 plan 04 才引入 service trait 与 `EngineEvent`；本 crate 现在只提供
//! 一个最小编译单元，用来把 workspace 形状固定下来。

pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 最小的可序列化载荷，确认 serde derive 真的被链接进依赖树。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CrateInfo {
    pub name: String,
    pub version: String,
}

impl CrateInfo {
    pub fn of_types_crate() -> Self {
        Self {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: CRATE_VERSION.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_version_is_populated() {
        assert!(!CRATE_VERSION.is_empty());
    }

    #[test]
    fn crate_info_identifies_this_crate() {
        let info = CrateInfo::of_types_crate();
        assert_eq!(info.name, "prism-types");
        assert_eq!(info.version, CRATE_VERSION);
    }
}
