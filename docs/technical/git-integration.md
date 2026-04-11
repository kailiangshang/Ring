# Ring Git 集成方案

> **Affects**: [backend.md](../api/backend.md) · [data-model.md](data-model.md)
> **Depends on**: [PRD.md](../product/PRD.md) · [data-model.md](data-model.md)
> **Last verified**: 2026-04-11

## 1. 概述

Ring 使用公司内部 GitLab 作为归档内容的版本管理平台。所有 Git 操作在 Ring 内完成，用户无需跳转到 GitLab。

### 1.1 核心原则

- **仓库复用**：使用公司 GitLab 仓库，不搭建独立 Git 服务
- **透明操作**：后端封装所有 Git 命令，用户无感知
- **内嵌审核**：PR 审核、Diff 查看在 Ring 界面内完成
- **仅管归档**：日常对话不进 Git，只有用户确认归档的内容才走 Git 流程

---

## 2. 仓库关联

### 2.1 创建 Ring 时关联

```
用户填写 Ring 名称 + Group Ring 角色描述
  → 选择关联方式：
    a) 输入已有 GitLab 仓库地址
    b) 点击"自动创建"，Ring 通过 GitLab API 创建新仓库（可选填 namespace）
  → GitLab 凭证复用全局配置（Setup 时已配置，无需重复输入）
  → 后端验证仓库可访问性
  → 初始化仓库结构（.ring-local/ + blueprint.json + .ring/ + graphs/ + nodes/ + assets/）
  → 推送初始 commit
```

> **每个 Ring（Group Ring）一个独立仓库**。用户身份通过本地 `.ring-local/identity.json` 管理（不进 Git），跨设备时重新 Setup。

### 2.2 跨设备说明

> 对话记录仅保存在当前设备（存本地 SQLite，不进 Git）。换设备后可以看到归档内容（Git 同步），但看不到历史对话。UI 上会提示用户"对话记录仅保存在当前设备"。

### 2.3 GitLab API 认证

- 使用 Personal Access Token 或 SSH key
- Token 加密存储在本地 SQLite
- 支持的 GitLab API 操作：
  - 创建仓库
  - 创建 Merge Request
  - 合并 / 关闭 Merge Request
  - 获取 MR 列表和 Diff
  - 添加评论

---

## 3. 仓库目录结构

```
ring-{name}/
├── .ring-local/              # .gitignore 排除，纯本地
│   └── identity.json         # 当前用户身份（不进 Git）
├── blueprint.json            # 蓝图配置
├── .ring/                    # AI 上下文文档（进 Git，创建者/管理员可写入）
│   ├── role.md               # 角色定义（创建者/管理员可编辑）
│   ├── conventions.md        # 团队约定（创建者/管理员可编辑）
│   ├── archive-patterns.md   # 归档模式（AI 自动积累）
│   ├── corrections.md        # 修正记录（AI 自动积累）
│   ├── knowledge-summary.md  # 知识总结（AI 自动生成）
│   └── active-context.md     # 活跃上下文（AI 动态维护）
├── graphs/
│   ├── knowledge/
│   │   └── graph.json
│   ├── event/
│   │   └── graph.json
│   └── competitor/
│       └── graph.json
├── nodes/
│   ├── competitor-analysis.md
│   ├── competitor-a.md
│   └── ...
└── assets/
    └── images/
```

### 3.1 文件规范

- `.ring-local/identity.json`：本地用户身份，不进 Git（.gitignore 排除），跨设备时重新 Setup
- `blueprint.json`：蓝图配置，蓝图确认时创建
- `.ring/role.md`：Group Ring 角色定义（替代原 `ai_prompt`），创建 Ring 时初始化，用户可编辑
- `.ring/conventions.md`：团队约定和术语，随使用积累
- `.ring/archive-patterns.md`：AI 学到的归档偏好，自动积累
- `.ring/corrections.md`：用户对 AI 的修正记录，自动积累
- `.ring/knowledge-summary.md`：定期知识全貌总结，AI 自动生成
- `.ring/active-context.md`：当前活跃上下文片段，AI 动态维护
- `graphs/{graph-name}/graph.json`：图谱数据（节点+边），每次图谱变更时更新
- `nodes/{node-id}.md`：节点对应的 Markdown 文件
- `assets/`：二进制文件（图片等）存引用路径，不直接进 Git（使用 Git LFS 或外部存储）

### 3.2 .gitignore

```
# 不追踪的文件
*.tmp
.DS_Store
thumbs.db
```

---

## 4. Git 操作流程

### 4.1 创建者直接归档

```
用户确认归档
  → 后端执行：
    1. 生成/更新 Markdown 文件
    2. 更新 graph.json
    3. git add .
    4. git commit -m "归档: {描述}"
    5. git push origin main
  → 通知所有在线成员 pull 最新内容
```

### 4.2 成员提交归档（PR 流程，串行审核队列）

```
成员确认归档
  → 后端自动 git pull（确保本地图谱最新）
  → 后端执行：
    1. 生成/更新 Markdown 文件
    2. 更新 graph.json
    3. git checkout -b archive/{member-id}/{timestamp}
    4. git add .
    5. git commit -m "归档请求: {描述}"
    6. git push origin archive/{member-id}/{timestamp}
    7. 通过 GitLab API 创建 MR（目标分支：main）
  → PR 进入审核队列（前端显示队列位置）
  → 创建者收到 PR 通知
```

### 4.2.1 PR 审核队列（串行）

```
PR 按提交顺序逐个审核（串行队列，避免 graph.json 合并冲突）

创建者审核当前 PR：
  ├── 无冲突：
  │   → 合并 MR → git pull origin main → 通知所有成员 pull
  └── 有冲突（与已合并内容冲突）：
      → 打回：关闭 MR + 通知提交成员"与已合并内容冲突，请重新归档"
      → 成员 pull 最新 → Group Ring 重新推荐 → 重新提交
```

### 4.3 创建者审核 PR

```
创建者点击"查看 Diff"
  → 后端执行：
    1. 通过 GitLab API 获取 MR 的 Diff
    2. 或使用 git diff main...archive/xxx
    3. 返回 Diff 数据（含 graph.json 人类可读变更摘要）
  → 前端渲染并排对比视图

创建者点击"合并"
  → 后端执行：
    1. 通过 GitLab API 合并 MR
    2. git pull origin main（同步本地）
    3. 通知所有在线成员 pull

创建者点击"拒绝"（含冲突打回）
  → 后端执行：
    1. 通过 GitLab API 关闭 MR
    2. 删除远程分支
    3. 通知提交成员（附原因：冲突/内容不符）
```

### 4.4 成员同步

```
成员加入 Ring
  → 后端执行：
    1. git clone {创建者的仓库} {本地路径}
    2. 加载 blueprint.json
    3. 加载所有 graph.json → 全量导入 petgraph 内存图
    4. 加载 nodes/ 下的所有 Markdown 文件
    5. 加载 .ring/ 目录（只读）
    6. 初始化 Ring 界面

成员收到合并通知
  → 后端执行：
    1. git pull origin main
    2. 解析变更内容
    3. 更新本地 SQLite 数据 + petgraph 内存图
    4. 通过 WebSocket 推送更新到前端

成员打开 Ring 时
  → 后端执行：
    1. 自动 git pull（确保本地状态最新）

成员归档前
  → 后端执行：
    1. 自动 git pull（确保图谱状态最新，Group Ring 基于最新图谱推荐）
```

---

## 5. Commit 消息规范

```
归档: {操作描述}

示例：
归档: 新增节点"竞品 A 功能分析"
归档: 更新节点"产品定位"的内容
归档: 新增边"竞品 A" → "定价策略"（关系：包含）
归档: 删除节点"临时讨论"
归档: 蓝图确认 — 3个图谱
```

---

## 6. Diff 渲染

### 6.1 Diff 查看界面

前端使用 Monaco Editor 或 CodeMirror 渲染并排对比视图：

```
┌─────────────────────────────────────────────────┐
│ PR #3: 新增节点"竞品分析"                        │
│ 提交者：张三  |  2 files changed, +65 lines      │
├─────────────────────────────────────────────────┤
│ 文件变更列表                                     │
│ ✓ graphs/knowledge/graph.json (+15 lines)       │
│ ✓ nodes/competitor-analysis.md (new file)       │
├─────────────────────────────────────────────────┤
│ [并排对比视图]                                   │
│  左侧：修改前（或空）    右侧：修改后            │
├─────────────────────────────────────────────────┤
│ [合并] [拒绝] [添加评论]                         │
└─────────────────────────────────────────────────┘
```

### 6.2 graph.json 的 Diff 可读性

由于 graph.json 是结构化数据，后端在返回 Diff 时可以额外提供"人类可读"的变更摘要：

```json
{
  "summary": {
    "addedNodes": [{"id": "node-uuid", "label": "竞品分析"}],
    "removedNodes": [],
    "updatedNodes": [],
    "addedEdges": [{"source": "产品分析", "target": "竞品分析", "relation": "contains"}],
    "removedEdges": []
  }
}
```

---

## 7. 技术实现

### 7.1 Rust 依赖

```toml
[dependencies]
git2 = "0.20"        # 本地 Git 操作（libgit2 Rust binding）
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### 7.2 凭证管理

git2 通过 `RemoteCallbacks` 设置凭证回调。支持三种认证方式：

**SSH Key**：
```rust
callbacks.credentials(|_url, username_from_url, _allowed_types| {
    Cred::ssh_key(
        username_from_url.unwrap_or("git"),
        None,
        Path::new("~/.ssh/id_rsa"),
        None,
    )
})
```

**HTTPS + Personal Access Token**：
```rust
callbacks.credentials(|_url, _username, _allowed_types| {
    Cred::userpass_plaintext("x-access-token", &stored_token)
})
```

**组合策略（推荐）**：
```rust
callbacks.credentials(|url, username, allowed_types| {
    // 1. 优先尝试 SSH（从 ssh-agent 或默认密钥）
    if allowed_types.contains(CredentialType::SSH_KEY) {
        if let Ok(c) = Cred::ssh_key_from_agent(username.unwrap_or("git")) {
            return Ok(c);
        }
    }
    // 2. 回退到 HTTPS + PAT
    if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
        return Cred::userpass_plaintext("x-access-token", &stored_pat);
    }
    Err(Error::from_str("no authentication available"))
})
```

> **注意**：`RemoteCallbacks` 是 `!Send`，clone/push 等操作需在 `tokio::task::spawn_blocking` 中执行。
> 凭证加密存储在 SQLite 中，使用 AES-256-GCM 加密。加密密钥派生自用户本地机器信息。

### 7.3 Git 服务接口

```rust
trait GitService {
    async fn clone_repo(&self, url: &str, path: &str, auth: &GitAuthType) -> Result<()>;
    async fn pull(&self, path: &str, auth: &GitAuthType) -> Result<PullResult>;
    async fn push(&self, path: &str, branch: &str, auth: &GitAuthType) -> Result<()>;
    async fn commit(&self, path: &str, message: &str) -> Result<String>;
    async fn create_branch(&self, path: &str, branch: &str) -> Result<()>;
    async fn get_diff(&self, path: &str, from: &str, to: &str) -> Result<DiffResult>;
    async fn get_log(&self, path: &str, limit: usize) -> Result<Vec<CommitInfo>>;
}
```

### 7.4 图谱同步机制

```
graph.json (Git 仓库)  ←→  petgraph 内存图
    ↑ 持久化                        ↑ 查询
    │                               │
    │  graph.json → 全量导入内存图    │  直接查内存图
    │  （启动/pull 后触发）            │  （日常查询）
    │
    ↓ 导出
    内存图 → 导出 graph.json → Git commit/push
```

同步触发时机：

| 事件 | 操作 |
|------|------|
| Ring 启动 | 全量导入所有 graph.json → 初始化内存图 + 重建索引 |
| git pull | 检测 graph.json SHA 变更 → 全量重新导入该图谱 |
| 创建/更新/删除节点 | 内存图操作 → 导出 graph.json → git add + commit |
| 归档前 | 自动 git pull → 确保 graph.json 最新 |



