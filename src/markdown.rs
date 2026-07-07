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

        assert!(html.contains("2026-07-03"));
        assert!(html.contains("/photos/day-1.jpg"));
        assert!(html.contains("雨后的树影"));
        assert!(html.contains("慢慢来"));
        assert!(html.contains("月下林边"));
        assert!(html.contains("温柔收束"));
    }

    #[test]
    fn md_to_wechat_html_renders_collection_opener_recipe_blocks() {
        let t = theme::Theme::from_name("editorial");
        let md = r#"
:::meta-strip
mood: 松弛、克制、慢慢写
闲月隐林：七分明说，三分自留。这是一篇给新合集立边界的开篇文。
:::

:::intro
欢迎来到这片林子，这里会慢慢记录日常、山野步履和书页碎思。
:::

:::letter-card
title: 给读者的一封短笺
date: 2026-07-04
这个合集不追流量，只想把能公开说的话安静留下来。
:::

:::scene-card
label: 起点
place: 月下林边
起因是一段路上偶然刷到的记录，也是一点想重新开始写的心。
:::

:::closing-card
label: 欢迎进来
以后就在这里，慢慢写，慢慢聊。
:::
"#;

        let html = md_to_wechat_html(md, &t);

        assert!(html.contains("闲月隐林"));
        assert!(html.contains("七分明说，三分自留"));
        assert!(html.contains("给读者的一封短笺"));
        assert!(html.contains("2026-07-04"));
        assert!(html.contains("月下林边"));
        assert!(html.contains("慢慢写，慢慢聊"));
    }

    #[test]
    fn md_to_wechat_html_renders_quiet_opening_recipe_blocks() {
        let t = theme::Theme::from_name("moonlit");
        let md = r#"
:::meta-strip
mood: 月下栖林、七分明说、三分自留
闲月隐林是一片安静的小林子，用来慢慢记录日常和所思所感。
:::

:::intro
欢迎来到这片林子。这里不追求热闹，只想把可以公开说的部分认真留下来。
:::

:::letter-card
title: 写在开篇
date: 2026-07-07
有些话明说，有些话留白。这个合集会在两者之间慢慢找到自己的节奏。
:::

:::scene-card
label: 起念
place: 月下林边
最开始只是路上刷到一个安静记录生活的账号，于是也想重新给自己留一个地方。
:::

:::closing-card
label: 入林
以后就在这里，慢慢写，慢慢沉淀。
:::
"#;

        let html = md_to_wechat_html(md, &t);
        let audit = crate::layout_audit::audit_html(std::path::Path::new("quiet.html"), &html);

        assert!(audit.errors.is_empty(), "{:?}", audit.errors);
        assert!(html.contains("#5d6f8c"));
        assert!(html.contains("闲月隐林"));
        assert!(html.contains("七分明说"));
        assert!(html.contains("慢慢沉淀"));
    }

    #[test]
    fn md_to_wechat_html_renders_memory_note_recipe_blocks() {
        let t = theme::Theme::from_name("fieldnote");
        let md = r#"
:::meta-strip
date: 2026-07-07
place: 河边小路
mood: 想留住这一天
这是一组从手机相册里捡回来的生活片段，只记录真实发生过的事。
:::

:::intro
这天没有特别大的事情，只是几张照片刚好把当时的路、风和心情留下来了。
:::

:::photo-grid
- /photos/river-1.jpg | 树影落在路边
- /photos/river-2.jpg | 天色慢慢暗下来
- /photos/river-3.jpg | 回家前拍的一张
:::

:::scene-card
label: 现场
place: 河边小路
照片里能确认的是：路边有风，天色在变暗，我那时候正慢慢往回走。
:::

:::closing-card
label: 留档
先把这一天留在这里，免得以后只剩下一堆快要删掉的照片。
:::
"#;

        let html = md_to_wechat_html(md, &t);
        let audit = crate::layout_audit::audit_html(std::path::Path::new("memory.html"), &html);

        assert!(audit.errors.is_empty(), "{:?}", audit.errors);
        assert!(html.contains("#8c7356"));
        assert!(html.contains("/photos/river-1.jpg"));
        assert!(html.contains("真实发生过的事"));
        assert!(html.contains("快要删掉的照片"));
    }

    #[test]
    fn md_to_wechat_html_renders_spoken_note_recipe_blocks() {
        let t = theme::Theme::from_name("letter");
        let md = r#"
:::meta-strip
date: 2026-07-05
place: 散步路上
mood: 边走边想
这篇来自一次口述记录，只保留当时真正说到的线索。
:::

:::intro
今天这段口述，是想把一个还没有完全想清楚的念头先放下来。
:::

:::letter-card
title: 当时想说的是
date: 2026-07-05
我真正想留下来的，不是一个漂亮结论，而是当时那个念头出现的瞬间。
:::

:::summary
- 一个确定发生过的事实
- 一个当时冒出来的判断
- 一个可以以后再展开的问题
:::

:::closing-card
label: 先记到这里
这次先不急着下结论，留给下一次继续想。
:::
"#;

        let html = md_to_wechat_html(md, &t);

        assert!(html.contains("#a6633c"));
        assert!(html.contains("总 结"));
        assert!(html.contains("散步路上"));
        assert!(html.contains("当时想说的是"));
        assert!(html.contains("2026-07-05"));
        assert!(html.contains("一个确定发生过的事实"));
        assert!(html.contains("先记到这里"));
    }

    #[test]
    fn md_to_wechat_html_renders_daily_report_recipe_blocks() {
        let t = theme::Theme::from_name("notebook");
        let md = r#"
:::intro
今天最值得看的主线是 AI 工具和 Web3 基础设施同时更新。
:::

:::divider
label: 今日速览
:::

:::summary
- AI 工具继续靠近真实工作流
- Web3 基础设施出现新信号
- 开源项目仍需回到原文核对
:::

:::callout
label: 先读这条
这里放今天最重要的一件事：发生了什么、为什么重要、后续看什么。
:::

## 参考来源

:::compact-links
- 01 | OpenAI 发布 | OpenAI｜官方公告 | https://openai.com/news?ref=moonpub&from=report
- 02 | 研究文章 | Ethereum Research｜深读 | https://ethresear.ch/t/zk
:::
"#;

        let html = md_to_wechat_html(md, &t);

        assert!(html.contains("今天最值得看的主线"));
        assert!(html.contains("今日速览"));
        assert!(html.contains("总 结"));
        assert!(html.contains("AI 工具继续靠近真实工作流"));
        assert!(html.contains("先读这条"));
        assert!(html.contains("moonpub-compact-links"));
        assert!(html.contains("font-size:12px"));
        assert!(
            html.contains("原文：<a href=\"https://openai.com/news?ref=moonpub&amp;from=report\"")
        );
        assert!(html.contains(">https://openai.com/news?ref=moonpub&amp;from=report</a>"));
        assert!(!html.contains(">OpenAI 发布</a>"));
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
