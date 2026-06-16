use std::path::{Path, PathBuf};

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Config {
    pub articles_root: Option<PathBuf>,
    pub wechat_appid: Option<String>,
    pub wechat_author: Option<String>,
    pub wechat_thumb_media_id: Option<String>,
    pub wechat_account_type: Option<String>,
    pub wechat_auto_publish: bool,
    pub wechat_theme: Option<String>,
    pub wechat_collection: Option<String>,
    pub blog_kind: Option<String>,
    pub blog_root: Option<PathBuf>,
    pub author_bio: Option<String>,
    pub qrcode_path: Option<String>,
}

impl Config {
    /// Minimal TOML parser that extracts string values from our known keys.
    pub fn from_toml(content: &str) -> Self {
        let mut cfg = Self::default();
        let mut section = "";

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if line.starts_with('[') {
                section = line.trim_matches(|c: char| c == '[' || c == ']');
                continue;
            }
            if let Some((key, value)) = split_toml_pair(line) {
                match key {
                    "root" => match section {
                        "articles" => cfg.articles_root = Some(PathBuf::from(value)),
                        "blog" => cfg.blog_root = Some(PathBuf::from(value)),
                        _ => {}
                    },
                    "appid" => cfg.wechat_appid = Some(value.to_owned()),
                    "author" => cfg.wechat_author = Some(value.to_owned()),
                    "account_type" => cfg.wechat_account_type = Some(value.to_owned()),
                    "auto_publish" => cfg.wechat_auto_publish = value == "true",
                    "theme" => cfg.wechat_theme = Some(value.to_owned()),
                    "collection" => cfg.wechat_collection = Some(value.to_owned()),
                    "thumb_media_id" => cfg.wechat_thumb_media_id = Some(value.to_owned()),
                    "kind" => cfg.blog_kind = Some(value.to_owned()),
                    "author_bio" => cfg.author_bio = Some(value.to_owned()),
                    "qrcode" => cfg.qrcode_path = Some(value.to_owned()),
                    _ => {}
                }
            }
        }

        cfg
    }

    pub fn load(path: &Path) -> Result<Self, AppError> {
        let content = std::fs::read_to_string(path).map_err(|source| AppError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(Self::from_toml(&content))
    }
}

pub fn split_toml_pair(line: &str) -> Option<(&str, &str)> {
    let (key, rest) = line.split_once('=')?;
    let key = key.trim();
    let rest = rest.trim();
    let value = rest
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(rest);
    Some((key, value))
}

pub fn sample_config() -> &'static str {
    r#"[articles]
root = "/path/to/ObsidianMain"

[wechat]
appid = ""
author = ""
account_type = "personal"
auto_publish = false
theme = "default"
collection = "书"
thumb_media_id = ""
author_bio = "每周分享读书笔记与思考。"
qrcode = "Context/assets/qrcode-group.jpg"

[blog]
kind = "zola"
root = "/path/to/blog"
"#
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::cli::Options;
    use crate::config::{Config, split_toml_pair};
    use crate::test_helpers::{create_file, temp_root};

    #[test]
    fn config_root_uses_section_not_order() {
        let toml = r#"
[blog]
root = "/my/blog"

[articles]
root = "/my/vault"
"#;
        let cfg = Config::from_toml(toml);
        assert_eq!(cfg.articles_root, Some(PathBuf::from("/my/vault")));
        assert_eq!(cfg.blog_root, Some(PathBuf::from("/my/blog")));
    }

    #[test]
    fn config_parses_articles_root() {
        let toml = r#"
[articles]
root = "/my/vault"

[wechat]
appid = "wx123"
"#;
        let cfg = Config::from_toml(toml);
        assert_eq!(cfg.articles_root, Some(PathBuf::from("/my/vault")));
        assert_eq!(cfg.wechat_appid.as_deref(), Some("wx123"));
    }

    #[test]
    fn config_overrides_articles_in_options() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("config-articles")?;
        let config_path = root.join("moonpub.toml");
        let articles_path = root.join("my-articles");
        std::fs::create_dir_all(&articles_path)?;
        create_file(
            &config_path,
            &format!("[articles]\nroot = \"{}\"\n", articles_path.display()),
        )?;

        let options = Options::parse([
            "--config".to_owned(),
            config_path.to_str().unwrap().to_owned(),
            "status".to_owned(),
        ])?;

        assert_eq!(options.articles, articles_path);

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn split_toml_pair_quoted_value() {
        let (k, v) = split_toml_pair(r#"root = "/my/vault""#).unwrap();
        assert_eq!(k, "root");
        assert_eq!(v, "/my/vault");
    }

    #[test]
    fn split_toml_pair_unquoted_value() {
        let (k, v) = split_toml_pair("auto_publish = true").unwrap();
        assert_eq!(k, "auto_publish");
        assert_eq!(v, "true");
    }

    #[test]
    fn split_toml_pair_value_with_equals() {
        let (k, v) = split_toml_pair(r#"appid = "wx=abc""#).unwrap();
        assert_eq!(k, "appid");
        assert_eq!(v, "wx=abc");
    }

    #[test]
    fn split_toml_pair_no_equals_returns_none() {
        assert!(split_toml_pair("just a header line").is_none());
    }
}
