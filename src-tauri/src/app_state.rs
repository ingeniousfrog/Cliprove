use std::sync::Arc;

use crate::db::Database;
use crate::sidecar::SidecarManager;

pub struct AppState {
    pub db: Arc<Database>,
    pub sidecar: Arc<SidecarManager>,
    pub download_slots: Arc<tokio::sync::Semaphore>,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        let max_slots = db
            .settings()
            .get_all()
            .map(|settings| settings.max_concurrent_downloads.max(1) as usize)
            .unwrap_or(3);

        Self {
            db: Arc::new(db),
            sidecar: Arc::new(SidecarManager::new()),
            download_slots: Arc::new(tokio::sync::Semaphore::new(max_slots)),
        }
    }
}
