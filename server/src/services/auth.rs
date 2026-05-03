use crate::state::AppState;

pub async fn rotate_token(state: &AppState, current_token: &str) -> crate::error::Result<String> {
    let new_token = format!("user-{}", ulid::Ulid::new());
    sqlx::query(
        "UPDATE users SET token_id = ?1, token_created_at = datetime('now') WHERE token_id = ?2",
    )
    .bind(&new_token)
    .bind(current_token)
    .execute(&state.db)
    .await?;
    Ok(new_token)
}
