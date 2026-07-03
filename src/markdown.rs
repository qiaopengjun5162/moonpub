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
    fn inline_md_renders_highlight_and_strikethrough() {
        let t = theme::Theme::from_name("newsletter");
        let html = inline_md("这是 ==重点== 和 ~~旧说法~~", &t);

        assert!(html.contains("<mark"));
        assert!(html.contains("重点"));
        assert!(html.contains("<del"));
        assert!(html.contains("旧说法"));
        assert!(html.contains(t.accent_soft));
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
    fn parse_blocks_handles_empty_fence_without_panic() {
        let md = ":::divider\n:::";
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
    fn md_to_wechat_html_renders_life_story_recipe_blocks() {
        let t = theme::Theme::from_name("mist");
        let md = r#"
:::meta-strip
date: 2026-07-03
place: 河边小路
weather: 晚风
mood: 安静
今天只记一个真实的小片段。
:::

:::intro
开篇用 1-3 句话交代这篇文章为什么写。
:::

:::photo-grid
- /photos/day-1.jpg | 雨后的树影
- /photos/day-2.jpg | 回家的路
:::

:::scene-card
label: 路上
place: 月下林边
这里放一段真实场景，不要过度修饰。
:::

:::closing-card
label: 慢慢来
给文章一个温柔收束。
:::
"#;

        let html = md_to_wechat_html(md, &t);

        assert!(html.contains("moonpub-meta-strip"));
        assert!(html.contains("moonpub-photo-grid"));
        assert!(html.contains("moonpub-scene-card"));
        assert!(html.contains("moonpub-closing-card"));
        assert!(html.contains("2026-07-03"));
        assert!(html.contains("/photos/day-1.jpg"));
        assert!(html.contains("雨后的树影"));
        assert!(html.contains("月下林边"));
        assert!(html.contains("温柔收束"));
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
    fn plain_markdown_renders_task_lists_as_checklist_blocks() {
        let t = theme::Theme::from_name("newsletter");
        let md = "- [x] 完成选题\n- [ ] 补充配图";
        let html = render_markdown_segment(md, &t);

        assert!(html.contains("moonpub-checklist"));
        assert!(html.contains("✔"));
        assert!(html.contains("○"));
        assert!(html.contains("完成选题"));
        assert!(html.contains("补充配图"));
        assert!(!html.contains("] 完成选题"));
        assert!(!html.contains("] 补充配图"));
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

    #[test]
    fn plain_markdown_renders_first_paragraph_as_lead() {
        let t = theme::Theme::from_name("editorial");
        let html = render_markdown_segment("第一段开篇。\n\n第二段正文。", &t);

        assert!(html.contains("moonpub-lead"));
        assert!(html.contains("font-size: 16px"));
        assert!(html.contains("第一段开篇"));
        assert!(html.contains("text-indent: 2em"));
        assert!(html.contains("第二段正文"));
    }

    #[test]
    fn plain_markdown_renders_h4_as_compact_subhead() {
        let t = theme::Theme::from_name("zen");
        let html = render_markdown_segment("#### 细节清单", &t);

        assert!(html.contains("<h4"));
        assert!(html.contains("细节清单"));
        assert!(html.contains(t.accent));
    }

    #[test]
    fn plain_markdown_renders_image_alt_as_caption() {
        let t = theme::Theme::from_name("default");
        let html = render_markdown_segment("![架构图](https://example.com/arch.png)", &t);

        assert!(html.contains("moonpub-figure"));
        assert!(html.contains("https://example.com/arch.png"));
        assert!(html.contains(">架构图</p>"));
    }
}
