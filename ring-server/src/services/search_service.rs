use std::sync::Arc;

use tokio::sync::RwLock;

use crate::db::traits::Repository;
use crate::error::Result;
use crate::graph::petgraph_store::PetgraphStore;
use crate::models::graph_model::SearchResult;

pub struct SearchService {
    repo: Arc<dyn Repository>,
    #[allow(dead_code)]
    store: Arc<RwLock<PetgraphStore>>,
}

impl SearchService {
    pub fn new(repo: Arc<dyn Repository>, store: Arc<RwLock<PetgraphStore>>) -> Self {
        SearchService { repo, store }
    }

    pub async fn search_nodes(
        &self,
        query: &str,
        graph_ids: Option<Vec<String>>,
        limit: i64,
    ) -> Result<Vec<SearchResult>> {
        self.repo.search_nodes_fts(query, graph_ids, limit).await
    }

    pub async fn index_node(
        &self,
        node_id: &str,
        graph_id: &str,
        label: &str,
        content: &str,
    ) -> Result<()> {
        self.repo
            .index_node_search(node_id, graph_id, label, content)
            .await
    }

    pub async fn delete_node_index(&self, node_id: &str) -> Result<()> {
        self.repo.delete_node_search(node_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_service() -> SearchService {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let repo = crate::db::sqlite::SqliteRepository::new(pool);
        let store = Arc::new(RwLock::new(PetgraphStore::new()));
        SearchService::new(Arc::new(repo), store)
    }

    #[tokio::test]
    async fn index_and_search_node() {
        let svc = setup_service().await;
        svc.index_node("n1", "g1", "Rust programming", "Rust is a systems language")
            .await
            .unwrap();

        let results = svc.search_nodes("Rust", None, 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].node_id, "n1");
        assert_eq!(results[0].graph_id, "g1");
    }

    #[tokio::test]
    async fn search_no_results() {
        let svc = setup_service().await;
        svc.index_node("n1", "g1", "Python", "Python is great")
            .await
            .unwrap();

        let results = svc.search_nodes("nonexistent_term_xyz", None, 10).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn search_filters_by_graph_id() {
        let svc = setup_service().await;
        svc.index_node("n1", "g1", "Rust language", "Rust systems")
            .await
            .unwrap();
        svc.index_node("n2", "g2", "Rust language", "Rust web")
            .await
            .unwrap();

        let results = svc
            .search_nodes("Rust", Some(vec!["g1".into()]), 10)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].node_id, "n1");
        assert_eq!(results[0].graph_id, "g1");
    }

    #[tokio::test]
    async fn delete_node_removes_from_search() {
        let svc = setup_service().await;
        svc.index_node("n1", "g1", "Rust language", "Rust systems")
            .await
            .unwrap();

        svc.delete_node_index("n1").await.unwrap();

        let results = svc.search_nodes("Rust", None, 10).await.unwrap();
        assert!(results.is_empty());
    }
}
