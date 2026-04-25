# Ring API Testing Plan

> **For agentic workers:** Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 启动 cargo run，通过 bash 脚本测试后端 API 接口，验证功能是否存在问题。

**前置条件:** 已完成初始化（配置了 chat api，未配置 gitlab）

**测试原则:**
- 先测无需认证的接口（setup, health）
- 再测需认证的接口（需要先完成 setup 获取 token）
- 涉及 LLM 调用的接口记录预期失败
- 涉及 GitLab 的接口跳过

---

## Phase 1: 服务启动

- [ ] **Step 1: 检查 target 目录是否存在编译产物**

```bash
ls -la /Users/kaiiangs/Desktop/open-source-project/Ring/server/target/debug/ 2>/dev/null || echo "need build"
```

- [ ] **Step 2: 启动服务（后台运行）**

```bash
cd /Users/kaiiangs/Desktop/open-source-project/Ring/server
cargo run &
sleep 5
```

**预期:** 服务在端口 7420 启动

---

## Phase 2: 无认证接口测试

- [ ] **Step 3: 测试 health 接口**

```bash
curl -s http://localhost:7420/api/health
```

**预期:** 返回 `{"ok": true}` 或类似

- [ ] **Step 4: 测试 setup/status**

```bash
curl -s http://localhost:7420/api/setup/status
```

**预期:** 返回 `{"is_setup": true/false, "step": ...}`

---

## Phase 3: 认证与 Setup Flow

- [ ] **Step 5: 获取 setup token（若未 setup）**

```bash
# 如果 is_setup=false，需要先 setup
curl -s -X POST http://localhost:7420/api/setup \
  -H "Content-Type: application/json" \
  -d '{
    "display_name": "TestUser",
    "avatar": "🤖",
    "llm_provider": "openai",
    "llm_api_key": "test-key",
    "llm_model": "gpt-4o"
  }'
```

**预期:** 返回 token_id，保存该 token 用于后续请求

---

## Phase 4: Ring CRUD 测试（需认证）

假设获取到的 token 为 `TEST_TOKEN`，实际值从 Step 5 获取

- [ ] **Step 6: 创建 Ring**

```bash
curl -s -X POST http://localhost:7420/api/rings \
  -H "Content-Type: application/json" \
  -H "X-Ring-Token: TEST_TOKEN" \
  -d '{
    "name": "测试 Ring",
    "role_description": "测试用 Ring"
  }'
```

**预期:** 返回 201 + ring 对象，包含 id

- [ ] **Step 7: 列出 Rings**

```bash
curl -s http://localhost:7420/api/rings -H "X-Ring-Token: TEST_TOKEN"
```

**预期:** 返回 rings 列表

- [ ] **Step 8: 获取单个 Ring**

```bash
curl -s http://localhost:7420/api/rings/{ring_id} -H "X-Ring-Token: TEST_TOKEN"
```

**预期:** 返回 ring 详情

---

## Phase 5: Graph 操作测试

- [ ] **Step 9: 获取 Graph**

```bash
curl -s http://localhost:7420/api/rings/{ring_id}/graph -H "X-Ring-Token: TEST_TOKEN"
```

**预期:** 返回 graph 对象（可能为空）

- [ ] **Step 10: 创建 Graph**

```bash
curl -s -X POST http://localhost:7420/api/rings/{ring_id}/graphs \
  -H "Content-Type: application/json" \
  -H "X-Ring-Token: TEST_TOKEN" \
  -d '{"name": "测试图谱"}'
```

**预期:** 返回 201 + graph 对象

- [ ] **Step 11: 创建 Node**

```bash
curl -s -X POST "http://localhost:7420/api/rings/{ring_id}/graph" \
  -H "Content-Type: application/json" \
  -H "X-Ring-Token: TEST_TOKEN" \
  -d '{
    "label": "测试节点",
    "node_type": "leaf",
    "content": "# 测试节点\n内容"
  }'
```

**预期:** 返回 201 + node 对象

- [ ] **Step 12: 列出 Graphs**

```bash
curl -s http://localhost:7420/api/rings/{ring_id}/graphs -H "X-Ring-Token: TEST_TOKEN"
```

---

## Phase 6: Chat 测试（可能因无 LLM 失败）

- [ ] **Step 13: 发送 Chat 消息**

```bash
curl -s -X POST http://localhost:7420/api/rings/{ring_id}/chat \
  -H "Content-Type: application/json" \
  -H "X-Ring-Token: TEST_TOKEN" \
  -d '{
    "content": "你好",
    "node_refs": [],
    "tag_refs": []
  }'
```

**预期:** SSE 流式响应（可能因无 LLM 而失败）

- [ ] **Step 14: 获取 Chat History**

```bash
curl -s "http://localhost:7420/api/rings/{ring_id}/chat/history?limit=10" \
  -H "X-Ring-Token: TEST_TOKEN"
```

---

## Phase 7: Self 接口测试

- [ ] **Step 15: Self Chat**

```bash
curl -s -X POST http://localhost:7420/api/self/chat \
  -H "Content-Type: application/json" \
  -H "X-Ring-Token: TEST_TOKEN" \
  -d '{"content": "你好"}'
```

- [ ] **Step 16: Self Metrics**

```bash
curl -s http://localhost:7420/api/self/metrics -H "X-Ring-Token: TEST_TOKEN"
```

- [ ] **Step 17: Self Memory**

```bash
curl -s http://localhost:7420/api/self/memory -H "X-Ring-Token: TEST_TOKEN"
```

---

## Phase 8: Super Chat 测试

- [ ] **Step 18: Super Chat**

```bash
curl -s -X POST http://localhost:7420/api/super/chat \
  -H "Content-Type: application/json" \
  -H "X-Ring-Token: TEST_TOKEN" \
  -d '{"content": "测试"}'
```

- [ ] **Step 19: Cross Ring Query**

```bash
curl -s -X POST http://localhost:7420/api/super/cross-ring/query \
  -H "Content-Type: application/json" \
  -H "X-Ring-Token: TEST_TOKEN" \
  -d '{"query": "测试", "ring_ids": []}'
```

---

## Phase 9: Session 测试

- [ ] **Step 20: 创建 Session**

```bash
curl -s -X POST http://localhost:7420/api/rings/{ring_id}/sessions \
  -H "Content-Type: application/json" \
  -H "X-Ring-Token: TEST_TOKEN" \
  -d '{
    "title": "测试 Session",
    "skill": "discussion",
    "archivable": true
  }'
```

**预期:** 返回 201 + session 对象

- [ ] **Step 21: 列出 Sessions**

```bash
curl -s "http://localhost:7420/api/rings/{ring_id}/sessions" -H "X-Ring-Token: TEST_TOKEN"
```

- [ ] **Step 22: 获取 Session**

```bash
curl -s "http://localhost:7420/api/rings/{ring_id}/sessions/{session_id}" \
  -H "X-Ring-Token: TEST_TOKEN"
```

- [ ] **Step 23: 关闭 Session**

```bash
curl -s -X POST "http://localhost:7420/api/rings/{ring_id}/sessions/{session_id}/close" \
  -H "X-Ring-Token: TEST_TOKEN"
```

---

## Phase 10: Archive 测试

- [ ] **Step 24: 触发 Archive**

```bash
curl -s -X POST http://localhost:7420/api/rings/{ring_id}/archive \
  -H "Content-Type: application/json" \
  -H "X-Ring-Token: TEST_TOKEN" \
  -d '{"content": "测试归档内容"}'
```

**注意:** 可能因无 GitLab 而失败

- [ ] **Step 25: 列出 Archives**

```bash
curl -s http://localhost:7420/api/rings/{ring_id}/archives -H "X-Ring-Token: TEST_TOKEN"
```

- [ ] **Step 26: Archive Queue**

```bash
curl -s http://localhost:7420/api/rings/{ring_id}/archive-queue -H "X-Ring-Token: TEST_TOKEN"
```

---

## Phase 11: Skills 测试

- [ ] **Step 27: 列出 Skills**

```bash
curl -s http://localhost:7420/api/skills -H "X-Ring-Token: TEST_TOKEN"
```

**预期:** 返回 skills 列表

---

## Phase 12: Notifications 测试

- [ ] **Step 28: 列出 Notifications**

```bash
curl -s http://localhost:7420/api/notifications -H "X-Ring-Token: TEST_TOKEN"
```

- [ ] **Step 29: 获取 Unread Count**

```bash
curl -s http://localhost:7420/api/notifications/unread-count -H "X-Ring-Token: TEST_TOKEN"
```

---

## Phase 13: Config 测试

- [ ] **Step 30: 获取 LLM Config**

```bash
curl -s http://localhost:7420/api/config/llm -H "X-Ring-Token: TEST_TOKEN"
```

- [ ] **Step 31: 测试 LLM Config**

```bash
curl -s -X POST http://localhost:7420/api/config/llm/test \
  -H "Content-Type: application/json" \
  -H "X-Ring-Token: TEST_TOKEN" \
  -d '{
    "provider": "openai",
    "api_key": "test-key",
    "model": "gpt-4o"
  }'
```

---

## Phase 14: Mode 与 Group Docs 测试

- [ ] **Step 32: 获取 Mode**

```bash
curl -s http://localhost:7420/api/rings/{ring_id}/mode -H "X-Ring-Token: TEST_TOKEN"
```

- [ ] **Step 33: 获取 Group Docs**

```bash
curl -s http://localhost:7420/api/rings/{ring_id}/group-docs/role \
  -H "X-Ring-Token: TEST_TOKEN"
```

---

## Phase 15: Blueprint 测试

- [ ] **Step 34: 获取 Blueprint**

```bash
curl -s http://localhost:7420/api/rings/{ring_id}/blueprint -H "X-Ring-Token: TEST_TOKEN"
```

- [ ] **Step 35: 从模板预览**

```bash
curl -s -X POST http://localhost:7420/api/rings/{ring_id}/blueprint/from-template \
  -H "Content-Type: application/json" \
  -H "X-Ring-Token: TEST_TOKEN" \
  -d '{"template": "blank"}'
```

---

## Phase 16: Invite 与 Members 测试

- [ ] **Step 36: 列出 Members**

```bash
curl -s http://localhost:7420/api/rings/{ring_id}/members -H "X-Ring-Token: TEST_TOKEN"
```

- [ ] **Step 37: 创建 Invite Token**

```bash
curl -s -X POST http://localhost:7420/api/rings/{ring_id}/invite-tokens \
  -H "Content-Type: application/json" \
  -H "X-Ring-Token: TEST_TOKEN" \
  -d '{
    "type": "open",
    "role": "member"
  }'
```

- [ ] **Step 38: 列出 Invite Tokens**

```bash
curl -s http://localhost:7420/api/rings/{ring_id}/invite-tokens -H "X-Ring-Token: TEST_TOKEN"
```

---

## Phase 17: Export 测试

- [ ] **Step 39: 导出 Chat**

```bash
curl -s "http://localhost:7420/api/rings/{ring_id}/export/chat" \
  -H "X-Ring-Token: TEST_TOKEN"
```

- [ ] **Step 40: 导出 Graph**

```bash
curl -s "http://localhost:7420/api/rings/{ring_id}/export/graph" \
  -H "X-Ring-Token: TEST_TOKEN"
```

- [ ] **Step 41: 导出 Backup**

```bash
curl -s "http://localhost:7420/api/rings/{ring_id}/export/backup" \
  -H "X-Ring-Token: TEST_TOKEN" \
  -o /tmp/ring_backup.tar.gz
```

---

## Phase 18: Join Flow 测试（无认证）

- [ ] **Step 42: Join Info**

```bash
curl -s http://localhost:7420/api/join/info?token=test-token
```

- [ ] **Step 43: Join Apply Status**

```bash
curl -s http://localhost:7420/api/join/apply/status?token=test-token
```

---

## 测试结果记录

| 接口 | 方法 | 路径 | 状态 | 备注 |
|------|------|------|------|------|
| health | GET | /api/health | PASS/FAIL | |
| setup/status | GET | /api/setup/status | PASS/FAIL | |
| setup | POST | /api/setup | PASS/FAIL | |
| rings | GET | /api/rings | PASS/FAIL | |
| rings | POST | /api/rings | PASS/FAIL | |
| graph | GET | /api/rings/{id}/graph | PASS/FAIL | |
| graphs | POST | /api/rings/{id}/graphs | PASS/FAIL | |
| chat | POST | /api/rings/{id}/chat | PASS/FAIL | |
| ... | ... | ... | ... | |

---

## 注意事项

1. **GitLab 相关跳过**: repo/status, repo/init 等需要 GitLab，跳过
2. **LLM 调用可能失败**: chat, super_chat 等需要 LLM，记录失败原因
3. **Bearer Token**: 测试前从 setup response 获取实际 token
4. **错误响应检查**: 即使 4xx/5xx 也记录响应体

---

## 执行命令汇总

```bash
# 1. 启动服务
cd /Users/kaiiangs/Desktop/open-source-project/Ring/server
cargo run &

# 2. 健康检查
curl -s http://localhost:7420/api/health

# 3. 获取 token（若需要）
# ... (见各阶段)
```