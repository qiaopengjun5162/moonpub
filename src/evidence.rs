use std::path::{Path, PathBuf};

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceReport {
    pub base_dir: PathBuf,
    pub passed: bool,
    pub sections: Vec<EvidenceSection>,
    pub next_step: &'static str,
    pub next_command: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceSection {
    pub id: &'static str,
    pub title: &'static str,
    pub items: Vec<EvidenceItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceItem {
    pub id: &'static str,
    pub path: PathBuf,
    pub exists: bool,
}

struct EvidenceSectionSpec {
    id: &'static str,
    title: &'static str,
    items: &'static [EvidenceItemSpec],
}

struct EvidenceItemSpec {
    id: &'static str,
    rel_path: &'static str,
}

const EVIDENCE_SECTIONS: &[EvidenceSectionSpec] = &[
    EvidenceSectionSpec {
        id: "homepage",
        title: "插件首页",
        items: &[
            EvidenceItemSpec {
                id: "homepage-workspace",
                rel_path: "homepage/homepage-workspace.png",
            },
            EvidenceItemSpec {
                id: "homepage-context",
                rel_path: "homepage/homepage-context.png",
            },
        ],
    },
    EvidenceSectionSpec {
        id: "feishu",
        title: "飞书首次体验",
        items: &[
            EvidenceItemSpec {
                id: "feishu-home-entry",
                rel_path: "feishu/feishu-home-entry.png",
            },
            EvidenceItemSpec {
                id: "feishu-result-modal",
                rel_path: "feishu/feishu-result-modal.png",
            },
            EvidenceItemSpec {
                id: "feishu-draft-opened",
                rel_path: "feishu/feishu-draft-opened.png",
            },
        ],
    },
    EvidenceSectionSpec {
        id: "photos",
        title: "照片首次体验",
        items: &[
            EvidenceItemSpec {
                id: "photos-image-opened",
                rel_path: "photos/photos-image-opened.png",
            },
            EvidenceItemSpec {
                id: "photos-result-modal",
                rel_path: "photos/photos-result-modal.png",
            },
            EvidenceItemSpec {
                id: "photos-draft-opened",
                rel_path: "photos/photos-draft-opened.png",
            },
        ],
    },
    EvidenceSectionSpec {
        id: "wechat",
        title: "真实微信回归",
        items: &[
            EvidenceItemSpec {
                id: "wechat-draft-created",
                rel_path: "wechat/wechat-draft-created.png",
            },
            EvidenceItemSpec {
                id: "configure-headed",
                rel_path: "wechat/configure-headed.png",
            },
            EvidenceItemSpec {
                id: "preview-sent",
                rel_path: "wechat/preview-sent.png",
            },
        ],
    },
];

pub fn evidence_status() -> Result<EvidenceReport, AppError> {
    let cwd = std::env::current_dir().map_err(|source| AppError::Io {
        path: PathBuf::from("."),
        source,
    })?;
    Ok(evidence_status_from(&find_evidence_dir(&cwd)))
}

pub fn evidence_status_from(base_dir: &Path) -> EvidenceReport {
    let sections = EVIDENCE_SECTIONS
        .iter()
        .map(|section| EvidenceSection {
            id: section.id,
            title: section.title,
            items: section
                .items
                .iter()
                .map(|item| {
                    let path = base_dir.join(item.rel_path);
                    EvidenceItem {
                        id: item.id,
                        exists: path.is_file(),
                        path,
                    }
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    let passed = sections
        .iter()
        .all(|section| section.items.iter().all(|item| item.exists));
    let next_step = if passed {
        "manually review screenshots for secrets before preparing v0.4.2 release"
    } else {
        "add missing redacted first-run and WeChat evidence files before v0.4.2 release"
    };
    EvidenceReport {
        base_dir: base_dir.to_path_buf(),
        passed,
        sections,
        next_step,
        next_command: "moonpub evidence-status --json",
    }
}

fn find_evidence_dir(start: &Path) -> PathBuf {
    let mut cur = Some(start);
    while let Some(dir) = cur {
        let candidate = dir.join("docs/first-run-evidence");
        if candidate.join("README.md").is_file() {
            return candidate;
        }
        cur = dir.parent();
    }
    start.join("docs/first-run-evidence")
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::{create_file, temp_root};

    use super::evidence_status_from;

    #[test]
    fn evidence_status_reports_missing_required_files() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("evidence-missing")?;
        let report = evidence_status_from(&root);

        assert!(!report.passed);
        assert_eq!(report.sections.len(), 4);
        assert!(report.sections[0].items.iter().all(|item| !item.exists));
        assert!(
            report
                .next_step
                .contains("add missing redacted first-run and WeChat evidence files")
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn evidence_status_passes_when_all_required_files_exist()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("evidence-complete")?;
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
            create_file(&root.join(path), "redacted screenshot placeholder")?;
        }

        let report = evidence_status_from(&root);

        assert!(report.passed);
        assert!(
            report
                .sections
                .iter()
                .all(|section| section.items.iter().all(|item| item.exists))
        );
        assert!(report.next_step.contains("manually review screenshots"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }
}
