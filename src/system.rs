pub(crate) fn find_chrome() -> Option<String> {
    let candidates = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
    ];
    for c in &candidates {
        if c.starts_with('/') {
            if std::path::Path::new(c).exists() {
                return Some(c.to_string());
            }
        } else if std::process::Command::new("which")
            .arg(c)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some(c.to_string());
        }
    }
    None
}
