use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn health_check(State(state): State<AppState>) -> Json<Value> {
    let db_ok = sqlx::query("SELECT 1").fetch_one(&state.db).await.is_ok();

    let disk_ok = check_disk_space(&state.rings_dir);

    let status = if db_ok && disk_ok { "ok" } else { "degraded" };

    Json(json!({
        "status": status,
        "checks": {
            "database": db_ok,
            "disk": disk_ok,
        }
    }))
}

fn check_disk_space(path: &std::path::Path) -> bool {
    // Simple check: can we read the directory?
    std::fs::read_dir(path).is_ok()
}
