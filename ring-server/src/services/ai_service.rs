use std::pin::Pin;
use std::sync::Arc;

use futures::Stream;

use crate::db::traits::Repository;
use crate::error::{Result, RingError};
use crate::services::context_loader::{
    build_blueprint_prompt, build_group_ring_prompt, build_super_ring_prompt,
};
use crate::services::llm_provider::{LlmEvent, LlmMessage, LlmProvider};

pub struct AiService {
    db: Arc<dyn Repository>,
    llm: Arc<dyn LlmProvider>,
}

impl AiService {
    pub fn new(db: Arc<dyn Repository>, llm: Arc<dyn LlmProvider>) -> Self {
        AiService { db, llm }
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
        self.llm.chat_stream(messages).await
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

        self.llm.chat_stream(messages).await
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
        self.llm.chat_stream(messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::llm_provider::{MockLlmProvider, TokenUsage};
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
        let svc = AiService::new(db, llm);
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
        let svc = AiService::new(db.clone(), llm);
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
        let svc = AiService::new(db, llm);
        let _stream = svc.super_ring_chat("test".into()).await.unwrap();
        let prompt = build_super_ring_prompt();
        assert!(prompt.contains("Super Ring"));
        assert!(prompt.contains("核心能力"));
    }
}
