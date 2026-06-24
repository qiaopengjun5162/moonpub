use std::path::Path;

use crate::config::Config;
use crate::error::AppError;

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
}
