//! Article render themes — color/font presets for WeChat HTML output.

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Theme {
    pub name: &'static str,
    pub section_bg: &'static str,
    pub section_font: &'static str,
    pub section_color: &'static str,
    pub heading_color: &'static str,
    pub heading_border: &'static str,
    pub text_color: &'static str,
    pub text_muted: &'static str,
    pub accent: &'static str,
    /// Background for inset blocks (blockquote, intro, figure, generic-fence).
    pub block_bg: &'static str,
    /// Background for inline `code` spans.
    pub code_bg: &'static str,
    /// Text color for inline `code` spans; empty string inherits body text color.
    pub code_color: &'static str,
}

#[allow(dead_code)]
impl Theme {
    pub fn default() -> Self {
        Theme {
            name: "default",
            section_bg: "#fff",
            section_font: "-apple-system, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif",
            section_color: "#333",
            heading_color: "#1a1a1a",
            heading_border: "#2c2c2c",
            text_color: "#555",
            text_muted: "#888",
            accent: "#2c2c2c",
            block_bg: "#f8f8f8",
            code_bg: "#f5f5f5",
            code_color: "",
        }
    }
    pub fn warm() -> Self {
        Theme {
            name: "warm",
            section_bg: "#fdf8f4",
            section_font: "-apple-system, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif",
            section_color: "#3e2723",
            heading_color: "#4e342e",
            heading_border: "#e67e22",
            text_color: "#5d4037",
            text_muted: "#8d6e63",
            accent: "#e67e22",
            block_bg: "#f5f0eb",
            code_bg: "#ede8e3",
            code_color: "",
        }
    }
    pub fn dark() -> Self {
        Theme {
            name: "dark",
            section_bg: "#1a1a1a",
            section_font: "-apple-system, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif",
            section_color: "#e0e0e0",
            heading_color: "#ffffff",
            heading_border: "#64b5f6",
            text_color: "#b0b0b0",
            text_muted: "#888888",
            accent: "#64b5f6",
            block_bg: "#2a2a2a",
            code_bg: "#333333",
            code_color: "#e0e0e0",
        }
    }
    pub fn geek() -> Self {
        // Light background with GitHub-flavored green accent and dark code blocks.
        // Pure dark section_bg rendered badly in WeChat mobile — light bg is safer.
        Theme {
            name: "geek",
            section_bg: "#f6f8fa",
            section_font: "-apple-system, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif",
            section_color: "#24292f",
            heading_color: "#24292f",
            heading_border: "#2da44e",
            text_color: "#24292f",
            text_muted: "#57606a",
            accent: "#2da44e",
            block_bg: "#dafbe1",
            code_bg: "#0d1117",
            code_color: "#7ee787",
        }
    }
    pub fn from_name(name: &str) -> Self {
        match name {
            "warm" => Self::warm(),
            "dark" => Self::dark(),
            "geek" => Self::geek(),
            _ => Self::default(),
        }
    }

    pub fn section_style(&self) -> String {
        format!(
            "font-family: {}; font-size: 15px; line-height: 1.85; letter-spacing: 0.05em; color: {}; background: {}; padding: 0 4px;",
            self.section_font, self.section_color, self.section_bg
        )
    }
}
