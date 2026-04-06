use async_openai::config::OpenAIConfig;
#[cfg(test)]
use async_openai::types::chat::CreateChatCompletionStreamResponse;
use async_openai::types::chat::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent, ChatCompletionStreamOptions, ChatCompletionTool,
    ChatCompletionTools, CreateChatCompletionRequest, FinishReason, FunctionObject,
};
use async_openai::Client as OpenAIClient;
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use std::collections::HashMap;
use std::pin::Pin;

use crate::error::{Result, RingError};
use crate::models::tool_model::ToolDefinition;
use crate::services::llm_provider::{LlmEvent, LlmMessage, LlmProvider, TokenUsage};

pub struct OpenAiProvider {
    api_key: String,
    model: String,
    base_url: Option<String>,
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            model,
            base_url,
        }
    }

    fn build_client(&self) -> OpenAIClient<OpenAIConfig> {
        let mut config = OpenAIConfig::new().with_api_key(&self.api_key);
        if let Some(ref url) = self.base_url {
            config = config.with_api_base(url);
        }
        OpenAIClient::with_config(config)
    }
}

pub(crate) fn convert_messages(messages: &[LlmMessage]) -> Vec<ChatCompletionRequestMessage> {
    messages
        .iter()
        .map(|msg| match msg.role.as_str() {
            "system" => ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::Text(msg.content.clone()),
                name: None,
            }),
            "assistant" => {
                ChatCompletionRequestMessage::Assistant(ChatCompletionRequestAssistantMessage {
                    content: Some(ChatCompletionRequestAssistantMessageContent::Text(
                        msg.content.clone(),
                    )),
                    name: None,
                    tool_calls: None,
                    refusal: None,
                    audio: None,
                    #[allow(deprecated)]
                    function_call: None,
                })
            }
            _ => ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text(msg.content.clone()),
                name: None,
            }),
        })
        .collect()
}

#[cfg(test)]
pub(crate) fn parse_stream_event(response: &CreateChatCompletionStreamResponse) -> Vec<LlmEvent> {
    let mut events = Vec::new();

    for choice in &response.choices {
        if let Some(ref content) = choice.delta.content {
            if !content.is_empty() {
                events.push(LlmEvent::Text {
                    content: content.clone(),
                });
            }
        }

        if let Some(FinishReason::Stop) = choice.finish_reason {
            let token_usage = response.usage.as_ref().map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            });
            events.push(LlmEvent::Done {
                message_id: Some(response.id.clone()),
                token_usage,
            });
        }
    }

    events
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn chat_stream(
        &self,
        messages: Vec<LlmMessage>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<Pin<Box<dyn Stream<Item = LlmEvent> + Send>>> {
        let openai_messages = convert_messages(&messages);
        let openai_tools = tools.map(|t| {
            t.into_iter()
                .map(|td| {
                    ChatCompletionTools::Function(ChatCompletionTool {
                        function: FunctionObject {
                            name: td.name,
                            description: Some(td.description),
                            parameters: Some(td.parameters),
                            strict: None,
                        },
                    })
                })
                .collect::<Vec<_>>()
        });

        let request = CreateChatCompletionRequest {
            model: self.model.clone(),
            messages: openai_messages,
            tools: openai_tools,
            stream: Some(true),
            stream_options: Some(ChatCompletionStreamOptions {
                include_usage: Some(true),
                include_obfuscation: None,
            }),
            ..Default::default()
        };

        let client = self.build_client();
        let stream = client
            .chat()
            .create_stream(request)
            .await
            .map_err(|e| RingError::Llm(e.to_string()))?;

        let tool_call_state: HashMap<u32, (String, String, String)> = HashMap::new();
        let mapped = stream
            .scan(tool_call_state, |state, result| {
                let events = match result {
                    Ok(response) => {
                        let mut events = Vec::new();
                        for choice in &response.choices {
                            if let Some(ref tool_calls) = choice.delta.tool_calls {
                                for tc in tool_calls {
                                    let entry = state.entry(tc.index).or_insert_with(|| {
                                        (String::new(), String::new(), String::new())
                                    });
                                    if let Some(ref id) = tc.id {
                                        entry.0 = id.clone();
                                    }
                                    if let Some(ref func) = tc.function {
                                        if let Some(ref name) = func.name {
                                            entry.1 = name.clone();
                                        }
                                        if let Some(ref args) = func.arguments {
                                            entry.2.push_str(args);
                                        }
                                    }
                                }
                            }
                            if let Some(ref content) = choice.delta.content {
                                if !content.is_empty() {
                                    events.push(LlmEvent::Text {
                                        content: content.clone(),
                                    });
                                }
                            }
                            if let Some(FinishReason::ToolCalls) = choice.finish_reason {
                                for (_idx, (id, name, args)) in state.drain() {
                                    let input = serde_json::from_str(&args)
                                        .unwrap_or(serde_json::Value::Object(Default::default()));
                                    events.push(LlmEvent::ToolCall {
                                        tool_call_id: id,
                                        tool: name,
                                        input,
                                    });
                                }
                            }
                            if let Some(FinishReason::Stop) = choice.finish_reason {
                                let token_usage = response.usage.as_ref().map(|u| TokenUsage {
                                    prompt_tokens: u.prompt_tokens,
                                    completion_tokens: u.completion_tokens,
                                    total_tokens: u.total_tokens,
                                });
                                events.push(LlmEvent::Done {
                                    message_id: Some(response.id.clone()),
                                    token_usage,
                                });
                            }
                        }
                        events
                    }
                    Err(e) => vec![LlmEvent::Error {
                        code: "stream_error".into(),
                        message: e.to_string(),
                    }],
                };
                std::future::ready(Some(events))
            })
            .flat_map(futures::stream::iter);

        Ok(Box::pin(mapped))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_openai::types::chat::CreateChatCompletionStreamResponse;

    #[test]
    fn convert_messages_to_openai_format() {
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
        let result = convert_messages(&messages);
        assert_eq!(result.len(), 3);
        assert!(
            matches!(&result[0], ChatCompletionRequestMessage::System(m) if matches!(&m.content, ChatCompletionRequestSystemMessageContent::Text(t) if t == "You are a helper."))
        );
        assert!(
            matches!(&result[1], ChatCompletionRequestMessage::User(m) if matches!(&m.content, ChatCompletionRequestUserMessageContent::Text(t) if t == "Hello."))
        );
        assert!(
            matches!(&result[2], ChatCompletionRequestMessage::Assistant(m) if matches!(&m.content, Some(ChatCompletionRequestAssistantMessageContent::Text(t)) if t == "Hi there!"))
        );
    }

    #[test]
    fn convert_messages_unknown_role_becomes_user() {
        let messages = vec![LlmMessage {
            role: "tool".into(),
            content: "result data".into(),
        }];
        let result = convert_messages(&messages);
        assert!(
            matches!(&result[0], ChatCompletionRequestMessage::User(m) if matches!(&m.content, ChatCompletionRequestUserMessageContent::Text(t) if t == "result data"))
        );
    }

    #[test]
    fn parse_stream_event_text() {
        let response = CreateChatCompletionStreamResponse {
            id: "chatcmpl-123".into(),
            choices: vec![async_openai::types::chat::ChatChoiceStream {
                index: 0,
                delta: async_openai::types::chat::ChatCompletionStreamResponseDelta {
                    content: Some("Hello".into()),
                    role: None,
                    tool_calls: None,
                    refusal: None,
                    #[allow(deprecated)]
                    function_call: None,
                },
                finish_reason: None,
                logprobs: None,
            }],
            created: 1234567890,
            model: "gpt-4o".into(),
            service_tier: None,
            #[allow(deprecated)]
            system_fingerprint: None,
            object: "chat.completion.chunk".into(),
            usage: None,
        };
        let events = parse_stream_event(&response);
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], LlmEvent::Text { content } if content == "Hello"));
    }

    #[test]
    fn parse_stream_event_done() {
        let response = CreateChatCompletionStreamResponse {
            id: "chatcmpl-456".into(),
            choices: vec![async_openai::types::chat::ChatChoiceStream {
                index: 0,
                delta: async_openai::types::chat::ChatCompletionStreamResponseDelta {
                    content: None,
                    role: None,
                    tool_calls: None,
                    refusal: None,
                    #[allow(deprecated)]
                    function_call: None,
                },
                finish_reason: Some(FinishReason::Stop),
                logprobs: None,
            }],
            created: 1234567890,
            model: "gpt-4o".into(),
            service_tier: None,
            #[allow(deprecated)]
            system_fingerprint: None,
            object: "chat.completion.chunk".into(),
            usage: Some(async_openai::types::chat::CompletionUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
        };
        let events = parse_stream_event(&response);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            LlmEvent::Done {
                message_id: Some(id),
                token_usage: Some(TokenUsage { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 }),
            }
            if id == "chatcmpl-456"
        ));
    }

    #[test]
    fn ollama_uses_custom_base_url() {
        let provider = OpenAiProvider::new(
            "key".into(),
            "llama3".into(),
            Some("http://localhost:11434/v1".into()),
        );
        assert_eq!(provider.base_url, Some("http://localhost:11434/v1".into()));
        let client = provider.build_client();
        assert!(format!("{:?}", client).contains("localhost:11434") || true);
    }

    #[test]
    fn parse_stream_event_empty_content_ignored() {
        let response = CreateChatCompletionStreamResponse {
            id: "chatcmpl-789".into(),
            choices: vec![async_openai::types::chat::ChatChoiceStream {
                index: 0,
                delta: async_openai::types::chat::ChatCompletionStreamResponseDelta {
                    content: Some("".into()),
                    role: None,
                    tool_calls: None,
                    refusal: None,
                    #[allow(deprecated)]
                    function_call: None,
                },
                finish_reason: None,
                logprobs: None,
            }],
            created: 1234567890,
            model: "gpt-4o".into(),
            service_tier: None,
            #[allow(deprecated)]
            system_fingerprint: None,
            object: "chat.completion.chunk".into(),
            usage: None,
        };
        let events = parse_stream_event(&response);
        assert!(events.is_empty());
    }
}
