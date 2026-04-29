<div align="center">

<pre style="font-family: monospace; line-height: 1.2;">
██████╗ ██╗███╗   ██╗ ██████╗ 
██╔══██╗██║████╗  ██║██╔════╝ 
██████╔╝██║██╔██╗ ██║██║  ███╗
██╔══██╗██║██║╚██╗██║██║   ██║
██║  ██║██║██║ ╚████║╚██████╔╝
╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝ ╚═════╝ 
</pre>

**面向内网的群组知识协作空间**

[![Rust](https://img.shields.io/badge/Rust-1.85+-orange?logo=rust)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/React-19-61DAFB?logo=react)](https://react.dev)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/Tests-69%2F69%20passing-brightgreen)]()

</div>

---

## 一句话介绍

**Ring 是一个部署在你自己机器上的 AI 驱动知识协作平台。** 让团队对话自动沉淀为结构化知识，支持多人实时讨论、知识图谱可视化、Git 版本化管理，所有数据本地存储，完全私有可控。

---

## 核心特性

### 四层 AI 架构

| 层级 | 角色 | 能力 |
|------|------|------|
| **Super Ring** | 全局助手 | 跨 Ring 知识检索、偏好管理、Skill 安装 |
| **Group Ring** | 群组 AI | 读写知识图谱、归档对话、网页爬取调研 |
| **Session Ring** | 讨论主持人 | 多人实时聊天、材料准备、AI 总结 |
| **Self** | 私有助手 | 个人记忆提取、成长追踪、完全隔离 |

### 知识图谱

- D3.js 力导向图可视化，支持缩放/平移
- 节点树列表视图（Canvas/Tree 双模式切换）
- 多图谱支持（每 Ring 最多 3 个）
- 节点类型：topic / category / leaf
- 展开/折叠子节点，标签过滤

### 协作机制

- **零配置部署**：单一 16MB 二进制，前后端一体
- **本地优先**：SQLite + 文件系统，无外部依赖
- **Git 版本化**：所有归档自动 commit，支持 revert
- **多人互连**：HTTP bundle 同步，creator-wins 策略
- **邀请系统**：Open 链接 / Audit 审核两种模式

### AI 工具箱

- 文件解析（PDF/TXT/MD/CSV/代码）→ 结构化提取
- 知识提取 → 自动推荐图谱节点
- 网页爬取 → 深度调研材料收集
- 蓝图构建器 → AI 引导的多轮对话式图谱设计

---

## 安装

### 方式一：二进制下载（推荐）

```bash
# macOS (Intel)
curl -L https://github.com/kailiangshang/Ring/releases/latest/download/ring-server-darwin-x64 -o ring-server
chmod +x ring-server
./ring-server

# 浏览器打开 http://localhost:7420
```

### 方式二：从源码构建

```bash
git clone https://github.com/kailiangshang/Ring.git
cd Ring

# 构建前端
cd ui && npm install && npm run build && cd ..

# 构建后端（Release 模式，16MB 二进制）
cd server && cargo build --release
./target/release/ring-server
```

### 方式三：npm 全局安装（即将支持）

```bash
npm install -g ring-server
ring-server
```

---

## 快速开始

```bash
# 启动服务
./ring-server

# 浏览器访问
open http://localhost:7420
```

**首次使用：**

1. 打开浏览器，进入 Setup 向导
2. 设置昵称和 LLM 配置（OpenAI / Ollama）
3. 跳过 GitLab 配置（本地模式无需）
4. 进入主界面，开始创建你的第一个 Ring

**常用命令：**

| 命令 | 说明 |
|------|------|
| `/save` | 触发归档 |
| `/graph` | 打开图谱面板 |
| `/session` | 打开 Session 面板 |
| `/help` | 查看所有命令 |

---

## 项目结构

```
Ring/
├── server/                 Rust 后端
│   ├── src/
│   │   ├── routes/         HTTP API 端点
│   │   ├── services/       业务逻辑
│   │   ├── models/         数据模型 + SQL
│   │   └── prompts.rs      所有 AI 提示词（统一管理）
│   ├── migrations/         16 个 SQLite 迁移
│   └── Cargo.toml
├── ui/                     React 前端
│   ├── src/
│   │   ├── components/     UI 组件
│   │   ├── stores/         Zustand 状态管理
│   │   └── services/       API 调用
│   └── package.json
└── docs/                   文档
    ├── STATUS.md           功能完成状态 + 路线图
    ├── ARCHITECTURE.md     架构全流程文档
    ├── BACKEND_TEST.md     后端 API 测试清单
    └── product/
        ├── PRD.md          产品需求文档
        └── UI-DESIGN.md    前端设计规范
```

---

## 数据目录

```
~/.ring/
├── hub/
│   ├── system_prompt.md        Super Ring 行为定义
│   └── user_preferences.md     用户全局偏好
├── rings/
│   └── <ring-id>/
│       ├── graph.json          群组图谱
│       ├── sessions/           Session 数据
│       ├── archives/           归档文件（Git 仓库）
│       └── .group/             Group Ring 知识文档
│           ├── role.md
│           ├── conventions.md
│           ├── active-context.md
│           ├── archive-patterns.md
│           ├── corrections.md
│           └── knowledge-summary.md
├── self/                       私有 AI 数据
│   ├── memory/
│   │   ├── user_profile.md
│   │   ├── preferences.md
│   │   ├── active_goals.md
│   │   └── growth.md
│   └── metrics/
├── skills/                     Skill 插件
└── ring.db                     SQLite 数据库
```

---

## 技术栈

| 层 | 技术 | 说明 |
|----|------|------|
| 后端 | Rust + Axum 0.8 | 异步 HTTP + WebSocket |
| 数据库 | SQLite + sqlx | 本地存储，零配置 |
| 前端 | React 19 + TypeScript + Vite 8 | 现代 React，极速构建 |
| 状态 | Zustand 5 | 轻量状态管理 |
| 可视化 | D3.js v7 | 力导向图 + 树形图 |
| LLM | async-openai | OpenAI / Anthropic / Ollama |
| 实时 | WebSocket + SSE | 多人聊天 + 流式 AI 输出 |
| 分发 | include_dir! | 前端嵌入后端，单二进制 |

---

## 测试

```bash
# 后端测试
cd server && cargo test
# 69/69 通过

# 前端构建验证
cd ui && npm run build

# 手动测试清单
cat docs/BACKEND_TEST.md
```

---

## 版本历史

| 版本 | 日期 | 说明 |
|------|------|------|
| v1.0.0 | 2026-04-28 | 首个稳定版本，PRD 7.2 全部完成 |

### v1.0.0 完整功能清单

**基础设施**：Setup 向导、Auth、LLM 配置、GitLab 配置、隐私过滤
**聊天系统**：Group/Super/Self 三层聊天、SSE 流式、tool_calls、历史分页、Markdown 渲染、长消息折叠、命令补全
**知识图谱**：D3 可视化、CRUD、多图谱、节点树视图、标签过滤、蓝图构建器
**归档系统**：AI 驱动归档、Git commit、PR Review、Git revert、diff 视图
**Session 系统**：全生命周期、WebSocket 实时聊天、材料准备、AI 总结
**协作**：邀请链接、审核流程、成员管理、数据同步、通知系统
**导出**：Markdown / PDF / JSON / tar.gz / SVG / PNG
**AI 工具**：文件解析、知识提取、网页爬取、跨 Ring 搜索

---

## 未来路线图

### v1.1 — 体验优化
- 移动端响应式适配
- Linux / Windows / macOS ARM 多平台二进制
- npm 全局安装
- 自动更新检查

### v1.2 — AI 增强
- Headless Chrome 网页爬取
- 本地 Embedding + 向量语义搜索
- 多步骤 Agent 工作流
- 语音输入

### v1.3 — 企业级
- SSO 集成（OAuth2 / LDAP）
- 审计日志
- 数据库加密
- 集群部署

详见 [docs/STATUS.md](docs/STATUS.md)

---

## 文档

| 文档 | 内容 |
|------|------|
| [STATUS.md](docs/STATUS.md) | 功能状态 + 路线图 |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | 架构全流程 |
| [BACKEND_TEST.md](docs/BACKEND_TEST.md) | 后端 API 测试清单 |
| [PRD.md](docs/product/PRD.md) | 产品需求文档 |
| [UI-DESIGN.md](docs/product/UI-DESIGN.md) | 设计规范 |
| [AGENTS.md](AGENTS.md) | AI 编码指引 |

---

## License

MIT License — 详见 [LICENSE](LICENSE)

---

<div align="center">

**用 AI 把团队对话变成可复用的知识**

</div>
