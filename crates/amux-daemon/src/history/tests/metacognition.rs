use super::*;

#[tokio::test]
async fn init_schema_adds_meta_cognition_tables() -> Result<()> {
    let (store, root) = make_test_store().await?;

    store
        .conn
        .call(|conn| {
            conn.execute_batch(
                "
                DROP TABLE IF EXISTS workflow_profiles;
                DROP TABLE IF EXISTS cognitive_biases;
                DROP TABLE IF EXISTS meta_cognition_model;
                ",
            )?;
            Ok(())
        })
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    store.init_schema().await?;

    let status = store
        .conn
        .call(|conn| {
            let model_has_agent_id = table_has_column(conn, "meta_cognition_model", "agent_id")?;
            let model_has_offset = table_has_column(conn, "meta_cognition_model", "calibration_offset")?;
            let bias_has_pattern = table_has_column(conn, "cognitive_biases", "trigger_pattern_json")?;
            let bias_index: Option<String> = conn
                .query_row(
                    "SELECT name FROM sqlite_master WHERE type = 'index' AND name = 'idx_cognitive_biases_model'",
                    [],
                    |row| row.get(0),
                )
                .optional()?;
            let workflow_has_tools = table_has_column(conn, "workflow_profiles", "typical_tools_json")?;
            let workflow_index: Option<String> = conn
                .query_row(
                    "SELECT name FROM sqlite_master WHERE type = 'index' AND name = 'idx_workflow_profiles_model'",
                    [],
                    |row| row.get(0),
                )
                .optional()?;
            Ok((
                model_has_agent_id,
                model_has_offset,
                bias_has_pattern,
                bias_index,
                workflow_has_tools,
                workflow_index,
            ))
        })
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    assert!(status.0);
    assert!(status.1);
    assert!(status.2);
    assert_eq!(status.3.as_deref(), Some("idx_cognitive_biases_model"));
    assert!(status.4);
    assert_eq!(status.5.as_deref(), Some("idx_workflow_profiles_model"));

    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[tokio::test]
async fn meta_cognition_rows_round_trip() -> Result<()> {
    let (store, root) = make_test_store().await?;

    store
        .upsert_meta_cognition_model("svarog", -0.12, 1_717_200_000)
        .await?;

    let model = store
        .get_meta_cognition_model()
        .await?
        .expect("meta cognition model should exist");
    assert_eq!(model.agent_id, "svarog");
    assert!((model.calibration_offset + 0.12).abs() < f64::EPSILON);

    store
        .replace_cognitive_biases(
            model.id,
            &[CognitiveBiasRow {
                id: 0,
                model_id: model.id,
                name: "sunk_cost".to_string(),
                trigger_pattern_json: serde_json::json!({
                    "tool_sequence": ["bash_command"],
                    "max_revisions": 3,
                    "context_tags": ["retry_loop"]
                })
                .to_string(),
                mitigation_prompt: "re-evaluate approach".to_string(),
                severity: 0.8,
                occurrence_count: 4,
            }],
        )
        .await?;

    store
        .replace_workflow_profiles(
            model.id,
            &[WorkflowProfileRow {
                id: 0,
                model_id: model.id,
                name: "debug_loop".to_string(),
                avg_success_rate: 0.61,
                avg_steps: 7,
                typical_tools_json: serde_json::json!([
                    "read_file",
                    "search_files",
                    "bash_command"
                ])
                .to_string(),
            }],
        )
        .await?;

    let biases = store.list_cognitive_biases(model.id).await?;
    assert_eq!(biases.len(), 1);
    assert_eq!(biases[0].name, "sunk_cost");
    assert_eq!(biases[0].occurrence_count, 4);

    let workflows = store.list_workflow_profiles(model.id).await?;
    assert_eq!(workflows.len(), 1);
    assert_eq!(workflows[0].name, "debug_loop");
    assert_eq!(workflows[0].avg_steps, 7);

    std::fs::remove_dir_all(root)?;
    Ok(())
}
