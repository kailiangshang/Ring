use sqlx::SqlitePool;

use crate::ws_hub::WsHub;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub ws_hub: WsHub,
}

impl AppState {
    pub fn new(db: SqlitePool) -> Self {
        Self {
            db,
            ws_hub: WsHub::new(),
        }
    }
}
