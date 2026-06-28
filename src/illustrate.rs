//! Article illustration generation using inline HTML/CSS.
//! Reference: ian-xiaohei-illustrations, guizang-ppt-skill

use crate::theme::Theme;

pub enum IllustType {
    QuoteCard {
        text: String,
        source: String,
    },
    Divider {
        label: String,
    },
    ConceptCard {
        number: u32,
        title: String,
        desc: String,
    },
    EmotionCard {
        mood: String,
        text: String,
    },
}

pub fn render_illustration(ill: &IllustType, theme: &Theme) -> String {
    match ill {
        IllustType::QuoteCard { text, source } => render_quote_card(text, source, theme),
        IllustType::Divider { label } => render_divider(label, theme),
        IllustType::ConceptCard {
            number,
            title,
            desc,
        } => render_concept_card(*number, title, desc, theme),
        IllustType::EmotionCard { mood, text } => render_emotion_card(mood, text, theme),
    }
}

fn render_quote_card(text: &str, source: &str, theme: &Theme) -> String {
    let source_line = if source.is_empty() {
        String::new()
    } else {
        format!(
            "<p style=\"margin:12px 0 0;text-align:right;font-size:13px;color:{};\">—— {source}</p>",
            theme.text_muted
        )
    };
    format!(
        "<section style=\"margin:28px 0;padding:32px 28px;background:linear-gradient(135deg,#1a1a2e,#16213e);border-radius:4px;\">\n<p style=\"margin:0;font-size:20px;font-weight:bold;color:#f0f0f0;line-height:1.7;\">\u{201c}{text}\u{201d}</p>\n{source_line}\n</section>\n\n"
    )
}

fn render_divider(label: &str, theme: &Theme) -> String {
    if label.is_empty() {
        return "<hr style=\"border:none;border-top:1px solid #e8e8e8;margin:2em 0;\" />\n\n"
            .to_owned();
    }
    format!(
        "<section style=\"margin:2em 0;text-align:center;\">\n<span style=\"display:inline-block;background:{};padding:0 16px;color:{};font-size:12px;letter-spacing:2px;position:relative;top:-0.7em;\">{label}</span>\n<hr style=\"border:none;border-top:1px solid #e8e8e8;margin:0;\" />\n</section>\n\n",
        theme.section_bg, theme.text_muted
    )
}

fn render_concept_card(number: u32, title: &str, desc: &str, theme: &Theme) -> String {
    let colors = [theme.accent, "#e65100", "#1565c0", "#2e7d32", "#6a1b9a"];
    let accent = colors[(number as usize).saturating_sub(1).min(colors.len() - 1)];
    format!(
        "<section style=\"margin:18px 0;background:{};border:1px solid #e8e8e8;border-left:4px solid {accent};padding:16px 20px;\">\n<table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\"><tr>\n<td style=\"width:36px;vertical-align:top;\"><span style=\"display:inline-block;width:28px;height:28px;background:{accent};color:#fff;font-weight:bold;text-align:center;line-height:28px;border-radius:50%;font-size:13px;\">{number}</span></td>\n<td style=\"vertical-align:top;\">\n<p style=\"margin:0 0 4px;font-size:15px;font-weight:bold;color:{};\">{title}</p>\n<p style=\"margin:0;font-size:13px;color:{};line-height:1.7;\">{desc}</p>\n</td></tr></table></section>\n\n",
        theme.block_bg, theme.heading_color, theme.text_muted
    )
}

fn render_emotion_card(mood: &str, text: &str, theme: &Theme) -> String {
    let (bg, _accent, emoji) = match mood {
        "hope" | "希望" => ("#fef9e7", "#f39c12", "✨"),
        "calm" | "平静" => ("#eaf2f8", "#3498db", "🌊"),
        "strong" | "力量" => ("#fdedec", "#e74c3c", "🔥"),
        "think" | "思考" => ("#f4ecf7", "#8e44ad", "💭"),
        _ => ("#f5f5f5", "#888", "💡"),
    };
    format!(
        "<section style=\"margin:20px 0;padding:20px 24px;background:{bg};border-radius:4px;text-align:center;\">\n<span style=\"font-size:28px;\">{emoji}</span>\n<p style=\"margin:8px 0 0;font-size:14px;color:{};line-height:1.8;\">{text}</p>\n</section>\n\n",
        theme.text_color
    )
}

pub fn render_code_block(lang: &str, code: &str, theme: &Theme) -> String {
    let label = if lang.is_empty() { "CODE" } else { lang };
    let escaped = code
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
    let code_color = if theme.code_color.is_empty() {
        theme.text_color
    } else {
        theme.code_color
    };
    format!(
        "<section style=\"margin:20px 0;\">\n<section style=\"display:inline-block;background:{};color:#fff;font-weight:bold;font-size:12px;padding:6px 12px;letter-spacing:1px;border-radius:6px 6px 0 0;\">{label}</section>\n<section style=\"background:{};border:1px solid {};border-radius:0 6px 6px 6px;padding:16px;font-family:SF Mono,Menlo,Consolas,monospace;font-size:13px;line-height:1.75;color:{};overflow-x:auto;\">\n<pre style=\"margin:0;\">{escaped}</pre>\n</section></section>\n\n",
        theme.accent, theme.code_bg, theme.border, code_color
    )
}

pub fn render_timeline(items: &[(String, String)], theme: &Theme) -> String {
    let mut html = String::from("<section style=\"margin:24px 0;padding:0 8px;\">\n");
    for (i, (date, desc)) in items.iter().enumerate() {
        let dot_color = if i == 0 { theme.accent } else { "#e0e0e0" };
        let line = if i == items.len() - 1 {
            ""
        } else {
            "<section style=\"width:2px;height:16px;background:#e0e0e0;margin-left:6px;\"></section>"
        };
        html.push_str(&format!(
            "<table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;\"><tr>\n<td style=\"width:14px;vertical-align:top;padding-top:3px;\"><section style=\"width:10px;height:10px;background:{dot_color};border-radius:50%;\"></section></td>\n<td style=\"padding:0 0 8px 12px;vertical-align:top;\">\n<span style=\"font-size:12px;color:{};font-weight:bold;\">{date}</span>\n<p style=\"margin:2px 0 0;font-size:14px;color:{};\">{desc}</p>\n</td></tr></table>\n{line}\n",
            theme.text_muted, theme.text_color
        ));
    }
    html.push_str("</section>\n\n");
    html
}

pub fn render_comparison(
    left_title: &str,
    right_title: &str,
    rows: &[(String, String)],
    theme: &Theme,
) -> String {
    let mut html = format!(
        "<section style=\"margin:24px 0;\">\n<table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\">\n<tr><td style=\"width:50%;background:{};color:#fff;font-weight:bold;font-size:13px;padding:10px 14px;text-align:center;\">{}</td>\n<td style=\"width:2px;\"></td>\n<td style=\"width:50%;background:{};color:#fff;font-weight:bold;font-size:13px;padding:10px 14px;text-align:center;\">{}</td></tr>\n",
        theme.accent, left_title, theme.heading_border, right_title
    );
    for (i, (left, right)) in rows.iter().enumerate() {
        let bg = if i % 2 == 0 {
            theme.section_bg
        } else {
            theme.block_bg
        };
        html.push_str(&format!(
            "<tr><td style=\"padding:8px 14px;font-size:13px;color:{};background:{bg};\">{}</td>\n<td></td>\n<td style=\"padding:8px 14px;font-size:13px;color:{};background:{bg};\">{}</td></tr>\n",
            theme.text_color, left, theme.text_color, right
        ));
    }
    html.push_str("</table></section>\n\n");
    html
}

pub fn render_tip(icon: &str, text: &str, theme: &Theme) -> String {
    let emoji = if icon.is_empty() { "💡" } else { icon };
    format!(
        "<section style=\"margin:18px 0;padding:14px 18px;background:{};border-left:3px solid {};border-radius:0 4px 4px 0;\">\n<span style=\"font-size:16px;\">{emoji}</span> <span style=\"font-size:14px;color:{};line-height:1.8;\">{text}</span>\n</section>\n\n",
        theme.block_bg, theme.accent, theme.text_color
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_theme() -> Theme {
        Theme::default()
    }

    #[test]
    fn quote_card_has_text() {
        let html = render_illustration(
            &IllustType::QuoteCard {
                text: "六便士".to_owned(),
                source: "毛姆".to_owned(),
            },
            &test_theme(),
        );
        assert!(html.contains("六便士"));
    }

    #[test]
    fn divider_with_label_works() {
        assert!(render_divider("第二部分", &test_theme()).contains("第二部分"));
    }

    #[test]
    fn concept_card_works() {
        let html = render_illustration(
            &IllustType::ConceptCard {
                number: 1,
                title: "T".to_owned(),
                desc: "D".to_owned(),
            },
            &test_theme(),
        );
        assert!(html.contains("T"));
    }

    #[test]
    fn emotion_hope_has_emoji() {
        let html = render_illustration(
            &IllustType::EmotionCard {
                mood: "希望".to_owned(),
                text: "T".to_owned(),
            },
            &test_theme(),
        );
        assert!(html.contains("✨"));
    }

    #[test]
    fn code_block_works() {
        assert!(render_code_block("rust", "fn main()", &test_theme()).contains("rust"));
    }

    #[test]
    fn code_block_uses_theme_code_palette() {
        let theme = Theme::paper();
        let html = render_code_block("rust", "println!(\"hi\")", &theme);

        assert!(html.contains(theme.code_bg));
        assert!(html.contains(theme.code_color));
        assert!(html.contains(theme.border));
        assert!(html.contains("&quot;hi&quot;"));
    }

    #[test]
    fn timeline_works() {
        let items = vec![("2024".to_owned(), "事件".to_owned())];
        assert!(render_timeline(&items, &test_theme()).contains("2024"));
    }

    #[test]
    fn comparison_works() {
        let rows = vec![("A".to_owned(), "B".to_owned())];
        assert!(render_comparison("L", "R", &rows, &test_theme()).contains("L"));
    }

    #[test]
    fn tip_has_emoji() {
        assert!(render_tip("", "测试", &test_theme()).contains("💡"));
    }
}
