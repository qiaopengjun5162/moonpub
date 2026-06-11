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
}

#[allow(dead_code)]
impl Theme {
    pub fn default() -> Self {
        Theme {
            name: "default", section_bg: "#fff",
            section_font: "-apple-system, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif",
            section_color: "#333", heading_color: "#1a1a1a", heading_border: "#2c2c2c",
            text_color: "#555", text_muted: "#888", accent: "#2c2c2c",
        }
    }
    pub fn warm() -> Self {
        Theme {
            name: "warm", section_bg: "#fdf8f4",
            section_font: "-apple-system, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif",
            section_color: "#3e2723", heading_color: "#4e342e", heading_border: "#e67e22",
            text_color: "#5d4037", text_muted: "#8d6e63", accent: "#e67e22",
        }
    }
    pub fn dark() -> Self {
        Theme {
            name: "dark", section_bg: "#1a1a1a",
            section_font: "-apple-system, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif",
            section_color: "#e0e0e0", heading_color: "#ffffff", heading_border: "#64b5f6",
            text_color: "#b0b0b0", text_muted: "#888888", accent: "#64b5f6",
        }
    }
    pub fn from_name(name: &str) -> Self {
        match name { "warm" => Self::warm(), "dark" => Self::dark(), _ => Self::default() }
    }
}
