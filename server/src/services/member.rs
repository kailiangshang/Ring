use crate::error::{Result, RingError};
use crate::models::member::{self, MemberResponse};
use crate::models::notification;
use crate::models::ring;
use crate::state::AppState;

pub async fn list_members(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
) -> Result<Vec<MemberResponse>> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    member::list_members(&state.db, ring_id).await
}

pub async fn update_member_role(
    state: &AppState,
    ring_id: &str,
    caller_id: &str,
    target_id: &str,
    new_role: &str,
) -> Result<()> {
    let caller_role = ring::get_user_role(&state.db, ring_id, caller_id).await?;
    if caller_role != "creator" {
        return Err(RingError::Forbidden("only creator can change roles".into()));
    }
    if new_role != "admin" && new_role != "member" && new_role != "readonly" {
        return Err(RingError::BadRequest("invalid role".into()));
    }
    member::update_role(&state.db, ring_id, target_id, new_role).await?;
    let ring_name = crate::services::search::get_ring_name(&state.db, ring_id)
        .await
        .unwrap_or_default();
    let _ = notification::create_notification(
        &state.db,
        &format!("notif-{}", ulid::Ulid::new()),
        &notification::CreateNotification {
            user_id: target_id.to_string(),
            ring_id: Some(ring_id.to_string()),
            notification_type: "role_changed".into(),
            title: "Role changed".into(),
            content: Some(format!(
                "Your role in Ring \"{}\" was changed to {}",
                ring_name, new_role
            )),
            related_id: None,
        },
    )
    .await;
    Ok(())
}

pub async fn remove_member(
    state: &AppState,
    ring_id: &str,
    caller_id: &str,
    target_id: &str,
) -> Result<()> {
    let caller_role = ring::get_user_role(&state.db, ring_id, caller_id).await?;
    if caller_role != "creator" && caller_role != "admin" {
        return Err(RingError::Forbidden(
            "only creator or admin can remove members".into(),
        ));
    }
    let active_session_owner: Option<(String, String)> = sqlx::query_as::<_, (String, String)>(
        "SELECT s.id, s.title FROM sessions s WHERE s.ring_id = ?1 AND s.owner = ?2 AND s.phase != 'closed'",
    )
    .bind(ring_id)
    .bind(target_id)
    .fetch_optional(&state.db)
    .await?;

    if let Some((sid, title)) = active_session_owner {
        return Err(RingError::BadRequest(format!(
            "Cannot remove: user owns active session \"{}\" ({}). Transfer ownership first.",
            title, sid
        )));
    }

    member::remove_member(&state.db, ring_id, target_id).await?;
    let ring_name = crate::services::search::get_ring_name(&state.db, ring_id)
        .await
        .unwrap_or_default();
    let _ = notification::create_notification(
        &state.db,
        &format!("notif-{}", ulid::Ulid::new()),
        &notification::CreateNotification {
            user_id: target_id.to_string(),
            ring_id: Some(ring_id.to_string()),
            notification_type: "member_removed".into(),
            title: "Removed from Ring".into(),
            content: Some(format!("You were removed from Ring \"{}\"", ring_name)),
            related_id: None,
        },
    )
    .await;
    Ok(())
}

pub async fn add_member_service(
    state: &AppState,
    ring_id: &str,
    caller_id: &str,
    target_id: &str,
) -> Result<MemberResponse> {
    let caller_role = ring::get_user_role(&state.db, ring_id, caller_id).await?;
    if caller_role != "creator" && caller_role != "admin" {
        return Err(RingError::Forbidden(
            "only creator or admin can add members".into(),
        ));
    }

    let _target = crate::models::user::get_user(&state.db, target_id).await?;

    let result = member::add_member(&state.db, ring_id, target_id, "member").await?;

    tracing::info!("member added: user={target_id}, ring={ring_id}");

    let repo_url: Option<String> =
        sqlx::query_scalar("SELECT gitlab_repo_url FROM rings WHERE id = ?1")
            .bind(ring_id)
            .fetch_optional(&state.db)
            .await?
            .flatten();

    if let Some(url) = repo_url {
        let rings_dir = state.rings_dir.clone();
        let ring_id = ring_id.to_string();
        tracing::info!("spawning git clone for ring {ring_id}");
        tokio::spawn(async move {
            let git = crate::services::git_service::GitService::new();
            let repo_path = rings_dir.join(&ring_id);
            if repo_path.join(".git").exists() {
                if let Err(e) = git.pull(&repo_path) {
                    tracing::warn!("git pull failed for ring {ring_id}: {e}");
                }
            } else {
                if let Err(e) = crate::services::git_service::GitService::clone(&url, &repo_path) {
                    tracing::warn!("git clone failed for ring {ring_id}: {e}");
                    return;
                }
                let _ = std::fs::create_dir_all(repo_path.join("archives"));
                let _ = std::fs::create_dir_all(repo_path.join("graphs"));
                let _ = std::fs::create_dir_all(repo_path.join(".group"));
                tracing::info!("git clone completed: ring={ring_id}");
            }
        });
    }

    Ok(result)
}
