use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AuthUser {
    pub user_id: String,
}

pub async fn auth_middleware(request: Request, next: Next) -> Response {
    let user_id = request
        .headers()
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
        .filter(|v| !v.is_empty());

    match user_id {
        Some(user_id) => {
            let (mut parts, body) = request.into_parts();
            parts.extensions.insert(AuthUser { user_id });
            let request = Request::from_parts(parts, body);
            next.run(request).await
        }
        None => (
            StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({ "error": "missing X-User-Id header" })),
        )
            .into_response(),
    }
}
