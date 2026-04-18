# CLI Command System Design

Date: 2026-04-17

## 1. Overview

Ring 的聊天输入框同时支持自然语言和结构化命令。四个特殊前缀覆盖全部功能操作：

| 前缀 | 语义 | 一句话 |
|------|------|--------|
| `@` | 寻址 | "对谁说话" |
| `#` | 引用 | "指向什么东西" |
| `!` | 操作 | "做什么事" |
| `%` | 元操作 | "改变怎么运作" |

**设计原则**：
- 命令与自然语言共存，命令是快捷方式不是唯一入口
- 所有命令都有对应的 UI 按钮操作，命令只是加速器
- 输入前缀后弹出自动补全，降低记忆负担
- 命令不区分大小写

---

## 2. `@` — 寻址

指向某个 AI 或人。

### 2.1 命令列表

| 命令 | 效果 | 场景 |
|------|------|------|
| `@self` | 打开 Self 浮窗 | 任何对话中 |
| `@self <消息>` | 召唤 Self 并发送消息，回复仅自己可见 | Ring / Session 对话中 |
| `@ring` | 明确对 Group Ring 说话 | 对话上下文不清晰时 |
| `@super` | 跨 Ring 向 Super Ring 提问 | 任何 Ring 中 |
| `@<username>` | @某个成员 | Session 讨论中 |

### 2.2 自动补全

输入 `@` 后弹出候选列表，按优先级排序：
1. `self`（始终第一）
2. `ring` / `super`（AI 实体）
3. 当前 Ring / Session 的成员列表

### 2.3 权限

- `@self`：所有用户可用
- `@ring`：在 Ring 内且拥有对话权限的用户
- `@super`：所有用户可用
- `@<username>`：Session 中所有参与者，Ring 中有对话权限的成员

---

## 3. `#` — 引用

指向图谱中的实体（节点或标签）。

### 3.1 命令列表

| 命令 | 效果 |
|------|------|
| `#<节点名>` | 引用图谱节点，渲染为可点击链接 |
| `#<标签名>` | 引用标签，用于过滤/搜索 |

### 3.2 典型用法

- `"把这段归档到 #竞品分析 下面"` → AI 理解归档目标节点
- `"帮我看看 #技术方案 的相关内容"` → AI 检索该节点内容
- `"这个和 #Q2计划 有关"` → 建立关联关系

### 3.3 自动补全

输入 `#` 后弹出当前图谱的节点和标签列表。搜索逻辑：
- 模糊匹配节点 label
- 匹配标签名
- 按最近更新时间排序（最近使用的排前面）

### 3.4 渲染

聊天消息中的 `#xxx` 渲染为可点击链接：
- 点击节点引用 → 打开 Graph 面板并高亮该节点
- 点击标签引用 → 打开 Graph 面板并过滤显示该标签的节点

### 3.5 权限

- Ring 内所有角色可用（包括只读）
- Session 中所有参与者可用

---

## 4. `!` — 操作

触发一个具体动作。

### 4.1 面板 / 导航

| 命令 | 效果 |
|------|------|
| `!graph` | 打开 Graph 面板 |
| `!archive` | 打开 Archive 面板 |
| `!config` | 打开 Config 面板（Members + Blueprint） |
| `!session` | 打开 Session 面板 |

### 4.2 归档 / 导出

| 命令 | 效果 |
|------|------|
| `!save` | 触发归档流程（AI 推荐挂载位置） |
| `!export <type>` | 导出，type 取值：`graph` / `md` / `chat` / `report` / `backup` / `json` |
| `!pr` | 查看 PR 审核队列 |
| `!pr approve <id>` | 批准 PR |
| `!pr reject <id>` | 打回 PR |

### 4.3 模式切换

| 命令 | 效果 |
|------|------|
| `!auto` | 切换 Auto 模式开/关 |
| `!compact` | 压缩对话上下文（storage 模式） |
| `!ephemeral` | 切换到临时会话模式 |

### 4.4 Session 管理

| 命令 | 效果 |
|------|------|
| `!session new` | 创建新 Session（弹出 Skill 选择界面） |
| `!session close` | 关闭当前 Session（仅 owner） |
| `!session reopen <id>` | 重新打开已关闭的 Session（仅 owner） |

### 4.5 成员管理

| 命令 | 效果 |
|------|------|
| `!invite` | 生成邀请链接（弹出交互式配置：开放/审核、角色、有效期） |
| `!members` | 查看成员列表 |

### 4.6 权限矩阵

| 命令 | 创建者 | 管理员 | 成员 | 只读 |
|------|--------|--------|------|------|
| `!graph` / `!archive` / `!config` | ✅ | ✅ | ✅ | ✅ |
| `!session` | ✅ | ✅ | ✅ | ❌ |
| `!save` | ✅ | ✅ | ✅ | ❌ |
| `!export` | ✅ | ✅ | ✅ | 仅 graph/json |
| `!auto` | ✅ | ✅ | ✅ | ❌ |
| `!compact` / `!ephemeral` | ✅ | ✅ | ✅ | ❌ |
| `!session new` | ✅ | ✅ | 需授权 | ❌ |
| `!session close` / `!session reopen` | session owner only |
| `!pr` | ✅ | ✅ | 仅自己的 | ❌ |
| `!pr approve` / `!pr reject` | ✅ | ✅ | ❌ | ❌ |
| `!invite` | ✅ | ❌ | ❌ | ❌ |
| `!members` | ✅ | ✅ | ✅ | ✅ |

---

## 5. `%` — 元操作

改变系统行为、配置、Skill 管理。

### 5.1 Skill 管理

| 命令 | 效果 |
|------|------|
| `%skill list` | 列出已安装 Skill |
| `%skill install <name>` | 安装 Skill（通过 Super Ring） |
| `%skill remove <name>` | 卸载 Skill |

### 5.2 Ring 配置

| 命令 | 效果 |
|------|------|
| `%role` | 查看 Group Ring 角色定义（创建者/管理员可编辑） |
| `%conventions` | 查看团队约定（创建者/管理员可编辑） |
| `%blueprint` | 进入蓝图编辑模式（仅创建者） |
| `%mode auto` | 切换到 Auto 模式（AI 自主归档，无需确认） |
| `%mode normal` | 切换回日常对话模式（默认） |
| `%mode skill <auto\|plan\|edit>` | 设置 Skill 执行权限模式（AI 工具调用时的确认级别） |

### 5.3 LLM / 系统

| 命令 | 效果 |
|------|------|
| `%llm` | 查看当前 LLM 配置 |
| `%llm set <provider>` | 切换 LLM 提供商（openai / anthropic / ollama） |

### 5.4 权限矩阵

| 命令 | 创建者 | 管理员 | 成员 | 只读 |
|------|--------|--------|------|------|
| `%skill list` | ✅ | ✅ | ✅ | ✅ |
| `%skill install` / `%skill remove` | ✅ | ❌ | ❌ | ❌ |
| `%role` | 读+写 | 读+写 | 只读 | 只读 |
| `%conventions` | 读+写 | 读+写 | 只读 | 只读 |
| `%blueprint` | ✅ | ❌ | ❌ | ❌ |
| `%mode` | ✅ | ✅ | ✅ | ❌ |
| `%llm` | ✅ | ✅ | ✅ | ❌ |
| `%llm set` | ✅ | ❌ | ❌ | ❌ |

---

## 6. 输入框交互规则

### 6.1 自动补全触发

| 输入 | 触发时机 | 补全内容 |
|------|---------|---------|
| `@` | 立即 | AI 实体 + 成员列表 |
| `#` | 立即 | 图谱节点 + 标签 |
| `!` | 立即 | 命令列表 |
| `%` | 立即 | 元命令列表 |
| `!pr` | 空格后 | `approve` / `reject` + PR 编号 |
| `!session` | 空格后 | `new` / `close` / `reopen` |
| `!export` | 空格后 | 导出类型列表 |
| `%skill` | 空格后 | `list` / `install` / `remove` |
| `%mode` | 空格后 | `auto` / `normal` / `skill` |
| `%mode skill` | 空格后 | `auto` / `plan` / `edit` |
| `%llm set` | 空格后 | `openai` / `anthropic` / `ollama` |

### 6.2 命令 vs 自然语言

输入框同时支持两种输入，判断规则：
- 以 `@`、`#`、`!`、`%` 开头 → 解析为命令
- 其他 → 自然语言发给当前 AI

混合输入：`@self 帮我看看 #竞品分析 里最近的内容` — `@self` 寻址，`#竞品分析` 引用，其余为自然语言。

### 6.3 Command Hints

输入框底部显示可点击的命令提示：
```
 !graph  !archive  !config  !session  @self
```
点击等同于输入该命令。

---

## 7. 模式切换

Ring 有两个独立的模式维度，用户通过命令或 UI 切换。

### 7.1 交互模式（AI 自主权）

控制 Group Ring 的读写权限。

| 模式 | 命令 | AI 权限 | 确认要求 |
|------|------|---------|---------|
| normal（默认） | `%mode normal` 或 `!auto`（切换回） | 只读 — 查询图谱和归档 | — |
| auto | `%mode auto` 或 `!auto`（切换到） | 完全开放 — 自主归档 | 不需要逐个确认 |

手动归档不是一个持久模式，由 `!save` 或说"归档"触发一次性流程。

### 7.2 Skill 权限模式（工具确认级别）

控制 AI 工具调用时需要多少人工确认。

| 模式 | 命令 | 行为 |
|------|------|------|
| auto | `%mode skill auto` | AI 直接执行，无需确认 |
| plan | `%mode skill plan` | AI 先展示计划，用户确认后执行 |
| edit | `%mode skill edit` | AI 只生成建议，用户手动执行 |

### 7.3 UI — 输入框左侧 Mode Indicator

输入框左侧的 `[ring]` indicator 可点击，弹出模式选择器：

```
┌──────────────────────────┐
│ 交互模式                 │
│ ○ 正常对话               │
│ ● Auto                   │
├──────────────────────────┤
│ 工具确认级别             │
│ ○ auto  ● plan  ○ edit   │
└──────────────────────────┘
```

Indicator 显示当前状态：
- 正常对话：`[ring]`
- Auto 模式：`[ring·auto]`，`auto` 用 `accent-amber` 色

---

## 8. 与 UI 按钮的映射

所有命令都有对应的 UI 操作入口，命令只是键盘快捷方式：

| 命令 | UI 入口 |
|------|---------|
| `!graph` / `!archive` / `!config` / `!session` | Header Tab 栏 |
| `@self` | 右下角 🐱 按钮 |
| `!save` | 输入框旁 export 按钮 |
| `!auto` / `%mode auto` / `%mode normal` | 输入框左侧 Mode Indicator 点击 |
| `%mode skill <mode>` | 输入框左侧 Mode Indicator 点击 |
| `!invite` | Config 面板内邀请按钮 |
| `!session new` | 侧栏 Session 创建按钮 |
| `!pr` | Archive 面板内 PR 队列 |

---

## 9. 错误处理

| 场景 | 表现 |
|------|------|
| 无效命令（`!xyz`） | 输入框下方提示："未知命令。输入 ! 查看可用命令" |
| 权限不足 | 系统消息提示："你需要 XX 权限才能执行此操作" |
| `#` 引用不存在的节点 | 提示："未找到节点 'xxx'。按 Enter 创建新节点" |
| `@` 不存在的用户 | 不匹配任何候选时按普通文本处理 |
| 命令参数缺失 | 提示用法，如 `"用法: !export <graph\|md\|chat\|report\|backup\|json>"` |

---

## 10. 上下文感知

命令可用性随当前上下文变化：

| 上下文 | 可用的 `@` | 可用的 `#` | 可用的 `!` |
|--------|-----------|-----------|-----------|
| Super Ring | `@self` | 无（无图谱） | `!export` / `!config` |
| Group Ring | `@self` / `@ring` / `@super` / `@<member>` | 当前 Ring 图谱节点和标签 | 全部 |
| Session | `@self` / `@ring` / `@<participant>` | 继承 Group Ring 图谱 | `!save` / `!session` / `!export` |
| Self 浮窗 | 无（直接对话） | 无 | 无 |
