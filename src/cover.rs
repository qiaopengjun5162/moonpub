//! Article cover generation using HTML templates.
//! Reference: guizang-ppt-skill, article-tools cover.html

/// Cover template variants.
#[allow(dead_code)]
pub enum CoverStyle {
    Dark,
    Clean,
    Minimal,
    Warm,
    Serif,
    Gradient,
}

/// Generate a standalone HTML cover page from article frontmatter.
pub fn generate_cover_html(title: &str, subtitle: &str, author: &str, style: CoverStyle) -> String {
    match style {
        CoverStyle::Dark => render_dark_cover(title, subtitle, author),
        CoverStyle::Clean => render_clean_cover(title, subtitle, author),
        CoverStyle::Minimal => render_minimal_cover(title, subtitle, author),
        CoverStyle::Warm => render_warm_cover(title, subtitle, author),
        CoverStyle::Serif => render_serif_cover(title, subtitle, author),
        CoverStyle::Gradient => render_gradient_cover(title, subtitle, author),
    }
}

fn render_dark_cover(title: &str, subtitle: &str, author: &str) -> String {
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

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
    assert!(html.contains("764ba2"));
}

#[test]
fn all_six_styles_generate_html() {
    let styles = [
        CoverStyle::Dark,
        CoverStyle::Clean,
        CoverStyle::Minimal,
        CoverStyle::Warm,
        CoverStyle::Serif,
        CoverStyle::Gradient,
    ];
    for style in styles.iter() {
        let html = match style {
            CoverStyle::Dark => generate_cover_html("T", "S", "A", CoverStyle::Dark),
            CoverStyle::Clean => generate_cover_html("T", "S", "A", CoverStyle::Clean),
            CoverStyle::Minimal => generate_cover_html("T", "S", "A", CoverStyle::Minimal),
            CoverStyle::Warm => generate_cover_html("T", "S", "A", CoverStyle::Warm),
            CoverStyle::Serif => generate_cover_html("T", "S", "A", CoverStyle::Serif),
            CoverStyle::Gradient => generate_cover_html("T", "S", "A", CoverStyle::Gradient),
        };
        assert!(html.contains("<!DOCTYPE html>"));
    }
}
