use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;

use crate::error::AppError;

/// 向导需要外部世界提供的两件事：stdin 是不是终端、读一行输入。
///
/// `init_config` 本身是一台状态机（问什么 → 什么默认值 → 落到 TOML 哪个字段），
/// 但 `prompt()` 直接读 stdin，状态机被 I/O 缝死：400 多行里只有 26% 覆盖，
/// 而覆盖的那部分全是「非交互直接写样例」的旁路，真正的交互流程一条测试都没有。
///
/// 「读」抽出来之后，交互流程可以用脚本化应答跑完；`StdinIo` 是唯一的线上实现。
pub trait WizardIo {
    /// stdin 是否为终端。非终端（CI / 管道 / `--yes`）走样例配置，不阻塞等待。
    fn is_interactive(&self) -> bool;

    /// 打印 `prompt_text` 并读一行原始输入。
    /// `None` 表示输入已结束（EOF / Ctrl+D），调用方回落默认值而不是 panic。
    fn read_line(&mut self, prompt_text: &str) -> Option<String>;
}

/// 线上实现：真 stdin / stdout。
pub struct StdinIo;

impl WizardIo for StdinIo {
    fn is_interactive(&self) -> bool {
        io::stdin().is_terminal()
    }

    fn read_line(&mut self, prompt_text: &str) -> Option<String> {
        print!("{prompt_text}");
        io::stdout().flush().ok();
        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            // Ok(0) 是 EOF。以前 `.ok()` 把它变成空串、再靠「空串回落默认值」
            // 兜住——行为一样，但语义藏在两层逻辑之间，读代码的人看不出
            // 「Ctrl+D 走的是默认值」是设计还是巧合。现在显式返回 None。
            Ok(0) | Err(_) => None,
            Ok(_) => Some(line),
        }
    }
}

/// 问一行：空行回落 `default`，EOF 也回落 `default`。
///
/// 「空 = 默认」这条语义只写在这里，所有实现自动一致——各实现只负责读原始行，
/// 不需要（也不允许）各自再判一次空串。
fn ask(io: &mut impl WizardIo, question: &str, default: &str) -> String {
    let prompt_text = if default.is_empty() {
        format!("{question} ")
    } else {
        format!("{question} [{default}] ")
    };
    match io.read_line(&prompt_text) {
        Some(line) => {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                default.to_owned()
            } else {
                trimmed.to_owned()
            }
        }
        None => default.to_owned(),
    }
}

/// 向导第 n 项（从 1 起）对应的主题。**菜单打印和编号解析共用这一张表。**
///
/// 以前是 31 行 `println!` 和 31 个 `match` 分支各写一遍顺序——两侧任何一次
/// 插入或删除都会让「打印出来的编号」和「选出来的主题」错位，而错位不会报错：
/// 用户敲 5 拿到 notebook 还是 paper，全靠两份列表碰巧还对得上。
const THEME_MENU: [(&str, &str); 31] = [
    ("default", "白底简洁"),
    ("warm", "暖色阅读风"),
    ("dark", "深蓝黑"),
    ("geek", "GitHub 风格（推荐）"),
    ("paper", "纸张读书风"),
    ("magazine", "杂志专栏风"),
    ("notebook", "蓝色笔记风"),
    ("classic", "经典衬线风"),
    ("forest", "森林长文风"),
    ("sunset", "日落观点风"),
    ("ocean", "清爽教程风"),
    ("mono", "黑白专注风"),
    ("editorial", "编辑部开篇风"),
    ("zen", "安静慢读风"),
    ("newsletter", "周报合集风"),
    ("academic", "研究笔记风"),
    ("cyber", "高对比技术风"),
    ("letter", "信笺随笔风"),
    ("mist", "雾感生活风"),
    ("gallery", "图文展陈风"),
    ("moonlit", "月下隐林风"),
    ("porcelain", "瓷白慢读风"),
    ("fieldnote", "生活手记风"),
    ("geek-black", "极客黑工程风"),
    ("blueprint", "系统设计图纸风"),
    ("ai-lab", "AI 实验室风"),
    ("monokai", "Monokai 代码配色风"),
    ("dracula", "Dracula 夜色配色风"),
    ("nord", "Nord 北欧冷调风"),
    ("one-dark", "One Dark 编辑器风"),
    ("gruvbox", "Gruvbox 复古暖调风"),
];

/// 空输入 / 越界 / 非数字都落到这个主题（也是菜单里标的「推荐」）。
const DEFAULT_THEME: &str = "geek";

/// 用户 2026-11-22 明确否决的色系（暖橙 / 琥珀 / 米黄底，色相 30°–75°）。
///
/// 向导不删这些主题——老用户记得住编号，删掉会让后面序号全部前移——但会把它们
/// 标成「偏黄」，不再和别的选项看起来一样。`zen` 是米白底 + 橄榄，属「慎用」。
const YELLOWISH_THEMES: &[&str] = &[
    "warm",
    "paper",
    "sunset",
    "newsletter",
    "editorial",
    "letter",
    "gallery",
    "fieldnote",
    "zen",
];

/// 把用户敲的编号换成主题名：`"4"` → `"geek"`，`""` / `"32"` / `"x"` → `"geek"`。
fn theme_from_choice(choice: &str) -> &'static str {
    choice
        .trim()
        .parse::<usize>()
        .ok()
        .filter(|n| (1..=THEME_MENU.len()).contains(n))
        .map(|n| THEME_MENU[n - 1].0)
        .unwrap_or(DEFAULT_THEME)
}

fn print_theme_menu() {
    for (index, (name, label)) in THEME_MENU.iter().enumerate() {
        let hint = if YELLOWISH_THEMES.contains(name) {
            "（偏黄，不推荐）"
        } else {
            ""
        };
        println!("    [{:>2}] {name:<10} — {label}{hint}", index + 1);
    }
}

fn toml_prompt_value(value: &str) -> String {
    crate::html::escape_toml_string(value)
}

pub fn init_config(path: &Path, yes: bool) -> Result<String, AppError> {
    init_config_with(path, yes, &mut StdinIo)
}

/// 与 `init_config` 相同，但把 stdin / stdout 换成调用方给的 `io`。
///
/// 线上只有 `init_config` 一个调用点；测试用脚本化应答跑完整交互流程。
pub fn init_config_with(
    path: &Path,
    yes: bool,
    io: &mut impl WizardIo,
) -> Result<String, AppError> {
    if path.exists() {
        return Err(AppError::ConfigExists(path.to_path_buf()));
    }

    // 非交互环境（CI / 管道 / 演示脚本）或显式 --yes 时，直接写示例配置，
    // 使用默认主题，不阻塞等待用户输入。
    if !io.is_interactive() || yes {
        let articles_root =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let config = crate::config::sample_config_for_articles_root(&articles_root);
        fs::write(path, config).map_err(|source| AppError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        return Ok(format!("created {}", path.display()));
    }

    println!("\n  MoonPub 配置向导\n");
    println!("  按回车使用默认值，Ctrl+C 退出\n");

    let root_default = std::env::current_dir()
        .map(|d| d.display().to_string())
        .unwrap_or_default();
    let articles_root = ask(io, "  文章根目录（存放文章的文件夹）:", &root_default);

    println!();
    let appid = ask(io, "  公众号 AppID（wx 开头）:", "");
    let secret = ask(io, "  公众号 AppSecret:", "");

    let author = ask(io, "  公众号作者名:", "");

    println!("\n  选择主题风格:");
    print_theme_menu();
    let theme = theme_from_choice(&ask(io, "  选择 [1-31]:", "4"));

    println!();
    let want_footer = ask(
        io,
        "  需要文章结尾模板吗？（社群二维码、关注引导等）[y/N]:",
        "N",
    );
    let footer_enabled = want_footer.to_lowercase().starts_with('y');

    let mut footer_title = String::new();
    let mut footer_description = String::new();
    let mut footer_rules = String::new();
    let mut footer_qrcode = String::new();
    let mut footer_qrcode_note = String::new();
    let mut footer_follow_image = String::new();
    let mut footer_follow_text = String::new();
    let mut footer_divider = String::new();

    if footer_enabled {
        println!("\n  --- 结尾模板配置 ---");
        footer_title = ask(io, "  社群名称（如「我的社群」）:", "");
        footer_description = ask(io, "  社群描述（一行话）:", "");
        footer_rules = ask(io, "  群规（用 \\n 分隔多条规则）:", "");
        footer_qrcode = ask(io, "  群二维码图片路径:", "");
        footer_qrcode_note = ask(
            io,
            "  二维码说明文字:",
            "请先关注我的微信公众号，再长按下方二维码入群。\\n若二维码过期，请在公众号后台回复 加群 获取最新二维码。",
        );
        footer_follow_image = ask(io, "  关注引导图片 URL:", "");
        footer_follow_text = ask(io, "  结尾文案:", "");
        footer_divider = ask(io, "  分隔符:", "— · —");
    }

    println!();
    let template_name = ask(io, "  模板结尾名称（可选，留空则不启用）：", "");

    println!();
    let want_ai = ask(io, "  是否配置 AI 写作助手？[y/N]：", "N");
    let ai_enabled = want_ai.to_lowercase().starts_with('y');
    let mut ai_provider = String::new();
    let mut ai_model = String::new();
    if ai_enabled {
        ai_provider = ask(io, "  AI provider (deepseek/openai)：", "deepseek");
        // 默认模型跟着 provider 走：选了 openai 还默认 deepseek-chat，
        // 写进配置就是一组必然 404 的组合。
        let default_model = if ai_provider == "openai" {
            "gpt-4o-mini"
        } else {
            "deepseek-chat"
        };
        ai_model = ask(io, "  AI model：", default_model);
    }

    println!();
    let has_blog = ask(io, "  有博客需要同步导出吗？[y/N]:", "N");
    let blog_enabled = has_blog.to_lowercase().starts_with('y');
    let mut blog_kind = String::new();
    let mut blog_root = String::new();
    if blog_enabled {
        blog_kind = ask(io, "  博客类型 [zola]:", "zola");
        blog_root = ask(io, "  博客根目录:", "");
    }

    let mut toml = format!(
        "[articles]\nroot = \"{}\"\n\n\
         [wechat]\nappid = \"{}\"\n\
         author = \"{}\"\n\
         account_type = \"personal\"\n\
         auto_publish = false\n\
         theme = \"{theme}\"\n\
         collection = \"\"\n\
         thumb_media_id = \"\"\n\
         author_bio = \"\"\n\
         qrcode = \"\"\n",
        toml_prompt_value(&articles_root),
        toml_prompt_value(&appid),
        toml_prompt_value(&author),
    );

    if footer_enabled {
        toml.push_str("\n[footer]\n");
        toml.push_str("enabled = true\n");
        toml.push_str(&format!(
            "title = \"{}\"\n",
            toml_prompt_value(&footer_title)
        ));
        toml.push_str(&format!(
            "description = \"{}\"\n",
            toml_prompt_value(&footer_description)
        ));
        toml.push_str(&format!(
            "rules = \"{}\"\n",
            toml_prompt_value(&footer_rules)
        ));
        toml.push_str(&format!(
            "qrcode = \"{}\"\n",
            toml_prompt_value(&footer_qrcode)
        ));
        toml.push_str(&format!(
            "qrcode_note = \"{}\"\n",
            toml_prompt_value(&footer_qrcode_note)
        ));
        toml.push_str(&format!(
            "follow_image = \"{}\"\n",
            toml_prompt_value(&footer_follow_image)
        ));
        toml.push_str(&format!(
            "follow_text = \"{}\"\n",
            toml_prompt_value(&footer_follow_text)
        ));
        toml.push_str(&format!(
            "divider = \"{}\"\n",
            toml_prompt_value(&footer_divider)
        ));
    }

    if blog_enabled {
        toml.push_str(&format!(
            "\n[blog]\nkind = \"{}\"\nroot = \"{}\"\n",
            toml_prompt_value(&blog_kind),
            toml_prompt_value(&blog_root)
        ));
    }

    toml.push_str("\n[template]\n");
    toml.push_str(&format!(
        "name = \"{}\"\n",
        toml_prompt_value(&template_name)
    ));

    toml.push_str("\n[ai]\n");
    if ai_enabled {
        toml.push_str(&format!(
            "provider = \"{}\"\n",
            toml_prompt_value(&ai_provider)
        ));
        toml.push_str(&format!("model = \"{}\"\n", toml_prompt_value(&ai_model)));
    } else {
        toml.push_str("provider = \"deepseek\"\n");
        toml.push_str("model = \"deepseek-chat\"\n");
    }
    toml.push_str(
        "# api_key = \"sk-...\"   # 优先使用 DEEPSEEK_API_KEY / OPENAI_API_KEY 环境变量\n",
    );

    fs::write(path, &toml).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    if !secret.is_empty() {
        let env_path = path.parent().unwrap_or(path).join(".env");
        let existing = fs::read_to_string(&env_path).unwrap_or_default();
        let env_content = upsert_wechat_env(&existing, &appid, &secret);
        fs::write(&env_path, env_content).map_err(|source| AppError::Io {
            path: env_path.clone(),
            source,
        })?;
    }

    println!();
    println!("  ✅ 配置已创建: {}", path.display());
    if !secret.is_empty() {
        let env_path = path.parent().unwrap_or(path).join(".env");
        println!("  ✅ 凭证已写入: {}", env_path.display());
    }
    println!();
    println!("  下一步:");
    println!("    moonpub login          # 扫码登录微信");
    println!("    moonpub new \"标题\"     # 创建文章");
    println!("    moonpub ship 文章.md    # 发布");

    Ok(format!("created {}", path.display()))
}

fn upsert_wechat_env(existing: &str, appid: &str, secret: &str) -> String {
    let mut env_content = String::new();
    let mut wrote_appid = false;
    let mut wrote_secret = false;

    for line in existing.lines() {
        if line.starts_with("WECHAT_APPID=") {
            env_content.push_str(&format!("WECHAT_APPID={appid}\n"));
            wrote_appid = true;
        } else if line.starts_with("WECHAT_SECRET=") {
            env_content.push_str(&format!("WECHAT_SECRET={secret}\n"));
            wrote_secret = true;
        } else {
            env_content.push_str(line);
            env_content.push('\n');
        }
    }
    if !wrote_appid && !appid.is_empty() {
        env_content.push_str(&format!("WECHAT_APPID={appid}\n"));
    }
    if !wrote_secret {
        env_content.push_str(&format!("WECHAT_SECRET={secret}\n"));
    }
    env_content
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::{
        DEFAULT_THEME, THEME_MENU, WizardIo, YELLOWISH_THEMES, ask, init_config, init_config_with,
        theme_from_choice, toml_prompt_value, upsert_wechat_env,
    };
    use crate::error::AppError;
    use crate::test_helpers::{create_file, temp_root};
    use crate::theme::Theme;

    /// 脚本化应答：按顺序吐出，用完即 EOF（触发「回落默认值」那条路）。
    struct ScriptedIo {
        answers: VecDeque<String>,
        prompts: Vec<String>,
        interactive: bool,
    }

    impl ScriptedIo {
        fn new(answers: &[&str]) -> Self {
            Self {
                answers: answers.iter().map(|a| (*a).to_owned()).collect(),
                prompts: Vec::new(),
                interactive: true,
            }
        }
    }

    impl WizardIo for ScriptedIo {
        fn is_interactive(&self) -> bool {
            self.interactive
        }

        fn read_line(&mut self, prompt_text: &str) -> Option<String> {
            self.prompts.push(prompt_text.trim().to_owned());
            self.answers.pop_front()
        }
    }

    // ---- 菜单表与主题解析 ----

    #[test]
    fn theme_menu_covers_every_theme_exactly_once() {
        // 新加主题忘了进菜单 → 向导里永远选不到；删主题忘了出菜单 → 选到后
        // `Theme::from_name` 静默退化到 default。两个方向都在这条里红。
        let mut menu: Vec<&str> = THEME_MENU.iter().map(|(name, _)| *name).collect();
        let mut registry: Vec<&str> = Theme::names().to_vec();

        menu.sort_unstable();
        registry.sort_unstable();

        assert_eq!(menu, registry, "THEME_MENU 与 Theme::names() 必须一一对应");
    }

    #[test]
    fn every_menu_entry_is_a_registered_theme() {
        for (name, label) in THEME_MENU {
            assert!(
                Theme::is_known_name(name),
                "菜单项 {name}（{label}）不是已注册主题"
            );
            assert!(!label.is_empty(), "菜单项 {name} 缺中文说明");
        }
    }

    #[test]
    fn theme_from_choice_maps_every_numbered_entry() {
        for (index, (name, _)) in THEME_MENU.iter().enumerate() {
            let choice = (index + 1).to_string();
            assert_eq!(theme_from_choice(&choice), *name, "编号 {choice} 解析错误");
        }
    }

    #[test]
    fn theme_from_choice_falls_back_to_recommended_theme() {
        // 空输入、越界、0、负数、非数字、带空白 —— 全部落到推荐主题，
        // 而不是生成一个 `theme = ""` 的坏配置。
        for bad in ["", " ", "0", "32", "999", "-1", "abc", "4.5", " 25 "] {
            let expected = if bad.trim() == "25" {
                "blueprint"
            } else {
                DEFAULT_THEME
            };
            assert_eq!(theme_from_choice(bad), expected, "输入 {bad:?}");
        }
        assert_eq!(DEFAULT_THEME, "geek");
    }

    #[test]
    fn yellowish_list_only_names_real_menu_entries() {
        // 字符串打错（`warm ` / `Warm`）不会报错，只会让标注悄悄失效。
        for name in YELLOWISH_THEMES {
            assert!(
                THEME_MENU.iter().any(|(menu, _)| menu == name),
                "YELLOWISH_THEMES 里的 {name} 不在菜单中"
            );
        }
        assert!(!YELLOWISH_THEMES.contains(&DEFAULT_THEME));
    }

    // ---- ask 语义 ----

    #[test]
    fn ask_falls_back_to_default_on_eof() {
        let mut io = ScriptedIo::new(&[]);

        assert_eq!(ask(&mut io, "  问题:", "默认"), "默认");
        assert_eq!(io.prompts, vec!["问题: [默认]".to_owned()]);
    }

    #[test]
    fn ask_trims_whitespace_and_treats_blank_as_default() {
        let mut io = ScriptedIo::new(&["   ", "  显式  "]);

        assert_eq!(ask(&mut io, "q", "d"), "d");
        assert_eq!(ask(&mut io, "q", "d"), "显式");
    }

    #[test]
    fn ask_omits_bracket_hint_when_default_is_empty() {
        let mut io = ScriptedIo::new(&["x"]);

        assert_eq!(ask(&mut io, "  AppID:", ""), "x");
        assert_eq!(io.prompts, vec!["AppID:".to_owned()]);
    }

    // ---- 完整交互流程 ----

    #[test]
    fn interactive_wizard_with_all_defaults_writes_minimal_config()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("init-wizard-defaults")?;
        let config = root.join("moonpub.toml");
        // 空应答 = 每个问题都按回车：所有字段走默认值。
        let mut io = ScriptedIo::new(&[]);

        let output = init_config_with(&config, false, &mut io)?;
        let content = std::fs::read_to_string(&config)?;

        assert_eq!(output, format!("created {}", config.display()));
        // 17 个问题：根目录 / AppID / AppSecret / 作者 / 主题 / footer 开关 /
        // 模板名 / AI 开关 / 博客开关 = 9 个；footer 与 ai/blog 分支未开启不再追问。
        assert_eq!(io.prompts.len(), 9);
        assert!(content.contains("[articles]"));
        assert!(content.contains("theme = \"geek\""), "默认主题应为推荐项");
        assert!(content.contains("[template]\nname = \"\""));
        assert!(content.contains("provider = \"deepseek\"\nmodel = \"deepseek-chat\""));
        assert!(!content.contains("[footer]"), "未开启时不应写 [footer]");
        assert!(!content.contains("[blog]"), "未开启时不应写 [blog]");
        assert!(!root.join(".env").exists(), "没填 secret 就不写 .env");

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn interactive_wizard_writes_footer_block_and_env_credentials()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("init-wizard-footer")?;
        let config = root.join("moonpub.toml");
        let mut io = ScriptedIo::new(&[
            "/tmp/articles", // 文章根目录
            "wx1234567890",  // AppID
            "the-secret",    // AppSecret
            "巧鹏",          // 作者
            "22",            // 主题 → porcelain
            "y",             // 需要结尾模板
            "山居知音",      // 社群名称
            "安静的地方",    // 社群描述
            "禁止广告\\n禁止刷屏",
            "assets/qr.png",
            "",       // 二维码说明 → 回落默认
            "",       // 关注引导图 → 空
            "关注我", // 结尾文案
            "— · —",  // 分隔符
            "",       // 模板结尾名称
            "n",      // 不配 AI
            "n",      // 不同步博客
        ]);

        init_config_with(&config, false, &mut io)?;
        let content = std::fs::read_to_string(&config)?;
        let env = std::fs::read_to_string(root.join(".env"))?;

        assert!(content.contains("root = \"/tmp/articles\""));
        assert!(content.contains("appid = \"wx1234567890\""));
        assert!(content.contains("author = \"巧鹏\""));
        assert_eq!(theme_from_choice("22"), "porcelain");
        assert!(content.contains("theme = \"porcelain\""));
        assert!(content.contains("[footer]\nenabled = true\n"));
        assert!(content.contains("title = \"山居知音\""));
        assert!(content.contains("qrcode = \"assets/qr.png\""));
        // 空应答要走默认文案，而不是写成空串。
        assert!(content.contains("qrcode_note = \"请先关注我的微信公众号"));
        assert!(content.contains("follow_text = \"关注我\""));
        assert!(content.contains("divider = \"— · —\""));

        // AppSecret 只进 .env，不进 TOML。
        assert_eq!(env, "WECHAT_APPID=wx1234567890\nWECHAT_SECRET=the-secret\n");
        assert!(!content.contains("the-secret"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn interactive_wizard_defaults_ai_model_to_the_chosen_provider()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("init-wizard-ai")?;
        let config = root.join("moonpub.toml");
        let mut io = ScriptedIo::new(&[
            "",       // 根目录 → 当前目录
            "",       // AppID
            "",       // AppSecret
            "",       // 作者
            "25",     // 主题 → blueprint
            "n",      // 不要 footer
            "",       // 模板名
            "y",      // 配 AI
            "openai", // provider
            "",       // model → 应跟着 provider 变成 gpt-4o-mini
            "y",      // 同步博客
            "",       // 博客类型 → zola
            "/blog",  // 博客根目录
        ]);

        init_config_with(&config, false, &mut io)?;
        let content = std::fs::read_to_string(&config)?;

        assert!(content.contains("theme = \"blueprint\""));
        assert!(content.contains("[ai]\nprovider = \"openai\"\nmodel = \"gpt-4o-mini\""));
        assert!(content.contains("[blog]\nkind = \"zola\"\nroot = \"/blog\""));
        assert!(!root.join(".env").exists());

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn interactive_wizard_defaults_ai_model_to_deepseek_chat()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("init-wizard-ai-deepseek")?;
        let config = root.join("moonpub.toml");
        let mut io = ScriptedIo::new(&[
            "", "", "", "",  // 根目录 / AppID / AppSecret / 作者 → 全默认
            "",  // 主题 → geek
            "n", // 不要 footer
            "",  // 模板名
            "y", "",  // 配 AI，provider 直接回车 → deepseek
            "",  // model → 默认 deepseek-chat
            "n", // 不同步博客
        ]);

        init_config_with(&config, false, &mut io)?;
        let content = std::fs::read_to_string(&config)?;

        // provider 默认值是 deepseek，模型默认值必须跟着它走——否则回车的用户
        // 会拿到一组 provider/model 不匹配的配置。
        assert!(content.contains("[ai]\nprovider = \"deepseek\"\nmodel = \"deepseek-chat\""));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn non_interactive_io_never_asks_anything() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("init-wizard-pipe")?;
        let config = root.join("moonpub.toml");
        let mut io = ScriptedIo {
            answers: VecDeque::new(),
            prompts: Vec::new(),
            interactive: false,
        };

        init_config_with(&config, false, &mut io)?;

        assert!(io.prompts.is_empty(), "非终端环境不应提问");
        let content = std::fs::read_to_string(&config)?;
        assert!(content.contains("theme = \"default\""));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn interactive_wizard_refuses_to_overwrite_existing_config()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("init-wizard-existing")?;
        let config = root.join("moonpub.toml");
        create_file(&config, "existing")?;
        let mut io = ScriptedIo::new(&["/tmp"]);

        let err = init_config_with(&config, false, &mut io).unwrap_err();

        match err {
            AppError::ConfigExists(path) => assert_eq!(path, config),
            other => panic!("expected ConfigExists, got {other:?}"),
        }
        // 拒绝要发生在提问之前：否则用户答完 17 个问题才被告知文件已存在。
        assert!(io.prompts.is_empty(), "应在提问前拒绝");
        assert_eq!(std::fs::read_to_string(&config)?, "existing");

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    // ---- 原有测试 ----

    #[test]
    fn upsert_wechat_env_replaces_credentials_and_preserves_other_lines() {
        let existing = "OTHER=value\nWECHAT_APPID=old\nWECHAT_SECRET=old-secret\n";

        let env = upsert_wechat_env(existing, "wx-new", "new-secret");

        assert_eq!(
            env,
            "OTHER=value\nWECHAT_APPID=wx-new\nWECHAT_SECRET=new-secret\n"
        );
    }

    #[test]
    fn toml_prompt_value_escapes_quotes_backslashes_and_newlines() {
        // 引号必须转义，否则会闭合 TOML 字符串。
        assert_eq!(
            toml_prompt_value(r#"关注"公众号"入群"#),
            r#"关注\"公众号\"入群"#
        );
        // 反斜杠也必须转义：Windows 路径 C:\Users 里的 \U 在 TOML 里是非法转义。
        assert_eq!(
            toml_prompt_value(r"C:\Users\me\articles"),
            r"C:\\Users\\me\\articles"
        );
        // 真实换行转成字面 \n，不能折行破坏后续字段。
        assert_eq!(toml_prompt_value("第一行\n第二行"), r"第一行\n第二行");
    }

    #[test]
    fn non_interactive_init_writes_sample_config_without_env_file()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("init-non-interactive")?;
        let config = root.join("moonpub.toml");

        let output = init_config(&config, false)?;
        let content = std::fs::read_to_string(&config)?;

        assert_eq!(output, format!("created {}", config.display()));
        assert!(content.contains("[articles]"));
        assert!(content.contains("[wechat]"));
        assert!(content.contains("qrcode_note = \"请先关注我的微信公众号"));
        assert!(!root.join(".env").exists());

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn init_yes_writes_sample_config_with_default_theme() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = temp_root("init-yes")?;
        let config = root.join("moonpub.toml");

        let output = init_config(&config, true)?;
        let content = std::fs::read_to_string(&config)?;

        assert_eq!(output, format!("created {}", config.display()));
        assert!(content.contains("[articles]"));
        assert!(content.contains("theme = \"default\""));
        assert!(!root.join(".env").exists());

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn init_refuses_to_overwrite_existing_config() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("init-existing")?;
        let config = root.join("moonpub.toml");
        create_file(&config, "existing")?;

        let err = init_config(&config, false).unwrap_err();

        match err {
            AppError::ConfigExists(path) => assert_eq!(path, config),
            other => panic!("expected ConfigExists, got {other:?}"),
        }
        assert_eq!(std::fs::read_to_string(&config)?, "existing");

        std::fs::remove_dir_all(root)?;
        Ok(())
    }
}
