//! Article cover generation using HTML templates.
//! Reference: guizang-ppt-skill, article-tools cover.html

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::system::find_chrome;

/// Cover template variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoverStyle {
    Dark,
    Clean,
    Minimal,
    Warm,
    Serif,
    Gradient,
    Literary,
    Ink,
    Sunset,
    Forest,
}

pub struct CoverArtifact {
    pub html: String,
    pub html_path: PathBuf,
}

pub fn style_from_name(name: Option<&str>) -> CoverStyle {
    match name {
        Some("dark") => CoverStyle::Dark,
        Some("clean") => CoverStyle::Clean,
        Some("minimal") => CoverStyle::Minimal,
        Some("warm") => CoverStyle::Warm,
        Some("serif") => CoverStyle::Serif,
        Some("gradient") => CoverStyle::Gradient,
        Some("literary") => CoverStyle::Literary,
        Some("ink") => CoverStyle::Ink,
        Some("sunset") => CoverStyle::Sunset,
        Some("forest") => CoverStyle::Forest,
        _ => CoverStyle::Literary,
    }
}

pub fn write_cover_html(
    article_path: &Path,
    title: &str,
    digest: &str,
    author: &str,
    style: CoverStyle,
) -> Result<CoverArtifact, AppError> {
    let html = generate_cover_html(title, digest, author, style);
    let html_path = cover_html_path(article_path);
    fs::write(&html_path, &html).map_err(|source| AppError::Io {
        path: html_path.clone(),
        source,
    })?;
    Ok(CoverArtifact { html, html_path })
}

pub fn cover_html_path(article_path: &Path) -> PathBuf {
    let slug = article_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("cover");
    let dir = article_path.parent().unwrap_or(article_path);
    dir.join(format!("{slug}.cover.html"))
}

pub fn cover_png_path(article_path: &Path) -> PathBuf {
    let slug = article_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("cover");
    let dir = article_path.parent().unwrap_or(article_path);
    dir.join(format!("{slug}.cover.png"))
}

pub fn capture_cover_png(html_path: &Path, png_path: &Path) -> Option<String> {
    let Some(bin) = find_chrome() else {
        return Some("screenshot skipped: Chrome/Chromium not found".to_owned());
    };

    let abs_html = fs::canonicalize(html_path).unwrap_or_else(|e| {
        eprintln!(
            "moonpub: cannot resolve absolute path for {}: {e}",
            html_path.display()
        );
        html_path.to_path_buf()
    });
    let status = std::process::Command::new(&bin)
        .args([
            "--headless",
            "--disable-gpu",
            "--no-sandbox",
            "--window-size=900,500",
            &format!("--screenshot={}", png_path.display()),
            &format!("file://{}", abs_html.display()),
        ])
        .output();

    if png_path.exists() {
        None
    } else {
        Some(format!(
            "screenshot failed: {}",
            status
                .err()
                .map(|e| e.to_string())
                .unwrap_or_else(|| "unknown error".to_owned())
        ))
    }
}

/// Generate a standalone HTML cover page from article frontmatter.
pub fn generate_cover_html(title: &str, subtitle: &str, author: &str, style: CoverStyle) -> String {
    let title = escape_html(title);
    let subtitle = escape_html(subtitle);
    let author = escape_html(author);

    match style {
        CoverStyle::Dark => render_dark_cover(&title, &subtitle, &author),
        CoverStyle::Clean => render_clean_cover(&title, &subtitle, &author),
        CoverStyle::Minimal => render_minimal_cover(&title, &subtitle, &author),
        CoverStyle::Warm => render_warm_cover(&title, &subtitle, &author),
        CoverStyle::Serif => render_serif_cover(&title, &subtitle, &author),
        CoverStyle::Gradient => render_gradient_cover(&title, &subtitle, &author),
        CoverStyle::Literary => render_literary_cover(&title, &subtitle, &author),
        CoverStyle::Ink => render_ink_cover(&title, &subtitle, &author),
        CoverStyle::Sunset => render_sunset_cover(&title, &subtitle, &author),
        CoverStyle::Forest => render_forest_cover(&title, &subtitle, &author),
    }
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn render_literary_cover(title: &str, subtitle: &str, author: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:'PingFang SC','Hiragino Sans GB','Microsoft YaHei',serif}}
.cover{{width:900px;height:500px;background:#1c1c1e;position:relative;display:flex;flex-direction:column;justify-content:flex-end;padding:80px 90px 70px}}
.cover::before{{content:'';position:absolute;top:0;left:0;right:0;height:3px;background:linear-gradient(90deg,#c9a96e,#e8d5b7,#c9a96e)}}
.cover::after{{content:'';position:absolute;top:3px;left:40px;right:40px;height:1px;background:rgba(201,169,110,0.3)}}
.book-icon{{position:absolute;top:50px;right:80px;width:120px;height:160px;border:2px solid rgba(201,169,110,0.3);border-radius:2px 8px 8px 2px;background:linear-gradient(135deg,rgba(201,169,110,0.08),rgba(201,169,110,0.02))}}
.book-icon::after{{content:'';position:absolute;left:8px;top:0;bottom:0;width:1px;background:rgba(201,169,110,0.15)}}
.book-spine{{position:absolute;top:40px;right:195px;width:6px;height:170px;background:linear-gradient(180deg,rgba(201,169,110,0.2),rgba(201,169,110,0.05));border-radius:1px}}
.tag{{font-size:11px;font-weight:600;letter-spacing:4px;color:#c9a96e;text-transform:uppercase;margin-bottom:24px}}
.title{{font-size:40px;font-weight:900;line-height:1.25;color:#f5f0e8;margin-bottom:14px;letter-spacing:1px;max-width:620px}}
.subtitle{{font-size:16px;color:#a09580;line-height:1.8;margin-bottom:30px;max-width:540px;font-style:italic}}
.meta{{display:flex;align-items:center;gap:10px;border-top:1px solid rgba(255,255,255,0.06);padding-top:24px}}
.author{{font-size:14px;color:#8a8070;letter-spacing:1px}}
.dot{{color:#c9a96e;margin:0 6px}}
</style>
</head>
<body><div class="cover"><div class="book-icon"></div><div class="book-spine"></div><div class="tag">READING NOTES</div><h1 class="title">{title}</h1><p class="subtitle">{subtitle}</p><div class="meta"><span class="author">{author}</span></div></div></body>
</html>"#
    )
}

fn render_dark_cover(title: &str, subtitle: &str, author: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif}}
.cover{{width:900px;height:500px;background:linear-gradient(135deg,#0a0a0a 0%,#1a1a2e 50%,#16213e 100%);display:flex;flex-direction:column;justify-content:center;padding:60px 80px;position:relative}}
.tag{{display:inline-block;background:rgba(255,255,255,0.12);color:#aaa;font-size:11px;font-weight:600;letter-spacing:3px;padding:6px 16px;border-radius:2px;margin-bottom:28px;text-transform:uppercase}}
.title{{font-size:38px;font-weight:900;line-height:1.2;color:#fff;margin-bottom:16px;letter-spacing:1px}}
.title em{{font-style:normal;color:#64b5f6}}
.subtitle{{font-size:16px;color:#999;line-height:1.7;margin-bottom:32px;max-width:600px}}
.meta{{display:flex;align-items:center;gap:12px}}
.avatar{{width:32px;height:32px;border-radius:50%;background:linear-gradient(135deg,#64b5f6,#42a5f5);display:flex;align-items:center;justify-content:center;color:#fff;font-size:13px;font-weight:bold}}
.author{{font-size:14px;color:#ccc}}
.line{{position:absolute;left:80px;bottom:60px;width:60px;height:2px;background:#64b5f6}}
</style>
</head>
<body>
<div class="cover">
  <div class="tag">READING · NOTES</div>
  <h1 class="title">{title}</h1>
  <p class="subtitle">{subtitle}</p>
  <div class="meta">
    <div class="avatar">寻</div>
    <span class="author">{author}</span>
  </div>
  <div class="line"></div>
</div>
</body>
</html>"#
    )
}

fn render_clean_cover(title: &str, subtitle: &str, author: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif}}
.cover{{width:900px;height:500px;background:#fafafa;display:flex;flex-direction:column;justify-content:center;padding:60px 80px;position:relative}}
.tag{{display:inline-block;font-size:11px;font-weight:600;letter-spacing:3px;padding:6px 0;margin-bottom:28px;text-transform:uppercase;color:#2c2c2c;border-bottom:2px solid #2c2c2c}}
.title{{font-size:38px;font-weight:900;line-height:1.2;color:#1a1a1a;margin-bottom:16px;letter-spacing:1px}}
.title em{{font-style:normal;color:#e65100}}
.subtitle{{font-size:16px;color:#888;line-height:1.7;margin-bottom:32px;max-width:600px}}
.author{{font-size:14px;color:#aaa}}
.line{{position:absolute;right:80px;top:60px;width:40px;height:40px;border-right:2px solid #2c2c2c;border-top:2px solid #2c2c2c}}
</style>
</head>
<body>
<div class="cover">
  <div class="line"></div>
  <div class="tag">READING · NOTES</div>
  <h1 class="title">{title}</h1>
  <p class="subtitle">{subtitle}</p>
  <span class="author">{author}</span>
</div>
</body>
</html>"#
    )
}

fn render_minimal_cover(title: &str, subtitle: &str, author: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:'Noto Serif SC',Georgia,'Songti SC',serif}}
.cover{{width:900px;height:500px;background:#fff;display:flex;flex-direction:column;justify-content:center;align-items:center;padding:60px;text-align:center;position:relative}}
.border{{position:absolute;top:40px;left:40px;right:40px;bottom:40px;border:1px solid #e0e0e0}}
.title{{font-size:34px;font-weight:700;line-height:1.3;color:#1a1a1a;margin-bottom:20px;letter-spacing:2px}}
.subtitle{{font-size:15px;color:#999;line-height:1.8;margin-bottom:36px;max-width:500px}}
.author{{font-size:13px;color:#bbb;letter-spacing:1px}}
.dot{{width:4px;height:4px;background:#1a1a1a;border-radius:50%;margin-bottom:20px}}
</style>
</head>
<body>
<div class="cover">
  <div class="border"></div>
  <h1 class="title">{title}</h1>
  <div class="dot"></div>
  <p class="subtitle">{subtitle}</p>
  <span class="author">{author}</span>
</div>
</body>
</html>"#
    )
}

fn render_warm_cover(title: &str, subtitle: &str, author: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif}}
.cover{{width:900px;height:500px;background:linear-gradient(135deg,#fef9e7 0%,#fdebd0 40%,#fad7a1 100%);display:flex;flex-direction:column;justify-content:center;padding:60px 80px;position:relative}}
.tag{{display:inline-block;color:#e67e22;font-size:12px;font-weight:700;letter-spacing:4px;margin-bottom:24px;text-transform:uppercase}}
.title{{font-size:36px;font-weight:900;line-height:1.25;color:#2c1810;margin-bottom:16px}}
.subtitle{{font-size:16px;color:#8b6914;line-height:1.7;margin-bottom:32px;max-width:580px}}
.author{{font-size:14px;color:#b87333}}
.accent{{position:absolute;right:60px;bottom:50px;width:80px;height:4px;background:#e67e22;border-radius:2px}}
</style>
</head>
<body><div class="cover"><div class="accent"></div><div class="tag">READING · NOTES</div><h1 class="title">{title}</h1><p class="subtitle">{subtitle}</p><span class="author">{author}</span></div></body>
</html>"#
    )
}

fn render_serif_cover(title: &str, subtitle: &str, author: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:'Noto Serif SC',Georgia,'Songti SC',serif}}
.cover{{width:900px;height:500px;background:#fdf6f0;display:flex;flex-direction:column;justify-content:center;align-items:center;padding:60px 100px;text-align:center;position:relative}}
.top-line{{position:absolute;top:50px;left:80px;right:80px;height:1px;background:#d4a574}}
.bottom-line{{position:absolute;bottom:50px;left:80px;right:80px;height:1px;background:#d4a574}}
.title{{font-size:32px;font-weight:700;line-height:1.35;color:#3e2723;margin-bottom:20px;letter-spacing:3px}}
.subtitle{{font-size:15px;color:#8d6e63;line-height:1.8;margin-bottom:40px;max-width:500px;font-style:italic}}
.author{{font-size:13px;color:#a1887f;letter-spacing:4px}}
</style>
</head>
<body><div class="cover"><div class="top-line"></div><div class="bottom-line"></div><h1 class="title">{title}</h1><p class="subtitle">{subtitle}</p><span class="author">{author}</span></div></body>
</html>"#
    )
}

fn render_gradient_cover(title: &str, subtitle: &str, author: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif}}
.cover{{width:900px;height:500px;background:linear-gradient(160deg,#667eea 0%,#764ba2 50%,#f093fb 100%);display:flex;flex-direction:column;justify-content:center;padding:60px 80px;position:relative}}
.tag{{display:inline-block;background:rgba(255,255,255,0.2);color:#fff;font-size:11px;font-weight:600;letter-spacing:3px;padding:6px 16px;border-radius:20px;margin-bottom:28px}}
.title{{font-size:38px;font-weight:900;line-height:1.2;color:#fff;margin-bottom:16px}}
.subtitle{{font-size:16px;color:rgba(255,255,255,0.85);line-height:1.7;margin-bottom:32px;max-width:580px}}
.author{{font-size:14px;color:rgba(255,255,255,0.7)}}
.circle{{position:absolute;right:-40px;top:-40px;width:200px;height:200px;border-radius:50%;background:rgba(255,255,255,0.08)}}
</style>
</head>
<body><div class="cover"><div class="circle"></div><div class="tag">READING · NOTES</div><h1 class="title">{title}</h1><p class="subtitle">{subtitle}</p><span class="author">{author}</span></div></body>
</html>"#
    )
}

fn render_ink_cover(title: &str, subtitle: &str, author: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:'Noto Serif SC','Songti SC',serif}}
.cover{{width:900px;height:500px;background:#faf8f5;position:relative;display:flex;flex-direction:column;justify-content:flex-end;padding:70px 90px 80px}}
.ink{{position:absolute;top:60px;right:80px;width:100px;height:100px;border-radius:50%;background:radial-gradient(circle,rgba(0,0,0,0.06) 0%,rgba(0,0,0,0.01) 70%,transparent 100%)}}
.ink2{{position:absolute;top:40px;right:140px;width:60px;height:60px;border-radius:50%;background:radial-gradient(circle,rgba(0,0,0,0.04) 0%,transparent 70%)}}
.line{{position:absolute;left:90px;top:60px;width:1px;height:80px;background:linear-gradient(180deg,transparent,rgba(0,0,0,0.1))}}
.tag{{font-size:11px;font-weight:400;letter-spacing:6px;color:#999;margin-bottom:24px}}
.title{{font-size:42px;font-weight:900;line-height:1.25;color:#1a1a1a;margin-bottom:14px;letter-spacing:2px;max-width:620px}}
.subtitle{{font-size:15px;color:#777;line-height:1.8;margin-bottom:30px;max-width:520px}}
.author{{font-size:12px;color:#bbb;letter-spacing:3px}}
</style>
</head>
<body><div class="cover"><div class="ink"></div><div class="ink2"></div><div class="line"></div><div class="tag">读书笔记</div><h1 class="title">{title}</h1><p class="subtitle">{subtitle}</p><span class="author">{author}</span></div></body>
</html>"#
    )
}

fn render_sunset_cover(title: &str, subtitle: &str, author: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Microsoft YaHei',sans-serif}}
.cover{{width:900px;height:500px;background:linear-gradient(160deg,#ff9a56 0%,#e8734a 30%,#d4624a 60%,#2d1b33 100%);display:flex;flex-direction:column;justify-content:flex-end;padding:70px 80px 80px;position:relative}}
.sun{{position:absolute;top:60px;right:100px;width:120px;height:120px;border-radius:50%;background:radial-gradient(circle,rgba(255,255,255,0.25) 0%,rgba(255,200,150,0.1) 40%,transparent 70%)}}
.mountains{{position:absolute;bottom:0;left:0;right:0;height:120px;background:linear-gradient(180deg,transparent 0%,rgba(0,0,0,0.2) 40%,rgba(0,0,0,0.4) 100%)}}
.tag{{font-size:10px;font-weight:600;letter-spacing:4px;color:rgba(255,255,255,0.7);margin-bottom:20px;text-transform:uppercase}}
.title{{font-size:40px;font-weight:900;line-height:1.2;color:#fff;margin-bottom:14px;letter-spacing:1px;max-width:640px;text-shadow:0 2px 8px rgba(0,0,0,0.15)}}
.subtitle{{font-size:15px;color:rgba(255,255,255,0.85);line-height:1.8;margin-bottom:28px;max-width:520px}}
.author{{font-size:13px;color:rgba(255,255,255,0.6);letter-spacing:2px}}
</style>
</head>
<body><div class="cover"><div class="sun"></div><div class="mountains"></div><div class="tag">Reading Notes</div><h1 class="title">{title}</h1><p class="subtitle">{subtitle}</p><span class="author">{author}</span></div></body>
</html>"#
    )
}

fn render_forest_cover(title: &str, subtitle: &str, author: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Microsoft YaHei',sans-serif}}
.cover{{width:900px;height:500px;background:linear-gradient(150deg,#1b4332 0%,#2d6a4f 35%,#40916c 70%,#1b4332 100%);display:flex;flex-direction:column;justify-content:flex-end;padding:70px 80px 80px;position:relative}}
.leaf{{position:absolute;top:50px;right:70px;width:80px;height:80px;border-radius:60% 0 60% 0;background:rgba(255,255,255,0.08);transform:rotate(-15deg)}}
.leaf2{{position:absolute;top:70px;right:120px;width:50px;height:50px;border-radius:60% 0 60% 0;background:rgba(255,255,255,0.05);transform:rotate(25deg)}}
.light{{position:absolute;top:0;left:30%;width:1px;height:200px;background:linear-gradient(180deg,rgba(255,255,255,0.15),transparent)}}
.tag{{display:inline-block;border:1px solid rgba(255,255,255,0.25);color:rgba(255,255,255,0.8);font-size:10px;font-weight:600;letter-spacing:4px;padding:6px 14px;border-radius:2px;margin-bottom:24px;text-transform:uppercase}}
.title{{font-size:40px;font-weight:900;line-height:1.25;color:#e9f5ec;margin-bottom:14px;letter-spacing:1px;max-width:620px}}
.subtitle{{font-size:15px;color:rgba(233,245,236,0.7);line-height:1.8;margin-bottom:28px;max-width:520px}}
.author{{font-size:13px;color:rgba(233,245,236,0.5);letter-spacing:2px}}
</style>
</head>
<body><div class="cover"><div class="leaf"></div><div class="leaf2"></div><div class="light"></div><div class="tag">Reading · Notes</div><h1 class="title">{title}</h1><p class="subtitle">{subtitle}</p><span class="author">{author}</span></div></body>
</html>"#
    )
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::temp_root;

    #[test]
    fn dark_cover_contains_title() {
        let html = generate_cover_html("测试标题", "测试副标题", "寻月隐君", CoverStyle::Dark);
        assert!(html.contains("测试标题"));
        assert!(html.contains("READING"));
        assert!(html.contains("900px"));
    }

    #[test]
    fn clean_cover_no_gradient() {
        let html = generate_cover_html("T", "S", "A", CoverStyle::Clean);
        assert!(html.contains("T"));
        assert!(!html.contains("linear-gradient"));
    }

    #[test]
    fn minimal_cover_uses_serif() {
        let html = generate_cover_html("T", "S", "A", CoverStyle::Minimal);
        assert!(html.contains("serif"));
        assert!(html.contains("text-align:center"));
    }

    #[test]
    fn cover_html_well_formed() {
        let html = generate_cover_html("测试", "副标题", "作者", CoverStyle::Dark);
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn cover_escapes_frontmatter_text() {
        let html = generate_cover_html(
            r#"Rust & <WeChat> "drafts""#,
            "A > B's note",
            "<script>alert(1)</script>",
            CoverStyle::Literary,
        );

        assert!(html.contains("Rust &amp; &lt;WeChat&gt; &quot;drafts&quot;"));
        assert!(html.contains("A &gt; B&#39;s note"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!html.contains("<script>alert(1)</script>"));
    }

    #[test]
    fn style_from_name_defaults_to_literary() {
        assert_eq!(style_from_name(Some("dark")), CoverStyle::Dark);
        assert_eq!(style_from_name(Some("clean")), CoverStyle::Clean);
        assert_eq!(style_from_name(Some("unknown")), CoverStyle::Literary);
        assert_eq!(style_from_name(None), CoverStyle::Literary);
    }

    #[test]
    fn write_cover_html_uses_article_slug() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("cover-write")?;
        let article = root.join("Articles/drafts/demo.md");
        fs::create_dir_all(article.parent().unwrap())?;
        fs::write(&article, "---\n---\n")?;

        let artifact = write_cover_html(&article, "Title", "Digest", "Author", CoverStyle::Clean)?;

        assert_eq!(
            artifact.html_path,
            root.join("Articles/drafts/demo.cover.html")
        );
        assert!(artifact.html.contains("Title"));
        assert_eq!(fs::read_to_string(&artifact.html_path)?, artifact.html);

        fs::remove_dir_all(root)?;
        Ok(())
    }
}

#[test]
fn warm_cover_uses_orange() {
    let html = generate_cover_html("T", "S", "A", CoverStyle::Warm);
    assert!(html.contains("fef9e7"));
    assert!(html.contains("e67e22"));
}

#[test]
fn serif_cover_uses_serif_font() {
    let html = generate_cover_html("T", "S", "A", CoverStyle::Serif);
    assert!(html.contains("Noto Serif SC"));
}

#[test]
fn gradient_cover_has_purple() {
    let html = generate_cover_html("T", "S", "A", CoverStyle::Gradient);
    assert!(html.contains("764ba2")); // typos:ignore
}

#[test]
fn all_ten_styles_generate_html() {
    let styles = [
        CoverStyle::Dark,
        CoverStyle::Clean,
        CoverStyle::Minimal,
        CoverStyle::Warm,
        CoverStyle::Serif,
        CoverStyle::Gradient,
        CoverStyle::Literary,
        CoverStyle::Ink,
        CoverStyle::Sunset,
        CoverStyle::Forest,
    ];
    for &style in &styles {
        let html = generate_cover_html("T", "S", "A", style);
        assert!(html.contains("<!DOCTYPE html>"));
    }
}
