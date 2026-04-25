use axum::extract::{Multipart, Path, State};
use axum::Json;

use crate::error::{Result, RingError};
use crate::extractors::AuthUser;
use crate::models::message::MessageRow;
use crate::models::ring;
use crate::models::session::SessionMaterialRow;
use crate::state::AppState;

pub async fn upload_ring_file(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<MessageRow>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    ring::reject_readonly(&role)?;
    let user_row = state.get_user_decrypted(&user.token_id).await?;

    let (filename, data) = extract_file(&mut multipart).await?;

    let msg = crate::services::upload::upload_to_chat(
        &state.db,
        Some(&ring_id),
        &user.token_id,
        &user_row.display_name,
        &filename,
        &data,
    )
    .await?;

    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
    if let Err(e) = crate::services::self_data::record_tool_usage(&self_dir, "upload") {
        tracing::warn!("failed to record tool usage: {e}");
    }

    Ok(Json(msg))
}

pub async fn upload_super_file(
    State(state): State<AppState>,
    user: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<MessageRow>> {
    let user_row = state.get_user_decrypted(&user.token_id).await?;

    let (filename, data) = extract_file(&mut multipart).await?;

    let msg = crate::services::upload::upload_to_chat(
        &state.db,
        Some("super"),
        &user.token_id,
        &user_row.display_name,
        &filename,
        &data,
    )
    .await?;

    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
    if let Err(e) = crate::services::self_data::record_tool_usage(&self_dir, "upload") {
        tracing::warn!("failed to record tool usage: {e}");
    }

    Ok(Json(msg))
}

pub async fn upload_session_file(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
    mut multipart: Multipart,
) -> Result<Json<SessionMaterialRow>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    ring::reject_readonly(&role)?;
    let is_participant =
        crate::models::session::is_participant(&state.db, &session_id, &user.token_id).await?;
    if !is_participant {
        return Err(RingError::Forbidden("not a session participant".into()));
    }

    let (filename, data) = extract_file(&mut multipart).await?;

    let material = crate::services::upload::upload_to_session(
        &state.db,
        &ring_id,
        &session_id,
        &filename,
        &data,
    )
    .await?;

    let broadcast = serde_json::json!({
        "type": "session_material_added",
        "session_id": session_id,
        "material": &material,
    });
    state
        .ws_hub
        .broadcast_to_session(&session_id, &broadcast.to_string());

    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
    if let Err(e) = crate::services::self_data::record_tool_usage(&self_dir, "upload") {
        tracing::warn!("failed to record tool usage: {e}");
    }

    Ok(Json(material))
}

async fn extract_file(multipart: &mut Multipart) -> Result<(String, Vec<u8>)> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| RingError::BadRequest(format!("failed to read upload: {e}")))?
        .ok_or_else(|| RingError::BadRequest("no file provided".into()))?;

    let filename = field.file_name().unwrap_or("unnamed.txt").to_string();

    let data = field
        .bytes()
        .await
        .map_err(|e| RingError::BadRequest(format!("failed to read file data: {e}")))?
        .to_vec();

    Ok((filename, data))
}
