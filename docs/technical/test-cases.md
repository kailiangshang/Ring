# Ring 测试用例设计

> **Affects**: (test files in ring-server/tests/ and ring-frontend/src/)
> **Depends on**: [PRD.md](../product/PRD.md) · [api-design.md](api-design.md) · [permissions.md](../product/permissions.md)
> **Last verified**: 2026-04-11

## 1. 概述

### 1.1 测试策略

按核心业务流程组织测试用例（非按技术层）。每个流程覆盖 happy path + 异常路径 + 权限边界。

两条测试线：

| 测试线 | 工具 | 覆盖范围 |
|--------|------|---------|
| 后端集成测试 | Rust `#[tokio::test]` + Axum test router | API 端到端，覆盖全部核心流程 |
| E2E 测试 | Playwright | 用户视角的关键流程，覆盖 UI 交互 |

### 1.2 用例编号规则

`TC-P{phase}-{seq}`，例如 `TC-P1-001`。

### 1.3 每个用例结构

```
用例编号
流程名称
前置条件
测试步骤
预期结果
```

---

## Phase 1：基础框架

### TC-P1-001 首次启动 Setup 向导

**前置条件**：无（全新安装）

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | GET /api/v1/setup/status | `{"setup_completed": false, "step": "username"}` |
| 2 | POST /api/v1/setup/username `{"display_name": "张三"}` | 200，返回 user_id（UUID） |
| 3 | POST /api/v1/setup/llm `{"provider": "openai", "model": "gpt-4", "api_key": "sk-xxx"}` | 200 |
| 4 | POST /api/v1/setup/gitlab `{"repo_url": "git@gitlab.company.com:user/ring.git", "auth_type": "ssh_key", "ssh_key_path": "~/.ssh/id_rsa"}` | 200 |
| 5 | POST /api/v1/setup/complete | 200，`setup_completed = true` |
| 6 | GET /api/v1/setup/status | `{"setup_completed": true}` |

**异常路径**：

- display_name 为空 → 400 Validation
- display_name 超过 50 字符 → 400 Validation
- provider 不是 openai/anthropic/ollama → 400 Validation
- api_key 为空 → 400 Validation
- 已 setup 后再调 setup 接口 → 409 Conflict
- gitlab 连接失败（地址不通）→ 502 Git error

### TC-P1-002 创建 Ring

**前置条件**：已完成 Setup

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | POST /api/v1/rings `{"name": "产品竞品分析组", "description": "竞品研究", "role_description": "产品分析专家", "gitlab_repo": "auto_create", "namespace": "team"}` | 201，返回 ring_id，status = blueprint_pending |
| 2 | GET /api/v1/rings/{ringId} | 200，返回 Ring 详情 |
| 3 | GET /api/v1/rings | 200，列表包含刚创建的 Ring |

**异常路径**：

- name 为空 → 400
- name 超过 100 字符 → 400
- gitlab_repo 无效地址 → 502
- namespace 不存在 → 502
- 未 Setup → 401

### TC-P1-003 Ring CRUD

**前置条件**：已完成 Setup，已有 Ring

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | PUT /api/v1/rings/{ringId} `{"name": "新名称", "description": "新描述"}` | 200，返回更新后的 Ring |
| 2 | GET /api/v1/rings/{ringId} | 200，名称和描述已更新 |
| 3 | DELETE /api/v1/rings/{ringId} | 204 |
| 4 | GET /api/v1/rings/{ringId} | 404 |

**异常路径**：

- 更新不存在的 Ring → 404
- 删除不存在的 Ring → 404
- name/description 字段超长 → 400

### TC-P1-004 安装导航页

**前置条件**：创建者已生成邀请链接

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | GET /join?token={validToken} | 200，返回 HTML 页面，包含 Ring 名称、成员数、四平台下载链接 |
| 2 | HTML 中包含 `window.__RING_JOIN_DATA__` | 包含 ring_name、downloads（windows/linux/macos_arm/macos_intel）、creator_ip |
| 3 | GET /join?token={invalidToken} | 404 或 错误提示页面 |
| 4 | GET /join?token={expiredToken} | 错误提示"链接已过期" |
| 5 | GET /join（无 token 参数） | 400 |

---

## Phase 2：AI 对话与蓝图

### TC-P2-001 Group Ring 流式对话

**前置条件**：已完成 Setup，已有 Ring，已完成蓝图

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | POST /api/v1/rings/{ringId}/conversations `{"title": "测试对话"}` | 201，返回 conv_id |
| 2 | POST /api/v1/rings/{ringId}/conversations/{convId}/messages `{"content": "你好"}` | SSE 流：text → done |
| 3 | GET /api/v1/rings/{ringId}/conversations/{convId}/messages | 200，返回 2 条消息（用户 + AI） |
| 4 | GET /api/v1/rings/{ringId}/conversations/{convId}/token-stats | 200，token_count > 0 |

**异常路径**：

- Ring 不存在 → 404
- conversation 不存在 → 404
- content 为空 → 400
- LLM 连接失败 → SSE 返回 error event
- LLM 超时 → SSE 返回 error event（code = llm_timeout）
- 未 Setup → 401

### TC-P2-002 Super Ring 对话

**前置条件**：已完成 Setup

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | POST /api/v1/super-ring/chat `{"message": "帮我创建一个竞品分析的 Ring"}` | SSE 流式返回 |
| 2 | POST /api/v1/super-ring/analyze `{"ring_ids": ["ring-1", "ring-2"], "query": "对比分析"}` | SSE 流式返回跨 Ring 分析结果 |
| 3 | POST /api/v1/super-ring/summarize `{"ring_ids": ["ring-1"], "topic": "本周决策"}` | SSE 流式返回总结 |
| 4 | POST /api/v1/super-ring/merge-suggest `{"source_ring_id": "ring-1", "target_ring_id": "ring-2"}` | 200，返回合并推荐列表 |

**异常路径**：

- ring_ids 中包含不存在的 Ring → 404
- ring_ids 为空数组 → 400
- merge-suggest 的 source 和 target 相同 → 400

### TC-P2-003 蓝图快速路径

**前置条件**：已完成 Setup，已有 Ring（status = blueprint_pending）

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | GET /api/v1/rings/{ringId}/blueprint/templates | 200，返回模板列表（至少包含"产品研究"、"项目管理"等） |
| 2 | POST /api/v1/rings/{ringId}/blueprint/preview `{"graphs": [{"name": "知识图谱", "type": "knowledge", "categories": ["概念", "方法"]}]}` | 200，返回节点+边预览数据 |
| 3 | POST /api/v1/rings/{ringId}/blueprint/confirm `{"graphs": [...]}` | 200，返回 blueprint_id 和 graph_ids，Ring status 变为 active |

**异常路径**：

- graphs 为空 → 400
- graphs 超过 3 个 → 提醒资源消耗（warning），不阻止
- categories 为空 → 400
- 确认时 Ring 已有蓝图 → 409

### TC-P2-004 蓝图深度路径

**前置条件**：已完成 Setup，已有 Ring（status = blueprint_pending）

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | POST /api/v1/rings/{ringId}/blueprint/chat `{"message": "我需要一个竞品研究的图谱"}` | SSE 流：text + blueprint_proposal + done |
| 2 | 继续对话调整 `{"message": "加一个事件图谱"}` | SSE 流：text + blueprint_proposal（包含新图谱）+ done |
| 3 | 预览确认蓝图 | 同 TC-P2-003 步骤 2-3 |

**异常路径**：

- 非创建者尝试蓝图对话 → 403
- Ring 已确认蓝图后再对话 → 409

### TC-P2-005 对话上下文管理

**前置条件**：已有 Ring + 已完成蓝图

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 创建 storage 模式对话 | 对话创建成功 |
| 2 | 发送大量消息直到 token_count 接近 token_limit | GET token-stats 返回 warning |
| 3 | POST /api/v1/rings/{ringId}/conversations/{convId}/compact | 200，token_count_before > token_count_after |
| 4 | 发消息验证 compact 后仍可正常对话 | SSE 正常流式返回 |

**异常路径**：

- ephemeral 模式对话调用 compact → 400
- auto_compact 设置后自动触发（验证触发时机正确）

---

## Phase 3：知识图谱

### TC-P3-001 节点 CRUD

**前置条件**：已有 Ring + 已完成蓝图

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | POST /api/v1/rings/{ringId}/graphs/{graphId}/nodes `{"label": "竞品 A", "type": "concept", "parent_id": "root", "description": "分析"}` | 201，返回 node_id |
| 2 | GET /api/v1/rings/{ringId}/graphs/{graphId}/nodes/{nodeId} | 200，返回节点详情 |
| 3 | PUT /api/v1/rings/{ringId}/graphs/{graphId}/nodes/{nodeId} `{"label": "竞品 A 深度分析"}` | 200，label 已更新 |
| 4 | GET /api/v1/rings/{ringId}/graphs/{graphId}/nodes/{nodeId}/content | 200，返回 Markdown 内容 |
| 5 | DELETE /api/v1/rings/{ringId}/graphs/{graphId}/nodes/{nodeId} | 204 |
| 6 | 再次 GET 该节点 | 404 |

**异常路径**：

- parent_id 不存在 → 404
- label 为空 → 400
- 删除有子节点的节点 → 409（需先删除子节点）
- graphId 不存在 → 404

### TC-P3-002 边 CRUD

**前置条件**：已有 Ring + 图谱 + 至少 2 个节点

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | POST /api/v1/rings/{ringId}/graphs/{graphId}/edges `{"source_id": "node-1", "target_id": "node-2", "relation": "depends_on", "label": "依赖"}` | 201，返回 edge_id |
| 2 | GET /api/v1/rings/{ringId}/graphs/{graphId} | 200，edges 中包含新边 |
| 3 | DELETE /api/v1/rings/{ringId}/graphs/{graphId}/edges/{edgeId} | 204 |
| 4 | 再次 GET 图谱 | edges 中不含已删除边 |

**异常路径**：

- source_id = target_id（自环）→ 400
- source_id 或 target_id 不存在 → 404
- 重复边（相同 source + target + relation）→ 409

### TC-P3-003 graph.json 同步

**前置条件**：已有 Ring + 图谱

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 创建节点 | petgraph 内存图 + graph.json 文件同步更新 |
| 2 | GET /api/v1/rings/{ringId}/exports/graph-json/{graphId} | 返回包含新节点的 graph.json |
| 3 | 删除节点 | graph.json 同步移除该节点 |
| 4 | 模拟 git pull（外部修改 graph.json）→ 重启 Ring | petgraph 从 graph.json 全量导入，状态一致 |

**异常路径**：

- graph.json 损坏（无效 JSON）→ 启动报错，日志明确提示
- graph.json 中引用了不存在的 parent_id → 导入时跳过无效引用，记录警告

---

## Phase 4：Git 集成

### TC-P4-001 仓库关联

**前置条件**：已完成 Setup，已有 GitLab 凭证

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 创建 Ring 时 `gitlab_repo: "auto_create"` + `namespace: "team"` | GitLab 上创建仓库，本地 clone 成功 |
| 2 | 创建 Ring 时传入已有仓库地址 | 直接 clone，本地仓库结构完整 |
| 3 | 验证本地仓库结构 | 包含 .ring/、graphs/、nodes/、.ring-local/ 目录 |

**异常路径**：

- namespace 不存在 → 502，错误信息明确
- GitLab 凭证无效 → 401
- 仓库地址不存在 → 404
- SSH key 无权限 → 403
- 本地路径已存在同名目录 → 409

### TC-P4-002 创建者归档（直接 commit）

**前置条件**：创建者身份 + 已有 Ring + 已完成蓝图

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 在对话中产生有价值内容 | AI 推荐归档（archive_suggestion event） |
| 2 | POST /api/v1/rings/{ringId}/archive `{"message_ids": [...], "graph_id": "g1", "target_node_id": "n1", "label": "竞品分析"}` | 后端自动 git pull → 生成 Markdown → 更新 graph.json → git commit + push |
| 3 | 验证返回 | `git_status: "committed"`，`pr_url: null` |
| 4 | 验证 GitLab 仓库 | 包含新 commit，Markdown 文件和 graph.json 更新 |

**异常路径**：

- target_node_id 不存在 → 404
- message_ids 中包含不存在的消息 → 400
- git pull 时有冲突 → 409，提示先解决冲突
- git push 失败 → 502

### TC-P4-003 成员归档（提交 PR）

**前置条件**：成员身份 + 已有 Ring + 已完成蓝图

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 成员标记归档 | 后端创建分支 → commit → push → GitLab API 创建 MR |
| 2 | 验证返回 | `git_status: "pr_pending"`，`queue_position: 1`，`pr_url: "https://gitlab.../merge_requests/3"` |
| 3 | GET /api/v1/rings/{ringId}/archive/queue | 返回当前审核队列 |

**异常路径**：

- 只读角色尝试归档 → 403
- 同一成员在队列中已有 PR → 允许（不阻止，但提醒排队）
- GitLab MR 创建失败 → 502

### TC-P4-004 PR 审核

**前置条件**：队列中有成员提交的 PR

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | GET /api/v1/rings/{ringId}/prs?status=opened | 返回 PR 列表 |
| 2 | GET /api/v1/rings/{ringId}/prs/{prId}/diff | 返回文件变更列表和 diff 内容 |
| 3 | POST /api/v1/rings/{ringId}/prs/{prId}/merge | MR 合并，所有成员自动 pull |
| 4 | 成员验证本地 | git pull 后本地图谱更新 |

**异常路径**：

- 成员尝试合并 PR → 403（只有创建者/管理员可审核）
- 合并时 graph.json 冲突 → 冲突检测，拒绝合并 → 打回通知成员
- PR 不存在 → 404
- PR 已关闭/已合并 → 409

---

## Phase 5：协作与权限

### TC-P5-001 开放链接加入

**前置条件**：创建者已生成开放邀请链接

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | GET /join?token={openToken}（创建者 ring-server） | 200，HTML 安装导航页 |
| 2 | POST /api/v1/rings/join?token={openToken} `{"display_name": "李四"}` | 200，自动 clone 仓库，分配 token_id |
| 3 | GET /api/v1/rings/{ringId}/members | 列表包含新成员 |
| 4 | 验证新成员本地 | 仓库已 clone，图谱已加载 |

**异常路径**：

- token 无效 → 404
- token 已过期 → 410 Gone
- token 已用完（max_uses 达到）→ 410
- Ring 成员数达到 max_members → 409
- 已是该 Ring 成员 → 409
- 未 Setup 先加入 → 先走 Setup，完成后自动 join

### TC-P5-002 审核链接加入

**前置条件**：创建者已生成审核邀请链接

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | POST /api/v1/rings/join/apply?token={auditToken} `{"display_name": "王五", "reason": "产品组新成员"}` | 200，申请已提交 |
| 2 | 创建者收到通知 | WebSocket 推送 join_request 通知 |
| 3 | POST /api/v1/rings/{ringId}/join-requests/{requestId}/approve | 200，分配 token_id，通知申请人 |
| 4 | 申请人验证 | 收到通知，可访问 Ring |

**异常路径**：

- 创建者拒绝 → POST reject → 申请人收到拒绝通知（附理由）
- token 过期后申请 → 410
- 创建者撤销 token 后申请 → 410
- 非创建者尝试审批 → 403

### TC-P5-003 权限校验

**前置条件**：Ring 内有创建者、管理员、成员、只读四种角色

| 操作 | 创建者 | 管理员 | 成员 | 只读 |
|------|--------|--------|------|------|
| 和 Group Ring 对话 | ✅ | ✅ | ✅ | ❌ → 403 |
| 使用工具 | ✅ | ✅ | ✅ | ❌ → 403 |
| Export 对话 | ✅ | ✅ | ✅ | ❌ → 403 |
| 直接 commit 归档 | ✅ | ✅ | ❌ → 403 | ❌ → 403 |
| 审核 PR | ✅ | ✅ | ❌ → 403 | ❌ → 403 |
| 管理 Ring 成员 | ✅ | ❌ → 403 | ❌ → 403 | ❌ → 403 |
| 编辑 .ring/ 文档 | ✅ | ✅ | ❌ → 403 | ❌ → 403 |
| 修改蓝图模板 | ✅ | ❌ → 403 | ❌ → 403 | ❌ → 403 |
| 查看图谱和归档 | ✅ | ✅ | ✅ | ✅ |

**额外用例**：

- 创建者将成员提升为管理员 → 角色变更后成员获得管理员权限
- 创建者将管理员降级为成员 → 角色变更后失去管理员权限
- 创建者将成员设为只读 → 失去对话和归档权限
- 管理员尝试变更角色 → 403
- 创建者移除成员 → 该成员无法再访问 Ring

### TC-P5-004 三模式切换

**前置条件**：创建者身份，已完成蓝图

| 模式 | 操作 | 预期结果 |
|------|------|---------|
| 日常对话 | 发送消息"今天天气真好" | AI 回复，不触发任何归档建议 |
| 日常对话 | 点击 Export 按钮 | 触发归档流程（进入手动归档模式） |
| 手动归档 | 说"归档" | AI 推荐节点位置 → 用户确认 → commit/PR |
| Auto | 点击 Auto 按钮 | AI 自动判断并归档，无需逐个确认 |
| Auto → 退出 | 再次点击 Auto | 退出 Auto 模式，回到日常对话 |

**异常路径**：

- 只读角色尝试切换模式 → 403
- 成员 Auto 模式下的 Git 操作 → 批量标记，用户一次性确认后合并 PR（非逐个）

### TC-P5-005 实时通知

**前置条件**：创建者和成员均在线

| 事件 | 触发 | 预期结果 |
|------|------|---------|
| PR 提交通知 | 成员提交归档 PR | 创建者收到 WebSocket `pr_notification` |
| PR 合并通知 | 创建者合并 PR | 成员收到通知 + 自动 git pull |
| PR 拒绝通知 | 创建者拒绝 PR | 成员收到通知（附原因） |
| 成员加入通知 | 新成员加入 Ring | 所有在线成员收到 `member_update` |
| 成员移除通知 | 创建者移除成员 | 被移除者收到踢出通知 |
| 图谱变更通知 | 归档后图谱更新 | 所有在线成员收到 `graph_update` |

**离线通知**：

- 成员离线期间发生事件 → 通知缓存到 SQLite → 下次启动展示未读列表
- GET /api/v1/rings/{ringId}/notifications → 返回未读通知列表
- PUT /api/v1/rings/{ringId}/notifications/{id}/read → 标记已读

---

## Phase 5（续）：Session 多人讨论

### TC-P5-006 Session 生命周期

**前置条件**：创建者身份，Ring 内有至少 3 名成员

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | POST /api/v1/rings/{ringId}/sessions `{"title": "竞品讨论", "scenario": "discussion", "invite_member_ids": ["user-2", "user-3"]}` | 201，返回 session，status = active |
| 2 | 验证并发限制 | 同一 Ring 再创建 Session → 409 |
| 3 | POST /api/v1/rings/{ringId}/sessions/{sessionId}/messages `{"content": "大家好"}` | SSE 正常返回 |
| 4 | 被邀请成员发消息 | 消息通过 session owner 后端中转，广播给所有参与者 |
| 5 | POST /api/v1/rings/{ringId}/sessions/{sessionId}/close | 200，status = closed |
| 6 | 再次发消息 | 拒绝（session 已关闭） |
| 7 | DELETE /api/v1/rings/{ringId}/sessions/{sessionId} | 204，所有消息记录清除 |

**异常路径**：

- 普通成员未授权创建 Session → 403
- scenario 不是预设值 → 400
- invite_member_ids 包含非 Ring 成员 → 400
- 已关闭的 session 发消息 → 409
- 非 session owner 关闭/删除 → 403

### TC-P5-007 Session 暂停与恢复

**前置条件**：活跃 Session，有 2+ 参与者

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | Session owner 断开连接 | 所有参与者收到 `session_paused` WebSocket 事件 |
| 2 | 参与者尝试发消息 | 被拒绝（session 暂停） |
| 3 | Session owner 重连 | 所有参与者收到 `session_resumed` 事件 |
| 4 | 参与者发消息 | 正常发送和广播 |

### TC-P5-008 Session 离线补发

**前置条件**：活跃 Session

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 参与者 A 离线 | 继续产生消息 seq_num = 10, 11, 12 |
| 2 | 参与者 A 重连 | 发送 `after_seq = 9` |
| 3 | GET /api/v1/rings/{ringId}/sessions/{sessionId}/messages?after_seq=9 | 返回 seq_num 10, 11, 12 的消息 |

### TC-P5-009 Session 归档

**前置条件**：活跃 Session，archive_enabled = true，session owner 身份

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | PUT sessions/{sessionId}/archive-toggle `{"archive_enabled": true}` | 200 |
| 2 | POST sessions/{sessionId}/archive `{"message_ids": [...], "graph_id": "g1"}` | 走标准归档流程 |
| 3 | 验证 | 归档内容写入 Ring 知识图谱 |

**异常路径**：

- archive_enabled = false 时尝试归档 → 409
- 非 session owner 尝试归档 → 403
- 非 session owner 切换归档开关 → 403

### TC-P5-010 成员移除的 Session 处理

**前置条件**：活跃 Session，被移除者是 participant

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 创建者移除 participant | 该成员收到 `session_member_kicked` WebSocket 事件 |
| 2 | 被移除者尝试发消息 | 被拒绝 |
| 3 | 验证历史消息 | 被移除者的历史消息保留 |

**Session Owner 被移除**：

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 创建者尝试移除 Session Owner | 403，提示需先转移 session ownership |

---

## Phase 6：工具引擎与打磨

### TC-P6-001 文件解析与归档

**前置条件**：已有 Ring + 已完成蓝图

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 上传 PDF 文件 | AI 解析文件内容 |
| 2 | AI 推荐归档 | archive_suggestion event 包含结构化提取结果 |
| 3 | 确认归档 | 生成 Markdown + 更新 graph.json |

**异常路径**：

- 不支持的文件格式 → 400
- 文件过大 → 413
- PDF 内容为空（扫描件无 OCR）→ 返回提示

### TC-P6-002 搜索

**前置条件**：已有 Ring + 图谱中有多个节点

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | POST /api/v1/rings/{ringId}/search `{"query": "定价策略", "search_type": "keyword"}` | 返回匹配的节点列表，包含 snippet 和 score |
| 2 | POST /api/v1/rings/{ringId}/search/global `{"query": "本周决策", "include_conversations": true}` | 跨图谱搜索，返回节点 + 对话片段 |
| 3 | 搜索无结果 | 返回空列表，`total: 0` |

**异常路径**：

- query 为空 → 400
- graph_ids 包含不存在的图谱 → 404
- time_range 格式错误 → 400

### TC-P6-003 预设工作流

**前置条件**：已有 Ring + 已完成蓝图

| 工作流 | 操作 | 预期结果 |
|--------|------|---------|
| 会议归档 | 上传会议记录 → AI 提取关键信息 | 生成结构化 Markdown，推荐挂载到图谱节点 |
| 学习中心 | 上传 PDF → AI 解读 + 概念提取 | 生成知识解读 + 推荐创建/更新图谱节点 |
| 深度调研 | 对话中说"调研 XX" | AI 聚合资源 → 生成报告 Markdown → 推荐挂载 |

**异常路径**：

- 只读角色使用工作流 → 403
- 工作流执行中 LLM 超时 → error event，已提取的部分保留

### TC-P6-004 导出

**前置条件**：已有 Ring + 图谱 + 对话历史

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | POST exports/graph-image `{"graph_id": "g1", "format": "svg"}` | 返回 SVG 文件 |
| 2 | GET exports/markdown/{nodeId} | 返回 Markdown 文件 |
| 3 | POST exports/conversation `{"conversation_id": "c1", "format": "markdown"}` | 返回对话 Markdown |
| 4 | POST exports/backup | 返回 .tar.gz 压缩包 |
| 5 | GET exports/graph-json/{graphId} | 返回 graph.json 原始数据 |

**异常路径**：

- 只读角色导出对话 → 403
- 只读角色导出图谱图片和 graph.json → 200（允许）
- nodeId 不存在 → 404
- 不支持的 format → 400

---

## E2E 测试用例（Playwright）

### E2E-001 完整新用户旅程

```
安装 → 首次 Setup → 创建 Ring → 蓝图快速路径 → 对话 → 归档
```

覆盖：Setup 向导 + Ring 创建 + 蓝图模板选择 + 对话 + 归档 commit

### E2E-002 成员加入与协作

```
创建者生成邀请 → 新用户安装 → 加入 Ring → 对话 → 提交 PR → 创建者审核合并
```

覆盖：邀请机制 + 安装导航页 + PR 流程

### E2E-003 Session 多人讨论

```
创建 Session → 邀请成员 → 多人发消息 → AI 回复广播 → Session 归档
```

覆盖：Session 创建 + WebSocket 消息中转 + 归档

### E2E-004 Auto 模式

```
进入 Auto 模式 → 上传文件 → AI 自动归档 → 退出 Auto 模式
```

覆盖：Auto 模式切换 + 自动归档

---

## 测试优先级矩阵

| 优先级 | Phase | 用例 |
|--------|-------|------|
| P0（阻塞） | 1 | TC-P1-001, TC-P1-002, TC-P1-004 |
| P0（阻塞） | 2 | TC-P2-001, TC-P2-003 |
| P0（阻塞） | 3 | TC-P3-001, TC-P3-003 |
| P0（阻塞） | 4 | TC-P4-002, TC-P4-004 |
| P0（阻塞） | 5 | TC-P5-001, TC-P5-003 |
| P1（核心） | 2 | TC-P2-002, TC-P2-004, TC-P2-005 |
| P1（核心） | 3 | TC-P3-002 |
| P1（核心） | 4 | TC-P4-001, TC-P4-003 |
| P1（核心） | 5 | TC-P5-002, TC-P5-004, TC-P5-005, TC-P5-006 |
| P1（核心） | 6 | TC-P6-001, TC-P6-002 |
| P2（重要） | 5 | TC-P5-007, TC-P5-008, TC-P5-009, TC-P5-010 |
| P2（重要） | 6 | TC-P6-003, TC-P6-004 |
| P2（重要） | E2E | E2E-001, E2E-002, E2E-003, E2E-004 |
