#[derive(Debug)]
pub(super) enum MdBlock<'a> {
    Fence(&'a str, Vec<(&'a str, &'a str)>, &'a str),
    Markdown(&'a str),
}

/// 把正文切成「`:::name` 块」与「普通 Markdown 段」。
///
/// 两条线必须分开看：
///
/// 1. **围栏里的 `:::` 是代码内容**。`~~~markdown` 里教 `:::callout` 怎么写，
///    那一行不是指令；把它当成指令会从半路切开代码块——前面半个代码块照渲染，
///    后面几行被当成另一个块的正文，语言标记还可能漏进正文。所以扫描时要
///    跟着 [`fence_open`] / [`fence_closes`] 的围栏状态走。
/// 2. **`:::` 块的正文里也一样**。`:::card` 里放一段带 `:::` 的示例代码，
///    那个 `:::` 同样不是收尾。
///
/// 收尾只认**整行就是围墙**（允许尾部空白与多余的冒号），不认 `::: 随便写`：
/// 认错的代价是「块永远不结束」或者「块被半路切断」，两种都比多一行正文糟。
pub(super) fn parse_blocks(md: &str) -> Vec<MdBlock<'_>> {
    // 预先算好每行的字节起点：切块要的是 &md[a..b]，不能用 trim 后的 &str 拼。
    let mut lines: Vec<(usize, &str)> = Vec::new();
    let mut offset = 0usize;
    for raw in md.split_inclusive('\n') {
        let end = raw.trim_end_matches(['\n', '\r']).len();
        lines.push((offset, &raw[..end]));
        offset += raw.len();
    }

    let mut blocks = Vec::new();
    let mut markdown_start = 0usize;
    let mut fence: Option<(char, usize)> = None;
    let mut directive: Option<(&str, usize)> = None;

    let mut i = 0usize;
    while i < lines.len() {
        let (line_start, line) = lines[i];
        let trimmed = line.trim();

        // 代码围栏内部：整行都是代码，`:::` 没有语法意义。
        if let Some((marker, len)) = fence {
            if super::plain::fence_closes(trimmed, marker, len) {
                fence = None;
            }
            i += 1;
            continue;
        }
        if let Some((marker, len, _)) = super::plain::fence_open(trimmed) {
            fence = Some((marker, len));
            i += 1;
            continue;
        }

        if directive.is_none()
            && let Some(name) = directive_open(trimmed)
        {
            push_markdown(&mut blocks, &md[markdown_start..line_start]);
            let body_start = lines
                .get(i + 1)
                .map(|(start, _)| *start)
                .unwrap_or(md.len());
            directive = Some((name, body_start));
            i += 1;
            continue;
        }

        if let Some((name, body_start)) = directive
            && is_directive_close(trimmed)
        {
            push_fence(
                &mut blocks,
                name,
                strip_line_ending(&md[body_start..line_start]),
            );
            directive = None;
            markdown_start = next_line_start(&lines, i, md);
            i += 1;
            continue;
        }

        i += 1;
    }

    match directive {
        // 没写收尾：按旧口径，剩下全部算正文，不能悄悄丢掉。
        Some((name, body_start)) => push_fence(&mut blocks, name, &md[body_start..]),
        None => push_markdown(&mut blocks, &md[markdown_start..]),
    }

    blocks
}

/// `:::name` 开头返回名字；`:::` 自己（没有名字）返回 `None`。
fn directive_open(trimmed: &str) -> Option<&str> {
    let rest = trimmed.strip_prefix(":::")?;
    let name = rest.split_whitespace().next().unwrap_or("");
    // 名字是 ASCII 标识符：`:::figure src: a.png` 取 `figure`。
    if name.is_empty() || name.starts_with(':') {
        return None;
    }
    Some(name)
}

fn is_directive_close(trimmed: &str) -> bool {
    trimmed
        .strip_prefix(":::")
        .is_some_and(|rest| rest.chars().all(|ch| ch == ':' || ch.is_whitespace()))
}

/// 收尾行之前的正文不以换行结尾：`&md[..closing_start]` 会带上最后一个 `\n`，
/// 直接当正文会让 `body` 多一行空行（旧口径是切到 `\n:::` 的 `\n` 之前）。
fn strip_line_ending(text: &str) -> &str {
    let text = text.strip_suffix('\n').unwrap_or(text);
    text.strip_suffix('\r').unwrap_or(text)
}

fn next_line_start(lines: &[(usize, &str)], i: usize, md: &str) -> usize {
    lines
        .get(i + 1)
        .map(|(start, _)| *start)
        .unwrap_or(md.len())
}

fn push_markdown<'a>(blocks: &mut Vec<MdBlock<'a>>, segment: &'a str) {
    let trimmed = segment.trim();
    if !trimmed.is_empty() {
        blocks.push(MdBlock::Markdown(trimmed));
    }
}

fn push_fence<'a>(blocks: &mut Vec<MdBlock<'a>>, name: &'a str, inner: &'a str) {
    let (props, body) = split_fence_props(inner);
    blocks.push(MdBlock::Fence(name, props, body));
}

pub(super) fn split_fence_props(inner: &str) -> (Vec<(&str, &str)>, &str) {
    let mut props = Vec::new();
    let mut body_start = 0;
    for line in inner.lines() {
        let trimmed = line.trim();
        if let Some((k, v)) = trimmed.split_once(':') {
            let k = k.trim();
            let v = v.trim().trim_matches('"');
            if !k.is_empty() && !k.contains(' ') && k.len() < 30 {
                props.push((k, v));
                body_start += line.len() + 1;
                continue;
            }
        }
        if trimmed.is_empty() {
            body_start += line.len() + 1;
            continue;
        }
        break;
    }
    let body = if body_start < inner.len() {
        &inner[body_start..]
    } else {
        ""
    };
    (props, body.trim_start())
}

#[cfg(test)]
impl MdBlock<'_> {
    fn text(&self) -> &str {
        match self {
            MdBlock::Markdown(text) => text,
            MdBlock::Fence(_, _, body) => body,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_blocks_empty_is_empty() {
        assert!(parse_blocks("").is_empty());
    }

    #[test]
    fn parse_blocks_plain_markdown_segment() {
        let blocks = parse_blocks("hello world");
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            MdBlock::Markdown(s) => assert_eq!(*s, "hello world"),
            other => panic!("expected Markdown, got {other:?}"),
        }
    }

    #[test]
    fn parse_blocks_unterminated_fence_consumes_rest_as_body() {
        // without a closing ::: the remaining text becomes the fence body
        let blocks = parse_blocks(":::note\nbody text");
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            MdBlock::Fence(name, props, body) => {
                assert_eq!(*name, "note");
                assert!(props.is_empty());
                assert_eq!(*body, "body text");
            }
            other => panic!("expected Fence, got {other:?}"),
        }
    }

    #[test]
    fn parse_blocks_closed_fence_without_props() {
        let blocks = parse_blocks(":::note\nbody\n:::");
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            MdBlock::Fence(name, props, body) => {
                assert_eq!(*name, "note");
                assert!(props.is_empty());
                assert_eq!(*body, "body");
            }
            other => panic!("expected Fence, got {other:?}"),
        }
    }

    #[test]
    fn parse_blocks_closed_fence_with_props() {
        let blocks = parse_blocks(":::callout\nkey: val\nbody\n:::");
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            MdBlock::Fence(name, props, body) => {
                assert_eq!(*name, "callout");
                assert_eq!(props.as_slice(), &[("key", "val")]);
                assert_eq!(*body, "body");
            }
            other => panic!("expected Fence, got {other:?}"),
        }
    }

    #[test]
    fn parse_blocks_multiple_segments() {
        let blocks = parse_blocks("intro\n\n:::note\nx\n:::\ntail");
        assert_eq!(blocks.len(), 3);
        assert!(matches!(blocks[0], MdBlock::Markdown(_)));
        assert!(matches!(blocks[1], MdBlock::Fence(_, _, _)));
        assert!(matches!(blocks[2], MdBlock::Markdown(_)));
    }

    #[test]
    fn split_fence_props_empty() {
        let (props, body) = split_fence_props("");
        assert!(props.is_empty());
        assert_eq!(body, "");
    }

    #[test]
    fn split_fence_props_single_key_value() {
        let (props, body) = split_fence_props("key: value\nbody text");
        assert_eq!(props.as_slice(), &[("key", "value")]);
        assert_eq!(body, "body text");
    }

    #[test]
    fn split_fence_props_strips_quoted_value() {
        let (props, body) = split_fence_props("title: \"hello world\"\ntext");
        assert_eq!(props.as_slice(), &[("title", "hello world")]);
        assert_eq!(body, "text");
    }

    #[test]
    fn split_fence_props_rejects_keys_with_spaces() {
        let (props, body) = split_fence_props("my key: value\nbody");
        assert!(props.is_empty());
        assert_eq!(body, "my key: value\nbody");
    }

    #[test]
    fn split_fence_props_rejects_overlong_keys() {
        let long = "a".repeat(31);
        let input = format!("{long}: value\nbody");
        let (props, body) = split_fence_props(&input);
        assert!(props.is_empty());
        assert!(body.contains("body"));
    }

    /// 代码块里教 `:::` 怎么写，那一行是代码，不是文档结构。
    /// 此前 `parse_blocks` 用 `find("\n:::")` 扫字符串，围栏里的围栏标记
    /// 会把文档从半路切开：代码块前半段留下，后半段变成另一个块的正文。
    #[test]
    fn fence_line_inside_a_code_block_does_not_split_the_document() {
        let md = "```markdown\n:::callout\n重点\n:::\n```";
        let blocks = parse_blocks(md);
        assert_eq!(blocks.len(), 1, "{blocks:?}");
        assert!(matches!(blocks[0], MdBlock::Markdown(_)), "{blocks:?}");
        assert_eq!(blocks[0].text(), md);
    }

    /// `:::` 块正文里的代码块，里面的 `:::` 不能当成收尾。
    #[test]
    fn code_block_inside_a_directive_keeps_its_own_lines() {
        let md = ":::card\n```markdown\n:::tip\n提示\n:::\n```\n:::";
        let blocks = parse_blocks(md);
        assert_eq!(blocks.len(), 1, "{blocks:?}");
        match &blocks[0] {
            MdBlock::Fence(name, _, body) => {
                assert_eq!(*name, "card");
                assert!(body.contains(":::tip"), "{body}");
                assert!(body.contains("提示"), "{body}");
                assert_eq!(body.matches("```").count(), 2, "围栏被吃掉了: {body}");
            }
            other => panic!("expected Fence, got {other:?}"),
        }
    }

    /// 收尾必须是整行围墙：`::: 后面随便写` 不是收尾，否则块会被半路切断。
    #[test]
    fn closing_line_with_trailing_text_does_not_close_the_block() {
        let md = ":::card\n正文\n::: 这不是收尾\n更多正文\n:::";
        let blocks = parse_blocks(md);
        assert_eq!(blocks.len(), 1, "{blocks:?}");
        match &blocks[0] {
            MdBlock::Fence(_, _, body) => {
                assert!(body.contains("这不是收尾"), "{body}");
                assert!(body.contains("更多正文"), "{body}");
            }
            other => panic!("expected Fence, got {other:?}"),
        }
    }
}
