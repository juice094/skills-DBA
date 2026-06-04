# skills-DBA

Skills Database Administration — local skill index with FTS5 full-text search, multi-source async sync, and recommendation engine.

## Architecture

```
SearchService → SkillRepo → SQLite + FTS5
                    ↑
SyncPipeline → [GitHubSource, HttpSource, LocalFileSource]
```

### Design Decisions

- **SQLite + FTS5** — single-file, zero-infrastructure database with built-in full-text search. WAL mode for concurrent reads during sync.
- **Async sync pipeline** — skill ingestion is decoupled from query serving. External sources are polled via pluggable `SkillSource` trait.
- **BM25 ranking** — SQLite FTS5's built-in BM25 scoring for relevance ranking, no external vector DB dependency.
- **No real-time external API calls on the search path** — all searches hit local DB only, ensuring <10ms p99 latency.

## Usage

```rust
use skills_dba::{Skill, SkillRepo, SearchQuery};

let repo = SkillRepo::open("skills.db")?;

let skill = Skill::new(
    "rust-http-server".into(),
    "Build an HTTP server in Rust using hyper".into(),
    "backend".into(),
    "local".into(),
    "0.1.0".into(),
    vec!["rust".into(), "http".into(), "server".into()],
);
repo.insert(&skill)?;

let results = repo.search(&SearchQuery {
    text: "rust server".into(),
    category: None,
    source: None,
    tags: vec![],
    limit: Some(20),
    offset: None,
})?;
```

## Sync from External Sources

```rust
use skills_dba::{SkillRepo, SyncPipeline};
use skills_dba::sync::GitHubSource;

let repo = SkillRepo::open("skills.db")?;
let mut pipeline = SyncPipeline::new(repo);
pipeline.add_source(Box::new(
    GitHubSource::new("anthropics", "skills", "skills")
));
let outcomes = pipeline.sync_all().await;
```

## Reference Projects

The design is informed by analysis of 9 mature open-source skill registries (see [RESEARCH.md](RESEARCH.md)):

| Project | Scale | Search | Storage |
|---------|-------|--------|---------|
| [UseSkill](https://github.com/c2s/agent-skills-hub) | 100K+ skills | MySQL FULLTEXT | MySQL |
| [skilldb](https://github.com/AmazingAng/skilldb) | 180K+ skills | JSON index | File-based |
| [SkillX.sh](https://github.com/nextlevelbuilder/skillx) | 500+ skills | Hybrid (FTS5 + vectors) | SQLite |
| [skill-forge](https://github.com/vystartasv/skill-forge) | 149+ skills | FTS5 | SQLite |
| [LangSkills](https://github.com/LabRAI/LangSkills) | 119K skills | FTS5 | SQLite |
| [GBrain](https://github.com/garrytan/gbrain) | Personal scale | FTS5 + vectors | SQLite |
| [skillscat](https://github.com/backrunner/skillscat) | Platform | Queue-based | Cloudflare D1 |
| [ClawHub](https://github.com/openclaw/clawhub) | Registry | Vector (OpenAI) | Convex |
| [OpenSkill Manager](https://github.com/BoboloveIC/openskill-manager) | Desktop | Filesystem watch | Local |

## License

MIT
