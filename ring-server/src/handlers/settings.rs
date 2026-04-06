use axum::extract::State;
use axum::Json;

use crate::error::RingError;
use crate::services::settings_service::SettingsService;
use crate::state::AppState;

pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, RingError> {
    let svc = SettingsService::new(state.db);
    let settings = svc.get_all_settings().await?;
    Ok(Json(settings))
}

pub async fn update_settings(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, RingError> {
    let svc = SettingsService::new(state.db);
    svc.update_settings(body).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
