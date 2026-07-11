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

pub fn validate_ffmpeg(path: &str) -> AppResult<(bool, String, Option<String>)> {
    let trimmed = path.trim();
    let candidates = if trimmed.is_empty() || trimmed == "ffmpeg" {
        vec![PathBuf::from("ffmpeg")]
    } else {
        vec![PathBuf::from(trimmed)]
    };

    for candidate in candidates {
        let output = match Command::new(&candidate).arg("-version").output() {
            Ok(output) => output,
            Err(_) => continue,
        };

        if !output.status.success() {
            continue;
        }

        let version_line = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("FFmpeg")
            .to_string();
        return Ok((true, version_line, Some(candidate.to_string_lossy().to_string())));
    }

    Ok((
        false,
        "无法执行 FFmpeg，请检查路径或安装 FFmpeg".to_string(),
        None,
    ))
}
