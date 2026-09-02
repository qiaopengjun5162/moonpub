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
        "<section style=\"margin:18px 0;background:{};border:1px solid #e8e8e8;border-left:4px solid {accent};padding:16px 18px;border-radius:8px;\">\n<span style=\"display:inline-block;width:28px;height:28px;background:{accent};color:#fff;font-weight:bold;text-align:center;line-height:28px;border-radius:50%;font-size:13px;margin:0 8px 8px 0;\">{number}</span>\n<p style=\"margin:0 0 4px;font-size:15px;font-weight:bold;color:{};\">{title}</p>\n<p style=\"margin:0;font-size:13px;color:{};line-height:1.75;\">{desc}</p>\n</section>\n\n",
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

pub fn render_code_block(lang: &str, code: &str) -> String {
    let _ = lang; // 不再显示语言标签，代码块顶部用 macOS 窗口圆点装饰
    // 微信兼容：不用 <pre>（换行会被剥掉），每行一个 <p>，word-break 强制折行
    let mut body = String::new();
    for raw in code.split('\n') {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if line.is_empty() {
            body.push_str("<p style=\"margin:0;\">&nbsp;</p>");
            continue;
        }
        let hl = xcode_highlight(line);
        body.push_str(&format!(
            "<p style=\"margin:0;white-space:pre-wrap;word-break:break-all;font-family:Menlo,Consolas,monospace;\">{hl}</p>"
        ));
    }
    format!(
        "<section style=\"margin:20px 0;border-radius:10px;overflow:hidden;background:{XCODE_BG};border:1px solid #2c2c2e;\">\n<section style=\"background:#2c2c2e;padding:8px 14px;\">\n<span style=\"display:inline-block;width:12px;height:12px;border-radius:50%;background:#ff5f57;margin-right:8px;\">&nbsp;</span>\n<span style=\"display:inline-block;width:12px;height:12px;border-radius:50%;background:#febc2e;margin-right:8px;\">&nbsp;</span>\n<span style=\"display:inline-block;width:12px;height:12px;border-radius:50%;background:#28c840;\">&nbsp;</span>\n</section>\n<section style=\"padding:16px;\">\n{body}\n</section></section>\n\n"
    )
}

// ── Xcode Dark 语法高亮（macOS 风格） ────────────────────────────────
const XCODE_BG: &str = "#1e1e1e";
const XCODE_FG: &str = "#e5e5ea";
const XCODE_COMMENT: &str = "#7f8c98";
const XCODE_KEYWORD: &str = "#ff6482";
const XCODE_STRING: &str = "#ff8170";
const XCODE_NUMBER: &str = "#d0bf69";
const XCODE_FUNC: &str = "#5dd8ff";
const XCODE_TYPE: &str = "#5dd8ff";

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}
fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

const KEYWORDS: &[&str] = &[
    "fn", "let", "mut", "pub", "struct", "impl", "enum", "match", "if", "else", "for", "while",
    "loop", "return", "use", "mod", "trait", "async", "await", "move", "ref", "const", "static",
    "break", "continue", "in", "as", "where", "self", "true", "false", "None", "Some", "Ok", "Err",
    "def", "class", "import", "from", "try", "except", "with", "lambda", "yield", "global",
    "nonlocal", "pass", "elif", "and", "or", "not", "is", "True", "False", "echo", "export",
    "local", "then", "fi", "done", "esac", "case", "do", "select", "return",
];

fn xcode_highlight(code: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = code.chars().collect();
    let n = chars.len();
    let mut i = 0;

    while i < n {
        let c = chars[i];

        // 块注释 /* ... */
        if c == '/' && i + 1 < n && chars[i + 1] == '*' {
            let mut j = i + 2;
            while j + 1 < n && !(chars[j] == '*' && chars[j + 1] == '/') {
                j += 1;
            }
            let end = if j + 1 < n { j + 2 } else { n };
            let text: String = chars[i..end].iter().collect();
            out.push_str(&format!(
                "<span style=\"color:{XCODE_COMMENT};\">{}</span>",
                esc(&text)
            ));
            i = end;
            continue;
        }
        // 行注释 // 或 #
        if (c == '/' && i + 1 < n && chars[i + 1] == '/') || c == '#' {
            let mut j = i + 1;
            while j < n && chars[j] != '\n' {
                j += 1;
            }
            let text: String = chars[i..j].iter().collect();
            out.push_str(&format!(
                "<span style=\"color:{XCODE_COMMENT};\">{}</span>",
                esc(&text)
            ));
            i = j;
            continue;
        }
        // 字符串
        if c == '"' || c == '\'' || c == '`' {
            let quote = c;
            let mut j = i + 1;
            while j < n {
                if chars[j] == '\\' && j + 1 < n {
                    j += 2;
                    continue;
                }
                if chars[j] == quote {
                    j += 1;
                    break;
                }
                j += 1;
            }
            let text: String = chars[i..j.min(n)].iter().collect();
            out.push_str(&format!(
                "<span style=\"color:{XCODE_STRING};\">{}</span>",
                esc(&text)
            ));
            i = j;
            continue;
        }
        // 数字
        if c.is_ascii_digit() {
            let mut j = i;
            while j < n && (chars[j].is_ascii_alphanumeric() || chars[j] == '.' || chars[j] == '_')
            {
                j += 1;
            }
            let text: String = chars[i..j].iter().collect();
            out.push_str(&format!(
                "<span style=\"color:{XCODE_NUMBER};\">{}</span>",
                esc(&text)
            ));
            i = j;
            continue;
        }
        // 标识符
        if is_ident_start(c) {
            let mut j = i;
            while j < n && is_ident_char(chars[j]) {
                j += 1;
            }
            let word: String = chars[i..j].iter().collect();
            // 函数调用：word 后紧跟 ( 或宏调用 !
            let is_call = j < n && (chars[j] == '(' || chars[j] == '!');
            let color = if KEYWORDS.contains(&word.as_str()) {
                XCODE_KEYWORD
            } else if word
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_uppercase())
            {
                XCODE_TYPE
            } else if is_call {
                XCODE_FUNC
            } else {
                XCODE_FG
            };
            out.push_str(&format!(
                "<span style=\"color:{color};\">{}</span>",
                esc(&word)
            ));
            i = j;
            continue;
        }
        // 其他原样
        out.push(c);
        i += 1;
    }
    out
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
            "<section style=\"margin:0 0 8px;\">\n<span style=\"display:inline-block;width:10px;height:10px;background:{dot_color};border-radius:50%;margin:4px 12px 0 0;vertical-align:top;\"></span>\n<section style=\"display:inline-block;width:88%;vertical-align:top;\"><span style=\"font-size:12px;color:{};font-weight:bold;\">{date}</span>\n<p style=\"margin:2px 0 0;font-size:14px;color:{};line-height:1.75;\">{desc}</p></section>\n</section>\n{line}\n",
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
        "<section style=\"margin:24px 0;padding:14px;background:{};border:1px solid {};border-radius:10px;\">\n",
        theme.block_bg, theme.border
    );
    for (i, (left, right)) in rows.iter().enumerate() {
        let bg = if i % 2 == 0 {
            theme.section_bg
        } else {
            theme.block_bg
        };
        html.push_str(&format!(
            "<section style=\"margin:0 0 12px;padding:12px 14px;background:{bg};border:1px solid {};border-radius:8px;\">\n<p style=\"margin:0 0 6px;color:{};font-size:12px;font-weight:bold;letter-spacing:0.08em;\">{}</p>\n<p style=\"margin:0 0 10px;color:{};font-size:14px;line-height:1.75;\">{}</p>\n<p style=\"margin:0 0 6px;color:{};font-size:12px;font-weight:bold;letter-spacing:0.08em;\">{}</p>\n<p style=\"margin:0;color:{};font-size:14px;line-height:1.75;\">{}</p>\n</section>\n",
            theme.border,
            theme.accent,
            left_title,
            theme.text_color,
            left,
            theme.heading_border,
            right_title,
            theme.text_color,
            right
        ));
    }
    html.push_str("</section>\n\n");
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
        assert!(!html.contains("<table"));
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
        // 语言标签不再显示，代码块带 macOS 窗口圆点
        assert!(render_code_block("rust", "fn main()").contains("#ff5f57"));
        assert!(!render_code_block("rust", "fn main()").contains(">rust<"));
    }

    #[test]
    fn code_block_uses_fixed_xcode_palette() {
        let html = render_code_block("rust", "println!(\"hi\")");

        // xcode 固定配色：背景 + 语法高亮
        assert!(html.contains("#1e1e1e"));
        assert!(!html.contains("monokai"));
        assert!(html.contains("&quot;hi&quot;"));
        // 微信兼容：不用 <pre>（换行会被剥），改用每行 <p>
        assert!(!html.contains("<pre"));
        // 字符串高亮
        assert!(html.contains("#ff8170"));
        // 函数调用高亮（println 后跟 (）
        assert!(html.contains("#5dd8ff"));
    }

    #[test]
    fn timeline_works() {
        let items = vec![("2024".to_owned(), "事件".to_owned())];
        let html = render_timeline(&items, &test_theme());
        assert!(html.contains("2024"));
        assert!(!html.contains("<table"));
    }

    #[test]
    fn comparison_works() {
        let rows = vec![("A".to_owned(), "B".to_owned())];
        let html = render_comparison("L", "R", &rows, &test_theme());
        assert!(html.contains("L"));
        assert!(!html.contains("<table"));
    }

    #[test]
    fn tip_has_emoji() {
        assert!(render_tip("", "测试", &test_theme()).contains("💡"));
    }
}
