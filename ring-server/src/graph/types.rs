use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub parent_id: Option<String>,
    pub description: Option<String>,
    pub graph_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeData {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation: String,
    pub label: Option<String>,
    pub graph_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewNode {
    pub label: String,
    pub node_type: String,
    pub parent_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEdge {
    pub source_id: String,
    pub target_id: String,
    pub relation: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphJson {
    pub nodes: Vec<NodeData>,
    pub edges: Vec<EdgeData>,
}
