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
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: DashMap::new(),
            max_requests,
            window: Duration::from_secs(window_secs),
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

        let mut entry = self.requests.entry(key).or_default();
        entry.retain(|t| now.duration_since(*t) < self.window);

        if entry.len() >= self.max_requests {
            return Err(crate::error::RingError::TooManyRequests);
        }

        entry.push(now);
        drop(entry);

        Ok(next.run(request).await)
    }
}
