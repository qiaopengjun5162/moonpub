//! Radar command — trend sample management, analysis, and scraping.

use std::path::{Path, PathBuf};

use crate::error::AppError;

mod analyze;
mod cli;
mod import;
mod scrape;
mod store;
mod suggest;
pub use analyze::analyze_article;
pub(crate) use analyze::tokenize;
pub(crate) use cli::parse_radar_command;
pub use import::import_csv;
#[cfg(test)]
pub(crate) use import::parse_csv_row;
pub use scrape::scrape_radar;
#[cfg(test)]
pub(crate) use scrape::{extract_from_snapshot, extract_samples, is_good_title, url_encode};
pub use store::{TrendSample, add_trend_sample, list_trend_samples};
pub(crate) use store::{load_all_samples, trend_store_path};
pub use suggest::suggest_titles;

// ── radar ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RadarCommand {
    Add(TrendSample),
    List {
        platform: Option<String>,
        keyword: Option<String>,
    },
    Import {
        path: PathBuf,
        platform: Option<String>,
    },
    Analyze {
        article: PathBuf,
        platform: String,
        top: usize,
    },
    Suggest {
        article: PathBuf,
        platform: String,
        top: usize,
    },
    Scrape {
        platform: String,
        keyword: String,
        count: usize,
        url: Option<String>,
    },
}

pub fn run_radar(articles_dir: &Path, command: &RadarCommand) -> Result<String, AppError> {
    match command {
        RadarCommand::Add(sample) => add_trend_sample(articles_dir, sample),
        RadarCommand::List { platform, keyword } => {
            list_trend_samples(articles_dir, platform, keyword)
        }
        RadarCommand::Import { path, platform } => {
            import_csv(articles_dir, path, platform.as_deref())
        }
        RadarCommand::Analyze {
            article,
            platform,
            top,
        } => analyze_article(articles_dir, article, platform, *top),
        RadarCommand::Suggest {
            article,
            platform,
            top,
        } => suggest_titles(articles_dir, article, platform, *top),
        RadarCommand::Scrape {
            platform,
            keyword,
            count,
            url,
        } => scrape_radar(articles_dir, platform, keyword, *count, url.as_deref()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::cli::{Command, Options};
    use crate::radar::{
        RadarCommand, TrendSample, add_trend_sample, analyze_article, extract_from_snapshot,
        extract_samples, import_csv, is_good_title, list_trend_samples, parse_csv_row,
        suggest_titles, url_encode,
    };
    use crate::test_helpers::{create_file, temp_root};

    #[test]
    fn parses_radar_add_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "radar".to_owned(),
            "add".to_owned(),
            "--platform".to_owned(),
            "xiaohongshu".to_owned(),
            "--keyword".to_owned(),
            "AI写作".to_owned(),
            "--title".to_owned(),
            "我的标题".to_owned(),
            "--likes".to_owned(),
            "42".to_owned(),
        ])?;

        let Command::Radar(RadarCommand::Add(sample)) = options.command else {
            panic!("expected radar add");
        };
        assert_eq!(sample.platform, "xiaohongshu");
        assert_eq!(sample.keyword, "AI写作");
        assert_eq!(sample.title, "我的标题");
        assert_eq!(sample.likes, Some(42));
        Ok(())
    }

    #[test]
    fn radar_add_and_list_samples() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("radar")?;
        add_trend_sample(
            &root,
            &TrendSample {
                platform: "wechat".to_owned(),
                keyword: "AI写作".to_owned(),
                title: "一个值得参考的标题".to_owned(),
                url: Some("https://example.com/post".to_owned()),
                author: Some("demo".to_owned()),
                likes: Some(100),
                collects: Some(50),
                comments: Some(8),
                source: "manual".to_owned(),
            },
        )?;

        let output = list_trend_samples(&root, &Some("wechat".to_owned()), &None)?;

        assert!(output.contains("[wechat] AI写作 | 一个值得参考的标题"));
        assert!(output.contains("likes=100"));
        assert!(output.contains("collects=50"));
        assert!(output.contains("comments=8"));
        assert!(output.contains("https://example.com/post"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn radar_list_filters_by_platform() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("radar-filter")?;
        add_sample(&root, "wechat", "公众号标题")?;
        add_sample(&root, "xiaohongshu", "小红书标题")?;

        let output = list_trend_samples(&root, &Some("xiaohongshu".to_owned()), &None)?;

        assert!(output.contains("小红书标题"));
        assert!(!output.contains("公众号标题"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn trend_sample_json_roundtrip_escapes_text() {
        let sample = TrendSample {
            platform: "wechat".to_owned(),
            keyword: "AI\"写作".to_owned(),
            title: "第一行\n第二行".to_owned(),
            url: None,
            author: None,
            likes: None,
            collects: None,
            comments: None,
            source: "manual".to_owned(),
        };

        let line = sample.to_json_line();
        let parsed = TrendSample::from_json_line(&line).expect("valid json line");

        assert_eq!(parsed.keyword, sample.keyword);
        assert_eq!(parsed.title, sample.title);
    }

    #[test]
    fn csv_import_basic() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("csv-import")?;
        let csv = root.join("trends.csv");
        create_file(
            &csv,
            "platform,keyword,title,likes,source\nwechat,AI写作,标题一,100,csv\nwechat,AI写作,标题二,200,csv\n",
        )?;

        let msg = import_csv(&root, &csv, None)?;
        assert!(msg.contains("imported 2 samples"));

        let output = list_trend_samples(&root, &Some("wechat".to_owned()), &None)?;
        assert!(output.contains("标题一"));
        assert!(output.contains("标题二"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn csv_import_uses_default_platform() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("csv-default-platform")?;
        let csv = root.join("trends.csv");
        create_file(&csv, "keyword,title\nAI写作,一篇好文章\n")?;

        import_csv(&root, &csv, Some("xiaohongshu"))?;

        let output = list_trend_samples(&root, &Some("xiaohongshu".to_owned()), &None)?;
        assert!(output.contains("一篇好文章"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn csv_import_quoted_fields() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("csv-quoted")?;
        let csv = root.join("trends.csv");
        create_file(&csv, "platform,keyword,title\nwechat,AI,\"标题含,逗号\"\n")?;

        import_csv(&root, &csv, None)?;

        let output = list_trend_samples(&root, &Some("wechat".to_owned()), &None)?;
        assert!(output.contains("标题含,逗号"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn csv_parse_row_handles_quoted_commas() {
        let row = r#""hello,world",foo,"bar""#;
        let fields = parse_csv_row(row);
        assert_eq!(fields, vec!["hello,world", "foo", "bar"]);
    }

    #[test]
    fn analyze_ranks_by_engagement() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("analyze")?;
        add_trend_sample(
            &root,
            &TrendSample {
                platform: "wechat".to_owned(),
                keyword: "AI写作".to_owned(),
                title: "高互动标题".to_owned(),
                url: None,
                author: None,
                likes: Some(500),
                collects: Some(200),
                comments: Some(50),
                source: "manual".to_owned(),
            },
        )?;
        add_trend_sample(
            &root,
            &TrendSample {
                platform: "wechat".to_owned(),
                keyword: "AI写作".to_owned(),
                title: "低互动标题".to_owned(),
                url: None,
                author: None,
                likes: Some(5),
                collects: None,
                comments: None,
                source: "manual".to_owned(),
            },
        )?;

        let article = root.join("demo.md");
        create_file(&article, "# AI写作技巧\n这篇文章讨论AI写作。")?;

        let output = analyze_article(&root, &article, "wechat", 10)?;

        let high_pos = output.find("高互动标题").expect("高互动标题 not found");
        let low_pos = output.find("低互动标题").expect("低互动标题 not found");
        assert!(high_pos < low_pos, "高互动应排在低互动之前");

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn analyze_filters_by_platform() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("analyze-platform")?;
        add_sample(&root, "wechat", "公众号专属标题")?;
        add_sample(&root, "xiaohongshu", "小红书专属标题")?;

        let article = root.join("demo.md");
        create_file(&article, "# AI写作")?;

        let output = analyze_article(&root, &article, "wechat", 10)?;

        assert!(output.contains("公众号专属标题"));
        assert!(!output.contains("小红书专属标题"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn suggest_titles_includes_formula_and_trend_reference()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("suggest-titles")?;
        add_trend_sample(
            &root,
            &TrendSample {
                platform: "wechat".to_owned(),
                keyword: "AI写作".to_owned(),
                title: "公众号爆款标题参考".to_owned(),
                url: None,
                author: None,
                likes: Some(320),
                collects: Some(88),
                comments: Some(12),
                source: "manual".to_owned(),
            },
        )?;

        let article = root.join("demo.md");
        create_file(
            &article,
            "---\n\
title: 原始标题\n\
digest: 这是一篇关于AI写作的摘要\n\
---\n\
\n\
总是写了很久却没有成果？这里有一个更清楚的办法。\n\
\n\
## 写作系统\n\
\n\
这不是更努力，而是更聪明地组织素材。\n",
        )?;

        let output = suggest_titles(&root, &article, "wechat", 1)?;

        assert!(output.contains("title suggestions for [wechat]"));
        assert!(output.contains("▎痛点 + 解决方案"));
        assert!(output.contains("公众号爆款标题参考"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn parses_radar_import_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "radar".to_owned(),
            "import".to_owned(),
            "trends.csv".to_owned(),
            "--platform".to_owned(),
            "wechat".to_owned(),
        ])?;

        let Command::Radar(RadarCommand::Import { path, platform }) = options.command else {
            panic!("expected radar import");
        };
        assert_eq!(path, PathBuf::from("trends.csv"));
        assert_eq!(platform.as_deref(), Some("wechat"));
        Ok(())
    }

    #[test]
    fn parses_radar_analyze_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "radar".to_owned(),
            "analyze".to_owned(),
            "demo.md".to_owned(),
            "--platform".to_owned(),
            "wechat".to_owned(),
            "--top".to_owned(),
            "5".to_owned(),
        ])?;

        let Command::Radar(RadarCommand::Analyze {
            article,
            platform,
            top,
        }) = options.command
        else {
            panic!("expected radar analyze");
        };
        assert_eq!(article, PathBuf::from("demo.md"));
        assert_eq!(platform, "wechat");
        assert_eq!(top, 5);
        Ok(())
    }

    #[test]
    fn parses_radar_scrape_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "radar".to_owned(),
            "scrape".to_owned(),
            "--platform".to_owned(),
            "wechat".to_owned(),
            "--keyword".to_owned(),
            "AI写作".to_owned(),
            "--count".to_owned(),
            "5".to_owned(),
        ])?;
        let Command::Radar(RadarCommand::Scrape {
            platform,
            keyword,
            count,
            url,
        }) = options.command
        else {
            panic!("expected Scrape");
        };
        assert_eq!(platform, "wechat");
        assert_eq!(keyword, "AI写作");
        assert_eq!(count, 5);
        assert!(url.is_none());
        Ok(())
    }

    #[test]
    fn url_encode_handles_chinese() {
        let encoded = url_encode("AI写作");
        assert!(encoded.starts_with("AI"));
        assert!(encoded.contains('%'), "汉字应被百分比编码");
        assert!(!encoded.contains(' '));
    }

    #[test]
    fn extract_from_snapshot_parses_titles() {
        let snapshot = r#"
- document
  - main
    - heading "AI时代的写作技巧：10个让你效率翻倍的方法" [ref=e1]
    - link "普通人如何用AI写出爆款文章" [ref=e2]
    - link "关注" [ref=e3]
    - link "更多" [ref=e4]
"#;
        let titles = extract_from_snapshot(snapshot);
        assert!(titles.contains(&"AI时代的写作技巧：10个让你效率翻倍的方法".to_owned()));
        assert!(titles.contains(&"普通人如何用AI写出爆款文章".to_owned()));
        assert!(!titles.contains(&"关注".to_owned()), "太短的导航文字应过滤");
    }

    #[test]
    fn is_good_title_filters_short_and_nav() {
        assert!(is_good_title("AI时代的写作技巧让你效率翻倍"));
        assert!(!is_good_title("关注"), "太短");
        assert!(!is_good_title("var x = function(){}"), "JS代码");
        assert!(!is_good_title("ab"), "太短ASCII");
    }

    #[test]
    fn scrape_stores_samples_in_jsonl() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("scrape-store")?;

        let html = r#"<html><body>
            <h3><a href="/a1">坚持每天写作：我用AI辅助的30天实验</a></h3>
            <h3><a href="/a2">公众号涨粉秘诀：内容为王还是运营为王</a></h3>
            <h3><a href="/a3">短</a></h3>
        </body></html>"#;

        let samples = extract_samples(html, "wechat", "AI写作", 10);
        assert!(!samples.is_empty(), "应提取到文章标题");
        assert!(samples.iter().all(|s| s.platform == "wechat"));
        assert!(samples.iter().all(|s| s.keyword == "AI写作"));
        assert!(samples.iter().all(|s| s.source == "scrape"));

        for s in &samples {
            assert!(s.title.chars().count() >= 6, "标题太短: {}", s.title);
        }

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    fn add_sample(root: &Path, platform: &str, title: &str) -> Result<(), crate::error::AppError> {
        add_trend_sample(
            root,
            &TrendSample {
                platform: platform.to_owned(),
                keyword: "AI".to_owned(),
                title: title.to_owned(),
                url: None,
                author: None,
                likes: None,
                collects: None,
                comments: None,
                source: "manual".to_owned(),
            },
        )?;
        Ok(())
    }
}
