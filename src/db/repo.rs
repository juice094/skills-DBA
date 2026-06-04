use rusqlite::{params, Connection};

use super::models::{SearchQuery, SearchResult, Skill};
use crate::error::Result;

pub struct SkillRepo {
    conn: Connection,
}

impl SkillRepo {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        super::schema::initialize(&conn)?;
        Ok(SkillRepo { conn })
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        super::schema::initialize(&conn)?;
        Ok(SkillRepo { conn })
    }

    // ── CRUD ──────────────────────────────────────────────

    pub fn insert(&self, skill: &Skill) -> Result<()> {
        self.conn.execute(
            "INSERT INTO skills (id, name, description, category, source, source_url, version, tags, metadata, quality_score, download_count, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                skill.id,
                skill.name,
                skill.description,
                skill.category,
                skill.source,
                skill.source_url,
                skill.version,
                serde_json::to_string(&skill.tags).unwrap_or_default(),
                skill.metadata.to_string(),
                skill.quality_score,
                skill.download_count,
                skill.created_at.to_rfc3339(),
                skill.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn insert_batch(&self, skills: &[Skill]) -> Result<usize> {
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;
        for skill in skills {
            tx.execute(
                "INSERT OR REPLACE INTO skills (id, name, description, category, source, source_url, version, tags, metadata, quality_score, download_count, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    skill.id,
                    skill.name,
                    skill.description,
                    skill.category,
                    skill.source,
                    skill.source_url,
                    skill.version,
                    serde_json::to_string(&skill.tags).unwrap_or_default(),
                    skill.metadata.to_string(),
                    skill.quality_score,
                    skill.download_count,
                    skill.created_at.to_rfc3339(),
                    skill.updated_at.to_rfc3339(),
                ],
            )?;
            count += 1;
        }
        tx.commit()?;
        Ok(count)
    }

    pub fn get_by_id(&self, id: &str) -> Result<Skill> {
        let skill = self.conn.query_row(
            "SELECT id, name, description, category, source, source_url, version, tags, metadata,
                    quality_score, download_count, created_at, updated_at
             FROM skills WHERE id = ?1",
            params![id],
            row_to_skill_13,
        )?;
        Ok(skill)
    }

    pub fn delete(&self, id: &str) -> Result<bool> {
        let n = self
            .conn
            .execute("DELETE FROM skills WHERE id = ?1", params![id])?;
        Ok(n > 0)
    }

    pub fn count(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM skills", [], |r| r.get(0))?;
        Ok(count)
    }

    // ── FTS5 Search ───────────────────────────────────────

    pub fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let limit = query.limit.unwrap_or(20).min(100);
        let offset = query.offset.unwrap_or(0);
        let fts_query = build_fts_query(&query.text);

        let mut sql = String::from(
            "SELECT s.id, s.name, s.description, s.category, s.source, s.source_url,
                    s.version, s.tags, s.metadata, s.quality_score, s.download_count,
                    s.created_at, s.updated_at,
                    bm25(skills_fts, 0.0, 1.0, 0.5, 0.3) AS relevance
             FROM skills_fts f
             JOIN skills s ON s.rowid = f.rowid
             WHERE skills_fts MATCH ?1",
        );

        if query.category.is_some() {
            sql.push_str(" AND s.category = ?2");
        }
        if query.source.is_some() {
            sql.push_str(" AND s.source = ?3");
        }

        sql.push_str(" ORDER BY relevance ASC LIMIT ?4 OFFSET ?5");

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(
            params![
                fts_query,
                query.category,
                query.source,
                limit as i64,
                offset as i64
            ],
            |row| {
                let skill = row_to_skill_13(row)?;
                let relevance: f64 = row.get(13)?;
                Ok(SearchResult {
                    skill,
                    relevance,
                    match_fields: vec![],
                })
            },
        )?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    // ── Sync Sources ──────────────────────────────────────

    pub fn register_source(&self, name: &str, url: &str, source_type: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO sync_sources (name, url, source_type) VALUES (?1, ?2, ?3)",
            params![name, url, source_type],
        )?;
        Ok(())
    }

    pub fn list_sources(&self) -> Result<Vec<(String, String, String, bool)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, url, source_type, enabled FROM sync_sources ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, bool>(3)?,
            ))
        })?;
        let mut sources = Vec::new();
        for row in rows {
            sources.push(row?);
        }
        Ok(sources)
    }

    pub fn record_sync(
        &self,
        source_name: &str,
        status: &str,
        added: i64,
        updated: i64,
        error: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sync_log (source_name, status, skills_added, skills_updated, error_message, finished_at)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
            params![source_name, status, added, updated, error],
        )?;
        Ok(())
    }
}

// ── Row mappers ──────────────────────────────────────────
// Column layout (13 cols): id(0), name(1), desc(2), category(3), source(4),
//   source_url(5), version(6), tags(7), metadata(8), quality_score(9),
//   download_count(10), created_at(11), updated_at(12)

fn row_to_skill_13(row: &rusqlite::Row) -> rusqlite::Result<Skill> {
    let tags_str: String = row.get(7)?;
    let metadata_str: String = row.get(8)?;
    let created_str: String = row.get(11)?;
    let updated_str: String = row.get(12)?;
    Ok(Skill {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        category: row.get(3)?,
        source: row.get(4)?,
        source_url: row.get(5)?,
        version: row.get(6)?,
        tags: serde_json::from_str(&tags_str).unwrap_or_default(),
        metadata: serde_json::from_str(&metadata_str).unwrap_or_default(),
        quality_score: row.get(9)?,
        download_count: row.get(10)?,
        created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
            .unwrap()
            .with_timezone(&chrono::Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&updated_str)
            .unwrap()
            .with_timezone(&chrono::Utc),
    })
}

fn build_fts_query(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "*".to_string();
    }
    let terms: Vec<&str> = trimmed.split_whitespace().collect();
    if terms.len() == 1 {
        format!("\"{}\"", terms[0].replace('"', ""))
    } else {
        terms
            .iter()
            .map(|t| format!("\"{}\"", t.replace('"', "")))
            .collect::<Vec<_>>()
            .join(" AND ")
    }
}
