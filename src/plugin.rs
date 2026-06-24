use std::path::Path;

use crate::config::Config;
use crate::error::AppError;
use crate::json_util::escape_json;

pub struct TargetCapability {
    pub id: &'static str,
    pub display_name: &'static str,
    pub kind: &'static str,
    pub requires_network: bool,
    pub requires_browser: bool,
    pub risk: &'static str,
    pub next_step: &'static str,
}

pub struct PublishContext<'a> {
    pub articles_dir: &'a Path,
    pub article: &'a Path,
    pub auto_render: bool,
    pub config: &'a Config,
}

pub struct PublishOutcome {
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

pub fn builtin_capabilities() -> Vec<TargetCapability> {
    vec![TargetCapability {
        id: "wechat-draft",
        display_name: "WeChat Draft",
        kind: "publish",
        requires_network: true,
        requires_browser: true,
        risk: "calls WeChat API and may open Chrome automation",
        next_step: "manual final confirmation in WeChat backend",
    }]
}

pub fn capabilities_text() -> String {
    let mut output = String::from("capabilities\n");
    for capability in builtin_capabilities() {
        output.push_str(&format!(
            "  {} ({})\n    kind: {}\n    network: {}\n    browser: {}\n    risk: {}\n    next: {}\n",
            capability.id,
            capability.display_name,
            capability.kind,
            yes_no(capability.requires_network),
            yes_no(capability.requires_browser),
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
            format!(
                "{{\"id\":\"{}\",\"display_name\":\"{}\",\"kind\":\"{}\",\"requires_network\":{},\"requires_browser\":{},\"risk\":\"{}\",\"next_step\":\"{}\"}}",
                escape_json(capability.id),
                escape_json(capability.display_name),
                escape_json(capability.kind),
                capability.requires_network,
                capability.requires_browser,
                escape_json(capability.risk),
                escape_json(capability.next_step)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"targets\":[{items}]}}")
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::config::Config;
    use crate::error::AppError;
    use crate::plugin::{PublishContext, PublishOutcome, PublishTarget, run_publish_target};

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
                config: &cfg,
            },
        )?;

        assert_eq!(outcome.message, "/vault:Articles/drafts/demo.md:true");
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
    fn capabilities_text_is_human_readable() {
        let text = crate::plugin::capabilities_text();

        assert!(text.contains("wechat-draft"));
        assert!(text.contains("network: yes"));
        assert!(text.contains("browser: yes"));
        assert!(text.contains("manual final confirmation"));
    }
}
