use rusqlite::{Connection, Result};

const SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS skills (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT 'general',
    source TEXT NOT NULL,
    source_url TEXT,
    version TEXT NOT NULL DEFAULT '0.1.0',
    tags TEXT NOT NULL DEFAULT '[]',
    metadata TEXT NOT NULL DEFAULT '{}',
    quality_score REAL NOT NULL DEFAULT 0.0,
    download_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE VIRTUAL TABLE IF NOT EXISTS skills_fts USING fts5(
    name,
    description,
    category,
    tags,
    content='skills',
    content_rowid='rowid',
    tokenize='unicode61'
);

CREATE TABLE IF NOT EXISTS sync_sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    url TEXT NOT NULL,
    source_type TEXT NOT NULL DEFAULT 'http',
    enabled INTEGER NOT NULL DEFAULT 1,
    last_sync_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sync_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_name TEXT NOT NULL,
    status TEXT NOT NULL,
    skills_added INTEGER NOT NULL DEFAULT 0,
    skills_updated INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    finished_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_skills_source ON skills(source);
CREATE INDEX IF NOT EXISTS idx_skills_category ON skills(category);
CREATE INDEX IF NOT EXISTS idx_skills_updated ON skills(updated_at);
";

const TRIGGERS_SQL: &str = "
CREATE TRIGGER IF NOT EXISTS skills_ai AFTER INSERT ON skills BEGIN
    INSERT INTO skills_fts(rowid, name, description, category, tags)
    VALUES (new.rowid, new.name, new.description, new.category, new.tags);
END;

CREATE TRIGGER IF NOT EXISTS skills_ad AFTER DELETE ON skills BEGIN
    INSERT INTO skills_fts(skills_fts, rowid, name, description, category, tags)
    VALUES ('delete', old.rowid, old.name, old.description, old.category, old.tags);
END;

CREATE TRIGGER IF NOT EXISTS skills_au AFTER UPDATE ON skills BEGIN
    INSERT INTO skills_fts(skills_fts, rowid, name, description, category, tags)
    VALUES ('delete', old.rowid, old.name, old.description, old.category, old.tags);
    INSERT INTO skills_fts(rowid, name, description, category, tags)
    VALUES (new.rowid, new.name, new.description, new.category, new.tags);
END;
";

pub fn initialize(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA_SQL)?;
    conn.execute_batch(TRIGGERS_SQL)?;
    Ok(())
}
