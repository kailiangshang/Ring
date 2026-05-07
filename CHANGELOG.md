# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-05-07

### 核心功能

- **四层 AI 架构** — Super Ring（全局）/ Group Ring（群组）/ Session Ring（讨论）/ Self（私有）
- **知识图谱系统** — D3.js 可视化、多图谱支持、节点树列表视图、蓝图构建器
- **Git 协作归档** — 自动 commit、PR Review、Git revert、diff 视图
- **多人实时讨论** — WebSocket 聊天、材料准备、AI 总结
- **数据同步** — HTTP bundle 同步、creator-wins 策略、自动同步

### AI 能力

- **文件解析** — PDF/TXT/MD/CSV/代码上传，结构化提取
- **知识提取** — 自动推荐图谱节点
- **网页爬取** — `fetch_url` tool，HTML 清洗 + 15K 截断
- **跨 Ring 搜索** — SQLite FTS5 全文索引

### 导出格式

- Markdown（聊天/节点/Session/Self/Super）
- PDF（聊天导出）
- JSON（图谱/备份）
- tar.gz（整库备份）
- SVG / PNG（图谱可视化）

### 安全加固（Release Hardening）

#### 认证与授权
- `skills.rs` — 仅 setup creator 可安装/删除 skills（之前任何用户都可以）
- `super_chat.rs` — 仅 setup creator 可修改 system prompt 和 preferences
- `config.rs` — `test_llm_config` 和 `test_gitlab_config` 添加认证要求
- `setup.rs` — `recover_token` 在 setup 完成后才允许访问

#### 数据隔离
- `self_data.rs` — `get_self_dir()` 从共享目录改为 `~/.ring/self/{user_id}/`，并自动迁移旧数据
- `ring.rs` — `creator_ip` 仅对 creator 返回，其他成员看不到

#### 路径安全
- `export.rs` — `export_node_markdown` 的 `canonicalize()` 失败时返回错误，不回退到原始路径
- `upload.rs` — `sanitize_filename` 清理路径分隔符和 `..` 序列
- `skill.rs` — `validate_skill_url` 阻止内网地址访问（SSRF 防护）
- `workflow.rs` — `is_url_allowed` 阻止 localhost/内网 IP 的 `fetch_url`

#### 加密与密钥
- `encryption.rs` — `magic-crypt` (ECB) 升级为 `AES-GCM`，兼容旧数据
- `encryption.rs` — 密钥文件创建后设置权限 `0o600`（Unix）
- `encryption.rs` — 添加父目录创建和错误日志

#### CORS 与网络
- `mod.rs` — CORS 从 `Any` 限制为明确的 methods 和 headers
- `workflow.rs` — `fetch_url` 添加 30 秒超时
- `ws_hub.rs` — WebSocket 发送失败时记录日志，不再静默丢弃

#### 资源保护
- `rate_limit.rs` — 添加 `max_entries` 限制（10000）和定期清理，防止内存无限增长
- `main.rs` — 所有 `expect()` 替换为结构化错误处理 + 退出码
- `search.rs` — 使用 `QueryBuilder` 替代 `format!`，消除 SQL 注入风险

### 性能优化

- `main.rs` — `std::fs::create_dir_all` 替换为 `tokio::fs::create_dir_all`
- `self_memory.rs` — 所有文件操作通过 `tokio::task::spawn_blocking`
- `group_doc_maintenance.rs` — `persist_group_doc` 改为异步
- `super_chat.rs` — 配置读写改为异步
- `super_chat.rs` — `build_ring_summary` 修复 N+1 查询（单次 JOIN）
- `chat.rs` — `build_system_prompt` 改为异步

### v0.1.0 Pre-release Optimization

**Phase 1 — Security & Stability**
- `sync_import` SSRF protection (reuse `is_url_allowed`)
- Fix non-ASCII string slicing (`chars().take()` instead of byte slice)
- Hide `ErrorBoundary` stack traces in production
- WebSocket rate limiting (10/min)
- Atomic `create_ring`, `increment_use_count`, `conversation_token` (eliminate race conditions)
- Remove manual `map_err` overrides, use `?` with `From<sqlx::Error> for RingError`
- Fix `unwrap_or(false)` bypassing safety checks

**Phase 2 — Architecture & Dedup**
- Extract `sse_event_to_axum()` shared SSE helper (3 duplicates → 1)
- Merge 3 delete_message handlers into shared `delete_message_for_user()`
- Merge 3 export_chat functions into `build_chat_markdown()` + `record_export_usage()`
- Move `remove_member` session check to service layer (handler → service pattern)
- Move ad-hoc `ALTER TABLE` from `main.rs` to migration 018
- Add `codegen-units = 1` to release profile

**Phase 3 — Performance**
- Replace `std::fs` with `tokio::fs` for all async I/O paths
- Consolidate `list_rings_for_user` from 3 queries to 1 (correlated subqueries)
- Batch `delete_messages` with `DELETE WHERE id IN (...)`
- `React.memo` on 5 list components (RingListItem, SessionRow, TreeNodeRow, TabItem, MaterialCard)

**Phase 4 — Frontend Cleanup**
- Extract `MarkdownRenderer` shared component (3 duplicates → 1)
- Extract `LLMConfigForm` + `useLLMTest` shared (3 duplicates → 1)
- Replace 10 `window.confirm` with `ConfirmModal`, 1 `window.prompt` with `PromptModal`
- Remove unused dependencies: `mermaid`, `react-router-dom`, `rehype-highlight`, `@testing-library/user-event`
- Move `@types/d3` to devDependencies

**Phase 5 — Engineering**
- CI pipeline with caching + security audit (`.github/workflows/ci.yml`)
- Remove dead code: `OptionalUser` extractor, `should_recommend_archive` function, orphaned `RingListItem.tsx`
- One-click `build.sh` script

### CLI 功能

- 支持 `--port` / `-p` 参数指定监听端口（默认 7420）
- 支持 `--help` 显示使用说明
- 支持 `--version` 显示版本号

### 前端修复

- 添加全局 `ErrorBoundary` 组件
- 修复所有空 `catch {}` 块，添加错误日志
- 修复 `RingList.tsx` 中 `setState` 在 effect 中的警告

### 测试

- 73 个集成测试全部通过（+2 安全回归测试）
- 新增：路径遍历防护测试、skills 认证测试、super settings 权限测试

### 基础设施

- 单一 17MB 二进制（前后端一体）
- SQLite + 文件系统，零外部依赖
- 19 个数据库迁移
- clippy 完全清洁（0 warnings）
- CI 自动化（fmt + clippy + test + lint + build）
- 73 后端测试 + 75 前端测试全部通过

[0.1.0]: https://github.com/kailiangshang/Ring/releases/tag/v0.1.0