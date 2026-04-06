# Phase 1 Implementation Plan — Basic Framework (TDD)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 搭建 Ring 后端骨架 + 前端骨架，实现 Ring CRUD + Setup 向导 + 安装导航页，可端到端运行。

**Architecture:** Rust + Axum 后端，SQLite 存储，petgraph 内存图。每个模块独立测试（单元测试 + Axum test router 集成测试）。前端 React + TS + Zustand。

**Tech Stack:** Rust/Axum/sqlx/petgraph/git2, React/TypeScript/Vite/Zustand, Playwright (E2E)

**Reference docs:**
- `docs/technical/developer-guide.md` — 项目结构、依赖、迁移脚本、错误类型、AppState、路由
- `docs/technical/api-design.md` — API 端点定义
- `docs/technical/data-model.md` — 数据模型
- `docs/technical/test-cases.md` — 测试用例设计

---

## File Structure

```
ring-server/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── error.rs
│   ├── state.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   ├── ring.rs
│   │   └── member.rs
│   ├── db/
│   │   ├── mod.rs
│   │   ├── traits.rs
│   │   └── sqlite.rs
│   ├── graph/
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   └── petgraph_store.rs
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── setup.rs
│   │   ├── ring.rs
│   │   └── install.rs
│   ├── services/
│   │   ├── mod.rs
│   │   └── ring_service.rs
│   └── routes.rs
├── migrations/
│   └── 001_initial.sql
└── templates/
    └── install_guide.html

ring-frontend/
├── package.json
├── vite.config.ts
├── tsconfig.json
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── api/
│   │   └── client.ts
│   ├── stores/
│   │   └── setupStore.ts
│   ├── pages/
│   │   ├── Setup/
│   │   │   ├── SetupWizard.tsx
│   │   │   ├── StepUsername.tsx
│   │   │   ├── StepLlm.tsx
│   │   │   └── StepGitlab.tsx
│   │   └── RingHub/
│   │       ├── RingHub.tsx
│   │       ├── RingList.tsx
│   │       └── CreateRing.tsx
│   └── types/
│       └── index.ts
└── index.html
```

---

## Module 1: Error Types

**Files:**
- Create: `ring-server/src/error.rs`

- [ ] **Step 1: Create Cargo project + write error test**

Create `ring-server/Cargo.toml` with all dependencies from `docs/technical/developer-guide.md` section 2.

Create `ring-server/src/lib.rs` with `pub mod error;`

Write test in `ring-server/src/error.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::body::Body;
    use axum::extract::Request;
    use http_body_util::BodyExt;
    use serde_json::Value;

    async fn body_to_json(body: Body) -> Value {
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn not_found_maps_to_404() {
        let err = RingError::NotFound("ring not found".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn unauthorized_maps_to_401() {
        let err = RingError::Unauthorized("not setup".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn forbidden_maps_to_403() {
        let err = RingError::Forbidden("members cannot edit".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn conflict_maps_to_409() {
        let err = RingError::Conflict("already setup".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn validation_maps_to_400() {
        let err = RingError::Validation("name required".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn internal_errors_hide_details() {
        let err = RingError::Internal("database exploded".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn database_error_is_internal() {
        let sqlx_err = sqlx::Error::RowNotFound;
        let err = RingError::from(sqlx_err);
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn response_body_contains_error_field() {
        let err = RingError::NotFound("ring-123".into());
        let resp = err.into_response();
        let (parts, body) = resp.into_parts();
        let json = body_to_json(body).await;
        assert_eq!(json["error"], "not found: ring-123");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd ring-server && cargo test --lib error`
Expected: FAIL — `RingError` not defined

- [ ] **Step 3: Write minimal implementation**

Implement `ring-server/src/error.rs` with the `RingError` enum and `IntoResponse` impl from `docs/technical/developer-guide.md` section 4.

- [ ] **Step 4: Run test to verify it passes**

Run: `cd ring-server && cargo test --lib error`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add ring-server/
git commit -m "feat: init cargo project with error types and tests"
```

---

## Module 2: Config

**Files:**
- Create: `ring-server/src/config.rs`
- Modify: `ring-server/src/lib.rs` — add `pub mod config;`

- [ ] **Step 1: Write config test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_correct_port() {
        let config = Config::default();
        assert_eq!(config.port, 7420);
    }

    #[test]
    fn default_config_has_data_dir() {
        let config = Config::default();
        assert!(config.data_dir.to_string_lossy().contains(".ring"));
    }

    #[test]
    fn default_config_has_release_repo() {
        let config = Config::default();
        assert!(!config.release_repo.is_empty());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd ring-server && cargo test --lib config`
Expected: FAIL

- [ ] **Step 3: Implement Config struct**

`Config` with fields: `port: u16`, `data_dir: PathBuf`, `release_repo: String`, `database_url: String`. Implement `Default`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cd ring-server && cargo test --lib config`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/config.rs ring-server/src/lib.rs
git commit -m "feat: add config module with defaults"
```

---

## Module 3: Data Models

**Files:**
- Create: `ring-server/src/models/mod.rs`
- Create: `ring-server/src/models/user.rs`
- Create: `ring-server/src/models/ring.rs`
- Create: `ring-server/src/models/member.rs`

- [ ] **Step 1: Write model tests**

Test serialization/deserialization for `User`, `NewUser`, `Ring`, `NewRing`, `Member`. Verify JSON field names are `snake_case`.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_serializes_snake_case() {
        let user = User {
            id: "uuid-1".into(),
            display_name: "张三".into(),
            avatar_url: None,
            ip_address: Some("192.168.1.1".into()),
            setup_completed: true,
            created_at: "2026-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_value(&user).unwrap();
        assert_eq!(json["display_name"], "张三");
        assert_eq!(json["setup_completed"], true);
    }

    #[test]
    fn new_ring_requires_name() {
        let json = r#"{"name": "test", "role_description": "expert"}"#;
        let ring: NewRing = serde_json::from_str(json).unwrap();
        assert_eq!(ring.name, "test");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd ring-server && cargo test --lib models`
Expected: FAIL

- [ ] **Step 3: Implement data models**

Define structs matching `docs/technical/data-model.md` tables. `Serialize`/`Deserialize` on all.

- [ ] **Step 4: Run test to verify it passes**

Run: `cd ring-server && cargo test --lib models`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/models/
git commit -m "feat: add data models (User, Ring, Member)"
```

---

## Module 4: SQLite Migration + Repository

**Files:**
- Create: `ring-server/migrations/001_initial.sql`
- Create: `ring-server/src/db/mod.rs`
- Create: `ring-server/src/db/traits.rs`
- Create: `ring-server/src/db/sqlite.rs`

- [ ] **Step 1: Write repository tests**

Test `SqliteRepository` against in-memory SQLite (`sqlite::memory:`):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> SqliteRepository {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        SqliteRepository::new(pool)
    }

    #[tokio::test]
    async fn create_and_get_user() {
        let repo = setup_test_db().await;
        let user = repo.create_user(NewUser {
            display_name: "张三".into(),
        }).await.unwrap();
        assert_eq!(user.display_name, "张三");
        assert!(user.id.len() > 0);

        let fetched = repo.get_user(&user.id).await.unwrap().unwrap();
        assert_eq!(fetched.id, user.id);
    }

    #[tokio::test]
    async fn create_and_list_rings() {
        let repo = setup_test_db().await;
        let user = repo.create_user(NewUser { display_name: "张三".into() }).await.unwrap();

        let ring = repo.create_ring(NewRing {
            name: "竞品分析".into(),
            description: Some("desc".into()),
            creator_id: user.id.clone(),
            gitlab_repo: "auto_create".into(),
            namespace: None,
            role_description: "产品专家".into(),
        }).await.unwrap();
        assert_eq!(ring.name, "竞品分析");

        let rings = repo.list_rings_by_user(&user.id).await.unwrap();
        assert_eq!(rings.len(), 1);
    }

    #[tokio::test]
    async fn setup_status_defaults_to_false() {
        let repo = setup_test_db().await;
        let status = repo.is_setup_completed().await.unwrap();
        assert!(!status);
    }

    #[tokio::test]
    async fn complete_setup_sets_flag() {
        let repo = setup_test_db().await;
        repo.complete_setup("user-1").await.unwrap();
        let status = repo.is_setup_completed().await.unwrap();
        assert!(status);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd ring-server && cargo test --lib db`
Expected: FAIL

- [ ] **Step 3: Create migration + implement repository**

- Copy migration SQL from `docs/technical/developer-guide.md` section 3 into `ring-server/migrations/001_initial.sql`
- Define `Repository` trait in `db/traits.rs` (methods: `create_user`, `get_user`, `create_ring`, `get_ring`, `list_rings_by_user`, `update_ring`, `delete_ring`, `is_setup_completed`, `complete_setup`)
- Implement `SqliteRepository` in `db/sqlite.rs`

- [ ] **Step 4: Run test to verify it passes**

Run: `cd ring-server && cargo test --lib db`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/db/ ring-server/migrations/
git commit -m "feat: add SQLite migration and repository with tests"
```

---

## Module 5: Graph Store (petgraph)

**Files:**
- Create: `ring-server/src/graph/mod.rs`
- Create: `ring-server/src/graph/types.rs`
- Create: `ring-server/src/graph/petgraph_store.rs`

- [ ] **Step 1: Write graph store tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn new_store() -> PetgraphStore {
        PetgraphStore::new()
    }

    #[tokio::test]
    async fn create_and_get_node() {
        let store = new_store();
        let node = store.create_node("graph-1", NewNode {
            label: "竞品A".into(),
            node_type: "concept".into(),
            parent_id: None,
            description: Some("分析".into()),
        }).await.unwrap();
        assert_eq!(node.label, "竞品A");

        let fetched = store.get_node("graph-1", &node.id).await.unwrap().unwrap();
        assert_eq!(fetched.id, node.id);
    }

    #[tokio::test]
    async fn create_edge() {
        let store = new_store();
        let n1 = store.create_node("graph-1", NewNode { label: "A".into(), node_type: "concept".into(), parent_id: None, description: None }).await.unwrap();
        let n2 = store.create_node("graph-1", NewNode { label: "B".into(), node_type: "concept".into(), parent_id: None, description: None }).await.unwrap();
        let edge = store.create_edge("graph-1", NewEdge {
            source_id: n1.id.clone(),
            target_id: n2.id.clone(),
            relation: "depends_on".into(),
            label: Some("依赖".into()),
        }).await.unwrap();
        assert_eq!(edge.source_id, n1.id);
    }

    #[tokio::test]
    async fn delete_node_removes_children_edges() {
        let store = new_store();
        let parent = store.create_node("graph-1", NewNode { label: "P".into(), node_type: "category".into(), parent_id: None, description: None }).await.unwrap();
        let child = store.create_node("graph-1", NewNode { label: "C".into(), node_type: "concept".into(), parent_id: Some(parent.id.clone()), description: None }).await.unwrap();
        store.delete_node("graph-1", &parent.id).await.unwrap();
        assert!(store.get_node("graph-1", &child.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn export_and_import_graph_json() {
        let store = new_store();
        let n = store.create_node("graph-1", NewNode { label: "X".into(), node_type: "concept".into(), parent_id: None, description: None }).await.unwrap();
        let exported = store.export_graph_json("graph-1").await.unwrap();

        let store2 = new_store();
        store2.import_graph_json("graph-1", &exported).await.unwrap();
        let fetched = store2.get_node("graph-1", &n.id).await.unwrap().unwrap();
        assert_eq!(fetched.label, "X");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd ring-server && cargo test --lib graph`
Expected: FAIL

- [ ] **Step 3: Implement PetgraphStore**

Define `GraphStore` trait, `NewNode`, `NewEdge`, `NodeData`, `EdgeData`, `GraphJson` types. Implement `PetgraphStore` with `StableDiGraph<NodeData, EdgeData>` + `HashMap` indexes.

- [ ] **Step 4: Run test to verify it passes**

Run: `cd ring-server && cargo test --lib graph`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/graph/
git commit -m "feat: add petgraph store with CRUD and import/export"
```

---

## Module 6: AppState + Services

**Files:**
- Create: `ring-server/src/state.rs`
- Create: `ring-server/src/services/mod.rs`
- Create: `ring-server/src/services/ring_service.rs`

- [ ] **Step 1: Write ring service tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_service() -> RingService {
        // create in-memory db + graph store, return RingService
    }

    #[tokio::test]
    async fn create_ring_initializes_repo() {
        let svc = setup_service().await;
        let ring = svc.create_ring(CreateRingRequest {
            name: "竞品分析".into(),
            description: None,
            role_description: "专家".into(),
            gitlab_repo: "auto_create".into(),
            namespace: None,
        }).await.unwrap();
        assert_eq!(ring.name, "竞品分析");
        assert_eq!(ring.status, "blueprint_pending");
    }

    #[tokio::test]
    async fn create_ring_validates_name() {
        let svc = setup_service().await;
        let result = svc.create_ring(CreateRingRequest {
            name: "".into(),
            description: None,
            role_description: "专家".into(),
            gitlab_repo: "auto_create".into(),
            namespace: None,
        }).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delete_nonexistent_ring_fails() {
        let svc = setup_service().await;
        let result = svc.delete_ring("nonexistent").await;
        assert!(matches!(result, Err(RingError::NotFound(_))));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd ring-server && cargo test --lib services`
Expected: FAIL

- [ ] **Step 3: Implement AppState + RingService**

- `AppState` with `db`, `graph_store`, `config`
- `RingService` with create/get/list/update/delete logic, validation

- [ ] **Step 4: Run test to verify it passes**

Run: `cd ring-server && cargo test --lib services`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/state.rs ring-server/src/services/
git commit -m "feat: add AppState and RingService with validation"
```

---

## Module 7: Setup Handler (API Integration Test)

**Files:**
- Create: `ring-server/src/handlers/mod.rs`
- Create: `ring-server/src/handlers/setup.rs`
- Create: `ring-server/tests/setup_integration.rs`

- [ ] **Step 1: Write setup API integration test**

Test against Axum test router with in-memory DB:

```rust
#[tokio::test]
async fn full_setup_wizard_flow() {
    let app = create_test_app().await;

    // Step 1: check status — not setup
    let resp = app.clone().oneshot(
        Request::builder().uri("/api/v1/setup/status").body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = body_to_json(resp).await;
    assert_eq!(body["setup_completed"], false);

    // Step 2: set username
    let resp = app.clone().oneshot(
        Request::builder().method("POST").uri("/api/v1/setup/username")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"display_name": "张三"}"#)).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), 200);

    // Step 3: set LLM
    let resp = app.clone().oneshot(
        Request::builder().method("POST").uri("/api/v1/setup/llm")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"provider": "openai", "model": "gpt-4", "api_key": "sk-xxx"}"#)).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), 200);

    // Step 4: set GitLab
    let resp = app.clone().oneshot(
        Request::builder().method("POST").uri("/api/v1/setup/gitlab")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"repo_url": "git@gitlab.corp:user/ring.git", "auth_type": "ssh_key", "ssh_key_path": "~/.ssh/id_rsa"}"#)).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), 200);

    // Step 5: complete
    let resp = app.clone().oneshot(
        Request::builder().method("POST").uri("/api/v1/setup/complete")
            .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), 200);

    // Step 6: verify status
    let resp = app.clone().oneshot(
        Request::builder().uri("/api/v1/setup/status").body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: Value = body_to_json(resp).await;
    assert_eq!(body["setup_completed"], true);
}

#[tokio::test]
async fn setup_rejects_empty_username() {
    let app = create_test_app().await;
    let resp = app.clone().oneshot(
        Request::builder().method("POST").uri("/api/v1/setup/username")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"display_name": ""}"#)).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn setup_twice_returns_conflict() {
    let app = create_test_app().await;
    // complete full setup...
    // then try again
    let resp = app.clone().oneshot(
        Request::builder().method("POST").uri("/api/v1/setup/username")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"display_name": "李四"}"#)).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), 409);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd ring-server && cargo test --test setup_integration`
Expected: FAIL

- [ ] **Step 3: Implement setup handler**

`handlers/setup.rs` — 5 endpoints: `get_status`, `set_username`, `set_llm`, `set_gitlab`, `complete`. Each handler: parse JSON → call service → return JSON.

- [ ] **Step 4: Run test to verify it passes**

Run: `cd ring-server && cargo test --test setup_integration`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/handlers/ ring-server/tests/
git commit -m "feat: add setup handler with integration tests"
```

---

## Module 8: Ring CRUD Handler (API Integration Test)

**Files:**
- Create: `ring-server/src/handlers/ring.rs`
- Create: `ring-server/tests/ring_integration.rs`
- Create: `ring-server/src/routes.rs`

- [ ] **Step 1: Write ring CRUD integration tests**

Test TC-P1-002 and TC-P1-003 from `docs/technical/test-cases.md`:

- `create_ring` returns 201 with ring_id
- `get_ring` returns ring details
- `list_rings` returns array
- `update_ring` modifies name/description
- `delete_ring` returns 204, then GET returns 404
- `create_ring` with empty name returns 400
- `get_ring` with nonexistent id returns 404

- [ ] **Step 2: Run test to verify it fails**

Run: `cd ring-server && cargo test --test ring_integration`
Expected: FAIL

- [ ] **Step 3: Implement ring handler + routes**

`handlers/ring.rs` — list/get/create/update/delete.
`routes.rs` — `build_router()` assembling all routes.

- [ ] **Step 4: Run test to verify it passes**

Run: `cd ring-server && cargo test --test ring_integration`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/handlers/ring.rs ring-server/src/routes.rs
git commit -m "feat: add ring CRUD handler and route registration"
```

---

## Module 9: Install Guide Page

**Files:**
- Create: `ring-server/src/handlers/install.rs`
- Create: `ring-server/templates/install_guide.html`
- Create: `ring-server/tests/install_integration.rs`

- [ ] **Step 1: Write install page integration tests**

Test TC-P1-004 from test cases:

```rust
#[tokio::test]
async fn join_page_with_valid_token_returns_html() {
    let app = create_test_app_with_invite_token("tok-123").await;
    let resp = app.clone().oneshot(
        Request::builder().uri("/join?token=tok-123").body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), 200);
    assert!(resp.headers().get("content-type").unwrap().to_str().unwrap().contains("text/html"));
}

#[tokio::test]
async fn join_page_html_contains_ring_data() {
    // verify HTML body contains window.__RING_JOIN_DATA__ with ring_name, downloads
}

#[tokio::test]
async fn join_page_invalid_token_returns_404() {
    let resp = ... // GET /join?token=invalid
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn join_page_missing_token_returns_400() {
    let resp = ... // GET /join
    assert_eq!(resp.status(), 400);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd ring-server && cargo test --test install_integration`
Expected: FAIL

- [ ] **Step 3: Implement install handler**

`handlers/install.rs` — `join_page()` handler. Read HTML template via `include_str!`, inject `window.__RING_JOIN_DATA__` with ring info + download URLs. Validate token, return HTML.

- [ ] **Step 4: Run test to verify it passes**

Run: `cd ring-server && cargo test --test install_integration`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/handlers/install.rs ring-server/templates/
git commit -m "feat: add decentralized install guide page"
```

---

## Module 10: main.rs — Wire Everything

**Files:**
- Create: `ring-server/src/main.rs`

- [ ] **Step 1: Write startup test**

```rust
#[tokio::test]
async fn app_starts_and_serves_setup_status() {
    // spawn server on random port, hit /api/v1/setup/status, verify 200
}
```

- [ ] **Step 2: Implement main.rs**

Initialize config → connect SQLite → run migrations → create AppState → build router → start Axum on port 7420.

- [ ] **Step 3: Run all tests**

Run: `cd ring-server && cargo test`
Expected: ALL PASS

- [ ] **Step 4: Manual smoke test**

```bash
cd ring-server && cargo run
curl http://localhost:7420/api/v1/setup/status
# Expected: {"setup_completed": false, "step": "username"}
```

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/main.rs
git commit -m "feat: wire main.rs, backend fully runnable"
```

---

## Module 11: Frontend Scaffolding

**Files:**
- Initialize `ring-frontend/` with Vite + React + TS

- [ ] **Step 1: Initialize project**

```bash
cd ring-frontend && npm create vite@latest . -- --template react-ts
npm install zustand react-router-dom
npm install -D vitest @testing-library/react @testing-library/jest-dom jsdom
```

- [ ] **Step 2: Verify dev server starts**

Run: `cd ring-frontend && npm run dev`
Expected: dev server starts on port 5173

- [ ] **Step 3: Verify test runner works**

Write a placeholder test, run `npm test`, verify it passes.

- [ ] **Step 4: Commit**

```bash
git add ring-frontend/
git commit -m "feat: init frontend with Vite + React + TS + Zustand"
```

---

## Module 12: Frontend — API Client + Types

**Files:**
- Create: `ring-frontend/src/types/index.ts`
- Create: `ring-frontend/src/api/client.ts`

- [ ] **Step 1: Write API client tests**

Test that API client functions construct correct URLs and parse responses (mock fetch).

- [ ] **Step 2: Implement API client**

`client.ts` — functions for each API endpoint: `getSetupStatus`, `setUsername`, `setLlm`, `setGitlab`, `completeSetup`, `listRings`, `createRing`, `getRing`, `deleteRing`.

- [ ] **Step 3: Run tests**

Run: `cd ring-frontend && npm test`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add ring-frontend/src/types/ ring-frontend/src/api/
git commit -m "feat: add API client and TypeScript types"
```

---

## Module 13: Frontend — Setup Wizard

**Files:**
- Create: `ring-frontend/src/stores/setupStore.ts`
- Create: `ring-frontend/src/pages/Setup/SetupWizard.tsx`
- Create: `ring-frontend/src/pages/Setup/StepUsername.tsx`
- Create: `ring-frontend/src/pages/Setup/StepLlm.tsx`
- Create: `ring-frontend/src/pages/Setup/StepGitlab.tsx`

- [ ] **Step 1: Write component tests**

- `StepUsername` renders input + submit button
- `StepUsername` submits display_name on click
- `SetupWizard` shows correct step based on state

- [ ] **Step 2: Implement pages**

Zustand store tracks setup state. Multi-step wizard with username → LLM → GitLab → complete.

- [ ] **Step 3: Run tests**

Run: `cd ring-frontend && npm test`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add ring-frontend/src/stores/ ring-frontend/src/pages/Setup/
git commit -m "feat: add setup wizard with step components"
```

---

## Module 14: Frontend — Ring Hub

**Files:**
- Create: `ring-frontend/src/pages/RingHub/RingHub.tsx`
- Create: `ring-frontend/src/pages/RingHub/RingList.tsx`
- Create: `ring-frontend/src/pages/RingHub/CreateRing.tsx`
- Create: `ring-frontend/src/App.tsx`

- [ ] **Step 1: Write component tests**

- `RingList` renders ring cards
- `CreateRing` form submits name + description
- Route guard redirects to /setup when not setup

- [ ] **Step 2: Implement pages**

`RingHub.tsx` — list + create form. `App.tsx` — router with setup guard.

- [ ] **Step 3: Run tests**

Run: `cd ring-frontend && npm test`
Expected: PASS

- [ ] **Step 4: Manual E2E smoke test**

Start backend + frontend → complete setup → create a Ring → verify appears in list.

- [ ] **Step 5: Commit**

```bash
git add ring-frontend/src/pages/RingHub/ ring-frontend/src/App.tsx
git commit -m "feat: add Ring Hub page with list and create form"
```

---

## Module 15: Full Integration Verification

- [ ] **Step 1: Run all backend tests**

Run: `cd ring-server && cargo test`
Expected: ALL PASS

- [ ] **Step 2: Run all frontend tests**

Run: `cd ring-frontend && npm test`
Expected: ALL PASS

- [ ] **Step 3: Run cargo clippy + fmt**

```bash
cd ring-server && cargo fmt --check && cargo clippy -- -D warnings
```

- [ ] **Step 4: Full manual walkthrough**

Setup → Create Ring → List Rings → Delete Ring → Install page with valid token

- [ ] **Step 5: Final commit**

```bash
git commit --allow-empty -m "milestone: Phase 1 complete — basic framework"
```
