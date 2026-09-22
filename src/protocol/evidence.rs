use super::escape_json;
use crate::evidence::EvidenceReport;
use crate::release_check::{ReleaseCheckReport, ReleaseCheckStatus};

pub(crate) fn evidence_status_text(report: &EvidenceReport) -> String {
    let mut output = String::from("evidence status\n");
    output.push_str(&format!("  base_dir: {}\n", report.base_dir.display()));
    output.push_str(&format!("  passed: {}\n", report.passed));
    output.push_str(&format!(
        "  summary: {}/{} present, {} missing\n",
        report.present_count, report.required_count, report.missing_count
    ));
    for section in &report.sections {
        output.push_str(&format!("\n  {} ({})\n", section.title, section.id));
        for item in &section.items {
            output.push_str(&format!(
                "    [{}] {}: {}\n",
                if item.exists { "x" } else { " " },
                item.id,
                item.path.display()
            ));
        }
    }
    if !report.missing_paths.is_empty() {
        output.push_str("\n  missing_paths:\n");
        for path in &report.missing_paths {
            output.push_str(&format!("    - {}\n", path.display()));
        }
    }
    output.push_str(&format!("\n  next: {}\n", report.next_command));
    output.push_str(&format!("  step: {}", report.next_step));
    output
}

pub(crate) fn evidence_status_json(report: &EvidenceReport) -> String {
    let missing_paths = report
        .missing_paths
        .iter()
        .map(|path| format!("\"{}\"", escape_json(&path.display().to_string())))
        .collect::<Vec<_>>()
        .join(",");
    let sections = report
        .sections
        .iter()
        .map(|section| {
            let items = section
                .items
                .iter()
                .map(|item| {
                    format!(
                        "{{\"id\":\"{}\",\"path\":\"{}\",\"exists\":{}}}",
                        escape_json(item.id),
                        escape_json(&item.path.display().to_string()),
                        item.exists
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"id\":\"{}\",\"title\":\"{}\",\"items\":[{}]}}",
                escape_json(section.id),
                escape_json(section.title),
                items
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"command\":\"evidence-status\",\"base_dir\":\"{}\",\"passed\":{},\"required_count\":{},\"present_count\":{},\"missing_count\":{},\"missing_paths\":[{}],\"sections\":[{}],\"next_step\":\"{}\",\"next_command\":\"{}\"}}",
        escape_json(&report.base_dir.display().to_string()),
        report.passed,
        report.required_count,
        report.present_count,
        report.missing_count,
        missing_paths,
        sections,
        escape_json(report.next_step),
        escape_json(report.next_command)
    )
}

pub(crate) fn release_check_text(report: &ReleaseCheckReport) -> String {
    let mut output = String::from("release check\n");
    output.push_str(&format!("  version: {}\n", report.release_version));
    output.push_str(&format!("  repo_root: {}\n", report.repo_root.display()));
    output.push_str(&format!("  passed: {}\n", report.passed));
    for check in &report.checks {
        output.push_str(&format!(
            "  [{}] {}: {}\n",
            release_check_status_text(check.status),
            check.id,
            check.detail
        ));
        if let Some(next) = &check.next_command {
            output.push_str(&format!("      next: {next}\n"));
        }
    }
    output.push_str(&format!("  next: {}\n", report.next_command));
    output.push_str(&format!("  step: {}", report.next_step));
    output
}

pub(crate) fn release_check_json(report: &ReleaseCheckReport) -> String {
    let checks = report
        .checks
        .iter()
        .map(|check| {
            let next_command = check
                .next_command
                .as_ref()
                .map(|next| format!("\"{}\"", escape_json(next)))
                .unwrap_or_else(|| "null".to_owned());
            format!(
                "{{\"id\":\"{}\",\"title\":\"{}\",\"status\":\"{}\",\"detail\":\"{}\",\"next_command\":{}}}",
                escape_json(check.id),
                escape_json(check.title),
                escape_json(release_check_status_text(check.status)),
                escape_json(&check.detail),
                next_command
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"command\":\"release-check\",\"release_version\":\"{}\",\"repo_root\":\"{}\",\"passed\":{},\"checks\":[{}],\"next_step\":\"{}\",\"next_command\":\"{}\"}}",
        escape_json(report.release_version),
        escape_json(&report.repo_root.display().to_string()),
        report.passed,
        checks,
        escape_json(report.next_step),
        escape_json(&report.next_command)
    )
}

fn release_check_status_text(status: ReleaseCheckStatus) -> &'static str {
    match status {
        ReleaseCheckStatus::Pass => "pass",
        ReleaseCheckStatus::Fail => "fail",
    }
}

#[cfg(test)]
mod tests {
    use crate::release_check::{ReleaseCheckItem, ReleaseCheckReport, ReleaseCheckStatus};
    use std::path::{Path, PathBuf};

    #[test]
    fn evidence_status_json_lists_required_release_files() {
        let report = crate::evidence::evidence_status_from(Path::new("docs/first-run-evidence"));

        let output = super::evidence_status_json(&report);

        assert!(
            output.contains(r#""command":"evidence-status""#),
            "{output}"
        );
        assert!(
            output.contains(r#""base_dir":"docs/first-run-evidence""#),
            "{output}"
        );
        assert!(output.contains(r#""required_count":11"#), "{output}");
        assert!(output.contains(r#""present_count":"#), "{output}");
        assert!(output.contains(r#""missing_count":"#), "{output}");
        assert!(
            output.contains(r#""path":"docs/first-run-evidence/homepage/homepage-workspace.png""#),
            "{output}"
        );
        assert!(output.contains(r#""id":"homepage-workspace""#), "{output}");
        assert!(output.contains(r#""id":"preview-sent""#), "{output}");
        assert!(
            output.contains(r#""next_command":"moonpub evidence-status --json""#),
            "{output}"
        );
    }

    #[test]
    fn release_check_json_lists_gate_checks() {
        let report = ReleaseCheckReport {
            release_version: "0.4.2",
            repo_root: PathBuf::from("/repo"),
            passed: false,
            checks: vec![
                ReleaseCheckItem {
                    id: "release-gate-doc",
                    title: "v0.4.2 release gate document",
                    status: ReleaseCheckStatus::Pass,
                    detail: "found docs/RELEASE_GATE_v0.4.2_ZH.md".to_owned(),
                    next_command: None,
                },
                ReleaseCheckItem {
                    id: "release-evidence-files",
                    title: "required evidence files present",
                    status: ReleaseCheckStatus::Fail,
                    detail: "0/11 present, 11 missing".to_owned(),
                    next_command: Some("moonpub evidence-status --json".to_owned()),
                },
            ],
            next_step: "complete the first failing v0.4.2 release gate before preparing release assets",
            next_command: "moonpub evidence-status --json".to_owned(),
        };

        let output = super::release_check_json(&report);

        assert!(output.contains(r#""command":"release-check""#), "{output}");
        assert!(output.contains(r#""release_version":"0.4.2""#), "{output}");
        assert!(
            output.contains(r#""id":"release-evidence-files""#),
            "{output}"
        );
        assert!(output.contains(r#""status":"fail""#), "{output}");
        assert!(
            output.contains(r#""next_command":"moonpub evidence-status --json""#),
            "{output}"
        );
    }
}
