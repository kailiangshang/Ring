# Ring 实施路线图

## 1. 总体规划

完整多人版一步到位，不分阶段简化。按功能模块划分为 6 个实施阶段，每个阶段可独立交付和验证。

---

## 2. 阶段划分

### 阶段 1：基础框架

**目标**：搭建前后端框架，实现 Ring 的基本 CRUD。

 FalkorDB 作为本地图谱存储引擎。

（注：FalkorDB 已替换为 petgraph 内存图，见技术决策表）

**范围**：
- Rust + Axum 后端框架搭建
- React + TypeScript 前端框架搭建
- SQLite 数据库初始化和迁移系统
- petgraph 内存图引擎集成
- Ring Hub 页面（Ring 列表、创建 Ring 表单）
- Ring 基础 CRUD API
- 基础路由和页面布局

**交付物**：
- 可运行的 Ring Hub，能创建和查看 Ring
- 完整的开发环境配置

---

### 阶段 2：AI 对话与蓝图

**目标**：实现 三层 AI 对话（Super Ring + Group Ring + Session Ring）和蓝图构建流程。

**范围**：
- LLM API 集成（OpenAI / Anthropic / Ollama）
- 流式响应（SSE）
- Super Ring 对话
- Group Ring 对话（含 system prompt 注入）
- 蓝图模板管理
- 蓝图构建向导（Group Ring 多轮对话，特殊 prompt）
- 蓝图预览和确认
- 图谱数据模型和基础 CRUD

**交付物**：
- 可在 Ring 内与 Group Ring对话
- 可通过向导构建蓝图并确认

---

### 阶段 3：知识图谱

**目标**：实现知识图谱的创建、编辑和可视化。

**范围**：
- 图谱节点 CRUD（创建、编辑、删除）
- 图谱边 CRUD
- 节点层级管理（父子关系）
- 多图谱管理
- D3.js 力导向图可视化
- 节点树导航组件
- 图谱交互（拖拽、缩放、搜索）
- Markdown 文件生成和关联
- 图谱修正功能（对话修正、Git 回滚）

**交付物**：
- 可视化的知识图谱
- 节点点击显示 Markdown 内容
- 图谱编辑功能
- 所有图谱修改持久化到 graph.json 并同步到 petgraph 内存图

---

### 阶段 4：Git 集成

**目标**：实现 GitLab 仓库关联和版本管理。

**范围**：
- Git 仓库关联（已有仓库 / 自动创建）
- 凭证管理（加密存储）
- git2 集成（clone、pull、push、commit）
- GitLab API 集成（创建/合并/关闭 MR、获取 Diff）
- 归档流程（创建者直接 commit、成员提交 PR）
- PR 列表和通知
- Diff 查看界面（Monaco Editor / CodeMirror）
- PR 审核（合并/拒绝）
- 成员加入时自动 clone
- 实时同步（合并后自动 pull）

**交付物**：
- 完整的 Git 协作流程
- PR 审核界面
- Diff 查看界面

---

### 阶段 5：协作与权限

**目标**：实现多人协作、权限管理和实时通信。

**范围**：
- 用户系统（本地用户管理）
- 邀请机制（生成邀请链接、验证 token）
- 成员管理（角色分配、移除成员）
- 权限校验中间件（角色 + 模式）
- 三模式切换（日常对话 / 手动归档 / Auto）
- WebSocket 实时通信
  - 对话消息实时推送
  - 图谱变更广播
  - PR 状态通知
- Export 按钮（对话片段标记归档）

**交付物**：
- 多人可同时在线协作
- 权限正确控制
- 实时消息推送

---

### 阶段 6：工具引擎与打磨

**目标**：实现原子工具引擎、预设工作流和整体打磨。

**范围**：
- 原子工具引擎框架
  - 工具注册表
  - 工具调度器
  - 权限检查
- 原子工具实现
  - 文件解析（PDF/Markdown/Docx）
  - 文本清洗
  - 结构化提取（LLM）
  - 全文搜索
  - 网页爬取
  - Markdown 生成
  - 隐私过滤
- 预设工作流
  - 会议归档
  - 学习中心
  - 深度调研
- 工具栏 UI
- AI 主动触发规则
  - 归档推荐
  - PR 提醒
  - 图谱总结
  - 空图谱引导
- 全局设置页面
- 性能优化
- 错误处理和用户反馈
- 整体 UI 打磨

**交付物**：
- 完整可用的 Ring 系统
- 三个预设工作流可用
- AI 主动介入功能正常

---

## 3. 技术依赖关系

```
阶段 1（基础框架）
  ├── 阶段 2（AI 对话与蓝图）
  │     └── 阶段 3（知识图谱）
  │           └── 阶段 4（Git 集成）
  │                 └── 阶段 5（协作与权限）
  │                       └── 阶段 6（工具引擎与打磨）
  └── 阶段 5（部分可并行：用户系统）
```

阶段 5 的用户系统可以和阶段 2-4 并行开发。其他阶段有严格的先后依赖。

---

## 4. 关键技术决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 后端框架 | Axum | Rust 生态中成熟的异步 Web 框架 |
| 数据库 | SQLite（sqlx） | 本地优先，无需额外服务。WAL 模式支持多读单写 |
| 图数据引擎 | petgraph `StableDiGraph` | 纯 Rust 进程内嵌，零外部依赖。几百节点规模微秒级操作 |
| Git 操作 | git2 crate | Rust 原生 Git 库，支持 SSH/PAT 凭证 |
| 图谱可视化 | D3.js | 成熟的力导向图实现，可定制性强 |
| Diff 渲染 | Monaco Editor | VS Code 同款，Diff 渲染成熟 |
| 前端框架 | React + TypeScript | 生态成熟，类型安全 |
| AI 流式响应 | SSE | 比 WebSocket 更简单，单向流式够用 |
| 实时通信 | WebSocket | 双向通信，状态同步和消息推送 |
| LLM 客户端 | async-openai + 自建 Anthropic 适配层 | OpenAI/Ollama 共用 async-openai，Anthropic 用 reqwest 自写 |
| 中文搜索 | SQLite FTS5 + jieba-rs | 预分词后空格拼接插入 FTS5，MVP 够用 |
| 语义搜索（后期） | fastembed（BGEM3）+ hnsw_rs | 本地 embedding 推理 + HNSW 向量索引 |
| 前端连接 | 始终连 localhost:7420 | 无 CORS，读操作零延迟 |
| 传输加密 | MVP 不做，后续加 TLS | 内网环境，HTTP 明文先行 |

---

## 5. 测试策略

### 5.1 Rust 后端测试

| 层级 | 工具 | 覆盖范围 |
|------|------|---------|
| 单元测试 | `#[test]` + `tokio::test` | Repository trait 实现、业务逻辑、数据转换 |
| 集成测试 | `tests/` 目录 | API 端到端测试（Axum test router） |
| GraphStore 测试 | 内存 mock 实现 | petgraph 交互逻辑 |

### 5.2 前端测试

| 层级 | 工具 | 覆盖范围 |
|------|------|---------|
| 单元测试 | Vitest | 工具函数、状态管理、数据转换 |
| 组件测试 | React Testing Library | UI 组件交互和渲染 |
| E2E 测试 | Playwright | 关键用户流程（创建 Ring、对话、归档） |

### 5.3 交付要求

每个 Phase 交付时：
- 核心功能的单元测试覆盖
- 至少 1 个集成测试覆盖主流程
- `cargo test` 和 `npm test` 全部通过
- 不要求 100% 覆盖率，但关键路径必须有测试
