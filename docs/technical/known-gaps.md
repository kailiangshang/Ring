# Known Gaps — 已实现但存在缺陷的修复清单

> **Affects**: [implementation-roadmap.md](implementation-roadmap.md) · [future-features.md](future-features.md)
> **Depends on**: [architecture.md](architecture.md) · [backend.md](../api/backend.md)
> **Last verified**: 2026-04-12

本文档记录**已有代码但未正确工作**的功能缺陷。每条含根因分析、修复方案和验收标准。

---

## P0 — 致命缺陷（不修则产品不可用）

### GAP-01: graph.json 无持久化生命周期 — ✅ 已修复

**状态**：已修复（2026-04-12）。

启动加载、node CRUD 持久化、Ring 创建时初始化均已实现。edge CRUD 原先绕过 `GraphService` 直接调用 `graph_store`，已改为走 `GraphService`，edge 变更现在也会触发 `persist_graph()`。

**涉及文件**：`main.rs`、`services/graph_service.rs`、`handlers/graph.rs`、`services/ring_service.rs`

---

### GAP-02: 搜索索引不会自动填充 — ✅ 已修复

**状态**：已修复（启动加载和 node CRUD 时已自动重建）。

`GraphService` 持有 `Arc<SearchService>`，`create_node`/`update_node`/`delete_node` 均已接入搜索索引。jieba 分词 + FTS5 完整工作。

**涉及文件**：`services/graph_service.rs`、`services/search_service.rs`

---

### GAP-03: 工具系统未接入（死代码） — ✅ 已修复

**状态**：已修复（2026-04-12）。

后端 `ToolDispatcher` 已注册 5 个工具，`AiService` 的 `super_ring_chat`/`group_ring_chat` 使用 `definitions_filtered(active_tools)` 调用 `chat_with_tools`。后端 request struct 支持 `active_tools: Option<Vec<String>>`。前端 Toolbar 状态通过 `chatStore` → `api/client.ts` 传递到后端。

**涉及文件**：`services/ai_service.rs`、`handlers/conversation.rs`、`handlers/ai.rs`、`services/tool_engine/dispatcher.rs`、前端 `chatStore.ts`/`ChatView.tsx`/`client.ts`

---

### GAP-04: 节点无 markdown 文档（知识闭环断裂） — ✅ 已修复

**状态**：已修复（启动加载时已自动完成）。

`GraphService::create_node` 创建 `{node_id}.md` 文件（YAML frontmatter + description body），`update_node` 同步更新，`delete_node` 级联删除。`get_node_content` 正确读取文件。

**涉及文件**：`services/graph_service.rs`、`graph/types.rs`

---

### GAP-05: 命名层级不一致 — ✅ 已修复

**状态**：已修复（2026-04-12）。

后端 prompt：`Super Ring` → `Ring Super`、`Group Ring` → `Ring Group`。路由 `/super-ring` → `/ring-super`。前端组件 `SuperRingChat` → `RingSuperChat`、store `superRingStore` → `ringSuperStore`、CSS classes `.super-ring-*` → `.ring-super-*`、mock data `group_ring` → `ring_group`。

**涉及文件**：`services/context_loader.rs`、`routes.rs`、前端约 10 个文件

---

## P1 — 重要缺陷（核心体验受损）

### GAP-06: .ring/ 模板目录不初始化 — ✅ 已修复

**状态**：已修复（2026-04-12）。

`create_ring` 现在创建 `repos/ring-{name}/.ring/` 目录并写入 6 个模板文件（role.md, conventions.md, active-context.md, archive-patterns.md, corrections.md, knowledge-summary.md）。`AiService` 从 `.ring/` 读取 role.md 和 conventions.md 构建系统提示。

**涉及文件**：`services/ring_service.rs`、`services/ai_service.rs`

---

### GAP-07: Archive git merge 未实现 — ✅ 已修复

**状态**：已修复（2026-04-12）。

`merge_pr()` 现在执行实际的本地 git merge：解析 `pr_url` 中的 `branch://` 前缀获取分支名，checkout main，merge 分支（含冲突检测），然后更新 DB 状态。`GitService` 新增 `checkout()` 和 `merge_branch()` 方法。Handler 现在通过 `ring.creator_id` 正确判断 `is_creator`。

**涉及文件**：`services/archive_service.rs`、`services/git_service.rs`、`handlers/archive.rs`

---

### GAP-08: SSE stream 切换会话时无 abort — ✅ 已修复

**状态**：已修复（2026-04-12）。

`chatStore` 和 `sessionChatStore` 现在使用 `AbortController`。每次发送新消息或切换对话/会话时，旧的 stream 被 abort。API 函数接受 `signal` 参数传递给 `fetch()`。

**涉及文件**：`stores/chatStore.ts`、`stores/sessionChatStore.ts`、`api/client.ts`

---

## 验收总览

| ID | 优先级 | 简述 | 涉及层 | 状态 |
|----|--------|------|--------|------|
| GAP-01 | P0 | graph.json 持久化生命周期 | 后端 | ✅ 已修复 |
| GAP-02 | P0 | 搜索索引自动填充 | 后端 | ✅ 已修复 |
| GAP-03 | P0 | 工具系统接入 | 全栈 | ✅ 已修复 |
| GAP-04 | P0 | 节点 markdown 文档闭环 | 后端 | ✅ 已修复 |
| GAP-05 | P0 | 命名层级统一 | 前端 | ✅ 已修复 |
| GAP-06 | P1 | .ring/ 目录初始化 + AI 读取 | 后端 | ✅ 已修复 |
| GAP-07 | P1 | Archive git merge 实现 | 后端 | ✅ 已修复 |
| GAP-08 | P1 | SSE stream abort | 前端 | ✅ 已修复 |
