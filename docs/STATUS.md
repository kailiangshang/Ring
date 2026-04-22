# Ring 项目现状

> 最后更新：2026-04-21

## 技术栈

| 层 | 技术 |
|---|------|
| 后端 | Rust + Axum 0.8 + SQLite (sqlx) |
| 前端 | React 19 + TypeScript + Zustand 5 + Vite 8 |
| LLM | async-openai（OpenAI + Anthropic + Ollama）+ 自建 Anthropic 适配层 |
| 实时通信 | WebSocket (axum ws) + SSE 流式输出 |
| 分发 | 单一二进制，前端嵌入后端 serve |

## 架构

```
Ring Hub（用户入口）
├── Super Ring    — 全局助手，Ring 管理引导，跨 Ring 分析
├── Group Ring    — 群组专属 AI，读写本 Ring 图谱和归档
├── Session Ring  — 多人实时讨论，加载 Skill 决定行为
└── Self          — 用户私有 AI 宠物，完全私有，不进 Git
```

## 已完成的功能

### 基础架构
- 三栏布局（Sidebar + ChatArea + PanelStack）
- IceChat 深色主题（Cascadia Code + Space Grotesk）
- Handler → Service → Model 三层分离
- Auth: X-Ring-Token header
- Token 恢复机制（写入 ~/.ring/hub/token，前端自动恢复）

### Setup 向导
- 5 步：Welcome → Identity → LLM Config → GitLab → Done
- LLM Model 输入框（按 provider 默认值：openai→gpt-4o, anthropic→claude-sonnet-4, ollama→qwen2.5）
- TEST CONNECTION 按钮（LLM + GitLab，15s 超时，返回具体错误信息）
- JOIN EXISTING 分支（token + creator_ip URL 参数自动进入 join 流程）

### Chat 系统
- Group Ring / Super Ring / Self 三层聊天
- SSE 流式输出（Super Ring 始终流式，支持 tool_calls 中途执行工具再流式）
- 聊天历史加载
- Markdown 渲染（react-markdown + remark-gfm，表格/代码/标题/列表/引用全部适配 IceChat 主题）

### CLI 命令系统
- 四前缀：`@` (addressing) / `#` (reference) / `!` (action) / `%` (meta)
- `/` 统一命令前缀（`/graph` = `!graph`，`/prefs` = `%prefs`，等）
- `/help` 显示完整命令表
- 命令自动补全弹出框（输入 `/`、`!`、`%`、`@` 触发，上下键选择 + 回车确认）
- 上下文感知命令提示栏（Super Ring / Ring / Session 显示不同命令）
- UI 命令不发送给 AI（只切换面板）

### 图谱可视化
- D3.js force-directed graph（缩放、拖拽、节点选中）
- Graph CRUD backend（nodes + edges）
- 节点类型颜色区分 + 边标签

### 归档系统
- 对话 → 图谱节点 + Markdown + Git commit
- Creator 直接 commit，Member 提交 MR
- Archive queue + PR review（merge/reject）
- 自动归档

### Session 全生命周期
- 4 张表：sessions, session_participants, session_messages, session_materials
- WebSocket 实时聊天（WsHub + DashMap）
- Owner 离线暂停 / 重连恢复
- Catch-up（基于 seq_num）
- Session pause/resume
- AI 总结（SSE 流式）
- 材料准备 + 高亮
- Skill 系统（5 个内置 + 从 URL 安装 + 卸载）

### Super Ring
- 始终流式输出 + tool_calls 支持（跨 Ring 查询、偏好管理、Skill 管理）
- HeaderTabBar：Chat / Skills / Settings
- Skills 面板：已安装列表 + 安装/卸载
- Settings 面板：LLM Config + GitLab Config (Optional) + System Prompt + Preferences
- System Prompt 编辑 + 用户偏好编辑
- ModeIndicator 显示 `[super]` / `[ring]` / `[session]`

### 邀请/加入流程
- Invite Token CRUD（open / audit 两种类型）
- 开放链接加入（3 个端点：验证、加入、本地加入）
- 审核链接 + 审批流程（5 个端点：申请、查状态、列表、批准、拒绝）
- 安装导航页（检测 OS，显示下载链接）
- 前端 CreateInviteModal（ConfigPanel 内触发）
- 前端 StepJoin（验证 token → join/poll）

### 成员管理
- 成员列表 + 角色变更 + 移除
- 侧栏 + new ring 按钮（可点击创建）

## 后端 API 端点（完整）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/health` | 健康检查 |
| GET | `/api/ws` | WebSocket 连接 |
| GET | `/api/setup/status` | Setup 状态 |
| GET | `/api/setup/recover` | 恢复 token（无 auth） |
| POST | `/api/setup` | 提交 Setup |
| PUT | `/api/setup` | 更新 Setup |
| GET | `/api/rings` | Ring 列表 |
| POST | `/api/rings` | 创建 Ring |
| GET | `/api/rings/{id}` | Ring 详情 |
| GET | `/api/rings/{id}/members` | 成员列表 |
| POST | `/api/rings/{id}/members` | 添加成员 |
| PUT | `/api/rings/{id}/members/{tid}/role` | 角色变更 |
| DELETE | `/api/rings/{id}/members/{tid}` | 移除成员 |
| GET/PUT | `/api/config/llm` | LLM 配置 |
| POST | `/api/config/llm/test` | 测试 LLM 连接（无 auth） |
| POST | `/api/config/gitlab/test` | 测试 GitLab 连接（无 auth） |
| GET/PUT | `/api/rings/{id}/mode` | 交互模式 |
| GET/PUT | `/api/rings/{id}/group-docs/{name}` | Group 文档 |
| POST | `/api/rings/{id}/chat` | Group Ring 聊天 (SSE) |
| GET | `/api/rings/{id}/chat/history` | 聊天历史 |
| POST | `/api/self/chat` | Self 聊天 (SSE) |
| GET | `/api/self/chat/history` | Self 聊天历史 |
| POST | `/api/super/chat` | Super Ring 聊天 (SSE) |
| GET | `/api/super/chat/history` | Super Ring 聊天历史 |
| GET/PUT | `/api/super/system-prompt` | 系统提示词 |
| GET/PUT | `/api/super/preferences` | 用户偏好 |
| GET | `/api/skills` | Skill 列表 |
| POST | `/api/skills/install` | 安装 Skill |
| GET/DELETE | `/api/skills/{name}` | Skill 详情/删除 |
| GET/POST/DELETE | `/api/rings/{id}/graph` | 图谱 CRUD |
| PUT/DELETE | `/api/rings/{id}/graph/nodes/{nid}` | 节点更新/删除 |
| POST/DELETE | `/api/rings/{id}/graph/edges` | 边 CRUD |
| GET/POST | `/api/rings/{id}/sessions` | Session 列表/创建 |
| GET/DELETE | `/api/rings/{id}/sessions/{sid}` | Session 详情/删除 |
| POST | `/api/rings/{id}/sessions/{sid}/close` | 关闭 |
| POST | `/api/rings/{id}/sessions/{sid}/reopen` | 重开 |
| POST | `/api/rings/{id}/sessions/{sid}/start` | 开始讨论 |
| POST | `/api/rings/{id}/sessions/{sid}/summarize` | AI 总结 |
| GET | `/api/rings/{id}/sessions/{sid}/messages` | 消息历史 |
| GET | `/api/rings/{id}/sessions/{sid}/material-prep` | 材料准备 |
| POST | `/api/rings/{id}/sessions/{sid}/material-prep/highlights` | 标记高亮 |
| POST/DELETE | `/api/rings/{id}/sessions/{sid}/participants` | 参与者管理 |
| PUT | `/api/rings/{id}/sessions/{sid}/archive-toggle` | 归档开关 |
| POST | `/api/rings/{id}/archive` | 触发归档 |
| GET | `/api/rings/{id}/archives` | 归档列表 |
| GET | `/api/rings/{id}/archives/{aid}` | 归档详情 |
| POST | `/api/rings/{id}/archives/{aid}/review` | 审核 MR |
| GET | `/api/rings/{id}/archive-queue` | 归档队列 |
| GET | `/api/rings/{id}/repo/status` | Git 仓库状态 |
| POST | `/api/rings/{id}/repo/init` | 初始化仓库 |
| POST | `/api/rings/{id}/invite-tokens` | 创建邀请 token |
| GET | `/api/rings/{id}/invite-tokens` | 列出邀请 token |
| DELETE | `/api/rings/{id}/invite-tokens/{token}` | 撤销 token |
| GET | `/api/rings/{id}/join-requests` | 审批请求列表 |
| POST | `/api/rings/{id}/join-requests/{rid}/approve` | 批准 |
| POST | `/api/rings/{id}/join-requests/{rid}/reject` | 拒绝 |
| GET | `/api/join/info` | 验证邀请 token |
| POST | `/api/join` | 开放链接加入 |
| POST | `/api/join/local` | 本地加入 |
| POST | `/api/join/apply` | 申请加入 |
| GET | `/api/join/apply/status` | 查询申请状态 |
| GET | `/ring/join` | 安装导航页（HTML） |

## 未完成的功能

### 核心功能

| # | 功能 | PRD 章节 | 说明 |
|---|------|----------|------|
| 1 | **Self 完整实现** | 2.6, UI 8 | Memory（行为画像/统计/偏好）、Personality 设置、隐私控制、数据导出/重置、主动建议、@self 转发消息 |
| 2 | **Blueprint/模板系统** | 6.1.2 | 模板选择、AI 引导共建图谱、预览确认、%blueprint 命令 |
| 3 | **通知系统** | 2.10 | 数据模型、PR/成员/Session 变更通知、未读列表 UI |
| 4 | **导出中心** | 2.8 | 图谱图片、Markdown、聊天记录、会话记录、全 Ring 备份（7 种格式） |
| 5 | **Context 管理** | 2.9 | ~~Token 用量追踪~~ ✅、自动 compact、ephemeral 模式 |

### AI 自动化

| # | 功能 | PRD 章节 | 说明 |
|---|------|----------|------|
| 6 | **`.group/` AI 自动维护** | 2.6 | active-context、archive-patterns、corrections、knowledge-summary 自动更新 |
| 7 | **Session 材料准备** | 2.12 | AI 根据 Skill 自动收集/生成材料（API 在但逻辑未实现） |
| 8 | **Graph 对话修正** | 6.8 | "删掉那个节点"→ AI 执行变更并提交 |

### 增强功能

| # | 功能 | PRD 章节 | 说明 |
|---|------|----------|------|
| 9 | **PR Review Diff 视图** | 6.4 | PR diff 对比展示 |
| 10 | **图谱展开/折叠、标签过滤** | 2.3 | 节点树操作、标签筛选、节点内容加载 |
| 11 | **CLI 命令补全** | CLI doc | ~~`!session new/close`、`!invite`、`!members`、`@ring`/`@super`/`@username`、`%blueprint`~~ → `/` 和 `@` 统一前缀 ✅ |
| 12 | **API Key / Git 凭证加密** | 3.2 | 当前明文存储 |

### Setup 流程待优化

| # | 功能 | 说明 |
|---|------|------|
| 13 | **GitLab 配置标注 Optional** | ~~StepGitLab 标注可选、加 Skip 按钮、说明文字~~ ✅ |
| 14 | **Setup Done 命令速查** | ~~StepDone 展示可用命令列表~~ ✅ |

## 测试

```bash
cd server && cargo test          # 54/54 集成测试通过
cd ui && npm test                # 22/23 前端测试通过（1 个 pre-existing failure）
cd ui && npx tsc --noEmit        # TypeScript 检查通过
cargo clippy -- -D warnings      # 无警告
```