use std::collections::HashMap;
use chrono::{Duration, Utc};

use crate::models::analysis::{
    CodingStyle, ExperienceLevel, LLMAnalysisResult, ProfileSummary, StrengthWeakness,
};
use crate::models::skill::{
    AggregatedSkill, SkillCategory, SkillDomain, SkillEvidence, SkillOccurrence, SkillRating,
    SkillTrend,
};

pub struct RatingEngine {
    weights: RatingWeights,
}

#[derive(Debug, Clone)]
pub struct RatingWeights {
    pub frequency_weight: f32,
    pub recency_weight: f32,
    pub complexity_weight: f32,
    pub quality_weight: f32,
    pub consistency_weight: f32,
    pub proficiency_weight: f32,
}

impl Default for RatingWeights {
    fn default() -> Self {
        Self {
            frequency_weight: 0.15,
            recency_weight: 0.15,
            complexity_weight: 0.20,
            quality_weight: 0.20,
            consistency_weight: 0.10,
            proficiency_weight: 0.20,
        }
    }
}

impl RatingEngine {
    pub fn new() -> Self {
        Self {
            weights: RatingWeights::default(),
        }
    }

    pub fn calculate_ratings(
        &self,
        aggregated_skills: &HashMap<String, AggregatedSkill>,
    ) -> Vec<SkillRating> {
        let mut ratings: Vec<SkillRating> = aggregated_skills
            .values()
            .map(|agg| self.calculate_single_rating(agg))
            .collect();

        // Sort by proficiency score (descending)
        ratings.sort_by(|a, b| b.proficiency_score.cmp(&a.proficiency_score));

        ratings
    }

    fn calculate_single_rating(&self, agg: &AggregatedSkill) -> SkillRating {
        let now = Utc::now();

        // 1. Frequency score (normalized by log scale, max at ~100 occurrences)
        let frequency_score = ((agg.occurrences.len() as f32).ln() + 1.0).min(5.0) / 5.0 * 100.0;

        // 2. Recency score
        let most_recent = agg
            .occurrences
            .iter()
            .map(|o| o.timestamp)
            .max()
            .unwrap_or(now);
        let days_since = (now - most_recent).num_days().max(0) as f32;
        let recency_score = (1.0 - (days_since / 365.0).min(1.0)) * 100.0;

        // 3. Complexity score (average of LLM assessments, scaled to 100)
        let complexity_score = if agg.complexity_scores.is_empty() {
            50.0
        } else {
            agg.complexity_scores.iter().sum::<f32>() / agg.complexity_scores.len() as f32 * 10.0
        };

        // 4. Quality score (average of LLM assessments, scaled to 100)
        let quality_score = if agg.quality_scores.is_empty() {
            50.0
        } else {
            agg.quality_scores.iter().sum::<f32>() / agg.quality_scores.len() as f32 * 10.0
        };

        // 5. Consistency score (how regularly the skill is used)
        let consistency_score = self.calculate_consistency(&agg.occurrences);

        // 6. Proficiency score from LLM assessments
        let proficiency_score = self.calculate_proficiency_from_signals(&agg.occurrences);

        // Weighted combination
        let final_score = (frequency_score * self.weights.frequency_weight
            + recency_score * self.weights.recency_weight
            + complexity_score * self.weights.complexity_weight
            + quality_score * self.weights.quality_weight
            + consistency_score * self.weights.consistency_weight
            + proficiency_score * self.weights.proficiency_weight)
            .round() as u8;

        // Calculate confidence based on evidence quantity
        let confidence = (agg.occurrences.len() as f32 / 20.0).min(1.0);

        // Determine trend
        let trend = self.calculate_trend(&agg.occurrences);

        // Build evidence
        let first_seen = agg
            .occurrences
            .iter()
            .map(|o| o.timestamp)
            .min()
            .unwrap_or(now);

        let evidence = SkillEvidence {
            commit_count: agg.occurrences.len() as u32,
            total_lines_changed: agg.total_lines,
            first_seen,
            last_seen: most_recent,
            repositories: agg.repositories(),
        };

        SkillRating {
            skill: agg.skill.clone(),
            proficiency_score: final_score.max(1).min(100),
            percentile_rank: None,
            confidence,
            evidence,
            trend,
        }
    }

    fn calculate_proficiency_from_signals(&self, occurrences: &[SkillOccurrence]) -> f32 {
        if occurrences.is_empty() {
            return 50.0;
        }

        let level_scores: Vec<(f32, f32)> = occurrences
            .iter()
            .map(|o| {
                let score = match o.proficiency_signal.to_lowercase().as_str() {
                    "expert" => 95.0,
                    "advanced" => 80.0,
                    "intermediate" => 60.0,
                    "beginner" => 35.0,
                    _ => 50.0,
                };
                (score, o.confidence)
            })
            .collect();

        // Weighted average by confidence
        let total_weight: f32 = level_scores.iter().map(|(_, c)| c).sum();
        if total_weight == 0.0 {
            return 50.0;
        }

        let weighted_sum: f32 = level_scores.iter().map(|(s, c)| s * c).sum();
        weighted_sum / total_weight
    }

    fn calculate_consistency(&self, occurrences: &[SkillOccurrence]) -> f32 {
        if occurrences.len() < 2 {
            return 50.0;
        }

        let mut timestamps: Vec<_> = occurrences.iter().map(|o| o.timestamp).collect();
        timestamps.sort();

        let gaps: Vec<i64> = timestamps.windows(2).map(|w| (w[1] - w[0]).num_days()).collect();

        if gaps.is_empty() {
            return 50.0;
        }

        let avg_gap = gaps.iter().sum::<i64>() as f32 / gaps.len() as f32;
        // Good consistency = gaps of ~7 days or less
        // Poor consistency = gaps of 90+ days
        let consistency = (1.0 - (avg_gap / 90.0).min(1.0)) * 100.0;

        consistency.max(0.0)
    }

    fn calculate_trend(&self, occurrences: &[SkillOccurrence]) -> SkillTrend {
        let now = Utc::now();
        let six_months_ago = now - Duration::days(180);
        let one_year_ago = now - Duration::days(365);

        let recent_count = occurrences
            .iter()
            .filter(|o| o.timestamp > six_months_ago)
            .count();
        let older_count = occurrences
            .iter()
            .filter(|o| o.timestamp <= six_months_ago && o.timestamp > one_year_ago)
            .count();

        if occurrences.len() <= 2 {
            return SkillTrend::New;
        }

        if recent_count == 0 && older_count > 0 {
            return SkillTrend::Dormant;
        }

        let ratio = if older_count > 0 {
            recent_count as f32 / older_count as f32
        } else if recent_count > 0 {
            2.0 // Active recently with no older history = improving
        } else {
            1.0 // No activity = stable (shouldn't happen)
        };

        match ratio {
            r if r > 1.5 => SkillTrend::Improving,
            r if r < 0.5 => SkillTrend::Declining,
            _ => SkillTrend::Stable,
        }
    }

    pub fn generate_summary(
        &self,
        skill_ratings: &[SkillRating],
        analyses: &[LLMAnalysisResult],
    ) -> ProfileSummary {
        let primary_languages = self.extract_primary_languages(skill_ratings);
        let primary_domains = self.extract_primary_domains(analyses);
        let strengths = self.detect_strengths(skill_ratings, analyses);
        let weaknesses = self.detect_weaknesses(skill_ratings, analyses);
        let experience_level = self.assess_experience_level(skill_ratings);
        let coding_style = self.assess_coding_style(analyses);

        ProfileSummary {
            primary_languages,
            primary_domains,
            strengths,
            weaknesses,
            experience_level,
            coding_style,
        }
    }

    fn extract_primary_languages(&self, ratings: &[SkillRating]) -> Vec<String> {
        ratings
            .iter()
            .filter(|r| r.skill.category == SkillCategory::Language)
            .filter(|r| r.proficiency_score >= 40)
            .take(5)
            .map(|r| r.skill.name.clone())
            .collect()
    }

    fn extract_primary_domains(&self, analyses: &[LLMAnalysisResult]) -> Vec<SkillDomain> {
        let mut domain_counts: HashMap<String, u32> = HashMap::new();

        for analysis in analyses {
            for domain in &analysis.domain_signals {
                *domain_counts.entry(domain.to_lowercase()).or_insert(0) += 1;
            }
        }

        let mut domains: Vec<_> = domain_counts.into_iter().collect();
        domains.sort_by(|a, b| b.1.cmp(&a.1));

        domains
            .into_iter()
            .take(3)
            .filter_map(|(d, _)| match d.as_str() {
                "frontend" => Some(SkillDomain::Frontend),
                "backend" => Some(SkillDomain::Backend),
                "fullstack" | "full-stack" => Some(SkillDomain::FullStack),
                "mobile" => Some(SkillDomain::Mobile),
                "devops" => Some(SkillDomain::DevOps),
                "ml" | "machine learning" => Some(SkillDomain::MachineLearning),
                "data" | "data science" => Some(SkillDomain::DataScience),
                "security" => Some(SkillDomain::Security),
                "database" | "databases" => Some(SkillDomain::Database),
                "cloud" => Some(SkillDomain::Cloud),
                "embedded" => Some(SkillDomain::Embedded),
                "systems" => Some(SkillDomain::SystemsProgramming),
                _ => None,
            })
            .collect()
    }

    fn detect_strengths(
        &self,
        ratings: &[SkillRating],
        analyses: &[LLMAnalysisResult],
    ) -> Vec<StrengthWeakness> {
        let mut strengths = Vec::new();

        // High proficiency skills
        for rating in ratings.iter().filter(|r| r.proficiency_score >= 70) {
            strengths.push(StrengthWeakness {
                area: rating.skill.name.clone(),
                description: format!(
                    "Strong {} proficiency with {} commits",
                    rating.skill.category, rating.evidence.commit_count
                ),
                evidence: rating.evidence.repositories.clone(),
                score: rating.proficiency_score,
            });
        }

        // Good patterns detected
        let good_patterns: Vec<_> = analyses
            .iter()
            .flat_map(|a| a.patterns.iter())
            .filter(|p| p.quality_impact > 0.3)
            .collect();

        if !good_patterns.is_empty() {
            let pattern_names: Vec<_> = good_patterns.iter().map(|p| p.name.clone()).collect();
            strengths.push(StrengthWeakness {
                area: "Design Patterns".to_string(),
                description: "Uses good design patterns and practices".to_string(),
                evidence: pattern_names,
                score: 75,
            });
        }

        // High quality scores
        let avg_quality: f32 = analyses
            .iter()
            .map(|a| a.quality_assessment.code_quality as f32)
            .sum::<f32>()
            / analyses.len().max(1) as f32;

        if avg_quality >= 7.0 {
            strengths.push(StrengthWeakness {
                area: "Code Quality".to_string(),
                description: format!("Consistently high code quality (avg: {:.1}/10)", avg_quality),
                evidence: vec![],
                score: (avg_quality * 10.0) as u8,
            });
        }

        strengths.sort_by(|a, b| b.score.cmp(&a.score));
        strengths.truncate(5);
        strengths
    }

    fn detect_weaknesses(
        &self,
        ratings: &[SkillRating],
        analyses: &[LLMAnalysisResult],
    ) -> Vec<StrengthWeakness> {
        let mut weaknesses = Vec::new();

        // Low testing coverage
        let avg_testing: f32 = analyses
            .iter()
            .map(|a| a.quality_assessment.testing_coverage)
            .sum::<f32>()
            / analyses.len().max(1) as f32;

        if avg_testing < 0.3 {
            weaknesses.push(StrengthWeakness {
                area: "Testing".to_string(),
                description: format!(
                    "Low test coverage across commits ({:.0}%)",
                    avg_testing * 100.0
                ),
                evidence: vec![],
                score: (avg_testing * 100.0) as u8,
            });
        }

        // Low documentation
        let avg_docs: f32 = analyses
            .iter()
            .map(|a| a.quality_assessment.documentation_quality as f32)
            .sum::<f32>()
            / analyses.len().max(1) as f32;

        if avg_docs < 4.0 {
            weaknesses.push(StrengthWeakness {
                area: "Documentation".to_string(),
                description: format!("Limited documentation quality (avg: {:.1}/10)", avg_docs),
                evidence: vec![],
                score: (avg_docs * 10.0) as u8,
            });
        }

        // Declining skills
        for rating in ratings.iter().filter(|r| r.trend == SkillTrend::Declining) {
            weaknesses.push(StrengthWeakness {
                area: rating.skill.name.clone(),
                description: format!(
                    "{} usage declining over time",
                    rating.skill.name
                ),
                evidence: vec![format!(
                    "Last used: {}",
                    rating.evidence.last_seen.format("%Y-%m-%d")
                )],
                score: rating.proficiency_score,
            });
        }

        // Anti-patterns detected
        let anti_patterns: Vec<_> = analyses
            .iter()
            .flat_map(|a| a.patterns.iter())
            .filter(|p| p.quality_impact < -0.3)
            .collect();

        if !anti_patterns.is_empty() {
            let pattern_names: Vec<_> = anti_patterns.iter().map(|p| p.name.clone()).collect();
            weaknesses.push(StrengthWeakness {
                area: "Code Patterns".to_string(),
                description: "Some anti-patterns detected in code".to_string(),
                evidence: pattern_names,
                score: 30,
            });
        }

        weaknesses.sort_by(|a, b| a.score.cmp(&b.score));
        weaknesses.truncate(5);
        weaknesses
    }

    fn assess_experience_level(&self, ratings: &[SkillRating]) -> ExperienceLevel {
        // Calculate based on:
        // - Number of high-proficiency skills
        // - Duration of activity (first_seen to last_seen)
        // - Average proficiency

        let high_proficiency_count = ratings.iter().filter(|r| r.proficiency_score >= 70).count();
        let avg_proficiency: f32 = ratings.iter().map(|r| r.proficiency_score as f32).sum::<f32>()
            / ratings.len().max(1) as f32;

        let earliest = ratings
            .iter()
            .map(|r| r.evidence.first_seen)
            .min();
        let latest = ratings
            .iter()
            .map(|r| r.evidence.last_seen)
            .max();

        let years_active = match (earliest, latest) {
            (Some(e), Some(l)) => (l - e).num_days() as f32 / 365.0,
            _ => 0.0,
        };

        // Heuristic-based assessment
        match (high_proficiency_count, avg_proficiency as u8, years_active as u32) {
            (hp, avg, years) if hp >= 5 && avg >= 70 && years >= 5 => ExperienceLevel::Principal,
            (hp, avg, years) if hp >= 4 && avg >= 65 && years >= 4 => ExperienceLevel::Staff,
            (hp, avg, years) if hp >= 3 && avg >= 60 && years >= 2 => ExperienceLevel::Senior,
            (hp, avg, years) if hp >= 1 && avg >= 50 && years >= 1 => ExperienceLevel::Mid,
            _ => ExperienceLevel::Junior,
        }
    }

    fn assess_coding_style(&self, analyses: &[LLMAnalysisResult]) -> CodingStyle {
        if analyses.is_empty() {
            return CodingStyle::default();
        }

        let count = analyses.len() as f32;

        let writes_tests = analyses
            .iter()
            .map(|a| a.quality_assessment.testing_coverage)
            .sum::<f32>()
            / count;

        let documents_code = analyses
            .iter()
            .map(|a| a.quality_assessment.documentation_quality as f32 / 10.0)
            .sum::<f32>()
            / count;

        let follows_conventions = analyses
            .iter()
            .map(|a| a.quality_assessment.code_quality as f32 / 10.0)
            .sum::<f32>()
            / count;

        // Check for refactoring signals in commit messages / patterns
        let refactors_regularly = analyses
            .iter()
            .flat_map(|a| a.patterns.iter())
            .any(|p| p.name.to_lowercase().contains("refactor"));

        CodingStyle {
            prefers_small_commits: true, // Would need commit size analysis
            writes_tests,
            documents_code,
            refactors_regularly,
            follows_conventions,
        }
    }
}

impl Default for RatingEngine {
    fn default() -> Self {
        Self::new()
    }
}
