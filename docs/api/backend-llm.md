# LLM Provider API 参考

> 源码路径：`ring-server/src/services/llm_provider.rs`、`llm_openai.rs`、`llm_anthropic.rs`

## LlmProvider Trait

### `trait LlmProvider`
源文件：`services/llm_provider.rs:53`

| 方法 | 签名 | 说明 |
|------|------|------|
| `chat_stream` | `(messages, tools) -> Result<Pin<Box<dyn Stream<Item=LlmEvent> + Send>>>` | 发起流式对话 |

---

## 数据结构

### `LlmMessage`
源文件：`services/llm_provider.rs:10`

| 字段 | 类型 | 说明 |
|------|------|------|
| `role` | `String` | 角色（system/user/assistant/tool） |
| `content` | `String` | 内容 |

### `TokenUsage`
源文件：`services/llm_provider.rs:16`

| 字段 | 类型 | 说明 |
|------|------|------|
| `prompt_tokens` | `u32` | 提示 Token 数 |
| `completion_tokens` | `u32` | 完成 Token 数 |
| `total_tokens` | `u32` | 总 Token 数 |

### `LlmEvent` 枚举
源文件：`services/llm_provider.rs:24`

流式事件类型，`serde` tag 格式：

- `Text { content: String }` — 文本片段
- `ToolCall { tool_call_id, tool, input }` — 工具调用请求
- `ToolResult { tool_call_id, tool, output }` — 工具执行结果
- `ArchiveSuggestion { data }` — 归档建议
- `BlueprintProposal { data }` — 蓝图提案
- `Error { code, message }` — 错误
- `Done { message_id, token_usage }` — 结束信号

---

## OpenAiProvider

### `struct OpenAiProvider`
源文件：`services/llm_openai.rs:22`

支持 OpenAI API 和 Ollama（通过自定义 base_url）。

| 字段 | 类型 | 说明 |
|------|------|------|
| `api_key` | `String` | API Key |
| `model` | `String` | 模型名称（如 gpt-4o、llama3） |
| `base_url` | `Option<String>` | 自定义 Base URL（Ollama 用） |

### `impl OpenAiProvider`
源文件：`services/llm_openai.rs:28`

- `fn new(api_key, model, base_url) -> Self` — 构造函数
- `fn build_client() -> OpenAIClient<OpenAIConfig>` — 构建客户端
- `async fn chat_stream(messages, tools) -> Result<...>` — 流式对话，通过 `async-openai` 调用 OpenAI/Ollama API

### `convert_messages(messages: &[LlmMessage]) -> Vec<ChatCompletionRequestMessage>`
源文件：`services/llm_openai.rs:46`

将 `LlmMessage` 转换为 OpenAI 格式。system → SystemMessage，assistant → AssistantMessage，tool → ToolMessage，user/其他 → UserMessage。

---

## AnthropicProvider

### `struct AnthropicProvider`
源文件：`services/llm_anthropic.rs:12`

支持 Anthropic Claude API。

| 字段 | 类型 | 说明 |
|------|------|------|
| `api_key` | `String` | API Key |
| `model` | `String` | 模型名称（如 claude-3-5-sonnet） |
| `base_url` | `Option<String>` | 自定义 Base URL |

### `impl AnthropicProvider`
源文件：`services/llm_anthropic.rs:18`

- `fn new(api_key, model, base_url) -> Self` — 构造函数
- `fn base_url() -> &str` — 返回 base URL，默认 `https://api.anthropic.com`
- `async fn chat_stream(messages, tools) -> Result<...>` — 流式对话，通过 `reqwest` 发送 SSE 请求

### `convert_messages(messages: &[LlmMessage]) -> (Option<String>, Vec<Value>)`
源文件：`services/llm_anthropic.rs:34`

将 `LlmMessage` 转换为 Anthropic 格式。多个 system 消息合并为一个 system，role 保持不变。

### `parse_sse_event(data: &str) -> Option<AnthropicStreamEvent>`
源文件：`services/llm_anthropic.rs:74`

解析 Anthropic SSE 事件。处理 `content_block_delta`（text_delta/input_json_delta）、`content_block_start`（tool_use）、`content_block_stop`、`message_stop`、`error` 类型。

---

## MockLlmProvider

### `struct MockLlmProvider`
源文件：`services/llm_provider.rs:62`

用于测试的模拟 LLM Provider。

| 字段 | 类型 | 说明 |
|------|------|------|
| `events` | `Vec<LlmEvent>` | 预设事件列表 |

- `fn new(events: Vec<LlmEvent>) -> Self` — 构造函数
- `async fn chat_stream(...) -> Result<...>` — 返回预设事件的流
