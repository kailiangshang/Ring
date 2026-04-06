use std::sync::Arc;

use tokio::sync::RwLock;

use crate::config::Config;
use crate::db::traits::Repository;
use crate::graph::petgraph_store::PetgraphStore;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<dyn Repository>,
    pub graph_store: Arc<RwLock<PetgraphStore>>,
    pub config: Arc<Config>,
}
