use async_trait::async_trait;
use futures::stream;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LlmEvent {
    #[serde(rename = "text")]
    Text { content: String },
    #[serde(rename = "tool_call")]
    ToolCall {
        tool_call_id: String,
        tool: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_call_id: String,
        tool: String,
        output: serde_json::Value,
    },
    #[serde(rename = "archive_suggestion")]
    ArchiveSuggestion { data: serde_json::Value },
    #[serde(rename = "blueprint_proposal")]
    BlueprintProposal { data: serde_json::Value },
    #[serde(rename = "error")]
    Error { code: String, message: String },
    #[serde(rename = "done")]
    Done {
        message_id: Option<String>,
        token_usage: Option<TokenUsage>,
    },
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn chat_stream(
        &self,
        messages: Vec<LlmMessage>,
    ) -> Result<Pin<Box<dyn Stream<Item = LlmEvent> + Send>>>;
}

pub struct MockLlmProvider {
    events: Vec<LlmEvent>,
}

impl MockLlmProvider {
    pub fn new(events: Vec<LlmEvent>) -> Self {
        Self { events }
    }
}

#[async_trait]
impl LlmProvider for MockLlmProvider {
    async fn chat_stream(
        &self,
        _messages: Vec<LlmMessage>,
    ) -> Result<Pin<Box<dyn Stream<Item = LlmEvent> + Send>>> {
        let events = self.events.clone();
        Ok(Box::pin(stream::iter(events)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[tokio::test]
    async fn mock_provider_returns_text_events() {
        let events = vec![
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
        ];
        let provider = MockLlmProvider::new(events);
        let stream = provider.chat_stream(vec![]).await.unwrap();
        let collected: Vec<LlmEvent> = stream.collect().await;
        assert_eq!(collected.len(), 2);
        assert!(matches!(&collected[0], LlmEvent::Text { content } if content == "hello"));
        assert!(
            matches!(&collected[1], LlmEvent::Done { message_id: Some(id), .. } if id == "msg-1")
        );
    }

    #[tokio::test]
    async fn mock_provider_returns_empty_stream() {
        let provider = MockLlmProvider::new(vec![]);
        let stream = provider.chat_stream(vec![]).await.unwrap();
        let collected: Vec<LlmEvent> = stream.collect().await;
        assert!(collected.is_empty());
    }

    #[test]
    fn llm_message_serialization() {
        let msg = LlmMessage {
            role: "user".into(),
            content: "hello".into(),
        };
        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["role"], "user");
        assert_eq!(json["content"], "hello");
        let back: LlmMessage = serde_json::from_value(json).unwrap();
        assert_eq!(back.role, "user");
        assert_eq!(back.content, "hello");
    }

    #[test]
    fn llm_event_serialization() {
        let text = LlmEvent::Text {
            content: "hi".into(),
        };
        let json = serde_json::to_value(&text).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["content"], "hi");

        let tool_call = LlmEvent::ToolCall {
            tool_call_id: "tc-1".into(),
            tool: "search".into(),
            input: serde_json::json!({"query": "rust"}),
        };
        let json = serde_json::to_value(&tool_call).unwrap();
        assert_eq!(json["type"], "tool_call");
        assert_eq!(json["tool_call_id"], "tc-1");
        assert_eq!(json["tool"], "search");
        assert_eq!(json["input"]["query"], "rust");

        let tool_result = LlmEvent::ToolResult {
            tool_call_id: "tc-1".into(),
            tool: "search".into(),
            output: serde_json::json!({"results": []}),
        };
        let json = serde_json::to_value(&tool_result).unwrap();
        assert_eq!(json["type"], "tool_result");

        let archive = LlmEvent::ArchiveSuggestion {
            data: serde_json::json!({"doc_id": "d1"}),
        };
        let json = serde_json::to_value(&archive).unwrap();
        assert_eq!(json["type"], "archive_suggestion");
        assert_eq!(json["data"]["doc_id"], "d1");

        let blueprint = LlmEvent::BlueprintProposal {
            data: serde_json::json!({"title": "plan"}),
        };
        let json = serde_json::to_value(&blueprint).unwrap();
        assert_eq!(json["type"], "blueprint_proposal");
        assert_eq!(json["data"]["title"], "plan");

        let error = LlmEvent::Error {
            code: "rate_limit".into(),
            message: "too many requests".into(),
        };
        let json = serde_json::to_value(&error).unwrap();
        assert_eq!(json["type"], "error");
        assert_eq!(json["code"], "rate_limit");
        assert_eq!(json["message"], "too many requests");

        let done = LlmEvent::Done {
            message_id: None,
            token_usage: None,
        };
        let json = serde_json::to_value(&done).unwrap();
        assert_eq!(json["type"], "done");
        assert_eq!(json["message_id"], serde_json::Value::Null);
        assert_eq!(json["token_usage"], serde_json::Value::Null);
    }
}
