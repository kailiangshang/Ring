use axum::{extract::Request, middleware::Next, response::Response};
use dashmap::DashMap;
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
}

pub async fn rate_limit(
    axum::extract::State(limiter): axum::extract::State<RateLimiter>,
    request: Request,
    next: Next,
) -> Result<Response, crate::error::RingError> {
    let token = request
        .headers()
        .get("x-ring-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous");
    let key = format!("{}:{}", token, request.uri().path());
    let now = Instant::now();

    if limiter.requests.len() > limiter.max_entries {
        let keys_to_remove: Vec<String> = limiter
            .requests
            .iter()
            .filter(|entry| {
                let timestamps = entry.value();
                timestamps.is_empty()
                    || timestamps
                        .iter()
                        .all(|t| now.duration_since(*t) >= limiter.window)
            })
            .map(|entry| entry.key().clone())
            .collect();
        for key in keys_to_remove {
            limiter.requests.remove(&key);
        }
    }

    let mut entry = limiter.requests.entry(key).or_default();
    entry.retain(|t| now.duration_since(*t) < limiter.window);

    if entry.len() >= limiter.max_requests {
        return Err(crate::error::RingError::TooManyRequests);
    }

    entry.push(now);
    drop(entry);

    Ok(next.run(request).await)
}
