pub fn escape_json(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            value => escaped.push(value),
        }
    }
    escaped
}

pub fn extract_json_string(line: &str, name: &str) -> Option<String> {
    let marker = format!("\"{name}\":\"");
    let start = line.find(&marker)? + marker.len();
    let mut output = String::new();
    let mut chars = line[start..].chars();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => return Some(output),
            '\\' => match chars.next()? {
                '"' => output.push('"'),
                '\\' => output.push('\\'),
                'n' => output.push('\n'),
                'r' => output.push('\r'),
                't' => output.push('\t'),
                other => output.push(other),
            },
            other => output.push(other),
        }
    }

    None
}

pub fn extract_json_optional_string(line: &str, name: &str) -> Option<String> {
    if line.contains(&format!("\"{name}\":null")) {
        None
    } else {
        extract_json_string(line, name)
    }
}

pub fn extract_json_optional_u64(line: &str, name: &str) -> Option<u64> {
    let marker = format!("\"{name}\":");
    let start = line.find(&marker)? + marker.len();
    let value = line[start..].split([',', '}']).next().unwrap_or("").trim();
    if value == "null" {
        None
    } else {
        value.parse().ok()
    }
}
