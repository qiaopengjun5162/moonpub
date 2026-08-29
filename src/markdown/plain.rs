use crate::theme;

use super::inline::{inline_md, parse_image};

pub(super) fn render_markdown_segment(md: &str, theme: &theme::Theme) -> String {
    let mut out = String::new();
    let mut in_blockquote = false;
    let mut is_callout = false;
    let mut blockquote_buf = String::new();
    let mut table_buf: Vec<&str> = Vec::new();
    let mut checklist_buf: Vec<ChecklistItem> = Vec::new();
    let mut list_buf: Vec<ListItem> = Vec::new();
    let mut code_lang = String::new();
    let mut code_buf = String::new();
    let mut in_code = false;
    let mut has_lead_paragraph = false;

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
                ));
                code_buf.clear();
                code_lang.clear();
                in_code = false;
            } else {
                flush_paragraph_buffers(
                    &mut out,
                    &mut ParagraphBuffers {
                        in_blockquote: &mut in_blockquote,
                        is_callout: &mut is_callout,
                        blockquote_buf: &mut blockquote_buf,
                        table_buf: &mut table_buf,
                        checklist_buf: &mut checklist_buf,
                        list_buf: &mut list_buf,
                    },
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
            if !checklist_buf.is_empty() {
                out.push_str(&render_checklist(&checklist_buf, theme));
                checklist_buf.clear();
            }
            table_buf.push(line);
            continue;
        } else if !table_buf.is_empty() {
            out.push_str(&render_table(&table_buf, theme));
            table_buf.clear();
        }

        if let Some(item) = parse_checklist_item(trimmed) {
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
            checklist_buf.push(item);
            continue;
        } else if !checklist_buf.is_empty() {
            out.push_str(&render_checklist(&checklist_buf, theme));
            checklist_buf.clear();
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

        if let Some(rest) = line.strip_prefix("#### ") {
            out.push_str(&render_h4(rest, theme));
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
                out.push_str(&render_image_figure(&alt, &url, theme));
                continue;
            }
        }
        if has_lead_paragraph {
            out.push_str(&render_p(line, theme));
        } else {
            out.push_str(&render_lead_p(line, theme));
            has_lead_paragraph = true;
        }
    }

    if !table_buf.is_empty() {
        out.push_str(&render_table(&table_buf, theme));
    }
    if !checklist_buf.is_empty() {
        out.push_str(&render_checklist(&checklist_buf, theme));
    }
    if !list_buf.is_empty() {
        out.push_str(&render_list(&list_buf, theme));
    }
    if in_code {
        out.push_str(&crate::illustrate::render_code_block(
            &code_lang,
            code_buf.trim_end(),
        ));
    }
    if in_blockquote && !is_callout {
        out.push_str(&render_blockquote(&blockquote_buf, theme));
    }

    out
}

fn render_table(lines: &[&str], theme: &theme::Theme) -> String {
    let mut headers: Vec<&str> = Vec::new();
    let mut rows: Vec<Vec<&str>> = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            let inner = &trimmed[1..trimmed.len() - 1];
            if inner
                .split('|')
                .all(|c| c.trim().chars().all(|x| x == '-' || x == ':' || x == ' '))
            {
                continue;
            }
            let cells: Vec<&str> = inner.split('|').collect();
            if headers.is_empty() {
                headers = cells.into_iter().map(str::trim).collect();
            } else {
                rows.push(cells.into_iter().map(str::trim).collect());
            }
        }
    }

    if headers.is_empty() {
        return String::new();
    }

    let mut html = format!(
        "<section style=\"margin:22px 0;padding:12px 14px;background:{};border:1px solid {};border-radius:10px;\">\n",
        theme.block_bg, theme.border
    );
    for row in rows {
        html.push_str(&format!(
            "<section style=\"margin:0 0 12px;padding:12px 14px;background:{};border:1px solid {};border-radius:8px;\">\n",
            theme.section_bg, theme.border
        ));
        for (idx, header) in headers.iter().enumerate() {
            let cell = row.get(idx).copied().unwrap_or("");
            if cell.is_empty() {
                continue;
            }
            html.push_str(&format!(
                "<p style=\"margin:0 0 7px;font-size:14px;line-height:1.75;color:{};\"><span style=\"display:block;margin:0 0 2px;color:{};font-size:12px;font-weight:bold;letter-spacing:0.08em;\">{}</span>{}</p>\n",
                theme.text_color,
                theme.accent,
                inline_md(header, theme),
                inline_md(cell, theme)
            ));
        }
        html.push_str("</section>\n");
    }
    html.push_str("</section>\n\n");
    html
}

struct ParagraphBuffers<'a, 'md> {
    in_blockquote: &'a mut bool,
    is_callout: &'a mut bool,
    blockquote_buf: &'a mut String,
    table_buf: &'a mut Vec<&'md str>,
    checklist_buf: &'a mut Vec<ChecklistItem>,
    list_buf: &'a mut Vec<ListItem>,
}

fn flush_paragraph_buffers(
    out: &mut String,
    buffers: &mut ParagraphBuffers<'_, '_>,
    theme: &theme::Theme,
) {
    if !buffers.table_buf.is_empty() {
        out.push_str(&render_table(buffers.table_buf, theme));
        buffers.table_buf.clear();
    }
    if !buffers.list_buf.is_empty() {
        out.push_str(&render_list(buffers.list_buf, theme));
        buffers.list_buf.clear();
    }
    if !buffers.checklist_buf.is_empty() {
        out.push_str(&render_checklist(buffers.checklist_buf, theme));
        buffers.checklist_buf.clear();
    }
    if *buffers.in_blockquote {
        if !*buffers.is_callout {
            out.push_str(&render_blockquote(buffers.blockquote_buf, theme));
        }
        buffers.blockquote_buf.clear();
        *buffers.in_blockquote = false;
        *buffers.is_callout = false;
    }
}

#[derive(Debug)]
struct ChecklistItem {
    checked: bool,
    text: String,
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

fn parse_checklist_item(trimmed: &str) -> Option<ChecklistItem> {
    let rest = trimmed.strip_prefix("- [")?;
    let (marker, content) = rest.split_once("] ")?;
    let marker = marker.trim();
    let checked = if marker.eq_ignore_ascii_case("x") {
        true
    } else if marker.is_empty() {
        false
    } else {
        return None;
    };
    Some(ChecklistItem {
        checked,
        text: content.trim().to_owned(),
    })
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

fn render_checklist(items: &[ChecklistItem], theme: &theme::Theme) -> String {
    let mut html = format!(
        "<section style=\"margin: 18px 0 22px; padding: 14px 16px; background: {}; border: 1px solid {}; border-radius: 8px;\">\n<table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\">\n",
        theme.block_bg, theme.border
    );
    for item in items {
        let (mark, color, bg) = if item.checked {
            ("✔", theme.accent, theme.accent_soft)
        } else {
            ("○", theme.text_muted, theme.section_bg)
        };
        html.push_str(&format!(
            "<tr><td style=\"width:34px;padding:7px 0;vertical-align:top;text-align:center;\"><span style=\"display:inline-block;width:22px;height:22px;line-height:22px;text-align:center;border-radius:50%;background:{bg};color:{color};font-weight:bold;font-size:13px;\">{mark}</span></td><td style=\"padding:7px 0 7px 8px;vertical-align:top;color:{};font-size:15px;line-height:1.8;\">{}</td></tr>\n",
            theme.text_color,
            inline_md(&item.text, theme)
        ));
    }
    html.push_str("</table></section>\n\n");
    html
}

fn render_list(items: &[ListItem], theme: &theme::Theme) -> String {
    let mut html = String::from(
        "<section style=\"margin: 18px 0 22px; padding: 4px 0;\">\n<table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\">\n",
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

fn render_h4(text: &str, theme: &theme::Theme) -> String {
    format!(
        "<h4 style=\"font-size: 15px; font-weight: 700; color: {}; margin: 1.5em 0 0.75em; padding: 0 0 0 10px; border-left: 3px solid {}; letter-spacing: 0.06em; line-height:1.5;\">{}</h4>\n\n",
        theme.heading_color,
        theme.accent,
        inline_md(text, theme)
    )
}

fn render_lead_p(text: &str, theme: &theme::Theme) -> String {
    format!(
        "<p style=\"margin: 0 0 1.55em; color: {}; font-size: 16px; line-height: 2.05; letter-spacing: 0.08em; word-spacing: 0.05em; text-align: justify;\">{}</p>\n\n",
        theme.text_color,
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

fn render_image_figure(alt: &str, url: &str, theme: &theme::Theme) -> String {
    let caption = if alt.trim().is_empty() {
        String::new()
    } else {
        format!(
            "<p style=\"margin:0;padding:9px 12px 0;color:{};font-size:12px;line-height:1.65;text-align:center;letter-spacing:0.04em;\">{}</p>",
            theme.text_muted,
            inline_md(alt.trim(), theme)
        )
    };

    format!(
        "<section style=\"margin: 1.8em 0 2em; padding: 10px; background: {}; border: 1px solid {}; border-radius: 8px; text-align:center;\">\n<img src=\"{url}\" alt=\"{alt}\" style=\"max-width: 100%; display: block; margin: 0 auto; border-radius: 5px;\" />\n{caption}</section>\n\n",
        theme.block_bg, theme.border
    )
}

fn render_blockquote(text: &str, theme: &theme::Theme) -> String {
    format!(
        "<section style=\"margin: 1.9em 0; padding: 18px 20px 18px 24px; background: {}; border: 1px solid {}; border-left: 4px solid {}; border-radius: 0 8px 8px 0; box-shadow: 0 2px 8px rgba(15,23,42,0.05); color: {}; font-size: 15px; line-height: 1.9; letter-spacing: 0.08em;\">{}</section>\n\n",
        theme.block_bg,
        theme.border,
        theme.accent,
        theme.text_color,
        inline_md(text, theme)
    )
}
