use axum::Json;
use crate::error::Result;
use crate::services::network;

pub async fn get_network_info() -> Result<Json<serde_json::Value>> {
    match network::get_local_ip() {
        Ok(ip) => Ok(Json(serde_json::json!({
            "local_ip": ip,
            "port": 7420,
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "local_ip": null,
            "port": 7420,
            "error": e.to_string(),
        }))),
    }
}
