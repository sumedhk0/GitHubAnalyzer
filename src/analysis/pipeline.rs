use std::sync::Arc;
use chrono::Utc;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::Semaphore;

use crate::config::PipelineConfig;
use crate::error::Result;
use crate::github::GitHubClient;
use crate::llm::{AnalysisContext, AnalysisRequest, CommitBatcher, LLMProvider};
use crate::models::analysis::LLMAnalysisResult;
use crate::models::commit::{CommitForAnalysis, FileForAnalysis};
use crate::models::{Commit, Repository, UserProfile};
use crate::analysis::skill_extractor::SkillExtractor;
use crate::analysis::rating_engine::RatingEngine;
use crate::storage::Storage;
use crate::taxonomy::detect_language;

pub struct AnalysisPipeline {
    github: Arc<GitHubClient>,
    llm: Arc<dyn LLMProvider>,
    batcher: CommitBatcher,
    skill_extractor: SkillExtractor,
    rating_engine: RatingEngine,
    storage: Storage,
    config: PipelineConfig,
}

impl AnalysisPipeline {
    pub fn new(
        github: GitHubClient,
        llm: impl LLMProvider + 'static,
        storage: Storage,
        config: PipelineConfig,
    ) -> Self {
        let max_tokens = llm.max_context_tokens();
        Self {
            github: Arc::new(github),
            llm: Arc::new(llm),
            batcher: CommitBatcher::new(max_tokens),
            skill_extractor: SkillExtractor::new(),
            rating_engine: RatingEngine::new(),
            storage,
            config,
        }
    }

    pub async fn analyze_user(&self, username: &str) -> Result<UserProfile> {
        // Step 1: Fetch user profile
        tracing::info!("Fetching GitHub profile for: {}", username);
        let user = self.github.get_user(username).await?;

        // Step 2: Fetch all repositories
        tracing::info!("Fetching repositories...");
        let repos = self.github.get_user_repos(username).await?;

        // Filter out forks if configured
        let repos: Vec<_> = repos
            .into_iter()
            .filter(|r| self.config.include_forks || !r.fork)
            .collect();

        tracing::info!("Found {} repositories to analyze", repos.len());

        // Step 3: Fetch commits from all repos concurrently
        let all_commits = self.fetch_all_commits(username, &repos).await?;
        tracing::info!("Fetched {} commits total", all_commits.len());

        if all_commits.is_empty() {
            tracing::warn!("No commits found for user {}", username);
            return Ok(UserProfile {
                user,
                repositories: repos,
                total_commits_analyzed: 0,
                analysis_date: Utc::now(),
                skills: Vec::new(),
                summary: Default::default(),
            });
        }

        // Step 4: Prepare commits for analysis
        let commits_for_analysis: Vec<_> = all_commits
            .iter()
            .map(|(repo, commit)| self.prepare_commit_for_analysis(repo, commit))
            .collect();

        // Step 5: Batch commits for LLM analysis
        let batches = self.batcher.create_batches(commits_for_analysis.clone());
        tracing::info!("Created {} batches for LLM analysis", batches.len());

        // Step 6: Run LLM analysis on batches
        let analyses = self.run_llm_analysis(batches, &all_commits).await?;
        tracing::info!("Completed {} LLM analyses", analyses.len());

        // Step 7: Extract and aggregate skills
        let analysis_pairs: Vec<_> = analyses
            .iter()
            .zip(commits_for_analysis.iter())
            .map(|(a, c)| (a.clone(), c.clone()))
            .collect();

        let aggregated_skills = self.skill_extractor.aggregate_skills(&analysis_pairs);
        tracing::info!("Extracted {} unique skills", aggregated_skills.len());

        // Step 8: Calculate ratings
        let skill_ratings = self.rating_engine.calculate_ratings(&aggregated_skills);

        // Step 9: Generate summary
        let summary = self.rating_engine.generate_summary(&skill_ratings, &analyses);

        let profile = UserProfile {
            user,
            repositories: repos,
            total_commits_analyzed: all_commits.len() as u32,
            analysis_date: Utc::now(),
            skills: skill_ratings,
            summary,
        };

        // Step 10: Save to storage
        self.storage.save_profile(&profile)?;
        tracing::info!("Profile saved to database");

        Ok(profile)
    }

    async fn fetch_all_commits(
        &self,
        username: &str,
        repos: &[Repository],
    ) -> Result<Vec<(Repository, Commit)>> {
        let semaphore = Arc::new(Semaphore::new(self.config.concurrency_limit));

        let pb = ProgressBar::new(repos.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} repos")
                .unwrap()
                .progress_chars("#>-"),
        );

        let mut commit_futures = Vec::new();

        for repo in repos {
            let github = self.github.clone();
            let sem = semaphore.clone();
            let owner = repo.owner.login.clone();
            let name = repo.name.clone();
            let author = username.to_string();
            let max_commits = self.config.max_commits_per_repo;
            let repo_clone = repo.clone();
            let pb_clone = pb.clone();

            commit_futures.push(async move {
                let _permit = sem.acquire().await.ok()?;

                let commits = github
                    .get_repo_commits(&owner, &name, Some(&author), max_commits)
                    .await
                    .ok()?;

                let mut full_commits = Vec::new();
                for commit_summary in commits.into_iter().take(max_commits as usize) {
                    if let Ok(full_commit) = github
                        .get_commit_with_diff(&owner, &name, &commit_summary.sha)
                        .await
                    {
                        // Only include commits that have actual file changes
                        if full_commit.files.as_ref().map(|f| !f.is_empty()).unwrap_or(false) {
                            full_commits.push((repo_clone.clone(), full_commit));
                        }
                    }
                }

                pb_clone.inc(1);
                Some(full_commits)
            });
        }

        let results = join_all(commit_futures).await;
        pb.finish_with_message("Fetched all commits");

        Ok(results
            .into_iter()
            .flatten()
            .flatten()
            .collect())
    }

    async fn run_llm_analysis(
        &self,
        batches: Vec<Vec<CommitForAnalysis>>,
        all_commits: &[(Repository, Commit)],
    ) -> Result<Vec<LLMAnalysisResult>> {
        let pb = ProgressBar::new(batches.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} batches")
                .unwrap()
                .progress_chars("#>-"),
        );

        let mut all_analyses = Vec::new();

        for batch in batches {
            if batch.is_empty() {
                continue;
            }

            // Get context from first commit in batch
            let context = if let Some(first) = batch.first() {
                let repo = all_commits
                    .iter()
                    .find(|(r, _)| r.full_name == first.repository)
                    .map(|(r, _)| r);

                AnalysisContext {
                    repository_name: first.repository.clone(),
                    repository_description: repo.and_then(|r| r.description.clone()),
                    primary_language: repo.and_then(|r| r.language.clone()),
                }
            } else {
                AnalysisContext::default()
            };

            let request = AnalysisRequest::new(batch, context);

            match self.llm.analyze_commits(request).await {
                Ok(analysis) => {
                    all_analyses.push(analysis);
                }
                Err(e) => {
                    tracing::warn!("LLM analysis failed for batch: {}", e);
                }
            }

            pb.inc(1);
        }

        pb.finish_with_message("LLM analysis complete");
        Ok(all_analyses)
    }

    fn prepare_commit_for_analysis(&self, repo: &Repository, commit: &Commit) -> CommitForAnalysis {
        let files = commit.files.as_ref().map(|files| {
            files
                .iter()
                .filter(|f| f.patch.is_some())
                .map(|f| FileForAnalysis {
                    filename: f.filename.clone(),
                    language: detect_language(&f.filename),
                    diff: f.patch.clone().unwrap_or_default(),
                    additions: f.additions,
                    deletions: f.deletions,
                })
                .collect()
        }).unwrap_or_default();

        CommitForAnalysis {
            sha: commit.sha.clone(),
            repository: repo.full_name.clone(),
            message: commit.commit.message.clone(),
            stats: commit.stats.clone().unwrap_or_default(),
            files_changed: files,
            committed_at: commit.commit.author.date,
        }
    }
}
