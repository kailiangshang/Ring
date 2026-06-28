# Ring Test Cases Spec

## 背景

Ring 是本地优先的 AI 知识协作平台，包含 Setup/Auth、Ring 管理、成员与邀请、AI 聊天、知识图谱、Session 协作、归档、同步、Super Ring、Self、Skill、Blueprint、导出、上传、通知和 WebSocket 等功能。

当前需求是为人工测试生成完整测试方案和测试内容，重点帮助测试人员发现功能问题点，而不是新增自动化测试代码。

## 目标

- 总结每个主要功能点的输入与输出。
- 为每个功能构造可人工执行的测试用例。
- 为文本类和 AI 类功能提供可直接复制的测试文本。
- 将测试内容写入根目录 `test/` 下，并按功能分类放入不同文件夹。

## 范围

- 覆盖 `docs/api.md` 与 `server/src/routes/mod.rs` 中的主要 API/功能入口。
- 覆盖现有协作能力：成员、邀请、Session、通知、WebSocket、同步。
- 覆盖 Ring 的通用能力：聊天、图谱、归档、Self、Super、Skill、Blueprint、导出、上传、配置。

## 非目标

- 不编写 Rust/Vitest 自动化测试。
- 不修改业务代码、路由、模型或前端组件。
- 不要求真实 LLM 输出完全稳定，只检查事件流、落库、状态切换和可观察结果。

## 约束条件

- 测试文档应可被人工测试人员直接执行。
- 接口路径以当前代码路由为准。
- 测试数据尽量使用中文业务场景，贴近 Ring 的知识协作定位。
- 不覆盖已有测试文件，不移动现有 `server/tests` 或 `ui/src/test`。

## 验收标准

- `worklog/ring-test-cases/` 下存在 `spec.md`、`plan.md`、`status.md`。
- 根目录 `test/` 下按功能建立测试文件夹。
- 每个功能文件夹包含清晰的人工测试用例。
- 总览文档包含执行顺序、测试账号、覆盖矩阵和共用测试数据链接。

## 相关文件

- `docs/api.md`
- `server/src/routes/mod.rs`
- `server/tests/integration.rs`
- `ui/src/test/scenarios.test.ts`
