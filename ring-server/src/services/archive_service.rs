use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::db::traits::Repository;
use crate::error::{Result, RingError};
use crate::graph::petgraph_store::PetgraphStore;
use crate::models::git_model::{
    ArchiveQueueResponse, ArchiveRecord, ArchiveRequest, ArchiveResponse, CommitEntry,
    CommitLogResponse, FileChange, PrResponse, QueueItem,
};
use crate::services::git_service::GitService;
use crate::services::gitlab_service::GitlabService;

pub struct ArchiveService {
    db: Arc<dyn Repository>,
    git_service: Arc<GitService>,
    graph_store: Arc<RwLock<PetgraphStore>>,
    gitlab_service: Option<Arc<GitlabService>>,
}

impl ArchiveService {
    pub fn new(
        db: Arc<dyn Repository>,
        git_service: Arc<GitService>,
        graph_store: Arc<RwLock<PetgraphStore>>,
        gitlab_service: Option<Arc<GitlabService>>,
    ) -> Self {
        ArchiveService {
            db,
            git_service,
            graph_store,
            gitlab_service,
        }
    }

    pub async fn archive(
        &self,
        ring_id: &str,
        request: &ArchiveRequest,
        archived_by: &str,
        is_creator: bool,
    ) -> Result<ArchiveResponse> {
        let ring = self
            .db
            .get_ring(ring_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("ring {}", ring_id)))?;

        let repo_path = PathBuf::from(&ring.local_path);
        if !repo_path.exists() {
            return Err(RingError::Internal(format!(
                "ring repo path does not exist: {}",
                ring.local_path
            )));
        }

        let mut messages = Vec::new();
        for mid in &request.message_ids {
            let all = self
                .db
                .get_messages(&request.conversation_id, 1000, None)
                .await?;
            for msg in &all {
                if msg.id == *mid {
                    messages.push(msg.content.clone());
                }
            }
        }

        let markdown = messages.join("\n\n---\n\n");
        let slug = request.label.replace(' ', "-").to_lowercase();
        let relative_path = format!("nodes/{}.md", slug);
        let file_path = repo_path.join(&relative_path);

        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&file_path, &markdown).await?;

        let graph_data = self
            .graph_store
            .read()
            .await
            .export_graph_json(&request.graph_id)
            .await?;
        let graph_json = serde_json::to_string_pretty(&graph_data)?;
        let graph_path = repo_path.join("graph.json");
        tokio::fs::write(&graph_path, graph_json).await?;

        let archive_id = uuid::Uuid::new_v4().to_string();

        if is_creator {
            self.git_service.add_all(&repo_path).await?;
            let commit_sha = self
                .git_service
                .commit(&repo_path, &format!("archive: {}", request.label))
                .await?;

            self.db
                .create_archive_record(
                    &archive_id,
                    ring_id,
                    request.target_node_id.as_deref(),
                    Some(&request.conversation_id),
                    &serde_json::to_string(&request.message_ids)?,
                    &relative_path,
                    archived_by,
                    Some(&commit_sha),
                    Some("committed"),
                    None,
                )
                .await?;

            Ok(ArchiveResponse {
                archive_id,
                markdown_path: relative_path,
                git_status: "committed".to_string(),
                pr_url: None,
                queue_position: None,
            })
        } else {
            let timestamp = chrono::Utc::now().timestamp();
            let branch_name = format!("archive/{}", timestamp);
            self.git_service
                .create_branch(&repo_path, &branch_name)
                .await?;

            self.git_service.add_all(&repo_path).await?;
            let commit_sha = self
                .git_service
                .commit(&repo_path, &format!("archive: {}", request.label))
                .await?;

            let pr_url = self.gitlab_service.as_ref().map(|gl| {
                format!(
                    "{}/-/merge_requests/new?source_branch={}",
                    gl.base_url, branch_name
                )
            });

            self.db
                .create_archive_record(
                    &archive_id,
                    ring_id,
                    request.target_node_id.as_deref(),
                    Some(&request.conversation_id),
                    &serde_json::to_string(&request.message_ids)?,
                    &relative_path,
                    archived_by,
                    Some(&commit_sha),
                    Some("pr_pending"),
                    pr_url.as_deref(),
                )
                .await?;

            Ok(ArchiveResponse {
                archive_id,
                markdown_path: relative_path,
                git_status: "pr_pending".to_string(),
                pr_url,
                queue_position: None,
            })
        }
    }

    pub async fn get_queue(&self, ring_id: &str) -> Result<ArchiveQueueResponse> {
        let records = self.db.list_archive_records_by_ring(ring_id).await?;
        let pending: Vec<&ArchiveRecord> = records
            .iter()
            .filter(|r| r.pr_status.as_deref() == Some("pr_pending"))
            .collect();

        let queue: Vec<QueueItem> = pending
            .iter()
            .enumerate()
            .map(|(i, r)| QueueItem {
                pr_id: i as i64,
                author: r.archived_by.clone(),
                title: r.markdown_path.clone(),
                position: i as i64 + 1,
            })
            .collect();

        Ok(ArchiveQueueResponse {
            current_review: queue.first().cloned(),
            queue,
        })
    }

    pub async fn confirm_archive(&self, _ring_id: &str, archive_id: &str) -> Result<()> {
        self.db
            .update_archive_pr_status(archive_id, "confirmed")
            .await
    }

    pub async fn list_prs(&self, ring_id: &str, state: &str) -> Result<Vec<PrResponse>> {
        let records = self.db.list_archive_records_by_ring(ring_id).await?;
        let filtered: Vec<&ArchiveRecord> = records
            .iter()
            .filter(|r| r.pr_status.as_deref() == Some(state))
            .collect();

        Ok(filtered
            .iter()
            .map(|r| PrResponse {
                pr_id: 0,
                title: r.markdown_path.clone(),
                author: r.archived_by.clone(),
                state: r.pr_status.clone().unwrap_or_default(),
                changes: vec![FileChange {
                    file: r.markdown_path.clone(),
                    status: "added".to_string(),
                    additions: 0,
                    deletions: 0,
                    diff: String::new(),
                }],
            })
            .collect())
    }

    pub async fn get_pr_diff(&self, _ring_id: &str, _pr_id: i64) -> Result<PrResponse> {
        Err(RingError::NotFound("pr not found".into()))
    }

    pub async fn merge_pr(&self, ring_id: &str, archive_id: &str) -> Result<()> {
        let _record = self
            .db
            .get_archive_record(archive_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("archive {}", archive_id)))?;

        if let Some(ref _gl) = self.gitlab_service {
            let ring = self
                .db
                .get_ring(ring_id)
                .await?
                .ok_or_else(|| RingError::NotFound(format!("ring {}", ring_id)))?;
            let _ = ring;
        }

        self.db.update_archive_pr_status(archive_id, "merged").await
    }

    pub async fn reject_pr(&self, _ring_id: &str, archive_id: &str) -> Result<()> {
        self.db
            .update_archive_pr_status(archive_id, "rejected")
            .await
    }

    pub async fn get_commit_log(&self, ring_id: &str, limit: usize) -> Result<CommitLogResponse> {
        let ring = self
            .db
            .get_ring(ring_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("ring {}", ring_id)))?;

        let repo_path = Path::new(&ring.local_path);
        let log = self.git_service.get_log(repo_path, limit).await?;

        Ok(CommitLogResponse {
            commits: log
                .into_iter()
                .map(|c| CommitEntry {
                    id: c.id,
                    message: c.message,
                    author: c.author,
                    date: c.timestamp,
                })
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ring::Ring;
    use std::fs;

    struct MockArchiveRepo {
        records: tokio::sync::Mutex<Vec<ArchiveRecord>>,
        ring: Ring,
    }

    impl MockArchiveRepo {
        fn new(ring: Ring) -> Self {
            MockArchiveRepo {
                records: tokio::sync::Mutex::new(Vec::new()),
                ring,
            }
        }
    }

    #[async_trait::async_trait]
    impl Repository for MockArchiveRepo {
        async fn create_user(
            &self,
            _new_user: crate::models::user::NewUser,
        ) -> crate::error::Result<crate::models::user::User> {
            unimplemented!()
        }
        async fn get_user(
            &self,
            _id: &str,
        ) -> crate::error::Result<Option<crate::models::user::User>> {
            unimplemented!()
        }
        async fn list_all_users(&self) -> crate::error::Result<Vec<crate::models::user::User>> {
            unimplemented!()
        }
        async fn is_setup_completed(&self) -> crate::error::Result<bool> {
            Ok(true)
        }
        async fn complete_setup(&self, _user_id: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn create_ring(
            &self,
            _new_ring: crate::models::ring::NewRing,
        ) -> crate::error::Result<crate::models::ring::Ring> {
            unimplemented!()
        }
        async fn get_ring(
            &self,
            id: &str,
        ) -> crate::error::Result<Option<crate::models::ring::Ring>> {
            if id == self.ring.id {
                Ok(Some(self.ring.clone()))
            } else {
                Ok(None)
            }
        }
        async fn list_rings_by_user(
            &self,
            _user_id: &str,
        ) -> crate::error::Result<Vec<crate::models::ring::Ring>> {
            unimplemented!()
        }
        async fn update_ring(
            &self,
            _id: &str,
            _name: Option<String>,
            _description: Option<String>,
        ) -> crate::error::Result<crate::models::ring::Ring> {
            unimplemented!()
        }
        async fn delete_ring(&self, _id: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn create_invite_token(
            &self,
            _ring_id: &str,
            _token: &str,
            _token_type: &str,
            _inviter_id: &str,
        ) -> crate::error::Result<crate::models::invite::InviteToken> {
            unimplemented!()
        }
        async fn get_invite_token(
            &self,
            _token: &str,
        ) -> crate::error::Result<Option<crate::models::invite::InviteToken>> {
            unimplemented!()
        }
        async fn get_setting(&self, _key: &str) -> crate::error::Result<Option<String>> {
            unimplemented!()
        }
        async fn set_setting(&self, _key: &str, _value: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn count_members_by_ring(&self, _ring_id: &str) -> crate::error::Result<i64> {
            unimplemented!()
        }
        async fn create_conversation(
            &self,
            _ring_id: &str,
            _title: Option<String>,
            _context_mode: &str,
            _created_by: &str,
        ) -> crate::error::Result<crate::models::conversation::Conversation> {
            unimplemented!()
        }
        async fn list_conversations(
            &self,
            _ring_id: &str,
        ) -> crate::error::Result<Vec<crate::models::conversation::Conversation>> {
            unimplemented!()
        }
        async fn get_conversation(
            &self,
            _id: &str,
        ) -> crate::error::Result<Option<crate::models::conversation::Conversation>> {
            unimplemented!()
        }
        async fn create_message(
            &self,
            _conversation_id: &str,
            _role: &str,
            _content: &str,
            _sender_id: Option<&str>,
        ) -> crate::error::Result<crate::models::conversation::Message> {
            unimplemented!()
        }
        async fn get_messages(
            &self,
            conversation_id: &str,
            _limit: i64,
            _before_id: Option<&str>,
        ) -> crate::error::Result<Vec<crate::models::conversation::Message>> {
            Ok(vec![crate::models::conversation::Message {
                id: "msg-1".into(),
                conversation_id: conversation_id.to_string(),
                role: "user".into(),
                content: "test message content".into(),
                sender_id: None,
                tool_calls: None,
                archived: false,
                created_at: "2025-01-01T00:00:00Z".into(),
            }])
        }
        async fn update_ring_status(&self, _id: &str, _status: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn list_blueprint_templates(
            &self,
        ) -> crate::error::Result<Vec<crate::models::blueprint::BlueprintTemplate>> {
            unimplemented!()
        }
        async fn create_blueprint_template(
            &self,
            _id: &str,
            _name: &str,
            _description: Option<&str>,
            _graphs_json: &str,
            _is_system: bool,
        ) -> crate::error::Result<crate::models::blueprint::BlueprintTemplate> {
            unimplemented!()
        }
        async fn index_node_search(
            &self,
            _node_id: &str,
            _graph_id: &str,
            _label: &str,
            _content: &str,
        ) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn delete_node_search(&self, _node_id: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn search_nodes_fts(
            &self,
            _query: &str,
            _graph_ids: Option<Vec<String>>,
            _limit: i64,
        ) -> crate::error::Result<Vec<crate::models::graph_model::SearchResult>> {
            unimplemented!()
        }
        async fn create_archive_record(
            &self,
            id: &str,
            ring_id: &str,
            node_id: Option<&str>,
            conversation_id: Option<&str>,
            message_ids: &str,
            markdown_path: &str,
            archived_by: &str,
            git_commit_sha: Option<&str>,
            pr_status: Option<&str>,
            pr_url: Option<&str>,
        ) -> crate::error::Result<()> {
            self.records.lock().await.push(ArchiveRecord {
                id: id.to_string(),
                ring_id: ring_id.to_string(),
                node_id: node_id.map(|s| s.to_string()),
                conversation_id: conversation_id.map(|s| s.to_string()),
                message_ids: Some(message_ids.to_string()),
                markdown_path: markdown_path.to_string(),
                archived_by: archived_by.to_string(),
                git_commit_sha: git_commit_sha.map(|s| s.to_string()),
                pr_status: pr_status.map(|s| s.to_string()),
                pr_url: pr_url.map(|s| s.to_string()),
                created_at: chrono::Utc::now().to_rfc3339(),
            });
            Ok(())
        }
        async fn list_archive_records_by_ring(
            &self,
            ring_id: &str,
        ) -> crate::error::Result<Vec<ArchiveRecord>> {
            let records = self.records.lock().await;
            Ok(records
                .iter()
                .filter(|r| r.ring_id == ring_id)
                .cloned()
                .collect())
        }
        async fn get_archive_record(
            &self,
            id: &str,
        ) -> crate::error::Result<Option<ArchiveRecord>> {
            let records = self.records.lock().await;
            Ok(records.iter().find(|r| r.id == id).cloned())
        }
        async fn update_archive_pr_status(
            &self,
            id: &str,
            pr_status: &str,
        ) -> crate::error::Result<()> {
            let mut records = self.records.lock().await;
            if let Some(r) = records.iter_mut().find(|r| r.id == id) {
                r.pr_status = Some(pr_status.to_string());
            }
            Ok(())
        }
        async fn create_member(
            &self,
            _new_member: crate::models::member::NewMember,
        ) -> crate::error::Result<crate::models::member::Member> {
            unimplemented!()
        }
        async fn get_member(
            &self,
            _id: &str,
        ) -> crate::error::Result<Option<crate::models::member::Member>> {
            unimplemented!()
        }
        async fn list_members_by_ring(
            &self,
            _ring_id: &str,
        ) -> crate::error::Result<Vec<crate::models::member::Member>> {
            unimplemented!()
        }
        async fn get_member_by_user_and_ring(
            &self,
            _user_id: &str,
            _ring_id: &str,
        ) -> crate::error::Result<Option<crate::models::member::Member>> {
            unimplemented!()
        }
        async fn update_member_role(&self, _id: &str, _role: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn delete_member(&self, _id: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn get_next_token_id(&self, _ring_id: &str) -> crate::error::Result<i64> {
            unimplemented!()
        }
        async fn create_notification(
            &self,
            _n: crate::models::notification_model::NewNotification,
        ) -> crate::error::Result<crate::models::notification_model::Notification> {
            unimplemented!()
        }
        async fn list_notifications_by_user(
            &self,
            _user_id: &str,
            _unread_only: bool,
        ) -> crate::error::Result<Vec<crate::models::notification_model::Notification>> {
            unimplemented!()
        }
        async fn mark_notification_read(&self, _id: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn create_session(
            &self,
            _ring_id: &str,
            _title: Option<&str>,
            _scenario: &str,
            _created_by: &str,
            _archive_enabled: bool,
        ) -> crate::error::Result<crate::models::session_model::Session> {
            unimplemented!()
        }
        async fn get_session(&self, _id: &str) -> crate::error::Result<Option<crate::models::session_model::Session>> {
            unimplemented!()
        }
        async fn list_sessions_by_ring(
            &self,
            _ring_id: &str,
            _status: Option<&str>,
        ) -> crate::error::Result<Vec<crate::models::session_model::Session>> {
            unimplemented!()
        }
        async fn update_session_status(&self, _id: &str, _status: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn update_session_archive(&self, _id: &str, _enabled: bool) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn delete_session(&self, _id: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn create_session_member(
            &self,
            _session_id: &str,
            _user_id: &str,
            _role: &str,
        ) -> crate::error::Result<crate::models::session_model::SessionMember> {
            unimplemented!()
        }
        async fn list_session_members(&self, _session_id: &str) -> crate::error::Result<Vec<crate::models::session_model::SessionMember>> {
            unimplemented!()
        }
        async fn leave_session_member(&self, _session_id: &str, _user_id: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn create_session_message(
            &self,
            _session_id: &str,
            _sender_id: &str,
            _role: &str,
            _content: &str,
            _seq_num: i64,
        ) -> crate::error::Result<crate::models::session_model::SessionMessage> {
            unimplemented!()
        }
        async fn get_session_messages(
            &self,
            _session_id: &str,
            _after_seq: Option<i64>,
            _limit: i64,
        ) -> crate::error::Result<Vec<crate::models::session_model::SessionMessage>> {
            unimplemented!()
        }
    }

    fn make_test_ring(local_path: &std::path::Path) -> Ring {
        Ring {
            id: "ring-test".into(),
            name: "TestRing".into(),
            description: None,
            creator_id: "user-1".into(),
            gitlab_repo: "auto_create".into(),
            local_path: local_path.to_string_lossy().to_string(),
            next_token_id: 2,
            status: "active".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-01-01T00:00:00Z".into(),
        }
    }

    async fn setup_service(dir: &tempfile::TempDir) -> ArchiveService {
        let git_svc = Arc::new(GitService::new());
        git_svc.init_repo(dir.path()).await.unwrap();

        let graph_store = Arc::new(RwLock::new(PetgraphStore::new()));
        let ring = make_test_ring(dir.path());
        let db = Arc::new(MockArchiveRepo::new(ring));

        ArchiveService::new(db, git_svc, graph_store, None)
    }

    #[tokio::test]
    async fn creator_archive_commits_directly() {
        let dir = tempfile::TempDir::new().unwrap();
        let svc = setup_service(&dir).await;

        fs::write(dir.path().join("graph.json"), "{}").unwrap();

        let request = ArchiveRequest {
            message_ids: vec!["msg-1".into()],
            conversation_id: "conv-1".into(),
            graph_id: "graph-1".into(),
            target_node_id: None,
            label: "test archive".into(),
        };

        let resp = svc
            .archive("ring-test", &request, "user-1", true)
            .await
            .unwrap();

        assert_eq!(resp.git_status, "committed");
        assert!(resp.pr_url.is_none());
        assert!(resp.markdown_path.contains("test-archive.md"));
        assert!(dir.path().join("nodes/test-archive.md").exists());

        let log = svc.git_service.get_log(dir.path(), 10).await.unwrap();
        assert_eq!(log.len(), 1);
        assert!(log[0].message.contains("archive: test archive"));
    }

    #[tokio::test]
    async fn member_archive_creates_branch() {
        let dir = tempfile::TempDir::new().unwrap();
        let svc = setup_service(&dir).await;

        fs::write(dir.path().join("graph.json"), "{}").unwrap();

        let request = ArchiveRequest {
            message_ids: vec!["msg-1".into()],
            conversation_id: "conv-1".into(),
            graph_id: "graph-1".into(),
            target_node_id: None,
            label: "member note".into(),
        };

        let resp = svc
            .archive("ring-test", &request, "user-2", false)
            .await
            .unwrap();

        assert_eq!(resp.git_status, "pr_pending");
        assert!(resp.markdown_path.contains("member-note.md"));
    }

    #[tokio::test]
    async fn get_queue_returns_empty_initially() {
        let dir = tempfile::TempDir::new().unwrap();
        let svc = setup_service(&dir).await;

        let queue = svc.get_queue("ring-test").await.unwrap();
        assert!(queue.current_review.is_none());
        assert!(queue.queue.is_empty());
    }

    #[tokio::test]
    async fn get_queue_returns_pending_items() {
        let dir = tempfile::TempDir::new().unwrap();
        let svc = setup_service(&dir).await;

        fs::write(dir.path().join("graph.json"), "{}").unwrap();

        let request = ArchiveRequest {
            message_ids: vec!["msg-1".into()],
            conversation_id: "conv-1".into(),
            graph_id: "graph-1".into(),
            target_node_id: None,
            label: "queued".into(),
        };

        svc.archive("ring-test", &request, "user-2", false)
            .await
            .unwrap();

        let queue = svc.get_queue("ring-test").await.unwrap();
        assert!(queue.current_review.is_some());
        assert_eq!(queue.queue.len(), 1);
    }

    #[tokio::test]
    async fn confirm_archive_updates_status() {
        let dir = tempfile::TempDir::new().unwrap();
        let svc = setup_service(&dir).await;

        fs::write(dir.path().join("graph.json"), "{}").unwrap();

        let request = ArchiveRequest {
            message_ids: vec!["msg-1".into()],
            conversation_id: "conv-1".into(),
            graph_id: "graph-1".into(),
            target_node_id: None,
            label: "confirm test".into(),
        };

        let resp = svc
            .archive("ring-test", &request, "user-2", false)
            .await
            .unwrap();

        svc.confirm_archive("ring-test", &resp.archive_id)
            .await
            .unwrap();

        let record = svc
            .db
            .get_archive_record(&resp.archive_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(record.pr_status, Some("confirmed".to_string()));
    }

    #[tokio::test]
    async fn reject_pr_updates_status() {
        let dir = tempfile::TempDir::new().unwrap();
        let svc = setup_service(&dir).await;

        fs::write(dir.path().join("graph.json"), "{}").unwrap();

        let request = ArchiveRequest {
            message_ids: vec!["msg-1".into()],
            conversation_id: "conv-1".into(),
            graph_id: "graph-1".into(),
            target_node_id: None,
            label: "reject test".into(),
        };

        let resp = svc
            .archive("ring-test", &request, "user-2", false)
            .await
            .unwrap();

        svc.reject_pr("ring-test", &resp.archive_id).await.unwrap();

        let record = svc
            .db
            .get_archive_record(&resp.archive_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(record.pr_status, Some("rejected".to_string()));
    }

    #[tokio::test]
    async fn get_commit_log_returns_entries() {
        let dir = tempfile::TempDir::new().unwrap();
        let svc = setup_service(&dir).await;

        fs::write(dir.path().join("initial.txt"), "hello").unwrap();
        svc.git_service.add_all(dir.path()).await.unwrap();
        svc.git_service
            .commit(dir.path(), "initial commit")
            .await
            .unwrap();

        let log = svc.get_commit_log("ring-test", 10).await.unwrap();
        assert!(!log.commits.is_empty());
        assert!(log.commits[0].message.contains("initial commit"));
    }
}
