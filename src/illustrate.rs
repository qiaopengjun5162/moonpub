//! Article illustration generation using inline HTML/CSS.
//! Reference: ian-xiaohei-illustrations, guizang-ppt-skill

pub enum IllustType {
    QuoteCard { text: String, source: String },
    Divider { label: String },
    ConceptCard { number: u32, title: String, desc: String },
    EmotionCard { mood: String, text: String },
}

pub fn render_illustration(ill: &IllustType) -> String {
    match ill {
        IllustType::QuoteCard { text, source } => render_quote_card(text, source),
        IllustType::Divider { label } => render_divider(label),
        IllustType::ConceptCard { number, title, desc } => render_concept_card(*number, title, desc),
        IllustType::EmotionCard { mood, text } => render_emotion_card(mood, text),
    }
}

fn render_quote_card(text: &str, source: &str) -> String {
    let source_line = if source.is_empty() { String::new() } else {
        format!("<p style=\"margin:12px 0 0;text-align:right;font-size:13px;color:#888;\">—— {source}</p>")
    };
    format!("<section style=\"margin:28px 0;padding:32px 28px;background:linear-gradient(135deg,#1a1a2e,#16213e);border-radius:4px;\">\n<p style=\"margin:0;font-size:20px;font-weight:bold;color:#f0f0f0;line-height:1.7;\">\u{201c}{text}\u{201d}</p>\n{source_line}\n</section>\n\n")
}

fn render_divider(label: &str) -> String {
    if label.is_empty() { return "<hr style=\"border:none;border-top:1px solid #e8e8e8;margin:2em 0;\" />\n\n".to_owned(); }
    format!("<section style=\"margin:2em 0;text-align:center;\">\n<span style=\"display:inline-block;background:#fff;padding:0 16px;color:#aaa;font-size:12px;letter-spacing:2px;position:relative;top:-0.7em;\">{label}</span>\n<hr style=\"border:none;border-top:1px solid #e8e8e8;margin:0;\" />\n</section>\n\n")
}

fn render_concept_card(number: u32, title: &str, desc: &str) -> String {
    let colors = ["#2c2c2c", "#e65100", "#1565c0", "#2e7d32", "#6a1b9a"];
    let accent = colors[(number as usize).saturating_sub(1).min(colors.len() - 1)];
    format!("<section style=\"margin:18px 0;background:#fff;border:1px solid #e8e8e8;border-left:4px solid {accent};padding:16px 20px;\">\n<table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\"><tr>\n<td style=\"width:36px;vertical-align:top;\"><span style=\"display:inline-block;width:28px;height:28px;background:{accent};color:#fff;font-weight:bold;text-align:center;line-height:28px;border-radius:50%;font-size:13px;\">{number}</span></td>\n<td style=\"vertical-align:top;\">\n<p style=\"margin:0 0 4px;font-size:15px;font-weight:bold;color:#1a1a1a;\">{title}</p>\n<p style=\"margin:0;font-size:13px;color:#666;line-height:1.7;\">{desc}</p>\n</td></tr></table></section>\n\n")
}

fn render_emotion_card(mood: &str, text: &str) -> String {
    let (bg, _accent, emoji) = match mood {
        "hope" | "希望" => ("#fef9e7", "#f39c12", "✨"),
        "calm" | "平静" => ("#eaf2f8", "#3498db", "🌊"),
        "strong" | "力量" => ("#fdedec", "#e74c3c", "🔥"),
        "think" | "思考" => ("#f4ecf7", "#8e44ad", "💭"),
        _ => ("#f5f5f5", "#888", "💡"),
    };
    format!("<section style=\"margin:20px 0;padding:20px 24px;background:{bg};border-radius:4px;text-align:center;\">\n<span style=\"font-size:28px;\">{emoji}</span>\n<p style=\"margin:8px 0 0;font-size:14px;color:#555;line-height:1.8;\">{text}</p>\n</section>\n\n")
}

pub fn render_code_block(lang: &str, code: &str) -> String {
    let label = if lang.is_empty() { "CODE" } else { lang };
    let escaped = code.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    format!("<section style=\"margin:18px 0;\">\n<section style=\"display:inline-block;background:#1a1a1a;color:#64b5f6;font-weight:bold;font-size:12px;padding:6px 12px;letter-spacing:1px;border-radius:4px 4px 0 0;\">{label}</section>\n<section style=\"background:#0f0f10;border:1px solid #333;border-radius:0 4px 4px 4px;padding:16px;font-family:SF Mono,Menlo,Consolas,monospace;font-size:13px;line-height:1.7;color:#e6e6e6;overflow-x:auto;\">\n<pre style=\"margin:0;\">{escaped}</pre>\n</section></section>\n\n")
}

pub fn render_timeline(items: &[(String, String)]) -> String {
    let mut html = String::from("<section style=\"margin:24px 0;padding:0 8px;\">\n");
    for (i, (date, desc)) in items.iter().enumerate() {
        let dot_color = if i == 0 { "#e65100" } else { "#2c2c2c" };
        let line = if i == items.len() - 1 { "" } else { "<section style=\"width:2px;height:16px;background:#e0e0e0;margin-left:6px;\"></section>" };
        html.push_str(&format!("<table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;\"><tr>\n<td style=\"width:14px;vertical-align:top;padding-top:3px;\"><section style=\"width:10px;height:10px;background:{dot_color};border-radius:50%;\"></section></td>\n<td style=\"padding:0 0 8px 12px;vertical-align:top;\">\n<span style=\"font-size:12px;color:#888;font-weight:bold;\">{date}</span>\n<p style=\"margin:2px 0 0;font-size:14px;color:#555;\">{desc}</p>\n</td></tr></table>\n{line}\n"));
    }
    html.push_str("</section>\n\n");
    html
}

pub fn render_comparison(left_title: &str, right_title: &str, rows: &[(String, String)]) -> String {
    let mut html = format!("<section style=\"margin:24px 0;\">\n<table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\">\n<tr><td style=\"width:50%;background:#1a1a1a;color:#fff;font-weight:bold;font-size:13px;padding:10px 14px;text-align:center;\">{}</td>\n<td style=\"width:2px;\"></td>\n<td style=\"width:50%;background:#e65100;color:#fff;font-weight:bold;font-size:13px;padding:10px 14px;text-align:center;\">{}</td></tr>\n", left_title, right_title);
    for (i, (left, right)) in rows.iter().enumerate() {
        let bg = if i % 2 == 0 { "#fff" } else { "#fafafa" };
        html.push_str(&format!("<tr><td style=\"padding:8px 14px;font-size:13px;color:#555;background:{bg};\">{}</td>\n<td></td>\n<td style=\"padding:8px 14px;font-size:13px;color:#555;background:{bg};\">{}</td></tr>\n", left, right));
    }
    html.push_str("</table></section>\n\n");
    html
}

pub fn render_tip(icon: &str, text: &str) -> String {
    let emoji = if icon.is_empty() { "💡" } else { icon };
    format!("<section style=\"margin:18px 0;padding:14px 18px;background:#fef9e7;border-left:3px solid #f39c12;border-radius:0 4px 4px 0;\">\n<span style=\"font-size:16px;\">{emoji}</span> <span style=\"font-size:14px;color:#555;line-height:1.8;\">{text}</span>\n</section>\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_card_has_text() {
        let html = render_illustration(&IllustType::QuoteCard { text: "六便士".to_owned(), source: "毛姆".to_owned() });
        assert!(html.contains("六便士"));
    }

    #[test]
    fn divider_with_label_works() {
        assert!(render_divider("第二部分").contains("第二部分"));
    }

    #[test]
    fn concept_card_works() {
        let html = render_illustration(&IllustType::ConceptCard { number: 1, title: "T".to_owned(), desc: "D".to_owned() });
        assert!(html.contains("T"));
    }

    #[test]
    fn emotion_hope_has_emoji() {
        let html = render_illustration(&IllustType::EmotionCard { mood: "希望".to_owned(), text: "T".to_owned() });
        assert!(html.contains("✨"));
    }

    #[test]
    fn code_block_works() {
        assert!(render_code_block("rust", "fn main()").contains("rust"));
    }

    #[test]
    fn timeline_works() {
        let items = vec![("2024".to_owned(), "事件".to_owned())];
        assert!(render_timeline(&items).contains("2024"));
    }

    #[test]
    fn comparison_works() {
        let rows = vec![("A".to_owned(), "B".to_owned())];
        assert!(render_comparison("L", "R", &rows).contains("L"));
    }

    #[test]
    fn tip_has_emoji() {
        assert!(render_tip("", "测试").contains("💡"));
    }
}
