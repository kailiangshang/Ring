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
        name: "产品研究".into(),
        description: "产品分析、竞品调研、市场洞察".into(),
        nodes: vec![
            BlueprintNode {
                label: "产品概览".into(),
                node_type: "category".into(),
                tags: vec!["overview".into()],
            },
            BlueprintNode {
                label: "目标用户".into(),
                node_type: "topic".into(),
                tags: vec!["user".into()],
            },
            BlueprintNode {
                label: "核心场景".into(),
                node_type: "topic".into(),
                tags: vec!["scenario".into()],
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
                label: "功能对比".into(),
                node_type: "topic".into(),
                tags: vec!["comparison".into()],
            },
            BlueprintNode {
                label: "市场趋势".into(),
                node_type: "topic".into(),
                tags: vec!["market".into()],
            },
            BlueprintNode {
                label: "用户反馈".into(),
                node_type: "topic".into(),
                tags: vec!["feedback".into()],
            },
            BlueprintNode {
                label: "差异化策略".into(),
                node_type: "topic".into(),
                tags: vec!["strategy".into()],
            },
            BlueprintNode {
                label: "决策记录".into(),
                node_type: "leaf".into(),
                tags: vec!["decision".into()],
            },
        ],
        edges: vec![
            BlueprintEdge {
                from: "产品概览".into(),
                to: "目标用户".into(),
                relation: "related_to".into(),
            },
            BlueprintEdge {
                from: "产品概览".into(),
                to: "核心场景".into(),
                relation: "related_to".into(),
            },
            BlueprintEdge {
                from: "竞品 A".into(),
                to: "功能对比".into(),
                relation: "related_to".into(),
            },
            BlueprintEdge {
                from: "竞品 B".into(),
                to: "功能对比".into(),
                relation: "related_to".into(),
            },
            BlueprintEdge {
                from: "市场趋势".into(),
                to: "差异化策略".into(),
                relation: "derives_from".into(),
            },
            BlueprintEdge {
                from: "用户反馈".into(),
                to: "差异化策略".into(),
                relation: "derives_from".into(),
            },
            BlueprintEdge {
                from: "功能对比".into(),
                to: "差异化策略".into(),
                relation: "derives_from".into(),
            },
            BlueprintEdge {
                from: "差异化策略".into(),
                to: "决策记录".into(),
                relation: "related_to".into(),
            },
        ],
    }
}

fn project_management_template() -> BlueprintTemplate {
    BlueprintTemplate {
        id: "project-management".into(),
        name: "项目管理".into(),
        description: "项目规划、任务跟踪、风险管理".into(),
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
                label: "任务拆解".into(),
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
            BlueprintNode {
                label: "决策记录".into(),
                node_type: "topic".into(),
                tags: vec!["decision".into()],
            },
            BlueprintNode {
                label: "进度更新".into(),
                node_type: "leaf".into(),
                tags: vec!["progress".into()],
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
                to: "任务拆解".into(),
                relation: "derives_from".into(),
            },
            BlueprintEdge {
                from: "任务拆解".into(),
                to: "里程碑".into(),
                relation: "related_to".into(),
            },
            BlueprintEdge {
                from: "风险记录".into(),
                to: "决策记录".into(),
                relation: "related_to".into(),
            },
            BlueprintEdge {
                from: "会议记录".into(),
                to: "决策记录".into(),
                relation: "related_to".into(),
            },
            BlueprintEdge {
                from: "里程碑".into(),
                to: "进度更新".into(),
                relation: "related_to".into(),
            },
        ],
    }
}

fn learning_notes_template() -> BlueprintTemplate {
    BlueprintTemplate {
        id: "learning-notes".into(),
        name: "学习笔记".into(),
        description: "知识学习、概念梳理、实践总结".into(),
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
                label: "前置知识".into(),
                node_type: "topic".into(),
                tags: vec!["prerequisite".into()],
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
                label: "常见问题".into(),
                node_type: "topic".into(),
                tags: vec!["question".into()],
            },
            BlueprintNode {
                label: "总结反思".into(),
                node_type: "leaf".into(),
                tags: vec!["summary".into()],
            },
            BlueprintNode {
                label: "延伸学习".into(),
                node_type: "leaf".into(),
                tags: vec!["extension".into()],
            },
        ],
        edges: vec![
            BlueprintEdge {
                from: "学习主题".into(),
                to: "核心概念".into(),
                relation: "related_to".into(),
            },
            BlueprintEdge {
                from: "前置知识".into(),
                to: "核心概念".into(),
                relation: "depends_on".into(),
            },
            BlueprintEdge {
                from: "核心概念".into(),
                to: "参考资料".into(),
                relation: "related_to".into(),
            },
            BlueprintEdge {
                from: "核心概念".into(),
                to: "实践案例".into(),
                relation: "related_to".into(),
            },
            BlueprintEdge {
                from: "实践案例".into(),
                to: "常见问题".into(),
                relation: "related_to".into(),
            },
            BlueprintEdge {
                from: "常见问题".into(),
                to: "总结反思".into(),
                relation: "related_to".into(),
            },
            BlueprintEdge {
                from: "总结反思".into(),
                to: "延伸学习".into(),
                relation: "related_to".into(),
            },
        ],
    }
}

fn technical_docs_template() -> BlueprintTemplate {
    BlueprintTemplate {
        id: "technical-docs".into(),
        name: "技术文档".into(),
        description: "技术方案、架构设计、运维手册".into(),
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
                label: "安全设计".into(),
                node_type: "topic".into(),
                tags: vec!["security".into()],
            },
            BlueprintNode {
                label: "性能指标".into(),
                node_type: "topic".into(),
                tags: vec!["performance".into()],
            },
            BlueprintNode {
                label: "监控告警".into(),
                node_type: "topic".into(),
                tags: vec!["monitoring".into()],
            },
            BlueprintNode {
                label: "故障手册".into(),
                node_type: "leaf".into(),
                tags: vec!["troubleshoot".into()],
            },
            BlueprintNode {
                label: "变更记录".into(),
                node_type: "leaf".into(),
                tags: vec!["changelog".into()],
            },
        ],
        edges: vec![
            BlueprintEdge {
                from: "系统架构".into(),
                to: "接口设计".into(),
                relation: "related_to".into(),
            },
            BlueprintEdge {
                from: "系统架构".into(),
                to: "数据模型".into(),
                relation: "related_to".into(),
            },
            BlueprintEdge {
                from: "系统架构".into(),
                to: "安全设计".into(),
                relation: "related_to".into(),
            },
            BlueprintEdge {
                from: "接口设计".into(),
                to: "部署方案".into(),
                relation: "depends_on".into(),
            },
            BlueprintEdge {
                from: "部署方案".into(),
                to: "监控告警".into(),
                relation: "related_to".into(),
            },
            BlueprintEdge {
                from: "部署方案".into(),
                to: "性能指标".into(),
                relation: "related_to".into(),
            },
            BlueprintEdge {
                from: "监控告警".into(),
                to: "故障手册".into(),
                relation: "derives_from".into(),
            },
            BlueprintEdge {
                from: "故障手册".into(),
                to: "变更记录".into(),
                relation: "related_to".into(),
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
