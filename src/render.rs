use std::fs;
use std::path::{Path, PathBuf};

use crate::article::{
    first_non_empty_line, parse_frontmatter, strip_frontmatter, strip_wechat_footer, wechat_title,
};
use crate::config::Config;
use crate::error::AppError;
use crate::footer;
use crate::json_util::escape_json;
use crate::status::add_status;
use crate::theme;
use crate::wechat::WechatClient;

pub fn render_article(
    vault: &Path,
    article: &Path,
    author: &str,
    thumb_media_id: &str,
    theme_name: &str,
    cover_html: Option<&str>,
    qrcode_path: &str,
) -> Result<String, AppError> {
    let article = crate::article::resolve_article_path(vault, article);
    if article.extension().and_then(|e| e.to_str()) != Some("md") {
        return Err(AppError::InvalidArticlePath(article));
    }

    let md = fs::read_to_string(&article).map_err(|source| AppError::Io {
        path: article.clone(),
        source,
    })?;

    let front = parse_frontmatter(&md);
    let body = strip_frontmatter(&md);
    let body = strip_wechat_footer(body);

    // Per-article overrides take priority over caller-supplied values.
    let effective_author = front.wechat_author.as_deref().unwrap_or(author);
    let effective_theme = front.theme.as_deref().unwrap_or(theme_name);

    let t = theme::Theme::from_name(effective_theme);
    let html_body = md_to_wechat_html(body, &t);
    let body_with_cover = match cover_html {
        Some(cover) => format!("{cover}\n{html_body}"),
        None => html_body,
    };

    // Resolve qrcode path relative to vault root so upload_local_images
    // (which resolves relative to article_dir) gets an absolute path.
    let abs_qrcode: String;
    let resolved_qrcode = if qrcode_path.is_empty()
        || qrcode_path.starts_with("http://")
        || qrcode_path.starts_with("https://")
        || qrcode_path.starts_with('/')
    {
        qrcode_path
    } else {
        abs_qrcode = vault.join(qrcode_path).to_string_lossy().into_owned();
        &abs_qrcode
    };

    let footer_cfg = footer::FooterConfig::from_config(effective_author, resolved_qrcode);
    let full_html = wrap_wechat_html(&body_with_cover, &t, &footer_cfg);

    let title = wechat_title(&front);
    let digest = front
        .digest
        .clone()
        .unwrap_or_else(|| first_non_empty_line(body).to_owned());

    let slug = article
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?;
    let dir = article
        .parent()
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?;

    let html_path = dir.join(format!("{slug}.html"));
    let json_path = dir.join(format!("{slug}.draft.json"));

    fs::write(&html_path, &full_html).map_err(|source| AppError::Io {
        path: html_path.clone(),
        source,
    })?;

    let draft_json = build_draft_json(
        &title,
        effective_author,
        &digest,
        &full_html,
        thumb_media_id,
    );
    fs::write(&json_path, &draft_json).map_err(|source| AppError::Io {
        path: json_path.clone(),
        source,
    })?;

    let _ = add_status(vault, slug, "rendered", "");

    Ok(format!(
        "rendered\n  html:  {}\n  draft: {}",
        html_path.display(),
        json_path.display()
    ))
}

// ── Markdown → WeChat HTML ────────────────────────────────────────────────────

fn md_to_wechat_html(md: &str, theme: &theme::Theme) -> String {
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

#[derive(Debug)]
enum MdBlock<'a> {
    /// A `:::name` fenced block with optional YAML-like properties and body content.
    Fence(&'a str, Vec<(&'a str, &'a str)>, &'a str),
    /// Plain markdown text to be rendered as usual.
    Markdown(&'a str),
}

/// Split markdown into block segments. `:::name` fences and plain markdown.
fn parse_blocks(md: &str) -> Vec<MdBlock<'_>> {
    let mut blocks = Vec::new();
    let mut rest = md;

    while !rest.is_empty() {
        // Check if current position starts with `:::` at line start
        let is_line_start = rest.as_ptr() == md.as_ptr()
            || rest.as_bytes()[0] == b'\n'
            || (rest.len() > 1 && rest.as_bytes()[0] == b'\r' && rest.as_bytes()[1] == b'\n');
        let starts_fence =
            rest.starts_with(":::") || rest.starts_with("\n:::") || rest.starts_with("\r\n:::");

        if starts_fence {
            // Skip leading whitespace/newline to get to :::
            let _fence_start = if rest.starts_with("\r\n:::") {
                rest = &rest[2..];
                rest
            } else if rest.starts_with("\n:::") {
                rest = &rest[1..];
                rest
            } else {
                rest
            };

            // rest now starts with ":::"
            // Read block name
            let after_fence = &rest[3..]; // skip :::
            let name_end = after_fence.find('\n').unwrap_or(after_fence.len());
            let name_line = after_fence[..name_end].trim();
            let name = name_line.split_whitespace().next().unwrap_or("");

            // Find closing `:::`
            let inner_start = name_end + 1; // skip past \n
            let after_name = &after_fence[inner_start..];

            // Search for `\n:::` as closing marker
            let close_offset = after_name.find("\n:::");
            let (inner, remaining) = if let Some(off) = close_offset {
                let inner_text = &after_name[..off];
                // skip past \n:::\n
                let after_close = &after_name[off + 4..]; // skip \n:::
                let after_newline = after_close
                    .find('\n')
                    .map(|n| n + 1)
                    .unwrap_or(after_close.len());
                (inner_text, &after_close[after_newline..])
            } else {
                // No closing found, treat remaining as block body (maybe end of file)
                (after_name, "")
            };

            if !name.is_empty() {
                let (props, body) = split_fence_props(inner);
                blocks.push(MdBlock::Fence(name, props, body));
            }
            rest = remaining;
            continue;
        }

        // Regular markdown — find next `:::` fence or EOF
        if is_line_start {
            // Already at line start, just accumulate
        }
        let next_fence = rest.find("\n:::");
        if let Some(pos) = next_fence {
            let segment = &rest[..pos + 1]; // include the \n before :::
            let trimmed = segment.trim();
            if !trimmed.is_empty() {
                blocks.push(MdBlock::Markdown(trimmed));
            }
            rest = &rest[pos + 1..]; // point to ::: for next iteration
        } else {
            // No more fences, everything is markdown
            let trimmed = rest.trim();
            if !trimmed.is_empty() {
                blocks.push(MdBlock::Markdown(trimmed));
            }
            break;
        }
    }

    blocks
}

/// Parse key: value lines at the start of a fence body; rest is body content.
fn split_fence_props(inner: &str) -> (Vec<(&str, &str)>, &str) {
    let mut props = Vec::new();
    let mut body_start = 0;
    for line in inner.lines() {
        let trimmed = line.trim();
        if let Some((k, v)) = trimmed.split_once(':') {
            let k = k.trim();
            let v = v.trim().trim_matches('"');
            if !k.is_empty() && !k.contains(' ') && k.len() < 30 {
                props.push((k, v));
                body_start += line.len() + 1;
                continue;
            }
        }
        if trimmed.is_empty() {
            body_start += line.len() + 1;
            continue;
        }
        break;
    }
    let body = if body_start < inner.len() {
        &inner[body_start..]
    } else {
        ""
    };
    (props, body.trim_start())
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
            crate::illustrate::render_illustration(
                &crate::illustrate::IllustType::QuoteCard {
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
            crate::illustrate::render_illustration(
                &crate::illustrate::IllustType::Divider {
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
                .join(
                    "
",
                )
                .trim()
                .to_owned();
            crate::illustrate::render_illustration(
                &crate::illustrate::IllustType::ConceptCard {
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
            crate::illustrate::render_illustration(
                &crate::illustrate::IllustType::EmotionCard {
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
            crate::illustrate::render_code_block(lang, body.trim(), theme)
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
                crate::illustrate::render_timeline(&items, theme)
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
                crate::illustrate::render_comparison(left, right, &rows, theme)
            }
        }
        "tip" => {
            let icon = props
                .iter()
                .find(|(k, _)| *k == "icon")
                .map(|(_, v)| *v)
                .unwrap_or("");
            crate::illustrate::render_tip(icon, body.trim(), theme)
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

fn render_markdown_segment(md: &str, theme: &theme::Theme) -> String {
    let mut out = String::new();
    let mut in_blockquote = false;
    let mut is_callout = false;
    let mut blockquote_buf = String::new();

    for line in md.lines() {
        if let Some(rest) = line
            .strip_prefix("> ")
            .or_else(|| if line == ">" { Some("") } else { None })
        {
            if !in_blockquote {
                // First line of a blockquote: detect Obsidian callout [!type].
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

        if line.trim() == "---" || line.trim() == "***" || line.trim() == "___" {
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

        if line.trim().is_empty() {
            continue;
        }

        let trimmed = line.trim();
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

    if in_blockquote && !is_callout {
        out.push_str(&render_blockquote(&blockquote_buf, theme));
    }

    out
}

pub fn inline_md(text: &str, theme: &theme::Theme) -> String {
    // **bold**, *italic*, `code` — applied in order
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
        // image ![alt](url)
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
        s.push(chars[i]);
        i += 1;
    }
    s
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
        "<p style=\"margin: 0 0 1.2em; color: {}; font-size: 15px; line-height: 1.85; letter-spacing: 0.1em; word-spacing: 0.05em; text-align: justify;\">{}</p>\n\n",
        theme.text_color,
        inline_md(text, theme)
    )
}

fn render_blockquote(text: &str, theme: &theme::Theme) -> String {
    // Use <section> instead of <blockquote>: new WeChat editor strips <blockquote> inline styles (doocs/md issue #447)
    format!(
        "<section style=\"margin: 1.8em 0; padding: 16px 20px 16px 24px; background: {}; border-left: 4px solid {}; border-radius: 0 6px 6px 0; color: {}; font-size: 15px; line-height: 1.85; letter-spacing: 0.1em;\">{}</section>\n\n",
        theme.block_bg,
        theme.accent,
        theme.text_muted,
        inline_md(text, theme)
    )
}

fn wrap_wechat_html(body: &str, theme: &theme::Theme, footer_cfg: &footer::FooterConfig) -> String {
    let ending = footer::render_footer(footer_cfg, theme);
    format!(
        "<section style=\"{}\">\n\n{body}\n\n{ending}\n\n</section>\n",
        theme.section_style()
    )
}

/// If frontmatter has a `cover` field pointing to a local image, upload it to
/// WeChat permanent material and return the media_id. Otherwise return None.
pub fn resolve_cover_thumb(
    front: &crate::article::Frontmatter,
    _cfg: &Config,
    dir: &Path,
    client: &WechatClient,
    token: &str,
) -> Result<Option<String>, AppError> {
    let cover_path = match &front.cover {
        Some(c) => c,
        None => return Ok(None),
    };
    // Skip if it's already a URL or media_id (starts with http or contains uppercase alphanumeric)
    if cover_path.starts_with("http://") || cover_path.starts_with("https://") {
        return Ok(None);
    }
    let full_path = if cover_path.starts_with('/') {
        PathBuf::from(cover_path)
    } else {
        dir.join(cover_path)
    };
    if !full_path.exists() {
        return Ok(None);
    }
    let media_id = client.upload_image(token, &full_path)?;
    Ok(Some(media_id))
}

pub fn build_draft_json(
    title: &str,
    author: &str,
    digest: &str,
    content: &str,
    thumb_media_id: &str,
) -> String {
    // WeChat digest limit is 120 chars; truncate at a char boundary.
    let digest = {
        let mut end = 120usize.min(digest.len());
        while !digest.is_char_boundary(end) {
            end -= 1;
        }
        &digest[..end]
    };
    // Hand-build JSON to keep zero deps.
    // WeChat draft/add API rejects empty thumb_media_id (error 40007).
    let thumb_field = if thumb_media_id.is_empty() {
        String::new()
    } else {
        format!(
            ",\n      \"thumb_media_id\": \"{}\"",
            escape_json(thumb_media_id)
        )
    };
    format!(
        "{{\n  \"articles\": [\n    {{\n      \"title\": \"{}\",\n      \"author\": \"{}\",\n      \"digest\": \"{}\",\n      \"content\": \"{}\"{thumb_field}\n    }}\n  ]\n}}\n",
        escape_json(title),
        escape_json(author),
        escape_json(digest),
        escape_json(content),
    )
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::app::run;
    use crate::article::parse_frontmatter;
    use crate::cli::Options;
    use crate::config::Config;
    use crate::render::{build_draft_json, render_article, resolve_cover_thumb};
    use crate::test_helpers::{create_file, temp_root};
    use crate::wechat::WechatClient;

    #[test]
    fn render_produces_html_and_draft_json() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("render-basic")?;
        let md_path = root.join("demo.md");
        create_file(
            &md_path,
            "---\ntitle: 测试文章标题\ndigest: 这是摘要\n---\n\n正文第一段。\n",
        )?;

        render_article(&root, &md_path, "寻月隐君", "thumb123", "default", None, "")?;

        let html = fs::read_to_string(root.join("demo.html"))?;
        assert!(html.contains("<section"), "缺少 section 容器");
        assert!(html.contains("正文第一段"), "正文未渲染");

        let json_str = fs::read_to_string(root.join("demo.draft.json"))?;
        assert!(json_str.contains("\"title\": \"测试文章标题\""));
        assert!(json_str.contains("\"author\": \"寻月隐君\""));
        assert!(json_str.contains("\"digest\": \"这是摘要\""));
        assert!(json_str.contains("\"thumb_media_id\": \"thumb123\""));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn render_digest_falls_back_to_first_paragraph() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("render-digest")?;
        let md_path = root.join("article.md");
        create_file(
            &md_path,
            "---\ntitle: 标题\n---\n\n## 一级标题\n\n第一段文字内容。\n",
        )?;

        render_article(&root, &md_path, "作者", "", "default", None, "")?;

        let json_str = fs::read_to_string(root.join("article.draft.json"))?;
        assert!(json_str.contains("第一段文字内容"), "摘要应取自第一段正文");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn render_markdown_elements() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("render-elements")?;
        let md_path = root.join("elem.md");
        create_file(
            &md_path,
            "---\ntitle: T\n---\n\n## 章节标题\n\n**粗体** 和 *斜体* 和 `代码`。\n\n> 引用文字\n\n---\n",
        )?;

        render_article(&root, &md_path, "a", "", "default", None, "")?;

        let html = fs::read_to_string(root.join("elem.html"))?;
        assert!(html.contains("<h2 "), "h2 未渲染");
        assert!(html.contains("<strong "), "strong 未渲染");
        assert!(html.contains("<em>"), "em 未渲染");
        assert!(html.contains("<code "), "code 未渲染");
        assert!(html.contains("border-left: 4px solid"), "blockquote 未渲染");
        assert!(html.contains("<hr "), "hr 未渲染");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn render_uses_config_author_and_thumb() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("render-config")?;
        let cfg_path = root.join("moonpub.toml");
        create_file(
            &cfg_path,
            "[wechat]\nauthor = \"从配置读\"\nthumb_media_id = \"cfg_thumb\"\n",
        )?;
        let md_path = root.join("article.md");
        create_file(&md_path, "---\ntitle: T\n---\n\n正文。\n")?;

        let options = Options::parse([
            "--config".to_owned(),
            cfg_path.to_str().unwrap().to_owned(),
            "--vault".to_owned(),
            root.to_str().unwrap().to_owned(),
            "render".to_owned(),
            md_path.to_str().unwrap().to_owned(),
        ])?;
        run(&options)?;

        let json_str = fs::read_to_string(root.join("article.draft.json"))?;
        assert!(json_str.contains("\"author\": \"从配置读\""));
        assert!(json_str.contains("\"thumb_media_id\": \"cfg_thumb\""));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn render_cli_flag_overrides_config() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("render-override")?;
        let cfg_path = root.join("moonpub.toml");
        create_file(
            &cfg_path,
            "[wechat]\nauthor = \"配置作者\"\nthumb_media_id = \"cfg_thumb\"\n",
        )?;
        let md_path = root.join("article.md");
        create_file(&md_path, "---\ntitle: T\n---\n\n正文。\n")?;

        let options = Options::parse([
            "--config".to_owned(),
            cfg_path.to_str().unwrap().to_owned(),
            "--vault".to_owned(),
            root.to_str().unwrap().to_owned(),
            "render".to_owned(),
            md_path.to_str().unwrap().to_owned(),
            "--author".to_owned(),
            "命令行作者".to_owned(),
            "--thumb".to_owned(),
            "cli_thumb".to_owned(),
        ])?;
        run(&options)?;

        let json_str = fs::read_to_string(root.join("article.draft.json"))?;
        assert!(json_str.contains("\"author\": \"命令行作者\""));
        assert!(json_str.contains("\"thumb_media_id\": \"cli_thumb\""));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn build_draft_json_omits_thumb_when_empty() {
        let json = build_draft_json("标题", "作者", "摘要", "<p>内容</p>", "");
        assert!(
            !json.contains("thumb_media_id"),
            "空 thumb 不应出现在 JSON 里"
        );
        assert!(json.contains("\"title\": \"标题\""));
        assert!(json.contains("\"author\": \"作者\""));
    }

    #[test]
    fn build_draft_json_includes_thumb_when_set() {
        let json = build_draft_json("标题", "作者", "摘要", "<p>内容</p>", "media_abc123");
        assert!(json.contains("\"thumb_media_id\": \"media_abc123\""));
    }

    #[test]
    fn frontmatter_cover_field_is_parsed() {
        let md = "---\ntitle: 测试\ncover: ./my-cover.jpg\n---\n\n正文。\n";
        let front = parse_frontmatter(md);
        assert_eq!(front.cover.as_deref(), Some("./my-cover.jpg"));
    }

    #[test]
    fn resolve_cover_thumb_skips_nonexistent_file() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("cover-nonexistent")?;
        let dir = root.join("Articles/drafts");
        fs::create_dir_all(&dir)?;
        let front = parse_frontmatter("---\ntitle: T\ncover: missing.jpg\n---\n\n正文\n");
        let client = WechatClient::new("fake_appid", "fake_secret");
        let result = resolve_cover_thumb(&front, &Config::default(), &dir, &client, "fake_token");
        assert!(
            result.is_ok(),
            "nonexistent cover should return Ok(None), not error"
        );
        assert!(result.unwrap().is_none(), "nonexistent cover file → None");
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn resolve_cover_thumb_skips_http_url() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("cover-http")?;
        let dir = root.join("Articles/drafts");
        fs::create_dir_all(&dir)?;
        let front =
            parse_frontmatter("---\ntitle: T\ncover: https://example.com/img.jpg\n---\n\n正文\n");
        let client = WechatClient::new("fake_appid", "fake_secret");
        let result = resolve_cover_thumb(&front, &Config::default(), &dir, &client, "fake_token");
        assert!(result.is_ok());
        assert!(
            result.unwrap().is_none(),
            "http URL cover → None (skip upload)"
        );
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn resolve_cover_thumb_returns_none_when_no_cover_field()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("cover-none")?;
        let dir = root.join("Articles/drafts");
        fs::create_dir_all(&dir)?;
        let front = parse_frontmatter("---\ntitle: T\n---\n\n正文\n");
        let client = WechatClient::new("fake_appid", "fake_secret");
        let result = resolve_cover_thumb(&front, &Config::default(), &dir, &client, "fake_token");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none(), "no cover field → None");
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
