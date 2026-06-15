use std::path::{Path, PathBuf};

use crate::error::AppError;

#[derive(Debug, Default)]
pub struct Frontmatter {
    pub title: Option<String>,
    /// Book/source author from weread import — used to detect book notes and auto-format title.
    pub author: Option<String>,
    pub digest: Option<String>,
    pub date: Option<String>,
    pub tags: Vec<String>,
    pub cover: Option<String>,
    /// Explicit WeChat article title override (bypasses auto-formatting).
    pub wechat_title: Option<String>,
    /// Per-article WeChat author override; falls back to config `wechat.author`.
    pub wechat_author: Option<String>,
    /// Per-article theme override; falls back to config `wechat.theme`.
    pub theme: Option<String>,
}

/// WeChat article title: explicit `wechat_title` > auto "读《X》笔记" for book notes > raw title.
pub fn wechat_title(front: &Frontmatter) -> String {
    if let Some(t) = &front.wechat_title {
        return t.clone();
    }
    let base = front.title.as_deref().unwrap_or("").to_owned();
    if front.author.is_some() && !base.is_empty() {
        format!("读《{}》笔记", base)
    } else {
        base
    }
}

pub(crate) fn parse_frontmatter(md: &str) -> Frontmatter {
    let mut fm = Frontmatter::default();
    let body = md.trim_start();
    if !body.starts_with("---") {
        return fm;
    }
    let rest = &body[3..];
    let end = rest.find("\n---").unwrap_or(rest.len());
    for line in rest[..end].lines() {
        let line = line.trim();
        // tags: ["a", "b", "c"]
        if line.starts_with("tags:") {
            fm.tags = parse_yaml_string_array(line);
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            let k = k.trim();
            let v = v.trim().trim_matches('"');
            match k {
                "title" => fm.title = Some(v.to_owned()),
                "author" => fm.author = Some(v.to_owned()),
                "digest" | "description" => fm.digest = Some(v.to_owned()),
                "date" => fm.date = Some(v.to_owned()),
                "cover" => fm.cover = Some(v.to_owned()),
                "wechat_title" => fm.wechat_title = Some(v.to_owned()),
                "wechat_author" => fm.wechat_author = Some(v.to_owned()),
                "theme" => fm.theme = Some(v.to_owned()),
                _ => {}
            }
        }
    }
    fm
}

/// Parse `tags: ["a", "b"]` or `tags: [a, b]` into a Vec<String>.
pub fn parse_yaml_string_array(line: &str) -> Vec<String> {
    let Some(bracket_start) = line.find('[') else {
        return vec![];
    };
    let Some(bracket_end) = line.rfind(']') else {
        return vec![];
    };
    let inner = &line[bracket_start + 1..bracket_end];
    inner
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_owned())
        .filter(|s| !s.is_empty())
        .collect()
}

pub(crate) fn strip_frontmatter(md: &str) -> &str {
    let body = md.trim_start();
    if !body.starts_with("---") {
        return md;
    }
    let rest = &body[3..];
    if let Some(pos) = rest.find("\n---") {
        rest[pos + 4..].trim_start()
    } else {
        md
    }
}

/// Strip the standard banner+CTA footer that some articles already have in their Markdown.
pub fn strip_wechat_footer(body: &str) -> &str {
    // The footer always starts with a standalone `---` followed by a banner image line.
    // Walk backwards to find the last `---` that precedes a banner image URL.
    let lines: Vec<&str> = body.lines().collect();
    for i in (0..lines.len()).rev() {
        if lines[i].trim() == "---" {
            // Check if any line after this is the banner image
            let rest = &lines[i + 1..];
            if rest
                .iter()
                .any(|l| l.contains("mmbiz.qpic.cn") || l.contains("寻月者"))
            {
                // trim back to just before this `---`
                let cut = lines[..i]
                    .iter()
                    .rfind(|l| !l.trim().is_empty())
                    .map(|last| {
                        let pos = body.rfind(last).unwrap_or(body.len());
                        pos + last.len()
                    })
                    .unwrap_or(body.len());
                return body[..cut].trim_end();
            }
        }
    }
    body.trim_end()
}

pub fn first_non_empty_line(text: &str) -> &str {
    text.lines()
        .find(|l| {
            let t = l.trim();
            !t.is_empty()
                && !t.starts_with('#')
                && !t.starts_with("> [!")  // Obsidian callout
                && !t.starts_with("> ") // blockquote continuation
        })
        .unwrap_or("")
        .trim()
}

pub(crate) fn resolve_article_path(vault: &Path, article: &Path) -> PathBuf {
    if article.is_absolute() {
        article.to_path_buf()
    } else {
        vault.join(article)
    }
}

pub fn article_slug(article: &Path) -> Result<String, AppError> {
    article
        .file_stem()
        .and_then(|s| s.to_str())
        .map(str::to_owned)
        .ok_or_else(|| AppError::InvalidArticlePath(article.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use crate::article::{
        first_non_empty_line, parse_frontmatter, parse_yaml_string_array, strip_frontmatter,
    };

    #[test]
    fn strip_frontmatter_removes_yaml_block() {
        let md = "---\ntitle: T\ndate: 2024-01-01\n---\n\n正文内容\n";
        assert_eq!(strip_frontmatter(md), "正文内容\n");
    }

    #[test]
    fn strip_frontmatter_no_frontmatter_passthrough() {
        let md = "# 标题\n\n正文\n";
        assert_eq!(strip_frontmatter(md), md);
    }

    #[test]
    fn strip_frontmatter_unclosed_returns_original() {
        let md = "---\ntitle: T\n\n正文\n";
        assert_eq!(strip_frontmatter(md), md);
    }

    #[test]
    fn first_non_empty_line_skips_blanks_and_headings() {
        let text = "\n\n# 标题\n\n第一段正文\n";
        assert_eq!(first_non_empty_line(text), "第一段正文");
    }

    #[test]
    fn first_non_empty_line_empty_input() {
        assert_eq!(first_non_empty_line(""), "");
    }

    #[test]
    fn first_non_empty_line_only_headings() {
        assert_eq!(first_non_empty_line("# H1\n## H2\n"), "");
    }

    #[test]
    fn parse_yaml_string_array_quoted() {
        let tags = parse_yaml_string_array(r#"tags: ["Rust", "编程", "工具"]"#);
        assert_eq!(tags, vec!["Rust", "编程", "工具"]);
    }

    #[test]
    fn parse_yaml_string_array_unquoted() {
        let tags = parse_yaml_string_array("tags: [Rust, 编程, 工具]");
        assert_eq!(tags, vec!["Rust", "编程", "工具"]);
    }

    #[test]
    fn parse_yaml_string_array_empty_brackets() {
        let tags = parse_yaml_string_array("tags: []");
        assert!(tags.is_empty());
    }

    #[test]
    fn parse_yaml_string_array_no_bracket() {
        let tags = parse_yaml_string_array("tags: Rust");
        assert!(tags.is_empty());
    }

    #[test]
    fn frontmatter_cover_field_is_parsed() {
        let md = "---\ntitle: 测试\ncover: ./my-cover.jpg\n---\n\n正文。\n";
        let front = parse_frontmatter(md);
        assert_eq!(front.cover.as_deref(), Some("./my-cover.jpg"));
    }
}
