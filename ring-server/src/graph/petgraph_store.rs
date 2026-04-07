use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;
use petgraph::visit::EdgeRef;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::store_trait::GraphStore;
use super::types::{EdgeData, GraphJson, NewEdge, NewNode, NodeData};
use crate::error::{Result, RingError};

pub struct PetgraphStore {
    inner: Arc<RwLock<GraphInner>>,
}

struct GraphInner {
    graph: StableDiGraph<NodeData, EdgeData>,
    node_id_to_index: HashMap<String, NodeIndex>,
    graph_id_to_nodes: HashMap<String, Vec<NodeIndex>>,
}

impl Default for PetgraphStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PetgraphStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(GraphInner {
                graph: StableDiGraph::new(),
                node_id_to_index: HashMap::new(),
                graph_id_to_nodes: HashMap::new(),
            })),
        }
    }

    pub async fn create_node(&self, graph_id: &str, input: NewNode) -> Result<NodeData> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let node = NodeData {
            id: id.clone(),
            label: input.label,
            node_type: input.node_type,
            parent_id: input.parent_id,
            description: input.description,
            graph_id: graph_id.to_string(),
            markdown_path: Some(format!("nodes/{id}.md")),
            created_at: now.clone(),
            updated_at: now,
        };

        let mut inner = self.inner.write().await;
        let idx = inner.graph.add_node(node.clone());
        inner.node_id_to_index.insert(node.id.clone(), idx);
        inner
            .graph_id_to_nodes
            .entry(graph_id.to_string())
            .or_default()
            .push(idx);

        Ok(node)
    }

    pub async fn get_node(&self, graph_id: &str, node_id: &str) -> Result<Option<NodeData>> {
        let inner = self.inner.read().await;
        match inner.node_id_to_index.get(node_id) {
            Some(idx) => {
                let node = &inner.graph[*idx];
                if node.graph_id == graph_id {
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    pub async fn update_node(
        &self,
        graph_id: &str,
        node_id: &str,
        label: Option<String>,
        description: Option<String>,
        node_type: Option<String>,
    ) -> Result<NodeData> {
        let mut inner = self.inner.write().await;
        let idx = inner
            .node_id_to_index
            .get(node_id)
            .copied()
            .ok_or_else(|| RingError::NotFound(format!("node {} not found", node_id)))?;

        let node = &mut inner.graph[idx];
        if node.graph_id != graph_id {
            return Err(RingError::NotFound(format!(
                "node {} not found in graph {}",
                node_id, graph_id
            )));
        }

        if let Some(l) = label {
            node.label = l;
        }
        if let Some(d) = description {
            node.description = Some(d);
        }
        if let Some(t) = node_type {
            node.node_type = t;
        }
        node.updated_at = Utc::now().to_rfc3339();

        Ok(node.clone())
    }

    pub async fn delete_node(&self, graph_id: &str, node_id: &str) -> Result<()> {
        let mut inner = self.inner.write().await;

        if !inner.node_id_to_index.contains_key(node_id) {
            return Err(RingError::NotFound(format!("node {} not found", node_id)));
        }
        {
            let idx = inner.node_id_to_index[node_id];
            let node = &inner.graph[idx];
            if node.graph_id != graph_id {
                return Err(RingError::NotFound(format!(
                    "node {} not found in graph {}",
                    node_id, graph_id
                )));
            }
        }

        let mut all_ids = vec![node_id.to_string()];
        let mut i = 0;
        while i < all_ids.len() {
            let current = &all_ids[i];
            let children: Vec<String> = inner
                .graph
                .node_indices()
                .filter(|idx| {
                    let n = &inner.graph[*idx];
                    n.graph_id == graph_id && n.parent_id.as_deref() == Some(current)
                })
                .map(|idx| inner.graph[idx].id.clone())
                .collect();
            all_ids.extend(children);
            i += 1;
        }

        for del_id in all_ids.iter().rev() {
            if let Some(idx) = inner.node_id_to_index.remove(del_id) {
                inner
                    .graph
                    .edges(idx)
                    .map(|e| e.id())
                    .collect::<Vec<_>>()
                    .into_iter()
                    .for_each(|e| {
                        inner.graph.remove_edge(e);
                    });
                inner.graph.remove_node(idx);
                if let Some(nodes) = inner.graph_id_to_nodes.get_mut(graph_id) {
                    nodes.retain(|n| *n != idx);
                }
            }
        }

        Ok(())
    }

    pub async fn create_edge(&self, graph_id: &str, input: NewEdge) -> Result<EdgeData> {
        let edge = EdgeData {
            id: Uuid::new_v4().to_string(),
            source_id: input.source_id,
            target_id: input.target_id,
            relation: input.relation,
            label: input.label,
            graph_id: graph_id.to_string(),
        };

        let mut inner = self.inner.write().await;
        let source_idx = inner
            .node_id_to_index
            .get(&edge.source_id)
            .copied()
            .ok_or_else(|| RingError::NotFound(format!("node {} not found", edge.source_id)))?;
        let target_idx = inner
            .node_id_to_index
            .get(&edge.target_id)
            .copied()
            .ok_or_else(|| RingError::NotFound(format!("node {} not found", edge.target_id)))?;

        inner.graph.add_edge(source_idx, target_idx, edge.clone());

        Ok(edge)
    }

    pub async fn delete_edge(&self, graph_id: &str, edge_id: &str) -> Result<()> {
        let mut inner = self.inner.write().await;
        let edge_idx = inner
            .graph
            .edge_indices()
            .find(|e| match inner.graph.edge_weight(*e) {
                Some(w) => w.id == edge_id && w.graph_id == graph_id,
                None => false,
            })
            .ok_or_else(|| RingError::NotFound(format!("edge {} not found", edge_id)))?;

        inner.graph.remove_edge(edge_idx);
        Ok(())
    }

    pub async fn get_children(&self, graph_id: &str, parent_id: &str) -> Result<Vec<NodeData>> {
        let inner = self.inner.read().await;
        let children: Vec<NodeData> = inner
            .graph
            .node_indices()
            .filter_map(|i| {
                let n = &inner.graph[i];
                if n.graph_id == graph_id && n.parent_id.as_deref() == Some(parent_id) {
                    Some(n.clone())
                } else {
                    None
                }
            })
            .collect();
        Ok(children)
    }

    pub async fn list_graph_ids(&self) -> Vec<String> {
        let inner = self.inner.read().await;
        inner.graph_id_to_nodes.keys().cloned().collect()
    }

    pub async fn export_graph_json(&self, graph_id: &str) -> Result<GraphJson> {
        let inner = self.inner.read().await;
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        for i in inner.graph.node_indices() {
            let n = &inner.graph[i];
            if n.graph_id == graph_id {
                nodes.push(n.clone());
            }
        }

        for e in inner.graph.edge_indices() {
            if let Some(w) = inner.graph.edge_weight(e) {
                if w.graph_id == graph_id {
                    edges.push(w.clone());
                }
            }
        }

        Ok(GraphJson { nodes, edges })
    }

    pub async fn import_graph_json(&self, graph_id: &str, data: &GraphJson) -> Result<()> {
        let mut inner = self.inner.write().await;

        let old_indices: Vec<NodeIndex> = inner
            .graph
            .node_indices()
            .filter(|i| inner.graph[*i].graph_id == graph_id)
            .collect();

        for idx in &old_indices {
            let id = inner.graph[*idx].id.clone();
            inner.node_id_to_index.remove(&id);
        }

        for idx in old_indices.iter().rev() {
            inner
                .graph
                .edges(*idx)
                .map(|e| e.id())
                .collect::<Vec<_>>()
                .into_iter()
                .for_each(|e| {
                    inner.graph.remove_edge(e);
                });
            inner.graph.remove_node(*idx);
        }

        inner.graph_id_to_nodes.remove(graph_id);

        let mut new_node_map: HashMap<String, NodeIndex> = HashMap::new();
        for mut node in data.nodes.clone() {
            node.graph_id = graph_id.to_string();
            let idx = inner.graph.add_node(node.clone());
            inner.node_id_to_index.insert(node.id.clone(), idx);
            inner
                .graph_id_to_nodes
                .entry(graph_id.to_string())
                .or_default()
                .push(idx);
            new_node_map.insert(node.id.clone(), idx);
        }

        for mut edge in data.edges.clone() {
            edge.graph_id = graph_id.to_string();
            if let (Some(&src), Some(&tgt)) = (
                new_node_map.get(&edge.source_id),
                new_node_map.get(&edge.target_id),
            ) {
                inner.graph.add_edge(src, tgt, edge);
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl GraphStore for PetgraphStore {
    async fn create_node(&self, graph_id: &str, input: NewNode) -> Result<NodeData> {
        PetgraphStore::create_node(self, graph_id, input).await
    }

    async fn get_node(&self, graph_id: &str, node_id: &str) -> Result<Option<NodeData>> {
        PetgraphStore::get_node(self, graph_id, node_id).await
    }

    async fn update_node(
        &self,
        graph_id: &str,
        node_id: &str,
        label: Option<String>,
        description: Option<String>,
        node_type: Option<String>,
    ) -> Result<NodeData> {
        PetgraphStore::update_node(self, graph_id, node_id, label, description, node_type).await
    }

    async fn delete_node(&self, graph_id: &str, node_id: &str) -> Result<()> {
        PetgraphStore::delete_node(self, graph_id, node_id).await
    }

    async fn create_edge(&self, graph_id: &str, input: NewEdge) -> Result<EdgeData> {
        PetgraphStore::create_edge(self, graph_id, input).await
    }

    async fn delete_edge(&self, graph_id: &str, edge_id: &str) -> Result<()> {
        PetgraphStore::delete_edge(self, graph_id, edge_id).await
    }

    async fn get_children(&self, graph_id: &str, parent_id: &str) -> Result<Vec<NodeData>> {
        PetgraphStore::get_children(self, graph_id, parent_id).await
    }

    async fn list_graph_ids(&self) -> Vec<String> {
        PetgraphStore::list_graph_ids(self).await
    }

    async fn export_graph_json(&self, graph_id: &str) -> Result<GraphJson> {
        PetgraphStore::export_graph_json(self, graph_id).await
    }

    async fn import_graph_json(&self, graph_id: &str, data: &GraphJson) -> Result<()> {
        PetgraphStore::import_graph_json(self, graph_id, data).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_store() -> PetgraphStore {
        PetgraphStore::new()
    }

    #[tokio::test]
    async fn create_and_get_node() {
        let store = new_store();
        let node = store
            .create_node(
                "graph-1",
                NewNode {
                    label: "竞品A".into(),
                    node_type: "concept".into(),
                    parent_id: None,
                    description: Some("分析".into()),
                },
            )
            .await
            .unwrap();
        assert_eq!(node.label, "竞品A");

        let fetched = store.get_node("graph-1", &node.id).await.unwrap().unwrap();
        assert_eq!(fetched.id, node.id);
    }

    #[tokio::test]
    async fn create_edge() {
        let store = new_store();
        let n1 = store
            .create_node(
                "graph-1",
                NewNode {
                    label: "A".into(),
                    node_type: "concept".into(),
                    parent_id: None,
                    description: None,
                },
            )
            .await
            .unwrap();
        let n2 = store
            .create_node(
                "graph-1",
                NewNode {
                    label: "B".into(),
                    node_type: "concept".into(),
                    parent_id: None,
                    description: None,
                },
            )
            .await
            .unwrap();
        let edge = store
            .create_edge(
                "graph-1",
                NewEdge {
                    source_id: n1.id.clone(),
                    target_id: n2.id.clone(),
                    relation: "depends_on".into(),
                    label: Some("依赖".into()),
                },
            )
            .await
            .unwrap();
        assert_eq!(edge.source_id, n1.id);
    }

    #[tokio::test]
    async fn delete_node_cascades_to_children() {
        let store = new_store();
        let parent = store
            .create_node(
                "graph-1",
                NewNode {
                    label: "P".into(),
                    node_type: "category".into(),
                    parent_id: None,
                    description: None,
                },
            )
            .await
            .unwrap();
        let child = store
            .create_node(
                "graph-1",
                NewNode {
                    label: "C".into(),
                    node_type: "concept".into(),
                    parent_id: Some(parent.id.clone()),
                    description: None,
                },
            )
            .await
            .unwrap();
        store.delete_node("graph-1", &parent.id).await.unwrap();
        assert!(store
            .get_node("graph-1", &child.id)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn export_and_import_graph_json() {
        let store = new_store();
        let n = store
            .create_node(
                "graph-1",
                NewNode {
                    label: "X".into(),
                    node_type: "concept".into(),
                    parent_id: None,
                    description: None,
                },
            )
            .await
            .unwrap();
        let exported = store.export_graph_json("graph-1").await.unwrap();

        let store2 = new_store();
        store2
            .import_graph_json("graph-1", &exported)
            .await
            .unwrap();
        let fetched = store2.get_node("graph-1", &n.id).await.unwrap().unwrap();
        assert_eq!(fetched.label, "X");
    }

    #[tokio::test]
    async fn get_children_returns_child_nodes() {
        let store = new_store();
        let parent = store
            .create_node(
                "graph-1",
                NewNode {
                    label: "Parent".into(),
                    node_type: "category".into(),
                    parent_id: None,
                    description: None,
                },
            )
            .await
            .unwrap();
        let child1 = store
            .create_node(
                "graph-1",
                NewNode {
                    label: "C1".into(),
                    node_type: "concept".into(),
                    parent_id: Some(parent.id.clone()),
                    description: None,
                },
            )
            .await
            .unwrap();
        let _child2 = store
            .create_node(
                "graph-1",
                NewNode {
                    label: "C2".into(),
                    node_type: "concept".into(),
                    parent_id: Some(parent.id.clone()),
                    description: None,
                },
            )
            .await
            .unwrap();

        let children = store.get_children("graph-1", &parent.id).await.unwrap();
        assert_eq!(children.len(), 2);
        let ids: Vec<&str> = children.iter().map(|c| c.id.as_str()).collect();
        assert!(ids.contains(&child1.id.as_str()));
    }

    #[tokio::test]
    async fn delete_edge_removes_it() {
        let store = new_store();
        let n1 = store
            .create_node(
                "graph-1",
                NewNode {
                    label: "A".into(),
                    node_type: "concept".into(),
                    parent_id: None,
                    description: None,
                },
            )
            .await
            .unwrap();
        let n2 = store
            .create_node(
                "graph-1",
                NewNode {
                    label: "B".into(),
                    node_type: "concept".into(),
                    parent_id: None,
                    description: None,
                },
            )
            .await
            .unwrap();
        let edge = store
            .create_edge(
                "graph-1",
                NewEdge {
                    source_id: n1.id.clone(),
                    target_id: n2.id.clone(),
                    relation: "related_to".into(),
                    label: None,
                },
            )
            .await
            .unwrap();

        store.delete_edge("graph-1", &edge.id).await.unwrap();

        let exported = store.export_graph_json("graph-1").await.unwrap();
        assert!(exported.edges.is_empty());
    }

    #[tokio::test]
    async fn get_node_nonexistent_returns_none() {
        let store = new_store();
        let result = store.get_node("graph-1", "fake-id").await.unwrap();
        assert!(result.is_none());
    }
}
