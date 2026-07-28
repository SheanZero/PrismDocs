//! Markdown 解析层：**comrak 是本项目 Block 边界的唯一真相源**。
//!
//! 前端的 react-markdown 只负责渲染——两个 parser 各自分块必然导致锚点漂移
//! （CLAUDE.md What-NOT-to-use 首条）。因此本 crate 不得引入第二个 markdown parser。
//!
//! Phase 1 plan 01（D-08）只建立骨架；sourcepos 字节区间、frontmatter 边界与
//! 根级 Block 切分是 Phase 3 的内容。

/// Phase 3 会在此扩展（sourcepos 缺失、frontmatter 边界异常等）。
/// 现在只占位一个变体，保证错误类型从 Phase 1 起就是 thiserror 形态。
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("comrak produced no root node for the given source")]
    EmptyDocument,
}

/// 数一份 markdown 的根级块数量——comrak AST 根节点的直接子节点个数。
///
/// 这是 Phase 3 Block 切分的最小前身，同时让 comrak 真正进入依赖树。
pub fn root_block_count(md: &str) -> usize {
    let arena = comrak::Arena::new();
    let root = comrak::parse_document(&arena, md, &comrak::Options::default());
    root.children().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_document_has_no_root_blocks() {
        assert_eq!(root_block_count(""), 0);
    }

    #[test]
    fn each_top_level_construct_is_one_root_block() {
        let md = "# Title\n\nA paragraph.\n\n- a\n- b\n\n```rust\nfn main() {}\n```\n";
        // heading + paragraph + list + code block
        assert_eq!(root_block_count(md), 4);
    }

    #[test]
    fn a_lazy_continuation_stays_inside_one_paragraph() {
        assert_eq!(root_block_count("line one\nline two\n"), 1);
    }

    #[test]
    fn parse_error_renders_a_message() {
        assert!(!ParseError::EmptyDocument.to_string().is_empty());
    }
}
