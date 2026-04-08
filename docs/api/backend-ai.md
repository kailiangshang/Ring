# AI 服务 API 参考

> 源码路径：`ring-server/src/services/ai_service.rs`、`context_loader.rs`

## AiService

### `struct AiService`
源文件：`services/ai_service.rs:15`

| 字段 | 类型 | 说明 |
|------|------|------|
| `db` | `Arc<dyn Repository>` | 数据库 |
| `llm` | `Arc<dyn LlmProvider>` | LLM |
| `tool_dispatcher` | `Arc<ToolDispatcher>` | 工具调度器 |

### `impl AiService`
源文件：`services/ai_service.rs:21`

- `fn new(db, llm, tool_dispatcher) -> Self` — 构造函数
- `async fn super_ring_chat(user_id, message, history) -> Result<SseStream>` — 全局超级助手对话，在 prompt 中注入用户 Ring 列表
- `async fn group_ring_chat(ring_id, conv_id, message) -> Result<SseStream>` — 群组助手对话，自动保存用户消息
- `async fn blueprint_chat(ring_id, message, history) -> Result<SseStream>` — 蓝图构建对话
- `async fn session_chat(ring_id, session_id, sender_id, ring_name, scenario, message) -> Result<SseStream>` — Session 协作对话
- `async fn chat_with_tools(messages, tools) -> Result<SseStream>` — 工具调用循环（最多 5 轮）

### Token 预算控制
所有对话方法都会：
1. 构建 system prompt（通过 `context_loader`）
2. 计算 `estimate_tokens(context)`
3. 从 history 中截取不超过 100,000 - system_tokens 的内容
4. `truncate_llm_messages` / `truncate_messages`：从后往前保留，总字符数不超过 budget_chars

---

## Context Loader

### `fn build_super_ring_prompt() -> String`
源文件：`services/context_loader.rs:1`

构建 Super Ring 系统提示词。包含：Ring 管理引导、跨 Ring 分析、跨 Ring 问答、新用户引导。严格禁止虚构 Ring 数据。

### `fn build_group_ring_prompt(ring_name, role_md, conventions_md, active_context_md) -> String`
源文件：`services/context_loader.rs:26`

构建 Group Ring 系统提示词。包含：角色定义、团队约定、当前活跃上下文。

### `fn build_blueprint_prompt(role_md) -> String`
源文件：`services/context_loader.rs:53`

构建蓝图构建器提示词。核心原则：图谱节点必须对应 `.ring/` 目录下的 Markdown 文档。交互流程：追问 → 确认维度 → 说明模板 → mermaid 预览 → 用户确认。

### `fn build_session_prompt(ring_name, scenario) -> String`
源文件：`services/context_loader.rs:95`

构建 Session 助手提示词。根据 scenario（discussion/deep_research/meeting_archive/learning_center）调整行为。

---

## 辅助函数

### `fn estimate_tokens(text: &str) -> usize`
源文件：`services/ai_service.rs:316`

粗略估算：`text.len() / 3`

### `fn truncate_messages(messages: &[Message], budget_tokens) -> Vec<Message>`
源文件：`services/ai_service.rs:322`

从后往前保留消息，总字符数不超过 `budget_tokens * 3`。

### `fn truncate_llm_messages(messages: &[LlmMessage], budget_tokens) -> Vec<LlmMessage>`
源文件：`services/ai_service.rs:340`

同 `truncate_messages`，但作用于 `LlmMessage`。
