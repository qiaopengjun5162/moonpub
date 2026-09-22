use super::escape_json;

pub(crate) struct WechatChecklistSection {
    pub id: &'static str,
    pub title: &'static str,
    pub why: &'static str,
    pub checks: &'static [&'static str],
}

pub(crate) const WECHAT_CHECKLIST_SECTIONS: &[WechatChecklistSection] = &[
    WechatChecklistSection {
        id: "account-positioning",
        title: "账号定位",
        why: "先让读者和平台知道这个账号持续解决什么问题。",
        checks: &[
            "头像、名称、简介是否指向同一个细分领域",
            "简介是否说清你是谁、服务谁、提供什么价值",
            "文章开头是否延续固定身份和价值承诺",
        ],
    },
    WechatChecklistSection {
        id: "topic-consistency",
        title: "选题稳定",
        why: "新号或新栏目先稳定标签，不要让系统和读者猜方向。",
        checks: &[
            "最近一组文章是否围绕同一个细分话题",
            "正文是否自然重复核心关键词，而不是泛泛而谈",
            "是否保留对标来源和可复盘的标题/结构拆解",
        ],
    },
    WechatChecklistSection {
        id: "title-hook",
        title: "标题入口",
        why: "标题不是摘要，而是读者愿意点进来的入口。",
        checks: &[
            "标题是否给出数字、反差、疑问、利益或好奇点",
            "标题是否能让目标读者一眼判断和自己有关",
            "标题是否控制在微信标题硬约束内",
        ],
    },
    WechatChecklistSection {
        id: "read-through",
        title: "完读体验",
        why: "公众号文章要让人读完，不只是把观点写完。",
        checks: &[
            "开头是否像和朋友说话，而不是端着讲道理",
            "正文是否用短段落、故事、冲突和情绪推进阅读",
            "结尾是否有明确总结、互动问题或下一步行动",
        ],
    },
    WechatChecklistSection {
        id: "safety-boundary",
        title: "运营红线",
        why: "MoonPub 只辅助进入可发布状态，不鼓励污染画像或违规刷量。",
        checks: &[
            "不做刷量、诱导点击、亲友集中干预等污染画像动作",
            "转载或借鉴内容必须重写、标注来源并人工确认版权风险",
            "发布前先跑本地预览、排版审计和 preflight",
        ],
    },
];

pub(crate) fn wechat_checklist_text() -> String {
    let mut output = String::from("wechat content checklist\n");
    output.push_str("  source: built-in public-account content review checklist\n");
    output.push_str(
        "  boundary: local read-only checklist; no WeChat API, browser, AI call, or publishing\n",
    );
    output.push_str("  next: moonpub check <article.md> && moonpub preflight <article.md>\n");
    for section in WECHAT_CHECKLIST_SECTIONS {
        output.push_str(&format!(
            "\n  {} ({})\n    why: {}\n",
            section.title, section.id, section.why
        ));
        for check in section.checks {
            output.push_str(&format!("    - {check}\n"));
        }
    }
    output.push_str("\n  tip: 先用这份清单复盘内容，再进入 preview / push。");
    output
}

pub(crate) fn wechat_checklist_json() -> String {
    let sections = WECHAT_CHECKLIST_SECTIONS
        .iter()
        .map(|section| {
            let checks = section
                .checks
                .iter()
                .map(|check| format!("\"{}\"", escape_json(check)))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"id\":\"{}\",\"title\":\"{}\",\"why\":\"{}\",\"checks\":[{}]}}",
                escape_json(section.id),
                escape_json(section.title),
                escape_json(section.why),
                checks
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"command\":\"wechat-checklist\",\"source\":\"built-in\",\"boundary\":\"local read-only checklist; no WeChat API, browser, AI call, or publishing\",\"sections\":[{}],\"next_command\":\"moonpub check <article.md> && moonpub preflight <article.md>\",\"next_step\":\"review positioning, title hook, read-through, and safety boundary before preview or push\"}}",
        sections
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wechat_checklist_json_includes_all_checklist_sections() {
        let json = wechat_checklist_json();
        assert!(json.contains(r#""command":"wechat-checklist"#));
        assert!(json.contains("account-positioning"));
        assert!(json.contains("topic-consistency"));
    }
}
