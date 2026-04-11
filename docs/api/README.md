# Ring API 参考文档

> **Affects**: All api reference docs
> **Depends on**: [PRD.md](../product/PRD.md) · [api-design.md](../technical/api-design.md)
> **Last verified**: 2026-04-11

> 面向内网的群组知识协作空间 — 全栈 API 索引手册

## 文档索引

### 后端（Rust + Axum）

| 文档 | 覆盖范围 |
|------|---------|
| [backend.md](backend.md) | 合并文档：Config、Error、State、Routes、所有数据结构体、Repository trait、所有 Service、AiService、GraphStore、LlmProvider、ToolEngine、GitService、所有 Handler、Auth 中间件 |

### 前端（React + TypeScript + Zustand）

| 文档 | 覆盖范围 |
|------|---------|
| [frontend.md](frontend.md) | 合并文档：所有 TypeScript 接口、API client 所有函数（含 SSE）、Zustand store 状态和 actions、页面路由 + 组件 props |

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
→ [backend.md#图谱引擎](backend.md#图谱引擎) + [backend.md#服务层](backend.md#服务层) GraphService

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
