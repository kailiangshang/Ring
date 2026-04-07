use super::*;
use crate::models::ring::NewRing;
use crate::models::user::NewUser;
use sqlx::SqlitePool;

async fn setup_test_db() -> SqliteRepository {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    SqliteRepository::new(pool)
}

#[tokio::test]
async fn create_and_get_user() {
    let repo = setup_test_db().await;
    let user = repo
        .create_user(NewUser {
            display_name: "张三".into(),
        })
        .await
        .unwrap();
    assert_eq!(user.display_name, "张三");
    assert!(!user.id.is_empty());

    let fetched = repo.get_user(&user.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, user.id);
}

#[tokio::test]
async fn create_and_list_rings() {
    let repo = setup_test_db().await;
    let user = repo
        .create_user(NewUser {
            display_name: "张三".into(),
        })
        .await
        .unwrap();

    let ring = repo
        .create_ring(NewRing {
            name: "竞品分析".into(),
            description: Some("desc".into()),
            creator_id: user.id.clone(),
            gitlab_repo: "auto_create".into(),
            namespace: None,
            role_description: "产品专家".into(),
        })
        .await
        .unwrap();
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
    let user = repo
        .create_user(NewUser {
            display_name: "张三".into(),
        })
        .await
        .unwrap();
    repo.complete_setup(&user.id).await.unwrap();
    let status = repo.is_setup_completed().await.unwrap();
    assert!(status);
}

#[tokio::test]
async fn create_and_get_invite_token() {
    let repo = setup_test_db().await;
    let user = repo
        .create_user(NewUser {
            display_name: "张三".into(),
        })
        .await
        .unwrap();
    let ring = repo
        .create_ring(NewRing {
            name: "竞品分析".into(),
            description: None,
            creator_id: user.id.clone(),
            gitlab_repo: "auto_create".into(),
            namespace: None,
            role_description: "专家".into(),
        })
        .await
        .unwrap();

    let invite = repo
        .create_invite_token(&ring.id, "test-token-123", "open", "member", &user.id)
        .await
        .unwrap();
    assert_eq!(invite.token, "test-token-123");
    assert_eq!(invite.token_type, "open");

    let fetched = repo
        .get_invite_token("test-token-123")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetched.id, invite.id);
}

#[tokio::test]
async fn delete_ring_removes_from_list() {
    let repo = setup_test_db().await;
    let user = repo
        .create_user(NewUser {
            display_name: "张三".into(),
        })
        .await
        .unwrap();
    let ring = repo
        .create_ring(NewRing {
            name: "竞品分析".into(),
            description: None,
            creator_id: user.id.clone(),
            gitlab_repo: "auto_create".into(),
            namespace: None,
            role_description: "专家".into(),
        })
        .await
        .unwrap();

    repo.delete_ring(&ring.id).await.unwrap();

    let fetched = repo.get_ring(&ring.id).await.unwrap();
    assert!(fetched.is_none());

    let rings = repo.list_rings_by_user(&user.id).await.unwrap();
    assert!(rings.is_empty());
}

#[tokio::test]
async fn create_and_list_conversations() {
    let repo = setup_test_db().await;
    let user = repo
        .create_user(NewUser {
            display_name: "张三".into(),
        })
        .await
        .unwrap();
    let ring = repo
        .create_ring(NewRing {
            name: "test-ring".into(),
            description: None,
            creator_id: user.id.clone(),
            gitlab_repo: "auto_create".into(),
            namespace: None,
            role_description: "专家".into(),
        })
        .await
        .unwrap();

    let conv = repo
        .create_conversation(&ring.id, Some("my chat".into()), "storage", &user.id)
        .await
        .unwrap();
    assert_eq!(conv.title, Some("my chat".into()));
    assert_eq!(conv.context_mode, "storage");
    assert_eq!(conv.mode, "chat");

    let fetched = repo.get_conversation(&conv.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, conv.id);

    let list = repo.list_conversations(&ring.id).await.unwrap();
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn create_and_get_messages() {
    let repo = setup_test_db().await;
    let user = repo
        .create_user(NewUser {
            display_name: "张三".into(),
        })
        .await
        .unwrap();
    let ring = repo
        .create_ring(NewRing {
            name: "test-ring".into(),
            description: None,
            creator_id: user.id.clone(),
            gitlab_repo: "auto_create".into(),
            namespace: None,
            role_description: "专家".into(),
        })
        .await
        .unwrap();
    let conv = repo
        .create_conversation(&ring.id, None, "storage", &user.id)
        .await
        .unwrap();

    let msg = repo
        .create_message(&conv.id, "user", "hello", Some(&user.id))
        .await
        .unwrap();
    assert_eq!(msg.role, "user");
    assert_eq!(msg.content, "hello");
    assert_eq!(msg.sender_id, Some(user.id.clone()));

    let msgs = repo.get_messages(&conv.id, 50, None).await.unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].content, "hello");
}

#[tokio::test]
async fn get_messages_with_limit() {
    let repo = setup_test_db().await;
    let user = repo
        .create_user(NewUser {
            display_name: "张三".into(),
        })
        .await
        .unwrap();
    let ring = repo
        .create_ring(NewRing {
            name: "test-ring".into(),
            description: None,
            creator_id: user.id.clone(),
            gitlab_repo: "auto_create".into(),
            namespace: None,
            role_description: "专家".into(),
        })
        .await
        .unwrap();
    let conv = repo
        .create_conversation(&ring.id, None, "storage", &user.id)
        .await
        .unwrap();

    for i in 0..5 {
        repo.create_message(&conv.id, "user", &format!("msg {}", i), Some(&user.id))
            .await
            .unwrap();
    }

    let msgs = repo.get_messages(&conv.id, 3, None).await.unwrap();
    assert_eq!(msgs.len(), 3);

    let msgs_before = repo
        .get_messages(&conv.id, 10, Some(&msgs[2].id))
        .await
        .unwrap();
    assert_eq!(msgs_before.len(), 2);
}

#[tokio::test]
async fn list_blueprint_templates_empty() {
    let repo = setup_test_db().await;
    let templates = repo.list_blueprint_templates().await.unwrap();
    assert!(templates.is_empty());
}

#[tokio::test]
async fn create_and_list_blueprint_templates() {
    let repo = setup_test_db().await;

    let bt = repo
        .create_blueprint_template(
            "bp-1",
            "knowledge-graph",
            Some("standard knowledge graph"),
            r#"[{"name":"concepts","graph_type":"knowledge"}]"#,
            true,
        )
        .await
        .unwrap();
    assert_eq!(bt.name, "knowledge-graph");
    assert!(bt.is_system);

    let templates = repo.list_blueprint_templates().await.unwrap();
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].id, "bp-1");
}
