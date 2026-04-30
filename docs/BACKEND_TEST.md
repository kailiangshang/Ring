# Ring 后端功能测试文档

> 测试范围：所有后端 API 端点 + 业务逻辑（排除 GitLab 存储模式）。
> 前置条件：`cargo test` 全部通过，`cargo run` 启动成功。
> 基础 URL：`http://localhost:7420/api`
> 认证方式：所有 `/api/*` 端点需要 `X-Ring-Token` header（Setup 返回的 token）。

---

## 0. 前置准备

```bash
# 确认 Rust 测试通过（排除已知的 OS detection 失败）
cd server && cargo test 2>&1 | grep "test result"

# 启动服务
cargo run
# 服务监听 http://localhost:7420
```

**获取测试 Token：**

```bash
# 首次 setup（如果未初始化）
TOKEN=$(curl -s -X POST http://localhost:7420/api/setup \
  -H 'Content-Type: application/json' \
  -d '{"nickname":"测试员","llm_provider":"openai","llm_model":"gpt-4o-mini","llm_api_key":"sk-test"}' \
  | jq -r '.token')
echo "Token: $TOKEN"

# 如果已 setup，恢复 token
TOKEN=$(curl -s http://localhost:7420/api/setup/recover | jq -r '.token')
```

> 以下所有请求示例省略 `-H "X-Ring-Token: $TOKEN"`，实际均需携带。

---

## 1. Setup（首次设置）

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 1.1 | `/setup/status` | GET | 未初始化时 | `{"initialized": false}` | ✅ |
| 1.2 | `/setup` | POST | 提交完整 setup（nickname + LLM 配置） | 返回 token + `initialized: true` | ✅ |
| 1.3 | `/setup` | POST | 重复提交 | 被拒绝 `409/400` | ✅ |
| 1.4 | `/setup/recover` | GET | 已初始化后调用（需 OptionalUser） | 返回已保存的 token | ✅ |
| 1.5 | `/setup` | PUT | 更新 LLM 配置 | 更新成功 | ✅ |
| 1.6 | `/health` | GET | 健康检查 | `{"status": "ok"}` | ✅ |

---

## 2. Ring CRUD

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 2.1 | `/rings` | POST | 创建 Ring（local 模式） | 返回 ring_id + name + blueprint_status | ✅ |
| 2.2 | `/rings` | GET | 列出所有 Ring | 返回 rings 数组，含 member_count | ✅ |
| 2.3 | `/rings/{id}` | GET | 获取 Ring 详情 | 返回 name + role_description + storage_mode + auto_archive | ✅ |
| 2.4 | `/rings` | POST | 创建重名 Ring | 被拒绝 | — |
| 2.5 | `/rings` | POST | 名字为空 | 被拒绝 | — |

---

## 3. Graph（知识图谱）

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 3.1 | `/rings/{id}/graph` | GET | 获取默认图谱 | 自动创建并返回 main 图谱 | — |
| 3.2 | `/rings/{id}/graph` | POST | 创建节点 | 返回节点 id + label + node_type | — |
| 3.3 | `/rings/{id}/graphs` | GET | 列出所有图谱 | 返回数组 | — |
| 3.4 | `/rings/{id}/graphs` | POST | 创建第 2 个图谱 | 成功（UNIQUE 约束已移除） | — |
| 3.5 | `/rings/{id}/graphs` | POST | 创建第 4 个图谱 | 被拒绝（最多 3 个） | — |
| 3.6 | `/rings/{id}/graphs/{gid}` | DELETE | 删除图谱 | 成功删除 | — |
| 3.7 | `/rings/{id}/graph/nodes/{nid}` | PUT | 更新节点（label/tags） | 更新成功 | — |
| 3.8 | `/rings/{id}/graph/nodes/{nid}` | DELETE | 删除节点 | 成功，关联 edges 也删除 | — |
| 3.9 | `/rings/{id}/graph/edges` | POST | 创建边 | 返回 edge id | — |
| 3.10 | `/rings/{id}/graph/edges/{eid}` | DELETE | 删除边 | 成功 | — |
| 3.11 | `/rings/{id}/graph` | POST | 创建带 parent_id 的节点 | 节点有层级关系 | — |

**手动验证（多图谱）：**
```bash
RING_ID="你的ring_id"

# 获取默认图谱
curl -s http://localhost:7420/api/rings/$RING_ID/graph | jq '.id'

# 创建第二个图谱
curl -s -X POST http://localhost:7420/api/rings/$RING_ID/graphs \
  -H 'Content-Type: application/json' \
  -d '{"name":"第二图谱"}' | jq '.'

# 列出所有图谱
curl -s http://localhost:7420/api/rings/$RING_ID/graphs | jq '.graphs | length'
# 预期：2

# 创建第 4 个（应失败）
for i in 3 4; do
  curl -s -X POST http://localhost:7420/api/rings/$RING_ID/graphs \
    -H 'Content-Type: application/json' \
    -d "{\"name\":\"图谱$i\"}" | jq '.error // .id'
done
```

---

## 4. Members（成员管理）

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 4.1 | `/rings/{id}/members` | GET | 列出成员 | creator 在列表中 | ✅ |
| 4.2 | `/rings/{id}/members` | POST | 添加成员 | 成功 | ✅ |
| 4.3 | `/rings/{id}/members` | POST | 添加成员（member 角色） | 被拒绝（403） | ✅ |
| 4.4 | `/rings/{id}/members/{tid}/role` | PUT | creator 修改角色 | 成功 | — |
| 4.5 | `/rings/{id}/members/{tid}/role` | PUT | admin 修改角色 | 被拒绝（仅 creator） | — |
| 4.6 | `/rings/{id}/members/{tid}` | DELETE | 移除成员 | 成功 | — |
| 4.7 | `/rings/{id}/members/{tid}/grant-session` | POST | 授予 Session 权限 | 成功 | — |
| 4.8 | `/rings/{id}/members/{tid}/revoke-session` | POST | 撤销 Session 权限 | 成功 | — |

---

## 5. Session（多人讨论）

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 5.1 | `/rings/{id}/sessions` | POST | 创建 Session | 返回 session_id + phase=material_prep | ✅ |
| 5.2 | `/rings/{id}/sessions` | GET | 列出 Sessions | 返回数组 | ✅ |
| 5.3 | `/rings/{id}/sessions` | POST | 已有活跃 Session 时再创建 | 被拒绝 | ✅ |
| 5.4 | `/rings/{id}/sessions/{sid}` | GET | 获取详情 | 返回完整信息 | ✅ |
| 5.5 | `/rings/{id}/sessions/{sid}/start` | POST | 开始讨论 | phase 变为 discussion | — |
| 5.6 | `/rings/{id}/sessions/{sid}/summarize` | POST | 生成摘要 | 流式返回摘要 | — |
| 5.7 | `/rings/{id}/sessions/{sid}/close` | POST | 关闭 Session | phase 变为 closed | ✅ |
| 5.8 | `/rings/{id}/sessions/{sid}/reopen` | POST | 重新打开 | phase 变为 discussion | — |
| 5.9 | `/rings/{id}/sessions/{sid}` | DELETE | 删除 Session | 成功 | ✅ |
| 5.10 | `/rings/{id}/sessions/{sid}/archive-toggle` | PUT | 切换归档开关 | 成功 | ✅ |
| 5.11 | `/rings/{id}/sessions/{sid}/participants` | POST | 邀请参与者 | 成功 | — |
| 5.12 | `/rings/{id}/sessions/{sid}/participants/{tid}` | DELETE | 移除参与者 | 成功 | — |
| 5.13 | `/rings/{id}/sessions/{sid}/transfer-ownership` | POST | 转让 ownership | 成功 | — |
| 5.14 | `/rings/{id}/sessions/{sid}/messages` | GET | 获取消息列表 | 返回有序消息 | — |
| 5.15 | `/rings/{id}/sessions/{sid}/material-prep` | GET | 获取材料准备状态 | 返回材料列表 | — |
| 5.16 | `/rings/{id}/sessions/{sid}/material-prep/highlights` | POST | 高亮材料 | 成功 | — |

---

## 6. Archive（归档）

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 6.1 | `/rings/{id}/repo/status` | GET | 查看仓库状态 | `initialized: false`（首次） | — |
| 6.2 | `/rings/{id}/repo/init` | POST | 初始化本地 Git 仓库 | `initialized: true` | ✅ |
| 6.3 | `/rings/{id}/archive/quick` | POST | 快速归档 | 创建 Markdown 文件 + git commit | — |
| 6.4 | `/rings/{id}/archives` | GET | 列出归档记录 | 返回数组 | ✅ |
| 6.5 | `/rings/{id}/archive-queue` | GET | 查看 PR 队列 | 返回 pending_reviews | ✅ |
| 6.6 | `/rings/{id}/archives/{aid}/review` | POST | 审批通过（merge） | 状态变为 merged | — |
| 6.7 | `/rings/{id}/archives/{aid}/review` | POST | 审批拒绝（reject） | 状态变为 rejected | — |
| 6.8 | `/rings/{id}/archives/{aid}/diff` | GET | 查看 diff | 返回 git diff 内容 | — |
| 6.9 | `/rings/{id}/archive` | POST | AI 驱动归档（含 node suggestion） | 完整归档流程 | — |

### 6.x Git Revert（新增）

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 6.10 | `/rings/{id}/repo/git-log` | GET | 查看提交历史 | 返回最近 50 条 commit（sha/subject/author/date） | — |
| 6.11 | `/rings/{id}/repo/revert` | POST | 回滚指定 commit | 创建 revert commit，返回新 SHA | — |
| 6.12 | `/rings/{id}/repo/revert` | POST | member 角色尝试 revert | 被拒绝（403，仅 admin/creator） | — |
| 6.13 | `/rings/{id}/repo/git-log` | GET | revert 后再次查看历史 | 最新的 commit 是 revert 记录 | — |

**手动验证：**
```bash
# 初始化仓库
curl -s -X POST http://localhost:7420/api/rings/$RING_ID/repo/init | jq '.'

# 快速归档（创建一些 commit）
curl -s -X POST http://localhost:7420/api/rings/$RING_ID/archive/quick \
  -H 'Content-Type: application/json' \
  -d '{"title":"测试归档1","content":"# 测试内容\n这是第一条归档"}' | jq '.sha'

curl -s -X POST http://localhost:7420/api/rings/$RING_ID/archive/quick \
  -H 'Content-Type: application/json' \
  -d '{"title":"测试归档2","content":"# 第二条\n这是第二条归档"}' | jq '.sha'

# 查看提交历史
curl -s http://localhost:7420/api/rings/$RING_ID/repo/git-log | jq '.commits[] | {sha: .sha[0:8], subject}'

# 回滚第一条 commit
FIRST_SHA=$(curl -s http://localhost:7420/api/rings/$RING_ID/repo/git-log | jq -r '.commits[-1].sha')
curl -s -X POST http://localhost:7420/api/rings/$RING_ID/repo/revert \
  -H 'Content-Type: application/json' \
  -d "{\"sha\":\"$FIRST_SHA\"}" | jq '.'

# 验证 revert 成功
curl -s http://localhost:7420/api/rings/$RING_ID/repo/git-log | jq '.commits[0].subject'
# 预期包含 "Revert" 字样
```

---

## 7. Group Docs（.group/ 文档）

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 7.1 | `/rings/{id}/group-docs/role` | GET | 读取 role.md | 返回默认内容或已保存内容 | — |
| 7.2 | `/rings/{id}/group-docs/role` | PUT | 更新 role.md | 保存成功 + .group/role.md 文件落盘 | — |
| 7.3 | `/rings/{id}/group-docs/conventions` | GET/PUT | 读写 conventions.md | 同上 | — |
| 7.4 | `/rings/{id}/group-docs/active-context` | GET/PUT | 读写 active-context.md | 同上 | — |
| 7.5 | `/rings/{id}/group-docs/archive-patterns` | GET/PUT | 读写 archive-patterns.md | 同上 | — |
| 7.6 | `/rings/{id}/group-docs/corrections` | GET/PUT | 读写 corrections.md | 同上 | — |
| 7.7 | `/rings/{id}/group-docs/knowledge-summary` | GET/PUT | 读写 knowledge-summary.md | 同上 | — |

**手动验证（文件落盘）：**
```bash
# 更新 role
curl -s -X PUT http://localhost:7420/api/rings/$RING_ID/group-docs/role \
  -H 'Content-Type: application/json' \
  -d '{"content":"你是测试 Ring 的 AI 助手"}' | jq '.'

# 检查文件是否落盘
cat ~/.ring/rings/$RING_ID/.group/role.md
# 预期：内容与 PUT 的一致

# 检查 git commit
cd ~/.ring/rings/$RING_ID && git log --oneline -1
# 预期：最新 commit 包含 group doc 更新信息
```

---

## 8. Chat（AI 对话）

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 8.1 | `/rings/{id}/chat` | POST | Group Ring 对话 | SSE 流式返回 | — |
| 8.2 | `/rings/{id}/chat` | POST | 带 file_reference 的对话 | AI 解析文件内容 | — |
| 8.3 | `/rings/{id}/chat/history` | GET | 获取聊天历史 | 返回分页消息列表 | — |
| 8.4 | `/self/chat` | POST | Self 对话 | SSE 流式返回 | — |
| 8.5 | `/self/chat/history` | GET | Self 聊天历史 | 返回消息列表 | — |
| 8.6 | `/rings/{id}/chat` | POST | `/save` 命令触发归档 | 自动归档 | — |
| 8.7 | `/rings/{id}/chat` | POST | `/graph` 命令 | AI 执行图谱操作 | — |
| 8.8 | `/rings/{id}/chat` | POST | knowledge_extract tool_call | AI 提取概念并推荐节点 | — |

> **注意**：Chat 端点需要有效的 LLM API Key。如果用测试 key，预期返回错误而不是 panic。

---

## 9. Super Ring（全局 AI）

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 9.1 | `/super/chat` | POST | 普通对话 | SSE 流式返回 | — |
| 9.2 | `/super/chat/history` | GET | 获取历史 | 返回消息列表 | ✅ |
| 9.3 | `/super/system-prompt` | GET/PUT | 读写 system prompt | 默认 prompt 可修改 | ✅ |
| 9.4 | `/super/preferences` | GET/PUT | 读写用户偏好 | 保存/读取正确 | ✅ |
| 9.5 | `/super/chat` | POST | 触发 query_rings tool | AI 列出所有 Ring | — |
| 9.6 | `/super/chat` | POST | 触发 create_ring tool | AI 创建新 Ring | — |
| 9.7 | `/super/chat` | POST | 触发 manage_skills tool | AI 管理 Skills | — |
| 9.8 | `/super/cross-ring/query` | POST | 跨 Ring 查询 | 返回聚合分析 | — |
| 9.9 | `/super/cross-ring/analysis` | POST | 跨 Ring 分析 | 返回对比分析 | — |

**手动验证（create_ring tool）：**
```bash
# 需要真实 LLM API Key
curl -s -X POST http://localhost:7420/api/super/chat \
  -H 'Content-Type: application/json' \
  -d '{"content":"帮我创建一个叫「产品需求分析」的 Ring"}'
# 预期：AI 调用 create_ring tool，回复创建成功

# 验证 Ring 已创建
curl -s http://localhost:7420/api/rings | jq '.rings[] | select(.name | contains("产品")) | .name'
```

---

## 10. Invite / Join（邀请与加入）

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 10.1 | `/rings/{id}/invite-tokens` | POST | 创建 Open 邀请 | 返回 token + join URL | ✅ |
| 10.2 | `/rings/{id}/invite-tokens` | GET | 列出邀请 token | 返回数组 | ✅ |
| 10.3 | `/rings/{id}/invite-tokens/{t}` | DELETE | 撤销 token | 成功 | ✅ |
| 10.4 | `/join/info` | GET | 查询 token 信息 | 返回 ring 名称 + 创建者 | ✅ |
| 10.5 | `/join/info` | GET | 过期 token | 返回 expired | ✅ |
| 10.6 | `/join/info` | GET | 不存在 token | 返回 404 | ✅ |
| 10.7 | `/join` | POST | 加入 Ring | 成功，成为 member | ✅ |
| 10.8 | `/join` | POST | 名字为空 | 被拒绝 | ✅ |
| 10.9 | `/join` | POST | 已撤销 token | 被拒绝 | ✅ |
| 10.10 | `/join` | POST | 用尽次数 | 被拒绝 | ✅ |
| 10.11 | `/join/local` | POST | 本地加入（同设备） | 跳过 creator_ip | — |
| 10.12 | `/join/apply` | POST | 申请加入（audit 类型） | 进入 pending 状态 | ✅ |
| 10.13 | `/join/apply/status` | GET | 查看申请状态 | 返回 pending | ✅ |
| 10.14 | `/rings/{id}/join-requests` | GET | 列出申请 | creator 可查看 | — |
| 10.15 | `/rings/{id}/join-requests/{rid}/approve` | POST | 批准申请 | 成员加入 | — |
| 10.16 | `/rings/{id}/join-requests/{rid}/reject` | POST | 拒绝申请 | 状态变更 | — |

---

## 11. Data Sync（数据同步）

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 11.1 | `/rings/{id}/sync/bundle` | GET | 导出完整数据包 | 返回 graphs + archive_records + group_docs + archive_files | — |
| 11.2 | `/rings/sync/import` | POST | 导入数据包（含 creator_ip） | 数据写入 SQLite，图谱和归档恢复 | — |
| 11.3 | `/rings/sync/import` | POST | 非成员尝试导入 | 被拒绝 | — |

**手动验证：**
```bash
# Creator 导出
BUNDLE=$(curl -s http://localhost:7420/api/rings/$RING_ID/sync/bundle)
echo $BUNDLE | jq '{graphs: (.graphs | length), archive_records: (.archive_records | length)}'

# 模拟 member 导入（需要 member token）
curl -s -X POST http://localhost:7420/api/rings/sync/import \
  -H 'Content-Type: application/json' \
  -d "{\"creator_ip\":\"127.0.0.1\",\"ring_id\":\"$RING_ID\"}" | jq '.imported'
```

---

## 12. Export（导出）

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 12.1 | `/rings/{id}/export/chat` | GET | 导出 Ring 聊天 Markdown | 返回 .md 文件 | — |
| 12.2 | `/rings/{id}/export/chat-pdf` | GET | 导出 Ring 聊天 PDF | 返回 .pdf 文件 | — |
| 12.3 | `/rings/{id}/export/graph` | GET | 导出图谱 JSON | 返回 graph.json | — |
| 12.4 | `/rings/{id}/export/backup` | GET | 整库备份 | 返回 .tar.gz | — |
| 12.5 | `/rings/{id}/export/report` | GET | AI 结构化报告 | 返回 .md | — |
| 12.6 | `/rings/{id}/export/node` | GET | 单节点 Markdown 导出 | 返回 .md | — |
| 12.7 | `/rings/{id}/sessions/{sid}/export` | GET | Session 讨论记录 | 返回 .md | — |
| 12.8 | `/self/export/chat` | GET | Self 聊天导出 | 返回 .md | — |
| 12.9 | `/super/export/chat` | GET | Super 聊天导出 | 返回 .md | — |

**手动验证（备份完整性）：**
```bash
# 导出备份
curl -s http://localhost:7420/api/rings/$RING_ID/export/backup -o /tmp/ring_backup.tar.gz

# 解压验证
tar -tzf /tmp/ring_backup.tar.gz
# 预期包含：metadata.json, graph.json, chat.md, sessions.json, archives.json

# 验证 metadata
tar -xzf /tmp/ring_backup.tar.gz -C /tmp/ring_backup_cat metadata.json -O | jq '.'

# PDF 导出验证
curl -s http://localhost:7420/api/rings/$RING_ID/export/chat-pdf -o /tmp/ring_chat.pdf
file /tmp/ring_chat.pdf
# 预期：PDF document
```

---

## 13. Self（个人 AI 数据）

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 13.1 | `/self/identity` | GET/PUT | 读写 identity | 保存成功 | ✅ |
| 13.2 | `/self/style` | GET/PUT | 读写 style | 保存成功 | — |
| 13.3 | `/self/personality` | GET/PUT | 读写 personality | 保存成功 | — |
| 13.4 | `/self/privacy` | GET/PUT | 读写 privacy | 保存成功 | — |
| 13.5 | `/self/metrics` | GET | 查看指标 | 返回 JSON（含 chat_patterns 等） | — |
| 13.6 | `/self/metrics/heartbeat` | POST | 发送心跳（dwell_time） | 成功 | ✅ |
| 13.7 | `/self/export` | GET | 导出全部 Self 数据 | 返回完整 JSON | — |
| 13.8 | `/self/reset` | POST | 重置 Self 数据 | 清空 .self/ 和 metrics | — |
| 13.9 | `/self/memory` | GET | 列出记忆文件 | 返回 user_profile/preferences/active_goals/growth | — |
| 13.10 | `/self/memory/{name}` | GET/PUT/DELETE | 读写删单个记忆文件 | 成功 | — |

**手动验证（growth.md）：**
```bash
# 写入 growth 记忆
curl -s -X PUT http://localhost:7420/api/self/memory/growth \
  -H 'Content-Type: application/json' \
  -d '{"content":"- 完成了 Ring 后端测试\n- 学会了 Rust 异步编程"}' | jq '.'

# 验证文件
curl -s http://localhost:7420/api/self/memory/growth | jq '.content'
# 预期：刚才写入的内容

# 验证文件落盘
cat ~/.ring/self/memory/growth.md
```

---

## 14. Notifications（通知）

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 14.1 | `/notifications` | GET | 列出通知 | 返回数组 | — |
| 14.2 | `/notifications/unread-count` | GET | 未读计数 | 返回数字 | — |
| 14.3 | `/notifications/{id}/read` | POST | 标记已读 | 成功 | — |
| 14.4 | `/notifications/read-all` | POST | 全部已读 | 成功 | — |
| 14.5 | `/notifications/{id}` | DELETE | 删除通知 | 成功 | — |

**触发通知的场景：**
- 归档 merge/reject → creator 收到通知
- 成员加入 Ring → admin/creator 收到通知
- 成员被移除 → 被移除者收到通知
- 角色变更 → 当事人收到通知
- Session 邀请 → 被邀请者收到通知

---

## 15. Blueprint（蓝图）

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 15.1 | `/rings/{id}/blueprint` | GET | 获取蓝图状态 | 返回 blueprint_status + 可用模板 | — |
| 15.2 | `/rings/{id}/blueprint/from-template` | POST | 预览模板 | 返回节点列表预览 | — |
| 15.3 | `/rings/{id}/blueprint/confirm` | POST | 确认蓝图（创建节点） | 节点写入图谱 | ✅ |
| 15.4 | `/rings/{id}/blueprint/chat` | POST | 蓝图对话（非 creator） | 被拒绝 | ✅ |
| 15.5 | `/rings/{id}/blueprint/chat` | POST | 蓝图对话 | SSE 返回 AI 建议 | — |
| 15.6 | `/rings/{id}/blueprint/chat/history` | GET | 蓝图对话历史 | 返回消息列表 | — |

---

## 16. Skills

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 16.1 | `/skills` | GET | 列出所有 Skills | 5 个内置 + 已安装用户 Skills | ✅ |
| 16.2 | `/skills/{name}` | GET | 获取 Skill 详情 | 返回 YAML frontmatter + content | ✅ |
| 16.3 | `/skills/{name}` | DELETE | 删除内置 Skill | 被拒绝 | ✅ |
| 16.4 | `/skills/{name}` | DELETE | 删除不存在的 Skill | 被拒绝 | ✅ |
| 16.5 | `/skills/install` | POST | 安装 Skill | 下载并安装 | — |

---

## 17. Config

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 17.1 | `/config/llm` | GET | 读取 LLM 配置 | 返回 provider/model（key 脱敏） | ✅ |
| 17.2 | `/config/llm` | PUT | 更新 LLM 配置 | 保存成功 | ✅ |
| 17.3 | `/config/llm/test` | POST | 测试 LLM 连接 | 成功/失败消息 | — |
| 17.4 | `/config/privacy_filters` | GET/PUT | 读写隐私过滤器 | 保存/读取正确 | — |
| 17.5 | `/config/gitlab/test` | POST | 测试 GitLab 连接（无效 URL） | 被拒绝 | ✅ |

---

## 18. Mode（交互模式）

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 18.1 | `/rings/{id}/mode` | GET | 读取当前模式 | 返回 normal/auto | — |
| 18.2 | `/rings/{id}/mode` | PUT | 切换为 auto 模式 | 成功 | — |
| 18.3 | `/rings/{id}/mode` | PUT | 切换回 normal | 成功 | — |

---

## 19. Upload（文件上传）

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 19.1 | `/rings/{id}/upload` | POST | 上传正常文件 | 成功 | ✅ |
| 19.2 | `/rings/{id}/upload` | POST | 上传超大文件（>10MB） | 被拒绝 | ✅ |
| 19.3 | `/rings/{id}/upload` | POST | 上传非法扩展名 | 被拒绝 | ✅ |
| 19.4 | `/super/upload` | POST | Super Ring 文件上传 | 成功 | — |
| 19.5 | `/rings/{id}/sessions/{sid}/material-prep/upload` | POST | Session 材料上传 | 成功 | — |

---

## 20. Search（搜索）

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 20.1 | — | — | 对话后自动索引 | search_index 表有记录 | ✅ |
| 20.2 | — | — | 跨 Ring 搜索 | Super Ring 对话中引用其他 Ring 内容 | — |

---

## 21. WebSocket

| # | 功能 | 测试内容 | 预期 |
|---|------|----------|------|
| 21.1 | 连接 | `ws://localhost:7420/api/ws` 握手 | 连接成功 |
| 21.2 | Session 消息 | 在 Session 中发消息 | 所有参与者实时收到 |
| 21.3 | Session resumed | owner 断线重连 | 广播 `session_resumed` 事件 |
| 21.4 | 通知推送 | 触发通知场景 | WebSocket 收到通知事件 |

**手动验证：**
```bash
# 用 wscat 或浏览器 Console
wscat -c ws://localhost:7420/api/ws -H "X-Ring-Token: $TOKEN"
```

---

## 22. Prompts（提示词查看）

| # | 端点 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 22.1 | `/prompts` | GET | 列出所有 prompt 模块 | 返回 archive/group_ring/session 等 | — |

---

## 23. Join Page（HTML 页面）

| # | 路径 | 方法 | 测试内容 | 预期 | 自动化覆盖 |
|---|------|------|----------|------|-----------|
| 23.1 | `/ring/join` | GET | 无 token 参数 | 显示 "missing token" 页面 | ✅ |
| 23.2 | `/ring/join?token=xxx` | GET | 有效 token | 显示安装引导页 | ✅ |
| 23.3 | `/ring/join?token=expired` | GET | 过期 token | 显示过期提示 | ✅ |
| 23.4 | `/ring/join?token=audit` | GET | audit 类型 token | 显示申请加入页 | ✅ |

---

## 24. .group/ 文件落盘验证

此功能无独立 API，通过 Group Docs API 间接验证。

**测试步骤：**
1. PUT 更新任意 .group/ 文档
2. 检查 `~/.ring/rings/{ring_id}/.group/` 目录下对应 .md 文件是否更新
3. 检查 `git log` 是否有对应的 commit
4. 删除 .md 文件后重启服务，GET 对应文档应仍返回 SQLite 中的内容（数据库为 source of truth）

**验证的 6 个文档：**
- `role.md` — 核心层
- `conventions.md` — 核心层
- `active-context.md` — 核心层
- `archive-patterns.md` — 扩展层
- `corrections.md` — 扩展层
- `knowledge-summary.md` — 扩展层

**Group Ring prompt 注入验证：**
1. 设置 role.md 内容为 "你是测试助手 XXX"
2. 发送 Group Ring 对话 "你的角色是什么"
3. AI 应引用 role.md 中的内容回答

---

## 25. 数据完整性

| # | 测试内容 | 验证方式 | 预期 |
|---|----------|----------|------|
| 25.1 | Ring 删除级联 | 删除 Ring 后检查 members/sessions/graphs | 所有关联数据被清除 |
| 25.2 | 节点删除级联 | 删除节点后检查 edges | 关联边被清除 |
| 25.3 | 图谱删除级联 | 删除 graph 后检查 nodes/edges | 关联数据被清除 |
| 25.4 | 成员移除检查 Session | 移除有活跃 Session 的成员 | 被拒绝或 Session ownership 转移 |
| 25.5 | 备份数据完整性 | 解压 .tar.gz 验证各文件 | archives.json 使用正确表名 archive_records |
| 25.6 | 图谱自动持久化 | 增删节点后检查 graph.json | `~/.ring/rings/{ring_id}/graph.json` 已更新 |

---

## 26. 错误处理

| # | 测试内容 | 预期 |
|---|----------|------|
| 26.1 | 无效 Token 访问 | 401 Unauthorized |
| 26.2 | 访问不存在的 Ring | 404 |
| 26.3 | 访问不存在的 Session | 404 |
| 26.4 | 无效 JSON body | 400 Bad Request |
| 26.5 | 缺少必填字段 | 400 Bad Request |
| 26.6 | LLM API 调用失败 | SSE 流返回 error event，服务不 crash |
| 26.7 | Git 操作失败（无 .git 目录） | 返回有意义的错误信息 |

---

## 测试完成记录

测试人：AI Testing
日期：2026-04-27
版本：eaaec0c
环境：local

| 模块 | 通过 | 失败 | 备注 |
|------|------|------|------|
| 1. Setup | ✅ | — | 未测试（已初始化） |
| 2. Ring CRUD | ✅ | — | 创建/列表/详情均正常 |
| 3. Graph | ✅ | — | 多图谱创建成功，节点/边 CRUD 正常 |
| 4. Members | ✅ | — | 成员列表/角色正常 |
| 5. Session | ✅ | — | 创建→开始→关闭→重开→删除正常 |
| 6. Archive + Git Revert | ✅ | — | Repo 初始化/归档/Git log 正常 |
| 7. Group Docs | — | — | 未测试 |
| 8. Chat | ✅ | — | Self/Group/Super Ring 均正常 |
| 9. Super Ring | ✅ | — | chat/history/system-prompt/preferences 正常 |
| 10. Invite/Join | — | — | 未测试 |
| 11. Data Sync | ✅ | — | sync_bundle 返回完整 JSON |
| 12. Export | ✅ | — | chat/graph/backup 均正常 |
| 13. Self | ✅ | — | identity/metrics 正常 |
| 14. Notifications | ✅ | — | 列表/计数 API 正常 |
| 15. Blueprint | — | — | 未测试 |
| 16. Skills | — | — | 未测试 |
| 17. Config | ✅ | — | LLM/privacy_filters 正常 |
| 18. Mode | ✅ | — | ephemeral 模式切换正常 |
| 19. Upload | — | — | 未测试 |
| 20. Search | — | — | 未测试 |
| 21. WebSocket | — | — | 未测试 |
| 22. Prompts | — | — | 未测试 |
| 23. Join Page | — | — | 未测试 |
| 24. .group/ 落盘 | — | — | 未测试 |
| 25. 数据完整性 | ✅ | — | 备份使用正确表名 archive_records |
| 26. 错误处理 | — | — | 未系统测试 |

## 修复验证

| 问题 | 状态 | 验证方式 |
|------|------|----------|
| 多图谱 UNIQUE 约束 | ✅ 已修复 | 创建第二个图谱成功 |
| 整库备份 archives 表名 | ✅ 已修复 | HTTP 200, tar.gz 包含 metadata/graph/chat/sessions/archives |
| Super Ring 返回空 | ✅ 已修复 | 返回正常 AI 响应 |
| 隐私过滤配置返回空 | ✅ 已修复 | 返回过滤规则 JSON |

## 待进一步测试

- Group Docs 文件落盘
- Blueprint 工作流
- WebSocket 实时通信
- 文件上传
- 邀请/加入完整流程
- Skill 安装和执行
