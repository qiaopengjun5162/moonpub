use std::fs;
use std::path::{Path, PathBuf};

use super::{
    INBOX_STATUS, InboxMetadata, IntakeAction, IntakeOutput, find_existing_inbox_by_external_id,
    slugify, today_utc,
};
use crate::error::AppError;

const PHOTOS_SOURCE: &str = "photos";
const PHOTO_NOTE_TYPE: &str = "photo-note";

pub fn intake_photos(articles_dir: &Path, inputs: &[PathBuf]) -> Result<IntakeOutput, AppError> {
    let batch = PhotoBatch::from_inputs(inputs)?;
    write_photo_batch(articles_dir, &batch)
}

struct PhotoBatch {
    title: String,
    summary: String,
    files: Vec<PhotoAsset>,
    external_id: String,
    original_file: Option<String>,
    captured_at: Option<String>,
}

struct PhotoAsset {
    path: PathBuf,
    size_bytes: u64,
    modified_at: Option<String>,
}

impl PhotoBatch {
    fn from_inputs(inputs: &[PathBuf]) -> Result<Self, AppError> {
        let files = collect_photo_assets(inputs)?;
        if files.is_empty() {
            return Err(AppError::MissingValue(
                "intake photos <file-or-dir> requires at least one image file",
            ));
        }
        let title = infer_photo_batch_title(&files);
        let total_bytes = files.iter().map(|file| file.size_bytes).sum::<u64>();
        let original_file = files
            .first()
            .and_then(|file| file.path.parent())
            .map(|path| path.display().to_string());
        let captured_at = files.first().and_then(|file| file.modified_at.clone());
        let summary = format!(
            "这一批素材共 {} 张，总计 {} bytes。先按真实文件信息归档，后续可继续整理成生活文章草稿。",
            files.len(),
            total_bytes
        );

        let external_id = build_photo_external_id(&files);

        Ok(Self {
            title,
            summary,
            files,
            external_id,
            original_file,
            captured_at,
        })
    }
}

fn write_photo_batch(articles_dir: &Path, batch: &PhotoBatch) -> Result<IntakeOutput, AppError> {
    let date = today_utc();
    let slug = slugify(&batch.title);
    let inbox_dir = articles_dir.join("Inbox/Photos");
    fs::create_dir_all(&inbox_dir).map_err(|source| AppError::Io {
        path: inbox_dir.clone(),
        source,
    })?;
    let metadata = InboxMetadata {
        source: PHOTOS_SOURCE.to_owned(),
        status: INBOX_STATUS.to_owned(),
        created: date.clone(),
        content_type: PHOTO_NOTE_TYPE.to_owned(),
        external_id: Some(batch.external_id.clone()),
        source_url: None,
        original_file: batch.original_file.clone(),
        captured_at: batch.captured_at.clone(),
        source_title: Some(batch.title.clone()),
        minute_token: None,
    };
    let output = find_existing_inbox_by_external_id(&inbox_dir, &batch.external_id)?
        .unwrap_or_else(|| inbox_dir.join(format!("{date}-{slug}.md")));
    let action = if output.exists() {
        IntakeAction::Updated
    } else {
        IntakeAction::Created
    };
    let assets = batch
        .files
        .iter()
        .map(|asset| {
            format!(
                "- {} | {} bytes | {}",
                asset.path.display(),
                asset.size_bytes,
                asset.modified_at.as_deref().unwrap_or("unknown-time")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let content = format!(
        "{}\n\n# {}\n\n## 素材概览\n\n{}\n\n## 照片清单\n\n{}\n",
        metadata.to_frontmatter(),
        batch.title,
        batch.summary,
        assets
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

fn collect_photo_assets(inputs: &[PathBuf]) -> Result<Vec<PhotoAsset>, AppError> {
    let mut files = Vec::new();
    for input in inputs {
        collect_photo_assets_from_path(input, &mut files)?;
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

fn collect_photo_assets_from_path(
    input: &Path,
    files: &mut Vec<PhotoAsset>,
) -> Result<(), AppError> {
    let metadata = fs::metadata(input).map_err(|source| AppError::Io {
        path: input.to_path_buf(),
        source,
    })?;
    if metadata.is_dir() {
        let entries = fs::read_dir(input).map_err(|source| AppError::Io {
            path: input.to_path_buf(),
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| AppError::Io {
                path: input.to_path_buf(),
                source,
            })?;
            collect_photo_assets_from_path(&entry.path(), files)?;
        }
        return Ok(());
    }
    if !is_supported_photo(input) {
        return Ok(());
    }
    files.push(PhotoAsset {
        path: input.to_path_buf(),
        size_bytes: metadata.len(),
        modified_at: metadata.modified().ok().map(system_time_label),
    });
    Ok(())
}

fn is_supported_photo(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_ascii_lowercase()),
        Some(ext) if matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "heic" | "webp")
    )
}

fn infer_photo_batch_title(files: &[PhotoAsset]) -> String {
    files
        .first()
        .and_then(|file| file.path.parent())
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        .map(|name| format!("{name} 照片记录"))
        .unwrap_or_else(|| format!("{} 张照片记录", files.len()))
}

fn build_photo_external_id(files: &[PhotoAsset]) -> String {
    let mut seed = String::new();
    for file in files {
        seed.push_str(&file.path.display().to_string());
        seed.push('|');
        seed.push_str(&file.size_bytes.to_string());
        seed.push('|');
    }
    let checksum = seed.bytes().fold(0_u64, |acc, byte| {
        acc.wrapping_mul(131).wrapping_add(byte as u64)
    });
    format!("photos-{checksum:016x}")
}

fn system_time_label(time: std::time::SystemTime) -> String {
    let days = time
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() / 86_400)
        .unwrap_or(0);
    super::civil_from_days(days as i64)
}
