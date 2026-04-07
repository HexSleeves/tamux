use super::{
    discover_local_skills, extract_skill_metadata, SkillRecommendationAction,
    SkillRecommendationConfidence,
};
use crate::agent::types::SkillRecommendationConfig;
use crate::history::HistoryStore;
use anyhow::Result;
use std::fs;
use tempfile::tempdir;

fn write_skill(
    root: &std::path::Path,
    relative: &str,
    content: &str,
) -> Result<std::path::PathBuf> {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, content)?;
    Ok(path)
}

#[test]
fn extract_skill_metadata_reads_description_and_triggers() {
    let metadata = extract_skill_metadata(
        "builtin/brainstorming/SKILL.md",
        r#"---
name: brainstorming
description: Guide feature design before implementation.
keywords:
  - design
  - planning
triggers:
  - feature work
  - modifying behavior
---

# Brainstorming

Help turn ideas into plans.

## Triggers

- architecture change
- unclear requirements
"#,
    );

    assert_eq!(
        metadata.summary.as_deref(),
        Some("Guide feature design before implementation.")
    );
    assert!(metadata
        .triggers
        .iter()
        .any(|trigger| trigger == "feature work"));
    assert!(metadata
        .triggers
        .iter()
        .any(|trigger| trigger == "architecture change"));
    assert!(metadata.keywords.iter().any(|keyword| keyword == "design"));
    assert!(metadata.search_text.contains("Brainstorming"));
}

#[tokio::test]
async fn rank_skill_candidates_prefers_context_and_success() -> Result<()> {
    let root = tempdir()?;
    let store = HistoryStore::new_test_store(root.path()).await?;
    let skills_root = root.path().join("skills");
    let generated = skills_root.join("generated");

    let strong = write_skill(
        &generated,
        "debug-rust-build.md",
        r#"---
description: Debug Rust build and cargo test failures.
keywords: [rust, cargo, build]
triggers: [build failure, cargo test]
---

# Debug Rust Build

## Triggers
- cargo build fails
"#,
    )?;
    let weak_variant = write_skill(
        &generated,
        "debug-rust-build--legacy.md",
        r#"---
description: Older Rust build debugging flow.
keywords: [rust, build]
---

# Legacy Debug Rust Build
"#,
    )?;
    let other = write_skill(
        &generated,
        "debug-python-service.md",
        r#"---
description: Debug Python service startup issues.
keywords: [python, service]
triggers: [service crash]
---

# Debug Python Service
"#,
    )?;

    let strong_record = store.register_skill_document(&strong).await?;
    let weak_variant_record = store.register_skill_document(&weak_variant).await?;
    let other_record = store.register_skill_document(&other).await?;

    for _ in 0..4 {
        store
            .record_skill_variant_use(&strong_record.variant_id, Some(true))
            .await?;
    }
    store
        .record_skill_variant_use(&weak_variant_record.variant_id, Some(false))
        .await?;
    store
        .record_skill_variant_use(&other_record.variant_id, Some(true))
        .await?;

    let result = discover_local_skills(
        &store,
        &skills_root,
        "debug the rust cargo build failure in this backend workspace",
        &["rust".to_string(), "backend".to_string()],
        3,
        &SkillRecommendationConfig::default(),
    )
    .await?;

    assert_eq!(result.confidence, SkillRecommendationConfidence::Strong);
    assert_eq!(
        result.recommended_action,
        SkillRecommendationAction::ReadSkill
    );
    assert_eq!(
        result
            .recommendations
            .first()
            .map(|item| item.record.skill_name.as_str()),
        Some("debug-rust-build")
    );
    assert_eq!(
        result
            .recommendations
            .iter()
            .filter(|item| item.record.skill_name == "debug-rust-build")
            .count(),
        1
    );

    Ok(())
}

#[tokio::test]
async fn confidence_tier_is_none_when_scores_do_not_clear_threshold() -> Result<()> {
    let root = tempdir()?;
    let store = HistoryStore::new_test_store(root.path()).await?;
    let skills_root = root.path().join("skills");
    let generated = skills_root.join("generated");

    let skill = write_skill(
        &generated,
        "frontend-polish.md",
        r#"---
description: Polish a React UI flow.
keywords: [react, css]
---

# Frontend Polish
"#,
    )?;
    store.register_skill_document(&skill).await?;

    let result = discover_local_skills(
        &store,
        &skills_root,
        "debug a postgres replication timeout in production",
        &["database".to_string(), "infra".to_string()],
        3,
        &SkillRecommendationConfig::default(),
    )
    .await?;

    assert_eq!(result.confidence, SkillRecommendationConfidence::None);
    assert_eq!(result.recommended_action, SkillRecommendationAction::None);
    assert!(result.recommendations.is_empty());

    Ok(())
}
