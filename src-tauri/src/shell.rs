use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use crate::errors::{AppError, AppResult};

pub fn reveal_in_finder(path: &str) -> AppResult<()> {
    let target = Path::new(path);
    if !target.exists() {
        return Err(AppError::Message(format!("路径不存在: {path}")));
    }

    #[cfg(target_os = "macos")]
    {
        let status = Command::new("open")
            .args(["-R", path])
            .status()
            .map_err(|error| AppError::Message(error.to_string()))?;
        if status.success() {
            return Ok(());
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let parent = target
            .parent()
            .ok_or_else(|| AppError::Message("无法解析父目录".to_string()))?;
        let status = Command::new("xdg-open")
            .arg(parent)
            .status()
            .map_err(|error| AppError::Message(error.to_string()))?;
        if status.success() {
            return Ok(());
        }
    }

    Err(AppError::Message("无法在文件管理器中显示".to_string()))
}

pub fn read_text_file(path: &str, max_bytes: usize) -> AppResult<String> {
    let metadata = std::fs::metadata(path)
        .map_err(|_| AppError::Message(format!("无法读取文件: {path}")))?;
    if metadata.len() as usize > max_bytes {
        return Err(AppError::Message(format!(
            "文件过大（>{max_bytes} 字节）"
        )));
    }
    std::fs::read_to_string(path).map_err(|error| AppError::Message(error.to_string()))
}

fn ffmpeg_candidates(path: &str) -> Vec<PathBuf> {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "ffmpeg" {
        vec![
            PathBuf::from("/opt/homebrew/bin/ffmpeg"),
            PathBuf::from("/usr/local/bin/ffmpeg"),
            PathBuf::from("/usr/bin/ffmpeg"),
            PathBuf::from("ffmpeg"),
        ]
    } else {
        vec![PathBuf::from(trimmed)]
    }
}

pub fn resolve_ffmpeg_path(path: &str) -> Option<PathBuf> {
    for candidate in ffmpeg_candidates(path) {
        let output = match Command::new(&candidate).arg("-version").output() {
            Ok(output) => output,
            Err(_) => continue,
        };

        if !output.status.success() {
            continue;
        }

        if candidate.is_absolute() {
            return candidate.canonicalize().ok().or(Some(candidate));
        }

        for absolute in [
            "/opt/homebrew/bin/ffmpeg",
            "/usr/local/bin/ffmpeg",
            "/usr/bin/ffmpeg",
        ] {
            let resolved = PathBuf::from(absolute);
            if resolved.is_file() {
                return resolved.canonicalize().ok().or(Some(resolved));
            }
        }

        return Some(candidate);
    }

    None
}

pub fn validate_ffmpeg(path: &str) -> AppResult<(bool, String, Option<String>)> {
    if let Some(resolved) = resolve_ffmpeg_path(path) {
        let output = Command::new(&resolved)
            .arg("-version")
            .output()
            .map_err(|error| AppError::Message(error.to_string()))?;

        let version_line = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("FFmpeg")
            .to_string();
        return Ok((
            true,
            version_line,
            Some(resolved.to_string_lossy().to_string()),
        ));
    }

    Ok((
        false,
        "无法执行 FFmpeg，请检查路径或安装 FFmpeg".to_string(),
        None,
    ))
}
