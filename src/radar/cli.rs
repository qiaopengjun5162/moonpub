use std::path::PathBuf;

use crate::error::AppError;

use super::{RadarCommand, TrendSample};

pub(crate) fn parse_radar_command(args: &[String]) -> Result<RadarCommand, AppError> {
    let Some(command) = args.first() else {
        return Err(AppError::MissingValue("radar <add|list|import|analyze>"));
    };

    match command.as_str() {
        "add" => parse_radar_add(&args[1..]),
        "list" => parse_radar_list(&args[1..]),
        "import" => parse_radar_import(&args[1..]),
        "analyze" => parse_radar_analyze(&args[1..]),
        "suggest" => parse_radar_suggest(&args[1..]),
        "scrape" => parse_radar_scrape(&args[1..]),
        value => Err(AppError::UnknownCommand(format!("radar {value}"))),
    }
}

fn parse_radar_add(args: &[String]) -> Result<RadarCommand, AppError> {
    let mut platform = None;
    let mut keyword = None;
    let mut title = None;
    let mut url = None;
    let mut author = None;
    let mut likes = None;
    let mut collects = None;
    let mut comments = None;
    let mut source = String::from("manual");

    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--platform" => platform = Some(next_arg(&mut args, "--platform")?),
            "--keyword" => keyword = Some(next_arg(&mut args, "--keyword")?),
            "--title" => title = Some(next_arg(&mut args, "--title")?),
            "--url" => url = Some(next_arg(&mut args, "--url")?),
            "--author" => author = Some(next_arg(&mut args, "--author")?),
            "--likes" => likes = Some(parse_u64("--likes", next_arg(&mut args, "--likes")?)?),
            "--collects" => {
                collects = Some(parse_u64("--collects", next_arg(&mut args, "--collects")?)?);
            }
            "--comments" => {
                comments = Some(parse_u64("--comments", next_arg(&mut args, "--comments")?)?);
            }
            "--source" => source = next_arg(&mut args, "--source")?,
            value if value.starts_with('-') => {
                return Err(AppError::UnknownOption(value.to_owned()));
            }
            value => return Err(AppError::UnknownCommand(value.to_owned())),
        }
    }

    Ok(RadarCommand::Add(TrendSample {
        platform: platform.ok_or(AppError::MissingValue("--platform"))?,
        keyword: keyword.ok_or(AppError::MissingValue("--keyword"))?,
        title: title.ok_or(AppError::MissingValue("--title"))?,
        url,
        author,
        likes,
        collects,
        comments,
        source,
    }))
}

fn parse_radar_list(args: &[String]) -> Result<RadarCommand, AppError> {
    let mut platform = None;
    let mut keyword = None;
    let mut args = args.iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--platform" => platform = Some(next_arg(&mut args, "--platform")?),
            "--keyword" => keyword = Some(next_arg(&mut args, "--keyword")?),
            value if value.starts_with('-') => {
                return Err(AppError::UnknownOption(value.to_owned()));
            }
            value => return Err(AppError::UnknownCommand(value.to_owned())),
        }
    }

    Ok(RadarCommand::List { platform, keyword })
}

fn parse_radar_import(args: &[String]) -> Result<RadarCommand, AppError> {
    let path = args
        .first()
        .map(PathBuf::from)
        .ok_or(AppError::MissingValue("radar import <file.csv>"))?;

    let mut platform = None;
    let mut args = args[1..].iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--platform" => platform = Some(next_arg(&mut args, "--platform")?),
            value if value.starts_with('-') => {
                return Err(AppError::UnknownOption(value.to_owned()));
            }
            value => return Err(AppError::UnknownCommand(value.to_owned())),
        }
    }

    Ok(RadarCommand::Import { path, platform })
}

fn parse_radar_suggest(args: &[String]) -> Result<RadarCommand, AppError> {
    let mut article = None;
    let mut platform = None;
    let mut top = 10usize;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if let Some(a) = parse_radar_article_arg(arg) {
            article = a;
        } else {
            match arg.as_str() {
                "--platform" => platform = Some(next_arg(&mut args, "--platform")?),
                "--top" => {
                    let v = next_arg(&mut args, "--top")?;
                    top = v.parse().map_err(|_| AppError::InvalidNumber {
                        flag: "--top",
                        value: v,
                    })?;
                }
                _ => {}
            }
        }
    }
    Ok(RadarCommand::Suggest {
        article: PathBuf::from(article.ok_or(AppError::MissingValue("suggest <article.md>"))?),
        platform: platform.ok_or(AppError::MissingValue("--platform"))?,
        top,
    })
}

fn parse_radar_article_arg(arg: &str) -> Option<Option<String>> {
    if !arg.starts_with('-') {
        Some(Some(arg.to_owned()))
    } else {
        None
    }
}

fn parse_radar_analyze(args: &[String]) -> Result<RadarCommand, AppError> {
    let article = args
        .first()
        .map(PathBuf::from)
        .ok_or(AppError::MissingValue("radar analyze <article.md>"))?;

    let mut platform = None;
    let mut top = 10usize;
    let mut args = args[1..].iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--platform" => platform = Some(next_arg(&mut args, "--platform")?),
            "--top" => {
                let v = next_arg(&mut args, "--top")?;
                top = v.parse().map_err(|_| AppError::InvalidNumber {
                    flag: "--top",
                    value: v,
                })?;
            }
            value if value.starts_with('-') => {
                return Err(AppError::UnknownOption(value.to_owned()));
            }
            value => return Err(AppError::UnknownCommand(value.to_owned())),
        }
    }

    Ok(RadarCommand::Analyze {
        article,
        platform: platform.ok_or(AppError::MissingValue("--platform"))?,
        top,
    })
}

fn parse_radar_scrape(args: &[String]) -> Result<RadarCommand, AppError> {
    let mut platform = None;
    let mut keyword = None;
    let mut count = 10usize;
    let mut url = None;
    let mut args = args.iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--platform" => platform = Some(next_arg(&mut args, "--platform")?),
            "--keyword" => keyword = Some(next_arg(&mut args, "--keyword")?),
            "--count" => {
                let v = next_arg(&mut args, "--count")?;
                count = v.parse().map_err(|_| AppError::InvalidNumber {
                    flag: "--count",
                    value: v,
                })?;
            }
            "--url" => url = Some(next_arg(&mut args, "--url")?),
            value if value.starts_with('-') => {
                return Err(AppError::UnknownOption(value.to_owned()));
            }
            value => return Err(AppError::UnknownCommand(value.to_owned())),
        }
    }

    Ok(RadarCommand::Scrape {
        platform: platform.ok_or(AppError::MissingValue("--platform"))?,
        keyword: keyword.ok_or(AppError::MissingValue("--keyword"))?,
        count,
        url,
    })
}

fn next_arg<'a>(
    args: &mut impl Iterator<Item = &'a String>,
    flag: &'static str,
) -> Result<String, AppError> {
    args.next().cloned().ok_or(AppError::MissingValue(flag))
}

fn parse_u64(flag: &'static str, value: String) -> Result<u64, AppError> {
    value
        .parse()
        .map_err(|_| AppError::InvalidNumber { flag, value })
}
