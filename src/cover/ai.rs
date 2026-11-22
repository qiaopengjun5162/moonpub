//! AI 绘图封面的 prompt 推导与落盘。
//!
//! 方向（art style / 配色 / 情绪 / 年代标签）由文章内容决定，关键词提取与哈希都刻意
//! 确定性——同一篇文章永远得到同一张 prompt，便于复现与回归。

use super::*;

pub fn generate_ai_cover_prompt(title: &str, digest: &str, article_text: &str) -> String {
    let (art_style, color_scheme, mood, era_tag) =
        derive_art_direction(title, digest, article_text);
    let theme_keywords = extract_theme_keywords(title, digest, article_text);

    format!(
        "WeChat article cover, landscape 2.35:1 aspect ratio, cinematic anime quality.
         Art direction: {art_style} {color_scheme}.
         Atmosphere: {mood}, {era_tag}.
         Subject: A visually striking composition inspired by: {theme_keywords}.
         Make it feel like a frame from a high-budget anime movie or a concept art piece —          vivid, atmospheric, with a strong sense of place. Not a stock photo, not realistic people.
         The artwork fills the entire frame edge to edge. Leave the bottom 15% relatively clean          (darker or simpler) so a title can be overlaid later.
         Do NOT render any text, words, letters, or watermarks in the image — the title will be          added as a separate overlay. The image should be pure illustration.
         Style: anime, cartoon, sci-fi, futuristic. Bold colors, clean linework, dynamic composition.
         No watermarks, no borders, no logos, no text."
    )
}
/// Determine art direction parameters from article content using a content hash,
/// biased by detected content category for more relevant visual styles.
/// Returns (art_style_prefix, color_scheme, mood, era_suffix).
pub(super) fn derive_art_direction(
    title: &str,
    digest: &str,
    article_text: &str,
) -> (&'static str, &'static str, &'static str, &'static str) {
    let combined = format!("{title} {digest} {article_text}");
    let lower = combined.to_lowercase();
    let h = fnv1a(&combined) as usize;

    // 内容分类检测：每类文章匹配不同风格的视觉方向
    let category = if contains_any(
        &lower,
        &[
            "ai",
            "人工智能",
            "模型",
            "代码",
            "编程",
            "rust",
            "算法",
            "技术",
            "芯片",
            "数据",
            "程序",
            "系统",
            "agent",
            "智能体",
            "gpt",
            "llm",
            "token",
            "部署",
            "架构",
            "后端",
            "前端",
        ],
    ) {
        0usize // 科技
    } else if contains_any(
        &lower,
        &[
            "自然", "生活", "旅行", "植物", "风景", "山", "海", "咖啡", "日常", "散步", "城市",
            "街", "雨", "风", "花", "草", "天气",
        ],
    ) {
        1usize // 自然
    } else if contains_any(
        &lower,
        &[
            "成长", "效率", "方法", "习惯", "学习", "工作", "管理", "产品", "创业", "副业", "赚钱",
            "目标", "自律", "时间", "阅读", "写作", "读书",
        ],
    ) {
        2usize // 成长
    } else if contains_any(
        &lower,
        &[
            "情感", "关系", "爱", "心理", "亲密", "孤独", "焦虑", "治愈", "家庭", "朋友", "情绪",
        ],
    ) {
        3usize // 情感
    } else if contains_any(
        &lower,
        &[
            "设计", "审美", "视觉", "艺术", "配色", "品牌", "界面", "ui", "体验", "创意",
        ],
    ) {
        4usize // 设计
    } else if contains_any(
        &lower,
        &[
            "赛博",
            "cyber",
            "数字",
            "元宇宙",
            "区块链",
            "web3",
            "nft",
            "编程",
            "geek",
            "黑客",
            "加密",
            "defi",
            "钱包",
        ],
    ) {
        5usize // 赛博
    } else if contains_any(
        &lower,
        &[
            "旅行", "旅途", "远方", "自驾", "火车", "徒步", "骑行", "地图", "机场", "road", "路线",
            "背包",
        ],
    ) {
        6usize // 旅途
    } else if contains_any(
        &lower,
        &[
            "星空", "宇宙", "天", "夜", "光", "月亮", "太阳", "星", "云", "科幻", "未来", "梦",
            "时空", "维度",
        ],
    ) {
        7usize // 星辰
    } else {
        h % 8
    };

    // 每个分类有专属的艺术风格、配色、情绪和时代标签
    let art_styles: [&str; 8] = [
        // 科技：霓虹粒子、数据流、极简未来
        "anime-style digital illustration of glowing circuits and data streams with a",
        // 自然：水彩风格、手绘、植物园
        "Studio Ghibli-inspired watercolor landscape with a",
        // 成长：抽象几何、渐变、向上的动势
        "cel-shaded anime scene of upward motion and geometric abstraction with a",
        // 情感：柔和线条、温暖色调、手绘质感
        "vibrant manga-style illustration with soft brush strokes and a",
        // 设计：干净极简、建筑感、精细排版
        "sci-fi concept art of minimalist architecture and clean forms with a",
        // 赛博：蒸汽波、霓虹、数字故障
        "cyberpunk neon-lit cityscape with holographic elements and a",
        // 旅途：印象派、笔触感、旅途风景
        "anime-style scenery of a winding journey through atmospheric landscapes with a",
        // 星辰：太空歌剧、星云、深邃宇宙
        "mecha anime-style space vista with nebulae and cosmic light with a",
    ];

    let color_schemes: [&str; 8] = [
        // 科技
        "Electric blue and deep violet with cyan and silver highlights",
        // 自然
        "Emerald green and warm gold with soft ivory and sage",
        // 成长
        "Amber and rose gold with deep navy grounding",
        // 情感
        "Soft peach and lavender with warm coral accents",
        // 设计
        "Cool teal and crisp white with slate gray and a pop of yellow",
        // 赛博
        "Neon magenta and electric cyan against deep midnight blue",
        // 旅途
        "Warm terracotta and golden hour orange with indigo shadows",
        // 星辰
        "Deep space purple and cosmic blue with silver star highlights",
    ];

    let moods: [&str; 8] = [
        "energetic and futuristic",
        "calm and serene",
        "determined and optimistic",
        "tender and introspective",
        "clean and sophisticated",
        "intense and electrifying",
        "wistful and atmospheric",
        "mysterious and awe-inspiring",
    ];

    let era_tags: [&str; 8] = [
        "futuristic cyberpunk anime",
        "modern Ghibli-inspired landscape",
        "contemporary cel-shaded anime",
        "soft modern manga",
        "clean futuristic minimal",
        "neon-drenched synthwave",
        "impressionist travel sketch",
        "space opera cinematic",
    ];

    (
        art_styles[category],
        color_schemes[category],
        moods[category],
        era_tags[category],
    )
}

/// Check if any keyword is contained in the lowercase text.
pub(super) fn contains_any(text: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|kw| text.contains(kw))
}
/// Extract theme keywords from the article title, digest, and first ~500 chars of body.
pub(super) fn extract_theme_keywords(title: &str, digest: &str, article_text: &str) -> String {
    let stop_words = [
        "的", "了", "在", "是", "我", "有", "和", "就", "不", "人", "都", "一", "一个", "上", "也",
        "很", "到", "说", "要", "去", "你", "会", "着", "没有", "看", "好", "自己", "这", "他",
        "她", "它", "们", "那", "些",
    ];

    let mut seen = std::collections::HashSet::new();
    let mut keywords: Vec<String> = Vec::new();

    for source in [title, digest] {
        for word in source.split_whitespace() {
            let clean = word
                .trim_matches(|c: char| c.is_ascii_punctuation())
                .to_lowercase();
            if clean.len() < 2 {
                continue;
            }
            if stop_words.contains(&clean.as_str()) {
                continue;
            }
            if seen.insert(clean.clone()) {
                keywords.push(clean);
                if keywords.len() >= 6 {
                    break;
                }
            }
        }
        if keywords.len() >= 6 {
            break;
        }
    }

    if keywords.len() < 10 {
        let body: String = article_text.chars().take(500).collect();
        for word in body.split_whitespace() {
            if keywords.len() >= 15 {
                break;
            }
            let clean = word
                .trim_matches(|c: char| c.is_ascii_punctuation())
                .to_lowercase();
            if clean.len() < 3 {
                continue;
            }
            if stop_words.contains(&clean.as_str()) {
                continue;
            }
            if seen.insert(clean.clone()) {
                keywords.push(clean);
            }
        }
    }

    if keywords.is_empty() {
        return title.to_owned();
    }
    keywords.join(", ")
}

/// Generate an AI cover image and save it as PNG next to the article.
/// Also writes an HTML cover that uses the AI image as background with a title overlay,
/// so the final cover has the article title rendered cleanly over the AI illustration.
pub fn generate_ai_cover(
    article_path: &Path,
    title: &str,
    digest: &str,
    article_text: &str,
    provider: crate::ai::AiProvider,
    api_key: &str,
) -> Result<PathBuf, AppError> {
    let prompt = generate_ai_cover_prompt(title, digest, article_text);
    let png_bytes = crate::ai::generate_image(provider, &prompt, "1792x1024", api_key)?;

    let png_path = cover_png_path(article_path);
    fs::write(&png_path, &png_bytes).map_err(|source| AppError::Io {
        path: png_path.clone(),
        source,
    })?;

    let title_escaped = escape_html(title);
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cover</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:900px;height:500px;overflow:hidden;font-family:-apple-system,'PingFang SC','Hiragino Sans GB','Microsoft YaHei',sans-serif}}
.cover{{width:900px;height:500px;position:relative;overflow:hidden;background-image:url('{png_name}');background-size:cover;background-position:center}}
.overlay{{position:absolute;bottom:0;left:0;right:0;padding:40px 56px 36px;background:linear-gradient(transparent,rgba(0,0,0,.55))}}
.title{{font-size:36px;font-weight:900;line-height:1.2;color:#fff;letter-spacing:1px;max-width:680px;text-shadow:0 2px 16px rgba(0,0,0,.6)}}
</style>
</head>
<body><main class="cover" data-cover-style="ai-art">
  <div class="overlay"><h1 class="title">{title_escaped}</h1></div>
</main></body>
</html>"#,
        png_name = escape_css_url_string(
            png_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("cover.png"),
        ),
    );
    let html_path = cover_html_path(article_path);
    fs::write(&html_path, &html).map_err(|source| AppError::Io {
        path: html_path.clone(),
        source,
    })?;

    Ok(png_path)
}
