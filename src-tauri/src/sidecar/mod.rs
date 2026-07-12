mod client;

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

pub use client::{SidecarClient, SidecarJob};

use crate::errors::{AppError, AppResult};
use crate::models::SidecarHealth;

const DEFAULT_PORT: u16 = 18765;
const EXPECTED_SIDECAR_VERSION: &str = "0.5.2-youtube";
const SIDECAR_BINARY_NAME: &str = "cliprove-sidecar";
const STARTUP_ATTEMPTS: usize = 120;
const STARTUP_POLL_MS: u64 = 250;

pub struct SidecarManager {
    port: u16,
    child: Mutex<Option<Child>>,
}

impl SidecarManager {
    pub fn new() -> Self {
        Self {
            port: DEFAULT_PORT,
            child: Mutex::new(None),
        }
    }

    pub fn client(&self) -> AppResult<SidecarClient> {
        SidecarClient::new(self.port)
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn start(&self) -> AppResult<SidecarHealth> {
        if let Ok(health) = self.health() {
            if is_current_sidecar(&health) {
                return Ok(health);
            }
        }

        self.stop()?;

        let (program, args, working_dir) = resolve_sidecar_launcher(self.port)?;
        let launcher_label = format_launcher(&program, &args);
        let mut command = Command::new(&program);
        command.args(&args);
        command.env("PATH", augmented_path());
        if let Some(bundled) = crate::shell::bundled_ffmpeg_path() {
            command.env(
                "CLIPROVE_BUNDLED_FFMPEG",
                bundled.to_string_lossy().to_string(),
            );
        }
        if let Some(dir) = working_dir {
            command.current_dir(dir);
        }
        let child = command
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| {
                AppError::structured(
                    "engine_failure",
                    "无法启动 Python Sidecar",
                    Some(format!(
                        "请运行 scripts/build-sidecar.sh 或配置 sidecar/.venv ({error})"
                    )),
                )
            })?;

        {
            let mut guard = self.child.lock().map_err(|_| {
                AppError::Message("sidecar lock poisoned".to_string())
            })?;
            *guard = Some(child);
        }

        for _ in 0..STARTUP_ATTEMPTS {
            std::thread::sleep(std::time::Duration::from_millis(STARTUP_POLL_MS));
            if let Ok(health) = self.health() {
                if is_current_sidecar(&health) {
                    return Ok(health);
                }
            }

            let mut guard = self
                .child
                .lock()
                .map_err(|_| AppError::Message("sidecar lock poisoned".to_string()))?;
            if let Some(child) = guard.as_mut() {
                if let Ok(Some(status)) = child.try_wait() {
                    guard.take();
                    return Err(AppError::structured(
                        "engine_failure",
                        "Sidecar 启动失败",
                        Some(format!(
                            "启动器已退出（{status}）：{launcher_label}。请检查 sidecar/.venv 依赖或重新运行 scripts/dev.sh"
                        )),
                    ));
                }
            }
        }

        Err(AppError::structured(
            "engine_failure",
            "Sidecar 启动超时",
            Some(format!(
                "已等待约 {} 秒：{launcher_label}。请检查 sidecar/.venv 依赖，或重新运行 scripts/dev.sh",
                STARTUP_ATTEMPTS as u64 * STARTUP_POLL_MS / 1000
            )),
        ))
    }

    pub fn health(&self) -> AppResult<SidecarHealth> {
        SidecarClient::with_timeout(self.port, std::time::Duration::from_secs(5))?.health()
    }

    pub fn stop(&self) -> AppResult<()> {
        let mut guard = self.child.lock().map_err(|_| {
            AppError::Message("sidecar lock poisoned".to_string())
        })?;
        if let Some(mut child) = guard.take() {
            child.kill().ok();
            child.wait().ok();
        }
        Ok(())
    }
}

impl Drop for SidecarManager {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn is_current_sidecar(health: &SidecarHealth) -> bool {
    health.status == "ok" && health.version.as_deref() == Some(EXPECTED_SIDECAR_VERSION)
}

fn format_launcher(program: &std::path::Path, args: &[String]) -> String {
    let mut parts = vec![program.to_string_lossy().to_string()];
    parts.extend(args.iter().cloned());
    parts.join(" ")
}

fn augmented_path() -> String {
    let existing = std::env::var("PATH").unwrap_or_default();
    let extras = [
        "/opt/homebrew/bin",
        "/usr/local/bin",
        "/opt/homebrew/sbin",
        "/usr/local/sbin",
    ];
    let mut paths: Vec<String> = extras.iter().map(|entry| (*entry).to_string()).collect();
    if !existing.is_empty() {
        paths.push(existing);
    }
    paths.join(":")
}

fn resolve_sidecar_launcher(port: u16) -> AppResult<(PathBuf, Vec<String>, Option<PathBuf>)> {
    // Prefer the dev venv when present so `tauri dev` does not depend on the
    // externalBin stub resolving paths from target/debug/.
    if let Ok(project_root) = project_root() {
        let venv_python = project_root.join("sidecar/.venv/bin/python3");
        let script = project_root.join("sidecar/app.py");
        if venv_python.exists() && script.exists() {
            return Ok((
                venv_python,
                vec![
                    script.to_string_lossy().to_string(),
                    "--port".to_string(),
                    port.to_string(),
                ],
                Some(project_root.join("sidecar")),
            ));
        }
    }

    if let Some(binary) = bundled_sidecar_binary() {
        return Ok((
            binary,
            vec!["--port".to_string(), port.to_string()],
            None,
        ));
    }

    let python = dev_sidecar_python()?;
    let script = dev_sidecar_entrypoint()?;
    let project_root = project_root()?;
    Ok((
        python,
        vec![
            script.to_string_lossy().to_string(),
            "--port".to_string(),
            port.to_string(),
        ],
        Some(project_root.join("sidecar")),
    ))
}

fn bundled_sidecar_binary() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let macos_candidate = exe
        .parent()?
        .join(SIDECAR_BINARY_NAME);
    if macos_candidate.exists() {
        return Some(macos_candidate);
    }

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let target = std::env::var("TARGET").ok();
        if let Some(target) = target {
            let dev_bundle = PathBuf::from(manifest_dir)
                .join("binaries")
                .join(format!("{SIDECAR_BINARY_NAME}-{target}"));
            if dev_bundle.exists() {
                return Some(dev_bundle);
            }
        }
    }

    None
}

fn project_root() -> AppResult<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| AppError::Message("invalid manifest dir".to_string()))
}

fn dev_sidecar_python() -> AppResult<PathBuf> {
    let venv_python = project_root()?.join("sidecar/.venv/bin/python3");
    if venv_python.exists() {
        return Ok(venv_python);
    }
    Ok(PathBuf::from("python3"))
}

fn dev_sidecar_entrypoint() -> AppResult<PathBuf> {
    let dev_path = project_root()?.join("sidecar/app.py");
    if dev_path.exists() {
        return Ok(dev_path);
    }

    Err(AppError::structured(
        "engine_failure",
        "未找到 sidecar 可执行文件或 app.py",
        Some("开发环境请保留 sidecar 目录；发布构建请运行 scripts/build-sidecar.sh".to_string()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_entrypoint_exists_in_repo() {
        let path = dev_sidecar_entrypoint();
        assert!(path.is_ok(), "expected sidecar/app.py in development tree");
    }

    #[test]
    fn current_sidecar_requires_expected_version() {
        assert!(is_current_sidecar(&SidecarHealth {
            status: "ok".to_string(),
            version: Some(EXPECTED_SIDECAR_VERSION.to_string()),
        }));
        assert!(!is_current_sidecar(&SidecarHealth {
            status: "ok".to_string(),
            version: Some("0.5.0-phase5".to_string()),
        }));
        assert!(!is_current_sidecar(&SidecarHealth {
            status: "failed".to_string(),
            version: Some(EXPECTED_SIDECAR_VERSION.to_string()),
        }));
    }

    #[test]
    fn format_launcher_includes_program_and_args() {
        let label = format_launcher(
            std::path::Path::new("/tmp/python"),
            &["app.py".to_string(), "--port".to_string(), "18765".to_string()],
        );
        assert_eq!(label, "/tmp/python app.py --port 18765");
    }
}
