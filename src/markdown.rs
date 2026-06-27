//! Markdown → WeChat HTML conversion.
//!
//! This module handles only the syntactic transformation: parsing `:::name` fences,
//! plain markdown segments, inline formatting, and emitting inline-styled HTML.
//! It intentionally knows nothing about file I/O, frontmatter, or WeChat API drafts.

use crate::theme;

mod blocks;
mod parser;
#[cfg(test)]
use parser::split_fence_props;
use parser::{MdBlock, parse_blocks};

/// Convert a markdown body into WeChat-compatible HTML using the given theme.
pub fn md_to_wechat_html(md: &str, theme: &theme::Theme) -> String {
    let parsed_blocks = parse_blocks(md);
    let mut out = String::new();

    for block in &parsed_blocks {
        match block {
            MdBlock::Fence(name, props, body) => {
                out.push_str(&blocks::render_fence_block(name, props, body, theme))
            }
            MdBlock::Markdown(text) => out.push_str(&render_markdown_segment(text, theme)),
        }
    }

    out
}

// ── Plain markdown segment renderer ───────────────────────────────────────────

fn render_table(lines: &[&str], theme: &theme::Theme) -> String {
    let mut html = format!(
        "<section style=\"margin: 22px 0; overflow-x: auto; border:1px solid {}; border-radius:6px;\"><table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse: collapse; width: 100%; font-size: 14px;\">\n",
        theme.border
    );
    let mut is_header = true;
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            // separator row (|---|---|) — skip, used to signal header done
            let inner = &trimmed[1..trimmed.len() - 1];
            if inner
                .split('|')
                .all(|c| c.trim().chars().all(|x| x == '-' || x == ':' || x == ' '))
            {
                is_header = false;
                continue;
            }
            let cells: Vec<&str> = inner.split('|').collect();
            html.push_str("<tr>\n");
            for cell in &cells {
                let cell = cell.trim();
                if is_header {
                    html.push_str(&format!(
                        "<th style=\"padding: 10px 12px; background: {}; color: {}; font-weight: bold; border-right: 1px solid {}; text-align: left; line-height:1.6;\">{}</th>\n",
                        theme.header_bg, "#fff", theme.border, inline_md(cell, theme)
                    ));
                } else {
                    html.push_str(&format!(
                        "<td style=\"padding: 9px 12px; border-top: 1px solid {}; border-right: 1px solid {}; color: {}; vertical-align: top; line-height:1.7;\">{}</td>\n",
                        theme.border, theme.border, theme.text_color, inline_md(cell, theme)
                    ));
                }
            }
            html.push_str("</tr>\n");
        }
    }
    html.push_str("</table></section>\n\n");
    html
}

fn render_markdown_segment(md: &str, theme: &theme::Theme) -> String {
    let mut out = String::new();
    let mut in_blockquote = false;
    let mut is_callout = false;
    let mut blockquote_buf = String::new();
    let mut table_buf: Vec<&str> = Vec::new();
    let mut list_buf: Vec<ListItem> = Vec::new();
    let mut code_lang = String::new();
    let mut code_buf = String::new();
    let mut in_code = false;

    let lines: Vec<&str> = md.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        i += 1;

        // Collect consecutive table rows
        let trimmed = line.trim();
        if let Some(lang) = trimmed.strip_prefix("```") {
            if in_code {
                out.push_str(&crate::illustrate::render_code_block(
                    &code_lang,
                    code_buf.trim_end(),
                    theme,
                ));
                code_buf.clear();
                code_lang.clear();
                in_code = false;
            } else {
                flush_paragraph_buffers(
                    &mut out,
                    &mut in_blockquote,
                    &mut is_callout,
                    &mut blockquote_buf,
                    &mut table_buf,
                    &mut list_buf,
                    theme,
                );
                code_lang = lang.trim().to_owned();
                in_code = true;
            }
            continue;
        }
        if in_code {
            code_buf.push_str(line);
            code_buf.push('\n');
            continue;
        }

        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            // Flush any open blockquote first
            if in_blockquote {
                if !is_callout {
                    out.push_str(&render_blockquote(&blockquote_buf, theme));
                }
                blockquote_buf.clear();
                in_blockquote = false;
                is_callout = false;
            }
            if !list_buf.is_empty() {
                out.push_str(&render_list(&list_buf, theme));
                list_buf.clear();
            }
            table_buf.push(line);
            continue;
        } else if !table_buf.is_empty() {
            out.push_str(&render_table(&table_buf, theme));
            table_buf.clear();
        }

        if let Some(item) = parse_list_item(trimmed) {
            if in_blockquote {
                if !is_callout {
                    out.push_str(&render_blockquote(&blockquote_buf, theme));
                }
                blockquote_buf.clear();
                in_blockquote = false;
                is_callout = false;
            }
            list_buf.push(item);
            continue;
        } else if !list_buf.is_empty() {
            out.push_str(&render_list(&list_buf, theme));
            list_buf.clear();
        }

        if let Some(rest) = line
            .strip_prefix("> ")
            .or_else(|| if line == ">" { Some("") } else { None })
        {
            if !in_blockquote {
                is_callout = rest.starts_with("[!");
                in_blockquote = true;
            }
            if !is_callout && !rest.is_empty() {
                if !blockquote_buf.is_empty() {
                    blockquote_buf.push('\n');
                }
                blockquote_buf.push_str(rest);
            }
            continue;
        }
        if in_blockquote {
            if !is_callout {
                out.push_str(&render_blockquote(&blockquote_buf, theme));
            }
            blockquote_buf.clear();
            in_blockquote = false;
            is_callout = false;
        }

        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            out.push_str(&render_hr(theme));
            continue;
        }

        if let Some(rest) = line.strip_prefix("### ") {
            out.push_str(&render_h3(rest, theme));
            continue;
        }
        if let Some(rest) = line.strip_prefix("## ") {
            out.push_str(&render_h2(rest, theme));
            continue;
        }
        if let Some(rest) = line.strip_prefix("# ") {
            out.push_str(&render_h2(rest, theme));
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("![") {
            let chars: Vec<char> = trimmed.chars().collect();
            if let Some((alt, url, _)) = parse_image(&chars) {
                out.push_str(&format!(
                    "<p style=\"margin: 1.5em 0; text-align: center;\"><img src=\"{url}\" alt=\"{alt}\" style=\"max-width: 100%; display: block; margin: 0 auto;\" /></p>\n\n"
                ));
                continue;
            }
        }
        out.push_str(&render_p(line, theme));
    }

    // Flush remaining buffers
    if !table_buf.is_empty() {
        out.push_str(&render_table(&table_buf, theme));
    }
    if !list_buf.is_empty() {
        out.push_str(&render_list(&list_buf, theme));
    }
    if in_code {
        out.push_str(&crate::illustrate::render_code_block(
            &code_lang,
            code_buf.trim_end(),
            theme,
        ));
    }
    if in_blockquote && !is_callout {
        out.push_str(&render_blockquote(&blockquote_buf, theme));
    }

    out
}

fn flush_paragraph_buffers(
    out: &mut String,
    in_blockquote: &mut bool,
    is_callout: &mut bool,
    blockquote_buf: &mut String,
    table_buf: &mut Vec<&str>,
    list_buf: &mut Vec<ListItem>,
    theme: &theme::Theme,
) {
    if !table_buf.is_empty() {
        out.push_str(&render_table(table_buf, theme));
        table_buf.clear();
    }
    if !list_buf.is_empty() {
        out.push_str(&render_list(list_buf, theme));
        list_buf.clear();
    }
    if *in_blockquote {
        if !*is_callout {
            out.push_str(&render_blockquote(blockquote_buf, theme));
        }
        blockquote_buf.clear();
        *in_blockquote = false;
        *is_callout = false;
    }
}

#[derive(Debug)]
struct ListItem {
    marker: ListMarker,
    text: String,
}

#[derive(Debug)]
enum ListMarker {
    Bullet,
    Ordered(usize),
}

fn parse_list_item(trimmed: &str) -> Option<ListItem> {
    if let Some(rest) = trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))
        .or_else(|| trimmed.strip_prefix("+ "))
    {
        return Some(ListItem {
            marker: ListMarker::Bullet,
            text: rest.trim().to_owned(),
        });
    }

    let (num, rest) = trimmed.split_once(". ")?;
    let value = num.parse().ok()?;
    Some(ListItem {
        marker: ListMarker::Ordered(value),
        text: rest.trim().to_owned(),
    })
}

fn render_list(items: &[ListItem], theme: &theme::Theme) -> String {
    let mut html = String::from(
        "<section class=\"moonpub-list\" style=\"margin: 18px 0 22px; padding: 4px 0;\">\n<table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\">\n",
    );
    for (idx, item) in items.iter().enumerate() {
        let marker = match item.marker {
            ListMarker::Bullet => format!(
                "<span style=\"display:inline-block;width:7px;height:7px;background:{};border-radius:50%;vertical-align:middle;\"></span>",
                theme.accent
            ),
            ListMarker::Ordered(n) => format!(
                "<span style=\"display:inline-block;min-width:22px;height:22px;padding:0 4px;background:{};color:#fff;border-radius:999px;text-align:center;line-height:22px;font-size:12px;font-weight:bold;\">{n}</span>",
                theme.accent
            ),
        };
        let bg = if idx % 2 == 0 {
            theme.accent_soft
        } else {
            theme.section_bg
        };
        html.push_str(&format!(
            "<tr><td style=\"width:34px;padding:8px 0 8px 4px;vertical-align:top;text-align:center;background:{bg};\">{marker}</td><td style=\"padding:8px 12px 8px 4px;vertical-align:top;background:{bg};color:{};font-size:15px;line-height:1.8;\">{}</td></tr>\n",
            theme.text_color,
            inline_md(&item.text, theme)
        ));
    }
    html.push_str("</table></section>\n\n");
    html
}

fn render_hr(theme: &theme::Theme) -> String {
    format!(
        "<section style=\"margin: 2.2em 0; text-align:center;\"><section style=\"display:inline-block;width:42px;height:2px;background:{};vertical-align:middle;\"></section><span style=\"display:inline-block;width:6px;height:6px;border:1px solid {};border-radius:50%;margin:0 10px;vertical-align:middle;background:{};\"></span><section style=\"display:inline-block;width:42px;height:2px;background:{};vertical-align:middle;\"></section></section>\n\n",
        theme.border, theme.accent, theme.section_bg, theme.border
    )
}

/// Render inline markdown formatting: `code`, **bold**, *italic*, and images.
fn inline_md(text: &str, theme: &theme::Theme) -> String {
    // Order matters: code spans are rendered literally first so that backticks inside
    // bold/italic do not get consumed by the emphasis parsers.
    let mut s = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '`' {
            let end = chars[i + 1..].iter().position(|&c| c == '`');
            if let Some(rel) = end {
                let code: String = chars[i + 1..i + 1 + rel].iter().collect();
                let color_style = if theme.code_color.is_empty() {
                    String::new()
                } else {
                    format!("color:{};", theme.code_color)
                };
                s.push_str(&format!(
                    "<code style=\"background:{};{}padding:2px 4px;border-radius:3px;font-size:14px;font-family:monospace;\">{}</code>",
                    theme.code_bg, color_style, html_escape(&code)
                ));
                i += rel + 2;
                continue;
            }
        }
        if chars[i] == '*' && i + 1 < chars.len() && chars[i + 1] == '*' {
            let end = chars[i + 2..].windows(2).position(|w| w == ['*', '*']);
            if let Some(rel) = end {
                let inner: String = chars[i + 2..i + 2 + rel].iter().collect();
                s.push_str(&format!(
                    "<strong style=\"color: {};\">{}</strong>",
                    theme.heading_color,
                    inline_md(&inner, theme)
                ));
                i += rel + 4;
                continue;
            }
        }
        if chars[i] == '*' {
            let end = chars[i + 1..].iter().position(|&c| c == '*');
            if let Some(rel) = end {
                let inner: String = chars[i + 1..i + 1 + rel].iter().collect();
                s.push_str(&format!("<em>{}</em>", inline_md(&inner, theme)));
                i += rel + 2;
                continue;
            }
        }
        // image ![alt](url) — must be tested before plain link [text](url)
        if chars[i] == '!'
            && i + 1 < chars.len()
            && chars[i + 1] == '['
            && let Some((alt, url, consumed)) = parse_image(&chars[i..])
        {
            s.push_str(&format!(
                "<p style=\"margin: 1.5em 0; text-align: center;\"><img src=\"{url}\" alt=\"{alt}\" style=\"max-width: 100%; display: block; margin: 0 auto;\" /></p>"
            ));
            i += consumed;
            continue;
        }
        // link [text](url)
        if chars[i] == '['
            && let Some((text, url, consumed)) = parse_link(&chars[i..])
        {
            s.push_str(&format!(
                "<a href=\"{url}\" style=\"color: #576b95; text-decoration: none;\">{}</a>",
                inline_md(&text, theme)
            ));
            i += consumed;
            continue;
        }
        s.push(chars[i]);
        i += 1;
    }
    s
}

fn parse_link(chars: &[char]) -> Option<(String, String, usize)> {
    // [text](url)
    if chars.first() != Some(&'[') {
        return None;
    }
    let text_end = chars[1..].iter().position(|&c| c == ']')?;
    let text: String = chars[1..1 + text_end].iter().collect();
    let rest = &chars[1 + text_end + 1..];
    if rest.first() != Some(&'(') {
        return None;
    }
    let url_end = rest[1..].iter().position(|&c| c == ')')?;
    let url: String = rest[1..1 + url_end].iter().collect();
    Some((text, url, 1 + text_end + 1 + 1 + url_end + 1))
}

fn parse_image(chars: &[char]) -> Option<(String, String, usize)> {
    // ![alt](url)
    if chars.len() < 5 || chars[0] != '!' || chars[1] != '[' {
        return None;
    }
    let alt_end = chars[2..].iter().position(|&c| c == ']')?;
    let alt: String = chars[2..2 + alt_end].iter().collect();
    let rest = &chars[2 + alt_end + 1..];
    if rest.first() != Some(&'(') {
        return None;
    }
    let url_end = rest[1..].iter().position(|&c| c == ')')?;
    let url: String = rest[1..1 + url_end].iter().collect();
    Some((alt, url, 2 + alt_end + 1 + 1 + url_end + 1))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn render_h2(text: &str, theme: &theme::Theme) -> String {
    format!(
        "<h2 style=\"font-size: 19px; font-weight: 800; color: {}; margin: 2.3em 0 1em; padding: 0 0 10px 14px; border-left: 5px solid {}; border-bottom: 1px solid {}; letter-spacing: 1px; line-height:1.45;\">{}</h2>\n\n",
        theme.heading_color,
        theme.heading_border,
        theme.border,
        inline_md(text, theme)
    )
}

fn render_h3(text: &str, theme: &theme::Theme) -> String {
    format!(
        "<h3 style=\"font-size: 16px; font-weight: 700; color: {}; margin: 1.8em 0 0.8em; padding: 8px 12px; border-left: 3px solid {}; background: {}; border-radius: 0 6px 6px 0; letter-spacing: 0.05em; line-height:1.5;\">{}</h3>\n\n",
        theme.heading_color,
        theme.accent,
        theme.accent_soft,
        inline_md(text, theme)
    )
}

fn render_p(text: &str, theme: &theme::Theme) -> String {
    format!(
        "<p style=\"margin: 0 0 1.25em; color: {}; font-size: 15px; line-height: 1.95; letter-spacing: 0.08em; word-spacing: 0.05em; text-align: justify; text-indent: 2em;\">{}</p>\n\n",
        theme.text_color,
        inline_md(text, theme)
    )
}

fn render_blockquote(text: &str, theme: &theme::Theme) -> String {
    // Use <section> instead of <blockquote>: new WeChat editor strips <blockquote> inline styles (doocs/md issue #447)
    format!(
        "<section style=\"margin: 1.9em 0; padding: 18px 20px 18px 24px; background: {}; border: 1px solid {}; border-left: 4px solid {}; border-radius: 0 8px 8px 0; box-shadow: 0 2px 8px rgba(15,23,42,0.05); color: {}; font-size: 15px; line-height: 1.9; letter-spacing: 0.08em;\">{}</section>\n\n",
        theme.block_bg,
        theme.border,
        theme.accent,
        theme.text_muted,
        inline_md(text, theme)
    )
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
