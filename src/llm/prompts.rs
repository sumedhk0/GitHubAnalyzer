use crate::models::commit::CommitForAnalysis;

pub const SYSTEM_PROMPT: &str = r#"You are an expert software engineer and technical recruiter analyzing Git commit history.
Your task is to extract skills, expertise levels, and coding patterns from commit diffs.

You must respond with valid JSON matching this exact schema:
{
    "skills": [
        {
            "name": "string (e.g., 'Rust', 'React', 'PostgreSQL')",
            "category": "language|framework|library|tool|domain|practice|concept",
            "proficiency_level": "beginner|intermediate|advanced|expert",
            "confidence": 0.0-1.0,
            "evidence": ["string describing specific evidence from the code"]
        }
    ],
    "patterns": [
        {
            "type": "design_pattern|anti_pattern|testing|security|performance|documentation",
            "name": "string",
            "description": "string",
            "quality_impact": -1.0 to 1.0 (negative for bad, positive for good)
        }
    ],
    "complexity_assessment": {
        "overall_score": 1-10,
        "algorithmic_complexity": 1-10,
        "architectural_complexity": 1-10,
        "reasoning": "string explaining the assessment"
    },
    "quality_assessment": {
        "code_quality": 1-10,
        "testing_coverage": 0.0-1.0 (estimated based on test files/code),
        "documentation_quality": 1-10,
        "error_handling": 1-10,
        "observations": ["string observations about code quality"]
    },
    "domain_signals": ["frontend", "backend", "devops", "ml", "security", "mobile", "data", "systems"],
    "notable_aspects": ["string describing notable things about this developer's code"]
}

Guidelines:
- Be specific with skill names (e.g., "React" not just "JavaScript framework")
- Only report skills you have strong evidence for from the actual code
- Proficiency levels: beginner (basic usage), intermediate (competent), advanced (sophisticated patterns), expert (mastery)
- Consider code complexity, patterns, and best practices when assessing proficiency
- Domain signals help categorize what type of development this is"#;

#[derive(Debug, Clone)]
pub struct AnalysisRequest {
    pub commits: Vec<CommitForAnalysis>,
    pub context: AnalysisContext,
}

#[derive(Debug, Clone, Default)]
pub struct AnalysisContext {
    pub repository_name: String,
    pub repository_description: Option<String>,
    pub primary_language: Option<String>,
}

impl AnalysisRequest {
    pub fn new(commits: Vec<CommitForAnalysis>, context: AnalysisContext) -> Self {
        Self { commits, context }
    }

    pub fn to_prompt(&self) -> String {
        let mut prompt = format!(
            "Analyze the following {} commit(s) from repository '{}'",
            self.commits.len(),
            self.context.repository_name
        );

        if let Some(desc) = &self.context.repository_description {
            if !desc.is_empty() {
                prompt.push_str(&format!(" ({})", desc));
            }
        }
        prompt.push_str(":\n\n");

        for commit in &self.commits {
            prompt.push_str(&format!("## Commit: {}\n", &commit.sha[..8.min(commit.sha.len())]));
            prompt.push_str(&format!("Message: {}\n", commit.message.lines().next().unwrap_or("")));
            prompt.push_str(&format!(
                "Stats: +{} -{}\n\n",
                commit.stats.additions, commit.stats.deletions
            ));

            for file in &commit.files_changed {
                prompt.push_str(&format!("### File: {}", file.filename));
                if let Some(lang) = &file.language {
                    prompt.push_str(&format!(" ({})", lang));
                }
                prompt.push_str("\n```\n");
                // Limit diff size per file to avoid huge prompts
                let diff = if file.diff.len() > 3000 {
                    format!("{}...\n[truncated]", &file.diff[..3000])
                } else {
                    file.diff.clone()
                };
                prompt.push_str(&diff);
                prompt.push_str("\n```\n\n");
            }
        }

        prompt.push_str("\nProvide your analysis as JSON:\n");
        prompt
    }

    pub fn estimate_tokens(&self) -> usize {
        let char_count: usize = self
            .commits
            .iter()
            .map(|c| {
                c.message.len()
                    + c.files_changed
                        .iter()
                        .map(|f| f.filename.len() + f.diff.len())
                        .sum::<usize>()
            })
            .sum();
        // Rough estimate: ~4 characters per token
        char_count / 4
    }
}
