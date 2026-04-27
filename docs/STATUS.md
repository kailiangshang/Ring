# Ring 项目状态

> 最后更新：2026-04-28

## 开发概况

- 后端 67 个 Rust 源文件，~14,000 行
- 前端 86 个 TS/TSX 文件，~9,800 行
- 16 个数据库迁移
- 70 集成测试通过（1 个已知 OS detection 问题）
- 所有 AI 提示词统一管理于 `server/src/prompts.rs`
- Release 二进制 16MB（含前后端全部代码，零外部依赖）

## 本轮完成（2026-04-28）

### PRD 7.2 全部清零

- **Graph Panel 节点树列表视图** — Canvas/Tree 双模式切换，缩进层级展示，Expand/Collapse All
- **深度调研网页爬取** — Group Ring `fetch_url` tool，HTML 清洗 + 15K 截断 + 安全限制
- **PDF 导出** — `/export/chat-pdf` 端点，Markdown→PDF 原生生成，前端 Export + PDF 双按钮
- **Git revert** — 后端 git-log/revert API + 前端 ArchivePanel Git History 区（Refresh + Revert 按钮）
- **Super Ring create_ring tool** — AI 通过 tool_call 创建 Ring
- **Self growth.md** — memory_files 增加 growth 类别，记忆提取支持成长里程碑

### 同级问题审计 + 修复

- 🔴 `archive_service.rs` 表名 `ring_members` → `members`
- 🟡 `RingRow` struct 补充 `storage_mode` + `gitlab_namespace` 字段
- 🟡 `delete_graph` 全局计数改为 per-ring 计数
- 🔴 `export.rs` 整库备份表名 `archives` → `archive_records`
- 🔴 `graphs.ring_id` UNIQUE 约束移除（migration 016），支持多图谱
- 🟡 `ensure_default_graph` ID 改用 ULID 避免 ID 冲突

### 发布优化

- `strip = true` + `thin LTO` → 19MB → 16MB
- `BACKEND_TEST.md` — 592 行完整后端测试文档，26 个模块

## 上轮完成（2026-04-26）

### 提示词优化（6 commits）

- Chat: Group Ring / Self / Super Ring — `<thinking>` + `<output_rules>`
- Archive: extract/judge/compact/group_docs — 粒度约束
- Session: 5 skills 统一框架
- Workflow/export/blueprint — 关系语义约束
- Blueprint: 追问式头脑风暴提示词（挑战模糊需求）

### 数据同步（6 commits）

- **Migration 015**: `sync_meta` 表（per-ring sync state tracking）
- **sync service**: `export_bundle`（graphs + archive_records + group_docs + archive_files 序列化为 JSON）+ `import_bundle`（upsert with creator-wins）
- **Routes**: `GET /rings/{id}/sync/bundle` + `POST /rings/sync/import`（member backend fetches from creator）
- **Auto-sync**: `local_join` 后自动从 creator 拉取 bundle
- **Graph auto-persist**: 每次图谱变更后写 `graphs/main.json` + git commit
- **Frontend API**: `getSyncBundle` + `postSyncImport` 替换旧的 `getSyncSnapshot`/`getSyncDelta`

### 架构文档

- `docs/ARCHITECTURE.md` — 全量数据模型、请求流程、多人协作现状、同步缺口分析

## 上轮完成（2026-04-25）

### 交互优化（36 项）

**P0（8 项）**：
- 聊天自动滚动（smart bottom detection）
- Self chat 网络失败消息保留
- D3 enter/update/exit 保留缩放状态
- 删除节点/Session/Archive merge 确认对话框
- Auto Compact 开关乐观更新 + 回滚
- Blueprint 确认逐节点错误追踪

**P1（19 项）**：
- Archive 按钮触发实际归档
- SelfChat STOP 按钮（AbortController）
- 方向键仅首尾导航历史
- Session context 系统消息
- Export 成功/错误反馈
- 弹出层 click-outside + Escape 关闭
- Skill 模式 tooltip 说明
- 通知删除确认 + 点击跳转
- Modal Escape + Tab 焦点陷阱
- RingList loading/空状态
- RingListItem hover 效果
- SessionIndicator 正确显示
- session-store 错误状态 UI
- Summarize 改为按钮触发
- Phase 标签翻译
- ConfigPanel revoke 确认
- SuperSettingsPanel 保存反馈
- SuperSkillsPanel 移除确认

**P2（9 项）**：
- Graph 缩放控件 + Download SVG
- SEND 空 input 禁用
- SelfChat input→textarea
- SessionPanel "Leave session"
- PanelWrapper Escape + aria-label
- ArchivePanel loading 状态
- 全局 :focus-visible CSS
- GraphPanel 空状态 + Export 反馈
- ConfigPanel/SuperSettings label htmlFor

### 功能改进

- **Session Tab** — 顶部 Tab 栏新增 Session 入口 + 活跃指示灯
- **Session 创建引导** — 欢迎说明 + 创建成功 banner + 材料准备解释
- **Ring 创建引导** — 创建后自动打开 Graph 面板引导蓝图
- **消息折叠** — 长 AI 回复（>200px）自动折叠，渐变遮罩 + EXPAND/COLLAPSE
- **命令键盘翻页** — autocomplete 选中项 scrollIntoView
- **@导航** — @self 打开 Self 浮窗并聚焦输入框，@super 切换上下文
- **重名 Ring 防护** — 同用户不可创建同名 Ring
- **提示词统一管理** — 12 处提示词收入 `prompts.rs`，围绕知识协作主线重写
- **用户不可见提示词** — 移除 System Prompt / Preferences 编辑器
- **UTF-8 安全截断** — 4 处 `&content[..n]` 改为 `chars().take(n)`
- **文件上传** — 支持在 Group Ring/Super Ring 聊天和 Session 材料准备中上传文本文件（PDF/TXT/MD/CSV/代码），自动提取内容注入对话上下文，📎 按钮上传 + 拖拽 + 粘贴，文件卡片渲染
- **跨 Ring 全文搜索** — SQLite FTS5 索引所有 Ring 的消息/节点/Session/文档/归档文本，Super Chat 自动检索注入 `<cross_ring_context>`，`[Ring > Title]` 可点击引用
- **Self Memory 自动提取** — core_identity/knowledge/goals 三个 Tier 1 文件，LLM 异步提取 + 自动压缩，前端记忆面板查看/编辑/删除
- **深度蓝图构建器** — AI 引导的多轮对话式蓝图设计，`<blueprint>` JSON 结构化输出，D3.js 实时预览，多图谱支持，滑动窗口 + current_blueprint 注入上下文管理，快速路径边创建 bug 修复
- **Self Metrics** — dwell_time 心跳式追踪（30s 前端心跳 + 后端批量刷盘），tool_usage 覆盖 9 类操作（search/graph_edit/archive/upload/export/blueprint/session_create/session_summarize/memory_extract），已有指标桩接入实际路由，metrics 摘要注入 Self 系统提示词，前端扩展指标面板
- **预设工作流工具** — Group Ring tool_calls 基础设施（chat_stream_with_tools），file_parse（文件解析 → 结构化提取 → 图谱节点推荐）和 knowledge_extract（文本 → 概念提取 → 图谱节点推荐），AI 自动决定何时调用，`<file_analysis>` / `<knowledge_extraction>` 结构化输出卡片，一键添加到图谱
- **Cross Ring Cache** — Super Ring 内存缓存基础设施，`summary:{user_id}` / `detail:{ring_id}` / `graph:{ring_id}` 三类缓存键，5 分钟 TTL + 主动失效（归档/图谱/环/成员变更时触发），`stream_super_chat_inner` 和 `stream_cross_ring_query_inner` 使用缓存避免重复 SQL + 文件 I/O

### 代码清理

- 技术债批次1：18 处 `.unwrap()` 改为直接 `Json(result)` 返回
- `active_ring_id` 去重（删除 app-store 副本，统一到 ring-store）
- Token 获取统一为 `api.getToken()`

## 功能完成状态

### 基础设施

| 功能 | 状态 |
|---|---|
| 三栏布局 (Sidebar + Chat + PanelStack) | done |
| IceChat 深色主题 (Cascadia Code + Space Grotesk) | done |
| Handler → Service → Model 三层分离 | done |
| Auth (X-Ring-Token) + Token 恢复 | done |
| Setup 向导 (5 步) + Skip GitLab | done |
| Setup Done 命令速查表 | done |
| API Key / Git 凭证 AES-256 加密 | done |

### 聊天系统

| 功能 | 状态 |
|---|---|
| Group Ring / Super Ring / Self 三层聊天 | done |
| SSE 流式输出 + tool_calls 支持 | done |
| 聊天历史分页加载 + 自动滚动 | done |
| Markdown 渲染 (react-markdown + remark-gfm) | done |
| 长消息折叠（>200px 自动收起） | done |
| 命令补全弹出框 (/ @ ! % 前缀触发 + 键盘翻页) | done |
| 命令历史 (上下箭头，仅首尾触发) | done |
| Token 用量显示 (每条消息) | done |
| Auto compact (上下文超限自动压缩) | done |
| Ephemeral 模式 (临时消息不保存) | done |
| 隐私过滤 (手机号/身份证/邮箱/银行卡脱敏) | done |
| 消息气泡左右分布（用户右，AI 左） | done |
| Self STOP 按钮（中断流式输出） | done |
| @self 跳转 Self 聊天，@super 切换上下文 | done |

### 图谱系统

| 功能 | 状态 |
|---|---|
| D3.js 力导向图可视化（缩放状态保持） | done |
| Node / Edge CRUD | done |
| 节点类型 (category/leaf) + metadata | done |
| 多图谱支持 (每 Ring 最多 3 个) | done |
| 图谱选择器 UI | done |
| 标签过滤 | done |
| 展开/折叠子节点 | done |
| Graph 对话修正 (自然语言操作图谱) | done |
| 导出 graph.json / SVG + 缩放控件 | done |
| 图谱空状态引导 | done |

### 归档系统

| 功能 | 状态 |
|---|---|
| 对话 → 图谱节点 + Markdown + Git commit | done |
| Creator 直接 commit, Member 提交 MR | done |
| Archive queue + PR Review (merge/reject 确认) | done |
| PR Diff 视图 | done |
| 归档触发多样化 (命令/按钮/自然语言/AI 推荐) | done |
| Auto 模式 (AI 自动判断归档) | done |
| Ring Git 初始化 (loading 状态) | done |

### Session 系统

| 功能 | 状态 |
|---|---|
| Session CRUD + 全生命周期管理 | done |
| 顶部 Tab 栏入口 + 活跃指示灯 | done |
| 创建引导 + 成功 banner + 材料准备说明 | done |
| WebSocket 实时多人聊天 | done |
| Owner 离线暂停 / 重连恢复 / Catch-up | done |
| 材料准备 + 高亮标记 | done |
| AI 总结 (按钮触发，SSE 流式) | done |
| Phase 标签翻译 (material_prep → Preparing Materials) | done |
| Skill 集成 (5 个 Skill，中文提示词) | done |
| Skill 安装/卸载 (从 URL) | done |
| Session grant/revoke 权限控制 | done |
| Session 所有权转移 | done |

### Super Ring

| 功能 | 状态 |
|---|---|
| 始终流式 + tool_calls (跨 Ring 查询/偏好/Skill 管理) | done |
| 跨 Ring 问答/分析 | done |
| **全文搜索（跨 Ring 知识检索）** — SQLite FTS5 索引所有 Ring 的消息/节点/Session/文档/归档文本，Super Chat 自动检索并注入 `<cross_ring_context>`，LLM 输出 `[Ring名 > 标题]` 格式引用，前端渲染为可点击链接 | done |
| Skill 管理 (安装/卸载) | done |
| LLM / GitLab 配置 | done |
| ~~System Prompt 编辑~~ | **removed** — 提示词统一管理，不对用户暴露 |
| ~~用户偏好编辑~~ | **removed** — 同上 |

### Self 系统

| 功能 | 状态 |
|---|---|
| Memory / Personality / Privacy / Export / Reset | done |
| **Self Memory 自动提取** — core_identity/knowledge/goals 三个 Tier 1 文件，LLM 异步提取 + 自动压缩，前端查看/编辑/删除 | done |
| @self 转发 (从 Group Ring) + 跳转聚焦 | done |
| 主动建议 | done |
| 数据导出/重置 | done |
| STOP 按钮 + textarea 多行输入 | done |

### 协作

| 功能 | 状态 |
|---|---|
| 邀请 Token CRUD (open/audit) + revoke 确认 | done |
| 开放链接加入 + 审核链接 + 审批流程 | done |
| 安装导航页 (OS 检测) | done |
| 成员列表 + 角色变更 + 移除（Session ownership 保护） | done |
| 通知系统 (bell + 未读数 + 点击跳转 + 删除确认) | done |
| 重名 Ring 防护 | done |

### 导出 / 配置

| 功能 | 状态 |
|---|---|
| 聊天/会话/图谱/全 Ring 备份导出（含反馈） | done |
| 图谱 SVG + AI 结构化报告 + tar.gz | done |
| .group/ 文档体系 (6 份文档) | done |
| .group/ AI 自动维护 (active-context/archive-patterns/corrections/knowledge-summary) | done |
| Blueprint / 模板系统 (5 个内置模板) + 错误追踪 | done |
| Token 阈值管理 (100k limit, 80% warning) | done |
| Config 面板 (LLM / GitLab / Privacy / auto_compact) | done |

### 通用交互

| 功能 | 状态 |
|---|---|
| 全局 :focus-visible 轮廓 | done |
| Modal / Panel Escape 关闭 + 焦点陷阱 | done |
| 弹出层 click-outside 关闭 | done |
| 确认对话框（删除节点/Session/Archive/Merge） | done |
| 操作反馈（保存/导出/删除成功或失败） | done |
| 空状态引导（Ring/Graph/Session/Archive/Notification） | done |

## PRD 缺失项

| PRD 要求 | 状态 | 优先级 |
|---|---|---|
| 预设工作流工具（文件解析/知识提取/深度调研） | done（file_parse + knowledge_extract + fetch_url，AI tool_calls 驱动） | 低 |
| Super Ring create_ring tool | done（AI 通过 tool_call 创建 Ring） | 低 |
| Self growth.md 成长日志 | done（memory_files 增加 growth 类别） | 低 |
| PDF 导出 | done（/export/chat-pdf + 前端 PDF 按钮） | 低 |
| Git revert | done（git-log + revert API + ArchivePanel Git History UI） | 低 |
| Graph Panel 节点树列表视图 | done（Canvas/Tree 双模式切换） | 中 |
| Super Ring `cross_ring_cache/` 缓存 | done（内存 TTL 缓存，ring_summary + ring_detail，主动失效） | 低 |
| Self 完整文件体系（knowledge/goals/growth） | done（Tier 1 记忆文件已实现） | 中 |
| Self metrics（dwell_time/tool_usage） | done（心跳式 dwell_time + 全路由 tool_usage + Self AI 注入） | 中 |
| 深度蓝图构建器（AI 对话式） | done（AI 引导多轮对话 + D3 实时预览 + 多图谱 + 边创建修复） | 中 |
| 图谱 PNG/PDF 导出 | done（SVG → Canvas → PNG 2400×1600） | 低 |
| Session 侧栏树形展示 | done（Ring 展开/折叠，嵌套 Session 列表，phase 色点 + skill 标签） | 中 |
| **存储后端抽象** | **done（StorageBackend trait，Local + GitLab 两种模式，归档/审核流程一致）** | 中 |
| **本地模式（无外部依赖）** | **done（本地 git init，pending_reviews 表审核队列，bundle sync API）** | 中 |
| **GitLab 模式** | **done（GitLab REST API v4 MR 审核，PRIVATE-TOKEN 认证，公司内网）** | 中 |
| **多人数据同步** | **done（HTTP bundle sync，creator-wins upsert，auto-sync after join，graph auto-persist）** | 高 |
| 移动端适配 | 缺失 | 延后 |

## 技术债（审计记录，不阻塞）

- ~~`chat-store.ts` 733 行 god object，导入 10 个 store~~ — **已修复**（拆分 command-handlers.ts + createSseCallbacks 工厂）
- ~~`super_chat.rs` 流式循环 3 份重复（~300 行），待统一到 LlmClient~~ — **已修复**（统一到 LlmClient::chat_stream）
- ~~非 stream 请求无 AbortController~~ — **已修复**（api.get/post/put/delete 支持 signal，triggerArchiveSSE/exportFile/uploadFile/summarize 支持 abort）
- ~~`active_ring_id` 双份状态（app-store + ring-store）~~ — **已修复**
- ~~Token 获取方式 4 种不统一~~ — **已修复**
- ~~16 个 `.unwrap()` 在路由 handler 中~~ — **已修复**
- ~~`readonly` 角色权限未执行~~ — **已修复**（reject_readonly 覆盖 30+ 写端点 + WebSocket）
- ~~CORS 允许所有 Origin~~ — **已修复**（仅 localhost）
