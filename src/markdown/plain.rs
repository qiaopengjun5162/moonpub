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
                    theme,
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
            theme,
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

    // 微信不支持 <table>，也不能靠 media query 收窄。早先把每行包成一张卡片、
    // 每个单元格上方重复一遍表头，5 行的表会变成 5 张大卡、表头出现 5 次，
    // 手机上完全读不下去。改成「标签 / 值」两列行式：标签是窄的 accent 小字，
    // 值占满剩余宽度，行间用细分隔线断开，接近定义列表的阅读节奏。
    let two_col = headers.len() == 2;
    // 深色主题下正文背景和行分隔线只差一点点亮度，纯线条撑不起层次；
    // 给整张表加一层比正文略亮的卡片底，行间再用更深的线断开。
    let mut html = format!(
        "<section style=\"margin:26px 0;padding:4px 18px;background:{};border:1px solid {};border-radius:12px;\">\n",
        theme.block_bg, theme.border
    );
    for row in rows {
        // 两列表格（项 / 内容）：左列做标签行，右列做正文，一行一件事。
        if two_col {
            let label = row.first().copied().unwrap_or("");
            let value = row.get(1).copied().unwrap_or("");
            if label.is_empty() && value.is_empty() {
                continue;
            }
            html.push_str(&format!(
                "<section style=\"padding:16px 0 15px;border-bottom:1px solid {};\">\n<p style=\"margin:0 0 7px;font-size:12px;font-weight:700;letter-spacing:0.14em;color:{};line-height:1.4;\">{}</p>\n<p style=\"margin:0;font-size:15px;line-height:1.85;color:{};letter-spacing:0.02em;\">{}</p>\n</section>\n",
                theme.border,
                theme.accent,
                inline_md(label, theme),
                theme.text_color,
                inline_md(value, theme)
            ));
            continue;
        }
        // 多列表格：首列当行标签，其余列并列成短句，避免横向挤压。
        let label = row.first().copied().unwrap_or("");
        html.push_str(&format!(
            "<section style=\"padding:13px 2px;border-bottom:1px solid {};\">\n",
            theme.border
        ));
        if !label.is_empty() {
            html.push_str(&format!(
                "<p style=\"margin:0 0 5px;font-size:12px;font-weight:700;letter-spacing:0.12em;color:{};line-height:1.4;\">{}</p>\n",
                theme.accent,
                inline_md(label, theme)
            ));
        }
        for (idx, header) in headers.iter().enumerate().skip(1) {
            let cell = row.get(idx).copied().unwrap_or("");
            if cell.is_empty() {
                continue;
            }
            html.push_str(&format!(
                "<p style=\"margin:0 0 4px;font-size:15px;line-height:1.8;color:{};letter-spacing:0.02em;\"><span style=\"color:{};font-size:13px;\">{}：</span>{}</p>\n",
                theme.text_color,
                theme.text_muted,
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
                "<span style=\"display:inline-block;width:7px;height:7px;background:{};border-radius:50%;vertical-align:middle;margin-right:10px;\"></span>",
                theme.accent
            ),
            ListMarker::Ordered(n) => format!(
                "<span style=\"display:inline-block;min-width:22px;height:22px;padding:0 4px;background:{};color:#fff;border-radius:999px;text-align:center;line-height:22px;font-size:12px;font-weight:bold;margin-right:8px;\">{n}</span>",
                theme.accent
            ),
        };
        let bg = if idx % 2 == 0 {
            theme.accent_soft
        } else {
            theme.section_bg
        };
        // 2026-09-09 修复：圆点/编号与文本同格内嵌，不再用独立 34px 首列。
        // 微信端纯样式圆点 span 不渲染，独立首列在手机上看是「空一列」。
        html.push_str(&format!(
            "<tr><td style=\"padding:8px 12px 8px 8px;vertical-align:top;background:{bg};color:{};font-size:15px;line-height:1.8;\">{marker}{}</td></tr>\n",
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
    // 中文小标题原先同时挂了左侧粗条和下边框，两个方向抢视线；字距 1px 把
    // 三四个字的标题拉得很散。收成一条左侧色条 + 正常字距，靠上下留白分层。
    format!(
        "<h2 style=\"font-size: 21px; font-weight: 700; color: {}; margin: 2.4em 0 1em; padding: 2px 0 2px 13px; border-left: 4px solid {}; letter-spacing: 0.02em; line-height:1.45;\">{}</h2>\n\n",
        theme.heading_color,
        theme.heading_border,
        inline_md(text, theme)
    )
}

fn render_h3(text: &str, theme: &theme::Theme) -> String {
    format!(
        "<h3 style=\"font-size: 17px; font-weight: 700; color: {}; margin: 1.9em 0 0.7em; padding: 7px 12px; border-left: 3px solid {}; background: {}; border-radius: 0 6px 6px 0; letter-spacing: 0.02em; line-height:1.5;\">{}</h3>\n\n",
        theme.heading_color,
        theme.accent,
        theme.accent_soft,
        inline_md(text, theme)
    )
}

fn render_h4(text: &str, theme: &theme::Theme) -> String {
    format!(
        "<h4 style=\"font-size: 16px; font-weight: 700; color: {}; margin: 1.5em 0 0.7em; padding: 0 0 0 10px; border-left: 3px solid {}; letter-spacing: 0.02em; line-height:1.5;\">{}</h4>\n\n",
        theme.heading_color,
        theme.accent,
        inline_md(text, theme)
    )
}

fn render_lead_p(text: &str, theme: &theme::Theme) -> String {
    format!(
        "<p style=\"margin: 0 0 1.5em; color: {}; font-size: 16px; line-height: 1.85; letter-spacing: 0.02em; text-align: left;\">{}</p>\n\n",
        theme.text_color,
        inline_md(text, theme)
    )
}

fn render_p(text: &str, theme: &theme::Theme) -> String {
    // 首行缩进 + 1.25em 段距 = 双重分隔，读起来发散；两端对齐会让中英混排的
    // 行内空隙忽大忽小。统一成「不缩进 + 1.5em 段距 + 左对齐」，这是中文长文
    // 在手机上最稳的一组参数。
    format!(
        "<p style=\"margin: 0 0 1.5em; color: {}; font-size: 16px; line-height: 1.85; letter-spacing: 0.02em; text-align: left;\">{}</p>\n\n",
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
    // 引用块里的软换行（一行一个 `>`）此前被原样塞进 HTML，而 HTML 不认 \n，
    // 三行信息会挤成一整段糊在一起。这里逐行渲染再用 <br> 断开。
    let body = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| inline_md(l.trim(), theme))
        .collect::<Vec<_>>()
        .join("<br>");
    // box-shadow 微信端不渲染，去掉；字距从 0.08em 收到 0.02em 跟正文一致。
    format!(
        "<section style=\"margin: 2em 0; padding: 18px 20px; background: {}; border: 1px solid {}; border-left: 4px solid {}; border-radius: 0 8px 8px 0; color: {}; font-size: 15px; line-height: 1.9; letter-spacing: 0.02em;\">{}</section>\n\n",
        theme.block_bg, theme.border, theme.accent, theme.text_color, body
    )
}

#[cfg(test)]
mod tests {
    use super::render_markdown_segment;
    use crate::theme;

    fn theme() -> theme::Theme {
        theme::Theme::default()
    }

    #[test]
    fn renders_empty_without_panic() {
        let out = render_markdown_segment("", &theme());
        assert!(out.is_empty() || out.chars().all(|c| c.is_whitespace()));
    }

    #[test]
    fn renders_heading_text() {
        let out = render_markdown_segment("## 社区营造", &theme());
        assert!(out.contains("社区营造"));
    }

    #[test]
    fn renders_blockquote_text() {
        let out = render_markdown_segment("> 这是引用", &theme());
        assert!(out.contains("这是引用"));
    }

    #[test]
    fn renders_list_items() {
        let out = render_markdown_segment("- 第一项\n- 第二项", &theme());
        assert!(out.contains("第一项"));
        assert!(out.contains("第二项"));
    }

    #[test]
    fn renders_paragraph_text() {
        let out = render_markdown_segment("普通正文一段。", &theme());
        assert!(out.contains("普通正文一段。"));
    }
}
