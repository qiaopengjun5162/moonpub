//! Article footer — 寻月隐君 固定结尾模板.
//! Based on brand.md. Stable, not changing.

use crate::theme::Theme;

pub struct FooterConfig {
    pub qrcode_path: String,
}

impl FooterConfig {
    pub fn from_config(_author: &str, qrcode_path: &str) -> Self {
        Self {
            qrcode_path: qrcode_path.to_owned(),
        }
    }
}

pub fn render_footer(cfg: &FooterConfig, theme: &Theme) -> String {
    let muted = theme.text_muted;

    let qrcode = if cfg.qrcode_path.is_empty() {
        String::new()
    } else {
        format!(
            "<p style=\"text-align:center;margin:1.5em 0 0.8em;\"><img src=\"{}\" style=\"max-width:80%;width:260px;\" alt=\"寻月阁群二维码\"></p>\n",
            cfg.qrcode_path
        )
    };

    format!(
        "{qrcode}<p style=\"margin:0.8em 0;color:{muted};font-size:13px;text-align:center;\">点个「赞」让我知道你喜欢，点个「推荐」让更多「寻月者」看到。</p>\n"
    )
}
