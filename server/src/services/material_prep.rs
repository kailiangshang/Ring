use crate::error::Result;
use crate::models::graph;
use crate::models::session;
use crate::models::user::UserRow;
use crate::services::llm::LlmClient;
use crate::state::AppState;

const MATERIAL_PREP_PROMPT: &str = r#"你正在为一场会议准备材料。请根据以下会议主题、Skill 类型和群组上下文，生成 3-5 条会议准备材料。

每条材料应包含：
- 类型（context / question / data / reference）
- 标题
- 具体内容

请以 JSON 数组格式输出，每条材料格式为：
{"item_type": "类型", "title": "标题", "content": "内容"}

会议信息：
"#;

pub async fn generate_materials(
    state: &AppState,
    session_id: &str,
    ring_id: &str,
    skill: &str,
    title: &str,
    description: &str,
    user: &UserRow,
) -> Result<()> {
    let graph = graph::ensure_default_graph(&state.db, ring_id).await?;
    let nodes = graph::list_nodes(&state.db, &graph.id).await?;

    let context = nodes
        .iter()
        .map(|n| format!("- {} ({}): {}", n.label, n.node_type, n.content))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "{}\nSkill: {}\n标题: {}\n描述: {}\n\n群组图谱上下文:\n{}",
        MATERIAL_PREP_PROMPT, skill, title, description, context
    );

    let llm = LlmClient::from_user(user)?;
    let response = llm
        .chat_complete(
            "你是一个会议材料准备助手。".into(),
            prompt,
        )
        .await?;

    let materials: Vec<serde_json::Value> =
        serde_json::from_str(&response).unwrap_or_else(|_| {
            parse_materials_from_text(&response, skill, title)
        });

    for material in materials.iter().take(5) {
        let item_type = material
            .get("item_type")
            .and_then(|v| v.as_str())
            .unwrap_or("context");
        let title = material
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled");
        let content = material
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let id = ulid::Ulid::new().to_string();
        let _ = session::create_material(
            &state.db,
            &id,
            session_id,
            item_type,
            title,
            content,
        )
        .await;
    }

    Ok(())
}

fn parse_materials_from_text(_text: &str, skill: &str, session_title: &str) -> Vec<serde_json::Value> {
    let mut materials = Vec::new();

    let skill_defaults: Vec<(&str, &str, &str)> = match skill {
        "decision" => vec![
            ("context", "背景信息", "收集与决策相关的背景资料"),
            ("question", "关键问题", "明确需要讨论和决策的核心问题"),
            ("data", "备选方案", "列出可能的解决方案及其优缺点"),
            ("reference", "决策标准", "定义评估和选择方案的标准"),
        ],
        "research" => vec![
            ("context", "研究背景", "阐述研究主题的背景和意义"),
            ("question", "研究问题", "明确需要解答的研究问题"),
            ("data", "现有资料", "汇总已有的相关研究和数据来源"),
            ("reference", "研究方法", "建议采用的研究方法和工具"),
        ],
        "review" => vec![
            ("context", "审查范围", "明确审查的对象和范围"),
            ("question", "审查标准", "列出审查的具体标准和检查项"),
            ("data", "现有状态", "描述当前的状态和已知问题"),
            ("reference", "参考规范", "引用相关的规范和最佳实践"),
        ],
        "retrospective" => vec![
            ("context", "迭代回顾", "回顾本周期的工作内容和目标"),
            ("question", "成功经验", "总结本周期做得好的方面"),
            ("data", "改进机会", "识别需要改进的问题和挑战"),
            ("reference", "行动项", "提出下一步的改进措施"),
        ],
        "knowledge_sharing" => vec![
            ("context", "知识背景", "介绍分享主题的背景信息"),
            ("question", "核心概念", "列出需要讲解的关键概念"),
            ("data", "案例素材", "准备相关的实例和案例"),
            ("reference", "延伸阅读", "提供进一步学习的参考资料"),
        ],
        _ => vec![
            ("context", "主题背景", "收集与主题相关的背景信息"),
            ("question", "讨论要点", "列出需要讨论的关键问题"),
            ("data", "相关资料", "汇总已有的相关资料和数据"),
        ],
    };

    for (item_type, default_title, default_content) in skill_defaults {
        materials.push(serde_json::json!({
            "item_type": item_type,
            "title": format!("{} - {}", session_title, default_title),
            "content": default_content,
        }));
    }

    materials
}
