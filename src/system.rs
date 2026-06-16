pub(crate) fn find_chrome() -> Option<String> {
    let candidates = [
        // macOS
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        // Linux
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
        // Windows
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
    ];
    for c in &candidates {
        if c.starts_with('/') || c.starts_with(r"C:\") {
            if std::path::Path::new(c).exists() {
                return Some(c.to_string());
            }
        } else if cfg!(windows) {
            if std::process::Command::new("where")
                .arg(c)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
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
    // Windows: check LOCALAPPDATA for Chrome/Edge user installs
    #[cfg(windows)]
    {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let extra = [
                r"Google\Chrome\Application\chrome.exe",
                r"Microsoft\Edge\Application\msedge.exe",
            ];
            for e in &extra {
                let p = std::path::PathBuf::from(&local).join(e);
                if p.exists() {
                    return Some(p.to_string_lossy().into_owned());
                }
            }
        }
    }
    None
}
