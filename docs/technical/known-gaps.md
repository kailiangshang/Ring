# Known Gaps — 已实现但存在缺陷的修复清单

> **Affects**: [implementation-roadmap.md](implementation-roadmap.md) · [future-features.md](future-features.md)
> **Depends on**: [architecture.md](architecture.md) · [backend.md](../api/backend.md)
> **Last verified**: 2026-04-11

本文档记录**已有代码但未正确工作**的功能缺陷。每条含根因分析、修复方案和验收标准。

---

## P0 — 致命缺陷（不修则产品不可用）

### GAP-01: graph.json 无持久化生命周期

**现象**：服务重启后所有图谱数据丢失。`PetgraphStore` 在 `main.rs` 中以 `new()` 创建空图，没有加载已有 `graph.json` 的逻辑。节点变更后也不会写回文件。

**根因**：
- `main.rs:38` 只做 `PetgraphStore::new()`，不扫描 `~/.ring/repos/ring-*/graph.json`
- `GraphService` 的 create/update/delete 操作不触发持久化
- `export_graph_json` / `import_graph_json` 方法存在但无调用方

**修复方案**：
1. `main.rs` 启动时：遍历 `{data_dir}/repos/ring-*/graph.json`，对每个文件调用 `import_graph_json` 加载到内存图
2. `GraphService` 的每次写操作（create_node、update_node、delete_node、create_edge、delete_edge）成功后，调用 `export_graph_json` 写回对应的 `graph.json`
3. Ring 创建时（`ring_service.create_ring`）确保 `repos/ring-{name}/` 目录和空 `graph.json` 存在

**验收标准**：
- [ ] 创建节点 → 重启服务 → 节点仍存在
- [ ] 删除节点 → 重启服务 → 节点确实不存在
- [ ] 多个 Ring 的图谱互不干扰
- [ ] `graph.json` 文件内容与内存图一致

**涉及文件**：`main.rs`、`services/graph_service.rs`、`services/ring_service.rs`、`graph/petgraph_store.rs`

---

### GAP-02: 搜索索引不会自动填充

**现象**：FTS5 搜索功能已实现（jieba 分词 + MATCH 查询 + snippet 高亮），但 `GraphService::create_node()` 不调用 `SearchService::index_node()`。搜索结果永远为空。

**根因**：`graph_service.rs` 的 `create_node` 方法只操作 petgraph，不触发索引。

**修复方案**：
1. `GraphService` 持有 `Arc<SearchService>` 引用
2. `create_node` 成功后调用 `search_service.index_node(node_id, graph_id, label, content)`
3. `update_node` 成功后先 `delete_node_index` 再 `index_node`（重建索引）
4. `delete_node` 成功后调用 `search_service.delete_node_index(node_id)`

**验收标准**：
- [ ] 创建节点后立即可通过关键词搜索到
- [ ] 更新节点标签/内容后搜索结果反映最新值
- [ ] 删除节点后搜索不到

**涉及文件**：`services/graph_service.rs`、`state.rs`（注入依赖）

---

### GAP-03: 工具系统未接入（死代码）

**现象**：
- `ToolDispatcher`、5 个系统工具（Search、TextClean、WebScrape、PrivacyFilter、MarkdownGen）、`chat_with_tools()` 全部已实现但从未被调用
- `ai_service.rs:70` 和 `ai_service.rs:121` 的 `group_ring_chat` / `super_ring_chat` 调用 `self.llm.chat_stream(messages, None)` 传 `None` 给 tools
- 前端 `Toolbar` 组件的 toggle 只改本地 `useState`，不传给 API

**修复方案**：
1. 后端：`group_ring_chat` 和 `super_ring_chat` 改为调用 `chat_with_tools`，传入 `ToolRegistry` 中已注册的工具
2. 后端：`POST /api/v1/rings/{ringId}/conversations/{convId}/messages` 接受可选参数 `active_tools: Option<Vec<String>>`
3. 前端：`chatStore.sendMessage()` 从 toolbar 状态读取 active tools，拼入请求 body
4. 前端：已有 `ToolCallBubble` / `ToolResultBubble` 组件，SSE 事件 `tool_call` / `tool_result` 已在 `chatStore` 中处理，无需额外 UI 工作

**验收标准**：
- [ ] 前端开启 Search 工具 → 发消息 → LLM 调用 SearchTool → 返回带 tool_call/tool_result 事件的流式响应
- [ ] 前端关闭所有工具 → 发消息 → LLM 正常回复不调工具
- [ ] 工具调用最多 5 轮（现有 `chat_with_tools` 的 max_rounds 限制生效）

**涉及文件**：`services/ai_service.rs`、`handlers/conversation.rs`、`stores/chatStore.ts`、`components/toolbar/Toolbar.tsx`

---

### GAP-04: 节点无 markdown 文档（知识闭环断裂）

**现象**：
- `NodeData.markdown_path` 始终为 `None`
- `GraphService::get_node_content()` 始终返回 `None`
- 节点只有 label/type/categories，没有实际内容

**根因**：节点创建流程不包含文件写入步骤。

**修复方案**：
1. `GraphService::create_node` 成功后，在 `{ring_local_path}/nodes/` 目录下创建 `{node_id}.md`
2. 文件格式：YAML frontmatter（node_id、type、labels、created_at）+ body（初始为空或从参数填入 content）
3. `NodeData.markdown_path` 设为 `Some(format!("nodes/{node_id}.md"))`
4. `get_node_content` 读取 markdown 文件内容返回
5. `update_node` 同步更新 md 文件
6. `delete_node` 同时删除 md 文件

**验收标准**：
- [ ] 创建节点 → `nodes/{node_id}.md` 文件存在且 frontmatter 正确
- [ ] `get_node_content` 返回文件内容
- [ ] 更新节点 → md 文件同步更新
- [ ] 删除节点 → md 文件同步删除

**涉及文件**：`services/graph_service.rs`、`graph/types.rs`

---

### GAP-05: 命名层级不一致

**现象**：代码和 UI 中 "Group Ring"、"Ring" 混用，缺乏统一层级。

**目标层级**：

| 层级 | 名称 | 代码标识 | 说明 |
|------|------|----------|------|
| 整体平台 | Ring Hub | `ring_hub` | 所有 Ring Group 的入口 |
| 跨 Ring AI | Ring Super | `ring_super` | 跨 Ring 的 meta AI 助手 |
| 群组空间 | Ring Group | `ring_group` | 一个知识协作空间（当前 "Ring"） |
| 协作会话 | Ring Session | `ring_session` | Ring Group 内的协作会话 |

**修复方案**：
1. 全局替换：`Group Ring` → `Ring Group`、代码中 `group_ring` 保持不变（已是 snake_case 正确形式）
2. UI 文案：所有用户可见文本遵循层级表
3. 组件重命名：`RingHub` → 确认为 `RingHub`（整体平台入口，名正确），`SuperRingChat` → `RingSuperChat`
4. 路由标签：`/super-ring` → `/ring-super`，`/rings/{id}/sessions` 保持

**验收标准**：
- [ ] `rg "Group Ring" --type ts --type rust` 无结果
- [ ] 所有页面标题和导航文本符合层级表
- [ ] 组件名 PascalCase 遵循 `RingHub`/`RingSuper`/`RingGroup`/`RingSession` 前缀

**涉及文件**：全局搜索替换，主要在 `ring-frontend/src/pages/`、`ring-frontend/src/components/`

---

## P1 — 重要缺陷（核心体验受损）

### GAP-06: .ring/ 模板目录不初始化

**现象**：创建 Ring 时只创建数据库记录，不创建文件系统目录和初始模板文件。

**修复方案**：
1. `ring_service.create_ring` 成功后创建目录结构：
   ```
   {data_dir}/repos/ring-{name}/
   ├── graph.json        # 空图谱 {"nodes":[], "edges":[]}
   └── nodes/            # 空，等待节点创建
   ```
2. 确保目录权限正确

**验收标准**：
- [ ] 创建 Ring → 目录存在、graph.json 可解析
- [ ] 删除 Ring → 目录保留（数据安全），或按用户选择清理

**涉及文件**：`services/ring_service.rs`

---

### GAP-07: Archive git merge 未实现

**现象**：`archive_service.merge_pr()` 只更新数据库状态为 "merged"，不实际 merge git 分支，不调用 GitLab merge API。

**修复方案**：
1. 本地 merge：调用 `git_service` 执行 branch merge（checkout main → merge branch）
2. GitLab merge：调用 `gitlab_service.merge_mr()` 执行远程 MR merge
3. 两者都成功后更新数据库状态

**验收标准**：
- [ ] Creator confirm archive → git 分支合入 main → 远程 MR 状态变为 merged
- [ ] merge 失败 → 状态不变，返回错误信息

**涉及文件**：`services/archive_service.rs`、`services/git_service.rs`、`services/gitlab_service.rs`

---

### GAP-08: SSE stream 切换会话时无 abort

**现象**：用户在对话 A 流式输出过程中切换到对话 B，对话 A 的 stream 仍在后台运行，可能覆盖 chatStore 状态。

**修复方案**：
1. `chatStore` 维护 `AbortController` 引用
2. `sendMessage` 开始前 abort 上一个 controller
3. 切换 conversation 时 abort 当前 stream
4. 后端：利用 Axum 的 `axum::body::BodyDataStream` 自然断开（客户端断开时 drop sender）

**验收标准**：
- [ ] 流式输出中切换会话 → 旧 stream 停止 → 无状态残留
- [ ] 连续快速切换多次 → 不崩溃、不错乱

**涉及文件**：`stores/chatStore.ts`、`stores/sessionChatStore.ts`

---

## 验收总览

| ID | 优先级 | 简述 | 涉及层 |
|----|--------|------|--------|
| GAP-01 | P0 | graph.json 持久化生命周期 | 后端 |
| GAP-02 | P0 | 搜索索引自动填充 | 后端 |
| GAP-03 | P0 | 工具系统接入 | 全栈 |
| GAP-04 | P0 | 节点 markdown 文档闭环 | 后端 |
| GAP-05 | P0 | 命名层级统一 | 前端 |
| GAP-06 | P1 | .ring/ 目录初始化 | 后端 |
| GAP-07 | P1 | Archive git merge 实现 | 后端 |
| GAP-08 | P1 | SSE stream abort | 前端 |
