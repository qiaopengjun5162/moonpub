use std::path::Path;

use crate::cli::{FeishuIntakeSource, Options};
use crate::config::Config;
use crate::error::AppError;
use crate::intake::{
    IntakeOutput, intake_feishu, intake_feishu_latest, intake_feishu_minute_token,
    intake_feishu_query,
};

pub(crate) fn load_config(options: &Options) -> Result<Config, AppError> {
    options
        .config
        .as_deref()
        .map(Config::load)
        .transpose()
        .map(|cfg| cfg.unwrap_or_default())
}

pub(crate) fn run_feishu_intake_source(
    articles_dir: &Path,
    source: &FeishuIntakeSource,
) -> Result<IntakeOutput, AppError> {
    match source {
        FeishuIntakeSource::File(input) => intake_feishu(articles_dir, input),
        FeishuIntakeSource::MinuteToken(token) => intake_feishu_minute_token(articles_dir, token),
        FeishuIntakeSource::Latest => intake_feishu_latest(articles_dir),
        FeishuIntakeSource::Query(query) => intake_feishu_query(articles_dir, query),
    }
}
