//! Article-level rendering: frontmatter → HTML + WeChat draft JSON.
//!
//! The actual Markdown → HTML transformation lives in `crate::markdown`; this
//! module only orchestrates file I/O, path resolution, footer injection, and
//! the final draft JSON shape.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::article::{parse_frontmatter, strip_frontmatter, strip_wechat_footer, wechat_title};
use crate::config::Config;
use crate::error::AppError;
use crate::footer;
use crate::json_util::escape_json;
use crate::markdown::md_to_wechat_html;
use crate::status::add_status;
use crate::theme;
use crate::wechat::WechatClient;

/// Render a single Markdown article into `.html` and `.draft.json`.
pub fn render_article(
    articles_dir: &Path,
    article: &Path,
    author: &str,
    thumb_media_id: &str,
    theme_name: &str,
    cover_html: Option<&str>,
    qrcode_path: &str,
) -> Result<String, AppError> {
    let article = crate::article::resolve_article_path(articles_dir, article);
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

    // Resolve qrcode path relative to articles root so upload_local_images
    // (which resolves relative to article_dir) gets an absolute path.
    let abs_qrcode: String;
    let resolved_qrcode = if qrcode_path.is_empty()
        || qrcode_path.starts_with("http://")
        || qrcode_path.starts_with("https://")
        || qrcode_path.starts_with('/')
    {
        qrcode_path
    } else {
        abs_qrcode = articles_dir
            .join(qrcode_path)
            .to_string_lossy()
            .into_owned();
        &abs_qrcode
    };

    let footer_cfg = footer::FooterConfig::from_config(effective_author, resolved_qrcode);
    let full_html = wrap_wechat_html(&body_with_cover, &t, &footer_cfg);

    let title = wechat_title(&front);
    // Only use explicit digest from frontmatter. If empty, WeChat auto-extracts.
    let digest = front.digest.clone().unwrap_or_default();

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

    let _ = add_status(articles_dir, slug, "rendered", "");

    Ok(format!(
        "rendered\n  html:  {}\n  draft: {}",
        html_path.display(),
        json_path.display()
    ))
}

/// Wrap the rendered body in the theme's outer section and append the footer.
fn wrap_wechat_html(body: &str, theme: &theme::Theme, footer_cfg: &footer::FooterConfig) -> String {
    let ending = footer::render_footer(footer_cfg, theme);
    format!(
        "<section style=\"{}\">\n\n{body}\n\n{ending}\n\n</section>\n",
        theme.section_style()
    )
}

/// If frontmatter has a `cover` field, resolve it to a WeChat thumb_media_id.
///
/// Supports:
///   - HTTP URLs: download → upload to WeChat
///   - Local absolute paths: upload directly
///   - Local relative paths: resolve relative to article directory → upload
pub fn resolve_cover_thumb(
    front: &crate::article::Frontmatter,
    _cfg: &Config,
    dir: &Path,
    client: &WechatClient,
    token: &str,
) -> Result<Option<String>, AppError> {
    let cover_src = match &front.cover {
        Some(c) => c,
        None => return Ok(None),
    };
    let file_path = if cover_src.starts_with("http://") || cover_src.starts_with("https://") {
        // Download the cover image to a temp file
        let resp = ureq::get(cover_src)
            .call()
            .map_err(|e| AppError::PushFailed {
                message: format!("failed to download cover image: {e}"),
                ip_hint: None,
            })?;
        let mut buf = Vec::new();
        resp.into_reader()
            .read_to_end(&mut buf)
            .map_err(|e| AppError::PushFailed {
                message: format!("failed to read cover image: {e}"),
                ip_hint: None,
            })?;
        let ext = cover_src
            .rsplit('.')
            .next()
            .unwrap_or("jpg")
            .split('?')
            .next()
            .unwrap_or("jpg");
        let tmp = dir.join(format!("_cover_download.{ext}"));
        std::fs::write(&tmp, &buf).map_err(|source| AppError::Io {
            path: tmp.clone(),
            source,
        })?;
        tmp
    } else if cover_src.starts_with('/') {
        PathBuf::from(cover_src)
    } else {
        dir.join(cover_src)
    };
    if !file_path.exists() {
        return Ok(None);
    }
    let media_id = client.upload_image(token, &file_path)?;
    // Clean up temp download
    if cover_src.starts_with("http") {
        let _ = std::fs::remove_file(&file_path);
    }
    Ok(Some(media_id))
}

/// Build the JSON payload expected by WeChat's draft/add API.
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
            "--articles".to_owned(),
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
            "--articles".to_owned(),
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
    fn resolve_cover_thumb_downloads_and_fails_for_bad_url()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("cover-http")?;
        let dir = root.join("Articles/drafts");
        fs::create_dir_all(&dir)?;
        let front = parse_frontmatter(
            "---\ntitle: T\ncover: https://invalid.example/not-found.jpg\n---\n\n正文\n",
        );
        let client = WechatClient::new("fake_appid", "fake_secret");
        let result = resolve_cover_thumb(&front, &Config::default(), &dir, &client, "fake_token");
        // Bad URL → download fails → returns error
        assert!(result.is_err());
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
