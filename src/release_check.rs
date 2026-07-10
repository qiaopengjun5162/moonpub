use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::evidence::{EvidenceReport, evidence_status_from};

const RELEASE_GATE_DOC: &str = "docs/RELEASE_GATE_v0.4.2_ZH.md";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseCheckReport {
    pub release_version: &'static str,
    pub repo_root: PathBuf,
    pub passed: bool,
    pub checks: Vec<ReleaseCheckItem>,
    pub next_step: &'static str,
    pub next_command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseCheckItem {
    pub id: &'static str,
    pub title: &'static str,
    pub status: ReleaseCheckStatus,
    pub detail: String,
    pub next_command: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseCheckStatus {
    Pass,
    Fail,
}

pub fn release_check() -> Result<ReleaseCheckReport, AppError> {
    let cwd = std::env::current_dir().map_err(|source| AppError::Io {
        path: PathBuf::from("."),
        source,
    })?;
    release_check_from(&find_repo_root(&cwd))
}

pub fn release_check_from(repo_root: &Path) -> Result<ReleaseCheckReport, AppError> {
    let release_doc_path = repo_root.join(RELEASE_GATE_DOC);
    let release_doc = std::fs::read_to_string(&release_doc_path).ok();
    let evidence = evidence_status_from(&repo_root.join("docs/first-run-evidence"));
    let mut checks = Vec::new();

    checks.push(ReleaseCheckItem {
        id: "release-gate-doc",
        title: "v0.4.2 release gate document",
        status: pass_if(release_doc.is_some()),
        detail: if release_doc.is_some() {
            format!("found {}", release_doc_path.display())
        } else {
            format!("missing {}", release_doc_path.display())
        },
        next_command: if release_doc.is_some() {
            None
        } else {
            Some(format!("open {}", RELEASE_GATE_DOC))
        },
    });

    checks.push(doc_checkbox_check(
        release_doc.as_deref(),
        "source-release-smoke-recorded",
        "source release smoke recorded",
        "本地 release build smoke",
        "cargo build --release --all-features",
    ));
    checks.push(doc_checkbox_check(
        release_doc.as_deref(),
        "ci-windows-smoke-recorded",
        "CI and Windows smoke recorded",
        "CI / Windows smoke",
        "gh pr checks",
    ));
    checks.push(doc_checkbox_check(
        release_doc.as_deref(),
        "wechat-regression-recorded",
        "real WeChat regression recorded",
        "真实微信路径人工回归",
        "moonpub wechat-health",
    ));
    checks.push(evidence_check(&evidence));
    checks.push(doc_checkbox_check(
        release_doc.as_deref(),
        "docs-consistency-recorded",
        "README and guide consistency recorded",
        "README / README_zh / USER_GUIDE / PROGRESS",
        "review README.md README_zh.md docs/USER_GUIDE.md PROGRESS.md",
    ));
    checks.push(doc_checkbox_check(
        release_doc.as_deref(),
        "secret-review-recorded",
        "secret and privacy review recorded",
        "没有真实凭据",
        "git status --short",
    ));

    let passed = checks
        .iter()
        .all(|check| check.status == ReleaseCheckStatus::Pass);
    let next_command = checks
        .iter()
        .find(|check| check.status == ReleaseCheckStatus::Fail)
        .and_then(|check| check.next_command.clone())
        .unwrap_or_else(|| "prepare v0.4.2 release notes".to_owned());
    let next_step = if passed {
        "all recorded release gates passed; do a final human review before publishing v0.4.2"
    } else {
        "complete the first failing v0.4.2 release gate before preparing release assets"
    };

    Ok(ReleaseCheckReport {
        release_version: env!("CARGO_PKG_VERSION"),
        repo_root: repo_root.to_path_buf(),
        passed,
        checks,
        next_step,
        next_command,
    })
}

fn evidence_check(evidence: &EvidenceReport) -> ReleaseCheckItem {
    ReleaseCheckItem {
        id: "release-evidence-files",
        title: "required evidence files present",
        status: pass_if(evidence.passed),
        detail: format!(
            "{}/{} present, {} missing",
            evidence.present_count, evidence.required_count, evidence.missing_count
        ),
        next_command: if evidence.passed {
            None
        } else {
            Some("moonpub evidence-status --json".to_owned())
        },
    }
}

fn doc_checkbox_check(
    release_doc: Option<&str>,
    id: &'static str,
    title: &'static str,
    marker: &str,
    next_command: &str,
) -> ReleaseCheckItem {
    let checked = release_doc.is_some_and(|doc| checked_line_contains(doc, marker));
    ReleaseCheckItem {
        id,
        title,
        status: pass_if(checked),
        detail: if checked {
            format!("recorded in {}", RELEASE_GATE_DOC)
        } else if release_doc.is_some() {
            format!("not checked in {}", RELEASE_GATE_DOC)
        } else {
            format!("cannot inspect missing {}", RELEASE_GATE_DOC)
        },
        next_command: if checked {
            None
        } else {
            Some(next_command.to_owned())
        },
    }
}

fn checked_line_contains(doc: &str, marker: &str) -> bool {
    doc.lines()
        .any(|line| line.trim_start().starts_with("- [x]") && line.contains(marker))
}

fn pass_if(value: bool) -> ReleaseCheckStatus {
    if value {
        ReleaseCheckStatus::Pass
    } else {
        ReleaseCheckStatus::Fail
    }
}

fn find_repo_root(start: &Path) -> PathBuf {
    let mut cur = Some(start);
    while let Some(dir) = cur {
        if dir.join(RELEASE_GATE_DOC).is_file() {
            return dir.to_path_buf();
        }
        cur = dir.parent();
    }
    start.to_path_buf()
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::{create_file, temp_root};

    use super::{ReleaseCheckStatus, release_check_from};

    #[test]
    fn release_check_reports_unfinished_gates() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("release-check-missing")?;
        create_file(
            &root.join("docs/RELEASE_GATE_v0.4.2_ZH.md"),
            "- [x] 本地 release build smoke 通过\n- [ ] 真实微信路径人工回归通过或失败原因已记录\n",
        )?;

        let report = release_check_from(&root)?;

        assert!(!report.passed);
        assert_eq!(report.checks[0].status, ReleaseCheckStatus::Pass);
        assert!(
            report
                .checks
                .iter()
                .any(|check| check.id == "release-evidence-files"
                    && check.status == ReleaseCheckStatus::Fail)
        );
        assert_eq!(report.next_command, "gh pr checks");

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn release_check_passes_when_doc_and_evidence_are_complete()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("release-check-complete")?;
        create_file(
            &root.join("docs/RELEASE_GATE_v0.4.2_ZH.md"),
            "- [x] 本地 release build smoke 通过\n\
             - [x] CI / Windows smoke 通过\n\
             - [x] 真实微信路径人工回归通过或失败原因已记录\n\
             - [x] README / README_zh / USER_GUIDE / PROGRESS 与 release 事实一致\n\
             - [x] 没有真实凭据、token、二维码或隐私截图被提交\n",
        )?;
        for path in [
            "homepage/homepage-workspace.png",
            "homepage/homepage-context.png",
            "feishu/feishu-home-entry.png",
            "feishu/feishu-result-modal.png",
            "feishu/feishu-draft-opened.png",
            "photos/photos-image-opened.png",
            "photos/photos-result-modal.png",
            "photos/photos-draft-opened.png",
            "wechat/wechat-draft-created.png",
            "wechat/configure-headed.png",
            "wechat/preview-sent.png",
        ] {
            create_file(
                &root.join("docs/first-run-evidence").join(path),
                "redacted screenshot placeholder",
            )?;
        }

        let report = release_check_from(&root)?;

        assert!(report.passed);
        assert_eq!(report.next_command, "prepare v0.4.2 release notes");
        assert!(
            report
                .checks
                .iter()
                .all(|check| check.status == ReleaseCheckStatus::Pass)
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }
}
