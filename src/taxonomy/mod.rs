pub mod languages;

use std::collections::HashMap;
use crate::models::skill::{Skill, SkillCategory};

pub use languages::detect_language;

pub struct SkillTaxonomy {
    skills: HashMap<String, Skill>,
    aliases: HashMap<String, String>,
}

impl SkillTaxonomy {
    pub fn new() -> Self {
        let mut taxonomy = Self {
            skills: HashMap::new(),
            aliases: HashMap::new(),
        };

        taxonomy.init_languages();
        taxonomy.init_frameworks();
        taxonomy.init_tools();
        taxonomy.init_domains();
        taxonomy.init_practices();

        taxonomy
    }

    fn init_languages(&mut self) {
        let languages = vec![
            ("rust", vec!["rs"]),
            ("python", vec!["py", "python3"]),
            ("javascript", vec!["js", "ecmascript", "es6", "es2015"]),
            ("typescript", vec!["ts"]),
            ("go", vec!["golang"]),
            ("java", vec![]),
            ("kotlin", vec!["kt"]),
            ("swift", vec![]),
            ("c", vec![]),
            ("cpp", vec!["c++", "cxx"]),
            ("csharp", vec!["c#", "cs"]),
            ("ruby", vec!["rb"]),
            ("php", vec![]),
            ("scala", vec![]),
            ("haskell", vec!["hs"]),
            ("elixir", vec!["ex"]),
            ("sql", vec!["plsql", "tsql"]),
            ("shell", vec!["bash", "sh", "zsh"]),
        ];

        for (name, aliases) in languages {
            self.add_skill(name, SkillCategory::Language, &aliases);
        }
    }

    fn init_frameworks(&mut self) {
        let frameworks = vec![
            // Frontend
            ("react", vec!["reactjs", "react.js"]),
            ("vue", vec!["vuejs", "vue.js"]),
            ("angular", vec!["angularjs"]),
            ("svelte", vec!["sveltekit"]),
            ("nextjs", vec!["next.js", "next"]),
            ("nuxt", vec!["nuxtjs", "nuxt.js"]),
            // Backend
            ("express", vec!["expressjs"]),
            ("django", vec![]),
            ("flask", vec![]),
            ("fastapi", vec![]),
            ("spring", vec!["spring boot", "springboot"]),
            ("rails", vec!["ruby on rails", "ror"]),
            ("actix", vec!["actix-web"]),
            ("axum", vec![]),
            ("rocket", vec![]),
            ("gin", vec![]),
            ("echo", vec![]),
            // Mobile
            ("react native", vec!["react-native", "rn"]),
            ("flutter", vec![]),
            ("swiftui", vec![]),
        ];

        for (name, aliases) in frameworks {
            self.add_skill(name, SkillCategory::Framework, &aliases);
        }
    }

    fn init_tools(&mut self) {
        let tools = vec![
            ("docker", vec!["dockerfile", "containerization"]),
            ("kubernetes", vec!["k8s"]),
            ("terraform", vec!["tf", "iac"]),
            ("aws", vec!["amazon web services"]),
            ("gcp", vec!["google cloud", "google cloud platform"]),
            ("azure", vec!["microsoft azure"]),
            ("git", vec![]),
            ("github actions", vec!["gha"]),
            ("gitlab ci", vec!["gitlab-ci"]),
            ("jenkins", vec![]),
            ("postgresql", vec!["postgres", "psql"]),
            ("mysql", vec!["mariadb"]),
            ("mongodb", vec!["mongo"]),
            ("redis", vec![]),
            ("elasticsearch", vec!["elastic", "es"]),
            ("graphql", vec!["gql"]),
            ("rest api", vec!["restful", "rest"]),
        ];

        for (name, aliases) in tools {
            self.add_skill(name, SkillCategory::Tool, &aliases);
        }
    }

    fn init_domains(&mut self) {
        let domains = vec![
            ("machine learning", vec!["ml", "deep learning", "dl", "ai"]),
            ("data science", vec!["data analysis", "analytics"]),
            ("devops", vec!["sre", "platform engineering"]),
            ("security", vec!["cybersecurity", "infosec", "appsec"]),
            ("frontend", vec!["front-end", "ui", "client-side"]),
            ("backend", vec!["back-end", "server-side"]),
            ("fullstack", vec!["full-stack", "full stack"]),
            ("mobile", vec!["ios", "android", "mobile development"]),
            ("embedded", vec!["embedded systems", "iot"]),
            ("distributed systems", vec!["microservices", "distributed"]),
            ("databases", vec!["database design", "data modeling"]),
        ];

        for (name, aliases) in domains {
            self.add_skill(name, SkillCategory::Domain, &aliases);
        }
    }

    fn init_practices(&mut self) {
        let practices = vec![
            ("testing", vec!["unit testing", "tdd", "test-driven", "integration testing"]),
            ("documentation", vec!["docs", "technical writing"]),
            ("code review", vec!["pr review", "pull request review"]),
            ("ci/cd", vec!["continuous integration", "continuous deployment", "continuous delivery"]),
            ("agile", vec!["scrum", "kanban"]),
            ("clean code", vec!["solid", "dry", "kiss"]),
            ("refactoring", vec![]),
            ("debugging", vec!["troubleshooting"]),
            ("performance optimization", vec!["perf", "optimization"]),
            ("error handling", vec!["exception handling"]),
        ];

        for (name, aliases) in practices {
            self.add_skill(name, SkillCategory::Practice, &aliases);
        }
    }

    fn add_skill(&mut self, name: &str, category: SkillCategory, aliases: &[&str]) {
        let skill = Skill {
            id: name.to_lowercase().replace(' ', "_"),
            name: name.to_string(),
            category,
            subcategory: None,
            aliases: aliases.iter().map(|s| s.to_string()).collect(),
        };

        self.skills.insert(name.to_lowercase(), skill);

        for alias in aliases {
            self.aliases
                .insert(alias.to_lowercase(), name.to_lowercase());
        }
    }

    pub fn normalize_skill_name(&self, name: &str) -> String {
        let lower = name.to_lowercase();
        self.aliases.get(&lower).cloned().unwrap_or(lower)
    }

    pub fn categorize(&self, category_str: &str) -> SkillCategory {
        match category_str.to_lowercase().as_str() {
            "language" => SkillCategory::Language,
            "framework" => SkillCategory::Framework,
            "library" => SkillCategory::Library,
            "tool" => SkillCategory::Tool,
            "domain" => SkillCategory::Domain,
            "practice" => SkillCategory::Practice,
            _ => SkillCategory::Concept,
        }
    }

    pub fn get_or_create_skill(&self, name: &str, category: SkillCategory) -> Skill {
        let normalized = self.normalize_skill_name(name);
        self.skills.get(&normalized).cloned().unwrap_or_else(|| Skill {
            id: normalized.replace(' ', "_"),
            name: name.to_string(),
            category,
            subcategory: None,
            aliases: Vec::new(),
        })
    }

    pub fn get_skill(&self, name: &str) -> Option<&Skill> {
        let normalized = self.normalize_skill_name(name);
        self.skills.get(&normalized)
    }
}

impl Default for SkillTaxonomy {
    fn default() -> Self {
        Self::new()
    }
}
