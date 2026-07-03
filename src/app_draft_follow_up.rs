use std::path::{Path, PathBuf};

use crate::cli::PreviewOptions;
use crate::config::Config;
use crate::error::AppError;
use crate::preview::{preview_article_with_open, preview_paths};
use crate::protocol::{PushJsonMeta, draft_from_inbox_json, intake_draft_preview_json};
use crate::push::push_article_output;
use crate::render::render_article;

pub(crate) enum DraftJsonKind<'a> {
    FromInbox {
        input_path: &'a Path,
    },
    Intake {
        command_name: &'a str,
        inbox_path: &'a Path,
    },
}

pub(crate) struct DraftFollowUp<'a> {
    pub(crate) preview: PreviewOptions,
    pub(crate) auto_push: bool,
    pub(crate) json: bool,
    pub(crate) leading_message: Option<&'a str>,
    pub(crate) json_kind: DraftJsonKind<'a>,
    pub(crate) draft_output: &'a crate::ai_workflow::DraftOutput,
}

pub(crate) fn ensure_preview_html(
    articles_dir: &Path,
    cfg: &Config,
    article: &Path,
    preview: PreviewOptions,
) -> Result<Option<PathBuf>, AppError> {
    if !preview.enabled {
        return Ok(None);
    }
    render_and_preview_draft(articles_dir, cfg, article, preview.open)?;
    let (_, html_path) = preview_paths(articles_dir, article)?;
    Ok(Some(html_path))
}

pub(crate) fn finalize_draft_follow_up(
    articles_dir: &Path,
    cfg: &Config,
    follow_up: DraftFollowUp<'_>,
) -> Result<String, AppError> {
    let push_output = if follow_up.auto_push {
        Some(push_article_output(
            articles_dir,
            &follow_up.draft_output.path,
            true,
            cfg,
        )?)
    } else {
        None
    };

    if follow_up.json {
        let html_path = ensure_preview_html(
            articles_dir,
            cfg,
            &follow_up.draft_output.path,
            follow_up.preview,
        )?;
        let next = push_output
            .as_ref()
            .map(|output| output.message.lines().last().unwrap_or_default())
            .and_then(|line| line.trim().strip_prefix("next: "))
            .unwrap_or("moonpub push <draft.md> --render");

        let output = match follow_up.json_kind {
            DraftJsonKind::FromInbox { input_path } => draft_from_inbox_json(
                input_path,
                &follow_up.draft_output.path,
                html_path.as_deref(),
                follow_up.draft_output.action.as_str(),
                next,
                push_output.as_ref().map(PushJsonMeta::from),
            ),
            DraftJsonKind::Intake {
                command_name,
                inbox_path,
            } => intake_draft_preview_json(
                command_name,
                inbox_path,
                &follow_up.draft_output.path,
                html_path.as_deref(),
                follow_up.draft_output.action.as_str(),
                next,
                push_output.as_ref().map(PushJsonMeta::from),
            ),
        };
        return Ok(output);
    }

    let mut message = match follow_up.leading_message {
        Some(leading) => format!("{leading}\n{}", follow_up.draft_output.message),
        None => follow_up.draft_output.message.clone(),
    };

    if let Some(push_output) = push_output {
        message.push('\n');
        message.push_str(&push_output.message);
    } else if follow_up.preview.enabled {
        message.push('\n');
        message.push_str(&render_and_preview_draft(
            articles_dir,
            cfg,
            &follow_up.draft_output.path,
            follow_up.preview.open,
        )?);
    }

    Ok(message)
}

fn render_and_preview_draft(
    articles_dir: &Path,
    cfg: &Config,
    article: &Path,
    open_browser: bool,
) -> Result<String, AppError> {
    let resolved_author = cfg.wechat_author.as_deref().unwrap_or("作者");
    let resolved_thumb = cfg.wechat_thumb_media_id.as_deref().unwrap_or("");
    let theme_name = cfg.wechat_theme.as_deref().unwrap_or("default");
    let mut footer_cfg = cfg.footer.clone();
    if footer_cfg.qrcode.is_empty() {
        footer_cfg.qrcode = cfg.qrcode_path.clone().unwrap_or_default();
    }
    let rendered = render_article(
        articles_dir,
        article,
        resolved_author,
        resolved_thumb,
        theme_name,
        None,
        &footer_cfg,
    )?;
    let previewed = preview_article_with_open(articles_dir, article, open_browser)?;
    Ok(format!("{rendered}\n{previewed}"))
}
