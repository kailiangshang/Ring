use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Html;
use serde::Deserialize;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct JoinQuery {
    pub token: Option<String>,
    pub creator_ip: Option<String>,
}

pub async fn join_page(
    Query(query): Query<JoinQuery>,
    State(state): State<AppState>,
) -> Result<(StatusCode, Html<String>), (StatusCode, Html<String>)> {
    let token = query.token.as_deref().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Html(error_html("Bad Request", "Missing invite token.")),
        )
    })?;

    let invite = state
        .db
        .get_invite_token(token)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(error_html("Error", "Internal server error.")),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Html(error_html(
                    "Not Found",
                    "Invite token not found or expired.",
                )),
            )
        })?;

    if invite.revoked_at.is_some() {
        return Err((
            StatusCode::NOT_FOUND,
            Html(error_html("Not Found", "Invite token has been revoked.")),
        ));
    }

    let expires = chrono::DateTime::parse_from_rfc3339(&invite.expires_at);
    if let Ok(exp) = expires {
        if exp < chrono::Utc::now() {
            return Err((
                StatusCode::NOT_FOUND,
                Html(error_html("Not Found", "Invite token has expired.")),
            ));
        }
    }

    let ring = state
        .db
        .get_ring(&invite.ring_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(error_html("Error", "Internal server error.")),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Html(error_html("Not Found", "Ring not found.")),
            )
        })?;

    let member_count = state.db.count_members_by_ring(&ring.id).await.unwrap_or(0);

    let release_repo = &state.config.release_repo;
    let join_data = serde_json::json!({
        "ring_name": ring.name,
        "ring_description": ring.description.unwrap_or_default(),
        "member_count": member_count,
        "token": token,
        "creator_ip": query.creator_ip.unwrap_or_default(),
        "downloads": {
            "windows": format!("{}/releases/latest/download/ring-server-windows-x86_64.exe", release_repo),
            "linux": format!("{}/releases/latest/download/ring-server-linux-x86_64", release_repo),
            "macos_arm": format!("{}/releases/latest/download/ring-server-macos-aarch64", release_repo),
            "macos_intel": format!("{}/releases/latest/download/ring-server-macos-x86_64", release_repo),
        }
    });

    let template = include_str!("../../templates/install_guide.html");
    let html = template.replace(
        "window.__RING_JOIN_DATA__",
        &format!(
            "window.__RING_JOIN_DATA__={}",
            serde_json::to_string(&join_data).unwrap()
        ),
    );

    Ok((StatusCode::OK, Html(html)))
}

fn error_html(title: &str, message: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>{title}</title>
<style>*{{margin:0;padding:0;box-sizing:border-box}}body{{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;background:#f5f5f7;color:#1d1d1f;min-height:100vh;display:flex;align-items:center;justify-content:center}}.error{{text-align:center;padding:60px 32px;background:#fff;border-radius:16px;box-shadow:0 2px 12px rgba(0,0,0,0.08);max-width:400px;width:100%}}h1{{font-size:20px;color:#ff3b30;margin-bottom:8px}}p{{color:#86868b;font-size:15px}}</style>
</head><body><div class="error"><h1>{title}</h1><p>{message}</p></div></body></html>"#,
        title = title,
        message = message
    )
}
