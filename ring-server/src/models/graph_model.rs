use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNodeRequest {
    pub label: String,
    pub node_type: String,
    pub parent_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNodeRequest {
    pub label: Option<String>,
    pub description: Option<String>,
    pub node_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEdgeRequest {
    pub source_id: String,
    pub target_id: String,
    pub relation: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResponse {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub parent_id: Option<String>,
    pub description: Option<String>,
    pub graph_id: String,
    pub markdown_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::graph::types::NodeData> for NodeResponse {
    fn from(n: crate::graph::types::NodeData) -> Self {
        NodeResponse {
            id: n.id,
            label: n.label,
            node_type: n.node_type,
            parent_id: n.parent_id,
            description: n.description,
            graph_id: n.graph_id,
            markdown_path: None,
            created_at: n.created_at,
            updated_at: n.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeResponse {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation: String,
    pub label: Option<String>,
    pub graph_id: String,
}

impl From<crate::graph::types::EdgeData> for EdgeResponse {
    fn from(e: crate::graph::types::EdgeData) -> Self {
        EdgeResponse {
            id: e.id,
            source_id: e.source_id,
            target_id: e.target_id,
            relation: e.relation,
            label: e.label,
            graph_id: e.graph_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDetailResponse {
    pub graph_id: String,
    pub nodes: Vec<NodeResponse>,
    pub edges: Vec<EdgeResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeContentResponse {
    pub node_id: String,
    pub label: String,
    pub markdown_path: Option<String>,
    pub content: Option<String>,
    pub last_modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub node_id: String,
    pub graph_id: String,
    pub label: String,
    pub snippet: String,
    pub rank: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub graph_ids: Option<Vec<String>>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: usize,
}
