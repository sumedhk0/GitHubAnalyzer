use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use reqwest::Response;

pub struct RateLimiter {
    state: Arc<Mutex<RateLimitState>>,
}

struct RateLimitState {
    remaining: u32,
    reset_at: Option<std::time::Instant>,
    requests_this_minute: u32,
    minute_start: std::time::Instant,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RateLimitState {
                remaining: 5000,
                reset_at: None,
                requests_this_minute: 0,
                minute_start: std::time::Instant::now(),
            })),
        }
    }

    pub async fn wait(&self) {
        let mut state = self.state.lock().await;

        // Check if we need to wait for rate limit reset
        if state.remaining == 0 {
            if let Some(reset_at) = state.reset_at {
                let now = std::time::Instant::now();
                if reset_at > now {
                    let wait_duration = reset_at - now;
                    drop(state);
                    tracing::info!("Rate limited, waiting {:?}", wait_duration);
                    sleep(wait_duration).await;
                    state = self.state.lock().await;
                }
            }
        }

        // Soft rate limiting: max 30 requests per minute to be polite
        let minute_elapsed = state.minute_start.elapsed();
        if minute_elapsed < Duration::from_secs(60) {
            if state.requests_this_minute >= 30 {
                let wait_time = Duration::from_secs(60) - minute_elapsed;
                drop(state);
                tracing::debug!("Soft rate limiting, waiting {:?}", wait_time);
                sleep(wait_time).await;
                state = self.state.lock().await;
                state.requests_this_minute = 0;
                state.minute_start = std::time::Instant::now();
            }
        } else {
            state.requests_this_minute = 0;
            state.minute_start = std::time::Instant::now();
        }

        state.requests_this_minute += 1;
    }

    pub fn update_from_response(&self, response: &Response) {
        if let Some(remaining) = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
        {
            let state = self.state.clone();
            let reset = response
                .headers()
                .get("x-ratelimit-reset")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());

            tokio::spawn(async move {
                let mut state = state.lock().await;
                state.remaining = remaining;
                if let Some(reset_timestamp) = reset {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    if reset_timestamp > now {
                        let wait_secs = reset_timestamp - now;
                        state.reset_at =
                            Some(std::time::Instant::now() + Duration::from_secs(wait_secs));
                    }
                }
            });
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}
