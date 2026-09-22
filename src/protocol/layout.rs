use super::escape_json;
use super::json_string_array;
use crate::layout_audit::LayoutAuditReport;
pub(crate) struct LayoutRecipe {
    pub id: &'static str,
    pub title: &'static str,
    pub best_for: &'static str,
    pub themes: &'static [&'static str],
    pub blocks: &'static [&'static str],
}

pub(crate) struct LayoutThemeGroup {
    pub id: &'static str,
    pub title: &'static str,
    pub themes: &'static [&'static str],
}

pub(crate) struct LayoutThemeSpotlight {
    pub id: &'static str,
    pub title: &'static str,
    pub best_for: &'static str,
    pub cover_style: &'static str,
    pub recipe_ids: &'static [&'static str],
}

pub(crate) const LAYOUT_THEME_GROUPS: &[LayoutThemeGroup] = &[
    LayoutThemeGroup {
        id: "tech-ai",
        title: "技术 / AI / 系统",
        themes: &[
            "geek",
            "geek-black",
            "blueprint",
            "ai-lab",
            "cyber",
            "notebook",
            "ocean",
        ],
    },
    LayoutThemeGroup {
        id: "life-essay",
        title: "生活 / 慢读 / 私人表达",
        themes: &[
            "mist",
            "letter",
            "moonlit",
            "porcelain",
            "forest",
            "zen",
            "warm",
        ],
    },
    LayoutThemeGroup {
        id: "photo-memory",
        title: "照片 / 记忆 / 现场",
        themes: &["gallery", "fieldnote", "porcelain", "mist", "warm"],
    },
    LayoutThemeGroup {
        id: "knowledge-note",
        title: "读书 / 研究 / 信息流",
        themes: &[
            "paper",
            "classic",
            "academic",
            "newsletter",
            "editorial",
            "notebook",
            "mono",
        ],
    },
];

pub(crate) const LAYOUT_THEME_SPOTLIGHTS: &[LayoutThemeSpotlight] = &[
    LayoutThemeSpotlight {
        id: "geek-black",
        title: "极客黑",
        best_for: "终端感 AI / Rust / Web3 工程复盘",
        cover_style: "geek-black",
        recipe_ids: &["tech-post", "ai-engineering-note"],
    },
    LayoutThemeSpotlight {
        id: "blueprint",
        title: "蓝图",
        best_for: "架构边界、系统设计、协议说明",
        cover_style: "blueprint",
        recipe_ids: &["system-design-review", "tech-post"],
    },
    LayoutThemeSpotlight {
        id: "ai-lab",
        title: "AI 实验室",
        best_for: "Agent 工作流、模型评测、AI 产品工程笔记",
        cover_style: "ai-lab",
        recipe_ids: &["ai-engineering-note"],
    },
    LayoutThemeSpotlight {
        id: "moonlit",
        title: "月下隐林",
        best_for: "克制私密的合集开篇和慢读随笔",
        cover_style: "literary",
        recipe_ids: &["quiet-opening", "collection-opener"],
    },
    LayoutThemeSpotlight {
        id: "fieldnote",
        title: "田野手记",
        best_for: "照片留档、散步记录、事实型生活片段",
        cover_style: "forest",
        recipe_ids: &["memory-note", "photo-story", "daily-image-card"],
    },
    LayoutThemeSpotlight {
        id: "paper",
        title: "纸面读书",
        best_for: "书摘、读书笔记、长文阅读",
        cover_style: "serif",
        recipe_ids: &["book-note"],
    },
    LayoutThemeSpotlight {
        id: "newsletter",
        title: "透明简报",
        best_for: "AI/Web3 日报、官方 release 汇总、带来源索引的可追溯信息流",
        cover_style: "workflow",
        recipe_ids: &["daily-report", "transparent-briefing"],
    },
];

pub(crate) const LAYOUT_RECIPES: &[LayoutRecipe] = &[
    LayoutRecipe {
        id: "life-essay",
        title: "生活随笔",
        best_for: "日常、散步、跑步、心绪记录",
        themes: &["mist", "letter", "forest"],
        blocks: &["meta-strip", "intro", "scene-card", "closing-card"],
    },
    LayoutRecipe {
        id: "spoken-note",
        title: "口述随记",
        best_for: "飞书妙记、散步录音、随口想法整理成文",
        themes: &["letter", "mist", "notebook"],
        blocks: &[
            "meta-strip",
            "intro",
            "letter-card",
            "summary",
            "closing-card",
        ],
    },
    LayoutRecipe {
        id: "collection-opener",
        title: "合集开篇",
        best_for: "栏目第一篇、付费合集序章、个人小专栏开场",
        themes: &["editorial", "mist", "letter"],
        blocks: &[
            "meta-strip",
            "intro",
            "letter-card",
            "scene-card",
            "closing-card",
        ],
    },
    LayoutRecipe {
        id: "quiet-opening",
        title: "静谧开篇",
        best_for: "闲月隐林、私人合集开场、需要克制边界感的第一篇",
        themes: &["moonlit", "porcelain", "letter"],
        blocks: &[
            "meta-strip",
            "intro",
            "letter-card",
            "scene-card",
            "closing-card",
        ],
    },
    LayoutRecipe {
        id: "photo-story",
        title: "照片记录",
        best_for: "同一天多张照片、跑步风景、旅行碎片、生活留档",
        themes: &["gallery", "mist", "warm"],
        blocks: &["intro", "photo-grid", "scene-card"],
    },
    LayoutRecipe {
        id: "memory-note",
        title: "记忆留档",
        best_for: "同一天照片、散步跑步记录、手机相册里的真实生活片段",
        themes: &["fieldnote", "gallery", "porcelain"],
        blocks: &[
            "meta-strip",
            "intro",
            "photo-grid",
            "scene-card",
            "closing-card",
        ],
    },
    LayoutRecipe {
        id: "daily-image-card",
        title: "日更贴图",
        best_for: "每天一组图文贴片、平台贴图流、用少量照片保持更新节奏",
        themes: &["gallery", "fieldnote", "newsletter"],
        blocks: &["meta-strip", "intro", "photo-grid", "compact-links"],
    },
    LayoutRecipe {
        id: "book-note",
        title: "读书笔记",
        best_for: "书摘、微信读书导入、阅读后的结构化思考",
        themes: &["paper", "classic", "academic"],
        blocks: &["book-info", "intro", "key-points", "pull-quote"],
    },
    LayoutRecipe {
        id: "tech-post",
        title: "技术文章",
        best_for: "教程、踩坑记录、项目复盘、工程说明",
        themes: &["geek", "geek-black", "blueprint", "notebook", "ocean"],
        blocks: &["intro", "callout", "steps", "summary"],
    },
    LayoutRecipe {
        id: "ai-engineering-note",
        title: "AI 工程笔记",
        best_for: "Agent 工作流、模型评测、提示词系统、AI 产品工程复盘",
        themes: &["ai-lab", "geek-black", "cyber", "blueprint"],
        blocks: &[
            "intro",
            "callout",
            "concept-card",
            "steps",
            "compact-links",
            "summary",
        ],
    },
    LayoutRecipe {
        id: "system-design-review",
        title: "系统设计复盘",
        best_for: "架构边界、模块拆分、协议设计、技术方案评审",
        themes: &["blueprint", "notebook", "academic", "geek"],
        blocks: &["intro", "concept-card", "steps", "key-points", "summary"],
    },
    LayoutRecipe {
        id: "daily-report",
        title: "日报周报",
        best_for: "AI/Web3 日报、资料索引、可追溯信息流",
        themes: &["notebook", "newsletter", "editorial"],
        blocks: &["intro", "divider", "summary", "callout", "compact-links"],
    },
    LayoutRecipe {
        id: "transparent-briefing",
        title: "透明信源简报",
        best_for: "AI 早报、官方 release 汇总、多源候选精编、每条消息都要保留来源和可信度",
        themes: &["newsletter", "notebook", "academic"],
        blocks: &["meta-strip", "intro", "summary", "callout", "compact-links"],
    },
];

pub(crate) fn layout_recipes_text() -> String {
    let mut output = String::from("layout recipes\n");
    output.push_str("  guide: docs/LAYOUT_RECIPES_ZH.md\n");
    output.push_str("\n  theme chooser\n");
    for group in LAYOUT_THEME_GROUPS {
        output.push_str(&format!(
            "    {} ({}): {}\n",
            group.title,
            group.id,
            group.themes.join(" / ")
        ));
    }
    output.push_str("\n  featured themes\n");
    for theme in LAYOUT_THEME_SPOTLIGHTS {
        output.push_str(&format!(
            "    {} ({}): {}; cover: {}; recipes: {}\n",
            theme.title,
            theme.id,
            theme.best_for,
            theme.cover_style,
            theme.recipe_ids.join(" / ")
        ));
    }
    for recipe in LAYOUT_RECIPES {
        output.push_str(&format!(
            "\n  {} ({})\n    best_for: {}\n    themes: {}\n    blocks: {}\n",
            recipe.title,
            recipe.id,
            recipe.best_for,
            recipe.themes.join(" / "),
            recipe.blocks.join(" -> ")
        ));
    }
    output.push_str("\n  tip: 一篇文章通常用 2-4 个视觉块就够了。");
    output
}

pub(crate) fn layout_recipes_json() -> String {
    let theme_groups = LAYOUT_THEME_GROUPS
        .iter()
        .map(|group| {
            let themes = group
                .themes
                .iter()
                .map(|theme| format!("\"{}\"", escape_json(theme)))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"id\":\"{}\",\"title\":\"{}\",\"themes\":[{}]}}",
                escape_json(group.id),
                escape_json(group.title),
                themes
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let theme_spotlights = LAYOUT_THEME_SPOTLIGHTS
        .iter()
        .map(|theme| {
            let recipe_ids = theme
                .recipe_ids
                .iter()
                .map(|recipe| format!("\"{}\"", escape_json(recipe)))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"id\":\"{}\",\"title\":\"{}\",\"best_for\":\"{}\",\"cover_style\":\"{}\",\"recipe_ids\":[{}]}}",
                escape_json(theme.id),
                escape_json(theme.title),
                escape_json(theme.best_for),
                escape_json(theme.cover_style),
                recipe_ids
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let recipes = LAYOUT_RECIPES
        .iter()
        .map(|recipe| {
            let themes = recipe
                .themes
                .iter()
                .map(|theme| format!("\"{}\"", escape_json(theme)))
                .collect::<Vec<_>>()
                .join(",");
            let blocks = recipe
                .blocks
                .iter()
                .map(|block| format!("\"{}\"", escape_json(block)))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"id\":\"{}\",\"title\":\"{}\",\"best_for\":\"{}\",\"themes\":[{}],\"blocks\":[{}]}}",
                escape_json(recipe.id),
                escape_json(recipe.title),
                escape_json(recipe.best_for),
                themes,
                blocks
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"command\":\"layout-recipes\",\"guide\":\"docs/LAYOUT_RECIPES_ZH.md\",\"theme_groups\":[{}],\"theme_spotlights\":[{}],\"recipes\":[{}]}}",
        theme_groups, theme_spotlights, recipes
    )
}

pub(crate) fn layout_audit_json(report: &LayoutAuditReport) -> String {
    let errors = json_string_array(&report.errors);
    let warnings = json_string_array(&report.warnings);
    format!(
        "{{\"command\":\"layout-audit\",\"html_path\":\"{}\",\"passed\":{},\"errors\":{},\"warnings\":{},\"next_step\":\"{}\"}}",
        escape_json(&report.html_path.display().to_string()),
        report.passed,
        errors,
        warnings,
        escape_json(if report.passed {
            "local preview or WeChat draft push"
        } else {
            "remove forbidden tags / attributes before publishing"
        })
    )
}

#[cfg(test)]
mod tests {

    #[test]
    fn layout_recipes_json_lists_recipe_choices() {
        let output = super::layout_recipes_json();

        assert!(output.contains(r#""command":"layout-recipes""#), "{output}");
        assert!(
            output.contains(r#""guide":"docs/LAYOUT_RECIPES_ZH.md""#),
            "{output}"
        );
        assert!(
            output.contains(
                r#""theme_groups":[{"id":"tech-ai","title":"技术 / AI / 系统","themes":["geek","geek-black","blueprint","ai-lab","cyber","notebook","ocean"]}"#
            ),
            "{output}"
        );
        assert!(
            output.contains(
                r#""theme_spotlights":[{"id":"geek-black","title":"极客黑","best_for":"终端感 AI / Rust / Web3 工程复盘","cover_style":"geek-black","recipe_ids":["tech-post","ai-engineering-note"]}"#
            ),
            "{output}"
        );
        assert!(output.contains(r#""id":"photo-story""#), "{output}");
        assert!(
            output.contains(r#""blocks":["intro","photo-grid","scene-card"]"#),
            "{output}"
        );
        assert!(output.contains(r#""id":"spoken-note""#), "{output}");
        assert!(
            output.contains(
                r#""blocks":["meta-strip","intro","letter-card","summary","closing-card"]"#
            ),
            "{output}"
        );
        assert!(output.contains(r#""id":"collection-opener""#), "{output}");
        assert!(
            output.contains(
                r#""blocks":["meta-strip","intro","letter-card","scene-card","closing-card"]"#
            ),
            "{output}"
        );
        assert!(output.contains(r#""id":"quiet-opening""#), "{output}");
        assert!(
            output.contains(r#""themes":["moonlit","porcelain","letter"]"#),
            "{output}"
        );
        assert!(output.contains(r#""id":"memory-note""#), "{output}");
        assert!(
            output.contains(r#""themes":["fieldnote","gallery","porcelain"]"#),
            "{output}"
        );
        assert!(output.contains(r#""id":"daily-image-card""#), "{output}");
        assert!(
            output.contains(r#""themes":["gallery","fieldnote","newsletter"]"#),
            "{output}"
        );
        assert!(
            output.contains(r#""blocks":["meta-strip","intro","photo-grid","compact-links"]"#),
            "{output}"
        );
        assert!(output.contains(r#""id":"daily-report""#), "{output}");
        assert!(
            output.contains(r#""themes":["geek","geek-black","blueprint","notebook","ocean"]"#),
            "{output}"
        );
        assert!(output.contains(r#""id":"ai-engineering-note""#), "{output}");
        assert!(
            output.contains(r#""themes":["ai-lab","geek-black","cyber","blueprint"]"#),
            "{output}"
        );
        assert!(
            output.contains(
                r#""blocks":["intro","callout","concept-card","steps","compact-links","summary"]"#
            ),
            "{output}"
        );
        assert!(
            output.contains(r#""id":"system-design-review""#),
            "{output}"
        );
        assert!(
            output.contains(r#""themes":["blueprint","notebook","academic","geek"]"#),
            "{output}"
        );
        assert!(
            output.contains(r#""blocks":["intro","divider","summary","callout","compact-links"]"#),
            "{output}"
        );
        assert!(
            output.contains(r#""id":"transparent-briefing""#),
            "{output}"
        );
        assert!(
            output.contains(r#""themes":["newsletter","notebook","academic"]"#),
            "{output}"
        );
        assert!(
            output
                .contains(r#""blocks":["meta-strip","intro","summary","callout","compact-links"]"#),
            "{output}"
        );
    }

    #[test]
    fn layout_audit_json_reports_errors_warnings_and_next_step() {
        let report = crate::layout_audit::LayoutAuditReport {
            html_path: std::path::PathBuf::from("Articles/drafts/demo.html"),
            passed: false,
            errors: vec!["contains forbidden tag `<div`".to_owned()],
            warnings: vec!["contains full HTML document shell".to_owned()],
        };

        let output = super::layout_audit_json(&report);

        assert!(output.contains(r#""command":"layout-audit""#), "{output}");
        assert!(
            output.contains(r#""html_path":"Articles/drafts/demo.html""#),
            "{output}"
        );
        assert!(output.contains(r#""passed":false"#), "{output}");
        assert!(
            output.contains(r#""errors":["contains forbidden tag `<div`"]"#),
            "{output}"
        );
        assert!(
            output.contains(r#""warnings":["contains full HTML document shell"]"#),
            "{output}"
        );
        assert!(
            output
                .contains(r#""next_step":"remove forbidden tags / attributes before publishing""#),
            "{output}"
        );
    }
}
