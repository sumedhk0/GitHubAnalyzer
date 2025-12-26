use std::collections::HashMap;
use chrono::Utc;

use crate::models::analysis::LLMAnalysisResult;
use crate::models::commit::CommitForAnalysis;
use crate::models::skill::{AggregatedSkill, Skill, SkillOccurrence};
use crate::taxonomy::SkillTaxonomy;

pub struct SkillExtractor {
    taxonomy: SkillTaxonomy,
}

impl SkillExtractor {
    pub fn new() -> Self {
        Self {
            taxonomy: SkillTaxonomy::new(),
        }
    }

    pub fn aggregate_skills(
        &self,
        analyses: &[(LLMAnalysisResult, CommitForAnalysis)],
    ) -> HashMap<String, AggregatedSkill> {
        let mut skill_map: HashMap<String, AggregatedSkill> = HashMap::new();

        for (analysis, commit) in analyses {
            let lines_changed = commit.stats.additions + commit.stats.deletions;

            for extracted in &analysis.skills {
                let normalized_name = self.taxonomy.normalize_skill_name(&extracted.name);
                let category = self.taxonomy.categorize(&extracted.category);

                let skill = self.taxonomy.get_or_create_skill(&extracted.name, category);

                let occurrence = SkillOccurrence {
                    commit_sha: commit.sha.clone(),
                    repository: commit.repository.clone(),
                    timestamp: commit.committed_at,
                    evidence: extracted.evidence.clone(),
                    proficiency_signal: extracted.proficiency_level.clone(),
                    confidence: extracted.confidence,
                    lines_changed,
                };

                let entry = skill_map
                    .entry(normalized_name)
                    .or_insert_with(|| AggregatedSkill::new(skill));

                entry.occurrences.push(occurrence);
                entry.total_lines += lines_changed;
                entry.complexity_scores.push(analysis.complexity_assessment.overall_score as f32);
                entry.quality_scores.push(analysis.quality_assessment.code_quality as f32);
            }
        }

        skill_map
    }

    pub fn extract_domain_signals(
        &self,
        analyses: &[LLMAnalysisResult],
    ) -> HashMap<String, u32> {
        let mut domain_counts: HashMap<String, u32> = HashMap::new();

        for analysis in analyses {
            for domain in &analysis.domain_signals {
                *domain_counts.entry(domain.to_lowercase()).or_insert(0) += 1;
            }
        }

        domain_counts
    }

    pub fn compute_average_quality(
        &self,
        analyses: &[LLMAnalysisResult],
    ) -> (f32, f32, f32) {
        if analyses.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let count = analyses.len() as f32;

        let avg_testing: f32 = analyses
            .iter()
            .map(|a| a.quality_assessment.testing_coverage)
            .sum::<f32>()
            / count;

        let avg_docs: f32 = analyses
            .iter()
            .map(|a| a.quality_assessment.documentation_quality as f32)
            .sum::<f32>()
            / count;

        let avg_quality: f32 = analyses
            .iter()
            .map(|a| a.quality_assessment.code_quality as f32)
            .sum::<f32>()
            / count;

        (avg_testing, avg_docs, avg_quality)
    }
}

impl Default for SkillExtractor {
    fn default() -> Self {
        Self::new()
    }
}
