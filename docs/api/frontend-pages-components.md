# 前端页面和组件 API 参考

> 源码路径：`ring-frontend/src/pages/`、`ring-frontend/src/components/`

## 页面路由

源文件：`App.tsx`

| 路径 | 组件 | 说明 |
|------|------|------|
| `/setup` | `SetupWizard` | 初始化向导（用户名 → LLM → GitLab → 完成） |
| `/hub` | `RingHub` | Ring 列表 + 创建 |
| `/ring/:ringId` | `RingSpace` | Ring 空间主页（重定向到 chat） |
| `/ring/:ringId/chat` | `ChatView` | 对话视图 |
| `/ring/:ringId/graph` | `GraphView` | 图谱视图 |
| `/ring/:ringId/blueprint` | `BlueprintWizard` | 蓝图构建向导 |
| `/ring/:ringId/members` | `MemberList` | 成员管理 |
| `/ring/:ringId/sessions` | `SessionView` | Session 列表 |
| `/ring/:ringId/prs` | `PrList` | PR 列表 |
| `/ring/:ringId/prs/:prId` | `PrDetail` | PR 详情 + Diff |
| `/settings` | `SettingsPage` | 设置页面 |

---

## Setup Wizard

源文件：`pages/Setup/SetupWizard.tsx`

步骤流程：
1. `StepUsername` — 输入用户名
2. `StepLlm` — 配置 LLM（provider/model/api_key/base_url）
3. `StepGitlab` — 配置 GitLab
4. 完成 → 跳转到 `/hub`

Props：children 渲染当前步骤组件。

### StepUsername

| Props | 类型 | 说明 |
|-------|------|------|
| `onNext` | `(name: string) => void` | 下一步 |

### StepLlm

| Props | 类型 | 说明 |
|-------|------|------|
| `onNext` | `(config: LlmConfig) => void` | 下一步 |
| `onBack` | `() => void` | 上一步 |

### StepGitlab

| Props | 类型 | 说明 |
|-------|------|------|
| `onComplete` | `(config: GitlabConfig) => void` | 完成 |
| `onBack` | `() => void` | 上一步 |

---

## RingHub

源文件：`pages/RingHub/RingHub.tsx`

包含 `RingList` + `CreateRing` + `SuperRingChat`。

### RingList

源文件：`pages/RingHub/RingList.tsx`

Props：渲染 Ring 列表（卡片），点击进入 RingSpace。

### CreateRing

源文件：`pages/RingHub/CreateRing.tsx`

Props：创建 Ring 表单。

### SuperRingChat

源文件：`pages/RingHub/SuperRingChat.tsx`

Props：Super Ring 全局对话界面（调用 `super_ring_chat` API，解析 SSE）。

---

## ChatView

源文件：`pages/RingSpace/ChatView.tsx`

Props：`ringId: string`

包含对话列表 + ChatInput。

**ChatInput**

| Props | 类型 | 说明 |
|-------|------|------|
| `onSend` | `(content: string) => void` | 发送消息 |

**ChatBubble**

| Props | 类型 | 说明 |
|-------|------|------|
| `message` | `Message` | 消息数据 |
| `isStreaming?` | `boolean` | 是否正在流式输出 |

**ToolCallBubble**

| Props | 类型 | 说明 |
|-------|------|------|
| `event` | `ToolEvent` | 工具调用事件 |

**ToolResultBubble**

| Props | 类型 | 说明 |
|-------|------|------|
| `event` | `ToolEvent` | 工具结果事件 |

**ArchiveSuggestion**

| Props | 类型 | 说明 |
|-------|------|------|
| `data` | `unknown` | 归档建议数据 |

---

## GraphView

源文件：`pages/RingSpace/GraphView.tsx`

Props：`ringId: string`

D3.js 力导向图渲染 + `NodeTree` 侧边导航。

**ForceGraph**

源文件：`components/graph/ForceGraph.tsx`

| Props | 类型 | 说明 |
|-------|------|------|
| `nodes` | `GraphNode[]` | 节点列表 |
| `edges` | `GraphEdge[]` | 边列表 |
| `onNodeClick` | `(node) => void` | 节点点击 |

**NodeTree**

源文件：`components/graph/NodeTree.tsx`

| Props | 类型 | 说明 |
|-------|------|------|
| `nodes` | `GraphNode[]` | 节点列表 |
| `onSelect` | `(node) => void` | 选中节点 |

---

## BlueprintWizard

源文件：`pages/RingSpace/BlueprintWizard.tsx`

Props：`ringId: string`

多步骤蓝图构建流程：选择模板 → 多轮对话 → 预览 → 确认。

---

## PrList / PrDetail

源文件：`pages/RingSpace/PrList.tsx`、`PrDetail.tsx`

**DiffView**

源文件：`components/git/DiffView.tsx`

| Props | 类型 | 说明 |
|-------|------|------|
| `diff` | `FileChange[]` | 文件变更列表 |

---

## MemberList

源文件：`components/member/MemberList.tsx`

| Props | 类型 | 说明 |
|-------|------|------|
| `ring_id` | `string` | Ring ID |
| `members` | `Member[]` | 成员列表 |

---

## SessionView

源文件：`components/session/SessionView.tsx`

| Props | 类型 | 说明 |
|-------||------|------|
| `ring_id` | `string` | Ring ID |
| `session_id` | `string` | Session ID |

---

## SettingsPage

源文件：`pages/Settings/SettingsPage.tsx`

LLM 配置表单（provider/model/api_key/base_url）+ 隐私设置。

---

## Toolbar

源文件：`components/toolbar/Toolbar.tsx`

顶部工具栏，包含：导航链接、操作按钮。
