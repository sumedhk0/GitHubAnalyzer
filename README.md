# GitAnalyzer

A Rust CLI tool that analyzes GitHub profiles by examining commit history across all repositories. Uses Claude AI to extract skills, assess proficiency levels, and identify strengths and weaknesses.

## Features

- **Skill Extraction**: Automatically detects programming languages, frameworks, tools, and practices from commit diffs
- **Proficiency Scoring**: Multi-dimensional ratings (1-100) based on frequency, recency, complexity, and code quality
- **Trend Analysis**: Tracks whether skills are improving, stable, declining, or dormant
- **Weakness Detection**: Identifies areas for improvement (low test coverage, anti-patterns, etc.)
- **Multiple Output Formats**: Text, JSON, or Markdown reports
- **Local Caching**: SQLite database stores results for quick re-access and cross-user comparison

## Prerequisites

- **Rust** (1.70+): [Install Rust](https://rustup.rs/)
- **GitHub Personal Access Token**: [Create token](https://github.com/settings/tokens)
- **Anthropic API Key**: [Get API key](https://console.anthropic.com/)

## Setup

### 1. Clone the Repository

```bash
git clone <your-repo-url>
cd gitanalyzer
```

### 2. Configure Environment Variables

Create a `.env` file in the project root:

```bash
cp .env.example .env
```

Edit `.env` with your API keys:

```env
# Required
GITHUB_TOKEN=ghp_your_github_personal_access_token
ANTHROPIC_API_KEY=your_anthropic_api_key

# Optional
DATABASE_PATH=gitanalyzer.db
MAX_COMMITS_PER_REPO=100
INCLUDE_FORKS=false
CONCURRENCY_LIMIT=5
```

#### Getting Your Tokens

**GitHub Token:**
1. Go to https://github.com/settings/tokens
2. Click "Generate new token (classic)"
3. Select scopes: `public_repo` (or `repo` for private repos)
4. Copy the token to your `.env` file

**Anthropic API Key:**
1. Go to https://console.anthropic.com/
2. Navigate to API Keys
3. Create a new key and copy it to your `.env` file

### 3. Build the Project

```bash
cargo build --release
```

## Usage

### Basic Analysis

```bash
# Analyze a GitHub user
cargo run --release -- --username <github-username>

# Example
cargo run --release -- --username torvalds
```

### Command Line Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--username` | `-u` | GitHub username to analyze | (required) |
| `--format` | `-f` | Output format: `text`, `json`, `markdown` | `text` |
| `--output` | `-o` | Write to file instead of stdout | stdout |
| `--max-commits-per-repo` | | Limit commits analyzed per repo | `50` |
| `--include-forks` | | Include forked repositories | `false` |
| `--database` | | SQLite database path | `gitanalyzer.db` |
| `--cached` | | Use cached profile if available | `false` |

### Examples

```bash
# Quick test with limited commits
cargo run --release -- -u octocat --max-commits-per-repo 5

# Full analysis with JSON output
cargo run --release -- -u gaearon -f json -o dan_abramov.json

# Markdown report saved to file
cargo run --release -- -u antirez -f markdown -o antirez_profile.md

# Use cached results (skip API calls if already analyzed)
cargo run --release -- -u torvalds --cached

# Include forked repositories
cargo run --release -- -u octocat --include-forks
```

## Output

### Text Format (Default)

```
=== Profile Analysis: octocat ===

Name: The Octocat
Commits analyzed: 142
Repositories: 8
Experience Level: Mid-Level

Top Skills:
  - Ruby (Language): 78/100 (confidence: 85%)
  - JavaScript (Language): 72/100 (confidence: 80%)
  - Git (Tool): 68/100 (confidence: 75%)

Primary Languages: Ruby, JavaScript

Strengths:
  + Ruby: Strong Language proficiency with 45 commits
  + Code Quality: Consistently high code quality (avg: 7.2/10)

Areas for Improvement:
  - Testing: Low test coverage across commits (18%)
  - Documentation: Limited documentation quality (avg: 4.5/10)

Coding Style:
  Tests: 18%
  Documentation: 45%
  Follows Conventions: 72%

Analyzed on: 2025-01-15 10:30:45 UTC
```

### JSON Format

```json
{
  "user": {
    "login": "octocat",
    "name": "The Octocat",
    "bio": "...",
    ...
  },
  "skills": [
    {
      "skill": { "name": "Ruby", "category": "Language" },
      "proficiency_score": 78,
      "confidence": 0.85,
      "trend": "Stable",
      "evidence": { "commit_count": 45, ... }
    }
  ],
  "summary": {
    "strengths": [...],
    "weaknesses": [...],
    "experience_level": "Mid"
  }
}
```

## How It Works

1. **Fetch Data**: Retrieves user profile, repositories, and commits from GitHub API
2. **Extract Diffs**: Downloads full commit diffs for analysis
3. **Batch Processing**: Groups commits into batches that fit Claude's context window
4. **AI Analysis**: Claude analyzes code patterns, skills, complexity, and quality
5. **Skill Aggregation**: Combines insights across all commits
6. **Rating Calculation**: Computes proficiency scores using weighted formula:
   - Frequency (15%): How often the skill appears
   - Recency (15%): How recently the skill was used
   - Complexity (20%): Sophistication of the code
   - Quality (20%): Code quality indicators
   - Consistency (10%): Regular usage over time
   - LLM Assessment (20%): Claude's proficiency evaluation
7. **Report Generation**: Produces formatted output with insights

## Project Structure

```
gitanalyzer/
├── Cargo.toml          # Dependencies
├── .env.example        # Environment template
├── src/
│   ├── main.rs         # CLI entry point
│   ├── lib.rs          # Library exports
│   ├── config.rs       # Configuration
│   ├── error.rs        # Error types
│   ├── models/         # Data structures
│   ├── github/         # GitHub API client
│   ├── llm/            # Claude integration
│   ├── analysis/       # Skill extraction & rating
│   ├── taxonomy/       # Skill definitions
│   └── storage/        # SQLite persistence
```

## Rate Limits

- **GitHub API**: 5,000 requests/hour with authentication
- **Anthropic API**: Varies by plan; tool implements soft rate limiting

The tool automatically handles rate limiting and will wait/retry as needed.

## Troubleshooting

### "GITHUB_TOKEN environment variable not set"
Ensure your `.env` file exists and contains `GITHUB_TOKEN=ghp_...`

### "User not found"
Check that the username exists on GitHub and is spelled correctly.

### "Rate limit exceeded"
Wait for the rate limit to reset (usually 1 hour for GitHub) or reduce `--max-commits-per-repo`.

### Build errors
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

## License

MIT

## Contributing

Contributions welcome! Please open an issue or submit a pull request.
