//! Article footer — configurable via `[footer]` section in moonpub.toml.
//! If no `[footer]` section or `enabled = false`, no footer is rendered.

use std::fs;

use crate::theme::Theme;
use base64::Engine;

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

fn local_to_data_uri(path: &str) -> Option<String> {
    let data = fs::read(path).ok()?;
    let mime = if path.to_lowercase().ends_with(".png") {
        "image/png"
    } else if path.to_lowercase().ends_with(".gif") {
        "image/gif"
    } else {
        "image/jpeg"
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    Some(format!("data:{mime};base64,{b64}"))
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

    // Brand card — always shown when not minimal
    if !minimal {
        html.push_str("<p style=\"margin:0.6em 0;color:#2c2c2c;font-size:15px;text-align:center;font-weight:bold;\">关于「寻月隐君」</p>\n\n");
        html.push_str("<section style=\"display:flex;align-items:center;padding:16px;margin:1em 0;background:#f7f8fa;border-radius:8px;\">\n");
        html.push_str("<div style=\"flex-shrink:0;margin-right:12px;\">\n");
        html.push_str("<img src=\"http://mmbiz.qpic.cn/sz_mmbiz_png/VatNneOWyngu9wIEDhDoiazP1Sw9SibtJBqibyOeTTTCmSzgSbM5Ke0K6lRjQRR7ic4MJKu84iasiapb4BRF805SgoCQ/0?wx_fmt=png\" style=\"width:56px;height:56px;border-radius:50%;\" alt=\"寻月隐君\">\n");
        html.push_str("</div>\n<div style=\"flex:1;min-width:0;\">\n");
        html.push_str(
            "<p style=\"margin:0;font-size:16px;font-weight:bold;color:#333;\">寻月隐君</p>\n",
        );
        html.push_str("<p style=\"margin:4px 0 0;font-size:13px;color:#888;line-height:1.6;overflow:hidden;text-overflow:ellipsis;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;\">🌟 寻月隐君 —— 技术的光，未来的道  关注《寻月隐君》，专注Web3、Python、Go、Rust等技术分享，探索区块链、智能合约、DApp等前沿内容。</p>\n");
        html.push_str("</div>\n<div style=\"flex-shrink:0;margin-left:8px;\">\n");
        html.push_str("<span style=\"font-size:12px;color:#888;\">公众号</span>\n");
        html.push_str("</div>\n</section>\n\n");
    }

    // Divider
    if !minimal && !cfg.divider.is_empty() {
        html.push_str(&format!(
            "<p style=\"margin:1em 0;color:{muted};font-size:15px;text-align:center;\">{}</p>\n\n",
            cfg.divider
        ));
    }

    let show_group_section = !minimal
        && (!cfg.description.is_empty() || !cfg.rules.is_empty() || !cfg.qrcode.is_empty());

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
    if !minimal {
        let qr_url = if cfg.qrcode.is_empty() {
            String::new()
        } else if cfg.qrcode.starts_with("http://") || cfg.qrcode.starts_with("https://") {
            cfg.qrcode.clone()
        } else {
            // Local file path — convert to data URI to avoid upload dependency.
            local_to_data_uri(&cfg.qrcode).unwrap_or_default()
        };
        // Skip the img entirely when there is nothing to show — an empty
        // src renders as a broken image in the WeChat editor.
        if !qr_url.is_empty() {
            html.push_str(&format!(
                "<p style=\"text-align:center;margin:1.5em 0 0.8em;\"><img src=\"{qr_url}\" style=\"display:block;margin:0 auto;max-width:80%;width:240px;height:auto;\" alt=\"群二维码\"></p>\n\n",
            ));
        }
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
    fn footer_without_qrcode_shows_group_copy_but_skips_qr_image() {
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

        assert!(html.contains("寻月阁"));
        assert!(html.contains("社群介绍"));
        assert!(html.contains("入群规则"));
        assert!(html.contains("扫码入群"));
        assert!(!html.contains("群二维码"));
        assert!(!html.contains("src=\"\""));
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
            qrcode: "https://example.com/qrcode.png".to_owned(),
            qrcode_note: "扫码入群".to_owned(),
            ..FooterConfig::default()
        };

        let html = render_footer(&cfg, &Theme::from_name("forest"));

        assert!(html.contains("寻月阁"));
        assert!(html.contains("社群介绍"));
        assert!(html.contains("入群规则"));
        assert!(html.contains("扫码入群"));
        assert!(html.contains("https://example.com/qrcode.png"));
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

    #[test]
    fn local_qrcode_is_embedded_as_data_uri() {
        let path = std::env::temp_dir().join("moonpub-test-qrcode.png");
        std::fs::write(&path, [0x89, 0x50, 0x4E, 0x47]).unwrap();
        let cfg = FooterConfig {
            enabled: true,
            qrcode: path.to_string_lossy().into_owned(),
            ..FooterConfig::default()
        };

        let html = render_footer(&cfg, &Theme::from_name("forest"));

        assert!(html.contains("src=\"data:image/png;base64,"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn unreadable_local_qrcode_skips_qr_image() {
        let cfg = FooterConfig {
            enabled: true,
            qrcode: "/nonexistent/qrcode.png".to_owned(),
            ..FooterConfig::default()
        };

        let html = render_footer(&cfg, &Theme::from_name("forest"));

        assert!(!html.contains("src=\"\""));
        assert!(!html.contains("群二维码"));
    }
}
