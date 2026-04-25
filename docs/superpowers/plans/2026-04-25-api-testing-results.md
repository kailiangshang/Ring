# Ring API 测试结果报告

**分支**: `api-testing-temp`
**测试时间**: 2026-04-25
**环境**: localhost:7420, token: `user-01KQ1F5B02Q5WYGFEJB5F6BRXS`

---

## 一、基础信息

### 用户与 Ring 数据
```
用户: user-01KQ1F5B02Q5WYGFEJB5F6BRXS (kai)
已有 Ring:
  - 01KQ1JK74P1T9826KXJBZRCCCE (每日论文学习)
  - 01KQ1F8ECE5AC5Y430KXQ11FYY (后端迁移rust)
```

---

## 二、接口测试结果

### 2.1 通过的接口 (22/25)

| 接口 | 方法 | 路径 | 状态 | 响应示例 |
|------|------|------|------|----------|
| health | GET | /api/health | ✅ PASS | `{"status":"ok"}` |
| setup/status | GET | /api/setup/status | ✅ PASS | `{"is_setup":true}` |
| rings | GET | /api/rings | ✅ PASS | 返回 2 个 ring |
| rings | POST | /api/rings | ✅ PASS | 创建成功 |
| config/llm | GET | /api/config/llm | ✅ PASS | qwen3.5-plus |
| config/llm/test | POST | /api/config/llm/test | ✅ PASS | |
| skills | GET | /api/skills | ✅ PASS | 5 个内置 skill |
| notifications | GET | /api/notifications | ✅ PASS | `[]` |
| self/metrics | GET | /api/self/metrics | ✅ PASS | 完整 metrics |
| self/memory | GET | /api/self/memory | ✅ PASS | 3 个 memory entries |
| sessions | GET | /api/rings/{id}/sessions | ✅ PASS | `{"sessions":[]}` |
| sessions | POST | /api/rings/{id}/sessions | ✅ PASS | 创建成功 |
| mode | GET | /api/rings/{id}/mode | ✅ PASS | interaction_mode=normal |
| group-docs | GET | /api/rings/{id}/group-docs/role | ✅ PASS | 返回空 content |
| blueprint | GET | /api/rings/{id}/blueprint | ✅ PASS | status=pending |
| invite-tokens | GET | /api/rings/{id}/invite-tokens | ✅ PASS | `[]` |
| archives | GET | /api/rings/{id}/archives | ✅ PASS | `[]` |
| archive-queue | GET | /api/rings/{id}/archive-queue | ✅ PASS | `[]` |
| super/chat | POST | /api/super/chat | ✅ PASS | SSE 流式 |
| self/chat | POST | /api/self/chat | ✅ PASS | SSE 流式 |
| cross-ring/query | POST | /api/super/cross-ring/query | ✅ PASS | SSE 流式 |
| session/messages | GET | /api/rings/{id}/sessions/{sid}/messages | ✅ PASS | |
| session/material-prep | GET | /api/rings/{id}/sessions/{sid}/material-prep | ✅ PASS | |
| export/chat | GET | /api/rings/{id}/export/chat | ✅ PASS | Markdown 格式 |

### 2.2 失败的接口 (3/25)

#### 问题 1: Session Close 返回 Forbidden

**请求**:
```bash
curl -X POST http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/sessions/01KQ1K7MWTGGVZ749D9XT7RNJ1/close \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS"
```

**响应**:
```json
{"error":{"code":"forbidden","message":"only owner can close session"}}
```

**DB 验证**:
```sql
-- session 数据
SELECT id, owner, phase FROM sessions WHERE id = '01KQ1K7MWTGGVZ749D9XT7RNJ1';
-- 结果: 01KQ1K7MWTGGVZ749D9XT7RNJ1|user-01KQ1F5B02Q5WYGFEJB5F6BRXS|discussion

-- participant 数据
SELECT session_id, token_id, role FROM session_participants WHERE session_id = '01KQ1K7MWTGGVZ749D9XT7RNJ1';
-- 结果: 01KQ1K7MWTGGVZ749D9XT7RNJ1|user-01KQ1F5B02Q5WYGFEJB5F6BRXS|owner

-- is_owner 查询验证
SELECT COUNT(*) FROM session_participants WHERE session_id = '01KQ1K7MWTGGVZ749D9XT7RNJ1' AND token_id = 'user-01KQ1F5B02Q5WYGFEJB5F6BRXS' AND role = 'owner';
-- 结果: 1 (应该返回 true)
```

**根因分析**:
- 代码链路: `routes/session.rs:70` → `services/session.rs:175`
- `is_owner` 函数查询正确，但返回 false
- 推测: 并发请求或 DB 连接问题导致查询结果不一致

**影响**: 用户无法关闭自己创建的 session，Session 功能不完整

---

#### 问题 2: Archive API 缺少必需字段

**请求**:
```bash
curl -X POST http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/archive \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS" \
  -H "Content-Type: application/json" \
  -d '{"content":"test content","message_ids":[]}'
```

**响应**:
```
Failed to deserialize the JSON body into the target type: missing field `suggested_title` at line 1 column 43
```

**模型定义** (`models/archive.rs:22-28`):
```rust
pub struct CreateArchiveInput {
    pub session_id: Option<String>,
    pub content: String,
    pub suggested_title: String,  // 必需字段，无默认值
    pub node_suggestion: NodeSuggestionInput,
}
```

**文档缺失**: `docs/technical/api-design.md` 第 547 行记录的 archive API 缺少 `suggested_title` 和 `node_suggestion` 字段

**影响**: Archive 功能完全不可用，前端触发归档请求必定失败

---

#### 问题 3: Ring Chat SSE 超时

**请求**:
```bash
curl -X POST http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/chat \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS" \
  -H "Content-Type: application/json" \
  -d '{"content":"test","node_refs":[],"tag_refs":[]}'
```

**现象**: 超时（curl 5 秒 timeout）

**对比**:
- `super/chat` → 正常流式返回
- `self/chat` → 正常流式返回
- `cross-ring/query` → 正常流式返回

**代码分析** (`routes/chat.rs:48-284`):

Ring chat 额外流程:
1. `try_handle_graph_command` - LLM 调用检测 graph command
2. `detect_archive_intent` - 检测归档意图
3. `llm.chat_stream_with_tools` - 带 tools 的 LLM 调用

**推测根因**:
- `try_handle_graph_command` 可能 hang 住
- `chat_stream_with_tools` 中 tool 执行卡住
- LLM API 超时但没有正确返回错误

**影响**: 群组聊天核心功能不可用

---

## 三、问题优先级

| 优先级 | 问题 | 影响范围 | 建议 |
|--------|------|----------|------|
| P0 | Archive 字段缺失 | Archive 功能完全不可用 | 修复 API 或添加默认值 |
| P0 | Ring chat 超时 | 核心聊天功能不可用 | Debug LLM 调用链路 |
| P1 | Session close | Session 生命周期管理 | 添加日志定位 is_owner 查询 |

---

## 四、API 文档问题

以下接口的文档与实际实现不一致:

1. **Archive API** (`docs/technical/api-design.md:547`)
   - 缺少 `suggested_title` (必需)
   - 缺少 `node_suggestion` (必需)
   - 缺少 `message_ids` 字段说明

2. **Session Close** (`docs/technical/api-design.md:910`)
   - 文档正常，实际返回 forbidden（代码 bug）

---

## 五、后续测试建议

1. 修复 3 个问题后重新测试
2. 测试 WebSocket 连接的稳定性
3. 测试 Session 完整生命周期（创建 → 材料准备 → 讨论 → 总结 → 关闭）
4. 测试多用户并发场景