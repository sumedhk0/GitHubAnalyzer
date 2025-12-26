use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRating {
    pub skill: Skill,
    pub proficiency_score: u8,
    pub percentile_rank: Option<u8>,
    pub confidence: f32,
    pub evidence: SkillEvidence,
    pub trend: SkillTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub category: SkillCategory,
    pub subcategory: Option<String>,
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SkillCategory {
    Language,
    Framework,
    Library,
    Tool,
    Domain,
    Practice,
    Concept,
}

impl std::fmt::Display for SkillCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillCategory::Language => write!(f, "Language"),
            SkillCategory::Framework => write!(f, "Framework"),
            SkillCategory::Library => write!(f, "Library"),
            SkillCategory::Tool => write!(f, "Tool"),
            SkillCategory::Domain => write!(f, "Domain"),
            SkillCategory::Practice => write!(f, "Practice"),
            SkillCategory::Concept => write!(f, "Concept"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SkillDomain {
    Frontend,
    Backend,
    FullStack,
    Mobile,
    DevOps,
    DataScience,
    MachineLearning,
    Security,
    Database,
    Cloud,
    Embedded,
    SystemsProgramming,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEvidence {
    pub commit_count: u32,
    pub total_lines_changed: u32,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub repositories: Vec<String>,
}

impl Default for SkillEvidence {
    fn default() -> Self {
        Self {
            commit_count: 0,
            total_lines_changed: 0,
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            repositories: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SkillTrend {
    Improving,
    Stable,
    Declining,
    New,
    Dormant,
}

impl std::fmt::Display for SkillTrend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillTrend::Improving => write!(f, "Improving"),
            SkillTrend::Stable => write!(f, "Stable"),
            SkillTrend::Declining => write!(f, "Declining"),
            SkillTrend::New => write!(f, "New"),
            SkillTrend::Dormant => write!(f, "Dormant"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkillOccurrence {
    pub commit_sha: String,
    pub repository: String,
    pub timestamp: DateTime<Utc>,
    pub evidence: Vec<String>,
    pub proficiency_signal: String,
    pub confidence: f32,
    pub lines_changed: u32,
}

#[derive(Debug, Clone)]
pub struct AggregatedSkill {
    pub skill: Skill,
    pub occurrences: Vec<SkillOccurrence>,
    pub total_lines: u32,
    pub complexity_scores: Vec<f32>,
    pub quality_scores: Vec<f32>,
}

impl AggregatedSkill {
    pub fn new(skill: Skill) -> Self {
        Self {
            skill,
            occurrences: Vec::new(),
            total_lines: 0,
            complexity_scores: Vec::new(),
            quality_scores: Vec::new(),
        }
    }

    pub fn repositories(&self) -> Vec<String> {
        self.occurrences
            .iter()
            .map(|o| o.repository.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }
}
