use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::footer::FooterConfig;

#[cfg(test)]
const THEME_HINT: &str = "default | warm | dark | geek | geek-black | blueprint | ai-lab | paper | magazine | notebook | classic | forest | sunset | ocean | mono | editorial | zen | newsletter | academic | cyber | letter | mist | gallery | moonlit | porcelain | fieldnote | monokai | dracula | nord | one-dark | gruvbox";

/// cookie 通道实际覆盖的命令。
///
/// 单一事实来源：`MissingEnvVar` 的提示、`cookie_auth_unsupported` 的报错
/// 文案、以及文档里的范围说明都从这里取。`list-drafts` / `update-draft`
/// 只有 appsecret 实现（`push_browser` 里没有对应的后台接口封装），把它们
/// 写进这里就是承诺一条走不通的路。
pub const COOKIE_AUTH_COMMANDS: &[&str] = &["push", "ship", "delete-draft"];

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Config {
    pub articles_root: Option<PathBuf>,
    pub wechat_appid: Option<String>,
    pub wechat_author: Option<String>,
    pub wechat_thumb_media_id: Option<String>,
    pub wechat_account_type: Option<String>,
    pub wechat_auth_method: Option<String>,
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
                        "auth_method" => cfg.wechat_auth_method = Some(value),
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
                        "brand_name" => cfg.footer.brand_name = value,
                        "brand_avatar" => cfg.footer.brand_avatar = value,
                        "brand_bio" => cfg.footer.brand_bio = value,
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

    /// 「这次推送走 cookie 还是 appsecret」的唯一判定处。
    ///
    /// 优先级 env > toml：非空的 `WECHAT_AUTH_METHOD` 覆盖 `[wechat] auth_method`，
    /// 值大小写不敏感，两端空白不算数——和 `error/v1` 的 `first_check` 承诺的
    /// 「env > .env > toml」以及 `ship` 的既有行为对齐。
    ///
    /// `push` 以前自己写 `auth_method == "cookie"`，既只看 toml 又区分大小写：
    /// 同一个工作区、同一个 `WECHAT_AUTH_METHOD=COOKIE`，`ship` 走 cookie 通道，
    /// `push` 却退回 appsecret 并报「缺 WECHAT_APPID」——而 cookie 模式根本
    /// 不需要 AppSecret。两条命令对同一份环境给出不同结论，用户只能猜。
    pub fn is_cookie_auth(&self, env_auth_method: Option<&str>) -> bool {
        let from_env = env_auth_method
            .map(str::trim)
            .filter(|value| !value.is_empty());
        from_env
            .or(self.wechat_auth_method.as_deref().map(str::trim))
            .is_some_and(|value| value.eq_ignore_ascii_case("cookie"))
    }

    /// 读真实环境的版本。命令入口用它，避免每个调用点各写一遍
    /// `std::env::var("WECHAT_AUTH_METHOD")`。
    pub fn cookie_auth_from_env(&self) -> bool {
        self.is_cookie_auth(std::env::var("WECHAT_AUTH_METHOD").ok().as_deref())
    }

    /// appsecret 通道要的两个值各在不在：`(appid, secret)`。
    ///
    /// appid 可以来自 env 或 toml（`push` 两处都认），secret 只认 env
    /// （含 `.env`，由 `main` 启动时载入）。环境值传进来是为了让分支
    /// 在测试里能确定地跑一遍，不依赖跑测试的机器设了什么。
    pub fn appsecret_parts_from(
        &self,
        appid_from_env: Option<&str>,
        secret_from_env: Option<&str>,
    ) -> (bool, bool) {
        let appid = appid_from_env.is_some_and(|value| !value.trim().is_empty())
            || self
                .wechat_appid
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty());
        let secret = secret_from_env.is_some_and(|value| !value.trim().is_empty());
        (appid, secret)
    }

    /// 读真实环境的版本，供命令入口判断「这条命令现在还跑不动吗」。
    pub fn appsecret_ready(&self) -> bool {
        let appid = std::env::var("WECHAT_APPID").ok();
        let secret = std::env::var("WECHAT_SECRET").ok();
        let (appid, secret) = self.appsecret_parts_from(appid.as_deref(), secret.as_deref());
        appid && secret
    }
}

pub fn split_toml_pair(line: &str) -> Option<(&str, &str)> {
    let (key, rest) = line.split_once('=')?;
    Some((key.trim(), parse_toml_value(rest.trim())))
}

/// 切开一条 `key = value` 的右侧。
///
/// `init` 生成的配置带行尾说明，例如：
/// `theme = "default" # default | warm | ...`。老实现只在「整行以引号结尾」时
/// 才去引号，于是把 `"default" # default | warm | ...` 整段当成主题名——主题
/// 校验一严，新建项目第一次 `render` 就报 `invalid theme name`。
///
/// 规则：引号字符串读到配对的收尾引号为止（`\"` 不算收尾，引号里的 `#` 也不是
/// 注释）；未加引号的值在第一个 ` #` 处截断。引号没闭合时原样返回。
fn parse_toml_value(rest: &str) -> &str {
    let Some(quote) = rest.chars().next().filter(|c| *c == '"' || *c == '\'') else {
        return match rest.find(" #") {
            Some(idx) => rest[..idx].trim_end(),
            None => rest,
        };
    };

    let mut escaped = false;
    for (idx, ch) in rest.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if quote == '"' => escaped = true,
            c if c == quote => return &rest[1..idx],
            _ => {}
        }
    }
    rest
}

pub fn sample_config() -> &'static str {
    r#"[articles]
root = "/path/to/articles"

[wechat]
appid = ""
author = ""
account_type = "personal"
auth_method = "appsecret" # appsecret (default, needs IP whitelist) | cookie (browser session, bypasses IP whitelist; run `moonpub login` first)
auto_publish = false
theme = "default" # default | warm | dark | geek | geek-black | blueprint | ai-lab | paper | magazine | notebook | classic | forest | sunset | ocean | mono | editorial | zen | newsletter | academic | cyber | letter | mist | gallery | moonlit | porcelain | fieldnote | monokai | dracula | nord | one-dark | gruvbox
collection = ""
thumb_media_id = ""
author_bio = ""
qrcode = ""

[footer]
enabled = false
variant = "community" # community | minimal
title = ""
description = ""
rules = ""
qrcode = ""
qrcode_note = "请先关注我的微信公众号，再长按下方二维码入群。\n若二维码过期，请在公众号后台回复 加群 获取最新二维码。"
follow_image = ""
follow_text = ""
divider = "— · —"

[blog]
kind = "zola"
root = "/path/to/blog"

[template]
name = ""

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
auth_method = "appsecret" # appsecret (default, needs IP whitelist) | cookie (browser session, bypasses IP whitelist; run `moonpub login` first)
auto_publish = false
theme = "default" # default | warm | dark | geek | geek-black | blueprint | ai-lab | paper | magazine | notebook | classic | forest | sunset | ocean | mono | editorial | zen | newsletter | academic | cyber | letter | mist | gallery | moonlit | porcelain | fieldnote | monokai | dracula | nord | one-dark | gruvbox
collection = ""
thumb_media_id = ""
author_bio = ""
qrcode = ""

[footer]
enabled = false
variant = "community" # community | minimal
title = ""
description = ""
rules = ""
qrcode = ""
qrcode_note = "请先关注我的微信公众号，再长按下方二维码入群。\n若二维码过期，请在公众号后台回复 加群 获取最新二维码。"
follow_image = ""
follow_text = ""
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
    crate::html::escape_toml_string(value)
}

fn unescape_toml_string(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
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
    use crate::config::{COOKIE_AUTH_COMMANDS, Config, THEME_HINT, sample_config, split_toml_pair};
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
    fn split_toml_pair_strips_a_trailing_comment() {
        // `init` 自己生成的配置就长这样：值后面跟一段 `#` 说明。
        // 老实现只在「整行以引号结尾」时才去引号，于是 `theme` 变成
        // `"default" # default | warm | ...` 一整串，主题校验一严就直接报错。
        let (k, v) = split_toml_pair(r#"theme = "default" # default | warm | dark"#).unwrap();
        assert_eq!(k, "theme");
        assert_eq!(v, "default", "行尾注释被当成了值的一部分");
    }

    #[test]
    fn split_toml_pair_unquoted_value_with_comment() {
        let (k, v) = split_toml_pair("auto_publish = false # 默认关闭").unwrap();
        assert_eq!(k, "auto_publish");
        assert_eq!(v, "false");
    }

    #[test]
    fn split_toml_pair_keeps_hash_inside_quotes() {
        let (_, v) = split_toml_pair(r##"description = "C# 与 # 号" # 注释"##).unwrap();
        assert_eq!(v, "C# 与 # 号", "引号里的 # 不是注释");
    }

    #[test]
    fn split_toml_pair_handles_escaped_quote() {
        let (_, v) = split_toml_pair(r#"title = "他说 \"你好\"" # 注释"#).unwrap();
        assert_eq!(v, r#"他说 \"你好\""#, "转义引号不能被当成字符串结束");
    }

    #[test]
    fn sample_config_round_trips_into_usable_values() {
        // 这条锁的是「自己写出来的配置必须能被自己读回来」——
        // 之前没有任何测试做过回读，行尾注释污染值的问题一直躲过了全量测试，
        // 直到发版 smoke 里 init → render 才炸出来。
        let cfg = Config::from_toml(sample_config());
        let theme = cfg.wechat_theme.clone().unwrap_or_default();
        assert!(
            crate::theme::Theme::is_known_name(&theme),
            "示例配置的主题必须能直接用于 render，实际解析出：{theme}"
        );
        assert_eq!(cfg.wechat_auth_method.as_deref(), Some("appsecret"));
        assert_eq!(cfg.footer.variant, "community");
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
    fn sample_configs_include_follow_first_qrcode_note() {
        let generated = crate::config::sample_config();
        let generated_for_root =
            crate::config::sample_config_for_articles_root(&PathBuf::from("/tmp/moonpub"));
        let note = "请先关注我的微信公众号，再长按下方二维码入群。";

        assert!(generated.contains(note));
        assert!(generated_for_root.contains(note));
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
name = "My Standard Ending"
"#,
        );
        assert_eq!(cfg.template_name, Some("My Standard Ending".to_owned()));
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

    /// 回归：toml 写着 appsecret 时，`WECHAT_AUTH_METHOD=cookie` 必须赢。
    /// `push` 曾把 toml 排在前面，于是照文档设了 env 的用户仍然被要求填
    /// AppSecret——一条命令上写的配置压过了用户当场给的环境变量。
    #[test]
    fn env_auth_method_overrides_config_auth_method() {
        let cfg = Config {
            wechat_auth_method: Some("appsecret".to_owned()),
            ..Config::default()
        };

        assert!(cfg.is_cookie_auth(Some("cookie")));
    }

    /// 回归：`WECHAT_AUTH_METHOD=COOKIE` 曾因为 `== "cookie"` 被判成 appsecret，
    /// 而同一个值的 `ship` 走 cookie。大小写、两端空白都不能改变结论。
    #[test]
    fn auth_method_value_is_trimmed_and_case_insensitive() {
        let cfg = Config::default();

        assert!(cfg.is_cookie_auth(Some("COOKIE")));
        assert!(cfg.is_cookie_auth(Some("  Cookie ")));
        assert!(
            Config {
                wechat_auth_method: Some("Cookie".to_owned()),
                ..Config::default()
            }
            .is_cookie_auth(None)
        );
    }

    /// cookie 覆盖范围是与「有没有实现」绑定的承诺：这两个命令只有
    /// appsecret 实现（`push_browser` 里没有对应封装），写进常量就会让
    /// 缺凭据提示把用户引回同一个错误。
    #[test]
    fn commands_without_a_cookie_implementation_are_not_advertised() {
        for command in ["list-drafts", "update-draft"] {
            assert!(
                !COOKIE_AUTH_COMMANDS.contains(&command),
                "{command} has no cookie implementation; advertising it sends users in a circle"
            );
        }
        for command in COOKIE_AUTH_COMMANDS {
            assert!(
                crate::help::for_command(command).is_some(),
                "{command} is not a registered command"
            );
        }
    }

    #[test]
    fn appsecret_parts_accept_appid_from_env_or_toml_but_secret_only_from_env() {
        let from_toml = Config {
            wechat_appid: Some("wx-toml".to_owned()),
            ..Config::default()
        };
        assert_eq!(
            from_toml.appsecret_parts_from(None, Some("s3cr3t")),
            (true, true)
        );
        // appid 在 toml 里，secret 缺 → 不算齐全
        assert_eq!(from_toml.appsecret_parts_from(None, None), (true, false));
        // 纯空白等于没写
        assert_eq!(
            Config::default().appsecret_parts_from(Some("   "), Some(" s3cr3t ")),
            (false, true)
        );
        assert_eq!(
            Config::default().appsecret_parts_from(Some("wx-env"), None),
            (true, false)
        );
    }

    #[test]
    fn empty_or_absent_auth_method_falls_back_to_config_then_appsecret() {
        let cookie_cfg = Config {
            wechat_auth_method: Some("cookie".to_owned()),
            ..Config::default()
        };
        assert!(cookie_cfg.is_cookie_auth(Some("")));
        assert!(cookie_cfg.is_cookie_auth(Some("   ")));

        // 两处都没写就是 appsecret，这是推出草稿的默认通道。
        assert!(!Config::default().is_cookie_auth(None));
        assert!(!Config::default().is_cookie_auth(Some("appsecret")));
    }
}
