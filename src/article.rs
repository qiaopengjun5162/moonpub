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
    /// Per-article footer variant override; falls back to config `footer.variant`.
    pub footer_variant: Option<String>,
    /// Per-article footer QR code path; falls back to config `footer.qrcode`.
    pub footer_qrcode: Option<String>,
    /// Per-article footer QR code note; falls back to config `footer.qrcode_note`.
    pub footer_qrcode_note: Option<String>,
    /// Per-article cover tag (geek-black style); None = omit the tag line.
    pub cover_tag: Option<String>,
    /// 封面主标题覆盖：公众号标题常带系列前缀且过长（>30 字），直接铺到
    /// 900×500 的封面上会挤成 4-5 行。留空则自动从 title 拆分。
    pub cover_title: Option<String>,
    /// 封面副标题覆盖：一句话说明，建议 ≤28 字。留空则用拆分后的后半句或 digest。
    pub cover_subtitle: Option<String>,
    /// 每篇文章的封面风格覆盖（如 swiss / ai-lab / geek-black）。
    /// ship 会按它重新生成封面，避免用默认 editorial 覆盖手写风格的图。
    pub cover_style: Option<String>,
}

/// Extract the first `# Heading` from the markdown body as a title fallback.
///
/// Fenced code blocks are skipped: a line like `# 1. 第一个提交的规模` inside a
/// ```bash fence is a shell comment, not an H1. Treating it as a title made
/// `preflight` report a bogus `title_h1_match` warning and could feed a code
/// comment into `wechat_title` when frontmatter has no `title`.
pub fn extract_title_from_body(md: &str) -> Option<String> {
    let body = strip_frontmatter(md);
    let mut fence: Option<&'static str> = None;

    for line in body.lines() {
        let trimmed = line.trim_start();
        if let Some(marker) = fence {
            if trimmed.starts_with(marker) {
                fence = None;
            }
            continue;
        }
        if let Some(marker) = fence_marker(trimmed) {
            fence = Some(marker);
            continue;
        }
        if line.starts_with("# ") {
            return Some(line.trim_start_matches('#').trim().to_owned()).filter(|s| !s.is_empty());
        }
    }
    None
}

/// 返回围栏起始标记（``` / ~~~ / `:::`），用于在提取标题时跳过代码块。
fn fence_marker(trimmed: &str) -> Option<&'static str> {
    if trimmed.starts_with("```") {
        Some("```")
    } else if trimmed.starts_with("~~~") {
        Some("~~~")
    } else if trimmed.starts_with(":::") {
        Some(":::")
    } else {
        None
    }
}

/// WeChat article title: explicit `wechat_title` > auto "读《X》笔记" > frontmatter `title` > first `# heading` in body.
pub fn wechat_title(front: &Frontmatter, md: &str) -> String {
    if let Some(t) = &front.wechat_title {
        return t.clone();
    }
    let base = front
        .title
        .as_deref()
        .map(str::to_owned)
        .or_else(|| extract_title_from_body(md))
        .unwrap_or_default();
    if front.author.is_some() && !base.is_empty() {
        format!("读《{}》笔记", base)
    } else {
        base
    }
}

/// Cover title fallback order:
/// frontmatter `cover_title` > `title` > first `# heading` > first meaningful body line > file stem.
pub fn cover_title(front: &Frontmatter, md: &str, article: &Path) -> String {
    front
        .cover_title
        .as_deref()
        .map(str::to_owned)
        .or_else(|| front.title.as_deref().map(str::to_owned))
        .or_else(|| extract_title_from_body(md))
        .or_else(|| {
            let line = first_non_empty_line(strip_frontmatter(md));
            (!line.is_empty()).then(|| line.to_owned())
        })
        .or_else(|| {
            article
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.replace(['-', '_'], " ").trim().to_owned())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_default()
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
            let v = parse_yaml_scalar(v);
            match k {
                "title" => fm.title = Some(v),
                "author" => fm.author = Some(v),
                "digest" | "description" => fm.digest = Some(v),
                "date" => fm.date = Some(v),
                "cover" => fm.cover = Some(v),
                "wechat_title" => fm.wechat_title = Some(v),
                "wechat_author" => fm.wechat_author = Some(v),
                "theme" => fm.theme = Some(v),
                "footer_variant" => fm.footer_variant = Some(v),
                "footer_qrcode" => fm.footer_qrcode = Some(v),
                "footer_qrcode_note" => fm.footer_qrcode_note = Some(v),
                "cover_tag" => fm.cover_tag = Some(v),
                "cover_title" => fm.cover_title = Some(v),
                "cover_subtitle" => fm.cover_subtitle = Some(v),
                "cover_style" => fm.cover_style = Some(v),
                _ => {}
            }
        }
    }
    fm
}

fn parse_yaml_scalar(value: &str) -> String {
    let value = value.trim();
    if let Some(inner) = value.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        return unescape_yaml_double_quoted_scalar(inner);
    }
    if let Some(inner) = value.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) {
        return inner.replace("''", "'");
    }
    value.to_owned()
}

fn unescape_yaml_double_quoted_scalar(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    out
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
    // Only strip a `---` that DIRECTLY precedes a trailing footer block — i.e. the
    // lines immediately after the `---` are footer material (banner image / CTA text)
    // with NO intervening section heading or table. A `---` that sits before
    // `## 小结` / `## 参考链接` is a normal section separator and must NOT be used as
    // a cut point, otherwise everything from that separator to the end of the
    // article (including 小结 / 参考链接 / 系列规划) is silently dropped.
    let lines: Vec<&str> = body.lines().collect();
    for i in (0..lines.len()).rev() {
        if lines[i].trim() == "---" {
            let rest = &lines[i + 1..];
            // Find the first footer marker (banner image or CTA text) after this `---`.
            let footer_marker = rest
                .iter()
                .position(|l| l.contains("mmbiz.qpic.cn") || l.contains("寻月者"));
            let Some(marker_idx) = footer_marker else {
                // No footer marker after this `---` → not a footer delimiter.
                continue;
            };
            // Reject if a section heading or table appears before the footer marker:
            // that means this `---` is a content separator, not a footer delimiter.
            let heading_before_marker = rest[..marker_idx].iter().any(|l| {
                let t = l.trim();
                t.starts_with("# ")
                    || t.starts_with("## ")
                    || t.starts_with("### ")
                    || t.starts_with("#### ")
                    || (t.starts_with('|') && t.ends_with('|'))
            });
            if heading_before_marker {
                continue;
            }
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
    body.trim_end()
}

pub fn first_non_empty_line(text: &str) -> &str {
    text.lines()
        .find(|l| {
            let t = l.trim();
            !t.is_empty()
                && !t.starts_with('#')
                && !t.starts_with(":::") // fence block (:::intro, :::summary, etc.)
                && !t.starts_with("> [!")  // Obsidian callout
                && !t.starts_with("> ") // blockquote continuation
        })
        .unwrap_or("")
        .trim()
}

pub(crate) fn resolve_article_path(articles_dir: &Path, article: &Path) -> PathBuf {
    if article.is_absolute() {
        article.to_path_buf()
    } else {
        articles_dir.join(article)
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
        cover_title, extract_title_from_body, first_non_empty_line, parse_frontmatter,
        parse_yaml_string_array, strip_frontmatter, strip_wechat_footer, wechat_title,
    };
    use std::path::Path;

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
    fn first_non_empty_line_skips_blanks_headings_and_fences() {
        let text = "\n\n# 标题\n\n:::intro\nintro text\n:::\n\n第一段正文\n";
        assert_eq!(first_non_empty_line(text), "intro text");
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
    fn extract_title_from_body_finds_h1() {
        let md = "---\ndate: 2024-01-01\n---\n\n# 从正文提取的标题\n\n正文内容。\n";
        assert_eq!(
            extract_title_from_body(md),
            Some("从正文提取的标题".to_owned())
        );
    }

    #[test]
    fn extract_title_from_body_no_h1_returns_none() {
        let md = "---\ndate: 2024-01-01\n---\n\n正文内容，没有标题行。\n";
        assert_eq!(extract_title_from_body(md), None);
    }

    #[test]
    fn extract_title_from_body_skips_hash_comments_inside_code_fence() {
        // 围栏里的 `# 1. ...` 是 shell 注释，不能当标题。
        let md = "---\ndate: 2024-01-01\n---\n\n```bash\n# 1. 不是标题\ngit show --stat\n```\n\n# 真正的标题\n\n正文。\n";
        assert_eq!(extract_title_from_body(md), Some("真正的标题".to_owned()));
    }

    #[test]
    fn extract_title_from_body_returns_none_when_hash_only_in_code_fence() {
        let md = "---\ndate: 2024-01-01\n---\n\n```bash\n# 只是注释\necho hi\n```\n\n正文。\n";
        assert_eq!(extract_title_from_body(md), None);
    }

    #[test]
    fn wechat_title_falls_back_to_body_h1_when_no_frontmatter_title() {
        let md = "---\ndate: 2024-01-01\n---\n\n# 正文标题\n\n正文内容。\n";
        let front = parse_frontmatter(md);
        assert!(front.title.is_none());
        assert_eq!(wechat_title(&front, md), "正文标题");
    }

    #[test]
    fn wechat_title_prefers_frontmatter_title_over_body_h1() {
        let md = "---\ntitle: 前置标题\n---\n\n# 正文标题\n\n正文。\n";
        let front = parse_frontmatter(md);
        assert_eq!(wechat_title(&front, md), "前置标题");
    }

    #[test]
    fn cover_title_prefers_frontmatter_cover_title_over_title() {
        // `cover` 命令一直单独读 cover_title，而 `cover_title()` 之前只看 title，
        // 于是 ship 会拿长标题去渲染封面。这里锁住两者的优先级一致。
        let md =
            "---\ntitle: 完整标题：一段很长很长的副题\ncover_title: 短标题\n---\n\n# 正文标题\n";
        let front = parse_frontmatter(md);
        assert_eq!(
            cover_title(&front, md, Path::new("Articles/drafts/demo.md")),
            "短标题"
        );

        let without_cover_title = "---\ntitle: 完整标题：一段很长很长的副题\n---\n\n# 正文标题\n";
        let front = parse_frontmatter(without_cover_title);
        assert_eq!(
            cover_title(
                &front,
                without_cover_title,
                Path::new("Articles/drafts/demo.md")
            ),
            "完整标题：一段很长很长的副题"
        );
    }

    #[test]
    fn cover_title_falls_back_to_first_meaningful_body_line() {
        let md = "---\ndate: 2024-01-01\n---\n\n这是第一段正文。\n\n后面还有内容。\n";
        let front = parse_frontmatter(md);

        assert_eq!(
            cover_title(&front, md, Path::new("Articles/drafts/demo.md")),
            "这是第一段正文。"
        );
    }

    #[test]
    fn cover_title_falls_back_to_normalized_file_stem() {
        let md = "---\ndate: 2024-01-01\n---\n\n";
        let front = parse_frontmatter(md);

        assert_eq!(
            cover_title(&front, md, Path::new("Articles/drafts/my-first-note.md")),
            "my first note"
        );
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

    #[test]
    fn frontmatter_cover_style_is_parsed_independently_of_theme() {
        // 正文主题和封面风格可以不同：theme 管排版，cover_style 管封面。
        let front = parse_frontmatter(
            "---\ntitle: 测试\ntheme: notebook\ncover_style: swiss\n---\n\n正文。\n",
        );

        assert_eq!(front.theme.as_deref(), Some("notebook"));
        assert_eq!(front.cover_style.as_deref(), Some("swiss"));
    }

    #[test]
    fn frontmatter_footer_overrides_are_parsed() {
        let front = parse_frontmatter(
            "---\nfooter_variant: community\nfooter_qrcode: Context/assets/group.png\nfooter_qrcode_note: 先关注公众号，再扫码入群\n---\n\n正文\n",
        );

        assert_eq!(front.footer_variant.as_deref(), Some("community"));
        assert_eq!(
            front.footer_qrcode.as_deref(),
            Some("Context/assets/group.png")
        );
        assert_eq!(
            front.footer_qrcode_note.as_deref(),
            Some("先关注公众号，再扫码入群")
        );
    }

    #[test]
    fn frontmatter_double_quoted_scalars_unescape_common_sequences() {
        let front = parse_frontmatter(
            "---\ntitle: \"标题: 带冒号\"\nfooter_qrcode_note: \"请先关注公众号\\n再扫码入群\"\n---\n\n正文\n",
        );

        assert_eq!(front.title.as_deref(), Some("标题: 带冒号"));
        assert_eq!(
            front.footer_qrcode_note.as_deref(),
            Some("请先关注公众号\n再扫码入群")
        );
    }

    #[test]
    fn strip_wechat_footer_does_not_cut_at_section_separator_before_footer() {
        // Regression for the 2026-09-14 article 12 bug: a `---` section separator
        // before `## 小结` followed (later in the document) by a trailing manual
        // footer containing 「寻月者」 must NOT truncate 小结 / 参考链接 / 系列规划.
        let body = "\
## 正文

内容。

---

## 小结

这是小结。

## 参考链接

### 系列规划

1. 第 1 篇

加入「寻月阁」

点个「推荐」让更多「寻月者」看到。
";
        let stripped = strip_wechat_footer(body);
        assert!(stripped.contains("## 小结"), "小结 被误删");
        assert!(stripped.contains("这是小结"), "小结内容被误删");
        assert!(stripped.contains("## 参考链接"), "参考链接 被误删");
        assert!(stripped.contains("### 系列规划"), "系列规划 被误删");
    }

    #[test]
    fn strip_wechat_footer_still_strips_trailing_footer_block() {
        // A `---` that DIRECTLY precedes a footer block (no heading/table between)
        // should still be stripped so the auto footer isn't duplicated.
        let body = "\
## 正文

内容。

---

加入「寻月阁」

点个「推荐」让更多「寻月者」看到。
";
        let stripped = strip_wechat_footer(body);
        assert!(!stripped.contains("寻月者"), "尾部手动 footer 未被剥离");
        assert!(stripped.contains("## 正文"), "正文被误删");
    }
}
