use async_trait::async_trait;

use crate::db::Skill;
use crate::error::Result;

#[async_trait]
pub trait SkillSource: Send + Sync {
    fn name(&self) -> &str;
    async fn fetch(&self) -> Result<Vec<Skill>>;
}

// ── GitHub Source ────────────────────────────────────────

pub struct GitHubSource {
    pub owner: String,
    pub repo: String,
    pub path: String,
    client: reqwest::Client,
}

impl GitHubSource {
    pub fn new(owner: &str, repo: &str, path: &str) -> Self {
        GitHubSource {
            owner: owner.to_string(),
            repo: repo.to_string(),
            path: path.to_string(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl SkillSource for GitHubSource {
    fn name(&self) -> &str {
        "github"
    }

    async fn fetch(&self) -> Result<Vec<Skill>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            self.owner, self.repo, self.path
        );
        let resp = self
            .client
            .get(&url)
            .header("User-Agent", "skills-dba/0.1")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(crate::error::Error::Sync(format!(
                "GitHub API returned {} for {}/{}",
                resp.status(),
                self.owner,
                self.repo
            )));
        }

        let entries: Vec<serde_json::Value> = resp.json().await?;
        let mut skills = Vec::new();

        for entry in entries {
            let name = entry["name"].as_str().unwrap_or("unknown");
            if !name.ends_with(".md") {
                continue;
            }
            let skill_name = name.trim_end_matches(".md").to_string();
            let download_url = entry["download_url"].as_str().unwrap_or("");
            let html_url = entry["html_url"].as_str().unwrap_or("");

            let content = self
                .client
                .get(download_url)
                .header("User-Agent", "skills-dba/0.1")
                .send()
                .await?
                .text()
                .await?;

            let description = extract_description(&content);
            let tags = extract_tags(&content);

            let mut skill = Skill::new(
                skill_name,
                description,
                "general".to_string(),
                format!("github:{}:{}", self.owner, self.repo),
                "0.1.0".to_string(),
                tags,
            );
            skill.source_url = Some(html_url.to_string());
            skills.push(skill);
        }

        Ok(skills)
    }
}

fn extract_description(content: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(desc) = trimmed.strip_prefix("description:") {
            return desc.trim().trim_matches('"').to_string();
        }
    }
    content
        .lines()
        .find(|l| {
            !l.trim().is_empty() && !l.trim().starts_with('#') && !l.trim().starts_with("---")
        })
        .unwrap_or("")
        .trim()
        .to_string()
}

fn extract_tags(content: &str) -> Vec<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(tags_str) = trimmed.strip_prefix("tags:") {
            return tags_str
                .split(',')
                .map(|t| {
                    t.trim()
                        .trim_matches('"')
                        .trim_matches('[')
                        .trim_matches(']')
                        .to_string()
                })
                .filter(|t| !t.is_empty())
                .collect();
        }
    }
    vec![]
}
