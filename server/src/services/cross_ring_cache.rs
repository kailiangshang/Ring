use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use sqlx::SqlitePool;
use tokio::sync::Mutex;

use crate::services::super_chat;

const CACHE_TTL: Duration = Duration::from_secs(300);

type CacheInner = HashMap<String, (String, Instant)>;
pub type CrossRingCache = Arc<Mutex<CacheInner>>;

pub fn new_cache() -> CrossRingCache {
    Arc::new(Mutex::new(HashMap::new()))
}

pub async fn get_summary(
    cache: &CrossRingCache,
    pool: &SqlitePool,
    user_id: &str,
) -> String {
    let key = format!("summary:{user_id}");
    {
        let map = cache.lock().await;
        if let Some((val, created)) = map.get(&key) {
            if created.elapsed() < CACHE_TTL {
                return val.clone();
            }
        }
    }

    let value = super_chat::build_ring_summary(pool, user_id).await;

    let mut map = cache.lock().await;
    map.insert(key, (value.clone(), Instant::now()));
    value
}

pub async fn get_detail(
    cache: &CrossRingCache,
    pool: &SqlitePool,
    rings_dir: &Path,
    user_id: &str,
    ring_id: &str,
    ring_name: &str,
) -> String {
    let key = format!("detail:{ring_id}");
    {
        let map = cache.lock().await;
        if let Some((val, created)) = map.get(&key) {
            if created.elapsed() < CACHE_TTL {
                return val.clone();
            }
        }
    }

    let value = super_chat::execute_query_ring_detail(pool, rings_dir, user_id, ring_name)
        .await
        .unwrap_or_default();

    let mut map = cache.lock().await;
    map.insert(key, (value.clone(), Instant::now()));
    value
}

pub async fn invalidate_ring(cache: &CrossRingCache, ring_id: &str) {
    let mut map = cache.lock().await;
    map.remove(&format!("detail:{ring_id}"));
    map.remove(&format!("graph:{ring_id}"));
}

pub async fn invalidate_summary(cache: &CrossRingCache, user_id: &str) {
    let mut map = cache.lock().await;
    map.remove(&format!("summary:{user_id}"));
}
