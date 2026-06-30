use std::fs;
use std::path::{Path, PathBuf};

use crate::article::resolve_article_path;
use crate::config::Config;
use crate::draft::{DraftWriteAction, write_article_file, write_or_update_article_file};
use crate::error::AppError;
use crate::ship::ship_article;

fn resolve_ai_config(cfg: &Config) -> Result<(crate::ai::AiProvider, String, String), AppError> {
    let provider = cfg
        .ai_provider
        .as_deref()
        .unwrap_or("deepseek")
        .parse::<crate::ai::AiProvider>()?;
    let model = cfg
        .ai_model
        .clone()
        .unwrap_or_else(|| provider.default_model().to_owned());
    let api_key = cfg
        .ai_api_key
        .clone()
        .map(Ok)
        .unwrap_or_else(|| crate::ai::api_key(provider))?;
    Ok((provider, model, api_key))
}

pub struct DraftOutput {
    pub path: PathBuf,
    pub action: DraftWriteAction,
    pub message: String,
}

pub fn write_article(articles_dir: &Path, cfg: &Config, idea: &str) -> Result<String, AppError> {
    let (provider, model, api_key) = resolve_ai_config(cfg)?;
    let user_prompt = format!(
        "请根据以下想法，写一篇微信公众号文章。\n\n想法：{idea}\n\n要求：800-2000字，有明确的标题和结构。"
    );
    let article = crate::ai::call_ai(
        provider,
        Some(&model),
        crate::ai::ARTICLE_SYSTEM_PROMPT,
        &user_prompt,
        &api_key,
    )?;
    let path = write_article_file(articles_dir, idea, &article)?;
    Ok(format!("generated\n  {}", path.display()))
}

pub fn draft_from_inbox(
    articles_dir: &Path,
    cfg: &Config,
    inbox: &Path,
) -> Result<DraftOutput, AppError> {
    let (provider, model, api_key) = resolve_ai_config(cfg)?;
    let inbox_path = resolve_article_path(articles_dir, inbox);
    let content = read_article(&inbox_path)?;
    let title = inbox_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("未命名素材");
    let user_prompt = draft_from_inbox_prompt(&content);
    let article = crate::ai::call_ai(
        provider,
        Some(&model),
        crate::ai::ARTICLE_SYSTEM_PROMPT,
        &user_prompt,
        &api_key,
    )?;
    let output = write_or_update_article_file(articles_dir, title, &article)?;
    Ok(DraftOutput {
        message: draft_from_inbox_message(&output.path, output.action),
        path: output.path,
        action: output.action,
    })
}

fn draft_from_inbox_message(path: &Path, action: DraftWriteAction) -> String {
    let draft = path.display();
    format!(
        "draft {}\n  {draft}\n  next: moonpub push {draft} --render",
        action.as_str()
    )
}

pub fn polish_article(
    articles_dir: &Path,
    cfg: &Config,
    article: &Path,
) -> Result<String, AppError> {
    let (provider, model, api_key) = resolve_ai_config(cfg)?;
    let art_path = resolve_article_path(articles_dir, article);
    let content = read_article(&art_path)?;
    let user_prompt = format!("请润色以下文章：\n\n{content}");
    let polished = crate::ai::call_ai(
        provider,
        Some(&model),
        crate::ai::POLISH_SYSTEM_PROMPT,
        &user_prompt,
        &api_key,
    )?;
    write_article_content(&art_path, &polished)?;
    Ok(format!("polished\n  {}", art_path.display()))
}

pub fn expand_article(
    articles_dir: &Path,
    cfg: &Config,
    article: &Path,
) -> Result<String, AppError> {
    let (provider, model, api_key) = resolve_ai_config(cfg)?;
    let art_path = resolve_article_path(articles_dir, article);
    let content = read_article(&art_path)?;
    let user_prompt =
        format!("请将以下读书笔记展开为一篇完整的微信公众号文章。\n\n笔记内容：\n\n{content}");
    let expanded = crate::ai::call_ai(
        provider,
        Some(&model),
        crate::ai::EXPAND_SYSTEM_PROMPT,
        &user_prompt,
        &api_key,
    )?;
    let output = expanded_article_output(&content, &expanded);
    write_article_content(&art_path, &output)?;
    Ok(format!("expanded\n  {}", art_path.display()))
}

pub fn ship_ai_article(
    articles_dir: &Path,
    config_path: Option<&Path>,
    cfg: &Config,
    article: &Path,
    style: Option<&str>,
) -> Result<String, AppError> {
    let (provider, model, api_key) = resolve_ai_config(cfg)?;
    let art_path = resolve_article_path(articles_dir, article);
    let content = read_article(&art_path)?;
    let user_prompt = format!("请润色以下文章：\n\n{content}");
    let polished = crate::ai::call_ai(
        provider,
        Some(&model),
        crate::ai::POLISH_SYSTEM_PROMPT,
        &user_prompt,
        &api_key,
    )?;
    write_article_content(&art_path, &polished)?;
    ship_article(articles_dir, config_path, &art_path, style)
}

fn read_article(path: &Path) -> Result<String, AppError> {
    fs::read_to_string(path).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn write_article_content(path: &Path, content: &str) -> Result<(), AppError> {
    fs::write(path, content).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn draft_from_inbox_prompt(content: &str) -> String {
    format!(
        "请把以下 Inbox 素材整理成一篇可继续编辑的微信公众号草稿。\n\n\
要求：\n\
1. 必须基于原始素材，实事求是，不要编造人物、地点、事件或结论。\n\
2. 保留作者随口记录的真实感，只做必要整理，不要过度修饰和拔高。\n\
3. 如果素材信息不足，就写成短文或记录，不要硬凑长篇。\n\
4. 输出必须包含 YAML frontmatter，并适合后续 moonpub render。\n\
5. 正文尽量自然分段，可以使用 :::intro 和 :::summary，但不要制造夸张标题党。\n\n\
Inbox 素材：\n\n{content}"
    )
}

fn expanded_article_output(original: &str, expanded: &str) -> String {
    let front = if original.starts_with("---") {
        original
            .lines()
            .skip(1)
            .take_while(|line| line.trim() != "---")
            .map(|line| format!("{line}\n"))
            .collect::<String>()
    } else {
        String::new()
    };

    if front.is_empty() {
        expanded.to_owned()
    } else {
        format!("---\n{front}---\n\n{expanded}")
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use crate::draft::{DraftWriteAction, write_or_update_article_file};
    use crate::test_helpers::temp_root;

    use super::{draft_from_inbox_message, draft_from_inbox_prompt, expanded_article_output};

    #[test]
    fn expanded_output_preserves_original_frontmatter() {
        let original = "---\ntitle: Demo\ndigest: Keep me\n---\n\nrough notes";
        let expanded = "polished body";

        let output = expanded_article_output(original, expanded);

        assert_eq!(
            output,
            "---\ntitle: Demo\ndigest: Keep me\n---\n\npolished body"
        );
    }

    #[test]
    fn draft_from_inbox_prompt_preserves_factual_constraints() {
        let prompt = draft_from_inbox_prompt("原始转写");

        assert!(prompt.contains("实事求是"));
        assert!(prompt.contains("不要编造"));
        assert!(prompt.contains("不要过度修饰"));
        assert!(prompt.contains("信息不足"));
        assert!(prompt.contains("原始转写"));
    }

    #[test]
    fn draft_from_inbox_message_includes_next_push_command() {
        let draft = PathBuf::from("Articles/drafts/demo.md");

        let message = draft_from_inbox_message(&draft, DraftWriteAction::Created);

        assert_eq!(
            message,
            "draft created\n  Articles/drafts/demo.md\n  next: moonpub push Articles/drafts/demo.md --render"
        );
    }

    #[test]
    fn writing_existing_draft_reuses_same_path() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("draft-reuse")?;
        let created = write_or_update_article_file(&root, "same title", "first")?;

        let rewritten = write_or_update_article_file(&root, "same title", "second")?;

        assert_eq!(created.action, DraftWriteAction::Created);
        assert_eq!(rewritten.action, DraftWriteAction::Updated);
        assert_eq!(rewritten.path, created.path);
        assert_eq!(fs::read_to_string(&created.path)?, "second");

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
