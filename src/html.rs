//! HTML escaping shared by every renderer.
//!
//! Article text, brand copy from user config and cover metadata all end up in
//! HTML, so they must be escaped before interpolation. This used to exist as
//! four near-identical private copies (markdown inline/blocks, footer,
//! illustrate) plus a wider one in cover, which meant a fix to the escaping
//! rules had to be applied in five places. Keeping one implementation removes
//! that divergence risk.

/// Escapes `&`, `<`, `>`, `"` and `'` so untrusted text can be embedded both in
/// HTML text nodes and in attribute values.
///
/// Single pass on purpose: chaining `.replace()` would allocate a new `String`
/// per escaped character class.
pub(crate) fn escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::escape;

    #[test]
    fn escapes_markup_and_quotes() {
        assert_eq!(escape("a&b<c>d\"e'f"), "a&amp;b&lt;c&gt;d&quot;e&#39;f");
    }

    #[test]
    fn leaves_plain_text_untouched() {
        assert_eq!(escape("MoonPub 自动发布"), "MoonPub 自动发布");
    }
}
