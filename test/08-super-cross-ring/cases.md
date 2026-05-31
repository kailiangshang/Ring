# Super Ring 与跨 Ring 测试用例

## 功能输入输出

| 功能 | 入口 | 输入 | 输出 |
| --- | --- | --- | --- |
| Super Chat | `POST /api/super/chat` | content | SSE |
| Super History | `GET /api/super/chat/history` | before、limit | messages |
| System Prompt | `/super/system-prompt` | content | prompt |
| Preferences | `/super/preferences` | content | preferences |
| Cross Ring Query | `/super/cross-ring/query` | query | SSE |
| Cross Ring Analysis | `/super/cross-ring/analysis` | ring_names、analysis_type、question | SSE |

## 用例

### SCR-01 查询所有 Ring 概况

- 前置条件：至少创建两个 Ring，且各有消息或图谱。
- 输入：

```text
列出我所有 Ring 的概况，包括成员数量、最近归档和主要主题。
```

- 步骤：提交 Super Chat。
- 期望输出：SSE 正常；回答包含多个 Ring 或明确说明无数据。
- 问题记录：

### SCR-02 跨 Ring 搜索

- 前置条件：不同 Ring 中存在 `权限模型`、`Session 协作` 相关内容。
- 输入：

```json
{"query":"权限模型 Session 协作 邀请机制"}
```

- 步骤：调用 `/api/super/cross-ring/query`。
- 期望输出：返回跨 Ring 汇总；不包含用户无权访问的 Ring。
- 问题记录：

### SCR-03 跨 Ring 分析

- 前置条件：存在 `技术架构讨论` 与 `产品需求评审` 两个 Ring。
- 输入：

```json
{"ring_names":["技术架构讨论","产品需求评审"],"analysis_type":"conflict","question":"找出权限和协作流程上的冲突"}
```

- 步骤：调用 `/api/super/cross-ring/analysis`。
- 期望输出：SSE 正常；分析只基于指定 Ring。
- 问题记录：

### SCR-04 更新全局偏好

- 前置条件：Creator token。
- 输入：

```markdown
## 语言
- default: zh-CN

## 输出格式
- style: concise
- risk_first: true
```

- 步骤：PUT preferences，再 GET。
- 期望输出：内容持久化；后续 Super 回答风格可观察变化。
- 问题记录：

### SCR-05 Super 工具创建 Ring

- 前置条件：LLM tool call 可用。
- 输入：

```text
帮我创建一个 Ring，名称是「竞品分析知识库」，它负责整理 Notion、飞书知识库和 Confluence 的竞品研究。
```

- 步骤：提交 Super Chat，再查看 Ring 列表。
- 期望输出：新 Ring 被创建，角色为 creator。
- 问题记录：
