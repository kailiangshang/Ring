# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2026-04-28

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

### 基础设施

- 单一 16MB 二进制（前后端一体）
- SQLite + 文件系统，零外部依赖
- 16 个数据库迁移
- 69/69 集成测试通过

### 修复的 Bug

- PDF 导出 CJK 字符崩溃（UTF-8 安全截断）
- `ring_members` 表名错误 → `members`
- `graphs.ring_id` UNIQUE 约束阻止多图谱
- `archives` 表名错误 → `archive_records`
- `RingRow` 缺少 `storage_mode`/`gitlab_namespace` 字段
- `delete_graph` 全局计数 → per-ring 计数
- Notifications `type` 列 → `notification_type`

[1.0.0]: https://github.com/kailiangshang/Ring/releases/tag/v1.0.0
