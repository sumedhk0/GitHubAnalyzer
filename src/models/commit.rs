use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSummary {
    pub sha: String,
    pub commit: CommitDetails,
    pub author: Option<CommitAuthorInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitDetails {
    pub message: String,
    pub author: CommitAuthor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
    pub date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAuthorInfo {
    pub login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub commit: CommitDetails,
    pub stats: Option<CommitStats>,
    pub files: Option<Vec<FileChange>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommitStats {
    pub additions: u32,
    pub deletions: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub filename: String,
    pub status: String,
    pub additions: u32,
    pub deletions: u32,
    pub patch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
}

impl From<&str> for FileStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "added" => FileStatus::Added,
            "modified" => FileStatus::Modified,
            "deleted" | "removed" => FileStatus::Deleted,
            "renamed" => FileStatus::Renamed,
            "copied" => FileStatus::Copied,
            _ => FileStatus::Modified,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitForAnalysis {
    pub sha: String,
    pub repository: String,
    pub message: String,
    pub stats: CommitStats,
    pub files_changed: Vec<FileForAnalysis>,
    pub committed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileForAnalysis {
    pub filename: String,
    pub language: Option<String>,
    pub diff: String,
    pub additions: u32,
    pub deletions: u32,
}
