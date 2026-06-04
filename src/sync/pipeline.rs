use std::sync::Arc;

use tokio::sync::Mutex;

use super::sources::SkillSource;
use crate::db::SkillRepo;

pub struct SyncPipeline {
    repo: Arc<Mutex<SkillRepo>>,
    sources: Vec<Box<dyn SkillSource>>,
}

impl SyncPipeline {
    pub fn new(repo: SkillRepo) -> Self {
        SyncPipeline {
            repo: Arc::new(Mutex::new(repo)),
            sources: Vec::new(),
        }
    }

    pub fn add_source(&mut self, source: Box<dyn SkillSource>) {
        self.sources.push(source);
    }

    pub async fn sync_all(&self) -> Vec<SyncOutcome> {
        let mut outcomes = Vec::new();
        for source in &self.sources {
            outcomes.push(self.sync_source(source.as_ref()).await);
        }
        outcomes
    }

    async fn sync_source(&self, source: &dyn SkillSource) -> SyncOutcome {
        let name = source.name().to_string();
        match source.fetch().await {
            Ok(skills) => {
                let repo = self.repo.lock().await;
                match repo.insert_batch(&skills) {
                    Ok(n) => {
                        let _ = repo.record_sync(&name, "success", n as i64, 0, None);
                        SyncOutcome {
                            source: name,
                            success: true,
                            skills_added: n,
                            error: None,
                        }
                    }
                    Err(e) => {
                        let _ = repo.record_sync(&name, "failed", 0, 0, Some(&e.to_string()));
                        SyncOutcome {
                            source: name,
                            success: false,
                            skills_added: 0,
                            error: Some(e.to_string()),
                        }
                    }
                }
            }
            Err(e) => SyncOutcome {
                source: name,
                success: false,
                skills_added: 0,
                error: Some(e.to_string()),
            },
        }
    }
}

#[derive(Debug)]
pub struct SyncOutcome {
    pub source: String,
    pub success: bool,
    pub skills_added: usize,
    pub error: Option<String>,
}
