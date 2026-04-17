# Ring Redesign Design

Date: 2026-04-15

## 1. Overview

四层并列架构，Self 独立于三层之外。

```
Ring Hub（用户入口）
├── Super Ring（Hub级）
├── Group Ring（Ring级）
├── Session Ring（Session级）
└── Self（独立层）
```

---

## 2. Layer Definitions

### 2.1 Super Ring（Hub级）

- **定位**：全局助手 + 跨 Ring 协调者
- **数据访问**：按需只读本机所有 Ring 内容
- **存储**：`~/.ring/hub/`（system_prompt.md / user_preferences.md / cross_ring_cache/）
- **职责**：
  - Ring 管理引导
  - 跨 Ring 分析、问答、汇总总结、合并推荐
  - Skill 安装

### 2.2 Group Ring（Ring级）

- **定位**：群组专属 AI
- **数据访问**：读写本 Ring 图谱和归档
- **职责**：
  - 群组 AI 讨论
  - 图谱读写
  - 成员管理

### 2.3 Session Ring（Session级）

- **定位**：多人实时讨论 AI，独立实例（不复用 Group Ring）
- **加载 Skill 决定行为**
- **流程**：
  1. 创建 Session（选择场景类型/Skill + 填写标题描述 + 邀请成员）
  2. 材料准备（Session Ring 基于 Skill 收集/生成材料，`discussion` 模式跳过）
  3. 讨论阶段（所有成员参与，**AI 不参与，只记录**）
  4. 总结（可选，auto 或 manual，用户确认后执行后续操作）
  5. 结束（结束信号必须由 owner 发起）

### 2.4 Self（独立层）

- **定位**：用户私有 AI 宠物，不对外暴露
- **交互方式**：
  - 独立对话页面：Ring Hub 中有 Self 入口，用户主动找它聊天
  - 可召唤：在 Ring 对话中 `@self` 召唤，回答仅召唤者可见
- **数据存储**：`~/.ring/self/`
- **数据内容**：
  - `.self/identity.md` - 身份定义
  - `.self/style.md` - 对话风格
  - `.self/knowledge.md` - 知识结构（用户上传文档抽象而来）
  - `.self/goals.md` - 目标和偏好
  - `.self/growth.md` - 成长记录
  - `metrics/session_stats.json` - Session 参与统计
  - `metrics/tool_usage.json` - 工具调用统计
  - `metrics/dwell_time.json` - 屏幕停留时长
  - `metrics/archive_patterns.json` - 归档行为模式
- **行为**：
  - 信息收集
  - 行为统计
  - 给用户提建议
  - 用户在时作为助手
  - 可配置自主行动边界
- **权限边界**：
  - 完全私有，不进 Git
  - 不被邀请到其他 Group
  - 不参与 Session
  - 不替代用户发言

---

## 3. Skill System

### 3.1 Format

采用 Claude Code Skill 格式（Markdown + YAML frontmatter）。

```
skill-name/
├── SKILL.md           # 主文件（必需）
├── prompts/          # 可选：prompt 模板
│   └── system.md
├── reference.md      # 可选：参考文档
└── scripts/          # 可选：脚本
    └── helper.py
```

### 3.2 SKILL.md Format

```yaml
---
name: skill-name
description: 简短描述（Session 创建时选择 + AI 自动判断加载）
disable-model-invocation: false   # 默认 false，AI 可自动加载
allowed-tools: Bash Read Grep     # 可选：授予免确认工具
---

## Skill 内容
Instructions here...
```

### 3.3 Pre-installed Skills

5 个业务 Skill + `discussion` 默认模式（无 Skill，跳过材料准备和总结）：

- `decision` - 团队决策：收集材料 → 讨论 → 决策结论 + 行动项
- `research` - 联合调研：收集材料 → 讨论 → 调研报告
- `review` - 集体评审：收集材料 → 讨论 → 评审意见 + 改进建议
- `retrospective` - 项目复盘：收集材料 → 讨论 → 经验总结 + 改进计划
- `knowledge_sharing` - 知识分享：收集材料 → 分享 → 整理笔记
- `discussion` - 自由讨论（默认模式，无 Skill）

### 3.4 Permission Modes

| 模式 | 行为 |
|------|------|
| auto | AI 自动执行，无需确认 |
| plan | AI 执行前先展示计划，用户确认 |
| edit | AI 只生成建议，用户手动执行 |

### 3.5 Installation

- 用户在 Super Ring 输入"安装 xxx"
- Super Ring 从网络获取 Skill 定义
- 保存到 `~/.ring/skills/xxx/`
- 用户 refresh 后生效

---

## 4. Session Lifecycle

### 4.1 Steps

1. **创建 Session**
   - 选择场景类型（加载对应 Skill，`discussion` 为无 Skill 默认模式）
   - 填写标题和描述
   - 邀请成员

2. **材料准备**（必需，`discussion` 模式跳过）
   - Session Ring 加载对应 Skill 的 system prompt
   - AI 基于描述收集/生成材料
   - 参与者可查看进度
   - 创建者可标记重点

3. **讨论阶段**
   - 所有成员参与讨论
   - **AI 不参与，只记录消息**

4. **总结**（可选，`discussion` 模式跳过）
   - 可配置 auto 或 manual
   - Session Ring 基于材料 + 讨论内容生成总结报告
   - 用户确认后执行后续操作

5. **结束**
   - 结束信号必须由 owner 发起
   - 聊天记录保留在创建者后端
   - 可随时重新打开

### 4.2 Permission Matrix

| 操作 | Owner | Participant |
|------|-------|-------------|
| 发送消息 | ✅ | ✅ |
| 邀请成员 | ✅ | ❌ |
| 移除成员 | ✅ | ❌ |
| 开关归档 | ✅ | ❌ |
| 触发归档 | ✅ | ❌ |
| 关闭 Session | ✅ | ❌ |
| 删除 Session | ✅ | ❌ |
| 离开 Session | ✅ | ✅ |

---

## 5. Data Storage

```
~/.ring/                              # 用户数据根目录
├── hub/                            # Super Ring 行为定义
├── rings/                          # Group Ring 数据
│   └── <ring-id>/
│       ├── graph.json              # 群组图谱
│       ├── sessions/               # Session Ring 数据
│       │   └── <session-id>/
│       │       └── .session/       # Session 行为定义
│       └── .group/                # Group Ring 行为定义
├── self/                           # Self 数据（私有，不进 Git）
│   ├── .self/
│   │   ├── identity.md
│   │   ├── style.md
│   │   ├── knowledge.md
│   │   ├── goals.md
│   │   └── growth.md
│   └── metrics/
│       ├── session_stats.json
│       ├── tool_usage.json
│       ├── dwell_time.json
│       └── archive_patterns.json
└── skills/                        # Skill 插件
    └── <skill-name>/
        └── SKILL.md
```

---

## 6. Distribution

- 单一安装包（前端 + Rust 后端二进制）
- 后端二进制 serving 前端静态资源
- 用户下载对应平台版本即可使用，无需手动运行后端
- 支持平台：Windows / Linux / macOS (ARM + x86)

---

## 7. Unspecified

以下方面留到实现阶段细化：

- Super Ring 具体功能边界
- Group Ring 具体功能边界
- LLM 集成细节
- 权限系统详细设计
- API 设计
- SQLite 表结构
- 前端 CLI 设计
