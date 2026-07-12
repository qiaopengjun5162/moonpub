use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::AppError;

mod photos;

pub use photos::{intake_photos, vision_photo_paths};

const FEISHU_SOURCE: &str = "feishu-minutes";
const INBOX_STATUS: &str = "inbox";
const VOICE_NOTE_TYPE: &str = "voice-note";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntakeAction {
    Created,
    Updated,
}

impl IntakeAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Updated => "updated",
        }
    }
}

pub struct IntakeOutput {
    pub path: PathBuf,
    pub action: IntakeAction,
    pub message: String,
}

pub fn intake_feishu(articles_dir: &Path, input: &Path) -> Result<IntakeOutput, AppError> {
    let raw = fs::read_to_string(input).map_err(|source| AppError::Io {
        path: input.to_path_buf(),
        source,
    })?;
    let title = feishu_title(&raw, input);
    let minutes = FeishuMinutes {
        title,
        transcript: raw,
        original_file: Some(input.display().to_string()),
        minute_token: None,
        source_url: None,
    };
    write_feishu_minutes(articles_dir, &minutes)
}

pub fn intake_feishu_minute_token(
    articles_dir: &Path,
    token: &str,
) -> Result<IntakeOutput, AppError> {
    let detail = fetch_feishu_minutes_detail(articles_dir, token)?;
    let transcript_path = resolve_transcript_path(articles_dir, &detail.transcript_file);
    let transcript = fs::read_to_string(&transcript_path).map_err(|source| AppError::Io {
        path: transcript_path.clone(),
        source,
    })?;
    let minutes = FeishuMinutes {
        title: detail.title,
        transcript,
        original_file: Some(transcript_path.display().to_string()),
        minute_token: Some(token.to_owned()),
        source_url: detail.source_url,
    };
    write_feishu_minutes(articles_dir, &minutes)
}

pub fn intake_feishu_latest(articles_dir: &Path) -> Result<IntakeOutput, AppError> {
    let hit = search_feishu_minutes(None)?;
    let mut output = intake_feishu_minute_token(articles_dir, &hit.token)?;
    output.message.push_str(&format!(
        "\n  source: latest Feishu Minutes ({})",
        hit.title
    ));
    Ok(output)
}

pub fn intake_feishu_query(articles_dir: &Path, query: &str) -> Result<IntakeOutput, AppError> {
    let hit = search_feishu_minutes(Some(query))?;
    let mut output = intake_feishu_minute_token(articles_dir, &hit.token)?;
    output.message.push_str(&format!(
        "\n  source: Feishu Minutes query \"{}\" ({})",
        query, hit.title
    ));
    Ok(output)
}

struct FeishuMinutes {
    title: String,
    transcript: String,
    original_file: Option<String>,
    minute_token: Option<String>,
    source_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InboxMetadata {
    source: String,
    status: String,
    created: String,
    content_type: String,
    external_id: Option<String>,
    source_url: Option<String>,
    original_file: Option<String>,
    captured_at: Option<String>,
    source_title: Option<String>,
    minute_token: Option<String>,
}

impl InboxMetadata {
    fn from_feishu_minutes(created: String, minutes: &FeishuMinutes) -> Self {
        Self {
            source: FEISHU_SOURCE.to_owned(),
            status: INBOX_STATUS.to_owned(),
            created,
            content_type: VOICE_NOTE_TYPE.to_owned(),
            external_id: minutes.minute_token.clone(),
            source_url: minutes.source_url.clone(),
            original_file: minutes.original_file.clone(),
            captured_at: None,
            source_title: Some(minutes.title.clone()),
            minute_token: minutes.minute_token.clone(),
        }
    }

    fn to_frontmatter(&self) -> String {
        let mut lines = vec![
            "---".to_owned(),
            format!("source: {}", self.source),
            format!("status: {}", self.status),
            format!("created: {}", self.created),
            format!("type: {}", self.content_type),
        ];
        push_optional_frontmatter(&mut lines, "external_id", self.external_id.as_deref());
        push_optional_frontmatter(&mut lines, "source_url", self.source_url.as_deref());
        push_optional_frontmatter(&mut lines, "original_file", self.original_file.as_deref());
        push_optional_frontmatter(&mut lines, "captured_at", self.captured_at.as_deref());
        push_optional_frontmatter(&mut lines, "source_title", self.source_title.as_deref());
        push_optional_frontmatter(&mut lines, "minute_token", self.minute_token.as_deref());
        lines.push("---".to_owned());
        lines.join("\n")
    }

    fn from_content(content: &str) -> Option<Self> {
        Some(Self {
            source: frontmatter_value(content, "source")?,
            status: frontmatter_value(content, "status")?,
            created: frontmatter_value(content, "created")?,
            content_type: frontmatter_value(content, "type")?,
            external_id: frontmatter_value(content, "external_id"),
            source_url: frontmatter_value(content, "source_url"),
            original_file: frontmatter_value(content, "original_file"),
            captured_at: frontmatter_value(content, "captured_at"),
            source_title: frontmatter_value(content, "source_title"),
            minute_token: frontmatter_value(content, "minute_token"),
        })
    }
}

fn write_feishu_minutes(
    articles_dir: &Path,
    minutes: &FeishuMinutes,
) -> Result<IntakeOutput, AppError> {
    let date = today_utc();
    let slug = slugify(&minutes.title);
    let inbox_dir = articles_dir.join("Inbox/Feishu");
    fs::create_dir_all(&inbox_dir).map_err(|source| AppError::Io {
        path: inbox_dir.clone(),
        source,
    })?;
    let metadata = InboxMetadata::from_feishu_minutes(date.clone(), minutes);
    let output = if let Some(external_id) = metadata.external_id.as_deref() {
        find_existing_inbox_by_external_id(&inbox_dir, external_id)?
            .unwrap_or_else(|| inbox_dir.join(format!("{date}-{slug}.md")))
    } else {
        inbox_dir.join(format!("{date}-{slug}.md"))
    };
    let action = if output.exists() {
        IntakeAction::Updated
    } else {
        IntakeAction::Created
    };
    let frontmatter = metadata.to_frontmatter();
    let content = format!(
        "{frontmatter}\n\n# {}\n\n## 原始转写\n\n{}\n",
        minutes.title,
        minutes.transcript.trim()
    );
    fs::write(&output, content).map_err(|source| AppError::Io {
        path: output.clone(),
        source,
    })?;

    Ok(IntakeOutput {
        message: format!("intake {}\n  {}", action.as_str(), output.display()),
        action,
        path: output,
    })
}

fn find_existing_inbox_by_external_id(
    inbox_dir: &Path,
    external_id: &str,
) -> Result<Option<PathBuf>, AppError> {
    let entries = fs::read_dir(inbox_dir).map_err(|source| AppError::Io {
        path: inbox_dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| AppError::Io {
            path: inbox_dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let content = fs::read_to_string(&path).map_err(|source| AppError::Io {
            path: path.clone(),
            source,
        })?;
        let metadata = match InboxMetadata::from_content(&content) {
            Some(metadata) => metadata,
            None => continue,
        };
        let existing_id = metadata
            .external_id
            .as_deref()
            .or(metadata.minute_token.as_deref());
        if existing_id == Some(external_id) {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

fn frontmatter_value(content: &str, key: &str) -> Option<String> {
    if !content.starts_with("---\n") {
        return None;
    }
    content
        .lines()
        .skip(1)
        .take_while(|line| line.trim() != "---")
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.trim() != key {
                return None;
            }
            let trimmed = value.trim().trim_matches('"');
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_owned())
            }
        })
}

struct FeishuMinutesDetail {
    title: String,
    transcript_file: std::path::PathBuf,
    source_url: Option<String>,
}

struct FeishuMinutesSearchHit {
    token: String,
    title: String,
}

fn search_feishu_minutes(query: Option<&str>) -> Result<FeishuMinutesSearchHit, AppError> {
    let mut command = Command::new("lark-cli");
    command.args(lark_minutes_search_args(query));
    let output = command.output().map_err(|source| AppError::Io {
        path: std::path::PathBuf::from("lark-cli"),
        source,
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        let message = if stderr.is_empty() { stdout } else { stderr };
        return Err(AppError::PushFailed {
            message: format!("lark-cli minutes +search failed: {message}"),
            ip_hint: None,
        });
    }

    parse_feishu_minutes_search(&output.stdout)
}

fn lark_minutes_search_args(query: Option<&str>) -> Vec<String> {
    let mut args = vec![
        "minutes".to_owned(),
        "+search".to_owned(),
        "--as".to_owned(),
        "user".to_owned(),
        "--page-size".to_owned(),
        "1".to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
    ];
    if let Some(query) = query {
        args.extend(["--query".to_owned(), query.to_owned()]);
    } else {
        args.extend(["--owner-ids".to_owned(), "me".to_owned()]);
    }
    args
}

fn fetch_feishu_minutes_detail(
    articles_dir: &Path,
    token: &str,
) -> Result<FeishuMinutesDetail, AppError> {
    let output_dir = articles_dir.join(".moonpub/feishu-minutes");
    fs::create_dir_all(&output_dir).map_err(|source| AppError::Io {
        path: output_dir,
        source,
    })?;
    let output = Command::new("lark-cli")
        .current_dir(articles_dir)
        .args(lark_minutes_detail_args(token))
        .output()
        .map_err(|source| AppError::Io {
            path: std::path::PathBuf::from("lark-cli"),
            source,
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        let message = if stderr.is_empty() { stdout } else { stderr };
        return Err(AppError::PushFailed {
            message: format!("lark-cli minutes +detail failed: {message}"),
            ip_hint: None,
        });
    }

    parse_feishu_minutes_detail(&output.stdout)
}

fn lark_minutes_detail_args(token: &str) -> Vec<String> {
    vec![
        "minutes".to_owned(),
        "+detail".to_owned(),
        "--as".to_owned(),
        "user".to_owned(),
        "--minute-tokens".to_owned(),
        token.to_owned(),
        "--transcript".to_owned(),
        "--overwrite".to_owned(),
        "--output-dir".to_owned(),
        ".moonpub/feishu-minutes".to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
    ]
}

fn parse_feishu_minutes_detail(bytes: &[u8]) -> Result<FeishuMinutesDetail, AppError> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|err| AppError::PushFailed {
            message: format!("invalid lark-cli minutes +detail json: {err}"),
            ip_hint: None,
        })?;
    let item = value
        .get("data")
        .and_then(|data| data.get("minutes"))
        .and_then(|minutes| minutes.as_array())
        .and_then(|minutes| minutes.first())
        .ok_or_else(|| AppError::PushFailed {
            message: "lark-cli minutes +detail returned no minutes".to_owned(),
            ip_hint: None,
        })?;
    let title = item
        .get("title")
        .and_then(|title| title.as_str())
        .filter(|title| !title.trim().is_empty())
        .unwrap_or("未命名飞书秒记")
        .to_owned();
    let transcript_file = item
        .get("artifacts")
        .and_then(|artifacts| artifacts.get("transcript_file"))
        .and_then(|path| path.as_str())
        .filter(|path| !path.trim().is_empty())
        .ok_or_else(|| AppError::PushFailed {
            message: "lark-cli minutes +detail returned no transcript_file".to_owned(),
            ip_hint: None,
        })?;
    let source_url = item
        .get("url")
        .or_else(|| item.get("app_link"))
        .and_then(|url| url.as_str())
        .filter(|url| !url.trim().is_empty())
        .map(str::to_owned);

    Ok(FeishuMinutesDetail {
        title,
        transcript_file: std::path::PathBuf::from(transcript_file),
        source_url,
    })
}

fn parse_feishu_minutes_search(bytes: &[u8]) -> Result<FeishuMinutesSearchHit, AppError> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|err| AppError::PushFailed {
            message: format!("invalid lark-cli minutes +search json: {err}"),
            ip_hint: None,
        })?;
    let item = value
        .get("data")
        .and_then(|data| data.get("items"))
        .and_then(|items| items.as_array())
        .and_then(|items| items.first())
        .ok_or_else(|| AppError::PushFailed {
            message: "lark-cli minutes +search returned no minutes".to_owned(),
            ip_hint: None,
        })?;
    let token = item
        .get("token")
        .and_then(|token| token.as_str())
        .filter(|token| !token.trim().is_empty())
        .ok_or_else(|| AppError::PushFailed {
            message: "lark-cli minutes +search returned no token".to_owned(),
            ip_hint: None,
        })?
        .to_owned();
    let title = item
        .get("display_info")
        .and_then(|display| display.as_str())
        .and_then(first_display_line)
        .map(strip_simple_tags)
        .unwrap_or("未命名飞书秒记")
        .to_owned();

    Ok(FeishuMinutesSearchHit { token, title })
}

fn first_display_line(display_info: &str) -> Option<&str> {
    display_info
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
}

fn strip_simple_tags(value: &str) -> &str {
    value
        .strip_prefix("<h>")
        .and_then(|value| value.strip_suffix("</h>"))
        .unwrap_or(value)
}

fn feishu_title(raw: &str, input: &Path) -> String {
    raw.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(trim_heading_marker)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .or_else(|| {
            input
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "未命名飞书秒记".to_owned())
}

fn trim_heading_marker(line: &str) -> &str {
    line.trim_start_matches('#').trim()
}

fn slugify(title: &str) -> String {
    let mut slug = String::new();
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if ch.is_whitespace() || ch == '-' || ch == '_' {
            if !slug.ends_with('-') {
                slug.push('-');
            }
        } else if ('\u{4e00}'..='\u{9fff}').contains(&ch) {
            slug.push(ch);
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "feishu-minutes".to_owned()
    } else {
        slug.to_owned()
    }
}

fn yaml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn push_optional_frontmatter(lines: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        lines.push(format!("{key}: \"{}\"", yaml_escape(value)));
    }
}

fn resolve_transcript_path(articles_dir: &Path, transcript_file: &Path) -> std::path::PathBuf {
    if transcript_file.is_absolute() {
        transcript_file.to_path_buf()
    } else {
        articles_dir.join(transcript_file)
    }
}

fn today_utc() -> String {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() / 86_400)
        .unwrap_or(0);
    civil_from_days(days as i64)
}

fn civil_from_days(days_since_epoch: i64) -> String {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    format!("{year:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use crate::intake::{
        FeishuMinutes, InboxMetadata, IntakeAction, civil_from_days, intake_feishu, intake_photos,
        lark_minutes_detail_args, lark_minutes_search_args, parse_feishu_minutes_detail,
        parse_feishu_minutes_search, resolve_transcript_path, slugify, vision_photo_paths,
        write_feishu_minutes,
    };
    use crate::test_helpers::{create_file, temp_root};

    #[test]
    fn intake_feishu_writes_raw_minutes_to_obsidian_inbox() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = temp_root("intake-feishu")?;
        let input = root.join("exports/morning.txt");
        create_file(&input, "早上散步想到的事情\n\n今天跑完步以后，想记录一下。")?;

        let intake = intake_feishu(&root, &input)?;
        let date = civil_from_days(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64
                / 86_400,
        );
        let expected = root.join(format!("Inbox/Feishu/{date}-早上散步想到的事情.md"));
        let content = std::fs::read_to_string(&expected)?;

        assert_eq!(intake.path, expected);
        assert!(intake.message.contains(expected.to_string_lossy().as_ref()));
        assert!(content.contains("source: feishu-minutes"));
        assert!(content.contains("status: inbox"));
        assert!(content.contains("type: voice-note"));
        assert!(content.contains("# 早上散步想到的事情"));
        assert!(content.contains("今天跑完步以后，想记录一下。"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn slugify_keeps_chinese_and_normalizes_ascii() {
        assert_eq!(slugify("Run Notes 01"), "run-notes-01");
        assert_eq!(slugify("早上散步 想法"), "早上散步-想法");
    }

    #[test]
    fn parse_feishu_minutes_detail_reads_transcript_path() -> Result<(), Box<dyn std::error::Error>>
    {
        let json = r#"{
          "ok": true,
          "data": {
            "minutes": [{
              "title": "散步录音",
              "url": "https://example.feishu.cn/minutes/obcn123",
              "artifacts": {
                "transcript_file": "minutes/obcn123/transcript.txt"
              }
            }]
          }
        }"#;

        let detail = parse_feishu_minutes_detail(json.as_bytes())?;

        assert_eq!(detail.title, "散步录音");
        assert_eq!(
            detail.transcript_file,
            std::path::PathBuf::from("minutes/obcn123/transcript.txt")
        );
        assert_eq!(
            detail.source_url.as_deref(),
            Some("https://example.feishu.cn/minutes/obcn123")
        );
        Ok(())
    }

    #[test]
    fn parse_feishu_minutes_search_reads_first_token() -> Result<(), Box<dyn std::error::Error>> {
        let json = r#"{
          "ok": true,
          "data": {
            "items": [{
              "display_info": "<h>新录音</h>\n\n所有者: 用户428714 开始时间: 2026.06.25 18:40:22 时长: 9 秒",
              "token": "obcn123"
            }]
          }
        }"#;

        let hit = parse_feishu_minutes_search(json.as_bytes())?;

        assert_eq!(hit.token, "obcn123");
        assert_eq!(hit.title, "新录音");
        Ok(())
    }

    #[test]
    fn lark_minutes_search_uses_user_identity() {
        let latest_args = lark_minutes_search_args(None);
        let query_args = lark_minutes_search_args(Some("散步"));

        assert_eq!(&latest_args[..2], ["minutes", "+search"]);
        assert_eq!(&latest_args[2..4], ["--as", "user"]);
        assert!(latest_args.ends_with(&["--owner-ids".to_owned(), "me".to_owned()]));
        assert_eq!(&query_args[..4], ["minutes", "+search", "--as", "user"]);
        assert!(query_args.ends_with(&["--query".to_owned(), "散步".to_owned()]));
    }

    #[test]
    fn lark_minutes_detail_uses_user_identity() {
        let args = lark_minutes_detail_args("obcn123");

        assert_eq!(&args[..2], ["minutes", "+detail"]);
        assert_eq!(&args[2..4], ["--as", "user"]);
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--minute-tokens", "obcn123"])
        );
    }

    #[test]
    fn resolve_transcript_path_uses_articles_root_for_relative_paths() {
        let root = std::path::Path::new("/tmp/articles");

        assert_eq!(
            resolve_transcript_path(root, std::path::Path::new(".moonpub/feishu/transcript.txt")),
            std::path::PathBuf::from("/tmp/articles/.moonpub/feishu/transcript.txt")
        );
    }

    #[test]
    fn civil_from_days_formats_unix_epoch() {
        assert_eq!(civil_from_days(0), "1970-01-01");
    }

    #[test]
    fn intake_feishu_minutes_with_same_token_updates_existing_inbox()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("intake-feishu-token-reuse")?;
        let first = FeishuMinutes {
            title: "晨跑录音".to_owned(),
            transcript: "第一版转写".to_owned(),
            original_file: Some("first.txt".to_owned()),
            minute_token: Some("obcn123".to_owned()),
            source_url: Some("https://example.com/first".to_owned()),
        };
        let second = FeishuMinutes {
            title: "晨跑录音".to_owned(),
            transcript: "第二版转写".to_owned(),
            original_file: Some("second.txt".to_owned()),
            minute_token: Some("obcn123".to_owned()),
            source_url: Some("https://example.com/second".to_owned()),
        };

        let created = write_feishu_minutes(&root, &first)?;
        let updated = write_feishu_minutes(&root, &second)?;
        let content = std::fs::read_to_string(&created.path)?;

        assert_eq!(updated.path, created.path);
        assert_eq!(created.action, IntakeAction::Created);
        assert_eq!(updated.action, IntakeAction::Updated);
        assert!(
            updated.message.starts_with("intake updated"),
            "{}",
            updated.message
        );
        assert!(content.contains("external_id: \"obcn123\""));
        assert!(content.contains("第二版转写"));
        assert!(content.contains("original_file: \"second.txt\""));
        assert!(content.contains("source_url: \"https://example.com/second\""));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn inbox_metadata_roundtrips_frontmatter() {
        let metadata = InboxMetadata {
            source: "feishu-minutes".to_owned(),
            status: "inbox".to_owned(),
            created: "2026-07-02".to_owned(),
            content_type: "voice-note".to_owned(),
            external_id: Some("obcn123".to_owned()),
            source_url: Some("https://example.com/minutes/obcn123".to_owned()),
            original_file: Some("/tmp/transcript.txt".to_owned()),
            captured_at: Some("2026-07-01T18:30:00+08:00".to_owned()),
            source_title: Some("晚间散步".to_owned()),
            minute_token: Some("obcn123".to_owned()),
        };

        let content = format!(
            "{}\n\n# 晚间散步\n\n## 原始转写\n\n测试内容\n",
            metadata.to_frontmatter()
        );
        let parsed = InboxMetadata::from_content(&content).expect("metadata should parse");

        assert_eq!(parsed, metadata);
    }

    #[test]
    fn intake_feishu_reuses_legacy_minute_token_files_without_external_id()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("intake-feishu-legacy-token")?;
        let inbox = root.join("Inbox/Feishu");
        std::fs::create_dir_all(&inbox)?;
        let existing = inbox.join("2026-07-01-晨跑录音.md");
        create_file(
            &existing,
            "---\nsource: feishu-minutes\nstatus: inbox\ncreated: 2026-07-01\ntype: voice-note\nminute_token: \"obcn123\"\n---\n\n# 晨跑录音\n\n## 原始转写\n\n旧内容\n",
        )?;

        let updated = write_feishu_minutes(
            &root,
            &FeishuMinutes {
                title: "晨跑录音".to_owned(),
                transcript: "新内容".to_owned(),
                original_file: Some("latest.txt".to_owned()),
                minute_token: Some("obcn123".to_owned()),
                source_url: Some("https://example.com/new".to_owned()),
            },
        )?;
        let content = std::fs::read_to_string(&existing)?;

        assert_eq!(updated.path, existing);
        assert_eq!(updated.action, IntakeAction::Updated);
        assert!(content.contains("external_id: \"obcn123\""));
        assert!(content.contains("minute_token: \"obcn123\""));
        assert!(content.contains("source_title: \"晨跑录音\""));
        assert!(content.contains("新内容"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn intake_photos_writes_photo_batch_to_obsidian_inbox() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = temp_root("intake-photos")?;
        let photo_dir = root.join("raw-photos/2026-07-02-run");
        create_file(&photo_dir.join("a.jpg"), "fake-jpg-a")?;
        create_file(&photo_dir.join("b.png"), "fake-png-b")?;

        let intake = intake_photos(&root, std::slice::from_ref(&photo_dir))?;
        let date = civil_from_days(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64
                / 86_400,
        );
        let expected = root.join(format!("Inbox/Photos/{date}-2026-07-02-run-照片记录.md"));
        let content = std::fs::read_to_string(&expected)?;

        assert_eq!(intake.path, expected);
        assert!(content.contains("source: photos"));
        assert!(content.contains("type: photo-note"));
        assert!(content.contains("# 2026-07-02-run 照片记录"));
        assert!(content.contains("## 照片清单"));
        assert!(content.contains("a.jpg"));
        assert!(content.contains("b.png"));
        assert!(content.contains("external_id: \"photos-"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn vision_photo_paths_skips_heic_but_keeps_supported_images()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("vision-photo-paths")?;
        let photo_dir = root.join("photos");
        create_file(&photo_dir.join("a.heic"), "fixture-heic")?;
        create_file(&photo_dir.join("b.jpg"), "fixture-jpg")?;

        let paths = vision_photo_paths(std::slice::from_ref(&photo_dir))?;

        assert_eq!(paths, vec![photo_dir.join("b.jpg")]);
        std::fs::remove_dir_all(root)?;
        Ok(())
    }
}
