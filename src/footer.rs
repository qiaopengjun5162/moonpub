//! Article footer / ending template for WeChat articles.
//! Based on brand.md and actual published article content.

use crate::theme::Theme;

pub struct FooterConfig {
    pub author: String,
}

impl FooterConfig {
    pub fn from_config(author: &str) -> Self {
        Self {
            author: if author.is_empty() {
                "寻月隐君".to_owned()
            } else {
                author.to_owned()
            },
        }
    }
}

/// Render the standard article ending: community invitation + banner + CTA.
pub fn render_footer(cfg: &FooterConfig, theme: &Theme) -> String {
    let author = &cfg.author;
    let muted = theme.text_muted;
    let accent = theme.accent;

    let banner_src = "http://mmbiz.qpic.cn/mmbiz_png/22kVflKPKaz3O4MSRb2u2yKrBNbRfDqicUjBNyyJIT1Qp6icRHFaGFZevTl24eGDaaYXFSg5YkKVFOWgia47Ck3OnRwEMwA2bJIuuRAIAicraWA/0?wx_fmt=png";

    // Community CTA section matching the actual article ending style
    format!(
        r#"<section style="margin-top:3em;padding-top:2em;border-top:1px solid #e8e8e8;">

<p style="margin:1em 0;color:{muted};font-size:15px;text-align:center;">— · —</p>

<p style="margin:0.6em 0;color:{accent};font-size:15px;text-align:center;font-weight:bold;">加入「寻月阁」</p>

<p style="margin:0.6em 0;color:{muted};font-size:14px;text-align:center;line-height:1.8;">
无论你是初出茅庐的新手，还是久经沙场的老兵——<br>
「寻月阁」欢迎每一位对技术保持热爱与好奇心的朋友。
</p>

<p style="margin:1em 0 0.4em;color:{muted};font-size:13px;text-align:left;line-height:1.8;">
<b>入阁规矩 📜</b><br>
· 亮出身份，以诚会友：入群后修改群昵称<br>
· 专注技术，言之有物：鼓励深度讨论，欢迎新手提问<br>
· 君子之交，和而不同：尊重每一位成员，求同存异<br>
· 广告勿扰，保持纯粹：严禁任何形式广告
</p>

<p style="margin:1.2em 0 0.6em;color:{muted};font-size:13px;text-align:center;">
长按下方二维码即可入阁。<br>
若二维码过期，请在公众号后台回复 <b style="color:{accent};">加群</b> 获取最新二维码。
</p>

<p style="text-align:center;margin:1.5em 0 0.8em;"><img src="{banner_src}" style="max-width:100%;" alt="关注{author}"></p>

<p style="margin:0.8em 0;color:{muted};font-size:13px;text-align:center;">
点个「赞」让我知道你喜欢，点个「推荐」让更多「寻月者」看到。
</p>

</section>
"#
    )
}
