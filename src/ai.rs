use crate::error::AppError;

const DEEPSEEK_URL: &str = "https://api.deepseek.com/v1/chat/completions";
const DEFAULT_MODEL: &str = "deepseek-chat";

const ARTICLE_SYSTEM_PROMPT: &str = r#"你是一位微信公众号作者。你的写作风格：简洁、真诚、不说教、不卖弄。
读者是普通中国人，教育程度从初中到大学不等。用他们能理解的语言写作。

## 文章格式要求

每篇文章必须使用 YAML frontmatter：

---
title: 文章标题
digest: 120字以内的摘要（微信限制）
date: YYYY-MM-DD
tags: [标签1, 标签2]
---

## 排版规则

1. 文章结构：`:::intro` → 正文（h2/h3分节） → `:::summary`
2. 可用 Block 模板（每个文档只能用一次）：
   - `:::intro` — 开场导语，1-3句话，迅速抓住读者
   - `:::callout` — 核心结论，一句话点明主旨
     ```
     :::callout
     label: 核心观点
     这里写一句话结论
     :::
     ```
   - `:::steps` — 分步骤说明
     ```
     :::steps
     1. 第一步
     2. 第二步
     :::
     ```
   - `:::book-info` — 书籍信息卡片（仅读书笔记用）
     ```
     :::book-info
     title: 书名
     author: 作者
     publisher: 出版社
     rating: 8.5
     :::
     ```
   - `:::summary` — 结尾总结
3. h2/h3 用于分节，保持在2-4个h2
4. 段落长度：3-5句，不要巨长段
5. 适合手机屏幕阅读

## 标题技巧

好的微信公众号标题：
- 痛点+方案："总是读完就忘？这个方法让我记住了90%的内容"
- 数字+结果："这本书我读了3遍，总结出5条改变认知的真相"
- 悬念+冲突："她说了一句让全场安静的话……不是鸡汤，是事实"
- 读者标签+共鸣："致每一个还在坚持读书的人"

## 风格要求

- 不用"首先、其次、最后"这类流水账连接词
- 不用"值得注意的是"、"综上所述"等书面套话
- 不用排比句
- 举例要具体，不说"比如某个名人"，直接说"王阳明在龙场驿的那个夜晚"
- 允许口语化表达：说白了、说人话就是、你想想看
- 用短句，少用超过30个字的长句
"#;

const POLISH_SYSTEM_PROMPT: &str = r#"你是一位资深文字编辑。你的任务是优化改进已有的文章，不改动作者的原意和核心观点。

## 你会做的

1. 删掉废话、套话、书面腔（"值得注意的是"、"综上所述"、"首先其次最后"）
2. 拆开超过30字的长句
3. 让举例更具体
4. 段落控制在3-5句
5. 保持原文的结构和 block 模板
6. 保持 frontmatter 不变
7. 润色 title 让它更有吸引力（应用标题公式）
8. 优化 digest 让它更能抓住读者

## 你不会做的

- 添加新的观点或事实
- 改动文章结构
- 改掉作者的写作风格
- 添加任何你们作为AI的语言特征
"#;

pub fn default_api_key() -> Result<String, AppError> {
    std::env::var("DEEPSEEK_API_KEY")
        .map_err(|_| AppError::MissingValue("DEEPSEEK_API_KEY environment variable"))
}

pub fn generate_article(idea: &str, api_key: &str) -> Result<String, AppError> {
    let user_prompt = format!(
        "请根据以下想法，写一篇微信公众号文章。\n\n想法：{idea}\n\n要求：800-2000字，有明确的标题和结构。"
    );
    call_deepseek(ARTICLE_SYSTEM_PROMPT, &user_prompt, api_key)
}

pub fn polish_article(content: &str, api_key: &str) -> Result<String, AppError> {
    let user_prompt = format!("请润色以下文章：\n\n{content}");
    call_deepseek(POLISH_SYSTEM_PROMPT, &user_prompt, api_key)
}

fn call_deepseek(system: &str, user: &str, api_key: &str) -> Result<String, AppError> {
    let body = serde_json::json!({
        "model": DEFAULT_MODEL,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "temperature": 0.7,
        "max_tokens": 4096
    });

    let resp = ureq::post(DEEPSEEK_URL)
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {api_key}"))
        .send_json(body)
        .map_err(|e| AppError::PushFailed {
            message: format!("DeepSeek API request failed: {e}"),
            ip_hint: None,
        })?;

    let json: serde_json::Value = resp.into_json().map_err(|e| AppError::PushFailed {
        message: format!("DeepSeek API parse error: {e}"),
        ip_hint: None,
    })?;

    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| AppError::PushFailed {
            message: "DeepSeek API returned no content".into(),
            ip_hint: None,
        })?;

    Ok(content.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_api_key_errors_when_unset() {
        // When env is not set, returns error
        if std::env::var("DEEPSEEK_API_KEY").is_ok() {
            // Key is set in dev env, so cannot test the error path
            return;
        }
        assert!(default_api_key().is_err());
    }

    #[test]
    fn prompts_are_non_empty() {
        assert!(!ARTICLE_SYSTEM_PROMPT.is_empty());
        assert!(!POLISH_SYSTEM_PROMPT.is_empty());
        assert!(ARTICLE_SYSTEM_PROMPT.contains("frontmatter"));
        assert!(ARTICLE_SYSTEM_PROMPT.contains(":::intro"));
        assert!(ARTICLE_SYSTEM_PROMPT.contains(":::summary"));
    }
}
