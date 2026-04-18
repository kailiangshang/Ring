use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent, CreateChatCompletionRequest, ChatCompletionTool,
};
use async_openai::Client;
use futures_util::StreamExt;

use crate::error::{Result, RingError};
use crate::models::user::UserRow;

pub struct LlmClient {
    client: Client<OpenAIConfig>,
    model: String,
}

pub enum SseEvent {
    Start {
        message_id: String,
        role: String,
    },
    Delta {
        content: String,
    },
    End {
        message_id: String,
        full_content: String,
    },
    Error(String),
}

pub enum ChatCompleteWithToolsResult {
    Message { content: String },
    ToolCalls { tool_calls: Vec<async_openai::types::ChatCompletionMessageToolCall> },
}

impl LlmClient {
    pub fn from_user(user: &UserRow) -> Result<Self> {
        let api_key = user
            .llm_api_key
            .as_deref()
            .ok_or_else(|| RingError::Internal("LLM API key not configured".into()))?;

        let mut config = OpenAIConfig::new().with_api_key(api_key);
        if let Some(base_url) = &user.llm_base_url {
            config = config.with_api_base(base_url);
        }

        Ok(Self {
            client: Client::with_config(config),
            model: user.llm_model.clone(),
        })
    }

    pub fn chat_stream(
        self,
        system_prompt: String,
        history: Vec<(String, String)>,
        user_message: String,
        ai_role: String,
    ) -> tokio::sync::mpsc::Receiver<SseEvent> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tokio::spawn(async move {
            let message_id = ulid::Ulid::new().to_string();

            let _ = tx
                .send(SseEvent::Start {
                    message_id: message_id.clone(),
                    role: ai_role,
                })
                .await;

            let mut messages = vec![ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text(system_prompt),
                    name: None,
                },
            )];

            for (role, content) in history {
                match role.as_str() {
                    "user" => {
                        messages.push(ChatCompletionRequestMessage::User(
                            ChatCompletionRequestUserMessage {
                                content: ChatCompletionRequestUserMessageContent::Text(content),
                                name: None,
                            },
                        ));
                    }
                    _ => {
                        messages.push(ChatCompletionRequestMessage::Assistant(
                            #[allow(deprecated)]
                            ChatCompletionRequestAssistantMessage {
                                content: Some(ChatCompletionRequestAssistantMessageContent::Text(
                                    content,
                                )),
                                name: None,
                                tool_calls: None,
                                refusal: None,
                                audio: None,
                                function_call: None,
                            },
                        ));
                    }
                }
            }

            messages.push(ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(user_message),
                    name: None,
                },
            ));

            let request = CreateChatCompletionRequest {
                messages,
                model: self.model,
                stream: Some(true),
                ..Default::default()
            };

            match self.client.chat().create_stream(request).await {
                Ok(mut stream) => {
                    let mut full_content = String::new();
                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(chunk) => {
                                if let Some(choice) = chunk.choices.first() {
                                    if let Some(delta) = &choice.delta.content {
                                        full_content.push_str(delta);
                                        let _ = tx
                                            .send(SseEvent::Delta {
                                                content: delta.clone(),
                                            })
                                            .await;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(SseEvent::Error(e.to_string())).await;
                                break;
                            }
                        }
                    }
                    let _ = tx
                        .send(SseEvent::End {
                            message_id: message_id.clone(),
                            full_content,
                        })
                        .await;
                }
                Err(e) => {
                    let _ = tx.send(SseEvent::Error(e.to_string())).await;
                }
            }
        });

        rx
    }

    pub async fn chat_complete(
        self,
        system_prompt: String,
        user_message: String,
    ) -> crate::error::Result<String> {
        let messages = vec![
            ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::Text(system_prompt),
                name: None,
            }),
            ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text(user_message),
                name: None,
            }),
        ];

        let request = CreateChatCompletionRequest {
            messages,
            model: self.model,
            ..Default::default()
        };

        let response = self.client.chat().create(request).await?;
        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();
        Ok(content)
    }

    pub async fn chat_complete_with_tools(
        self,
        system_prompt: String,
        history: Vec<(String, String)>,
        user_message: String,
        tools: Vec<ChatCompletionTool>,
    ) -> crate::error::Result<ChatCompleteWithToolsResult> {
        let mut messages = vec![ChatCompletionRequestMessage::System(
            ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::Text(system_prompt),
                name: None,
            },
        )];

        for (role, content) in history {
            match role.as_str() {
                "user" => {
                    messages.push(ChatCompletionRequestMessage::User(
                        ChatCompletionRequestUserMessage {
                            content: ChatCompletionRequestUserMessageContent::Text(content),
                            name: None,
                        },
                    ));
                }
                _ => {
                    messages.push(ChatCompletionRequestMessage::Assistant(
                        #[allow(deprecated)]
                        ChatCompletionRequestAssistantMessage {
                            content: Some(ChatCompletionRequestAssistantMessageContent::Text(
                                content,
                            )),
                            name: None,
                            tool_calls: None,
                            refusal: None,
                            audio: None,
                            function_call: None,
                        },
                    ));
                }
            }
        }

        messages.push(ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text(user_message),
                name: None,
            },
        ));

        let request = CreateChatCompletionRequest {
            messages,
            model: self.model,
            tools: Some(tools),
            tool_choice: Some(async_openai::types::ChatCompletionToolChoiceOption::Auto),
            ..Default::default()
        };

        let response = self.client.chat().create(request).await?;
        let choice = response.choices.first().ok_or_else(|| {
            crate::error::RingError::Internal("no choices in response".into())
        })?;

        if let Some(tool_calls) = &choice.message.tool_calls {
            Ok(ChatCompleteWithToolsResult::ToolCalls {
                tool_calls: tool_calls.clone(),
            })
        } else {
            Ok(ChatCompleteWithToolsResult::Message {
                content: choice.message.content.clone().unwrap_or_default(),
            })
        }
    }
}
