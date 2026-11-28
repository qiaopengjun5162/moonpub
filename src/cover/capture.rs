//! 封面 HTML → PNG：调本地 Chrome 截图，再把临时文件原子替换到目标路径。

use super::*;

/// 单张封面：900×500（微信首图的常用画布），1:1 像素。
pub fn capture_cover_png(html_path: &Path, png_path: &Path) -> Option<String> {
    capture_html(html_path, png_path, 900, 500, 1.0)
}

/// 按指定视口与设备缩放截图。
///
/// `scale` 走 Chrome 的 `--force-device-scale-factor`：总览图要放二十多张缩略图，
/// 与其截完再用 `image` 缩一遍（缩出来一样糊），不如让 Chrome 直接按半分辨率
/// 排版渲染——小尺寸下的文字反而更清楚。
pub(super) fn capture_html(
    html_path: &Path,
    png_path: &Path,
    width: u32,
    height: u32,
    scale: f64,
) -> Option<String> {
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
    let mut args: Vec<String> = vec![
        "--headless".to_owned(),
        "--disable-gpu".to_owned(),
        "--no-sandbox".to_owned(),
        format!("--window-size={width},{height}"),
    ];
    // 1.0 是默认值，显式传等于白多一个和 Chrome 版本相关的行为面。
    if (scale - 1.0).abs() > f64::EPSILON {
        args.push(format!("--force-device-scale-factor={scale}"));
    }
    args.push(format!("--screenshot={}", capture_path.display()));
    args.push(format!("file://{}", abs_html.display()));

    let output = std::process::Command::new(&bin).args(&args).output();

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

pub(super) fn temporary_capture_path(png_path: &Path) -> PathBuf {
    let file_name = png_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("cover.png");
    png_path.with_file_name(format!(
        ".{file_name}.moonpub-{}.tmp.png",
        std::process::id()
    ))
}

pub(super) fn replace_cover_png(capture_path: &Path, png_path: &Path) -> std::io::Result<()> {
    fs::copy(capture_path, png_path)?;
    fs::remove_file(capture_path)
}
