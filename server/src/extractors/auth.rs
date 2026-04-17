use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;

use crate::error::RingError;
use crate::state::AppState;

pub struct AuthUser {
    pub token_id: String,
}

impl<S: Send + Sync> FromRequestParts<S> for AuthUser
where
    AppState: axum::extract::FromRef<S>,
{
    type Rejection = RingError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let token = parts
            .headers
            .get("X-Ring-Token")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| RingError::Unauthorized("missing X-Ring-Token header".into()))?;

        let exists: bool =
            sqlx::query_scalar::<_, bool>("SELECT COUNT(*) > 0 FROM users WHERE token_id = ?1")
                .bind(token)
                .fetch_one(&app_state.db)
                .await
                .map_err(|e: sqlx::Error| RingError::Internal(e.to_string()))?;

        if !exists {
            return Err(RingError::Unauthorized("invalid token".into()));
        }

        Ok(AuthUser {
            token_id: token.to_string(),
        })
    }
}

pub struct OptionalUser {
    pub token_id: Option<String>,
}

impl<S: Send + Sync> FromRequestParts<S> for OptionalUser
where
    AppState: axum::extract::FromRef<S>,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("X-Ring-Token")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        Ok(OptionalUser { token_id: token })
    }
}
