use std::path::PathBuf;
use std::str::FromStr;

use crate::cli::flag_value;
use crate::error::AppError;

use super::{RadarCommand, TrendSample};

pub(crate) fn parse_radar_command(args: &[String]) -> Result<RadarCommand, AppError> {
    let Some(command) = args.first() else {
        return Err(AppError::MissingValue(
            "radar <add|list|import|analyze|suggest|scrape>",
        ));
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

/// Splits radar arguments into positional paths and named flag values.
///
/// Every value-bearing switch a subcommand accepts must be listed in
/// `value_flags`. Anything else is rejected instead of being silently
/// dropped: the old hand-rolled loops had a `_ => {}` arm in `suggest`, took
/// the path from `args.first()`, and used a bare `args.next()` as the value of
/// every switch. `radar list --platform --keyword` therefore returned rc=0
/// with `platform = "--keyword"` and an empty result, while
/// `radar suggest a.md b.md --platform wechat` quietly discarded `a.md`.
///
/// Values go through the same [`flag_value`] guard as the main CLI, so a
/// missing value is reported against the flag that lacks it instead of the
/// leftover token becoming `unknown command: 5`.
#[derive(Debug, Default)]
struct RadarTokens {
    positionals: Vec<String>,
    values: Vec<(&'static str, String)>,
}

fn split_radar_tokens(
    args: &[String],
    value_flags: &[&'static str],
) -> Result<RadarTokens, AppError> {
    let mut tokens = RadarTokens::default();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let mut matched = None;
        for &flag in value_flags {
            if flag == arg.as_str() {
                matched = Some(flag);
                break;
            }
        }
        if let Some(name) = matched {
            if tokens.values.iter().any(|(seen, _)| *seen == name) {
                return Err(AppError::UnknownOption(format!("{name} (repeated)")));
            }
            let value = flag_value(&mut args, name)?;
            tokens.values.push((name, value));
        } else if arg.starts_with('-') {
            return Err(AppError::UnknownOption(arg.clone()));
        } else {
            tokens.positionals.push(arg.clone());
        }
    }
    Ok(tokens)
}

impl RadarTokens {
    fn value(&self, name: &'static str) -> Option<&str> {
        self.values
            .iter()
            .find(|(seen, _)| *seen == name)
            .map(|(_, value)| value.as_str())
    }

    fn require(&self, name: &'static str) -> Result<String, AppError> {
        self.value(name)
            .map(str::to_owned)
            .ok_or(AppError::MissingValue(name))
    }

    fn parsed<T: FromStr>(&self, name: &'static str) -> Result<Option<T>, AppError> {
        match self.value(name) {
            Some(raw) => Ok(Some(raw.parse().map_err(|_| AppError::InvalidNumber {
                flag: name,
                value: raw.to_owned(),
            })?)),
            None => Ok(None),
        }
    }

    fn no_positionals(&self) -> Result<(), AppError> {
        if let Some(extra) = self.positionals.first() {
            return Err(AppError::UnknownCommand(extra.clone()));
        }
        Ok(())
    }

    /// Positionals and switches are order-independent now; a second path is
    /// an error rather than a silent overwrite of the first one.
    fn single_positional(&self, usage: &'static str) -> Result<PathBuf, AppError> {
        match self.positionals.as_slice() {
            [] => Err(AppError::MissingValue(usage)),
            [path] => Ok(PathBuf::from(path)),
            [_, extra, ..] => Err(AppError::UnknownCommand(extra.clone())),
        }
    }
}

const ADD_VALUE_FLAGS: &[&str] = &[
    "--platform",
    "--keyword",
    "--title",
    "--url",
    "--author",
    "--likes",
    "--collects",
    "--comments",
    "--source",
];

fn parse_radar_add(args: &[String]) -> Result<RadarCommand, AppError> {
    let tokens = split_radar_tokens(args, ADD_VALUE_FLAGS)?;
    tokens.no_positionals()?;
    Ok(RadarCommand::Add(TrendSample {
        platform: tokens.require("--platform")?,
        keyword: tokens.require("--keyword")?,
        title: tokens.require("--title")?,
        url: tokens.value("--url").map(str::to_owned),
        author: tokens.value("--author").map(str::to_owned),
        likes: tokens.parsed("--likes")?,
        collects: tokens.parsed("--collects")?,
        comments: tokens.parsed("--comments")?,
        source: tokens.value("--source").unwrap_or("manual").to_owned(),
    }))
}

const LIST_VALUE_FLAGS: &[&str] = &["--platform", "--keyword"];

fn parse_radar_list(args: &[String]) -> Result<RadarCommand, AppError> {
    let tokens = split_radar_tokens(args, LIST_VALUE_FLAGS)?;
    tokens.no_positionals()?;
    Ok(RadarCommand::List {
        platform: tokens.value("--platform").map(str::to_owned),
        keyword: tokens.value("--keyword").map(str::to_owned),
    })
}

const IMPORT_VALUE_FLAGS: &[&str] = &["--platform"];

fn parse_radar_import(args: &[String]) -> Result<RadarCommand, AppError> {
    let tokens = split_radar_tokens(args, IMPORT_VALUE_FLAGS)?;
    Ok(RadarCommand::Import {
        path: tokens.single_positional("radar import <file.csv>")?,
        platform: tokens.value("--platform").map(str::to_owned),
    })
}

const ARTICLE_VALUE_FLAGS: &[&str] = &["--platform", "--top"];

fn parse_radar_analyze(args: &[String]) -> Result<RadarCommand, AppError> {
    let tokens = split_radar_tokens(args, ARTICLE_VALUE_FLAGS)?;
    Ok(RadarCommand::Analyze {
        article: tokens.single_positional("radar analyze <article.md>")?,
        platform: tokens.require("--platform")?,
        top: tokens.parsed("--top")?.unwrap_or(10),
    })
}

fn parse_radar_suggest(args: &[String]) -> Result<RadarCommand, AppError> {
    let tokens = split_radar_tokens(args, ARTICLE_VALUE_FLAGS)?;
    Ok(RadarCommand::Suggest {
        article: tokens.single_positional("radar suggest <article.md>")?,
        platform: tokens.require("--platform")?,
        top: tokens.parsed("--top")?.unwrap_or(10),
    })
}

const SCRAPE_VALUE_FLAGS: &[&str] = &["--platform", "--keyword", "--count", "--url"];

fn parse_radar_scrape(args: &[String]) -> Result<RadarCommand, AppError> {
    let tokens = split_radar_tokens(args, SCRAPE_VALUE_FLAGS)?;
    tokens.no_positionals()?;
    Ok(RadarCommand::Scrape {
        platform: tokens.require("--platform")?,
        keyword: tokens.require("--keyword")?,
        count: tokens.parsed("--count")?.unwrap_or(10),
        url: tokens.value("--url").map(str::to_owned),
    })
}

#[cfg(test)]
mod tests {
    use super::parse_radar_command;
    use crate::error::AppError;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| item.to_string()).collect()
    }

    /// The hint used to stop at `analyze`, hiding `suggest` and `scrape`.
    #[test]
    fn missing_subcommand_hint_lists_every_variant() {
        let err = parse_radar_command(&[]).expect_err("no subcommand is an error");
        let AppError::MissingValue(hint) = err else {
            panic!("expected MissingValue, got {err:?}");
        };
        for sub in ["add", "list", "import", "analyze", "suggest", "scrape"] {
            assert!(hint.contains(sub), "hint omits {sub}: {hint}");
        }
    }

    /// Keep the `radar ` prefix so the hint reads like the other subcommand
    /// errors (`radar analyze <article.md>`).
    #[test]
    fn suggest_missing_article_keeps_the_radar_prefix() {
        let err = parse_radar_command(&args(&["suggest"])).expect_err("article is required");
        let AppError::MissingValue(hint) = err else {
            panic!("expected MissingValue, got {err:?}");
        };
        assert_eq!(hint, "radar suggest <article.md>");
    }

    #[test]
    fn scrape_parses_platform_keyword_and_count() {
        let command = parse_radar_command(&args(&[
            "scrape",
            "--platform",
            "wechat",
            "--keyword",
            "读书",
            "--count",
            "5",
        ]))
        .expect("valid scrape args");
        match command {
            super::RadarCommand::Scrape {
                platform,
                keyword,
                count,
                url,
            } => {
                assert_eq!(platform, "wechat");
                assert_eq!(keyword, "读书");
                assert_eq!(count, 5);
                assert_eq!(url, None);
            }
            other => panic!("expected Scrape, got {other:?}"),
        }
    }

    /// A switch where a value belongs used to be consumed as the value, so
    /// `radar list --platform --keyword` returned rc=0 with
    /// `platform = "--keyword"` and an empty result. Every subcommand now
    /// names the flag that lacks its value.
    #[test]
    fn a_switch_cannot_be_the_value_of_another_switch() {
        let cases: &[(&[&str], &str)] = &[
            (&["list", "--platform", "--keyword"], "--platform"),
            (&["list", "--keyword", "--platform"], "--keyword"),
            (
                &["add", "--platform", "--keyword", "读书", "--title", "T"],
                "--platform",
            ),
            (
                &["scrape", "--platform", "wechat", "--keyword", "--count"],
                "--keyword",
            ),
            (&["analyze", "a.md", "--platform", "--top"], "--platform"),
            (&["suggest", "a.md", "--platform", "--top"], "--platform"),
        ];
        for (argv, expected) in cases {
            let err = parse_radar_command(&args(argv)).expect_err("a switch is not a value");
            let AppError::MissingValue(flag) = err else {
                panic!("argv={argv:?}: expected MissingValue, got {err:?}");
            };
            assert_eq!(flag, *expected, "argv={argv:?}");
        }
    }

    #[test]
    fn a_trailing_switch_reports_its_own_missing_value() {
        let cases: &[(&[&str], &str)] = &[
            (&["list", "--platform", "wechat", "--keyword"], "--keyword"),
            (
                &[
                    "scrape",
                    "--platform",
                    "wechat",
                    "--keyword",
                    "读书",
                    "--count",
                ],
                "--count",
            ),
        ];
        for (argv, expected) in cases {
            let err = parse_radar_command(&args(argv)).expect_err("trailing flag has no value");
            let AppError::MissingValue(flag) = err else {
                panic!("argv={argv:?}: expected MissingValue, got {err:?}");
            };
            assert_eq!(flag, *expected, "argv={argv:?}");
        }
    }

    /// `suggest` had a `_ => {}` arm: extra paths overwrote the article and
    /// unknown switches vanished. Both must be reported now.
    #[test]
    fn suggest_rejects_extra_paths_and_unknown_switches() {
        let err = parse_radar_command(&args(&[
            "suggest",
            "FIRST.md",
            "SECOND.md",
            "--platform",
            "wechat",
        ]))
        .expect_err("a second path must not silently replace the first");
        assert!(
            matches!(&err, AppError::UnknownCommand(value) if value == "SECOND.md"),
            "expected UnknownCommand(SECOND.md), got {err:?}"
        );

        let err = parse_radar_command(&args(&[
            "suggest",
            "a.md",
            "--platform",
            "wechat",
            "--typo-flag",
        ]))
        .expect_err("unknown switch must not be swallowed");
        assert!(
            matches!(&err, AppError::UnknownOption(value) if value == "--typo-flag"),
            "expected UnknownOption(--typo-flag), got {err:?}"
        );
    }

    #[test]
    fn no_positional_subcommands_reject_stray_paths() {
        for argv in [
            &["list", "stray.md"][..],
            &[
                "add",
                "stray.md",
                "--platform",
                "p",
                "--keyword",
                "k",
                "--title",
                "t",
            ][..],
            &["scrape", "stray.md", "--platform", "p", "--keyword", "k"][..],
        ] {
            let err = parse_radar_command(&args(argv)).expect_err("stray path is an error");
            assert!(
                matches!(&err, AppError::UnknownCommand(value) if value == "stray.md"),
                "argv={argv:?}: expected UnknownCommand(stray.md), got {err:?}"
            );
        }
    }

    /// `import` and `analyze` used `args.first()` as the path, so putting a
    /// switch first turned the switch name into the path.
    #[test]
    fn path_commands_are_order_independent() {
        let import = parse_radar_command(&args(&["import", "--platform", "wechat", "x.csv"]))
            .expect("switch before path is valid");
        let super::RadarCommand::Import { path, platform } = import else {
            panic!("expected Import, got {import:?}");
        };
        assert_eq!(path.to_string_lossy(), "x.csv");
        assert_eq!(platform.as_deref(), Some("wechat"));

        let analyze = parse_radar_command(&args(&[
            "analyze",
            "--platform",
            "wechat",
            "--top",
            "3",
            "a.md",
        ]))
        .expect("switches before path are valid");
        let super::RadarCommand::Analyze {
            article,
            platform,
            top,
        } = analyze
        else {
            panic!("expected Analyze, got {analyze:?}");
        };
        assert_eq!(article.to_string_lossy(), "a.md");
        assert_eq!(platform, "wechat");
        assert_eq!(top, 3);
    }

    #[test]
    fn repeating_a_value_flag_is_reported() {
        let err = parse_radar_command(&args(&[
            "list",
            "--platform",
            "wechat",
            "--platform",
            "xiaohongshu",
        ]))
        .expect_err("repeated switch must not silently discard a value");
        assert!(
            matches!(&err, AppError::UnknownOption(value) if value.contains("--platform")),
            "expected an error naming --platform, got {err:?}"
        );
    }

    #[test]
    fn invalid_numbers_still_report_the_flag_and_value() {
        let err = parse_radar_command(&args(&[
            "scrape",
            "--platform",
            "wechat",
            "--keyword",
            "读书",
            "--count",
            "many",
        ]))
        .expect_err("non-numeric count is an error");
        let AppError::InvalidNumber { flag, value } = err else {
            panic!("expected InvalidNumber, got {err:?}");
        };
        assert_eq!(flag, "--count");
        assert_eq!(value, "many");
    }

    #[test]
    fn documented_spellings_still_parse() {
        let add = parse_radar_command(&args(&[
            "add",
            "--platform",
            "xiaohongshu",
            "--keyword",
            "AI写作",
            "--title",
            "我的标题",
            "--likes",
            "42",
        ]))
        .expect("documented add spelling");
        let super::RadarCommand::Add(sample) = add else {
            panic!("expected Add, got {add:?}");
        };
        assert_eq!(sample.platform, "xiaohongshu");
        assert_eq!(sample.keyword, "AI写作");
        assert_eq!(sample.likes, Some(42));
        assert_eq!(sample.source, "manual");

        let list = parse_radar_command(&args(&["list"])).expect("bare list is valid");
        assert!(matches!(
            list,
            super::RadarCommand::List {
                platform: None,
                keyword: None
            }
        ));

        let suggest = parse_radar_command(&args(&["suggest", "a.md", "--platform", "wechat"]))
            .expect("documented suggest spelling");
        let super::RadarCommand::Suggest { article, top, .. } = suggest else {
            panic!("expected Suggest, got {suggest:?}");
        };
        assert_eq!(article.to_string_lossy(), "a.md");
        assert_eq!(top, 10);
    }
}
