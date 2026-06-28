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
