use std::pin::Pin;
use std::sync::Arc;

use futures::{Stream, StreamExt};

use crate::db::traits::Repository;
use crate::error::{Result, RingError};
use crate::models::tool_model::{ToolCallRequest, ToolDefinition};
use crate::services::context_loader::{
    build_blueprint_prompt, build_group_ring_prompt, build_super_ring_prompt,
};
use crate::services::llm_provider::{LlmEvent, LlmMessage, LlmProvider};
use crate::services::tool_engine::ToolDispatcher;

pub struct AiService {
    db: Arc<dyn Repository>,
    llm: Arc<dyn LlmProvider>,
    tool_dispatcher: Arc<ToolDispatcher>,
}

impl AiService {
    pub fn new(
        db: Arc<dyn Repository>,
        llm: Arc<dyn LlmProvider>,
        tool_dispatcher: Arc<ToolDispatcher>,
    ) -> Self {
        AiService {
            db,
            llm,
            tool_dispatcher,
        }
    }

    pub async fn super_ring_chat(
        &self,
        message: String,
    ) -> Result<Pin<Box<dyn Stream<Item = LlmEvent> + Send>>> {
        let system_prompt = build_super_ring_prompt();
        let messages = vec![
            LlmMessage {
                role: "system".into(),
                content: system_prompt,
            },
            LlmMessage {
                role: "user".into(),
                content: message,
            },
        ];
        self.llm.chat_stream(messages, None).await
    }

    pub async fn group_ring_chat(
        &self,
        ring_id: &str,
        conv_id: &str,
        message: String,
    ) -> Result<Pin<Box<dyn Stream<Item = LlmEvent> + Send>>> {
        let ring = self
            .db
            .get_ring(ring_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("ring {}", ring_id)))?;

        let conv = self
            .db
            .get_conversation(conv_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("conversation {}", conv_id)))?;

        self.db
            .create_message(conv_id, "user", &message, None)
            .await?;

        let history = self.db.get_messages(conv_id, 100, None).await?;

        let role_md = "(未设置角色定义)";
        let system_prompt = build_group_ring_prompt(
            &ring.name,
            role_md,
            "(未设置团队约定)",
            conv.title.as_deref().unwrap_or("(无活跃上下文)"),
        );

        let mut messages = vec![LlmMessage {
            role: "system".into(),
            content: system_prompt,
        }];

        for msg in history.into_iter().rev() {
            messages.push(LlmMessage {
                role: msg.role,
                content: msg.content,
            });
        }

        self.llm.chat_stream(messages, None).await
    }

    pub async fn blueprint_chat(
        &self,
        ring_id: &str,
        message: String,
    ) -> Result<Pin<Box<dyn Stream<Item = LlmEvent> + Send>>> {
        let _ring = self
            .db
            .get_ring(ring_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("ring {}", ring_id)))?;

        let role_md = "(未设置角色定义)";
        let system_prompt = build_blueprint_prompt(role_md);

        let messages = vec![
            LlmMessage {
                role: "system".into(),
                content: system_prompt,
            },
            LlmMessage {
                role: "user".into(),
                content: message,
            },
        ];
        self.llm.chat_stream(messages, None).await
    }

    pub async fn chat_with_tools(
        &self,
        messages: Vec<LlmMessage>,
        tools: Vec<ToolDefinition>,
    ) -> Result<Pin<Box<dyn Stream<Item = LlmEvent> + Send>>> {
        let (tx, rx) = tokio::sync::mpsc::channel::<LlmEvent>(64);

        let llm = self.llm.clone();
        let dispatcher = self.tool_dispatcher.clone();

        tokio::spawn(async move {
            let mut current_messages = messages;
            let max_rounds = 5;

            for _round in 0..max_rounds {
                let stream = match llm
                    .chat_stream(current_messages.clone(), Some(tools.clone()))
                    .await
                {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = tx
                            .send(LlmEvent::Error {
                                code: "llm_error".into(),
                                message: e.to_string(),
                            })
                            .await;
                        return;
                    }
                };

                let mut tool_calls = Vec::new();
                let mut stream = std::pin::pin!(stream);

                while let Some(event) = stream.next().await {
                    match &event {
                        LlmEvent::ToolCall {
                            tool_call_id,
                            tool,
                            input,
                        } => {
                            tool_calls.push((tool_call_id.clone(), tool.clone(), input.clone()));
                        }
                        LlmEvent::Done { .. } => {}
                        _ => {}
                    }
                    if tx.send(event).await.is_err() {
                        return;
                    }
                }

                if tool_calls.is_empty() {
                    return;
                }

                let mut tool_args_parts = Vec::new();
                for (call_id, tool_name, input) in &tool_calls {
                    tool_args_parts.push(format!(
                        "Tool call: {} ({}) with {}",
                        tool_name, call_id, input
                    ));
                }
                current_messages.push(LlmMessage {
                    role: "assistant".into(),
                    content: tool_args_parts.join("\n"),
                });

                for (call_id, tool_name, input) in tool_calls {
                    let request = ToolCallRequest {
                        tool_call_id: call_id.clone(),
                        tool_name: tool_name.clone(),
                        input,
                    };
                    let result = dispatcher.dispatch(request).await;

                    let result_event = LlmEvent::ToolResult {
                        tool_call_id: result.tool_call_id.clone(),
                        tool: result.tool_name.clone(),
                        output: result.output.clone(),
                    };
                    if tx.send(result_event).await.is_err() {
                        return;
                    }

                    current_messages.push(LlmMessage {
                        role: "tool".into(),
                        content: serde_json::to_string(&result.output).unwrap_or_default(),
                    });
                }
            }

            let _ = tx
                .send(LlmEvent::Error {
                    code: "max_tool_rounds".into(),
                    message: "exceeded maximum tool call rounds".into(),
                })
                .await;
        });

        Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::llm_provider::{MockLlmProvider, TokenUsage};
    use crate::services::tool_engine::ToolDispatcher;
    use futures::StreamExt;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct MockRepo {
        create_message_calls: AtomicUsize,
    }

    impl MockRepo {
        fn new() -> Self {
            MockRepo {
                create_message_calls: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait::async_trait]
    impl Repository for MockRepo {
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
            _id: &str,
        ) -> crate::error::Result<Option<crate::models::ring::Ring>> {
            Ok(Some(crate::models::ring::Ring {
                id: "ring-1".into(),
                name: "TestRing".into(),
                description: None,
                creator_id: "user-1".into(),
                gitlab_repo: "auto_create".into(),
                local_path: ".ring/repos/ring-TestRing".into(),
                next_token_id: 2,
                status: "active".into(),
                created_at: "2025-01-01T00:00:00Z".into(),
                updated_at: "2025-01-01T00:00:00Z".into(),
            }))
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
            Ok(Some(crate::models::conversation::Conversation {
                id: "conv-1".into(),
                ring_id: "ring-1".into(),
                title: Some("test conv".into()),
                mode: "chat".into(),
                context_mode: "storage".into(),
                token_count: 0,
                token_limit: 100000,
                auto_compact: false,
                summary: None,
                compacted_at: None,
                created_by: "user-1".into(),
                created_at: "2025-01-01T00:00:00Z".into(),
                updated_at: "2025-01-01T00:00:00Z".into(),
            }))
        }
        async fn create_message(
            &self,
            _conversation_id: &str,
            _role: &str,
            _content: &str,
            _sender_id: Option<&str>,
        ) -> crate::error::Result<crate::models::conversation::Message> {
            self.create_message_calls.fetch_add(1, Ordering::SeqCst);
            Ok(crate::models::conversation::Message {
                id: "msg-1".into(),
                conversation_id: "conv-1".into(),
                role: "user".into(),
                content: _content.to_string(),
                sender_id: None,
                tool_calls: None,
                archived: false,
                created_at: "2025-01-01T00:00:00Z".into(),
            })
        }
        async fn get_messages(
            &self,
            _conversation_id: &str,
            _limit: i64,
            _before_id: Option<&str>,
        ) -> crate::error::Result<Vec<crate::models::conversation::Message>> {
            Ok(vec![])
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
            _id: &str,
            _ring_id: &str,
            _node_id: Option<&str>,
            _conversation_id: Option<&str>,
            _message_ids: &str,
            _markdown_path: &str,
            _archived_by: &str,
            _git_commit_sha: Option<&str>,
            _pr_status: Option<&str>,
            _pr_url: Option<&str>,
        ) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn list_archive_records_by_ring(
            &self,
            _ring_id: &str,
        ) -> crate::error::Result<Vec<crate::models::git_model::ArchiveRecord>> {
            unimplemented!()
        }
        async fn get_archive_record(
            &self,
            _id: &str,
        ) -> crate::error::Result<Option<crate::models::git_model::ArchiveRecord>> {
            unimplemented!()
        }
        async fn update_archive_pr_status(
            &self,
            _id: &str,
            _pr_status: &str,
        ) -> crate::error::Result<()> {
            unimplemented!()
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

    fn make_events() -> Vec<LlmEvent> {
        vec![
            LlmEvent::Text {
                content: "hello".into(),
            },
            LlmEvent::Done {
                message_id: Some("msg-1".into()),
                token_usage: Some(TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                }),
            },
        ]
    }

    #[tokio::test]
    async fn super_ring_chat_with_mock_returns_events() {
        let db = Arc::new(MockRepo::new());
        let llm = Arc::new(MockLlmProvider::new(make_events()));
        let dispatcher = Arc::new(ToolDispatcher::new(Arc::new(
            crate::services::tool_engine::ToolRegistry::new(),
        )));
        let svc = AiService::new(db, llm, dispatcher);
        let stream = svc.super_ring_chat("hi".into()).await.unwrap();
        let collected: Vec<LlmEvent> = stream.collect().await;
        assert_eq!(collected.len(), 2);
        assert!(matches!(&collected[0], LlmEvent::Text { content } if content == "hello"));
        assert!(matches!(&collected[1], LlmEvent::Done { .. }));
    }

    #[tokio::test]
    async fn group_ring_chat_saves_message() {
        let db = Arc::new(MockRepo::new());
        let db_ptr = db.as_ref() as *const MockRepo;
        let llm = Arc::new(MockLlmProvider::new(make_events()));
        let dispatcher = Arc::new(ToolDispatcher::new(Arc::new(
            crate::services::tool_engine::ToolRegistry::new(),
        )));
        let svc = AiService::new(db.clone(), llm, dispatcher);
        let _stream = svc
            .group_ring_chat("ring-1", "conv-1", "hello".into())
            .await
            .unwrap();
        assert_eq!(
            unsafe { &*db_ptr }
                .create_message_calls
                .load(Ordering::SeqCst),
            1
        );
    }

    #[tokio::test]
    async fn ai_service_builds_correct_prompt() {
        let db = Arc::new(MockRepo::new());
        let llm = Arc::new(MockLlmProvider::new(make_events()));
        let dispatcher = Arc::new(ToolDispatcher::new(Arc::new(
            crate::services::tool_engine::ToolRegistry::new(),
        )));
        let svc = AiService::new(db, llm, dispatcher);
        let _stream = svc.super_ring_chat("test".into()).await.unwrap();
        let prompt = build_super_ring_prompt();
        assert!(prompt.contains("Super Ring"));
        assert!(prompt.contains("核心能力"));
    }
}
