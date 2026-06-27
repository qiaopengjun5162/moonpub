//! Markdown → WeChat HTML conversion.
//!
//! This module handles only the syntactic transformation: parsing `:::name` fences,
//! plain markdown segments, inline formatting, and emitting inline-styled HTML.
//! It intentionally knows nothing about file I/O, frontmatter, or WeChat API drafts.

use crate::theme;

mod blocks;
mod inline;
mod parser;
mod plain;
#[cfg(test)]
use inline::inline_md;
#[cfg(test)]
use parser::split_fence_props;
use parser::{MdBlock, parse_blocks};
#[cfg(test)]
use plain::render_markdown_segment;

/// Convert a markdown body into WeChat-compatible HTML using the given theme.
pub fn md_to_wechat_html(md: &str, theme: &theme::Theme) -> String {
    let parsed_blocks = parse_blocks(md);
    let mut out = String::new();

    for block in &parsed_blocks {
        match block {
            MdBlock::Fence(name, props, body) => {
                out.push_str(&blocks::render_fence_block(name, props, body, theme))
            }
            MdBlock::Markdown(text) => out.push_str(&plain::render_markdown_segment(text, theme)),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use crate::markdown::{
        inline_md, md_to_wechat_html, parse_blocks, render_markdown_segment, split_fence_props,
    };
    use crate::theme;

    #[test]
    fn parse_blocks_splits_fence_and_markdown() {
        let md = "intro\n\n:::tip\nhello\n:::\n\noutro";
        let blocks = parse_blocks(md);
        assert_eq!(blocks.len(), 3);
    }

    #[test]
    fn inline_md_renders_bold_italic_code() {
        let t = theme::Theme::from_name("default");
        let html = inline_md("**bold** *italic* `code`", &t);
        assert!(html.contains("<strong"));
        assert!(html.contains("<em>"));
        assert!(html.contains("<code"));
    }

    #[test]
    fn render_markdown_skips_obsidian_callout() {
        let t = theme::Theme::from_name("default");
        let md = "> [!abstract]\n> hidden\n\nvisible";
        let html = render_markdown_segment(md, &t);
        assert!(!html.contains("hidden"));
        assert!(html.contains("visible"));
    }

    #[test]
    fn parse_blocks_handles_multiple_fences() {
        let md = ":::tip\nfirst\n:::\n\n:::tip\nsecond\n:::";
        let blocks = parse_blocks(md);
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn parse_blocks_handles_unclosed_fence() {
        let md = ":::tip\nno closing marker";
        let blocks = parse_blocks(md);
        assert_eq!(blocks.len(), 1);
    }

    #[test]
    fn split_fence_props_parses_key_value_pairs() {
        let inner = "label: 提示\nicon: 💡\n\n这是正文";
        let (props, body) = split_fence_props(inner);
        assert_eq!(props.len(), 2);
        assert_eq!(props[0], ("label", "提示"));
        assert_eq!(props[1], ("icon", "💡"));
        assert_eq!(body, "这是正文");
    }

    #[test]
    fn split_fence_props_stops_at_first_non_prop_line() {
        let inner = "label: 提示\n这不是属性\n\n正文";
        let (props, body) = split_fence_props(inner);
        assert_eq!(props.len(), 1);
        assert_eq!(props[0], ("label", "提示"));
        assert_eq!(body, "这不是属性\n\n正文");
    }

    #[test]
    fn md_to_wechat_html_renders_intro_fence() {
        let t = theme::Theme::from_name("default");
        let md = ":::intro\n这是引言\n:::";
        let html = md_to_wechat_html(md, &t);
        assert!(html.contains("这是引言"));
        assert!(html.contains("border-left"));
    }

    #[test]
    fn md_to_wechat_html_renders_callout_fence() {
        let t = theme::Theme::from_name("default");
        let md = ":::callout\nlabel: 重点\n\n注意内容\n:::";
        let html = md_to_wechat_html(md, &t);
        assert!(html.contains("注意内容"));
        assert!(html.contains("重点"));
    }

    #[test]
    fn plain_markdown_renders_unordered_lists_as_styled_blocks() {
        let t = theme::Theme::from_name("paper");
        let md = "- 第一条\n- 第二条";
        let html = render_markdown_segment(md, &t);

        assert!(html.contains("moonpub-list"));
        assert!(html.contains("第一条"));
        assert!(html.contains("第二条"));
        assert!(html.contains(t.accent));
    }

    #[test]
    fn plain_markdown_renders_ordered_lists_with_number_badges() {
        let t = theme::Theme::from_name("magazine");
        let md = "1. 起点\n2. 转折";
        let html = render_markdown_segment(md, &t);

        assert!(html.contains("moonpub-list"));
        assert!(html.contains(">1<"));
        assert!(html.contains(">2<"));
        assert!(html.contains("起点"));
        assert!(html.contains("转折"));
    }

    #[test]
    fn plain_markdown_renders_fenced_code_blocks() {
        let t = theme::Theme::from_name("geek");
        let md = "```rust\nfn main() {\n    println!(\"hi\");\n}\n```";
        let html = render_markdown_segment(md, &t);

        assert!(html.contains("rust"));
        assert!(html.contains("<pre"));
        assert!(html.contains("fn main()"));
        assert!(html.contains("&quot;hi&quot;"));
    }

    #[test]
    fn plain_markdown_renders_theme_aware_divider() {
        let t = theme::Theme::from_name("ocean");
        let html = render_markdown_segment("---", &t);

        assert!(html.contains(t.border));
        assert!(html.contains(t.accent));
        assert!(!html.contains("<hr"));
    }
}
