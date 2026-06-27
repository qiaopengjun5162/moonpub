use crate::theme;

use super::inline::{inline_md, parse_image};

pub(super) fn render_markdown_segment(md: &str, theme: &theme::Theme) -> String {
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

fn render_table(lines: &[&str], theme: &theme::Theme) -> String {
    let mut html = format!(
        "<section style=\"margin: 22px 0; overflow-x: auto; border:1px solid {}; border-radius:6px;\"><table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse: collapse; width: 100%; font-size: 14px;\">\n",
        theme.border
    );
    let mut is_header = true;
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
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
                        theme.header_bg,
                        "#fff",
                        theme.border,
                        inline_md(cell, theme)
                    ));
                } else {
                    html.push_str(&format!(
                        "<td style=\"padding: 9px 12px; border-top: 1px solid {}; border-right: 1px solid {}; color: {}; vertical-align: top; line-height:1.7;\">{}</td>\n",
                        theme.border,
                        theme.border,
                        theme.text_color,
                        inline_md(cell, theme)
                    ));
                }
            }
            html.push_str("</tr>\n");
        }
    }
    html.push_str("</table></section>\n\n");
    html
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
    format!(
        "<section style=\"margin: 1.9em 0; padding: 18px 20px 18px 24px; background: {}; border: 1px solid {}; border-left: 4px solid {}; border-radius: 0 8px 8px 0; box-shadow: 0 2px 8px rgba(15,23,42,0.05); color: {}; font-size: 15px; line-height: 1.9; letter-spacing: 0.08em;\">{}</section>\n\n",
        theme.block_bg,
        theme.border,
        theme.accent,
        theme.text_muted,
        inline_md(text, theme)
    )
}
