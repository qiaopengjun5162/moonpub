use super::serialize_json;
use crate::cdp::{WechatHealthReport, WechatHealthStatus};
use serde::Serialize;
use std::path::PathBuf;
pub(crate) struct DoctorReport {
    pub moonpub_version: &'static str,
    pub articles_root: PathBuf,
    pub config_status: &'static str,
    pub capabilities_summary: Vec<&'static str>,
    pub warnings: Vec<String>,
    pub next_step: &'static str,
    pub next_command: String,
}

#[derive(Serialize)]
struct DoctorJsonPayload<'a> {
    command: &'static str,
    moonpub_version: &'a str,
    articles_root: String,
    config_status: &'a str,
    capabilities_summary: &'a [&'static str],
    warnings: &'a [String],
    next_step: &'a str,
    next_command: &'a str,
}

#[derive(Serialize)]
struct WechatHealthJsonPayload<'a> {
    command: &'static str,
    status: &'static str,
    profile_mode: &'a str,
    session_file: Option<String>,
    session_file_exists: bool,
    current_url: &'a str,
    next_command: &'a str,
    next_step: &'a str,
}

pub(crate) fn doctor_text(report: &DoctorReport) -> String {
    let warnings = if report.warnings.is_empty() {
        "none".to_owned()
    } else {
        report.warnings.join("; ")
    };
    format!(
        "doctor\n  moonpub_version: {}\n  articles_root: {}\n  config_status: {}\n  capabilities: {}\n  warnings: {}\n  next: {}\n  next_step: {}",
        report.moonpub_version,
        report.articles_root.display(),
        report.config_status,
        report.capabilities_summary.join(" / "),
        warnings,
        report.next_command,
        report.next_step
    )
}

pub(crate) fn doctor_json(report: &DoctorReport) -> String {
    serialize_json(&DoctorJsonPayload {
        command: "doctor",
        moonpub_version: report.moonpub_version,
        articles_root: report.articles_root.display().to_string(),
        config_status: report.config_status,
        capabilities_summary: &report.capabilities_summary,
        warnings: &report.warnings,
        next_step: report.next_step,
        next_command: &report.next_command,
    })
}

pub(crate) fn wechat_health_text(report: &WechatHealthReport) -> String {
    let status = match report.status {
        WechatHealthStatus::Ready => "ready",
        WechatHealthStatus::NeedsLogin => "needs_login",
    };
    let session_file = report
        .session_file
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "<temporary profile>".to_owned());
    format!(
        "wechat browser automation health\n  status: {status}\n  profile_mode: {}\n  session_file: {session_file}\n  session_file_exists: {}\n  current_url: {}\n  next: {}\n  next_step: {}",
        report.profile_mode,
        report.session_file_exists,
        report.current_url,
        report.next_command,
        report.next_step
    )
}

pub(crate) fn wechat_health_json(report: &WechatHealthReport) -> String {
    let status = match report.status {
        WechatHealthStatus::Ready => "ready",
        WechatHealthStatus::NeedsLogin => "needs_login",
    };
    serialize_json(&WechatHealthJsonPayload {
        command: "wechat-health",
        status,
        profile_mode: report.profile_mode,
        session_file: report
            .session_file
            .as_ref()
            .map(|path| path.display().to_string()),
        session_file_exists: report.session_file_exists,
        current_url: &report.current_url,
        next_command: report.next_command,
        next_step: report.next_step,
    })
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::cdp::{WechatHealthReport, WechatHealthStatus};
    #[test]
    fn wechat_health_text_reports_next_command() {
        let report = WechatHealthReport {
            status: WechatHealthStatus::NeedsLogin,
            profile_mode: "persistent",
            session_file: Some(PathBuf::from("/tmp/session.json")),
            session_file_exists: false,
            current_url: "https://mp.weixin.qq.com/".to_owned(),
            next_command: "moonpub login",
            next_step: "scan the WeChat QR code once, then rerun wechat-health or configure",
        };

        let output = super::wechat_health_text(&report);

        assert!(output.contains("status: needs_login"));
        assert!(output.contains("session_file: /tmp/session.json"));
        assert!(output.contains("next: moonpub login"));
    }

    #[test]
    fn wechat_health_json_reports_ready_status() {
        let report = WechatHealthReport {
            status: WechatHealthStatus::Ready,
            profile_mode: "persistent",
            session_file: Some(PathBuf::from("/tmp/session.json")),
            session_file_exists: true,
            current_url: "https://mp.weixin.qq.com/cgi-bin/home".to_owned(),
            next_command: "moonpub configure --headed",
            next_step: "browser automation login is reusable",
        };

        let output = super::wechat_health_json(&report);

        assert!(output.contains(r#""command":"wechat-health""#));
        assert!(output.contains(r#""status":"ready""#));
        assert!(output.contains(r#""session_file":"/tmp/session.json""#));
        assert!(output.contains(r#""next_command":"moonpub configure --headed""#));
    }

    #[test]
    fn wechat_health_json_keeps_missing_session_as_null() -> Result<(), Box<dyn std::error::Error>>
    {
        let report = WechatHealthReport {
            status: WechatHealthStatus::NeedsLogin,
            profile_mode: "temporary",
            session_file: None,
            session_file_exists: false,
            current_url: "about:blank".to_owned(),
            next_command: "moonpub login",
            next_step: "scan once",
        };

        let payload: serde_json::Value = serde_json::from_str(&super::wechat_health_json(&report))?;

        assert_eq!(payload["command"], "wechat-health");
        assert_eq!(payload["status"], "needs_login");
        assert!(payload["session_file"].is_null());

        Ok(())
    }
}
