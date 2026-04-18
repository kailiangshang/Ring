use crate::error::{Result, RingError};
use crate::models::member::{self, MemberResponse};
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
    if caller_role != "creator" && caller_role != "admin" {
        return Err(RingError::Forbidden(
            "only creator or admin can change roles".into(),
        ));
    }
    if new_role != "admin" && new_role != "member" && new_role != "readonly" {
        return Err(RingError::BadRequest("invalid role".into()));
    }
    member::update_role(&state.db, ring_id, target_id, new_role).await
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
    member::remove_member(&state.db, ring_id, target_id).await
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

    let repo_url: Option<String> = sqlx::query_scalar(
        "SELECT gitlab_repo_url FROM rings WHERE id = ?1",
    )
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
                if let Err(e) =
                    crate::services::git_service::GitService::clone(&url, &repo_path)
                {
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
