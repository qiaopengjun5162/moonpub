use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::footer::FooterConfig;

#[cfg(test)]
const THEME_HINT: &str = "default | warm | dark | geek | paper | magazine | notebook | classic | forest | sunset | ocean | mono | editorial | zen | newsletter | academic | cyber | letter | mist | gallery";

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
    pub footer: FooterConfig,
    pub template_name: Option<String>,
    pub ai_provider: Option<String>,
    pub ai_model: Option<String>,
    pub ai_api_key: Option<String>,
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
                let value = unescape_toml_string(value);
                match section {
                    "articles" if key == "root" => {
                        cfg.articles_root = Some(PathBuf::from(value));
                    }
                    "blog" if key == "root" => {
                        cfg.blog_root = Some(PathBuf::from(value));
                    }
                    "blog" if key == "kind" => {
                        cfg.blog_kind = Some(value);
                    }
                    "wechat" => match key {
                        "appid" => cfg.wechat_appid = Some(value),
                        "author" => cfg.wechat_author = Some(value),
                        "account_type" => cfg.wechat_account_type = Some(value),
                        "auto_publish" => cfg.wechat_auto_publish = value == "true",
                        "theme" => cfg.wechat_theme = Some(value),
                        "collection" => cfg.wechat_collection = Some(value),
                        "thumb_media_id" => cfg.wechat_thumb_media_id = Some(value),
                        "author_bio" => cfg.author_bio = Some(value),
                        "qrcode" => cfg.qrcode_path = Some(value),
                        _ => {}
                    },
                    "footer" => match key {
                        "enabled" => cfg.footer.enabled = value == "true",
                        "variant" => cfg.footer.variant = value,
                        "title" => cfg.footer.title = value,
                        "description" => cfg.footer.description = value,
                        "rules" => cfg.footer.rules = value,
                        "qrcode" => cfg.footer.qrcode = value,
                        "qrcode_note" => cfg.footer.qrcode_note = value,
                        "follow_image" => cfg.footer.follow_image = value,
                        "follow_text" => cfg.footer.follow_text = value,
                        "divider" => cfg.footer.divider = value,
                        _ => {}
                    },
                    "template" if key == "name" => {
                        cfg.template_name = Some(value);
                    }
                    "ai" => match key {
                        "provider" => cfg.ai_provider = Some(value),
                        "model" => cfg.ai_model = Some(value),
                        "api_key" => cfg.ai_api_key = Some(value),
                        _ => {}
                    },
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
theme = "default" # default | warm | dark | geek | paper | magazine | notebook | classic | forest | sunset | ocean | mono | editorial | zen | newsletter | academic | cyber | letter | mist | gallery
collection = "书"
thumb_media_id = ""
author_bio = "每周分享读书笔记与思考。"
qrcode = "Context/assets/qrcode.png"

[footer]
enabled = false
variant = "community" # community | minimal
title = "加入「我的社群」"
description = "欢迎每一位对技术保持热爱与好奇心的朋友。"
rules = "· 亮出身份，以诚会友\n· 专注技术，言之有物\n· 君子之交，和而不同\n· 广告勿扰，保持纯粹"
qrcode = "Context/assets/qrcode.png"
qrcode_note = "长按下方二维码即可入群。\n若二维码过期，请在公众号后台回复 加群 获取最新二维码。"
follow_image = ""
follow_text = "点个「赞」让我知道你喜欢，点个「推荐」让更多人看到。"
divider = "— · —"

[blog]
kind = "zola"
root = "/path/to/blog"

[template]
name = "寻月阁标准结尾"

[ai]
provider = "deepseek"
model = "deepseek-chat"
# api_key = "sk-..."   # 优先使用 DEEPSEEK_API_KEY / OPENAI_API_KEY 环境变量
"#
}

pub fn sample_config_for_articles_root(articles_root: &Path) -> String {
    let articles_root = escape_toml_string(&articles_root.display().to_string());
    format!(
        r#"[articles]
root = "{articles_root}"

[wechat]
appid = ""
author = ""
account_type = "personal"
auto_publish = false
theme = "default" # default | warm | dark | geek | paper | magazine | notebook | classic | forest | sunset | ocean | mono | editorial | zen | newsletter | academic | cyber | letter | mist | gallery
collection = "书"
thumb_media_id = ""
author_bio = "每周分享读书笔记与思考。"
qrcode = "Context/assets/qrcode.png"

[footer]
enabled = false
variant = "community" # community | minimal
title = "加入「我的社群」"
description = "欢迎每一位对技术保持热爱与好奇心的朋友。"
rules = "· 亮出身份，以诚会友\n· 专注技术，言之有物\n· 君子之交，和而不同\n· 广告勿扰，保持纯粹"
qrcode = "Context/assets/qrcode.png"
qrcode_note = "长按下方二维码即可入群。\n若二维码过期，请在公众号后台回复 加群 获取最新二维码。"
follow_image = ""
follow_text = "点个「赞」让我知道你喜欢，点个「推荐」让更多人看到。"
divider = "— · —"

[template]
name = ""

[ai]
provider = "deepseek"
model = "deepseek-chat"
# api_key = "sk-..."   # 优先使用 DEEPSEEK_API_KEY / OPENAI_API_KEY 环境变量
"#
    )
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn unescape_toml_string(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::cli::Options;
    use crate::config::{Config, THEME_HINT, split_toml_pair};
    use crate::footer::FooterConfig;
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

    #[test]
    fn sample_config_for_articles_root_uses_given_path() {
        let root = PathBuf::from("/tmp/moonpub articles");
        let toml = crate::config::sample_config_for_articles_root(&root);
        let cfg = Config::from_toml(&toml);

        assert_eq!(cfg.articles_root, Some(root));
        assert_eq!(cfg.blog_root, None);
    }

    #[test]
    fn sample_configs_document_all_article_themes() {
        let generated = crate::config::sample_config();
        let generated_for_root =
            crate::config::sample_config_for_articles_root(&PathBuf::from("/tmp/moonpub"));

        assert!(generated.contains(THEME_HINT));
        assert!(generated_for_root.contains(THEME_HINT));
    }

    #[test]
    fn sample_config_for_articles_root_escapes_quotes() {
        let root = PathBuf::from("/tmp/moonpub \"drafts\"");
        let toml = crate::config::sample_config_for_articles_root(&root);
        let cfg = Config::from_toml(&toml);

        assert!(toml.contains(r#"root = "/tmp/moonpub \"drafts\"""#));
        assert_eq!(cfg.articles_root, Some(root));
    }

    #[test]
    fn config_unescapes_basic_toml_strings() {
        let toml = r#"
[articles]
root = "C:\\Users\\moonpub \"drafts\""
"#;
        let cfg = Config::from_toml(toml);

        assert_eq!(
            cfg.articles_root,
            Some(PathBuf::from(r#"C:\Users\moonpub "drafts""#))
        );
    }

    #[test]
    fn footer_parsing_enabled() {
        let toml = r#"
[footer]
enabled = true
variant = "minimal"
title = "加群"
description = "欢迎"
"#;
        let cfg = Config::from_toml(toml);
        assert!(cfg.footer.enabled);
        assert_eq!(cfg.footer.variant, "minimal");
        assert_eq!(cfg.footer.title, "加群");
        assert_eq!(cfg.footer.description, "欢迎");
        assert!(cfg.footer.qrcode.is_empty());
    }

    #[test]
    fn footer_default_disabled() {
        let toml = r#"
[articles]
root = "/tmp"
"#;
        let cfg = Config::from_toml(toml);
        assert!(!cfg.footer.enabled);
        assert_eq!(cfg.footer, FooterConfig::default());
    }

    #[test]
    fn parse_template_name() {
        let cfg = Config::from_toml(
            r#"
[template]
name = "寻月阁标准结尾"
"#,
        );
        assert_eq!(cfg.template_name, Some("寻月阁标准结尾".to_owned()));
    }

    #[test]
    fn parse_ai_config() {
        let cfg = Config::from_toml(
            r#"
[ai]
provider = "openai"
model = "gpt-4o-mini"
api_key = "sk-test"
"#,
        );
        assert_eq!(cfg.ai_provider, Some("openai".to_owned()));
        assert_eq!(cfg.ai_model, Some("gpt-4o-mini".to_owned()));
        assert_eq!(cfg.ai_api_key, Some("sk-test".to_owned()));
    }
}
