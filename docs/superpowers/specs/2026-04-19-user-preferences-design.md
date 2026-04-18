# User Preferences 设计

> **Affects**: `server/src/services/super_chat.rs`, `server/src/routes/super_chat.rs`, `ui/src/stores/chat-store.ts`, `ui/src/services/api.ts`
> **Depends on**: Super Ring Tool Framework（已完成）、`~/.ring/hub/` 目录（已创建）
> **Last verified**: 2026-04-19

## 1. 概述

用户偏好系统，存储用户全局配置（语言、默认 LLM、输出格式、默认模式）。文件存储于 `~/.ring/hub/user_preferences.md`，Markdown 格式，人类可读可编辑。

两种交互方式：
- **Super Ring 对话**：LLM 通过 tool 调用读写偏好
- **CLI 命令**：`%prefs` / `%prefs set <key> <value>` 直接操作

### 1.1 设计决策

| 决策项 | 选择 | 理由 |
|--------|------|------|
| 存储格式 | Markdown | 与 system_prompt.md 一致，人类可读可编辑 |
| 存储位置 | `~/.ring/hub/user_preferences.md` | PRD 定义，与其他 Hub 数据同目录 |
| 交互方式 | Tool + CLI 双通道 | 满足对话式和命令式两种用户习惯 |
| 偏好注入 | 追加到 Super Ring system prompt | LLM 始终知道当前偏好，无需额外调用 |
| 内容范围 | 纯配置偏好，不做用户画像 | 简单明确，YAGNI |

## 2. 文件格式

### 2.1 默认模板

首次访问时（文件不存在），返回此默认内容：

```markdown
## 语言
- default: zh-CN

## LLM
- default_provider: openai

## 输出格式
- style: concise

## 默认模式
- mode: normal
```

### 2.2 格式规范

- 每个偏好类别以 `## 标题` 开头
- 每个偏好项以 `- key: value` 格式
- 用户可自由添加新类别和新偏好项
- Super Ring 通过 `update_user_preferences` tool 覆写整个文件内容

## 3. API 端点

### 3.1 GET /api/super/preferences

读取当前偏好。

**响应**：
```json
{
  "content": "## 语言\n- default: zh-CN\n...",
  "is_custom": true
}
```

`is_custom` 为 `false` 表示文件不存在，返回的是默认模板。

### 3.2 PUT /api/super/preferences

更新偏好。

**请求体**：
```json
{
  "content": "## 语言\n- default: en\n..."
}
```

**响应**：
```json
{
  "ok": true
}
```

空内容时删除文件（恢复默认）。

## 4. Super Ring Tools

### 4.1 query_user_preferences

读取当前用户偏好。

**参数**：无

**返回**：`user_preferences.md` 的完整内容（如果文件不存在，返回默认模板）

**使用场景**：用户问"我的偏好设置是什么？"，LLM 调用此 tool 获取后告知用户。

### 4.2 update_user_preferences

更新用户偏好。

**参数**：
```json
{
  "content": "完整的 Markdown 内容"
}
```

**行为**：用 `content` 覆写 `~/.ring/hub/user_preferences.md`。

**使用场景**：用户说"把我的语言改成英文"，LLM 先读取当前偏好，修改对应字段，再调用此 tool 写回。

**注意**：tool 接收的是完整文件内容，不是增量更新。LLM 负责：
1. 先用 `query_user_preferences` 读取当前内容
2. 修改需要变更的部分
3. 调用 `update_user_preferences` 写回完整内容

## 5. System Prompt 注入

在 `start_super_chat` 中，将偏好内容追加到 system prompt：

```
{base_system_prompt}

{ring_summary}

## 用户偏好
{user_preferences.md 内容}
```

这样 LLM 始终知道当前偏好，大多数情况下不需要调用 `query_user_preferences` tool。

## 6. CLI 命令

### 6.1 %prefs

显示当前偏好设置。前端解析 `%prefs` 命令，调用 `GET /api/super/preferences`，以系统消息形式展示在聊天中。

### 6.2 %prefs set \<key\> \<value\>

修改单个偏好项。前端解析命令后：
1. 调用 `GET /api/super/preferences` 获取当前内容
2. 在当前 Markdown 中找到对应 key，修改 value
3. 调用 `PUT /api/super/preferences` 写回

支持的 key-value 映射：

| CLI key | Markdown 位置 | 示例 |
|---------|-------------|------|
| `language` | `## 语言 / - default:` | `%prefs set language en` |
| `provider` | `## LLM / - default_provider:` | `%prefs set provider ollama` |
| `style` | `## 输出格式 / - style:` | `%prefs set style detailed` |
| `mode` | `## 默认模式 / - mode:` | `%prefs set mode auto` |

对于不在映射中的 key，提示用户通过 Super Ring 对话修改。

## 7. 后端改动

### 7.1 services/super_chat.rs

新增函数：

```rust
pub fn get_user_preferences(hub_dir: &Path) -> String
// 读取 ~/.ring/hub/user_preferences.md，不存在则返回 DEFAULT_PREFERENCES

pub fn update_user_preferences(hub_dir: &Path, content: &str) -> Result<()>
// 写入 ~/.ring/hub/user_preferences.md，空内容则删除文件
```

新增常量：

```rust
const DEFAULT_PREFERENCES: &str = "..."; // 默认模板
```

修改 `get_super_tools()`：
- 新增 `query_user_preferences` 和 `update_user_preferences` 两个 tool 定义

修改 `execute_tool()`：
- 新增两个 tool 的路由分支

修改 `start_super_chat()`：
- system prompt 追加偏好内容

### 7.2 routes/super_chat.rs

新增 2 个 handler：

```rust
pub async fn get_preferences(State(state): State<AppState>) -> Result<Json<Value>>
pub async fn update_preferences(State(state): State<AppState>, Json(body): Json<Value>) -> Result<Json<Value>>
```

在 `routes/mod.rs` 注册路由：
- `GET /api/super/preferences`
- `PUT /api/super/preferences`

## 8. 前端改动

### 8.1 services/api.ts

新增：
```typescript
export async function getPreferences(): Promise<{ content: string; is_custom: boolean }>
export async function updatePreferences(content: string): Promise<void>
```

### 8.2 stores/chat-store.ts

在 CLI 命令解析中新增 `%prefs` 和 `%prefs set` 的处理分支。

`%prefs` 处理：
1. 调用 `getPreferences()`
2. 将内容作为系统消息插入聊天

`%prefs set <key> <value>` 处理：
1. 调用 `getPreferences()` 获取当前内容
2. 解析 Markdown，修改对应 key
3. 调用 `updatePreferences()` 写回
4. 在聊天中显示确认消息

## 9. 错误处理

| 场景 | 处理 |
|------|------|
| 文件不存在 | 返回默认模板，`is_custom: false` |
| 文件写入失败 | 返回 RingError::Io |
| update_user_preferences 收到空 content | 删除文件（恢复默认） |
| CLI key 不在映射中 | 提示用户通过 Super Ring 对话修改 |
| Markdown 解析失败（%prefs set） | 提示用户通过 Super Ring 对话修改 |

## 10. 修改文件清单

| 文件 | 改动 | 说明 |
|------|------|------|
| `server/src/services/super_chat.rs` | 修改 | 新增常量、函数、tool 定义、tool 路由、system prompt 注入 |
| `server/src/routes/super_chat.rs` | 修改 | 新增 2 个 handler |
| `server/src/routes/mod.rs` | 修改 | 注册 2 个新路由 |
| `ui/src/services/api.ts` | 修改 | 新增 2 个 API 函数 |
| `ui/src/stores/chat-store.ts` | 修改 | 新增 %prefs 命令解析 |

无 migration，无新依赖。
