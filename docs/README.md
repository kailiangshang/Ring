# Ring 文档导航

> 面向公司内网的群组知识协作空间

## 文档结构

```
docs/
├── README.md                         ← 你在这里
├── product/                          # 产品定义（不写技术实现）
│   ├── PRD.md                        # 产品需求文档（含权限、用户流程）
│   └── ai-behavior.md                # 三层 AI 行为设计
│
├── technical/                        # 技术设计（不抄代码）
│   ├── architecture.md               # 技术架构 + 开发者指南
│   ├── api-design.md                 # REST API 设计（标注实现状态）
│   ├── data-model.md                 # SQLite schema + 存储策略
│   ├── knowledge-graph.md            # 知识图谱设计
│   ├── sse-protocol.md               # SSE 流式协议
│   ├── git-integration.md            # Git/GitLab 集成
│   ├── llm-prompts.md                # LLM prompt 模板
│   ├── ring-templates.md             # .ring/ 初始模板
│   ├── implementation-roadmap.md     # 实施阶段规划
│   ├── known-gaps.md                 # 已知缺陷（P0/P1）
│   ├── future-features.md            # 未来功能计划
│   └── test-cases.md                 # 测试用例设计
│
├── api/                              # API 参考（跟代码走）
│   ├── README.md                     # API 参考索引
│   ├── frontend.md                   # 前端：类型 + API + 页面 + Store
│   └── backend.md                    # 后端：全模块参考
│
└── superpowers/                      # AI 辅助开发记录（归档）
    ├── specs/
    └── plans/
```

---

## 依赖关系图

改一个文档时，查 `Affects` 知道哪些文档可能要跟着改。

```
PRD.md (root)
├── ai-behavior.md
│   └── llm-prompts.md
├── architecture.md
│   ├── data-model.md ──→ knowledge-graph.md
│   │                 └──→ git-integration.md ──→ ring-templates.md
│   └── api-design.md ──→ sse-protocol.md
│
├── api/frontend.md (depends on api-design, backend)
├── api/backend.md  (depends on architecture, data-model)
│
├── implementation-roadmap.md
│   ├── known-gaps.md
│   └── future-features.md
└── test-cases.md
```

### 文字版依赖矩阵

| 文档 | Depends on | Affects |
|------|-----------|---------|
| **PRD.md** | — (root) | ai-behavior, api-design, architecture |
| **ai-behavior.md** | PRD | llm-prompts, sse-protocol, backend |
| **architecture.md** | PRD, ai-behavior | data-model, knowledge-graph, api-design, backend, frontend |
| **api-design.md** | PRD, architecture, data-model | frontend, backend, sse-protocol |
| **data-model.md** | PRD, architecture | knowledge-graph, backend, git-integration |
| **knowledge-graph.md** | data-model, PRD, architecture | backend, frontend |
| **sse-protocol.md** | api-design, architecture | frontend, backend |
| **git-integration.md** | PRD, data-model | backend, data-model |
| **llm-prompts.md** | ai-behavior, PRD | backend, ai-behavior |
| **ring-templates.md** | PRD, git-integration | data-model, backend |
| **implementation-roadmap.md** | PRD, architecture | known-gaps, future-features |
| **known-gaps.md** | architecture, backend | roadmap, future-features |
| **future-features.md** | PRD, known-gaps | roadmap |
| **test-cases.md** | PRD, api-design | (test files) |
| **api/frontend.md** | backend, PRD, sse-protocol | frontend code |
| **api/backend.md** | PRD, architecture, knowledge-graph | frontend.md, api-design |

---

## 快速定位

| 我想了解... | 去看 |
|-------------|------|
| 产品是什么 | [PRD.md](product/PRD.md) |
| AI 怎么工作 | [ai-behavior.md](product/ai-behavior.md) → [llm-prompts.md](technical/llm-prompts.md) |
| 技术架构 | [architecture.md](technical/architecture.md) |
| API 端点设计 | [api-design.md](technical/api-design.md) |
| 前端怎么调 API | [api/frontend.md](api/frontend.md) |
| 后端代码结构 | [api/backend.md](api/backend.md) |
| 数据库表结构 | [data-model.md](technical/data-model.md) |
| 知识图谱设计 | [knowledge-graph.md](technical/knowledge-graph.md) |
| 当前还有什么没做 | [known-gaps.md](technical/known-gaps.md) |
| 未来计划做什么 | [future-features.md](technical/future-features.md) |
| 开发进展到哪了 | [implementation-roadmap.md](technical/implementation-roadmap.md) |

---

## 更新规范

每个文档头部有三行元数据：

```
> **Affects**: file1.md · file2.md     ← 改本文件时，这些文件可能也要改
> **Depends on**: file3.md · file4.md   ← 本文件依赖的上游设计
> **Last verified**: 2026-04-11         ← 最后确认内容正确的日期
```

**改文档时**：检查 `Affects` 列表，确认相关文档是否需要同步更新。更新后将本文件的 `Last verified` 改为当天日期。
