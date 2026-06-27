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
    /// Softer accent background used by headings, tables, and list rows.
    pub accent_soft: &'static str,
    /// Thin border color that must stay readable in WeChat's editor.
    pub border: &'static str,
    /// Background for table headers and number badges.
    pub header_bg: &'static str,
}

#[allow(dead_code)]
impl Theme {
    pub const fn names() -> &'static [&'static str] {
        &[
            "default",
            "warm",
            "dark",
            "geek",
            "paper",
            "magazine",
            "notebook",
            "classic",
            "forest",
            "sunset",
            "ocean",
            "mono",
            "editorial",
            "zen",
            "newsletter",
            "academic",
            "cyber",
        ]
    }

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
            accent_soft: "#f3f4f6",
            border: "#e5e7eb",
            header_bg: "#2c2c2c",
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
            accent_soft: "#fff0df",
            border: "#ecd8c8",
            header_bg: "#d35400",
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
            accent_soft: "#243447",
            border: "#3b4552",
            header_bg: "#0f172a",
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
            accent_soft: "#ddf4ff",
            border: "#b6e3ff",
            header_bg: "#0969da",
        }
    }
    pub fn paper() -> Self {
        Theme {
            name: "paper",
            section_bg: "#fffdf8",
            section_font: "'Songti SC', 'Noto Serif CJK SC', 'Source Han Serif SC', serif",
            section_color: "#2f2a24",
            heading_color: "#1f1b16",
            heading_border: "#b88a44",
            text_color: "#3d352d",
            text_muted: "#8a7660",
            accent: "#b88a44",
            block_bg: "#f8f1e7",
            code_bg: "#f1e8db",
            code_color: "#5c4033",
            accent_soft: "#fbf3e6",
            border: "#eadcc8",
            header_bg: "#8f6a35",
        }
    }
    pub fn magazine() -> Self {
        Theme {
            name: "magazine",
            section_bg: "#ffffff",
            section_font: "-apple-system, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif",
            section_color: "#19202a",
            heading_color: "#111827",
            heading_border: "#ef4444",
            text_color: "#334155",
            text_muted: "#64748b",
            accent: "#ef4444",
            block_bg: "#f8fafc",
            code_bg: "#111827",
            code_color: "#f8fafc",
            accent_soft: "#fef2f2",
            border: "#e2e8f0",
            header_bg: "#111827",
        }
    }
    pub fn notebook() -> Self {
        Theme {
            name: "notebook",
            section_bg: "#fbfcff",
            section_font: "-apple-system, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif",
            section_color: "#1f2937",
            heading_color: "#1e3a8a",
            heading_border: "#3b82f6",
            text_color: "#374151",
            text_muted: "#6b7280",
            accent: "#3b82f6",
            block_bg: "#eff6ff",
            code_bg: "#e0f2fe",
            code_color: "#0f172a",
            accent_soft: "#dbeafe",
            border: "#bfdbfe",
            header_bg: "#2563eb",
        }
    }
    pub fn classic() -> Self {
        Theme {
            name: "classic",
            section_bg: "#fff",
            section_font: "'Times New Roman', 'Songti SC', 'Noto Serif CJK SC', serif",
            section_color: "#2b2b2b",
            heading_color: "#111111",
            heading_border: "#7f1d1d",
            text_color: "#333333",
            text_muted: "#777777",
            accent: "#7f1d1d",
            block_bg: "#f7f7f5",
            code_bg: "#f1f1ee",
            code_color: "#333333",
            accent_soft: "#f5eeee",
            border: "#dedbd4",
            header_bg: "#2b2b2b",
        }
    }
    pub fn forest() -> Self {
        Theme {
            name: "forest",
            section_bg: "#fbfcf7",
            section_font: "'Songti SC', 'Noto Serif CJK SC', 'Source Han Serif SC', -apple-system, 'PingFang SC', serif",
            section_color: "#17251c",
            heading_color: "#274934",
            heading_border: "#5f8f68",
            text_color: "#3f3c35",
            text_muted: "#7a7469",
            accent: "#5f8f68",
            block_bg: "#f1f6ef",
            code_bg: "#10291c",
            code_color: "#d3f4dc",
            accent_soft: "#edf4e9",
            border: "#d5e1d1",
            header_bg: "#25573b",
        }
    }
    pub fn sunset() -> Self {
        Theme {
            name: "sunset",
            section_bg: "#fffaf7",
            section_font: "-apple-system, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif",
            section_color: "#3d261f",
            heading_color: "#5b2116",
            heading_border: "#e85d3f",
            text_color: "#55382f",
            text_muted: "#9a756a",
            accent: "#e85d3f",
            block_bg: "#fff0e8",
            code_bg: "#4a1f17",
            code_color: "#ffe7dd",
            accent_soft: "#ffebe0",
            border: "#f1c8b8",
            header_bg: "#b43822",
        }
    }
    pub fn ocean() -> Self {
        Theme {
            name: "ocean",
            section_bg: "#f8fcff",
            section_font: "-apple-system, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif",
            section_color: "#183342",
            heading_color: "#0f3b57",
            heading_border: "#0284c7",
            text_color: "#2b4b5f",
            text_muted: "#647f91",
            accent: "#0284c7",
            block_bg: "#eaf7ff",
            code_bg: "#082f49",
            code_color: "#bae6fd",
            accent_soft: "#e0f2fe",
            border: "#bae6fd",
            header_bg: "#075985",
        }
    }
    pub fn mono() -> Self {
        Theme {
            name: "mono",
            section_bg: "#ffffff",
            section_font: "'SF Pro Text', -apple-system, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif",
            section_color: "#202020",
            heading_color: "#111111",
            heading_border: "#111111",
            text_color: "#333333",
            text_muted: "#737373",
            accent: "#111111",
            block_bg: "#f5f5f5",
            code_bg: "#111111",
            code_color: "#f5f5f5",
            accent_soft: "#eeeeee",
            border: "#d4d4d4",
            header_bg: "#111111",
        }
    }
    pub fn editorial() -> Self {
        Theme {
            name: "editorial",
            section_bg: "#fffaf2",
            section_font: "'Songti SC', 'Noto Serif CJK SC', 'Source Han Serif SC', serif",
            section_color: "#271b15",
            heading_color: "#1f130e",
            heading_border: "#c2410c",
            text_color: "#3b2a22",
            text_muted: "#8a6b5a",
            accent: "#c2410c",
            block_bg: "#fff1df",
            code_bg: "#f5e4cf",
            code_color: "#5b2b16",
            accent_soft: "#ffedd5",
            border: "#e7c7a4",
            header_bg: "#7c2d12",
        }
    }
    pub fn zen() -> Self {
        Theme {
            name: "zen",
            section_bg: "#fbfbf7",
            section_font: "'Songti SC', 'Noto Serif CJK SC', 'Source Han Serif SC', serif",
            section_color: "#25251f",
            heading_color: "#1f1f1a",
            heading_border: "#6b705c",
            text_color: "#3f3f35",
            text_muted: "#7a7a69",
            accent: "#6b705c",
            block_bg: "#f1f1ea",
            code_bg: "#e8e8dc",
            code_color: "#36362e",
            accent_soft: "#eeeeE2",
            border: "#d7d7c8",
            header_bg: "#4a4e3f",
        }
    }
    pub fn newsletter() -> Self {
        Theme {
            name: "newsletter",
            section_bg: "#ffffff",
            section_font: "-apple-system, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif",
            section_color: "#1f2937",
            heading_color: "#0f172a",
            heading_border: "#f59e0b",
            text_color: "#334155",
            text_muted: "#64748b",
            accent: "#f59e0b",
            block_bg: "#fffbeb",
            code_bg: "#fef3c7",
            code_color: "#78350f",
            accent_soft: "#fef3c7",
            border: "#fde68a",
            header_bg: "#92400e",
        }
    }
    pub fn academic() -> Self {
        Theme {
            name: "academic",
            section_bg: "#fbfbff",
            section_font: "'Times New Roman', 'Songti SC', 'Noto Serif CJK SC', serif",
            section_color: "#1f2937",
            heading_color: "#111827",
            heading_border: "#1d4ed8",
            text_color: "#374151",
            text_muted: "#6b7280",
            accent: "#1d4ed8",
            block_bg: "#eef2ff",
            code_bg: "#e0e7ff",
            code_color: "#1e1b4b",
            accent_soft: "#e0e7ff",
            border: "#c7d2fe",
            header_bg: "#1e3a8a",
        }
    }
    pub fn cyber() -> Self {
        Theme {
            name: "cyber",
            section_bg: "#07111f",
            section_font: "'SF Pro Text', -apple-system, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif",
            section_color: "#dbeafe",
            heading_color: "#e0f2fe",
            heading_border: "#22d3ee",
            text_color: "#bfdbfe",
            text_muted: "#7dd3fc",
            accent: "#22d3ee",
            block_bg: "#0f1f33",
            code_bg: "#020617",
            code_color: "#67e8f9",
            accent_soft: "#123047",
            border: "#155e75",
            header_bg: "#164e63",
        }
    }
    pub fn from_name(name: &str) -> Self {
        match name {
            "warm" => Self::warm(),
            "dark" => Self::dark(),
            "geek" => Self::geek(),
            "paper" => Self::paper(),
            "magazine" => Self::magazine(),
            "notebook" => Self::notebook(),
            "classic" => Self::classic(),
            "forest" => Self::forest(),
            "sunset" => Self::sunset(),
            "ocean" => Self::ocean(),
            "mono" => Self::mono(),
            "editorial" => Self::editorial(),
            "zen" => Self::zen(),
            "newsletter" => Self::newsletter(),
            "academic" => Self::academic(),
            "cyber" => Self::cyber(),
            _ => Self::default(),
        }
    }

    pub fn section_style(&self) -> String {
        format!(
            "font-family: {}; font-size: 15px; line-height: 1.9; letter-spacing: 0.05em; color: {}; background: {}; padding: 0 4px;",
            self.section_font, self.section_color, self.section_bg
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Theme;

    #[test]
    fn article_theme_names_include_more_reading_styles() {
        let names = Theme::names();

        assert!(names.contains(&"default"));
        assert!(names.contains(&"warm"));
        assert!(names.contains(&"dark"));
        assert!(names.contains(&"geek"));
        assert!(names.contains(&"paper"));
        assert!(names.contains(&"magazine"));
        assert!(names.contains(&"notebook"));
        assert!(names.contains(&"classic"));
        assert!(names.contains(&"forest"));
        assert!(names.contains(&"sunset"));
        assert!(names.contains(&"ocean"));
        assert!(names.contains(&"mono"));
        assert!(names.contains(&"editorial"));
        assert!(names.contains(&"zen"));
        assert!(names.contains(&"newsletter"));
        assert!(names.contains(&"academic"));
        assert!(names.contains(&"cyber"));
    }

    #[test]
    fn new_article_themes_have_distinct_names() {
        assert_eq!(Theme::from_name("paper").name, "paper");
        assert_eq!(Theme::from_name("magazine").name, "magazine");
        assert_eq!(Theme::from_name("notebook").name, "notebook");
        assert_eq!(Theme::from_name("classic").name, "classic");
        assert_eq!(Theme::from_name("forest").name, "forest");
        assert_eq!(Theme::from_name("sunset").name, "sunset");
        assert_eq!(Theme::from_name("ocean").name, "ocean");
        assert_eq!(Theme::from_name("mono").name, "mono");
        assert_eq!(Theme::from_name("editorial").name, "editorial");
        assert_eq!(Theme::from_name("zen").name, "zen");
        assert_eq!(Theme::from_name("newsletter").name, "newsletter");
        assert_eq!(Theme::from_name("academic").name, "academic");
        assert_eq!(Theme::from_name("cyber").name, "cyber");
    }

    #[test]
    fn forest_theme_uses_soft_reading_palette() {
        let theme = Theme::from_name("forest");

        assert_eq!(theme.text_color, "#3f3c35");
        assert_eq!(theme.text_muted, "#7a7469");
        assert_eq!(theme.heading_color, "#274934");
        assert_eq!(theme.section_bg, "#fbfcf7");
        assert_eq!(theme.accent, "#5f8f68");
        assert_eq!(theme.block_bg, "#f1f6ef");
        assert_eq!(theme.border, "#d5e1d1");
    }
}
