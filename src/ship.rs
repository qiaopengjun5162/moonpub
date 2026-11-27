//! 一键发布编排：cover → thumb upload → render → push → optional export。
//!
//! 这里只做编排，具体平台细节分别落在 `cover` / `render` / `push` /
//! `push_browser` / `export`。文件里的 helper 都围绕「把一段需要网络或
//! Chrome 的流程切成可单独推理的小块」：凭据解析、封面风格校验、封面字节
//! 落盘这三件是纯逻辑，因此各自带测试；真正调用微信 API 与 Chrome 的那几行
//! 保持薄，交给 `ship` 的人工回归闸（见 `docs/TEST_COVERAGE_ZH.md`）。

use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::article::{Frontmatter, cover_title, parse_frontmatter};
use crate::config::Config;
use crate::cover;
use crate::error::AppError;
use crate::export::export_article;
use crate::push::push_article;
use crate::render::{render_article, resolve_cover_thumb};
use crate::wechat::WechatClient;

/// 封面可能留下的三种扩展名。`cover_image_path()` 按这个顺序取第一张存在的
/// 文件，所以写新封面之前必须把旧的另外两种删掉。
const COVER_EXTENSIONS: [&str; 3] = ["png", "jpg", "jpeg"];

/// 解析 `WECHAT_APPID` / `WECHAT_SECRET`：环境变量优先，appid 可回落到
/// `[wechat] appid`，secret 只认环境变量（不进配置文件）。
///
/// `env` 是注入点，线上传 `std::env::var(..).ok()`。把它做成参数而不是直接
/// 读进程环境，是因为测试里改 env 既会和并行测试互相踩，也会让「凭据缺哪个」
/// 这类用户可见错误只能靠人工验证（同 `system::find_chrome_with` 的做法）。
fn wechat_credentials(
    cfg: &Config,
    env: impl Fn(&str) -> Option<String>,
) -> Result<(String, String), AppError> {
    let appid = env("WECHAT_APPID")
        .or_else(|| cfg.wechat_appid.clone())
        .ok_or(AppError::MissingEnvVar("WECHAT_APPID"))?;
    let secret = env("WECHAT_SECRET").ok_or(AppError::MissingEnvVar("WECHAT_SECRET"))?;
    Ok((appid, secret))
}

fn wechat_client(cfg: &Config) -> Result<WechatClient, AppError> {
    let (appid, secret) = wechat_credentials(cfg, |key| env::var(key).ok())?;
    Ok(WechatClient::new(appid, secret))
}

pub fn ship_article(
    articles_dir: &Path,
    config_path: Option<&Path>,
    art_path: &Path,
    style: Option<&str>,
) -> Result<String, AppError> {
    let slug = art_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    let mut cfg = config_path
        .map(Config::load)
        .transpose()?
        .unwrap_or_default();
    let author = cfg.wechat_author.as_deref().unwrap_or("作者").to_owned();

    let mut results = Vec::new();

    let md = fs::read_to_string(art_path).map_err(|e| AppError::Io {
        path: art_path.to_path_buf(),
        source: e,
    })?;
    let front = parse_frontmatter(&md);
    // 封面分支只有「文章自带 cover:」和「现场生成」两类。风格解析只做一次，
    // 结果装进 `CoverInputs` 同时喂给 AI 分支与模板分支——两条分支各自再解析
    // 一遍就会重演「同一个名字两条命令两种风格」的老问题。
    let mut cover_html = None;
    if should_generate_cover(&front) {
        // 名字写错就停在这儿。以前是 `⚠ 未知封面风格 …，已回退 editorial` 然后
        // 照常上传：用户明说了风格，发出去的却是暖米褐（editorial 在禁黄清单里）。
        if let Some(unknown) = cover::unknown_requested_style(style, &front) {
            return Err(cover::unknown_style_error(unknown));
        }
        let (style_name, parsed_style) = cover::resolve_style(style, &front);
        let inputs = CoverInputs {
            art_path,
            md: &md,
            front: &front,
            author: &author,
            style_name,
            parsed_style,
        };
        if parsed_style == cover::CoverStyle::AiArt {
            ai_art_cover(&inputs, &mut cfg, &mut results)?;
        } else {
            cover_html = html_cover(&inputs, &mut cfg, &mut results)?;
        }
    } else if !cfg.cookie_auth_from_env() {
        existing_cover_thumb(&front, &mut cfg, art_path, &mut results)?;
    } else if let Some(cover_url) = front.cover.as_deref().filter(|url| url.starts_with("http")) {
        // Cookie 模式：本地封面已经由 `cover` 生成、`push_browser` 会自己上传，
        // 只有 frontmatter 写的是远程 `cover:` 时才需要下载。
        match download_cover(cover_url, art_path) {
            Ok(path) => results.push(format!("cover:  {} ({cover_url})", path.display())),
            Err(e) => results.push(format!("⚠ cover download: {e}")),
        }
    }

    let thumb = cfg
        .wechat_thumb_media_id
        .as_deref()
        .unwrap_or("")
        .to_owned();
    let mut footer_cfg = cfg.footer.clone();
    if footer_cfg.qrcode.is_empty() {
        footer_cfg.qrcode = cfg.qrcode_path.clone().unwrap_or_default();
    }
    results.push(render_article(
        articles_dir,
        art_path,
        &author,
        &thumb,
        cfg.wechat_theme.as_deref().unwrap_or("default"),
        cover_html.as_deref(),
        &footer_cfg,
    )?);
    if cfg.cookie_auth_from_env() {
        results.push(crate::push_browser::push_article_cookie(
            articles_dir,
            art_path,
            false,
            false,
            &cfg,
        )?);
    } else {
        results.push(push_article(articles_dir, art_path, false, &cfg)?);
    }

    if let Some(br) = cfg.blog_root.as_deref() {
        let src = export_source_for_ship(articles_dir, slug, art_path);
        results.push(export_article(
            articles_dir,
            &src,
            br,
            cfg.blog_category.as_deref(),
        )?);
    }
    Ok(results.join("\n\n"))
}

/// 封面渲染需要的全部输入。打包成一个结构体是为了让两个分支函数都不必
/// 背七八个参数（`clippy::too_many_arguments` 之外，参数列表本身就是
/// 「这里该有边界」的信号）。
struct CoverInputs<'a> {
    art_path: &'a Path,
    md: &'a str,
    front: &'a Frontmatter,
    author: &'a str,
    style_name: &'a str,
    parsed_style: cover::CoverStyle,
}

/// AI 封面分支：直接产出 PNG，没有 HTML，因此 thumb 必须在这里传。
fn ai_art_cover(
    inputs: &CoverInputs<'_>,
    cfg: &mut Config,
    results: &mut Vec<String>,
) -> Result<(), AppError> {
    let provider = cfg
        .ai_provider
        .as_deref()
        .map(|s| s.parse::<crate::ai::AiProvider>())
        .transpose()?
        .unwrap_or_default();
    let api_key = crate::ai::api_key(provider)?;
    // 只取 frontmatter 之后的 500 个**字符**（不是字节）当 prompt 摘要。
    let article_text: String = inputs
        .md
        .split_once("---")
        .map(|x| x.1)
        .unwrap_or("")
        .chars()
        .take(500)
        .collect();
    let cover_png = cover::generate_ai_cover(
        inputs.art_path,
        &cover_title(inputs.front, inputs.md, inputs.art_path),
        cover::resolve_subtitle(inputs.front),
        &article_text,
        provider,
        &api_key,
    )?;
    results.push(format!("cover:  {} (style=ai-art)", cover_png.display()));

    if !cfg.cookie_auth_from_env()
        && let Some(media_id) = upload_cover_thumb(&wechat_client(cfg)?, &cover_png, results)?
    {
        cfg.wechat_thumb_media_id = Some(media_id);
    }
    Ok(())
}

/// 模板封面分支：写 HTML → 核对风格没被静默回退 → 截图 PNG → 截图失败时
/// 把已生成的 PNG 传成 thumb（appsecret 模式下微信 API 需要 media_id）。
fn html_cover(
    inputs: &CoverInputs<'_>,
    cfg: &mut Config,
    results: &mut Vec<String>,
) -> Result<Option<String>, AppError> {
    let cover = cover::write_cover_html(
        inputs.art_path,
        &cover_title(inputs.front, inputs.md, inputs.art_path),
        cover::resolve_subtitle(inputs.front),
        inputs.front.author.as_deref().unwrap_or(inputs.author),
        inputs.parsed_style,
        inputs.front.cover_tag.as_deref(),
    )?;
    results.push(format!(
        "cover:  {} (style={})",
        cover.html_path.display(),
        inputs.style_name
    ));
    if let Some(warning) =
        style_fallback_warning(&cover.html_path, inputs.parsed_style, inputs.style_name)
    {
        eprintln!("{warning}");
    }

    let cover_png = cover::cover_png_path(inputs.art_path);
    if cover::capture_cover_png(&cover.html_path, &cover_png).is_none()
        && !cfg.cookie_auth_from_env()
        && let Some(media_id) = upload_cover_thumb(&wechat_client(cfg)?, &cover_png, results)?
    {
        cfg.wechat_thumb_media_id = Some(media_id);
    }
    Ok(Some(cover.html))
}

/// 封面风格校验：防静默 fallback（2026-08-25 D19 事故——`ship` 漏
/// `--style geek-black` 时封面静默回退默认模板且不报错，草稿封面变成
/// READING NOTES 书图标）。
///
/// 比对解析后的枚举而不是名字：`--style international` 与标记里的 `swiss`
/// 是同一个风格，按字符串比会误报。返回待打印的警告而不是自己 `eprintln!`，
/// 这样文案可以被断言（此前这条防事故检查一条测试都没有）。
fn style_fallback_warning(
    html_path: &Path,
    parsed_style: cover::CoverStyle,
    style_name: &str,
) -> Option<String> {
    match cover::read_cover_style(html_path) {
        Some(actual) => (cover::style_from_name(Some(&actual)) != parsed_style).then(|| {
            format!("  ⚠ 封面风格不一致：请求 {style_name}，实际 {actual}（检查 --style 参数）")
        }),
        None => matches!(
            parsed_style,
            cover::CoverStyle::GeekBlack
                | cover::CoverStyle::Blueprint
                | cover::CoverStyle::AiLab
                | cover::CoverStyle::Workflow
        )
        .then(|| {
            format!(
                "  ⚠ 封面模板缺少 data-cover-style 标记（请求 {style_name}），疑似静默 fallback 到默认模板"
            )
        }),
    }
}

/// 文章自带 `cover:` 时不生成封面，直接把它（或同目录已生成的 PNG）传成
/// 永久 thumb。没有可用封面时 `resolve_cover_thumb` 返回 `None`，不写任何行。
fn existing_cover_thumb(
    front: &Frontmatter,
    cfg: &mut Config,
    art_path: &Path,
    results: &mut Vec<String>,
) -> Result<(), AppError> {
    let client = wechat_client(cfg)?;
    let token = client.access_token()?;
    if let Some(media_id) = resolve_cover_thumb(
        front,
        cfg,
        art_path.parent().unwrap_or_else(|| Path::new(".")),
        art_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default(),
        &client,
        &token,
    )? {
        results.push(format!(
            "cover:  {}",
            front.cover.as_deref().unwrap_or_default()
        ));
        results.push(format!("thumb:  {media_id}"));
        cfg.wechat_thumb_media_id = Some(media_id);
    }
    Ok(())
}

/// 把封面 PNG 传成永久 thumb。
///
/// 两件事必须区分：`access_token` 拿不到是**硬失败**（`?`），因为后面
/// 建草稿同样需要它；单张封面上传失败只是**软失败**，正文照发——这是
/// 微信后台偶发 `errcode` 时的既定行为，不要一起改成失败。
fn upload_cover_thumb(
    client: &WechatClient,
    cover_png: &Path,
    results: &mut Vec<String>,
) -> Result<Option<String>, AppError> {
    let token = client.access_token()?;
    match client.upload_image(&token, cover_png) {
        Ok(media_id) => {
            results.push(format!("thumb:  {media_id}"));
            Ok(Some(media_id))
        }
        Err(e) => {
            results.push(format!("⚠ cover upload failed: {e}"));
            Ok(None)
        }
    }
}

fn export_source_for_ship(articles_dir: &Path, slug: &str, current_article: &Path) -> PathBuf {
    let published = articles_dir
        .join("Articles/published")
        .join(slug)
        .with_extension("md");
    if published.exists() {
        published
    } else {
        current_article.to_path_buf()
    }
}

fn should_generate_cover(front: &crate::article::Frontmatter) -> bool {
    front.cover.is_none()
}

fn download_cover(url: &str, art_path: &Path) -> Result<PathBuf, AppError> {
    let resp = ureq::get(url).call().map_err(|e| AppError::PushFailed {
        message: format!("下载封面失败: {e}"),
        ip_hint: None,
    })?;
    let content_type = resp.header("Content-Type").unwrap_or("").to_owned();
    let mut buf = Vec::new();
    resp.into_reader()
        .read_to_end(&mut buf)
        .map_err(|e| AppError::PushFailed {
            message: format!("读取封面数据失败: {e}"),
            ip_hint: None,
        })?;
    // The extension must match the actual bytes — WeChat rejects uploads whose
    // declared MIME type disagrees with the payload (ret=200002).
    let ext = cover_extension(&content_type, &buf, url);
    let slug = art_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let dir = art_path.parent().unwrap_or(art_path);
    write_cover_bytes(dir, slug, ext, &buf)
}

/// 落盘封面字节，并先清掉同一篇文章的旧封面。
///
/// 不清旧的会留下「上一轮是 png、这一轮是 jpg」两份文件，
/// `cover::cover_image_path()` 按 png → jpg → jpeg 取第一张存在的，于是
/// `push_browser` 上传的是过期的那张，而日志里看不出任何异常。
fn write_cover_bytes(dir: &Path, slug: &str, ext: &str, bytes: &[u8]) -> Result<PathBuf, AppError> {
    for stale in COVER_EXTENSIONS {
        let p = dir.join(format!("{slug}.cover.{stale}"));
        if p.exists() {
            fs::remove_file(&p).map_err(|source| AppError::Io {
                path: p.clone(),
                source,
            })?;
        }
    }
    let dest = dir.join(format!("{slug}.cover.{ext}"));
    fs::write(&dest, bytes).map_err(|source| AppError::Io {
        path: dest.clone(),
        source,
    })?;
    Ok(dest)
}

fn cover_extension(content_type: &str, bytes: &[u8], url: &str) -> &'static str {
    if content_type.contains("png") || bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "png"
    } else if content_type.contains("jpeg")
        || content_type.contains("jpg")
        || bytes.starts_with(&[0xFF, 0xD8, 0xFF])
    {
        "jpg"
    } else if url.to_lowercase().ends_with(".png") {
        "png"
    } else {
        "jpg"
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cover_extension, export_source_for_ship, should_generate_cover, style_fallback_warning,
        wechat_credentials, write_cover_bytes,
    };
    use crate::article::Frontmatter;
    use crate::config::Config;
    use crate::cover::CoverStyle;
    use crate::error::AppError;
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;

    fn temp_root(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let root = std::env::temp_dir().join(format!("moonpub-ship-{name}-{nanos}"));
        fs::create_dir_all(root.join("Articles/drafts"))?;
        fs::create_dir_all(root.join("Articles/published"))?;
        Ok(root)
    }

    /// 目录里实际剩下的文件名（排序后），用来断言「旧封面被清干净」。
    fn file_names(dir: &std::path::Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        names
    }

    fn env_of(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        move |key| map.get(key).cloned()
    }

    #[test]
    fn export_source_prefers_published_article_when_available()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("published")?;
        let draft = root.join("Articles/drafts/demo.md");
        let published = root.join("Articles/published/demo.md");
        fs::write(&draft, "# draft")?;
        fs::write(&published, "# published")?;

        assert_eq!(export_source_for_ship(&root, "demo", &draft), published);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn export_source_falls_back_to_current_article() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("draft")?;
        let draft = root.join("Articles/drafts/demo.md");
        fs::write(&draft, "# draft")?;

        assert_eq!(export_source_for_ship(&root, "demo", &draft), draft);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn ship_skips_generated_cover_when_frontmatter_cover_is_set() {
        let front = Frontmatter {
            cover: Some("fixed-cover.png".to_owned()),
            ..Frontmatter::default()
        };

        assert!(!should_generate_cover(&front));
    }

    #[test]
    fn ship_generates_cover_when_frontmatter_cover_is_missing() {
        assert!(should_generate_cover(&Frontmatter::default()));
    }

    // ── 凭据解析：缺哪个报哪个 ───────────────────────────────────────────────

    #[test]
    fn credentials_prefer_environment_over_config() {
        let cfg = Config {
            wechat_appid: Some("cfg-appid".to_owned()),
            ..Config::default()
        };
        let (appid, secret) = wechat_credentials(
            &cfg,
            env_of(&[
                ("WECHAT_APPID", "env-appid"),
                ("WECHAT_SECRET", "env-secret"),
            ]),
        )
        .expect("both present");
        assert_eq!(appid, "env-appid");
        assert_eq!(secret, "env-secret");
    }

    #[test]
    fn credentials_fall_back_to_config_appid_but_never_for_secret() {
        // appid 允许写在 moonpub.toml，secret 只认环境变量（不进配置文件）。
        let cfg = Config {
            wechat_appid: Some("cfg-appid".to_owned()),
            ..Config::default()
        };
        let (appid, secret) = wechat_credentials(&cfg, env_of(&[("WECHAT_SECRET", "env-secret")]))
            .expect("config appid is enough");
        assert_eq!(appid, "cfg-appid");
        assert_eq!(secret, "env-secret");

        let cfg_with_secret = Config {
            wechat_appid: Some("cfg-appid".to_owned()),
            ..Config::default()
        };
        let err = wechat_credentials(&cfg_with_secret, env_of(&[]))
            .expect_err("secret has no config fallback");
        assert!(matches!(err, AppError::MissingEnvVar("WECHAT_SECRET")));
    }

    #[test]
    fn credentials_report_the_missing_appid_by_name() {
        let err = wechat_credentials(&Config::default(), env_of(&[("WECHAT_SECRET", "s")]))
            .expect_err("no appid anywhere");
        assert!(matches!(err, AppError::MissingEnvVar("WECHAT_APPID")));
        // 第一行只报事实，两条可照抄的出路由 AppError 的 hint 负责。
        assert_eq!(
            err.to_string().lines().next().unwrap_or_default(),
            "missing env var: WECHAT_APPID"
        );
    }

    // ── 封面扩展名：必须与真实字节一致 ──────────────────────────────────────

    #[test]
    fn cover_extension_trusts_content_type_for_png() {
        assert_eq!(cover_extension("image/png", b"", "https://x/c.png"), "png");
    }

    #[test]
    fn cover_extension_detects_png_by_magic_bytes_when_type_is_generic() {
        let bytes = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A];
        assert_eq!(
            cover_extension("application/octet-stream", &bytes, "https://x/c"),
            "png"
        );
    }

    #[test]
    fn cover_extension_accepts_both_jpeg_spellings() {
        assert_eq!(cover_extension("image/jpeg", b"", "https://x/c"), "jpg");
        assert_eq!(cover_extension("image/jpg", b"", "https://x/c"), "jpg");
    }

    #[test]
    fn cover_extension_detects_jpeg_by_magic_bytes_when_type_is_generic() {
        let bytes = [0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(
            cover_extension("application/octet-stream", &bytes, "https://x/c"),
            "jpg"
        );
    }

    #[test]
    fn cover_extension_falls_back_to_the_url_extension_case_insensitively() {
        assert_eq!(
            cover_extension("application/octet-stream", b"plain", "https://x/C.PNG"),
            "png"
        );
    }

    #[test]
    fn cover_extension_defaults_to_jpg_when_nothing_identifies_the_format() {
        assert_eq!(
            cover_extension("application/octet-stream", b"plain", "https://x/c"),
            "jpg"
        );
    }

    // ── 封面落盘：旧扩展名的文件必须先消失 ──────────────────────────────────

    /// 干净的空目录（`temp_root` 里有 `Articles/`，这里只要一个空壳）。
    fn empty_dir(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let dir = temp_root(name)?.join("covers");
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    #[test]
    fn write_cover_bytes_removes_stale_covers_with_other_extensions()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = empty_dir("cover-stale")?;
        fs::write(dir.join("demo.cover.png"), b"old png")?;
        fs::write(dir.join("demo.cover.jpeg"), b"old jpeg")?;

        let dest = write_cover_bytes(&dir, "demo", "jpg", b"new jpg")?;

        assert_eq!(dest, dir.join("demo.cover.jpg"));
        assert_eq!(fs::read(&dest)?, b"new jpg");
        // png / jpeg 都被清掉：否则 cover_image_path 会优先捡到过期的那张。
        assert_eq!(file_names(&dir), vec!["demo.cover.jpg"]);

        fs::remove_dir_all(dir)?;
        Ok(())
    }

    #[test]
    fn write_cover_bytes_leaves_other_articles_and_assets_alone()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = empty_dir("cover-keep")?;
        fs::write(dir.join("other.cover.png"), b"other article")?;
        fs::write(dir.join("demo.figure.png"), b"inline figure")?;

        write_cover_bytes(&dir, "demo", "png", b"fresh")?;

        assert_eq!(
            file_names(&dir),
            vec!["demo.cover.png", "demo.figure.png", "other.cover.png"]
        );

        fs::remove_dir_all(dir)?;
        Ok(())
    }

    // ── 封面风格校验（2026-08-25 D19 事故的回归） ────────────────────────────

    fn cover_html_with_marker(marker: Option<&str>) -> String {
        match marker {
            Some(value) => format!("<html><body data-cover-style=\"{value}\"></body></html>"),
            None => "<html><body class=\"literary\"></body></html>".to_owned(),
        }
    }

    #[test]
    fn style_warning_is_silent_when_the_marker_matches_the_request()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_root("style-ok")?;
        let html = dir.join("demo.cover.html");
        fs::write(&html, cover_html_with_marker(Some("geek-black")))?;

        assert_eq!(
            style_fallback_warning(&html, CoverStyle::GeekBlack, "geek-black"),
            None
        );

        fs::remove_dir_all(dir)?;
        Ok(())
    }

    #[test]
    fn style_warning_compares_parsed_styles_not_spelling() -> Result<(), Box<dyn std::error::Error>>
    {
        // `international` 与标记里的 `swiss` 是同一个风格，按名字比会误报。
        assert_eq!(
            crate::cover::style_from_name(Some("international")),
            crate::cover::style_from_name(Some("swiss"))
        );

        let dir = temp_root("style-alias")?;
        let html = dir.join("demo.cover.html");
        fs::write(&html, cover_html_with_marker(Some("swiss")))?;

        assert_eq!(
            style_fallback_warning(&html, CoverStyle::Swiss, "international"),
            None
        );

        fs::remove_dir_all(dir)?;
        Ok(())
    }

    #[test]
    fn style_warning_fires_when_the_page_fell_back_to_another_style()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_root("style-mismatch")?;
        let html = dir.join("demo.cover.html");
        fs::write(&html, cover_html_with_marker(Some("editorial")))?;

        let warning = style_fallback_warning(&html, CoverStyle::GeekBlack, "geek-black")
            .expect("mismatch must warn");
        assert!(warning.contains("封面风格不一致"), "{warning}");
        assert!(
            warning.contains("geek-black") && warning.contains("editorial"),
            "{warning}"
        );

        fs::remove_dir_all(dir)?;
        Ok(())
    }

    #[test]
    fn style_warning_fires_when_a_strict_style_lost_its_marker()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_root("style-markerless")?;
        let html = dir.join("demo.cover.html");
        fs::write(&html, cover_html_with_marker(None))?;

        for style in [
            CoverStyle::GeekBlack,
            CoverStyle::Blueprint,
            CoverStyle::AiLab,
            CoverStyle::Workflow,
        ] {
            let warning = style_fallback_warning(&html, style, "geek-black")
                .expect("strict styles must keep their marker");
            assert!(warning.contains("缺少 data-cover-style"), "{warning}");
        }

        fs::remove_dir_all(dir)?;
        Ok(())
    }

    #[test]
    fn style_warning_stays_quiet_for_templates_that_never_carry_a_marker()
    -> Result<(), Box<dyn std::error::Error>> {
        // literary 类模板本来就没有标记，缺标记不等于静默 fallback。
        let dir = temp_root("style-literary")?;
        let html = dir.join("demo.cover.html");
        fs::write(&html, cover_html_with_marker(None))?;

        assert_eq!(
            style_fallback_warning(&html, CoverStyle::Editorial, "editorial"),
            None
        );

        fs::remove_dir_all(dir)?;
        Ok(())
    }
}
