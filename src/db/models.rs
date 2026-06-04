use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub source: String,
    pub source_url: Option<String>,
    pub version: String,
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
    pub quality_score: f64,
    pub download_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    pub category: Option<String>,
    pub source: Option<String>,
    pub tags: Vec<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub skill: Skill,
    pub relevance: f64,
    pub match_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSource {
    pub name: String,
    pub url: String,
    pub source_type: SourceType,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    GitHub,
    Http,
    LocalFile,
}

impl Skill {
    pub fn new(
        name: String,
        description: String,
        category: String,
        source: String,
        version: String,
        tags: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        Skill {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            category,
            source,
            source_url: None,
            version,
            tags,
            metadata: serde_json::Value::Object(Default::default()),
            quality_score: 0.0,
            download_count: 0,
            created_at: now,
            updated_at: now,
        }
    }
}
