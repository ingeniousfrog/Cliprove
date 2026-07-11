use std::sync::Arc;

use crate::db::Database;
use crate::sidecar::SidecarManager;

pub struct AppState {
    pub db: Arc<Database>,
    pub sidecar: Arc<SidecarManager>,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        Self {
            db: Arc::new(db),
            sidecar: Arc::new(SidecarManager::new()),
        }
    }
}
