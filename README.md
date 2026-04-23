# Ring

面向内网的群组知识协作空间。四层 AI 架构 + 知识图谱 + Git 协作。

## 架构

```
Ring Hub（用户入口）
├── Super Ring    — 全局助手，跨 Ring 协调
├── Group Ring    — 群组专属 AI，读写图谱和归档
├── Session Ring  — 多人实时讨论，Skill 驱动行为
└── Self          — 用户私有 AI，完全隔离
```

## 技术栈

| 层 | 技术 |
|---|---|
| 后端 | Rust + Axum 0.8 + SQLite (sqlx) |
| 前端 | React 19 + TypeScript + Zustand 5 + Vite 8 |
| LLM | async-openai (OpenAI / Anthropic / Ollama) |
| 实时通信 | WebSocket + SSE 流式输出 |
| 分发 | 单一二进制，前端嵌入后端 serve |

## 项目结构

```
server/             Rust 后端 (64 文件, ~11,700 行)
  src/routes/       HTTP handlers
  src/services/     业务逻辑
  src/models/       数据模型
  migrations/       SQLite 迁移 (12 个)
ui/                 React 前端 (84 文件, ~9,100 行)
  src/components/   UI 组件
  src/stores/       Zustand 状态管理
  src/services/     API 调用
docs/               文档
```

## 快速开始

```bash
# 后端
cd server && cargo run        # http://localhost:7420

# 前端（开发模式）
cd ui && npm install && npm run dev   # http://localhost:5173

# 生产构建
cd ui && npm run build        # 输出到 ui/dist/
cd server && cargo run        # 自动 serve 前端
```

首次访问 `http://localhost:5173` 进入 Setup 向导。

## 数据目录

```
~/.ring/
├── hub/                     Super Ring 配置 (system_prompt.md, user_preferences.md)
├── rings/<ring-id>/         Ring 数据
│   ├── graph.json           群组图谱
│   ├── sessions/            Session 数据
│   ├── archives/            归档文件
│   └── .group/              Group Ring 行为定义 (role.md, conventions.md, ...)
├── self/                    Self 私有数据
├── skills/                  Skill 插件
└── ring.db                  SQLite 数据库
```

## 文档

| 文档 | 内容 |
|---|---|
| [STATUS.md](docs/STATUS.md) | 功能完成状态 |
| [TEST_GUIDE.md](docs/TEST_GUIDE.md) | 手动测试指南 |
| [PRD.md](docs/product/PRD.md) | 产品需求文档 |
| [UI-DESIGN.md](docs/product/UI-DESIGN.md) | 前端设计规范 |
| [api-design.md](docs/technical/api-design.md) | API 设计参考 |
| [AGENTS.md](AGENTS.md) | AI 编码指引 |

## 测试

```bash
cd server && cargo test       # 56/56 通过
cd ui && npm run build        # 构建验证
```

## License

MIT
