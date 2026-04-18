# Archive + Git/GitLab 集成设计

> **Affects**: `server/src/services/`, `server/src/routes/`, `server/src/models/`, `ui/src/`, `server/migrations/`
> **Depends on**: PRD 2.4（归档机制）、2.5（Git 集成）、2.6（四层 AI）
> **Last verified**: 2026-04-19

## 1. 概述

将 Ring 对话内容归档为 Markdown 文件，写入磁盘 Git 仓库，支持创建者直接 commit/push 和成员 PR 审核流程。集成 GitLab REST API 实现 MR 创建、审核、合并/拒绝。

### 1.1 设计决策

| 决策项 | 选择 | 理由 |
|--------|------|------|
| Git 操作方式 | Shell git 命令 | 简单轻量，调试方便，行为与手动 git 一致 |
| 存储位置 | 磁盘 Git 仓库 | 与 PRD 目录结构一致，天然支持 git commit/push |
| 实现范围 | 完整 Archive + Git + GitLab API | 拆成细粒度步骤逐步交付 |
| 归档触发 | 手动 + 自动 | PRD 定义了两种模式 |
| Markdown 组织 | Flat 文件 | 简单可预测，archives/ 目录下直接存放 |

### 1.2 实现架构

方案 A：直接 Shell 命令架构。每个 Git 操作通过 `Command::new("git")` 执行，GitLab API 通过 `reqwest` 调用 REST API。不引入 git2 crate，不引入抽象层。

## 2. 数据模型与存储

### 2.1 磁盘目录结构

```
~/.ring/
├── .gitlab.json                        # 全局 GitLab 凭证
└── rings/<ring-id>/
    ├── .ring-local/
    │   └── identity.json               # 本地用户身份（.gitignore 排除）
    ├── blueprint.json                   # 蓝图
    ├── .group/                          # AI 上下文文档（进 Git）
    │   ├── role.md
    │   ├── conventions.md
    │   ├── archive-patterns.md          # AI 积累的归档偏好
    │   └── corrections.md               # 用户修正记录
    ├── graphs/
    │   └── graph.json                   # 知识图谱
    ├── archives/                        # 归档 Markdown（flat）
    │   ├── 2026-04-19_决策-技术选型.md
    │   └── 2026-04-20_调研-Rust框架对比.md
    └── assets/                          # 二进制文件引用（不进 Git）
```

### 2.2 数据库变更

**新增 `archive_records` 表（migration 006）**：

```sql
CREATE TABLE archive_records (
    id TEXT PRIMARY KEY,
    ring_id TEXT NOT NULL REFERENCES rings(id),
    session_id TEXT,
    node_id TEXT,
    file_name TEXT NOT NULL,
    commit_sha TEXT,
    branch TEXT,
    merge_request_iid INTEGER,
    status TEXT NOT NULL DEFAULT 'pending',
    archived_by TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

**修改 `graph_nodes` 表**：新增 `markdown_path TEXT` 列，指向 `archives/` 下的相对路径。

**启用现有字段**：`rings` 表的 `gitlab_repo_url` 和 `gitlab_namespace` 字段已存在，本设计开始使用。

### 2.3 归档状态机

```
pending → committed → pushed                （创建者/管理员路径）
pending → committed → pushed → mr_opened    （成员路径）
mr_opened → merged                          （审核通过）
mr_opened → rejected                        （审核拒绝/冲突）
rejected → pending                          （成员重新提交）
```

`status` 取值：`pending` | `committed` | `pushed` | `mr_opened` | `merged` | `rejected`

### 2.4 文件命名规则

`archives/` 下文件名格式：`YYYY-MM-DD_<标题>.md`

标题由 AI 建议或用户指定，替换非文件系统安全字符为 `-`。

## 3. 后端模块架构

### 3.1 新增文件

```
server/src/
├── services/
│   ├── git_service.rs          # Shell git 命令封装
│   ├── gitlab_service.rs       # GitLab REST API 封装
│   └── archive_service.rs      # 归档业务逻辑
├── routes/
│   └── archive.rs              # 归档 HTTP 端点
└── models/
    └── archive.rs              # ArchiveRecord 数据模型
```

### 3.2 git_service.rs

执行 shell git 命令，所有操作在指定仓库目录下执行。

```rust
fn run_git(repo_path: &Path, args: &[&str]) -> Result<CommandOutput>
fn init(path: &Path) -> Result<()>
fn clone(url: &str, path: &Path) -> Result<()>
fn pull(repo_path: &Path) -> Result<()>
fn add_all(repo_path: &Path) -> Result<()>
fn commit(repo_path: &Path, msg: &str) -> Result<String>    // 返回 commit sha
fn push(repo_path: &Path, remote: &str, branch: &str) -> Result<()>
fn create_branch(repo_path: &Path, name: &str) -> Result<()>
fn checkout(repo_path: &Path, branch: &str) -> Result<()>
fn log(repo_path: &Path, n: usize) -> Result<Vec<LogEntry>>
```

`run_git` 统一处理 `Command` 执行，捕获 stdout/stderr，非零退出码转 `RingError::GitCommandFailed`。

### 3.3 gitlab_service.rs

```rust
struct GitLabClient {
    base_url: String,
    token: String,
    client: reqwest::Client,
}

fn create_mr(&self, project_id: &str, source: &str, target: &str, title: &str) -> Result<MergeRequest>
fn merge_mr(&self, project_id: &str, mr_iid: i64) -> Result<MergeRequest>
fn close_mr(&self, project_id: &str, mr_iid: i64) -> Result<MergeRequest>
fn get_mr_diff(&self, project_id: &str, mr_iid: i64) -> Result<Vec<DiffEntry>>
fn get_current_user(&self) -> Result<GitLabUser>    // 验证 token 用
```

`project_id` 从 `rings.gitlab_repo_url` 推导（URL encode 的 namespace/project）。

### 3.4 archive_service.rs

```rust
async fn archive_content(
    db: &SqlitePool,
    git: &GitService,
    gitlab: &GitLabClient,
    ring_id: &str,
    session_id: Option<&str>,
    content: &str,
    suggested_title: &str,
    node_suggestion: NodeSuggestion,
    user_id: &str,
    is_creator: bool,
) -> Result<ArchiveRecord>

async fn archive_auto(/* 同 archive_content，跳过用户确认 */) -> Result<ArchiveRecord>

async fn review_mr(
    db: &SqlitePool,
    git: &GitService,
    gitlab: &GitLabClient,
    record_id: &str,
    action: ReviewAction,    // Merge | Reject
) -> Result<ArchiveRecord>

async fn init_ring_repo(
    git: &GitService,
    ring_id: &str,
    gitlab_url: &str,
) -> Result<PathBuf>
```

### 3.5 归档流程

**创建者/管理员路径**：

1. `git pull`（确保最新）
2. AI 生成 Markdown → 写入 `archives/<file_name>`
3. 更新 `graphs/graph.json`（新增/更新节点，设置 markdown_path）
4. `git add .` → `git commit` → `git push origin main`
5. 更新 `archive_records: status = pushed`

**成员路径**：

1. `git pull`
2. AI 生成 Markdown → 写入 `archives/<file_name>`
3. 更新 `graphs/graph.json`
4. 创建分支 `archive/<record_id>` → checkout
5. `git add .` → `git commit` → `git push origin archive/<record_id>`
6. GitLab API 创建 MR（target: main）
7. checkout 回 main
8. 更新 `archive_records: status = mr_opened`

### 3.6 NodeSuggestion 类型

```rust
enum NodeAction {
    CreateNew { parent_id: Option<String>, node_title: String },
    AttachExisting { node_id: String },
    UpdateExisting { node_id: String },
}
```

## 4. API 端点

### 4.1 归档端点

| 方法 | 路径 | 用途 |
|------|------|------|
| POST | `/api/rings/:ring_id/archive` | 触发归档（手动） |
| POST | `/api/rings/:ring_id/archive/auto` | 触发归档（Auto 模式） |
| GET | `/api/rings/:ring_id/archives` | 列出归档记录 |
| GET | `/api/rings/:ring_id/archives/:id` | 查询单条归档状态 |
| POST | `/api/rings/:ring_id/archives/:id/review` | 审核 MR |
| GET | `/api/rings/:ring_id/archive-queue` | 查看 PR 审核队列 |
| GET | `/api/rings/:ring_id/repo/status` | 查看仓库状态 |
| POST | `/api/rings/:ring_id/repo/init` | 初始化 Git 仓库 |

### 4.2 触发归档请求体

```json
{
  "session_id": "optional-session-id",
  "content": "要归档的对话内容片段",
  "suggested_title": "决策-技术选型",
  "node_suggestion": {
    "action": "create_new",
    "parent_id": "optional-parent-node-id",
    "node_title": "技术选型"
  }
}
```

`node_suggestion.action` 取值：`create_new`（新建节点）| `attach_existing`（挂载到已有节点）| `update_existing`（更新已有节点）

### 4.3 审核 MR 请求体

```json
{
  "action": "merge"
}
```

`action` 取值：`merge` | `reject`

### 4.4 SSE 归档进度流

归档过程使用 SSE 流式返回进度（复用 `chat.rs` 的 SSE 模式）：

```
event: progress
data: {"step": "pulling", "message": "正在拉取最新内容..."}

event: progress
data: {"step": "generating", "message": "AI 正在生成归档内容..."}

event: progress
data: {"step": "writing", "message": "写入 Markdown 文件..."}

event: progress
data: {"step": "committing", "message": "提交到 Git..."}

event: progress
data: {"step": "pushing", "message": "推送到远程仓库..."}

event: complete
data: {"record": {...}}
```

## 5. GitLab 集成

### 5.1 Setup 流程扩展

Setup 新增步骤 3：GitLab 配置。

- 输入 GitLab URL（如 `https://gitlab.company.com`）
- 输入 Personal Access Token（需 `api` + `write_repository` 权限）
- 验证连接（`GET /api/v4/user` 测试 token 有效性）

凭证存入 `~/.ring/.gitlab.json`（全局共享，所有 Ring 复用）：

```json
{
  "base_url": "https://gitlab.company.com",
  "token": "glpat-xxxx",
  "user_id": 123
}
```

### 5.2 创建 Ring 时的 Git 初始化

提供 `gitlab_repo_url` 时后端自动：

1. `git clone <url>` 到 `~/.ring/rings/<ring-id>/`
2. 仓库为空：`git init` + 创建初始目录结构 + 首次 commit + push
3. 仓库有内容：`git pull` 确保最新
4. 将 `.group/` 下的 `role.md`、`conventions.md` 写入（来自 group_docs 表）

### 5.3 成员加入时的 Git 初始化

成员加入时后端自动：

1. `git clone <ring的gitlab_repo_url>` 到 `~/.ring/rings/<ring-id>/`
2. 加载 `graphs/graph.json`
3. 扫描 `archives/` 目录建立本地索引

### 5.4 GitLab MR 创建

```
POST /api/v4/projects/:id/merge_requests
{
  "source_branch": "archive/<record_id>",
  "target_branch": "main",
  "title": "归档: <标题>",
  "description": "由 <用户名> 提交的归档请求\n\n来源 Session: <session_id>"
}
```

### 5.5 MR 审核流程

1. `GET /api/v4/projects/:id/merge_requests/:iid/changes` → 获取 diff
2. 前端展示 diff（Markdown 文件变更 + graph.json 变更）
3. 合并：`PUT .../merge` → `git pull` → 更新 status = merged → 通知成员 pull
4. 拒绝：`PUT ...` state_event: close → 更新 status = rejected → 通知成员

### 5.6 冲突处理

GitLab 返回 405（conflict）时：

1. 关闭 MR
2. 更新 `archive_records: status = rejected`
3. 通知成员 pull 最新后重新提交

### 5.7 数据安全

- GitLab token 存本地文件 `~/.ring/.gitlab.json`，不进 SQLite、不进 Git
- `.ring-local/` 加入 `.gitignore`
- Token 仅在 `gitlab_service.rs` 中使用

## 6. 前端设计

### 6.1 新增文件

```
ui/src/
├── types/archive.ts
├── stores/archive-store.ts
├── components/panels/ArchivePanel.tsx
└── components/sidebar/ArchiveIndicator.tsx
```

### 6.2 CLI 命令扩展

| 命令 | 作用 |
|------|------|
| `/archive <标题>` | 手动归档当前对话内容 |
| `/archive list` | 查看归档列表 |
| `/archive queue` | 查看 PR 审核队列 |
| `/archive review <id> merge` | 通过 MR |
| `/archive review <id> reject` | 拒绝 MR |

### 6.3 ArchivePanel UI

- 左侧：归档文件列表（flat 列表，显示文件名 + 状态徽标 + 时间）
- 右侧：选中归档的 Markdown 预览
- 底部：PR 审核队列（待审核 MR 数量 + 当前审核项 + merge/reject 按钮）

## 7. 错误处理

扩展 `crate::error::RingError`，新增变体：

```rust
GitCommandFailed { cmd: String, stderr: String }
GitlabApiError { status: u16, message: String }
GitlabNotConfigured
RepoNotFound { ring_id: String }
ArchiveConflict { record_id: String }
InvalidArchiveState { record_id: String, current: String, expected: String }
```

不造新错误类型，统一使用 `RingError`。

## 8. 实现步骤

按依赖顺序排列，每步独立可验证：

| 步骤 | 内容 | 验证标准 |
|------|------|---------|
| 6-0 | DB migration 006：`archive_records` 表 + `graph_nodes.markdown_path` | migration 运行成功 |
| 6-1 | `models/archive.rs`：ArchiveRecord + CRUD 查询 | 单元测试通过 |
| 6-2 | `git_service.rs`：`run_git` + `init` + `add_all` + `commit` + `log` | 临时目录 init→add→commit→log |
| 6-3 | `git_service.rs`：`clone` + `pull` + `push` + `create_branch` + `checkout` | clone→分支→push |
| 6-4 | 磁盘目录初始化：创建 Ring 时建立目录结构 + git init | 目录存在且是 git 仓库 |
| 6-5 | `gitlab_service.rs`：`GitLabClient` + 全部 API 方法 | mock server 测试 |
| 6-6 | Setup 扩展：收集 GitLab URL + token → 存文件 → 验证连接 | Setup 后凭证文件存在 |
| 6-7 | `archive_service.rs`：创建者归档路径 | 归档→磁盘有文件→git log 有记录 |
| 6-8 | `archive_service.rs`：成员归档路径（分支→MR） | 成员归档→GitLab 有 MR |
| 6-9 | `routes/archive.rs`：归档触发端点 + SSE | curl→收到 SSE 事件流 |
| 6-10 | `routes/archive.rs`：归档列表 + 查询 + 仓库状态 | curl→返回正确数据 |
| 6-11 | `archive_service.rs`：`review_mr` + 审核队列 | merge→MR 合并→pull 成功 |
| 6-12 | `routes/archive.rs`：审核端点 + 队列端点 | curl→审核流程完整 |
| 6-13 | 前端 `types/archive.ts` + `archive-store.ts` | TypeScript 编译通过 |
| 6-14 | 前端 `ArchivePanel.tsx` | 浏览器中归档流程可见 |
| 6-15 | 前端 CLI 命令扩展 | 聊天框命令→触发归档 |
| 6-16 | 自动归档（Auto 模式） | session 结束→自动归档 |
| 6-17 | 成员加入时 git clone + 离线查看 | 加入→本地有仓库 |
| 6-18 | 端到端集成测试 | 完整流程通过 |

### 依赖关系

- 6-0 → 6-1
- 6-2 → 6-3
- 6-3 + 6-1 → 6-4
- 6-5 独立
- 6-3 + 6-1 + 6-5 → 6-7
- 6-7 → 6-8
- 6-7 → 6-9
- 6-8 + 6-11 → 6-12
- 6-9 → 6-13 → 6-14 → 6-15
- 6-8 → 6-16
- 6-3 → 6-17

可并行：6-2 与 6-5、6-13 与 6-7/6-8、6-14 与 6-11/6-12
