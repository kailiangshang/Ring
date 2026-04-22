# Super Ring 跨 Ring 能力实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 Super Ring 的跨 Ring 查询和分析能力，包括后端 API 和前端集成

**Architecture:** 复用现有的 SSE 流式聊天架构，新增 `/api/super/cross-ring-query` 和 `/api/super/cross-ring-analysis` 端点。跨 Ring 查询通过读取所有 Ring 的图谱和归档数据，构建 prompt 后流式返回 LLM 生成的分析结果。

**Tech Stack:** Rust + Axum + async-openai, React + TypeScript + Zustand

---

## 文件结构

| 文件 | 改动 | 说明 |
|------|------|------|
| `server/src/routes/super_chat.rs` | 修改 | 新增 `cross_ring_query_handler` 和 `cross_ring_analysis_handler` |
| `server/src/services/super_chat.rs` | 修改 | 新增 `stream_cross_ring_query` 和 `stream_cross_ring_analysis` 函数 |
| `server/src/routes/mod.rs` | 修改 | 注册新路由 `/super/cross-ring-query` 和 `/super/cross-ring-analysis` |
| `ui/src/services/api.ts` | 修改 | 新增 `crossRingQuery` 和 `crossRingAnalysis` API 函数 |
| `ui/src/stores/chat-store.ts` | 修改 | 集成跨 Ring 查询命令到聊天流程 |
| `server/tests/integration.rs` | 修改 | 新增集成测试 |

---

## Task 1: 后端 - 跨 Ring 查询服务函数

**Files:**
- Modify: `server/src/services/super_chat.rs`

- [ ] **Step 1: 在 `server/src/services/super_chat.rs` 中新增跨 Ring 查询函数**

在文件末尾（`get_super_history` 函数之后）添加：

```rust
pub fn stream_cross_ring_query(
    state: AppState,
    user: crate::models::user::UserRow,
    query: String,
) -> tokio::sync::mpsc::Receiver<SseEvent> {
    let (tx, rx) = tokio::sync::mpsc::channel(32);

    tokio::spawn(async move {
        if let Err(e) = stream_cross_ring_query_inner(state, user, query, &tx).await {
            let _ = tx.send(SseEvent::Error(e.to_string())).await;
        }
    });

    rx
}

async fn stream_cross_ring_query_inner(
    state: AppState,
    user: crate::models::user::UserRow,
    query: String,
    tx: &tokio::sync::mpsc::Sender<SseEvent>,
) -> Result<()> {
    let message_id = ulid::Ulid::new().to_string();
    let _ = tx
        .send(SseEvent::Start {
            message_id: message_id.clone(),
            role: "super_ring".to_string(),
        })
        .await;

    let ring_summary = build_ring_summary(&state.db, &user.token_id).await;
    
    let mut all_ring_details = String::new();
    let rings = sqlx::query_as::<_, (String, String)>(
        "SELECT r.id, r.name FROM rings r
         JOIN members m ON m.ring_id = r.id AND m.user_id = ?1
         ORDER BY r.created_at",
    )
    .bind(&user.token_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    for (ring_id, ring_name) in &rings {
        if let Ok(detail) = execute_query_ring_detail(&state.db, &state.rings_dir, &user.token_id, ring_name).await {
            all_ring_details.push_str(&format!("\n## Ring: {}\n{}", ring_name, detail));
        }
    }

    let system_prompt = format!(
        "你是 Super Ring，用户的全局 AI 助手。用户提出了一个跨 Ring 的查询问题。\n\n以下是用户的所有 Ring 的汇总信息：\n{}\n\n以下是每个 Ring 的详细数据：\n{}\n\n请基于以上信息，回答用户的问题。如果信息不足，请明确告知。",
        ring_summary, all_ring_details
    );

    let api_key = user
        .llm_api_key
        .as_deref()
        .ok_or_else(|| RingError::Internal("LLM API key not configured".into()))?;
    let mut config = OpenAIConfig::new().with_api_key(api_key);
    if let Some(base_url) = &user.llm_base_url {
        config = config.with_api_base(base_url);
    }
    let client = Client::with_config(config);
    let model = user.llm_model.clone();

    let request = CreateChatCompletionRequest {
        messages: vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text(system_prompt),
                    name: None,
                },
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(query),
                    name: None,
                },
            ),
        ],
        model,
        stream: Some(true),
        ..Default::default()
    };

    let mut stream = match client.chat().create_stream(request).await {
        Ok(s) => s,
        Err(e) => {
            let _ = tx.send(SseEvent::Error(e.to_string())).await;
            return Ok(());
        }
    };

    let mut full_content = String::new();
    let mut token_usage: Option<String> = None;

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
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
                if let Some(usage) = &chunk.usage {
                    token_usage = Some(serde_json::to_string(usage).unwrap_or_default());
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
            full_content: full_content.clone(),
            token_usage,
        })
        .await;

    let ai_msg_id = message_id;
    let _ = message::insert_message(
        &state.db,
        &message::NewMessage {
            id: &ai_msg_id,
            ring_id: Some(SUPER_RING_ID),
            user_id: &user.token_id,
            role: "super_ring",
            sender_name: "SUPER RING",
            content: &full_content,
            node_refs: &[],
            tag_refs: &[],
            token_usage: None,
        },
    )
    .await;

    Ok(())
}
```

- [ ] **Step 2: 在 `server/src/services/super_chat.rs` 中新增跨 Ring 分析函数**

在 `stream_cross_ring_query_inner` 之后添加：

```rust
#[derive(Debug, Deserialize)]
pub struct CrossRingAnalysisRequest {
    pub ring_names: Vec<String>,
    pub analysis_type: String, // "compare" | "merge" | "summary"
    pub question: Option<String>,
}

pub fn stream_cross_ring_analysis(
    state: AppState,
    user: crate::models::user::UserRow,
    request: CrossRingAnalysisRequest,
) -> tokio::sync::mpsc::Receiver<SseEvent> {
    let (tx, rx) = tokio::sync::mpsc::channel(32);

    tokio::spawn(async move {
        if let Err(e) = stream_cross_ring_analysis_inner(state, user, request, &tx).await {
            let _ = tx.send(SseEvent::Error(e.to_string())).await;
        }
    });

    rx
}

async fn stream_cross_ring_analysis_inner(
    state: AppState,
    user: crate::models::user::UserRow,
    request: CrossRingAnalysisRequest,
    tx: &tokio::sync::mpsc::Sender<SseEvent>,
) -> Result<()> {
    let message_id = ulid::Ulid::new().to_string();
    let _ = tx
        .send(SseEvent::Start {
            message_id: message_id.clone(),
            role: "super_ring".to_string(),
        })
        .await;

    let mut selected_ring_details = String::new();
    
    for ring_name in &request.ring_names {
        if let Ok(detail) = execute_query_ring_detail(&state.db, &state.rings_dir, &user.token_id, ring_name).await {
            selected_ring_details.push_str(&format!("\n## Ring: {}\n{}", ring_name, detail));
        }
    }

    let analysis_prompt = match request.analysis_type.as_str() {
        "compare" => {
            format!(
                "请对比以下 Ring 的差异和共同点：\n{}\n\n请从目标、成员、内容、进展等维度进行对比分析。",
                selected_ring_details
            )
        }
        "merge" => {
            format!(
                "请分析以下 Ring 的内容，找出可以整合或合并的部分：\n{}\n\n请提出具体的整合建议。",
                selected_ring_details
            )
        }
        "summary" => {
            format!(
                "请对以下 Ring 的内容进行汇总分析：\n{}\n\n请提供综合摘要和关键洞察。",
                selected_ring_details
            )
        }
        _ => {
            format!(
                "请分析以下 Ring 的内容：\n{}\n\n用户问题：{}\n\n请基于以上信息回答。",
                selected_ring_details,
                request.question.unwrap_or_default()
            )
        }
    };

    let api_key = user
        .llm_api_key
        .as_deref()
        .ok_or_else(|| RingError::Internal("LLM API key not configured".into()))?;
    let mut config = OpenAIConfig::new().with_api_key(api_key);
    if let Some(base_url) = &user.llm_base_url {
        config = config.with_api_base(base_url);
    }
    let client = Client::with_config(config);
    let model = user.llm_model.clone();

    let request = CreateChatCompletionRequest {
        messages: vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text(
                        "你是 Super Ring，用户的全局 AI 助手。你的任务是分析多个 Ring 的数据并提供洞察。".to_string()
                    ),
                    name: None,
                },
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(analysis_prompt),
                    name: None,
                },
            ),
        ],
        model,
        stream: Some(true),
        ..Default::default()
    };

    let mut stream = match client.chat().create_stream(request).await {
        Ok(s) => s,
        Err(e) => {
            let _ = tx.send(SseEvent::Error(e.to_string())).await;
            return Ok(());
        }
    };

    let mut full_content = String::new();
    let mut token_usage: Option<String> = None;

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
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
                if let Some(usage) = &chunk.usage {
                    token_usage = Some(serde_json::to_string(usage).unwrap_or_default());
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
            full_content: full_content.clone(),
            token_usage,
        })
        .await;

    let ai_msg_id = message_id;
    let _ = message::insert_message(
        &state.db,
        &message::NewMessage {
            id: &ai_msg_id,
            ring_id: Some(SUPER_RING_ID),
            user_id: &user.token_id,
            role: "super_ring",
            sender_name: "SUPER RING",
            content: &full_content,
            node_refs: &[],
            tag_refs: &[],
            token_usage: None,
        },
    )
    .await;

    Ok(())
}
```

---

## Task 2: 后端 - 新增路由处理函数

**Files:**
- Modify: `server/src/routes/super_chat.rs`

- [ ] **Step 1: 在 `server/src/routes/super_chat.rs` 中添加请求结构体和 handler**

在文件顶部添加新的请求结构体：

```rust
#[derive(Debug, Deserialize)]
pub struct CrossRingQueryRequest {
    pub query: String,
}

#[derive(Debug, Deserialize)]
pub struct CrossRingAnalysisRequest {
    pub ring_names: Vec<String>,
    pub analysis_type: String,
    pub question: Option<String>,
}
```

在文件末尾添加 handler 函数：

```rust
pub async fn cross_ring_query_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CrossRingQueryRequest>,
) -> Result<Sse<KeepAliveStream<BoxedSseStream>>> {
    let user_row = state.get_user_decrypted(&user.token_id).await?;
    let mut rx = super_chat::stream_cross_ring_query(state, user_row, body.query);

    let s: BoxedSseStream = Box::pin(stream! {
        while let Some(event) = rx.recv().await {
            match event {
                SseEvent::Start { message_id, role } => {
                    let data = serde_json::json!({"message_id": message_id, "role": role});
                    yield Ok(Event::default().event("message_start").data(data.to_string()));
                }
                SseEvent::Delta { content } => {
                    let data = serde_json::json!({ "content": content });
                    yield Ok(Event::default().event("delta").data(data.to_string()));
                }
                SseEvent::End { message_id, full_content: _, token_usage } => {
                    let usage_json = token_usage.as_deref().and_then(|u| serde_json::from_str::<serde_json::Value>(u).ok());
                    let data = serde_json::json!({
                        "message_id": message_id,
                        "usage": usage_json.unwrap_or(serde_json::json!({ "prompt_tokens": 0, "completion_tokens": 0 }))
                    });
                    yield Ok(Event::default().event("message_end").data(data.to_string()));
                }
                SseEvent::Error(msg) => {
                    let data = serde_json::json!({ "error": msg });
                    yield Ok(Event::default().event("error").data(data.to_string()));
                }
            }
        }
    });
    Ok(Sse::new(s).keep_alive(KeepAlive::default()))
}

pub async fn cross_ring_analysis_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CrossRingAnalysisRequest>,
) -> Result<Sse<KeepAliveStream<BoxedSseStream>>> {
    let user_row = state.get_user_decrypted(&user.token_id).await?;
    let request = super_chat::CrossRingAnalysisRequest {
        ring_names: body.ring_names,
        analysis_type: body.analysis_type,
        question: body.question,
    };
    let mut rx = super_chat::stream_cross_ring_analysis(state, user_row, request);

    let s: BoxedSseStream = Box::pin(stream! {
        while let Some(event) = rx.recv().await {
            match event {
                SseEvent::Start { message_id, role } => {
                    let data = serde_json::json!({"message_id": message_id, "role": role});
                    yield Ok(Event::default().event("message_start").data(data.to_string()));
                }
                SseEvent::Delta { content } => {
                    let data = serde_json::json!({ "content": content });
                    yield Ok(Event::default().event("delta").data(data.to_string()));
                }
                SseEvent::End { message_id, full_content: _, token_usage } => {
                    let usage_json = token_usage.as_deref().and_then(|u| serde_json::from_str::<serde_json::Value>(u).ok());
                    let data = serde_json::json!({
                        "message_id": message_id,
                        "usage": usage_json.unwrap_or(serde_json::json!({ "prompt_tokens": 0, "completion_tokens": 0 }))
                    });
                    yield Ok(Event::default().event("message_end").data(data.to_string()));
                }
                SseEvent::Error(msg) => {
                    let data = serde_json::json!({ "error": msg });
                    yield Ok(Event::default().event("error").data(data.to_string()));
                }
            }
        }
    });
    Ok(Sse::new(s).keep_alive(KeepAlive::default()))
}
```

---

## Task 3: 后端 - 注册新路由

**Files:**
- Modify: `server/src/routes/mod.rs`

- [ ] **Step 1: 在 `server/src/routes/mod.rs` 的 super_chat 路由部分添加新端点**

在现有的 super_chat 路由之后添加：

```rust
        .route("/super/cross-ring-query", post(super_chat::cross_ring_query_handler))
        .route("/super/cross-ring-analysis", post(super_chat::cross_ring_analysis_handler))
```

---

## Task 4: 前端 - 新增 API 函数

**Files:**
- Modify: `ui/src/services/api.ts`

- [ ] **Step 1: 在 `ui/src/services/api.ts` 中添加跨 Ring 查询函数**

在文件末尾（`exportSessionMessages` 之后）添加：

```typescript
export async function crossRingQuery(query: string): Promise<{ content: string }> {
  return api.post('/super/cross-ring-query', { query })
}

export async function crossRingAnalysis(
  ringNames: string[],
  analysisType: 'compare' | 'merge' | 'summary',
  question?: string
): Promise<{ content: string }> {
  return api.post('/super/cross-ring-analysis', { ring_names: ringNames, analysis_type: analysisType, question })
}
```

---

## Task 5: 前端 - 集成到聊天 Store

**Files:**
- Modify: `ui/src/stores/chat-store.ts`

- [ ] **Step 1: 导入新的 API 函数**

在文件顶部的 import 语句中添加：

```typescript
import { crossRingQuery, crossRingAnalysis } from '../services/api'
```

- [ ] **Step 2: 添加跨 Ring 查询命令处理**

在 `getCommandHelp` 函数中添加：

```typescript
    cross_ring_query: '### /cross-ring-query\n\nQuery across all your Rings.\n\n**Usage:** `/cross-ring-query <your question>`',
    cross_ring_analysis: '### /cross-ring-analysis\n\nAnalyze multiple Rings.\n\n**Usage:** `/cross-ring-analysis <compare|merge|summary> <ring1,ring2,...> [question]`',
```

在 `send` 函数的 `action` case 中添加新的命令处理：

```typescript
            else if (cmd.action === 'cross-ring-query') {
              const question = cmd.args.trim()
              if (question) {
                handleCrossRingQuery(question, addMessage, set, get)
              }
            }
            else if (cmd.action === 'cross-ring-analysis') {
              const parts = cmd.args.trim().split(/\s+/)
              const analysisType = parts[0] as 'compare' | 'merge' | 'summary'
              const ringNames = parts[1]?.split(',') || []
              const question = parts.slice(2).join(' ')
              if (analysisType && ringNames.length > 0) {
                handleCrossRingAnalysis(analysisType, ringNames, question, addMessage, set, get)
              }
            }
```

- [ ] **Step 3: 添加处理函数**

在文件末尾（`useChatStore` 定义之前）添加：

```typescript
async function handleCrossRingQuery(
  query: string,
  addMessage: (msg: ChatMessage) => void,
  set: any,
  get: any
) {
  addMessage({
    id: `msg-${Date.now()}`,
    role: 'user',
    sender_name: 'You',
    content: `/cross-ring-query ${query}`,
    created_at: new Date().toISOString(),
  })

  set({ sending: true })

  const controller = streamChat('/api/super/cross-ring-query', { query }, {
    onStart: (data) => {
      const aiMsg: ChatMessage = {
        id: data.message_id,
        role: 'super_ring',
        sender_name: 'SUPER RING',
        content: '',
        created_at: new Date().toISOString(),
      }
      set((s: any) => ({
        messages: [...s.messages, aiMsg],
        streaming_message_id: data.message_id,
      }))
    },
    onDelta: (data) => {
      const { streaming_message_id, messages } = get()
      if (!streaming_message_id) return
      set({
        messages: messages.map((m: ChatMessage) =>
          m.id === streaming_message_id ? { ...m, content: m.content + data.content } : m,
        ),
      })
    },
    onEnd: (data) => {
      const { streaming_message_id, messages } = get()
      if (streaming_message_id && data.usage) {
        set({
          messages: messages.map((m: ChatMessage) =>
            m.id === streaming_message_id ? { ...m, token_usage: data.usage } : m
          ),
        })
      }
      set({ sending: false, streaming_message_id: null, abort_controller: null })
    },
    onError: (data) => {
      const { streaming_message_id, messages } = get()
      if (streaming_message_id) {
        set({
          messages: messages.map((m: ChatMessage) =>
            m.id === streaming_message_id
              ? { ...m, content: m.content + `\n\n⚠ Error: ${data.error}` }
              : m,
          ),
          sending: false,
          streaming_message_id: null,
          abort_controller: null,
        })
      } else {
        addMessage({
          id: `err-${Date.now()}`,
          role: 'system',
          sender_name: 'SYSTEM',
          content: `Error: ${data.error}`,
          created_at: new Date().toISOString(),
        })
        set({ sending: false, abort_controller: null })
      }
    },
  })
  set({ abort_controller: controller })
}

async function handleCrossRingAnalysis(
  analysisType: 'compare' | 'merge' | 'summary',
  ringNames: string[],
  question: string,
  addMessage: (msg: ChatMessage) => void,
  set: any,
  get: any
) {
  addMessage({
    id: `msg-${Date.now()}`,
    role: 'user',
    sender_name: 'You',
    content: `/cross-ring-analysis ${analysisType} ${ringNames.join(',')}${question ? ' ' + question : ''}`,
    created_at: new Date().toISOString(),
  })

  set({ sending: true })

  const controller = streamChat('/api/super/cross-ring-analysis', { 
    ring_names: ringNames, 
    analysis_type: analysisType,
    question: question || undefined 
  }, {
    onStart: (data) => {
      const aiMsg: ChatMessage = {
        id: data.message_id,
        role: 'super_ring',
        sender_name: 'SUPER RING',
        content: '',
        created_at: new Date().toISOString(),
      }
      set((s: any) => ({
        messages: [...s.messages, aiMsg],
        streaming_message_id: data.message_id,
      }))
    },
    onDelta: (data) => {
      const { streaming_message_id, messages } = get()
      if (!streaming_message_id) return
      set({
        messages: messages.map((m: ChatMessage) =>
          m.id === streaming_message_id ? { ...m, content: m.content + data.content } : m,
        ),
      })
    },
    onEnd: (data) => {
      const { streaming_message_id, messages } = get()
      if (streaming_message_id && data.usage) {
        set({
          messages: messages.map((m: ChatMessage) =>
            m.id === streaming_message_id ? { ...m, token_usage: data.usage } : m
          ),
        })
      }
      set({ sending: false, streaming_message_id: null, abort_controller: null })
    },
    onError: (data) => {
      const { streaming_message_id, messages } = get()
      if (streaming_message_id) {
        set({
          messages: messages.map((m: ChatMessage) =>
            m.id === streaming_message_id
              ? { ...m, content: m.content + `\n\n⚠ Error: ${data.error}` }
              : m,
          ),
          sending: false,
          streaming_message_id: null,
          abort_controller: null,
        })
      } else {
        addMessage({
          id: `err-${Date.now()}`,
          role: 'system',
          sender_name: 'SYSTEM',
          content: `Error: ${data.error}`,
          created_at: new Date().toISOString(),
        })
        set({ sending: false, abort_controller: null })
      }
    },
  })
  set({ abort_controller: controller })
}
```

---

## Task 6: 测试

**Files:**
- Modify: `server/tests/integration.rs`

- [ ] **Step 1: 添加集成测试**

在 `server/tests/integration.rs` 中添加：

```rust
#[tokio::test]
async fn test_cross_ring_query_endpoint() {
    let state = setup_app().await;
    let app = build_router(state);

    // 先设置用户
    let setup_body = r#"{"display_name":"TestUser","avatar":"🧪","llm_provider":"openai","llm_api_key":"sk-test","llm_model":"gpt-4o"}"#;
    let resp = app
        .clone()
        .oneshot(make_request("POST", "/api/setup", Some(setup_body), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    let token = json["token"].as_str().unwrap();

    // 测试跨 Ring 查询端点
    let query_body = r#"{"query":"What are my rings about?"}"#;
    let resp = app
        .clone()
        .oneshot(make_request("POST", "/api/super/cross-ring-query", Some(query_body), Some(token)))
        .await
        .unwrap();
    
    // 应该返回 SSE 流
    assert_eq!(resp.status(), StatusCode::OK);
    let content_type = resp.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(content_type.contains("text/event-stream"));
}

#[tokio::test]
async fn test_cross_ring_analysis_endpoint() {
    let state = setup_app().await;
    let app = build_router(state);

    // 先设置用户
    let setup_body = r#"{"display_name":"TestUser","avatar":"🧪","llm_provider":"openai","llm_api_key":"sk-test","llm_model":"gpt-4o"}"#;
    let resp = app
        .clone()
        .oneshot(make_request("POST", "/api/setup", Some(setup_body), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    let token = json["token"].as_str().unwrap();

    // 测试跨 Ring 分析端点
    let analysis_body = r#"{"ring_names":["Test Ring"],"analysis_type":"summary"}"#;
    let resp = app
        .clone()
        .oneshot(make_request("POST", "/api/super/cross-ring-analysis", Some(analysis_body), Some(token)))
        .await
        .unwrap();
    
    // 应该返回 SSE 流
    assert_eq!(resp.status(), StatusCode::OK);
    let content_type = resp.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(content_type.contains("text/event-stream"));
}
```

---

## Task 7: 编译和测试

- [ ] **Step 1: 编译后端**

Run: `cd server && cargo build`
Expected: 编译成功，无错误

- [ ] **Step 2: 运行后端测试**

Run: `cd server && cargo test`
Expected: 所有测试通过

- [ ] **Step 3: 编译前端**

Run: `cd ui && npm run build`
Expected: 编译成功，无错误

---

## 自审检查

1. **Spec coverage:** 
   - ✅ Cross-Ring Query 端点 - Task 1, 2, 3
   - ✅ Cross-Ring Analysis 端点 - Task 1, 2, 3
   - ✅ 前端 API 函数 - Task 4
   - ✅ 前端聊天集成 - Task 5
   - ✅ 测试 - Task 6

2. **Placeholder scan:** 无 TBD/TODO/占位符

3. **类型一致性:** 
   - `CrossRingAnalysisRequest` 在 service 和 route 中定义一致
   - SSE 事件类型与现有代码一致
   - API 路径使用 snake_case 符合项目规范

## 执行交接

**计划完成。** 两个执行选项：

**1. Subagent-Driven (推荐)** - 每个任务分配一个子代理，任务间审查，快速迭代

**2. Inline Execution** - 在本会话中使用 executing-plans 执行任务，批量执行并设置检查点

**选择哪种方式？**