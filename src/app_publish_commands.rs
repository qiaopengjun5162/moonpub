use std::path::Path;

use crate::article::resolve_article_path;
use crate::config::Config;
use crate::error::AppError;
use crate::protocol::push_json;
use crate::push::push_article_output;

pub(crate) type PublishAutomationFn = fn(bool, bool) -> Result<String, String>;

pub(crate) struct PushCommand<'a> {
    pub(crate) article: &'a Path,
    pub(crate) auto_render: bool,
    pub(crate) temporary_profile: bool,
    pub(crate) json: bool,
}

pub(crate) fn run_publish_automation(
    headed: bool,
    temporary_profile: bool,
    action: PublishAutomationFn,
) -> Result<String, AppError> {
    action(headed, temporary_profile).map_err(|message| AppError::PushFailed {
        message,
        ip_hint: None,
    })
}

pub(crate) fn run_wechat_draft_command(
    articles_dir: &Path,
    cfg: &Config,
    command: PushCommand<'_>,
) -> Result<String, AppError> {
    if command.json {
        let article_path = resolve_article_path(articles_dir, command.article);
        let output = push_article_output(
            articles_dir,
            command.article,
            command.auto_render,
            command.temporary_profile,
            cfg,
        )?;
        let next = "check in WeChat backend, then publish manually";
        Ok(push_json(
            &article_path,
            &output.media_id,
            output.stage,
            next,
        ))
    } else {
        let output = push_article_output(
            articles_dir,
            command.article,
            command.auto_render,
            command.temporary_profile,
            cfg,
        )?;
        Ok(output.message)
    }
}

pub(crate) fn run_publish_command(
    articles_dir: &Path,
    cfg: &Config,
    target: &str,
    command: PushCommand<'_>,
) -> Result<String, AppError> {
    match target {
        "wechat-draft" => run_wechat_draft_command(articles_dir, cfg, command),
        other => Err(AppError::UnknownCommand(format!("publish target {other}"))),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{PushCommand, run_publish_automation, run_publish_command};
    use crate::config::Config;
    use crate::error::AppError;

    fn failing_automation(_: bool, _: bool) -> Result<String, String> {
        Err("browser session expired".to_owned())
    }

    #[test]
    fn publish_automation_maps_string_error_to_push_failed() {
        let err = run_publish_automation(false, true, failing_automation).unwrap_err();

        match err {
            AppError::PushFailed { message, ip_hint } => {
                assert_eq!(message, "browser session expired");
                assert_eq!(ip_hint, None);
            }
            other => panic!("expected PushFailed, got {other:?}"),
        }
    }

    #[test]
    fn unknown_publish_target_fails_before_article_or_network_access() {
        let command = PushCommand {
            article: Path::new("missing.md"),
            auto_render: true,
            temporary_profile: false,
            json: true,
        };
        let err = run_publish_command(
            Path::new("/definitely/missing/articles"),
            &Config::default(),
            "ghost",
            command,
        )
        .unwrap_err();

        match err {
            AppError::UnknownCommand(command) => {
                assert_eq!(command, "publish target ghost");
            }
            other => panic!("expected UnknownCommand, got {other:?}"),
        }
    }
}
