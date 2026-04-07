use crate::error::Result;
use crate::graph::types::{EdgeData, GraphJson, NewEdge, NewNode, NodeData};

#[async_trait::async_trait]
pub trait GraphStore: Send + Sync {
    async fn create_node(&self, graph_id: &str, input: NewNode) -> Result<NodeData>;
    async fn get_node(&self, graph_id: &str, node_id: &str) -> Result<Option<NodeData>>;
    async fn update_node(
        &self,
        graph_id: &str,
        node_id: &str,
        label: Option<String>,
        description: Option<String>,
        node_type: Option<String>,
    ) -> Result<NodeData>;
    async fn delete_node(&self, graph_id: &str, node_id: &str) -> Result<()>;
    async fn create_edge(&self, graph_id: &str, input: NewEdge) -> Result<EdgeData>;
    async fn delete_edge(&self, graph_id: &str, edge_id: &str) -> Result<()>;
    async fn get_children(&self, graph_id: &str, parent_id: &str) -> Result<Vec<NodeData>>;
    async fn list_graph_ids(&self) -> Vec<String>;
    async fn export_graph_json(&self, graph_id: &str) -> Result<GraphJson>;
    async fn import_graph_json(&self, graph_id: &str, data: &GraphJson) -> Result<()>;
}
