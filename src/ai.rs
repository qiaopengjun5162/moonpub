use crate::error::AppError;

#[cfg(test)]
use std::sync::{Mutex, OnceLock};

const DEEPSEEK_URL: &str = "https://api.deepseek.com/v1/chat/completions";
const OPENAI_URL: &str = "https://api.openai.com/v1/chat/completions";

pub const ARTICLE_SYSTEM_PROMPT: &str = r#"你是一位微信公众号作者。你的写作风格：简洁、真诚、不说教、不卖弄。
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

pub const POLISH_SYSTEM_PROMPT: &str = r#"你是一位资深文字编辑。你的任务是优化改进已有的文章，不改动作者的原意和核心观点。

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

pub const EXPAND_SYSTEM_PROMPT: &str = r#"你是一位读书博主。你擅长把零散的读书笔记组织成一篇可读的文章。

## 输入格式

你会收到一个 markdown 文件，包含 YAML frontmatter（书的信息）和正文（摘录、想法、评论片段）。

## 你的任务

1. **保留 frontmatter** — 标题、摘要、日期、标签等元数据保持不变或优化
2. **组织内容** — 把碎片化的摘录和想法按主题或逻辑重新组织
3. **补充过渡** — 在摘录之间加过渡段落，让文章流畅
4. **提炼观点** — 从散乱的想法中提炼出核心论点，放在 intro 里
5. **添加 context** — 如果原文缺少背景介绍（比如没写明白这书在讲什么），用你的知识补充
6. **保持真诚** — 这是读书笔记，不是营销文。语气平和、真诚，不夸张

## 结构

```
:::intro
这本书为什么值得读，以及你会从中获得什么
:::

h2 分节：按主题组织（如：核心观点 / 关键洞见 / 我的反思 / 实践启示）

:::callout（可选）
一句话点出全文核心
:::

:::summary
读完这本书，你最大的收获是什么
:::
```

## 注意事项

- 如果是微信读书（weread）导入的笔记，原文可能混有 `## 摘录` `## 想法` 等标注，请将其融入文章而非保留原标题
- 字数：800-2000字
- 不要编造书中没有的内容
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AiProvider {
    #[default]
    DeepSeek,
    OpenAi,
}

impl AiProvider {
    pub fn base_url(self) -> &'static str {
        match self {
            AiProvider::DeepSeek => DEEPSEEK_URL,
            AiProvider::OpenAi => OPENAI_URL,
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            AiProvider::DeepSeek => "deepseek-chat",
            AiProvider::OpenAi => "gpt-4o",
        }
    }

    pub fn env_var_name(self) -> &'static str {
        match self {
            AiProvider::DeepSeek => "DEEPSEEK_API_KEY",
            AiProvider::OpenAi => "OPENAI_API_KEY",
        }
    }
}

impl std::str::FromStr for AiProvider {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "deepseek" => Ok(AiProvider::DeepSeek),
            "openai" => Ok(AiProvider::OpenAi),
            _ => Err(AppError::PushFailed {
                message: format!("Unknown AI provider: {s}"),
                ip_hint: None,
            }),
        }
    }
}

pub fn api_key(provider: AiProvider) -> Result<String, AppError> {
    std::env::var(provider.env_var_name())
        .or_else(|_| std::env::var("AI_API_KEY"))
        .map_err(|_| {
            AppError::MissingValueString(format!(
                "{} environment variable (or AI_API_KEY)",
                provider.env_var_name()
            ))
        })
}

pub fn call_ai(
    provider: AiProvider,
    model: Option<&str>,
    system: &str,
    user: &str,
    api_key: &str,
) -> Result<String, AppError> {
    #[cfg(test)]
    if let Some(mock) = test_ai_response() {
        return Ok(mock);
    }

    let model = model.unwrap_or_else(|| provider.default_model());
    let body = serde_json::json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "temperature": 0.7,
        "max_tokens": 4096
    });

    let resp = ureq::post(provider.base_url())
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {api_key}"))
        .send_json(body)
        .map_err(|e| AppError::PushFailed {
            message: format!("AI API request failed: {e}"),
            ip_hint: None,
        })?;

    let json: serde_json::Value = resp.into_json().map_err(|e| AppError::PushFailed {
        message: format!("AI API parse error: {e}"),
        ip_hint: None,
    })?;

    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| AppError::PushFailed {
            message: "AI API returned no content".into(),
            ip_hint: None,
        })?;

    Ok(content.to_owned())
}

#[cfg(test)]
static TEST_AI_RESPONSE: OnceLock<Mutex<Option<String>>> = OnceLock::new();

#[cfg(test)]
fn test_ai_response() -> Option<String> {
    TEST_AI_RESPONSE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
}

#[cfg(test)]
pub fn set_test_ai_response(value: Option<&str>) {
    if let Ok(mut guard) = TEST_AI_RESPONSE.get_or_init(|| Mutex::new(None)).lock() {
        *guard = value.map(str::to_owned);
    }
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
        assert!(api_key(AiProvider::DeepSeek).is_err());
    }

    #[test]
    fn prompts_are_non_empty() {
        assert!(!ARTICLE_SYSTEM_PROMPT.is_empty());
        assert!(!POLISH_SYSTEM_PROMPT.is_empty());
        assert!(!EXPAND_SYSTEM_PROMPT.is_empty());
        assert!(ARTICLE_SYSTEM_PROMPT.contains("frontmatter"));
        assert!(EXPAND_SYSTEM_PROMPT.contains("读书博主"));
    }

    #[test]
    fn ai_provider_parses_case_insensitive() {
        assert_eq!(
            "deepseek".parse::<AiProvider>().unwrap(),
            AiProvider::DeepSeek
        );
        assert_eq!(
            "DeepSeek".parse::<AiProvider>().unwrap(),
            AiProvider::DeepSeek
        );
        assert_eq!(
            "DEEPSEEK".parse::<AiProvider>().unwrap(),
            AiProvider::DeepSeek
        );
        assert_eq!("openai".parse::<AiProvider>().unwrap(), AiProvider::OpenAi);
        assert_eq!("OpenAI".parse::<AiProvider>().unwrap(), AiProvider::OpenAi);
        assert_eq!("OPENAI".parse::<AiProvider>().unwrap(), AiProvider::OpenAi);
    }

    #[test]
    fn ai_provider_rejects_unknown() {
        let result = "unknown".parse::<AiProvider>();
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AppError::PushFailed { message, .. } => {
                assert!(message.contains("Unknown AI provider"));
            }
            _ => panic!("Expected PushFailed error, got {err:?}"),
        }
    }

    #[test]
    fn call_ai_uses_test_override_when_present() {
        set_test_ai_response(Some("mocked"));

        let output = call_ai(AiProvider::DeepSeek, None, "system", "user", "fake-key")
            .expect("test override should bypass network");

        assert_eq!(output, "mocked");
        set_test_ai_response(None);
    }
}
