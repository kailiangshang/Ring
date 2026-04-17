# Ring Frontend UI Design

> **Affects**: `ui/src/`
> **Depends on**: [PRD.md](PRD.md) · [Redesign Design](../superpowers/specs/2026-04-15-ring-redesign-design.md)  
> **Prototypes**: `style-previews/` 目录下所有 HTML 文件  
> **Last updated**: 2026-04-17

---

## 1. Design Philosophy

**Chat-first, CLI-style, panel-based.**

- 主界面永远是聊天窗口，永不切换为其他页面
- 所有辅助信息通过右侧滑出面板展示
- 全局采用 CLI/terminal 视觉风格
- 用户同时拥有命令行输入和可点击按钮两种操作方式

---

## 2. Visual Style — IceChat Theme

**Scheme B（深蓝冰）**：冷色调深蓝底 + 青色高亮，可自然延展为完整聊天应用。

### 2.1 Color Palette

| Token | Hex | Usage |
|-------|-----|-------|
| `bg-base` | `#06080c` | 页面底色 |
| `bg-panel` | `#0a0e14` | 面板/浮窗底色 |
| `bg-sidebar` | `#080c12` | 侧栏底色 |
| `bg-input` | `#0d1117` | 输入框底色 |
| `bg-hover` | `#0d1420` | hover 状态 |
| `bg-active` | `#0d2a35` | active/选中状态 |
| `border` | `#1a2030` | 所有分隔线/边框 |
| `text-primary` | `#bfc7d5` | 主文本 |
| `text-secondary` | `#8892a0` | 次要文本 |
| `text-muted` | `#6b7d8e` | 弱文本 |
| `text-dim` | `#3a4550` | 最弱文本 |
| `accent-cyan` | `#0891B2` | 主强调色（按钮、边框、AI标识） |
| `accent-ice` | `#67E8F9` | 高亮强调（Ring 名、关键数值） |
| `accent-teal` | `#06B6D4` | 代码/链接色 |
| `accent-green` | `#22c55e` | 成功/在线状态 |
| `accent-amber` | `#f59e0b` | Self 专属色/警告/待处理 |
| `placeholder` | `#2a3540` | 占位符文本 |

### 2.2 Typography

| 用途 | 字体 | 说明 |
|------|------|------|
| 正文/代码 | **Cascadia Code** | VS Code 字体，支持 ligature |
| 标题/品牌 | **Space Grotesk** | Display 字体，用于大标题 |

Weight 支持：300 / 400 / 500 / 600 / 700。

### 2.3 Component Conventions

- 所有 border `1px solid #1a2030`
- 圆角统一 `border-radius: 3px`（小组件）或 `4px`（输入框/按钮）或 `8-10px`（浮窗/模态框）
- 过渡 `transition: all 0.15s`
- 滚动条 `width: 3-4px`，颜色 `#1a2030`
- 无 emoji 装饰（仅 Self 头像使用 emoji）

---

## 3. Page Architecture

整个应用只有一个页面，通过侧栏切换上下文。

```
┌──────────┬────────────────────────────────┬──────────┐
│          │ Header (Tab Bar)               │          │
│ Sidebar  ├────────────────────────────────┤  Right   │
│          │                                │  Panels  │
│          │    Chat (永不被替换)             │  (可叠加) │
│          │                                │          │
│          ├────────────────────────────────┤          │
│          │ Input Area                     │          │
└──────────┴────────────────────────────────┴──────────┘
                                              🐱 Self Float
                                              (右下角浮窗)
```

---

## 4. Sidebar

宽度 `200px`，固定左侧。结构从上到下：

```
┌─────────────────────┐
│ 🎵 RING         v0.1│  ← Logo + 品牌名
├─────────────────────┤
│ [S] Super Ring      │  ← 置顶，渐变图标，特殊样式
│     global assistant│
├─────────────────────┤
│ ● 竞品分析组     12  │  ← Ring 列表（扁平，无折叠子项）
│   ⚡ Q2 竞品评审     │  ← Session 指示器（仅在 active Ring 下显示）
│ ● 技术架构组      8  │
│ ● 产品设计组      5  │
│                     │
│                     │  ← 可滚动区域
├─────────────────────┤
│ [K] kaiiang     ⚙  │  ← 用户信息 + 设置按钮
└─────────────────────┘
```

### 4.1 Super Ring Entry

- 置顶显示，与 Ring 列表用分隔线隔开
- 图标为渐变背景圆角方块（`linear-gradient(135deg, #0891B2, #67E8F9)`）
- 点击后切换到 Super Ring 上下文

### 4.2 Ring List

- 扁平列表，每个 Ring 一行
- 左侧圆角色块标识（不同 Ring 不同颜色）
- 右侧 badge 显示节点数
- 无折叠子项，无层级嵌套

### 4.3 Session Indicator

- 仅在当前 active Ring 下方显示
- 绿色脉冲圆点 + Session 标题
- 点击后作为右侧面板打开（不在 header tabs 里）

### 4.4 Active Ring Styling

当前选中 Ring 背景 `#0d2a35`，名称变 `#67E8F9` 加粗。

---

## 5. Header Tab Bar

高度 `38px`，固定顶部。结构：

```
┌────────────┬─────────┬──────────┬──────────┬───────────────┬──────────┐
│ 竞品分析组  │ 💬 Chat │ Graph 13 │ Archive 2│ Config 3      │auto export│
└────────────┴─────────┴──────────┴──────────┴───────────────┴──────────┘
```

### 5.1 Tab 定义

**Group Ring 上下文的 tabs：**

| Tab | 触发面板 | 说明 |
|-----|---------|------|
| 💬 Chat | 无 | 关闭所有面板，回到纯聊天 |
| Graph | 右侧面板 | 图谱节点树 + 操作 |
| Archive | 右侧面板 | PR 队列 + 文件列表 |
| Config | 右侧面板 | Members + Blueprint 合并展示 |

**Super Ring 上下文的 tabs：** Rings / Skills / Settings（面板内容待实现）。

**Session 不在 header tabs 中**，仅在侧栏显示，点击后作为右侧面板打开。

### 5.2 Tab 行为

- 点击 tab → 打开对应右侧面板
- 再次点击同一 tab → 关闭面板
- 点击 Chat tab → 关闭所有面板
- 多个面板可同时打开，从左到右依次叠加
- 每个 panel 有独立 × 关闭按钮，关闭一个不影响其他

---

## 6. Chat Area

永不被替换的主区域。

### 6.1 Messages

- AI 消息角色标签 `GROUP RING` / `SUPER RING` / `SESSION RING`，大写 + letter-spacing
- 用户消息标签 `YOU`，颜色 `#67E8F9`
- 系统消息标签 `SYSTEM`，颜色 `#22c55e`
- 支持 bold / italic / code 格式

### 6.2 Input Area

```
┌──────────┬────────────────────────────┬──────┐
│  [ring]  │ message / command...       │ SEND │
└──────────┴────────────────────────────┴──────┘
 !graph  !archive  !config  !session  @self
```

- 左侧 mode indicator 显示当前 AI 层级（`ring` / `super` / `review` / `self`）
- 输入框支持四前缀命令系统（`@` 寻址 / `#` 引用 / `!` 操作 / `%` 元操作），详见 [CLI Command System Design](../superpowers/specs/2026-04-17-cli-command-system-design.md)
- 输入 `@self` 自动打开 Self 浮窗，输入 `#` 弹出图谱节点补全，输入 `!` 弹出操作命令列表
- 底部 command hints 可点击

---

## 7. Right Panels — Stackable

### 7.1 面板规格

- 宽度 `320px`
- 从右向左滑入，CSS transition `0.2s ease`
- 面板按打开顺序叠加，通过 `data-depth` 属性区分背景色深度：
  - depth 1: `#0a0e14`
  - depth 2: `#0b1018`
  - depth 3: `#0c1220`

### 7.2 面板关闭规则

- 点击面板内 × 按钮 → **仅关闭该面板**
- 不级联关闭其他面板
- 关闭后重新计算剩余面板的 depth index

### 7.3 Graph Panel

- 节点树（缩进展示层级）
- 节点类型标识：root / topic / leaf
- Actions: expand all / export

### 7.4 Archive Panel

- Pending PR 列表（amber 状态）
- Merged PR 列表（green 状态）
- Files 列表（文件名 + 大小）

### 7.5 Config Panel

**Members + Blueprint 合并展示**，因为两者都相对静态：

- **Members 区域**：成员列表（头像 + 名 + 角色 + 在线状态）+ invite 按钮
- **Blueprint 区域**：当前图谱结构预览（树形文本）
- **Blueprint Templates 区域**：可选模板列表

### 7.6 Session Panel

- Session 信息（Skill / Phase / Owner / Archive 状态）
- Participants 列表
- Materials 列表
- Actions: start summary / end session

---

## 8. Self Floating Window

右下角独立浮窗，不属于任何面板，在所有界面层级都可用。

### 8.1 Trigger

- 右下角 🐱 浮动按钮（`48px`，渐变背景，呼吸动画）
- 按钮可拖拽移动位置
- 短按打开浮窗，拖拽移动按钮
- 输入 `@self` 也自动打开

### 8.2 浮窗结构

```
┌──────────────────────────────┐
│ 🐱 Self          [─] [📌] [×]│  ← Header（可拖拽）
│    [竞品分析组]               │  ← 当前上下文 badge
├──────┬───────┬───────────────┤
│ Chat │Memory │ Settings      │  ← 内部 Tabs
├──────┴───────┴───────────────┤
│                              │
│  对话内容 / 记忆面板 / 设置   │  ← Tab 内容区
│                              │
├──────────────────────────────┤
│ [和 Self 聊聊...]        [↑] │  ← 输入框
└──────────────────────────────┘
```

### 8.3 Chat Tab

- 私有对话，按日期分组
- Self 消息 + 用户消息 + suggestion 消息（amber 色）
- Self 会基于用户行为主动提供建议

### 8.4 Memory Tab

- **Behavior Profile**：归档风格、活跃时段、平均长度、auto-archive 偏好
- **Interaction Stats**：对话数、本周归档数、建议接受率（进度条可视化）
- **Known Preferences**：语言、详细程度、格式偏好、确认行为
- 底部声明：Memory 100% 私有，存储在 `~/.ring/self/`

### 8.5 Settings Tab

**Personality：**
- Tone 选择（friendly / professional / playful / minimal）
- Proactivity 开关
- Suggestions 开关

**Privacy：**
- Read context（读取当前 Ring/Session 上下文）
- Remember patterns（记录行为模式）
- Session awareness（访问 Session 讨论）

**Data：**
- Export memory
- Reset all memory（amber 警告色）

### 8.4 行为规则

- 浮窗可拖拽（header 区域）、可缩放（`resize: both`）
- 最小尺寸 `260px × 300px`
- 打开时定位在 🐱 按钮左上方
- Header 显示当前所在上下文（Ring 名 / Session 名 / Global）的 badge
- 关闭浮窗 → 回到 🐱 按钮

---

## 9. Setup / Join Flow

首次启动和加入 Ring 共用同一个引导流程。

### 9.1 流程分支

用户在首页选择：

| 选择 | 流程 |
|------|------|
| 🚀 New User | Welcome → Identity → LLM → Create Ring → Done |
| 🔗 Join Existing | Welcome → Identity → LLM → Join Ring → Done |

### 9.2 Welcome Step

- 品牌展示（Logo + RING + 副标题）
- 两张选择卡片：New User / Join Existing
- 进度条显示当前步骤

### 9.3 Identity Step

- **Avatar 选择**：字母或 emoji，实时预览
- **Username**（必填，不可修改）+ Display name（可选）
- 字母头像时背景 `#0d2a35` + cyan 色，emoji 头像时 amber 色

### 9.4 LLM Step

- 自动检测本地 Ollama（显示绿色状态点 + 可用模型）
- Provider 下拉：Ollama (local) / OpenAI / Anthropic
- 选 OpenAI/Anthropic 时显示 API Key 输入框
- API Key 声明：仅本地存储

### 9.5 Create Ring Step

- Ring 名称输入
- Blueprint 模板选择（卡片列表）：
  - 产品研究（6 nodes）
  - 项目管理（5 nodes）
  - 技术文档（4 nodes）
  - 空白（0 nodes）

### 9.6 Join Ring Step

- 邀请链接/代码输入框 + paste 按钮
- 链接解析后显示 Ring 信息（名称、成员数、节点数、Skills）
- 声明：加入后 clone 数据到 `~/.ring/rings/`

### 9.7 Done Step

- 完成总结（身份、LLM、Ring 信息）
- 常用命令速查 CLI 风格展示
- Launch 按钮进入主界面

### 9.8 导航规则

- Back/Next 按钮控制步骤
- Skip 可跳过直接进入主界面
- 进度条实时反映当前步骤

---

## 10. Interaction Summary

### 10.1 命令系统

采用四前缀命令系统（`@ # ! %`），详见 [CLI Command System Design](../superpowers/specs/2026-04-17-cli-command-system-design.md)。

常用命令速查：

| 前缀 | 语义 | 常用 |
|------|------|------|
| `@` | 寻址 | `@self` / `@ring` / `@super` / `@用户名` |
| `#` | 引用 | `#节点名` / `#标签名` |
| `!` | 操作 | `!graph` / `!save` / `!auto` / `!export` / `!invite` |
| `%` | 元操作 | `%role` / `%skill list` / `%llm` |

### 10.2 全局快捷方式

| 操作 | 触发方式 |
|------|---------|
| 打开 Self | 点击 🐱 / 输入 `@self` |
| 切换 Ring | 点击侧栏 Ring 项 |
| 切换到 Super Ring | 点击侧栏 Super Ring |
| 打开 Session | 点击侧栏 Session 指示器 |
| 移动 Self 浮窗 | 拖拽 header |
| 移动 🐱 按钮 | 拖拽按钮 |
| 关闭面板 | 点击面板 × |

---

## 11. Prototype Files

| 文件 | 内容 | 状态 |
|------|------|------|
| `style-system.html` | IceChat 主题系统（颜色/字体/组件/Cascadia Code） | ✅ |
| `group-panel-v2.html` | Group 主界面 + 堆叠面板 + Self 浮窗 | ✅ |
| `navigation-v2.html` | Super Ring / Session 上下文切换 + Tab 配置 | ✅ |
| `self-detail.html` | Self 浮窗详细版（Chat/Memory/Settings 三 Tab） | ✅ |
| `setup-flow.html` | Setup + Join 引导流（分步表单） | ✅ |
| `preview.html` | 早期三方案对比（A/B/C） | 历史参考 |

---

## 12. Not Yet Designed

以下 UI 细节留到实现阶段：

- Super Ring 右侧面板内容（Rings 管理 / Skills / Settings）
- 图谱 D3.js 可视化交互
- PR 审核 Diff 并排对比视图
- 通知列表 UI
- 移动端适配
