use crate::error::Result;
use crate::models::graph::{CreateEdgeInput, CreateNodeInput, UpdateNodeInput};
use crate::models::user::UserRow;
use crate::services::graph;
use crate::services::llm::LlmClient;
use crate::state::AppState;
use serde::Deserialize;

const GRAPH_COMMAND_PROMPT: &str = r#"分析以下用户消息，判断是否是图谱操作指令。如果是，输出 JSON 格式的操作：

可能的操作：
- create_node: {"action": "create_node", "label": "节点名称", "node_type": "topic|concept|task|note", "parent_id": "父节点ID或null", "content": "内容"}
- update_node: {"action": "update_node", "node_id": "节点ID", "label": "新名称或null", "content": "新内容或null", "tags": ["标签"]}
- delete_node: {"action": "delete_node", "node_id": "节点ID"}
- create_edge: {"action": "create_edge", "source_id": "源节点ID", "target_id": "目标节点ID", "relation": "关系类型", "label": "标签"}
- delete_edge: {"action": "delete_edge", "edge_id": "边ID"}

如果不是图谱操作指令，输出：{"action": "none"}

只输出 JSON，不要其他内容。

用户消息：
"#;

#[derive(Debug, Deserialize)]
struct GraphCommand {
    action: String,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    node_type: Option<String>,
    #[serde(default)]
    parent_id: Option<String>,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    node_id: Option<String>,
    #[serde(default)]
    source_id: Option<String>,
    #[serde(default)]
    target_id: Option<String>,
    #[serde(default)]
    relation: Option<String>,
    #[serde(default)]
    edge_id: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
}

pub async fn try_handle_graph_command(
    state: &AppState,
    ring_id: &str,
    user: &UserRow,
    message: &str,
) -> Result<Option<String>> {
    let prompt = format!("{}\n{}", GRAPH_COMMAND_PROMPT, message);

    let llm = LlmClient::from_user(user)?;
    let response = llm
        .chat_complete("你是一个图谱命令解析器。".into(), prompt)
        .await?;

    let cmd: GraphCommand = match serde_json::from_str(&response) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };

    let result = match cmd.action.as_str() {
        "create_node" => {
            let input = CreateNodeInput {
                label: cmd.label.unwrap_or_else(|| "New Node".into()),
                graph_id: None,
                parent_id: cmd.parent_id,
                node_type: cmd.node_type.unwrap_or_else(|| "topic".into()),
                tags: cmd.tags.unwrap_or_default(),
                content: cmd.content.unwrap_or_default(),
                markdown_path: None,
                metadata: serde_json::json!({}),
            };
            match graph::create_node(state, ring_id, &input).await {
                Ok(node) => format!("已创建节点: {} ({})", node.label, node.id),
                Err(e) => format!("创建节点失败: {}", e),
            }
        }
        "update_node" => {
            let node_id = match cmd.node_id {
                Some(id) => id,
                None => return Ok(Some("需要指定节点 ID".into())),
            };
            let input = UpdateNodeInput {
                label: cmd.label,
                tags: cmd.tags,
                content: cmd.content,
                markdown_path: None,
                metadata: None,
            };
            match graph::update_node(state, &node_id, &input).await {
                Ok(node) => format!("已更新节点: {} ({})", node.label, node.id),
                Err(e) => format!("更新节点失败: {}", e),
            }
        }
        "delete_node" => {
            let node_id = match cmd.node_id {
                Some(id) => id,
                None => return Ok(Some("需要指定节点 ID".into())),
            };
            match graph::delete_node(state, &node_id).await {
                Ok(_) => "已删除节点".into(),
                Err(e) => format!("删除节点失败: {}", e),
            }
        }
        "create_edge" => {
            let input = CreateEdgeInput {
                graph_id: None,
                source_id: cmd.source_id.unwrap_or_default(),
                target_id: cmd.target_id.unwrap_or_default(),
                relation: cmd.relation.unwrap_or_else(|| "related_to".into()),
                label: cmd.label.unwrap_or_default(),
            };
            match graph::create_edge(state, ring_id, &input).await {
                Ok(edge) => format!(
                    "已创建边: {} -> {} ({})",
                    edge.source_id, edge.target_id, edge.id
                ),
                Err(e) => format!("创建边失败: {}", e),
            }
        }
        "delete_edge" => {
            let edge_id = match cmd.edge_id {
                Some(id) => id,
                None => return Ok(Some("需要指定边 ID".into())),
            };
            match graph::delete_edge(state, &edge_id).await {
                Ok(_) => "已删除边".into(),
                Err(e) => format!("删除边失败: {}", e),
            }
        }
        _ => return Ok(None),
    };

    Ok(Some(result))
}
