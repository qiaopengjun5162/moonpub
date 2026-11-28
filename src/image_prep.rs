//! Prepare article images for WeChat upload.
//!
//! WeChat's in-article image endpoints (both the API path and the cookie-mode
//! web-console path) only accept **jpg/png**, one file of at most **1 MB**.
//! `preflight` only checks that referenced files exist, so an SVG diagram
//! (Mermaid, chart, vector icon) or a large photo passes preflight and then
//! fails at push time with an API error that says nothing about the image.
//!
//! This module normalizes images right before upload:
//!
//! - `svg` / `svgz` → rasterized to PNG at 2x with system fonts (4096 px cap)
//! - `gif` / `webp` → first frame re-encoded (WeChat does not animate them)
//! - oversized rasters → downscaled and re-encoded until they fit 1 MB
//!
//! The source file is never modified. Converted bytes live in a temporary file
//! that is deleted when the returned [`PreparedImage`] drops.

use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

use image::{DynamicImage, ImageFormat};

use crate::error::AppError;

/// WeChat's per-image cap for in-article images.
pub const MAX_IMAGE_BYTES: u64 = 1024 * 1024;

/// Render scale for SVG diagrams: 2x keeps chart labels legible on phones.
const SVG_SCALE: f32 = 2.0;
/// Hard cap on a rasterized SVG edge so a hostile `viewBox` cannot exhaust memory.
const MAX_SVG_EDGE: f32 = 4096.0;
/// Long-edge cap applied before re-encoding an oversized raster image.
const MAX_RASTER_EDGE: u32 = 1920;

/// System fonts for SVG `<text>`; loading once keeps a 30-image push fast.
static SVG_FONTS: LazyLock<Arc<resvg::usvg::fontdb::Database>> = LazyLock::new(|| {
    let mut db = resvg::usvg::fontdb::Database::new();
    db.load_system_fonts();
    Arc::new(db)
});

/// A converted image, deleted from disk when dropped.
#[derive(Debug)]
pub struct PreparedImage {
    file: tempfile::NamedTempFile,
}

impl PreparedImage {
    /// Path of the converted file, safe to hand to the upload functions.
    pub fn path(&self) -> &Path {
        self.file.path()
    }
}

fn has_extension(path: &Path, wanted: &[&str]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| wanted.iter().any(|w| ext.eq_ignore_ascii_case(w)))
}

fn prep_err(path: &Path, reason: impl Into<String>) -> AppError {
    AppError::ImagePrep {
        path: path.to_path_buf(),
        reason: reason.into(),
        hint: "WeChat accepts jpg/png only (max 1 MB per image); \
               `moonpub push` converts SVG/large images automatically — \
               if this still fails, export the image as PNG and retry",
    }
}

/// Does this path need conversion before it can be uploaded?
///
/// Cheap pre-check used by callers that want to skip work; the real decision
/// is made on the bytes inside [`prepare_image`].
pub fn may_need_preparation(path: &Path) -> bool {
    has_extension(path, &["svg", "svgz", "gif", "webp"])
        || std::fs::metadata(path).is_ok_and(|m| m.len() > MAX_IMAGE_BYTES)
}

/// Convert one article image so WeChat will accept it.
///
/// Returns `Ok(None)` when the file is already a jpg/png within the size cap
/// and can be uploaded as-is. Otherwise returns a temporary converted copy.
pub fn prepare_image(path: &Path) -> Result<Option<PreparedImage>, AppError> {
    let bytes = std::fs::read(path).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    if has_extension(path, &["svg", "svgz"]) {
        let png = rasterize_svg(&bytes, path)?;
        let out = if png.len() as u64 <= MAX_IMAGE_BYTES {
            png
        } else {
            // A 2x raster can exceed the cap; fall back to a white-backed JPEG.
            let img = decode(&png, path)?;
            compress_to_limit(img, path, true)?
        };
        return Ok(Some(write_temp(&out, ".png", path)?));
    }

    let format = image::guess_format(&bytes).ok();
    let within_cap = bytes.len() as u64 <= MAX_IMAGE_BYTES;
    let name_says_png = has_extension(path, &["png"]);
    let name_says_jpeg = has_extension(path, &["jpg", "jpeg"]);
    // Already uploadable only when the bytes and the filename agree: a PNG named
    // `.jpg` is sent with `Content-Type: image/jpeg` and WeChat rejects it.
    let name_matches_bytes = (format == Some(ImageFormat::Png) && name_says_png)
        || (format == Some(ImageFormat::Jpeg) && name_says_jpeg);
    if within_cap && name_matches_bytes {
        return Ok(None);
    }

    let img = decode(&bytes, path)?;
    // Diagrams and icons stay lossless as PNG (transparency kept); photo-like
    // or oversized images go to JPEG, which fits far more pixels under 1 MB.
    let prefer_png = matches!(format, Some(ImageFormat::Png | ImageFormat::Gif));
    let out = if prefer_png {
        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, ImageFormat::Png)
            .map_err(|e| prep_err(path, format!("PNG encode failed: {e}")))?;
        let png = buf.into_inner();
        if png.len() as u64 <= MAX_IMAGE_BYTES {
            png
        } else {
            compress_to_limit(img, path, false)?
        }
    } else {
        compress_to_limit(img, path, false)?
    };
    let ext = if out.starts_with(&[0x89, b'P', b'N', b'G']) {
        ".png"
    } else {
        ".jpg"
    };
    Ok(Some(write_temp(&out, ext, path)?))
}

/// Rasterize SVG/SVGZ bytes to PNG using system fonts.
///
/// `source` is only used for `resources_dir` (relative `<image href>`) and for
/// error messages, so a broken diagram names the real file, not `<svg>`.
fn rasterize_svg(bytes: &[u8], source: &Path) -> Result<Vec<u8>, AppError> {
    let opt = resvg::usvg::Options {
        resources_dir: source.parent().map(Path::to_path_buf),
        fontdb: Arc::clone(&SVG_FONTS),
        ..Default::default()
    };
    let tree = resvg::usvg::Tree::from_data(bytes, &opt)
        .map_err(|e| prep_err(source, format!("invalid SVG: {e}")))?;
    let size = tree.size();
    let longest = size.width().max(size.height());
    if longest <= 0.0 {
        return Err(prep_err(source, "SVG has zero size"));
    }
    let scale = SVG_SCALE.min(MAX_SVG_EDGE / longest);
    let width = ((size.width() * scale).ceil() as u32).max(1);
    let height = ((size.height() * scale).ceil() as u32).max(1);
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| prep_err(source, "cannot allocate render buffer"))?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    pixmap
        .encode_png()
        .map_err(|e| prep_err(source, format!("PNG encode failed: {e}")))
}

fn decode(bytes: &[u8], path: &Path) -> Result<DynamicImage, AppError> {
    image::load_from_memory(bytes).map_err(|e| prep_err(path, format!("cannot decode image: {e}")))
}

/// Composite any transparency onto white so JPEG has no black fringes.
fn flatten_alpha(img: DynamicImage) -> DynamicImage {
    if !img.color().has_alpha() {
        return img;
    }
    let rgba = img.to_rgba8();
    let mut canvas = image::RgbaImage::from_pixel(
        rgba.width(),
        rgba.height(),
        image::Rgba([255, 255, 255, 255]),
    );
    image::imageops::overlay(&mut canvas, &rgba, 0, 0);
    DynamicImage::ImageRgba8(canvas)
}

/// Downscale + re-encode until the bytes fit [`MAX_IMAGE_BYTES`].
///
/// Steps down through a few (edge, quality) pairs instead of binary-searching:
/// the loop is deterministic, cheap, and easy to assert in tests.
fn compress_to_limit(
    img: DynamicImage,
    path: &Path,
    force_jpeg: bool,
) -> Result<Vec<u8>, AppError> {
    let mut current = flatten_alpha(img);
    let edges = [MAX_RASTER_EDGE, 1600, 1280, 1024, 800];
    let qualities = [86u8, 78, 70];
    for edge in edges {
        if current.width() > edge || current.height() > edge {
            current = current.resize(edge, edge, image::imageops::FilterType::Lanczos3);
        }
        for quality in qualities {
            let jpeg = encode_jpeg(&current, quality)
                .map_err(|e| prep_err(path, format!("JPEG encode failed: {e}")))?;
            if jpeg.len() as u64 <= MAX_IMAGE_BYTES {
                return Ok(jpeg);
            }
        }
        if !force_jpeg {
            // Try lossless PNG at this size before shrinking further; diagrams
            // with flat colours compress much better as PNG than as JPEG.
            let mut png = Cursor::new(Vec::new());
            current
                .write_to(&mut png, ImageFormat::Png)
                .map_err(|e| prep_err(path, format!("PNG encode failed: {e}")))?;
            let png = png.into_inner();
            if png.len() as u64 <= MAX_IMAGE_BYTES {
                return Ok(png);
            }
        }
    }
    Err(prep_err(
        path,
        "image is still over 1 MB after downscaling to 800 px",
    ))
}

fn encode_jpeg(img: &DynamicImage, quality: u8) -> Result<Vec<u8>, image::ImageError> {
    let rgb = img.to_rgb8();
    let mut buf = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
    encoder.encode_image(&DynamicImage::ImageRgb8(rgb))?;
    Ok(buf)
}

fn write_temp(bytes: &[u8], extension: &str, source: &Path) -> Result<PreparedImage, AppError> {
    let mut file = tempfile::Builder::new()
        .prefix("moonpub-img-")
        .suffix(extension)
        .tempfile()
        .map_err(|source_err| AppError::Io {
            path: PathBuf::from("<temp>"),
            source: source_err,
        })?;
    file.write_all(bytes).map_err(|source_err| AppError::Io {
        path: file.path().to_path_buf(),
        source: source_err,
    })?;
    file.flush().map_err(|source_err| AppError::Io {
        path: file.path().to_path_buf(),
        source: source_err,
    })?;
    let _ = source;
    Ok(PreparedImage { file })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="60" height="40"><rect width="60" height="40" fill="#ff0000"/></svg>"##;

    fn temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    #[test]
    fn svg_is_rasterized_to_png() {
        let dir = temp_dir();
        let path = dir.path().join("chart.svg");
        std::fs::write(&path, SVG).unwrap();
        let prepared = prepare_image(&path)
            .expect("svg should convert")
            .expect("needs conversion");
        let bytes = std::fs::read(prepared.path()).unwrap();
        assert_eq!(image::guess_format(&bytes).unwrap(), ImageFormat::Png);
        // 2x scale: 60x40 -> 120x80.
        assert_eq!(image::image_dimensions(prepared.path()).unwrap(), (120, 80));
        // The source SVG must never be rewritten.
        assert_eq!(std::fs::read_to_string(&path).unwrap(), SVG);
    }

    #[test]
    fn small_png_is_left_alone() {
        let dir = temp_dir();
        let path = dir.path().join("ok.png");
        DynamicImage::new_rgb8(16, 16)
            .save_with_format(&path, ImageFormat::Png)
            .unwrap();
        assert!(prepare_image(&path).unwrap().is_none());
    }

    #[test]
    fn png_named_jpeg_is_normalized() {
        let dir = temp_dir();
        let path = dir.path().join("mismatch.jpg");
        DynamicImage::new_rgb8(16, 16)
            .save_with_format(&path, ImageFormat::Png)
            .unwrap();
        let prepared = prepare_image(&path).unwrap().expect("mismatch converts");
        let bytes = std::fs::read(prepared.path()).unwrap();
        assert_eq!(image::guess_format(&bytes).unwrap(), ImageFormat::Png);
    }

    #[test]
    fn gif_without_animation_becomes_png() {
        let dir = temp_dir();
        let path = dir.path().join("icon.gif");
        DynamicImage::new_rgb8(8, 8)
            .save_with_format(&path, ImageFormat::Gif)
            .unwrap();
        let prepared = prepare_image(&path).unwrap().expect("gif converts");
        assert_eq!(
            std::fs::read(prepared.path()).unwrap()[..4],
            [0x89, b'P', b'N', b'G']
        );
    }

    #[test]
    fn invalid_svg_reports_the_real_path() {
        let dir = temp_dir();
        let path = dir.path().join("broken.svg");
        std::fs::write(&path, "not an svg").unwrap();
        let err = prepare_image(&path).unwrap_err().to_string();
        assert!(err.contains("broken.svg"), "{err}");
    }

    #[test]
    fn huge_photo_is_shrunk_under_the_limit() {
        let dir = temp_dir();
        let path = dir.path().join("big.png");
        // Random noise defeats PNG compression, so the file starts over 1 MB
        // without needing a 2000 px canvas (which made this test take 30s).
        let mut img = image::RgbaImage::new(1100, 1100);
        let mut state = 0x1234_5678u32;
        for pixel in img.pixels_mut() {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            *pixel = image::Rgba([
                (state >> 24) as u8,
                (state >> 16) as u8,
                (state >> 8) as u8,
                255,
            ]);
        }
        DynamicImage::ImageRgba8(img)
            .save_with_format(&path, ImageFormat::Png)
            .unwrap();
        assert!(std::fs::metadata(&path).unwrap().len() > MAX_IMAGE_BYTES);
        let prepared = prepare_image(&path).unwrap().expect("oversized converts");
        let size = std::fs::metadata(prepared.path()).unwrap().len();
        assert!(size <= MAX_IMAGE_BYTES, "converted image is {size} bytes");
    }
}
