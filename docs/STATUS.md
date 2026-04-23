# Ring 项目状态

> 最后更新：2026-04-23

## 开发概况

- 后端 64 个 Rust 源文件，~11,700 行
- 前端 84 个 TS/TSX 文件，~9,100 行
- 12 个数据库迁移
- 56/56 集成测试通过
- 313 个 Git commits

## 功能完成状态

### 基础设施

| 功能 | 状态 |
|---|---|
| 三栏布局 (Sidebar + Chat + PanelStack) | done |
| IceChat 深色主题 (Cascadia Code + Space Grotesk) | done |
| Handler → Service → Model 三层分离 | done |
| Auth (X-Ring-Token) + Token 恢复 | done |
| Setup 向导 (5 步) + Skip GitLab | done |
| Setup Done 命令速查表 | done |
| API Key / Git 凭证 AES-256 加密 | done |

### 聊天系统

| 功能 | 状态 |
|---|---|
| Group Ring / Super Ring / Self 三层聊天 | done |
| SSE 流式输出 + tool_calls 支持 | done |
| 聊天历史分页加载 | done |
| Markdown 渲染 (react-markdown + remark-gfm) | done |
| 命令补全弹出框 (/ @ ! % 前缀触发) | done |
| 命令历史 (上下箭头) | done |
| Token 用量显示 (每条消息) | done |
| Auto compact (上下文超限自动压缩) | done |
| Ephemeral 模式 (临时消息不保存) | done |
| 隐私过滤 (手机号/身份证/邮箱/银行卡脱敏) | done |

### 图谱系统

| 功能 | 状态 |
|---|---|
| D3.js 力导向图可视化 | done |
| Node / Edge CRUD | done |
| 节点类型 (category/leaf) + metadata | done |
| 多图谱支持 (每 Ring 最多 3 个) | done |
| 图谱选择器 UI | done |
| 标签过滤 | done |
| 展开/折叠子节点 | done |
| Graph 对话修正 (自然语言操作图谱) | done |
| 导出 graph.json / SVG | done |

### 归档系统

| 功能 | 状态 |
|---|---|
| 对话 → 图谱节点 + Markdown + Git commit | done |
| Creator 直接 commit, Member 提交 MR | done |
| Archive queue + PR Review (merge/reject) | done |
| PR Diff 视图 | done |
| 归档触发多样化 (命令/按钮/自然语言/AI 推荐) | done |
| Auto 模式 (AI 自动判断归档) | done |
| Ring Git 初始化 (自动 git init + 初始 commit) | done |

### Session 系统

| 功能 | 状态 |
|---|---|
| Session CRUD + 全生命周期管理 | done |
| WebSocket 实时多人聊天 | done |
| Owner 离线暂停 / 重连恢复 / Catch-up | done |
| 材料准备 + 高亮标记 | done |
| AI 总结 (SSE 流式) | done |
| Skill 集成 (加载 Skill system_prompt) | done |
| 5 个内置 Skill (decision/research/review/retrospective/knowledge_sharing) | done |
| Skill 安装/卸载 (从 URL) | done |

### Super Ring

| 功能 | 状态 |
|---|---|
| 始终流式 + tool_calls (跨 Ring 查询/偏好/Skill 管理) | done |
| System Prompt 编辑 | done |
| 用户偏好编辑 | done |
| 跨 Ring 问答/分析 | done |

### Self 系统

| 功能 | 状态 |
|---|---|
| Memory / Personality / Privacy / Export / Reset | done |
| @self 转发 (从 Group Ring) | done |
| 主动建议 | done |
| 数据导出/重置 | done |

### 协作

| 功能 | 状态 |
|---|---|
| 邀请 Token CRUD (open/audit) | done |
| 开放链接加入 + 审核链接 + 审批流程 | done |
| 安装导航页 (OS 检测) | done |
| 成员列表 + 角色变更 + 移除 | done |
| 通知系统 (bell + 未读数) | done |

### 导出 / 配置

| 功能 | 状态 |
|---|---|
| 聊天/会话/图谱/全 Ring 备份导出 | done |
| 图谱 SVG + AI 结构化报告 + tar.gz | done |
| .group/ 文档体系 (role.md + conventions.md) | done |
| .group/ AI 自动维护 (active-context/archive-patterns/corrections/knowledge-summary) | done |
| Blueprint / 模板系统 (5 个内置模板) | done |
| Token 阈值管理 (100k limit, 80% warning) | done |
| Config 面板 (LLM / GitLab / Privacy / auto_compact) | done |

## 后端 API

共 60+ 端点。完整列表见 [api-design.md](technical/api-design.md)。

核心路由：

| 域 | 路径前缀 |
|---|---|
| Setup | `/api/setup` |
| Ring | `/api/rings` |
| Chat | `/api/rings/{id}/chat`, `/api/self/chat`, `/api/super/chat` |
| Graph | `/api/rings/{id}/graph` |
| Session | `/api/rings/{id}/sessions` |
| Archive | `/api/rings/{id}/archive`, `/api/rings/{id}/archives` |
| Export | `/api/rings/{id}/export` |
| Super | `/api/super/*` |
| Skill | `/api/skills` |
| Config | `/api/config/*` |
| Notification | `/api/notifications` |
| Invite/Join | `/api/rings/{id}/invite-tokens`, `/api/join/*` |

## PRD 缺失项（代码实况核查）

> 以下通过逐项对比 PRD 和代码确认。

| PRD 要求 | 状态 | 说明 |
|---|---|---|
| 图谱 SVG/PNG/PDF 图片导出 | **done** | `GraphCanvas.tsx` 前端 SVG 导出，D3 SVG 克隆 + XMLSerializer |
| AI 结构化报告导出 | **done** | `GET /rings/{id}/export/report?node_ids=...&topic=...`，SSE 流式生成 |
| 全 Ring tar.gz 备份 | **done** | `export_ring_backup` 返回 `.tar.gz`（metadata + graph + chat + sessions + archives） |
| 预设工作流工具（文件解析/知识提取/深度调研） | **缺失** | 后端无对应 service，PRD 2.7，优先级低 |
| 多图谱前端 UI（选择器/切换/创建/删除） | **done** | `GraphPanel.tsx` 图谱选择器标签页 + 创建/删除 |
| 成员创建 Session 需授权（grant-session） | **done** | `POST /members/{tid}/grant-session` + `revoke-session`，仅 creator/admin |
| 成员移除时 Session ownership 转移 | **done** | 移除成员时检查 session ownership，有活动 session 则拒绝并报错 |
| Session 所有权转移 | **done** | `POST /sessions/{sid}/transfer-ownership`，仅 creator，新 owner 必须是参与者 |
| Super Ring `cross_ring_cache/` 缓存 | **缺失** | PRD 2.6 存储结构，代码未实现缓存目录，优先级低 |
| Self 完整数据文件（identity/style/knowledge/goals/growth/*.json） | 部分 | 有 DB 字段，但非 PRD 定义的 `.self/` 文件体系 |
| Self metrics 四项统计（session_stats/tool_usage/dwell_time/archive_patterns） | 部分 | `self_data::get_metrics` 有基础统计，但非 PRD 定义的独立 JSON |

## 已知体验问题

| 问题 | 优先级 | 状态 |
|---|---|---|
| ~~消息气泡未左右分布~~ | ~~P1~~ | **fixed** — 用户右对齐 + AI 左对齐 |
| ~~Self 流式输出时阻塞 UI~~ | ~~P1~~ | **fixed** — 移除 input disabled={sending} |
| 缺少全文搜索（跨 Ring/节点/消息） | P1 | |
| 缺少文件上传（只支持文本） | P1 | |
| 长消息不可折叠 | P2 | |
| 缺少 /clear /new 命令 | P2 | |
| Settings 缺少个人信息展示 | P2 | |
| 前端 chunk 过大 (556KB) | P2 | |

## 优化方向

**短期**：实现全文搜索 + 文件上传
**中期**：移动端适配，性能优化，前端 chunk 优化
**长期**：插件系统，预设工作流工具
