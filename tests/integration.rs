use skills_dba::{SearchQuery, Skill, SkillRepo};

#[test]
fn test_insert_and_retrieve() {
    let repo = SkillRepo::open_in_memory().unwrap();

    let skill = Skill::new(
        "test-skill".into(),
        "A test skill for integration testing".into(),
        "testing".into(),
        "local".into(),
        "0.1.0".into(),
        vec!["test".into(), "integration".into()],
    );

    repo.insert(&skill).unwrap();
    assert_eq!(repo.count().unwrap(), 1);

    let retrieved = repo.get_by_id(&skill.id).unwrap();
    assert_eq!(retrieved.name, "test-skill");
    assert_eq!(retrieved.tags, vec!["test", "integration"]);
}

#[test]
fn test_search() {
    let repo = SkillRepo::open_in_memory().unwrap();

    let s1 = Skill::new(
        "rust-http-server".into(),
        "Build an HTTP server in Rust".into(),
        "backend".into(),
        "github:skills".into(),
        "0.1.0".into(),
        vec!["rust".into(), "http".into()],
    );
    let s2 = Skill::new(
        "python-data-pipeline".into(),
        "Data processing pipeline in Python".into(),
        "data".into(),
        "github:skills".into(),
        "0.2.0".into(),
        vec!["python".into(), "data".into()],
    );

    repo.insert(&s1).unwrap();
    repo.insert(&s2).unwrap();

    let results = repo
        .search(&SearchQuery {
            text: "rust".into(),
            category: None,
            source: None,
            tags: vec![],
            limit: Some(10),
            offset: None,
        })
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].skill.name, "rust-http-server");
}

#[test]
fn test_search_empty() {
    let repo = SkillRepo::open_in_memory().unwrap();
    let results = repo
        .search(&SearchQuery {
            text: "nonexistent".into(),
            category: None,
            source: None,
            tags: vec![],
            limit: Some(10),
            offset: None,
        })
        .unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_delete() {
    let repo = SkillRepo::open_in_memory().unwrap();
    let skill = Skill::new(
        "tmp".into(),
        "temporary".into(),
        "misc".into(),
        "local".into(),
        "0.1.0".into(),
        vec![],
    );
    repo.insert(&skill).unwrap();
    assert!(repo.delete(&skill.id).unwrap());
    assert_eq!(repo.count().unwrap(), 0);
}

#[test]
fn test_insert_batch() {
    let repo = SkillRepo::open_in_memory().unwrap();
    let skills: Vec<_> = (0..5)
        .map(|i| {
            Skill::new(
                format!("skill-{}", i),
                format!("Description {}", i),
                "batch".into(),
                "test".into(),
                "0.1.0".into(),
                vec![],
            )
        })
        .collect();

    let n = repo.insert_batch(&skills).unwrap();
    assert_eq!(n, 5);
    assert_eq!(repo.count().unwrap(), 5);
}

#[test]
fn test_register_source() {
    let repo = SkillRepo::open_in_memory().unwrap();
    repo.register_source("test-hub", "https://example.com/skills", "http")
        .unwrap();
    let sources = repo.list_sources().unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].0, "test-hub");
}
