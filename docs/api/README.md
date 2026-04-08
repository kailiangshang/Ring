# Ring API 参考文档

> 面向内网的群组知识协作空间 — 全栈 API 索引手册

## 文档索引

### 后端（Rust + Axum）

| 文档 | 覆盖范围 |
|------|---------|
| [backend-core.md](backend-core.md) | Config、Error、State、Routes（所有 API 路由表） |
| [backend-models.md](backend-models.md) | 所有数据结构体（User/Ring/Member/Session/Conversation/Graph/Git/Notification/Tool） |
| [backend-db.md](backend-db.md) | Repository trait + SqliteRepository 所有方法 |
| [backend-services.md](backend-services.md) | 业务逻辑层：Ring/Member/Session/Search/Archive/Graph/Settings/Notification/Permission/Credential/Workflow/Trigger/WsHub |
| [backend-ai.md](backend-ai.md) | AiService + ContextLoader（Super Ring / Group Ring / Blueprint / Session 对话） |
| [backend-graph.md](backend-graph.md) | GraphStore trait + PetgraphStore + 类型定义 |
| [backend-llm.md](backend-llm.md) | LlmProvider trait + OpenAiProvider + AnthropicProvider + MockLlmProvider |
| [backend-tool-engine.md](backend-tool-engine.md) | Tool trait + ToolRegistry + ToolDispatcher + 5 工具实现 |
| [backend-git.md](backend-git.md) | GitService + GitlabService |
| [backend-handlers.md](backend-handlers.md) | 所有 Handler 函数签名和路由 |
| [backend-middleware.md](backend-middleware.md) | Auth 中间件（X-User-Id header） |

### 前端（React + TypeScript + Zustand）

| 文档 | 覆盖范围 |
|------|---------|
| [frontend-types.md](frontend-types.md) | 所有 TypeScript 接口定义 |
| [frontend-api-client.md](frontend-api-client.md) | API client 所有函数（含 SSE 请求说明） |
| [frontend-stores.md](frontend-stores.md) | Zustand store 状态和 actions |
| [frontend-pages-components.md](frontend-pages-components.md) | 页面路由 + 组件 props |

---

## 快速定位

### 想了解某个 API 路由
→ [backend-handlers.md](backend-handlers.md) 按路由表查找 handler → handler 内调用 service

### 想了解某个数据结构
→ [backend-models.md](backend-models.md) 查找结构体定义

### 想了解某个 service 的业务逻辑
→ [backend-services.md](backend-services.md) 查找对应 service

### 想了解某个 Handler 的请求/响应格式
→ [backend-handlers.md](backend-handlers.md) handler 名称 → [backend-models.md](backend-models.md) 对应请求/响应结构

### 想了解前端如何调用某个 API
→ [frontend-api-client.md](frontend-api-client.md) 查找同名函数

### 想了解前端状态管理
→ [frontend-stores.md](frontend-stores.md) 查找 store 和 actions

### 想了解 LLM 层实现
→ [backend-llm.md](backend-llm.md) + [backend-ai.md](backend-ai.md)

### 想了解图谱存储
→ [backend-graph.md](backend-graph.md) + [backend-services.md](backend-graph.md) GraphService

---

## 核心架构

```
请求 → middleware/auth (X-User-Id) → handler (参数解析)
       → service (业务逻辑) → db/repository (数据持久化)
                            → graph_store (内存图)
                            → llm_provider (AI)
                            → tool_engine (工具调用)
```

## 错误处理

所有错误通过 `crate::error::RingError` 处理，映射到 HTTP 状态码：

| 错误类型 | HTTP 状态码 |
|---------|------------|
| `NotFound` | 404 |
| `Unauthorized` | 401 |
| `Forbidden` | 403 |
| `Conflict` | 409 |
| `Validation` | 400 |
| 其他（Internal/Database/Llm/Io/Serialization） | 500（对外隐藏详情） |

## 认证方式

除 `/setup/*`、`/join`、`/ws/{ringId}` 外，所有 API 均需携带 `X-User-Id` 请求头。

前端 API client 自动从 `localStorage.getItem('ring_user_id')` 读取并附加该 header。
