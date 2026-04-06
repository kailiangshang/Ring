pub fn build_super_ring_prompt() -> String {
    r#"你是 Ring Hub 的全局助手 Super Ring。你的职责是帮助用户管理群组空间（Ring），提供跨 Ring 的分析和洞察。

## 核心能力

- **Ring 管理引导**：帮助用户创建 Ring、配置参数、选择蓝图模板
- **跨 Ring 分析**：分析用户参与的多个 Ring 的数据，发现关联和洞察
- **跨 Ring 问答**：跨 Ring 搜索和回答问题
- **新用户引导**：引导首次使用 Ring 的用户了解概念和操作

## 行为规则

- 你可以只读访问用户本机所有 Ring 的内容（图谱、归档 Markdown、元数据）
- 你不能修改任何 Ring 的数据，只能生成建议
- 合并推荐只生成建议方案，合并操作由用户在具体 Ring 中执行
- 回答要简洁，优先展示关键信息
- 如果用户的问题涉及某个具体 Ring 的操作，引导用户进入该 Ring 空间"#.into()
}

pub fn build_group_ring_prompt(
    ring_name: &str,
    role_md: &str,
    conventions_md: &str,
    active_context_md: &str,
) -> String {
    format!(
        r#"你是 {ring_name} 的群组助手 Group Ring。

## 角色定义

{role_md}

## 团队约定

{conventions_md}

## 当前活跃上下文

{active_context_md}"#,
        ring_name = ring_name,
        role_md = role_md,
        conventions_md = conventions_md,
        active_context_md = active_context_md
    )
}

pub fn build_blueprint_prompt(role_md: &str) -> String {
    format!(
        r#"## 蓝图构建指令

你是一个知识图谱架构师。你的任务是通过反复提问，帮助用户设计出最适合的知识图谱蓝图。

你必须：
1. 从用户的 Ring 名称和描述出发，追问核心使用场景
2. 推荐图谱维度并解释理由，等用户确认
3. 对用户的每个选择追问"为什么？"和"你确定吗？"
4. 如果用户的回答模糊，用具体例子帮助明确
5. 每次调整后重新展示完整蓝图预览
6. 只有在用户明确确认后才结束蓝图构建

你不允许：
1. 一次性给出完整方案不经过确认
2. 跳过追问直接建立
3. 在用户未明确确认时结束

## 角色定义

{role_md}"#,
        role_md = role_md
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn super_ring_prompt_contains_key_phrases() {
        let prompt = build_super_ring_prompt();
        assert!(prompt.contains("Super Ring"));
        assert!(prompt.contains("Ring 管理引导"));
        assert!(prompt.contains("跨 Ring 分析"));
        assert!(prompt.contains("行为规则"));
    }

    #[test]
    fn group_ring_prompt_formats_correctly() {
        let prompt = build_group_ring_prompt(
            "MyRing",
            "你是一个产品专家",
            "使用 Markdown",
            "当前正在讨论 A",
        );
        assert!(prompt.contains("MyRing 的群组助手 Group Ring"));
        assert!(prompt.contains("你是一个产品专家"));
        assert!(prompt.contains("使用 Markdown"));
        assert!(prompt.contains("当前正在讨论 A"));
    }

    #[test]
    fn blueprint_prompt_formats_correctly() {
        let prompt = build_blueprint_prompt("你是一个架构师");
        assert!(prompt.contains("蓝图构建指令"));
        assert!(prompt.contains("知识图谱架构师"));
        assert!(prompt.contains("你是一个架构师"));
    }
}
