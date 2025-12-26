use reqwest::{header, Client};
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::github::paginator::Paginator;
use crate::github::rate_limiter::RateLimiter;
use crate::models::{Commit, CommitSummary, GitHubUser, Repository};

pub struct GitHubClient {
    client: Client,
    rate_limiter: RateLimiter,
    base_url: String,
}

impl GitHubClient {
    pub fn new(token: &str) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", token))?,
        );
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "X-GitHub-Api-Version",
            header::HeaderValue::from_static("2022-11-28"),
        );
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("git-profile-analyzer/1.0"),
        );

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            client,
            rate_limiter: RateLimiter::new(),
            base_url: "https://api.github.com".to_string(),
        })
    }

    pub async fn get_user(&self, username: &str) -> Result<GitHubUser> {
        self.rate_limiter.wait().await;
        let url = format!("{}/users/{}", self.base_url, username);
        tracing::info!("Fetching user: {}", username);

        let response = self.client.get(&url).send().await?;
        self.rate_limiter.update_from_response(&response);

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::UserNotFound(username.to_string()));
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::GitHubApi(format!(
                "Failed to fetch user {}: {} - {}",
                username, status, body
            )));
        }

        Ok(response.json().await?)
    }

    pub async fn get_user_repos(&self, username: &str) -> Result<Vec<Repository>> {
        let url = format!("{}/users/{}/repos?type=owner&sort=updated", self.base_url, username);
        let paginator = Paginator::new(&self.client, &self.rate_limiter);
        tracing::info!("Fetching repositories for: {}", username);
        paginator.fetch_all(&url, 100).await
    }

    pub async fn get_repo_commits(
        &self,
        owner: &str,
        repo: &str,
        author: Option<&str>,
        max_commits: u32,
    ) -> Result<Vec<CommitSummary>> {
        let mut url = format!("{}/repos/{}/{}/commits", self.base_url, owner, repo);
        if let Some(author) = author {
            url.push_str(&format!("?author={}", author));
        }

        let paginator = Paginator::new(&self.client, &self.rate_limiter);
        tracing::debug!("Fetching commits for: {}/{}", owner, repo);
        paginator.fetch_limited(&url, 100, max_commits).await
    }

    pub async fn get_commit_with_diff(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
    ) -> Result<Commit> {
        self.rate_limiter.wait().await;
        let url = format!("{}/repos/{}/{}/commits/{}", self.base_url, owner, repo, sha);
        tracing::debug!("Fetching commit diff: {}", &sha[..7]);

        let response = self.client.get(&url).send().await?;
        self.rate_limiter.update_from_response(&response);

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::GitHubApi(format!(
                "Failed to fetch commit {}: {} - {}",
                sha, status, body
            )));
        }

        Ok(response.json().await?)
    }

    pub async fn get_repo_languages(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<HashMap<String, u64>> {
        self.rate_limiter.wait().await;
        let url = format!("{}/repos/{}/{}/languages", self.base_url, owner, repo);

        let response = self.client.get(&url).send().await?;
        self.rate_limiter.update_from_response(&response);

        if !response.status().is_success() {
            return Ok(HashMap::new());
        }

        Ok(response.json().await?)
    }

    pub fn rate_limiter(&self) -> &RateLimiter {
        &self.rate_limiter
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}
