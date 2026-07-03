use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;

use crate::error::AppError;

fn prompt(question: &str, default: &str) -> String {
    if default.is_empty() {
        print!("{} ", question);
    } else {
        print!("{} [{}] ", question, default);
    }
    io::stdout().flush().ok();
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line).ok();
    let trimmed = line.trim().to_owned();
    if trimmed.is_empty() {
        default.to_owned()
    } else {
        trimmed
    }
}

pub fn init_config(path: &Path) -> Result<String, AppError> {
    if path.exists() {
        return Err(AppError::ConfigExists(path.to_path_buf()));
    }

    let is_tty = io::stdin().is_terminal();

    if !is_tty {
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
    let articles_root = prompt("  文章根目录（存放文章的文件夹）:", &root_default);

    println!();
    let appid = prompt("  公众号 AppID（wx 开头）:", "");
    let secret = prompt("  公众号 AppSecret:", "");

    let author = prompt("  公众号作者名:", "");

    println!("\n  选择主题风格:");
    println!("    [1] default   — 白底简洁");
    println!("    [2] warm      — 暖色阅读风");
    println!("    [3] dark      — 深蓝黑");
    println!("    [4] geek      — GitHub 风格（推荐）");
    println!("    [5] paper     — 纸张读书风");
    println!("    [6] magazine  — 杂志专栏风");
    println!("    [7] notebook  — 蓝色笔记风");
    println!("    [8] classic   — 经典衬线风");
    println!("    [9] forest    — 森林长文风");
    println!("    [10] sunset   — 日落观点风");
    println!("    [11] ocean    — 清爽教程风");
    println!("    [12] mono     — 黑白专注风");
    println!("    [13] editorial — 编辑部开篇风");
    println!("    [14] zen       — 安静慢读风");
    println!("    [15] newsletter — 周报合集风");
    println!("    [16] academic  — 研究笔记风");
    println!("    [17] cyber     — 高对比技术风");
    println!("    [18] letter    — 信笺随笔风");
    println!("    [19] mist      — 雾感生活风");
    println!("    [20] gallery   — 图文展陈风");
    let theme_choice = prompt("  选择 [1-20]:", "4");
    let theme = match theme_choice.as_str() {
        "1" => "default",
        "2" => "warm",
        "3" => "dark",
        "4" => "geek",
        "5" => "paper",
        "6" => "magazine",
        "7" => "notebook",
        "8" => "classic",
        "9" => "forest",
        "10" => "sunset",
        "11" => "ocean",
        "12" => "mono",
        "13" => "editorial",
        "14" => "zen",
        "15" => "newsletter",
        "16" => "academic",
        "17" => "cyber",
        "18" => "letter",
        "19" => "mist",
        "20" => "gallery",
        _ => "geek",
    };

    println!();
    let want_footer = prompt(
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
        footer_title = prompt("  社群名称（如「我的社群」）:", "");
        footer_description = prompt("  社群描述（一行话）:", "");
        footer_rules = prompt("  群规（用 \\n 分隔多条规则）:", "");
        footer_qrcode = prompt("  群二维码图片路径:", "");
        footer_qrcode_note = prompt("  二维码说明文字:", "");
        footer_follow_image = prompt("  关注引导图片 URL:", "");
        footer_follow_text = prompt("  结尾文案:", "");
        footer_divider = prompt("  分隔符:", "— · —");
    }

    println!();
    let template_name = prompt("  模板结尾名称（可选，留空则不启用）：", "");

    println!();
    let want_ai = prompt("  是否配置 AI 写作助手？[y/N]：", "N");
    let ai_enabled = want_ai.to_lowercase().starts_with('y');
    let mut ai_provider = String::new();
    let mut ai_model = String::new();
    if ai_enabled {
        ai_provider = prompt("  AI provider (deepseek/openai)：", "deepseek");
        let default_model = if ai_provider == "openai" {
            "gpt-4o-mini"
        } else {
            "deepseek-chat"
        };
        ai_model = prompt("  AI model：", default_model);
    }

    println!();
    let has_blog = prompt("  有博客需要同步导出吗？[y/N]:", "N");
    let blog_enabled = has_blog.to_lowercase().starts_with('y');
    let mut blog_kind = String::new();
    let mut blog_root = String::new();
    if blog_enabled {
        blog_kind = prompt("  博客类型 [zola]:", "zola");
        blog_root = prompt("  博客根目录:", "");
    }

    let mut toml = format!(
        "[articles]\nroot = \"{articles_root}\"\n\n\
         [wechat]\nappid = \"{appid}\"\n\
         author = \"{author}\"\n\
         account_type = \"personal\"\n\
         auto_publish = false\n\
         theme = \"{theme}\"\n\
         collection = \"\"\n\
         thumb_media_id = \"\"\n\
         author_bio = \"\"\n\
         qrcode = \"\"\n"
    );

    if footer_enabled {
        toml.push_str("\n[footer]\n");
        toml.push_str("enabled = true\n");
        toml.push_str(&format!("title = \"{footer_title}\"\n"));
        toml.push_str(&format!(
            "description = \"{}\"\n",
            footer_description.replace('\n', "\\n")
        ));
        toml.push_str(&format!(
            "rules = \"{}\"\n",
            footer_rules.replace('\n', "\\n")
        ));
        toml.push_str(&format!("qrcode = \"{footer_qrcode}\"\n"));
        toml.push_str(&format!(
            "qrcode_note = \"{}\"\n",
            footer_qrcode_note.replace('\n', "\\n")
        ));
        toml.push_str(&format!("follow_image = \"{footer_follow_image}\"\n"));
        toml.push_str(&format!("follow_text = \"{footer_follow_text}\"\n"));
        toml.push_str(&format!("divider = \"{footer_divider}\"\n"));
    }

    if blog_enabled {
        toml.push_str(&format!(
            "\n[blog]\nkind = \"{blog_kind}\"\nroot = \"{blog_root}\"\n"
        ));
    }

    toml.push_str("\n[template]\n");
    toml.push_str(&format!("name = \"{template_name}\"\n"));

    toml.push_str("\n[ai]\n");
    if ai_enabled {
        toml.push_str(&format!("provider = \"{ai_provider}\"\n"));
        toml.push_str(&format!("model = \"{ai_model}\"\n"));
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
    use super::upsert_wechat_env;

    #[test]
    fn upsert_wechat_env_replaces_credentials_and_preserves_other_lines() {
        let existing = "OTHER=value\nWECHAT_APPID=old\nWECHAT_SECRET=old-secret\n";

        let env = upsert_wechat_env(existing, "wx-new", "new-secret");

        assert_eq!(
            env,
            "OTHER=value\nWECHAT_APPID=wx-new\nWECHAT_SECRET=new-secret\n"
        );
    }
}
