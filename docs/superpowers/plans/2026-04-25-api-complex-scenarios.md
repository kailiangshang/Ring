# Ring API 复杂场景测试计划

**目的**: 基于真实用户情景，设计复杂场景测试
**范围**: 跨接口、跨页面、多步骤的复杂用户流程

---

## 场景 1: 用户在 Super Ring 与 AI 对话，同时切换到 Group Ring 查看图谱

### 场景描述
用户在 Super Ring 发起一个跨 Ring 分析请求，AI 正在处理中（流式响应）。用户不想等待，主动切换到 Group Ring 界面查看图谱和历史消息。

### 测试步骤

**Step 1: 在 Super Ring 发起复杂查询**
```bash
curl -s -X POST http://localhost:7420/api/super/chat \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "对比一下 每日论文学习 和 后端迁移rust 这两个 Ring 里关于架构设计讨论的内容",
    "context_ring_ids": ["01KQ1JK74P1T9826KXJBZRCCCE", "01KQ1F8ECE5AC5Y430KXQ11FYY"]
  }'
```
观察: SSE 开始，返回 message_start 事件

**Step 2: 立即切换到 Group Ring 获取图谱**
```bash
curl -s http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/graph \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS"
```

**Step 3: 获取 Group Ring 聊天历史**
```bash
curl -s "http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/chat/history?limit=10" \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS"
```

**Step 4: 继续 Super Ring 流式响应**
观察之前请求是否完成，返回 message_end 事件

### 预期结果
- Group Ring 请求在 Super Ring 流式响应期间正常返回
- 两边请求互不干扰
- 记录 Super Chat 的完整响应内容

### 验证点
- [ ] Super Ring SSE 能正确处理多 ring context
- [ ] 切换操作不打断正在进行的流式响应
- [ ] 响应内容是否包含两个 ring 的对比分析

---

## 场景 2: 用户创建 Session，邀请成员，开始讨论

### 场景描述
用户创建一个 decision skill 的 Session，邀请另一个成员（如果有），进入材料准备阶段，上传文档，开始讨论。

### 测试步骤

**Step 1: 创建 Session**
```bash
curl -s -X POST http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/sessions \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Rust 异步框架选型讨论",
    "description": "讨论 Tokio vs async-std",
    "skill": "decision",
    "archivable": true,
    "invitees": []
  }'
```
保存返回的 session_id

**Step 2: 检查 Session 状态**
```bash
curl -s http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/sessions/{session_id} \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS"
```

**Step 3: 获取材料准备状态**
```bash
curl -s http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/sessions/{session_id}/material-prep \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS"
```

**Step 4: 开启 Session（进入讨论阶段）**
```bash
curl -s -X POST http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/sessions/{session_id}/start \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS"
```

**Step 5: 在 Session 内发送消息（WebSocket 或 HTTP）**
```bash
# 先通过 HTTP 测试（WebSocket 需要前端配合）
curl -s "http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/sessions/{session_id}/messages" \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS"
```

**Step 6: 尝试总结 Session**
```bash
curl -s -X POST http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/sessions/{session_id}/summarize \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS"
```

**Step 7: 关闭 Session（复现问题 1）**
```bash
curl -s -X POST http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/sessions/{session_id}/close \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS"
```

### 验证点
- [ ] Session 状态机转换正确 (material_prep → discussion → summary → closed)
- [ ] Phase 转换时更新 session_participants 中的状态
- [ ] summarize 触发 LLM 调用
- [ ] **close 操作是否仍返回 forbidden**

---

## 场景 3: 用户在 Self 和 Group Ring 之间切换，验证记忆隔离

### 场景描述
用户先和 Self 对话（私人），再和 Group Ring 对话（群组），验证 Self 的记忆不会泄露到 Group Ring。

### 测试步骤

**Step 1: 与 Self 对话，谈论私有信息**
```bash
curl -s -X POST http://localhost:7420/api/self/chat \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS" \
  -H "Content-Type: application/json" \
  -d '{"content": "我的密码是 test-password-123，帮我记住"}' \
  --max-time 5
```

**Step 2: 查看 Self Memory 是否更新**
```bash
curl -s http://localhost:7420/api/self/memory \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS"
```

**Step 3: 与 Group Ring 对话，不应该知道私有信息**
```bash
curl -s -X POST http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/chat \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS" \
  -H "Content-Type: application/json" \
  -d '{"content": "我刚才告诉 Self 什么秘密？"}' \
  --max-time 5
```

**Step 4: 检查 Group Ring 响应**
观察 AI 是否知道私密信息（不应该知道）

### 验证点
- [ ] Self 对话正常保存
- [ ] Self memory 更新
- [ ] Group Ring 无法访问 Self 的私有上下文
- [ ] 隐私过滤器生效

---

## 场景 4: 用户触发归档流程，然后取消

### 场景描述
用户在 Group Ring 触发归档，AI 返回归档建议，用户拒绝或修改。

### 测试步骤

**Step 1: 触发归档（复现问题 2）**
```bash
curl -s -X POST http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/archive \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "讨论内容：Rust 的所有权系统非常强大...",
    "message_ids": ["msg1", "msg2"],
    "suggested_title": "Rust 所有权系统讨论",
    "node_suggestion": {
      "action": "create_new",
      "parent_id": null,
      "node_title": "Rust 所有权系统"
    }
  }'
```

**Step 2: 列出已有 archive 记录**
```bash
curl -s http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/archives \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS"
```

**Step 3: 查看 archive queue**
```bash
curl -s http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/archive-queue \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS"
```

### 验证点
- [ ] 带完整字段的 archive 请求能成功
- [ ] 归档建议返回正确的结构
- [ ] 用户可以拒绝或修改归档建议

---

## 场景 5: 多 Ring 切换，验证状态隔离

### 场景描述
用户依次访问多个 Ring，每个 Ring 有不同的配置（mode, skill_permission），验证切换时状态正确。

### 测试步骤

**Step 1: 获取所有 Ring**
```bash
curl -s http://localhost:7420/api/rings \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS"
```

**Step 2: 依次获取每个 Ring 的 mode**
```bash
# Ring 1
curl -s http://localhost:7420/api/rings/01KQ1JK74P1T9826KXJBZRCCCE/mode \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS"

# Ring 2
curl -s http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/mode \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS"
```

**Step 3: 依次获取每个 Ring 的 blueprint**
```bash
curl -s http://localhost:7420/api/rings/01KQ1JK74P1T9826KXJBZRCCCE/blueprint \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS"
```

**Step 4: 在不同 Ring 发聊天，验证上下文隔离**
```bash
# Ring 1 聊天
curl -s -X POST http://localhost:7420/api/rings/01KQ1JK74P1T9826KXJBZRCCCE/chat \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS" \
  -H "Content-Type: application/json" \
  -d '{"content": "这个 Ring 叫什么名字？"}' --max-time 5

# Ring 2 聊天
curl -s -X POST http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/chat \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS" \
  -H "Content-Type: application/json" \
  -d '{"content": "这个 Ring 叫什么名字？"}' --max-time 5
```

### 验证点
- [ ] 每个 Ring 的 mode 配置独立
- [ ] 每个 Ring 的 blueprint 独立
- [ ] AI 能正确识别当前 Ring 的名称
- [ ] 聊天上下文不串台

---

## 场景 6: 验证 Cross Ring Cache 失效

### 场景描述
在 Super Ring 发起跨 Ring 分析后，修改某个 Ring 的数据（新增节点），然后再次发起跨 Ring 查询，验证缓存失效。

### 测试步骤

**Step 1: 获取 Ring A 的 graph**
```bash
curl -s http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/graph \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS"
```

**Step 2: 发起跨 Ring 查询（会触发缓存）**
```bash
curl -s -X POST http://localhost:7420/api/super/cross-ring/query \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS" \
  -H "Content-Type: application/json" \
  -d '{"query": "架构设计", "ring_ids": ["01KQ1F8ECE5AC5Y430KXQ11FYY"]}'
```

**Step 3: 通过其他接口修改 Ring 数据**
（需要先找到修改 graph 的接口，可能是创建 node）
```bash
# 创建节点
curl -s -X POST http://localhost:7420/api/rings/01KQ1F8ECE5AC5Y430KXQ11FYY/graph \
  -H "X-Ring-Token: user-01KQ1F5B02Q5WYGFEJB5F6BRXS" \
  -H "Content-Type: application/json" \
  -d '{"label":"新节点","node_type":"leaf","content":"# 新内容"}'
```

**Step 4: 再次发起跨 Ring 查询**
观察返回结果是否包含新节点（验证缓存是否正确失效）

### 验证点
- [ ] Cross Ring Cache 在数据变更时正确失效
- [ ] 后续查询能获取到最新数据

---

## 场景 7: WebSocket 断连重连

### 场景描述
用户通过 WebSocket 连接，模拟断连（关闭连接），然后重连，验证消息补发机制。

### 测试步骤

**Step 1: 建立 WebSocket 连接**
```bash
# 使用 websocat 或 curl --include --no-buffer 测试
curl --include --no-buffer \
  -H "Upgrade: websocket" \
  -H "Connection: Upgrade" \
  -H "Sec-WebSocket-Key: test" \
  -H "Sec-WebSocket-Version: 13" \
  http://localhost:7420/api/ws?token=user-01KQ1F5B02Q5WYGFEJB5F6BRXS
```

**Step 2: 检查连接状态**
WebSocket 连接后服务端会返回 101 Switching Protocols

**Step 3: 模拟断开（关闭连接）**
断开后检查是否有自动清理机制

**Step 4: 重新连接**
新连接应能正常通信

### 验证点
- [ ] WebSocket 握手成功
- [ ] 断开后服务端能正确处理
- [ ] 重连机制正常

---

## 执行建议

1. **按顺序执行**: 每个场景独立测试，记录每步的实际响应
2. **记录异常**: 任何非预期响应都记录，包括完整响应体
3. **关注时序**: SSE/流式响应特别注意事件顺序和完整性
4. **复用数据**: 场景 1-5 可以共用同一个 Ring/Session 数据

---

## 测试数据准备

```bash
TOKEN="user-01KQ1F5B02Q5WYGFEJB5F6BRXS"
RING1="01KQ1JK74P1T9826KXJBZRCCCE"  # 每日论文学习
RING2="01KQ1F8ECE5AC5Y430KXQ11FYY"  # 后端迁移rust

# 启动服务后先执行场景 1 获取基础数据
```