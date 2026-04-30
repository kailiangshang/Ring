use axum::{
    extract::{ConnectInfo, Request},
    middleware::Next,
    response::Response,
};
use dashmap::DashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct RateLimiter {
    requests: DashMap<String, Vec<Instant>>,
    max_requests: usize,
    window: Duration,
    max_entries: usize,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: DashMap::new(),
            max_requests,
            window: Duration::from_secs(window_secs),
            max_entries: 10000,
        }
    }

    pub async fn limit(
        &self,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
        request: Request,
        next: Next,
    ) -> Result<Response, crate::error::RingError> {
        let key = format!("{}:{}", addr.ip(), request.uri().path());
        let now = Instant::now();

        // Cleanup old entries if map is too large
        if self.requests.len() > self.max_entries {
            self.cleanup_old_entries(now);
        }

        let mut entry = self.requests.entry(key).or_default();
        entry.retain(|t| now.duration_since(*t) < self.window);

        if entry.len() >= self.max_requests {
            return Err(crate::error::RingError::TooManyRequests);
        }

        entry.push(now);
        drop(entry);

        Ok(next.run(request).await)
    }

    fn cleanup_old_entries(&self,
        now: Instant,
    ) {
        let keys_to_remove: Vec<String> = self
            .requests
            .iter()
            .filter(|entry| {
                let timestamps = entry.value();
                timestamps.is_empty() || timestamps.iter().all(|t| now.duration_since(*t) >= self.window)
            })
            .map(|entry| entry.key().clone())
            .collect();

        for key in keys_to_remove {
            self.requests.remove(&key);
        }
    }
}