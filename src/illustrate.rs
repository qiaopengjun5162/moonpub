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
    // 卡片里的字来自作者的 `:::` 块，同样是正文：不转义的话一句 `<div>`
    // 就能在成品里开出真标签，把后面的内容一起吞掉。
    let text = esc(text);
    let source = esc(source);
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
    let label = esc(label);
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
    let title = esc(title);
    let desc = esc(desc);
    let colors = [theme.accent, "#e65100", "#1565c0", "#2e7d32", "#6a1b9a"];
    let accent = colors[(number as usize).saturating_sub(1).min(colors.len() - 1)];
    format!(
        "<section style=\"margin:18px 0;background:{};border:1px solid #e8e8e8;border-left:4px solid {accent};padding:16px 18px;border-radius:8px;\">\n<span style=\"display:inline-block;width:28px;height:28px;background:{accent};color:#fff;font-weight:bold;text-align:center;line-height:28px;border-radius:50%;font-size:13px;margin:0 8px 8px 0;\">{number}</span>\n<p style=\"margin:0 0 4px;font-size:15px;font-weight:bold;color:{};\">{title}</p>\n<p style=\"margin:0;font-size:13px;color:{};line-height:1.75;\">{desc}</p>\n</section>\n\n",
        theme.block_bg, theme.heading_color, theme.text_muted
    )
}

fn render_emotion_card(mood: &str, text: &str, theme: &Theme) -> String {
    let text = esc(text);
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
    let _ = lang; // 不再显示语言标签，代码块顶部用 macOS 窗口圆点装饰
    // 微信兼容：不用 <pre>（换行会被剥掉），每行一个 <p>。
    // 折行策略用 overflow-wrap:break-word 而不是 break-all：break-all 会把
    // 单词从中间剁开（实测把 "pure Rust" 断成 "pure R" / "ust"），
    // break-word 只在整词放不下时才断。
    //
    // **不能用 `white-space:pre-wrap`**。换行由结构负责（一行一个 `<p>`），
    // 空格与缩进由 `&nbsp;` 负责（见 `xcode_highlight` 的空格分支：**每个**
    // 空格都是字符，不留裸空格，因为后台会删掉只含空白的文本节点，跨元素的
    // 空格会整段消失、词粘成一串），排版一点也不依赖 `white-space`。
    // 反过来，`pre-wrap` 会把 HTML 源码里的
    // 每个换行都当成真实断行——公众号后台的编辑器会重新序列化我们贴进去的
    // HTML（在元素之间插换行），于是一行 `const opts: any = {...}` 到了后台
    // 变成一个 token 一行，整块代码竖着倒下来。同理，制表符必须在这里换成
    // 硬空格：`white-space` 不再替我们保留它。
    let palette = choose_code_palette(theme);
    let mut body = String::new();
    for raw in code.split('\n') {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if line.is_empty() {
            body.push_str("<p style=\"margin:0;\">&nbsp;</p>");
            continue;
        }
        let line = line.replace('\t', "    ");
        let hl = xcode_highlight(&line, &palette);
        body.push_str(&format!(
            "<p style=\"margin:0;word-break:normal;overflow-wrap:break-word;font-family:Menlo,Consolas,monospace;color:{};\">{hl}</p>",
            palette.fg
        ));
    }
    format!(
        "<section style=\"margin:20px 0;border-radius:10px;overflow:hidden;background:{};border:1px solid {};\">\n<section style=\"background:{};padding:8px 14px;\">\n<span style=\"display:inline-block;width:12px;height:12px;border-radius:50%;background:#ff5f57;margin-right:8px;\">&nbsp;</span>\n<span style=\"display:inline-block;width:12px;height:12px;border-radius:50%;background:#febc2e;margin-right:8px;\">&nbsp;</span>\n<span style=\"display:inline-block;width:12px;height:12px;border-radius:50%;background:#28c840;\">&nbsp;</span>\n</section>\n<section style=\"padding:16px;\">\n{body}\n</section></section>\n\n",
        palette.bg, palette.border, palette.header_bg
    )
}

struct Palette {
    bg: &'static str,
    header_bg: &'static str,
    border: &'static str,
    fg: &'static str,
    keyword: &'static str,
    string: &'static str,
    number: &'static str,
    func: &'static str,
    type_c: &'static str,
    comment: &'static str,
}

fn choose_code_palette(theme: &Theme) -> Palette {
    if theme.is_dark() {
        // 深色主题：沿用 Xcode Dark，与深色正文融为一体。
        Palette {
            bg: XCODE_BG,
            header_bg: "#2c2c2e",
            border: "#2c2c2e",
            fg: XCODE_FG,
            keyword: XCODE_KEYWORD,
            string: XCODE_STRING,
            number: XCODE_NUMBER,
            func: XCODE_FUNC,
            type_c: XCODE_TYPE,
            comment: XCODE_COMMENT,
        }
    } else {
        // 浅色主题：改用 GitHub Light 风格的浅色代码块，
        // 避免浅色正文里出现突兀的黑色终端块。
        Palette {
            bg: LIGHT_CODE_BG,
            header_bg: LIGHT_CODE_HEADER,
            border: LIGHT_CODE_BORDER,
            fg: LIGHT_CODE_FG,
            keyword: LIGHT_CODE_KEYWORD,
            string: LIGHT_CODE_STRING,
            number: LIGHT_CODE_NUMBER,
            func: LIGHT_CODE_FUNC,
            type_c: LIGHT_CODE_TYPE,
            comment: LIGHT_CODE_COMMENT,
        }
    }
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

// ── GitHub Light 语法高亮（浅色主题的代码块） ──────────────────────────
const LIGHT_CODE_BG: &str = "#f6f8fa";
const LIGHT_CODE_HEADER: &str = "#eaeef2";
const LIGHT_CODE_BORDER: &str = "#d0d7de";
const LIGHT_CODE_FG: &str = "#24292f";
const LIGHT_CODE_COMMENT: &str = "#6e7781";
const LIGHT_CODE_KEYWORD: &str = "#cf222e";
const LIGHT_CODE_STRING: &str = "#0a3069";
const LIGHT_CODE_NUMBER: &str = "#0550ae";
const LIGHT_CODE_FUNC: &str = "#8250df";
const LIGHT_CODE_TYPE: &str = "#953800";

fn esc(s: &str) -> String {
    crate::html::escape(s)
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

fn xcode_highlight(code: &str, palette: &Palette) -> String {
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
                "<span style=\"color:{};\">{}</span>",
                palette.comment,
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
                "<span style=\"color:{};\">{}</span>",
                palette.comment,
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
                "<span style=\"color:{};\">{}</span>",
                palette.string,
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
                "<span style=\"color:{};\">{}</span>",
                palette.number,
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
                palette.keyword
            } else if word
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_uppercase())
            {
                palette.type_c
            } else if is_call {
                palette.func
            } else {
                palette.fg
            };
            out.push_str(&format!(
                "<span style=\"color:{};\">{}</span>",
                color,
                esc(&word)
            ));
            i = j;
            continue;
        }
        // 空格：一个不留地写成 `&nbsp;`，行首缩进和词间间隔同一套。
        //
        // 公众号后台会把「只含空白字符的文本节点」整段删掉。2026-11-18 的
        // 线上文章实测过：我们输出
        //   <span>npx</span> <span>tsx</span> <span>simple</span>
        // 后台存下来变成
        //   <span>npx</span><span>tsx</span><span>simple</span>
        // 三个词粘成 `npxtsxsimple`，整行命令连成一片。
        // 同一个文本节点内部的空格不会被删（正文里 `课程的索引器 demo`
        // 完好），只有「跨元素」的空格会消失；而代码高亮恰好把每个 token
        // 都包成独立 `<span>`，于是每一处词间空格都是跨元素的。
        // 结论：空格必须是**字符**，不能是排版。
        //
        // 代价是整行不再有软换行点，超宽行只能由 `overflow-wrap:break-word`
        // 从容器边缘折断（不断就会溢出后被 `overflow:hidden` 裁掉）。
        if c == ' ' {
            let mut j = i;
            while j < n && chars[j] == ' ' {
                j += 1;
            }
            for _ in 0..(j - i) {
                out.push_str("&nbsp;");
            }
            i = j;
            continue;
        }
        // 标点也要转义。代码块里讲 HTML 时 `<div>` 是正文，原样写进 HTML
        // 就成了真标签——浏览器把它吞掉，读者看到的是「代码少了半行」；
        // `</T>` 更狠，会被解析成结束标签把后面的内容一起吃掉。
        // 上面每个 token 分支都走 `esc()`，只有这个兜底分支漏了。
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
        i += 1;
    }
    out
}

pub fn render_timeline(items: &[(String, String)], theme: &Theme) -> String {
    let mut html = String::from("<section style=\"margin:24px 0;padding:0 8px;\">\n");
    for (i, (date, desc)) in items.iter().enumerate() {
        let (date, desc) = (esc(date), esc(desc));
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
            esc(left_title),
            theme.text_color,
            esc(left),
            theme.heading_border,
            esc(right_title),
            theme.text_color,
            esc(right)
        ));
    }
    html.push_str("</section>\n\n");
    html
}

pub fn render_tip(icon: &str, text: &str, theme: &Theme) -> String {
    let emoji = if icon.is_empty() { "💡" } else { icon };
    let emoji = esc(emoji);
    let text = esc(text);
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
        let html = render_code_block("rust", "fn main()", &test_theme());
        assert!(html.contains("#ff5f57"));
        assert!(!html.contains(">rust<"));
    }

    /// 词间空格必须是**字符**，不能是排版。
    ///
    /// 公众号后台会删掉「只含空白的文本节点」：我们输出
    /// `<span>npx</span> <span>tsx</span>`，成品里那个空格根本不存在，
    /// 于是命令粘成 `npxtsx`（2026-11-18 线上文章实测）。这里把后台的
    /// 删法模拟一遍：删掉所有只含空白的文本节点后，`pure Rust` 仍要
    /// 读得出来，缩进也不能塌。
    #[test]
    fn code_block_spaces_survive_whitespace_stripping() {
        let html = render_code_block("text", "    pure Rust", &test_theme());

        assert!(
            !html.contains("</span> <span"),
            "词间又留裸空格了，后台会把整段删掉：{html}"
        );

        let text = visible_text(&drop_blank_text_nodes(&html));
        assert!(
            text.contains("    pure Rust"),
            "空白节点被删后代码读不出来了：{text}"
        );
    }

    /// 代码块里的 HTML 示例必须原样露出来。
    ///
    /// `<div>` 原样写进 HTML 就成了真标签，浏览器吞掉它，读者看到的是
    /// 「代码少了一截」；`</T>` 会被当成结束标签把后面内容一起吃掉。
    #[test]
    fn code_block_escapes_markup_so_examples_survive() {
        let html = render_code_block("html", "<div class=\"x\">hi</div>", &test_theme());

        assert!(
            !html.contains("<div"),
            "代码里的 <div> 漏成了真标签：{html}"
        );
        assert!(
            !html.contains("</div>"),
            "代码里的 </div> 漏成了真标签：{html}"
        );
        assert!(html.contains("&lt;"), "{html}");
        assert!(html.contains("&gt;"), "{html}");
    }

    /// 模拟公众号后台：删掉只含空白的文本节点，保留文本节点内部的空格。
    fn drop_blank_text_nodes(html: &str) -> String {
        let mut out = String::new();
        let mut in_tag = false;
        let mut pending_ws = String::new();
        for ch in html.chars() {
            if in_tag {
                out.push(ch);
                if ch == '>' {
                    in_tag = false;
                }
            } else if ch == '<' {
                pending_ws.clear();
                out.push(ch);
                in_tag = true;
            } else if ch.is_whitespace() {
                pending_ws.push(ch);
            } else {
                out.push_str(&pending_ws);
                pending_ws.clear();
                out.push(ch);
            }
        }
        out
    }

    fn visible_text(html: &str) -> String {
        let mut out = String::new();
        let mut in_tag = false;
        for ch in html.chars() {
            if in_tag {
                if ch == '>' {
                    in_tag = false;
                }
            } else if ch == '<' {
                in_tag = true;
            } else {
                out.push(ch);
            }
        }
        out.replace("&nbsp;", " ")
    }

    /// 代码行的排版不许依赖 `white-space`。
    ///
    /// 换行由「一行一个 `<p>`」负责、缩进由 `&nbsp;` 负责，一旦再叠一层
    /// `white-space:pre-wrap`，任何重新序列化 HTML 的上游（公众号后台的
    /// 编辑器会在元素之间插换行）都会把那些换行当成真实断行——一行
    /// `const opts: any = {limit: 10};` 到了后台变成一个 token 一行。
    /// 缩进也因此必须全部落成 `&nbsp;`，不能再指望 `pre-wrap` 替我们留空格。
    #[test]
    fn code_lines_do_not_depend_on_white_space() {
        let html = render_code_block(
            "ts",
            "const a = 1;\n    if (a) {\n        deep();\n    }",
            &test_theme(),
        );

        assert!(
            !html.contains("white-space"),
            "代码行又依赖 white-space 了：{html}"
        );
        // 4 空格 / 8 空格缩进全部落成 &nbsp;，剥掉 white-space 也不会塌
        assert!(html.contains("&nbsp;&nbsp;&nbsp;&nbsp;<span"), "{html}");
        assert!(
            html.contains("&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;<span"),
            "{html}"
        );
    }

    /// `white-space` 不在了，制表符就得自己换成硬空格：否则缩进直接消失。
    #[test]
    fn tabs_become_hard_indentation() {
        let html = render_code_block("ts", "\tif (x) {\n\t\treturn 1;", &test_theme());

        assert!(
            !html.contains('\t'),
            "制表符漏进成品（微信端会被吞掉）：{html}"
        );
        assert!(html.contains("&nbsp;&nbsp;&nbsp;&nbsp;<span"), "{html}");
        assert!(
            html.contains("&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;<span"),
            "{html}"
        );
    }

    #[test]
    fn code_block_uses_light_palette_for_light_theme() {
        let html = render_code_block("rust", "println!(\"hi\")", &test_theme());

        // 浅色主题（default/notebook 等）用 GitHub Light 风格的浅色代码块，
        // 不再在浅色正文里塞一块突兀的黑色终端。
        assert!(html.contains(LIGHT_CODE_BG));
        assert!(html.contains(LIGHT_CODE_STRING));
        assert!(!html.contains(XCODE_BG));
        // 微信兼容：不用 <pre>（换行会被剥），改用每行 <p>
        assert!(!html.contains("<pre"));
        assert!(html.contains("&quot;hi&quot;"));
    }

    #[test]
    fn code_block_keeps_xcode_palette_for_dark_themes() {
        let dark = crate::theme::Theme::from_name("dark");
        let html = render_code_block("rust", "println!(\"hi\")", &dark);

        // 暗色主题保留 Xcode Dark 配色
        assert!(html.contains("#1e1e1e"));
        assert!(html.contains("#ff8170"));
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
