use std::collections::BTreeMap;

use anyhow::Result;

use crate::agent::background_workers::protocol::{
    BackgroundWorkerCommand, BackgroundWorkerKind, BackgroundWorkerResult,
};
use crate::agent::background_workers::run_background_worker_command;
use crate::agent::engine::AgentEngine;
use crate::agent::skill_discovery::{infer_draft_context_tags, sanitize_agentskills_name};
use crate::history::{ExecutionTraceRow, SkillVariantRecord};

use super::types::{
    GenePoolArenaScore, GenePoolCandidate, GenePoolLifecycleAction, GenePoolRuntimeSnapshot,
};

const GENE_POOL_RUNTIME_SNAPSHOT_KEY: &str = "gene_pool_runtime_snapshot";
const GENE_POOL_TRACE_LIMIT: usize = 200;
const GENE_POOL_VARIANT_LIMIT: usize = 512;
const PROMOTION_SCORE_THRESHOLD: f64 = 0.78;
const RETIREMENT_SCORE_THRESHOLD: f64 = 0.32;
const MIN_PROMOTION_SUCCESSES: u32 = 3;
const MIN_RETIREMENT_USES: u32 = 4;

fn parse_tool_sequence(trace: &ExecutionTraceRow) -> Vec<String> {
    trace
        .tool_sequence_json
        .as_deref()
        .and_then(|json| serde_json::from_str::<Vec<String>>(json).ok())
        .unwrap_or_default()
        .into_iter()
        .filter(|tool| !tool.trim().is_empty())
        .collect()
}

fn normalize_fitness_score(fitness_score: f64) -> f64 {
    ((fitness_score + 6.0) / 12.0).clamp(0.0, 1.0)
}

fn compute_arena_score(record: &SkillVariantRecord) -> f64 {
    let success_rate = record.success_rate();
    let fitness_component = normalize_fitness_score(record.fitness_score);
    let usage_component = (record.use_count.min(10) as f64) / 10.0;
    (0.55 * success_rate + 0.30 * fitness_component + 0.15 * usage_component).clamp(0.0, 1.0)
}

fn build_candidate(trace: &ExecutionTraceRow) -> Option<GenePoolCandidate> {
    let task_type = trace
        .task_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown");
    let tool_sequence = parse_tool_sequence(trace);
    if tool_sequence.is_empty() {
        return None;
    }

    let prefix = tool_sequence
        .iter()
        .take(2)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("-");
    let proposed_skill_name = sanitize_agentskills_name(&format!("{task_type}-{prefix}"));
    let proposed_skill_name = if proposed_skill_name.is_empty() {
        sanitize_agentskills_name(task_type)
    } else {
        proposed_skill_name
    };

    Some(GenePoolCandidate {
        trace_id: trace.id.clone(),
        proposed_skill_name,
        task_type: task_type.to_string(),
        context_tags: infer_draft_context_tags(&tool_sequence, task_type, None),
        quality_score: trace.quality_score.unwrap_or(0.0),
        tool_sequence,
    })
}

pub(crate) fn build_gene_pool_runtime_snapshot(
    successful_traces: &[ExecutionTraceRow],
    variants: &[SkillVariantRecord],
    now_ms: u64,
) -> GenePoolRuntimeSnapshot {
    let mut candidates_by_skill = BTreeMap::<String, GenePoolCandidate>::new();
    for trace in successful_traces {
        let Some(candidate) = build_candidate(trace) else {
            continue;
        };
        match candidates_by_skill.get(&candidate.proposed_skill_name) {
            Some(existing) if existing.quality_score >= candidate.quality_score => {}
            _ => {
                candidates_by_skill.insert(candidate.proposed_skill_name.clone(), candidate);
            }
        }
    }
    let candidates = candidates_by_skill.into_values().collect::<Vec<_>>();

    let mut arena_scores = variants
        .iter()
        .map(|record| GenePoolArenaScore {
            variant_id: record.variant_id.clone(),
            skill_name: record.skill_name.clone(),
            variant_name: record.variant_name.clone(),
            status: record.status.clone(),
            arena_score: compute_arena_score(record),
            success_rate: record.success_rate(),
            fitness_score: record.fitness_score,
        })
        .collect::<Vec<_>>();
    arena_scores.sort_by(|left, right| {
        right
            .arena_score
            .partial_cmp(&left.arena_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.variant_id.cmp(&right.variant_id))
    });

    let by_variant = arena_scores
        .iter()
        .map(|score| (score.variant_id.clone(), score))
        .collect::<BTreeMap<_, _>>();
    let mut lifecycle_actions = Vec::new();
    for record in variants {
        let Some(score) = by_variant.get(&record.variant_id) else {
            continue;
        };
        if matches!(record.status.as_str(), "draft" | "testing")
            && score.arena_score >= PROMOTION_SCORE_THRESHOLD
            && record.success_count >= MIN_PROMOTION_SUCCESSES
        {
            lifecycle_actions.push(GenePoolLifecycleAction {
                action: "promote".to_string(),
                variant_id: Some(record.variant_id.clone()),
                reason: format!(
                    "arena score {:.2} with success rate {:.0}% is strong enough for promotion",
                    score.arena_score,
                    score.success_rate * 100.0
                ),
            });
        }
        if matches!(record.status.as_str(), "active" | "proven" | "canonical")
            && score.arena_score <= RETIREMENT_SCORE_THRESHOLD
            && record.use_count >= MIN_RETIREMENT_USES
            && record.failure_count > record.success_count
        {
            lifecycle_actions.push(GenePoolLifecycleAction {
                action: "retire".to_string(),
                variant_id: Some(record.variant_id.clone()),
                reason: format!(
                    "arena score {:.2} with {} failures exceeds retirement threshold",
                    score.arena_score, record.failure_count
                ),
            });
        }
    }

    GenePoolRuntimeSnapshot {
        generated_at_ms: now_ms,
        candidates,
        arena_scores,
        summary: format!(
            "gene pool snapshot with {} candidates, {} arena scores, {} lifecycle actions",
            successful_traces.len().min(GENE_POOL_TRACE_LIMIT),
            variants.len().min(GENE_POOL_VARIANT_LIMIT),
            lifecycle_actions.len()
        ),
        lifecycle_actions,
    }
}

impl AgentEngine {
    pub(crate) async fn refresh_gene_pool_runtime(&self) -> Result<GenePoolRuntimeSnapshot> {
        let successful_traces = self
            .history
            .list_recent_successful_traces(0, GENE_POOL_TRACE_LIMIT)
            .await?;
        let variants = self
            .history
            .list_skill_variants(None, GENE_POOL_VARIANT_LIMIT)
            .await?;
        let now = crate::agent::now_millis();

        let result = run_background_worker_command(
            BackgroundWorkerKind::Learning,
            BackgroundWorkerCommand::TickLearning {
                successful_traces,
                variants,
                now_ms: now,
            },
        )
        .await?;

        let snapshot = match result {
            BackgroundWorkerResult::LearningTick { snapshot } => snapshot,
            BackgroundWorkerResult::Error { message } => {
                anyhow::bail!("learning worker returned error: {message}");
            }
            other => anyhow::bail!("learning worker returned unexpected response: {other:?}"),
        };

        for action in &snapshot.lifecycle_actions {
            let Some(variant_id) = action.variant_id.as_deref() else {
                continue;
            };
            match action.action.as_str() {
                "promote" => {
                    self.history.promote_skill_variant(variant_id).await?;
                }
                "retire" => {
                    self.history.retire_skill_variant(variant_id).await?;
                }
                _ => {}
            }
        }

        self.history
            .set_consolidation_state(
                GENE_POOL_RUNTIME_SNAPSHOT_KEY,
                &serde_json::to_string(&snapshot)?,
                now,
            )
            .await?;

        Ok(snapshot)
    }
}
