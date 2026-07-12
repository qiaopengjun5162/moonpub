use crate::illustrate;
use crate::theme;

use super::inline::inline_md;

pub(super) fn render_fence_block(
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
        "key-points" => render_key_points(body, theme),
        "pull-quote" => render_pull_quote(props, body, theme),
        "cover" => render_cover(props, theme),
        "letter-card" => render_letter_card(props, body, theme),
        "scene-card" => render_scene_card(props, body, theme),
        "closing-card" => render_closing_card(props, body, theme),
        "compact-links" => render_compact_links(body, theme),
        "photo-grid" => render_photo_grid(body, theme),
        "meta-strip" => render_meta_strip(props, body, theme),
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

fn render_compact_links(body: &str, theme: &theme::Theme) -> String {
    let rows = body
        .lines()
        .filter(|line| line.trim().starts_with("- "))
        .map(|line| line.trim().trim_start_matches("- ").trim())
        .filter_map(|line| {
            let mut parts = line.splitn(4, " | ").map(str::trim);
            let number = parts.next()?;
            let title = parts.next()?;
            let source = parts.next()?;
            let url = parts.next()?;
            Some((number, title, source, url))
        })
        .collect::<Vec<_>>();

    if rows.is_empty() {
        return render_generic_fence("compact-links", body, theme);
    }

    let mut html = format!(
        "<section style=\"margin:14px 0 20px;padding:8px 10px;background:{};border:1px solid {};border-radius:6px;\">\n",
        theme.section_bg, theme.border
    );
    for (number, title, source, url) in rows {
        let escaped_url = html_escape(url);
        html.push_str(&format!(
            "<p style=\"margin:0 0 5px;color:{};font-size:12px;line-height:1.55;letter-spacing:0.01em;word-break:break-all;\"><span style=\"color:{};font-weight:bold;\">{}</span> <span style=\"color:{};font-weight:600;\">{}</span> <span style=\"color:{};\">｜{}｜原文：<a href=\"{}\" style=\"color:#576b95;text-decoration:none;\">{}</a></span></p>\n",
            theme.text_muted,
            theme.accent,
            inline_md(number, theme),
            theme.text_color,
            inline_md(title, theme),
            theme.text_muted,
            inline_md(source, theme),
            escaped_url,
            escaped_url
        ));
    }
    html.push_str("</section>\n\n");
    html
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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
    if theme.name == "forest" {
        return format!(
            "<section style=\"margin: 24px 0 28px; padding: 22px 24px; background: {}; border: 1px solid {}; border-left: 5px solid {}; border-radius: 0 16px 16px 0; box-shadow: 0 8px 20px rgba(31,107,67,0.08); font-size: 16px; color: {}; line-height: 2.05; letter-spacing: 0.07em;\">\n{}\n</section>\n\n",
            theme.block_bg,
            theme.border,
            theme.accent,
            theme.heading_color,
            inline_md(body.trim(), theme)
        );
    }

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
    let items: Vec<(bool, &str)> = body
        .lines()
        .filter_map(|line| parse_checklist_item(line.trim()))
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
                let (checked, content) = items[idx];
                let rest = if checked {
                    format!(
                        "<span style=\"color:{};font-weight:bold;\">✔</span>&nbsp;&nbsp;{content}",
                        theme.accent
                    )
                } else {
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

fn render_key_points(body: &str, theme: &theme::Theme) -> String {
    let items: Vec<&str> = body
        .lines()
        .filter_map(|line| line.trim().strip_prefix("- ").map(str::trim))
        .filter(|item| !item.is_empty())
        .collect();
    if items.is_empty() {
        return render_generic_fence("key-points", body, theme);
    }

    let mut html = format!(
        "<section style=\"margin:24px 0;padding:16px 18px;background:{};border:1px solid {};border-radius:10px;\">\n",
        theme.block_bg, theme.border
    );
    html.push_str(&format!(
        "<p style=\"margin:0 0 12px;color:{};font-size:13px;font-weight:bold;letter-spacing:0.18em;\">KEY POINTS</p>\n",
        theme.accent
    ));
    html.push_str("<table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\">\n");
    for (idx, item) in items.iter().enumerate() {
        html.push_str(&format!(
            "<tr><td style=\"width:34px;padding:8px 0;vertical-align:top;\"><span style=\"display:inline-block;width:24px;height:24px;line-height:24px;text-align:center;border-radius:8px;background:{};color:#fff;font-size:12px;font-weight:bold;\">{}</span></td><td style=\"padding:7px 0 7px 10px;color:{};font-size:15px;line-height:1.85;vertical-align:top;\">{}</td></tr>\n",
            theme.accent,
            idx + 1,
            theme.text_color,
            inline_md(item, theme)
        ));
    }
    html.push_str("</table></section>\n\n");
    html
}

fn render_pull_quote(props: &[(&str, &str)], body: &str, theme: &theme::Theme) -> String {
    let text = body.trim();
    if text.is_empty() {
        return render_generic_fence("pull-quote", body, theme);
    }
    let source = props
        .iter()
        .find(|(key, _)| *key == "source")
        .map(|(_, value)| *value)
        .unwrap_or("");
    let source_html = if source.is_empty() {
        String::new()
    } else {
        format!(
            "<p style=\"margin:12px 0 0;color:{};font-size:13px;line-height:1.7;text-align:right;\">— {}</p>\n",
            theme.text_muted,
            inline_md(source, theme)
        )
    };

    if theme.name == "forest" {
        return format!(
            "<section style=\"margin:30px 0;padding:24px 24px;background:{};border:1px solid {};border-top:4px solid {};border-radius:16px;text-align:center;box-shadow:0 8px 20px rgba(31,107,67,0.06);\">\n<p style=\"margin:0;color:{};font-size:18px;font-weight:bold;line-height:1.9;letter-spacing:0.1em;\">{}</p>\n{source_html}</section>\n\n",
            theme.accent_soft,
            theme.border,
            theme.accent,
            theme.heading_color,
            inline_md(text, theme)
        );
    }

    format!(
        "<section style=\"margin:28px 0;padding:22px 24px;background:{};border-top:3px solid {};border-bottom:1px solid {};text-align:center;\">\n<p style=\"margin:0;color:{};font-size:18px;font-weight:bold;line-height:1.85;letter-spacing:0.08em;\">{}</p>\n{source_html}</section>\n\n",
        theme.accent_soft,
        theme.accent,
        theme.border,
        theme.heading_color,
        inline_md(text, theme)
    )
}

fn parse_checklist_item(line: &str) -> Option<(bool, &str)> {
    let rest = line.strip_prefix("- [")?;
    let (marker, content) = rest.split_once("] ")?;
    let marker = marker.trim();
    if marker.is_empty() {
        return Some((false, content.trim()));
    }
    if marker.eq_ignore_ascii_case("x") {
        return Some((true, content.trim()));
    }
    None
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

fn render_letter_card(props: &[(&str, &str)], body: &str, theme: &theme::Theme) -> String {
    let title = props
        .iter()
        .find(|(k, _)| *k == "title")
        .map(|(_, v)| *v)
        .unwrap_or("一封短笺");
    let date = props
        .iter()
        .find(|(k, _)| *k == "date")
        .map(|(_, v)| *v)
        .unwrap_or("");
    let date_line = if date.is_empty() {
        String::new()
    } else {
        format!(
            "<p style=\"margin:0 0 12px;font-size:12px;color:{};letter-spacing:0.14em;text-align:right;\">{}</p>\n",
            theme.text_muted,
            inline_md(date, theme)
        )
    };
    format!(
        "<section style=\"margin:28px 0;padding:24px 24px 22px;background:{};border:1px solid {};border-radius:18px;box-shadow:0 10px 28px rgba(74,55,38,0.08);\">\n<p style=\"margin:0 0 10px;font-size:13px;color:{};font-weight:bold;letter-spacing:0.18em;\">{}</p>\n{date_line}<section style=\"font-size:15px;line-height:2.05;color:{};letter-spacing:0.06em;\">{}</section>\n</section>\n\n",
        theme.block_bg,
        theme.border,
        theme.accent,
        inline_md(title, theme),
        theme.text_color,
        inline_md(body.trim(), theme)
    )
}

fn render_scene_card(props: &[(&str, &str)], body: &str, theme: &theme::Theme) -> String {
    let label = props
        .iter()
        .find(|(k, _)| *k == "label")
        .map(|(_, v)| *v)
        .unwrap_or("此刻");
    let place = props
        .iter()
        .find(|(k, _)| *k == "place")
        .map(|(_, v)| *v)
        .unwrap_or("");
    let place_line = if place.is_empty() {
        String::new()
    } else {
        format!(
            "<span style=\"display:inline-block;margin-left:8px;color:{};font-weight:normal;\">{}</span>",
            theme.text_muted,
            inline_md(place, theme)
        )
    };
    format!(
        "<section style=\"margin:26px 0;padding:0;background:{};border:1px solid {};border-radius:16px;overflow:hidden;\">\n<section style=\"height:7px;background:linear-gradient(90deg,{},{});\"></section>\n<section style=\"padding:18px 20px 20px;\">\n<p style=\"margin:0 0 10px;font-size:13px;color:{};font-weight:bold;letter-spacing:0.16em;\">{}{}</p>\n<p style=\"margin:0;font-size:15px;line-height:1.95;color:{};\">{}</p>\n</section>\n</section>\n\n",
        theme.block_bg,
        theme.border,
        theme.accent,
        theme.accent_soft,
        theme.accent,
        inline_md(label, theme),
        place_line,
        theme.text_color,
        inline_md(body.trim(), theme)
    )
}

fn render_closing_card(props: &[(&str, &str)], body: &str, theme: &theme::Theme) -> String {
    let label = props
        .iter()
        .find(|(k, _)| *k == "label")
        .map(|(_, v)| *v)
        .unwrap_or("慢慢来");
    format!(
        "<section style=\"margin:32px 0 24px;padding:24px 22px;background:{};border-top:1px solid {};border-bottom:1px solid {};text-align:center;\">\n<p style=\"margin:0 0 10px;font-size:12px;color:{};font-weight:bold;letter-spacing:0.22em;\">{}</p>\n<p style=\"margin:0 auto;max-width:88%;font-size:15px;line-height:2;color:{};letter-spacing:0.07em;\">{}</p>\n</section>\n\n",
        theme.accent_soft,
        theme.border,
        theme.border,
        theme.accent,
        inline_md(label, theme),
        theme.heading_color,
        inline_md(body.trim(), theme)
    )
}

fn render_photo_grid(body: &str, theme: &theme::Theme) -> String {
    let photos: Vec<(&str, &str)> = body
        .lines()
        .filter_map(|line| {
            let item = line.trim().strip_prefix("- ")?.trim();
            if item.is_empty() {
                return None;
            }
            let (image, caption) = item.split_once(" | ").unwrap_or((item, ""));
            Some((image.trim(), caption.trim()))
        })
        .filter(|(image, _)| !image.is_empty())
        .collect();
    if photos.is_empty() {
        return render_generic_fence("photo-grid", body, theme);
    }

    let mut html = format!(
        "<section style=\"margin:28px 0;padding:14px;background:{};border:1px solid {};border-radius:16px;\">\n<table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\">\n",
        theme.block_bg, theme.border
    );
    for row in photos.chunks(2) {
        html.push_str("<tr>\n");
        for (image, caption) in row {
            let caption_html = if caption.is_empty() {
                String::new()
            } else {
                format!(
                    "<p style=\"margin:8px 0 0;color:{};font-size:12px;line-height:1.6;text-align:center;\">{}</p>",
                    theme.text_muted,
                    inline_md(caption, theme)
                )
            };
            html.push_str(&format!(
                "<td style=\"width:50%;padding:6px;vertical-align:top;\"><img src=\"{}\" style=\"display:block;width:100%;height:auto;border-radius:12px;\" />{caption_html}</td>\n",
                image
            ));
        }
        if row.len() == 1 {
            html.push_str("<td style=\"width:50%;padding:6px;\"></td>\n");
        }
        html.push_str("</tr>\n");
    }
    html.push_str("</table></section>\n\n");
    html
}

fn render_meta_strip(props: &[(&str, &str)], body: &str, theme: &theme::Theme) -> String {
    let items: Vec<(&str, &str)> = ["date", "place", "weather", "mood"]
        .iter()
        .filter_map(|key| {
            props
                .iter()
                .find(|(k, v)| k == key && !v.trim().is_empty())
                .map(|(_, v)| (*key, *v))
        })
        .collect();
    if items.is_empty() && body.trim().is_empty() {
        return String::new();
    }

    let mut html = format!(
        "<section style=\"margin:22px 0;padding:14px 16px;background:{};border-top:1px solid {};border-bottom:1px solid {};\">\n",
        theme.accent_soft, theme.border, theme.border
    );
    if !items.is_empty() {
        html.push_str("<table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\">\n<tr>\n");
        for (label, value) in items {
            html.push_str(&format!(
                "<td style=\"padding:4px 8px 4px 0;vertical-align:top;\"><p style=\"margin:0 0 3px;color:{};font-size:11px;font-weight:bold;letter-spacing:0.16em;text-transform:uppercase;\">{}</p><p style=\"margin:0;color:{};font-size:13px;line-height:1.6;\">{}</p></td>\n",
                theme.accent,
                label,
                theme.text_color,
                inline_md(value, theme)
            ));
        }
        html.push_str("</tr>\n</table>\n");
    }
    if !body.trim().is_empty() {
        html.push_str(&format!(
            "<p style=\"margin:10px 0 0;color:{};font-size:14px;line-height:1.85;letter-spacing:0.04em;\">{}</p>\n",
            theme.text_muted,
            inline_md(body.trim(), theme)
        ));
    }
    html.push_str("</section>\n\n");
    html
}

fn render_generic_fence(_name: &str, body: &str, theme: &theme::Theme) -> String {
    let body = body.trim();
    if body.is_empty() {
        return String::new();
    }

    format!(
        "<section style=\"margin: 18px 0; padding: 16px 20px; background: {}; border: 1px solid #e8e8e8; border-radius: 4px;\">\n{}\n</section>\n\n",
        theme.block_bg,
        inline_md(body, theme)
    )
}

#[cfg(test)]
mod tests {
    use super::render_fence_block;
    use crate::theme;

    fn default_theme() -> theme::Theme {
        theme::Theme::from_name("default")
    }

    #[test]
    fn book_info_renders_metadata_card() {
        let theme = default_theme();
        let props = [
            ("title", "Rust 之书"),
            ("author", "Ferris"),
            ("publisher", "MoonPub"),
            ("rating", "9.6"),
        ];

        let html = render_fence_block("book-info", &props, "", &theme);

        assert!(html.contains("《Rust 之书》"));
        assert!(html.contains("Ferris 著"));
        assert!(html.contains("MoonPub | 豆瓣 9.6"));
    }

    #[test]
    fn steps_renders_numbered_cards() {
        let theme = default_theme();
        let html = render_fence_block("steps", &[], "1. 选题\n2. 成稿\n3. 发布", &theme);

        assert!(html.contains(">1</section>"));
        assert!(html.contains(">2</section>"));
        assert!(html.contains(">3</section>"));
        assert!(html.contains("选题"));
        assert!(html.contains("发布"));
    }

    #[test]
    fn summary_renders_inline_markdown() {
        let theme = default_theme();
        let html = render_fence_block("summary", &[], "重点是 **节奏** 和 `细节`", &theme);

        assert!(html.contains("总 结"));
        assert!(html.contains("<strong"));
        assert!(html.contains("<code"));
    }

    #[test]
    fn figure_requires_image_and_renders_caption() {
        let theme = default_theme();
        let props = [
            ("image", "https://example.com/a.png"),
            ("caption", "架构图"),
        ];

        let html = render_fence_block("figure", &props, "", &theme);

        assert!(html.contains("https://example.com/a.png"));
        assert!(html.contains("架构图"));
        assert!(render_fence_block("figure", &[], "", &theme).is_empty());
    }

    #[test]
    fn checklist_renders_checked_and_unchecked_items_without_marker_leakage() {
        let theme = default_theme();
        let html = render_fence_block("checklist", &[], "- [x] 已完成\n- [ ] 待确认", &theme);

        assert!(html.contains("已完成"));
        assert!(html.contains("待确认"));
        assert!(html.contains("✔"));
        assert!(html.contains("○"));
        assert!(!html.contains("] 已完成"));
        assert!(!html.contains("] 待确认"));
    }

    #[test]
    fn key_points_renders_styled_points() {
        let theme = default_theme();
        let html = render_fence_block("key-points", &[], "- 先给结论\n- 再补证据", &theme);

        assert!(!html.contains("class="));
        assert!(html.contains("先给结论"));
        assert!(html.contains("再补证据"));
        assert!(html.contains(">1<"));
        assert!(html.contains(">2<"));
        assert!(html.contains(theme.accent));
    }

    #[test]
    fn pull_quote_renders_quote_and_source() {
        let theme = default_theme();
        let props = [("source", "《月亮与六便士》")];
        let html = render_fence_block(
            "pull-quote",
            &props,
            "满地都是六便士，他却抬头看见了月亮。",
            &theme,
        );

        assert!(!html.contains("class="));
        assert!(html.contains("满地都是六便士"));
        assert!(html.contains("《月亮与六便士》"));
        assert!(html.contains(theme.accent));
    }

    #[test]
    fn letter_card_renders_title_date_and_body() {
        let theme = default_theme();
        let props = [("title", "写给林子"), ("date", "2026-07-03")];
        let html = render_fence_block("letter-card", &props, "慢慢写，慢慢沉淀。", &theme);

        assert!(html.contains(theme.block_bg));
        assert!(html.contains("写给林子"));
        assert!(html.contains("2026-07-03"));
        assert!(html.contains("慢慢写"));
    }

    #[test]
    fn scene_card_renders_label_place_and_body() {
        let theme = default_theme();
        let props = [("label", "路上"), ("place", "月下林边")];
        let html = render_fence_block("scene-card", &props, "风从树影里慢慢穿过去。", &theme);

        assert!(html.contains("linear-gradient"));
        assert!(html.contains("路上"));
        assert!(html.contains("月下林边"));
        assert!(html.contains("树影"));
    }

    #[test]
    fn closing_card_renders_label_and_body() {
        let theme = default_theme();
        let props = [("label", "以后见")];
        let html = render_fence_block("closing-card", &props, "咱们就在这儿慢慢聊。", &theme);

        assert!(html.contains("text-align:center"));
        assert!(html.contains("以后见"));
        assert!(html.contains("慢慢聊"));
    }

    #[test]
    fn compact_links_renders_small_traceable_index() {
        let theme = default_theme();
        let html = render_fence_block(
            "compact-links",
            &[],
            "- 01 | OpenAI 发布 | OpenAI · 10 points | https://openai.com/news\n- 02 | ZK 文章 | Ethereum Research | https://ethresear.ch/t/zk\n- 03 | Query Link | Example | https://example.com/a?x=1&y=2",
            &theme,
        );

        assert!(!html.contains("class="));
        assert!(html.contains("font-size:12px"));
        assert!(html.contains("OpenAI 发布"));
        assert!(html.contains("原文：<a href=\"https://openai.com/news\""));
        assert!(html.contains(">https://openai.com/news</a>"));
        assert!(!html.contains("<a href=\"https://openai.com/news\" style=\"color:#576b95;text-decoration:none;\">OpenAI 发布</a>"));
        assert!(html.contains("https://ethresear.ch/t/zk"));
        assert!(html.contains("https://example.com/a?x=1&amp;y=2"));
    }

    #[test]
    fn photo_grid_renders_two_column_images_and_captions() {
        let theme = default_theme();
        let html = render_fence_block(
            "photo-grid",
            &[],
            "- /a.jpg | 树影\n- /b.jpg | 月光\n- /c.jpg",
            &theme,
        );

        assert!(html.contains("<table"));
        assert!(html.contains("/a.jpg"));
        assert!(html.contains("/b.jpg"));
        assert!(html.contains("/c.jpg"));
        assert!(html.contains("树影"));
        assert!(html.contains("月光"));
    }

    #[test]
    fn meta_strip_renders_props_and_optional_note() {
        let theme = default_theme();
        let props = [
            ("date", "2026-07-03"),
            ("place", "月下林边"),
            ("weather", "微风"),
            ("mood", "安静"),
        ];
        let html = render_fence_block("meta-strip", &props, "今天就记这一点。", &theme);

        assert!(html.contains(theme.accent_soft));
        assert!(html.contains("2026-07-03"));
        assert!(html.contains("月下林边"));
        assert!(html.contains("微风"));
        assert!(html.contains("安静"));
        assert!(html.contains("今天就记这一点"));
    }

    #[test]
    fn unknown_fence_uses_generic_inline_container() {
        let theme = default_theme();
        let html = render_fence_block("note-box", &[], "未知 **块**", &theme);

        assert!(html.contains(theme.block_bg));
        assert!(html.contains("<strong"));
        assert!(html.contains("块"));
    }
}
