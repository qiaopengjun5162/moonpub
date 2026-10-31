use std::fs;
use std::path::{Path, PathBuf};

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutAuditReport {
    pub html_path: PathBuf,
    pub passed: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

pub fn audit_html_file(path: &Path) -> Result<LayoutAuditReport, AppError> {
    let html = fs::read_to_string(path).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(audit_html(path, &html))
}

pub fn audit_html(path: &Path, html: &str) -> LayoutAuditReport {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let lower = html.to_ascii_lowercase();

    for tag in ["<script", "<style", "<iframe", "<div"] {
        if lower.contains(tag) {
            errors.push(format!("contains forbidden tag `{tag}`"));
        }
    }

    // 属性检查必须带引号。裸的 ` class=` 会命中正文里转义过的代码块文本
    // （`&lt;main class=&quot;cover&quot;` → 仍含 `class=`），把自己文章里的
    // 示例代码误判成真属性，2026-10-30 写第 05 篇时实测踩到。
    for attr in [" class=\"", " class='", " id=\"", " id='"] {
        if lower.contains(attr) {
            let name = attr.trim().trim_end_matches(['"', '\'']);
            errors.push(format!("contains forbidden attribute `{name}=`"));
        }
    }

    for css in [
        "position:absolute",
        "position: absolute",
        "position:fixed",
        "position: fixed",
        "position:sticky",
        "position: sticky",
        "display:grid",
        "display: grid",
        "@media",
        "@keyframes",
        "float:",
    ] {
        if lower.contains(css) {
            errors.push(format!("contains risky CSS `{css}`"));
        }
    }

    if lower.contains("<html") || lower.contains("<body") || lower.contains("<!doctype") {
        warnings.push(
            "contains full HTML document shell; WeChat usually needs body fragment only".to_owned(),
        );
    }
    if !lower.contains("<section") {
        warnings.push(
            "contains no <section>; MoonPub WeChat output should usually use section wrappers"
                .to_owned(),
        );
    }
    if lower.contains("<pre") {
        warnings
            .push("contains <pre>; verify code blocks paste correctly in WeChat editor".to_owned());
    }

    LayoutAuditReport {
        html_path: path.to_path_buf(),
        passed: errors.is_empty(),
        warnings,
        errors,
    }
}

pub fn layout_audit_text(report: &LayoutAuditReport) -> String {
    let mut output = format!(
        "layout audit {}\n  html: {}\n  status: {}",
        if report.passed { "passed" } else { "failed" },
        report.html_path.display(),
        if report.passed { "passed" } else { "failed" }
    );
    if !report.errors.is_empty() {
        output.push_str("\n  errors:");
        for error in &report.errors {
            output.push_str(&format!("\n    - {error}"));
        }
    }
    if !report.warnings.is_empty() {
        output.push_str("\n  warnings:");
        for warning in &report.warnings {
            output.push_str(&format!("\n    - {warning}"));
        }
    }
    if report.passed {
        output.push_str("\n  next: local preview or WeChat draft push");
    } else {
        output.push_str("\n  next: remove forbidden tags / attributes before publishing");
    }
    output
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{audit_html, layout_audit_text};

    #[test]
    fn layout_audit_passes_wechat_section_fragment() {
        let report = audit_html(
            Path::new("demo.html"),
            r#"<section style="margin:0;"><p style="color:#333;">正文</p></section>"#,
        );

        assert!(report.passed);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn layout_audit_rejects_forbidden_tags_and_attributes() {
        let report = audit_html(
            Path::new("bad.html"),
            r#"<div class="card" style="display:grid;position:absolute"><script>x</script></div>"#,
        );

        assert!(!report.passed);
        assert!(report.errors.iter().any(|error| error.contains("<div")));
        assert!(report.errors.iter().any(|error| error.contains("class=")));
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("display:grid"))
        );
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("position:absolute"))
        );
    }

    #[test]
    fn layout_audit_ignores_escaped_code_samples() {
        // 2026-10-30：正文代码块渲染成 `&lt;main class=&quot;cover&quot;`，
        // 转义后仍含 `class=`，裸子串匹配会把示例代码判成真属性。
        let report = audit_html(
            Path::new("escaped.html"),
            r#"<section style="margin:0;"><p>&lt;main class=&quot;cover&quot; data-cover-style=&quot;geek-black&quot;&gt;</p></section>"#,
        );

        assert!(report.passed, "errors: {:?}", report.errors);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn layout_audit_rejects_single_quoted_class() {
        let report = audit_html(Path::new("sq.html"), "<section class='card'>x</section>");

        assert!(!report.passed);
        assert!(report.errors.iter().any(|error| error.contains("class=")));
    }

    #[test]
    fn layout_audit_outputs_text_and_json() {
        let report = audit_html(
            Path::new("page.html"),
            "<html><body><p>正文</p></body></html>",
        );

        let text = layout_audit_text(&report);

        assert!(text.contains("layout audit passed"));
        assert!(text.contains("warnings:"));
    }
}
