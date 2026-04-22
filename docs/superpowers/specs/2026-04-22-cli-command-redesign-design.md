# Ring CLI 命令系统重设计

> 日期：2026-04-22
> 状态：设计完成，待实现

## 问题陈述

当前命令系统存在以下问题：

1. **四种前缀**（`@ # ! %`）+ `/` 映射，记忆成本高
2. **命令与消息混合**，`@self 消息内容` 到底是命令还是消息边界模糊
3. **无参数提示** — 用户不知道 `/node` 需要什么参数
4. **命令平铺无分类** — 所有命令混在一起，上下文感知弱
5. **无命令历史** — 无法快速重复操作

## 设计原则

1. **两个前缀，职责清晰**
   - `/` = 环境操作（让系统执行动作）
   - `@` = 内容交互（与实体对话/引用）

2. **命令即动作，寻址即交互**
   - `/` 改变 UI 状态、创建资源、切换模式
   - `@` 与知识、人、AI 对话

3. **参数化、可补全**
   - 所有命令支持参数和 Tab 补全
   - `/help [command]` 查看详细用法

4. **无需向后兼容**
   - 直接替换旧命令系统，不保留 `!` `%` `#` 前缀

## 架构

```
输入 → Parser → 命令类型判断
         ↓
    ┌────┴────┐
    ↓         ↓
  /-命令    @-寻址
    ↓         ↓
 执行动作   路由消息
```

## 命令清单

### `/`-命令：环境操作

| 分类 | 命令 | 参数 | 说明 |
|------|------|------|------|
| **面板** | `/graph` | - | 打开图谱面板 |
| | `/archive` | - | 打开归档面板 |
| | `/config` | - | 打开配置面板 |
| | `/session` | - | 打开 Session 面板 |
| | `/skills` | - | 打开 Skill 面板（Super） |
| | `/settings` | - | 打开设置面板 |
| **Ring** | `/new [name]` | `name`: Ring 名称 | 创建新 Ring |
| | `/save [content]` | `content`: 归档内容 | 归档对话 |
| | `/invite open` | - | 创建开放邀请 |
| | `/invite audit` | - | 创建审核邀请 |
| | `/members` | - | 成员列表 |
| **Session** | `/session create [title]` | `title`: Session 标题 | 创建 Session |
| | `/session close` | - | 关闭 Session |
| | `/session start` | - | 开始讨论 |
| | `/session summarize` | - | AI 总结 |
| **图谱** | `/node add [name]` | `name`: 节点名称 | 添加节点 |
| | `/node link [from] [to]` | `from`, `to`: 节点名 | 连接节点 |
| **配置** | `/mode [auto/plan/edit]` | `mode`: 模式名 | 切换交互模式 |
| | `/prefs` | - | 显示偏好 |
| | `/prefs set [key] [value]` | `key`, `value` | 设置偏好 |
| | `/skill list` | - | 列出 Skill |
| | `/skill install [url]` | `url`: Skill URL | 安装 Skill |
| | `/skill remove [name]` | `name`: Skill 名 | 移除 Skill |
| **帮助** | `/help` | - | 命令列表 |
| | `/help [command]` | `command`: 命令名 | 命令详情 |
| **未来预留** | `/export [format]` | `format`: 导出格式 | 导出中心 |
| | `/blueprint [name]` | `name`: 模板名 | 蓝图/模板 |

### `@`-寻址：内容交互

| 命令 | 参数 | 说明 |
|------|------|------|
| `@self [message]` | `message`: 消息内容 | 与 Self AI 对话 |
| `@ring [message]` | `message`: 消息内容 | 与当前 Ring AI 对话（显式） |
| `@super [message]` | `message`: 消息内容 | 与 Super Ring 对话 |
| `@node [name]` | `name`: 节点名 | 引用/聚焦到节点 |
| `@user [name]` | `name`: 用户名 | 提及用户（未来） |

## 废弃映射

| 旧命令 | 新命令 | 说明 |
|--------|--------|------|
| `!graph` | `/graph` | 动作命令统一用 `/` |
| `!archive` | `/archive` | |
| `!config` | `/config` | |
| `!session` | `/session` | |
| `!new [name]` | `/new [name]` | |
| `!save [content]` | `/save [content]` | |
| `!node [name]` | `/node add [name]` | |
| `!auto` | `/mode auto` | 更明确的语义 |
| `%prefs` | `/prefs` | 元命令统一用 `/` |
| `%skill` | `/skill` | |
| `%mode` | `/mode` | |
| `#节点名` | `@node 节点名` | 引用归到 `@` |

## 命令解析规则

```
输入字符串
  ↓
是否以 `/` 开头？
  → 是：解析为命令
    → 提取命令名（第一个空格前）
    → 提取参数（剩余部分）
    → 查找命令处理器
  → 否：是否以 `@` 开头？
    → 是：解析为寻址
      → 提取目标（`self`/`ring`/`super`/`node`/`user`）
      → 提取消息内容
      → 路由到对应实体
    → 否：普通消息，发给当前上下文 AI
```

## 补全系统

**触发方式：**
- 输入 `/` 显示所有可用命令（按上下文过滤）
- 输入 `/s` 过滤以 `s` 开头的命令
- 输入 `/session ` 显示 Session 子命令（`create`, `close`, `start`, `summarize`）
- 输入 `@` 显示所有寻址目标

**上下文过滤：**
- **Super Ring**：只显示 `/skills`, `/settings`, `/prefs`, `/help`, `@self`, `@ring`
- **Group Ring**：显示所有 Ring 相关命令
- **Session**：显示 Session 相关命令 + 通用命令

## 命令历史

- 按 `↑` 键浏览历史命令（仅命令，不含普通消息）
- 历史保存在前端内存中（不持久化）
- 最多保存 50 条

## 帮助系统

- `/help`：显示分类命令表（按当前上下文过滤）
- `/help [command]`：显示命令详细用法、参数说明、示例
- 每个命令在补全列表中显示简短描述

## UI 改动

1. **输入框提示**：从 `"message / command..."` 改为 `"Type / for commands, @ to address"`
2. **命令提示栏**：当前上下文可用的命令快捷提示（底部）
3. **补全弹出框**：支持子命令和参数提示

## 实现文件

| 文件 | 改动 |
|------|------|
| `ui/src/services/command-parser.ts` | 重写解析器，支持 `/` 和 `@` |
| `ui/src/components/chat/CommandAutocomplete.tsx` | 支持子命令补全、上下文过滤 |
| `ui/src/components/chat/CommandHints.tsx` | 更新提示命令 |
| `ui/src/components/chat/InputArea.tsx` | 命令历史（↑键） |
| `ui/src/stores/chat-store.ts` | 更新命令处理逻辑 |

## 测试计划

1. 单元测试：解析器覆盖所有命令和边界情况
2. 集成测试：补全系统上下文过滤
3. 手动测试：验证所有命令在实际场景中工作

## 验收标准

- [ ] 所有旧命令在新系统中有对应映射
- [ ] `/` 和 `@` 补全正常工作
- [ ] 上下文过滤正确（Super/Ring/Session 显示不同命令）
- [ ] 命令历史（↑键）可用
- [ ] `/help` 显示完整命令表
- [ ] 帮助文档 `/help [command]` 显示参数说明
- [ ] 无 TypeScript 错误，构建通过
