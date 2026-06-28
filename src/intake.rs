use std::fs;
use std::path::Path;

use crate::error::AppError;

pub fn intake_feishu(articles_dir: &Path, input: &Path) -> Result<String, AppError> {
    let raw = fs::read_to_string(input).map_err(|source| AppError::Io {
        path: input.to_path_buf(),
        source,
    })?;
    let title = feishu_title(&raw, input);
    let date = today_utc();
    let slug = slugify(&title);
    let inbox_dir = articles_dir.join("Inbox/Feishu");
    fs::create_dir_all(&inbox_dir).map_err(|source| AppError::Io {
        path: inbox_dir.clone(),
        source,
    })?;
    let output = inbox_dir.join(format!("{date}-{slug}.md"));
    let content = format!(
        "---\nsource: feishu-minutes\nstatus: inbox\ncreated: {date}\ntype: voice-note\noriginal_file: \"{}\"\n---\n\n# {title}\n\n## 原始转写\n\n{}\n",
        input.display(),
        raw.trim()
    );
    fs::write(&output, content).map_err(|source| AppError::Io {
        path: output.clone(),
        source,
    })?;

    Ok(format!("intake created\n  {}", output.display()))
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
    use crate::intake::{civil_from_days, intake_feishu, slugify};
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
    fn civil_from_days_formats_unix_epoch() {
        assert_eq!(civil_from_days(0), "1970-01-01");
    }
}
