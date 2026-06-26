# 可配置 AI Provider Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 `moonpub write` / `expand` / `polish` / `ship --ai` 支持用户通过配置切换 AI provider 和模型，优先保持 DeepSeek 默认行为不变，新增 OpenAI 支持作为多 provider 起点。

**Architecture:** 在 `Config` 中新增 `[ai]` section；在 `src/ai.rs` 中引入 `AiProvider` 枚举和统一 `call_ai` 函数；各 AI 工作流命令从配置或环境变量读取 provider/model/key，再调用统一接口。不内置任何共享 API key，用户必须自己提供。

**Tech Stack:** Rust, ureq, serde_json, 手写 TOML parser。

---

### Task 1: 在 `Config` 中新增 `[ai]` 配置

**Files:**
- Modify: `src/config.rs:7-21` (struct), `src/config.rs:50-75` (parser), `src/config.rs:102-128` (sample)

- [ ] **Step 1: 新增字段**

```rust
pub struct Config {
    // ... existing fields ...
    pub ai_provider: Option<String>,
    pub ai_model: Option<String>,
    pub ai_api_key: Option<String>,
}
```

- [ ] **Step 2: 在 `from_toml` 中解析 `[ai]` section**

```rust
"ai" => match key {
    "provider" => cfg.ai_provider = Some(value),
    "model" => cfg.ai_model = Some(value),
    "api_key" => cfg.ai_api_key = Some(value),
    _ => {}
},
```

- [ ] **Step 3: 在 `sample_config` 中加入示例**

```toml
[ai]
provider = "deepseek"
model = "deepseek-chat"
# api_key = "sk-..."   # 优先使用 DEEPSEEK_API_KEY / OPENAI_API_KEY 环境变量
```

- [ ] **Step 4: 添加解析测试**

```rust
#[test]
fn parse_ai_config() {
    let cfg = Config::from_toml(r#"
[ai]
provider = "openai"
model = "gpt-4o-mini"
api_key = "sk-test"
"#);
    assert_eq!(cfg.ai_provider, Some("openai".to_owned()));
    assert_eq!(cfg.ai_model, Some("gpt-4o-mini".to_owned()));
    assert_eq!(cfg.ai_api_key, Some("sk-test".to_owned()));
}
```

Run: `cargo nextest run config::tests::parse_ai_config`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/config.rs
git commit -m "feat: add [ai] provider/model/api_key config"
```

---

### Task 2: 抽象 AI Provider 调用

**Files:**
- Modify: `src/ai.rs`

- [ ] **Step 1: 新增 `AiProvider` 枚举**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AiProvider {
    #[default]
    DeepSeek,
    OpenAi,
}

impl AiProvider {
    fn base_url(self) -> &'static str {
        match self {
            AiProvider::DeepSeek => "https://api.deepseek.com/v1/chat/completions",
            AiProvider::OpenAi => "https://api.openai.com/v1/chat/completions",
        }
    }

    fn default_model(self) -> &'static str {
        match self {
            AiProvider::DeepSeek => "deepseek-chat",
            AiProvider::OpenAi => "gpt-4o-mini",
        }
    }

    fn env_var_name(self) -> &'static str {
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
```

- [ ] **Step 2: 把 `default_api_key` 改为按 provider 读取**

```rust
pub fn api_key(provider: AiProvider) -> Result<String, AppError> {
    std::env::var(provider.env_var_name())
        .or_else(|_| {
            // Fallback to a generic key name for local experimentation.
            std::env::var("AI_API_KEY")
        })
        .map_err(|_| AppError::MissingValue(
            &format!("{} environment variable", provider.env_var_name())))
}
```

- [ ] **Step 3: 新增统一 `call_ai` 函数**

```rust
pub fn call_ai(
    provider: AiProvider,
    model: Option<&str>,
    system: &str,
    user: &str,
    api_key: &str,
) -> Result<String, AppError> {
    let url = provider.base_url();
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

    let resp = ureq::post(url)
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {api_key}"))
        .send_json(body)
        .map_err(|e| AppError::PushFailed {
            message: format!("{provider:?} API request failed: {e}"),
            ip_hint: None,
        })?;

    let json: serde_json::Value = resp.into_json().map_err(|e| AppError::PushFailed {
        message: format!("{provider:?} API parse error: {e}"),
        ip_hint: None,
    })?;

    json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| AppError::PushFailed {
            message: format!("{provider:?} API returned no content"),
            ip_hint: None,
        })
        .map(|s| s.to_owned())
}
```

- [ ] **Step 4: 修改 `call_deepseek` 为内部兼容层**

```rust
fn call_deepseek(system: &str, user: &str, api_key: &str) -> Result<String, AppError> {
    call_ai(AiProvider::DeepSeek, Some(DEFAULT_MODEL), system, user, api_key)
}
```

并删除已定义的 `DEEPSEEK_URL` / `DEFAULT_MODEL` 常量中不再直接使用的部分（保留 `DEFAULT_MODEL` 供 `call_deepseek` 使用）。

- [ ] **Step 5: 更新测试**

```rust
#[test]
fn ai_provider_parses_case_insensitive() {
    assert_eq!(
        "deepseek".parse::<AiProvider>().unwrap(),
        AiProvider::DeepSeek
    );
    assert_eq!(
        "OpenAI".parse::<AiProvider>().unwrap(),
        AiProvider::OpenAi
    );
}

#[test]
fn ai_provider_rejects_unknown() {
    assert!("foobar".parse::<AiProvider>().is_err());
}
```

Run: `cargo nextest run ai::tests`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/ai.rs
git commit -m "feat: abstract AiProvider and add OpenAI support"
```

---

### Task 3: 在 `ai_workflow.rs` / `app.rs` 中接入配置

**Files:**
- Modify: `src/ai_workflow.rs`（如果存在）或 `src/app.rs`

先确认入口位置。当前 `write` / `expand` / `polish` / `ship --ai` 都会调用 `ai.rs`。假设 `app.rs` 负责路由：

- [ ] **Step 1: 新增辅助函数 `resolve_ai_config(cfg: &Config)`**

```rust
fn resolve_ai_config(cfg: &Config) -> Result<(AiProvider, String, String), AppError> {
    let provider = cfg
        .ai_provider
        .as_deref()
        .unwrap_or("deepseek")
        .parse::<AiProvider>()?;
    let model = cfg
        .ai_model
        .clone()
        .unwrap_or_else(|| provider.default_model().to_owned());
    let api_key = cfg
        .ai_api_key
        .clone()
        .map(Ok)
        .unwrap_or_else(|| ai::api_key(provider))?;
    Ok((provider, model, api_key))
}
```

- [ ] **Step 2: 修改 `write` / `expand` / `polish` 调用**

把原来的：

```rust
let api_key = ai::default_api_key()?;
ai::generate_article(&idea, &api_key)
```

改为：

```rust
let (provider, model, api_key) = resolve_ai_config(&cfg)?;
ai::call_ai(provider, Some(&model), ai::ARTICLE_SYSTEM_PROMPT, &user_prompt, &api_key)
```

注意：需要把 `ARTICLE_SYSTEM_PROMPT` / `EXPAND_SYSTEM_PROMPT` / `POLISH_SYSTEM_PROMPT` 设为 `pub`。

- [ ] **Step 3: Commit**

```bash
git add src/ai.rs src/app.rs
git commit -m "feat: use configured AI provider in write/expand/polish"
```

---

### Task 4: 更新 `.env.example` 与文档

**Files:**
- Modify: `.env.example`
- Modify: `docs/USER_GUIDE.md` 或 `README.md`

- [ ] **Step 1: 更新 `.env.example`**

```env
# Optional: required only for AI commands (write / expand / polish / ship --ai).
# Supported providers: deepseek, openai. Model defaults to provider's recommended model.
# You can also set DEEPSEEK_API_KEY or OPENAI_API_KEY directly.
DEEPSEEK_API_KEY=sk-...
# OPENAI_API_KEY=sk-...
```

- [ ] **Step 2: 在 README 的 Configuration 小节加入 `[ai]` 示例**

```toml
[ai]
provider = "deepseek"      # deepseek | openai
model = "deepseek-chat"    # optional, defaults per provider
```

- [ ] **Step 3: Commit**

```bash
git add .env.example README.md README_zh.md
git commit -m "docs: document configurable AI provider and env vars"
```

---

### Task 5: 回归验证

- [ ] **Step 1: 不配置时仍用 DeepSeek**

```bash
DEEPSEEK_API_KEY=sk-test cargo nextest run ai::tests
```

Expected: PASS

- [ ] **Step 2: 配置 OpenAI 时解析正确**

写临时 `moonpub.toml`：

```toml
[ai]
provider = "openai"
model = "gpt-4o"
```

运行 `moonpub capabilities --json` 或一个 dry-run 命令确认配置被加载（如果已有测试覆盖 Config 解析则无需手动）。

- [ ] **Step 3: 全量测试**

```bash
cargo nextest run
```

Expected: 169+ tests PASS

---

## Self-Review

**Spec coverage:**
- 用户自带 Key → Task 1/2/3 均不内置共享 key
- 多 provider 可选 → Task 2
- 配置 model → Task 1/3
- 向后兼容 → Task 3 默认 deepseek

**Placeholder scan:** 无 TBD/TODO。

**Type consistency:** `AiProvider` 在 config 解析、api_key 读取、call_ai 中保持一致；model 为 `String` / `Option<&str>`。

**Open questions:** 若未来加入 Anthropic，只需扩展 `AiProvider` 枚举和三处 match，无需改调用方。
