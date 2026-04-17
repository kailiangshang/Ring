# Ring - 群组知识协作空间

Ring 是一个面向公司内网的群组知识协作空间。

## 核心概念

### 四层 AI 架构

```
Ring Hub（用户入口）
├── Super Ring（Hub级）    - 全局助手 + 跨 Ring 协调者
├── Group Ring（Ring级）    - 群组专属 AI
├── Session Ring（Session级）- 多人实时讨论 AI
└── Self（独立层）          - 用户私有 AI 宠物
```

### 数据存储

```
~/.ring/                     # 用户数据根目录
├── hub/                     # Super Ring 行为定义
├── rings/                  # Group Ring 数据
│   └── <ring-id>/
│       ├── graph.json       # 群组图谱
│       ├── sessions/       # Session Ring 数据
│       └── .group/         # Group Ring 行为定义
├── self/                    # Self 数据（私有，不进 Git）
└── skills/                  # Skill 插件
```

### Session 生命周期

```
创建 → 材料准备（必需）→ 讨论 → 总结（可选）→ 结束
```

AI 在材料准备阶段收集整理材料，让讨论有内容可依。讨论阶段 AI 不参与，只记录。

### Skill 系统

5 个预装 Skill（Claude Code Skill 格式）：
- `decision` - 团队决策
- `research` - 联合调研
- `review` - 集体评审
- `retrospective` - 项目复盘
- `knowledge_sharing` - 知识分享

---

## 技术栈

- **后端**：Rust + Axum
- **前端**：React + TypeScript + Vite
- **数据库**：SQLite（via sqlx）

## 项目结构

```
src/            # Rust 后端（待实现）
ui/             # React 前端
docs/           # 设计文档
```

## 设计文档

- **[四层架构设计](docs/superpowers/specs/2026-04-15-ring-redesign-design.md)** - 核心架构确认
- **[实现计划](docs/superpowers/plans/)** - 4 个模块化实现计划

## License

MIT