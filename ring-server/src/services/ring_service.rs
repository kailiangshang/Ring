use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::db::traits::Repository;
use crate::error::{Result, RingError};
use crate::graph::types::GraphJson;
use crate::models::ring::{NewRing, Ring};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRingRequest {
    pub name: String,
    pub description: Option<String>,
    pub role_description: String,
    pub creator_id: String,
    pub gitlab_repo: String,
    pub namespace: Option<String>,
}

pub struct RingService {
    repo: Arc<dyn Repository>,
    data_dir: PathBuf,
}

impl RingService {
    pub fn new(repo: Arc<dyn Repository>, data_dir: PathBuf) -> Self {
        RingService { repo, data_dir }
    }

    pub async fn create_ring(&self, req: CreateRingRequest) -> Result<Ring> {
        if req.name.trim().is_empty() {
            return Err(RingError::Validation("name must not be empty".into()));
        }
        if req.name.len() > 100 {
            return Err(RingError::Validation(
                "name must be 100 characters or less".into(),
            ));
        }

        let creator_id = match self.repo.get_user(&req.creator_id).await? {
            Some(_) => req.creator_id.clone(),
            None => {
                let user = self
                    .repo
                    .create_user(crate::models::user::NewUser {
                        display_name: req.creator_id.clone(),
                    })
                    .await?;
                user.id
            }
        };

        let role_description = req.role_description.clone();

        let new_ring = NewRing {
            name: req.name.clone(),
            description: req.description,
            creator_id,
            gitlab_repo: req.gitlab_repo,
            namespace: req.namespace,
            role_description: req.role_description,
        };

        let ring = self.repo.create_ring(new_ring).await?;

        let repo_dir = self
            .data_dir
            .join("repos")
            .join(format!("ring-{}", ring.name));
        std::fs::create_dir_all(repo_dir.join("nodes"))?;
        let empty_graph = GraphJson {
            nodes: vec![],
            edges: vec![],
        };
        let json = serde_json::to_string_pretty(&empty_graph)?;
        std::fs::write(repo_dir.join("graph.json"), json)?;

        Self::init_ring_templates(&repo_dir, &role_description)?;

        Ok(ring)
    }

    fn init_ring_templates(
        repo_dir: &std::path::Path,
        role_description: &str,
    ) -> std::result::Result<(), std::io::Error> {
        let dot_ring = repo_dir.join(".ring");
        std::fs::create_dir_all(&dot_ring)?;

        std::fs::write(
            dot_ring.join("role.md"),
            format!(
                r#"# 角色定义

{role_description}

## 行为准则

- 回复使用中文
- 优先基于 Ring 内已有的知识回答问题
- 不确定时明确告知，不猜测
- 归档推荐时说明理由
"#
            ),
        )?;

        std::fs::write(
            dot_ring.join("conventions.md"),
            r#"# 团队约定

## 命名规范

- 节点名称使用中文
- Markdown 文件名使用英文小写 + 连字符

## 术语表

（暂无，使用中积累）

## 分类偏好

（暂无，使用中积累）
"#,
        )?;

        std::fs::write(
            dot_ring.join("active-context.md"),
            r#"# 当前活跃上下文

（系统自动维护，用户无需编辑）

最近活动：Ring 刚创建，等待首次使用。
"#,
        )?;

        std::fs::write(
            dot_ring.join("archive-patterns.md"),
            r#"# 归档模式

（AI 自动积累，记录用户的归档偏好）

## 归档粒度偏好

（暂无数据）

## 归档位置偏好

（暂无数据）

## 更新 vs 新建偏好

（暂无数据）
"#,
        )?;

        std::fs::write(
            dot_ring.join("corrections.md"),
            r#"# 修正记录

（AI 自动积累，记录用户对 AI 行为的修正）

（暂无修正记录）
"#,
        )?;

        std::fs::write(
            dot_ring.join("knowledge-summary.md"),
            r#"# 知识总结

（AI 定期生成，总结 Ring 的知识全貌）

（暂无数据，使用后由 AI 定期更新）
"#,
        )?;

        Ok(())
    }

    pub async fn get_ring(&self, id: &str) -> Result<Ring> {
        self.repo
            .get_ring(id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("ring {}", id)))
    }

    pub async fn list_rings(&self, user_id: &str) -> Result<Vec<Ring>> {
        self.repo.list_rings_by_user(user_id).await
    }

    pub async fn update_ring(
        &self,
        id: &str,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Ring> {
        self.repo.update_ring(id, name, description).await
    }

    pub async fn delete_ring(&self, id: &str) -> Result<()> {
        self.repo.delete_ring(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_service() -> RingService {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let repo = crate::db::sqlite::SqliteRepository::new(pool);
        let data_dir = tempfile::tempdir().unwrap().path().to_path_buf();
        RingService::new(Arc::new(repo), data_dir)
    }

    fn valid_request(creator_id: &str) -> CreateRingRequest {
        CreateRingRequest {
            name: "test-ring".into(),
            description: Some("a test ring".into()),
            role_description: "expert".into(),
            creator_id: creator_id.into(),
            gitlab_repo: "auto_create".into(),
            namespace: None,
        }
    }

    async fn create_test_user(svc: &RingService, display_name: &str) -> String {
        let user = svc
            .repo
            .create_user(crate::models::user::NewUser {
                display_name: display_name.into(),
            })
            .await
            .unwrap();
        user.id
    }

    #[tokio::test]
    async fn create_ring_success() {
        let svc = setup_service().await;
        let user_id = create_test_user(&svc, "user-1").await;
        let ring = svc.create_ring(valid_request(&user_id)).await.unwrap();
        assert_eq!(ring.name, "test-ring");
        assert_eq!(ring.creator_id, user_id);
        assert!(!ring.id.is_empty());
    }

    #[tokio::test]
    async fn create_ring_empty_name_fails() {
        let svc = setup_service().await;
        let mut req = valid_request("user-1");
        req.name = "   ".into();
        let err = svc.create_ring(req).await.unwrap_err();
        match err {
            RingError::Validation(msg) => assert!(msg.contains("empty")),
            _ => panic!("expected Validation error, got {:?}", err),
        }
    }

    #[tokio::test]
    async fn create_ring_name_too_long_fails() {
        let svc = setup_service().await;
        let mut req = valid_request("user-1");
        req.name = "x".repeat(101);
        let err = svc.create_ring(req).await.unwrap_err();
        match err {
            RingError::Validation(msg) => assert!(msg.contains("100")),
            _ => panic!("expected Validation error, got {:?}", err),
        }
    }

    #[tokio::test]
    async fn get_ring_nonexistent_returns_not_found() {
        let svc = setup_service().await;
        let err = svc.get_ring("nonexistent").await.unwrap_err();
        match err {
            RingError::NotFound(msg) => assert!(msg.contains("nonexistent")),
            _ => panic!("expected NotFound error, got {:?}", err),
        }
    }

    #[tokio::test]
    async fn delete_ring_nonexistent_returns_not_found() {
        let svc = setup_service().await;
        let err = svc.delete_ring("nonexistent").await.unwrap_err();
        match err {
            RingError::NotFound(msg) => assert!(msg.contains("nonexistent")),
            _ => panic!("expected NotFound error, got {:?}", err),
        }
    }

    #[tokio::test]
    async fn create_ring_initializes_templates() {
        let svc = setup_service().await;
        let user_id = create_test_user(&svc, "user-tpl").await;
        let ring = svc.create_ring(valid_request(&user_id)).await.unwrap();
        let repo_dir = svc
            .data_dir
            .join("repos")
            .join(format!("ring-{}", ring.name));
        let dot_ring = repo_dir.join(".ring");
        assert!(dot_ring.join("role.md").exists());
        assert!(dot_ring.join("conventions.md").exists());
        assert!(dot_ring.join("active-context.md").exists());
        assert!(dot_ring.join("archive-patterns.md").exists());
        assert!(dot_ring.join("corrections.md").exists());
        assert!(dot_ring.join("knowledge-summary.md").exists());
        let role = std::fs::read_to_string(dot_ring.join("role.md")).unwrap();
        assert!(role.contains("expert"));
    }

    #[tokio::test]
    async fn list_rings_returns_users_rings_only() {
        let svc = setup_service().await;
        let user_a = create_test_user(&svc, "user-a").await;
        let user_b = create_test_user(&svc, "user-b").await;
        svc.create_ring(valid_request(&user_a)).await.unwrap();
        svc.create_ring(valid_request(&user_b)).await.unwrap();

        let rings_a = svc.list_rings(&user_a).await.unwrap();
        let rings_b = svc.list_rings(&user_b).await.unwrap();

        assert_eq!(rings_a.len(), 1);
        assert_eq!(rings_b.len(), 1);
        assert_ne!(rings_a[0].id, rings_b[0].id);
    }
}
