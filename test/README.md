# Ring 人工测试总览

## 测试目标

验证 Ring 的主要功能输入、输出、状态变化和协作流程是否符合预期。本文档用于人工测试，不是自动化测试代码。

## 建议执行顺序

1. `01-setup-auth`
2. `02-ring-management`
3. `03-members-invite-collaboration`
4. `04-chat-ai`
5. `05-knowledge-graph`
6. `06-session-collaboration`
7. `07-archive-git-sync`
8. `08-super-cross-ring`
9. `09-self-memory-privacy`
10. `10-skills-blueprint`
11. `11-export-upload-config`
12. `12-notifications-websocket-network`

## 通用环境

- 后端运行在 `http://localhost:7420`。
- API Base URL 为 `http://localhost:7420/api`。
- 认证 Header 为 `X-Ring-Token: <token>`。
- 准备至少 3 个测试身份：Creator、Member、Guest。
- 建议测试前备份或清空本地 `~/.ring/`，避免旧数据影响判断。

## 通用记录格式

每条用例执行后记录：

- 实际结果：
- 是否通过：
- 截图/响应片段：
- 问题描述：
- 复现步骤：

## 覆盖矩阵

| 模块 | 覆盖内容 |
| --- | --- |
| Setup/Auth | 初始化、重复初始化、恢复、Token 轮换、未认证 |
| Ring | 创建、列表、详情、删除、模式隔离 |
| 协作 | 成员、角色、邀请、申请、批准、拒绝、通知 |
| Chat AI | Group/Self/Super、SSE、历史、删除、compact、隐私过滤 |
| Graph | 多图谱、节点、边、级联删除、Session 提取 |
| Session | 材料准备、讨论、参与者、总结、关闭、重开、所有权 |
| Archive/Git/Sync | 快速归档、AI 归档、review、diff、revert、bundle/import |
| Super | 跨 Ring 搜索、分析、工具调用、全局偏好 |
| Self | 身份、风格、人格、隐私、记忆、metrics、导出、reset |
| Skill/Blueprint | Skill 管理、蓝图模板、确认、蓝图对话权限 |
| Export/Upload/Config | 导出、上传解析、LLM/GitLab/隐私配置 |
| Network/WebSocket | 网络信息、WebSocket Session 消息、通知读写 |

## 共用测试数据

详见 `test/test-data.md`。
