//! 封面样张总览（contact sheet）：一次把所有模板封面渲成一张图。
//!
//! 起因：22 种封面风格只靠名字选不出来。`cover --style X` 一次只能看一张，
//! 想挑风格得连着渲 22 次、再在文件管理器里翻 22 个 PNG，来回切窗口对比。
//! 这里把同一篇文章按全部风格渲染，缩到一半、四列排开、每格左上角打上
//! `01 editorial` 这样的坐标角标，出一张 `*.cover-gallery.png`。
//!
//! 两个刻意的取舍：
//! - **不改源文件**：所有中间 HTML / PNG 都落在临时目录，输出只有一张总览图。
//!   调这个命令不该顺手覆盖掉文章已有的 `<slug>.cover.png`。
//! - **ai-art 不在总览里**：那个风格要真调文生图 API、要 key、每次出的图都不一样，
//!   放进 contact sheet 既贵又不可复现。

use std::fs;
use std::path::{Path, PathBuf};

use image::{ImageFormat, Rgba, RgbaImage};

use super::capture::capture_html;
use super::{CoverStyle, STYLE_NAMES, escape_html, generate_cover_html, style_from_name};
use crate::error::AppError;

/// 单格像素尺寸：900×500 的画布按 0.5 倍截图，正好是一半。
pub const CELL_WIDTH: u32 = 450;
pub const CELL_HEIGHT: u32 = 250;
/// 默认四列：1800px 宽在笔记本上不用缩放也看得清标题。
pub const DEFAULT_COLS: u32 = 4;
/// 格间距。留白比描边省事，也不会和深色封面的边撞色。
const GUTTER: u32 = 12;
const BACKGROUND: Rgba<u8> = Rgba([233, 233, 233, 255]);
/// 角标字号按 900px 画布给，0.5 倍截图后实际约 11px。
const BADGE_FONT_PX: u32 = 22;

/// 一次总览的输入。文字部分与 `cover` 命令同源，保证样张就是最终封面的样子。
pub struct GalleryRequest<'a> {
    pub out_path: &'a Path,
    pub title: &'a str,
    pub subtitle: &'a str,
    pub author: &'a str,
    pub tag: Option<&'a str>,
    pub styles: &'a [&'a str],
    pub cols: u32,
}

#[derive(Debug)]
pub struct GalleryOutcome {
    pub png: PathBuf,
    pub cols: u32,
    pub rows: u32,
    /// 成功渲进总览的风格名，按图片里的坐标顺序（从左上到右下）。
    pub rendered: Vec<String>,
    /// 渲染失败的风格与原因，不进图但要说出来。
    pub skipped: Vec<String>,
}

/// 能进总览的风格：注册表里除 `ai-art` 之外的全部。
///
/// 顺序直接用 `STYLE_NAMES`，不在别处再抄一份清单——清单抄两份，
/// 加风格时就一定有一份会忘。
pub fn gallery_style_names() -> Vec<&'static str> {
    STYLE_NAMES
        .iter()
        .copied()
        .filter(|name| *name != "ai-art")
        .collect()
}

/// `--styles a,b,c` 解析成受支持的风格名列表；不传就是全部。
///
/// 只认正名与别名（别名照原样写进角标，用户看到的仍是自己敲的那个词），
/// 写错的名字直接失败并给建议，和 `cover --style` 走同一个错误。
pub fn resolve_gallery_styles(spec: Option<&str>) -> Result<Vec<&'static str>, AppError> {
    let Some(spec) = spec else {
        return Ok(gallery_style_names());
    };
    let mut picked: Vec<&'static str> = Vec::new();
    for raw in spec.split(',') {
        let name = raw.trim();
        if name.is_empty() {
            continue;
        }
        if name == "ai-art" || name == "ai_art" || name == "cartoon" || name == "anime" {
            return Err(AppError::CoverGalleryFailed {
                reason: format!(
                    "`{name}` 需要真的调用文生图 API，不出样张；用 `moonpub cover <article.md> --style ai-art`"
                ),
            });
        }
        let Some(canonical) = STYLE_NAMES.iter().copied().find(|known| *known == name) else {
            // 名字不认识：交给 cover 那套「你是不是想写 X」的错误，别另造一份提示。
            return Err(super::unknown_style_error(name));
        };
        if !picked.contains(&canonical) {
            picked.push(canonical);
        }
    }
    if picked.is_empty() {
        return Err(AppError::MissingValueString(
            "--styles expects a comma-separated list of cover style names".to_owned(),
        ));
    }
    Ok(picked)
}

/// 网格行列数。`cols` 超过样张数时按样张数收缩，避免右侧拖一长条空白。
fn gallery_layout(count: usize, cols: u32) -> (u32, u32) {
    if count == 0 {
        return (1, 0);
    }
    let cols = cols.max(1).min(count as u32);
    let rows = (count as u32).div_ceil(cols);
    (cols, rows)
}

/// 单格左上角的角标：`03 swiss-dark`。深色封面用半透明黑底保证白字可读。
fn badge_markup(label: &str) -> String {
    format!(
        "<style>.moonpub-gallery-badge{{position:fixed;left:0;top:0;z-index:99999;\
         background:rgba(0,0,0,.72);color:#fff;font:600 {BADGE_FONT_PX}px/1.4 ui-monospace,Menlo,\
         monospace;padding:4px 10px;letter-spacing:.5px}}</style>\
         <div class=\"moonpub-gallery-badge\">{}</div>",
        escape_html(label)
    )
}

/// 把角标塞进封面 HTML 的 `</body>` 之前。
///
/// 用 `position:fixed` 而不是普通块：22 个模板的 `body` 都是
/// `height:500px;overflow:hidden`，普通元素会被裁掉，固定定位则贴着视口走。
fn inject_badge(html: &str, label: &str) -> String {
    let badge = badge_markup(label);
    match html.rfind("</body>") {
        Some(index) => format!("{}{}{}", &html[..index], badge, &html[index..]),
        // 模板都由 generate_cover_html 产出，正常都有 </body>；没有也让它跑完，
        // 大不了角标落在文档末尾——总比整张图不生成好。
        None => format!("{html}{badge}"),
    }
}

/// 把逐格图片拼成一张总览图：四周和格间都留 `GUTTER` 的灰底。
fn compose_sheet(cells: &[RgbaImage], cols: u32, rows: u32) -> RgbaImage {
    let width = cols * CELL_WIDTH + (cols + 1) * GUTTER;
    let height = rows * CELL_HEIGHT + (rows + 1) * GUTTER;
    let mut sheet = RgbaImage::from_pixel(width, height, BACKGROUND);
    for (index, cell) in cells.iter().enumerate() {
        let x = GUTTER + (index as u32 % cols) * (CELL_WIDTH + GUTTER);
        let y = GUTTER + (index as u32 / cols) * (CELL_HEIGHT + GUTTER);
        image::imageops::overlay(&mut sheet, cell, i64::from(x), i64::from(y));
    }
    sheet
}

/// 统一到单格尺寸。Chrome 已经按 0.5 倍出图，正常是原尺寸；测试假体或
/// 半分辨率没生效时这里兜底，免得某格把整张图撑变形。
fn fit_cell(image: RgbaImage) -> RgbaImage {
    if image.width() == CELL_WIDTH && image.height() == CELL_HEIGHT {
        return image;
    }
    image::imageops::resize(
        &image,
        CELL_WIDTH,
        CELL_HEIGHT,
        image::imageops::FilterType::Triangle,
    )
}

pub fn build_gallery(request: &GalleryRequest<'_>) -> Result<GalleryOutcome, AppError> {
    build_gallery_with(request, |html, png| {
        capture_html(html, png, CELL_WIDTH * 2, CELL_HEIGHT * 2, 0.5)
    })
}

/// 截图动作由调用方注入：线上是 Chrome，测试传假体写纯色 PNG。
///
/// 这样「网格怎么排、失败了怎么说」这些逻辑不需要本机装 Chrome 就能测，
/// 而真正调 Chrome 的那一行也不会被一堆测试围着改签名。
fn build_gallery_with(
    request: &GalleryRequest<'_>,
    capture: impl Fn(&Path, &Path) -> Option<String>,
) -> Result<GalleryOutcome, AppError> {
    let directory = tempfile::Builder::new()
        .prefix("moonpub-cover-gallery")
        .tempdir()
        .map_err(|source| AppError::Io {
            path: std::env::temp_dir(),
            source,
        })?;

    let mut cells: Vec<RgbaImage> = Vec::new();
    let mut rendered: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();

    for (index, name) in request.styles.iter().enumerate() {
        let style: CoverStyle = style_from_name(Some(name));
        let html = generate_cover_html(
            request.title,
            request.subtitle,
            request.author,
            style,
            request.tag,
        );
        let stem = format!("{index:02}-{}", slugify(name));
        let html_path = directory.path().join(format!("{stem}.html"));
        let png_path = directory.path().join(format!("{stem}.png"));
        let label = format!("{:02} {name}", index + 1);
        fs::write(&html_path, inject_badge(&html, &label)).map_err(|source| AppError::Io {
            path: html_path.clone(),
            source,
        })?;

        if let Some(message) = capture(&html_path, &png_path) {
            skipped.push(format!("{name}: {message}"));
            continue;
        }
        let Ok(image) = image::open(&png_path) else {
            skipped.push(format!("{name}: 截图文件无法解码：{}", png_path.display()));
            continue;
        };
        cells.push(fit_cell(image.to_rgba8()));
        rendered.push((*name).to_owned());
    }

    if cells.is_empty() {
        let first = skipped
            .first()
            .cloned()
            .unwrap_or_else(|| "没有可渲染的风格".to_owned());
        return Err(AppError::CoverGalleryFailed {
            reason: format!(
                "{} 个样张全部渲染失败（第一条：{first}）",
                request.styles.len()
            ),
        });
    }

    let (cols, rows) = gallery_layout(cells.len(), request.cols);
    let sheet = compose_sheet(&cells, cols, rows);
    if let Some(parent) = request.out_path.parent() {
        fs::create_dir_all(parent).map_err(|source| AppError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    image::save_buffer_with_format(
        request.out_path,
        sheet.as_raw(),
        sheet.width(),
        sheet.height(),
        image::ExtendedColorType::Rgba8,
        ImageFormat::Png,
    )
    .map_err(|source| AppError::CoverGalleryFailed {
        reason: format!("无法写出 {}：{source}", request.out_path.display()),
    })?;

    Ok(GalleryOutcome {
        png: request.out_path.to_path_buf(),
        cols,
        rows,
        rendered,
        skipped,
    })
}

/// 文件名只用风格名的安全字符，避免 `swiss/dark` 这类输入把临时文件写到别处。
fn slugify(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// 人看的输出：位置、图路径、跳过的格子，再按图的排布印一份图例。
pub fn gallery_text(outcome: &GalleryOutcome, display: &str, total: usize) -> String {
    let mut text = format!(
        "cover gallery\n  png:   {display}\n  cells: {} × {}（{} / {} 个风格）",
        outcome.cols,
        outcome.rows,
        outcome.rendered.len(),
        total
    );
    for line in outcome.skipped.iter() {
        text.push_str(&format!("\n  ⚠ {line}"));
    }
    text.push_str("\n  legend（按图从左到右、从上到下）:");
    // 编号必须和图上的角标一致：断行后从 1 重新数，用户就得自己算「第 7 格是谁」。
    let mut position = 1;
    for chunk in outcome.rendered.chunks(outcome.cols.max(1) as usize) {
        let mut line = String::new();
        for name in chunk {
            line.push_str(&format!("{position:02} {name:<12}"));
            position += 1;
        }
        text.push_str(&format!("\n    {line}"));
    }
    text
}

/// `--json` 版：同一个结果，给 agent 用来决定「选哪个风格」而不是看文字猜。
pub fn gallery_json(outcome: &GalleryOutcome, display: &str) -> String {
    let cells: Vec<serde_json::Value> = outcome
        .rendered
        .iter()
        .enumerate()
        .map(|(index, style)| serde_json::json!({"position": index + 1, "style": style}))
        .collect();
    let payload = serde_json::json!({
        "command": "cover-gallery",
        "png": display,
        "cols": outcome.cols,
        "rows": outcome.rows,
        "cells": cells,
        "skipped": outcome.skipped,
    });
    serde_json::to_string_pretty(&payload)
        .unwrap_or_else(|error| format!("{{\"serialization_error\":\"{error}\"}}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid_png(path: &Path, width: u32, height: u32) {
        let image = RgbaImage::from_pixel(width, height, Rgba([10, 20, 30, 255]));
        image.save(path).expect("write fake capture");
    }

    fn request<'a>(out: &'a Path, styles: &'a [&'a str]) -> GalleryRequest<'a> {
        GalleryRequest {
            out_path: out,
            title: "九次发版零个 job",
            subtitle: "检查从来没跑过",
            author: "寻月隐君",
            tag: None,
            styles,
            cols: DEFAULT_COLS,
        }
    }

    #[test]
    fn gallery_covers_every_registered_style_except_ai_art() {
        let names = gallery_style_names();
        assert_eq!(names.len(), STYLE_NAMES.len() - 1);
        assert!(!names.contains(&"ai-art"));
        // 清单顺序必须沿用注册表：另排一份顺序，图和图例就会对不上。
        let expected: Vec<&str> = STYLE_NAMES
            .iter()
            .copied()
            .filter(|name| *name != "ai-art")
            .collect();
        assert_eq!(names, expected);
    }

    #[test]
    fn gallery_styles_are_all_renderable_without_an_api_key() {
        for name in gallery_style_names() {
            assert_ne!(
                style_from_name(Some(name)),
                CoverStyle::AiArt,
                "{name} should not need the image API"
            );
        }
    }

    #[test]
    fn resolve_styles_defaults_to_the_whole_sheet() {
        assert_eq!(
            resolve_gallery_styles(None).expect("default list"),
            gallery_style_names()
        );
    }

    #[test]
    fn resolve_styles_keeps_order_and_drops_duplicates() {
        let picked = resolve_gallery_styles(Some(" geek-black , swiss ,geek-black,"))
            .expect("explicit list");
        assert_eq!(picked, vec!["geek-black", "swiss"]);
    }

    #[test]
    fn resolve_styles_rejects_a_typo_with_the_cover_style_hint() {
        let error = resolve_gallery_styles(Some("swis")).expect_err("typo must fail");
        let message = error.to_string();
        assert!(message.contains("swis"), "{message}");
        assert!(message.contains("swiss"), "{message}");
    }

    #[test]
    fn resolve_styles_explains_why_ai_art_is_not_in_the_sheet() {
        let error = resolve_gallery_styles(Some("ai-art")).expect_err("ai-art is excluded");
        assert!(error.to_string().contains("文生图"), "{error}");
    }

    #[test]
    fn resolve_styles_rejects_an_empty_list() {
        assert!(matches!(
            resolve_gallery_styles(Some(" , ")),
            Err(AppError::MissingValueString(_))
        ));
    }

    #[test]
    fn layout_wraps_into_rows_and_never_leaves_an_empty_column() {
        assert_eq!(gallery_layout(21, 4), (4, 6));
        assert_eq!(gallery_layout(4, 4), (4, 1));
        assert_eq!(gallery_layout(2, 4), (2, 1));
        assert_eq!(gallery_layout(21, 0), (1, 21));
        assert_eq!(gallery_layout(0, 4), (1, 0));
    }

    #[test]
    fn badge_goes_inside_the_document_before_the_body_ends() {
        let html = "<html><body><h1>x</h1></body></html>";
        let painted = inject_badge(html, "03 swiss-dark");
        let badge = painted
            .find("moonpub-gallery-badge\">03 swiss-dark")
            .expect("badge");
        let body_end = painted.find("</body>").expect("body end");
        assert!(badge < body_end, "{painted}");
        assert!(painted.ends_with("</body></html>"));
        // 固定定位是必需的：模板的 body 是 500px 固定高 + overflow:hidden。
        assert!(painted.contains("position:fixed"));
    }

    #[test]
    fn badge_escapes_a_style_name_that_looks_like_markup() {
        let painted = inject_badge("<body></body>", "<img src=x onerror=alert(1)>");
        assert!(!painted.contains("<img src=x"), "{painted}");
        assert!(painted.contains("&lt;img"), "{painted}");
    }

    #[test]
    fn inject_badge_still_returns_the_cover_when_body_tag_is_missing() {
        let painted = inject_badge("<html></html>", "01 editorial");
        assert!(painted.starts_with("<html></html>"));
        assert!(painted.contains("01 editorial"));
    }

    #[test]
    fn sheet_is_sized_for_full_and_partial_last_rows() {
        let cell = RgbaImage::from_pixel(CELL_WIDTH, CELL_HEIGHT, Rgba([9, 9, 9, 255]));
        let sheet = compose_sheet(&[cell.clone(), cell.clone(), cell], 4, 1);
        assert_eq!(
            sheet.dimensions(),
            (4 * CELL_WIDTH + 5 * GUTTER, CELL_HEIGHT + 2 * GUTTER)
        );
        // 左上角留白、第一格起点正好差一个 GUTTER，第三格在第二列不越界。
        assert_eq!(*sheet.get_pixel(0, 0), BACKGROUND);
        assert_eq!(*sheet.get_pixel(GUTTER, GUTTER), Rgba([9, 9, 9, 255]));
        assert_eq!(
            *sheet.get_pixel(GUTTER + 2 * (CELL_WIDTH + GUTTER), GUTTER),
            Rgba([9, 9, 9, 255])
        );
    }

    #[test]
    fn sheet_with_two_rows_keeps_the_gutter_between_rows() {
        let cell = RgbaImage::from_pixel(CELL_WIDTH, CELL_HEIGHT, Rgba([1, 2, 3, 255]));
        let sheet = compose_sheet(&[cell.clone(), cell], 1, 2);
        assert_eq!(
            sheet.dimensions(),
            (CELL_WIDTH + 2 * GUTTER, 2 * CELL_HEIGHT + 3 * GUTTER)
        );
        assert_eq!(
            *sheet.get_pixel(GUTTER, CELL_HEIGHT + GUTTER + 6),
            BACKGROUND,
            "行间必须留出灰底，否则深色封面会连成一片"
        );
    }

    #[test]
    fn fit_cell_rescales_a_capture_that_ignored_the_half_scale_flag() {
        let oversized =
            RgbaImage::from_pixel(CELL_WIDTH * 2, CELL_HEIGHT * 2, Rgba([0, 0, 0, 255]));
        assert_eq!(fit_cell(oversized).dimensions(), (CELL_WIDTH, CELL_HEIGHT));
        let exact = RgbaImage::from_pixel(CELL_WIDTH, CELL_HEIGHT, Rgba([0, 0, 0, 255]));
        assert_eq!(fit_cell(exact).dimensions(), (CELL_WIDTH, CELL_HEIGHT));
    }

    #[test]
    fn gallery_writes_one_cell_per_style_it_managed_to_capture() {
        let root = crate::test_helpers::temp_root("cover-gallery-ok").expect("temp dir");
        let out = root.join("gallery.png");
        let styles = ["geek-black", "swiss", "noir"];
        let outcome = build_gallery_with(&request(&out, &styles), |_, png| {
            solid_png(png, CELL_WIDTH, CELL_HEIGHT);
            None
        })
        .expect("gallery should build");

        assert_eq!(outcome.rendered.len(), 3);
        assert_eq!((outcome.cols, outcome.rows), (3, 1));
        assert!(outcome.skipped.is_empty());
        let sheet = image::open(&out).expect("sheet png").to_rgba8();
        assert_eq!(
            sheet.dimensions(),
            (3 * CELL_WIDTH + 4 * GUTTER, CELL_HEIGHT + 2 * GUTTER)
        );
    }

    #[test]
    fn gallery_skips_a_style_whose_screenshot_failed_but_keeps_the_rest() {
        let root = crate::test_helpers::temp_root("cover-gallery-partial").expect("temp dir");
        let out = root.join("out/gallery.png");
        let styles = ["geek-black", "swiss"];
        let outcome = build_gallery_with(&request(&out, &styles), |_, png| {
            if png.to_string_lossy().contains("swiss") {
                return Some("screenshot failed: boom".to_owned());
            }
            solid_png(png, CELL_WIDTH, CELL_HEIGHT);
            None
        })
        .expect("one good cell is enough");

        assert_eq!(outcome.rendered, vec!["geek-black"]);
        assert_eq!(outcome.skipped.len(), 1);
        assert!(outcome.skipped[0].contains("screenshot failed: boom"));
        assert!(out.exists(), "失败一格也要出图，否则用户什么都看不到");
    }

    #[test]
    fn gallery_without_chrome_fails_with_the_first_reason_intact() {
        let root = crate::test_helpers::temp_root("cover-gallery-nochrome").expect("temp dir");
        let out = root.join("gallery.png");
        let styles = ["geek-black"];
        let error = build_gallery_with(&request(&out, &styles), |_, _| {
            Some("screenshot skipped: Chrome/Chromium not found".to_owned())
        })
        .expect_err("no cells means no sheet");
        let message = error.to_string();
        assert!(message.contains("Chrome"), "{message}");
        assert!(message.contains("geek-black"), "{message}");
        assert!(!out.exists());
    }

    #[test]
    fn gallery_text_prints_a_legend_that_matches_the_numbers_on_the_image() {
        let outcome = GalleryOutcome {
            png: PathBuf::from("gallery.png"),
            cols: 2,
            rows: 2,
            rendered: vec![
                "editorial".to_owned(),
                "swiss".to_owned(),
                "noir".to_owned(),
                "riso".to_owned(),
            ],
            skipped: vec!["aurora: screenshot failed: boom".to_owned()],
        };
        let text = gallery_text(&outcome, "articles/x.cover-gallery.png", 5);
        assert!(text.contains("png:   articles/x.cover-gallery.png"));
        assert!(text.contains("2 × 2（4 / 5 个风格）"));
        assert!(text.contains("⚠ aurora: screenshot failed: boom"));
        // 第二行从 03 起，而不是从头再数一遍。
        assert!(text.contains("03 noir"), "{text}");
        assert!(text.contains("04 riso"), "{text}");
    }

    #[test]
    fn slugify_keeps_temp_file_names_inside_the_temp_directory() {
        assert_eq!(slugify("swiss-dark"), "swiss-dark");
        assert_eq!(slugify("../../etc/passwd"), "------etc-passwd");
    }
}
