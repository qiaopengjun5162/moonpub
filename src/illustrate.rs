//! Article illustration generation using inline HTML/CSS.
//! Reference: ian-xiaohei-illustrations, guizang-ppt-skill, html-anything

/// Illustration types that can be embedded in WeChat articles.
pub enum IllustType {
    /// Dark card with large quote text — ideal for key quotes
    QuoteCard { text: String, source: String },
    /// Visual separator between sections
    Divider { label: String },
    /// Numbered concept card with icon, title, and description
    ConceptCard { number: u32, title: String, desc: String },
    /// Abstract emotion card using CSS shapes and colors
    EmotionCard { mood: String, text: String },
}

/// Generate inline HTML for an illustration block.
pub fn render_illustration(ill: &IllustType) -> String {
    match ill {
        IllustType::QuoteCard { text, source } => render_quote_card(text, source),
        IllustType::Divider { label } => render_divider(label),
        IllustType::ConceptCard { number, title, desc } => {
            render_concept_card(*number, title, desc)
        }
        IllustType::EmotionCard { mood, text } => render_emotion_card(mood, text),
    }
}

// ── Quote Card: dark background, large text, optional source ─────────────────

fn render_quote_card(text: &str, source: &str) -> String {
    let source_line = if source.is_empty() {
        String::new()
    } else {
        format!(
            "<p style=\"margin:12px 0 0;text-align:right;font-size:13px;color:#888;\">—— {source}</p>"
        )
    };
    format!(
        "<section style=\"margin:28px 0;padding:32px 28px;background:linear-gradient(135deg,#1a1a2e,#16213e);border-radius:4px;\">\n\
         <p style=\"margin:0;font-size:20px;font-weight:bold;color:#f0f0f0;line-height:1.7;letter-spacing:1px;\">\u{201c}{text}\u{201d}</p>\n\
         {source_line}\n\
         </section>\n\n"
    )
}

// ── Divider: visual section break ────────────────────────────────────────────

fn render_divider(label: &str) -> String {
    if label.is_empty() {
        return "<hr style=\"border:none;border-top:1px solid #e8e8e8;margin:2em 0;\" />\n\n".to_owned();
    }
    format!(
        "<section style=\"margin:2em 0;text-align:center;\">\n\
         <span style=\"display:inline-block;background:#fff;padding:0 16px;color:#aaa;font-size:12px;letter-spacing:2px;position:relative;top:-0.7em;\">{label}</span>\n\
         <hr style=\"border:none;border-top:1px solid #e8e8e8;margin:0;\" />\n\
         </section>\n\n"
    )
}

// ── Concept Card: numbered idea card ─────────────────────────────────────────

fn render_concept_card(number: u32, title: &str, desc: &str) -> String {
    let colors = ["#2c2c2c", "#e65100", "#1565c0", "#2e7d32", "#6a1b9a"];
    let accent = colors[(number as usize).saturating_sub(1).min(colors.len() - 1)];
    format!(
        "<section style=\"margin:18px 0;background:#fff;border:1px solid #e8e8e8;border-left:4px solid {accent};padding:16px 20px;\">\n\
         <table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\"><tr>\n\
         <td style=\"width:36px;vertical-align:top;\"><span style=\"display:inline-block;width:28px;height:28px;background:{accent};color:#fff;font-weight:bold;text-align:center;line-height:28px;border-radius:50%;font-size:13px;\">{number}</span></td>\n\
         <td style=\"vertical-align:top;\">\n\
         <p style=\"margin:0 0 4px;font-size:15px;font-weight:bold;color:#1a1a1a;\">{title}</p>\n\
         <p style=\"margin:0;font-size:13px;color:#666;line-height:1.7;\">{desc}</p>\n\
         </td></tr></table></section>\n\n"
    )
}

// ── Emotion Card: mood-based abstract card ────────────────────────────────────

fn render_emotion_card(mood: &str, text: &str) -> String {
    let (bg, _accent, emoji) = match mood {
        "hope" | "希望" => ("#fef9e7", "#f39c12", "✨"),
        "calm" | "平静" => ("#eaf2f8", "#3498db", "🌊"),
        "strong" | "力量" => ("#fdedec", "#e74c3c", "🔥"),
        "think" | "思考" => ("#f4ecf7", "#8e44ad", "💭"),
        _ => ("#f5f5f5", "#888", "💡"),
    };
    format!(
        "<section style=\"margin:20px 0;padding:20px 24px;background:{bg};border-radius:4px;text-align:center;\">\n\
         <span style=\"font-size:28px;\">{emoji}</span>\n\
         <p style=\"margin:8px 0 0;font-size:14px;color:#555;line-height:1.8;\">{text}</p>\n\
         </section>\n\n"
    )
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_card_contains_text() {
        let html = render_illustration(&IllustType::QuoteCard {
            text: "满地都是六便士，他却抬头看见了月亮".to_owned(),
            source: "毛姆".to_owned(),
        });
        assert!(html.contains("六便士"));
        assert!(html.contains("毛姆"));
        assert!(html.contains("linear-gradient"));
    }

    #[test]
    fn quote_card_empty_source_omitted() {
        let html = render_illustration(&IllustType::QuoteCard {
            text: "测试".to_owned(),
            source: String::new(),
        });
        assert!(!html.contains("——"));
    }

    #[test]
    fn divider_with_label() {
        let html = render_illustration(&IllustType::Divider {
            label: "第二部分".to_owned(),
        });
        assert!(html.contains("第二部分"));
        assert!(html.contains("<hr"));
    }

    #[test]
    fn divider_empty_label() {
        let html = render_illustration(&IllustType::Divider {
            label: String::new(),
        });
        assert!(!html.contains("第二部分"));
        assert!(html.contains("<hr"));
    }

    #[test]
    fn concept_card_uses_colors() {
        let html = render_illustration(&IllustType::ConceptCard {
            number: 1,
            title: "测试".to_owned(),
            desc: "描述".to_owned(),
        });
        assert!(html.contains("测试"));
        assert!(html.contains("描述"));
        assert!(html.contains("border-left:4px"));
    }

    #[test]
    fn emotion_card_variants() {
        let hope = render_illustration(&IllustType::EmotionCard {
            mood: "希望".to_owned(),
            text: "测试".to_owned(),
        });
        assert!(hope.contains("✨"));

        let unknown = render_illustration(&IllustType::EmotionCard {
            mood: "unknown".to_owned(),
            text: "测试".to_owned(),
        });
        assert!(unknown.contains("💡"));
    }
}

// ── Code Block: syntax-highlighted code for tech articles ─────────────────────

pub fn render_code_block(lang: &str, code: &str) -> String {
    let label = if lang.is_empty() { "CODE" } else { lang };
    format!(
        "<section style=\"margin:18px 0;\">\n\
         <section style=\"display:inline-block;background:#1a1a1a;color:#64b5f6;font-weight:bold;font-size:12px;padding:6px 12px;letter-spacing:1px;border-radius:4px 4px 0 0;\">{label}</section>\n\
         <section style=\"background:#0f0f10;border:1px solid #333;border-radius:0 4px 4px 4px;padding:16px;font-family:'SF Mono',Menlo,Consolas,monospace;font-size:13px;line-height:1.7;color:#e6e6e6;overflow-x:auto;\">\n\
         <pre style=\"margin:0;\">{}</pre>\n\
         </section></section>\n\n",
        html_escape_code(code)
    )
}

// ── Timeline: chronological milestones ───────────────────────────────────────

pub fn render_timeline(items: &[(String, String)]) -> String {
    let mut html = String::from(
        "<section style=\"margin:24px 0;padding:0 8px;\">\n"
    );
    for (i, (date, desc)) in items.iter().enumerate() {
        let dot_color = if i == 0 { "#e65100" } else { "#2c2c2c" };
        let is_last = i == items.len() - 1;
        let line = if is_last { "" } else { "<section style=\"width:2px;height:16px;background:#e0e0e0;margin-left:6px;\"></section>" };
        html.push_str(&format!(
            "<table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;\"><tr>\n\
             <td style=\"width:14px;vertical-align:top;padding-top:3px;\"><section style=\"width:10px;height:10px;background:{dot_color};border-radius:50%;\"></section></td>\n\
             <td style=\"padding:0 0 8px 12px;vertical-align:top;\">\n\
             <span style=\"font-size:12px;color:#888;font-weight:bold;\">{date}</span>\n\
             <p style=\"margin:2px 0 0;font-size:14px;color:#555;\">{desc}</p>\n\
             </td></tr></table>\n\
             {line}\n"
        ));
    }
    html.push_str("</section>\n\n");
    html
}

// ── Comparison Table: side-by-side ────────────────────────────────────────────

pub fn render_comparison(left_title: &str, right_title: &str, rows: &[(String, String)]) -> String {
    let mut html = format!(
        "<section style=\"margin:24px 0;\">\n\
         <table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\">\n\
         <tr><td style=\"width:50%;background:#1a1a1a;color:#fff;font-weight:bold;font-size:13px;padding:10px 14px;text-align:center;\">{}</td>\n\
         <td style=\"width:2px;\"></td>\n\
         <td style=\"width:50%;background:#e65100;color:#fff;font-weight:bold;font-size:13px;padding:10px 14px;text-align:center;\">{}</td></tr>\n",
        left_title, right_title
    );
    for (i, (left, right)) in rows.iter().enumerate() {
        let bg = if i % 2 == 0 { "#fff" } else { "#fafafa" };
        html.push_str(&format!(
            "<tr><td style=\"padding:8px 14px;font-size:13px;color:#555;background:{bg};\">{}</td>\n\
             <td></td>\n\
             <td style=\"padding:8px 14px;font-size:13px;color:#555;background:{bg};\">{}</td></tr>\n",
            left, right
        ));
    }
    html.push_str("</table></section>\n\n");
    html
}

// ── Tip Card: practical tip/hint ──────────────────────────────────────────────

pub fn render_tip(icon: &str, text: &str) -> String {
    let emoji = if icon.is_empty() { "💡" } else { icon };
    format!(
        "<section style=\"margin:18px 0;padding:14px 18px;background:#fef9e7;border-left:3px solid #f39c12;border-radius:0 4px 4px 0;\">\n\
         <span style=\"font-size:16px;\">{emoji}</span>\n\
         <span style=\"font-size:14px;color:#555;line-height:1.8;\">{text}</span>\n\
         </section>\n\n"
    )
}

fn html_escape_code(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

    #[test]
    fn code_block_highlights() {
        let html = render_code_block("rust", "fn main() {}");
        assert!(html.contains("rust"));
        assert!(html.contains("fn main"));
        assert!(html.contains("monospace"));
    }

    #[test]
    fn code_block_empty_lang() {
        let html = render_code_block("", "code");
        assert!(html.contains("CODE"));
    }

    #[test]
    fn timeline_renders_items() {
        let items = vec![
            ("2024".to_owned(), "事件一".to_owned()),
            ("2025".to_owned(), "事件二".to_owned()),
        ];
        let html = render_timeline(&items);
        assert!(html.contains("2024"));
        assert!(html.contains("事件二"));
        assert!(html.contains("border-radius:50%"));
    }

    #[test]
    fn comparison_table_both_columns() {
        let rows = vec![("A".to_owned(), "B".to_owned())];
        let html = render_comparison("Left", "Right", &rows);
        assert!(html.contains("Left"));
        assert!(html.contains("Right"));
        assert!(html.contains("A"));
        assert!(html.contains("B"));
    }

    #[test]
    fn tip_card_default_icon() {
        let html = render_tip("", "测试提示");
        assert!(html.contains("💡"));
        assert!(html.contains("测试提示"));
        assert!(html.contains("fef9e7"));
    }
