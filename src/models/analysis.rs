use serde::{Deserialize, Serialize};
use super::skill::SkillDomain;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSummary {
    pub primary_languages: Vec<String>,
    pub primary_domains: Vec<SkillDomain>,
    pub strengths: Vec<StrengthWeakness>,
    pub weaknesses: Vec<StrengthWeakness>,
    pub experience_level: ExperienceLevel,
    pub coding_style: CodingStyle,
}

impl Default for ProfileSummary {
    fn default() -> Self {
        Self {
            primary_languages: Vec::new(),
            primary_domains: Vec::new(),
            strengths: Vec::new(),
            weaknesses: Vec::new(),
            experience_level: ExperienceLevel::Mid,
            coding_style: CodingStyle::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrengthWeakness {
    pub area: String,
    pub description: String,
    pub evidence: Vec<String>,
    pub score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExperienceLevel {
    Junior,
    Mid,
    Senior,
    Staff,
    Principal,
}

impl std::fmt::Display for ExperienceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExperienceLevel::Junior => write!(f, "Junior"),
            ExperienceLevel::Mid => write!(f, "Mid-Level"),
            ExperienceLevel::Senior => write!(f, "Senior"),
            ExperienceLevel::Staff => write!(f, "Staff"),
            ExperienceLevel::Principal => write!(f, "Principal"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodingStyle {
    pub prefers_small_commits: bool,
    pub writes_tests: f32,
    pub documents_code: f32,
    pub refactors_regularly: bool,
    pub follows_conventions: f32,
}

impl Default for CodingStyle {
    fn default() -> Self {
        Self {
            prefers_small_commits: true,
            writes_tests: 0.0,
            documents_code: 0.0,
            refactors_regularly: false,
            follows_conventions: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMAnalysisResult {
    pub skills: Vec<ExtractedSkill>,
    pub patterns: Vec<DetectedPattern>,
    pub complexity_assessment: ComplexityAssessment,
    pub quality_assessment: QualityAssessment,
    pub domain_signals: Vec<String>,
    pub notable_aspects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedSkill {
    pub name: String,
    pub category: String,
    pub proficiency_level: String,
    pub confidence: f32,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    #[serde(rename = "type")]
    pub pattern_type: String,
    pub name: String,
    pub description: String,
    pub quality_impact: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityAssessment {
    pub overall_score: u8,
    pub algorithmic_complexity: u8,
    pub architectural_complexity: u8,
    pub reasoning: String,
}

impl Default for ComplexityAssessment {
    fn default() -> Self {
        Self {
            overall_score: 5,
            algorithmic_complexity: 5,
            architectural_complexity: 5,
            reasoning: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAssessment {
    pub code_quality: u8,
    pub testing_coverage: f32,
    pub documentation_quality: u8,
    pub error_handling: u8,
    pub observations: Vec<String>,
}

impl Default for QualityAssessment {
    fn default() -> Self {
        Self {
            code_quality: 5,
            testing_coverage: 0.0,
            documentation_quality: 5,
            error_handling: 5,
            observations: Vec::new(),
        }
    }
}
