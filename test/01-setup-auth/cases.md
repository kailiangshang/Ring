# Setup/Auth 测试用例

## 功能输入输出

| 功能 | 入口 | 输入 | 输出 |
| --- | --- | --- | --- |
| Setup 状态 | `GET /api/setup/status` | 无 | `is_setup`、版本信息 |
| 首次 Setup | `POST /api/setup` | display_name、LLM、GitLab 配置 | token_id、display_name |
| 更新 Setup | `PUT /api/setup` | 更新后的配置 | 更新结果 |
| Token 恢复 | `GET /api/setup/recover` | 无 | token_id |
| Token 轮换 | `POST /api/auth/rotate` | 当前 token | new token_id |

## 用例

### SA-01 首次启动状态

- 前置条件：清空或隔离本地测试数据。
- 输入：无。
- 步骤：访问 `GET /api/setup/status`。
- 期望输出：HTTP 200，`is_setup=false`。
- 问题记录：

### SA-02 完成首次 Setup

- 前置条件：SA-01 通过。
- 输入：

```json
{"display_name":"Alice Creator","llm_provider":"openai","llm_api_key":"sk-test","llm_model":"gpt-4o","gitlab_url":"https://gitlab.test.com","gitlab_token":"glpat-test"}
```

- 步骤：提交 `POST /api/setup`。
- 期望输出：HTTP 200，返回非空 `token_id`，`display_name=Alice Creator`。
- 问题记录：

### SA-03 重复 Setup 被拒绝

- 前置条件：SA-02 已完成。
- 输入：重复 SA-02 请求体。
- 步骤：再次提交 `POST /api/setup`。
- 期望输出：HTTP 409。
- 问题记录：

### SA-04 未认证访问受保护接口

- 前置条件：Setup 已完成。
- 输入：不带 `X-Ring-Token`。
- 步骤：访问 `GET /api/rings`。
- 期望输出：HTTP 401。
- 问题记录：

### SA-05 Token 轮换

- 前置条件：持有 Creator token。
- 输入：Header `X-Ring-Token`。
- 步骤：访问 `POST /api/auth/rotate`，再用旧 token 和新 token 分别访问 `GET /api/rings`。
- 期望输出：新 token 可用；旧 token 行为按产品约定记录，若仍可用需确认是否符合预期。
- 问题记录：
