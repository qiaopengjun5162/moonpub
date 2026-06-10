// Humanize-zh: 基于规则的本地中文去AI味处理
// 参考: https://github.com/op7418/Humanizer-zh
//        https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing

/// Apply all humanization rules to a markdown string.
pub fn humanize(text: &str) -> String {
    let mut out = text.to_owned();

    // Phase 1: 删除填充短语和开场白
    out = remove_fillers(&out);

    // Phase 2: 替换AI高频词汇
    out = replace_ai_words(&out);

    // Phase 3: 打破三段式/排比结构
    out = break_parallelism(&out);

    // Phase 4: 简化过度修饰
    out = simplify_overqualification(&out);

    // Phase 5: 替换通用结论
    out = replace_bland_conclusions(&out);

    // Phase 6: 清理破折号过度使用
    out = reduce_em_dashes(&out);

    out
}

// ── Phase 1: 填充短语 ────────────────────────────────────────────────────────

fn remove_fillers(text: &str) -> String {
    let replacements = [
        // 开场白
        ("在当今这个快节奏的社会中", ""),
        ("在当今社会", ""),
        ("在当下这个时代", ""),
        ("众所周知，", ""),
        ("众所周知", ""),
        ("值得注意的是，", ""),
        ("值得一提的是，", ""),
        ("不可否认的是，", ""),
        ("毫无疑问，", ""),
        ("毋庸置疑，", ""),
        // 过度连接
        ("值得注意的是", ""),
        ("需要指出的是", ""),
        ("需要强调的是", ""),
        ("我们要认识到", ""),
        // 填充
        ("在这个时间点", "现在"),
        ("由于……的事实", "因为"),
        ("为了实现这一目标", "为此"),
        ("在……的过程中", ""),
        ("根据相关数据显示", "数据显示"),
    ];

    let mut out = text.to_owned();
    for (from, to) in &replacements {
        if out.contains(from) {
            out = out.replace(from, to);
        }
    }
    // 清理可能出现的多余逗号和空格
    out = collapse_punctuation(&out);
    out
}

// ── Phase 2: AI高频词汇 ───────────────────────────────────────────────────────

fn replace_ai_words(text: &str) -> String {
    let replacements = [
        // 单个词替换
        ("总而言之", "总的来说"),
        ("综上所述", "总结一下"),
        ("此外，", ""),
        ("深入探讨", "讨论"),
        ("起到了至关重要的作用", "很关键"),
        ("发挥了不可或缺的作用", "起到了作用"),
        ("彰显了", "体现了"),
        ("凸显出", "显示出"),
        // 过度强调
        ("极其重要的", "重要的"),
        ("至关重要的", "关键的"),
        ("不可或缺的", "必要的"),
        ("具有深远的意义", "意义重大"),
        // AI偏好用词
        ("赋能", "帮助"),
        ("抓手", "切入点"),
        ("底层逻辑", "基本原理"),
        ("闭环", "完整流程"),
        ("颗粒度", "细节"),
        ("组合拳", "综合方案"),
        ("护城河", "优势"),
        ("打法", "方法"),
        ("痛点", "问题"),
        // 单个"痛点"可以保留(日常用语), 但"核心痛点"改成"核心问题"
        ("核心痛点", "核心问题"),
    ];

    let mut out = text.to_owned();
    for (from, to) in &replacements {
        if out.contains(from) {
            out = out.replace(from, to);
        }
    }
    out
}

// ── Phase 3: 打破排比 ─────────────────────────────────────────────────────────

fn break_parallelism(text: &str) -> String {
    let mut out = text.to_owned();

    // "不仅...而且..." → 拆分为两个句子或简化
    if let Some(start) = out.find("不仅")
        && let Some(end) = out[start..].find("而且")
            && end < 80 {
                // 短距离内的"不仅...而且..."替换
                let segment = &out[start..start + end + 6];
                let simplified = segment
                    .replace("不仅", "")
                    .replace("而且", "也");
                out = out.replace(segment, &simplified);
            }

    // 破折号解释 → 去掉
    // "——确保用户能够高效地完成目标" → ""
    while let Some(start) = out.find("——确保") {
        let segment_end = out[start..]
            .find(['。', '；', '\n'])
            .map(|n| start + n)
            .unwrap_or(start + 30.min(out.len() - start));
        out.replace_range(start..segment_end, "");
    }

    out
}

// ── Phase 4: 简化过度修饰 ────────────────────────────────────────────────────

fn simplify_overqualification(text: &str) -> String {
    let mut out = text.to_owned();

    // 过度限定: "可能会对结果产生一些影响" → "可能影响结果"
    let patterns = [
        ("可以潜在地可能被认为", "可以认为"),
        ("可能会对结果产生一些", "可能会影响"),
        ("可以说是一个非常", "是一个非常"),
        ("在某种程度上来说", ""),
        ("从某种意义上看", ""),
    ];

    for (from, to) in &patterns {
        if out.contains(from) {
            out = out.replace(from, to);
        }
    }

    out
}

// ── Phase 5: 通用积极结论 ─────────────────────────────────────────────────────

fn replace_bland_conclusions(text: &str) -> String {
    let patterns = [
        "未来充满希望与机遇",
        "未来值得期待",
        "未来可期",
        "这标志着一个新的开始",
        "这是一个值得关注的方向",
        "让我们拭目以待",
    ];

    let mut out = text.to_owned();
    for p in &patterns {
        if out.contains(p) {
            // 替换为更实在的结尾，如果能找到上下文的话，否则直接删
            out = out.replace(p, "");
        }
    }
    collapse_punctuation(&out)
}

// ── Phase 6: 破折号 ───────────────────────────────────────────────────────────

fn reduce_em_dashes(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut dash_count = 0u32;
    let mut out = String::new();

    for &ch in chars.iter() {
        if ch == '—' {
            dash_count += 1;
            // 每段允许1个破折号，超过的换成逗号
            if dash_count > 2 {
                out.push('，');
            } else {
                out.push(ch);
            }
        } else {
            if ch == '\n' {
                dash_count = 0;
            }
            out.push(ch);
        }
    }
    out
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn collapse_punctuation(text: &str) -> String {
    let mut out = text.to_owned();
    // 连续逗号
    while out.contains("，，") {
        out = out.replace("，，", "，");
    }
    // 连续句号
    while out.contains("。。") {
        out = out.replace("。。", "。");
    }
    // 逗号后跟句号
    out = out.replace("，。", "。");
    // 句首逗号
    if out.starts_with('，') {
        out.remove(0);
    }
    // 多余空格
    while out.contains("  ") {
        out = out.replace("  ", " ");
    }
    out
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_zaidangjin() {
        let input = "在当今这个快节奏的社会中，人们越来越焦虑。";
        let output = humanize(input);
        assert!(!output.contains("在当今"));
        assert!(output.contains("焦虑"));
    }

    #[test]
    fn replaces_bujindan() {
        let input = "这不仅是一次更新，而且是我们思考生产力方式的革命。";
        let output = humanize(input);
        // Should remove/simplify the 不仅...而且... structure
        assert!(!output.contains("不仅"));
    }

    #[test]
    fn replaces_ai_words() {
        let input = "此外，这个系统起到了至关重要的作用，彰显了公司的技术实力。";
        let output = humanize(input);
        assert!(!output.contains("此外"));
        assert!(!output.contains("至关重要"));
        assert!(!output.contains("彰显了"));
    }

    #[test]
    fn reduces_excessive_dashes() {
        let input = "这个方案——经过多次讨论——最终确定——将于下周实施。";
        let output = humanize(input);
        // Should have reduced the number of dashes
        let dash_count = output.chars().filter(|&c| c == '—').count();
        assert!(dash_count <= 2);
    }

    #[test]
    fn replaces_buzzwords() {
        let input = "我们需要找到新的抓手，打通闭环，形成组合拳。";
        let output = humanize(input);
        assert!(!output.contains("抓手"));
        assert!(!output.contains("组合拳"));
        assert!(output.contains("切入点"));
        assert!(output.contains("综合方案"));
    }

    #[test]
    fn collapse_double_punctuation() {
        let input = "这是一个测试，，看看会不会被清理。。";
        let output = humanize(input);
        assert!(!output.contains("，，"));
        assert!(!output.contains("。。"));
    }
}
