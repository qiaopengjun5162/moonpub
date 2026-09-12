#[derive(Debug)]
pub(super) enum MdBlock<'a> {
    Fence(&'a str, Vec<(&'a str, &'a str)>, &'a str),
    Markdown(&'a str),
}

pub(super) fn parse_blocks(md: &str) -> Vec<MdBlock<'_>> {
    let mut blocks = Vec::new();
    let mut rest = md;

    while !rest.is_empty() {
        let starts_fence =
            rest.starts_with(":::") || rest.starts_with("\n:::") || rest.starts_with("\r\n:::");

        if starts_fence {
            if rest.starts_with("\r\n:::") {
                rest = &rest[2..];
            } else if rest.starts_with("\n:::") {
                rest = &rest[1..];
            }

            let after_fence = &rest[3..];
            let name_end = after_fence.find('\n').unwrap_or(after_fence.len());
            let name_line = after_fence[..name_end].trim();
            let name = name_line.split_whitespace().next().unwrap_or("");

            let inner_start = if name_end < after_fence.len() {
                name_end + 1
            } else {
                name_end
            };
            let after_name = &after_fence[inner_start..];

            let close_offset = after_name.find("\n:::");
            let (inner, remaining) = if let Some(off) = close_offset {
                let inner_text = &after_name[..off];
                let after_close = &after_name[off + 4..];
                let after_newline = after_close
                    .find('\n')
                    .map(|n| n + 1)
                    .unwrap_or(after_close.len());
                (inner_text, &after_close[after_newline..])
            } else {
                (after_name, "")
            };

            if !name.is_empty() {
                let (props, body) = split_fence_props(inner);
                blocks.push(MdBlock::Fence(name, props, body));
            }
            rest = remaining;
            continue;
        }

        let next_fence = rest.find("\n:::");
        if let Some(pos) = next_fence {
            let segment = &rest[..pos + 1];
            let trimmed = segment.trim();
            if !trimmed.is_empty() {
                blocks.push(MdBlock::Markdown(trimmed));
            }
            rest = &rest[pos + 1..];
        } else {
            let trimmed = rest.trim();
            if !trimmed.is_empty() {
                blocks.push(MdBlock::Markdown(trimmed));
            }
            break;
        }
    }

    blocks
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
            other => panic!("expected Markdown, got {:?}", other),
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
            other => panic!("expected Fence, got {:?}", other),
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
            other => panic!("expected Fence, got {:?}", other),
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
            other => panic!("expected Fence, got {:?}", other),
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
}
