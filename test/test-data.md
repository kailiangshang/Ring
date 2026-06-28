# 共用测试数据

## 测试身份

| 身份 | 显示名 | 用途 |
| --- | --- | --- |
| Creator | Alice Creator | 创建 Ring、审批邀请、管理成员 |
| Admin | Bob Admin | 验证管理权限 |
| Member | Carol Member | 验证普通协作 |
| Guest | Dave Guest | 验证只读/受限权限 |

## Ring 名称

- `技术架构讨论`
- `竞品分析知识库`
- `产品需求评审`

## 图谱节点

- `认证系统`
- `权限模型`
- `Session 生命周期`
- `归档策略`
- `跨 Ring 搜索`

## AI 聊天测试文本

### Group Ring

```text
我们正在讨论新一代知识协作平台的技术架构。候选方案有 Rust Axum、Node.js NestJS 和 Python FastAPI。请帮我整理各方案的优劣、适用场景、主要风险，并建议哪些内容应该进入知识图谱。
```

### 归档意图

```text
请把这次讨论归档，标题为 Q3 技术架构选型结论。结论包括：后端优先 Rust Axum，前端保持 React，短期不引入微服务，风险是团队 Rust 熟练度不足。
```

### Session 讨论

```text
我认为 Rust Axum 的优势是部署简单和性能稳定，但开发效率可能低于 Node.js。我们需要确认团队学习成本是否可接受。
```

### Super Ring 跨 Ring 查询

```text
帮我查一下所有 Ring 里和权限模型、邀请机制、Session 协作有关的结论，按 Ring 分组总结，并指出冲突点。
```

### Self 私人记忆

```text
请记住：我偏好先列风险再列方案，技术文档要简洁，默认用中文回答。我的邮箱是 alice@example.com，测试隐私过滤时不要暴露它。
```

## 归档 Markdown 样例

```markdown
# Q3 技术架构选型结论

## 背景

团队需要为知识协作平台选择后端框架。

## 结论

- 后端优先使用 Rust Axum。
- 前端保持 React。
- 短期不拆微服务。

## 风险

- Rust 学习成本较高。
- LLM 流式输出需要稳定降级策略。

## 行动项

- Alice：补充权限模型设计。
- Bob：验证 WebSocket 并发能力。
```

## 隐私过滤样例

```text
我的手机号是 13800138000，邮箱 alice@example.com，OpenAI Key 是 sk-test-123456，GitLab Token 是 glpat-test-abcdef。
```
