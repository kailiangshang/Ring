# Future Features — 新功能规划

本文档记录**尚未实现的新功能**。每条含用户故事、范围界定、验收标准和前置依赖。

已知缺陷修复见 [known-gaps.md](./known-gaps.md)。

---

## P2 — 核心功能增强

### FEAT-01: 结构化日志

**用户故事**：作为开发者/运维，我需要通过结构化日志快速定位问题，而不是翻裸 print。

**范围**：
- 引入 `tracing` + `tracing-subscriber`
- 按模块设置日志级别：`handlers=info, services=debug, db=warn, graph=debug`
- 日志输出到 stdout + 文件（`{data_dir}/logs/`）
- 日志轮转：按天分割，保留 7 天

**不在范围内**：日志聚合平台、ELK 集成、远程日志上传。

**验收标准**：
- [ ] 每个请求有 trace_id，贯穿 handler → service → repo
- [ ] `RUST_LOG=ring_server::services=debug` 只显示 services 层日志
- [ ] 日志文件自动轮转，超过 7 天的自动清理
- [ ] 错误日志包含完整调用链（service 方法名 + 错误类型 + 原因）

**前置依赖**：无

**涉及**：全局替换 `println!` / `eprintln!` 为 `tracing::info!` / `tracing::error!`

---

### FEAT-02: 工具调用对话内展示

**用户故事**：作为用户，我需要在聊天界面中看到 AI 调用了什么工具、传了什么参数、返回了什么结果，而不是只看到最终文本。

**范围**：
- 工具调用卡片：显示工具名称、状态（running / success / error）、耗时
- 可展开/折叠的参数和返回值面板
- 状态动画：running 时显示 spinner，完成后显示 checkmark 或 error icon
- 按时间顺序渲染：thinking → tool_call → tool_result → text

**不在范围内**：独立 Monitor 页面（见 FEAT-06）、工具编辑 UI。

**验收标准**：
- [ ] AI 调用 SearchTool 时，聊天中显示 "🔍 SearchTool" 卡片，状态从 running → success
- [ ] 点击卡片展开参数（搜索关键词）和返回值（搜索结果摘要）
- [ ] 多轮工具调用按序显示，不跳乱
- [ ] 流式输出中 tool_call 事件到达后立即显示卡片，不等 tool_result

**前置依赖**：GAP-03（工具系统接入）

**涉及**：`components/chat/ToolCallBubble.tsx`、`components/chat/ToolResultBubble.tsx`、`stores/chatStore.ts`

---

### FEAT-03: Ring Super 跨 Ring 按需查询

**用户故事**：作为管理多个 Ring Group 的用户，我需要 Ring Super 能回答"哪个 Ring 最近讨论了 X？"这类跨 Ring 问题。

**设计原则**：参数化按需查询，永远不一次加载所有 Ring 数据。

**支持的查询类型**：

| 查询类型 | 触发方式 | 返回内容 |
|----------|----------|----------|
| `ring_summary` | LLM tool call | 指定 Ring 的节点数、边数、根节点列表、最后更新时间 |
| `ring_nodes` | LLM tool call | 指定 Ring 的节点列表（可按 type/日期/关键词过滤） |
| `ring_recent` | LLM tool call | 指定 Ring 的最近对话摘要和归档文档（分页） |
| `ring_search` | LLM tool call | 对指定 Ring 执行 FTS5 搜索 |

**实现方式**：
1. 将上述 4 种查询注册为 Ring Super 的专属工具（`RingSummaryTool`、`RingNodesTool`、`RingRecentTool`、`RingSearchTool`）
2. Ring Super 的 `chat_with_tools` 传入这些工具
3. LLM 自行决定调哪些工具、传哪些参数
4. 结果注入上下文，LLM 汇总回答

**不在范围内**：`ring_graph_snapshot`（完整图结构导出，按需加）、缓存 TTL（首轮不做）。

**验收标准**：
- [ ] 用户问"哪个 Ring 讨论了微服务？" → Ring Super 调用 `ring_search` 查各 Ring → 返回命中的 Ring 列表和摘要
- [ ] 用户问"Ring-Backend 最近在做什么？" → Ring Super 调用 `ring_recent` → 返回最近对话和归档
- [ ] 单次查询不加载超过 1 个 Ring 的完整数据

**前置依赖**：GAP-01（graph 持久化）、GAP-02（搜索索引）、GAP-03（工具系统接入）

**涉及**：`services/tool_engine/tools/`（新增 4 个工具）、`services/ai_service.rs`、`handlers/conversation.rs`

---

### FEAT-04: Blueprint 结构化确认流程

**用户故事**：作为用户，我在确认 Blueprint 之前需要清楚看到将要创建什么，而不是盲目点按钮。

**范围**：

1. **确认前预览页**：
   - Mermaid 图渲染：将蓝图节点和关系渲染为可视化图
   - 节点类型列表：每种类型的名称、数量、是否有文档模板
   - 统计摘要：总节点数、总关系数
   - 警告：缺少文档模板的节点类型高亮标红

2. **确认交互**：
   - 勾选 "我已确认以上内容" checkbox
   - "创建图谱" 按钮在 checkbox 勾选前 disabled
   - 可点击单个维度进行局部编辑（不重开整个向导）

3. **创建后反馈**：
   - 创建进度条：显示 "正在创建节点 3/12..."
   - 创建完成后自动跳转到 GraphView

**不在范围内**：蓝图模板保存/加载、多人协作编辑蓝图。

**验收标准**：
- [ ] Preview 页面显示 Mermaid 图，节点和边数量与蓝图定义一致
- [ ] 未勾选确认 checkbox 时 "创建图谱" 按钮灰色不可点
- [ ] 创建过程中显示进度
- [ ] 创建完成后 GraphView 中的节点与蓝图一致

**前置依赖**：GAP-04（节点创建时写 markdown）

**涉及**：`pages/RingSpace/BlueprintWizard.tsx`、`stores/blueprintStore.ts`、`handlers/blueprint.rs`（新增 preview 端点）

---

### FEAT-05: Session Archive → Knowledge Graph 完整 Pipeline

**用户故事**：作为用户，我希望对话结束后能自动提取关键知识写入知识图谱，而不是手动整理。

**完整 pipeline**：

```
Session 对话
  → 归档为 markdown 文件（已有）
  → LLM 提取节点和关系（新增）
  → 写入 petgraph（新增）
  → 去重合并（新增）
```

**范围**：

1. **归档触发**（已有框架）：
   - 手动：用户点击归档按钮
   - 自动：Session 关闭时如果 `archive_enabled = true`

2. **LLM 知识提取**（新增）：
   - 将 session 全部消息作为输入
   - Prompt 要求 LLM 输出结构化 JSON：`{ nodes: [...], edges: [...] }`
   - 每个节点包含：label、type、category、summary
   - 每条边包含：source_label、target_label、relation_type

3. **写入图谱**（新增）：
   - 调用 `graph_service.create_node` / `create_edge` 将提取结果写入 petgraph
   - 同时创建 markdown 文档（GAP-04 修复后自动完成）

4. **去重合并**（新增）：
   - 对比提取的节点 label 与已有节点
   - 完全匹配 → 跳过创建，更新内容
   - 相似但不同 → 创建新节点并标注来源 session

5. **归档历史**（已有框架）：
   - `archive_records` 表记录哪些 session 已归档
   - 支持查看归档详情（提取了哪些节点/边）

**不在范围内**：归档结果人工审核、手动修正提取结果、增量提取（只提取新消息）。

**验收标准**：
- [ ] Session 关闭 → 自动归档 → 在图谱中可以看到新节点和边
- [ ] 提取的节点有 label、type、category、summary，不是空壳
- [ ] 重复概念不会创建重复节点
- [ ] 归档记录可查

**前置依赖**：GAP-01（graph 持久化）、GAP-04（节点 markdown 文档）、GAP-07（archive git merge）

**涉及**：`services/archive_service.rs`、`services/ai_service.rs`（新增 extraction prompt）、`services/graph_service.rs`

---

## P3 — 体验打磨

### FEAT-06: 独立 Monitor 页面

**用户故事**：作为 Ring Group 管理者，我需要监控 AI 服务的健康状态和资源消耗。

**范围**：
- 路由：`/rings/{ringId}/monitor`
- 请求时间线：每条消息的 TTFT（首 token 延迟）和总延迟
- Token 用量：按会话 / 按天的消耗统计图
- 工具调用日志：可搜索的历史记录（工具名、参数摘要、状态、耗时）
- LLM 错误率：429 / 500 错误趋势
- 后端健康：连接状态指示（reachable / unreachable）

**不在范围内**：告警通知、多 Ring 聚合监控、导出报表。

**验收标准**：
- [ ] 发送 5 条消息后，Monitor 页面显示 5 条请求的延迟数据
- [ ] Token 用量按天聚合显示
- [ ] 工具调用记录可按工具名和状态筛选
- [ ] 后端断开时健康指示变为红色

**前置依赖**：FEAT-01（结构化日志，提供数据源）、GAP-03（工具系统接入）

**涉及**：新增 `pages/RingSpace/MonitorView.tsx`、新增 `stores/monitorStore.ts`、后端新增监控数据采集中间件

---

### FEAT-07: 流式输出体验优化

**用户故事**：作为用户，我希望 AI 回复的渲染体验接近主流 AI 聊天产品。

**范围**：
- Markdown 流式解析：不等 token 累积，边收边渲染（处理不完整代码块、表格）
- 代码块语法高亮：流式输出中即高亮，不等完成
- Thinking/reasoning 块：折叠显示，点击展开
- 光标指示器：流式输出中在末尾显示闪烁光标
- Code block 复制按钮（所有聊天视图统一）

**不在范围内**：自定义主题、LaTeX 渲染。

**验收标准**：
- [ ] 流式输出中代码块实时语法高亮，不闪烁
- [ ] 不完整 Markdown（如只有 ` ``` ` 没闭合）不导致渲染错乱
- [ ] 点击代码块复制按钮 → 剪贴板内容正确 → 按钮变为 checkmark

**前置依赖**：无

**涉及**：`components/chat/ChatBubble.tsx`、引入 `react-markdown` 流式插件或自定义 renderer

---

### FEAT-08: UI 布局与视觉统一

**用户故事**：作为用户，我希望产品视觉上不粗糙，至少达到内部工具的体面水平。

**范围**：
- Logo 显示：导航栏 brand 区域显示 logo 图片 + 文字
- CSS 变量化：所有硬编码颜色提取为 CSS custom properties
- 暗色模式：基于 CSS 变量切换，所有页面适配
- 空状态：图谱为空、无对话、无成员时显示引导插图和操作提示
- 响应式：最小宽度 1024px 可用，不要求移动端适配

**不在范围内**：完整的 design system、移动端适配、动画系统。

**验收标准**：
- [ ] 所有页面暗色模式无颜色错误（无白字白底、黑字黑底）
- [ ] 空图谱页面显示引导文案和 "创建蓝图" 按钮
- [ ] 导航栏显示 logo

**前置依赖**：GAP-05（命名统一，避免改两遍）

**涉及**：全局 CSS、`components/layout/`、各页面组件

---

## P4 — 后期优化

### FEAT-09: 数据管理最小可用

**用户故事**：作为用户，我需要查看和清理 Ring 中的数据，避免无限增长。

**范围**：
- 文件预览：在管理页面可预览 markdown 文件内容
- 手动删除：按文件删除，显示文件大小和最后访问时间，删除前二次确认
- 存储概览：按类别显示磁盘占用（对话、归档、图谱、日志）

**不在范围内**：自动清理策略、保留期配置、undo grace period。

**验收标准**：
- [ ] 设置页显示各类数据的磁盘占用
- [ ] 点击文件可预览内容
- [ ] 删除需二次确认，删除后文件确实消失

**前置依赖**：GAP-04（节点 markdown 文件）

**涉及**：新增 `pages/Settings/DataManagement.tsx`、后端新增文件管理 API

---

## 依赖关系图

```
GAP-01 ──┬── FEAT-03 (Ring Super 查询)
         └── FEAT-05 (Archive Pipeline)

GAP-02 ──── FEAT-03 (Ring Super 查询)
GAP-03 ──┬── FEAT-02 (工具调用展示)
         ├── FEAT-03 (Ring Super 工具)
         └── FEAT-06 (Monitor 页面)
GAP-04 ──┬── FEAT-04 (Blueprint 预览)
         ├── FEAT-05 (Archive Pipeline)
         └── FEAT-09 (数据管理)
GAP-05 ──── FEAT-08 (UI 统一)
FEAT-01 ──── FEAT-06 (Monitor 数据源)
```

## 实施顺序建议

```
Phase 1 (P0): GAP-01 → GAP-02 → GAP-04 → GAP-03 → GAP-05
Phase 2 (P1): GAP-06 → GAP-07 → GAP-08
Phase 3 (P2): FEAT-01 → FEAT-02 → FEAT-04 → FEAT-03 → FEAT-05
Phase 4 (P3): FEAT-07 → FEAT-06 → FEAT-08
Phase 5 (P4): FEAT-09
```

## 功能总览

| ID | 优先级 | 简述 | 层级 | 前置依赖 |
|----|--------|------|------|----------|
| FEAT-01 | P2 | 结构化日志 (tracing) | 后端 | 无 |
| FEAT-02 | P2 | 工具调用对话内展示 | 前端 | GAP-03 |
| FEAT-03 | P2 | Ring Super 跨 Ring 查询 | 全栈 | GAP-01,02,03 |
| FEAT-04 | P2 | Blueprint 结构化确认 | 前端 | GAP-04 |
| FEAT-05 | P2 | Archive → Graph Pipeline | 后端 | GAP-01,04,07 |
| FEAT-06 | P3 | Monitor 监控页面 | 全栈 | FEAT-01, GAP-03 |
| FEAT-07 | P3 | 流式输出体验优化 | 前端 | 无 |
| FEAT-08 | P3 | UI 布局视觉统一 | 前端 | GAP-05 |
| FEAT-09 | P4 | 数据管理最小可用 | 全栈 | GAP-04 |
