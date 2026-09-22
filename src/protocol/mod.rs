pub(crate) use crate::json_util::escape_json;
use serde::Serialize;

#[derive(Serialize)]
struct TextOutputPayload<'a> {
    output: &'a str,
}

pub(crate) fn to_json_string(text: &str) -> String {
    serialize_json(&TextOutputPayload { output: text })
}

pub(crate) fn serialize_json<T: Serialize>(payload: &T) -> String {
    serde_json::to_string(payload).expect("protocol payload serialization should not fail")
}

pub(crate) fn json_string_array(values: &[String]) -> String {
    let items = values
        .iter()
        .map(|value| format!("\"{}\"", escape_json(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{items}]")
}

pub(crate) fn optional_json_string(value: Option<&str>) -> String {
    value
        .map(|text| format!("\"{}\"", escape_json(text)))
        .unwrap_or_else(|| "null".to_owned())
}

mod doctor;
pub(crate) use doctor::*;

mod workspace;
pub(crate) use workspace::*;

mod workflow;
pub(crate) use workflow::*;

mod wechat;
pub(crate) use wechat::*;

mod evidence;
pub(crate) use evidence::*;

mod layout;
pub(crate) use layout::*;

mod check;
pub(crate) use check::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn typed_json_builders_escape_special_fields() -> Result<(), Box<dyn std::error::Error>> {
        let report = DoctorReport {
            moonpub_version: "0.4.test",
            articles_root: PathBuf::from("/tmp/Moon \"Pub\""),
            config_status: "ready",
            capabilities_summary: vec!["local preview", "WeChat draft"],
            warnings: vec!["quote \" slash \\ newline\nkept".to_owned()],
            next_step: "keep JSON valid",
            next_command: "moonpub check \"demo\"".to_owned(),
        };

        let payload: serde_json::Value = serde_json::from_str(&doctor_json(&report))?;

        assert_eq!(payload["command"], "doctor");
        assert_eq!(payload["articles_root"], "/tmp/Moon \"Pub\"");
        assert_eq!(payload["warnings"][0], "quote \" slash \\ newline\nkept");
        assert_eq!(payload["next_command"], "moonpub check \"demo\"");

        let preview: serde_json::Value = serde_json::from_str(&preview_json(
            Path::new("Articles/drafts/a \"b\".md"),
            Path::new("Articles/drafts/a \"b\".html"),
            false,
            "moonpub push \"a b\" --render",
        ))?;

        assert_eq!(preview["command"], "preview");
        assert_eq!(preview["article_path"], "Articles/drafts/a \"b\".md");
        assert_eq!(preview["opened_browser"], false);
        assert_eq!(preview["next_command"], "moonpub push \"a b\" --render");

        let wrapped: serde_json::Value =
            serde_json::from_str(&to_json_string("line one\nline \"two\""))?;
        assert_eq!(wrapped["output"], "line one\nline \"two\"");

        Ok(())
    }
}
