use axum::extract::{Path, State};
use axum::http::header;
use axum::response::IntoResponse;

use crate::error::Result;
use crate::extractors::auth::AuthUser;
use crate::models::{graph, message, ring};
use crate::state::AppState;

fn markdown_response(body: String, filename: String) -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "text/markdown; charset=utf-8".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!(r#"attachment; filename="{}""#, filename),
            ),
        ],
        body,
    )
}

fn json_response(body: String, filename: String) -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "application/json; charset=utf-8".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!(r#"attachment; filename="{}""#, filename),
            ),
        ],
        body,
    )
}

pub async fn export_ring_chat(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<impl IntoResponse> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let messages = message::list_messages(
        &state.db, Some(&ring_id), &user.token_id, None, 10000).await?;

    let mut md = String::new();
    md.push_str(&format!("# Chat Export - Ring {}\n\n", ring_id));
    md.push_str(&format!("Exported by: {}\n", user.token_id));
    md.push_str(&format!("Total messages: {}\n\n", messages.len()));

    for msg in messages.iter().rev() {
        let role_label = if msg.role == "user" { "User" } else { "AI" };
        md.push_str(&format!("## {} ({})\n\n", role_label, msg.sender_name));
        md.push_str(&msg.content);
        md.push_str("\n\n---\n\n");
    }

    Ok(markdown_response(md, format!("ring_{}_chat.md", ring_id)))
}

pub async fn export_self_chat(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<impl IntoResponse> {
    let messages = message::list_messages(
        &state.db, None, &user.token_id, None, 10000).await?;

    let mut md = String::new();
    md.push_str("# Self Chat Export\n\n");
    md.push_str(&format!("Exported by: {}\n", user.token_id));
    md.push_str(&format!("Total messages: {}\n\n", messages.len()));

    for msg in messages.iter().rev() {
        let role_label = if msg.role == "user" { "User" } else { "AI" };
        md.push_str(&format!("## {} ({})\n\n", role_label, msg.sender_name));
        md.push_str(&msg.content);
        md.push_str("\n\n---\n\n");
    }

    Ok(markdown_response(md, "self_chat.md".into()))
}

pub async fn export_super_chat(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<impl IntoResponse> {
    let messages = message::list_messages(
        &state.db, None, &user.token_id, None, 10000).await?;

    let mut md = String::new();
    md.push_str("# Super Ring Chat Export\n\n");
    md.push_str(&format!("Exported by: {}\n", user.token_id));
    md.push_str(&format!("Total messages: {}\n\n", messages.len()));

    for msg in messages.iter().rev() {
        let role_label = if msg.role == "user" { "User" } else { "AI" };
        md.push_str(&format!("## {} ({})\n\n", role_label, msg.sender_name));
        md.push_str(&msg.content);
        md.push_str("\n\n---\n\n");
    }

    Ok(markdown_response(md, "super_chat.md".into()))
}

pub async fn export_ring_graph(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<impl IntoResponse> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let graph = graph::ensure_default_graph(&state.db, &ring_id).await?;
    let nodes = graph::list_nodes(&state.db, &graph.id).await?;
    let edges = graph::list_edges(&state.db, &graph.id).await?;

    let json = serde_json::json!({
        "ring_id": graph.ring_id,
        "name": graph.name,
        "nodes": nodes,
        "edges": edges,
        "updated_at": graph.updated_at,
    });

    let json_str = serde_json::to_string_pretty(&json)
        .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;

    Ok(json_response(json_str, format!("ring_{}_graph.json", ring_id)))
}

pub async fn export_ring_backup(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<impl IntoResponse> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let ring_info = sqlx::query_as::<_, ring::RingRow>(
        "SELECT * FROM rings WHERE id = ?1",
    )
    .bind(&ring_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;

    let messages = message::list_messages(
        &state.db, Some(&ring_id), &user.token_id, None, 10000).await?;

    let graph = graph::ensure_default_graph(&state.db, &ring_id).await?;
    let nodes = graph::list_nodes(&state.db, &graph.id).await?;
    let edges = graph::list_edges(&state.db, &graph.id).await?;

    let sessions = sqlx::query_as::<_, crate::models::session::SessionRow>(
        "SELECT * FROM sessions WHERE ring_id = ?1",
    )
    .bind(&ring_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;

    let archives = sqlx::query_as::<_, crate::models::archive::ArchiveRecord>(
        "SELECT * FROM archives WHERE ring_id = ?1",
    )
    .bind(&ring_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;

    let backup = serde_json::json!({
        "version": "1.0",
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "ring": {
            "id": ring_info.id,
            "name": ring_info.name,
            "role_description": ring_info.role_description,
            "interaction_mode": ring_info.interaction_mode,
            "created_at": ring_info.created_at,
        },
        "messages": messages.iter().rev().collect::<Vec<_>>(),
        "graph": {
            "nodes": nodes,
            "edges": edges,
        },
        "sessions": sessions,
        "archives": archives,
    });

    let json_str = serde_json::to_string_pretty(&backup)
        .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;

    Ok(json_response(json_str, format!("ring_{}_backup.json", ring_id)))
}

pub async fn export_session_messages(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let msgs = sqlx::query_as::<
        _,
        crate::models::session::SessionMessageRow,
    >(
        "SELECT * FROM session_messages WHERE session_id = ?1 ORDER BY seq_num",
    )
    .bind(&session_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;

    let mut md = String::new();
    md.push_str(&format!("# Session Export - {}\n\n", session_id));
    md.push_str(&format!("Ring: {}\n", ring_id));
    md.push_str(&format!("Total messages: {}\n\n", msgs.len()));

    for msg in msgs {
        let role_label = match msg.message_type.as_str() {
            "user" => "User",
            "system" => "System",
            _ => "AI",
        };
        md.push_str(&format!("## {} ({})\n\n", role_label, msg.sender_name));
        md.push_str(&msg.content);
        md.push_str("\n\n---\n\n");
    }

    Ok(markdown_response(md, format!("session_{}.md", session_id)))
}
