use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

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

    pub fn port(&self) -> u16 {
        self.port
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
        let child = Command::new(python)
            .arg(script)
            .arg("--port")
            .arg(self.port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| {
                AppError::structured(
                    "engine_failure",
                    "无法启动 Python Sidecar",
                    Some(format!("请确认已安装 Python 3 与 fastapi/uvicorn：{error}")),
                )
            })?;

        {
            let mut guard = self.child.lock().map_err(|_| {
                AppError::Message("sidecar lock poisoned".to_string())
            })?;
            *guard = Some(child);
        }

        for _ in 0..20 {
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
            Some("检查 sidecar/requirements.txt 是否已安装".to_string()),
        ))
    }

    pub fn health(&self) -> AppResult<SidecarHealth> {
        let url = format!("http://127.0.0.1:{}/health", self.port);
        let response = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()?
            .get(url)
            .send()?;

        if !response.status().is_success() {
            return Err(AppError::structured(
                "engine_failure",
                "Sidecar 健康检查失败",
                None,
            ));
        }

        Ok(response.json()?)
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

fn sidecar_python() -> AppResult<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir
        .parent()
        .ok_or_else(|| AppError::Message("invalid manifest dir".to_string()))?;

    let venv_python = project_root.join("sidecar/.venv/bin/python3");
    if venv_python.exists() {
        return Ok(venv_python);
    }

    Ok(PathBuf::from("python3"))
}

fn sidecar_entrypoint() -> AppResult<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dev_path = manifest_dir
        .parent()
        .map(|path| path.join("sidecar/app.py"))
        .ok_or_else(|| AppError::Message("invalid manifest dir".to_string()))?;

    if dev_path.exists() {
        return Ok(dev_path);
    }

    Err(AppError::structured(
        "engine_failure",
        "未找到 sidecar/app.py",
        Some("请在开发环境中保留 sidecar 目录".to_string()),
    ))
}
