use std::path::Path;

use crate::config::Config;
use crate::error::AppError;
use crate::json_util::escape_json;

const CAPABILITIES_SCHEMA_VERSION: &str = "capabilities/v1";

pub struct TargetCapability {
    pub id: &'static str,
    pub display_name: &'static str,
    pub kind: &'static str,
    pub command: &'static [&'static str],
    pub article_arg: &'static str,
    pub required_env: &'static [&'static str],
    pub required_config: &'static [&'static str],
    pub requires_network: bool,
    pub requires_browser: bool,
    pub risk: &'static str,
    pub next_step: &'static str,
}

pub struct PublishContext<'a> {
    pub articles_dir: &'a Path,
    pub article: &'a Path,
    pub auto_render: bool,
    pub temporary_profile: bool,
    pub config: &'a Config,
}

pub struct PublishOutcome {
    pub message: String,
}

pub struct ExportContext<'a> {
    pub articles_dir: &'a Path,
    pub article: &'a Path,
    pub export_root: &'a Path,
}

pub struct ExportOutcome {
    pub message: String,
}

pub trait PublishTarget {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn requires_network(&self) -> bool;
    fn requires_browser(&self) -> bool;
    fn publish(&self, ctx: PublishContext<'_>) -> Result<PublishOutcome, AppError>;
}

pub fn run_publish_target(
    target: &impl PublishTarget,
    ctx: PublishContext<'_>,
) -> Result<PublishOutcome, AppError> {
    target.publish(ctx)
}

pub trait ExportTarget {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn export(&self, ctx: ExportContext<'_>) -> Result<ExportOutcome, AppError>;
}

pub fn run_export_target(
    target: &impl ExportTarget,
    ctx: ExportContext<'_>,
) -> Result<ExportOutcome, AppError> {
    target.export(ctx)
}

pub fn builtin_capabilities() -> Vec<TargetCapability> {
    vec![
        TargetCapability {
            id: "wechat-draft",
            display_name: "WeChat Draft",
            kind: "publish",
            command: &["publish", "{article}", "--target", "wechat-draft"],
            article_arg: "{article}",
            required_env: &["WECHAT_APPID", "WECHAT_SECRET"],
            required_config: &[],
            requires_network: true,
            requires_browser: true,
            risk: "calls WeChat API and may open Chrome automation",
            next_step: "manual final confirmation in WeChat backend",
        },
        TargetCapability {
            id: "zola",
            display_name: "Zola",
            kind: "export",
            command: &["export", "{article}", "--target", "zola"],
            article_arg: "{article}",
            required_env: &[],
            required_config: &["blog.root"],
            requires_network: false,
            requires_browser: false,
            risk: "writes Markdown files into the configured local blog root",
            next_step: "review the generated Zola Markdown before publishing the blog",
        },
    ]
}

pub fn capabilities_text() -> String {
    let mut output = String::from("capabilities\n");
    for capability in builtin_capabilities() {
        output.push_str(&format!(
            "  {} ({})\n    kind: {}\n    network: {}\n    browser: {}\n    env: {}\n    config: {}\n    risk: {}\n    next: {}\n",
            capability.id,
            capability.display_name,
            capability.kind,
            yes_no(capability.requires_network),
            yes_no(capability.requires_browser),
            list_or_none(capability.required_env),
            list_or_none(capability.required_config),
            capability.risk,
            capability.next_step
        ));
    }
    output.trim_end().to_owned()
}

pub fn capabilities_json() -> String {
    let items = builtin_capabilities()
        .into_iter()
        .map(|capability| {
            let command = capability
                .command
                .iter()
                .map(|part| format!("\"{}\"", escape_json(part)))
                .collect::<Vec<_>>()
                .join(",");
            let required_env = json_string_array(capability.required_env);
            let required_config = json_string_array(capability.required_config);
            format!(
                "{{\"id\":\"{}\",\"display_name\":\"{}\",\"kind\":\"{}\",\"command\":[{}],\"article_arg\":\"{}\",\"required_env\":{},\"required_config\":{},\"requires_network\":{},\"requires_browser\":{},\"risk\":\"{}\",\"next_step\":\"{}\"}}",
                escape_json(capability.id),
                escape_json(capability.display_name),
                escape_json(capability.kind),
                command,
                escape_json(capability.article_arg),
                required_env,
                required_config,
                capability.requires_network,
                capability.requires_browser,
                escape_json(capability.risk),
                escape_json(capability.next_step)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"schema_version\":\"{}\",\"moonpub_version\":\"{}\",\"targets\":[{items}]}}",
        escape_json(CAPABILITIES_SCHEMA_VERSION),
        escape_json(env!("CARGO_PKG_VERSION"))
    )
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn list_or_none(values: &[&str]) -> String {
    if values.is_empty() {
        "none".to_owned()
    } else {
        values.join(", ")
    }
}

fn json_string_array(values: &[&str]) -> String {
    let items = values
        .iter()
        .map(|value| format!("\"{}\"", escape_json(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{items}]")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::config::Config;
    use crate::error::AppError;
    use crate::plugin::{
        ExportContext, ExportOutcome, ExportTarget, PublishContext, PublishOutcome, PublishTarget,
        run_export_target, run_publish_target,
    };

    struct FakeTarget;

    impl PublishTarget for FakeTarget {
        fn id(&self) -> &'static str {
            "fake"
        }

        fn display_name(&self) -> &'static str {
            "Fake"
        }

        fn requires_network(&self) -> bool {
            false
        }

        fn requires_browser(&self) -> bool {
            false
        }

        fn publish(&self, ctx: PublishContext<'_>) -> Result<PublishOutcome, AppError> {
            Ok(PublishOutcome {
                message: format!(
                    "{}:{}:{}",
                    ctx.articles_dir.display(),
                    ctx.article.display(),
                    ctx.auto_render
                ),
            })
        }
    }

    struct FakeExportTarget;

    impl ExportTarget for FakeExportTarget {
        fn id(&self) -> &'static str {
            "fake-export"
        }

        fn display_name(&self) -> &'static str {
            "Fake Export"
        }

        fn export(&self, ctx: ExportContext<'_>) -> Result<ExportOutcome, AppError> {
            Ok(ExportOutcome {
                message: format!(
                    "{}:{}:{}",
                    ctx.articles_dir.display(),
                    ctx.article.display(),
                    ctx.export_root.display()
                ),
            })
        }
    }

    #[test]
    fn publish_target_exposes_capability_metadata() {
        let target = FakeTarget;

        assert_eq!(target.id(), "fake");
        assert_eq!(target.display_name(), "Fake");
        assert!(!target.requires_network());
        assert!(!target.requires_browser());
    }

    #[test]
    fn run_publish_target_dispatches_context() -> Result<(), Box<dyn std::error::Error>> {
        let cfg = Config::default();
        let target = FakeTarget;
        let outcome = run_publish_target(
            &target,
            PublishContext {
                articles_dir: Path::new("/vault"),
                article: Path::new("Articles/drafts/demo.md"),
                auto_render: true,
                temporary_profile: false,
                config: &cfg,
            },
        )?;

        assert_eq!(outcome.message, "/vault:Articles/drafts/demo.md:true");
        Ok(())
    }

    #[test]
    fn export_target_dispatches_context() -> Result<(), Box<dyn std::error::Error>> {
        let target = FakeExportTarget;

        assert_eq!(target.id(), "fake-export");
        assert_eq!(target.display_name(), "Fake Export");

        let outcome = run_export_target(
            &target,
            ExportContext {
                articles_dir: Path::new("/vault"),
                article: Path::new("Articles/published/demo.md"),
                export_root: Path::new("/blog"),
            },
        )?;

        assert_eq!(outcome.message, "/vault:Articles/published/demo.md:/blog");
        Ok(())
    }

    #[test]
    fn capabilities_json_exposes_wechat_draft_risks() {
        let json = crate::plugin::capabilities_json();

        assert!(json.contains(r#""id":"wechat-draft""#));
        assert!(json.contains(r#""requires_network":true"#));
        assert!(json.contains(r#""requires_browser":true"#));
        assert!(json.contains("manual final confirmation"));
    }

    #[test]
    fn capabilities_json_exposes_wechat_draft_command_template() {
        let json = crate::plugin::capabilities_json();

        assert!(json.contains(r#""command":["publish","{article}","--target","wechat-draft"]"#));
        assert!(json.contains(r#""article_arg":"{article}""#));
    }

    #[test]
    fn capabilities_json_exposes_wechat_draft_requirements() {
        let json = crate::plugin::capabilities_json();

        assert!(json.contains(r#""required_env":["WECHAT_APPID","WECHAT_SECRET"]"#));
        assert!(json.contains(r#""required_config":[]"#));
    }

    #[test]
    fn capabilities_json_exposes_schema_and_cli_version() {
        let json = crate::plugin::capabilities_json();

        assert!(
            json.starts_with(r#"{"schema_version":"capabilities/v1","moonpub_version":""#),
            "{json}"
        );
        assert!(json.contains(r#""targets":["#));
    }

    #[test]
    fn capabilities_json_exposes_zola_export() {
        let json = crate::plugin::capabilities_json();

        assert!(json.contains(r#""id":"zola""#));
        assert!(json.contains(r#""kind":"export""#));
        assert!(json.contains(r#""requires_network":false"#));
        assert!(json.contains(r#""requires_browser":false"#));
    }

    #[test]
    fn capabilities_json_exposes_zola_command_template() {
        let json = crate::plugin::capabilities_json();

        assert!(json.contains(r#""command":["export","{article}","--target","zola"]"#));
        assert!(json.contains(r#""article_arg":"{article}""#));
    }

    #[test]
    fn capabilities_json_exposes_zola_requirements() {
        let json = crate::plugin::capabilities_json();

        assert!(json.contains(r#""required_env":[]"#));
        assert!(json.contains(r#""required_config":["blog.root"]"#));
    }

    #[test]
    fn capabilities_text_is_human_readable() {
        let text = crate::plugin::capabilities_text();

        assert!(text.contains("wechat-draft"));
        assert!(text.contains("network: yes"));
        assert!(text.contains("browser: yes"));
        assert!(text.contains("manual final confirmation"));
    }
}
