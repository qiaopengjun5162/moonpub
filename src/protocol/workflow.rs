use super::escape_json;
pub(crate) struct WorkflowRegistryEntry {
    pub id: &'static str,
    pub title: &'static str,
    pub package: &'static str,
    pub status: &'static str,
    pub owner: &'static str,
    pub entry_command: &'static str,
    pub safe_start_command: &'static str,
    pub next_command: &'static str,
    pub user_value: &'static str,
    pub requires_network: bool,
    pub requires_browser: bool,
    pub production_boundary: &'static str,
    pub evidence_status: &'static str,
    pub docs: &'static [&'static str],
}

pub(crate) const WORKFLOW_REGISTRY: &[WorkflowRegistryEntry] = &[
    WorkflowRegistryEntry {
        id: "current-article",
        title: "当前 Markdown 文章",
        package: "core/current-article",
        status: "active",
        owner: "moonpub-core",
        entry_command: "moonpub check <article.md>",
        safe_start_command: "moonpub preview <article.md>",
        next_command: "moonpub push <article.md> --render",
        user_value: "把已经写好的 Markdown 先变成本地可检查的公众号 HTML，再决定是否推进到微信草稿。",
        requires_network: false,
        requires_browser: false,
        production_boundary: "local render and preview are safe; push explicitly touches WeChat API",
        evidence_status: "code-and-ci",
        docs: &[
            "docs/FIRST_RUN_WALKTHROUGH_ZH.md",
            "docs/AGENT_PROTOCOL_ZH.md",
        ],
    },
    WorkflowRegistryEntry {
        id: "feishu-minutes",
        title: "飞书妙记到草稿",
        package: "input/feishu-minutes",
        status: "active",
        owner: "moonpub-input",
        entry_command: "moonpub intake feishu --latest --draft --preview --json",
        safe_start_command: "moonpub intake feishu --latest --draft --preview --no-open --json",
        next_command: "moonpub intake feishu --latest --draft --push --json",
        user_value: "把口述和转写先落到 Inbox，再生成可编辑草稿，方便去 AI 味、补充细节和保留来源。",
        requires_network: true,
        requires_browser: false,
        production_boundary: "reads Feishu Minutes and may call AI for draft generation; push is explicit",
        evidence_status: "cli-verified-needs-plugin-screenshot",
        docs: &[
            "docs/FIRST_RUN_EVIDENCE_CHECKLIST_ZH.md",
            "docs/INPUT_MODEL_ZH.md",
        ],
    },
    WorkflowRegistryEntry {
        id: "photo-memory",
        title: "照片素材到草稿",
        package: "input/photos",
        status: "active",
        owner: "moonpub-input",
        entry_command: "moonpub intake photos <file-or-dir> --draft --preview --json",
        safe_start_command: "moonpub intake photos <file-or-dir> --draft --preview --no-open --json",
        next_command: "moonpub intake photos <file-or-dir> --draft --push --json",
        user_value: "把同一天或同一组照片先沉淀成朴素记录，避免照片只留在手机里最后被删掉。",
        requires_network: false,
        requires_browser: false,
        production_boundary: "local photo metadata import and AI draft generation; push is explicit",
        evidence_status: "code-and-ci-needs-real-sample",
        docs: &[
            "docs/FIRST_RUN_EVIDENCE_CHECKLIST_ZH.md",
            "docs/INPUT_MODEL_ZH.md",
        ],
    },
    WorkflowRegistryEntry {
        id: "wechat-draft",
        title: "微信公众号草稿推进",
        package: "publish/wechat-draft",
        status: "active",
        owner: "moonpub-publish",
        entry_command: "moonpub push <article.md> --render",
        safe_start_command: "moonpub wechat-health",
        next_command: "moonpub configure --headed",
        user_value: "先检查微信凭证和浏览器登录态，再进入微信草稿与后台预览，最终发表仍由你确认。",
        requires_network: true,
        requires_browser: true,
        production_boundary: "touches WeChat API and assisted browser automation; final publish remains manual",
        evidence_status: "real-command-verified-needs-redacted-screenshots",
        docs: &[
            "docs/RELEASE_GATE_v0.4.2_ZH.md",
            "docs/WECHAT_REGRESSION_CHECKLIST_ZH.md",
        ],
    },
    WorkflowRegistryEntry {
        id: "wechat-content-review",
        title: "公众号内容复盘",
        package: "review/wechat-content",
        status: "active",
        owner: "moonpub-core",
        entry_command: "moonpub wechat-checklist",
        safe_start_command: "moonpub wechat-checklist --json",
        next_command: "moonpub check <article.md> && moonpub preflight <article.md>",
        user_value: "在触达微信前先复盘账号定位、选题稳定、标题入口、完读体验和运营红线，避免只会推送、不知道怎么改稿。",
        requires_network: false,
        requires_browser: false,
        production_boundary: "local read-only content checklist; does not call AI, WeChat API, or browser automation",
        evidence_status: "content-principles-no-runtime-side-effect",
        docs: &["docs/USER_GUIDE.md", "docs/RECOMMENDED_WORKFLOWS_ZH.md"],
    },
];

pub(crate) fn workflow_registry_text() -> String {
    let mut output = String::from("workflow registry\n");
    output.push_str("  source: built-in MoonPub workflow contracts\n");
    for workflow in WORKFLOW_REGISTRY {
        output.push_str(&format!(
            "\n  {} ({})\n    package: {}\n    status: {}\n    owner: {}\n    value: {}\n    safe_start: {}\n    next: {}\n    network: {}\n    browser: {}\n    boundary: {}\n    evidence: {}\n",
            workflow.title,
            workflow.id,
            workflow.package,
            workflow.status,
            workflow.owner,
            workflow.user_value,
            workflow.safe_start_command,
            workflow.next_command,
            workflow.requires_network,
            workflow.requires_browser,
            workflow.production_boundary,
            workflow.evidence_status
        ));
    }
    output.push_str("\n  tip: 先走 safe_start_command，确认后再进入 next_command。");
    output
}

pub(crate) fn workflow_registry_json() -> String {
    let workflows = WORKFLOW_REGISTRY
        .iter()
        .map(|workflow| {
            let docs = workflow
                .docs
                .iter()
                .map(|doc| format!("\"{}\"", escape_json(doc)))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"id\":\"{}\",\"title\":\"{}\",\"package\":\"{}\",\"status\":\"{}\",\"owner\":\"{}\",\"entry_command\":\"{}\",\"safe_start_command\":\"{}\",\"next_command\":\"{}\",\"user_value\":\"{}\",\"requires_network\":{},\"requires_browser\":{},\"production_boundary\":\"{}\",\"evidence_status\":\"{}\",\"docs\":[{}]}}",
                escape_json(workflow.id),
                escape_json(workflow.title),
                escape_json(workflow.package),
                escape_json(workflow.status),
                escape_json(workflow.owner),
                escape_json(workflow.entry_command),
                escape_json(workflow.safe_start_command),
                escape_json(workflow.next_command),
                escape_json(workflow.user_value),
                workflow.requires_network,
                workflow.requires_browser,
                escape_json(workflow.production_boundary),
                escape_json(workflow.evidence_status),
                docs
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"command\":\"workflow-registry\",\"source\":\"built-in\",\"workflows\":[{}]}}",
        workflows
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn workflow_registry_json_lists_first_run_contracts() {
        let output = super::workflow_registry_json();

        assert!(
            output.contains(r#""command":"workflow-registry""#),
            "{output}"
        );
        assert!(output.contains(r#""id":"current-article""#), "{output}");
        assert!(
            output.contains(r#""package":"input/feishu-minutes""#),
            "{output}"
        );
        assert!(
            output.contains(r#""safe_start_command":"moonpub intake photos <file-or-dir> --draft --preview --no-open --json""#),
            "{output}"
        );
        assert!(
            output.contains(r#""user_value":"把同一天或同一组照片先沉淀成朴素记录，避免照片只留在手机里最后被删掉。""#),
            "{output}"
        );
        assert!(output.contains(r#""requires_browser":true"#), "{output}");
        assert!(
            output.contains(r#""docs":["docs/RELEASE_GATE_v0.4.2_ZH.md","docs/WECHAT_REGRESSION_CHECKLIST_ZH.md"]"#),
            "{output}"
        );
    }
}
