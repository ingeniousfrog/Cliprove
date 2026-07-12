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

pub fn bundled_ffmpeg_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("CLIPROVE_BUNDLED_FFMPEG") {
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            return candidate.canonicalize().ok().or(Some(candidate));
        }
    }

    let exe = std::env::current_exe().ok()?;
    let resource_dirs = bundled_ffmpeg_resource_dirs(&exe);
    for dir in resource_dirs {
        if !dir.is_dir() {
            continue;
        }
        if let Ok(target) = std::env::var("TARGET") {
            for name in bundled_ffmpeg_names(&target) {
                let candidate = dir.join(&name);
                if candidate.is_file() {
                    return candidate.canonicalize().ok().or(Some(candidate));
                }
            }
        }
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && path
                        .file_name()
                        .is_some_and(|name| name.to_string_lossy().starts_with("ffmpeg-"))
                {
                    return path.canonicalize().ok().or(Some(path));
                }
            }
        }
    }

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let dev_resources = PathBuf::from(manifest_dir).join("resources/ffmpeg");
        if dev_resources.is_dir() {
            if let Ok(target) = std::env::var("TARGET") {
                for name in bundled_ffmpeg_names(&target) {
                    let candidate = dev_resources.join(&name);
                    if candidate.is_file() {
                        return candidate.canonicalize().ok().or(Some(candidate));
                    }
                }
            }
        }
    }

    None
}

fn bundled_ffmpeg_resource_dirs(exe: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(parent) = exe.parent() {
        #[cfg(target_os = "macos")]
        if let Some(contents) = parent.parent() {
            dirs.push(contents.join("Resources/ffmpeg"));
        }
        #[cfg(target_os = "windows")]
        {
            dirs.push(parent.join("resources/ffmpeg"));
            dirs.push(parent.join("resources").join("ffmpeg"));
        }
        dirs.push(parent.join("resources/ffmpeg"));
    }
    dirs
}

fn bundled_ffmpeg_names(target: &str) -> Vec<String> {
    vec![
        format!("ffmpeg-{target}"),
        format!("ffmpeg-{target}.exe"),
    ]
}

fn system_ffmpeg_candidates() -> Vec<PathBuf> {
    let mut candidates = vec![PathBuf::from("ffmpeg")];
    #[cfg(target_os = "macos")]
    {
        candidates.extend([
            PathBuf::from("/opt/homebrew/bin/ffmpeg"),
            PathBuf::from("/usr/local/bin/ffmpeg"),
            PathBuf::from("/usr/bin/ffmpeg"),
        ]);
    }
    #[cfg(target_os = "windows")]
    {
        candidates.push(PathBuf::from("ffmpeg.exe"));
    }
    candidates
}

fn ffmpeg_candidates(path: &str) -> Vec<PathBuf> {
    let trimmed = path.trim();
    let mut candidates = Vec::new();
    if !trimmed.is_empty() && trimmed != "ffmpeg" {
        candidates.push(PathBuf::from(trimmed));
    }
    candidates.extend(system_ffmpeg_candidates());
    if let Some(bundled) = bundled_ffmpeg_path() {
        candidates.push(bundled);
    }
    candidates
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
