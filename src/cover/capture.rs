//! 封面 HTML → PNG：调本地 Chrome 截图，再把临时文件原子替换到目标路径。

use super::*;

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
