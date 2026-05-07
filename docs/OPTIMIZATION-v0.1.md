# Ring v0.1.0 发布前优化清单

> 全量代码扫描报告，基于 82 个 Rust 源文件、86 个 TS/TSX 文件、17 个数据库迁移的深度分析。
> 每个条目标注优先级：P0（阻塞发布）、P1（应修复）、P2（建议修复）、P3（后续迭代）。

---

## 一、安全（P0）

### S1. SSRF 漏洞 — sync_import 允许任意 IP

- **位置**: `server/src/routes/archive.rs` sync_import handler
- **问题**: 用户可指定任意 `creator_ip`，服务端直接 `reqwest::get` 请求该地址，可探测内网服务
- **修复**: 复用已有的 `is_url_allowed`（`workflow.rs`）验证 IP，或在 service 层校验 IP 合法性

### S2. 非 ASCII 字符串切片 panic

- **位置**: `server/src/routes/export.rs:308-309`, `server/src/routes/chat.rs:308`
- **问题**: `&m.content[..100]` 按字节切片，中文等多字节字符会运行时 panic
- **状态**: STATUS.md 记录已修复，需确认所有实例均已替换为 `chars().take(n)`

### S3. ErrorBoundary 生产环境暴露堆栈

- **位置**: `ui/src/components/common/ErrorBoundary.tsx`
- **问题**: 生产构建中仍显示完整错误堆栈信息
- **修复**: 仅在 `import.meta.env.DEV` 时显示堆栈，生产环境显示友好错误信息

### S4. WebSocket 连接无速率限制

- **位置**: `server/src/routes/ws.rs`
- **问题**: WebSocket 连接建立无速率限制，10 秒 auth timeout 期间可占用资源
- **修复**: 在 ws route 上叠加 rate limiting middleware

---

## 二、架构合规（P0）

### A1. Handler 包含业务逻辑 — 违反项目约定

AGENTS.md 明确要求 "handlers 不写业务逻辑"，以下 handler 需要重构：

| Handler | 位置 | 问题描述 |
|---------|------|----------|
| `quick_archive_handler` | `routes/archive.rs:40-172` | ~130 行纯业务逻辑，应移至 `services/archive_service.rs` |
| `ring_chat` | `routes/chat.rs:66-289` | 包含 archive intent 检测、privacy filtering、DB writes |
| `self_chat` | `routes/chat.rs:376-446` | 同上 |
| `transfer_ownership` | `routes/session.rs:132-184` | 直接 SQL 查询，应移至 `services/session.rs` |
| `remove_member` | `routes/members.rs:40-71` | 包含 session ownership 检查逻辑 |
| `test_gitlab_config` | `routes/config.rs:57-85` | HTTP client 调用 GitLab API 属于业务逻辑 |

**修复策略**: handler 只做参数解析 → 调 service → 返回响应，业务逻辑一律下沉到 service 层。

### A2. Service 层粒度不一致

| 文件 | 行数 | 问题 |
|------|------|------|
| `services/auth.rs` | 13 | 过薄，可合并 |
| `services/super_chat.rs` | 1176 | 过厚，应拆分为 `super_chat.rs` + `super_tools.rs` |
| `services/session.rs` | 708 | 偏大，可拆分 material_prep 部分 |

---

## 三、并发安全（P1）

### C1. create_ring 竞态条件

- **位置**: `models/ring.rs:75-89`
- **问题**: SELECT 检查重名 + INSERT 非原子，并发请求可能绕过重名检查
- **修复**: 使用 `INSERT ... ON CONFLICT DO NOTHING` + 检查 `rows_affected()`

### C2. conversation_token 更新竞态

- **位置**: `models/conversation_token.rs:50-70`
- **问题**: get_or_create + UPDATE 分两步执行，并发下数据不一致
- **修复**: `INSERT ... ON CONFLICT (user_id, ring_id) DO UPDATE SET input_tokens = input_tokens + ?`

### C3. invite_tokens 计数竞态

- **位置**: `models/invite.rs:129-135`
- **问题**: use_count 检查和递增分离，并发下可超出 max_uses
- **修复**: `UPDATE invite_tokens SET use_count = use_count + 1 WHERE id = ? AND use_count < max_uses`，检查 `rows_affected()`

---

## 四、性能（P1）

### P1. 同步文件 IO 阻塞 tokio 运行时

以下位置使用 `std::fs` 而非 `tokio::fs`，会阻塞异步运行时：

| 文件 | 行号 | 操作 |
|------|------|------|
| `services/graph.rs` | 27, 84, 90 | read_to_string, create_dir_all, write |
| `services/self_data.rs` | 15, 24, 59 | read_to_string, write |
| `services/archive_service.rs` | 122, 213-234 | write, create_dir_all |
| `services/skill.rs` | 多处 | read_to_string |
| `services/git_service.rs` | 多处 | 文件系统操作 |

**修复**: 替换为 `tokio::fs::` 或用 `tokio::task::spawn_blocking` 包裹。
**注**: STATUS.md 记录部分已修复，需确认全量覆盖。

### P2. N+1 查询 — list_rings_for_user

- **位置**: `models/ring.rs:118-190`
- **问题**: 对每个 Ring 单独查询 node_count 和 creator_ip
- **修复**: 合并为 JOIN 查询或子查询

### P3. 消息批量删除低效

- **位置**: `models/message.rs:115-119`
- **问题**: 循环执行单条 DELETE
- **修复**: `DELETE FROM messages WHERE id IN (...)`

### P4. Release profile 缺少 codegen-units

- **位置**: `server/Cargo.toml` `[profile.release]`
- **修复**: 添加 `codegen-units = 1`（5-15% 运行时性能提升）

### P5. 前端全量 d3 导入

- **位置**: `ui/package.json`
- **问题**: 仅用 d3-force + d3-zoom，却引入完整 d3 包（~500KB）
- **修复**: 替换为 `d3-force` + `d3-zoom` + `d3-selection` 等子包

### P6. 前端无 React.memo 优化

- **位置**: 列表类组件（`RingListItem`, `MessageItem`, `NodeTreeList` 等）
- **问题**: 父组件状态变化导致所有列表项重渲染
- **修复**: 对纯展示列表项组件添加 `React.memo`

### P7. 前端无消息虚拟化

- **位置**: `ui/src/components/chat/MessageList.tsx`
- **问题**: 长 chat 历史 DOM 节点爆炸
- **修复**: 引入虚拟滚动（如 `@tanstack/react-virtual`），v0.1.0 可选，后续版本优化

---

## 五、错误处理（P1）

### E1. 手动 map_err 覆盖 From 实现

- **位置**: `models/message.rs`, `models/graph.rs`, `models/archive.rs`, `routes/members.rs` 等多处
- **问题**: `.map_err(|e| RingError::Internal(e.to_string()))` 覆盖了 `From<sqlx::Error> for RingError` 的自动转换，导致 DB "not found" 返回 500 而非 404
- **修复**: 去除手动 map_err，直接用 `?` 操作符

### E2. 后端大量静默错误吞没

| 位置 | 模式 |
|------|------|
| `services/graph.rs:140-150` | `let _ = ...` 吞掉搜索索引更新错误 |
| `services/session.rs:96-106` | 搜索索引错误静默忽略 |
| `services/super_chat.rs:823-851` | `save_super_message` 用 `let _ =` 吞错误 |
| `routes/chat.rs:179-183` | `record_chat_message` 错误仅 log |
| `main.rs:89` | ad-hoc ALTER TABLE 错误静默忽略 |

**修复策略**: 区分关键路径和非关键路径——关键路径错误应传播，非关键路径至少 `tracing::warn!` 并考虑重试。

### E3. 前端空 catch 块

- **位置**: 多个组件中 `catch {}` 完全静默
- **注**: STATUS.md 记录已修复，需确认全量覆盖

### E4. unwrap_or(false) 绕过安全检查

- **位置**: `models/ring.rs:82`
- **问题**: DB 查询失败时重复 Ring 检查被静默跳过，允许创建重名 Ring
- **修复**: 查询失败应返回错误而非默认值

---

## 六、代码重复（P1）

### D1. SSE 流转发逻辑 — 5 处重复

`SseEvent::Start/Delta/End/Error` 匹配逻辑几乎相同，出现在：
- `routes/chat.rs:185-289` (ring_chat)
- `routes/chat.rs:376-446` (self_chat)
- `routes/super_chat.rs:79-104` (super_chat)
- `routes/super_chat.rs:196-223` (cross_ring_query)
- `routes/super_chat.rs:238-264` (cross_ring_analysis)

**修复**: 提取为 `server/src/utils/sse.rs` 公共函数

### D2. 消息删除 handler — 3 处重复

- `routes/chat.rs:470-486` (delete_ring_message)
- `routes/chat.rs:488-503` (delete_self_message)
- `routes/chat.ts:505-520` (delete_super_message)

仅 Path 提取模式不同，逻辑完全一致。

### D3. Export Chat 函数 — 3 处重复

- `routes/export.rs:205-233` (export_ring_chat)
- `routes/export.rs:235-259` (export_self_chat)
- `routes/export.rs:261-285` (export_super_chat)

仅 title 和 ring_id 处理不同。

### D4. Tool usage 记录样板代码 — 20+ 处

```rust
let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
if let Err(e) = crate::services::self_data::record_tool_usage(&self_dir, "...") {
    tracing::warn!("failed to record tool usage: {e}");
}
```

**修复**: 提取为 `AppState` 上的方法或 axum middleware

### D5. 前端 Markdown 渲染组件 — 3 处重复

`ChatArea`, `MessageItem`, `SelfChat` 各自内联 `react-markdown` 配置。

**修复**: 提取为 `<MarkdownRenderer content={...} />` 共享组件

### D6. 前端 LLM 配置表单 — 3 处重复

`StepLLM`, `ConfigPanel`, `SuperSettingsPanel` 各自重复 LLM provider 选择 + config 表单。

**修复**: 提取为 `<LLMConfigForm value={...} onChange={...} />` 共享组件

### D7. 前端样式常量 — 8+ 处重复

`inputStyle`, `smallBtn`, `sectionHeader` 等样式常量在多文件中重复定义。

**修复**: 提取到 `ui/src/styles/common.ts` 或 CSS module

---

## 七、类型安全（P2）

### T1. 字符串代替枚举

以下字段均为 `String`，应使用 Rust enum + TypeScript union type：

| 字段 | 当前类型 | 应改为 |
|------|----------|--------|
| `role` (members) | `String` | `enum Role { Creator, Admin, Member, Readonly }` |
| `phase` (sessions) | `String` | `enum Phase { MaterialPrep, Discussion, Summary, Closed }` |
| `status` (archives) | `String` | `enum ArchiveStatus { Pending, Pushed, ... }` |
| `interaction_mode` | `String` | `enum InteractionMode { Manual, Auto }` |
| `storage_mode` | `String` | `enum StorageMode { Local, Gitlab }` |

**收益**: 编译器捕获拼写错误，`match` 穷尽检查，IDE 自动补全。

### T2. Super Ring 魔术字符串

- Super Ring 用 `ring_id = Some("super")`，Self 用 `ring_id = None`
- 建议统一为枚举或使用 newtype pattern

### T3. JSON 字段手动解析

- `MessageRow.node_refs` 和 `tags` 存为 JSON 字符串，使用处需手动 `serde_json::from_str`
- 应在 model 层自动反序列化为 `Vec<String>`

---

## 八、CI/CD（P2）

### CI1. 缺少 PR 持续集成流水线

当前仅有 release workflow（tag 触发），无 PR 检查。

**建议新增** `.github/workflows/ci.yml`：

```yaml
on: [push, pull_request]
jobs:
  backend:
    - cargo fmt --check
    - cargo clippy -- -D warnings
    - cargo test
  frontend:
    - npm ci
    - npm run lint
    - npm test
    - npm run build
```

### CI2. 无依赖缓存

Release workflow 无 Cargo/npm 缓存，每次全量构建。

**修复**: 添加 `actions/cache` for `~/.cargo`, `server/target`, `~/.npm`

### CI3. 无安全审计

**修复**: 添加 `cargo audit` 和 `npm audit` 步骤

### CI4. 无 rust-toolchain.toml

开发者本地 Rust 版本可能不一致。

**修复**: 添加 `rust-toolchain.toml` 指定最小 Rust 版本

---

## 九、测试覆盖（P2）

### 当前覆盖情况

| 层级 | 已测试 | 未测试 |
|------|--------|--------|
| Rust Services (32) | 4 | **28** |
| Rust Models (12) | 0 (间接) | **12** |
| Rust Middleware | 0 | 1 |
| Rust WebSocket | 0 | 1 |
| React Components (50+) | 0 | **全部** |
| Zustand Stores (19) | 3 | **16** |
| Frontend Services | 0 | sse, ws-client, metrics |

### v0.1.0 最低测试要求

**后端关键路径补充单元测试**:
- `services/archive_service.rs` — 归档是核心功能
- `services/session.rs` — Session 生命周期
- `services/llm.rs` — LLM 调用（mock）
- `ws_hub.rs` — WebSocket 连接管理
- `middleware/rate_limit.rs` — 速率限制逻辑

**前端核心 store 补充测试**:
- `chat-store.ts` — 聊天状态管理
- `ring-store.ts` — Ring 列表管理
- `session-store.ts` — Session 状态
- `auth-store.ts` — 认证状态

---

## 十、数据库（P2）

### DB1. main.rs 中 ad-hoc ALTER TABLE

- **位置**: `server/src/main.rs:89-91`
- **问题**: 运行时执行 `ALTER TABLE users ADD COLUMN token_created_at`，绕过迁移系统
- **修复**: 创建 migration 018，移除 main.rs 中的 ad-hoc SQL
- **同步**: 移除 `tests/integration.rs` 中的 `ensure_token_created_at` helper

### DB2. invite_tokens / join_requests 缺少 ON DELETE CASCADE

- **问题**: 删除 Ring 后留下孤儿邀请和加入请求记录
- **修复**: 创建 migration 019 添加外键级联删除

### DB3. list_rings_for_user 动态 SQL

- **位置**: `models/ring.rs:139-148`
- **问题**: 用 `format!` 构建 IN 子句
- **修复**: 使用 sqlx `QueryBuilder`（已在 `search.rs` 中使用）

### DB4. FTS5 中文分词局限

- **问题**: `unicode61` tokenizer 不支持 CJK 分词，应用层 `normalize_cjk` 补偿但 migration 013 回填数据未做标准化
- **修复**: v0.1.0 可不改，记录为已知限制

---

## 十一、前端架构（P2）

### FE1. chat-store 强耦合

- **位置**: `ui/src/stores/chat-store.ts`
- **问题**: 直接导入 10 个其他 store，形成强耦合网络
- **修复**: 通过事件/action 解耦，或合并相关 store

### FE2. window.confirm / window.prompt 替换

- **位置**: 11+ 处使用浏览器原生对话框
- **问题**: 破坏深色主题一致性，且无法自定义样式
- **修复**: 已有 `ConfirmModal` / `PromptModal` 组件，统一替换

### FE3. 未使用的依赖

| 包 | 问题 |
|----|------|
| `mermaid` | 安装但未使用，增加打包体积 |
| `react-router-dom` | 安装但未使用（使用手动路由） |
| `@testing-library/user-event` | 安装但未使用 |
| `immer` | 检查是否实际使用（zustand 5 可能不需要） |

### FE4. @types/d3 位置错误

- **问题**: 放在 `dependencies` 而非 `devDependencies`
- **修复**: 移至 devDependencies

---

## 十二、死代码清理（P3）

| 位置 | 内容 |
|------|------|
| `extractors/auth.rs:60-81` | `OptionalUser` 已定义但未使用 |
| `services/chat.rs:26-45` | `should_recommend_archive` 函数从未被调用 |
| `services/archive_service.rs:456-490` | `ArchiveStep::Generating/Pushing/Committing` 枚举变体未使用 |
| `ui/src/components/sidebar/RingListItem.tsx` | 从未被导入 |

---

## 优化执行计划

### 第一阶段：安全 + 稳定性（阻塞发布）

- [ ] S1: sync_import SSRF 防护
- [ ] S2: 确认所有字符串切片已修复
- [ ] S3: ErrorBoundary 生产环境隐藏堆栈
- [ ] S4: WebSocket 速率限制
- [ ] C1-C3: 并发竞态修复
- [ ] E1: 去除手动 map_err，使用 ? 操作符
- [ ] E4: unwrap_or(false) 安全检查修复

### 第二阶段：架构合规 + 去重

- [ ] A1: Handler 业务逻辑下沉到 service 层
- [ ] D1: 提取 SSE 流转发公共函数
- [ ] D2: 合并消息删除 handler
- [ ] D3: 合并 export chat 函数
- [ ] D4: 提取 tool usage 记录 helper
- [ ] DB1: ad-hoc ALTER TABLE 改为正式 migration

### 第三阶段：性能

- [ ] P1: 确认全量 std::fs → tokio::fs 替换
- [ ] P2: N+1 查询修复
- [ ] P3: 批量删除优化
- [ ] P4: Release profile codegen-units
- [ ] P5: d3 子包替换
- [ ] P6: React.memo 优化关键列表

### 第四阶段：前端整理

- [ ] D5: 提取 MarkdownRenderer 共享组件
- [ ] D6: 提取 LLMConfigForm 共享组件
- [ ] D7: 提取共享样式常量
- [ ] FE2: window.confirm → ConfirmModal 替换
- [ ] FE3: 移除未使用依赖
- [ ] FE4: @types/d3 移至 devDependencies

### 第五阶段：工程化

- [ ] CI1: 新增 PR 持续集成流水线
- [ ] CI2: CI 依赖缓存
- [ ] CI3: 安全审计步骤
- [ ] CI4: 添加 rust-toolchain.toml
- [ ] T1: 关键字段枚举化（role, phase, status）
- [ ] 补充关键路径单元测试
- [ ] 死代码清理

---

## 附录：已有积极实践

- 参数化 SQL 查询全覆盖，无 SQL 注入风险
- AES-256-GCM 加密 + 随机 nonce + 密钥文件权限控制
- 隐私过滤器实现完整（手机号/身份证/邮箱/银行卡脱敏）
- Rate limiting 有条目上限 + 定期清理
- Graceful shutdown 信号处理（Unix + Windows）
- WebSocket auth 10 秒超时防护
- SSE 流式输出 + AbortController 支持
- 消息折叠 + 自动滚动
- 错误边界 + 操作反馈

---

*文档生成时间: 2026-05-05*
*对应代码版本: v0.1.0 (pre-release)*
