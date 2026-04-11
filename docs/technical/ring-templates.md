# Ring .ring/ 初始模板

> **Affects**: [data-model.md](data-model.md) · [backend.md](../api/backend.md)
> **Depends on**: [PRD.md](../product/PRD.md) · [git-integration.md](git-integration.md)
> **Last verified**: 2026-04-11

创建 Ring 时，后端初始化 `.ring/` 目录下的 6 个文件。`role.md` 和 `conventions.md` 由用户输入填充，其余由系统生成默认内容。

---

## 1. role.md

```
# 角色定义

{role_description}

## 行为准则

- 回复使用中文
- 优先基于 Ring 内已有的知识回答问题
- 不确定时明确告知，不猜测
- 归档推荐时说明理由
```

> `{role_description}` = 创建 Ring 时用户填写的角色描述（如"你是一个产品分析专家，帮助团队进行竞品研究"）

---

## 2. conventions.md

```
# 团队约定

## 命名规范

- 节点名称使用中文
- Markdown 文件名使用英文小写 + 连字符

## 术语表

（暂无，使用中积累）

## 分类偏好

（暂无，使用中积累）
```

---

## 3. active-context.md

```
# 当前活跃上下文

（系统自动维护，用户无需编辑）

最近活动：Ring 刚创建，等待首次使用。
```

---

## 4. archive-patterns.md

```
# 归档模式

（AI 自动积累，记录用户的归档偏好）

## 归档粒度偏好

（暂无数据）

## 归档位置偏好

（暂无数据）

## 更新 vs 新建偏好

（暂无数据）
```

---

## 5. corrections.md

```
# 修正记录

（AI 自动积累，记录用户对 AI 行为的修正）

（暂无修正记录）
```

---

## 6. knowledge-summary.md

```
# 知识总结

（AI 定期生成，总结 Ring 的知识全貌）

（暂无数据，使用后由 AI 定期更新）
```
