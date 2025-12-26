use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::skill::SkillRating;
use super::analysis::ProfileSummary;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: String,
    pub bio: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    pub public_repos: u32,
    pub followers: u32,
    pub following: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub stargazers_count: u32,
    pub forks_count: u32,
    pub fork: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub owner: RepositoryOwner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryOwner {
    pub login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user: GitHubUser,
    pub repositories: Vec<Repository>,
    pub total_commits_analyzed: u32,
    pub analysis_date: DateTime<Utc>,
    pub skills: Vec<SkillRating>,
    pub summary: ProfileSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageBreakdown {
    pub language: String,
    pub bytes: u64,
    pub percentage: f32,
}
