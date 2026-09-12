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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_json_passthrough_when_no_special_chars() {
        assert_eq!(escape_json("hello world"), "hello world");
        assert_eq!(escape_json(""), "");
    }

    #[test]
    fn escape_json_handles_all_special_chars() {
        assert_eq!(escape_json("a\"b"), "a\\\"b");
        assert_eq!(escape_json("a\\b"), "a\\\\b");
        assert_eq!(escape_json("a\nb"), "a\\nb");
        assert_eq!(escape_json("a\rb"), "a\\rb");
        assert_eq!(escape_json("a\tb"), "a\\tb");
    }

    #[test]
    fn extract_json_string_basic() {
        assert_eq!(
            extract_json_string(r#""foo":"bar""#, "foo"),
            Some("bar".to_string())
        );
    }

    #[test]
    fn extract_json_string_missing_marker_returns_none() {
        assert_eq!(extract_json_string(r#""baz":"x""#, "foo"), None);
    }

    #[test]
    fn extract_json_string_unterminated_returns_none() {
        assert_eq!(extract_json_string(r#""foo":"bar"#, "foo"), None);
    }

    #[test]
    fn extract_json_string_empty_value() {
        assert_eq!(
            extract_json_string(r#""foo":"""#, "foo"),
            Some("".to_string())
        );
    }

    #[test]
    fn extract_json_string_unescapes_quotes_and_newlines() {
        assert_eq!(
            extract_json_string(r#""foo":"he said \"hi\"""#, "foo"),
            Some("he said \"hi\"".to_string())
        );
        let got = extract_json_string(r#""foo":"line1\nline2""#, "foo").unwrap();
        assert_eq!(got, "line1\nline2");
    }

    #[test]
    fn extract_json_optional_string_null_is_none() {
        assert_eq!(extract_json_optional_string(r#""foo":null"#, "foo"), None);
    }

    #[test]
    fn extract_json_optional_string_missing_is_none() {
        assert_eq!(extract_json_optional_string(r#""bar":"x""#, "foo"), None);
    }

    #[test]
    fn extract_json_optional_string_present() {
        assert_eq!(
            extract_json_optional_string(r#""foo":"val""#, "foo"),
            Some("val".to_string())
        );
    }

    #[test]
    fn extract_json_optional_u64_null_is_none() {
        assert_eq!(extract_json_optional_u64(r#""count":null"#, "count"), None);
    }

    #[test]
    fn extract_json_optional_u64_valid() {
        assert_eq!(
            extract_json_optional_u64(r#""count":42"#, "count"),
            Some(42)
        );
        assert_eq!(
            extract_json_optional_u64(r#""count":42,"#, "count"),
            Some(42)
        );
        assert_eq!(
            extract_json_optional_u64(r#""count":42}"#, "count"),
            Some(42)
        );
    }

    #[test]
    fn extract_json_optional_u64_missing_or_nonnumeric_is_none() {
        assert_eq!(extract_json_optional_u64(r#""other":1"#, "count"), None);
        assert_eq!(extract_json_optional_u64(r#""count":"abc""#, "count"), None);
        assert_eq!(extract_json_optional_u64(r#""count":-1"#, "count"), None);
    }
}
