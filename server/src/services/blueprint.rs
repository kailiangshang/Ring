use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub nodes: Vec<BlueprintNode>,
    pub edges: Vec<BlueprintEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintNode {
    pub label: String,
    pub node_type: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
}

pub fn get_builtin_templates() -> Vec<BlueprintTemplate> {
    vec![
        product_research_template(),
        project_management_template(),
        learning_notes_template(),
        technical_docs_template(),
        blank_template(),
    ]
}

fn product_research_template() -> BlueprintTemplate {
    BlueprintTemplate {
        id: "product-research".into(),
        name: "竞品分析".into(),
        description: "用于产品分析和竞品调研".into(),
        nodes: vec![
            BlueprintNode {
                label: "产品概览".into(),
                node_type: "category".into(),
                tags: vec!["overview".into()],
            },
            BlueprintNode {
                label: "竞品 A".into(),
                node_type: "topic".into(),
                tags: vec!["competitor".into()],
            },
            BlueprintNode {
                label: "竞品 B".into(),
                node_type: "topic".into(),
                tags: vec!["competitor".into()],
            },
            BlueprintNode {
                label: "市场趋势".into(),
                node_type: "topic".into(),
                tags: vec!["market".into()],
            },
            BlueprintNode {
                label: "功能对比".into(),
                node_type: "topic".into(),
                tags: vec!["comparison".into()],
            },
            BlueprintNode {
                label: "用户反馈".into(),
                node_type: "topic".into(),
                tags: vec!["feedback".into()],
            },
            BlueprintNode {
                label: "决策记录".into(),
                node_type: "topic".into(),
                tags: vec!["decision".into()],
            },
        ],
        edges: vec![
            BlueprintEdge {
                from: "产品概览".into(),
                to: "竞品 A".into(),
                relation: "contains".into(),
            },
            BlueprintEdge {
                from: "产品概览".into(),
                to: "竞品 B".into(),
                relation: "contains".into(),
            },
            BlueprintEdge {
                from: "产品概览".into(),
                to: "市场趋势".into(),
                relation: "contains".into(),
            },
            BlueprintEdge {
                from: "竞品 A".into(),
                to: "功能对比".into(),
                relation: "relates_to".into(),
            },
            BlueprintEdge {
                from: "竞品 B".into(),
                to: "功能对比".into(),
                relation: "relates_to".into(),
            },
            BlueprintEdge {
                from: "市场趋势".into(),
                to: "决策记录".into(),
                relation: "influences".into(),
            },
            BlueprintEdge {
                from: "用户反馈".into(),
                to: "决策记录".into(),
                relation: "influences".into(),
            },
        ],
    }
}

fn project_management_template() -> BlueprintTemplate {
    BlueprintTemplate {
        id: "project-management".into(),
        name: "项目管理".into(),
        description: "用于项目规划和进度跟踪".into(),
        nodes: vec![
            BlueprintNode {
                label: "项目目标".into(),
                node_type: "category".into(),
                tags: vec!["goal".into()],
            },
            BlueprintNode {
                label: "需求分析".into(),
                node_type: "topic".into(),
                tags: vec!["requirement".into()],
            },
            BlueprintNode {
                label: "技术方案".into(),
                node_type: "topic".into(),
                tags: vec!["tech".into()],
            },
            BlueprintNode {
                label: "任务清单".into(),
                node_type: "topic".into(),
                tags: vec!["task".into()],
            },
            BlueprintNode {
                label: "里程碑".into(),
                node_type: "topic".into(),
                tags: vec!["milestone".into()],
            },
            BlueprintNode {
                label: "风险记录".into(),
                node_type: "topic".into(),
                tags: vec!["risk".into()],
            },
            BlueprintNode {
                label: "会议记录".into(),
                node_type: "topic".into(),
                tags: vec!["meeting".into()],
            },
        ],
        edges: vec![
            BlueprintEdge {
                from: "项目目标".into(),
                to: "需求分析".into(),
                relation: "depends_on".into(),
            },
            BlueprintEdge {
                from: "需求分析".into(),
                to: "技术方案".into(),
                relation: "derives_from".into(),
            },
            BlueprintEdge {
                from: "技术方案".into(),
                to: "任务清单".into(),
                relation: "contains".into(),
            },
            BlueprintEdge {
                from: "任务清单".into(),
                to: "里程碑".into(),
                relation: "leads_to".into(),
            },
            BlueprintEdge {
                from: "风险记录".into(),
                to: "任务清单".into(),
                relation: "affects".into(),
            },
            BlueprintEdge {
                from: "会议记录".into(),
                to: "决策记录".into(),
                relation: "documents".into(),
            },
        ],
    }
}

fn learning_notes_template() -> BlueprintTemplate {
    BlueprintTemplate {
        id: "learning-notes".into(),
        name: "学习笔记".into(),
        description: "用于知识学习和笔记整理".into(),
        nodes: vec![
            BlueprintNode {
                label: "学习主题".into(),
                node_type: "category".into(),
                tags: vec!["topic".into()],
            },
            BlueprintNode {
                label: "核心概念".into(),
                node_type: "topic".into(),
                tags: vec!["concept".into()],
            },
            BlueprintNode {
                label: "参考资料".into(),
                node_type: "topic".into(),
                tags: vec!["reference".into()],
            },
            BlueprintNode {
                label: "实践案例".into(),
                node_type: "topic".into(),
                tags: vec!["example".into()],
            },
            BlueprintNode {
                label: "问题记录".into(),
                node_type: "topic".into(),
                tags: vec!["question".into()],
            },
            BlueprintNode {
                label: "总结反思".into(),
                node_type: "topic".into(),
                tags: vec!["summary".into()],
            },
        ],
        edges: vec![
            BlueprintEdge {
                from: "学习主题".into(),
                to: "核心概念".into(),
                relation: "contains".into(),
            },
            BlueprintEdge {
                from: "核心概念".into(),
                to: "参考资料".into(),
                relation: "documented_in".into(),
            },
            BlueprintEdge {
                from: "核心概念".into(),
                to: "实践案例".into(),
                relation: "illustrated_by".into(),
            },
            BlueprintEdge {
                from: "实践案例".into(),
                to: "问题记录".into(),
                relation: "raises".into(),
            },
            BlueprintEdge {
                from: "问题记录".into(),
                to: "总结反思".into(),
                relation: "resolved_in".into(),
            },
        ],
    }
}

fn technical_docs_template() -> BlueprintTemplate {
    BlueprintTemplate {
        id: "technical-docs".into(),
        name: "技术文档".into(),
        description: "用于技术方案设计和文档编写".into(),
        nodes: vec![
            BlueprintNode {
                label: "系统架构".into(),
                node_type: "category".into(),
                tags: vec!["architecture".into()],
            },
            BlueprintNode {
                label: "接口设计".into(),
                node_type: "topic".into(),
                tags: vec!["api".into()],
            },
            BlueprintNode {
                label: "数据模型".into(),
                node_type: "topic".into(),
                tags: vec!["data".into()],
            },
            BlueprintNode {
                label: "部署方案".into(),
                node_type: "topic".into(),
                tags: vec!["deploy".into()],
            },
            BlueprintNode {
                label: "性能指标".into(),
                node_type: "topic".into(),
                tags: vec!["performance".into()],
            },
            BlueprintNode {
                label: "故障处理".into(),
                node_type: "topic".into(),
                tags: vec!["troubleshoot".into()],
            },
            BlueprintNode {
                label: "变更记录".into(),
                node_type: "topic".into(),
                tags: vec!["changelog".into()],
            },
        ],
        edges: vec![
            BlueprintEdge {
                from: "系统架构".into(),
                to: "接口设计".into(),
                relation: "contains".into(),
            },
            BlueprintEdge {
                from: "系统架构".into(),
                to: "数据模型".into(),
                relation: "contains".into(),
            },
            BlueprintEdge {
                from: "接口设计".into(),
                to: "部署方案".into(),
                relation: "requires".into(),
            },
            BlueprintEdge {
                from: "部署方案".into(),
                to: "性能指标".into(),
                relation: "measured_by".into(),
            },
            BlueprintEdge {
                from: "故障处理".into(),
                to: "变更记录".into(),
                relation: "documented_in".into(),
            },
        ],
    }
}

fn blank_template() -> BlueprintTemplate {
    BlueprintTemplate {
        id: "blank".into(),
        name: "空白".into(),
        description: "从零开始构建图谱".into(),
        nodes: vec![BlueprintNode {
            label: "中心主题".into(),
            node_type: "category".into(),
            tags: vec![],
        }],
        edges: vec![],
    }
}
