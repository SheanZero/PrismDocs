//! 锚点层：Block ID 分配、内容指纹与迁移打分。
//!
//! Phase 1 plan 01（D-08）只建立骨架与真实依赖声明。TD-01 的三步迁移算法
//! （blake3 精确匹配 → 序列 diff → 残差相似度打分）是 Phase 3 的内容；
//! 算法内部保持不冻结，此处只固定「指纹 = blake3 十六进制串」这一条。

/// Phase 3 会在此扩展（迁移预算超时、残差集过大等）。
#[derive(Debug, thiserror::Error)]
pub enum AnchorError {
    #[error("anchor migration exceeded its time budget")]
    MigrationBudgetExceeded,
}

/// 内容指纹：blake3 哈希的十六进制串。Phase 3 的精确匹配步直接用它。
pub fn content_fingerprint(s: &str) -> String {
    blake3::hash(s.as_bytes()).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_a_64_char_lowercase_hex_string() {
        let fp = content_fingerprint("hello");
        assert_eq!(fp.len(), 64, "unexpected fingerprint: {fp}");
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()));
    }

    #[test]
    fn fingerprint_is_stable_and_content_addressed() {
        assert_eq!(content_fingerprint("same"), content_fingerprint("same"));
        assert_ne!(content_fingerprint("a"), content_fingerprint("b"));
    }

    /// 依赖真的可用：`similar` 的相似度打分是 Phase 3 第三步的核心。
    #[test]
    fn similar_scores_identical_text_as_a_perfect_match() {
        let diff = similar::TextDiff::from_lines("a\nb\n", "a\nb\n");
        assert!((diff.ratio() - 1.0).abs() < f32::EPSILON);
    }

    /// 依赖真的可用：Block ID 在 Phase 3 是不透明稳定 ULID。
    #[test]
    fn ulid_produces_a_26_char_identifier() {
        assert_eq!(ulid::Ulid::new().to_string().len(), 26);
    }

    #[test]
    fn anchor_error_renders_a_message() {
        assert!(!AnchorError::MigrationBudgetExceeded.to_string().is_empty());
    }
}
