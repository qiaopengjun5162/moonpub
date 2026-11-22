//! 22 种封面模板与 editorial 的内容驱动配色。

use super::*;

pub(super) fn render_literary_cover(title: &str, subtitle: &str, author: &str) -> String {
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

pub(super) fn render_dark_cover(title: &str, subtitle: &str, author: &str) -> String {
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

pub(super) fn render_geek_black_cover(
    title: &str,
    subtitle: &str,
    author: &str,
    tag: Option<&str>,
) -> String {
    let tag_line = match tag {
        Some(t) => format!(r#"    <div class="tag">{t}</div>"#),
        None => String::new(),
    };
    // 主标题字数不定（7–22 字），固定字号会让短标题显得单薄、长标题顶到边缘。
    // 按字数分档，字数越少字号越大，保持每张封面的视觉重量一致。
    // 超过 10 字时按首个句读断成两行：单行铺满 900px 会失去海报感，
    // 断行后同样的字能有更大字号，缩略图里更好认。
    let (title_html, title_lines) = match title.find(['，', '、']) {
        Some(idx) if title.chars().count() > 10 => {
            let (head, tail) = title.split_at(idx);
            let tail = tail.trim_start_matches(['，', '、']).trim();
            (format!("{head}，<br>{tail}"), 2usize)
        }
        _ => (title.to_owned(), 1usize),
    };
    let title_size = match (title.chars().count(), title_lines) {
        (_, 2) => 60,
        (0..=8, _) => 82,
        (9..=11, _) => 68,
        (12..=15, _) => 58,
        _ => 50,
    };
    let subtitle_size = if subtitle.chars().count() > 18 {
        18
    } else {
        21
    };
    // 设了 cover_tag 时上方已有系列标识，下方不再重复渲染同一个 chip。
    let chip = match tag {
        Some(_) => String::new(),
        None => r#"<span class="chip">TECH · NOTES</span>"#.to_owned(),
    };
    // 视觉原则：只用一种强调色，且只出现在细线和标签上；主标题用近白，
    // 不再整屏荧光绿 + 发光 + 扫描线（小图糊成一团，也显得廉价）。
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif;background:#0a0d12}}
.cover{{width:900px;height:500px;position:relative;overflow:hidden;background:#0a0d12;color:#f5f7fa;padding:0 76px;display:flex;flex-direction:column;justify-content:center}}
.glow{{position:absolute;top:-220px;right:-160px;width:620px;height:620px;border-radius:50%;background:radial-gradient(circle,rgba(34,197,94,.16) 0%,rgba(34,197,94,0) 68%)}}
.edge{{position:absolute;left:0;top:0;bottom:0;width:3px;background:linear-gradient(180deg,rgba(34,197,94,.85),rgba(34,197,94,.15))}}
.inner{{position:relative}}
.tag{{font-size:11px;font-weight:700;letter-spacing:4px;color:#4ade80;text-transform:uppercase;margin-bottom:26px}}
.rule{{width:46px;height:2px;background:#22c55e;border-radius:2px;margin-bottom:24px}}
.title{{font-size:{title_size}px;font-weight:800;line-height:1.24;color:#f8fafc;letter-spacing:-.5px;margin-bottom:22px}}
.subtitle{{font-size:{subtitle_size}px;color:#93a1b5;line-height:1.62;letter-spacing:.3px;margin-bottom:34px;max-width:640px}}
.meta{{position:absolute;left:76px;right:76px;bottom:42px;display:flex;align-items:center;gap:14px;color:#64748b;font-size:13px;letter-spacing:.5px;padding-top:18px;border-top:1px solid rgba(255,255,255,.09)}}
.chip{{border:1px solid rgba(34,197,94,.3);border-radius:3px;padding:4px 10px;color:#4ade80;font-size:11px;font-weight:600;letter-spacing:1.5px}}
.author{{color:#94a3b8}}
</style>
</head>
<body>
<main class="cover" data-cover-style="geek-black">
  <div class="glow"></div><div class="edge"></div>
  <section class="inner">
{tag_line}
    <div class="rule"></div>
    <h1 class="title">{title_html}</h1>
    <p class="subtitle">{subtitle}</p>
  </section>
  <div class="meta">{chip}<span class="author">{author}</span></div>
</main>
</body>
</html>"#
    )
}

pub(super) fn render_blueprint_cover(
    title: &str,
    subtitle: &str,
    author: &str,
    tag: Option<&str>,
) -> String {
    let tag = tag.unwrap_or("SYSTEM BLUEPRINT");
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
    <div class="tag">{tag}</div>
    <h1 class="title">{title}</h1>
    <p class="subtitle">{subtitle}</p>
    <div class="meta"><span class="stamp">ARCHITECTURE</span><span>{author}</span></div>
  </section>
</main>
</body>
</html>"#
    )
}

/// Splits an AiLab cover title into two display lines at a UTF-8 character
/// boundary.
///
/// `title` has already been HTML-escaped by `generate_cover_html`, so it may
/// contain multi-byte CJK/emoji characters alongside ASCII entities such as
/// `&amp;`. Neither a byte-count length check nor a fixed byte offset is safe
/// here: slicing at byte 15 used to panic on Chinese titles with
/// `byte index 15 is not a char boundary`, and could also cut an entity into a
/// broken `&am` + `p;` pair. Use character counts and `char_indices()` instead.
///
/// The `floor_char_boundary` API would express this more directly, but it is
/// only stable from Rust 1.91 while this crate's MSRV is 1.88.
pub(super) fn split_ai_lab_title(title: &str) -> (&str, Option<&str>) {
    // `find` returns the start byte of the separator, which is always a char
    // boundary; the offsets below skip the whole separator, never a partial
    // codepoint. A full-width colon is 3 UTF-8 bytes, an ASCII ": " is 2.
    if let Some(idx) = title.find('：') {
        return (&title[..idx], Some(&title[idx + '：'.len_utf8()..]));
    }
    if let Some(idx) = title.rfind(": ") {
        return (&title[..idx], Some(&title[idx + ": ".len()..]));
    }
    // Compare character counts, not bytes: six Chinese characters (18 bytes)
    // must not count as "long" while eighteen ASCII characters do.
    if title.chars().count() <= 18 {
        return (title, None);
    }
    // `char_indices().nth(15)` always lands on a char boundary, even when the
    // 15th character is a 3-byte CJK glyph or a 4-byte emoji.
    let cut = title
        .char_indices()
        .nth(15)
        .map_or(title.len(), |(byte_idx, _)| byte_idx);
    // Keep an `&amp;` / `&quot;` / `&#39;` entity whole: if an unclosed `&`
    // precedes the cut, move the cut back to that `&`.
    let cut = match title[..cut].rfind('&') {
        Some(amp) if !title[amp..cut].contains(';') => amp,
        _ => cut,
    };
    (&title[..cut], Some(&title[cut..]))
}

pub(super) fn render_ai_lab_cover(
    title: &str,
    subtitle: &str,
    author: &str,
    tag: Option<&str>,
) -> String {
    let tag_text = tag.unwrap_or("SOLANA · WEB3");
    let chip_text = tag_text.to_lowercase();
    // Split long title on 「：」/「: 」or after 15 characters, for readability.
    let (line1, line2) = split_ai_lab_title(title);
    let line2_text = line2.unwrap_or("");
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:'SF Pro Text',-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif;background:#0b0b18}}
.cover{{width:900px;height:500px;position:relative;overflow:hidden;background:radial-gradient(circle at 22% 24%,rgba(168,85,247,.3),transparent 26%),radial-gradient(circle at 78% 76%,rgba(168,85,247,.35),transparent 30%),linear-gradient(145deg,#0f1020 0%,#1a1040 52%,#2a1050 100%);color:#eef2ff;padding:62px 78px}}
.orb{{position:absolute;border-radius:50%}}
.orb.one{{right:72px;top:58px;width:150px;height:150px;border:1.5px solid rgba(168,85,247,.3);box-shadow:0 0 80px rgba(168,85,247,.25);background:radial-gradient(circle,rgba(168,85,247,.08),transparent)}}
.orb.two{{right:138px;top:112px;width:76px;height:76px;border:1px solid rgba(168,85,247,.2);box-shadow:0 0 40px rgba(168,85,247,.12)}}
.orb.three{{right:40px;bottom:100px;width:40px;height:40px;border:1px solid rgba(168,85,247,.1)}}
.trace{{position:absolute;left:80px;right:80px;bottom:70px;height:1px;background:linear-gradient(90deg,transparent,rgba(168,85,247,.6),rgba(234,179,8,.4),transparent)}}
.content{{position:relative;height:100%;display:flex;flex-direction:column;justify-content:center;max-width:620px}}
.tag{{display:inline-block;width:max-content;border:1px solid rgba(168,85,247,.5);border-radius:999px;padding:6px 14px;color:#c4b5fd;background:rgba(168,85,247,.12);font-size:10px;font-weight:800;letter-spacing:4px;text-transform:uppercase;margin-bottom:22px}}
.title{{font-size:50px;font-weight:900;line-height:1.12;color:#ffffff;letter-spacing:-0.5px;text-shadow:0 0 30px rgba(168,85,247,.28)}}
.title-line2{{font-size:42px;font-weight:900;line-height:1.12;color:#c4b5fd;letter-spacing:-0.5px;text-shadow:0 0 20px rgba(168,85,247,.18);margin-top:6px}}
.sub{{font-size:14px;color:#a5b4fc;margin-top:22px;letter-spacing:1px;opacity:.7}}
.meta{{margin-top:24px;display:flex;align-items:center;gap:10px;font-size:11px;color:#94a3b8}}
.chip{{color:#fbbf24;border-bottom:1px solid rgba(251,191,36,.3);padding-bottom:2px;font-weight:600}}
</style>
</head>
<body>
<main class="cover" data-cover-style="ai-lab">
  <div class="orb one"></div><div class="orb two"></div><div class="orb three"></div>
  <div class="trace"></div>
  <section class="content">
    <div class="tag">{tag_text}</div>
    <h1 class="title">{line1}</h1>
    <div class="title-line2">{line2_text}</div>
    <div class="sub">{subtitle}</div>
    <div class="meta"><span class="chip">{chip_text}</span><span>{author}</span></div>
  </section>
</main>
</body>
</html>"#
    )
}

pub(super) fn render_clean_cover(title: &str, subtitle: &str, author: &str) -> String {
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

pub(super) fn render_minimal_cover(title: &str, subtitle: &str, author: &str) -> String {
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

pub(super) fn render_warm_cover(title: &str, subtitle: &str, author: &str) -> String {
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

pub(super) fn render_serif_cover(title: &str, subtitle: &str, author: &str) -> String {
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

pub(super) fn render_gradient_cover(title: &str, subtitle: &str, author: &str) -> String {
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

pub(super) fn render_ink_cover(title: &str, subtitle: &str, author: &str) -> String {
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

pub(super) fn render_sunset_cover(title: &str, subtitle: &str, author: &str) -> String {
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

pub(super) fn render_forest_cover(title: &str, subtitle: &str, author: &str) -> String {
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

pub(super) fn render_workflow_cover(title: &str, subtitle: &str, author: &str) -> String {
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

// ── 内容驱动封面（Editorial / 编辑海报风）─────────────────────────────────────────
//
// 设计目标：根据文章标题+摘要自动匹配主题配色与图形母题，封面只保留"分类小标 + 大标题"
// 两段文字，弱化副标题/作者大块，避免文字过载。配色与母题由标题哈希做确定性扰动，
// 同一主题的不同文章看起来各不相同，但同一输入永远得到同一张图（可复现、可单测）。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Motif {
    Dots,
    Arcs,
    Bars,
    Waves,
    Grid,
    Bloom,
}

pub(super) struct CoverTheme {
    /// 封面上的中文分类小标（唯一的"元信息"文字）。
    pub(super) kicker: &'static str,
    /// 底色（深色，保证浅色文字可读）。
    pub(super) bg: &'static str,
    /// 网格渐变的 3 个光晕色。
    pub(super) mesh: [&'static str; 3],
    /// 主文字色（浅色）。
    pub(super) ink: &'static str,
    /// 强调色（小标/母题）。
    pub(super) accent: &'static str,
    /// 图形母题。
    pub(super) motif: Motif,
}

pub(super) const COVER_THEMES: &[CoverTheme] = &[
    CoverTheme {
        kicker: "科技",
        bg: "#0b1026",
        mesh: ["#4338ca", "#7c3aed", "#0ea5e9"],
        ink: "#f8fafc",
        accent: "#a5b4fc",
        motif: Motif::Grid,
    },
    CoverTheme {
        kicker: "自然",
        bg: "#04140f",
        mesh: ["#059669", "#10b981", "#22d3ee"],
        ink: "#ecfdf5",
        accent: "#6ee7b7",
        motif: Motif::Bloom,
    },
    CoverTheme {
        kicker: "成长",
        bg: "#1a0f04",
        mesh: ["#ea580c", "#f59e0b", "#fbbf24"],
        ink: "#fffbeb",
        accent: "#fcd34d",
        motif: Motif::Bars,
    },
    CoverTheme {
        kicker: "思考",
        bg: "#1c1814",
        mesh: ["#b45309", "#92400e", "#78716c"],
        ink: "#fafaf9",
        accent: "#d6c3a3",
        motif: Motif::Waves,
    },
    CoverTheme {
        kicker: "情感",
        bg: "#1a0712",
        mesh: ["#db2777", "#e11d48", "#f472b6"],
        ink: "#fff1f5",
        accent: "#fbcfe8",
        motif: Motif::Arcs,
    },
    CoverTheme {
        kicker: "设计",
        bg: "#04141f",
        mesh: ["#0ea5e9", "#06b6d4", "#38bdf8"],
        ink: "#f0f9ff",
        accent: "#7dd3fc",
        motif: Motif::Dots,
    },
    CoverTheme {
        kicker: "赛博",
        bg: "#0b0320",
        mesh: ["#ec4899", "#a855f7", "#06b6d4"],
        ink: "#faf5ff",
        accent: "#e879f9",
        motif: Motif::Grid,
    },
    CoverTheme {
        kicker: "旅途",
        bg: "#14100c",
        mesh: ["#d97706", "#f97316", "#fbbf24"],
        ink: "#fffbeb",
        accent: "#fcd34d",
        motif: Motif::Waves,
    },
    CoverTheme {
        kicker: "星辰",
        bg: "#060818",
        mesh: ["#6366f1", "#8b5cf6", "#38bdf8"],
        ink: "#f8fafc",
        accent: "#a5b4fc",
        motif: Motif::Arcs,
    },
];
/// 与 `COVER_THEMES` 顺序对应的主题关键词。命中越多越优先；全不命中时按标题哈希兜底。
pub(super) const THEME_KEYWORDS: &[&[&str]] = &[
    &[
        "ai",
        "人工智能",
        "模型",
        "代码",
        "编程",
        "rust",
        "技术",
        "算法",
        "agent",
        "智能体",
        "开发",
        "软件",
        "芯片",
        "数据",
        "程序",
        "系统",
    ],
    &[
        "自然", "生活", "旅行", "植物", "风景", "山", "海", "跑步", "运动", "咖啡", "日常", "散步",
        "城市", "街", "雨", "风",
    ],
    &[
        "成长", "效率", "方法", "习惯", "学习", "工作", "管理", "产品", "创业", "副业", "赚钱",
        "目标", "自律", "时间",
    ],
    &[
        "思考",
        "阅读",
        "写作",
        "读书",
        "哲学",
        "人生",
        "意义",
        "笔记",
        "认知",
        "反思",
        "观点",
        "想法",
        "读书笔记",
    ],
    &[
        "情感", "关系", "爱", "心理", "亲密", "孤独", "焦虑", "治愈", "家庭", "朋友", "情绪", "心",
        "陪伴",
    ],
    &[
        "设计", "审美", "排版", "视觉", "艺术", "配色", "品牌", "界面", "ui", "体验", "创意",
        "封面",
    ],
    &[
        "赛博",
        "cyber",
        "数字",
        "元宇宙",
        "区块链",
        "web3",
        "nft",
        "编程",
        "geek",
        "黑客",
        "加密",
        "钱包",
        "defi",
        "gpt",
    ],
    &[
        "旅行", "旅途", "远方", "自驾", "火车", "徒步", "骑行", "地图", "机场", "行", "road",
        "路线", "背包", "游",
    ],
    &[
        "星空", "宇宙", "天", "夜", "光", "月亮", "太阳", "星", "云", "科幻", "未来", "梦", "时空",
        "维度",
    ],
];

/// FNV-1a 32 位哈希，用于把标题映射成确定性的视觉种子。
/// 返回 (主题下标, 视觉种子)。无关键词命中时按哈希兜底选主题，保证任何文章都有封面。
pub(super) fn derive_cover_design(text: &str) -> (usize, u32) {
    let lower = text.to_lowercase();
    let mut best = 0usize;
    let mut best_score = 0i32;
    for (i, kws) in THEME_KEYWORDS.iter().enumerate() {
        let mut score = 0i32;
        for kw in *kws {
            if lower.contains(&kw.to_lowercase()) {
                score += 1;
            }
        }
        if score > best_score {
            best_score = score;
            best = i;
        }
    }
    let seed = fnv1a(text);
    if best_score == 0 {
        best = (seed as usize) % COVER_THEMES.len();
    }
    (best, seed)
}

/// 按母题生成一组半透明装饰图形（位置/尺寸/数量均由 seed 确定性决定）。
pub(super) fn render_motif_shapes(motif: Motif, accent: &str, seed: u32) -> String {
    let count = 7 + (seed % 6) as usize; // 7..12
    let mut out = String::with_capacity(count * 96);
    for i in 0..count {
        let s = seed
            .wrapping_mul(2_654_435_761)
            .wrapping_add(i as u32 * 40_503);
        let x = s % 100;
        let y = (s >> 7) % 100;
        let size = 22 + (s >> 14) % 92;
        let opacity = 0.05 + ((s >> 21) % 12) as f32 / 100.0;
        let shape = match motif {
            Motif::Dots | Motif::Grid => format!(
                "<span class=\"m\" style=\"left:{x}%;top:{y}%;width:{size}px;height:{size}px;border-radius:50%;background:{accent};opacity:{opacity:.2}\"></span>"
            ),
            Motif::Arcs | Motif::Bloom => format!(
                "<span class=\"m\" style=\"left:{x}%;top:{y}%;width:{size}px;height:{size}px;border-radius:50%;border:2px solid {accent};opacity:{opacity:.2}\"></span>"
            ),
            Motif::Bars => format!(
                "<span class=\"m\" style=\"left:{x}%;top:{y}%;width:3px;height:{size}px;background:{accent};opacity:{opacity:.2}\"></span>"
            ),
            Motif::Waves => format!(
                "<span class=\"m\" style=\"left:{x}%;top:{y}%;width:{size}px;height:{size}px;border-radius:50%;border-top:2px solid {accent};border-right:2px solid {accent};transform:rotate({}deg);opacity:{opacity:.2}\"></span>",
                (s >> 3) % 360
            ),
        };
        out.push_str(&shape);
    }
    out
}

pub(super) fn render_editorial_cover(
    title: &str,
    author: &str,
    theme_idx: usize,
    seed: u32,
) -> String {
    let theme = &COVER_THEMES[theme_idx];

    // 6 种布局模板，由 seed 确定性地选择，不同文章得到不同的布局结构。
    // Layout 0: 底部文字+光晕母题（原有布局）
    // Layout 1: 斜切对角分割，标题骑在分割线上
    // Layout 2: 居中徽章+边框构图
    // Layout 3: 不对称卡片，大字标题在左下
    // Layout 4: 全幅大字排版——超大标题充满画面中央，底部仅一线+作者
    // Layout 5: 水平色带分割，标题居中紧凑排列，大量留白
    match seed % 6 {
        0 => render_editorial_layout_a(title, author, theme, seed),
        1 => render_editorial_layout_b(title, author, theme, seed),
        2 => render_editorial_layout_c(title, author, theme, seed),
        3 => render_editorial_layout_d(title, author, theme, seed),
        4 => render_editorial_layout_e(title, author, theme, seed),
        _ => render_editorial_layout_f(title, author, theme, seed),
    }
}

/// Layout A: 底部文字 + 光晕 + 母题装饰（原有默认布局）。
pub(super) fn render_editorial_layout_a(
    title: &str,
    author: &str,
    theme: &CoverTheme,
    seed: u32,
) -> String {
    let positions: [(u32, u32); 3] = [
        (seed % 70, (seed >> 4) % 60),
        ((seed >> 8) % 55 + 30, (seed >> 12) % 50 + 30),
        ((seed >> 16) % 60 + 10, (seed >> 20) % 55 + 10),
    ];
    let mut blobs = String::with_capacity(3 * 160);
    for (i, &(x, y)) in positions.iter().enumerate() {
        let color = theme.mesh[i % theme.mesh.len()];
        blobs.push_str(&format!(
            "<span class=\"blob\" style=\"left:{x}%;top:{y}%;width:520px;height:520px;background:radial-gradient(circle at 32% 30%, {color}, transparent 70%)\"></span>"
        ));
    }

    let motif = render_motif_shapes(theme.motif, theme.accent, seed);

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif;background:{bg}}}
.cover{{width:900px;height:500px;position:relative;overflow:hidden;background:{bg}}}
.blob{{position:absolute;border-radius:50%;filter:blur(10px);mix-blend-mode:screen;transform:translate(-50%,-50%)}}
.m{{position:absolute}}
.grain{{position:absolute;inset:0;opacity:.06;mix-blend-mode:overlay;background-image:url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='140' height='140'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");pointer-events:none}}
.veil{{position:absolute;inset:0;background:linear-gradient(180deg,transparent 38%,rgba(0,0,0,.42))}}
.content{{position:absolute;left:72px;right:72px;bottom:58px;z-index:3}}
.kicker{{display:inline-block;font-size:12px;font-weight:800;letter-spacing:6px;color:{accent};margin-bottom:18px}}
.title{{font-size:46px;font-weight:900;line-height:1.2;color:{ink};letter-spacing:1px;max-width:690px;text-shadow:0 2px 22px rgba(0,0,0,.28)}}
.author{{margin-top:20px;font-size:13px;color:{ink};opacity:.68;letter-spacing:3px}}
</style>
</head>
<body><main class="cover" data-cover-style="editorial">
  {blobs}
  {motif}
  <div class="grain"></div>
  <div class="veil"></div>
  <div class="content"><div class="kicker">{kicker}</div><h1 class="title">{title}</h1><div class="author">{author}</div></div>
</main></body>
</html>"#,
        bg = theme.bg,
        accent = theme.accent,
        ink = theme.ink,
        kicker = theme.kicker,
        title = title,
        author = author,
        blobs = blobs,
        motif = motif,
    )
}

/// Layout B: 斜切对角分割。左上区块用强调色大块面，右下为底色，
/// 标题在右上区域交叠。更像当代电影/音乐海报的大胆切割版面。
pub(super) fn render_editorial_layout_b(
    title: &str,
    author: &str,
    theme: &CoverTheme,
    seed: u32,
) -> String {
    let angle = 10.0 + (seed % 6) as f64 * 5.0;

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif;background:{bg}}}
.cover{{width:900px;height:500px;position:relative;overflow:hidden;background:{bg}}}
.diagonal{{position:absolute;top:-60px;left:-60px;width:540px;height:620px;background:{accent};opacity:.24;transform:rotate({angle}deg);transform-origin:top left;mix-blend-mode:screen}}
.diagonal2{{position:absolute;top:80px;left:-60px;width:380px;height:540px;background:{ink};opacity:.05;transform:rotate({angle}deg);transform-origin:top left}}
.grain{{position:absolute;inset:0;opacity:.05;mix-blend-mode:overlay;background-image:url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='140' height='140'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");pointer-events:none}}
.content{{position:absolute;right:60px;top:50%;transform:translateY(-55%);text-align:right;z-index:2;max-width:460px}}
.kicker{{display:inline-block;font-size:11px;font-weight:800;letter-spacing:5px;color:{accent};margin-bottom:14px;opacity:.85}}
.title{{font-size:40px;font-weight:900;line-height:1.15;color:{ink};letter-spacing:1px;text-shadow:0 2px 20px rgba(0,0,0,.42)}}
.author{{margin-top:16px;font-size:12px;color:{ink};opacity:.55;letter-spacing:2px}}
.accent-line{{position:absolute;bottom:70px;left:60px;width:90px;height:2px;background:{accent};z-index:2}}
</style>
</head>
<body><main class="cover" data-cover-style="editorial">
  <div class="diagonal"></div>
  <div class="diagonal2"></div>
  <div class="grain"></div>
  <div class="accent-line"></div>
  <div class="content"><div class="kicker">{kicker}</div><h1 class="title">{title}</h1><div class="author">{author}</div></div>
</main></body>
</html>"#,
        bg = theme.bg,
        accent = theme.accent,
        ink = theme.ink,
        kicker = theme.kicker,
        title = title,
        author = author,
        angle = angle,
    )
}

/// Layout C: 居中徽章 + 边框。中央有圆形光晕徽章，标题在正下方，
/// 四角有装饰边框。偏向杂志封面气质。
pub(super) fn render_editorial_layout_c(
    title: &str,
    author: &str,
    theme: &CoverTheme,
    seed: u32,
) -> String {
    let badge_size = 120 + (seed % 6) * 8;

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif;background:{bg}}}
.cover{{width:900px;height:500px;position:relative;overflow:hidden;background:{bg}}}
.border-t{{position:absolute;top:20px;left:24px;right:24px;height:1px;background:{accent};opacity:.18}}
.border-b{{position:absolute;bottom:20px;left:24px;right:24px;height:1px;background:{accent};opacity:.18}}
.border-l{{position:absolute;top:20px;bottom:20px;left:24px;width:1px;background:{accent};opacity:.18}}
.border-r{{position:absolute;top:20px;bottom:20px;right:24px;width:1px;background:{accent};opacity:.18}}
.corner-tl{{position:absolute;top:20px;left:24px;width:24px;height:24px;border-top:2px solid {accent};border-left:2px solid {accent};opacity:.35}}
.corner-br{{position:absolute;bottom:20px;right:24px;width:24px;height:24px;border-bottom:2px solid {accent};border-right:2px solid {accent};opacity:.35}}
.badge{{position:absolute;top:90px;left:50%;transform:translateX(-50%);width:{badge_size}px;height:{badge_size}px;border-radius:50%;background:radial-gradient(circle at 38% 30%, {accent}, transparent 72%);opacity:.35}}
.badge-ring{{position:absolute;top:calc(90px - 8px);left:50%;transform:translateX(-50%);width:calc({badge_size}px + 16px);height:calc({badge_size}px + 16px);border-radius:50%;border:1px solid {accent};opacity:.24}}
.grain{{position:absolute;inset:0;opacity:.04;mix-blend-mode:overlay;background-image:url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='140' height='140'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");pointer-events:none}}
.content{{position:absolute;bottom:70px;left:0;right:0;text-align:center;z-index:3}}
.kicker{{display:inline-block;font-size:11px;font-weight:700;letter-spacing:8px;color:{accent};margin-bottom:12px;opacity:.8}}
.title{{font-size:38px;font-weight:900;line-height:1.2;color:{ink};letter-spacing:1px;max-width:660px;margin:0 auto;text-shadow:0 2px 22px rgba(0,0,0,.28)}}
.author{{margin-top:14px;font-size:12px;color:{ink};opacity:.55;letter-spacing:2px}}
</style>
</head>
<body><main class="cover" data-cover-style="editorial">
  <div class="border-t"></div><div class="border-b"></div><div class="border-l"></div><div class="border-r"></div>
  <div class="corner-tl"></div><div class="corner-br"></div>
  <div class="badge"></div><div class="badge-ring"></div>
  <div class="grain"></div>
  <div class="content"><div class="kicker">{kicker}</div><h1 class="title">{title}</h1><div class="author">{author}</div></div>
</main></body>
</html>"#,
        bg = theme.bg,
        accent = theme.accent,
        ink = theme.ink,
        kicker = theme.kicker,
        title = title,
        author = author,
        badge_size = badge_size,
    )
}

/// Layout D: 不对称卡片。超大标题在左下方，右上角有 CSS 绘制的几何装饰区。
/// 模拟当代设计杂志的"大字报"版面。
pub(super) fn render_editorial_layout_d(
    title: &str,
    author: &str,
    theme: &CoverTheme,
    seed: u32,
) -> String {
    let block_w = 160 + (seed % 7) * 20;
    let block_h = 160 + (seed >> 3) % 40;

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif;background:{bg}}}
.cover{{width:900px;height:500px;position:relative;overflow:hidden;background:{bg}}}
.art-block{{position:absolute;top:36px;right:40px;width:{block_w}px;height:{block_h}px;border:2px solid {accent};border-radius:4px;opacity:.18;transform:rotate(3deg)}}
.art-block-inner{{position:absolute;top:46px;right:50px;width:calc({block_w}px - 20px);height:calc({block_h}px - 20px);background:{accent};opacity:.07;border-radius:2px;transform:rotate(-2deg)}}
.art-circle{{position:absolute;bottom:100px;right:90px;width:50px;height:50px;border-radius:50%;border:1.5px solid {accent};opacity:.15}}
.grain{{position:absolute;inset:0;opacity:.05;mix-blend-mode:overlay;background-image:url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='140' height='140'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");pointer-events:none}}
.content{{position:absolute;bottom:64px;left:56px;z-index:3;max-width:620px}}
.kicker{{display:inline-block;font-size:10px;font-weight:800;letter-spacing:6px;color:{accent};margin-bottom:10px;opacity:.8}}
.title{{font-size:52px;font-weight:900;line-height:1.08;color:{ink};letter-spacing:-0.5px;text-shadow:0 2px 26px rgba(0,0,0,.32)}}
.author{{margin-top:18px;font-size:12px;color:{ink};opacity:.55;letter-spacing:2px}}
.accent-dot{{position:absolute;bottom:56px;right:60px;width:6px;height:6px;border-radius:50%;background:{accent};z-index:3}}
</style>
</head>
<body><main class="cover" data-cover-style="editorial">
  <div class="art-block"></div><div class="art-block-inner"></div><div class="art-circle"></div>
  <div class="grain"></div>
  <div class="content"><div class="kicker">{kicker}</div><h1 class="title">{title}</h1><div class="author">{author}</div></div>
  <div class="accent-dot"></div>
</main></body>
</html>"#,
        bg = theme.bg,
        accent = theme.accent,
        ink = theme.ink,
        kicker = theme.kicker,
        title = title,
        author = author,
        block_w = block_w,
        block_h = block_h,
    )
}

// ── tests ─────────────────────────────────────────────────────────────────────// ── tests ─────────────────────────────────────────────────────────────────────

/// Layout E: 全幅大字排版。超大标题几乎填满画面中央，字体粗壮且占据框架 2/3 以上高度，
/// 底部仅一条极细水平线和迷你作者名。模拟当代设计杂志的"大字报"风格。
pub(super) fn render_editorial_layout_e(
    title: &str,
    author: &str,
    theme: &CoverTheme,
    seed: u32,
) -> String {
    let line_y = 400 + (seed % 4) * 8;

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif;background:{bg}}}
.cover{{width:900px;height:500px;position:relative;overflow:hidden;background:{bg}}}
.grain{{position:absolute;inset:0;opacity:.04;mix-blend-mode:overlay;background-image:url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='140' height='140'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");pointer-events:none}}
.deco-line{{position:absolute;left:60px;right:60px;top:{line_y}px;height:1px;background:{accent};opacity:.25}}
.content{{position:absolute;inset:0;display:flex;flex-direction:column;justify-content:center;align-items:center;z-index:2;padding:60px}}
.kicker{{display:inline-block;font-size:10px;font-weight:800;letter-spacing:8px;color:{accent};margin-bottom:16px;opacity:.7}}
.title{{font-size:68px;font-weight:900;line-height:1.05;color:{ink};letter-spacing:-1px;text-align:center;max-width:780px;text-shadow:0 3px 32px rgba(0,0,0,.36)}}
.author{{position:absolute;bottom:32px;left:0;right:0;text-align:center;font-size:10px;color:{ink};opacity:.4;letter-spacing:3px}}
.layout-e-marker{{display:none}}
</style>
</head>
<body><main class="cover" data-cover-style="editorial">
  <div class="grain"></div>
  <div class="deco-line"></div>
  <div class="content"><div class="kicker">{kicker}</div><h1 class="title">{title}</h1></div>
  <div class="author">{author}</div>
  <div class="layout-e-marker">layout-e</div>
</main></body>
</html>"#,
        bg = theme.bg,
        accent = theme.accent,
        ink = theme.ink,
        kicker = theme.kicker,
        title = title,
        author = author,
        line_y = line_y,
    )
}

/// Layout F: 水平色带分割。画面被 2-3 根水平细线或色块分成几个平行的水平条带，
/// 标题紧凑居中排列在中央区域。大量留白，极简。
pub(super) fn render_editorial_layout_f(
    title: &str,
    author: &str,
    theme: &CoverTheme,
    seed: u32,
) -> String {
    let band1_top = 60 + (seed % 6) * 8;
    let band2_top = band1_top + 60 + (seed >> 3) % 20;

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif;background:{bg}}}
.cover{{width:900px;height:500px;position:relative;overflow:hidden;background:{bg}}}
.grain{{position:absolute;inset:0;opacity:.04;mix-blend-mode:overlay;background-image:url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='140' height='140'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");pointer-events:none}}
.band{{position:absolute;left:0;right:0;height:2px;background:{accent};opacity:.12}}
.band-1{{top:{band1_top}px}}
.band-2{{top:{band2_top}px}}
.band-3{{position:absolute;left:50px;right:50px;bottom:72px;height:1px;background:{accent};opacity:.08}}
.content{{position:absolute;inset:0;display:flex;flex-direction:column;justify-content:center;align-items:center;z-index:2;padding:80px 100px}}
.kicker{{display:inline-block;font-size:10px;font-weight:800;letter-spacing:8px;color:{accent};margin-bottom:14px;opacity:.7}}
.title{{font-size:42px;font-weight:800;line-height:1.15;color:{ink};letter-spacing:0.5px;text-align:center;max-width:680px;text-shadow:0 2px 24px rgba(0,0,0,.28)}}
.author{{margin-top:16px;font-size:11px;color:{ink};opacity:.5;letter-spacing:2px}}
.layout-f-marker{{display:none}}
</style>
</head>
<body><main class="cover" data-cover-style="editorial">
  <div class="grain"></div>
  <div class="band band-1"></div>
  <div class="band band-2"></div>
  <div class="band-3"></div>
  <div class="content"><div class="kicker">{kicker}</div><h1 class="title">{title}</h1><div class="author">{author}</div></div>
  <div class="layout-f-marker">layout-f</div>
</main></body>
</html>"#,
        bg = theme.bg,
        accent = theme.accent,
        ink = theme.ink,
        kicker = theme.kicker,
        title = title,
        author = author,
        band1_top = band1_top,
        band2_top = band2_top,
    )
}

// ── 内容驱动的高级海报风格（swiss / aurora / riso / noir / bauhaus）──────────
// 设计目标与 editorial 一致：只放"分类小标 + 大标题 + 小作者"，文字量极少；
// 配色/构图由文章标题+摘要确定性推导（同一文章永远同一张，不同文章各不相同）。

/// 颗粒质感底纹（内联 SVG 噪声），供 aurora / riso 复用。
pub(super) const GRAIN_DATA_URI: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='140' height='140'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E";

/// 浅底风格可用的强调色（在米白/纸色底上对比度充足）。
pub(super) const POSTER_ACCENTS: &[&str] = &[
    "#e11d48", "#2563eb", "#0d9488", "#7c3aed", "#ea580c", "#0891b2", "#65a30d", "#db2777",
];

/// 极光风格的三色光晕组合。
pub(super) const AURORA_PALETTES: &[(&str, &str, &str)] = &[
    ("#22d3ee", "#a78bfa", "#f472b6"),
    ("#34d399", "#60a5fa", "#c084fc"),
    ("#fbbf24", "#fb7185", "#8b5cf6"),
    ("#38bdf8", "#818cf8", "#22d3ee"),
    ("#f0abfc", "#5eead4", "#60a5fa"),
];

/// 丝网印刷的双色套印组合。
pub(super) const RISO_PAIRS: &[(&str, &str)] = &[
    ("#ff5a5f", "#2d5be3"),
    ("#ff7a00", "#00806a"),
    ("#e23e8f", "#3b3bff"),
    ("#f4c300", "#e2434b"),
    ("#00a0a0", "#f2545b"),
];

/// 海报类风格统一入口：按 seed 确定性挑配色，再分派到具体版式。
pub(super) fn render_poster_cover(
    style: CoverStyle,
    title: &str,
    author: &str,
    kicker: &str,
    seed: u32,
    subtitle: &str,
) -> String {
    match style {
        CoverStyle::Swiss => render_swiss_cover(
            title,
            author,
            kicker,
            POSTER_ACCENTS[seed as usize % POSTER_ACCENTS.len()],
            subtitle,
            false,
        ),
        CoverStyle::SwissDark => render_swiss_cover(
            title,
            author,
            kicker,
            POSTER_ACCENTS[(seed as usize + 2) % POSTER_ACCENTS.len()],
            subtitle,
            true,
        ),
        CoverStyle::Aurora => {
            let (c1, c2, c3) = AURORA_PALETTES[(seed as usize >> 3) % AURORA_PALETTES.len()];
            render_aurora_cover(title, author, kicker, c1, c2, c3, seed)
        }
        CoverStyle::Riso => {
            let (a, b) = RISO_PAIRS[(seed as usize >> 5) % RISO_PAIRS.len()];
            render_riso_cover(title, author, kicker, a, b, seed)
        }
        CoverStyle::Noir => render_noir_cover(title, author, kicker, seed),
        _ => render_bauhaus_cover(
            title,
            author,
            kicker,
            POSTER_ACCENTS[(seed as usize >> 11) % POSTER_ACCENTS.len()],
            seed,
        ),
    }
}

/// Swiss：瑞士国际主义网格——米白底、细网格线、超大粗体标题、单一强调色块。
pub(super) fn render_swiss_cover(
    title: &str,
    author: &str,
    kicker: &str,
    accent: &str,
    subtitle: &str,
    dark: bool,
) -> String {
    // 深色版沿用同一套网格，只换色板：近黑底 + 暖白字 + 亮色数据。
    let (bg, fg, accent) = if dark {
        ("#0d0f12", "#f2f0e9", "#14f195")
    } else {
        ("#f4f1ea", "#16181d", accent)
    };
    let line_opacity = if dark { ".09" } else { ".06" };
    // 副标题里带「469 字节」这类硬数据时，把数字提成右侧大字号锚点；
    // 抽不到就退回原来的装饰方块。
    let stat = extract_byte_stat(subtitle);
    let mark = match stat {
        Some(n) => format!(r#"<div class="stat"><b>{n}</b><span>BYTES</span></div>"#),
        None => r#"<div class="mark"></div>"#.to_owned(),
    };
    let sub = if subtitle.trim().is_empty() {
        String::new()
    } else {
        format!(r#"<div class="sub">{}</div>"#, subtitle)
    };
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif;background:{bg}}}
.cover{{width:900px;height:500px;position:relative;overflow:hidden;background:{bg}}}
.gline{{position:absolute;top:0;bottom:0;width:1px;background:{fg};opacity:{line_opacity}}}
.g1{{left:225px}}.g2{{left:450px}}.g3{{left:675px}}
.hline{{position:absolute;left:0;right:0;top:124px;height:1px;background:{fg};opacity:{line_opacity}}}
.mark{{position:absolute;right:96px;top:96px;width:92px;height:92px;background:{accent}}}
.stat{{position:absolute;right:72px;top:150px;text-align:right}}
.stat b{{display:block;font-size:136px;line-height:.82;font-weight:900;letter-spacing:-7px;color:{accent}}}
.stat span{{display:block;margin-top:12px;font-size:11px;font-weight:800;letter-spacing:4px;color:{fg};opacity:.55}}
.hair{{position:absolute;left:72px;top:96px;width:44px;height:3px;background:{accent}}}
.kicker{{position:absolute;left:128px;top:88px;font-size:12px;font-weight:800;letter-spacing:5px;color:{fg};opacity:.82}}
.title{{position:absolute;left:72px;bottom:132px;max-width:560px;font-size:52px;font-weight:900;line-height:1.12;letter-spacing:-.5px;color:{fg};word-break:keep-all;overflow-wrap:break-word}}
.sub{{position:absolute;left:72px;bottom:92px;font-size:15px;letter-spacing:.5px;color:{fg};opacity:.72}}
.author{{position:absolute;right:72px;bottom:44px;font-size:12px;letter-spacing:3px;color:{fg};opacity:.62}}
</style>
</head>
<body><main class="cover" data-cover-style="{marker}">
  <div class="gline g1"></div><div class="gline g2"></div><div class="gline g3"></div>
  <div class="hline"></div>{mark}<div class="hair"></div>
  <div class="kicker">{kicker}</div>
  <h1 class="title">{title}</h1>
  {sub}
  <div class="author">{author}</div>
</main></body>
</html>"#,
        bg = bg,
        fg = fg,
        accent = accent,
        line_opacity = line_opacity,
        marker = if dark { "swiss-dark" } else { "swiss" },
        kicker = kicker,
        title = title,
        mark = mark,
        sub = sub,
        author = author,
    )
}

/// 从文案里抠出「469 字节」中的数字，用于封面大字号数据锚点。
pub(super) fn extract_byte_stat(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut digits = String::new();
    for i in 0..chars.len() {
        if chars[i].is_ascii_digit() {
            digits.push(chars[i]);
            continue;
        }
        if !digits.is_empty() {
            let rest: String = chars[i..chars.len().min(i + 6)].iter().collect();
            if rest.trim_start().starts_with("字节") {
                return Some(digits);
            }
            digits.clear();
        }
    }
    None
}

/// Aurora：极光——深夜底 + 多层柔和光晕，标题居中，几乎无装饰。
pub(super) fn render_aurora_cover(
    title: &str,
    author: &str,
    kicker: &str,
    c1: &str,
    c2: &str,
    c3: &str,
    seed: u32,
) -> String {
    let x1 = 16 + (seed % 24);
    let y1 = 8 + ((seed >> 5) % 26);
    let x2 = 60 + ((seed >> 9) % 26);
    let y2 = 56 + ((seed >> 13) % 30);
    let x3 = 36 + ((seed >> 17) % 28);
    let y3 = 74 + ((seed >> 21) % 20);
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif;background:#070b18}}
.cover{{width:900px;height:500px;position:relative;overflow:hidden;background:#070b18}}
.blob{{position:absolute;border-radius:50%;filter:blur(72px);mix-blend-mode:screen;transform:translate(-50%,-50%)}}
.b1{{left:{x1}%;top:{y1}%;width:520px;height:520px;background:radial-gradient(circle at 42% 40%,{c1},transparent 68%)}}
.b2{{left:{x2}%;top:{y2}%;width:560px;height:560px;background:radial-gradient(circle at 50% 50%,{c2},transparent 68%)}}
.b3{{left:{x3}%;top:{y3}%;width:480px;height:480px;background:radial-gradient(circle at 55% 45%,{c3},transparent 70%)}}
.hair{{position:absolute;left:64px;right:64px;height:1px;background:linear-gradient(90deg,transparent,{c1},transparent);opacity:.55}}
.hair-t{{top:54px}}.hair-b{{bottom:54px}}
.grain{{position:absolute;inset:0;opacity:.05;mix-blend-mode:overlay;background-image:url("{grain}");pointer-events:none}}
.content{{position:absolute;inset:0;display:flex;flex-direction:column;align-items:center;justify-content:center;text-align:center;z-index:2;padding:0 110px}}
.kicker{{font-size:12px;font-weight:700;letter-spacing:9px;color:{c1};margin-bottom:20px}}
.title{{font-size:46px;font-weight:800;line-height:1.24;letter-spacing:1px;color:#f8fafc;text-shadow:0 4px 34px rgba(0,0,0,.4)}}
.author{{margin-top:22px;font-size:12px;letter-spacing:3px;color:#f8fafc;opacity:.5}}
</style>
</head>
<body><main class="cover" data-cover-style="aurora">
  <div class="blob b1"></div><div class="blob b2"></div><div class="blob b3"></div>
  <div class="hair hair-t"></div><div class="hair hair-b"></div>
  <div class="grain"></div>
  <div class="content"><div class="kicker">{kicker}</div><h1 class="title">{title}</h1><div class="author">{author}</div></div>
</main></body>
</html>"#,
        x1 = x1,
        y1 = y1,
        x2 = x2,
        y2 = y2,
        x3 = x3,
        y3 = y3,
        c1 = c1,
        c2 = c2,
        c3 = c3,
        grain = GRAIN_DATA_URI,
        kicker = kicker,
        title = title,
        author = author,
    )
}

/// Riso：丝网印刷——纸底 + 双色套印色块 + 噪点，标题带套印偏置阴影。
pub(super) fn render_riso_cover(
    title: &str,
    author: &str,
    kicker: &str,
    a: &str,
    b: &str,
    seed: u32,
) -> String {
    let rot = 8 + (seed % 14);
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif;background:#f7f2e6}}
.cover{{width:900px;height:500px;position:relative;overflow:hidden;background:#f7f2e6}}
.s1{{position:absolute;left:-70px;top:-96px;width:344px;height:344px;border-radius:50%;background:{a};opacity:.82;mix-blend-mode:multiply}}
.s2{{position:absolute;right:-56px;bottom:-64px;width:300px;height:300px;background:{b};opacity:.78;mix-blend-mode:multiply;transform:rotate({rot}deg)}}
.s3{{position:absolute;right:158px;top:64px;width:124px;height:124px;border-radius:50%;background:{b};opacity:.32;mix-blend-mode:multiply}}
.grain{{position:absolute;inset:0;opacity:.14;mix-blend-mode:multiply;background-image:url("{grain}");pointer-events:none}}
.content{{position:absolute;left:76px;top:50%;transform:translateY(-50%);z-index:2;max-width:560px}}
.kicker{{display:inline-block;background:{a};color:#fff;font-size:11px;font-weight:800;letter-spacing:4px;padding:5px 12px}}
.title{{margin-top:20px;font-size:50px;font-weight:900;line-height:1.12;letter-spacing:-.5px;color:#191a1f;text-shadow:4px 4px 0 {b}}}
.author{{margin-top:26px;font-size:12px;letter-spacing:3px;color:#191a1f;opacity:.55}}
</style>
</head>
<body><main class="cover" data-cover-style="riso">
  <div class="s1"></div><div class="s2"></div><div class="s3"></div>
  <div class="grain"></div>
  <div class="content"><div class="kicker">{kicker}</div><h1 class="title">{title}</h1><div class="author">{author}</div></div>
</main></body>
</html>"#,
        a = a,
        b = b,
        rot = rot,
        grain = GRAIN_DATA_URI,
        kicker = kicker,
        title = title,
        author = author,
    )
}

/// Noir：黑白电影——条纹光、胶片齿孔、细边框与衬线大标题。
pub(super) fn render_noir_cover(title: &str, author: &str, kicker: &str, seed: u32) -> String {
    let angle = 96 + (seed % 40);
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:Georgia,'Songti SC','Noto Serif SC',serif;background:#0a0a0b}}
.cover{{width:900px;height:500px;position:relative;overflow:hidden;background:#0a0a0b}}
.stripes{{position:absolute;inset:0;background:repeating-linear-gradient({angle}deg,rgba(255,255,255,.05) 0 2px,transparent 2px 44px)}}
.glow{{position:absolute;left:-10%;bottom:-58%;width:120%;height:118%;background:radial-gradient(ellipse at 50% 100%,rgba(231,217,184,.20),transparent 62%)}}
.frame{{position:absolute;inset:26px;border:1px solid rgba(245,245,244,.15)}}
.perf{{position:absolute;left:48px;top:70px;bottom:70px;width:9px;border-radius:3px;background:repeating-linear-gradient(180deg,rgba(245,245,244,.18) 0 10px,transparent 10px 36px)}}
.content{{position:absolute;left:104px;right:96px;top:50%;transform:translateY(-50%);z-index:2}}
.kicker{{font-size:11px;font-weight:800;letter-spacing:11px;color:#e7d9b8;opacity:.85;margin-bottom:18px}}
.title{{font-size:46px;font-weight:700;line-height:1.26;letter-spacing:2px;color:#f5f5f4}}
.author{{margin-top:24px;font-size:12px;letter-spacing:3px;color:#f5f5f4;opacity:.42}}
</style>
</head>
<body><main class="cover" data-cover-style="noir">
  <div class="stripes"></div><div class="glow"></div>
  <div class="frame"></div><div class="perf"></div>
  <div class="content"><div class="kicker">{kicker}</div><h1 class="title">{title}</h1><div class="author">{author}</div></div>
</main></body>
</html>"#,
        angle = angle,
        kicker = kicker,
        title = title,
        author = author,
    )
}

/// Bauhaus：包豪斯几何——原色圆 / 方 / 三角构成 + 大字标题。
pub(super) fn render_bauhaus_cover(
    title: &str,
    author: &str,
    kicker: &str,
    accent: &str,
    seed: u32,
) -> String {
    let rot = 12 + (seed % 22);
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif;background:#efe9dd}}
.cover{{width:900px;height:500px;position:relative;overflow:hidden;background:#efe9dd}}
.circle{{position:absolute;right:-46px;top:-64px;width:230px;height:230px;border-radius:50%;background:#d6402f}}
.tri{{position:absolute;left:648px;bottom:0;width:0;height:0;border-left:118px solid transparent;border-right:118px solid transparent;border-bottom:186px solid #1f4fa8}}
.sq{{position:absolute;right:126px;top:154px;width:96px;height:96px;background:#f2b705;transform:rotate({rot}deg)}}
.content{{position:absolute;left:76px;right:330px;top:50%;transform:translateY(-50%);z-index:2}}
.bar{{width:56px;height:6px;background:{accent};margin-bottom:18px}}
.kicker{{font-size:12px;font-weight:800;letter-spacing:6px;color:#141414;opacity:.62;margin-bottom:16px}}
.title{{font-size:46px;font-weight:900;line-height:1.12;letter-spacing:-.5px;color:#141414}}
.author{{margin-top:34px;font-size:12px;letter-spacing:3px;color:#141414;opacity:.5}}
</style>
</head>
<body><main class="cover" data-cover-style="bauhaus">
  <div class="circle"></div><div class="tri"></div><div class="sq"></div>
  <div class="content"><div class="bar"></div><div class="kicker">{kicker}</div><h1 class="title">{title}</h1><div class="author">{author}</div></div>
</main></body>
</html>"#,
        rot = rot,
        accent = accent,
        kicker = kicker,
        title = title,
        author = author,
    )
}
