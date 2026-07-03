use std::path::Path;

use crate::article::resolve_article_path;
use crate::config::Config;
use crate::error::AppError;
use crate::protocol::push_json;
use crate::push::{push_article, push_article_output};

pub(crate) type PublishAutomationFn = fn(bool, bool) -> Result<String, String>;

pub(crate) struct PushCommand<'a> {
    pub(crate) article: &'a Path,
    pub(crate) auto_render: bool,
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
        let output = push_article_output(articles_dir, command.article, command.auto_render, cfg)?;
        let next = "check in WeChat backend, then publish manually";
        Ok(push_json(
            &article_path,
            &output.media_id,
            output.stage,
            next,
        ))
    } else {
        push_article(articles_dir, command.article, command.auto_render, cfg)
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
