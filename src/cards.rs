//! Generate Xiaohongshu-style vertical knowledge cards from a MoonPub article.
//!
//! This is the inheritable core distilled from the "Punk XHS Cards" methodology:
//! a long article is split into a cover + a sequence of inner cards, plus a
//! platform-agnostic `publish-copy` (title + alt titles + body + tags).
//!
//! It deliberately does NOT generate any imagery — no AI image generation — so it
//! keeps image generation out of the publish prerequisites (per project policy).

use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::article::{
    article_slug, cover_title, parse_frontmatter, resolve_article_path, strip_frontmatter,
};
use crate::config::Config;
use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct CardsResult {
    pub article: String,
    pub title: String,
    pub theme: Option<String>,
    pub accent_color: String,
    pub card_count: usize,
    pub card_plan_path: String,
    pub publish_copy_path: String,
}

/// Map a MoonPub `theme` to a Xiaohongshu card accent color.
/// Mirrors each theme's visual identity without requiring image generation.
pub fn accent_color_for(theme: Option<&str>) -> &'static str {
    match theme {
        Some("geek-black") => "#22C55E",
        Some("cyber") => "#A855F7",
        Some("blueprint") => "#2563EB",
        Some("gradient") => "#EC4899",
        Some("newsletter") | Some("magazine") => "#B45309",
        Some("ink") | Some("serif") => "#1F2937",
        Some("minimal") | Some("porcelain") | Some("moonlit") => "#0F172A",
        Some("monokai") => "#5F8C1F",
        Some("dracula") => "#7C4DFF",
        Some("nord") => "#4C8A9E",
        Some("one-dark") => "#3B7DD8",
        Some("gruvbox") => "#C2410C",
        _ => "#2563EB",
    }
}

/// Extract the inner text of a `:::name ... :::` fenced block (if present).
fn extract_fence_block(body: &str, name: &str) -> Option<String> {
    let marker = format!(":::{name}");
    let mut capture = false;
    let mut out: Vec<&str> = Vec::new();
    for raw in body.lines() {
        let line = raw.trim();
        if line == marker || line.starts_with(&format!("{marker} ")) {
            capture = true;
            continue;
        }
        if capture && line == ":::" {
            break;
        }
        if capture {
            out.push(raw);
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out.join("\n").trim().to_owned())
    }
}

/// Remove the given `:::name ... :::` fenced blocks (including inner lines).
fn remove_fence_blocks(body: &str, names: &[&str]) -> String {
    let mut out: Vec<&str> = Vec::new();
    let mut skipping = false;
    for raw in body.lines() {
        let line = raw.trim();
        if line.starts_with(":::") {
            let is_target = names
                .iter()
                .any(|n| line == format!(":::{n}") || line.starts_with(&format!(":::{n} ")));
            if is_target {
                skipping = true;
                continue;
            }
            if skipping && line == ":::" {
                skipping = false;
                continue;
            }
        }
        if !skipping {
            out.push(raw);
        }
    }
    out.join("\n")
}

/// Split article body into inner-card sections. Each `## ` heading starts a new card.
fn split_into_sections(body: &str) -> Vec<(String, String)> {
    let mut sections: Vec<(String, String)> = Vec::new();
    let mut title: Option<String> = None;
    let mut buf: Vec<String> = Vec::new();

    for raw in body.lines() {
        let line = raw.trim_end();
        if let Some(rest) = line.strip_prefix("## ") {
            flush_section(&mut title, &mut buf, &mut sections);
            title = Some(rest.trim().to_owned());
            continue;
        }
        if line.starts_with("# ") {
            continue; // H1 is the article title (cover already carries it)
        }
        if line.starts_with(":::") {
            continue; // fence markers already stripped upstream
        }
        if is_directive_meta(line) {
            continue; // drop MoonPub block metadata (label:/number:/适合谁：…)
        }
        buf.push(line.to_owned());
    }
    flush_section(&mut title, &mut buf, &mut sections);

    if sections.is_empty() {
        let content = body.trim().to_owned();
        if !content.is_empty() {
            sections.push(("正文".to_owned(), content));
        }
    }
    sections
}

/// Drop MoonPub block metadata lines that are not real prose (shortcode directives).
fn is_directive_meta(line: &str) -> bool {
    let l = line.trim_start();
    l.starts_with("label:")
        || l.starts_with("number:")
        || l.starts_with("适合谁")
        || l.starts_with("type:")
        || l.starts_with("theme:")
        || l.starts_with("color:")
}

fn flush_section(
    title: &mut Option<String>,
    buf: &mut Vec<String>,
    sections: &mut Vec<(String, String)>,
) {
    let content = buf.join("\n").trim().to_owned();
    if content.is_empty() {
        return;
    }
    let heading = title.take().unwrap_or_else(|| "正文".to_owned());
    sections.push((heading, content));
    buf.clear();
}

/// Generate `card-plan.md` + `publish-copy.md` next to the article.
pub fn generate_cards(
    articles_dir: &Path,
    cfg: &Config,
    article: &Path,
) -> Result<CardsResult, AppError> {
    let article_path = resolve_article_path(articles_dir, article);
    let md = fs::read_to_string(&article_path).map_err(|source| AppError::Io {
        path: article_path.clone(),
        source,
    })?;
    let front = parse_frontmatter(&md);
    let body = strip_frontmatter(&md);

    let title = cover_title(&front, &md, &article_path);
    let theme = front.theme.clone();
    let accent = accent_color_for(theme.as_deref()).to_owned();
    let author = cfg
        .wechat_author
        .clone()
        .or(front.wechat_author.clone())
        .or(front.author.clone())
        .unwrap_or_else(|| "寻月阁".to_owned());

    let intro = extract_fence_block(body, "intro");
    let summary = extract_fence_block(body, "summary");
    let subtitle = intro
        .as_deref()
        .and_then(|t| t.lines().find(|l| !l.trim().is_empty()))
        .unwrap_or("")
        .to_owned();

    let split_body = remove_fence_blocks(body, &["intro", "summary"]);
    let sections = split_into_sections(&split_body);

    let slug = article_slug(&article_path)?;
    let out_dir = article_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| articles_dir.to_path_buf());
    let card_plan_path = out_dir.join(format!("{slug}.card-plan.md"));
    let publish_copy_path = out_dir.join(format!("{slug}.publish-copy.md"));

    let card_plan = build_card_plan(&title, &author, &accent, &theme, &subtitle, &sections);
    let publish_copy = build_publish_copy(&title, &summary, &intro, &sections, &front.tags);

    fs::write(&card_plan_path, card_plan).map_err(|source| AppError::Io {
        path: card_plan_path.clone(),
        source,
    })?;
    fs::write(&publish_copy_path, publish_copy).map_err(|source| AppError::Io {
        path: publish_copy_path.clone(),
        source,
    })?;

    Ok(CardsResult {
        article: article_path.display().to_string(),
        title,
        theme,
        accent_color: accent,
        card_count: sections.len(),
        card_plan_path: card_plan_path.display().to_string(),
        publish_copy_path: publish_copy_path.display().to_string(),
    })
}

fn build_card_plan(
    title: &str,
    author: &str,
    accent: &str,
    theme: &Option<String>,
    subtitle: &str,
    sections: &[(String, String)],
) -> String {
    let mut s = String::new();
    s.push_str(&format!("# 卡片切分计划 · {title}\n\n"));
    s.push_str("> 由 `moonpub cards` 生成（Punk XHS Cards 方法论：长文 → 竖版卡片）\n");
    s.push_str(&format!(
        "> 作者/IP：{author} · 主题色：{accent}（theme: {}）\n\n",
        theme.as_deref().unwrap_or("default")
    ));

    s.push_str("## 封面（无页码）\n");
    s.push_str(&format!("- 标题：{title}\n"));
    s.push_str(&format!("- 作者：{author}\n"));
    s.push_str(&format!("- 主色：{accent}\n"));
    if !subtitle.is_empty() {
        s.push_str(&format!("- 副信息：{subtitle}\n"));
    }
    s.push('\n');

    s.push_str("## 内页（从 01 编号）\n");
    for (i, (heading, content)) in sections.iter().enumerate() {
        s.push_str(&format!("**{:02} · {heading}**\n", i + 1));
        s.push_str(content);
        s.push_str("\n\n");
    }
    s
}

fn build_publish_copy(
    title: &str,
    summary: &Option<String>,
    intro: &Option<String>,
    sections: &[(String, String)],
    tags: &[String],
) -> String {
    let mut s = String::new();
    s.push_str(&format!("# 发布文案 · {title}\n\n"));
    s.push_str("> Punk XHS Cards 的 `publish-copy.md` 结构：1 主标题 + 3 备选 + 1 正文 + 标签\n\n");

    s.push_str("## 推荐标题\n");
    s.push_str(&format!("{title}\n\n"));

    s.push_str("## 备选标题\n");
    s.push_str(&format!("1. {title}｜一篇讲透\n"));
    s.push_str(&format!("2. 别再踩坑：{title}\n"));
    s.push_str(&format!("3. {title}（建议收藏）\n\n"));

    s.push_str("## 正文（可直接发布）\n");
    let body = if let Some(summary) = summary {
        summary.clone()
    } else if let Some(intro) = intro {
        let mut b = intro.clone();
        if let Some((_, first)) = sections.first() {
            b.push_str("\n\n");
            b.push_str(first);
        }
        b
    } else if let Some((_, first)) = sections.first() {
        first.clone()
    } else {
        String::new()
    };
    s.push_str(&body);
    s.push_str("\n\n");

    s.push_str("## 话题标签\n");
    if tags.is_empty() {
        s.push_str("#内容创作 #工具推荐 #公众号运营\n");
    } else {
        let joined = tags
            .iter()
            .map(|t| format!("#{t}"))
            .collect::<Vec<_>>()
            .join(" ");
        s.push_str(&joined);
        s.push('\n');
    }
    s
}

/// Human-readable summary for non-JSON output.
pub fn cards_text(result: &CardsResult) -> String {
    format!(
        "cards generated\n  title:    {}\n  accent:   {}\n  cards:    {}\n  plan:     {}\n  copy:     {}",
        result.title,
        result.accent_color,
        result.card_count,
        result.card_plan_path,
        result.publish_copy_path,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accent_color_falls_back_to_blueprint_blue() {
        assert_eq!(accent_color_for(None), "#2563EB");
        assert_eq!(accent_color_for(Some("blueprint")), "#2563EB");
        assert_eq!(accent_color_for(Some("geek-black")), "#22C55E");
        assert_eq!(accent_color_for(Some("unknown-theme")), "#2563EB");
    }

    #[test]
    fn accent_color_covers_editor_scheme_themes() {
        assert_eq!(accent_color_for(Some("monokai")), "#5F8C1F");
        assert_eq!(accent_color_for(Some("dracula")), "#7C4DFF");
        assert_eq!(accent_color_for(Some("nord")), "#4C8A9E");
        assert_eq!(accent_color_for(Some("one-dark")), "#3B7DD8");
        assert_eq!(accent_color_for(Some("gruvbox")), "#C2410C");
    }

    #[test]
    fn split_into_sections_uses_h2_headings() {
        let body =
            "# 标题\n\n:::intro\n引言内容\n:::\n\n## 第一节\n第一节正文\n\n## 第二节\n第二节正文\n";
        let stripped = remove_fence_blocks(body, &["intro", "summary"]);
        let sections = split_into_sections(&stripped);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].0, "第一节");
        assert!(sections[0].1.contains("第一节正文"));
        assert_eq!(sections[1].0, "第二节");
    }

    #[test]
    fn split_into_sections_falls_back_to_single_card() {
        let body = "只有一段正文，没有小标题。\n";
        let sections = split_into_sections(body);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].0, "正文");
    }

    #[test]
    fn extract_fence_block_reads_inner_text() {
        let body = "前言\n\n:::summary\n这是摘要。\n多行。\n:::\n\n后文";
        assert_eq!(
            extract_fence_block(body, "summary"),
            Some("这是摘要。\n多行。".to_owned())
        );
    }

    use crate::config::Config;
    use std::fs;
    use std::path::Path;

    #[test]
    fn build_card_plan_numbers_inner_sections_and_renders_meta() {
        let sections = vec![
            ("第一节".to_string(), "第一节正文".to_string()),
            ("第二节".to_string(), "第二节正文".to_string()),
        ];
        let plan = build_card_plan(
            "示例标题",
            "寻月阁",
            "#2563EB",
            &Some("blueprint".to_string()),
            "引言一句",
            &sections,
        );
        assert!(plan.contains("## 封面（无页码）"));
        assert!(plan.contains("**01 · 第一节**"));
        assert!(plan.contains("**02 · 第二节**"));
        assert!(plan.contains("寻月阁"));
        assert!(plan.contains("#2563EB"));
        assert!(plan.contains("theme: blueprint"));
        assert!(plan.contains("副信息：引言一句"));
    }

    #[test]
    fn build_card_plan_omits_subtitle_when_empty() {
        let sections = vec![("正文".to_string(), "内容".to_string())];
        let plan = build_card_plan("T", "A", "#000", &None, "", &sections);
        assert!(!plan.contains("副信息"));
        assert!(plan.contains("theme: default"));
    }

    #[test]
    fn build_publish_copy_prefers_summary_then_intro_then_first_section() {
        let sections = vec![("第一节".to_string(), "首节内容".to_string())];
        let summary = build_publish_copy(
            "标题",
            &Some("摘要文本".to_string()),
            &Some("引言文本".to_string()),
            &sections,
            &[],
        );
        assert!(summary.contains("摘要文本"));
        assert!(!summary.contains("引言文本"));

        let intro =
            build_publish_copy("标题", &None, &Some("引言文本".to_string()), &sections, &[]);
        assert!(intro.contains("引言文本"));
        assert!(intro.contains("首节内容"));

        let first = build_publish_copy("标题", &None, &None, &sections, &[]);
        assert!(first.contains("首节内容"));
    }

    #[test]
    fn build_publish_copy_uses_default_tags_when_empty() {
        let sections = vec![("正文".to_string(), "内容".to_string())];
        let copy = build_publish_copy("T", &None, &None, &sections, &[]);
        assert!(copy.contains("#内容创作 #工具推荐 #公众号运营"));
    }

    #[test]
    fn build_publish_copy_uses_provided_tags() {
        let sections = vec![("正文".to_string(), "内容".to_string())];
        let copy = build_publish_copy(
            "T",
            &None,
            &None,
            &sections,
            &["Rust".to_string(), "编程".to_string()],
        );
        assert!(copy.contains("#Rust #编程"));
        assert!(!copy.contains("#内容创作"));
    }

    #[test]
    fn remove_fence_blocks_strips_named_blocks() {
        let body = "前\n\n:::intro\n引言\n:::\n\n中\n\n:::summary\n摘要\n:::\n\n后";
        let out = remove_fence_blocks(body, &["intro", "summary"]);
        assert!(out.contains("前"));
        assert!(out.contains("中"));
        assert!(out.contains("后"));
        assert!(!out.contains("引言"));
        assert!(!out.contains("摘要"));
    }

    #[test]
    fn is_directive_meta_recognizes_shortcode_metadata() {
        assert!(is_directive_meta("label: foo"));
        assert!(is_directive_meta("number: 3"));
        assert!(is_directive_meta("适合谁：所有人"));
        assert!(is_directive_meta("type: callout"));
        assert!(is_directive_meta("theme: cyber"));
        assert!(is_directive_meta("color: red"));
        assert!(!is_directive_meta("普通正文一行"));
    }

    #[test]
    fn cards_text_renders_summary() {
        let r = CardsResult {
            article: "a.md".to_string(),
            title: "T".to_string(),
            theme: Some("cyber".to_string()),
            accent_color: "#A855F7".to_string(),
            card_count: 3,
            card_plan_path: "p".to_string(),
            publish_copy_path: "c".to_string(),
        };
        let t = cards_text(&r);
        assert!(t.contains("cards generated"));
        assert!(t.contains("T"));
        assert!(t.contains("3"));
    }

    fn tmp_articles_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("moonpub-cards-{tag}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn generate_cards_splits_long_article_into_multiple_cards() {
        let dir = tmp_articles_dir("multi");
        let md = dir.join("article.md");
        fs::write(
            &md,
            "---\ntitle: 示例文章\n---\n\n:::intro\n引言内容\n:::\n\n## 第一节\n第一节正文\n\n## 第二节\n第二节正文\n",
        )
        .unwrap();

        let result = generate_cards(&dir, &Config::default(), Path::new("article.md")).unwrap();
        assert_eq!(result.card_count, 2);
        assert!(fs::metadata(&result.card_plan_path).is_ok());
        assert!(fs::metadata(&result.publish_copy_path).is_ok());

        let plan = fs::read_to_string(&result.card_plan_path).unwrap();
        assert!(plan.contains("**01 · 第一节**"));
        assert!(plan.contains("**02 · 第二节**"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn generate_cards_falls_back_to_single_card_without_h2() {
        let dir = tmp_articles_dir("single");
        let md = dir.join("article.md");
        fs::write(&md, "---\ntitle: 单卡\n---\n\n只有一段正文，没有小标题。\n").unwrap();

        let result = generate_cards(&dir, &Config::default(), Path::new("article.md")).unwrap();
        assert_eq!(result.card_count, 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn generate_cards_strips_block_metadata_from_card_body() {
        let dir = tmp_articles_dir("meta");
        let md = dir.join("article.md");
        fs::write(
            &md,
            "---\ntitle: 元数据清理\n---\n\n## 一节\nlabel: foo\nnumber: 3\n适合谁：所有人\n真实正文\n",
        )
        .unwrap();

        let result = generate_cards(&dir, &Config::default(), Path::new("article.md")).unwrap();
        let plan = fs::read_to_string(&result.card_plan_path).unwrap();
        assert!(!plan.contains("label: foo"));
        assert!(!plan.contains("number: 3"));
        assert!(!plan.contains("适合谁：所有人"));
        assert!(plan.contains("真实正文"));

        let _ = fs::remove_dir_all(&dir);
    }
}
