# Ring 项目现状

> 最后更新：2026-04-18

## 技术栈

| 层 | 技术 |
|---|------|
| 后端 | Rust + Axum 0.8 + SQLite (sqlx) |
| 前端 | React 19 + TypeScript + Zustand 5 + Vite 8 |
| LLM | async-openai (OpenAI + Ollama) |
| 实时通信 | WebSocket (axum ws) |
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

### Plan 1: 前端骨架
- 三栏布局（Sidebar + ChatArea + PanelStack）
- IceChat 深色主题（Cascadia Code + Space Grotesk）
- Sidebar: Super Ring 入口 + Ring 列表 + Session 指示器
- HeaderTabBar: Chat / Graph / Archive / Config
- Self 浮动窗口（可拖拽，Chat/Memory/Settings）
- Setup 向导（5 步：Welcome→Identity→LLM→GitLab→Done）
- Zustand stores（app, ring, panel, mode, self, chat, graph, ws, session）

### Plan 2: Chat + Command + API 集成
- CLI 命令系统：`@` (addressing) / `#` (reference) / `!` (action) / `%` (meta)
- 前端 API 客户端（`services/api.ts`）
- Setup 提交真实后端
- Mode 同步到服务端

### Plan 3: 后端 API 核心
- Handler → Service → Model 三层分离
- 16 个 API 端点
- SQLite 迁移（001-004）
- Auth: X-Ring-Token header
- 6/6 集成测试通过

### Plan 4a: LLM Chat + SSE Streaming
- async-openai 集成（OpenAI + Ollama via base_url）
- SSE 流式输出（delta 逐 token）
- Chat 消息持久化到 SQLite
- 前端 SSE 消费 + streaming 光标动画

### Plan 4b: Graph 可视化
- D3.js force-directed graph
- Graph CRUD backend（nodes + edges）
- `!graph` 打开面板, `!node <name>` 快速创建
- 节点选中详情 + 删除

### Plan 5a: Session CRUD 后端
- 4 张新表：sessions, session_participants, session_messages, session_materials
- 10 个 API 端点（CRUD + participants + archive toggle）
- 单 Ring 单活跃 Session 约束
- 9/9 集成测试通过

### Plan 5b: WebSocket 实时聊天
- WsHub（DashMap 并发连接管理）
- WS 端点：`/api/ws?token=<token>`
- 消息中转 + 广播
- Owner 离线检测 → session_paused
- Catch-up 机制（基于 seq_num）
- 心跳（30s ping / pong）

### Plan 5c: 前端 SessionPanel
- WebSocket 客户端（自动重连 + 指数退避）
- Session store（CRUD + WS 消息处理）
- SessionPanel 完整 UI：创建表单 + 实时聊天
- Sidebar SessionIndicator 显示标题 + 人数

### Plan 5d: Material Prep + Summary + Skills
- Skill 服务（5 个内置 skill 定义）
- 后端端点：`POST start` / `POST summarize` (SSE) / `GET material-prep` / `POST highlights`
- 前端 MaterialPrepView（材料列表 + 高亮 + 开始讨论）
- 前端 SummarizeView（SSE 流式 AI 总结）

## 当前后端 API 端点

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/health` | 健康检查 |
| GET | `/api/ws` | WebSocket 连接 |
| GET | `/api/setup/status` | Setup 状态 |
| POST | `/api/setup` | 提交 Setup |
| PUT | `/api/setup` | 更新 Setup |
| GET | `/api/rings` | Ring 列表 |
| POST | `/api/rings` | 创建 Ring |
| GET | `/api/rings/{id}` | Ring 详情 |
| GET | `/api/rings/{id}/members` | 成员列表 |
| PUT | `/api/rings/{id}/members/{tid}/role` | 角色变更 |
| DELETE | `/api/rings/{id}/members/{tid}` | 移除成员 |
| GET/PUT | `/api/config/llm` | LLM 配置 |
| GET/PUT | `/api/rings/{id}/mode` | 交互模式 |
| GET/PUT | `/api/rings/{id}/group-docs/{name}` | Group 文档 |
| POST | `/api/rings/{id}/chat` | Group Ring 聊天 (SSE) |
| GET | `/api/rings/{id}/chat/history` | 聊天历史 |
| POST | `/api/self/chat` | Self 聊天 (SSE) |
| GET | `/api/self/chat/history` | Self 聊天历史 |
| GET | `/api/rings/{id}/graph` | 图谱 |
| POST | `/api/rings/{id}/graph` | 创建节点 |
| PUT | `/api/rings/{id}/graph/nodes/{nid}` | 更新节点 |
| DELETE | `/api/rings/{id}/graph/nodes/{nid}` | 删除节点 |
| POST | `/api/rings/{id}/graph/edges` | 创建边 |
| DELETE | `/api/rings/{id}/graph/edges/{eid}` | 删除边 |
| GET | `/api/rings/{id}/sessions` | Session 列表 |
| POST | `/api/rings/{id}/sessions` | 创建 Session |
| GET | `/api/rings/{id}/sessions/{sid}` | Session 详情 |
| DELETE | `/api/rings/{id}/sessions/{sid}` | 删除 Session |
| POST | `/api/rings/{id}/sessions/{sid}/close` | 关闭 |
| POST | `/api/rings/{id}/sessions/{sid}/reopen` | 重开 |
| POST | `/api/rings/{id}/sessions/{sid}/start` | 开始讨论 |
| POST | `/api/rings/{id}/sessions/{sid}/summarize` | AI 总结 (SSE) |
| GET | `/api/rings/{id}/sessions/{sid}/messages` | 消息历史 |
| GET | `/api/rings/{id}/sessions/{sid}/material-prep` | 材料准备 |
| POST | `/api/rings/{id}/sessions/{sid}/material-prep/highlights` | 标记高亮 |
| POST | `/api/rings/{id}/sessions/{sid}/participants` | 邀请成员 |
| DELETE | `/api/rings/{id}/sessions/{sid}/participants/{tid}` | 移除成员 |
| PUT | `/api/rings/{id}/sessions/{sid}/archive-toggle` | 归档开关 |

## 未完成的功能

按优先级排列：

| 优先级 | 功能 | PRD 章节 | 说明 |
|--------|------|----------|------|
| **高** | 归档机制 | 2.4 | 对话内容 → 图谱节点 + Markdown + Git commit，核心价值 |
| **高** | Git/GitLab 集成 | 2.5 | clone, commit, push, pull, MR，协作基础 |
| **高** | PR 审核队列 | 2.4 | 创建者审核成员归档 PR，多用户工作流 |
| **中** | 邀请/加入流程 | 4.4, 6.3 | 开放链接 + 审核链接，多用户入门 |
| **中** | Super Ring | 2.6 | 跨 Ring 分析、Skill 安装、全局助手 |
| **中** | 蓝图构建器 | 6.1.2 | 快速模板 + 深度对话，Ring 初始化 |
| **中** | 通知系统 | 2.10 | PR/成员/Session 事件通知 |
| **低** | 导出中心 | 2.8 | 7 种导出格式 |
| **低** | Self 增强 | 2.6 | metrics, knowledge files |
| **低** | 上下文压缩 | 2.9 | Token 管理，对话 compact |
| **低** | 单二进制分发 | 6 | 前端嵌入 Rust 二进制打包 |

## 关键文件索引

```
Ring/
├── AGENTS.md                              # AI 编码指引
├── README.md                              # 项目概述
├── docs/
│   ├── STATUS.md                          # 本文件 — 项目现状
│   ├── product/
│   │   ├── PRD.md                         # 产品需求文档（1,049 行）
│   │   ├── UI-DESIGN.md                   # 前端 UI 设计规范
│   │   └── style-previews/               # HTML 原型预览
│   ├── superpowers/
│   │   ├── specs/                         # 设计规格
│   │   │   ├── 2026-04-15-ring-redesign-design.md    # 四层架构设计
│   │   │   ├── 2026-04-17-cli-command-system-design.md  # CLI 命令系统
│   │   │   └── 2026-04-19-session-lifecycle-design.md    # Session 生命周期
│   │   └── plans/                         # 实施计划（已完成，留作参考）
│   ├── technical/
│   │   └── api-design.md                  # REST API 文档
│   └── testing/
│       └── manual-test-guide.md           # 手动测试指南
├── server/                                # Rust 后端
│   ├── src/
│   │   ├── state.rs                       # AppState { db, ws_hub }
│   │   ├── ws_hub.rs                      # WebSocket Hub
│   │   ├── error.rs                       # RingError 统一错误
│   │   ├── models/                        # 数据模型 + SQL 查询
│   │   ├── services/                      # 业务逻辑
│   │   │   ├── chat.rs                    # Group Ring / Self 聊天
│   │   │   ├── llm.rs                     # LLM 客户端 + SSE 流
│   │   │   ├── session.rs                 # Session 业务逻辑
│   │   │   └── skill.rs                   # Skill 定义 + prompt
│   │   └── routes/                        # HTTP/WS 路由
│   │       ├── chat.rs                    # SSE 聊天端点
│   │       ├── session.rs                 # Session 端点
│   │       └── ws.rs                      # WebSocket handler
│   └── migrations/                        # SQLite 迁移 (001-005)
└── ui/                                    # React 前端
    └── src/
        ├── stores/                        # Zustand stores
        │   ├── session-store.ts           # Session CRUD + WS
        │   ├── ws-store.ts                # WebSocket 连接管理
        │   ├── chat-store.ts              # 聊天状态
        │   └── graph-store.ts             # 图谱状态
        ├── services/
        │   ├── api.ts                     # REST API 客户端
        │   ├── ws-client.ts               # WebSocket 客户端
        │   └── sse.ts                     # SSE 流消费
        ├── components/
        │   ├── panels/SessionPanel.tsx     # Session 完整 UI
        │   ├── panels/GraphPanel.tsx       # D3 图谱面板
        │   ├── sidebar/                    # 侧边栏组件
        │   └── chat/                       # 聊天组件
        └── types/session.ts               # Session 类型定义
```

## 测试

```bash
cd server && cargo test     # 9/9 集成测试通过
cd ui && npm run build      # TypeScript + Vite build 通过
```
