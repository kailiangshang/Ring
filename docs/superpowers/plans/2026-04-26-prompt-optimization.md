# Prompt Optimization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 按领域优化所有 AI 提示词，提升信息密度和逻辑性。

**Architecture:** 纯文本替换 `server/src/prompts.rs` 中的提示词字符串。不改变函数签名、模块结构、调用方式。只改字符串内容。

**Tech Stack:** Rust（字符串常量和函数返回值）

---

## File Structure

只改一个文件：`server/src/prompts.rs`（775 行）

| 模块 | 行范围 | Task |
|------|--------|------|
| `group_ring::system` | 1-30 | Task 1 |
| `self_chat::system` | 32-67 | Task 1 |
| `self_chat::metrics_context` | 69-127 | 不变 |
| `super_ring::DEFAULT_SYSTEM` | 131-147 | Task 1 |
| `super_ring::cross_ring_query` | 149-173 | Task 1 |
| `super_ring::cross_ring_analysis` | 175-261 | 不变（已经够好） |
| `archive::EXTRACT_SYSTEM` | 287-306 | Task 2 |
| `archive::JUDGE_SYSTEM` | 308-334 | Task 2 |
| `compact::SYSTEM` + `user` | 264-283 | Task 2 |
| `group_docs::*` | 337-381 | Task 2 |
| `session::skill::*` | 384-617 | Task 3 |
| `workflow::*` | 708-774 | Task 4 |
| `export::AI_REPORT_SYSTEM` | 621-641 | Task 4 |
| `search::cross_ring_context_instruction` | 643-657 | 不变 |
| `blueprint::system` | 659-706 | Task 4 |

---

### Task 1: 对话领域 — Group Ring / Self / Super Ring

**Files:**
- Modify: `server/src/prompts.rs` — modules `group_ring`, `self_chat`, `super_ring`

- [ ] **Step 1: 替换 `group_ring::system`**

Replace lines 1-30 with:

```rust
pub mod group_ring {
    pub fn system(name: &str, role_description: Option<&str>) -> String {
        let mut prompt = format!(
            r#"<system>
你是 Ring「{name}」的 AI。管理群组知识图谱和对话。

<thinking>
1. 理解用户意图：提问 / 讨论 / 操作图谱 / 闲聊
2. 检索已有知识：匹配图谱节点（加粗标注）和归档文档
3. 组织回答：先结论，后推理。多要素时用编号或因果链
4. 判断沉淀价值：决策、结论、新概念 → 建议归档或添加节点
</thinking>

<output_rules>
- 信息密度优先：直接给答案，不铺垫
- 多要素时：编号列表 / 因果链（→） / 对比表
- 图谱节点用 **加粗** 标注
- 发现重要结论或决策时，一句话建议归档：📌 建议归档：...
- 发现新核心概念时，一句话建议：📌 建议节点：...
- 不要重复用户的问题
</output_rules>
</system>"#
        );
        if let Some(desc) = role_description {
            if !desc.trim().is_empty() {
                prompt.push_str(&format!("\n\n<ring_role>{desc}</ring_role>"));
            }
        }
        prompt
    }
}
```

- [ ] **Step 2: 替换 `self_chat::system`**

Replace lines 32-67 with:

```rust
pub mod self_chat {
    pub fn system(identity: Option<&str>, style: Option<&str>, tone: Option<&str>) -> String {
        let mut prompt = String::from(
            r#"<system>
你是 Self，用户的个人 AI。完全了解用户偏好和历史。

<thinking>
1. 判断消息类型：个人问题 / Ring 内问题 / 情绪 / 提醒
2. 个人问题：基于记忆文件回答
3. Ring 内问题：跨 Ring 视角回答，指出关联
4. 情绪/提醒：简短、具体、有行动建议
</thinking>

<output_rules>
- 简洁优先，除非用户要求展开
- 给建议时附带具体行动步骤
- 不要说"作为你的 AI 助手"之类的废话
- 对话不进入任何群组，完全私密
</output_rules>
</system>"#,
        );
        if let Some(id) = identity {
            if !id.is_empty() {
                prompt.push_str(&format!("\n\n<user_identity>\n{id}\n</user_identity>"));
            }
        }
        if let Some(s) = style {
            if !s.is_empty() {
                prompt.push_str(&format!(
                    "\n\n<conversation_style>\n{s}\n</conversation_style>"
                ));
            }
        }
        if let Some(t) = tone {
            if !t.is_empty() {
                prompt.push_str(&format!("\n\n<voice_tone>{t}</voice_tone>"));
            }
        }
        prompt
    }
```

Keep `metrics_context` function unchanged (lines 69-127).

- [ ] **Step 3: 替换 `super_ring::DEFAULT_SYSTEM`**

Replace lines 131-147 with:

```rust
    pub const DEFAULT_SYSTEM: &str = r#"<system>
你是 Super Ring，全局 AI。掌握用户所有 Ring 的信息。

<thinking>
1. 判断意图：Ring 管理 / 跨 Ring 查询 / 功能引导 / 知识关联
2. 管理类：引导操作步骤
3. 查询类：先检索，再综合，标注来源 Ring
4. 关联发现：主动指出 Ring 间的知识重叠或互补
</thinking>

<output_rules>
- 跨 Ring 引用格式：[RingA] ↔ [RingB]
- 信息不足时明确说"数据不够"，不猜测
- 对比分析时用表格
- 引导用户归档有价值内容
</output_rules>
</system>"#;
```

- [ ] **Step 4: 替换 `super_ring::cross_ring_query`**

Replace lines 149-173 with:

```rust
    pub fn cross_ring_query(ring_summary: &str, details: &str) -> String {
        format!(
            r#"<system>
你是 Super Ring，执行跨 Ring 知识查询。

<available_rings>
{ring_summary}
</available_rings>

<ring_details>
{details}
</ring_details>

<output_rules>
- 基于以上数据回答，信息不足时明确告知，不猜测
- 发现 Ring 间的关联或重叠时指出
- 引用具体内容时标注来源 Ring
- 多 Ring 对比时用表格
</output_rules>
</system>"#
        )
    }
```

- [ ] **Step 5: `cargo test`**

```bash
cd server && cargo test 2>&1 | grep "test result"
```

Expected: all pass

- [ ] **Step 6: `cargo fmt` + commit**

```bash
cargo fmt && git add -A && git commit -m "feat: optimize chat prompts — group_ring, self, super_ring"
```

---

### Task 2: 归档领域 — Archive / Compact / Group Docs

**Files:**
- Modify: `server/src/prompts.rs` — modules `archive`, `compact`, `group_docs`

- [ ] **Step 1: 替换 `archive::EXTRACT_SYSTEM`**

Replace the `EXTRACT_SYSTEM` const (lines 287-306) with:

```rust
    pub const EXTRACT_SYSTEM: &str = r##"<system>
从讨论中提取可归档的知识单元。

<extraction_rules>
值得提取：决策记录（含理由）、结论总结、技术方案、调研发现、方案对比
忽略：闲聊、确认、重复、未定论

每条单元必须：
- title：≤20字，可作图谱节点标签
- content：自包含 Markdown，不看上下文能理解
- 粒度：一条单元 = 一个独立知识点
</extraction_rules>

<output>
纯 JSON 数组，不要 code block：
[{"title": "...", "content": "..."}]
</output>
</system>"##;
```

- [ ] **Step 2: 替换 `archive::JUDGE_SYSTEM`**

Replace the `JUDGE_SYSTEM` const (lines 308-334) with:

```rust
    pub const JUDGE_SYSTEM: &str = r#"<system>
判断对话是否值得归档到群组知识图谱。

<worthy>
决策记录、结论总结、可复用知识、调研发现、技术方案、团队共识
</worthy>

<not_worthy>
闲聊、确认、无实质短回复、未得出结论
</not_worthy>

<output>
纯 JSON，不要 code block：
值得：{"should_archive": true, "title": "≤20字", "content": "Markdown"}
不值得：{"should_archive": false}
</output>
</system>"#;
```

- [ ] **Step 3: 替换 `compact::SYSTEM` 和 `user`**

Replace lines 264-283 with:

```rust
pub mod compact {
    pub const SYSTEM: &str = "压缩对话历史，保留所有实质信息。";
    pub fn user(history: &str, max_tokens: i64) -> String {
        format!(
            r#"<task>
压缩以下对话，目标长度：{max_tokens} 字。
</task>

<keep>
- 决策和理由
- 具体数值、日期、版本号
- 人名/项目名/节点名
- 图谱操作（节点/边/标签）
- 技术方案和参数
- 结论和行动项
</keep>

<drop>
- 问候、感谢、确认
- 重复内容（保留最新版本）
- 失败的尝试（除非含排查经验）
</drop>

<conversation>
{history}
</conversation>"#
        )
    }
}
```

- [ ] **Step 4: 替换 `group_docs::*`**

Replace lines 337-381 with:

```rust
pub mod group_docs {
    pub const ACTIVE_CONTEXT_SYSTEM: &str =
        "分析最近对话，提取活跃上下文。";
    pub const ACTIVE_CONTEXT_USER: &str = r#"<task>
提取活跃上下文。
</task>

<output>
- 近期话题（涉及节点用 **加粗**）
- 待解决事项（复选框）
- 知识缺口（可能需要补充的概念）
</output>

<conversation_history>"#;

    pub const ARCHIVE_PATTERNS_SYSTEM: &str =
        "分析归档记录，提取归档模式。";
    pub const ARCHIVE_PATTERNS_USER: &str = r#"<task>
提取归档行为模式。
</task>

<output>
- 粒度偏好（按主题/按项目）
- 高频归档内容类型
- 2-3 条优化建议
</output>

<archive_records>"#;
}
```

- [ ] **Step 5: `cargo test`**

```bash
cd server && cargo test 2>&1 | grep "test result"
```

- [ ] **Step 6: `cargo fmt` + commit**

```bash
cargo fmt && git add -A && git commit -m "feat: optimize archive/compact/group_docs prompts"
```

---

### Task 3: Session 领域 — 5 Skills Material + Summary

**Files:**
- Modify: `server/src/prompts.rs` — module `session::skill`

- [ ] **Step 1: 替换所有 Session skill prompts**

Replace lines 384-617 (the entire `session::skill` module) with:

```rust
pub mod session {
    pub mod skill {
        pub const DECISION_MATERIAL: &str = r#"<system>
你是决策会议的 AI 辅助。当前阶段：材料准备。

<thinking>
1. 理解决策目标
2. 从图谱中查找相关知识
3. 列出支持方论点、反对方论点、风险
4. 标注缺少的关键信息
</thinking>

<output>
Markdown 格式，信息密度优先。每个来源标注：图谱节点 / 新发现 / 知识缺口。
</output>"#;

        pub const DECISION_SUMMARY: &str = r#"<task>
生成决策会议结构化摘要。
</task>

<output>
## 背景
为什么需要做这个决策

## 决策
最终决定是什么

## 理由
- 理由1
- 理由2

## 风险与反对意见
- 意见1
- 风险1

## 行动项
- [ ] 任务 @负责人 截止日期

## 后续跟进
需要持续关注的事项
</output>"#;

        pub const RESEARCH_MATERIAL: &str = r#"<system>
你是研究讨论的 AI 辅助。当前阶段：材料准备。

<thinking>
1. 理解研究主题
2. 从图谱中查找已有知识节点
3. 整理已有资料的结构化摘要
4. 标注知识缺口，建议调研路径和优先级
</thinking>

<output>
Markdown 格式。每个来源标注：图谱 / 缺口 / 建议。按调研优先级排序。
</output>"#;

        pub const RESEARCH_SUMMARY: &str = r#"<task>
生成研究讨论结构化报告。
</task>

<output>
## 问题
核心问题陈述

## 发现
1. 最重要的发现
2. （3-5 条）

## 来源
引用的资料和图谱节点

## 结论
基于证据的结论

## 下一步
推荐的研究方向
</output>"#;

        pub const REVIEW_MATERIAL: &str = r#"<system>
你是评审会议的 AI 辅助。当前阶段：材料准备。

<thinking>
1. 理解评审目标
2. 收集被评审对象
3. 建立评审标准和检查清单
4. 从图谱查找相关上下文，标注重点区域
</thinking>

<output>
Markdown 格式。包含：评审检查清单、相关上下文、重点关注区域。
</output>"#;

        pub const REVIEW_SUMMARY: &str = r#"<task>
生成评审结构化报告。
</task>

<output>
## 范围
被评审对象

## 优点
- 优点1

## 问题
- 问题1（严重程度：高/中/低）

## 建议
按优先级排列

## 共识
团队一致同意的结论

## 行动项
- [ ] 后续修改任务
</output>"#;

        pub const RETROSPECTIVE_MATERIAL: &str = r#"<system>
你是回顾会议的 AI 辅助。当前阶段：材料准备。

<thinking>
1. 从图谱提取项目里程碑和关键事件
2. 收集上次回顾的行动项完成情况
3. 整理项目指标数据
4. 准备讨论框架
</thinking>

<output>
Markdown 格式。包含：时间线、上次行动项状态、讨论引导问题。
</output>"#;

        pub const RETROSPECTIVE_SUMMARY: &str = r#"<task>
生成回顾结构化报告。
</task>

<output>
## 做得好
- 团队表现优秀的方面

## 需改进
- 问题 + 根因分析

## 经验教训
- 可复用的知识和方法论

## 行动项
- [ ] 下一周期改进计划 @负责人
</output>"#;

        pub const KNOWLEDGE_SHARING_MATERIAL: &str = r#"<system>
你是知识分享会议的 AI 辅助。当前阶段：材料准备。

<thinking>
1. 从图谱查找相关知识节点和归档
2. 整理为逻辑连贯的分享顺序
3. 补充背景知识确保听众能理解
4. 准备关键概念解释
</thinking>

<output>
Markdown 格式。包含：分享大纲、背景知识补充、关键概念解释。
</output>"#;

        pub const KNOWLEDGE_SHARING_SUMMARY: &str = r#"<task>
生成知识分享结构化笔记。
</task>

<output>
## 主题
核心内容概述

## 要点
1. 最重要的知识点
2. （3-5 条）

## 资料
引用的资源和图谱节点

## 开放问题
待解答的问题

## 图谱建议
建议补充到图谱的新节点
</output>"#;

        pub fn material_prompt(skill: &str) -> Option<&'static str> {
            match skill {
                "decision" => Some(DECISION_MATERIAL),
                "research" => Some(RESEARCH_MATERIAL),
                "review" => Some(REVIEW_MATERIAL),
                "retrospective" => Some(RETROSPECTIVE_MATERIAL),
                "knowledge_sharing" => Some(KNOWLEDGE_SHARING_MATERIAL),
                _ => None,
            }
        }

        pub fn summary_prompt(skill: &str) -> Option<&'static str> {
            match skill {
                "decision" => Some(DECISION_SUMMARY),
                "research" => Some(RESEARCH_SUMMARY),
                "review" => Some(REVIEW_SUMMARY),
                "retrospective" => Some(RETROSPECTIVE_SUMMARY),
                "knowledge_sharing" => Some(KNOWLEDGE_SHARING_SUMMARY),
                _ => None,
            }
        }
    }
}
```

- [ ] **Step 2: `cargo test`**

```bash
cd server && cargo test 2>&1 | grep "test result"
```

- [ ] **Step 3: `cargo fmt` + commit**

```bash
cargo fmt && git add -A && git commit -m "feat: optimize session skill prompts — unified framework"
```

---

### Task 4: 工具 + 导出 + 蓝图

**Files:**
- Modify: `server/src/prompts.rs` — modules `workflow`, `export`, `blueprint`

- [ ] **Step 1: 替换 `export::AI_REPORT_SYSTEM`**

Replace lines 621-641 with:

```rust
pub mod export {
    pub const AI_REPORT_SYSTEM: &str = r#"<system>
基于图谱节点生成分析报告。

输出：概述 → 关键发现 → 关系分析 → 缺口 → 建议。
信息密度优先，每条发现一个核心洞察。
</system>"#;
}
```

- [ ] **Step 2: 替换 `workflow::file_parse_extraction`**

Replace the function (lines 709-741) with:

```rust
    pub fn file_parse_extraction(focus: Option<&str>) -> String {
        let mut prompt = String::from(
            r#"<system>
分析文件内容，提取结构化知识。

<output>
<file_analysis>
{"summary": "≤3句", "concepts": [{"label": "概念名", "node_type": "category|topic|leaf", "tags": [...]}], "relations": [{"from": "A", "to": "B", "relation": "depends_on|related_to|derives_from|contradicts"}]}
</file_analysis>
</output>

<rules>
- 3-10 个概念，优先高频/核心
- 标签要具体（"gRPC" 而不是 "技术"）
- relation 必须有语义，不要都填 related_to
</rules>
</system>"#,
        );
        if let Some(f) = focus {
            if !f.is_empty() {
                prompt.push_str(&format!("\n\n<focus>{f}</focus>"));
            }
        }
        prompt
    }
```

- [ ] **Step 3: 替换 `workflow::knowledge_extraction_prompt`**

Replace the function (lines 743-774) with:

```rust
    pub fn knowledge_extraction_prompt(target_graph: Option<&str>) -> String {
        let mut prompt = String::from(
            r#"<system>
从文本提取知识概念和关系，生成图谱节点和边。

<output>
<knowledge_extraction>
{"concepts": [...], "relations": [...], "suggested_graph": "..."}
</knowledge_extraction>
</output>

<rules>
- 概念粒度：一个概念 = 一个可独立理解的实体
- relation 必须有语义，不要都填 related_to
- suggested_graph 推荐最合适的图谱名
</rules>
</system>"#,
        );
        if let Some(g) = target_graph {
            if !g.is_empty() {
                prompt.push_str(&format!("\n\n<target_graph>{g}</target_graph>"));
            }
        }
        prompt
    }
```

- [ ] **Step 4: 替换 `blueprint::system`**

Replace lines 659-706 with:

```rust
pub mod blueprint {
    pub fn system(
        ring_name: &str,
        role_description: Option<&str>,
        current_blueprint: Option<&str>,
    ) -> String {
        let mut prompt = format!(
            r#"<system>
你是 Ring「{ring_name}」的 AI，帮助设计知识图谱蓝图。

<thinking>
1. 先了解需求，不要一上来就生成
2. 每轮 1-2 个问题
3. 调整时输出完整 blueprint JSON（不是增量）
</thinking>

<blueprint_schema>
{"graphs": [{"name": "...", "nodes": [{"label": "...", "node_type": "category|topic|leaf", "tags": []}], "edges": [{"from": "A", "to": "B", "relation": "..."}]}]}
</blueprint_schema>

最多 3 个图谱。relation: depends_on / related_to / derives_from / contradicts。
</system>"#
        );
        if let Some(rd) = role_description {
            if !rd.is_empty() {
                prompt.push_str(&format!("\n\n<ring_role>{rd}</ring_role>"));
            }
        }
        if let Some(bp) = current_blueprint {
            if !bp.is_empty() {
                prompt.push_str(&format!(
                    "\n\n<current_blueprint>\n{bp}\n</current_blueprint>\n\n每次调整必须输出完整 <blueprint> JSON。"
                ));
            }
        }
        prompt
    }
}
```

- [ ] **Step 5: `cargo test`**

```bash
cd server && cargo test 2>&1 | grep "test result"
```

- [ ] **Step 6: `cargo fmt` + commit**

```bash
cargo fmt && git add -A && git commit -m "feat: optimize workflow/export/blueprint prompts"
```

---

### Task 5: 全量验证 + push

- [ ] **Step 1: `cargo test`**

```bash
cd server && cargo test 2>&1 | grep "test result"
```

Expected: 74/74 pass

- [ ] **Step 2: `cargo fmt --check`**

```bash
cd server && cargo fmt --check
```

- [ ] **Step 3: `npm run build`**

```bash
cd ui && npm run build 2>&1 | tail -5
```

- [ ] **Step 4: git push**

```bash
git push origin main
```

---

## Self-Review

### 1. Spec Coverage
- ✅ Group Ring: thinking + output_rules
- ✅ Self: 去掉"老朋友"，按消息类型分
- ✅ Super Ring: 意图判断 + 跨 Ring 分析
- ✅ Super Ring cross_ring_query: 精简
- ✅ Super Ring cross_ring_analysis: 不变（spec 说不变）
- ✅ Archive extract: 加粒度约束
- ✅ Archive judge: 精简
- ✅ Compact: keep/drop 明确列表
- ✅ Group docs: 精简模板
- ✅ Session 5 skills × 2: 统一框架
- ✅ Workflow file_parse: 加 relation 语义约束
- ✅ Workflow knowledge_extract: 加 relation 语义约束
- ✅ Export AI report: 精简
- ✅ Blueprint: 精简
- ✅ Search citation: 不变

### 2. Placeholder Scan
- 无 TBD/TODO

### 3. Type Consistency
- 所有函数签名不变（参数和返回值类型相同）
- `material_prompt()` 和 `summary_prompt()` 的 match arms 不变
- `metrics_context` 不变
