use clap::Parser;
use tracing_subscriber::EnvFilter;

use gitanalyzer::{
    AnalysisPipeline, ClaudeProvider, Config, GitHubClient, PipelineConfig, Storage,
};
use gitanalyzer::models::UserProfile;

#[derive(Parser, Debug)]
#[command(name = "gitanalyzer")]
#[command(version = "0.1.0")]
#[command(about = "Analyze GitHub profiles and extract developer skills")]
#[command(author = "Git Profile Analyzer")]
struct Args {
    /// GitHub username to analyze
    #[arg(short, long)]
    username: String,

    /// Output format (json, text, markdown)
    #[arg(short, long, default_value = "text")]
    format: String,

    /// Output file (defaults to stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Maximum commits to analyze per repository
    #[arg(long, default_value = "50")]
    max_commits_per_repo: u32,

    /// Include forked repositories
    #[arg(long)]
    include_forks: bool,

    /// Database path for storing results
    #[arg(long, default_value = "gitanalyzer.db")]
    database: String,

    /// Use cached profile if available
    #[arg(long)]
    cached: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("gitanalyzer=info".parse()?)
                .add_directive("reqwest=warn".parse()?),
        )
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Parse CLI arguments
    let args = Args::parse();

    // Load configuration
    let config = Config::from_env()?;

    // Initialize storage
    let storage = Storage::new(&args.database)?;

    // Check for cached profile if requested
    if args.cached {
        if let Some(profile) = storage.get_profile(&args.username)? {
            tracing::info!("Using cached profile from {}", profile.analysis_date);
            output_profile(&profile, &args)?;
            return Ok(());
        }
        tracing::info!("No cached profile found, performing fresh analysis");
    }

    // Initialize clients
    let github = GitHubClient::new(&config.github_token)?;
    let llm = ClaudeProvider::new(
        config.anthropic_api_key.clone(),
        Some("claude-sonnet-4-20250514".to_string()),
    );

    // Create pipeline
    let pipeline_config = PipelineConfig {
        max_commits_per_repo: args.max_commits_per_repo,
        include_forks: args.include_forks,
        concurrency_limit: config.concurrency_limit,
    };

    let pipeline = AnalysisPipeline::new(github, llm, storage, pipeline_config);

    // Run analysis
    tracing::info!("Starting analysis for GitHub user: {}", args.username);
    let profile = pipeline.analyze_user(&args.username).await?;

    // Output results
    output_profile(&profile, &args)?;

    Ok(())
}

fn output_profile(profile: &UserProfile, args: &Args) -> anyhow::Result<()> {
    let output = match args.format.as_str() {
        "json" => serde_json::to_string_pretty(profile)?,
        "markdown" => format_markdown(profile),
        _ => format_text(profile),
    };

    if let Some(ref path) = args.output {
        std::fs::write(path, &output)?;
        tracing::info!("Output written to: {}", path);
    } else {
        println!("{}", output);
    }

    Ok(())
}

fn format_text(profile: &UserProfile) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "\n=== Profile Analysis: {} ===\n\n",
        profile.user.login
    ));

    if let Some(ref name) = profile.user.name {
        output.push_str(&format!("Name: {}\n", name));
    }
    if let Some(ref bio) = profile.user.bio {
        output.push_str(&format!("Bio: {}\n", bio));
    }

    output.push_str(&format!(
        "Commits analyzed: {}\n",
        profile.total_commits_analyzed
    ));
    output.push_str(&format!(
        "Repositories: {}\n",
        profile.repositories.len()
    ));
    output.push_str(&format!(
        "Experience Level: {}\n\n",
        profile.summary.experience_level
    ));

    // Top Skills
    output.push_str("Top Skills:\n");
    for skill in profile.skills.iter().take(10) {
        let trend_indicator = match skill.trend {
            gitanalyzer::models::skill::SkillTrend::Improving => " ↑",
            gitanalyzer::models::skill::SkillTrend::Declining => " ↓",
            gitanalyzer::models::skill::SkillTrend::Dormant => " ⏸",
            _ => "",
        };
        output.push_str(&format!(
            "  - {} ({}): {}/100 (confidence: {:.0}%){}\n",
            skill.skill.name,
            skill.skill.category,
            skill.proficiency_score,
            skill.confidence * 100.0,
            trend_indicator
        ));
    }

    // Primary Languages
    if !profile.summary.primary_languages.is_empty() {
        output.push_str(&format!(
            "\nPrimary Languages: {}\n",
            profile.summary.primary_languages.join(", ")
        ));
    }

    // Strengths
    if !profile.summary.strengths.is_empty() {
        output.push_str("\nStrengths:\n");
        for strength in &profile.summary.strengths {
            output.push_str(&format!("  + {}: {}\n", strength.area, strength.description));
        }
    }

    // Weaknesses
    if !profile.summary.weaknesses.is_empty() {
        output.push_str("\nAreas for Improvement:\n");
        for weakness in &profile.summary.weaknesses {
            output.push_str(&format!("  - {}: {}\n", weakness.area, weakness.description));
        }
    }

    // Coding Style
    output.push_str("\nCoding Style:\n");
    output.push_str(&format!(
        "  Tests: {:.0}%\n",
        profile.summary.coding_style.writes_tests * 100.0
    ));
    output.push_str(&format!(
        "  Documentation: {:.0}%\n",
        profile.summary.coding_style.documents_code * 100.0
    ));
    output.push_str(&format!(
        "  Follows Conventions: {:.0}%\n",
        profile.summary.coding_style.follows_conventions * 100.0
    ));

    output.push_str(&format!(
        "\nAnalyzed on: {}\n",
        profile.analysis_date.format("%Y-%m-%d %H:%M:%S UTC")
    ));

    output
}

fn format_markdown(profile: &UserProfile) -> String {
    let mut output = String::new();

    output.push_str(&format!("# Profile Analysis: {}\n\n", profile.user.login));

    if let Some(ref name) = profile.user.name {
        output.push_str(&format!("**Name:** {}\n\n", name));
    }
    if let Some(ref bio) = profile.user.bio {
        output.push_str(&format!("> {}\n\n", bio));
    }

    output.push_str("## Summary\n\n");
    output.push_str("| Metric | Value |\n|--------|-------|\n");
    output.push_str(&format!(
        "| Commits Analyzed | {} |\n",
        profile.total_commits_analyzed
    ));
    output.push_str(&format!(
        "| Repositories | {} |\n",
        profile.repositories.len()
    ));
    output.push_str(&format!(
        "| Experience Level | {} |\n",
        profile.summary.experience_level
    ));

    if !profile.summary.primary_languages.is_empty() {
        output.push_str(&format!(
            "| Primary Languages | {} |\n",
            profile.summary.primary_languages.join(", ")
        ));
    }

    output.push_str("\n## Top Skills\n\n");
    output.push_str("| Skill | Category | Score | Confidence | Trend |\n");
    output.push_str("|-------|----------|-------|------------|-------|\n");

    for skill in profile.skills.iter().take(15) {
        output.push_str(&format!(
            "| {} | {} | {}/100 | {:.0}% | {} |\n",
            skill.skill.name,
            skill.skill.category,
            skill.proficiency_score,
            skill.confidence * 100.0,
            skill.trend
        ));
    }

    if !profile.summary.strengths.is_empty() {
        output.push_str("\n## Strengths\n\n");
        for strength in &profile.summary.strengths {
            output.push_str(&format!(
                "- **{}**: {}\n",
                strength.area, strength.description
            ));
        }
    }

    if !profile.summary.weaknesses.is_empty() {
        output.push_str("\n## Areas for Improvement\n\n");
        for weakness in &profile.summary.weaknesses {
            output.push_str(&format!(
                "- **{}**: {}\n",
                weakness.area, weakness.description
            ));
        }
    }

    output.push_str("\n## Coding Style\n\n");
    output.push_str("| Metric | Score |\n|--------|-------|\n");
    output.push_str(&format!(
        "| Test Coverage | {:.0}% |\n",
        profile.summary.coding_style.writes_tests * 100.0
    ));
    output.push_str(&format!(
        "| Documentation | {:.0}% |\n",
        profile.summary.coding_style.documents_code * 100.0
    ));
    output.push_str(&format!(
        "| Convention Adherence | {:.0}% |\n",
        profile.summary.coding_style.follows_conventions * 100.0
    ));

    output.push_str(&format!(
        "\n---\n*Analyzed on {}*\n",
        profile.analysis_date.format("%Y-%m-%d %H:%M:%S UTC")
    ));

    output
}
