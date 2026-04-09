# Ring 前端 UI 重建设计文档

## 1. 设计方向

| 维度 | 决策 |
|------|------|
| 气质 | 精密/理性，面向公司全员（非纯开发者） |
| 色调 | 亮色主导，Mono + 冷蓝点缀 |
| 字体 | Helvetica Neue 正文/标题，SF Mono 代码 |
| 布局 | Ring Space 三栏弹性，图谱树常驻 |
| 范围 | 全量页面，仅亮色主题 |
| 方案 | Token 驱动重建 |

## 2. 设计令牌

### 2.1 色彩

```css
:root {
  --color-bg-primary: #FFFFFF;
  --color-bg-secondary: #FAFAFA;
  --color-bg-tertiary: #F5F5F5;
  --color-border: #E5E5E5;
  --color-border-light: #F0F0F0;
  --color-text-primary: #171717;
  --color-text-secondary: #737373;
  --color-text-tertiary: #A3A3A3;
  --color-accent: #2563EB;
  --color-accent-light: #EFF6FF;
  --color-accent-hover: #1D4ED8;
  --color-success: #16A34A;
  --color-success-light: #F0FDF4;
  --color-danger: #DC2626;
  --color-danger-light: #FEF2F2;
  --color-warning: #D97706;
  --color-warning-light: #FFFBEB;
}
```

### 2.2 间距

4px 基准：`4, 8, 12, 16, 20, 24, 32, 48`

```css
:root {
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-5: 20px;
  --space-6: 24px;
  --space-8: 32px;
  --space-12: 48px;
}
```

### 2.3 圆角

```css
:root {
  --radius-sm: 4px;
  --radius-md: 6px;
  --radius-lg: 8px;
  --radius-xl: 12px;
  --radius-full: 9999px;
}
```

### 2.4 字体

```css
:root {
  --font-sans: 'Helvetica Neue', Helvetica, Arial, sans-serif;
  --font-mono: 'SF Mono', 'Fira Code', 'JetBrains Mono', ui-monospace, monospace;
  --font-size-xs: 11px;
  --font-size-sm: 12px;
  --font-size-base: 13px;
  --font-size-md: 14px;
  --font-size-lg: 16px;
  --font-size-xl: 20px;
  --font-size-2xl: 28px;
  --font-size-3xl: 36px;
  --line-height-tight: 1.25;
  --line-height-normal: 1.5;
  --line-height-relaxed: 1.6;
}
```

### 2.5 阴影

```css
:root {
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.05);
  --shadow-md: 0 2px 8px rgba(0, 0, 0, 0.08);
  --shadow-lg: 0 4px 16px rgba(0, 0, 0, 0.1);
}
```

## 3. 共享组件

### 3.1 Button

| 变体 | 背景 | 文字 | 边框 |
|------|------|------|------|
| primary | `--color-accent` | #FFF | none |
| primary:hover | `--color-accent-hover` | #FFF | none |
| secondary | `--color-bg-tertiary` | `--color-text-primary` | `--color-border` |
| ghost | transparent | `--color-text-secondary` | none |
| danger | `--color-danger` | #FFF | none |

尺寸：sm（28px高）/ md（34px高）/ lg（40px高）

### 3.2 Input / TextArea / Select

- 背景 `--color-bg-primary`
- 边框 `--color-border`，focus 时 `--color-accent`
- 内边距 8px 12px
- 圆角 `--radius-md`
- placeholder 色 `--color-text-tertiary`

### 3.3 Badge

| 状态 | 背景 | 文字 |
|------|------|------|
| active/opened | `--color-accent-light` | `--color-accent` |
| merged/success | `--color-success-light` | `--color-success` |
| closed/danger | `--color-danger-light` | `--color-danger` |
| warning | `--color-warning-light` | `--color-warning` |
| neutral | `--color-bg-tertiary` | `--color-text-secondary` |

圆角 `--radius-sm`，padding 2px 8px，font-size `--font-size-xs`，font-weight 500。

### 3.4 Avatar

- 圆形，32px（sm: 24px, lg: 40px）
- 首字母居中，font-weight 600
- 颜色映射：根据用户名 hash 从预设色板（`#2563EB`, `#16A34A`, `#D97706`, `#7C3AED`, `#DB2777`, `#0891B2`）中取
- 边框 2px `--color-bg-primary`（用于 AvatarGroup 堆叠）

### 3.5 AvatarGroup

- 堆叠重叠 -8px
- 最后一个显示 "+N" 计数
- 最多显示 4 个头像

### 3.6 Card

- 背景 `--color-bg-primary`
- 边框 `--color-border`
- 圆角 `--radius-lg`
- 内边距 20px
- hover：边框变为 `--color-accent`，阴影 `--shadow-sm`

### 3.7 Tabs

- 用在 Setup Wizard 等场景
- 底部指示线 2px `--color-accent`
- 激活态：文字 `--color-accent` font-weight 500
- 非激活态：文字 `--color-text-secondary`
- 间距 4px

### 3.8 Modal

- 遮罩 rgba(0,0,0,0.3)
- 内容区白底，圆角 `--radius-xl`，阴影 `--shadow-lg`
- 最大宽度 480px（表单类）/ 720px（diff 类）
- 顶部标题 + 右上关闭按钮 + 底部操作按钮区

### 3.9 EmptyState

- 居中，图标 + 标题 + 描述 + 可选操作按钮
- 图标色 `--color-text-tertiary`

### 3.10 Skeleton

- 背景 `--color-bg-tertiary`
- 动画：shimmer 渐变扫过

### 3.11 NotificationBell

- 铃铛图标 + 红色数字 badge
- 点击展开下拉面板，列出未读通知
- 每条通知：图标 + 标题 + 时间 + 点击跳转

### 3.12 ArchiveSuggestion（重设计）

- 蓝色左边框卡片
- 显示：AI 推荐的操作描述 + 目标节点路径
- 左栏图谱树同步高亮推荐节点（蓝色背景）
- 三个按钮：确认归档（primary）/ 换个位置（secondary）/ 跳过（ghost）
- 确认后显示 toast："已归档到 [节点名]"
- "换个位置"弹出节点选择器 Modal

## 4. 页面设计

### 4.1 顶层导航结构

**变更：合并 NavBar 和 RingNavBar 为统一导航。**

Ring Hub 模式：
```
┌─────────────────────────────────────────────────┐
│ [Ring logo] Ring Hub          [🔔] [⚙ Settings] │
├─────────────────────────────────────────────────┤
│                                                 │
│  Ring 卡片网格                                   │
│                                                 │
└─────────────────────────────────────────────────┘
```

Ring Space 模式：
```
┌─────────────────────────────────────────────────┐
│ [← Hub] 产品竞品分析组   [👥 K L M +2] [🔔] [⚙] │
├────────┬────────────────────────────────────────┤
│ 图谱树  │  Ring 标题栏                            │
│        │                                        │
│ 导航列表│  主内容区                               │
│ Chat   │                                        │
│ Graph  │                                        │
│ PRs    │                                        │
│ Members│                                        │
│ Sessions│                                       │
└────────┴────────────────────────────────────────┘
```

### 4.2 Ring Hub

**结构：**
- 顶栏：Logo "Ring" 导航链接 + Super Ring 导航链接 + 通知铃铛 + Settings 导航链接
- 标题区：h1 "Ring Hub" + 副标题 "你的群组知识协作空间"
- 操作栏：Create Ring 按钮（primary）
- 内容区：Ring 卡片网格（2列，min-width 320px）
- 底部提示："对话记录仅保存在当前设备"（`--color-text-tertiary`，12px）

**Ring 卡片内容：**
- 左上：色块点 + Ring 名称（font-weight 600）
- 名称下：角色描述摘要（截断一行）
- 右上：最后活动时间（relative time）
- 底部分割线
- 底行：AvatarGroup（最多4个）+ 节点数 + 角色 Badge

**空状态：** EmptyState 组件，"还没有 Ring" + "创建你的第一个 Ring 群组空间" + Create Ring 按钮

**Create Ring 流程：** 点击按钮弹出 Modal，包含名称/描述/角色描述三个字段。提交后卡片出现在列表中，自动跳转到 Blueprint Wizard。

### 4.3 Ring Space — 三栏弹性布局

**左栏（240px，可收起至 0）：**
- 图谱选择器（select dropdown）
- 图谱节点树（NodeTree 组件）
  - 展开折叠
  - 点击节点：中栏和右栏联动响应
  - 归档建议时：推荐节点蓝色高亮背景
- 底部分割线
- 导航列表（垂直排列）：
  - Chat（默认激活）
  - Graph
  - PRs（如有待审核 PR 显示红色数字 badge）
  - Members
  - Sessions
- 收起按钮：点击折叠左栏为图标模式（48px 宽，只显示图标），再点展开

**中栏（flex: 1，弹性宽度）：**

Chat 模式：
- 顶栏：当前模式名 "Chat" + Auto 模式状态标签（如有）+ 成员头像组 + 通知铃铛
- 消息区：聊天气泡列表，自动滚底
- 底部：工具栏（Toolbar）+ 输入框 + Send 按钮
- ArchiveSuggestion 内联在消息流中

Graph 模式：
- 顶栏：Graph + 图谱名 + 导出按钮
- D3 力导向图占据中栏全部高度
- 点击节点：右栏弹出节点详情

PRs 模式：
- 顶栏：PRs + 筛选 tabs（Opened/Merged/Closed）
- PR 列表：紧凑行布局（状态 Badge + 标题 + 作者 + 时间）
- 点击 PR：中栏切换为 PR Detail（返回按钮 + 标题 + Merge/Reject + DiffView）

Members 模式：
- 成员列表：头像 + 名称 + 角色 Badge + 加入时间
- 创建者可见：邀请按钮（弹出邀请链接生成 Modal）

Sessions 模式：
- Session 列表或当前活跃 Session 的聊天界面
- 新建 Session：选择场景 + 标题 + 邀请成员

**右栏（280px，默认隐藏，按需弹出）：**

触发条件：
- 点击左栏树中节点
- ArchiveSuggestion 中"换个位置"
- PR Detail 查看 diff
- Graph 模式点击节点

内容：
- 节点详情：标签 + 类型 Badge + Markdown 预览
- Diff View：并排对比
- 节点选择器：树形选择（归档时选位置）

关闭按钮在右上角，点击关闭收起右栏。

### 4.4 Blueprint Wizard

**结构：**
- 顶部 tabs：模板 / 自定义
- 模板模式：卡片网格（2-3列），每张模板卡片显示名称+描述+节点数预览
  - 点击卡片：预览区展示图谱结构概要
  - "使用此模板"按钮 → 确认蓝图
- 自定义模式：与 Group Ring 对话，构建图谱
  - 聊天界面（复用 ChatBubble + ChatInput）
  - AI 每次调整后更新预览
- 预览区：蓝色边框面板，显示图谱列表（名称+类型+分类），"确认蓝图"按钮

### 4.5 Setup Wizard

**结构：**
- 居中卡片（max-width 420px），白底 + 阴影
- 顶部：Ring logo + "Welcome to Ring" 标题
- 步骤指示器：三个圆点连线，当前步骤高亮
- 三步表单：
  1. Username：display name 输入框
  2. LLM：provider select + model input + api key password + base url input
  3. GitLab：repo url + auth type select + ssh key path + Skip 按钮
- 底部：Back / Next 按钮

### 4.6 Settings

**结构：**
- 居中单列（max-width 560px）
- 分区卡片：
  - Profile（只读）：display name + user id
  - LLM Configuration：provider + model + api key + base url + privacy toggle + Save
- 分区之间 32px 间距

### 4.7 Super Ring Chat

**结构：**
- 独立页面，布局同 Chat 模式但无左栏/右栏
- 顶栏："Super Ring" 标题 + 副标题 "全局助手"
- 全宽聊天界面

## 5. 功能联动规则

### 5.1 归档流程联动

```
用户在 Chat 中标记归档
  → ArchiveSuggestion 出现在消息流
  → 左栏图谱树高亮 AI 推荐的目标节点（蓝色背景）
  → 右栏弹出节点预览（Markdown 摘要）
  → 用户点"确认"：toast "已归档到 X" + 树更新 + 右栏关闭
  → 用户点"换个位置"：右栏切换为节点选择器（树形）
  → 用户点"跳过"：suggestion 收起，恢复普通消息样式
```

### 5.2 PR 流程联动

```
成员提交 PR
  → 创建者收到通知（铃铛 badge +1）
  → 点击通知 → 左栏自动切到 PRs + 中栏显示该 PR Detail
  → PR Detail 中栏内展示 Diff（或右栏弹出 Diff 并排视图）
  → Merge → toast "已合并" + PR 列表更新 + 通知提交成员
  → Reject → toast "已拒绝" + PR 列表更新 + 通知提交成员
```

### 5.3 图谱导航联动

```
左栏点击节点
  → Chat 模式：右栏弹出节点详情（Markdown 预览）
  → Graph 模式：D3 图聚焦该节点 + 右栏弹出详情
  → PRs/Members/Sessions 模式：右栏弹出节点详情（只读）
```

### 5.4 通知联动

```
通知来源 → 点击跳转目标：
  PR 新建/合并/拒绝 → PRs 视图（自动打开对应 PR Detail）
  成员加入/角色变更 → Members 视图
  Session 邀请 → Sessions 视图（自动加入 Session）
```

### 5.5 Auto 模式

```
Chat 顶栏显示 Auto badge（蓝色）
  → AI 自动归档，不弹 ArchiveSuggestion
  → 归档完成后 toast 通知："已自动归档到 [节点名]"
  → 左栏图谱树实时更新（新节点淡入动画）
  → 用户可随时点击 Auto badge 切回手动模式
```

## 6. 动效规范

克制为主，只在关键节点使用：

| 场景 | 动效 | 时长 |
|------|------|------|
| 页面切换 | 淡入 | 150ms ease |
| 右栏弹出 | 从右滑入 | 200ms ease-out |
| 卡片 hover | 边框色变化 | 150ms ease |
| 按钮 hover | 背景色变化 | 100ms ease |
| Toast 出现 | 从下淡入上移 | 200ms ease |
| Toast 消失 | 淡出 | 150ms ease |
| 树节点高亮 | 背景色变化 | 200ms ease |
| 新节点出现 | 淡入 | 300ms ease |

不使用：弹跳、旋转、缩放、粒子等装饰性动效。

## 7. 响应式

- 基准宽度 1126px（沿用当前 #root 设置）
- < 1024px：左栏默认收起为图标模式
- < 768px：不支持（PRD 明确内网桌面使用）

## 8. 实施顺序

1. 建立 CSS 变量令牌系统（替换 `index.css` 中的 `:root`）
2. 创建共享组件（Button, Input, Badge, Avatar, Card, Modal, Tabs, EmptyState, Skeleton, NotificationBell, ArchiveSuggestion）
3. 重建 Layout 组件（统一导航 + Ring Space 三栏框架）
4. 逐页重建：
   - Setup Wizard
   - Ring Hub
   - Ring Space（Chat / Graph / PRs / Members / Sessions）
   - Blueprint Wizard
   - Super Ring Chat
   - Settings
5. 清理旧样式（删除 `App.css`，移除所有内联 style）

## 9. 不做的事

- 暗色模式（后续迭代）
- 国际化 UI 文案调整（保持当前中英混合）
- 动画库引入（纯 CSS transition/animation）
- 图表/可视化库变更（沿用 D3.js）
- 路由结构变更（沿用当前路由）
- 新增 store 或 API 调用（纯 UI 层重建，不改变数据流）
