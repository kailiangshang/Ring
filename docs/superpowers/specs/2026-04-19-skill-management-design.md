# Skill Management 设计

> **Affects**: `server/src/services/skill.rs`, `server/src/services/super_chat.rs`, `server/src/routes/skills.rs` (新建), `server/src/routes/mod.rs`, `server/src/main.rs`, `ui/src/services/api.ts`, `ui/src/services/command-parser.ts`, `ui/src/stores/chat-store.ts`
> **Depends on**: Super Ring Tool Framework（已完成）、`~/.ring/skills/` 目录
> **Last verified**: 2026-04-19

## 1. 概述

Skill 管理系统。支持列出内置 Skill、从远程 URL 安装 Skill、卸载 Skill。交互方式为 Super Ring tool + CLI `%skill` 命令双通道。

Skill 格式为 Claude Code Skill 格式：YAML frontmatter + Markdown body，存储为 `~/.ring/skills/{skill_name}/SKILL.md`。

### 1.1 设计决策

| 决策项 | 选择 | 理由 |
|--------|------|------|
| Skill 格式 | YAML frontmatter + Markdown body（Claude Code 格式） | 生态成熟，用户可直接复用现有 Skill |
| 存储结构 | `~/.ring/skills/{name}/SKILL.md` 目录结构 | 支持附加资源文件 |
| 安装来源 | Git 仓库 URL + 单文件 URL 均支持 | 最大兼容性 |
| 下载方式 | Git clone（仓库）+ HTTP GET（单文件） | 复用现有 git_service，可靠 |
| 交互方式 | Tool + CLI 双通道 | 与 User Preferences 一致 |
| 错误告警 | 安装失败返回详细错误信息 | 用户要求 |

## 2. Skill 格式

### 2.1 SKILL.md 结构

```markdown
---
name: decision
description: 团队决策：收集材料 → 讨论 → 决策结论 + 行动项
version: 1.0.0
---

# Decision Skill

You are assisting a decision-making session...
```

YAML frontmatter 字段：

| 字段 | 必填 | 说明 |
|------|------|------|
| `name` | 是 | Skill 名称，小写字母+下划线+连字符 |
| `description` | 是 | 一句话描述 |
| `version` | 否 | 版本号 |

Markdown body 为 Skill 的完整内容（system prompt + 行为定义）。

### 2.2 内置 Skill 自动导出

5 个内置 Skill（decision, research, review, retrospective, knowledge_sharing）在首次被文件系统查询时自动导出到 `~/.ring/skills/{name}/SKILL.md`。导出后用户可自定义，文件版本优先于内置版本。

## 3. API 端点

### 3.1 GET /api/skills

列出所有 Skill（内置 + 用户安装）。

**响应**：
```json
{
  "skills": [
    {
      "name": "decision",
      "description": "团队决策：收集材料 → 讨论 → 决策结论 + 行动项",
      "source": "builtin",
      "installed_at": null
    },
    {
      "name": "custom-skill",
      "description": "自定义 Skill 描述",
      "source": "user",
      "installed_at": "2026-04-19T08:00:00Z"
    }
  ]
}
```

### 3.2 POST /api/skills/install

从远程 URL 安装 Skill。

**请求体**：
```json
{
  "name": "custom-skill",
  "source_url": "https://github.com/user/skills/tree/main/custom-skill"
}
```

**URL 类型自动判断**：
- 以 `.md` 结尾 → 单文件下载（HTTP GET）
- Git 仓库/目录 URL → Git clone 后提取 skill 目录

**处理流程**：
1. 判断 URL 类型
2. 下载 Skill 内容到临时目录
3. 验证 SKILL.md 格式（必须有 YAML frontmatter，必须包含 name + description）
4. 如果 name 与请求中的 name 不一致，使用 SKILL.md 中的 name
5. 移动到 `~/.ring/skills/{name}/`
6. 返回安装结果

**成功响应**：
```json
{
  "ok": true,
  "name": "custom-skill",
  "description": "自定义 Skill 描述"
}
```

**失败响应**（下载/验证失败）：
```json
{
  "ok": false,
  "error": "下载失败: 连接超时",
  "detail": "https://github.com/user/skills/tree/main/custom-skill"
}
```

### 3.3 GET /api/skills/{name}

读取指定 Skill 的完整 SKILL.md 内容。

**响应**：
```json
{
  "name": "decision",
  "description": "...",
  "source": "builtin",
  "content": "---\nname: decision\n...\n---\n\n# Decision Skill\n..."
}
```

### 3.4 DELETE /api/skills/{name}

卸载用户安装的 Skill。

**约束**：
- 内置 Skill（source=builtin）不可卸载
- 只能卸载 `~/.ring/skills/{name}/` 中存在且非内置导出的 Skill

**响应**：
```json
{
  "ok": true,
  "name": "custom-skill"
}
```

内置 Skill 尝试卸载时返回 400：
```json
{
  "error": "Cannot remove built-in skill"
}
```

## 4. Super Ring Tool

### 4.1 manage_skills

统一 Skill 管理 tool。

**参数**：
```json
{
  "action": "list" | "install" | "remove",
  "name": "skill-name",
  "source_url": "https://..."
}
```

**action=list**：无需 name 和 source_url。返回所有 Skill 列表。

**action=install**：需要 name 和 source_url。安装远程 Skill。

**action=remove**：需要 name。卸载 Skill。

### 4.2 使用场景

用户："有什么可用的 Skill？" → LLM 调用 `manage_skills(action=list)`
用户："安装 xxx skill，地址是 https://..." → LLM 调用 `manage_skills(action=install, name=xxx, source_url=https://...)`
用户："卸载 xxx skill" → LLM 调用 `manage_skills(action=remove, name=xxx)`

## 5. Skill Service 改动

### 5.1 现有 skill.rs 扩展

```rust
pub fn get_skill_resolved(name: &str, skills_dir: &Path) -> Option<ResolvedSkill>
// 1. 检查 ~/.ring/skills/{name}/SKILL.md
// 2. 如果文件存在，解析返回（source = "user" 或 "builtin_exported"）
// 3. 如果不存在，检查内置定义
//    - 内置存在：导出到文件系统，返回（source = "builtin"）
//    - 内置不存在：返回 None

pub struct ResolvedSkill {
    pub name: String,
    pub description: String,
    pub source: String,  // "builtin" | "user"
    pub content: String, // 完整 SKILL.md 内容
    pub installed_at: Option<String>,
}

pub fn list_skills(skills_dir: &Path) -> Vec<SkillInfo>
// 1. 扫描 ~/.ring/skills/ 目录下所有子目录
// 2. 对每个包含 SKILL.md 的目录，解析 frontmatter
// 3. 加上内置 Skill（如果文件系统没有对应目录）
// 4. 返回合并列表

pub fn install_skill(skills_dir: &Path, name: &str, source_url: &str) -> Result<InstallResult>
// 1. 判断 URL 类型
// 2. 下载到临时目录
// 3. 验证 SKILL.md 格式
// 4. 移动到 ~/.ring/skills/{name}/
// 5. 返回安装结果

pub fn remove_skill(skills_dir: &Path, name: &str) -> Result<()>
// 1. 检查是否为内置 Skill
// 2. 删除 ~/.ring/skills/{name}/ 目录
```

### 5.2 内置 Skill 导出

内置 Skill 首次被 `get_skill_resolved` 查询时自动导出：

```rust
fn export_builtin_skill(name: &str, skills_dir: &Path) -> std::io::Result<()>
// 从内置常量生成 SKILL.md（含 YAML frontmatter）
// 写入 ~/.ring/skills/{name}/SKILL.md
```

### 5.3 URL 下载

```rust
async fn download_skill_from_url(url: &str) -> Result<String>
// 判断 URL 类型：
// - 以 .md 结尾 → reqwest GET 下载文件内容
// - Git URL → git clone 到临时目录，查找 SKILL.md
// 返回 SKILL.md 的完整内容
```

## 6. CLI 命令

### 6.1 %skill list

显示所有已安装 Skill。前端解析后调用 `GET /api/skills`，以系统消息展示。

### 6.2 %skill install \<name\> \<url\>

从 URL 安装 Skill。前端解析后调用 `POST /api/skills/install`。

### 6.3 %skill remove \<name\>

卸载 Skill。前端解析后调用 `DELETE /api/skills/{name}`。

### 6.4 命令解析

在 command-parser.ts 中，`%skill` 作为 `%` 前缀命令的特殊分支处理：
- `%skill` → `{ type: 'skill', subcommand: 'show' }`（同 list）
- `%skill list` → `{ type: 'skill', subcommand: 'list' }`
- `%skill install <name> <url>` → `{ type: 'skill', subcommand: 'install', name, url }`
- `%skill remove <name>` → `{ type: 'skill', subcommand: 'remove', name }`

## 7. 后端改动

### 7.1 新建 routes/skills.rs

4 个 handler：
- `list_skills` — GET /api/skills
- `install_skill_handler` — POST /api/skills/install
- `get_skill_detail` — GET /api/skills/{name}
- `remove_skill` — DELETE /api/skills/{name}

### 7.2 修改 routes/mod.rs

注册 4 个新路由。

### 7.3 修改 main.rs

启动时创建 `~/.ring/skills/` 目录。

### 7.4 修改 services/skill.rs

扩展 skill service（见第 5 节）。

### 7.5 修改 services/super_chat.rs

在 `get_super_tools()` 中添加 `manage_skills` tool。在 `execute_tool()` 中添加路由。

### 7.6 Cargo.toml

添加 `reqwest` 依赖（用于 HTTP 下载 Skill）。如果 git_service 已使用 git2 或 shell git，则 HTTP 下载用 reqwest。

## 8. 前端改动

### 8.1 services/api.ts

新增：
```typescript
export async function listSkills(): Promise<{ skills: SkillInfo[] }>
export async function installSkill(name: string, sourceUrl: string): Promise<InstallResult>
export async function getSkillDetail(name: string): Promise<SkillDetail>
export async function removeSkill(name: string): Promise<void>
```

### 8.2 services/command-parser.ts

`ParsedCommand` 新增 `skill` 变体。`parseCommand` 处理 `%skill` 前缀。

### 8.3 stores/chat-store.ts

switch 中新增 `skill` case，处理 list/install/remove。

## 9. 错误处理

| 场景 | 处理 |
|------|------|
| 下载 URL 连接失败 | 返回详细错误（连接超时 / DNS 解析失败 / 404 等） |
| SKILL.md 格式无效 | 返回 "Invalid SKILL.md format: missing YAML frontmatter" |
| SKILL.md 缺少 name | 返回 "Invalid SKILL.md: missing required field 'name'" |
| SKILL.md 缺少 description | 返回 "Invalid SKILL.md: missing required field 'description'" |
| name 与内置 Skill 冲突 | 覆盖内置导出文件，source 标记为 user |
| 卸载内置 Skill | 返回 400 "Cannot remove built-in skill" |
| 卸载不存在的 Skill | 返回 404 |
| Git clone 失败 | 返回详细 git 错误 |
| 目标目录已存在（同名 Skill） | 覆盖（先删除旧目录再移动新内容） |

## 10. 修改文件清单

| 文件 | 改动 | 说明 |
|------|------|------|
| `server/src/services/skill.rs` | 重大修改 | 新增 ResolvedSkill, list_skills, install_skill, remove_skill, export_builtin_skill |
| `server/src/services/super_chat.rs` | 修改 | 新增 manage_skills tool 定义 + 路由 |
| `server/src/routes/skills.rs` | 新建 | 4 个 API handler |
| `server/src/routes/mod.rs` | 修改 | 注册 4 个新路由 |
| `server/src/main.rs` | 修改 | 创建 ~/.ring/skills/ 目录 |
| `server/Cargo.toml` | 修改 | 添加 reqwest 依赖 |
| `server/tests/integration.rs` | 修改 | 新增 Skill API 测试 |
| `ui/src/services/api.ts` | 修改 | 新增 4 个 API 函数 |
| `ui/src/services/command-parser.ts` | 修改 | 新增 skill 命令类型 |
| `ui/src/stores/chat-store.ts` | 修改 | 新增 skill 命令处理 |
| `ui/src/types/skill.ts` | 新建 | Skill 相关 TypeScript 类型 |

## 11. 依赖

- `reqwest` — HTTP 客户端，用于下载远程 Skill 文件
- 已有 `git_service` — Git 操作，用于 clone 仓库类 Skill URL
