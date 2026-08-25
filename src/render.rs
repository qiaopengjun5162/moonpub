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
    footer_cfg: &footer::FooterConfig,
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
    // 2026-08-25: 封面 HTML 不再嵌入正文。
    //
    // 此前 `cover_html` 会整段拼到正文开头（<main class="cover"
    // data-cover-style=…> 标题 + digest + tag + 作者）。微信编辑器会剥离
    // 封面样式表，只留下裸 DOM，导致已发布文章正文顶部出现
    // "WEB3 · DEV Paxon Qiao" 这类构建 tag 文字（D17/D18/D19 线上文章实测）。
    // 封面作为 thumb_media_id 图片已单独设置，本地预览看 .cover.html/.cover.png，
    // 正文不需要再嵌一份封面模板。参数保留仅为兼容现有调用方，后续大版本清理。
    let _ = cover_html;
    let body_with_cover = html_body;

    let mut final_footer = footer_cfg.clone();
    // Per-article footer fields imply the user wants the footer even when the
    // global [footer] section is disabled.
    if front.footer_variant.is_some() || front.footer_qrcode.is_some() {
        final_footer.enabled = true;
    }
    if let Some(variant) = front.footer_variant.as_deref() {
        final_footer.variant = variant.to_owned();
    }
    if let Some(qrcode) = front.footer_qrcode.as_deref() {
        final_footer.qrcode = qrcode.to_owned();
    }

    // Resolve qrcode path relative to articles root so upload_local_images
    // (which resolves relative to article_dir) gets an absolute path.
    let abs_qrcode: String;
    let resolved_qrcode = if final_footer.qrcode.is_empty()
        || final_footer.qrcode.starts_with("http://")
        || final_footer.qrcode.starts_with("https://")
        || final_footer.qrcode.starts_with('/')
    {
        &final_footer.qrcode
    } else {
        abs_qrcode = articles_dir
            .join(&final_footer.qrcode)
            .to_string_lossy()
            .into_owned();
        &abs_qrcode
    };

    final_footer.qrcode = resolved_qrcode.to_owned();
    if !final_footer.qrcode.is_empty()
        && !final_footer.qrcode.starts_with("http://")
        && !final_footer.qrcode.starts_with("https://")
        && !std::path::Path::new(&final_footer.qrcode).is_file()
    {
        eprintln!(
            "  ⚠ footer qrcode 路径不存在或不可读：{}；群二维码将不会显示",
            final_footer.qrcode
        );
    }
    let full_html = wrap_wechat_html(&body_with_cover, &t, &final_footer);

    let title = wechat_title(&front, &md);
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

    let preview_html = wrap_preview_html(&full_html);
    fs::write(&html_path, &preview_html).map_err(|source| AppError::Io {
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

fn wrap_preview_html(content: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"zh-CN\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>MoonPub Preview</title>\n</head>\n<body style=\"margin:0;background:#f6f7f9;padding:24px 12px;\">\n<main style=\"box-sizing: border-box; max-width: 720px; margin: 0 auto; background:#fff; padding:28px 24px; box-shadow:0 12px 36px rgba(15,23,42,0.08);\">\n{content}\n</main>\n</body>\n</html>\n"
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
    use crate::footer;
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

        render_article(
            &root,
            &md_path,
            "Test Author",
            "thumb123",
            "default",
            None,
            &footer::FooterConfig::default(),
        )?;

        let html = fs::read_to_string(root.join("demo.html"))?;
        assert!(
            html.starts_with("<!doctype html>"),
            "本地预览应有 HTML 文档头"
        );
        assert!(
            html.contains("<meta charset=\"utf-8\">"),
            "本地预览应声明 UTF-8"
        );
        assert!(
            html.contains("max-width: 720px"),
            "本地预览应模拟微信阅读宽度"
        );
        assert!(
            html.contains("box-sizing: border-box"),
            "本地预览卡片实际外宽不应被 padding 撑大"
        );
        assert!(html.contains("margin: 0 auto"), "本地预览应居中显示");
        assert!(html.contains("<section"), "缺少 section 容器");
        assert!(html.contains("正文第一段"), "正文未渲染");

        let json_str = fs::read_to_string(root.join("demo.draft.json"))?;
        assert!(json_str.contains("\"title\": \"测试文章标题\""));
        assert!(json_str.contains("\"author\": \"Test Author\""));
        assert!(json_str.contains("\"digest\": \"这是摘要\""));
        assert!(json_str.contains("\"thumb_media_id\": \"thumb123\""));
        assert!(
            !json_str.contains("<!doctype html>"),
            "微信 draft JSON 不应包含本地预览外壳"
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn cover_html_not_embedded_into_body() -> Result<(), Box<dyn std::error::Error>> {
        // 2026-08-25 回归测试：封面模板不得拼进正文（微信剥离 CSS 后
        // 会露出 "WEB3 · DEV Paxon Qiao" 裸 tag，D17/D18/D19 线上文章实测）。
        let root = temp_root("render-cover-no-embed")?;
        let md_path = root.join("demo.md");
        create_file(
            &md_path,
            "---\ntitle: 测试文章标题\ndigest: 这是摘要\n---\n\n正文第一段。\n",
        )?;

        let cover = "<main class=\"cover\" data-cover-style=\"geek-black\"><div class=\"tag\">WEB3 · DEV</div><h1>测试文章标题</h1><p>Paxon Qiao</p></main>";
        render_article(
            &root,
            &md_path,
            "Test Author",
            "thumb123",
            "default",
            Some(cover),
            &footer::FooterConfig::default(),
        )?;

        let html = fs::read_to_string(root.join("demo.html"))?;
        let json_str = fs::read_to_string(root.join("demo.draft.json"))?;
        assert!(
            !html.contains("data-cover-style"),
            "本地预览不应嵌入封面 HTML"
        );
        assert!(
            !json_str.contains("data-cover-style"),
            "微信推送正文不应嵌入封面 HTML"
        );
        assert!(
            !json_str.contains("WEB3 · DEV"),
            "tag 文字不得泄漏进微信正文"
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn render_enables_footer_when_frontmatter_has_footer_fields()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("render-footer-auto-enable")?;
        let md_path = root.join("article.md");
        create_file(
            &md_path,
            "---\ntitle: T\nfooter_variant: community\nfooter_qrcode: Context/assets/group.png\n---\n\n正文。\n",
        )?;
        create_file(&root.join("Context/assets/group.png"), "fake-png-data")?;

        render_article(
            &root,
            &md_path,
            "作者",
            "",
            "default",
            None,
            &footer::FooterConfig::default(),
        )?;

        let html = fs::read_to_string(root.join("article.html"))?;
        assert!(
            html.contains("群二维码"),
            "frontmatter footer_qrcode 应自动启用 footer"
        );
        assert!(
            html.contains("src=\"data:image/png;base64,"),
            "本地二维码应嵌入为 data URI"
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn render_applies_article_footer_variant_and_qrcode_overrides()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("render-footer-override")?;
        let md_path = root.join("article.md");
        create_file(
            &md_path,
            "---\ntitle: T\nfooter_variant: community\nfooter_qrcode: Context/assets/group.png\n---\n\n正文。\n",
        )?;
        create_file(&root.join("Context/assets/group.png"), "png-bytes")?;

        let footer_cfg = footer::FooterConfig {
            enabled: true,
            variant: "minimal".to_owned(),
            title: "社群标题".to_owned(),
            qrcode: String::new(),
            ..footer::FooterConfig::default()
        };
        render_article(&root, &md_path, "作者", "", "default", None, &footer_cfg)?;

        let html = fs::read_to_string(root.join("article.html"))?;
        assert!(html.contains("社群标题"));
        assert!(html.contains("src=\"data:image/png;base64,"));

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

        render_article(
            &root,
            &md_path,
            "作者",
            "",
            "default",
            None,
            &footer::FooterConfig::default(),
        )?;

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

        render_article(
            &root,
            &md_path,
            "a",
            "",
            "default",
            None,
            &footer::FooterConfig::default(),
        )?;

        let html = fs::read_to_string(root.join("elem.html"))?;
        assert!(html.contains("<h2 "), "h2 未渲染");
        assert!(html.contains("<strong "), "strong 未渲染");
        assert!(html.contains("<em>"), "em 未渲染");
        assert!(html.contains("<code "), "code 未渲染");
        assert!(html.contains("border-left: 4px solid"), "blockquote 未渲染");
        assert!(html.contains("width:42px;height:2px"), "hr 未渲染");

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
