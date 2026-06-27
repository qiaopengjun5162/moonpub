//! Article footer — configurable via `[footer]` section in moonpub.toml.
//! If no `[footer]` section or `enabled = false`, no footer is rendered.

use crate::theme::Theme;

/// All fields are optional. Empty strings are silently skipped.
/// Set `enabled = true` to render the footer; default is disabled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FooterConfig {
    pub enabled: bool,
    pub variant: String,
    pub title: String,
    pub description: String,
    pub rules: String,
    pub qrcode: String,
    pub qrcode_note: String,
    pub follow_image: String,
    pub follow_text: String,
    pub divider: String,
}

impl FooterConfig {
    pub fn from_config(enabled: bool, qrcode_path: &str) -> Self {
        Self {
            enabled,
            variant: default_variant(),
            qrcode: qrcode_path.to_owned(),
            ..Default::default()
        }
    }
}

fn default_variant() -> String {
    "community".to_owned()
}

impl Default for FooterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            variant: default_variant(),
            title: String::new(),
            description: String::new(),
            rules: String::new(),
            qrcode: String::new(),
            qrcode_note: String::new(),
            follow_image: String::new(),
            follow_text: String::new(),
            divider: String::new(),
        }
    }
}

pub fn render_footer(cfg: &FooterConfig, theme: &Theme) -> String {
    if !cfg.enabled {
        return String::new();
    }

    let muted = theme.text_muted;
    let accent = theme.accent;

    let mut html =
        "<section style=\"margin-top:3em;padding-top:2em;border-top:1px solid #e8e8e8;\">\n\n"
            .to_string();

    let minimal = cfg.variant == "minimal";

    // Divider
    if !minimal && !cfg.divider.is_empty() {
        html.push_str(&format!(
            "<p style=\"margin:1em 0;color:{muted};font-size:15px;text-align:center;\">{}</p>\n\n",
            cfg.divider
        ));
    }

    let show_group_section = !minimal && !cfg.qrcode.is_empty();

    // Title
    if show_group_section && !cfg.title.is_empty() {
        html.push_str(&format!(
            "<p style=\"margin:0.6em 0;color:{accent};font-size:15px;text-align:center;font-weight:bold;\">{}</p>\n\n",
            cfg.title
        ));
    }

    // Description
    if show_group_section && !cfg.description.is_empty() {
        html.push_str(&format!(
            "<p style=\"margin:0.6em 0;color:{muted};font-size:14px;text-align:center;line-height:1.8;\">{}</p>\n\n",
            cfg.description.replace('\n', "<br>\n")
        ));
    }

    // Rules
    if show_group_section && !cfg.rules.is_empty() {
        html.push_str(&format!(
            "<p style=\"margin:1em 0 0.4em;color:{muted};font-size:13px;text-align:left;line-height:1.8;\">{}</p>\n\n",
            cfg.rules.replace('\n', "<br>\n")
        ));
    }

    // QR code note
    if show_group_section && !cfg.qrcode_note.is_empty() {
        html.push_str(&format!(
            "<p style=\"margin:1.2em 0 0.6em;color:{muted};font-size:13px;text-align:center;\">{}</p>\n\n",
            cfg.qrcode_note.replace('\n', "<br>\n")
        ));
    }

    // QR code image
    if !minimal && !cfg.qrcode.is_empty() {
        html.push_str(&format!(
            "<p style=\"text-align:center;margin:1.5em 0 0.8em;\"><img src=\"{}\" style=\"max-width:80%;width:260px;\" alt=\"群二维码\"></p>\n\n",
            cfg.qrcode
        ));
    }

    // Follow image
    if !cfg.follow_image.is_empty() {
        html.push_str(&format!(
            "<p style=\"text-align:center;margin:1.5em 0 0.8em;\"><img src=\"{}\" style=\"max-width:100%;\" alt=\"关注公众号\"></p>\n\n",
            cfg.follow_image
        ));
    }

    // Follow text
    if !cfg.follow_text.is_empty() {
        html.push_str(&format!(
            "<p style=\"margin:0.8em 0;color:{muted};font-size:13px;text-align:center;\">{}</p>\n\n",
            cfg.follow_text
        ));
    }

    html.push_str("</section>\n");
    html
}

#[cfg(test)]
mod tests {
    use super::{FooterConfig, render_footer};
    use crate::theme::Theme;

    #[test]
    fn footer_without_qrcode_hides_group_copy_but_keeps_follow_cta() {
        let cfg = FooterConfig {
            enabled: true,
            title: "寻月阁".to_owned(),
            description: "社群介绍".to_owned(),
            rules: "入群规则".to_owned(),
            qrcode_note: "扫码入群".to_owned(),
            follow_image: "https://example.com/follow.png".to_owned(),
            follow_text: "点个赞让我知道你喜欢。".to_owned(),
            ..FooterConfig::default()
        };

        let html = render_footer(&cfg, &Theme::from_name("forest"));

        assert!(!html.contains("寻月阁"));
        assert!(!html.contains("社群介绍"));
        assert!(!html.contains("入群规则"));
        assert!(!html.contains("扫码入群"));
        assert!(html.contains("https://example.com/follow.png"));
        assert!(html.contains("点个赞让我知道你喜欢。"));
    }

    #[test]
    fn footer_with_qrcode_keeps_group_copy() {
        let cfg = FooterConfig {
            enabled: true,
            title: "寻月阁".to_owned(),
            description: "社群介绍".to_owned(),
            rules: "入群规则".to_owned(),
            qrcode: "qrcode.png".to_owned(),
            qrcode_note: "扫码入群".to_owned(),
            ..FooterConfig::default()
        };

        let html = render_footer(&cfg, &Theme::from_name("forest"));

        assert!(html.contains("寻月阁"));
        assert!(html.contains("社群介绍"));
        assert!(html.contains("入群规则"));
        assert!(html.contains("扫码入群"));
        assert!(html.contains("qrcode.png"));
    }

    #[test]
    fn minimal_footer_hides_all_community_fields_even_with_qrcode() {
        let cfg = FooterConfig {
            enabled: true,
            variant: "minimal".to_owned(),
            title: "寻月阁".to_owned(),
            description: "社群介绍".to_owned(),
            rules: "入群规则".to_owned(),
            qrcode: "qrcode.png".to_owned(),
            qrcode_note: "扫码入群".to_owned(),
            follow_image: "https://example.com/follow.png".to_owned(),
            follow_text: "点个赞让我知道你喜欢。".to_owned(),
            divider: "— · —".to_owned(),
        };

        let html = render_footer(&cfg, &Theme::from_name("forest"));

        assert!(!html.contains("寻月阁"));
        assert!(!html.contains("社群介绍"));
        assert!(!html.contains("入群规则"));
        assert!(!html.contains("扫码入群"));
        assert!(!html.contains("qrcode.png"));
        assert!(!html.contains("— · —"));
        assert!(html.contains("https://example.com/follow.png"));
        assert!(html.contains("点个赞让我知道你喜欢。"));
    }
}
