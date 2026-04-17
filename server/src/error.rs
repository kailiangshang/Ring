use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum RingError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<sqlx::Error> for RingError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => RingError::NotFound("resource not found".into()),
            sqlx::Error::Database(ref db) => {
                if db.code().is_some_and(|c| c == "2067") {
                    RingError::Conflict("resource already exists".into())
                } else {
                    RingError::Internal(e.to_string())
                }
            }
            _ => RingError::Internal(e.to_string()),
        }
    }
}

impl IntoResponse for RingError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            RingError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            RingError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            RingError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            RingError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            RingError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            RingError::Internal(msg) => {
                tracing::error!("Internal error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".into(),
                )
            }
        };
        let code = status
            .canonical_reason()
            .unwrap_or("error")
            .to_lowercase()
            .replace(' ', "_");
        let body = json!({
            "error": {
                "code": code,
                "message": message,
            }
        });
        (status, axum::Json(body)).into_response()
    }
}

pub type Result<T> = std::result::Result<T, RingError>;
