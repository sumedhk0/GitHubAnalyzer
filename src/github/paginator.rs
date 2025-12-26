use reqwest::Client;
use serde::de::DeserializeOwned;
use crate::github::rate_limiter::RateLimiter;
use crate::error::Result;

pub struct Paginator<'a> {
    client: &'a Client,
    rate_limiter: &'a RateLimiter,
}

impl<'a> Paginator<'a> {
    pub fn new(client: &'a Client, rate_limiter: &'a RateLimiter) -> Self {
        Self {
            client,
            rate_limiter,
        }
    }

    pub async fn fetch_all<T: DeserializeOwned>(
        &self,
        base_url: &str,
        per_page: u32,
    ) -> Result<Vec<T>> {
        let mut all_items = Vec::new();
        let mut page = 1;

        loop {
            self.rate_limiter.wait().await;

            let separator = if base_url.contains('?') { "&" } else { "?" };
            let url = format!("{}{}per_page={}&page={}", base_url, separator, per_page, page);

            tracing::debug!("Fetching: {}", url);
            let response = self.client.get(&url).send().await?;
            self.rate_limiter.update_from_response(&response);

            // Check for next page in Link header
            let has_next = response
                .headers()
                .get("link")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.contains("rel=\"next\""))
                .unwrap_or(false);

            let items: Vec<T> = response.json().await?;
            let items_count = items.len();
            all_items.extend(items);

            if !has_next || items_count < per_page as usize {
                break;
            }

            page += 1;
        }

        Ok(all_items)
    }

    pub async fn fetch_limited<T: DeserializeOwned>(
        &self,
        base_url: &str,
        per_page: u32,
        max_items: u32,
    ) -> Result<Vec<T>> {
        let mut all_items = Vec::new();
        let mut page = 1;

        loop {
            self.rate_limiter.wait().await;

            let separator = if base_url.contains('?') { "&" } else { "?" };
            let url = format!("{}{}per_page={}&page={}", base_url, separator, per_page, page);

            tracing::debug!("Fetching: {}", url);
            let response = self.client.get(&url).send().await?;
            self.rate_limiter.update_from_response(&response);

            let has_next = response
                .headers()
                .get("link")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.contains("rel=\"next\""))
                .unwrap_or(false);

            let items: Vec<T> = response.json().await?;
            let items_count = items.len();
            all_items.extend(items);

            if all_items.len() >= max_items as usize || !has_next || items_count < per_page as usize
            {
                break;
            }

            page += 1;
        }

        all_items.truncate(max_items as usize);
        Ok(all_items)
    }
}
