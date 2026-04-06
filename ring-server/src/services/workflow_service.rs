use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::models::tool_model::ToolDefinition;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub tool_names: Vec<String>,
}

static WORKFLOWS: LazyLock<Vec<Workflow>> = LazyLock::new(|| {
    vec![
        Workflow {
            id: "meeting_archive".into(),
            name: "会议归档".into(),
            description: "将讨论内容整理为结构化文档并归档到知识图谱".into(),
            system_prompt: "你是一个会议归档助手。请整理讨论内容，提取关键要点，生成结构化的归档文档。".into(),
            tool_names: vec![
                "text_clean".into(),
                "privacy_filter".into(),
                "markdown_gen".into(),
                "search".into(),
            ],
        },
        Workflow {
            id: "deep_research".into(),
            name: "深度研究".into(),
            description: "对指定主题进行深入调研，搜索并整合多个信息源".into(),
            system_prompt: "你是一个研究助手。请对指定主题进行深入调研，搜索相关信息源，整合分析后生成研究报告。".into(),
            tool_names: vec![
                "search".into(),
                "web_scrape".into(),
                "text_clean".into(),
                "markdown_gen".into(),
            ],
        },
        Workflow {
            id: "learning_center".into(),
            name: "学习中心".into(),
            description: "帮助用户学习新知识，通过搜索和整理知识图谱构建学习路径".into(),
            system_prompt: "你是一个学习助手。请帮助用户理解概念，搜索相关知识节点，整理学习材料。".into(),
            tool_names: vec![
                "search".into(),
                "text_clean".into(),
                "markdown_gen".into(),
            ],
        },
    ]
});

pub fn get_workflow(id: &str) -> Option<Workflow> {
    WORKFLOWS.iter().find(|w| w.id == id).cloned()
}

pub fn list_workflows() -> Vec<Workflow> {
    WORKFLOWS.clone()
}

pub fn filter_tools_for_workflow(
    all_tools: &[ToolDefinition],
    tool_names: &[String],
) -> Vec<ToolDefinition> {
    all_tools
        .iter()
        .filter(|t| tool_names.contains(&t.name))
        .cloned()
        .collect()
}
