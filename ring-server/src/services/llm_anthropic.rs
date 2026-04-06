use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use reqwest::Client as HttpClient;
use serde_json::{json, Value};
use std::pin::Pin;

use crate::error::{Result, RingError};
use crate::services::llm_provider::{LlmEvent, LlmMessage, LlmProvider, TokenUsage};

pub struct AnthropicProvider {
    api_key: String,
    model: String,
    base_url: Option<String>,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            model,
            base_url,
        }
    }

    fn base_url(&self) -> &str {
        self.base_url
            .as_deref()
            .unwrap_or("https://api.anthropic.com")
    }
}

pub(crate) fn convert_messages(messages: &[LlmMessage]) -> (Option<String>, Vec<Value>) {
    let mut system_parts: Vec<String> = Vec::new();
    let mut converted: Vec<Value> = Vec::new();

    for msg in messages {
        if msg.role == "system" {
            system_parts.push(msg.content.clone());
        } else {
            converted.push(json!({
                "role": msg.role,
                "content": msg.content,
            }));
        }
    }

    let system = if system_parts.is_empty() {
        None
    } else {
        Some(system_parts.join("\n"))
    };

    (system, converted)
}

pub(crate) fn parse_sse_event(data: &str) -> Option<LlmEvent> {
    let parsed: Value = serde_json::from_str(data).ok()?;

    match parsed.get("type")?.as_str()? {
        "content_block_delta" => {
            let delta = parsed.get("delta")?;
            if delta.get("type")?.as_str()? == "text_delta" {
                let text = delta.get("text")?.as_str()?.to_string();
                if text.is_empty() {
                    return None;
                }
                return Some(LlmEvent::Text { content: text });
            }
            None
        }
        "message_stop" => Some(LlmEvent::Done {
            message_id: parsed
                .get("message")?
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    parsed
                        .get("id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                }),
            token_usage: parsed.get("message").and_then(|msg| {
                let usage = msg.get("usage")?;
                Some(TokenUsage {
                    prompt_tokens: usage.get("input_tokens")?.as_u64()? as u32,
                    completion_tokens: usage.get("output_tokens")?.as_u64()? as u32,
                    total_tokens: usage.get("input_tokens")?.as_u64()? as u32
                        + usage.get("output_tokens")?.as_u64()? as u32,
                })
            }),
        }),
        "error" => Some(LlmEvent::Error {
            code: parsed
                .get("error")
                .and_then(|e| e.get("type"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            message: parsed
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error")
                .to_string(),
        }),
        _ => None,
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn chat_stream(
        &self,
        messages: Vec<LlmMessage>,
    ) -> Result<Pin<Box<dyn Stream<Item = LlmEvent> + Send>>> {
        let (system, converted_messages) = convert_messages(&messages);

        let mut body = json!({
            "model": self.model,
            "messages": converted_messages,
            "max_tokens": 4096,
            "stream": true,
        });

        if let Some(sys) = system {
            body["system"] = json!(sys);
        }

        let url = format!("{}/v1/messages", self.base_url());

        let http_client = HttpClient::new();
        let response = http_client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| RingError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RingError::Llm(format!(
                "anthropic api error: {} - {}",
                status, body
            )));
        }

        let stream = response
            .bytes_stream()
            .scan(Vec::<u8>::new(), |buffer, chunk| {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => {
                        return std::future::ready(Some(vec![LlmEvent::Error {
                            code: "stream_error".into(),
                            message: e.to_string(),
                        }]));
                    }
                };
                buffer.extend_from_slice(&chunk);
                let text = String::from_utf8_lossy(buffer);
                let mut events = Vec::new();
                let mut consumed = 0;
                while let Some(pos) = text[consumed..].find("\n\n") {
                    let raw = &text[consumed..consumed + pos];
                    consumed += pos + 2;
                    for line in raw.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data == "[DONE]" {
                                continue;
                            }
                            if let Some(event) = parse_sse_event(data) {
                                events.push(event);
                            }
                        }
                    }
                }
                buffer.drain(..consumed);
                std::future::ready(Some(events))
            })
            .flat_map(futures::stream::iter);

        Ok(Box::pin(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_messages_to_anthropic_format() {
        let messages = vec![
            LlmMessage {
                role: "system".into(),
                content: "You are a helper.".into(),
            },
            LlmMessage {
                role: "user".into(),
                content: "Hello.".into(),
            },
            LlmMessage {
                role: "assistant".into(),
                content: "Hi there!".into(),
            },
        ];
        let (system, converted) = convert_messages(&messages);
        assert_eq!(system, Some("You are a helper.".into()));
        assert_eq!(converted.len(), 2);
        assert_eq!(converted[0]["role"], "user");
        assert_eq!(converted[0]["content"], "Hello.");
        assert_eq!(converted[1]["role"], "assistant");
        assert_eq!(converted[1]["content"], "Hi there!");
    }

    #[test]
    fn convert_messages_merges_system() {
        let messages = vec![
            LlmMessage {
                role: "system".into(),
                content: "Part one.".into(),
            },
            LlmMessage {
                role: "user".into(),
                content: "Hi".into(),
            },
            LlmMessage {
                role: "system".into(),
                content: "Part two.".into(),
            },
        ];
        let (system, converted) = convert_messages(&messages);
        assert_eq!(system, Some("Part one.\nPart two.".into()));
        assert_eq!(converted.len(), 1);
        assert_eq!(converted[0]["role"], "user");
    }

    #[test]
    fn convert_messages_no_system() {
        let messages = vec![LlmMessage {
            role: "user".into(),
            content: "Hello".into(),
        }];
        let (system, converted) = convert_messages(&messages);
        assert_eq!(system, None);
        assert_eq!(converted.len(), 1);
    }

    #[test]
    fn parse_anthropic_text_event() {
        let data = r#"{"type":"content_block_delta","delta":{"type":"text_delta","text":"Hello"}}"#;
        let event = parse_sse_event(data).unwrap();
        assert!(matches!(&event, LlmEvent::Text { content } if content == "Hello"));
    }

    #[test]
    fn parse_anthropic_text_event_empty_ignored() {
        let data = r#"{"type":"content_block_delta","delta":{"type":"text_delta","text":""}}"#;
        assert!(parse_sse_event(data).is_none());
    }

    #[test]
    fn parse_anthropic_done_event() {
        let data = r#"{"type":"message_stop","message":{"id":"msg-123","usage":{"input_tokens":10,"output_tokens":5}}}"#;
        let event = parse_sse_event(data).unwrap();
        assert!(matches!(
            &event,
            LlmEvent::Done {
                message_id: Some(id),
                token_usage: Some(TokenUsage { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 }),
            }
            if id == "msg-123"
        ));
    }

    #[test]
    fn parse_anthropic_done_event_with_id_at_top_level() {
        let data = r#"{"type":"message_stop","id":"msg-456","message":{"usage":{"input_tokens":8,"output_tokens":3}}}"#;
        let event = parse_sse_event(data).unwrap();
        assert!(matches!(
            &event,
            LlmEvent::Done {
                message_id: Some(id),
                token_usage: Some(TokenUsage { prompt_tokens: 8, completion_tokens: 3, total_tokens: 11 }),
            }
            if id == "msg-456"
        ));
    }

    #[test]
    fn parse_anthropic_error_event() {
        let data = r#"{"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}"#;
        let event = parse_sse_event(data).unwrap();
        assert!(matches!(
            &event,
            LlmEvent::Error { code, message }
            if code == "overloaded_error" && message == "Overloaded"
        ));
    }

    #[test]
    fn parse_anthropic_unknown_type_returns_none() {
        let data = r#"{"type":"message_start","message":{"id":"msg-abc"}}"#;
        assert!(parse_sse_event(data).is_none());
    }

    #[test]
    fn parse_anthropic_invalid_json_returns_none() {
        let data = "not json";
        assert!(parse_sse_event(data).is_none());
    }
}
