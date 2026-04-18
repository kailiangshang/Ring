use std::path::PathBuf;

use sqlx::SqlitePool;

use crate::ws_hub::WsHub;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub ws_hub: WsHub,
    pub rings_dir: PathBuf,
    pub hub_dir: PathBuf,
}

impl AppState {
    pub fn new(db: SqlitePool, rings_dir: PathBuf, hub_dir: PathBuf) -> Self {
        Self {
            db,
            ws_hub: WsHub::new(),
            rings_dir,
            hub_dir,
        }
    }
}
