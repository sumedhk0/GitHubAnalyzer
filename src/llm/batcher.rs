use crate::models::commit::{CommitForAnalysis, FileForAnalysis};

pub struct CommitBatcher {
    max_tokens: usize,
    reserved_tokens: usize,
}

impl CommitBatcher {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            // Reserve tokens for system prompt (~1000) and response (~3000)
            reserved_tokens: 4_000,
        }
    }

    pub fn create_batches(
        &self,
        commits: Vec<CommitForAnalysis>,
    ) -> Vec<Vec<CommitForAnalysis>> {
        let available_tokens = self.max_tokens.saturating_sub(self.reserved_tokens);
        let mut batches = Vec::new();
        let mut current_batch = Vec::new();
        let mut current_tokens = 0;

        for commit in commits {
            let commit_tokens = self.estimate_commit_tokens(&commit);

            // If single commit is too large, truncate its diffs
            if commit_tokens > available_tokens {
                let truncated = self.truncate_commit(commit, available_tokens);
                if !current_batch.is_empty() {
                    batches.push(std::mem::take(&mut current_batch));
                    current_tokens = 0;
                }
                batches.push(vec![truncated]);
                continue;
            }

            if current_tokens + commit_tokens > available_tokens {
                if !current_batch.is_empty() {
                    batches.push(std::mem::take(&mut current_batch));
                }
                current_tokens = 0;
            }

            current_tokens += commit_tokens;
            current_batch.push(commit);
        }

        if !current_batch.is_empty() {
            batches.push(current_batch);
        }

        batches
    }

    fn estimate_commit_tokens(&self, commit: &CommitForAnalysis) -> usize {
        let char_count = commit.message.len()
            + commit
                .files_changed
                .iter()
                .map(|f| f.filename.len() + f.diff.len())
                .sum::<usize>();
        // Add overhead for formatting
        (char_count / 4) + 100
    }

    fn truncate_commit(
        &self,
        mut commit: CommitForAnalysis,
        max_tokens: usize,
    ) -> CommitForAnalysis {
        let max_chars = max_tokens * 4;
        let overhead = commit.message.len() + 200;
        let available_for_diffs = max_chars.saturating_sub(overhead);

        // Sort files by relevance (code files first, then by size)
        commit
            .files_changed
            .sort_by(|a, b| {
                let a_priority = self.file_priority(&a.filename);
                let b_priority = self.file_priority(&b.filename);
                b_priority.cmp(&a_priority)
            });

        let mut used_chars = 0;
        let mut truncated_files = Vec::new();

        for mut file in commit.files_changed {
            let file_overhead = file.filename.len() + 50;
            let available = available_for_diffs.saturating_sub(used_chars + file_overhead);

            if available == 0 {
                break;
            }

            if file.diff.len() > available {
                file.diff = file.diff.chars().take(available).collect();
                file.diff.push_str("\n... [truncated]");
            }

            used_chars += file_overhead + file.diff.len();
            truncated_files.push(file);
        }

        commit.files_changed = truncated_files;
        commit
    }

    fn file_priority(&self, filename: &str) -> u32 {
        let ext = filename.rsplit('.').next().unwrap_or("");
        match ext.to_lowercase().as_str() {
            // High priority: main code files
            "rs" | "py" | "ts" | "js" | "go" | "java" | "cpp" | "c" | "rb" | "swift" | "kt" => 100,
            // Frontend
            "tsx" | "jsx" | "vue" | "svelte" => 90,
            // Data/query
            "sql" | "graphql" => 80,
            // Config (still useful)
            "yaml" | "yml" | "toml" | "json" => 50,
            // Docs
            "md" | "txt" | "rst" => 30,
            // Lock files (usually not useful)
            "lock" => 0,
            _ => 40,
        }
    }
}

impl Default for CommitBatcher {
    fn default() -> Self {
        // Default to Claude's context window
        Self::new(200_000)
    }
}
