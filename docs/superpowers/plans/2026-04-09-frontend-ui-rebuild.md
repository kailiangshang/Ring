# Ring 前端 UI 重建实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 用 Token 驱动方式重建 Ring 前端全部页面的 UI，建立一致的设计系统，实现三栏联动的 Ring Space 布局。

**Architecture:** 建立 CSS 变量设计令牌系统 + 共享 UI 组件库，替换所有内联样式。重构 Layout 为统一导航（Hub 模式单栏 / Ring Space 三栏弹性）。逐页重建，保持现有路由、store、API 不变。

**Tech Stack:** React 19 + TypeScript, CSS Variables (no CSS-in-JS), Vite, Vitest + Testing Library

---

## File Structure

### 新建文件

```
src/
  components/
    ui/
      Button.tsx              # 按钮组件（primary/secondary/ghost/danger）
      Button.css
      Input.tsx               # 输入框组件
      Input.css
      Badge.tsx               # 状态标签
      Badge.css
      Avatar.tsx              # 头像 + 颜色映射
      Avatar.css
      AvatarGroup.tsx         # 头像堆叠
      Modal.tsx               # 弹窗
      Modal.css
      Tabs.tsx                # 标签切换
      Tabs.css
      EmptyState.tsx          # 空状态
      EmptyState.css
      Skeleton.tsx            # 加载骨架
      Skeleton.css
      NotificationBell.tsx    # 通知铃铛
      NotificationBell.css
  layout/
    AppShell.tsx              # 统一外壳（替代 Layout.tsx）
    AppShell.css
    HubNavBar.tsx             # Hub 模式顶栏
    HubNavBar.css
    RingSpaceLayout.tsx       # Ring Space 三栏布局
    RingSpaceLayout.css
    RingSidebar.tsx           # 左栏（图谱树+导航）
    RingSidebar.css
    RightPanel.tsx            # 右栏（按需弹出）
    RightPanel.css
  chat/
    ChatBubble.css            # ChatBubble 样式（替换内联）
    ChatInput.css
    ToolCallBubble.css
    ToolResultBubble.css
    ArchiveSuggestion.css     # 重设计的归档建议
```

### 修改文件

```
src/index.css                 # 替换 :root 为新 Token 系统
src/App.tsx                   # 更新路由使用 AppShell
src/App.css                   # 清空（所有样式迁移到组件 CSS）
src/components/layout/Layout.tsx    → 删除，被 AppShell 替代
src/components/layout/NavBar.tsx    → 删除，被 HubNavBar 替代
src/components/layout/RingNavBar.tsx → 删除，被 RingSidebar 替代
src/components/chat/ChatBubble.tsx      # 移除内联样式，用 CSS class
src/components/chat/ChatInput.tsx       # 移除内联样式
src/components/chat/ToolCallBubble.tsx  # 移除内联样式
src/components/chat/ToolResultBubble.tsx # 移除内联样式
src/components/chat/ArchiveSuggestion.tsx # 重设计
src/components/graph/NodeTree.tsx       # 移除内联样式，加 highlighted_node_id prop
src/components/graph/ForceGraph.tsx     # 更新颜色为 Token
src/components/git/DiffView.tsx         # 移除内联样式
src/components/toolbar/Toolbar.tsx      # 移除内联样式
src/components/member/MemberList.tsx    # 移除内联样式
src/components/session/SessionView.tsx  # 移除内联样式
src/pages/RingHub/RingHub.tsx           # 重建 UI
src/pages/RingHub/RingList.tsx          # 重建为卡片网格
src/pages/RingHub/CreateRing.tsx        # 改为 Modal
src/pages/RingHub/SuperRingChat.tsx     # 重建 UI
src/pages/RingSpace/ChatView.tsx        # 适配三栏布局
src/pages/RingSpace/GraphView.tsx       # 适配三栏布局
src/pages/RingSpace/PrList.tsx          # 重建 UI
src/pages/RingSpace/PrDetail.tsx        # 重建 UI
src/pages/RingSpace/BlueprintWizard.tsx # 重建 UI
src/pages/Setup/SetupWizard.tsx         # 重建 UI
src/pages/Setup/StepUsername.tsx        # 重建 UI
src/pages/Setup/StepLlm.tsx             # 重建 UI
src/pages/Setup/StepGitlab.tsx          # 重建 UI
src/pages/Settings/SettingsPage.tsx     # 重建 UI
```

---

## Task 1: Design Tokens — 替换 index.css 的 :root

**Files:**
- Modify: `src/index.css`

- [ ] **Step 1: Write the new :root token system into index.css**

Replace the entire `:root` block and all existing global styles with the new token system. Keep `#root`, body, h1/h2, code, .spinner-container, .spinner, @keyframes spin, .toast-container, .toast-item, @keyframes toast-in, .chat-bubble-assistant` styles but update them to use new tokens. Remove `.navbar`, `.navbar-brand`, `.navbar-links`, `.nav-link`, `.ring-navbar`, `.ring-back`, `.ring-tabs`, `.ring-tab`, `.main-content` (these move to component CSS).

The new `index.css` should contain:
- `:root` with all design tokens from spec section 2
- `#root` styles (unchanged width/margin, remove `min-height: 100svh`)
- `body { margin: 0; }`
- Global typography: `h1, h2` using `var(--font-sans)`, `var(--color-text-primary)`
- Global `code` styles
- `.spinner-container` and `.spinner` updated with token colors
- `.toast-container` and `.toast-item` updated with token colors
- All `.chat-bubble-assistant` markdown styles updated with token colors

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
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-5: 20px;
  --space-6: 24px;
  --space-8: 32px;
  --space-12: 48px;
  --radius-sm: 4px;
  --radius-md: 6px;
  --radius-lg: 8px;
  --radius-xl: 12px;
  --radius-full: 9999px;
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
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.05);
  --shadow-md: 0 2px 8px rgba(0, 0, 0, 0.08);
  --shadow-lg: 0 4px 16px rgba(0, 0, 0, 0.1);
}
```

Remove the entire `@media (prefers-color-scheme: dark)` block (spec says light-only). Remove all `.navbar*`, `.ring-navbar*`, `.nav-link`, `.ring-tab`, `.main-content` rules.

- [ ] **Step 2: Clear App.css**

Replace `src/App.css` contents with only:
```css
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}
```

Remove `body`, `form`, `label`, `input/select/textarea`, `button`, `[role="alert"]` rules (these will be handled by component CSS and tokens).

- [ ] **Step 3: Run build to verify no breakage**

Run: `cd ring-frontend && npm run build`
Expected: Build succeeds (pages may look broken visually but no compile errors)

- [ ] **Step 4: Commit**

```
feat(ui): establish design token system
```

---

## Task 2: Button 组件

**Files:**
- Create: `src/components/ui/Button.tsx`
- Create: `src/components/ui/Button.css`

- [ ] **Step 1: Create Button.css**

```css
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-family: var(--font-sans);
  font-size: var(--font-size-base);
  font-weight: 500;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background 100ms ease, border-color 150ms ease;
  white-space: nowrap;
  line-height: 1;
}
.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-sm { height: 28px; padding: 0 12px; font-size: var(--font-size-sm); }
.btn-md { height: 34px; padding: 0 16px; }
.btn-lg { height: 40px; padding: 0 20px; font-size: var(--font-size-md); }

.btn-primary {
  background: var(--color-accent);
  color: #fff;
}
.btn-primary:hover:not(:disabled) {
  background: var(--color-accent-hover);
}

.btn-secondary {
  background: var(--color-bg-tertiary);
  color: var(--color-text-primary);
  border: 1px solid var(--color-border);
}
.btn-secondary:hover:not(:disabled) {
  border-color: var(--color-text-tertiary);
}

.btn-ghost {
  background: transparent;
  color: var(--color-text-secondary);
}
.btn-ghost:hover:not(:disabled) {
  color: var(--color-text-primary);
  background: var(--color-bg-tertiary);
}

.btn-danger {
  background: var(--color-danger);
  color: #fff;
}
.btn-danger:hover:not(:disabled) {
  background: #B91C1C;
}
```

- [ ] **Step 2: Create Button.tsx**

```tsx
import './Button.css'

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger'
  size?: 'sm' | 'md' | 'lg'
}

export function Button({
  variant = 'primary',
  size = 'md',
  className = '',
  ...props
}: ButtonProps) {
  return (
    <button
      className={`btn btn-${variant} btn-${size} ${className}`}
      {...props}
    />
  )
}
```

- [ ] **Step 3: Commit**

```
feat(ui): add Button component
```

---

## Task 3: Input + Badge + Avatar + AvatarGroup 组件

**Files:**
- Create: `src/components/ui/Input.tsx`
- Create: `src/components/ui/Input.css`
- Create: `src/components/ui/Badge.tsx`
- Create: `src/components/ui/Badge.css`
- Create: `src/components/ui/Avatar.tsx`
- Create: `src/components/ui/Avatar.css`
- Create: `src/components/ui/AvatarGroup.tsx`

- [ ] **Step 1: Create Input.css and Input.tsx**

Input.css:
```css
.input-field {
  width: 100%;
  padding: 8px 12px;
  font-family: var(--font-sans);
  font-size: var(--font-size-base);
  color: var(--color-text-primary);
  background: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  outline: none;
  transition: border-color 150ms ease;
  line-height: var(--line-height-normal);
}
.input-field::placeholder {
  color: var(--color-text-tertiary);
}
.input-field:focus {
  border-color: var(--color-accent);
}
.input-field:disabled {
  background: var(--color-bg-tertiary);
  opacity: 0.6;
}

.textarea-field {
  min-height: 80px;
  resize: vertical;
}
```

Input.tsx:
```tsx
import './Input.css'

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  input_type?: 'input' | 'textarea' | 'select'
}

export function Input({ input_type = 'input', className = '', ...props }: InputProps) {
  const cls = `input-field ${className}`

  if (input_type === 'textarea') {
    return <textarea className={`${cls} textarea-field`} {...(props as React.TextareaHTMLAttributes<HTMLTextAreaElement>)} />
  }
  if (input_type === 'select') {
    return <select className={cls} {...(props as React.SelectHTMLAttributes<HTMLSelectElement>)} />
  }
  return <input className={cls} {...props} />
}
```

- [ ] **Step 2: Create Badge.css and Badge.tsx**

Badge.css:
```css
.badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  font-size: var(--font-size-xs);
  font-weight: 500;
  font-family: var(--font-sans);
  line-height: 1.4;
}

.badge-accent { background: var(--color-accent-light); color: var(--color-accent); }
.badge-success { background: var(--color-success-light); color: var(--color-success); }
.badge-danger { background: var(--color-danger-light); color: var(--color-danger); }
.badge-warning { background: var(--color-warning-light); color: var(--color-warning); }
.badge-neutral { background: var(--color-bg-tertiary); color: var(--color-text-secondary); }
```

Badge.tsx:
```tsx
import './Badge.css'

type BadgeVariant = 'accent' | 'success' | 'danger' | 'warning' | 'neutral'

const STATUS_MAP: Record<string, BadgeVariant> = {
  active: 'accent',
  opened: 'accent',
  merged: 'success',
  success: 'success',
  closed: 'danger',
  error: 'danger',
  warning: 'warning',
  creator: 'warning',
  admin: 'accent',
  member: 'success',
  readonly: 'neutral',
}

interface BadgeProps {
  variant?: BadgeVariant
  status?: string
  children: React.ReactNode
}

export function Badge({ variant, status, children }: BadgeProps) {
  const v = variant || (status ? STATUS_MAP[status] || 'neutral' : 'neutral')
  return <span className={`badge badge-${v}`}>{children}</span>
}
```

- [ ] **Step 3: Create Avatar.css, Avatar.tsx, AvatarGroup.tsx**

Avatar.css:
```css
.avatar {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-full);
  font-weight: 600;
  font-family: var(--font-sans);
  color: #fff;
  flex-shrink: 0;
}
.avatar-sm { width: 24px; height: 24px; font-size: 10px; }
.avatar-md { width: 32px; height: 32px; font-size: var(--font-size-sm); }
.avatar-lg { width: 40px; height: 40px; font-size: var(--font-size-md); }

.avatar-group {
  display: inline-flex;
  align-items: center;
}
.avatar-group .avatar {
  border: 2px solid var(--color-bg-primary);
}
.avatar-group .avatar:not(:first-child) {
  margin-left: -8px;
}
.avatar-group-count {
  font-size: var(--font-size-sm);
  color: var(--color-text-tertiary);
  margin-left: var(--space-2);
}
```

Avatar.tsx:
```tsx
import './Avatar.css'

const COLORS = ['#2563EB', '#16A34A', '#D97706', '#7C3AED', '#DB2777', '#0891B2']

function hash_color(name: string): string {
  let hash = 0
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash)
  }
  return COLORS[Math.abs(hash) % COLORS.length]
}

interface AvatarProps {
  name: string
  size?: 'sm' | 'md' | 'lg'
}

export function Avatar({ name, size = 'md' }: AvatarProps) {
  const initial = name.charAt(0).toUpperCase()
  return (
    <div
      className={`avatar avatar-${size}`}
      style={{ background: hash_color(name) }}
      title={name}
    >
      {initial}
    </div>
  )
}
```

AvatarGroup.tsx:
```tsx
import { Avatar } from './Avatar'
import './Avatar.css'

interface AvatarGroupProps {
  names: string[]
  max?: number
  size?: 'sm' | 'md' | 'lg'
}

export function AvatarGroup({ names, max = 4, size = 'sm' }: AvatarGroupProps) {
  const visible = names.slice(0, max)
  const extra = names.length - max
  return (
    <div className="avatar-group">
      {visible.map((name) => (
        <Avatar key={name} name={name} size={size} />
      ))}
      {extra > 0 && (
        <span className="avatar-group-count">+{extra}</span>
      )}
    </div>
  )
}
```

- [ ] **Step 4: Commit**

```
feat(ui): add Input, Badge, Avatar, AvatarGroup components
```

---

## Task 4: Modal + EmptyState + Skeleton 组件

**Files:**
- Create: `src/components/ui/Modal.tsx`
- Create: `src/components/ui/Modal.css`
- Create: `src/components/ui/EmptyState.tsx`
- Create: `src/components/ui/EmptyState.css`
- Create: `src/components/ui/Skeleton.tsx`
- Create: `src/components/ui/Skeleton.css`

- [ ] **Step 1: Create Modal.css + Modal.tsx**

Modal.css:
```css
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: modal-fade-in 150ms ease;
}
.modal-content {
  background: var(--color-bg-primary);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-lg);
  width: 100%;
  max-width: 480px;
  max-height: 90vh;
  overflow-y: auto;
  animation: modal-scale-in 150ms ease;
}
.modal-content.modal-wide {
  max-width: 720px;
}
.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-5) var(--space-6);
  border-bottom: 1px solid var(--color-border-light);
}
.modal-header h3 {
  font-size: var(--font-size-lg);
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}
.modal-close {
  background: none;
  border: none;
  font-size: 18px;
  color: var(--color-text-tertiary);
  cursor: pointer;
  padding: 4px;
  line-height: 1;
}
.modal-close:hover {
  color: var(--color-text-primary);
}
.modal-body {
  padding: var(--space-6);
}
.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: var(--space-4) var(--space-6);
  border-top: 1px solid var(--color-border-light);
}
@keyframes modal-fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}
@keyframes modal-scale-in {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}
```

Modal.tsx:
```tsx
import { useEffect } from 'react'
import './Modal.css'

interface ModalProps {
  open: boolean
  on_close: () => void
  title: string
  wide?: boolean
  children: React.ReactNode
  footer?: React.ReactNode
}

export function Modal({ open, on_close, title, wide, children, footer }: ModalProps) {
  useEffect(() => {
    if (!open) return
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') on_close()
    }
    document.addEventListener('keydown', handler)
    return () => document.removeEventListener('keydown', handler)
  }, [open, on_close])

  if (!open) return null

  return (
    <div className="modal-overlay" onClick={on_close}>
      <div
        className={`modal-content${wide ? ' modal-wide' : ''}`}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-header">
          <h3>{title}</h3>
          <button className="modal-close" onClick={on_close}>&times;</button>
        </div>
        <div className="modal-body">{children}</div>
        {footer && <div className="modal-footer">{footer}</div>}
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Create EmptyState.css + EmptyState.tsx**

EmptyState.css:
```css
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-12) var(--space-6);
  text-align: center;
}
.empty-state-icon {
  font-size: 32px;
  color: var(--color-text-tertiary);
  margin-bottom: var(--space-4);
}
.empty-state-title {
  font-size: var(--font-size-lg);
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: var(--space-2);
}
.empty-state-desc {
  font-size: var(--font-size-base);
  color: var(--color-text-secondary);
  max-width: 320px;
  margin-bottom: var(--space-6);
}
```

EmptyState.tsx:
```tsx
import './EmptyState.css'
import { Button } from './Button'

interface EmptyStateProps {
  icon?: string
  title: string
  description?: string
  action_label?: string
  on_action?: () => void
}

export function EmptyState({ icon, title, description, action_label, on_action }: EmptyStateProps) {
  return (
    <div className="empty-state">
      {icon && <div className="empty-state-icon">{icon}</div>}
      <div className="empty-state-title">{title}</div>
      {description && <div className="empty-state-desc">{description}</div>}
      {action_label && on_action && (
        <Button onClick={on_action}>{action_label}</Button>
      )}
    </div>
  )
}
```

- [ ] **Step 3: Create Skeleton.css + Skeleton.tsx**

Skeleton.css:
```css
.skeleton {
  background: var(--color-bg-tertiary);
  border-radius: var(--radius-sm);
  position: relative;
  overflow: hidden;
}
.skeleton::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, transparent, rgba(255,255,255,0.5), transparent);
  animation: skeleton-shimmer 1.5s infinite;
}
@keyframes skeleton-shimmer {
  from { transform: translateX(-100%); }
  to { transform: translateX(100%); }
}
```

Skeleton.tsx:
```tsx
import './Skeleton.css'

interface SkeletonProps {
  width?: string
  height?: string
}

export function Skeleton({ width = '100%', height = '16px' }: SkeletonProps) {
  return <div className="skeleton" style={{ width, height }} />
}
```

- [ ] **Step 4: Commit**

```
feat(ui): add Modal, EmptyState, Skeleton components
```

---

## Task 5: Tabs + NotificationBell 组件

**Files:**
- Create: `src/components/ui/Tabs.tsx`
- Create: `src/components/ui/Tabs.css`
- Create: `src/components/ui/NotificationBell.tsx`
- Create: `src/components/ui/NotificationBell.css`

- [ ] **Step 1: Create Tabs.css + Tabs.tsx**

Tabs.css:
```css
.tabs {
  display: flex;
  gap: var(--space-1);
  border-bottom: 1px solid var(--color-border);
}
.tab-item {
  padding: var(--space-2) var(--space-4);
  font-family: var(--font-sans);
  font-size: var(--font-size-base);
  color: var(--color-text-secondary);
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  transition: color 150ms ease, border-color 150ms ease;
  margin-bottom: -1px;
}
.tab-item:hover {
  color: var(--color-text-primary);
}
.tab-item.tab-active {
  color: var(--color-accent);
  font-weight: 500;
  border-bottom-color: var(--color-accent);
}
```

Tabs.tsx:
```tsx
import './Tabs.css'

interface Tab {
  key: string
  label: string
}

interface TabsProps {
  tabs: Tab[]
  active_key: string
  on_change: (key: string) => void
}

export function Tabs({ tabs, active_key, on_change }: TabsProps) {
  return (
    <div className="tabs">
      {tabs.map((tab) => (
        <button
          key={tab.key}
          className={`tab-item${tab.key === active_key ? ' tab-active' : ''}`}
          onClick={() => on_change(tab.key)}
        >
          {tab.label}
        </button>
      ))}
    </div>
  )
}
```

- [ ] **Step 2: Create NotificationBell.css + NotificationBell.tsx**

NotificationBell.css:
```css
.notification-bell {
  position: relative;
  background: none;
  border: none;
  cursor: pointer;
  padding: var(--space-2);
  color: var(--color-text-secondary);
  font-size: var(--font-size-lg);
  line-height: 1;
}
.notification-bell:hover {
  color: var(--color-text-primary);
}
.notification-badge {
  position: absolute;
  top: 0;
  right: 0;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  background: var(--color-danger);
  color: #fff;
  font-size: 10px;
  font-weight: 600;
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
}
.notification-panel {
  position: absolute;
  top: 100%;
  right: 0;
  width: 320px;
  max-height: 400px;
  overflow-y: auto;
  background: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  z-index: 100;
}
.notification-panel-header {
  padding: var(--space-3) var(--space-4);
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--color-text-primary);
  border-bottom: 1px solid var(--color-border-light);
}
.notification-item {
  display: flex;
  align-items: flex-start;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  cursor: pointer;
  transition: background 100ms ease;
}
.notification-item:hover {
  background: var(--color-bg-tertiary);
}
.notification-item-title {
  font-size: var(--font-size-sm);
  color: var(--color-text-primary);
  font-weight: 500;
}
.notification-item-time {
  font-size: var(--font-size-xs);
  color: var(--color-text-tertiary);
}
.notification-empty {
  padding: var(--space-8) var(--space-4);
  text-align: center;
  color: var(--color-text-tertiary);
  font-size: var(--font-size-sm);
}
```

NotificationBell.tsx:
```tsx
import { useState, useRef, useEffect } from 'react'
import './NotificationBell.css'

export interface NotificationItem {
  id: string
  title: string
  time: string
  target_path: string
}

interface NotificationBellProps {
  items: NotificationItem[]
  on_click: (item: NotificationItem) => void
}

export function NotificationBell({ items, on_click }: NotificationBellProps) {
  const [open, set_open] = useState(false)
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!open) return
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) set_open(false)
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [open])

  const unread = items.length

  return (
    <div ref={ref} style={{ position: 'relative' }}>
      <button className="notification-bell" onClick={() => set_open(!open)}>
        🔔
        {unread > 0 && <span className="notification-badge">{unread > 9 ? '9+' : unread}</span>}
      </button>
      {open && (
        <div className="notification-panel">
          <div className="notification-panel-header">通知</div>
          {items.length === 0 ? (
            <div className="notification-empty">没有新通知</div>
          ) : (
            items.map((item) => (
              <div
                key={item.id}
                className="notification-item"
                onClick={() => {
                  on_click(item)
                  set_open(false)
                }}
              >
                <div>
                  <div className="notification-item-title">{item.title}</div>
                  <div className="notification-item-time">{item.time}</div>
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  )
}
```

- [ ] **Step 3: Commit**

```
feat(ui): add Tabs, NotificationBell components
```

---

## Task 6: AppShell 布局框架

**Files:**
- Create: `src/components/layout/AppShell.tsx`
- Create: `src/components/layout/AppShell.css`
- Create: `src/components/layout/HubNavBar.tsx`
- Create: `src/components/layout/HubNavBar.css`
- Modify: `src/App.tsx`

This task creates the top-level layout shell. It replaces the old `Layout.tsx` + `NavBar.tsx` + `RingNavBar.tsx` with a single unified system.

- [ ] **Step 1: Create HubNavBar.css + HubNavBar.tsx**

HubNavBar.css:
```css
.hub-navbar {
  display: flex;
  align-items: center;
  padding: 0 var(--space-6);
  height: 48px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-bg-primary);
}
.hub-navbar-brand {
  font-family: var(--font-sans);
  font-size: var(--font-size-lg);
  font-weight: 600;
  color: var(--color-accent);
  text-decoration: none;
  letter-spacing: -0.5px;
}
.hub-navbar-links {
  display: flex;
  gap: var(--space-1);
  margin-left: var(--space-8);
}
.hub-nav-link {
  padding: var(--space-1) var(--space-3);
  border-radius: var(--radius-md);
  text-decoration: none;
  font-size: var(--font-size-base);
  color: var(--color-text-secondary);
  transition: background 150ms ease, color 150ms ease;
}
.hub-nav-link:hover {
  background: var(--color-accent-light);
  color: var(--color-text-primary);
}
.hub-nav-link.hub-nav-active {
  background: var(--color-accent-light);
  color: var(--color-accent);
  font-weight: 500;
}
.hub-navbar-right {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: var(--space-2);
}
```

HubNavBar.tsx:
```tsx
import { NavLink } from 'react-router-dom'
import { NotificationBell } from '../ui/NotificationBell'
import type { NotificationItem } from '../ui/NotificationBell'
import './HubNavBar.css'

interface HubNavBarProps {
  notifications?: NotificationItem[]
  on_notification_click?: (item: NotificationItem) => void
}

export function HubNavBar({ notifications = [], on_notification_click }: HubNavBarProps) {
  return (
    <nav className="hub-navbar">
      <NavLink to="/" className="hub-navbar-brand">Ring</NavLink>
      <div className="hub-navbar-links">
        <NavLink
          to="/"
          end
          className={({ isActive }) => `hub-nav-link${isActive ? ' hub-nav-active' : ''}`}
        >
          Ring Hub
        </NavLink>
        <NavLink
          to="/super-ring"
          className={({ isActive }) => `hub-nav-link${isActive ? ' hub-nav-active' : ''}`}
        >
          Super Ring
        </NavLink>
      </div>
      <div className="hub-navbar-right">
        <NotificationBell items={notifications} on_click={on_notification_click || (() => {})} />
        <NavLink to="/settings" className={({ isActive }) => `hub-nav-link${isActive ? ' hub-nav-active' : ''}`}>
          Settings
        </NavLink>
      </div>
    </nav>
  )
}
```

- [ ] **Step 2: Create AppShell.css + AppShell.tsx**

AppShell.css:
```css
.app-shell {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  width: 100%;
  max-width: 100%;
}
.app-shell-body {
  flex: 1;
  display: flex;
  flex-direction: column;
}
```

AppShell.tsx:
```tsx
import { Outlet } from 'react-router-dom'
import { HubNavBar } from './HubNavBar'
import './AppShell.css'

export function AppShell() {
  return (
    <div className="app-shell">
      <HubNavBar />
      <div className="app-shell-body">
        <Outlet />
      </div>
    </div>
  )
}
```

- [ ] **Step 3: Update App.tsx to use AppShell**

Replace the `Layout` import with `AppShell`. Remove the `Layout` wrapper Route. Nest non-ring routes directly under `SetupGuard`. Ring routes get their own `SetupGuard` wrapping `RingSpaceLayout` (which is inside AppShell).

Rewrite `src/App.tsx`. The routing structure splits into two top-level branches:
- Hub routes (`/`, `/super-ring`, `/settings`) render inside `<AppShell>` which provides `<HubNavBar>` + `<Outlet />`
- Ring routes (`/ring/:ringId/*`) render inside `<RingSpaceLayout>` which provides its own header + sidebar + `<Outlet />`

```tsx
import { useEffect, useState } from 'react'
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { SetupWizard } from './pages/Setup/SetupWizard'
import { RingHub } from './pages/RingHub/RingHub'
import { ChatView } from './pages/RingSpace/ChatView'
import { BlueprintWizard } from './pages/RingSpace/BlueprintWizard'
import { GraphView } from './pages/RingSpace/GraphView'
import { PrList } from './pages/RingSpace/PrList'
import { PrDetail } from './pages/RingSpace/PrDetail'
import { SuperRingChat } from './pages/RingHub/SuperRingChat'
import { MemberList } from './components/member/MemberList'
import { SessionView } from './components/session/SessionView'
import { SettingsPage } from './pages/Settings/SettingsPage'
import { AppShell } from './components/layout/AppShell'
import { RingSpaceLayout } from './components/layout/RingSpaceLayout'
import { get_setup_status } from './api/client'
import { Toast } from './components/Toast'

function SetupGuard({ children }: { children: React.ReactNode }) {
  const [checking, set_checking] = useState(true)
  const [completed, set_completed] = useState(false)

  useEffect(() => {
    get_setup_status()
      .then((status) => {
        if (status.user_id) localStorage.setItem('ring_user_id', status.user_id)
        set_completed(status.setup_completed)
        set_checking(false)
      })
      .catch(() => set_checking(false))
  }, [])

  if (checking) return <div className="spinner-container"><div className="spinner" /></div>
  if (!completed) return <Navigate to="/setup" replace />
  return <>{children}</>
}

function SetupWizardRedirect() {
  const [checking, set_checking] = useState(true)
  const [completed, set_completed] = useState(false)

  useEffect(() => {
    get_setup_status()
      .then((status) => {
        if (status.user_id) localStorage.setItem('ring_user_id', status.user_id)
        set_completed(status.setup_completed)
        set_checking(false)
      })
      .catch(() => set_checking(false))
  }, [])

  if (checking) return <div className="spinner-container"><div className="spinner" /></div>
  if (completed) return <Navigate to="/" replace />
  return <SetupWizard />
}

export default function App() {
  return (
    <BrowserRouter>
      <Toast />
      <Routes>
        <Route path="/setup" element={<SetupWizardRedirect />} />
        <Route
          element={<SetupGuard><AppShell /></SetupGuard>}
        >
          <Route path="/" element={<RingHub />} />
          <Route path="/super-ring" element={<SuperRingChat />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Route>
        <Route
          path="/ring/:ringId"
          element={<SetupGuard><RingSpaceLayout /></SetupGuard>}
        >
          <Route index element={<ChatView />} />
          <Route path="blueprint" element={<BlueprintWizard />} />
          <Route path="graph" element={<GraphView />} />
          <Route path="prs" element={<PrList />} />
          <Route path="prs/:prId" element={<PrDetail />} />
          <Route path="members" element={<MemberList />} />
          <Route path="sessions" element={<SessionView />} />
          <Route path="sessions/:sessionId" element={<SessionView />} />
        </Route>
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  )
}
```

- [ ] **Step 4: Run build to verify routing works**

Run: `cd ring-frontend && npm run build`
Expected: Build succeeds

- [ ] **Step 5: Commit**

```
feat(ui): add AppShell layout with HubNavBar
```

---

## Task 7: RingSpaceLayout 三栏布局

**Files:**
- Create: `src/components/layout/RingSpaceLayout.tsx`
- Create: `src/components/layout/RingSpaceLayout.css`
- Create: `src/components/layout/RingSidebar.tsx`
- Create: `src/components/layout/RingSidebar.css`
- Create: `src/components/layout/RightPanel.tsx`
- Create: `src/components/layout/RightPanel.css`

- [ ] **Step 1: Create RingSpaceLayout.css + RingSpaceLayout.tsx**

RingSpaceLayout.css:
```css
.ring-space {
  display: flex;
  flex-direction: column;
  height: 100vh;
}
.ring-space-header {
  display: flex;
  align-items: center;
  padding: 0 var(--space-4) 0 0;
  height: 48px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-bg-primary);
  flex-shrink: 0;
}
.ring-space-back {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: 0 var(--space-4);
  height: 100%;
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  text-decoration: none;
  border-right: 1px solid var(--color-border);
  transition: color 150ms ease;
}
.ring-space-back:hover {
  color: var(--color-accent);
}
.ring-space-name {
  font-size: var(--font-size-md);
  font-weight: 600;
  color: var(--color-text-primary);
  padding: 0 var(--space-4);
}
.ring-space-header-right {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: var(--space-3);
}
.ring-space-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}
.ring-space-main {
  flex: 1;
  overflow: auto;
  display: flex;
  flex-direction: column;
}
```

RingSpaceLayout.tsx:
```tsx
import { useState, createContext, useContext } from 'react'
import { useParams, Outlet, Link } from 'react-router-dom'
import { AvatarGroup } from '../ui/AvatarGroup'
import { NotificationBell, type NotificationItem } from '../ui/NotificationBell'
import { RingSidebar } from './RingSidebar'
import { RightPanel } from './RightPanel'
import './RingSpaceLayout.css'

interface RightPanelState {
  open: boolean
  content: 'node_detail' | 'diff' | 'node_selector' | null
  data: unknown
}

const RightPanelContext = createContext<{
  panel: RightPanelState
  set_panel: (s: RightPanelState) => void
}>({
  panel: { open: false, content: null, data: null },
  set_panel: () => {},
})

export function useRightPanel() {
  return useContext(RightPanelContext)
}

export function RingSpaceLayout() {
  const { ringId } = useParams<{ ringId: string }>()
  const [panel, set_panel] = useState<RightPanelState>({
    open: false,
    content: null,
    data: null,
  })
  const [sidebar_collapsed, set_sidebar_collapsed] = useState(false)

  return (
    <RightPanelContext.Provider value={{ panel, set_panel }}>
      <div className="ring-space">
        <div className="ring-space-header">
          <Link to="/" className="ring-space-back">&larr; Hub</Link>
          <div className="ring-space-name">Ring</div>
          <div className="ring-space-header-right">
            <AvatarGroup names={[]} size="sm" />
            <NotificationBell items={[]} on_click={() => {}} />
          </div>
        </div>
        <div className="ring-space-body">
          <RingSidebar collapsed={sidebar_collapsed} on_toggle={() => set_sidebar_collapsed(!sidebar_collapsed)} />
          <div className="ring-space-main">
            <Outlet />
          </div>
          {panel.open && <RightPanel state={panel} on_close={() => set_panel({ open: false, content: null, data: null })} />}
        </div>
      </div>
    </RightPanelContext.Provider>
  )
}
```

Note: `<AvatarGroup names={[]} />` and `<NotificationBell items={[]} />` are placeholders. Real member data and notifications will be wired from store/page context when those pages are rebuilt.

- [ ] **Step 2: Create RingSidebar.css + RingSidebar.tsx**

RingSidebar.css:
```css
.ring-sidebar {
  width: 240px;
  border-right: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  transition: width 200ms ease;
  overflow: hidden;
}
.ring-sidebar.ring-sidebar-collapsed {
  width: 48px;
}
.ring-sidebar-graph-select {
  padding: var(--space-2);
  border-bottom: 1px solid var(--color-border-light);
}
.ring-sidebar-tree {
  flex: 1;
  overflow: auto;
}
.ring-sidebar-divider {
  height: 1px;
  background: var(--color-border-light);
  margin: 0;
}
.ring-sidebar-nav {
  padding: var(--space-2);
}
.ring-sidebar-nav-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-1) var(--space-3);
  border-radius: var(--radius-md);
  font-size: var(--font-size-base);
  color: var(--color-text-secondary);
  text-decoration: none;
  cursor: pointer;
  transition: background 150ms ease, color 150ms ease;
  margin-bottom: 2px;
}
.ring-sidebar-nav-item:hover {
  background: var(--color-accent-light);
  color: var(--color-text-primary);
}
.ring-sidebar-nav-item.sidebar-active {
  background: var(--color-accent-light);
  color: var(--color-accent);
  font-weight: 500;
}
.ring-sidebar-nav-badge {
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  background: var(--color-danger);
  color: #fff;
  font-size: 10px;
  font-weight: 600;
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
}
.ring-sidebar-collapse-btn {
  padding: var(--space-2) var(--space-3);
  border: none;
  background: none;
  color: var(--color-text-tertiary);
  cursor: pointer;
  font-size: var(--font-size-sm);
  border-top: 1px solid var(--color-border-light);
  text-align: left;
}
.ring-sidebar-collapse-btn:hover {
  color: var(--color-text-primary);
}
```

RingSidebar.tsx:
```tsx
import { useParams, NavLink, useLocation } from 'react-router-dom'
import './RingSidebar.css'

interface RingSidebarProps {
  collapsed: boolean
  on_toggle: () => void
}

const NAV_ITEMS = [
  { path: '', label: 'Chat', icon: '💬' },
  { path: '/graph', label: 'Graph', icon: '◉' },
  { path: '/prs', label: 'PRs', icon: '📋' },
  { path: '/members', label: 'Members', icon: '👥' },
  { path: '/sessions', label: 'Sessions', icon: '🔍' },
]

export function RingSidebar({ collapsed, on_toggle }: RingSidebarProps) {
  const { ringId } = useParams<{ ringId: string }>()
  const location = useLocation()
  if (!ringId) return null

  return (
    <div className={`ring-sidebar${collapsed ? ' ring-sidebar-collapsed' : ''}`}>
      <div className="ring-sidebar-tree">
        {collapsed ? null : (
          <div style={{ padding: 'var(--space-2)', color: 'var(--color-text-tertiary)', fontSize: 'var(--font-size-xs)' }}>
            图谱节点树（待数据接入）
          </div>
        )}
      </div>
      <div className="ring-sidebar-divider" />
      <div className="ring-sidebar-nav">
        {NAV_ITEMS.map((item) => {
          const to = `/ring/${ringId}${item.path}`
          const is_active = item.path === ''
            ? location.pathname === `/ring/${ringId}` || location.pathname === `/ring/${ringId}/`
            : location.pathname.startsWith(to)
          return (
            <NavLink
              key={item.path}
              to={to}
              end={item.path === ''}
              className={`ring-sidebar-nav-item${is_active ? ' sidebar-active' : ''}`}
              title={collapsed ? item.label : undefined}
            >
              <span>{collapsed ? item.icon : item.label}</span>
              {item.path === '/prs' && !collapsed && (
                <span className="ring-sidebar-nav-badge">0</span>
              )}
            </NavLink>
          )
        })}
      </div>
      <button className="ring-sidebar-collapse-btn" onClick={on_toggle}>
        {collapsed ? '→' : '← 收起'}
      </button>
    </div>
  )
}
```

Note: The tree section shows a placeholder. The actual NodeTree component will be rendered here when wired with graphStore data during the GraphView rebuild (Task 11). The PR badge count "0" is a placeholder — real data comes from gitStore.

- [ ] **Step 3: Create RightPanel.css + RightPanel.tsx**

RightPanel.css:
```css
.right-panel {
  width: 280px;
  border-left: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  animation: right-panel-slide 200ms ease-out;
}
.right-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--color-border-light);
}
.right-panel-header h4 {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.right-panel-close {
  background: none;
  border: none;
  color: var(--color-text-tertiary);
  cursor: pointer;
  font-size: var(--font-size-lg);
  padding: 2px;
}
.right-panel-close:hover {
  color: var(--color-text-primary);
}
.right-panel-body {
  flex: 1;
  overflow: auto;
  padding: var(--space-4);
}
@keyframes right-panel-slide {
  from { transform: translateX(280px); opacity: 0; }
  to { transform: translateX(0); opacity: 1; }
}
```

RightPanel.tsx:
```tsx
import './RightPanel.css'

interface RightPanelState {
  open: boolean
  content: 'node_detail' | 'diff' | 'node_selector' | null
  data: unknown
}

interface RightPanelProps {
  state: RightPanelState
  on_close: () => void
}

export function RightPanel({ state, on_close }: RightPanelProps) {
  const titles: Record<string, string> = {
    node_detail: '节点详情',
    diff: 'Changes',
    node_selector: '选择节点',
  }

  return (
    <div className="right-panel">
      <div className="right-panel-header">
        <h4>{state.content ? titles[state.content] || '' : ''}</h4>
        <button className="right-panel-close" onClick={on_close}>&times;</button>
      </div>
      <div className="right-panel-body">
        {state.content === 'node_detail' && (
          <div style={{ fontSize: 'var(--font-size-sm)', color: 'var(--color-text-secondary)' }}>
            {state.data ? JSON.stringify(state.data) : 'No data'}
          </div>
        )}
        {state.content === 'diff' && (
          <div style={{ fontSize: 'var(--font-size-sm)', color: 'var(--color-text-secondary)' }}>
            Diff view（待接入 DiffView 组件）
          </div>
        )}
        {state.content === 'node_selector' && (
          <div style={{ fontSize: 'var(--font-size-sm)', color: 'var(--color-text-secondary)' }}>
            节点选择器（待接入 NodeTree 组件）
          </div>
        )}
      </div>
    </div>
  )
}
```

Note: Right panel content is stubbed. Each page component will call `useRightPanel().set_panel(...)` with real data. Full rendering of NodeTree/DiffView inside RightPanel happens during feature page rebuilds.

- [ ] **Step 4: Run build**

Run: `cd ring-frontend && npm run build`
Expected: Build succeeds

- [ ] **Step 5: Commit**

```
feat(ui): add RingSpaceLayout with sidebar and right panel
```

---

## Task 8: Rebuild Setup Wizard

**Files:**
- Modify: `src/pages/Setup/SetupWizard.tsx`
- Modify: `src/pages/Setup/StepUsername.tsx`
- Modify: `src/pages/Setup/StepLlm.tsx`
- Modify: `src/pages/Setup/StepGitlab.tsx`

- [ ] **Step 1: Rewrite SetupWizard.tsx**

Remove all inline styles. Use the new `Button`, `Input` components. Center card layout. Add step indicator dots.

The page should render a centered card (max-width 420px) with:
- Title "Welcome to Ring"
- Step indicator: 3 dots connected by lines, current highlighted
- Step content (each step component)
- Back / Next buttons at bottom

All form components use `<Input>` instead of raw `<input>` / `<select>` / `<textarea>`.

- [ ] **Step 2: Rewrite StepUsername, StepLlm, StepGitlab**

Replace all inline styles with CSS classes. Use `<Input>` component for all fields. Use `<Input input_type="select">` for selects. Use `<Input input_type="textarea">` for role_description. Use `<Button>` for submit buttons.

For each step file, remove all `style={{...}}` attributes and use the shared components.

- [ ] **Step 3: Run tests**

Run: `cd ring-frontend && npm test`
Expected: All existing Setup tests pass (they test functionality, not styles)

- [ ] **Step 4: Commit**

```
feat(ui): rebuild Setup Wizard with design tokens
```

---

## Task 9: Rebuild Ring Hub

**Files:**
- Modify: `src/pages/RingHub/RingHub.tsx`
- Modify: `src/pages/RingHub/RingList.tsx`
- Modify: `src/pages/RingHub/CreateRing.tsx`
- Modify: `src/pages/RingHub/SuperRingChat.tsx`

- [ ] **Step 1: Rewrite RingHub.tsx**

- Title section: h1 "Ring Hub" + subtitle "你的群组知识协作空间"
- Create Ring button (primary)
- Ring card grid (CSS grid, 2 columns, min 320px)
- Footer note: "对话记录仅保存在当前设备"
- Remove all inline styles

- [ ] **Step 2: Rewrite RingList.tsx**

Each Ring card should show:
- Color dot + Ring name (font-weight 600)
- Role description (truncated, color-text-secondary)
- Last activity (relative time, color-text-tertiary)
- Divider line
- Bottom row: node count + role Badge
- Card hover effect: border-color accent + shadow-sm

Use `<Badge status={ring.role}>{ring.role}</Badge>` for role.
Use `<EmptyState>` when no rings.

- [ ] **Step 3: Rewrite CreateRing.tsx as Modal**

Instead of inline form, use `<Modal>` component:
- `open` state controlled by button
- Modal title: "Create Ring"
- Body: Name input + Description input + Role description textarea
- Footer: Cancel (secondary) + Create (primary) buttons
- On success: close modal + navigate to blueprint

- [ ] **Step 4: Rewrite SuperRingChat.tsx**

Full-width chat layout with:
- Title bar: "Super Ring" + subtitle "全局助手"
- Message area (flex: 1, overflow auto)
- Bottom: ChatInput + Send button
- Remove all inline styles

- [ ] **Step 5: Run tests**

Run: `cd ring-frontend && npm test`
Expected: All RingHub tests pass

- [ ] **Step 6: Commit**

```
feat(ui): rebuild Ring Hub pages with design system
```

---

## Task 10: Rebuild Ring Space Chat + ArchiveSuggestion

**Files:**
- Modify: `src/pages/RingSpace/ChatView.tsx`
- Modify: `src/components/chat/ChatBubble.tsx`
- Modify: `src/components/chat/ChatInput.tsx`
- Modify: `src/components/chat/ToolCallBubble.tsx`
- Modify: `src/components/chat/ToolResultBubble.tsx`
- Modify: `src/components/chat/ArchiveSuggestion.tsx`
- Modify: `src/components/toolbar/Toolbar.tsx`
- Create: `src/components/chat/ChatBubble.css`
- Create: `src/components/chat/ChatInput.css`
- Create: `src/components/chat/ToolCallBubble.css`
- Create: `src/components/chat/ToolResultBubble.css`
- Create: `src/components/chat/ArchiveSuggestion.css`
- Create: `src/components/toolbar/Toolbar.css`

- [ ] **Step 1: Create CSS files for all chat components**

For each chat component, extract inline styles to a dedicated CSS file using token variables. Then update the TSX to use className instead of style={{}}.

ChatBubble.css key classes:
- `.chat-bubble-row` (flex container, justify end/start)
- `.chat-bubble-user` (accent bg, white text, border-radius)
- `.chat-bubble-assistant` (accent-light bg, primary text)
- `.chat-bubble-avatar` (small avatar circle)
- `.chat-bubble-content` (max-width 70%, padding)

ChatInput.css:
- `.chat-input-form` (flex row)
- `.chat-input-field` (flex: 1)
- `.chat-input-send` (primary button)

ArchiveSuggestion.css:
- `.archive-suggestion` (border-left: 3px solid accent, accent-light bg)
- `.archive-suggestion-title`
- `.archive-suggestion-path`
- `.archive-suggestion-actions`

- [ ] **Step 2: Rewrite ArchiveSuggestion.tsx**

Per spec section 3.12:
- Blue left border card (not orange)
- Shows: AI recommendation description + target node path
- Three buttons: "确认归档" (primary) / "换个位置" (secondary) / "跳过" (ghost)
- Add `on_relocate` prop for "换个位置" action
- Keep `on_accept` and `on_dismiss` props

- [ ] **Step 3: Rewrite ChatView.tsx**

Adapt to work within RingSpaceLayout (no full-height self-management, it's inside `<Outlet />`):
- Remove `height: '80vh'` wrapper — it should fill the flex column from RingSpaceLayout
- Use `.ring-space-main` flex column
- Chat header bar: "Chat" label + Auto mode toggle
- Messages area: flex: 1, overflow auto
- Bottom bar: Toolbar + ChatInput
- Wire ArchiveSuggestion with `useRightPanel()` — when "换个位置" is clicked, open right panel with `content: 'node_selector'`

- [ ] **Step 4: Update Toolbar.tsx to use token styles**

Extract inline styles to Toolbar.css. Use token colors.

- [ ] **Step 5: Run tests**

Run: `cd ring-frontend && npm test`
Expected: All ChatView tests pass

- [ ] **Step 6: Commit**

```
feat(ui): rebuild chat components with design system
```

---

## Task 11: Rebuild Graph + NodeTree + ForceGraph

**Files:**
- Modify: `src/pages/RingSpace/GraphView.tsx`
- Modify: `src/components/graph/NodeTree.tsx`
- Modify: `src/components/graph/ForceGraph.tsx`
- Create: `src/components/graph/NodeTree.css`
- Create: `src/components/graph/ForceGraph.css`

- [ ] **Step 1: Create NodeTree.css + update NodeTree.tsx**

Extract styles from inline to CSS:
- `.tree-node-row` (padding, cursor, hover, selected state)
- `.tree-node-selected` (accent-light bg)
- `.tree-node-highlighted` (accent-light bg for archive suggestions)
- Add `highlighted_node_id` prop to NodeTree interface

NodeTree props become:
```tsx
interface NodeTreeProps {
  nodes: GraphNode[]
  selected_node_id: string | null
  highlighted_node_id?: string | null
  on_select: (node_id: string) => void
}
```

- [ ] **Step 2: Update ForceGraph.tsx colors**

Update `NODE_COLORS` to use token-appropriate hex colors matching the design system:
```tsx
const NODE_COLORS: Record<string, string> = {
  concept: '#2563EB',
  category: '#D97706',
  document: '#16A34A',
  event: '#7C3AED',
  person: '#0891B2',
  task: '#DC2626',
}
```

Update selected node stroke to use accent color.

- [ ] **Step 3: Rewrite GraphView.tsx**

Remove the `height: 100vh` hack. Instead, the graph fills the RingSpaceLayout's center column:
- Graph header bar: "Graph" + graph name + export button
- D3 force graph fills remaining space (flex: 1)
- No left sidebar tree (that's now in RingSidebar) — but GraphView should accept nodes/edges from graphStore as before
- Node click opens RightPanel via `useRightPanel()`

Remove the old left panel (240px sidebar) from GraphView since RingSidebar now handles that.

- [ ] **Step 4: Run tests**

Run: `cd ring-frontend && npm test`
Expected: All GraphView tests pass

- [ ] **Step 5: Commit**

```
feat(ui): rebuild Graph view with design system
```

---

## Task 12: Rebuild PR pages + DiffView

**Files:**
- Modify: `src/pages/RingSpace/PrList.tsx`
- Modify: `src/pages/RingSpace/PrDetail.tsx`
- Modify: `src/components/git/DiffView.tsx`
- Create: `src/components/git/DiffView.css`

- [ ] **Step 1: Create DiffView.css + update DiffView.tsx**

Extract all inline styles to DiffView.css using tokens. Update file header bg/status colors to use token variables.

- [ ] **Step 2: Rewrite PrList.tsx**

- PR header: "PRs" title + Tabs component for Opened/Merged/Closed filter
- PR list: compact rows with Badge for state + title + author + time
- Click navigates to PR detail
- Empty state when no PRs

- [ ] **Step 3: Rewrite PrDetail.tsx**

- Back link to PR list
- Title row: #pr_id + title + author
- Action buttons: Merge (primary) + Reject (danger) using Button component
- DiffView below

- [ ] **Step 4: Run tests**

Run: `cd ring-frontend && npm test`
Expected: All PR tests pass

- [ ] **Step 5: Commit**

```
feat(ui): rebuild PR pages with design system
```

---

## Task 13: Rebuild Members + Sessions

**Files:**
- Modify: `src/components/member/MemberList.tsx`
- Modify: `src/components/session/SessionView.tsx`
- Create: `src/components/member/MemberList.css`
- Create: `src/components/session/SessionView.css`

- [ ] **Step 1: Create MemberList.css + rewrite MemberList.tsx**

- Header: "Members" + invite button
- Invite link display in accent-light card
- Member list: card-based layout instead of table
  - Each member: Avatar + name + Badge(role) + joined time
  - Role change dropdown + remove button for non-creators
- Use Avatar, Badge, Button, Input components

- [ ] **Step 2: Create SessionView.css + rewrite SessionView.tsx**

- Session list view:
  - Header: "Sessions" + New Session button
  - Session cards: status Badge + title + member count + archive badge
  - Action buttons: Close / Archive toggle / Delete
- New Session form (inline, not modal):
  - Title input + Scenario select + Create button
- Session chat view:
  - Same layout as ChatView (messages + input)
  - Back button to list

- [ ] **Step 3: Run tests**

Run: `cd ring-frontend && npm test`
Expected: All tests pass

- [ ] **Step 4: Commit**

```
feat(ui): rebuild Members and Sessions with design system
```

---

## Task 14: Rebuild Blueprint Wizard + Settings + Super Ring Chat

**Files:**
- Modify: `src/pages/RingSpace/BlueprintWizard.tsx`
- Modify: `src/pages/Settings/SettingsPage.tsx`

- [ ] **Step 1: Rewrite BlueprintWizard.tsx**

- Tabs: "模板" / "自定义" using Tabs component
- Template mode: Card grid (2-3 cols) with template cards
  - Each card: name + description + click to preview
  - Preview panel: accent border card showing graph list
  - "使用此模板" button
- Custom mode: Chat interface + preview panel side by side
- All inline styles removed

- [ ] **Step 2: Rewrite SettingsPage.tsx**

- Centered single column (max-width 560px)
- Profile section: card with display name + user id (read-only)
- LLM section: card with form fields (provider select, model input, api key password, base url input, privacy checkbox)
- Use Input, Button components
- Section gaps: var(--space-8)

- [ ] **Step 3: Run tests**

Run: `cd ring-frontend && npm test`
Expected: All tests pass

- [ ] **Step 4: Commit**

```
feat(ui): rebuild Blueprint Wizard and Settings pages
```

---

## Task 15: Cleanup — delete old layout files + verify

**Files:**
- Delete: `src/components/layout/Layout.tsx`
- Delete: `src/components/layout/NavBar.tsx`
- Delete: `src/components/layout/RingNavBar.tsx`

- [ ] **Step 1: Delete old layout files**

These are replaced by `AppShell.tsx`, `HubNavBar.tsx`, `RingSpaceLayout.tsx`.

- [ ] **Step 2: Verify no imports of deleted files**

Search codebase for any remaining imports of `Layout`, `NavBar`, `RingNavBar` from the old paths. Fix any found.

Run: `cd ring-frontend && npm run build`
Expected: Build succeeds with no errors

- [ ] **Step 3: Run full test suite**

Run: `cd ring-frontend && npm test`
Expected: All tests pass

- [ ] **Step 4: Run lint**

Run: `cd ring-frontend && npm run lint`
Expected: No lint errors

- [ ] **Step 5: Final commit**

```
chore(ui): remove old layout components, finalize UI rebuild
```
