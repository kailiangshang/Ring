use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum RingError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("git error: {0}")]
    Git(#[from] git2::Error),

    #[error("database error")]
    Database(#[from] sqlx::Error),

    #[error("llm error: {0}")]
    Llm(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    Internal(String),
}

impl IntoResponse for RingError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            RingError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            RingError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            RingError::Forbidden(_) => (StatusCode::FORBIDDEN, self.to_string()),
            RingError::Conflict(_) => (StatusCode::CONFLICT, self.to_string()),
            RingError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error".to_string(),
            ),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, RingError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;

    use http_body_util::BodyExt;
    use serde_json::Value;

    async fn body_to_json(body: Body) -> Value {
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn not_found_maps_to_404() {
        let err = RingError::NotFound("ring not found".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn unauthorized_maps_to_401() {
        let err = RingError::Unauthorized("not setup".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn forbidden_maps_to_403() {
        let err = RingError::Forbidden("members cannot edit".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn conflict_maps_to_409() {
        let err = RingError::Conflict("already setup".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn validation_maps_to_400() {
        let err = RingError::Validation("name required".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn internal_errors_hide_details() {
        let err = RingError::Internal("database exploded".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn database_error_is_internal() {
        let sqlx_err = sqlx::Error::RowNotFound;
        let err = RingError::from(sqlx_err);
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn response_body_contains_error_field() {
        let err = RingError::NotFound("ring-123".into());
        let resp = err.into_response();
        let (_parts, body) = resp.into_parts();
        let json = body_to_json(body).await;
        assert_eq!(json["error"], "not found: ring-123");
    }
}
