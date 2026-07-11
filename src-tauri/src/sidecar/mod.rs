mod client;

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

pub use client::SidecarClient;

use crate::errors::{AppError, AppResult};
use crate::models::SidecarHealth;

const DEFAULT_PORT: u16 = 18765;

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

    pub fn start(&self) -> AppResult<SidecarHealth> {
        if let Ok(health) = self.health() {
            if health.status == "ok" {
                return Ok(health);
            }
        }

        self.stop()?;

        let script = sidecar_entrypoint()?;
        let python = sidecar_python()?;
        let project_root = project_root()?;
        let child = Command::new(python)
            .arg(script)
            .arg("--port")
            .arg(self.port.to_string())
            .current_dir(project_root.join("sidecar"))
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                AppError::structured(
                    "engine_failure",
                    "无法启动 Python Sidecar",
                    Some(format!(
                        "请运行 sidecar/.venv 安装依赖：pip install -r sidecar/requirements.txt ({error})"
                    )),
                )
            })?;

        {
            let mut guard = self.child.lock().map_err(|_| {
                AppError::Message("sidecar lock poisoned".to_string())
            })?;
            *guard = Some(child);
        }

        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_millis(200));
            if let Ok(health) = self.health() {
                if health.status == "ok" {
                    return Ok(health);
                }
            }
        }

        Err(AppError::structured(
            "engine_failure",
            "Sidecar 启动超时",
            Some("检查 sidecar/requirements.txt 与 engines/douyin-downloader submodule".to_string()),
        ))
    }

    pub fn health(&self) -> AppResult<SidecarHealth> {
        self.client()?.health()
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

fn project_root() -> AppResult<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| AppError::Message("invalid manifest dir".to_string()))
}

fn sidecar_python() -> AppResult<PathBuf> {
    let venv_python = project_root()?.join("sidecar/.venv/bin/python3");
    if venv_python.exists() {
        return Ok(venv_python);
    }
    Ok(PathBuf::from("python3"))
}

fn sidecar_entrypoint() -> AppResult<PathBuf> {
    let dev_path = project_root()?.join("sidecar/app.py");
    if dev_path.exists() {
        return Ok(dev_path);
    }

    Err(AppError::structured(
        "engine_failure",
        "未找到 sidecar/app.py",
        Some("请在开发环境中保留 sidecar 目录".to_string()),
    ))
}
