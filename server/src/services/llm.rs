use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent, ChatCompletionRequestToolMessage,
    ChatCompletionRequestToolMessageContent, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent, ChatCompletionTool, CreateChatCompletionRequest,
};
use async_openai::Client;
use futures_util::StreamExt;

use crate::error::{Result, RingError};
use crate::models::user::UserRow;

#[derive(Clone)]
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
        token_usage: Option<String>,
    },
    Error(String),
}

pub fn sse_event_to_axum(event: SseEvent) -> axum::response::sse::Event {
    match event {
        SseEvent::Start { message_id, role } => {
            let data = serde_json::json!({"message_id": message_id, "role": role});
            axum::response::sse::Event::default()
                .event("message_start")
                .data(data.to_string())
        }
        SseEvent::Delta { content } => {
            let data = serde_json::json!({ "content": content });
            axum::response::sse::Event::default()
                .event("delta")
                .data(data.to_string())
        }
        SseEvent::End {
            message_id,
            full_content: _,
            token_usage,
        } => {
            let usage_json = token_usage
                .as_deref()
                .and_then(|u| serde_json::from_str::<serde_json::Value>(u).ok());
            let data = serde_json::json!({
                "message_id": message_id,
                "usage": usage_json.unwrap_or(serde_json::json!({ "prompt_tokens": 0, "completion_tokens": 0 }))
            });
            axum::response::sse::Event::default()
                .event("message_end")
                .data(data.to_string())
        }
        SseEvent::Error(msg) => {
            let data = serde_json::json!({ "error": msg });
            axum::response::sse::Event::default()
                .event("error")
                .data(data.to_string())
        }
    }
}

pub enum ChatCompleteWithToolsResult {
    Message {
        content: String,
    },
    ToolCalls {
        tool_calls: Vec<async_openai::types::ChatCompletionMessageToolCall>,
    },
}

pub fn build_messages(
    system_prompt: String,
    history: Vec<(String, String)>,
    user_message: String,
) -> Vec<ChatCompletionRequestMessage> {
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
                        content: Some(ChatCompletionRequestAssistantMessageContent::Text(content)),
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

    messages
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

    pub fn chat_stream<C>(
        self,
        system_prompt: String,
        history: Vec<(String, String)>,
        user_message: String,
        ai_role: String,
        on_complete: C,
    ) -> tokio::sync::mpsc::Receiver<SseEvent>
    where
        C: Fn(
                String,
                Option<String>,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + 'static,
    {
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tokio::spawn(async move {
            let _start_time = std::time::Instant::now();
            let message_id = ulid::Ulid::new().to_string();

            let _ = tx
                .send(SseEvent::Start {
                    message_id: message_id.clone(),
                    role: ai_role.clone(),
                })
                .await;

            let messages = build_messages(system_prompt, history, user_message);
            tracing::info!("{ai_role}: messages count = {}", messages.len());

            let request = CreateChatCompletionRequest {
                messages: messages.clone(),
                model: self.model.clone(),
                stream: Some(true),
                ..Default::default()
            };

            let api_start = std::time::Instant::now();
            tracing::info!(
                "{ai_role}: calling LLM API with model {} at {:?}",
                self.model,
                api_start.elapsed().as_secs_f64()
            );
            match self.client.chat().create_stream(request).await {
                Ok(mut stream) => {
                    let stream_start = std::time::Instant::now();
                    tracing::info!(
                        "{ai_role}: stream started at {:?}",
                        stream_start.elapsed().as_secs_f64()
                    );
                    let mut full_content = String::new();
                    let mut token_usage: Option<String> = None;
                    let mut chunk_count = 0;
                    let mut first_chunk_time = None;
                    let mut last_chunk_time = None;
                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(chunk) => {
                                let chunk_time = std::time::Instant::now();
                                if chunk_count == 0 {
                                    first_chunk_time = Some(chunk_time);
                                    tracing::info!(
                                        "{ai_role}: first chunk received at {:?}",
                                        chunk_time.elapsed().as_secs_f64()
                                    );
                                }
                                last_chunk_time = Some(chunk_time);
                                chunk_count += 1;
                                tracing::debug!("{ai_role}: received chunk");
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
                                if let Some(usage) = &chunk.usage {
                                    token_usage =
                                        Some(serde_json::to_string(usage).unwrap_or_default());
                                }
                            }
                            Err(e) => {
                                tracing::error!("{ai_role}: stream error: {}", e);
                                let _ = tx.send(SseEvent::Error(e.to_string())).await;
                                break;
                            }
                        }
                    }
                    let stream_end = std::time::Instant::now();
                    tracing::info!("{ai_role}: stream ended at {:?}, chunks={}, first_chunk_delay={:?}, last_chunk_delay={:?}", 
                        stream_end.elapsed().as_secs_f64(),
                        chunk_count,
                        first_chunk_time.map(|t| t.elapsed().as_secs_f64()),
                        last_chunk_time.map(|t| t.elapsed().as_secs_f64())
                    );
                    let _ = tx
                        .send(SseEvent::End {
                            message_id: message_id.clone(),
                            full_content: full_content.clone(),
                            token_usage: token_usage.clone(),
                        })
                        .await;
                    on_complete(full_content, token_usage).await;
                }
                Err(e) => {
                    tracing::error!(
                        "{ai_role}: API call error at {:?}: {}",
                        api_start.elapsed().as_secs_f64(),
                        e
                    );
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
        let messages = build_messages(system_prompt, history, user_message);

        let request = CreateChatCompletionRequest {
            messages,
            model: self.model,
            tools: Some(tools),
            tool_choice: Some(async_openai::types::ChatCompletionToolChoiceOption::Auto),
            ..Default::default()
        };

        let response = self.client.chat().create(request).await?;
        let choice = response
            .choices
            .first()
            .ok_or_else(|| crate::error::RingError::Internal("no choices in response".into()))?;

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

    #[allow(clippy::too_many_arguments)]
    pub fn chat_stream_with_tools<F, C>(
        self,
        system_prompt: String,
        history: Vec<(String, String)>,
        user_message: String,
        ai_role: String,
        tools: Vec<async_openai::types::ChatCompletionTool>,
        tool_executor: F,
        on_complete: C,
    ) -> tokio::sync::mpsc::Receiver<SseEvent>
    where
        F: Fn(
                String,
                serde_json::Value,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = crate::error::Result<String>> + Send>,
            > + Send
            + 'static,
        C: Fn(
                String,
                Option<String>,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + 'static,
    {
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tokio::spawn(async move {
            let message_id = ulid::Ulid::new().to_string();

            let _ = tx
                .send(SseEvent::Start {
                    message_id: message_id.clone(),
                    role: ai_role.clone(),
                })
                .await;

            let mut messages = build_messages(system_prompt, history, user_message);

            let request = CreateChatCompletionRequest {
                messages: messages.clone(),
                model: self.model.clone(),
                tools: Some(tools),
                tool_choice: Some(async_openai::types::ChatCompletionToolChoiceOption::Auto),
                stream: Some(true),
                ..Default::default()
            };

            let client = self.client.clone();
            let model = self.model.clone();

            match client.chat().create_stream(request).await {
                Ok(mut stream) => {
                    let mut first_pass_content = String::new();
                    let mut first_pass_usage: Option<String> = None;
                    let mut tool_calls: Vec<async_openai::types::ChatCompletionMessageToolCall> =
                        Vec::new();
                    let mut has_tool_calls = false;

                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(chunk) => {
                                if let Some(choice) = chunk.choices.first() {
                                    if let Some(delta) = &choice.delta.content {
                                        first_pass_content.push_str(delta);
                                        let _ = tx
                                            .send(SseEvent::Delta {
                                                content: delta.clone(),
                                            })
                                            .await;
                                    }
                                    if let Some(tc) = &choice.delta.tool_calls {
                                        has_tool_calls = true;
                                        for tc_delta in tc {
                                            let idx = tc_delta.index as usize;
                                            while tool_calls.len() <= idx {
                                                tool_calls.push(async_openai::types::ChatCompletionMessageToolCall {
                                                    id: String::new(),
                                                    r#type: async_openai::types::ChatCompletionToolType::Function,
                                                    function: async_openai::types::FunctionCall {
                                                        name: String::new(),
                                                        arguments: String::new(),
                                                    },
                                                });
                                            }
                                            if let Some(id) = &tc_delta.id {
                                                tool_calls[idx].id = id.clone();
                                            }
                                            if let Some(func) = &tc_delta.function {
                                                if let Some(name) = &func.name {
                                                    tool_calls[idx].function.name = name.clone();
                                                }
                                                if let Some(args) = &func.arguments {
                                                    tool_calls[idx]
                                                        .function
                                                        .arguments
                                                        .push_str(args);
                                                }
                                            }
                                        }
                                    }
                                    if choice.finish_reason
                                        == Some(async_openai::types::FinishReason::ToolCalls)
                                    {
                                        break;
                                    }
                                }
                                if let Some(usage) = &chunk.usage {
                                    first_pass_usage =
                                        Some(serde_json::to_string(usage).unwrap_or_default());
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(SseEvent::Error(e.to_string())).await;
                                let _ = tx
                                    .send(SseEvent::End {
                                        message_id: message_id.clone(),
                                        full_content: first_pass_content.clone(),
                                        token_usage: first_pass_usage.clone(),
                                    })
                                    .await;
                                on_complete(first_pass_content, first_pass_usage).await;
                                return;
                            }
                        }
                    }

                    if has_tool_calls && !tool_calls.is_empty() {
                        let mut tool_messages = vec![];

                        for tc in &tool_calls {
                            let args: serde_json::Value =
                                serde_json::from_str(&tc.function.arguments)
                                    .unwrap_or(serde_json::json!({}));
                            let result = tool_executor(tc.function.name.clone(), args).await;

                            let tool_result = match result {
                                Ok(r) => r,
                                Err(e) => format!("Error: {e}"),
                            };

                            tool_messages.push(ChatCompletionRequestMessage::Tool(
                                ChatCompletionRequestToolMessage {
                                    content: ChatCompletionRequestToolMessageContent::Text(
                                        tool_result,
                                    ),
                                    tool_call_id: tc.id.clone(),
                                },
                            ));
                        }

                        messages.push(ChatCompletionRequestMessage::Assistant(
                            #[allow(deprecated)]
                            ChatCompletionRequestAssistantMessage {
                                content: Some(ChatCompletionRequestAssistantMessageContent::Text(
                                    first_pass_content.clone(),
                                )),
                                name: None,
                                tool_calls: Some(tool_calls),
                                refusal: None,
                                audio: None,
                                function_call: None,
                            },
                        ));

                        for tm in tool_messages {
                            messages.push(tm);
                        }

                        let second_request = CreateChatCompletionRequest {
                            messages,
                            model,
                            stream: Some(true),
                            ..Default::default()
                        };

                        let mut combined = first_pass_content.clone();
                        match client.chat().create_stream(second_request).await {
                            Ok(mut stream2) => {
                                let mut token_usage: Option<String> = None;
                                while let Some(result) = stream2.next().await {
                                    match result {
                                        Ok(chunk) => {
                                            if let Some(choice) = chunk.choices.first() {
                                                if let Some(delta) = &choice.delta.content {
                                                    combined.push_str(delta);
                                                    let _ = tx
                                                        .send(SseEvent::Delta {
                                                            content: delta.clone(),
                                                        })
                                                        .await;
                                                }
                                            }
                                            if let Some(usage) = &chunk.usage {
                                                token_usage = Some(
                                                    serde_json::to_string(usage)
                                                        .unwrap_or_default(),
                                                );
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
                                        full_content: combined.clone(),
                                        token_usage: token_usage.clone(),
                                    })
                                    .await;
                                on_complete(combined, token_usage).await;
                            }
                            Err(e) => {
                                tracing::error!("{ai_role}: API call error: {}", e);
                                let _ = tx.send(SseEvent::Error(e.to_string())).await;
                                let _ = tx
                                    .send(SseEvent::End {
                                        message_id: message_id.clone(),
                                        full_content: combined.clone(),
                                        token_usage: None,
                                    })
                                    .await;
                                on_complete(combined, None).await;
                            }
                        }
                    } else {
                        let _ = tx
                            .send(SseEvent::End {
                                message_id: message_id.clone(),
                                full_content: first_pass_content.clone(),
                                token_usage: first_pass_usage.clone(),
                            })
                            .await;
                        on_complete(first_pass_content, first_pass_usage).await;
                    }
                }
                Err(e) => {
                    let _ = tx.send(SseEvent::Error(e.to_string())).await;
                }
            }
        });

        rx
    }
}

pub async fn test_connection(
    provider: &str,
    model: &str,
    api_key: Option<&str>,
    base_url: Option<&str>,
) -> crate::error::Result<(bool, String)> {
    let key = if provider == "ollama" {
        api_key.unwrap_or("ollama").to_string()
    } else {
        api_key
            .ok_or_else(|| crate::error::RingError::BadRequest("API key required".into()))?
            .to_string()
    };

    let mut config = OpenAIConfig::new().with_api_key(&key);
    if let Some(url) = base_url {
        if !url.is_empty() {
            config = config.with_api_base(url);
        }
    }
    if provider == "ollama" && base_url.is_none_or(|u| u.is_empty()) {
        config = config.with_api_base("http://localhost:11434/v1");
    }

    let client = Client::with_config(config);
    let messages = vec![
        ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
            content: ChatCompletionRequestSystemMessageContent::Text(
                "Respond with only the word OK.".into(),
            ),
            name: None,
        }),
        ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
            content: ChatCompletionRequestUserMessageContent::Text("test".into()),
            name: None,
        }),
    ];

    let request = CreateChatCompletionRequest {
        messages,
        model: model.to_string(),
        ..Default::default()
    };

    match tokio::time::timeout(
        std::time::Duration::from_secs(15),
        client.chat().create(request),
    )
    .await
    {
        Ok(Ok(_)) => Ok((true, "Connection successful".into())),
        Ok(Err(e)) => Ok((false, format!("{e}"))),
        Err(_) => Ok((false, "Connection timed out after 15s".into())),
    }
}
