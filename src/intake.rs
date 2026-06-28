use std::fs;
use std::path::Path;
use std::process::Command;

use crate::error::AppError;

pub fn intake_feishu(articles_dir: &Path, input: &Path) -> Result<String, AppError> {
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

pub fn intake_feishu_minute_token(articles_dir: &Path, token: &str) -> Result<String, AppError> {
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

struct FeishuMinutes {
    title: String,
    transcript: String,
    original_file: Option<String>,
    minute_token: Option<String>,
    source_url: Option<String>,
}

fn write_feishu_minutes(articles_dir: &Path, minutes: &FeishuMinutes) -> Result<String, AppError> {
    let date = today_utc();
    let slug = slugify(&minutes.title);
    let inbox_dir = articles_dir.join("Inbox/Feishu");
    fs::create_dir_all(&inbox_dir).map_err(|source| AppError::Io {
        path: inbox_dir.clone(),
        source,
    })?;
    let output = inbox_dir.join(format!("{date}-{slug}.md"));
    let mut frontmatter =
        format!("---\nsource: feishu-minutes\nstatus: inbox\ncreated: {date}\ntype: voice-note\n");
    if let Some(token) = &minutes.minute_token {
        frontmatter.push_str(&format!("minute_token: \"{}\"\n", yaml_escape(token)));
    }
    if let Some(url) = &minutes.source_url {
        frontmatter.push_str(&format!("source_url: \"{}\"\n", yaml_escape(url)));
    }
    if let Some(file) = &minutes.original_file {
        frontmatter.push_str(&format!("original_file: \"{}\"\n", yaml_escape(file)));
    }
    frontmatter.push_str("---");
    let content = format!(
        "{frontmatter}\n\n# {}\n\n## 原始转写\n\n{}\n",
        minutes.title,
        minutes.transcript.trim()
    );
    fs::write(&output, content).map_err(|source| AppError::Io {
        path: output.clone(),
        source,
    })?;

    Ok(format!("intake created\n  {}", output.display()))
}

struct FeishuMinutesDetail {
    title: String,
    transcript_file: std::path::PathBuf,
    source_url: Option<String>,
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
        .args([
            "minutes",
            "+detail",
            "--minute-tokens",
            token,
            "--transcript",
            "--overwrite",
            "--output-dir",
            ".moonpub/feishu-minutes",
            "--format",
            "json",
        ])
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
        civil_from_days, intake_feishu, parse_feishu_minutes_detail, resolve_transcript_path,
        slugify,
    };
    use crate::test_helpers::{create_file, temp_root};

    #[test]
    fn intake_feishu_writes_raw_minutes_to_obsidian_inbox() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = temp_root("intake-feishu")?;
        let input = root.join("exports/morning.txt");
        create_file(&input, "早上散步想到的事情\n\n今天跑完步以后，想记录一下。")?;

        let message = intake_feishu(&root, &input)?;
        let date = civil_from_days(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64
                / 86_400,
        );
        let output = root.join(format!("Inbox/Feishu/{date}-早上散步想到的事情.md"));
        let content = std::fs::read_to_string(&output)?;

        assert!(message.contains(output.to_string_lossy().as_ref()));
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
}
