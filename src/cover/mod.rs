//! Article cover generation: 22 styles, AI prompt derivation, HTML/PNG capture.
//!
//! 拆成四个文件，按职责而不是按行数：
//! - `mod.rs`：对外 API（`CoverStyle` 注册表、路径推导、`generate_cover_html` 分发、HTML 写盘）
//! - `styles.rs`：22 个 `render_*_cover` 模板与 editorial 的内容驱动配色
//! - `ai.rs`：AI 绘图 prompt 推导（`generate_ai_cover_prompt` / `generate_ai_cover`）
//! - `capture.rs`：调 Chrome 截图并把 PNG 原子替换到位
//!
//! 子模块的条目对 `mod.rs` 是 `pub(super)`：dispatcher 与测试需要它们，
//! crate 外部不需要——所以不升 `pub(crate)`。

mod ai;
mod capture;
mod styles;

// 外部只需要这两个：AI 封面落盘、截图。其余（22 个模板、prompt 推导细节）留在子模块里。
pub use ai::generate_ai_cover;
pub use capture::capture_cover_png;

use styles::*; // 22 个 render_* 模板与 editorial 的配色表，只给本模块的 dispatcher 用

/// FNV-1a：ai.rs 与 styles.rs 都要用同一个稳定哈希，放父模块避免互相依赖。
pub(super) fn fnv1a(text: &str) -> u32 {
    let mut hash: u32 = 0x811c9dc5;
    for b in text.bytes() {
        hash ^= b as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}

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
    /// 内容驱动、少文字的"编辑海报"风：根据文章标题+摘要自动匹配主题配色与图形母题。
    Editorial,
    /// AI 生成图片封面：用 OpenAI 等 AI 绘图 API 生成卡通/动漫/未来感风格封面图片。
    /// 需要配置 AI provider 和 API key，只在 `moonpub cover --style ai-art` 或 frontmatter `cover_style: ai-art` 时触发。
    AiArt,
    /// 瑞士国际主义网格：米白底、细网格线、超大粗体标题 + 单一强调色块，大留白。
    Swiss,
    /// 瑞士网格暗色版：近黑底、亮色数据锚点，适合深色公众号列表页。
    SwissDark,
    /// 极光渐变：深夜底色上的多层柔和光晕，标题居中，极简克制。
    Aurora,
    /// 丝网印刷：纸质底色 + 双色套印 + 噪点颗粒，复古印刷质感。
    Riso,
    /// 黑白电影：纯黑底、条纹光、胶片齿孔与衬线大标题。
    Noir,
    /// 包豪斯几何：原色圆 / 方 / 三角构成 + 大字标题。
    Bauhaus,
}

pub struct CoverArtifact {
    pub html: String,
    pub html_path: PathBuf,
}

pub fn style_from_name(name: Option<&str>) -> CoverStyle {
    name.and_then(style_lookup).unwrap_or(CoverStyle::Editorial)
}

/// 是否是已知的封面风格名（含别名）。
///
/// `style_from_name` 对未知名字会静默降级成 `editorial`——2026-08-25 D19 事故就是
/// 这样发生的（请求的风格不存在，封面悄悄换成默认模板，没人报错）。调用方可以
/// 用这个函数在降级前先把「名字写错了」说出来。
pub fn is_known_style(name: &str) -> bool {
    style_lookup(name).is_some()
}

/// 对外推荐的风格名（别名不在内），供 `layout-recipes` 与错误提示列出。
///
/// 别名（`poster` / `grid` / `film` 等）仍然可用，但不对外宣传——一个风格一个名字，
/// 否则用户不知道哪个是「正名」。
pub const STYLE_NAMES: &[&str] = &[
    "editorial",
    "swiss",
    "swiss-dark",
    "aurora",
    "riso",
    "noir",
    "bauhaus",
    "ai-art",
    "dark",
    "geek-black",
    "blueprint",
    "ai-lab",
    "clean",
    "minimal",
    "warm",
    "serif",
    "gradient",
    "literary",
    "ink",
    "sunset",
    "forest",
    "workflow",
];

pub fn style_names() -> &'static [&'static str] {
    STYLE_NAMES
}

fn style_lookup(name: &str) -> Option<CoverStyle> {
    let style = match name {
        "dark" => CoverStyle::Dark,
        "geek-black" | "geek_black" => CoverStyle::GeekBlack,
        "blueprint" => CoverStyle::Blueprint,
        "ai-lab" | "ai_lab" => CoverStyle::AiLab,
        "clean" => CoverStyle::Clean,
        "minimal" => CoverStyle::Minimal,
        "warm" => CoverStyle::Warm,
        "serif" => CoverStyle::Serif,
        "gradient" => CoverStyle::Gradient,
        "literary" => CoverStyle::Literary,
        "ink" => CoverStyle::Ink,
        "sunset" => CoverStyle::Sunset,
        "forest" => CoverStyle::Forest,
        "workflow" => CoverStyle::Workflow,
        "editorial" | "poster" | "content" => CoverStyle::Editorial,
        "ai-art" | "ai_art" | "cartoon" | "anime" => CoverStyle::AiArt,
        "swiss" | "grid" | "international" => CoverStyle::Swiss,
        "aurora" | "glow" | "mesh" => CoverStyle::Aurora,
        "riso" | "risograph" | "print" => CoverStyle::Riso,
        "noir" | "film" | "cinema" => CoverStyle::Noir,
        "bauhaus" | "geo" | "geometric" => CoverStyle::Bauhaus,
        "swiss-dark" | "swiss_dark" | "swissdark" => CoverStyle::SwissDark,
        _ => return None,
    };
    Some(style)
}

/// 命令行 `--style` > frontmatter `cover_style` > frontmatter `theme` > `editorial`。
///
/// `cover` 与 `ship` 必须共用这一条链：两条命令各写一套回退顺序时，同一篇文章
/// 会被渲染成两种风格的封面，而用户看不出是哪条命令「赢」了。`theme` 留在链上
/// 是历史行为（`theme: geek-black` 的文章本来就把正文主题当封面风格），
/// `cover_style` 则是给「正文用 notebook、封面要 swiss」这类搭配留的独立旋钮。
pub fn resolve_style<'a>(
    cli_style: Option<&'a str>,
    front: &'a crate::article::Frontmatter,
) -> (&'a str, CoverStyle) {
    let name = cli_style
        .or(front.cover_style.as_deref())
        .or(front.theme.as_deref())
        .unwrap_or("editorial");
    (name, style_from_name(Some(name)))
}

/// 封面副标题。`cover_subtitle` 是逐篇覆盖，缺省回落到 `digest`——
/// `cover` 与 `ship` 必须取同一个值，否则同一篇文章在两条命令下副标题不同。
pub fn resolve_subtitle(front: &crate::article::Frontmatter) -> &str {
    front
        .cover_subtitle
        .as_deref()
        .or(front.digest.as_deref())
        .unwrap_or("")
}

/// 显式请求了、但无法识别的风格名（`--style` 或 frontmatter `cover_style`）。
///
/// 只看显式来源：`theme` 是正文主题，`notebook` / `letter` 这类名字本来就不是
/// 封面风格，对它告警会让每篇设了正文主题的文章都报一次假警。真正需要提醒的是
/// 「我明说要用 swis」这种笔误——它现在会静默变成 editorial 封面。
pub fn unknown_requested_style<'a>(
    cli_style: Option<&'a str>,
    front: &'a crate::article::Frontmatter,
) -> Option<&'a str> {
    cli_style
        .or(front.cover_style.as_deref())
        .filter(|name| !is_known_style(name))
}

/// 显式请求的风格名不存在时构造的硬失败错误。
///
/// 调用方（`cover` / `ship`）一律用它 `return Err(...)`，不再「告警一句然后
/// 照旧生成 editorial」：那意味着用户明说了 `swis`，拿到的却是一张暖米褐
/// 封面（`editorial` 在禁用配色清单里），而进程 exit 0、什么都没坏。参数错
/// 就是参数错，早报早改。
pub fn unknown_style_error(name: &str) -> AppError {
    AppError::UnknownCoverStyle {
        name: name.to_owned(),
        hint: unknown_style_hint(name),
        count: STYLE_NAMES.len(),
    }
}

/// 「为什么这个名字不行」——分两种情况，提示完全不同。
///
/// 写错字（`swis`）要的是最近的那个名字；而 `notebook` / `paper` 这类是
/// **正文主题名**，用户不是打错字，是把两个清单搞混了，此时给「你是不是想写
/// notebook」毫无意义（`notebook` 本来就没有同名封面风格），要说清楚它们是
/// 两套名字。
fn unknown_style_hint(name: &str) -> String {
    if crate::theme::Theme::is_known_name(name) {
        format!(
            "\n  `{name}` is a body theme name, not a cover style — keep it on `theme:` and pick a cover style separately"
        )
    } else {
        closest_style_name(name)
            .map(|candidate| format!("\n  did you mean `{candidate}`?"))
            .unwrap_or_default()
    }
}

/// 与 `help::suggest` 同一套判据（距离 ≤2 或前缀匹配），只是把候选集换成
/// 封面风格名。别名不参与匹配——只推荐正名，否则用户永远学不到哪个才是它。
fn closest_style_name(name: &str) -> Option<&'static str> {
    let lowered = name.to_ascii_lowercase();
    if lowered.len() < 3 {
        return None;
    }
    let mut best: Option<(usize, &'static str)> = None;
    for candidate in STYLE_NAMES {
        let distance = crate::help::edit_distance(&lowered, candidate);
        let close_enough = distance <= 2 || candidate.starts_with(&lowered);
        if close_enough && best.is_none_or(|(best_distance, _)| distance < best_distance) {
            best = Some((distance, candidate));
        }
    }
    best.map(|(_, candidate)| candidate)
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

/// Generate a standalone HTML cover page from article frontmatter.
pub fn generate_cover_html(
    title: &str,
    subtitle: &str,
    author: &str,
    style: CoverStyle,
    tag: Option<&str>,
) -> String {
    // 内容驱动风（Editorial）需要原文做主题匹配，先保留未转义的标题+摘要。
    let design_text = format!("{title} {subtitle}");
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
        CoverStyle::Blueprint => render_blueprint_cover(&title, &subtitle, &author, tag.as_deref()),
        CoverStyle::AiLab => render_ai_lab_cover(&title, &subtitle, &author, tag.as_deref()),
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
        CoverStyle::Editorial => {
            let (theme_idx, seed) = derive_cover_design(&design_text);
            render_editorial_cover(&title, &author, theme_idx, seed)
        }
        CoverStyle::Swiss
        | CoverStyle::SwissDark
        | CoverStyle::Aurora
        | CoverStyle::Riso
        | CoverStyle::Noir
        | CoverStyle::Bauhaus => {
            let (theme_idx, seed) = derive_cover_design(&design_text);
            // 显式给了 tag 就用它（如 SOLANA · WEB3），否则回退到主题 kicker。
            let kicker = tag.as_deref().unwrap_or(COVER_THEMES[theme_idx].kicker);
            render_poster_cover(style, &title, &author, kicker, seed, &subtitle)
        }
        CoverStyle::AiArt => {
            "<!-- AiArt cover: use generate_ai_cover() to produce a PNG image, not HTML -->"
                .to_owned()
        }
    }
}

/// 把公众号标题拆成「封面主标题 + 副标题」。
///
/// 公众号标题常是「系列前缀：结论一，结论二」的结构，动辄 40–60 字。
/// 原样铺到 900×500 的画布上会挤成 4–5 行大字，主次不分、互相抢空间。
/// 这里按三步收敛：
///
/// 1. 剥掉系列前缀（冒号前含「第 N 篇」的那一段）
/// 2. 在第一个句读处切分，前半做主标题、后半做副标题
/// 3. 分别限长，超长截断并补省略号
///
/// 显式 `cover_title` / `cover_subtitle` 在调用方优先，不走这里。
fn cover_text(title: &str, subtitle: &str) -> (String, String) {
    let raw = title.trim();
    let subtitle = subtitle.trim();

    if raw.is_empty() {
        if !subtitle.is_empty() {
            return (truncate_chars(subtitle, MAIN_TITLE_MAX), String::new());
        }
        return (String::new(), String::new());
    }

    // 系列前缀剥离（对显式 title 也安全：无「篇：」则不剥离）
    let core = match raw.split_once('：') {
        Some((head, tail)) if head.contains('篇') && !tail.trim().is_empty() => tail.trim(),
        _ => raw,
    };

    // 调用方已显式提供副标题（cover_subtitle 或 digest）：主标题整段用，
    // 不再从 title 里的逗号切分副标题——否则显式副标题会被悄无声息地吃掉。
    // 2026-10-31 第 06/07 篇封面事故：cover_title 含逗号时，原逻辑把逗号
    // 后半当副标题，覆盖掉作者写的 cover_subtitle。
    if !subtitle.is_empty() {
        return (
            truncate_chars(core, MAIN_TITLE_MAX),
            truncate_chars(subtitle, SUBTITLE_MAX),
        );
    }

    // 无显式副标题：从第一个句读切分，后半作副标题（自动兜底）
    let (head, tail) = match core.find(['，', '。', '；', ';']) {
        Some(idx) => {
            let (h, t) = core.split_at(idx);
            (
                h.trim(),
                t.trim_start_matches(['，', '。', '；', ';']).trim(),
            )
        }
        None => (core, ""),
    };

    // 限长：主标题优先保证一行可读，副标题留一句说明
    let main = truncate_chars(head, MAIN_TITLE_MAX);
    let sub = truncate_chars(tail, SUBTITLE_MAX);
    (main, sub)
}

/// 封面主标题上限（中文按字符算）。38px 字号下约一行半，不挤压版式。
const MAIN_TITLE_MAX: usize = 22;
/// 封面副标题上限。15px 字号下约两行。
const SUBTITLE_MAX: usize = 34;

/// 按字符（非字节）截断，超长补省略号。
fn truncate_chars(input: &str, max: usize) -> String {
    let count = input.chars().count();
    if count <= max {
        return input.to_owned();
    }
    let mut out: String = input.chars().take(max).collect();
    out.push('…');
    out
}

fn escape_html(input: &str) -> String {
    crate::html::escape(input)
}

/// 文件名会被拼进 CSS 的 `url('...')`（AI 封面 HTML）。文件名来自文章的
/// `file_stem`，用户可控；一个单引号就能闭合 URL 字符串、注入任意 CSS
/// 声明（外链资源、伪装样式）。这里按 CSS 字符串口径转义会截断的字节：
/// 反斜杠、单引号、换行/回车与控制字符，避免闭合或折行。
fn escape_css_url_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\a "),
            '\r' => out.push_str("\\d "),
            c if c.is_control() => out.push_str(&format!("\\{:x} ", c as u32)),
            other => out.push(other),
        }
    }
    out
}

/// Generate a DALL-E / Midjourney-style prompt for AI cover art.
/// The prompt adapts to article content category, producing visually distinct
/// anime/sci-fi/illustrative images for different topics.
#[cfg(test)]
mod tests {
    use super::ai::*;
    use super::capture::*;
    use super::*;
    use crate::test_helpers::temp_root;

    #[test]
    fn css_url_string_escapes_quote_that_could_close_url() {
        // 单引号闭合 url('...') 后可注入任意 CSS；必须转成字面 \'。
        assert_eq!(escape_css_url_string("a'b"), r"a\'b");
        assert_eq!(escape_css_url_string("a\\b"), r"a\\b");
        assert_eq!(escape_css_url_string("a\nb"), r"a\a b");
        let payload = "a';background:url(//evil)#.png";
        let escaped = escape_css_url_string(payload);
        // 单引号不是删掉，而是转成 CSS 转义 \'；关键是每个引号前都必须有反斜杠，
        // 否则仍能闭合 url('...') 注入声明。
        assert!(
            escaped
                .match_indices('\'')
                .all(|(i, _)| i > 0 && escaped.as_bytes()[i - 1] == b'\\'),
            "存在未转义的单引号：{escaped}"
        );
        assert!(!escaped.contains('\n'));
    }

    #[test]
    fn css_url_string_leaves_plain_name_untouched() {
        assert_eq!(
            escape_css_url_string("my-article.cover.png"),
            "my-article.cover.png"
        );
    }

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
    fn cover_text_strips_series_prefix_and_splits_on_comma() {
        // 无显式副标题时，从完整标题剥系列前缀 + 逗号切分，后半作副标题
        let (main, sub) = cover_text(
            "用 Rust 写公众号发布工具（第 1 篇）：第一个提交塞了 4967 行，四个月后 lib.rs 只剩 131 行",
            "",
        );
        assert_eq!(main, "第一个提交塞了 4967 行");
        assert_eq!(sub, "四个月后 lib.rs 只剩 131 行");
    }

    #[test]
    fn cover_text_keeps_short_title_intact() {
        let (main, sub) = cover_text("我的第一篇 MoonPub 文章", "用 MoonPub 把 Markdown 变成草稿");
        assert_eq!(main, "我的第一篇 MoonPub 文章");
        assert_eq!(sub, "用 MoonPub 把 Markdown 变成草稿");
    }

    #[test]
    fn cover_text_falls_back_to_digest_when_no_tail() {
        let (main, sub) = cover_text("覆盖率 71.99% 是假的", "总量好看但核心链路只有 1.6%");
        assert_eq!(main, "覆盖率 71.99% 是假的");
        assert_eq!(sub, "总量好看但核心链路只有 1.6%");
    }

    #[test]
    fn cover_text_keeps_explicit_subtitle_over_title_comma() {
        // 2026-10-31 封面事故回归：cover_title 含逗号时，显式 cover_subtitle
        // 不能被逗号后半覆盖
        let (main, sub) = cover_text("72% 覆盖率，是假的", "核心链路真实只有 1.6%");
        assert_eq!(main, "72% 覆盖率，是假的");
        assert_eq!(sub, "核心链路真实只有 1.6%");

        let (main, sub) = cover_text("本地全绿，CI 全红", "测试通过的真相是环境凑巧");
        assert_eq!(main, "本地全绿，CI 全红");
        assert_eq!(sub, "测试通过的真相是环境凑巧");
    }

    #[test]
    fn cover_text_truncates_rather_than_overflowing() {
        // 没有标点可切的无空格长标题，必须限长，否则封面会溢出
        let long = "一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十";
        let (main, sub) = cover_text(long, "");
        assert_eq!(main.chars().count(), MAIN_TITLE_MAX + 1); // 含省略号
        assert!(main.ends_with('…'));
        assert!(sub.is_empty());
    }

    #[test]
    fn read_cover_style_detects_marker() {
        // 2026-08-25：ship 封面风格校验的读回函数
        let html = "<html><head><style>…</style></head><body><main class=\"cover\" data-cover-style=\"geek-black\"><div class=\"tag\">TECH · NOTES</div></main></body></html>";
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
        // 标题控制在 MAIN_TITLE_MAX 内，避免被截断逻辑干扰转义断言。
        let html = generate_cover_html(
            r#"R&<W>"d""#,
            "A > B's note",
            "<script>alert(1)</script>",
            CoverStyle::Literary,
            None,
        );

        assert!(html.contains("R&amp;&lt;W&gt;&quot;d&quot;"));
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
    fn style_from_name_defaults_to_editorial() {
        assert_eq!(style_from_name(Some("dark")), CoverStyle::Dark);
        assert_eq!(style_from_name(Some("geek-black")), CoverStyle::GeekBlack);
        assert_eq!(style_from_name(Some("geek_black")), CoverStyle::GeekBlack);
        assert_eq!(style_from_name(Some("blueprint")), CoverStyle::Blueprint);
        assert_eq!(style_from_name(Some("ai-lab")), CoverStyle::AiLab);
        assert_eq!(style_from_name(Some("ai_lab")), CoverStyle::AiLab);
        assert_eq!(style_from_name(Some("clean")), CoverStyle::Clean);
        assert_eq!(style_from_name(Some("workflow")), CoverStyle::Workflow);
        assert_eq!(style_from_name(Some("editorial")), CoverStyle::Editorial);
        assert_eq!(style_from_name(Some("poster")), CoverStyle::Editorial);
        assert_eq!(style_from_name(Some("content")), CoverStyle::Editorial);
        assert_eq!(style_from_name(Some("unknown")), CoverStyle::Editorial);
        assert_eq!(style_from_name(None), CoverStyle::Editorial);
    }

    #[test]
    fn unknown_style_name_is_detectable_before_silent_fallback() {
        // style_from_name 对未知名字静默降级成 editorial；调用方靠这个判断能否告警，
        // 否则 `cover_style: swis` 这种笔误只会得到一张默认封面，没有任何提示。
        assert!(is_known_style("swiss"));
        assert!(is_known_style("geek_black"));
        assert!(is_known_style("editorial"));
        assert!(!is_known_style("swis"));
        assert!(!is_known_style(""));
    }

    fn front_with(theme: Option<&str>, cover_style: Option<&str>) -> crate::article::Frontmatter {
        crate::article::Frontmatter {
            theme: theme.map(str::to_owned),
            cover_style: cover_style.map(str::to_owned),
            ..Default::default()
        }
    }

    #[test]
    fn resolve_style_prefers_cli_then_cover_style_then_theme() {
        let front = front_with(Some("notebook"), Some("swiss"));
        assert_eq!(resolve_style(Some("noir"), &front).0, "noir");
        // 正文用 notebook、封面用 swiss 是合法搭配：cover_style 要压过 theme。
        assert_eq!(resolve_style(None, &front), ("swiss", CoverStyle::Swiss));

        // 没有 cover_style 时沿用 theme（历史行为：theme: geek-black 也决定封面）。
        let theme_only = front_with(Some("geek-black"), None);
        assert_eq!(
            resolve_style(None, &theme_only),
            ("geek-black", CoverStyle::GeekBlack)
        );

        let none_at_all = front_with(None, None);
        assert_eq!(
            resolve_style(None, &none_at_all),
            ("editorial", CoverStyle::Editorial)
        );
    }

    #[test]
    fn resolve_subtitle_prefers_cover_subtitle_over_digest() {
        let mut front = front_with(None, None);
        front.digest = Some("摘要".to_owned());
        assert_eq!(resolve_subtitle(&front), "摘要");

        front.cover_subtitle = Some("一句话".to_owned());
        assert_eq!(resolve_subtitle(&front), "一句话");

        let empty = front_with(None, None);
        assert_eq!(resolve_subtitle(&empty), "");
    }

    #[test]
    fn unknown_requested_style_ignores_body_theme_fallbacks() {
        // theme 是正文主题：notebook 不是封面风格，但也不该因此告警。
        let theme_only = front_with(Some("notebook"), None);
        assert_eq!(unknown_requested_style(None, &theme_only), None);

        let typo = front_with(Some("notebook"), Some("swis"));
        assert_eq!(unknown_requested_style(None, &typo), Some("swis"));

        let cli_typo = front_with(None, None);
        assert_eq!(
            unknown_requested_style(Some("swis"), &cli_typo),
            Some("swis")
        );

        let ok = front_with(None, Some("swiss"));
        assert_eq!(unknown_requested_style(None, &ok), None);
    }

    #[test]
    fn unknown_style_error_names_the_typo_and_suggests_the_closest_style() {
        // `swis` 少一个 s：报错要带上原文（用户才知道自己敲的是什么）
        // 和最近的那个名字（用户才知道该改成什么）。
        let msg = unknown_style_error("swis").to_string();
        assert!(
            msg.lines().next().unwrap_or_default() == "unknown cover style: swis",
            "{msg}"
        );
        assert!(msg.contains("did you mean `swiss`?"), "{msg}");
        // 22 个正名的数量写进提示，避免用户去数文档。
        assert!(msg.contains(&STYLE_NAMES.len().to_string()), "{msg}");
        assert!(msg.contains("layout-recipes"), "{msg}");
    }

    #[test]
    fn unknown_style_error_explains_a_theme_name_misplaced_as_a_style() {
        // `notebook` 是**正文主题**，不是打错字：给「你是不是想写 notebook」
        // 等于原地打转（它本来就没有同名封面风格）。这时要讲清是两套名字。
        assert!(!is_known_style("notebook"));
        let msg = unknown_style_error("notebook").to_string();
        assert!(msg.contains("body theme name"), "{msg}");
        assert!(msg.contains("`theme:`"), "{msg}");
        assert!(!msg.contains("did you mean"), "{msg}");
    }

    #[test]
    fn unknown_style_error_omits_a_useless_suggestion_for_far_off_names() {
        // 完全不像的名字不要硬凑一个「你是不是想写」——那比不猜更让人困惑。
        let msg = unknown_style_error("banana").to_string();
        assert!(msg.contains("unknown cover style: banana"), "{msg}");
        assert!(!msg.contains("did you mean"), "{msg}");
        // 但仍然要给出唯一确定的下一步。
        assert!(msg.contains("layout-recipes"), "{msg}");
    }

    #[test]
    fn closest_style_name_only_ever_returns_published_names() {
        // 别名（poster / grid / film）不参与推荐：推荐别名等于教用户写不该公开的名字。
        for typo in ["swis", "swiss-dar", "blueprin", "minima", "geek-blac"] {
            let candidate = closest_style_name(typo).unwrap_or_else(|| panic!("{typo}"));
            assert!(
                STYLE_NAMES.contains(&candidate),
                "`{typo}` 被推荐成别名 `{candidate}`"
            );
            assert!(is_known_style(candidate));
        }
        // 短到无法判断的输入不猜。
        assert_eq!(closest_style_name("ab"), None);
        assert_eq!(closest_style_name(""), None);
    }

    #[test]
    fn frontmatter_cover_style_reaches_the_rendered_html() {
        // 把「frontmatter → resolve_style → 渲染」串起来测：只断言 resolve_style
        // 返回值的话，渲染入口哪天改回只读 theme 也发现不了。
        let front = crate::article::parse_frontmatter(
            "---\ncover_style: swiss\ntheme: notebook\ndigest: 摘要\n---\n",
        );
        let (name, style) = resolve_style(None, &front);

        assert_eq!(name, "swiss");
        assert_eq!(style, CoverStyle::Swiss);

        let html = generate_cover_html("标题", resolve_subtitle(&front), "作者", style, None);
        assert!(html.contains(r#"data-cover-style="swiss""#), "{html}");
        assert!(html.contains("摘要"), "swiss 应渲染副标题：{html}");
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
        assert!(html.contains("TECH · NOTES"));
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
            "Agent 实验",
            "记录一次 AI 工程实践",
            "Test Author",
            CoverStyle::AiLab,
            None,
        );

        assert!(html.contains("data-cover-style=\"ai-lab\""));
        assert!(html.contains("SOLANA · WEB3"));
        assert!(html.contains("solana · web3"));
        assert!(html.contains("Agent 实验"));
    }

    /// Regression: `render_ai_lab_cover` used to slice the escaped title at a
    /// fixed byte offset 15. These titles all have byte 15 inside a multi-byte
    /// character, so the old code panicked with
    /// `byte index 15 is not a char boundary` on `moonpub cover --style ai-lab`.
    #[test]
    fn ai_lab_cover_handles_multibyte_titles_without_panicking() {
        let titles = [
            "A一二三四五六七八",
            "R&D 团队的工程实践复盘记录",
            "AI 时代的公众号写作实践总结",
            "🚀🚀🚀🚀 用 Rust 写发布工具",
            "一二三四五六七八九十",
        ];
        for title in titles {
            let html = generate_cover_html(title, "副标题", "作者", CoverStyle::AiLab, None);
            assert!(
                html.contains("<!DOCTYPE html>"),
                "ai-lab failed for title: {title}"
            );
        }
    }

    #[test]
    fn ai_lab_title_split_preserves_characters_and_html_entities() {
        // Short titles (<= 18 chars) stay on one line even when their byte
        // length exceeds 18 — six Chinese chars are 18 bytes.
        let (line1, line2) = split_ai_lab_title("一二三四五六");
        assert_eq!(line1, "一二三四五六");
        assert!(line2.is_none());

        // Long titles split after 15 characters, never mid-codepoint, and the
        // two lines always reassemble into the original escaped title.
        let long = "A一二三四五六七八九十更多内容继续补充文字";
        assert!(long.chars().count() > 18);
        let (line1, line2) = split_ai_lab_title(long);
        assert_eq!(line1.chars().count(), 15);
        assert_eq!(format!("{line1}{}", line2.unwrap()), long);

        // An HTML entity straddling the cut must stay whole. Here the 15th
        // character falls inside `&amp;`, so the cut retreats to the `&`.
        let with_entity = "AAAAAAAAAAAAA&amp;中文内容继续补充";
        let (line1, line2) = split_ai_lab_title(with_entity);
        assert_eq!(line1, "AAAAAAAAAAAAA");
        assert_eq!(line2, Some("&amp;中文内容继续补充"));
        assert_eq!(format!("{line1}{}", line2.unwrap()), with_entity);

        // Separator branches still take precedence.
        assert_eq!(split_ai_lab_title("系列篇：正文内容").0, "系列篇");
        assert_eq!(split_ai_lab_title("Prefix: suffix").0, "Prefix");
    }

    /// Every cover style receives the same escaped title, so a new style that
    /// reintroduces a fixed byte slice would be caught here.
    #[test]
    fn all_cover_styles_handle_multibyte_titles() {
        let titles = [
            "A一二三四五六七八",
            "R&D 团队的工程实践复盘记录",
            "🚀🚀 中文 & 实体 <测试> 更多内容",
        ];
        for style in [
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
            CoverStyle::Editorial,
            CoverStyle::Swiss,
            CoverStyle::Aurora,
            CoverStyle::Riso,
            CoverStyle::Noir,
            CoverStyle::Bauhaus,
        ] {
            for title in titles {
                let html = generate_cover_html(title, "副标题", "作者", style, None);
                assert!(
                    html.contains("<!DOCTYPE html>"),
                    "{style:?} failed for title: {title}"
                );
            }
        }
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
    fn all_twenty_styles_generate_html() {
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
            CoverStyle::Editorial,
            CoverStyle::Swiss,
            CoverStyle::Aurora,
            CoverStyle::Riso,
            CoverStyle::Noir,
            CoverStyle::Bauhaus,
        ];
        for &style in &styles {
            let html = generate_cover_html("T", "S", "A", style, None);
            assert!(html.contains("<!DOCTYPE html>"), "{style:?} 未产出 HTML");
        }
    }

    #[test]
    fn style_from_name_maps_poster_styles() {
        assert_eq!(style_from_name(Some("swiss")), CoverStyle::Swiss);
        assert_eq!(style_from_name(Some("grid")), CoverStyle::Swiss);
        assert_eq!(style_from_name(Some("aurora")), CoverStyle::Aurora);
        assert_eq!(style_from_name(Some("glow")), CoverStyle::Aurora);
        assert_eq!(style_from_name(Some("riso")), CoverStyle::Riso);
        assert_eq!(style_from_name(Some("risograph")), CoverStyle::Riso);
        assert_eq!(style_from_name(Some("noir")), CoverStyle::Noir);
        assert_eq!(style_from_name(Some("cinema")), CoverStyle::Noir);
        assert_eq!(style_from_name(Some("bauhaus")), CoverStyle::Bauhaus);
        assert_eq!(style_from_name(Some("geometric")), CoverStyle::Bauhaus);
        assert_eq!(style_from_name(Some("swiss-dark")), CoverStyle::SwissDark);
    }

    #[test]
    fn poster_styles_emit_own_style_marker() {
        // ship 依赖 data-cover-style 做静默 fallback 检测，每个新风格必须自报家门。
        for (style, name) in [
            (CoverStyle::Swiss, "swiss"),
            (CoverStyle::SwissDark, "swiss-dark"),
            (CoverStyle::Aurora, "aurora"),
            (CoverStyle::Riso, "riso"),
            (CoverStyle::Noir, "noir"),
            (CoverStyle::Bauhaus, "bauhaus"),
        ] {
            let html = generate_cover_html("测试标题", "摘要", "作者", style, None);
            assert!(
                html.contains(&format!("data-cover-style=\"{name}\"")),
                "{name} 缺少 data-cover-style 标记"
            );
            assert!(html.contains("测试标题"));
            if matches!(style, CoverStyle::Swiss | CoverStyle::SwissDark) {
                // swiss 两个版本都有副标题位（标题 + 一句话说明 + 署名三层）。
                assert!(html.contains("摘要"), "{name} 应渲染副标题");
            } else {
                assert!(!html.contains("摘要"), "{name} 不应渲染副标题文字");
            }
        }
    }

    #[test]
    fn swiss_promotes_a_byte_count_to_the_number_anchor() {
        let html = generate_cover_html(
            "第一个提交 4967 行",
            "四个多月后只剩 469 字节",
            "作者",
            CoverStyle::Swiss,
            None,
        );
        // 硬数据提到右侧大字号锚点，标题里的 4967 不参与提取（只看副标题）。
        assert!(html.contains(r#"<div class="stat"><b>469</b>"#), "{html}");
        assert!(html.contains("BYTES"), "{html}");
        assert!(
            !html.contains(r#"<div class="mark"></div>"#),
            "有硬数据时不应再画装饰方块"
        );
    }

    #[test]
    fn swiss_without_a_byte_count_keeps_the_plain_mark() {
        let html = generate_cover_html(
            "标题",
            "四个多月后只剩一半",
            "作者",
            CoverStyle::Swiss,
            None,
        );
        assert!(html.contains(r#"<div class="mark"></div>"#), "{html}");
        assert!(
            !html.contains(r#"class="stat""#),
            "抽不到数字就不要留空锚点"
        );
    }

    #[test]
    fn swiss_dark_inverts_the_palette_but_keeps_the_grid() {
        let light = generate_cover_html("标题", "", "作者", CoverStyle::Swiss, None);
        let dark = generate_cover_html("标题", "", "作者", CoverStyle::SwissDark, None);

        assert!(light.contains("background:#f4f1ea"));
        assert!(dark.contains("background:#0d0f12"));
        // 同一套网格：三条竖线 + 一条横线在深色版里仍然存在，只是颜色换成暖白。
        assert_eq!(dark.matches("class=\"gline").count(), 3);
        assert!(dark.contains(r#"<div class="hline"></div>"#));
        assert!(dark.contains("color:#f2f0e9"));
    }

    #[test]
    fn style_names_are_all_valid_and_distinct() {
        // 对外清单必须和 style_lookup 对齐：漏注册的名字会被用户直接抄进 frontmatter，
        // 两个名字指向同一风格则说明清单里混进了别名。
        let names = style_names();
        assert_eq!(names.len(), 22, "新增风格时同步这段计数与 README");
        let mut seen = Vec::new();
        for name in names {
            assert!(is_known_style(name), "{name} 未在 style_lookup 注册");
            let style = style_from_name(Some(name));
            assert!(
                !seen.contains(&style),
                "{name} 与其他名字指向同一个风格（清单里混进了别名）"
            );
            seen.push(style);
        }
    }

    #[test]
    fn extract_byte_stat_reads_only_the_number_followed_by_字节()
    -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            extract_byte_stat("四个多月后只剩 469 字节").as_deref(),
            Some("469")
        );
        assert_eq!(extract_byte_stat("源码 12 字节").as_deref(), Some("12"));
        assert_eq!(
            extract_byte_stat("只剩 469 字节，另有 4967 行代码").as_deref(),
            Some("469")
        );
        assert_eq!(extract_byte_stat("一共 4967 行"), None);
        // 英文单位不认：宁可退回装饰方块，也不要猜错单位。
        assert_eq!(extract_byte_stat("only 469 bytes left"), None);
        assert_eq!(extract_byte_stat(""), None);
        Ok(())
    }

    #[test]
    fn poster_styles_are_deterministic_and_content_aware() {
        for style in [
            CoverStyle::Swiss,
            CoverStyle::Aurora,
            CoverStyle::Riso,
            CoverStyle::Noir,
            CoverStyle::Bauhaus,
        ] {
            let a = generate_cover_html("Rust 异步编程实战", "摘要", "作者", style, None);
            let b = generate_cover_html("Rust 异步编程实战", "摘要", "作者", style, None);
            assert_eq!(a, b, "{style:?} 同一输入应产出同一封面");

            let c = generate_cover_html("亲密关系里的孤独", "摘要", "作者", style, None);
            assert_ne!(a, c, "{style:?} 不同内容应产出不同封面");
        }
    }

    #[test]
    fn editorial_cover_derives_kicker_from_content_and_minimal_text() {
        // 科技类标题应匹配"科技"母题，且只显示分类小标+标题，不堆副标题文字。
        let html = generate_cover_html(
            "Rust 异步编程实战",
            "讲清 await 与运行时的关系",
            "Test Author",
            CoverStyle::Editorial,
            None,
        );
        assert!(html.contains("data-cover-style=\"editorial\""));
        assert!(html.contains("科技")); // 由内容推导出的分类小标
        assert!(html.contains("Rust 异步编程实战"));
        assert!(!html.contains("讲清 await 与运行时的关系")); // 副标题不作为文字呈现
        // 4 种布局模板共享的共性元素
        assert!(html.contains("feTurbulence") || html.contains("grain")); // 颗粒质感
        // 至少含一种布局特征
        let layout_a = html.contains("radial-gradient");
        let layout_b = html.contains("diagonal");
        let layout_c = html.contains("border-t") && html.contains("badge");
        let layout_d = html.contains("art-block");
        let layout_e = html.contains("layout-e-marker");
        let layout_f = html.contains("layout-f-marker");
        // Diagnostic slices must be char-safe too: the HTML contains Chinese text,
        // so a fixed byte offset (especially `len() - 80`) could panic while
        // formatting the very failure we are trying to explain.
        let head_end = html.char_indices().nth(80).map_or(html.len(), |(i, _)| i);
        let tail_start = html.char_indices().rev().nth(80).map_or(0, |(i, _)| i);
        assert!(
            layout_a || layout_b || layout_c || layout_d || layout_e || layout_f,
            "cover must contain at least one layout feature: {}..{}",
            &html[..head_end],
            &html[tail_start..],
        );
    }

    #[test]
    fn editorial_cover_is_deterministic_for_same_input() {
        let a = generate_cover_html("同一主题文章", "摘要", "作者", CoverStyle::Editorial, None);
        let b = generate_cover_html("同一主题文章", "摘要", "作者", CoverStyle::Editorial, None);
        assert_eq!(a, b);
    }

    #[test]
    fn editorial_cover_falls_back_to_hash_theme_when_no_keywords() {
        // 全无关键词命中时仍按哈希兜底选主题，且一定有封面。
        let html = generate_cover_html("zxqwvbnm", "", "作者", CoverStyle::Editorial, None);
        assert!(html.contains("data-cover-style=\"editorial\""));
        assert!(html.contains("class=\"kicker\""));
    }

    #[test]
    fn editorial_cover_distinct_articles_look_different() {
        let a = generate_cover_html(
            "AI 智能体开发笔记",
            "模型与工具调用",
            "作者",
            CoverStyle::Editorial,
            None,
        );
        let b = generate_cover_html(
            "周末爬山看海日记",
            "自然与日常",
            "作者",
            CoverStyle::Editorial,
            None,
        );
        assert_ne!(a, b); // 不同内容 → 不同配色/母题
    }

    #[test]
    fn style_from_name_maps_ai_art() {
        assert_eq!(style_from_name(Some("ai-art")), CoverStyle::AiArt);
        assert_eq!(style_from_name(Some("ai_art")), CoverStyle::AiArt);
        assert_eq!(style_from_name(Some("cartoon")), CoverStyle::AiArt);
        assert_eq!(style_from_name(Some("anime")), CoverStyle::AiArt);
    }

    #[test]
    fn ai_art_cover_returns_placeholder_html() {
        let html = generate_cover_html("测试", "副标题", "作者", CoverStyle::AiArt, None);
        assert!(html.contains("AiArt cover"));
    }

    #[test]
    fn ai_cover_prompt_contains_art_direction() {
        let prompt = generate_ai_cover_prompt(
            "Rust 并发编程入门",
            "学习 async/await",
            "本文介绍 Rust 的 async/await 机制和 Tokio 运行时。",
        );
        assert!(prompt.contains("Art direction:"));
        assert!(prompt.contains("WeChat article cover"));
        assert!(prompt.contains("Do NOT render any text"));
    }

    #[test]
    fn ai_cover_prompt_does_not_render_text() {
        let prompt = generate_ai_cover_prompt("她说了一句\"你好\"", "", "");
        assert!(prompt.contains("Do NOT render any text"));
        assert!(prompt.contains("pure illustration"));
    }

    #[test]
    fn derive_art_direction_varies_by_article() {
        let (style_a, _, mood_a, _) = derive_art_direction(
            "Rust Tokio 异步编程",
            "async/await 深度解析",
            "tokio  runtime 调度器",
        );
        let (style_b, _, mood_b, _) =
            derive_art_direction("周末去公园散步", "春天花开了", "樱花 柳树 池塘 野餐 阳光");
        // Different content should produce different art direction.
        // (There's a tiny hash-collision chance — 1/4096 for each triple.)
        let same = style_a == style_b && mood_a == mood_b;
        assert!(!same, "two contrasting articles should not share art+ mood");
    }

    #[test]
    fn derive_art_direction_is_deterministic() {
        let (s1, c1, m1, e1) = derive_art_direction("Rust 编程", "并发", "tokio");
        let (s2, c2, m2, e2) = derive_art_direction("Rust 编程", "并发", "tokio");
        assert_eq!(s1, s2);
        assert_eq!(c1, c2);
        assert_eq!(m1, m2);
        assert_eq!(e1, e2);
    }

    #[test]
    fn extract_theme_keywords_removes_stop_words() {
        let kw = extract_theme_keywords(
            "Rust Tokio 教程",
            "异步编程指南",
            "这是一个关于 tokio 的 教程 并发 异步 Runtime",
        );
        // Stop words like "的", "一个", "关于" should be removed
        // Keywords should include meaningful words
        assert!(
            kw.contains("rust"),
            "expected 'rust' in keywords, got: {kw}"
        );
        assert!(
            kw.contains("tokio"),
            "expected 'tokio' in keywords, got: {kw}"
        );
        assert!(!kw.contains("的"), "stop word '的' should be removed");
    }

    #[test]
    fn extract_theme_keywords_falls_back_to_title() {
        let kw = extract_theme_keywords("zz", "", "");
        assert_eq!(kw, "zz");
    }

    #[test]
    fn geek_black_default_tag_no_longer_hardcodes_web3() {
        let html = generate_cover_html("测试", "", "作者", CoverStyle::GeekBlack, None);
        // The chip should show TECH · NOTES instead of the old WEB3 · DEV
        assert!(
            html.contains("TECH · NOTES"),
            "geek-black should show TECH · NOTES default chip"
        );
        assert!(
            !html.contains("WEB3"),
            "should not contain old hardcoded WEB3 tag"
        );
    }
}
