use rusqlite::{Connection, params};
use std::path::Path;

use crate::error::Result;
use crate::models::{UserProfile, SkillRating};

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let storage = Self { conn };
        storage.init_db()?;
        Ok(storage)
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let storage = Self { conn };
        storage.init_db()?;
        Ok(storage)
    }

    fn init_db(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                name TEXT,
                avatar_url TEXT,
                bio TEXT,
                company TEXT,
                location TEXT,
                public_repos INTEGER,
                followers INTEGER,
                created_at TEXT
            );

            CREATE TABLE IF NOT EXISTS profiles (
                id INTEGER PRIMARY KEY,
                user_id INTEGER NOT NULL REFERENCES users(id),
                total_commits_analyzed INTEGER,
                analysis_date TEXT NOT NULL,
                summary_json TEXT,
                UNIQUE(user_id)
            );

            CREATE TABLE IF NOT EXISTS skills (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                category TEXT NOT NULL,
                UNIQUE(name, category)
            );

            CREATE TABLE IF NOT EXISTS skill_ratings (
                id INTEGER PRIMARY KEY,
                profile_id INTEGER NOT NULL REFERENCES profiles(id),
                skill_id INTEGER NOT NULL REFERENCES skills(id),
                proficiency_score INTEGER NOT NULL,
                percentile_rank INTEGER,
                confidence REAL NOT NULL,
                trend TEXT,
                evidence_json TEXT,
                UNIQUE(profile_id, skill_id)
            );

            CREATE INDEX IF NOT EXISTS idx_profiles_user_id ON profiles(user_id);
            CREATE INDEX IF NOT EXISTS idx_skill_ratings_profile_id ON skill_ratings(profile_id);
            CREATE INDEX IF NOT EXISTS idx_skill_ratings_skill_id ON skill_ratings(skill_id);
            "#,
        )?;

        Ok(())
    }

    pub fn save_profile(&self, profile: &UserProfile) -> Result<()> {
        // Insert or update user
        self.conn.execute(
            r#"
            INSERT INTO users (username, name, avatar_url, bio, company, location, public_repos, followers, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(username) DO UPDATE SET
                name = excluded.name,
                avatar_url = excluded.avatar_url,
                bio = excluded.bio,
                company = excluded.company,
                location = excluded.location,
                public_repos = excluded.public_repos,
                followers = excluded.followers
            "#,
            params![
                profile.user.login,
                profile.user.name,
                profile.user.avatar_url,
                profile.user.bio,
                profile.user.company,
                profile.user.location,
                profile.user.public_repos,
                profile.user.followers,
                profile.user.created_at.to_rfc3339(),
            ],
        )?;

        let user_id: i64 = self.conn.query_row(
            "SELECT id FROM users WHERE username = ?1",
            params![profile.user.login],
            |row| row.get(0),
        )?;

        // Insert or update profile
        let summary_json = serde_json::to_string(&profile.summary)?;
        self.conn.execute(
            r#"
            INSERT INTO profiles (user_id, total_commits_analyzed, analysis_date, summary_json)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(user_id) DO UPDATE SET
                total_commits_analyzed = excluded.total_commits_analyzed,
                analysis_date = excluded.analysis_date,
                summary_json = excluded.summary_json
            "#,
            params![
                user_id,
                profile.total_commits_analyzed,
                profile.analysis_date.to_rfc3339(),
                summary_json,
            ],
        )?;

        let profile_id: i64 = self.conn.query_row(
            "SELECT id FROM profiles WHERE user_id = ?1",
            params![user_id],
            |row| row.get(0),
        )?;

        // Clear existing skill ratings for this profile
        self.conn.execute(
            "DELETE FROM skill_ratings WHERE profile_id = ?1",
            params![profile_id],
        )?;

        // Insert skill ratings
        for rating in &profile.skills {
            // Insert or get skill
            self.conn.execute(
                r#"
                INSERT OR IGNORE INTO skills (name, category)
                VALUES (?1, ?2)
                "#,
                params![rating.skill.name, rating.skill.category.to_string()],
            )?;

            let skill_id: i64 = self.conn.query_row(
                "SELECT id FROM skills WHERE name = ?1 AND category = ?2",
                params![rating.skill.name, rating.skill.category.to_string()],
                |row| row.get(0),
            )?;

            let evidence_json = serde_json::to_string(&rating.evidence)?;
            self.conn.execute(
                r#"
                INSERT INTO skill_ratings (profile_id, skill_id, proficiency_score, percentile_rank, confidence, trend, evidence_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
                params![
                    profile_id,
                    skill_id,
                    rating.proficiency_score,
                    rating.percentile_rank,
                    rating.confidence,
                    rating.trend.to_string(),
                    evidence_json,
                ],
            )?;
        }

        Ok(())
    }

    pub fn get_profile(&self, username: &str) -> Result<Option<UserProfile>> {
        let result = self.conn.query_row(
            r#"
            SELECT p.id, p.total_commits_analyzed, p.analysis_date, p.summary_json,
                   u.username, u.name, u.avatar_url, u.bio, u.company, u.location,
                   u.public_repos, u.followers, u.created_at, u.id as github_id
            FROM profiles p
            JOIN users u ON p.user_id = u.id
            WHERE u.username = ?1
            "#,
            params![username],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,      // profile_id
                    row.get::<_, u32>(1)?,      // total_commits_analyzed
                    row.get::<_, String>(2)?,   // analysis_date
                    row.get::<_, String>(3)?,   // summary_json
                    row.get::<_, String>(4)?,   // username
                    row.get::<_, Option<String>>(5)?, // name
                    row.get::<_, String>(6)?,   // avatar_url
                    row.get::<_, Option<String>>(7)?, // bio
                    row.get::<_, Option<String>>(8)?, // company
                    row.get::<_, Option<String>>(9)?, // location
                    row.get::<_, u32>(10)?,     // public_repos
                    row.get::<_, u32>(11)?,     // followers
                    row.get::<_, String>(12)?,  // created_at
                    row.get::<_, u64>(13)?,     // github_id
                ))
            },
        );

        match result {
            Ok((profile_id, total_commits, analysis_date_str, summary_json, username, name, avatar_url, bio, company, location, public_repos, followers, created_at_str, github_id)) => {
                let user = crate::models::GitHubUser {
                    login: username,
                    id: github_id,
                    name,
                    email: None,
                    avatar_url,
                    bio,
                    company,
                    location,
                    public_repos,
                    followers,
                    following: 0,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                };

                let summary = serde_json::from_str(&summary_json).unwrap_or_default();
                let analysis_date = chrono::DateTime::parse_from_rfc3339(&analysis_date_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now());

                // Fetch skill ratings
                let skills = self.get_skill_ratings(profile_id)?;

                Ok(Some(UserProfile {
                    user,
                    repositories: Vec::new(), // Not stored in DB currently
                    total_commits_analyzed: total_commits,
                    analysis_date,
                    skills,
                    summary,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn get_skill_ratings(&self, profile_id: i64) -> Result<Vec<SkillRating>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT s.name, s.category, sr.proficiency_score, sr.percentile_rank,
                   sr.confidence, sr.trend, sr.evidence_json
            FROM skill_ratings sr
            JOIN skills s ON sr.skill_id = s.id
            WHERE sr.profile_id = ?1
            ORDER BY sr.proficiency_score DESC
            "#,
        )?;

        let ratings = stmt.query_map(params![profile_id], |row| {
            let name: String = row.get(0)?;
            let category_str: String = row.get(1)?;
            let proficiency_score: u8 = row.get(2)?;
            let percentile_rank: Option<u8> = row.get(3)?;
            let confidence: f32 = row.get(4)?;
            let trend_str: String = row.get(5)?;
            let evidence_json: String = row.get(6)?;

            let category = match category_str.as_str() {
                "Language" => crate::models::skill::SkillCategory::Language,
                "Framework" => crate::models::skill::SkillCategory::Framework,
                "Library" => crate::models::skill::SkillCategory::Library,
                "Tool" => crate::models::skill::SkillCategory::Tool,
                "Domain" => crate::models::skill::SkillCategory::Domain,
                "Practice" => crate::models::skill::SkillCategory::Practice,
                _ => crate::models::skill::SkillCategory::Concept,
            };

            let trend = match trend_str.as_str() {
                "Improving" => crate::models::skill::SkillTrend::Improving,
                "Stable" => crate::models::skill::SkillTrend::Stable,
                "Declining" => crate::models::skill::SkillTrend::Declining,
                "New" => crate::models::skill::SkillTrend::New,
                _ => crate::models::skill::SkillTrend::Dormant,
            };

            let evidence: crate::models::skill::SkillEvidence =
                serde_json::from_str(&evidence_json).unwrap_or_default();

            Ok(SkillRating {
                skill: crate::models::skill::Skill {
                    id: name.to_lowercase().replace(' ', "_"),
                    name,
                    category,
                    subcategory: None,
                    aliases: Vec::new(),
                },
                proficiency_score,
                percentile_rank,
                confidence,
                evidence,
                trend,
            })
        })?;

        ratings.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn list_profiles(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT u.username FROM profiles p JOIN users u ON p.user_id = u.id ORDER BY p.analysis_date DESC",
        )?;

        let usernames = stmt.query_map([], |row| row.get(0))?;
        usernames.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_percentile(&self, skill_name: &str, score: u8) -> Result<Option<u8>> {
        let result = self.conn.query_row(
            r#"
            SELECT COUNT(*) as total,
                   SUM(CASE WHEN sr.proficiency_score < ?1 THEN 1 ELSE 0 END) as below
            FROM skill_ratings sr
            JOIN skills s ON sr.skill_id = s.id
            WHERE s.name = ?2
            "#,
            params![score, skill_name],
            |row| {
                let total: i64 = row.get(0)?;
                let below: i64 = row.get(1)?;
                Ok((total, below))
            },
        );

        match result {
            Ok((total, below)) if total > 0 => {
                let percentile = ((below as f64 / total as f64) * 100.0).round() as u8;
                Ok(Some(percentile))
            }
            Ok(_) => Ok(None),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
