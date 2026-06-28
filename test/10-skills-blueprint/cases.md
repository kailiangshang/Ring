# Skills 与 Blueprint 测试用例

## 功能输入输出

| 功能 | 入口 | 输入 | 输出 |
| --- | --- | --- | --- |
| Skill 列表 | `GET /api/skills` | token | skills |
| Skill 安装 | `POST /api/skills/install` | url | installed skill |
| Skill 详情 | `GET /api/skills/{name}` | name | detail |
| Skill 删除 | `DELETE /api/skills/{name}` | name | ok |
| Blueprint | `/blueprint` | ring_id | status |
| Template | `/blueprint/from-template` | template | preview |
| Confirm | `/blueprint/confirm` | blueprint graphs | status |
| Blueprint Chat | `/blueprint/chat` | content、current_blueprint | SSE |

## 用例

### SB-01 获取内置 Skills

- 前置条件：用户已登录。
- 输入：无。
- 步骤：访问 `GET /api/skills`。
- 期望输出：HTTP 200，包含 decision/research/review 等 Skill 或当前安装列表。
- 问题记录：

### SB-02 Skill 鉴权

- 前置条件：不带 token。
- 输入：无。
- 步骤：访问 `GET /api/skills`。
- 期望输出：HTTP 401。
- 问题记录：

### SB-03 安装和删除 Skill

- 前置条件：准备合法 Skill URL 或本地可访问源。
- 输入：

```json
{"url":"https://example.com/skills/decision-making.yaml"}
```

- 步骤：POST install，GET detail，DELETE skill。
- 期望输出：安装成功后可见详情；删除后列表不再包含。
- 问题记录：

### SB-04 Blueprint 模板预览

- 前置条件：Ring 存在。
- 输入：

```json
{"template":"technical_architecture"}
```

- 步骤：调用 `/blueprint/from-template`。
- 期望输出：返回 nodes/edges 预览，不直接写入正式图谱。
- 问题记录：

### SB-05 确认 Blueprint 写入图谱

- 前置条件：Ring 存在。
- 输入：

```json
{"blueprint":{"graphs":[{"name":"架构图谱","nodes":[{"label":"认证系统","node_type":"category","tags":["安全"]},{"label":"权限模型","node_type":"topic","tags":["协作"]}],"edges":[{"from":"认证系统","to":"权限模型","relation":"contains"}]}]}}
```

- 步骤：POST `/blueprint/confirm`，再查看 `/graph` 或 `/graphs`。
- 期望输出：status confirmed；图谱中出现节点和边。
- 问题记录：

### SB-06 Blueprint Chat 权限

- 前置条件：Creator 和普通 Member 都在 Ring。
- 输入：

```json
{"content":"请帮我设计一个协作权限图谱","current_blueprint":null}
```

- 步骤：Member 调用 blueprint chat，再 Creator 调用。
- 期望输出：Member 若无权限应 403；Creator 可正常 SSE。
- 问题记录：
