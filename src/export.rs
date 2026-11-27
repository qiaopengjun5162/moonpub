use std::fs;
use std::path::Path;

use crate::article::{
    extract_title_from_body, parse_frontmatter, strip_frontmatter, strip_wechat_footer,
};
use crate::error::AppError;
use crate::plugin::{ExportContext, ExportOutcome, ExportTarget, run_export_target};

pub struct ZolaExportTarget;

impl ExportTarget for ZolaExportTarget {
    fn id(&self) -> &'static str {
        "zola"
    }

    fn display_name(&self) -> &'static str {
        "Zola"
    }

    fn export(&self, ctx: ExportContext<'_>) -> Result<ExportOutcome, AppError> {
        let message =
            export_zola_article(ctx.articles_dir, ctx.article, ctx.export_root, ctx.category)?;
        Ok(ExportOutcome { message })
    }
}

pub fn export_article(
    articles_dir: &Path,
    article: &Path,
    blog_root: &Path,
    category: Option<&str>,
) -> Result<String, AppError> {
    let outcome = run_export_target(
        &ZolaExportTarget,
        ExportContext {
            articles_dir,
            article,
            export_root: blog_root,
            category,
        },
    )?;
    Ok(outcome.message)
}

fn export_zola_article(
    articles_dir: &Path,
    article: &Path,
    blog_root: &Path,
    category: Option<&str>,
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

    let title = front
        .title
        .as_deref()
        .map(str::to_owned)
        .or_else(|| extract_title_from_body(&md))
        .unwrap_or_default();

    // 摘要：优先用 frontmatter 的 digest——那是作者为公众号写的摘要，本来就该复用。
    // 原来这里把 title 塞了两次，导出的博客 `description` 与 `title` 一模一样，
    // 搜索结果和卡片里同一句话出现两遍。
    let description = front
        .digest
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .or_else(|| first_body_paragraph(body))
        .unwrap_or_else(|| title.clone());

    // 日期：frontmatter 写了就用它；没写就取文件的最后修改时间。
    // 原来没写就回落 "1970-01-01"，文件名也跟着变成 `1970-01-01-xxx.md`；
    // Zola 按 date 排文章，整站会一起掉到 1970 年。
    let date = front
        .date
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| article_date(&article));
    let tags = front.tags.clone();

    let slug = article
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?;

    // Replace WeChat CDN banner with local blog image.
    let body = replace_wechat_images(body);

    // 本地图片搬进 `static/images/` 并改写成站内绝对路径。
    let article_dir = article.parent().unwrap_or(Path::new("."));
    let (body, copied_images) = export_local_images(&body, article_dir, blog_root)?;

    let zola_fm = build_zola_frontmatter(&title, &description, &date, &tags, category);
    let content = format!("{zola_fm}\n<!-- more -->\n\n{}", body.trim_start());

    let filename = format!("{date}-{slug}.md");
    let content_dir = blog_root.join("content");
    fs::create_dir_all(&content_dir).map_err(|source| AppError::Io {
        path: content_dir.clone(),
        source,
    })?;
    let dst = content_dir.join(&filename);
    fs::write(&dst, &content).map_err(|source| AppError::Io {
        path: dst.clone(),
        source,
    })?;

    let mut message = format!("exported\n  {}", dst.display());
    if copied_images > 0 {
        message.push_str(&format!(
            "\n  images: {copied_images} copied to static/images/"
        ));
    }
    Ok(message)
}

/// 正文里的本地图片搬进 `static/images/`，引用改写成 `/images/<name>`。
///
/// Zola 的 `content/*.md` 里写 `../assets/x.png` 是断链——那个相对路径相对的是
/// **博客仓库**，不是写作仓库。图片必须真的落在 `static/` 下，引用写成 `/images/…`。
/// 只处理相对路径：`http(s)://` 是外链（微信 CDN 那张已由 `replace_wechat_images`
/// 换成 `/images/wechat-follow.png`），以 `/` 开头的是已经是站内路径，都原样保留。
///
/// 返回（改写后的正文，复制了几张图）。
fn export_local_images(
    body: &str,
    article_dir: &Path,
    blog_root: &Path,
) -> Result<(String, usize), AppError> {
    let images_dir = blog_root.join("static/images");
    let mut copied = 0usize;

    let mut rewritten = String::with_capacity(body.len());
    for (index, line) in body.lines().enumerate() {
        if index > 0 {
            rewritten.push('\n');
        }
        if !line.contains("![") {
            rewritten.push_str(line);
            continue;
        }
        let mut failure: Option<AppError> = None;
        let replaced = rewrite_image_links(line, |src| {
            if failure.is_some() || !is_local_image_ref(src) {
                return None;
            }
            match copy_image_into_blog(src, article_dir, &images_dir) {
                Ok(Some((name, created))) => {
                    if created {
                        copied += 1;
                    }
                    Some(format!("/images/{name}"))
                }
                Ok(None) => None,
                Err(err) => {
                    failure = Some(err);
                    None
                }
            }
        });
        if let Some(err) = failure {
            return Err(err);
        }
        rewritten.push_str(&replaced);
    }

    Ok((rewritten, copied))
}

/// 相对路径的本地图片才算要搬的；外链与站内绝对路径不动。
fn is_local_image_ref(src: &str) -> bool {
    let src = src.trim();
    !src.is_empty()
        && !src.starts_with('/')
        && !src.starts_with("http://")
        && !src.starts_with("https://")
        && !src.starts_with("data:")
}

/// 复制一张图进 `static/images/`，返回（站内文件名, 这次是否真的新建了文件）。
///
/// 优先沿用原 basename；同名文件已存在且字节相同就直接复用（多次导出不会堆副本，
/// 同一张图被正文引用两次也只算一张），同名但内容不同才依次试 `-1` `-2`，
/// 避免不同目录里的同名图互相覆盖。
fn copy_image_into_blog(
    src: &str,
    article_dir: &Path,
    images_dir: &Path,
) -> Result<Option<(String, bool)>, AppError> {
    let source = article_dir.join(src);
    if !source.is_file() {
        return Err(AppError::Io {
            path: source,
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "export 需要这张图存在：Zola 的 content 目录里写相对路径会变成断链",
            ),
        });
    }
    let bytes = fs::read(&source).map_err(|source_err| AppError::Io {
        path: source.clone(),
        source: source_err,
    })?;
    fs::create_dir_all(images_dir).map_err(|source_err| AppError::Io {
        path: images_dir.to_path_buf(),
        source: source_err,
    })?;

    let name = source
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AppError::InvalidArticlePath(source.clone()))?;

    let mut candidate = images_dir.join(name);
    if candidate.is_file() {
        let existing = fs::read(&candidate).map_err(|source_err| AppError::Io {
            path: candidate.clone(),
            source: source_err,
        })?;
        if existing == bytes {
            return Ok(Some((name.to_owned(), false)));
        }
        let (stem, ext) = split_image_name(name);
        let mut index = 1;
        loop {
            let alt = format!("{stem}-{index}{ext}");
            candidate = images_dir.join(&alt);
            if !candidate.is_file() {
                fs::write(&candidate, &bytes).map_err(|source_err| AppError::Io {
                    path: candidate.clone(),
                    source: source_err,
                })?;
                return Ok(Some((alt, true)));
            }
            let existing = fs::read(&candidate).map_err(|source_err| AppError::Io {
                path: candidate.clone(),
                source: source_err,
            })?;
            if existing == bytes {
                return Ok(Some((alt, false)));
            }
            index += 1;
        }
    }

    fs::write(&candidate, &bytes).map_err(|source_err| AppError::Io {
        path: candidate.clone(),
        source: source_err,
    })?;
    Ok(Some((name.to_owned(), true)))
}

fn split_image_name(name: &str) -> (&str, &str) {
    match name.rsplit_once('.') {
        // 扩展名带上点：`cover.png` → ("cover", ".png")。
        // 切点来自 `rsplit_once`，一定是合法字符边界。
        Some((stem, _)) => name.split_at(stem.len()),
        None => (name, ""),
    }
}

/// 把一行里的每个 markdown 图片引用交给 `f`；返回 `Some` 就替换目标路径。
///
/// 认 `![alt](src)` 与 `![alt](<带空格的路径>)` 两种写法；`f` 返回 `None` 表示
/// 保持原样。不是图片语法的 `[` 原样输出，所以正文里的方括号不会被吃掉。
fn rewrite_image_links<F>(line: &str, mut f: F) -> String
where
    F: FnMut(&str) -> Option<String>,
{
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(start) = rest.find("![") {
        out.push_str(&rest[..start]);
        let after = &rest[start..];
        let Some(close_alt) = after.find("](") else {
            out.push_str(after);
            return out;
        };
        let alt = &after[2..close_alt];
        let body_start = close_alt + 2;
        let Some(url_len) = after[body_start..].find(')') else {
            out.push_str(after);
            return out;
        };
        let raw_src = &after[body_start..body_start + url_len];
        let src = raw_src
            .strip_prefix('<')
            .and_then(|s| s.strip_suffix('>'))
            .unwrap_or(raw_src);
        match f(src) {
            Some(next) => out.push_str(&format!("![{alt}]({next})")),
            None => out.push_str(&after[..body_start + url_len + 1]),
        }
        rest = &after[body_start + url_len + 1..];
    }
    out.push_str(rest);
    out
}

fn build_zola_frontmatter(
    title: &str,
    description: &str,
    date: &str,
    tags: &[String],
    category: Option<&str>,
) -> String {
    let tags_toml = tags
        .iter()
        .map(|t| format!("\"{}\"", escape_toml_string(t)))
        .collect::<Vec<_>>()
        .join(", ");

    // 分类不再写死 `["读书"]`：那个值跟文章毫无关系，却会出现在每一篇导出的博客里。
    // 现在只认 `[blog] category`，没配就不写这一行。tags 也空时连 `[taxonomies]`
    // 段一起省掉——Zola 允许缺省，留个空段只是噪音。
    let categories_line = match category.map(str::trim).filter(|s| !s.is_empty()) {
        Some(name) => format!("categories = [\"{}\"]\n", escape_toml_string(name)),
        None => String::new(),
    };
    let taxonomies = if categories_line.is_empty() && tags_toml.is_empty() {
        String::new()
    } else {
        format!("[taxonomies]\n{categories_line}tags = [{tags_toml}]\n")
    };

    format!(
        "+++\ntitle = \"{}\"\ndescription = \"{}\"\ndate = {date}T00:00:00Z\n{taxonomies}+++\n",
        escape_toml_string(title),
        escape_toml_string(description),
    )
}

/// 取文件的最后修改时间（UTC 日期），用作没写 `date:` 时的兜底。
///
/// 连 mtime 都读不到（文件被删 / 文件系统不支持）时才回落 `1970-01-01`——
/// 那条路径现在只在权限异常时出现，不再是「没写 date」的默认行为。
fn article_date(article: &Path) -> String {
    fs::metadata(article)
        .and_then(|meta| meta.modified())
        .map(crate::timefmt::date_utc)
        .unwrap_or_else(|_| "1970-01-01".to_owned())
}

/// 正文第一段，用作没写 `digest:` 时的 description。
///
/// 跳过标题、引用块、图片、代码围栏和 HTML 注释：这些当摘要都不合适
/// （14 篇开头就是一段 `> **项目**：MoonPub …` 的引用块，拿它当 description
/// 会在搜索卡片里出现「项目：MoonPub —— 用 Rust 写的…」这种半句话）。
fn first_body_paragraph(body: &str) -> Option<String> {
    body.lines()
        .map(str::trim)
        .find(|line| {
            !line.is_empty()
                && !line.starts_with('#')
                && !line.starts_with('>')
                && !line.starts_with("<!--")
                && !line.starts_with("![")
                && !line.starts_with("```")
                && !line.starts_with("~~~")
        })
        .map(|line| line.trim_matches('*').trim().to_owned())
        .filter(|line| !line.is_empty())
}

pub(crate) fn escape_toml_string(s: &str) -> String {
    crate::html::escape_toml_string(s)
}

fn replace_wechat_images(body: &str) -> String {
    // Replace inline WeChat CDN banner references with the local blog image path.
    let re_cdn = "mmbiz.qpic.cn";
    let local = "/images/wechat-follow.png";
    body.lines()
        .map(|line| {
            if line.contains(re_cdn) {
                // Replace the whole src URL inside markdown image syntax.
                // Pattern: ![alt](http://mmbiz.qpic.cn/...)
                let mut out = String::new();
                let mut rest = line;
                while let Some(start) = rest.find("![") {
                    out.push_str(&rest[..start]);
                    let after = &rest[start..];
                    // Find matching )
                    if let Some(url_end) = after.find(')') {
                        let img_tag = &after[..url_end + 1];
                        if img_tag.contains(re_cdn) {
                            // Extract alt text
                            let alt_start = 2;
                            let alt_end = img_tag.find(']').unwrap_or(alt_start);
                            let alt = &img_tag[alt_start..alt_end];
                            out.push_str(&format!("![{alt}]({local})"));
                        } else {
                            out.push_str(img_tag);
                        }
                        rest = &after[url_end + 1..];
                    } else {
                        out.push_str(after);
                        rest = "";
                        break;
                    }
                }
                out.push_str(rest);
                out
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::article::parse_frontmatter;
    use crate::export::{escape_toml_string, export_article};
    use crate::plugin::ExportTarget;
    use crate::test_helpers::{create_file, temp_root};

    #[test]
    fn export_creates_zola_file() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("export-basic")?;
        let blog = root.join("blog");
        let md_path = root.join("Articles/published/demo.md");
        create_file(
            &md_path,
            "---\ntitle: \"我的文章\"\ndate: 2026-06-10\ntags: [\"Rust\", \"AI\"]\n---\n\n正文段落。\n\n## 章节\n\n更多内容。\n",
        )?;

        export_article(&root, &md_path, &blog, None)?;

        let out = blog.join("content/2026-06-10-demo.md");
        assert!(out.exists(), "Zola 文件应已创建");

        let content = fs::read_to_string(&out)?;
        assert!(content.starts_with("+++"), "应以 +++ 开头");
        assert!(content.contains("title = \"我的文章\""));
        assert!(content.contains("date = 2026-06-10T00:00:00Z"));
        assert!(content.contains("tags = [\"Rust\", \"AI\"]"));
        assert!(content.contains("<!-- more -->"));
        assert!(content.contains("正文段落"));
        assert!(!content.contains("---"), "YAML frontmatter 不应保留");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn export_replaces_cdn_image() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("export-img")?;
        let blog = root.join("blog");
        let md_path = root.join("article.md");
        create_file(
            &md_path,
            "---\ntitle: T\ndate: 2026-01-01\n---\n\n正文。\n\n![banner](http://mmbiz.qpic.cn/xxx/0?wx_fmt=png)\n",
        )?;

        export_article(&root, &md_path, &blog, None)?;

        let content = fs::read_to_string(blog.join("content/2026-01-01-article.md"))?;
        assert!(
            content.contains("/images/wechat-follow.png"),
            "CDN 图片应替换为本地路径"
        );
        assert!(!content.contains("mmbiz.qpic.cn"), "不应保留 CDN URL");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn export_strips_wechat_footer() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("export-footer")?;
        let blog = root.join("blog");
        let md_path = root.join("article.md");
        create_file(
            &md_path,
            "---\ntitle: T\ndate: 2026-01-01\n---\n\n正文内容。\n\n---\n\n![banner](http://mmbiz.qpic.cn/xxx)\n\n点个\"赞\"让我知道你喜欢，点个\"推荐\"让更多「寻月者」看到。\n",
        )?;

        export_article(&root, &md_path, &blog, None)?;

        let content = fs::read_to_string(blog.join("content/2026-01-01-article.md"))?;
        assert!(content.contains("正文内容"), "正文应保留");
        assert!(!content.contains("寻月者"), "WeChat footer 应被剥离");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn export_parses_tags() {
        let md =
            "---\ntitle: T\ndate: 2026-01-01\ntags: [\"读书\", \"Rust\", \"AI\"]\n---\n\n正文。\n";
        let fm = parse_frontmatter(md);
        assert_eq!(fm.tags, vec!["读书", "Rust", "AI"]);
        assert_eq!(fm.date.as_deref(), Some("2026-01-01"));
    }

    #[test]
    fn escape_toml_string_quotes_and_backslashes() {
        assert_eq!(escape_toml_string(r#"say "hi""#), r#"say \"hi\""#);
        assert_eq!(escape_toml_string(r"C:\path"), r"C:\\path");
    }

    #[test]
    fn escape_toml_string_plain_passthrough() {
        assert_eq!(escape_toml_string("hello world"), "hello world");
    }

    #[test]
    fn export_cannot_inject_toml_fields_via_title() -> Result<(), Box<dyn std::error::Error>> {
        // frontmatter 标题里的换行曾能闭合 TOML 字符串、注入新字段，改写导出的
        // Zola 配置。端到端验证：注入载荷只能作为标题文本出现，不能成为独立字段。
        let root = temp_root("export-inject")?;
        let blog = root.join("blog");
        let md_path = root.join("article.md");
        create_file(
            &md_path,
            "---\ntitle: \"x\\n[extra]\\ninjected = true\\n#\"\ndate: 2026-01-01\n---\n\n正文。\n",
        )?;

        export_article(&root, &md_path, &blog, None)?;

        let content = fs::read_to_string(blog.join("content/2026-01-01-article.md"))?;
        // 取开头 `+++` 与闭合 `+++` 之间的 frontmatter 段。
        let body_start =
            content.find("+++\n").expect("frontmatter 必须以 +++ 开头") + "+++\n".len();
        let frontmatter_end = content[body_start..]
            .find("\n+++")
            .map(|offset| body_start + offset)
            .expect("frontmatter 必须以 +++ 闭合");
        let frontmatter = &content[..frontmatter_end];

        assert!(
            !frontmatter.contains("\n[extra]"),
            "标题注入出了新 TOML 段：\n{frontmatter}"
        );
        assert!(
            !frontmatter.lines().any(|line| line.starts_with("injected")),
            "标题注入出了新字段：\n{frontmatter}"
        );
        assert!(
            frontmatter.contains(r"\n[extra]\ninjected = true"),
            "换行应以字面形式留在标题值里：\n{frontmatter}"
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn export_falls_back_to_the_file_mtime_when_date_is_missing()
    -> Result<(), Box<dyn std::error::Error>> {
        // 没写 date 时曾回落 "1970-01-01"，文件名也跟着变成 `1970-01-01-xxx.md`。
        // Zola 按 date 排序，那就是整站文章一起掉到 1970 年。
        let root = temp_root("export-no-date")?;
        let blog = root.join("blog");
        let md_path = root.join("Articles/published/nodate.md");
        create_file(&md_path, "---\ntitle: \"没有日期\"\n---\n\n正文段落。\n")?;

        export_article(&root, &md_path, &blog, None)?;

        let entries = fs::read_dir(blog.join("content"))?.collect::<Result<Vec<_>, _>>()?;
        assert_eq!(entries.len(), 1, "应只导出一个文件");
        let name = entries[0].file_name().to_string_lossy().into_owned();
        assert!(
            !name.starts_with("1970-01-01"),
            "没写 date 不该回落 1970：{name}"
        );

        let content = fs::read_to_string(entries[0].path())?;
        assert!(
            !content.contains("1970-01-01"),
            "frontmatter 里不该出现 1970-01-01：\n{content}"
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn export_uses_the_digest_as_description() -> Result<(), Box<dyn std::error::Error>> {
        // description 曾把 title 塞两遍：导出的博客摘要与标题一模一样，
        // 搜索结果和分享卡片里同一句话出现两次。
        let root = temp_root("export-digest")?;
        let blog = root.join("blog");
        let md_path = root.join("article.md");
        create_file(
            &md_path,
            "---\ntitle: \"标题\"\ndigest: \"一句话摘要：讲清楚这篇到底说什么。\"\ndate: 2026-06-10\n---\n\n正文段落。\n",
        )?;

        export_article(&root, &md_path, &blog, None)?;

        let content = fs::read_to_string(blog.join("content/2026-06-10-article.md"))?;
        assert!(content.contains("description = \"一句话摘要：讲清楚这篇到底说什么。\""));
        assert!(
            !content.contains("description = \"标题\""),
            "description 不该复读标题：\n{content}"
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn export_takes_the_first_body_paragraph_when_there_is_no_digest()
    -> Result<(), Box<dyn std::error::Error>> {
        // 没有 digest、正文开头又是引用块时，摘要应取第一段真正的正文，
        // 而不是那段 `> **项目**：…` 的半句话，也不是标题。
        let root = temp_root("export-desc-fallback")?;
        let blog = root.join("blog");
        let md_path = root.join("article.md");
        create_file(
            &md_path,
            "---\ntitle: \"标题\"\ndate: 2026-06-10\n---\n\n> **项目**：MoonPub\n\n真正的第一段正文。\n",
        )?;

        export_article(&root, &md_path, &blog, None)?;

        let content = fs::read_to_string(blog.join("content/2026-06-10-article.md"))?;
        assert!(content.contains("description = \"真正的第一段正文。\""));
        assert!(
            !content.contains("description = \"**项目**"),
            "引用块不该当摘要：\n{content}"
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn export_omits_categories_unless_blog_category_is_set()
    -> Result<(), Box<dyn std::error::Error>> {
        // categories 曾写死 ["读书"]，跟文章毫无关系却出现在每一篇导出里。
        let root = temp_root("export-no-category")?;
        let blog = root.join("blog");
        let md_path = root.join("article.md");
        create_file(
            &md_path,
            "---\ntitle: \"标题\"\ndate: 2026-06-10\n---\n\n正文。\n",
        )?;

        export_article(&root, &md_path, &blog, None)?;

        let content = fs::read_to_string(blog.join("content/2026-06-10-article.md"))?;
        assert!(
            !content.contains("categories"),
            "没配 category 不该写分类：\n{content}"
        );
        assert!(
            !content.contains("读书"),
            "不该再出现硬编码的「读书」：\n{content}"
        );
        assert!(
            !content.contains("[taxonomies]"),
            "categories 与 tags 都空时应省掉整个 taxonomies 段：\n{content}"
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn export_writes_the_configured_category() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("export-category")?;
        let blog = root.join("blog");
        let md_path = root.join("article.md");
        create_file(
            &md_path,
            "---\ntitle: \"标题\"\ndate: 2026-06-10\ntags: [\"Rust\"]\n---\n\n正文。\n",
        )?;

        export_article(&root, &md_path, &blog, Some("随笔"))?;

        let content = fs::read_to_string(blog.join("content/2026-06-10-article.md"))?;
        assert!(content.contains("[taxonomies]"));
        assert!(content.contains("categories = [\"随笔\"]"));
        assert!(content.contains("tags = [\"Rust\"]"));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn export_copies_local_images_into_static_and_rewrites_the_path()
    -> Result<(), Box<dyn std::error::Error>> {
        // Zola 的 content/*.md 里写 `../assets/x.png` 是断链——那个相对路径相对的是
        // 博客仓库，不是写作仓库。图片必须真的落到 static/ 下、引用写成 /images/…
        let root = temp_root("export-images")?;
        let blog = root.join("blog");
        let md_path = root.join("Articles/published/pic.md");
        create_file(
            &md_path,
            "---\ntitle: \"带图\"\ndate: 2026-06-10\n---\n\n![对比图](../assets/chart.png)\n",
        )?;
        create_file(&root.join("Articles/assets/chart.png"), "PNGDATA")?;

        export_article(&root, &md_path, &blog, None)?;

        assert!(
            blog.join("static/images/chart.png").is_file(),
            "图片应被复制进 static/images/"
        );
        let content = fs::read_to_string(blog.join("content/2026-06-10-pic.md"))?;
        assert!(
            content.contains("![对比图](/images/chart.png)"),
            "{content}"
        );
        assert!(
            !content.contains("../assets/chart.png"),
            "相对路径应被改写：\n{content}"
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn export_leaves_remote_and_site_absolute_images_alone()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("export-images-skip")?;
        let blog = root.join("blog");
        let md_path = root.join("article.md");
        create_file(
            &md_path,
            "---\ntitle: \"外链\"\ndate: 2026-06-10\n---\n\n![远端](https://example.com/a.png)\n\n![站内](/images/b.png)\n",
        )?;

        export_article(&root, &md_path, &blog, None)?;

        let content = fs::read_to_string(blog.join("content/2026-06-10-article.md"))?;
        assert!(content.contains("![远端](https://example.com/a.png)"));
        assert!(content.contains("![站内](/images/b.png)"));
        assert!(
            !blog.join("static/images").exists(),
            "没有本地图时不该创建 static/images"
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn export_does_not_overwrite_a_different_image_with_the_same_name()
    -> Result<(), Box<dyn std::error::Error>> {
        // 两篇文章各自的 assets/cover.png 内容不同：第二篇不能把第一篇的图盖掉。
        let root = temp_root("export-images-clash")?;
        let blog = root.join("blog");
        fs::create_dir_all(root.join("a/assets"))?;
        fs::create_dir_all(root.join("b/assets"))?;
        create_file(&root.join("a/assets/cover.png"), "FIRST")?;
        create_file(&root.join("b/assets/cover.png"), "SECOND")?;
        create_file(
            &root.join("a/post.md"),
            "---\ntitle: \"A\"\ndate: 2026-06-10\n---\n\n![c](assets/cover.png)\n",
        )?;
        create_file(
            &root.join("b/post.md"),
            "---\ntitle: \"B\"\ndate: 2026-06-11\n---\n\n![c](assets/cover.png)\n",
        )?;

        export_article(&root, &root.join("a/post.md"), &blog, None)?;
        export_article(&root, &root.join("b/post.md"), &blog, None)?;

        assert_eq!(
            fs::read_to_string(blog.join("static/images/cover.png"))?,
            "FIRST"
        );
        assert_eq!(
            fs::read_to_string(blog.join("static/images/cover-1.png"))?,
            "SECOND"
        );
        let second = fs::read_to_string(blog.join("content/2026-06-11-post.md"))?;
        assert!(second.contains("/images/cover-1.png"), "{second}");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn exporting_the_same_article_twice_reuses_the_copied_image()
    -> Result<(), Box<dyn std::error::Error>> {
        // 反复导出不该在 static/images 里堆出 cover.png / cover-1.png / cover-2.png…
        let root = temp_root("export-images-idempotent")?;
        let blog = root.join("blog");
        let md_path = root.join("post.md");
        create_file(
            &md_path,
            "---\ntitle: \"重复导出\"\ndate: 2026-06-10\n---\n\n![c](assets/cover.png)\n",
        )?;
        create_file(&root.join("assets/cover.png"), "SAME")?;

        export_article(&root, &md_path, &blog, None)?;
        export_article(&root, &md_path, &blog, None)?;

        let names = fs::read_dir(blog.join("static/images"))?
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["cover.png"], "同名同内容应复用：{names:?}");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn export_fails_loudly_when_a_local_image_is_missing() -> Result<(), Box<dyn std::error::Error>>
    {
        // 静默跳过 = 博客上线后才发现破图。导出时当场报错，改一行 Markdown 就能修。
        let root = temp_root("export-images-missing")?;
        let blog = root.join("blog");
        let md_path = root.join("post.md");
        create_file(
            &md_path,
            "---\ntitle: \"缺图\"\ndate: 2026-06-10\n---\n\n![没了](assets/nope.png)\n",
        )?;

        let err = export_article(&root, &md_path, &blog, None).unwrap_err();
        assert!(
            format!("{err}").contains("nope.png"),
            "报错要点名是哪张图：{err}"
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn export_rewrites_angle_bracketed_image_paths_with_spaces()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("export-images-spaces")?;
        let blog = root.join("blog");
        let md_path = root.join("post.md");
        create_file(
            &md_path,
            "---\ntitle: \"空格\"\ndate: 2026-06-10\n---\n\n![带空格](<assets/my shot.png>)\n",
        )?;
        create_file(&root.join("assets/my shot.png"), "PIXELS")?;

        export_article(&root, &md_path, &blog, None)?;

        let content = fs::read_to_string(blog.join("content/2026-06-10-post.md"))?;
        assert!(
            content.contains("![带空格](/images/my shot.png)"),
            "{content}"
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn rewrite_image_links_keeps_plain_brackets_and_unclosed_syntax() {
        // 正文里的方括号不是图片语法，必须原样留下；没闭合的 `![` 也不能吃掉后文。
        assert_eq!(
            super::rewrite_image_links("普通 [链接](x)", |_| None),
            "普通 [链接](x)"
        );
        assert_eq!(
            super::rewrite_image_links("半截 ![图", |_| None),
            "半截 ![图"
        );
        assert_eq!(
            super::rewrite_image_links("![a](b.png) 与 ![c](d.png)", |src| Some(format!(
                "/images/{src}"
            ))),
            "![a](/images/b.png) 与 ![c](/images/d.png)"
        );
    }

    #[test]
    fn zola_export_target_reports_capabilities() {
        let target = super::ZolaExportTarget;

        assert_eq!(target.id(), "zola");
        assert_eq!(target.display_name(), "Zola");
    }
}
