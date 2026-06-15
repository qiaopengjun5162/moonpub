use std::fs;
use std::path::Path;

use crate::article::{parse_frontmatter, strip_frontmatter, strip_wechat_footer};
use crate::error::AppError;

pub fn export_article(vault: &Path, article: &Path, blog_root: &Path) -> Result<String, AppError> {
    let article = crate::article::resolve_article_path(vault, article);
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

    let title = front.title.as_deref().unwrap_or("").to_owned();
    let date = front.date.as_deref().unwrap_or("1970-01-01").to_owned();
    let tags = front.tags.clone();

    let slug = article
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?;

    // Replace WeChat CDN banner with local blog image.
    let body = replace_wechat_images(body);

    let zola_fm = build_zola_frontmatter(&title, &date, &tags);
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

    Ok(format!("exported\n  {}", dst.display()))
}

fn build_zola_frontmatter(title: &str, date: &str, tags: &[String]) -> String {
    let tags_toml = tags
        .iter()
        .map(|t| format!("\"{}\"", escape_toml_string(t)))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "+++\ntitle = \"{}\"\ndescription = \"{}\"\ndate = {date}T00:00:00Z\n[taxonomies]\ncategories = [\"读书\"]\ntags = [{tags_toml}]\n+++\n",
        escape_toml_string(title),
        escape_toml_string(title),
    )
}

pub(crate) fn escape_toml_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
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

        export_article(&root, &md_path, &blog)?;

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

        export_article(&root, &md_path, &blog)?;

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

        export_article(&root, &md_path, &blog)?;

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
}
