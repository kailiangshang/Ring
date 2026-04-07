use std::path::PathBuf;
use std::sync::Arc;

use crate::error::{Result, RingError};
use crate::graph::store_trait::GraphStore;
use crate::graph::types::{EdgeData, NewNode, NodeData};
use crate::models::graph_model::{CreateNodeRequest, NodeContentResponse, UpdateNodeRequest};
use crate::services::search_service::SearchService;

pub struct GraphService {
    store: Arc<dyn GraphStore>,
    search_service: Arc<SearchService>,
    data_dir: PathBuf,
}

impl GraphService {
    pub fn new(
        store: Arc<dyn GraphStore>,
        search_service: Arc<SearchService>,
        data_dir: PathBuf,
    ) -> Self {
        GraphService {
            store,
            search_service,
            data_dir,
        }
    }

    fn repo_dir(&self, graph_id: &str) -> PathBuf {
        self.data_dir.join("repos").join(graph_id)
    }

    fn nodes_dir(&self, graph_id: &str) -> PathBuf {
        self.repo_dir(graph_id).join("nodes")
    }

    fn node_md_path(&self, graph_id: &str, node_id: &str) -> PathBuf {
        self.nodes_dir(graph_id).join(format!("{node_id}.md"))
    }

    fn write_node_markdown(&self, graph_id: &str, node: &NodeData) -> Result<()> {
        let dir = self.nodes_dir(graph_id);
        std::fs::create_dir_all(&dir)?;
        let path = self.node_md_path(graph_id, &node.id);
        let labels = [node.label.as_str()];
        let labels_str = labels
            .iter()
            .map(|l| format!("\"{}\"", l.replace('"', "\\\"")))
            .collect::<Vec<_>>()
            .join(", ");
        let content = format!(
            "---\nnode_id: {}\ntype: {}\nlabels: [{}]\ncreated_at: {}\nupdated_at: {}\n---\n\n{}",
            node.id,
            node.node_type,
            labels_str,
            node.created_at,
            node.updated_at,
            node.description.as_deref().unwrap_or("")
        );
        std::fs::write(&path, content)?;
        Ok(())
    }

    fn delete_node_markdown(&self, graph_id: &str, node_id: &str) -> Result<()> {
        let path = self.node_md_path(graph_id, node_id);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    async fn persist_graph(&self, graph_id: &str) {
        let result: Result<()> = async {
            let data = self.store.export_graph_json(graph_id).await?;
            let json = serde_json::to_string_pretty(&data)?;
            let repo_dir = self.repo_dir(graph_id);
            std::fs::create_dir_all(&repo_dir)?;
            let graph_json_path = repo_dir.join("graph.json");
            let tmp_path = repo_dir.join("graph.json.tmp");
            std::fs::write(&tmp_path, &json)?;
            std::fs::rename(&tmp_path, &graph_json_path)?;
            Ok(())
        }
        .await;

        if let Err(e) = result {
            tracing::warn!("failed to persist graph {}: {}", graph_id, e);
        }
    }

    async fn collect_descendant_ids(&self, graph_id: &str, node_id: &str) -> Vec<String> {
        let graph_data = match self.store.export_graph_json(graph_id).await {
            Ok(d) => d,
            Err(_) => return vec![node_id.to_string()],
        };
        let mut ids = vec![node_id.to_string()];
        let mut i = 0;
        while i < ids.len() {
            let current = ids[i].clone();
            let mut new_ids = Vec::new();
            for n in &graph_data.nodes {
                if n.graph_id == graph_id
                    && n.parent_id.as_deref() == Some(current.as_str())
                    && !ids.contains(&n.id)
                {
                    new_ids.push(n.id.clone());
                }
            }
            ids.extend(new_ids);
            i += 1;
        }
        ids
    }

    pub async fn create_node(&self, graph_id: &str, req: CreateNodeRequest) -> Result<NodeData> {
        if req.label.trim().is_empty() {
            return Err(RingError::Validation("label must not be empty".into()));
        }
        let input = NewNode {
            label: req.label,
            node_type: req.node_type,
            parent_id: req.parent_id,
            description: req.description,
        };
        let node = self.store.create_node(graph_id, input).await?;

        self.write_node_markdown(graph_id, &node)?;

        let _ = self
            .search_service
            .index_node(
                &node.id,
                graph_id,
                &node.label,
                node.description.as_deref().unwrap_or(""),
            )
            .await;

        self.persist_graph(graph_id).await;

        Ok(node)
    }

    pub async fn get_node(&self, graph_id: &str, node_id: &str) -> Result<Option<NodeData>> {
        self.store.get_node(graph_id, node_id).await
    }

    pub async fn update_node(
        &self,
        graph_id: &str,
        node_id: &str,
        req: UpdateNodeRequest,
    ) -> Result<NodeData> {
        if req.label.is_none() && req.description.is_none() && req.node_type.is_none() {
            return Err(RingError::Validation(
                "at least one field must be provided".into(),
            ));
        }

        let _old_node = self
            .store
            .get_node(graph_id, node_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("node {} not found", node_id)))?;

        let _ = self.search_service.delete_node_index(node_id).await;

        let node = self
            .store
            .update_node(graph_id, node_id, req.label, req.description, req.node_type)
            .await?;

        self.write_node_markdown(graph_id, &node)?;

        let _ = self
            .search_service
            .index_node(
                &node.id,
                graph_id,
                &node.label,
                node.description.as_deref().unwrap_or(""),
            )
            .await;

        self.persist_graph(graph_id).await;

        Ok(node)
    }

    pub async fn delete_node(&self, graph_id: &str, node_id: &str) -> Result<()> {
        let descendant_ids = self.collect_descendant_ids(graph_id, node_id).await;

        self.store.delete_node(graph_id, node_id).await?;

        for id in &descendant_ids {
            let _ = self.delete_node_markdown(graph_id, id);
            let _ = self.search_service.delete_node_index(id).await;
        }

        self.persist_graph(graph_id).await;

        Ok(())
    }

    pub async fn get_children(&self, graph_id: &str, parent_id: &str) -> Result<Vec<NodeData>> {
        self.store.get_children(graph_id, parent_id).await
    }

    pub async fn get_root_nodes(&self, graph_id: &str) -> Result<Vec<NodeData>> {
        let graph_data = self.store.export_graph_json(graph_id).await?;
        let roots: Vec<NodeData> = graph_data
            .nodes
            .into_iter()
            .filter(|n| n.parent_id.is_none())
            .collect();
        Ok(roots)
    }

    pub async fn get_neighbors(
        &self,
        graph_id: &str,
        node_id: &str,
    ) -> Result<Vec<(NodeData, EdgeData)>> {
        let graph_data = self.store.export_graph_json(graph_id).await?;

        let neighbor_edge_ids: Vec<String> = graph_data
            .edges
            .iter()
            .filter(|e| e.source_id == node_id || e.target_id == node_id)
            .map(|e| e.id.clone())
            .collect();

        let mut result = Vec::new();
        for edge_id in &neighbor_edge_ids {
            let edge = graph_data
                .edges
                .iter()
                .find(|e| &e.id == edge_id)
                .ok_or_else(|| RingError::NotFound(format!("edge {} not found", edge_id)))?;
            let neighbor_id = if edge.source_id == node_id {
                &edge.target_id
            } else {
                &edge.source_id
            };
            if let Some(node) = graph_data.nodes.iter().find(|n| &n.id == neighbor_id) {
                result.push((node.clone(), edge.clone()));
            }
        }
        Ok(result)
    }

    pub async fn get_node_content(
        &self,
        graph_id: &str,
        node_id: &str,
    ) -> Result<NodeContentResponse> {
        let node = self
            .store
            .get_node(graph_id, node_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("node {} not found", node_id)))?;

        let md_path = self.node_md_path(graph_id, node_id);
        let content = if md_path.exists() {
            Some(std::fs::read_to_string(&md_path)?)
        } else {
            None
        };

        Ok(NodeContentResponse {
            node_id: node.id.clone(),
            label: node.label,
            markdown_path: node.markdown_path,
            content,
            last_modified: node.updated_at,
        })
    }

    pub async fn list_graphs(&self, _ring_id: &str) -> Result<Vec<String>> {
        Ok(self.store.list_graph_ids().await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::sqlite::SqliteRepository;
    use crate::graph::petgraph_store::PetgraphStore;
    use crate::graph::types::NewEdge;

    async fn new_service() -> GraphService {
        let store: Arc<dyn GraphStore> = Arc::new(PetgraphStore::new());
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Arc::new(SqliteRepository::new(pool));
        let search_service = Arc::new(SearchService::new(db, store.clone()));
        let data_dir = tempfile::tempdir().unwrap().path().to_path_buf();
        GraphService::new(store, search_service, data_dir)
    }

    #[tokio::test]
    async fn create_node_with_valid_data() {
        let svc = new_service().await;
        let req = CreateNodeRequest {
            label: "Test Node".into(),
            node_type: "concept".into(),
            parent_id: None,
            description: Some("a test node".into()),
        };
        let node = svc.create_node("graph-1", req).await.unwrap();
        assert_eq!(node.label, "Test Node");
        assert_eq!(node.node_type, "concept");
        assert_eq!(node.graph_id, "graph-1");
        assert_eq!(node.description, Some("a test node".into()));
        assert!(node.parent_id.is_none());
        assert!(!node.id.is_empty());
    }

    #[tokio::test]
    async fn create_node_empty_label_fails() {
        let svc = new_service().await;
        let req = CreateNodeRequest {
            label: "  ".into(),
            node_type: "concept".into(),
            parent_id: None,
            description: None,
        };
        let err = svc.create_node("graph-1", req).await.unwrap_err();
        match err {
            RingError::Validation(msg) => assert!(msg.contains("label")),
            _ => panic!("expected Validation error, got {:?}", err),
        }
    }

    #[tokio::test]
    async fn update_node_label() {
        let svc = new_service().await;
        let node = svc
            .create_node(
                "graph-1",
                CreateNodeRequest {
                    label: "Original".into(),
                    node_type: "concept".into(),
                    parent_id: None,
                    description: None,
                },
            )
            .await
            .unwrap();

        let original_updated_at = node.updated_at.clone();
        let updated = svc
            .update_node(
                "graph-1",
                &node.id,
                UpdateNodeRequest {
                    label: Some("Updated".into()),
                    description: None,
                    node_type: None,
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.label, "Updated");
        assert_ne!(updated.updated_at, original_updated_at);
    }

    #[tokio::test]
    async fn update_node_no_fields_fails() {
        let svc = new_service().await;
        let node = svc
            .create_node(
                "graph-1",
                CreateNodeRequest {
                    label: "Node".into(),
                    node_type: "concept".into(),
                    parent_id: None,
                    description: None,
                },
            )
            .await
            .unwrap();

        let err = svc
            .update_node(
                "graph-1",
                &node.id,
                UpdateNodeRequest {
                    label: None,
                    description: None,
                    node_type: None,
                },
            )
            .await
            .unwrap_err();

        match err {
            RingError::Validation(msg) => assert!(msg.contains("at least one field")),
            _ => panic!("expected Validation error, got {:?}", err),
        }
    }

    #[tokio::test]
    async fn delete_node_cascades_children() {
        let svc = new_service().await;
        let parent = svc
            .create_node(
                "graph-1",
                CreateNodeRequest {
                    label: "Parent".into(),
                    node_type: "category".into(),
                    parent_id: None,
                    description: None,
                },
            )
            .await
            .unwrap();

        let child = svc
            .create_node(
                "graph-1",
                CreateNodeRequest {
                    label: "Child".into(),
                    node_type: "concept".into(),
                    parent_id: Some(parent.id.clone()),
                    description: None,
                },
            )
            .await
            .unwrap();

        svc.delete_node("graph-1", &parent.id).await.unwrap();
        let result = svc.get_node("graph-1", &child.id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_children_returns_correct_order() {
        let svc = new_service().await;
        let parent = svc
            .create_node(
                "graph-1",
                CreateNodeRequest {
                    label: "Parent".into(),
                    node_type: "category".into(),
                    parent_id: None,
                    description: None,
                },
            )
            .await
            .unwrap();

        let c1 = svc
            .create_node(
                "graph-1",
                CreateNodeRequest {
                    label: "C1".into(),
                    node_type: "concept".into(),
                    parent_id: Some(parent.id.clone()),
                    description: None,
                },
            )
            .await
            .unwrap();
        let c2 = svc
            .create_node(
                "graph-1",
                CreateNodeRequest {
                    label: "C2".into(),
                    node_type: "concept".into(),
                    parent_id: Some(parent.id.clone()),
                    description: None,
                },
            )
            .await
            .unwrap();
        let c3 = svc
            .create_node(
                "graph-1",
                CreateNodeRequest {
                    label: "C3".into(),
                    node_type: "concept".into(),
                    parent_id: Some(parent.id.clone()),
                    description: None,
                },
            )
            .await
            .unwrap();

        let children = svc.get_children("graph-1", &parent.id).await.unwrap();
        assert_eq!(children.len(), 3);
        let ids: Vec<&str> = children.iter().map(|c| c.id.as_str()).collect();
        assert!(ids.contains(&c1.id.as_str()));
        assert!(ids.contains(&c2.id.as_str()));
        assert!(ids.contains(&c3.id.as_str()));
    }

    #[tokio::test]
    async fn get_root_nodes_filters_correctly() {
        let svc = new_service().await;
        let root = svc
            .create_node(
                "graph-1",
                CreateNodeRequest {
                    label: "Root".into(),
                    node_type: "category".into(),
                    parent_id: None,
                    description: None,
                },
            )
            .await
            .unwrap();
        let _child = svc
            .create_node(
                "graph-1",
                CreateNodeRequest {
                    label: "Child".into(),
                    node_type: "concept".into(),
                    parent_id: Some(root.id.clone()),
                    description: None,
                },
            )
            .await
            .unwrap();
        let _root2 = svc
            .create_node(
                "graph-1",
                CreateNodeRequest {
                    label: "Root2".into(),
                    node_type: "concept".into(),
                    parent_id: None,
                    description: None,
                },
            )
            .await
            .unwrap();

        let roots = svc.get_root_nodes("graph-1").await.unwrap();
        assert_eq!(roots.len(), 2);
        let labels: Vec<&str> = roots.iter().map(|n| n.label.as_str()).collect();
        assert!(labels.contains(&"Root"));
        assert!(labels.contains(&"Root2"));
        assert!(!labels.contains(&"Child"));
    }

    #[tokio::test]
    async fn get_neighbors_returns_edges() {
        let svc = new_service().await;
        let n1 = svc
            .create_node(
                "graph-1",
                CreateNodeRequest {
                    label: "A".into(),
                    node_type: "concept".into(),
                    parent_id: None,
                    description: None,
                },
            )
            .await
            .unwrap();
        let n2 = svc
            .create_node(
                "graph-1",
                CreateNodeRequest {
                    label: "B".into(),
                    node_type: "concept".into(),
                    parent_id: None,
                    description: None,
                },
            )
            .await
            .unwrap();

        svc.store
            .create_edge(
                "graph-1",
                NewEdge {
                    source_id: n1.id.clone(),
                    target_id: n2.id.clone(),
                    relation: "depends_on".into(),
                    label: Some("A depends on B".into()),
                },
            )
            .await
            .unwrap();

        let neighbors = svc.get_neighbors("graph-1", &n1.id).await.unwrap();
        assert_eq!(neighbors.len(), 1);
        let (neighbor_node, edge) = &neighbors[0];
        assert_eq!(neighbor_node.id, n2.id);
        assert_eq!(edge.source_id, n1.id);
        assert_eq!(edge.target_id, n2.id);
        assert_eq!(edge.relation, "depends_on");
    }
}
