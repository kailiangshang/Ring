# Add Member + Git Clone Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `POST /api/rings/:ring_id/members` endpoint so creators/admins can add registered users as Ring members, with async git clone when the Ring has a remote repo.

**Architecture:** New add-member query in models layer, service validates permissions + user existence + spawns clone task, handler parses request and delegates. Follows existing handler→service→model pattern.

**Tech Stack:** Rust, Axum, sqlx, tokio::spawn, git_service, archive_service::init_ring_repo

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `server/src/models/member.rs` | Modify | Add `add_member` query |
| `server/src/services/member.rs` | Modify | Add `add_member_service` with clone spawn |
| `server/src/routes/members.rs` | Modify | Add `add_member` handler + request struct |
| `server/src/routes/mod.rs` | Modify | Register POST route |
| `server/tests/integration.rs` | Modify | Add integration tests |

---

### Task 1: Add `add_member` query to models/member.rs

**Files:**
- Modify: `server/src/models/member.rs` (after `remove_member` function, line 80)

- [ ] **Step 1: Add the `add_member` function**

Add after `remove_member` at the end of the file:

```rust
pub async fn add_member(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
    user_id: &str,
    role: &str,
) -> Result<MemberResponse> {
    let result = sqlx::query(
        "INSERT INTO members (ring_id, user_id, role) VALUES (?1, ?2, ?3)",
    )
    .bind(ring_id)
    .bind(user_id)
    .bind(role)
    .execute(pool)
    .await;

    match result {
        Ok(_) => {
            let row = sqlx::query_as::<_, MemberRow>(
                "SELECT m.user_id, u.display_name, u.avatar, m.role, m.joined_at
                 FROM members m
                 JOIN users u ON u.token_id = m.user_id
                 WHERE m.ring_id = ?1 AND m.user_id = ?2",
            )
            .bind(ring_id)
            .bind(user_id)
            .fetch_one(pool)
            .await?;
            Ok(MemberResponse {
                token_id: row.user_id,
                display_name: row.display_name,
                avatar: row.avatar,
                role: row.role,
                joined_at: row.joined_at,
                online: false,
            })
        }
        Err(sqlx::Error::Database(ref db)) => {
            if db.code().is_some_and(|c| c == "2067") {
                Err(crate::error::RingError::Conflict(
                    "user is already a member".into(),
                ))
            } else {
                Err(crate::error::RingError::Internal(db.to_string()))
            }
        }
        Err(e) => Err(e.into()),
    }
}
```

This handles duplicate membership by catching SQLite UNIQUE constraint error (code 2067) and returning 409 Conflict.

- [ ] **Step 2: Run cargo check**

Run: `cargo check --manifest-path server/Cargo.toml`
Expected: compiles without errors

- [ ] **Step 3: Commit**

```bash
git add server/src/models/member.rs
git commit -m "feat: add add_member query to models/member"
```

---

### Task 2: Add `add_member_service` to services/member.rs

**Files:**
- Modify: `server/src/services/member.rs` (after `remove_member` function, line 47)

- [ ] **Step 1: Add the `add_member_service` function**

Add at the end of the file:

```rust
pub async fn add_member_service(
    state: &AppState,
    ring_id: &str,
    caller_id: &str,
    target_id: &str,
) -> Result<crate::models::member::MemberResponse> {
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
        let pool = state.db.clone();
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
                if let Err(e) = std::fs::create_dir_all(repo_path.join("archives")) {
                    tracing::warn!("failed to create archives dir: {e}");
                }
                if let Err(e) = std::fs::create_dir_all(repo_path.join("graphs")) {
                    tracing::warn!("failed to create graphs dir: {e}");
                }
                if let Err(e) = std::fs::create_dir_all(repo_path.join(".group")) {
                    tracing::warn!("failed to create .group dir: {e}");
                }
                tracing::info!("git clone completed: ring={ring_id}");
            }
        });
    }

    Ok(result)
}
```

Note: `get_user` returns `Err(RingError::NotFound)` if user doesn't exist, which maps to 404 response. The clone task creates archives/, graphs/, .group/ directories after clone to match the expected directory structure.

- [ ] **Step 2: Run cargo check**

Run: `cargo check --manifest-path server/Cargo.toml`
Expected: compiles without errors

- [ ] **Step 3: Commit**

```bash
git add server/src/services/member.rs
git commit -m "feat: add add_member_service with async git clone"
```

---

### Task 3: Add handler and register route

**Files:**
- Modify: `server/src/routes/members.rs`
- Modify: `server/src/routes/mod.rs`

- [ ] **Step 1: Add handler to routes/members.rs**

Add the request struct and handler at the end of `server/src/routes/members.rs`:

```rust
#[derive(Deserialize)]
pub struct AddMemberRequest {
    pub user_id: String,
}

pub async fn add_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<AddMemberRequest>,
) -> Result<Json<Value>> {
    let result = member::add_member_service(
        &state,
        &ring_id,
        &user.token_id,
        &body.user_id,
    )
    .await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}
```

- [ ] **Step 2: Register route in routes/mod.rs**

In `server/src/routes/mod.rs`, change the members route line from:

```rust
.route("/rings/{ring_id}/members", get(members::list_members))
```

to:

```rust
.route(
    "/rings/{ring_id}/members",
    get(members::list_members).post(members::add_member),
)
```

- [ ] **Step 3: Run cargo check**

Run: `cargo check --manifest-path server/Cargo.toml`
Expected: compiles without errors

- [ ] **Step 4: Commit**

```bash
git add server/src/routes/members.rs server/src/routes/mod.rs
git commit -m "feat: add POST /api/rings/:ring_id/members endpoint"
```

---

### Task 4: Integration tests

**Files:**
- Modify: `server/tests/integration.rs`

- [ ] **Step 1: Add helper function for creating additional users**

After the existing `create_ring` helper in `server/tests/integration.rs`, add:

```rust
async fn create_second_user(pool: &SqlitePool) -> String {
    let token_id = format!("user-test-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos());
    sqlx::query(
        "INSERT INTO users (token_id, display_name, avatar, is_creator, llm_provider, llm_api_key, llm_model, gitlab_url, gitlab_token)
         VALUES (?1, 'Bob', '🧑', 0, 'openai', 'sk-test', 'gpt-4o', 'https://gitlab.test.com', 'glpat-test')",
    )
    .bind(&token_id)
    .execute(pool)
    .await
    .unwrap();
    token_id
}
```

Note: We need direct DB insert because `PUT /api/setup` updates the existing user (doesn't create new), and `POST /api/setup` can only run once. Uses nanosecond timestamp for unique token_id — no extra crate dependency needed.

- [ ] **Step 2: Add test for add-member and duplicate rejection**

Add at the end of `server/tests/integration.rs`:

```rust
#[tokio::test]
async fn test_add_member() {
    let state = setup_app().await;
    let app = build_router(state.clone());
    let pool = state.db.clone();

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;
    let bob_token = create_second_user(&pool).await;

    let add_body = &format!(r#"{{"user_id":"{bob_token}"}}"#);
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/members"),
            Some(add_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["token_id"], bob_token);
    assert_eq!(json["role"], "member");

    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/members"),
            Some(add_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_add_member_forbidden() {
    let state = setup_app().await;
    let app = build_router(state.clone());
    let pool = state.db.clone();

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;
    let bob_token = create_second_user(&pool).await;

    let add_body = &format!(r#"{{"user_id":"{bob_token}"}}"#);
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/members"),
            Some(add_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/members"),
            Some(add_body),
            Some(&bob_token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
```

Note: `setup_app` returns `AppState` which has `db: SqlitePool`. We clone it to get a reference for the helper. The `build_router(state)` consumes state but `state.clone()` gives us a copy to keep the pool reference.

- [ ] **Step 3: Run the new tests**

Run: `cargo test --manifest-path server/Cargo.toml test_add_member`
Expected: both tests pass

- [ ] **Step 4: Run all tests**

Run: `cargo test --manifest-path server/Cargo.toml`
Expected: all tests pass

- [ ] **Step 5: Commit**

```bash
git add server/tests/integration.rs
git commit -m "test: add integration tests for add-member endpoint"
```

---

### Task 5: Final verification

- [ ] **Step 1: Run cargo clippy**

Run: `cargo clippy --manifest-path server/Cargo.toml -- -D warnings`
Expected: no warnings

- [ ] **Step 2: Run cargo fmt check**

Run: `cargo fmt --manifest-path server/Cargo.toml -- --check`
Expected: no formatting issues

- [ ] **Step 3: Run full test suite**

Run: `cargo test --manifest-path server/Cargo.toml`
Expected: all tests pass
