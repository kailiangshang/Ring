# Add Member + Git Clone 设计

> **Affects**: `server/src/models/member.rs`, `server/src/services/member.rs`, `server/src/routes/members.rs`, `server/src/routes/mod.rs`
> **Depends on**: Archive + Git/GitLab 集成（Plan 6 已完成）、`git_service.rs`、`archive_service.rs` 的 `init_ring_repo`
> **Last verified**: 2026-04-19

## 1. 概述

新增 `POST /api/rings/:ring_id/members` 端点，允许 creator/admin 添加已注册用户为 Ring 成员。成员添加后，若 Ring 配置了 `gitlab_repo_url`，后台异步 clone 仓库到本地。

### 1.1 设计决策

| 决策项 | 选择 | 理由 |
|--------|------|------|
| 触发方式 | 新增 add-member API | 目前没有添加成员的端点，需要新建 |
| 默认角色 | member | 新加入成员默认为普通成员 |
| Git clone 时机 | 异步 spawn | 不阻塞 API 响应，clone 失败不影响成员添加 |
| 无远程仓库 | 跳过 clone | 没有 gitlab_repo_url 的 Ring 不需要 clone |
| 目标用户 | 仅本地已注册用户 | Ring 是 creator-hosted，所有用户在同一个实例 |
| 目录已存在 | git pull 替代 clone | 防止重复 clone 导致冲突 |

## 2. API 端点

### 2.1 Add Member

```
POST /api/rings/:ring_id/members
Authorization: X-Ring-Token: <creator/admin token>

{
  "user_id": "<目标用户的 token_id>"
}
```

**响应 (200)：**
```json
{
  "token_id": "<user_id>",
  "display_name": "Alice",
  "avatar": "🧑",
  "role": "member",
  "joined_at": "2026-04-19T12:00:00"
}
```

**错误情况：**

| 条件 | 状态码 | 错误信息 |
|------|--------|---------|
| caller 不是 creator/admin | 403 | only creator or admin can add members |
| 目标用户不存在 | 404 | user not found |
| 目标用户已是成员 | 409 | user is already a member |

## 3. 流程

```
POST /api/rings/:ring_id/members
  ├── 1. 权限检查（caller 是 creator/admin）
  ├── 2. 验证 user_id 存在于 users 表
  ├── 3. 验证 user_id 不是已有成员
  ├── 4. INSERT INTO members (ring_id, user_id, role='member')
  ├── 5. 查询 ring 的 gitlab_repo_url
  ├── 6. if gitlab_repo_url is Some:
  │     tokio::spawn(clone_or_pull_task)
  └── 7. 返回 MemberResponse（不等待 clone）
```

### 3.1 clone_or_pull_task

```
clone_or_pull_task(state, ring_id, gitlab_repo_url)
  ├── 1. 获取 repo_path = rings_dir/ring_id
  ├── 2. if repo_path/.git exists:
  │     git pull（已有仓库，拉取最新）
  ├── 3. else:
  │     git clone <gitlab_repo_url> <repo_path>
  ├── 4. 确保目录结构完整（archives/, graphs/, .group/ 等）
  └── 5. 成功/失败均 log
```

使用现有的 `git_service.rs` 的 `clone` 和 `pull` 方法。目录结构初始化复用 `archive_service::init_ring_repo` 的逻辑。

## 4. 修改文件清单

| 文件 | 改动 | 说明 |
|------|------|------|
| `server/src/models/member.rs` | 新增 `add_member` 查询 | INSERT INTO members + 返回 MemberResponse |
| `server/src/services/member.rs` | 新增 `add_member_service` | 业务逻辑：权限、验证、spawn clone |
| `server/src/routes/members.rs` | 新增 `add_member` handler | 解析请求、调用 service、返回响应 |
| `server/src/routes/mod.rs` | 注册路由 | 添加 POST route |

无数据库 migration，无前端改动。

## 5. 错误处理

| 场景 | 处理 |
|------|------|
| 目标用户不在 users 表 | 返回 404 |
| 目标用户已是成员 | 返回 409 |
| git clone 失败 | log warning，不影响成员添加 |
| git pull 失败 | log warning，不影响成员添加 |
| 目录结构创建失败 | log warning |
| ring 没有 gitlab_repo_url | 跳过 clone，正常返回 |

## 6. 边界情况

| 情况 | 处理 |
|------|------|
| 添加自己为成员 | 用户已是 creator（成员），返回 409 |
| 重复添加同一用户 | INSERT OR IGNORE 或检查后返回 409 |
| clone 过程中再次添加成员 | 不影响，pull 是幂等的 |
| 远程仓库为空 | clone 后调用 init_ring_repo 补全目录结构 |

## 7. 日志

- `info!("member added: user={user_id}, ring={ring_id}")`
- `info!("spawning git clone for ring {ring_id}")`
- `info!("git clone completed: ring={ring_id}")`
- `warn!("git clone failed for ring {ring_id}: {error}")`
- `warn!("git pull failed for ring {ring_id}: {error}")`
