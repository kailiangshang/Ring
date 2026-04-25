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

    #[error("Gone: {0}")]
    Gone(String),

    #[error("Bad gateway")]
    BadGateway,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Git command failed: {cmd}: {stderr}")]
    GitCommandFailed { cmd: String, stderr: String },

    #[error("GitLab API error ({status}): {message}")]
    GitlabApiError { status: u16, message: String },

    #[error("GitLab not configured")]
    GitlabNotConfigured,

    #[error("Repository not found for ring: {ring_id}")]
    RepoNotFound { ring_id: String },

    #[error("Archive conflict: {record_id}")]
    ArchiveConflict { record_id: String },

    #[error("Invalid archive state: record {record_id} is {current}, expected {expected}")]
    InvalidArchiveState {
        record_id: String,
        current: String,
        expected: String,
    },
}

impl From<sqlx::Error> for RingError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => RingError::NotFound("resource not found".into()),
            sqlx::Error::Database(ref db) => {
                if db.code().is_some_and(|c| c == "2067" || c == "1555") {
                    RingError::Conflict("resource already exists".into())
                } else {
                    RingError::Internal(e.to_string())
                }
            }
            _ => RingError::Internal(e.to_string()),
        }
    }
}

impl From<async_openai::error::OpenAIError> for RingError {
    fn from(e: async_openai::error::OpenAIError) -> Self {
        RingError::Internal(format!("LLM error: {e}"))
    }
}

impl From<std::io::Error> for RingError {
    fn from(e: std::io::Error) -> Self {
        RingError::Internal(format!("IO error: {e}"))
    }
}

impl From<serde_json::Error> for RingError {
    fn from(e: serde_json::Error) -> Self {
        RingError::Internal(format!("JSON error: {e}"))
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
            RingError::Gone(msg) => (StatusCode::GONE, msg.clone()),
            RingError::BadGateway => (StatusCode::BAD_GATEWAY, "bad gateway".into()),
            RingError::Internal(msg) => {
                tracing::error!("Internal error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".into(),
                )
            }
            RingError::GitCommandFailed { cmd, stderr } => {
                tracing::error!("Git command failed: {cmd}: {stderr}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".into(),
                )
            }
            RingError::GitlabApiError { status, message } => {
                tracing::error!("GitLab API error ({status}): {message}");
                (
                    StatusCode::BAD_GATEWAY,
                    format!("gitlab api error: {message}"),
                )
            }
            RingError::GitlabNotConfigured => {
                (StatusCode::BAD_REQUEST, "gitlab not configured".into())
            }
            RingError::RepoNotFound { .. } => {
                tracing::warn!("Repo not found (ring_id redacted)");
                (StatusCode::NOT_FOUND, "resource not found".into())
            }
            RingError::ArchiveConflict { .. } => (StatusCode::CONFLICT, "archive conflict".into()),
            RingError::InvalidArchiveState { .. } => {
                (StatusCode::CONFLICT, "invalid archive state".into())
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
