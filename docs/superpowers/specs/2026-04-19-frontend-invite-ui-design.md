# Frontend Invite UI 设计

> **Affects**: `ui/src/types/invite.ts`, `ui/src/stores/invite-store.ts`, `ui/src/components/common/Modal.tsx`, `ui/src/components/invite/CreateInviteModal.tsx`, `ui/src/components/panels/ConfigPanel.tsx`, `ui/src/components/setup/StepWelcome.tsx`, `ui/src/components/setup/StepJoin.tsx`, `ui/src/components/setup/SetupWizard.tsx`, `ui/src/App.tsx`, `ui/src/services/api.ts`
> **Depends on**: 邀请 Token API（已实现）、开放链接加入（已实现）、审核链接 + 审批流程（已实现）、安装导航页（已实现）
> **Last verified**: 2026-04-19

## 1. 概述

邀请/加入流程的前端界面，覆盖两个用户角色：

- **Creator/Admin**：在 ConfigPanel 中创建邀请链接、管理 token、审批加入申请
- **Joiner**：通过 SetupWizard 的 "Join Existing" 分支或 URL 参数进入加入流程

### 1.1 架构

```
Creator 端:
  ConfigPanel (Members 区域)
    ├── "+ invite member" 按钮 → 打开 CreateInviteModal
    ├── Active Invites 区域 → 列出 token + revoke 按钮
    └── Pending Requests 区域 → 列出申请 + approve/reject

  CreateInviteModal (Modal Overlay)
    ├── 状态 1: 表单（type / role / max_uses / max_members / expires）
    └── 状态 2: 链接展示 + COPY + CREATE ANOTHER / DONE

Joiner 端:
  SetupWizard
    ├── 入口 A: StepWelcome "Join Existing" → StepJoin
    └── 入口 B: URL ?token=xxx&creator_ip=xxx → 自动进入 join

  StepJoin
    ├── 输入 invite link 或自动填入 URL 参数
    ├── 显示 Ring 信息（名称、成员数）
    ├── 输入 display_name
    └── JOIN（open: 直接加入 / audit: 提交申请 → 轮询状态）
```

### 1.2 设计决策

| 决策项 | 选择 | 理由 |
|--------|------|------|
| 邀请创建交互 | Modal Overlay | 表单参数多，320px 面板空间不足 |
| 通用 Modal 组件 | 新建 | 代码库无现有 modal，需建立此模式 |
| Token 管理位置 | ConfigPanel 内 | 与 Members 区域同属成员管理范畴 |
| 审批操作位置 | ConfigPanel 内 | creator 在 Config 中管理所有成员相关操作 |
| Joiner 入口 | SetupWizard 分支 + URL 参数 | 匹配设计稿 setup-flow.html 的 "Join Existing" 分支 |
| URL 参数检测 | App.tsx 根组件 | 安装导航页"继续加入"跳转到 localhost 带 query params |

## 2. TypeScript 类型

### 2.1 `ui/src/types/invite.ts`（新建）

```typescript
export interface InviteToken {
  token: string
  ring_id: string
  type: 'open' | 'audit'
  role: string
  max_uses: number
  use_count: number
  max_members: number | null
  expires_at: string
  revoked_at: string | null
  created_by: string
  created_at: string
}

export interface JoinRequest {
  id: string
  ring_id: string
  invite_token: string
  display_name: string
  message: string | null
  status: 'pending' | 'approved' | 'rejected'
  reviewer_id: string | null
  review_note: string | null
  reviewed_at: string | null
  created_at: string
}

export interface CreateInviteInput {
  type: 'open' | 'audit'
  role?: string
  max_uses?: number
  max_members?: number | null
  expires_in_hours?: number
}

export interface JoinInfo {
  valid: boolean
  reason?: string
  ring_id?: string
  ring_name?: string
  member_count?: number
  role?: string
  token_type?: string
}
```

## 3. Zustand Store

### 3.1 `ui/src/stores/invite-store.ts`（新建）

```
状态:
  tokens: InviteToken[]
  join_requests: JoinRequest[]
  loading: boolean
  modal_open: boolean

Actions:
  fetch_tokens(ring_id)     — GET /api/rings/{ring_id}/invite-tokens
  create_token(ring_id, input) — POST /api/rings/{ring_id}/invite-tokens
  revoke_token(ring_id, token) — DELETE /api/rings/{ring_id}/invite-tokens/{token}
  fetch_requests(ring_id)   — GET /api/rings/{ring_id}/join-requests?status=pending
  approve_request(ring_id, request_id) — POST .../approve
  reject_request(ring_id, request_id, note?) — POST .../reject
  open_modal() / close_modal()
```

## 4. Creator 端组件

### 4.1 Modal 组件（`ui/src/components/common/Modal.tsx`，新建）

通用 Modal 容器：
- `position: fixed`，`inset: 0`，`z-index: 1000`
- 半透明黑色遮罩（`rgba(0,0,0,0.5)`），点击遮罩关闭
- 居中白色容器，`max-width: 480px`，`border-radius: 8px`
- Props: `open: boolean`，`on_close: () => void`，`children: ReactNode`

### 4.2 CreateInviteModal（`ui/src/components/invite/CreateInviteModal.tsx`，新建）

两个渲染状态：

**状态 1 — 表单：**
- Link Type: open / audit 二选一（卡片式）
- Role: member / admin / readonly 三选一（按钮组）
- Max Uses: 数字输入，默认 1
- Max Members: 数字输入，默认无限制
- Expires: 数字输入 + 单位选择（小时），默认 24h
- GENERATE LINK 按钮

**状态 2 — 成功：**
- 绿色标题 "✓ LINK CREATED"
- 链接展示区：`http://{creator_ip}:7420/ring/join?token={token}`
- COPY 按钮（`navigator.clipboard.writeText`）
- 摘要：type / role / uses / expires
- CREATE ANOTHER（回到状态 1）/ DONE（关闭 modal）

### 4.3 ConfigPanel 更新（`ui/src/components/panels/ConfigPanel.tsx`，修改）

现有布局保持 LLM Config + Members 不变。新增：

**Members 区域末尾**：`+ invite member` 按钮（仅 creator/admin 可见）
  → 点击调用 `invite_store.open_modal()`

**Active Invites 区域**（Members 下方）：
- 标题：`ACTIVE INVITES · {count}`
- 每条 token 一行：类型标签（open=cyan / audit=amber）+ 用量 + 剩余时间 + revoke 按钮
- 无 active token 时不显示此区域

**Pending Requests 区域**（仅当有 pending requests 时显示）：
- 标题：`PENDING REQUESTS · {count}`
- 每条申请一个卡片：申请人名 + 申请时间 + 申请理由 + APPROVE / REJECT 按钮
- APPROVE 调用 `invite_store.approve_request()`
- REJECT 调用 `invite_store.reject_request()`，可选附 note（用 `window.prompt`，与现有 SessionPanel 一致）

## 5. Joiner 端组件

### 5.1 StepWelcome 更新（`ui/src/components/setup/StepWelcome.tsx`，修改）

现有 "Start Setup" 按钮下方增加 "Join Existing" 按钮：
- 样式：border 样式（非填充），与 "Start Setup" 对齐
- 点击后进入 join 流程（不改变 setup 步进逻辑）

### 5.2 StepJoin（`ui/src/components/setup/StepJoin.tsx`，新建）

加入流程页面：

1. **Invite Link 输入**：输入框 + paste 按钮
   - 如果 URL 带有 `token` + `creator_ip` 参数，自动填入
2. **验证 token**：调用 `GET /api/join/info?token=xxx`（通过 creator_ip 代理）
   - 显示 Ring 信息：名称、成员数、角色、类型
   - 无效/过期显示错误信息
3. **Display Name 输入**
4. **JOIN 按钮**：
   - open 类型：`POST /api/join/local`（`{invite_token, creator_ip, display_name}`）
   - audit 类型：`POST /api/join/apply`（`{invite_token, display_name, message}`）→ 轮询 `GET /api/join/apply/status?id=xxx`
5. **成功后**：自动进入主界面（`setSetup(true)`）

### 5.3 SetupWizard 更新（`ui/src/components/setup/SetupWizard.tsx`，修改）

增加 join 分支状态：
- `mode: 'setup' | 'join'`
- StepWelcome 中选 "Join Existing" → `setMode('join')`
- join 模式下渲染 StepJoin 而非 setup 步骤
- join 成功后同样调 `setSetup(true)` 进入主界面

### 5.4 App.tsx 更新（`ui/src/App.tsx`，修改）

URL 参数检测：
- 页面加载时检查 `window.location.search` 中的 `token` 和 `creator_ip` 参数
- 如果存在且未 setup → 自动切换到 join 模式
- 如果存在且已 setup → 直接调用 join API（不需要再走 wizard）

## 6. API 调用

### 6.1 新增 api.ts 函数

```typescript
// invite token CRUD
create_invite_token(ring_id: string, input: CreateInviteInput): Promise<InviteToken>
list_invite_tokens(ring_id: string): Promise<InviteToken[]>
revoke_invite_token(ring_id: string, token: string): Promise<void>

// join flow
verify_join_token(token: string, creator_ip?: string): Promise<JoinInfo>
join_ring(invite_token: string, display_name: string): Promise<JoinResult>
local_join(invite_token: string, creator_ip: string): Promise<JoinResult>

// audit flow
apply_join(invite_token: string, display_name: string, message?: string): Promise<ApplyResult>
check_apply_status(request_id: string): Promise<ApplyStatusResult>

// admin review
list_join_requests(ring_id: string, status?: string): Promise<JoinRequest[]>
approve_join_request(ring_id: string, request_id: string): Promise<ApproveResult>
reject_join_request(ring_id: string, request_id: string, note?: string): Promise<void>
```

注意：`verify_join_token` 和 `join_ring` 需要通过 creator_ip 构建完整 URL（`http://{creator_ip}:7420/api/...`），不走默认 `/api` 代理。本地 API（`list_invite_tokens` 等）正常走 `/api`。

## 7. 样式规范

遵循现有 IceChat 主题：
- 使用 CSS 变量（`var(--accent-cyan)`、`var(--bg-panel)` 等）
- Inline styles，不引入 CSS 框架
- 字体：Cascadia Code（monospace），标签用 `fontSize: 9px` + `letterSpacing: 1.5px` + `textTransform: uppercase`
- Modal 容器：`bg-panel` 背景 + `border: 1px solid var(--border)` + `borderRadius: 8px`
- 按钮组选中态：`var(--accent-cyan)` 边框 + `var(--bg-active)` 背景
- 无注释

## 8. 权限控制

- `+ invite member` 按钮仅 creator/admin 可见（检查当前用户在 active_ring 中的 role）
- Active Invites / Pending Requests 区域仅 creator/admin 可见
- Joiner 端无需权限检查（公开 API）

## 9. 修改文件清单

| 文件 | 改动 | 说明 |
|------|------|------|
| `ui/src/types/invite.ts` | 新建 | 邀请相关 TypeScript 类型 |
| `ui/src/stores/invite-store.ts` | 新建 | Zustand store：token CRUD + join requests |
| `ui/src/components/common/Modal.tsx` | 新建 | 通用 Modal 组件 |
| `ui/src/components/invite/CreateInviteModal.tsx` | 新建 | 邀请创建 Modal（表单 + 成功状态） |
| `ui/src/components/setup/StepJoin.tsx` | 新建 | Joiner 加入流程页面 |
| `ui/src/services/api.ts` | 修改 | 新增 invite 相关 API 函数 |
| `ui/src/components/panels/ConfigPanel.tsx` | 修改 | 加 invite 按钮 + Active Invites + Pending Requests |
| `ui/src/components/setup/StepWelcome.tsx` | 修改 | 加 "Join Existing" 按钮 |
| `ui/src/components/setup/SetupWizard.tsx` | 修改 | 加入 join 分支 |
| `ui/src/App.tsx` | 修改 | URL 参数检测 → 自动 join |
