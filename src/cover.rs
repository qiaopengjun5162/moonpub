//! Article cover generation using HTML templates.
//! Reference: guizang-ppt-skill, article-tools cover.html

/// Cover template variants.
pub enum CoverStyle {
    Dark,
    Clean,
    Minimal,
}

/// Generate a standalone HTML cover page from article frontmatter.
pub fn generate_cover_html(title: &str, subtitle: &str, author: &str, style: CoverStyle) -> String {
    match style {
        CoverStyle::Dark => render_dark_cover(title, subtitle, author),
        CoverStyle::Clean => render_clean_cover(title, subtitle, author),
        CoverStyle::Minimal => render_minimal_cover(title, subtitle, author),
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
