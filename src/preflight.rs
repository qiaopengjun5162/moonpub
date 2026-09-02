use std::path::{Path, PathBuf};

use crate::article::{
    extract_title_from_body, parse_frontmatter, resolve_article_path, wechat_title,
};
use crate::bundle::ArticleBundle;
use crate::error::AppError;
use crate::layout_audit::audit_html_file;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightCheck {
    pub id: &'static str,
    pub status: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightReport {
    pub article_path: PathBuf,
    pub html_path: PathBuf,
    pub draft_json_path: PathBuf,
    pub media_id_path: PathBuf,
    pub passed: bool,
    pub checks: Vec<PreflightCheck>,
    pub next_command: String,
    pub next_step: &'static str,
}

pub fn preflight_article(articles_dir: &Path, article: &Path) -> Result<PreflightReport, AppError> {
    let article = resolve_article_path(articles_dir, article);
    let bundle = ArticleBundle::from_markdown(&article)?;
    let mut checks = Vec::new();

    push_check(
        &mut checks,
        "markdown",
        bundle.has_markdown(),
        format!("markdown exists: {}", bundle.markdown_path().display()),
        format!("markdown missing: {}", bundle.markdown_path().display()),
    );
    push_check(
        &mut checks,
        "html",
        bundle.has_html(),
        format!("rendered HTML exists: {}", bundle.html_path().display()),
        format!("rendered HTML missing: {}", bundle.html_path().display()),
    );
    push_check(
        &mut checks,
        "draft_json",
        bundle.has_draft_json(),
        format!("draft JSON exists: {}", bundle.draft_json_path().display()),
        format!("draft JSON missing: {}", bundle.draft_json_path().display()),
    );

    if bundle.has_html() {
        let audit = audit_html_file(bundle.html_path())?;
        if audit.passed {
            checks.push(PreflightCheck {
                id: "layout_audit",
                status: if audit.warnings.is_empty() {
                    "pass"
                } else {
                    "warn"
                },
                message: if audit.warnings.is_empty() {
                    "layout audit passed".to_owned()
                } else {
                    format!(
                        "layout audit passed with warnings: {}",
                        audit.warnings.join("; ")
                    )
                },
            });
        } else {
            checks.push(PreflightCheck {
                id: "layout_audit",
                status: "fail",
                message: format!("layout audit failed: {}", audit.errors.join("; ")),
            });
        }
    } else {
        checks.push(PreflightCheck {
            id: "layout_audit",
            status: "skip",
            message: "rendered HTML missing; run render before layout audit".to_owned(),
        });
    }

    checks.push(PreflightCheck {
        id: "media_id",
        status: if bundle.has_media_id() {
            "pass"
        } else {
            "warn"
        },
        message: if bundle.has_media_id() {
            format!("media_id exists: {}", bundle.media_id_path().display())
        } else {
            "media_id missing; article has not been pushed to WeChat draft yet".to_owned()
        },
    });

    // WeChat hard limits (ported from wx-cli scripts/check-public-draft-style.mjs):
    // title must exist and fit 64 chars, digest (when set) must fit 120 chars,
    // and the body H1 should match the authored title. Limits are character-based
    // (not UTF-8 bytes) so Chinese titles are not false-flagged.
    let markdown = bundle
        .has_markdown()
        .then(|| std::fs::read_to_string(bundle.markdown_path()))
        .and_then(Result::ok);
    if let Some(md) = markdown {
        let front = parse_frontmatter(&md);
        let effective_title = wechat_title(&front, &md);
        let title_chars = effective_title.chars().count();

        if effective_title.trim().is_empty() {
            checks.push(PreflightCheck {
                    id: "title_required",
                    status: "fail",
                    message: "title missing; WeChat requires a title (set frontmatter `title` or `wechat_title`)"
                        .to_owned(),
                });
        } else if title_chars > 64 {
            checks.push(PreflightCheck {
                id: "title_limit",
                status: "fail",
                message: format!(
                    "title exceeds WeChat 64-char limit ({} chars): {}",
                    title_chars, effective_title
                ),
            });
        } else {
            checks.push(PreflightCheck {
                id: "title_limit",
                status: "pass",
                message: format!("title within 64-char limit ({} chars)", title_chars),
            });
        }

        match &front.digest {
            Some(d) if !d.trim().is_empty() => {
                let n = d.chars().count();
                if n > 120 {
                    checks.push(PreflightCheck {
                        id: "digest_limit",
                        status: "fail",
                        message: format!("digest exceeds WeChat 120-char limit ({} chars)", n),
                    });
                } else {
                    checks.push(PreflightCheck {
                        id: "digest_limit",
                        status: "pass",
                        message: format!("digest within 120-char limit ({} chars)", n),
                    });
                }
            }
            _ => checks.push(PreflightCheck {
                id: "digest_limit",
                status: "warn",
                message: "digest empty; WeChat will auto-extract from content".to_owned(),
            }),
        }

        if let (Some(h1), Some(authored)) = (extract_title_from_body(&md), &front.title)
            && !authored.trim().is_empty()
            && h1.trim() != authored.trim()
        {
            checks.push(PreflightCheck {
                id: "title_h1_match",
                status: "warn",
                message: format!(
                    "body H1 `{}` differs from frontmatter title `{}`",
                    h1.trim(),
                    authored.trim()
                ),
            });
        }

        // Image link integrity (ported from wx-cli scripts/check-media-links.mjs):
        // local image references in the body must resolve on disk, otherwise the
        // WeChat upload silently drops the image. Orphan images named after this
        // article's slug but not referenced are reported as warnings only.
        let md_dir = bundle
            .markdown_path()
            .parent()
            .unwrap_or_else(|| Path::new("."));
        let slug = bundle
            .markdown_path()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let image_refs = extract_local_image_refs(&md, md_dir);
        let missing_images: Vec<&PathBuf> = image_refs.iter().filter(|p| !p.exists()).collect();
        if missing_images.is_empty() {
            checks.push(PreflightCheck {
                id: "image_links",
                status: "pass",
                message: format!(
                    "all {} local image reference(s) resolve on disk",
                    image_refs.len()
                ),
            });
        } else {
            let listed = missing_images
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join("; ");
            checks.push(PreflightCheck {
                id: "image_links",
                status: "fail",
                message: format!("{} broken image link(s): {}", missing_images.len(), listed),
            });
        }

        let orphan_images = find_orphan_images(md_dir, slug, &image_refs, front.cover.as_deref());
        if !orphan_images.is_empty() {
            let listed = orphan_images
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join("; ");
            checks.push(PreflightCheck {
                id: "orphan_images",
                status: "warn",
                message: format!(
                    "{} orphan image(s) not referenced by the article: {}",
                    orphan_images.len(),
                    listed
                ),
            });
        }
    }

    let passed = checks.iter().all(|check| check.status != "fail");
    let (next_command, next_step) = if !bundle.has_html() || !bundle.has_draft_json() {
        (
            format!("moonpub render {}", bundle.markdown_path().display()),
            "render the article before preview or WeChat draft push",
        )
    } else if checks
        .iter()
        .any(|check| check.id == "layout_audit" && check.status == "fail")
    {
        (
            format!("moonpub layout-audit {}", bundle.html_path().display()),
            "fix the rendered HTML compatibility issues before publishing",
        )
    } else if !bundle.has_media_id() {
        (
            format!("moonpub push {} --render", bundle.markdown_path().display()),
            "review local preview, then explicitly push to WeChat draft when ready",
        )
    } else {
        (
            "moonpub wechat-health".to_owned(),
            "check browser automation login before backend preview-send",
        )
    };

    Ok(PreflightReport {
        article_path: bundle.markdown_path().to_path_buf(),
        html_path: bundle.html_path().to_path_buf(),
        draft_json_path: bundle.draft_json_path().to_path_buf(),
        media_id_path: bundle.media_id_path().to_path_buf(),
        passed,
        checks,
        next_command,
        next_step,
    })
}

fn push_check(
    checks: &mut Vec<PreflightCheck>,
    id: &'static str,
    ok: bool,
    ok_message: String,
    fail_message: String,
) {
    checks.push(PreflightCheck {
        id,
        status: if ok { "pass" } else { "fail" },
        message: if ok { ok_message } else { fail_message },
    });
}

/// Image extensions considered when scanning for orphan assets.
const IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp"];

/// Extract local (non-external) image references from a markdown body, resolving
/// relative paths against the markdown's parent directory. Handles both the
/// standard `![alt](url)` syntax and Obsidian wiki embeds `![[path]]`.
fn extract_local_image_refs(md: &str, md_dir: &Path) -> Vec<PathBuf> {
    let mut refs = Vec::new();
    let bytes = md.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'!' && bytes[i + 1] == b'[' {
            if i + 2 < bytes.len() && bytes[i + 2] == b'[' {
                // Obsidian wiki embed: ![[path|alias#heading]]
                if let Some(end) = md[i + 3..].find("]]") {
                    let inner = &md[i + 3..i + 3 + end];
                    let raw = inner.split(['|', '#']).next().unwrap_or("").trim();
                    if !raw.is_empty() && !is_external(raw) {
                        refs.push(resolve_local(md_dir, raw));
                    }
                    i = i + 3 + end + 2;
                    continue;
                }
            } else if let Some(paren) = md[i + 2..].find("](") {
                // Markdown image: ![alt](url)
                let start = i + 2 + paren + 2;
                if let Some(close) = md[start..].find(')') {
                    let url = md[start..start + close].trim();
                    if !url.is_empty() && !is_external(url) && !url.contains(char::is_whitespace) {
                        refs.push(resolve_local(md_dir, url));
                    }
                    i = start + close + 1;
                    continue;
                }
            }
        }
        i += 1;
    }
    refs
}

fn is_external(raw: &str) -> bool {
    let lower = raw.to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("data:")
        || lower.starts_with("mailto:")
        || lower.starts_with("ftp://")
}

fn resolve_local(md_dir: &Path, raw: &str) -> PathBuf {
    let p = Path::new(raw);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        md_dir.join(raw)
    }
}

/// Find image files in the article directory named after this article's slug but
/// not referenced by the body or cover. Restricting to slug-prefixed names avoids
/// false positives from sibling articles sharing a `drafts`/`ready` directory.
fn find_orphan_images(
    md_dir: &Path,
    slug: &str,
    referenced: &[PathBuf],
    cover: Option<&str>,
) -> Vec<PathBuf> {
    let Ok(dir) = md_dir.read_dir() else {
        return Vec::new();
    };
    let cover_path = cover.map(|c| resolve_local(md_dir, c));
    let mut orphans = Vec::new();
    for entry in dir.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !IMAGE_EXTS.contains(&ext.as_str()) {
            continue;
        }
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name == format!("{slug}.cover.png") {
            continue;
        }
        if referenced.iter().any(|r| r == &path) {
            continue;
        }
        if let Some(cp) = &cover_path
            && cp == &path
        {
            continue;
        }
        if !slug.is_empty() && name.starts_with(slug) {
            orphans.push(path);
        }
    }
    orphans
}

#[cfg(test)]
mod tests {
    use crate::preflight::preflight_article;
    use crate::test_helpers::{create_file, temp_root};

    #[test]
    fn preflight_reports_missing_render_outputs() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("preflight-missing-render")?;
        let article = root.join("Articles/drafts/demo.md");
        create_file(&article, "# demo")?;

        let report = preflight_article(&root, &article)?;

        assert!(!report.passed);
        assert!(
            report
                .checks
                .iter()
                .any(|check| check.id == "html" && check.status == "fail")
        );
        assert_eq!(
            report.next_command,
            format!("moonpub render {}", article.display())
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn preflight_passes_local_ready_bundle_without_media_id()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("preflight-local-ready")?;
        let article = root.join("Articles/drafts/demo.md");
        create_file(&article, "# demo")?;
        create_file(
            &root.join("Articles/drafts/demo.html"),
            r#"<section style="margin:0;"><p style="color:#333;">正文</p></section>"#,
        )?;
        create_file(&root.join("Articles/drafts/demo.draft.json"), "{}")?;

        let report = preflight_article(&root, &article)?;

        assert!(report.passed);
        assert!(
            report
                .checks
                .iter()
                .any(|check| check.id == "media_id" && check.status == "warn")
        );
        assert_eq!(
            report.next_command,
            format!("moonpub push {} --render", article.display())
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn preflight_flags_overlong_wechat_title() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("preflight-title-limit")?;
        let article = root.join("Articles/drafts/demo.md");
        let long_title = "测".repeat(70);
        create_file(
            &article,
            &format!(
                "---\ntitle: \"{}\"\ndigest: 摘要\n---\n\n# {}\n\n正文\n",
                long_title, long_title
            ),
        )?;

        let report = preflight_article(&root, &article)?;

        assert!(
            report
                .checks
                .iter()
                .any(|c| c.id == "title_limit" && c.status == "fail"),
            "overlong title should fail the WeChat 64-char limit"
        );
        assert!(!report.passed);

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn preflight_passes_valid_title_and_digest() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("preflight-valid-limits")?;
        let article = root.join("Articles/drafts/demo.md");
        create_file(
            &article,
            "---\ntitle: \"做公众号必备的 4 个工具站\"\ndigest: 从查违规词到抓素材，这份清单覆盖做号最核心的需求。\n---\n\n# 做公众号必备的 4 个工具站\n\n正文\n",
        )?;

        let report = preflight_article(&root, &article)?;

        assert!(
            report
                .checks
                .iter()
                .any(|c| c.id == "title_limit" && c.status == "pass"),
            "valid title should pass"
        );
        assert!(
            report
                .checks
                .iter()
                .any(|c| c.id == "digest_limit" && c.status == "pass"),
            "valid digest should pass"
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn preflight_flags_broken_image_link() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("preflight-broken-image")?;
        let article = root.join("Articles/drafts/demo.md");
        create_file(
            &article,
            "---\ntitle: \"标题\"\ndigest: 摘要\n---\n\n# 标题\n\n![图](./missing.png)\n",
        )?;

        let report = preflight_article(&root, &article)?;

        assert!(!report.passed);
        assert!(
            report
                .checks
                .iter()
                .any(|c| c.id == "image_links" && c.status == "fail"),
            "broken image link should fail preflight"
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn preflight_passes_resolved_image_reference() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("preflight-resolved-image")?;
        let article = root.join("Articles/drafts/demo.md");
        create_file(
            &article,
            "---\ntitle: \"标题\"\ndigest: 摘要\n---\n\n# 标题\n\n![图](./ok.png)\n",
        )?;
        create_file(&root.join("Articles/drafts/ok.png"), "png-bytes")?;

        let report = preflight_article(&root, &article)?;

        assert!(
            report
                .checks
                .iter()
                .any(|c| c.id == "image_links" && c.status == "pass"),
            "resolved image reference should pass"
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn preflight_warns_on_orphan_image() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("preflight-orphan-image")?;
        let article = root.join("Articles/drafts/demo.md");
        create_file(
            &article,
            "---\ntitle: \"标题\"\ndigest: 摘要\n---\n\n# 标题\n\n正文\n",
        )?;
        create_file(&root.join("Articles/drafts/demo-unused.png"), "png-bytes")?;

        let report = preflight_article(&root, &article)?;

        assert!(
            report
                .checks
                .iter()
                .any(|c| c.id == "orphan_images" && c.status == "warn"),
            "slug-prefixed unreferenced image should warn"
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }
}
