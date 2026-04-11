# AGENTS.md — Ring 项目 AI 编码指引

## 项目概述

Ring 是一个面向公司内网的群组知识协作空间。Rust + Axum 后端，React + TypeScript 前端。

## 技术栈

- **后端**：Rust + Axum 0.8 + SQLite（sqlx）+ petgraph 内存图 + git2
- **前端**：React + TypeScript + D3.js + Zustand
- **LLM**：async-openai（OpenAI + Ollama）+ 自建 Anthropic 适配层
- **搜索**：SQLite FTS5 + jieba-rs（MVP）

## 代码规范

- 全栈 `snake_case`：Rust 函数/变量、TypeScript 函数/变量、JSON 字段、API 路径
- Rust：`cargo fmt` + `cargo clippy` 必须通过
- TypeScript：ESLint + Prettier
- 不加注释（除非用户要求）
- 错误处理：使用 `crate::error::RingError`，不造新错误类型

## 架构约束

- **handlers 不写业务逻辑**：handler 只做参数解析 → 调 service → 返回响应
- **所有业务逻辑在 services 层**
- **图数据**：petgraph 内存图（`Arc<RwLock<dyn GraphStore>>`），不依赖外部图数据库
- **graph.json 是持久化格式**，petgraph 是运行时查询引擎，不做三方同步
- **LLM 适配层**：统一 `LlmProvider` trait，OpenAI/Ollama 用 async-openai，Anthropic 用 reqwest
- **前端始终连 localhost:7420**，不直接连创建者后端
- **.ring/ 文档**：只有创建者和管理员可写入，成员只读
- **安装导航页去中心化**：由分享链接的用户的 ring-server 服务，独立 HTML 页面嵌入二进制，下载链接指向 GitHub Releases
- **平台支持**：Windows / Linux (WSL) / macOS (Apple Silicon + Intel)

## 关键文件位置

| 内容 | 路径 |
|------|------|
| 文档导航 | `docs/README.md` |
| 产品需求 | `docs/product/PRD.md`（含权限、用户流程） |
| AI 行为设计 | `docs/product/ai-behavior.md` |
| 技术架构 + 开发者指南 | `docs/technical/architecture.md` |
| 数据模型 | `docs/technical/data-model.md` |
| API 设计 | `docs/technical/api-design.md` |
| 前端 API 参考 | `docs/api/frontend.md` |
| 后端 API 参考 | `docs/api/backend.md` |
| SQLite 迁移 | `docs/technical/architecture.md` 第 5 节 |
| 错误类型 | `docs/technical/architecture.md` 第 4.4 节 |
| 路由注册 | `docs/technical/architecture.md` 第 6 节 |
| 知识图谱设计 | `docs/technical/knowledge-graph.md` |
| LLM prompt 模板 | `docs/technical/llm-prompts.md` |
| .ring/ 初始模板 | `docs/technical/ring-templates.md` |
| 已知缺陷 | `docs/technical/known-gaps.md` |

## 测试

```bash
cargo test                    # Rust 单元 + 集成测试
cd ring-frontend && npm test  # 前端测试
```

## 实施阶段

当前在 **Phase 1（基础框架）**。详见 `docs/technical/implementation-roadmap.md`。

## 命名约定

- 代码中 `Ring` = 群组空间（与文档一致）
- 产品整体 = `ring-server`（二进制名）
- 用户数据目录 = `~/.ring/`
- 产品源码仓库 ≠ 用户 Ring（群组）的 GitLab 数据仓库
