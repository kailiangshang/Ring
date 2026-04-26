use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::response::IntoResponse;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::Deserialize;
use tar::Builder;

use crate::error::Result;
use crate::extractors::auth::AuthUser;
use crate::models::{graph, message, ring};
use crate::state::AppState;

fn markdown_response(body: String, filename: String) -> impl IntoResponse {
    (
        [
            (
                header::CONTENT_TYPE,
                "text/markdown; charset=utf-8".to_string(),
            ),
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
            (
                header::CONTENT_TYPE,
                "application/json; charset=utf-8".to_string(),
            ),
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

    let messages =
        message::list_messages(&state.db, Some(&ring_id), &user.token_id, None, 10000).await?;

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

    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
    if let Err(e) = crate::services::self_data::record_tool_usage(&self_dir, "export") {
        tracing::warn!("failed to record tool usage: {e}");
    }

    Ok(markdown_response(md, format!("ring_{}_chat.md", ring_id)))
}

pub async fn export_self_chat(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<impl IntoResponse> {
    let messages = message::list_messages(&state.db, None, &user.token_id, None, 10000).await?;

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

    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
    if let Err(e) = crate::services::self_data::record_tool_usage(&self_dir, "export") {
        tracing::warn!("failed to record tool usage: {e}");
    }

    Ok(markdown_response(md, "self_chat.md".into()))
}

pub async fn export_super_chat(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<impl IntoResponse> {
    let messages = message::list_messages(&state.db, None, &user.token_id, None, 10000).await?;

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

    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
    if let Err(e) = crate::services::self_data::record_tool_usage(&self_dir, "export") {
        tracing::warn!("failed to record tool usage: {e}");
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

    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
    if let Err(e) = crate::services::self_data::record_tool_usage(&self_dir, "export") {
        tracing::warn!("failed to record tool usage: {e}");
    }

    Ok(json_response(
        json_str,
        format!("ring_{}_graph.json", ring_id),
    ))
}

pub async fn export_ring_backup(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<impl IntoResponse> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let ring_info = sqlx::query_as::<_, ring::RingRow>("SELECT * FROM rings WHERE id = ?1")
        .bind(&ring_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;

    let messages =
        message::list_messages(&state.db, Some(&ring_id), &user.token_id, None, 10000).await?;

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

    let metadata = serde_json::json!({
        "version": "1.0",
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "ring": {
            "id": ring_info.id,
            "name": ring_info.name,
            "role_description": ring_info.role_description,
            "interaction_mode": ring_info.interaction_mode,
            "created_at": ring_info.created_at,
        }
    });

    let graph_json = serde_json::json!({
        "nodes": nodes,
        "edges": edges,
    });

    let mut chat_md = String::new();
    chat_md.push_str(&format!("# Ring: {}\n\n", ring_info.name));
    chat_md.push_str(&format!(
        "Exported: {}\n\n---\n\n",
        chrono::Utc::now().to_rfc3339()
    ));
    for msg in messages.iter().rev() {
        let role_label = if msg.role == "user" { "User" } else { "AI" };
        chat_md.push_str(&format!("## {} ({})\n\n", role_label, msg.sender_name));
        chat_md.push_str(&msg.content);
        chat_md.push_str("\n\n---\n\n");
    }

    let sessions_json = serde_json::to_string_pretty(&sessions)
        .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;

    let archives_json = serde_json::to_string_pretty(&archives)
        .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;

    let mut buf = Vec::new();
    {
        let enc = GzEncoder::new(&mut buf, Compression::default());
        let mut tar = Builder::new(enc);
        tar_append(
            &mut tar,
            "metadata.json",
            &serde_json::to_string_pretty(&metadata)
                .map_err(|e| crate::error::RingError::Internal(e.to_string()))?,
        )?;
        tar_append(
            &mut tar,
            "graph.json",
            &serde_json::to_string_pretty(&graph_json)
                .map_err(|e| crate::error::RingError::Internal(e.to_string()))?,
        )?;
        tar_append(&mut tar, "chat.md", &chat_md)?;
        tar_append(&mut tar, "sessions.json", &sessions_json)?;
        tar_append(&mut tar, "archives.json", &archives_json)?;
        tar.finish()
            .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;
    }

    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
    if let Err(e) = crate::services::self_data::record_tool_usage(&self_dir, "export") {
        tracing::warn!("failed to record tool usage: {e}");
    }

    Ok((
        [
            (header::CONTENT_TYPE, "application/gzip".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!(r#"attachment; filename="ring_{}_backup.tar.gz""#, ring_id),
            ),
        ],
        buf,
    ))
}

fn tar_append(tar: &mut Builder<GzEncoder<&mut Vec<u8>>>, path: &str, content: &str) -> Result<()> {
    let data = content.as_bytes();
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tar.append_data(&mut header, path, data)
        .map_err(|e| crate::error::RingError::Internal(e.to_string()))
}

pub async fn export_session_messages(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let msgs = sqlx::query_as::<_, crate::models::session::SessionMessageRow>(
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

    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
    if let Err(e) = crate::services::self_data::record_tool_usage(&self_dir, "export") {
        tracing::warn!("failed to record tool usage: {e}");
    }

    Ok(markdown_response(md, format!("session_{}.md", session_id)))
}

#[derive(Debug, Deserialize)]
pub struct ReportQuery {
    pub node_ids: Option<String>,
    pub topic: Option<String>,
}

pub async fn export_ai_report(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Query(query): Query<ReportQuery>,
) -> Result<impl IntoResponse> {
    let user_row = state.get_user_decrypted(&user.token_id).await?;
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let graph = graph::ensure_default_graph(&state.db, &ring_id).await?;
    let all_nodes = graph::list_nodes(&state.db, &graph.id).await?;

    let selected_nodes: Vec<_> = if let Some(ids_str) = &query.node_ids {
        let ids: Vec<&str> = ids_str.split(',').collect();
        all_nodes
            .into_iter()
            .filter(|n| ids.contains(&n.id.as_str()))
            .collect()
    } else {
        all_nodes
    };

    if selected_nodes.is_empty() {
        return Err(crate::error::RingError::BadRequest(
            "No nodes selected".into(),
        ));
    }

    let topic = query.topic.as_deref().unwrap_or("综合分析");
    let mut nodes_info = String::new();
    for n in &selected_nodes {
        nodes_info.push_str(&format!(
            "### {} [{}]\n标签: {}\n路径: {}\n\n",
            n.label,
            n.node_type,
            n.tags,
            n.markdown_path.as_deref().unwrap_or("N/A"),
        ));
    }

    let system_prompt = crate::prompts::export::AI_REPORT_SYSTEM.to_string();
    let user_message = format!(
        "主题：{}\n\n以下是选中的图谱节点：\n\n{}",
        topic, nodes_info
    );

    let llm = crate::services::llm::LlmClient::from_user(&user_row)?;
    let report = llm.chat_complete(system_prompt, user_message).await?;

    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
    if let Err(e) = crate::services::self_data::record_tool_usage(&self_dir, "export") {
        tracing::warn!("failed to record tool usage: {e}");
    }

    Ok(markdown_response(
        report,
        format!("ring_{}_report.md", ring_id),
    ))
}

#[derive(Debug, Deserialize)]
pub struct ExportNodeQuery {
    pub node_id: String,
}

pub async fn export_node_markdown(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Query(query): Query<ExportNodeQuery>,
) -> Result<impl IntoResponse> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let node = sqlx::query_as::<_, graph::GraphNodeRow>(
        "SELECT n.* FROM graph_nodes n JOIN graphs g ON n.graph_id = g.id WHERE g.ring_id = ?1 AND n.id = ?2",
    )
    .bind(&ring_id)
    .bind(&query.node_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| crate::error::RingError::NotFound("node not found".into()))?;

    let md_content = if let Some(ref mp) = node.markdown_path {
        let full_path = state.rings_dir.join(&ring_id).join(mp);
        if full_path.exists() {
            std::fs::read_to_string(&full_path).unwrap_or_default()
        } else {
            node.content.clone()
        }
    } else {
        node.content.clone()
    };

    let label = node.label.replace(' ', "_");
    Ok(markdown_response(md_content, format!("{}.md", label)))
}
