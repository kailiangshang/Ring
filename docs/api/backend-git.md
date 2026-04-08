# Git / GitLab 服务 API 参考

> 源码路径：`ring-server/src/services/git_service.rs`、`gitlab_service.rs`

## GitService

### `struct GitService`
源文件：`services/git_service.rs:29`

纯 `git2` 封装，所有操作通过 `tokio::task::spawn_blocking` 在阻塞线程池执行。

### 公开方法

| 方法 | 签名 | 说明 |
|------|------|------|
| `fn new() -> Self` | 构造函数 |
| `async fn init_repo(path) -> Result<()>` | 在指定路径初始化 git 仓库 |
| `async fn clone_repo(url, to_path) -> Result<()>` | 克隆仓库 |
| `async fn add_all(repo_path) -> Result<()>` | `git add .` 所有文件 |
| `async fn commit(repo_path, message) -> Result<String>` | 提交，返回 commit SHA |
| `async fn create_branch(repo_path, name) -> Result<()>` | 创建分支 |
| `async fn get_current_branch(repo_path) -> Result<String>` | 获取当前分支名 |
| `async fn get_diff(repo_path, from, to) -> Result<DiffResult>` | 获取两个 commit 间的 diff |
| `async fn get_log(repo_path, limit) -> Result<Vec<CommitInfo>>` | 获取提交日志（按时间排序） |
| `async fn has_changes(repo_path) -> Result<bool>` | 是否有未提交变更 |
| `async fn status_files(repo_path) -> Result<Vec<String>>` | 获取变更文件列表 |

---

## GitlabService

### `struct GitlabService`
源文件：`services/gitlab_service.rs:7`

| 字段 | 类型 | 说明 |
|------|------|------|
| `base_url` | `String` | GitLab 实例 URL |
| `token` | `String` | Private Token |
| `client` | `reqwest::Client` | HTTP 客户端 |

### 公开方法

| 方法 | 签名 | 说明 |
|------|------|------|
| `fn new(base_url, token) -> Self` | 构造函数（自动去除 URL 尾部斜杠） |
| `async fn create_repo(name, namespace) -> Result<CreateRepoResponse>` | 创建项目 |
| `async fn create_mr(project_id, source_branch, target_branch, title) -> Result<MergeRequestInfo>` | 创建 MR |
| `async fn merge_mr(project_id, mr_iid) -> Result<MergeRequestInfo>` | 合并 MR |
| `async fn close_mr(project_id, mr_iid) -> Result<MergeRequestInfo>` | 关闭 MR |
| `async fn list_mrs(project_id, state) -> Result<Vec<MergeRequestInfo>>` | 列出 MR |
| `async fn get_mr_diff(project_id, mr_iid) -> Result<Vec<MrDiff>>` | 获取 MR diff |
| `fn get_repo_url(project_path) -> Result<String>` | 拼接 git 仓库 URL |

---

## 数据结构

### `PullResult`
源文件：`services/git_service.rs:5`

| 字段 | 类型 | 说明 |
|------|------|------|
| `had_changes` | `bool` | 是否有变更 |
| `changed_files` | `Vec<String>` | 变更文件列表 |

### `CommitInfo`
源文件：`services/git_service.rs:10`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | Commit SHA |
| `message` | `String` | 提交信息 |
| `author` | `String` | 作者名称 |
| `timestamp` | `String` | 时间戳 |

### `DiffResult`
源文件：`services/git_service.rs:17`

| 字段 | 类型 | 说明 |
|------|------|------|
| `files` | `Vec<FileDiff>` | 变更文件列表 |

### `FileDiff`
源文件：`services/git_service.rs:21`

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | `String` | 文件路径 |
| `status` | `String` | 状态（added/modified/deleted/renamed/unknown） |
| `additions` | `i64` | 新增行数 |
| `deletions` | `i64` | 删除行数 |
| `content` | `String` | diff 内容摘要（old → new） |

### `CreateRepoResponse`
源文件：`services/git_model.rs:14`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `i64` | 项目 ID |
| `url` | `String` | HTTP 克隆 URL |
| `ssh_url` | `String` | SSH 克隆 URL |

### `MergeRequestInfo`
源文件：`services/gitlab_service.rs:23`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `i64` | MR ID |
| `iid` | `i64` | MR 编号 |
| `title` | `String` | 标题 |
| `author` | `MergeRequestAuthor` | 作者信息 |
| `state` | `String` | 状态 |
| `web_url` | `String` | Web URL |

### `MergeRequestAuthor`
源文件：`services/gitlab_service.rs:34`

| 字段 | 类型 | 说明 |
|------|------|------|
| `username` | `String` | 用户名 |

### `MrDiff`
源文件：`services/gitlab_service.rs:45`

| 字段 | 类型 | 说明 |
|------|------|------|
| `old_path` | `String` | 旧文件路径 |
| `new_path` | `String` | 新文件路径 |
| `diff` | `String` | unified diff |
