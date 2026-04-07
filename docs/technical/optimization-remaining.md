# Optimization Pass — 剩余项

Phase 1-6 完成后，优化 pass 已完成 15/23 项，剩余 8 项待后续实施。

## 已完成 (15/23)

| # | 优化项 | 优先级 |
|---|------|--------|
| O1 | Auth middleware，替换所有硬编码 user-1 | Critical |
| O2 | list_rings SQL LEFT JOIN members 查询 | Critical |
| O3 | CredentialService PBKDF2 + 随机 nonce | Critical |
| O4 | update_settings 白名单 | Important |
| O5 | archive N+1 查询优化 | Important |
| O7 | list_graphs 返回真实数据 | Important |
| O8 | 删除 get_pr_diff stub | Important |
| O9 | 添加 CORS 配置 | Important |
| O10 | get_diff 按文件分组统计 | Important |
| O11 | preview_blueprint 纯内存预览 | Important |
| O12 | 提取 SSE helper (spawn_sse_stream) | Important |
| O13 | 5 处 unwrap() 改为 safe handling | Important |
| O14 | convert_messages 处理 tool role | Important |
| O15 | GraphStore trait + dyn GraphStore | Important |
| O17 | 消息排序改为 ASC | Minor |

## 待实施 (8/23)

| # | 优化项 | 优先级 | 位置 | 修复方案 |
|---|------|--------|------|---------|
| O6 | 从 DB 读 LLM 配置构建真实 provider | Important | `main.rs:40` | 在 main.rs 添加 `build_llm_provider` 函数，从 DB settings 读 `llm_config` JSON，根据 provider 字段构建 `OpenAiProvider`(openai/ollama) 或 `AnthropicProvider`。无配置时 fallback 到 MockLlmProvider |
| O16 | Jieba 缓存到 SqliteRepository | Minor | `db/sqlite.rs` search_service | 给 SqliteRepository 添加 `Jieba` 字段（`Option<jieba::Jieba>`），在 `new()` 时 lazy init，搜索时复用实例 |
| O18 | 前端 404 路由 | Minor | `ring-frontend/src/App.tsx` | 在 Routes 中添加 `<Route path="*" element={<Navigate to="/" />} />` |
| O19 | send_message stream ID 冲突 | Minor | `ring-frontend/src/stores/chatStore.ts` | 将 `stream-${index}` 改为 `stream-${Date.now()}` |
| O20 | db/sqlite.rs 拆分为多个 repo 文件 | Minor | `db/sqlite.rs` (1700+ 行) | 拆分为 `user_repo.rs`, `ring_repo.rs`, `session_repo.rs`, `conversation_repo.rs`, `graph_repo.rs`, `settings_repo.rs` 等，通过 `mod.rs` re-export |
| O21 | 前端全局错误提示 toast | Minor | `ring-frontend/src/` | 创建 Toast 组件，在 API client 的 catch 中统一弹出错误提示 |
| O22 | SetupGuard spinner 样式 | Minor | `ring-frontend/src/pages/Setup/` | 给 loading 状态加 spinner 动画 CSS |
| O23 | InviteToken role 从 DB 读取 | Minor | `handlers/member.rs` | 从 invite_tokens 表读取 role 字段，而非硬编码 "member" |
