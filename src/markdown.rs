//! Markdown → WeChat HTML conversion.
//!
//! This module handles only the syntactic transformation: parsing `:::name` fences,
//! plain markdown segments, inline formatting, and emitting inline-styled HTML.
//! It intentionally knows nothing about file I/O, frontmatter, or WeChat API drafts.

use crate::illustrate;
use crate::theme;

mod parser;
#[cfg(test)]
use parser::split_fence_props;
use parser::{MdBlock, parse_blocks};

/// Convert a markdown body into WeChat-compatible HTML using the given theme.
pub fn md_to_wechat_html(md: &str, theme: &theme::Theme) -> String {
    let blocks = parse_blocks(md);
    let mut out = String::new();

    for block in &blocks {
        match block {
            MdBlock::Fence(name, props, body) => {
                out.push_str(&render_fence_block(name, props, body, theme))
            }
            MdBlock::Markdown(text) => out.push_str(&render_markdown_segment(text, theme)),
        }
    }

    out
}

// ── Fence block renderers ────────────────────────────────────────────────────

fn render_fence_block(
    name: &str,
    props: &[(&str, &str)],
    body: &str,
    theme: &theme::Theme,
) -> String {
    match name {
        "book-info" => render_book_info(props, theme),
        "intro" => render_intro(body, theme),
        "callout" => render_callout(props, body, theme),
        "steps" => render_steps(body, theme),
        "summary" => render_summary(body, theme),
        "figure" => render_figure(props, theme),
        "checklist" => render_checklist(body, theme),
        "cover" => render_cover(props, theme),
        "quote-card" => {
            let text = body.trim().to_owned();
            let source = props
                .iter()
                .find(|(k, _)| *k == "source")
                .map(|(_, v)| *v)
                .unwrap_or("");
            illustrate::render_illustration(
                &illustrate::IllustType::QuoteCard {
                    text,
                    source: source.to_owned(),
                },
                theme,
            )
        }
        "divider" => {
            let label = props
                .iter()
                .find(|(k, _)| *k == "label")
                .map(|(_, v)| *v)
                .unwrap_or("");
            illustrate::render_illustration(
                &illustrate::IllustType::Divider {
                    label: label.to_owned(),
                },
                theme,
            )
        }
        "concept-card" => {
            let number: u32 = props
                .iter()
                .find(|(k, _)| *k == "number")
                .and_then(|(_, v)| v.parse().ok())
                .unwrap_or(1);
            let title = body.lines().next().unwrap_or("").trim().to_owned();
            let desc = body
                .lines()
                .skip(1)
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_owned();
            illustrate::render_illustration(
                &illustrate::IllustType::ConceptCard {
                    number,
                    title,
                    desc,
                },
                theme,
            )
        }
        "emotion-card" => {
            let mood = props
                .iter()
                .find(|(k, _)| *k == "mood")
                .map(|(_, v)| *v)
                .unwrap_or("think");
            illustrate::render_illustration(
                &illustrate::IllustType::EmotionCard {
                    mood: mood.to_owned(),
                    text: body.trim().to_owned(),
                },
                theme,
            )
        }
        "code" => {
            let lang = props
                .iter()
                .find(|(k, _)| *k == "lang")
                .map(|(_, v)| *v)
                .unwrap_or("");
            illustrate::render_code_block(lang, body.trim(), theme)
        }
        "timeline" => {
            let items: Vec<(String, String)> = body
                .lines()
                .filter(|l| l.trim().starts_with("- "))
                .filter_map(|l| {
                    let s = l.trim().trim_start_matches("- ").trim();
                    s.split_once(": ")
                        .map(|(d, t)| (d.to_owned(), t.to_owned()))
                })
                .collect();
            if items.is_empty() {
                render_generic_fence("timeline", body, theme)
            } else {
                illustrate::render_timeline(&items, theme)
            }
        }
        "comparison" => {
            let left = props
                .iter()
                .find(|(k, _)| *k == "left")
                .map(|(_, v)| *v)
                .unwrap_or("A");
            let right = props
                .iter()
                .find(|(k, _)| *k == "right")
                .map(|(_, v)| *v)
                .unwrap_or("B");
            let rows: Vec<(String, String)> = body
                .lines()
                .filter(|l| l.trim().starts_with("- "))
                .filter_map(|l| {
                    let s = l.trim().trim_start_matches("- ").trim();
                    s.split_once(" | ")
                        .map(|(a, b)| (a.to_owned(), b.to_owned()))
                })
                .collect();
            if rows.is_empty() {
                render_generic_fence("comparison", body, theme)
            } else {
                illustrate::render_comparison(left, right, &rows, theme)
            }
        }
        "tip" => {
            let icon = props
                .iter()
                .find(|(k, _)| *k == "icon")
                .map(|(_, v)| *v)
                .unwrap_or("");
            illustrate::render_tip(icon, body.trim(), theme)
        }
        _ => {
            // Unknown block — render as a styled container
            render_generic_fence(name, body, theme)
        }
    }
}

fn render_book_info(props: &[(&str, &str)], theme: &theme::Theme) -> String {
    let get = |key: &str| -> &str {
        props
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| *v)
            .unwrap_or("")
    };
    let title = get("title");
    let author = get("author");
    let cover = get("cover");
    let publisher = get("publisher");
    let rating = get("rating");
    let has_cover = !cover.is_empty();

    let mut html = String::new();
    html.push_str(&format!(
        "<section style=\"margin: 24px 0; background: {}; border: 1px solid #e8e8e8; border-radius: 6px; overflow: hidden;\">\n",
        theme.block_bg
    ));
    html.push_str("<table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\"><tr>\n");

    if has_cover {
        html.push_str(&format!(
            "<td style=\"width:90px;padding:16px;vertical-align:top;\"><img src=\"{cover}\" style=\"width:90px;height:auto;border-radius:4px;box-shadow:0 2px 8px rgba(0,0,0,0.12);\" /></td>\n"
        ));
    }
    html.push_str("<td style=\"padding:16px;vertical-align:middle;\">\n");
    html.push_str(&format!(
        "<p style=\"margin:0 0 6px;font-size:16px;font-weight:bold;color:{};\">《{title}》</p>\n",
        theme.heading_color
    ));
    if !author.is_empty() {
        html.push_str(&format!(
            "<p style=\"margin:0 0 4px;font-size:13px;color:{};\">{author} 著</p>\n",
            theme.text_muted
        ));
    }
    if !publisher.is_empty() || !rating.is_empty() {
        let pub_str = if rating.is_empty() {
            publisher.to_owned()
        } else {
            format!("{publisher} | 豆瓣 {rating}")
        };
        html.push_str(&format!(
            "<p style=\"margin:0;font-size:12px;color:{};\">{pub_str}</p>\n",
            theme.text_muted
        ));
    }
    html.push_str("</td>\n");
    html.push_str("</tr></table>\n");
    html.push_str("</section>\n\n");
    html
}

fn render_intro(body: &str, theme: &theme::Theme) -> String {
    format!(
        "<section style=\"margin: 24px 0; padding: 20px 24px; background: {}; border-left: 4px solid {}; font-size: 16px; color: {}; line-height: 1.9; letter-spacing: 0.5px;\">\n{}\n</section>\n\n",
        theme.block_bg,
        theme.accent,
        theme.text_color,
        inline_md(body.trim(), theme)
    )
}

fn render_callout(props: &[(&str, &str)], body: &str, theme: &theme::Theme) -> String {
    let label = props
        .iter()
        .find(|(k, _)| *k == "label")
        .map(|(_, v)| *v)
        .unwrap_or("重点");
    format!(
        "<section style=\"margin: 24px 0;\"><table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\"><tr>\n<td style=\"background:{};color:#fff;font-weight:bold;font-size:13px;padding:12px 16px;white-space:nowrap;letter-spacing:2px;vertical-align:top;\">{label}</td>\n<td style=\"background:{};border:1px solid {};border-left:none;padding:14px 18px;font-size:15px;line-height:1.85;color:{};\">{}</td>\n</tr></table></section>\n\n",
        theme.accent,
        theme.block_bg,
        theme.accent,
        theme.heading_color,
        inline_md(body.trim(), theme)
    )
}

fn render_steps(body: &str, theme: &theme::Theme) -> String {
    let items: Vec<&str> = body
        .lines()
        .filter(|l| l.trim().starts_with(|c: char| c.is_ascii_digit()) && l.trim().contains(". "))
        .filter_map(|l| l.trim().split_once(". ").map(|(_, rest)| rest))
        .collect();

    if items.is_empty() {
        return render_generic_fence("steps", body, theme);
    }
    let count = items.len();
    let mut html = String::new();
    html.push_str("<section style=\"margin: 24px 0;\"><table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\"><tr>\n");

    let pct = 100usize.div_ceil(count);
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            html.push_str("<td style=\"width:8px;\"></td>\n");
        }
        html.push_str(&format!(
            "<td style=\"width:{pct}%;background:#fff;border:1px solid #e8e8e8;padding:14px 12px;vertical-align:top;\">\n<section style=\"display:inline-block;width:24px;height:24px;background:{};color:#fff;font-weight:bold;text-align:center;line-height:24px;border-radius:50%;font-size:13px;margin-bottom:8px;\">{}</section>\n<p style=\"margin:0;font-size:13px;color:{};line-height:1.7;\">{}</p>\n</td>\n",
            theme.accent, i + 1, theme.text_color, inline_md(item, theme),
        ));
    }
    html.push_str("</tr></table></section>\n\n");
    html
}

fn render_summary(body: &str, theme: &theme::Theme) -> String {
    format!(
        "<section style=\"margin: 24px 0;\"><table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\"><tr>\n<td style=\"background:{};color:#fff;font-weight:bold;font-size:13px;padding:10px 14px;white-space:nowrap;letter-spacing:1px;vertical-align:top;\">总 结</td>\n<td style=\"background:#fff;border:1px solid {};border-left:none;padding:12px 16px;font-size:14px;line-height:1.8;color:{};\">{}</td>\n</tr></table></section>\n\n",
        theme.accent,
        theme.accent,
        theme.heading_color,
        inline_md(body.trim(), theme)
    )
}

fn render_figure(props: &[(&str, &str)], theme: &theme::Theme) -> String {
    let image = props
        .iter()
        .find(|(k, _)| *k == "image")
        .map(|(_, v)| *v)
        .unwrap_or("");
    let caption = props
        .iter()
        .find(|(k, _)| *k == "caption")
        .map(|(_, v)| *v)
        .unwrap_or("");
    if image.is_empty() {
        return String::new();
    }
    let cap_html = if caption.is_empty() {
        String::new()
    } else {
        format!(
            "<p style=\"margin:0;padding:10px 14px;background:{};color:{};font-size:12px;text-align:center;\">{caption}</p>",
            theme.block_bg, theme.text_muted
        )
    };
    format!(
        "<section style=\"margin: 24px 0;\"><section style=\"border:2px solid #e8e8e8;padding:0;background:{};\">\n<img src=\"{image}\" style=\"display:block;width:100%;height:auto;\" />\n{cap_html}</section></section>\n\n",
        theme.block_bg
    )
}

fn render_checklist(body: &str, theme: &theme::Theme) -> String {
    let items: Vec<&str> = body
        .lines()
        .filter(|l| l.trim().starts_with("- [") || l.trim().starts_with("- ["))
        .collect();
    if items.is_empty() {
        return render_generic_fence("checklist", body, theme);
    }
    let mut html = String::new();
    html.push_str(&format!(
        "<section style=\"margin:18px 0;\"><section style=\"background:{};border:1px solid #e8e8e8;padding:18px 20px;\"><table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\">\n",
        theme.block_bg
    ));
    let half = items.len().div_ceil(2);
    for row in 0..half {
        html.push_str("<tr>\n");
        for col in 0..2 {
            let idx = if col == 0 { row } else { row + half };
            if idx < items.len() {
                let item = items[idx]
                    .trim()
                    .trim_start_matches("- [")
                    .trim_start_matches("- [")
                    .trim_end_matches(']');
                let rest = if item.starts_with('x') || item.starts_with("x ") {
                    let content = item[1..].trim();
                    format!(
                        "<span style=\"color:{};font-weight:bold;\">✔</span>&nbsp;&nbsp;{content}",
                        theme.accent
                    )
                } else {
                    let content = item[1..].trim();
                    format!(
                        "<span style=\"color:#ccc;font-weight:bold;\">○</span>&nbsp;&nbsp;{content}"
                    )
                };
                html.push_str(&format!(
                    "<td style=\"width:50%;padding:6px 0;font-size:14px;color:{};vertical-align:top;\">{rest}</td>\n",
                    theme.text_color
                ));
            } else {
                html.push_str("<td style=\"width:50%;\"></td>\n");
            }
        }
        html.push_str("</tr>\n");
    }
    html.push_str("</table></section></section>\n\n");
    html
}

fn render_cover(props: &[(&str, &str)], theme: &theme::Theme) -> String {
    let get = |key: &str| -> &str {
        props
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| *v)
            .unwrap_or("")
    };
    let title = get("title");
    let subtitle = get("subtitle");
    format!(
        "<section style=\"margin:0;background:{};padding:48px 24px 36px;color:#fff;\">\n<section style=\"display:inline-block;background:#fff;color:{};font-size:11px;font-weight:bold;letter-spacing:2px;padding:4px 10px;margin-bottom:18px;\">READING · NOTES</section>\n<h1 style=\"margin:0 0 8px;font-size:28px;font-weight:900;line-height:1.2;color:#fff;\">{title}</h1>\n<p style=\"margin:8px 0 0;font-size:14px;color:{};\">{subtitle}</p>\n</section>\n\n",
        theme.accent, theme.accent, theme.text_muted
    )
}

fn render_generic_fence(_name: &str, body: &str, theme: &theme::Theme) -> String {
    format!(
        "<section style=\"margin: 18px 0; padding: 16px 20px; background: {}; border: 1px solid #e8e8e8; border-radius: 4px;\">\n{}\n</section>\n\n",
        theme.block_bg,
        inline_md(body.trim(), theme)
    )
}

// ── Plain markdown segment renderer ───────────────────────────────────────────

fn render_table(lines: &[&str], theme: &theme::Theme) -> String {
    let mut html = "<section style=\"margin: 16px 0; overflow-x: auto;\"><table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse: collapse; width: 100%; font-size: 14px;\">\n".to_string();
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
                        "<th style=\"padding: 8px 12px; background: {}; color: {}; font-weight: bold; border: 1px solid #e0e0e0; text-align: left;\">{}</th>\n",
                        theme.accent, "#fff", inline_md(cell, theme)
                    ));
                } else {
                    html.push_str(&format!(
                        "<td style=\"padding: 7px 12px; border: 1px solid #e0e0e0; color: {}; vertical-align: top;\">{}</td>\n",
                        theme.text_color, inline_md(cell, theme)
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

    let lines: Vec<&str> = md.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        i += 1;

        // Collect consecutive table rows
        let trimmed = line.trim();
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
            table_buf.push(line);
            continue;
        } else if !table_buf.is_empty() {
            out.push_str(&render_table(&table_buf, theme));
            table_buf.clear();
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
            out.push_str(
                "<hr style=\"border: none; border-top: 1px solid #eee; margin: 2em 0;\" />\n\n",
            );
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
    if in_blockquote && !is_callout {
        out.push_str(&render_blockquote(&blockquote_buf, theme));
    }

    out
}

/// Render inline markdown formatting: `code`, **bold**, *italic*, and images.
pub fn inline_md(text: &str, theme: &theme::Theme) -> String {
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
        "<h2 style=\"font-size: 18px; font-weight: bold; color: {}; margin: 2em 0 0.8em; padding: 0 0 8px 12px; border-left: 4px solid {}; border-bottom: 1px solid #f0f0f0; letter-spacing: 1px;\">{}</h2>\n\n",
        theme.heading_color,
        theme.heading_border,
        inline_md(text, theme)
    )
}

fn render_h3(text: &str, theme: &theme::Theme) -> String {
    // Left border + background tint — matches doocs/md community pattern
    format!(
        "<h3 style=\"font-size: 16px; font-weight: bold; color: {}; margin: 1.5em 0 0.6em; padding: 6px 12px; border-left: 3px solid {}; background: {}18; border-radius: 0 4px 4px 0; letter-spacing: 0.05em;\">{}</h3>\n\n",
        theme.heading_color,
        theme.accent,
        theme.accent,
        inline_md(text, theme)
    )
}

fn render_p(text: &str, theme: &theme::Theme) -> String {
    format!(
        "<p style=\"margin: 0 0 1.2em; color: {}; font-size: 15px; line-height: 1.85; letter-spacing: 0.1em; word-spacing: 0.05em; text-align: justify; text-indent: 2em;\">{}</p>\n\n",
        theme.text_color,
        inline_md(text, theme)
    )
}

fn render_blockquote(text: &str, theme: &theme::Theme) -> String {
    // Use <section> instead of <blockquote>: new WeChat editor strips <blockquote> inline styles (doocs/md issue #447)
    format!(
        "<section style=\"margin: 1.8em 0; padding: 16px 20px 16px 24px; background: {}; border-left: 4px solid {}; border-radius: 0 6px 6px 0; box-shadow: 0 1px 3px rgba(0,0,0,0.04); color: {}; font-size: 15px; line-height: 1.85; letter-spacing: 0.1em;\">{}</section>\n\n",
        theme.block_bg,
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
}
