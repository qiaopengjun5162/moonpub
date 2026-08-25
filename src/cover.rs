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
    GeekBlack,
    Blueprint,
    AiLab,
    Clean,
    Minimal,
    Warm,
    Serif,
    Gradient,
    Literary,
    Ink,
    Sunset,
    Forest,
    Workflow,
}

pub struct CoverArtifact {
    pub html: String,
    pub html_path: PathBuf,
}

pub fn style_from_name(name: Option<&str>) -> CoverStyle {
    match name {
        Some("dark") => CoverStyle::Dark,
        Some("geek-black" | "geek_black") => CoverStyle::GeekBlack,
        Some("blueprint") => CoverStyle::Blueprint,
        Some("ai-lab" | "ai_lab") => CoverStyle::AiLab,
        Some("clean") => CoverStyle::Clean,
        Some("minimal") => CoverStyle::Minimal,
        Some("warm") => CoverStyle::Warm,
        Some("serif") => CoverStyle::Serif,
        Some("gradient") => CoverStyle::Gradient,
        Some("literary") => CoverStyle::Literary,
        Some("ink") => CoverStyle::Ink,
        Some("sunset") => CoverStyle::Sunset,
        Some("forest") => CoverStyle::Forest,
        Some("workflow") => CoverStyle::Workflow,
        _ => CoverStyle::Literary,
    }
}

pub fn write_cover_html(
    article_path: &Path,
    title: &str,
    digest: &str,
    author: &str,
    style: CoverStyle,
    tag: Option<&str>,
) -> Result<CoverArtifact, AppError> {
    let html = generate_cover_html(title, digest, author, style, tag);
    let html_path = cover_html_path(article_path);
    fs::write(&html_path, &html).map_err(|source| AppError::Io {
        path: html_path.clone(),
        source,
    })?;
    Ok(CoverArtifact { html, html_path })
}

/// Read back the `data-cover-style` attribute from a generated cover HTML.
///
/// Returns `None` when the template carries no style marker (literary-class
/// templates) or the file is unreadable. Used by `ship` to detect silent
/// style fallback: 2026-08-25 D19 事故——ship 命令漏 `--style geek-black` 时
/// 静默 fallback 到默认模板（无 data-cover-style 标记），调用方却以为
/// geek-black 生效，草稿封面变成 READING NOTES 书图标且不报错。
pub fn read_cover_style(html_path: &Path) -> Option<String> {
    let html = fs::read_to_string(html_path).ok()?;
    html.split("data-cover-style=\"")
        .nth(1)?
        .split('"')
        .next()
        .map(str::to_owned)
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

/// Existing cover image for the article — generated PNG or a downloaded
/// remote cover (JPG/PNG). Returns the first match in preference order.
pub fn cover_image_path(article_path: &Path) -> Option<PathBuf> {
    let slug = article_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("cover");
    let dir = article_path.parent().unwrap_or(article_path);
    ["png", "jpg", "jpeg"]
        .iter()
        .map(|ext| dir.join(format!("{slug}.cover.{ext}")))
        .find(|p| p.exists())
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
    let capture_path = temporary_capture_path(png_path);
    let output = std::process::Command::new(&bin)
        .args([
            "--headless",
            "--disable-gpu",
            "--no-sandbox",
            "--window-size=900,500",
            &format!("--screenshot={}", capture_path.display()),
            &format!("file://{}", abs_html.display()),
        ])
        .output();

    match output {
        Ok(_) if capture_path.exists() => match replace_cover_png(&capture_path, png_path) {
            Ok(()) => None,
            Err(error) => Some(format!(
                "screenshot failed: cannot replace {}: {error}",
                png_path.display()
            )),
        },
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let detail = stderr.trim();
            let detail = if detail.is_empty() {
                format!("Chrome exited with {}", output.status)
            } else {
                detail.to_owned()
            };
            Some(format!("screenshot failed: {detail}"))
        }
        Err(error) => Some(format!("screenshot failed: {error}")),
    }
}

fn temporary_capture_path(png_path: &Path) -> PathBuf {
    let file_name = png_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("cover.png");
    png_path.with_file_name(format!(
        ".{file_name}.moonpub-{}.tmp.png",
        std::process::id()
    ))
}

fn replace_cover_png(capture_path: &Path, png_path: &Path) -> std::io::Result<()> {
    fs::copy(capture_path, png_path)?;
    fs::remove_file(capture_path)
}

/// Generate a standalone HTML cover page from article frontmatter.
pub fn generate_cover_html(
    title: &str,
    subtitle: &str,
    author: &str,
    style: CoverStyle,
    tag: Option<&str>,
) -> String {
    let (title, subtitle) = cover_text(title, subtitle);
    let title = escape_html(&title);
    let subtitle = escape_html(&subtitle);
    let author = escape_html(author.trim());
    let tag = tag
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(escape_html);

    match style {
        CoverStyle::Dark => render_dark_cover(&title, &subtitle, &author),
        CoverStyle::GeekBlack => {
            render_geek_black_cover(&title, &subtitle, &author, tag.as_deref())
        }
        CoverStyle::Blueprint => render_blueprint_cover(&title, &subtitle, &author),
        CoverStyle::AiLab => render_ai_lab_cover(&title, &subtitle, &author),
        CoverStyle::Clean => render_clean_cover(&title, &subtitle, &author),
        CoverStyle::Minimal => render_minimal_cover(&title, &subtitle, &author),
        CoverStyle::Warm => render_warm_cover(&title, &subtitle, &author),
        CoverStyle::Serif => render_serif_cover(&title, &subtitle, &author),
        CoverStyle::Gradient => render_gradient_cover(&title, &subtitle, &author),
        CoverStyle::Literary => render_literary_cover(&title, &subtitle, &author),
        CoverStyle::Ink => render_ink_cover(&title, &subtitle, &author),
        CoverStyle::Sunset => render_sunset_cover(&title, &subtitle, &author),
        CoverStyle::Forest => render_forest_cover(&title, &subtitle, &author),
        CoverStyle::Workflow => render_workflow_cover(&title, &subtitle, &author),
    }
}

fn cover_text(title: &str, subtitle: &str) -> (String, String) {
    let title = title.trim();
    let subtitle = subtitle.trim();

    if title.is_empty() && !subtitle.is_empty() {
        (subtitle.to_owned(), String::new())
    } else {
        (title.to_owned(), subtitle.to_owned())
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

fn render_geek_black_cover(title: &str, subtitle: &str, author: &str, tag: Option<&str>) -> String {
    let tag_line = match tag {
        Some(t) => format!(r#"    <div class="tag"><span class="prompt">$</span>{t}</div>"#),
        None => String::new(),
    };
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:'SF Pro Text',-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif;background:#030712}}
.cover{{width:900px;height:500px;position:relative;overflow:hidden;background:radial-gradient(circle at 78% 18%,rgba(34,197,94,.22),transparent 25%),linear-gradient(135deg,#020617 0%,#0b1020 48%,#111827 100%);color:#e5edf7;padding:64px 76px}}
.grid{{position:absolute;inset:0;background-image:linear-gradient(rgba(148,163,184,.08) 1px,transparent 1px),linear-gradient(90deg,rgba(148,163,184,.08) 1px,transparent 1px);background-size:34px 34px;opacity:.65}}
.glow{{position:absolute;right:74px;top:54px;width:220px;height:220px;border:1px solid rgba(34,197,94,.28);border-radius:50%;box-shadow:0 0 60px rgba(34,197,94,.16)}}
.panel{{position:relative;height:100%;border:1px solid rgba(148,163,184,.22);border-radius:18px;background:rgba(2,6,23,.72);box-shadow:0 22px 55px rgba(0,0,0,.35);padding:38px 44px;display:flex;flex-direction:column;justify-content:center}}
.toolbar{{position:absolute;top:20px;left:24px;display:flex;gap:8px}}
.dot{{width:9px;height:9px;border-radius:50%;background:#22c55e;box-shadow:0 0 16px rgba(34,197,94,.8)}}
.dot:nth-child(2){{background:#38bdf8;box-shadow:0 0 14px rgba(56,189,248,.7)}}
.dot:nth-child(3){{background:#f59e0b;box-shadow:0 0 14px rgba(245,158,11,.65)}}
.tag{{display:inline-block;color:#86efac;font-size:11px;font-weight:800;letter-spacing:4px;text-transform:uppercase;margin-bottom:24px}}
.prompt{{color:#22c55e;margin-right:10px}}
.title{{font-size:42px;font-weight:900;line-height:1.16;color:#f8fafc;max-width:640px;margin-bottom:18px;letter-spacing:.4px;text-shadow:0 0 24px rgba(34,197,94,.16)}}
.subtitle{{font-size:16px;color:#a7b6c8;line-height:1.75;max-width:560px;margin-bottom:34px}}
.meta{{display:flex;align-items:center;gap:14px;color:#94a3b8;font-size:13px}}
.chip{{border:1px solid rgba(34,197,94,.32);border-radius:999px;padding:7px 13px;color:#bbf7d0;background:rgba(34,197,94,.08);font-family:'SF Mono',Consolas,monospace}}
.author{{letter-spacing:1px}}
.scanline{{position:absolute;left:0;right:0;bottom:62px;height:1px;background:linear-gradient(90deg,transparent,#22c55e,transparent);opacity:.75}}
</style>
</head>
<body>
<main class="cover" data-cover-style="geek-black">
  <div class="grid"></div><div class="glow"></div>
  <section class="panel">
    <div class="toolbar"><span class="dot"></span><span class="dot"></span><span class="dot"></span></div>
{tag_line}
    <h1 class="title">{title}</h1>
    <p class="subtitle">{subtitle}</p>
    <div class="meta"><span class="chip">WEB3 · DEV</span><span class="author">{author}</span></div>
    <div class="scanline"></div>
  </section>
</main>
</body>
</html>"#
    )
}

fn render_blueprint_cover(title: &str, subtitle: &str, author: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif;background:#eff6ff}}
.cover{{width:900px;height:500px;position:relative;overflow:hidden;background:#f5f8ff;color:#102a43;padding:54px 70px}}
.cover::before{{content:'';position:absolute;inset:0;background-image:linear-gradient(#d8e7ff 1px,transparent 1px),linear-gradient(90deg,#d8e7ff 1px,transparent 1px);background-size:28px 28px}}
.cover::after{{content:'';position:absolute;left:70px;right:70px;top:54px;bottom:54px;border:2px solid #2563eb;opacity:.35}}
.draft-line{{position:absolute;background:#2563eb;opacity:.22}}
.draft-line.one{{left:110px;top:88px;width:210px;height:2px}}
.draft-line.two{{right:112px;bottom:98px;width:230px;height:2px}}
.draft-box{{position:absolute;right:92px;top:76px;width:145px;height:96px;border:2px solid rgba(37,99,235,.32);border-radius:4px;background:rgba(255,255,255,.4)}}
.draft-box::before{{content:'';position:absolute;left:18px;right:18px;top:26px;height:2px;background:#2563eb;box-shadow:0 18px 0 #93b4f6,0 36px 0 #93b4f6}}
.content{{position:relative;height:100%;display:flex;flex-direction:column;justify-content:flex-end;padding:0 36px 20px}}
.tag{{font-size:11px;font-weight:800;letter-spacing:5px;color:#2563eb;text-transform:uppercase;margin-bottom:22px}}
.title{{font-size:41px;font-weight:900;line-height:1.18;color:#0f2742;max-width:630px;margin-bottom:16px}}
.subtitle{{font-size:16px;color:#57708c;line-height:1.78;max-width:540px;margin-bottom:30px}}
.meta{{display:flex;align-items:center;gap:12px;font-size:13px;color:#66819f}}
.stamp{{border:1px solid #93b4f6;border-radius:4px;padding:7px 12px;color:#1e40af;background:rgba(219,234,254,.65);font-weight:700;letter-spacing:1px}}
</style>
</head>
<body>
<main class="cover" data-cover-style="blueprint">
  <div class="draft-line one"></div><div class="draft-line two"></div><div class="draft-box"></div>
  <section class="content">
    <div class="tag">SYSTEM BLUEPRINT</div>
    <h1 class="title">{title}</h1>
    <p class="subtitle">{subtitle}</p>
    <div class="meta"><span class="stamp">ARCHITECTURE</span><span>{author}</span></div>
  </section>
</main>
</body>
</html>"#
    )
}

fn render_ai_lab_cover(title: &str, subtitle: &str, author: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:'SF Pro Text',-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif;background:#0b0b18}}
.cover{{width:900px;height:500px;position:relative;overflow:hidden;background:radial-gradient(circle at 22% 24%,rgba(56,189,248,.2),transparent 26%),radial-gradient(circle at 78% 76%,rgba(139,92,246,.26),transparent 30%),linear-gradient(145deg,#0f1020 0%,#15132b 52%,#20123d 100%);color:#eef2ff;padding:62px 78px}}
.orb{{position:absolute;border-radius:50%;border:1px solid rgba(196,181,253,.24);box-shadow:0 0 60px rgba(139,92,246,.18)}}
.orb.one{{right:72px;top:58px;width:150px;height:150px}}
.orb.two{{right:138px;top:112px;width:76px;height:76px}}
.trace{{position:absolute;left:80px;right:80px;bottom:70px;height:1px;background:linear-gradient(90deg,transparent,#38bdf8,#8b5cf6,transparent)}}
.content{{position:relative;height:100%;display:flex;flex-direction:column;justify-content:center;max-width:650px}}
.tag{{display:inline-block;width:max-content;border:1px solid rgba(139,92,246,.45);border-radius:999px;padding:7px 15px;color:#c4b5fd;background:rgba(139,92,246,.12);font-size:11px;font-weight:800;letter-spacing:4px;text-transform:uppercase;margin-bottom:26px}}
.title{{font-size:42px;font-weight:900;line-height:1.17;color:#ffffff;margin-bottom:18px;text-shadow:0 0 30px rgba(139,92,246,.28)}}
.subtitle{{font-size:16px;color:#bac4f4;line-height:1.78;max-width:560px;margin-bottom:32px}}
.meta{{display:flex;align-items:center;gap:12px;font-size:13px;color:#9aa7d9}}
.chip{{color:#7dd3fc;border-bottom:2px solid rgba(125,211,252,.45);padding-bottom:3px;font-weight:700}}
</style>
</head>
<body>
<main class="cover" data-cover-style="ai-lab">
  <div class="orb one"></div><div class="orb two"></div><div class="trace"></div>
  <section class="content">
    <div class="tag">AI LAB NOTE</div>
    <h1 class="title">{title}</h1>
    <p class="subtitle">{subtitle}</p>
    <div class="meta"><span class="chip">experiment log</span><span>{author}</span></div>
  </section>
</main>
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

fn render_workflow_cover(title: &str, subtitle: &str, author: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif;background:#eef2f4}}
.cover{{width:900px;height:500px;position:relative;overflow:hidden;background:#f7f8f7;color:#172c3b;padding:28px 42px}}
.cover::before{{content:'';position:absolute;inset:0;background:linear-gradient(115deg,rgba(255,255,255,.9) 0%,rgba(236,241,244,.72) 58%,rgba(220,229,234,.72) 100%)}}
.frame{{position:relative;height:100%;border-top:4px solid #17384b}}
.header{{height:124px;padding-top:18px;display:flex;justify-content:center;align-items:flex-start;position:relative}}
.heading{{width:620px;text-align:center}}
.eyebrow{{display:flex;align-items:center;justify-content:center;gap:10px;margin-bottom:10px;font-size:10px;font-weight:700;color:#6b7e89}}
.eyebrow-mark{{width:24px;height:3px;background:#dc825b}}
.eyebrow-text{{letter-spacing:2px}}
.title{{font-size:34px;font-weight:800;line-height:1.22;color:#132b3a;letter-spacing:0;max-height:82px;overflow:hidden}}
.byline{{position:absolute;right:0;top:23px;text-align:right}}
.product{{font-size:15px;font-weight:800;color:#17384b;letter-spacing:1px}}
.author{{display:block;margin-top:7px;font-size:11px;color:#86959d}}
.pipeline{{height:270px;display:grid;grid-template-columns:190px 68px 220px 68px 190px;align-items:center;justify-content:center}}
.sources{{height:226px;display:flex;flex-direction:column;justify-content:space-between}}
.source{{height:66px;padding:10px 12px;background:rgba(255,255,255,.9);border:1px solid #d5dee3;border-left:4px solid #879dab;border-radius:6px;display:flex;align-items:center;gap:11px;box-shadow:0 7px 18px rgba(29,54,69,.06)}}
.source:nth-child(2){{border-left-color:#dc825b}}
.source:nth-child(3){{border-left-color:#526f82}}
.source-icon{{width:34px;height:34px;flex:0 0 34px;position:relative;border:1px solid #9babb4;border-radius:5px;background:#f8fafb}}
.document::before{{content:'';position:absolute;left:8px;right:8px;top:9px;height:2px;background:#526f82;box-shadow:0 6px 0 #a2b0b8,0 12px 0 #a2b0b8}}
.waveform{{display:flex;align-items:center;justify-content:center;gap:3px}}
.waveform i{{display:block;width:3px;background:#dc825b;border-radius:2px}}
.waveform i:nth-child(1),.waveform i:nth-child(5){{height:8px}}
.waveform i:nth-child(2),.waveform i:nth-child(4){{height:18px}}
.waveform i:nth-child(3){{height:26px}}
.photo::before{{content:'';position:absolute;left:7px;right:7px;bottom:7px;height:16px;background:linear-gradient(140deg,transparent 0 20%,#879dab 21% 48%,transparent 49%),linear-gradient(220deg,transparent 0 34%,#526f82 35% 65%,transparent 66%)}}
.photo::after{{content:'';position:absolute;top:7px;right:7px;width:6px;height:6px;border-radius:50%;background:#dc825b}}
.source-copy{{min-width:0}}
.source-title{{font-size:15px;font-weight:700;color:#203c4c;margin-bottom:3px}}
.source-note{{font-size:9px;color:#82919a;white-space:nowrap}}
.flow{{position:relative;height:2px;background:#9fb0ba}}
.flow::after{{content:'';position:absolute;right:-1px;top:-4px;width:8px;height:8px;border-top:2px solid #526f82;border-right:2px solid #526f82;transform:rotate(45deg)}}
.core{{height:196px;border-radius:8px;background:#17384b;padding:18px 20px;color:#f6f8f8;box-shadow:0 16px 30px rgba(23,56,75,.18);position:relative;overflow:hidden}}
.core::before{{content:'';position:absolute;left:0;top:0;bottom:0;width:5px;background:#dc825b}}
.core-kicker{{font-size:9px;font-weight:700;letter-spacing:2px;color:#a9bac3;margin-bottom:6px}}
.core-title{{font-size:25px;font-weight:800;letter-spacing:1px;margin-bottom:13px}}
.steps{{display:grid;grid-template-columns:1fr 1fr;gap:8px}}
.step{{height:39px;border:1px solid rgba(255,255,255,.18);border-radius:5px;background:rgba(255,255,255,.06);padding:6px 8px}}
.step-index{{font-size:8px;color:#dc9a7e;margin-bottom:2px}}
.step-name{{font-size:12px;font-weight:650;color:#edf2f4}}
.core-note{{position:absolute;left:20px;bottom:11px;font-size:9px;color:#9eb0ba}}
.phone{{height:250px;border:6px solid #17384b;border-radius:8px;background:#fff;box-shadow:0 15px 28px rgba(23,56,75,.16);padding:17px 12px 12px;position:relative}}
.phone::before{{content:'';position:absolute;top:6px;left:50%;width:38px;height:3px;transform:translateX(-50%);border-radius:2px;background:#8799a3}}
.phone-bar{{display:flex;justify-content:space-between;align-items:center;font-size:7px;color:#84939b;margin-bottom:12px}}
.preview-cover{{height:56px;border-radius:4px;background:#e7edf0;padding:8px 9px;position:relative;overflow:hidden}}
.preview-cover::after{{content:'';position:absolute;right:8px;top:7px;width:34px;height:42px;border:3px solid #17384b;border-radius:4px;background:#fff}}
.preview-tag{{font-size:6px;font-weight:700;color:#dc825b;letter-spacing:1px;margin-bottom:5px}}
.preview-title{{width:82px;font-size:9px;font-weight:800;line-height:1.35;color:#17384b}}
.article-title{{font-size:10px;font-weight:800;line-height:1.4;color:#203744;margin:9px 0 6px;max-height:28px;overflow:hidden}}
.text-line{{height:4px;border-radius:2px;background:#d5dde1;margin-top:6px}}
.text-line.short{{width:70%}}
.confirm{{position:absolute;left:12px;right:12px;bottom:11px;height:25px;border-radius:4px;background:#dc825b;color:#fff;text-align:center;font-size:10px;font-weight:700;line-height:25px}}
.caption{{position:absolute;left:42px;bottom:12px;font-size:10px;color:#788a94}}
.caption strong{{color:#17384b}}
</style>
</head>
<body>
<main class="cover" data-cover-style="workflow">
  <div class="frame">
    <header class="header">
      <div class="heading">
        <div class="eyebrow"><span class="eyebrow-mark"></span><span class="eyebrow-text">LOCAL-FIRST PUBLISHING WORKFLOW</span></div>
        <h1 class="title">{title}</h1>
      </div>
      <div class="byline"><span class="product">MOONPUB</span><span class="author">{author}</span></div>
    </header>
    <section class="pipeline" aria-label="Markdown、飞书秒记和照片经 MoonPub 自动化进入手机预览">
      <div class="sources">
        <div class="source"><span class="source-icon document"></span><div class="source-copy"><div class="source-title">Markdown</div><div class="source-note">文章与 Obsidian 草稿</div></div></div>
        <div class="source"><span class="source-icon waveform"><i></i><i></i><i></i><i></i><i></i></span><div class="source-copy"><div class="source-title">飞书秒记</div><div class="source-note">完整转写与口述素材</div></div></div>
        <div class="source"><span class="source-icon photo"></span><div class="source-copy"><div class="source-title">生活照片</div><div class="source-note">真实记录与本地元数据</div></div></div>
      </div>
      <div class="flow"></div>
      <div class="core">
        <div class="core-kicker">AUTOMATION CORE</div>
        <div class="core-title">MoonPub</div>
        <div class="steps">
          <div class="step"><div class="step-index">01</div><div class="step-name">整理草稿</div></div>
          <div class="step"><div class="step-index">02</div><div class="step-name">优化排版</div></div>
          <div class="step"><div class="step-index">03</div><div class="step-name">生成封面</div></div>
          <div class="step"><div class="step-index">04</div><div class="step-name">推进预览</div></div>
        </div>
        <div class="core-note">本地优先 · 关键步骤由作者确认</div>
      </div>
      <div class="flow"></div>
      <div class="phone">
        <div class="phone-bar"><span>9:41</span><span>公众号预览</span></div>
        <div class="preview-cover"><div class="preview-tag">MOONPUB</div><div class="preview-title">从内容到手机预览</div></div>
        <div class="article-title">{subtitle}</div>
        <div class="text-line"></div><div class="text-line"></div><div class="text-line short"></div>
        <div class="confirm">手机确认</div>
      </div>
    </section>
  </div>
  <div class="caption"><strong>素材进入，手机确认。</strong> 重复流程交给自动化。</div>
</main>
</body>
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
        let html = generate_cover_html(
            "测试标题",
            "测试副标题",
            "Test Author",
            CoverStyle::Dark,
            None,
        );
        assert!(html.contains("测试标题"));
        assert!(html.contains("READING"));
        assert!(html.contains("900px"));
    }

    #[test]
    fn clean_cover_no_gradient() {
        let html = generate_cover_html("T", "S", "A", CoverStyle::Clean, None);
        assert!(html.contains("T"));
        assert!(!html.contains("linear-gradient"));
    }

    #[test]
    fn minimal_cover_uses_serif() {
        let html = generate_cover_html("T", "S", "A", CoverStyle::Minimal, None);
        assert!(html.contains("serif"));
        assert!(html.contains("text-align:center"));
    }

    #[test]
    fn read_cover_style_detects_marker() {
        // 2026-08-25：ship 封面风格校验的读回函数
        let html = "<html><head><style>…</style></head><body><main class=\"cover\" data-cover-style=\"geek-black\"><div class=\"tag\">WEB3 · DEV</div></main></body></html>";
        let p = std::env::temp_dir().join("moonpub-cover-style-test.html");
        std::fs::write(&p, html).unwrap();
        assert_eq!(read_cover_style(&p).as_deref(), Some("geek-black"));
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn read_cover_style_none_when_no_marker() {
        // literary 类模板无 data-cover-style 标记 → None（不误报）
        let html = "<main class=\"cover\"><div class=\"book-icon\"></div><div class=\"tag\">READING NOTES</div></main>";
        let p = std::env::temp_dir().join("moonpub-cover-style-none-test.html");
        std::fs::write(&p, html).unwrap();
        assert_eq!(read_cover_style(&p), None);
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn cover_html_well_formed() {
        let html = generate_cover_html("测试", "副标题", "作者", CoverStyle::Dark, None);
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
            None,
        );

        assert!(html.contains("Rust &amp; &lt;WeChat&gt; &quot;drafts&quot;"));
        assert!(html.contains("A &gt; B&#39;s note"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!html.contains("<script>alert(1)</script>"));
    }

    #[test]
    fn empty_title_promotes_subtitle_to_primary_line() {
        let html = generate_cover_html("   ", "这是摘要标题", "作者", CoverStyle::Literary, None);

        assert!(html.contains("<h1 class=\"title\">这是摘要标题</h1>"));
        assert!(!html.contains("<p class=\"subtitle\">这是摘要标题</p>"));
    }

    #[test]
    fn style_from_name_defaults_to_literary() {
        assert_eq!(style_from_name(Some("dark")), CoverStyle::Dark);
        assert_eq!(style_from_name(Some("geek-black")), CoverStyle::GeekBlack);
        assert_eq!(style_from_name(Some("geek_black")), CoverStyle::GeekBlack);
        assert_eq!(style_from_name(Some("blueprint")), CoverStyle::Blueprint);
        assert_eq!(style_from_name(Some("ai-lab")), CoverStyle::AiLab);
        assert_eq!(style_from_name(Some("ai_lab")), CoverStyle::AiLab);
        assert_eq!(style_from_name(Some("clean")), CoverStyle::Clean);
        assert_eq!(style_from_name(Some("workflow")), CoverStyle::Workflow);
        assert_eq!(style_from_name(Some("unknown")), CoverStyle::Literary);
        assert_eq!(style_from_name(None), CoverStyle::Literary);
    }

    #[test]
    fn geek_black_cover_uses_terminal_motif() {
        let html = generate_cover_html(
            "Rust 发布流水线",
            "用本地自动化减少重复动作",
            "Test Author",
            CoverStyle::GeekBlack,
            None,
        );

        assert!(html.contains("data-cover-style=\"geek-black\""));
        assert!(!html.contains("class=\"tag\"")); // 默认无 tag
        assert!(html.contains("WEB3 · DEV"));
        assert!(!html.contains("moonpub render"));
        assert!(!html.contains("BUILD NOTES"));
        assert!(html.contains("Rust 发布流水线"));
    }

    #[test]
    fn blueprint_cover_uses_architecture_motif() {
        let html = generate_cover_html(
            "系统设计复盘",
            "把关键边界画清楚",
            "Test Author",
            CoverStyle::Blueprint,
            None,
        );

        assert!(html.contains("data-cover-style=\"blueprint\""));
        assert!(html.contains("SYSTEM BLUEPRINT"));
        assert!(html.contains("ARCHITECTURE"));
        assert!(html.contains("系统设计复盘"));
    }

    #[test]
    fn ai_lab_cover_uses_experiment_motif() {
        let html = generate_cover_html(
            "Agent 工作流实验",
            "记录一次 AI 工程实践",
            "Test Author",
            CoverStyle::AiLab,
            None,
        );

        assert!(html.contains("data-cover-style=\"ai-lab\""));
        assert!(html.contains("AI LAB NOTE"));
        assert!(html.contains("experiment log"));
        assert!(html.contains("Agent 工作流实验"));
    }

    #[test]
    fn workflow_cover_shows_complete_pipeline() {
        let html = generate_cover_html(
            "MoonPub & 自动发布",
            "从内容到手机预览",
            "Test Author",
            CoverStyle::Workflow,
            None,
        );

        assert!(html.contains("data-cover-style=\"workflow\""));
        assert!(html.contains("Markdown"));
        assert!(html.contains("飞书秒记"));
        assert!(html.contains("生活照片"));
        assert!(html.contains("MoonPub"));
        assert!(html.contains("手机确认"));
        assert!(html.contains("MoonPub &amp; 自动发布"));
    }

    #[test]
    fn write_cover_html_uses_article_slug() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("cover-write")?;
        let article = root.join("Articles/drafts/demo.md");
        fs::create_dir_all(article.parent().unwrap())?;
        fs::write(&article, "---\n---\n")?;

        let artifact = write_cover_html(
            &article,
            "Title",
            "Digest",
            "Author",
            CoverStyle::Clean,
            None,
        )?;

        assert_eq!(
            artifact.html_path,
            root.join("Articles/drafts/demo.cover.html")
        );
        assert!(artifact.html.contains("Title"));
        assert_eq!(fs::read_to_string(&artifact.html_path)?, artifact.html);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn completed_capture_replaces_stale_png_and_removes_temp_file()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("cover-replace-png")?;
        let png = root.join("article.cover.png");
        let capture = temporary_capture_path(&png);
        fs::write(&png, b"old screenshot")?;
        fs::write(&capture, b"new screenshot")?;

        replace_cover_png(&capture, &png)?;

        assert_eq!(fs::read(&png)?, b"new screenshot");
        assert!(!capture.exists());
        fs::remove_dir_all(root)?;
        Ok(())
    }
}

#[test]
fn warm_cover_uses_orange() {
    let html = generate_cover_html("T", "S", "A", CoverStyle::Warm, None);
    assert!(html.contains("fef9e7"));
    assert!(html.contains("e67e22"));
}

#[test]
fn serif_cover_uses_serif_font() {
    let html = generate_cover_html("T", "S", "A", CoverStyle::Serif, None);
    assert!(html.contains("Noto Serif SC"));
}

#[test]
fn gradient_cover_has_purple() {
    let html = generate_cover_html("T", "S", "A", CoverStyle::Gradient, None);
    assert!(html.contains("764ba2")); // typos:ignore
}

#[test]
fn all_fourteen_styles_generate_html() {
    let styles = [
        CoverStyle::Dark,
        CoverStyle::GeekBlack,
        CoverStyle::Blueprint,
        CoverStyle::AiLab,
        CoverStyle::Clean,
        CoverStyle::Minimal,
        CoverStyle::Warm,
        CoverStyle::Serif,
        CoverStyle::Gradient,
        CoverStyle::Literary,
        CoverStyle::Ink,
        CoverStyle::Sunset,
        CoverStyle::Forest,
        CoverStyle::Workflow,
    ];
    for &style in &styles {
        let html = generate_cover_html("T", "S", "A", style, None);
        assert!(html.contains("<!DOCTYPE html>"));
    }
}
