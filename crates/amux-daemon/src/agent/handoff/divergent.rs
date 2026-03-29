//! Divergent subagent mode — spawn 2-3 parallel framings of the same problem
//! with different system prompt perspectives, detect disagreements between their
//! outputs, surface tensions as the valuable output (not forced consensus), and
//! synthesize a mediator recommendation that acknowledges tradeoffs.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::super::collaboration::Disagreement;

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ---------------------------------------------------------------------------
// DivergentStatus
// ---------------------------------------------------------------------------

/// Status of a divergent session as it moves through its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DivergentStatus {
    Spawning,
    Running,
    Mediating,
    Complete,
    Failed,
}

// ---------------------------------------------------------------------------
// Framing
// ---------------------------------------------------------------------------

/// A single perspective/lens used to frame the problem for a subagent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Framing {
    pub label: String,
    pub system_prompt_override: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contribution_id: Option<String>,
}

// ---------------------------------------------------------------------------
// DivergentSession
// ---------------------------------------------------------------------------

/// A divergent session that manages parallel framings of a problem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergentSession {
    pub id: String,
    pub collaboration_session_id: String,
    pub problem_statement: String,
    pub framings: Vec<Framing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tensions_markdown: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mediator_prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mediation_result: Option<String>,
    pub status: DivergentStatus,
    pub created_at: u64,
}

impl DivergentSession {
    /// Create a new divergent session. Validates that framings count is 2-3.
    pub fn new(problem_statement: String, framings: Vec<Framing>) -> Result<Self> {
        if framings.len() < 2 {
            anyhow::bail!(
                "divergent session requires at least 2 framings, got {}",
                framings.len()
            );
        }
        if framings.len() > 3 {
            anyhow::bail!(
                "divergent session supports at most 3 framings, got {}",
                framings.len()
            );
        }
        Ok(Self {
            id: format!("divergent_{}", uuid::Uuid::new_v4()),
            collaboration_session_id: String::new(),
            problem_statement,
            framings,
            tensions_markdown: None,
            mediator_prompt: None,
            mediation_result: None,
            status: DivergentStatus::Spawning,
            created_at: now_millis(),
        })
    }

    /// Validate and apply a status transition. Returns error on invalid transition.
    pub fn transition_to(&mut self, status: DivergentStatus) -> Result<()> {
        let valid = match (self.status, status) {
            (DivergentStatus::Spawning, DivergentStatus::Running) => true,
            (DivergentStatus::Running, DivergentStatus::Mediating) => true,
            (DivergentStatus::Mediating, DivergentStatus::Complete) => true,
            // Any state can transition to Failed
            (_, DivergentStatus::Failed) => true,
            _ => false,
        };
        if !valid {
            anyhow::bail!(
                "invalid divergent status transition: {:?} -> {:?}",
                self.status,
                status
            );
        }
        self.status = status;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Framing generation
// ---------------------------------------------------------------------------

/// Generate default framing prompts for a problem. Produces 2 complementary
/// perspectives: analytical-lens and pragmatic-lens. The agent/operator can
/// override with custom framings.
pub fn generate_framing_prompts(problem: &str) -> Vec<Framing> {
    vec![
        Framing {
            label: "analytical-lens".to_string(),
            system_prompt_override: format!(
                "Approach this problem analytically. Focus on correctness, edge cases, \
                 and formal reasoning. Identify what could go wrong.\n\nProblem: {}",
                problem
            ),
            task_id: None,
            contribution_id: None,
        },
        Framing {
            label: "pragmatic-lens".to_string(),
            system_prompt_override: format!(
                "Approach this problem pragmatically. Focus on simplicity, speed of \
                 delivery, and practical tradeoffs. Identify what gets results fastest.\n\n\
                 Problem: {}",
                problem
            ),
            task_id: None,
            contribution_id: None,
        },
    ]
}

// ---------------------------------------------------------------------------
// Tension formatting
// ---------------------------------------------------------------------------

/// Format disagreements as readable markdown with per-framing positions.
/// Maps disagreement agent IDs to framing labels when possible.
pub fn format_tensions(disagreements: &[Disagreement], framings: &[Framing]) -> String {
    if disagreements.is_empty() {
        return "No significant disagreements detected between framings.".to_string();
    }

    let mut output = String::new();
    for disagreement in disagreements {
        output.push_str(&format!("### {}\n\n", disagreement.topic));

        // Map positions to framing labels when we can match agent IDs to framing task_ids.
        // The `agents` field in Disagreement holds task_ids of contributing agents.
        // The `positions` field holds the distinct position labels.
        for (idx, position) in disagreement.positions.iter().enumerate() {
            let fallback = format!("Position {}", (b'A' + idx as u8) as char);
            let label = if idx < framings.len() {
                &framings[idx].label
            } else {
                &fallback
            };
            output.push_str(&format!("**{}:** {}\n", label, position));
        }

        // Evidence: include vote data if available
        if !disagreement.votes.is_empty() {
            let vote_summary: Vec<String> = disagreement
                .votes
                .iter()
                .map(|v| format!("{}: {} (weight {:.1})", v.task_id, v.position, v.weight))
                .collect();
            output.push_str(&format!("\nEvidence: {}\n", vote_summary.join("; ")));
        } else {
            output.push_str(&format!(
                "\nEvidence: confidence gap {:.2}\n",
                disagreement.confidence_gap
            ));
        }
        output.push('\n');
    }
    output
}

// ---------------------------------------------------------------------------
// Mediator prompt
// ---------------------------------------------------------------------------

/// Generate a mediator prompt that synthesizes tensions into a recommendation
/// acknowledging tradeoffs. Per locked decision: disagreement between framings
/// is surfaced as the valuable output (tensions, not forced consensus).
pub fn format_mediator_prompt(session: &DivergentSession, tensions: &str) -> String {
    let framing_descriptions: Vec<String> = session
        .framings
        .iter()
        .map(|f| format!("- **{}**: {}", f.label, f.system_prompt_override))
        .collect();

    format!(
        "You are mediating between {} different perspectives on a problem.\n\n\
         ## Problem\n{}\n\n\
         ## Framings\n{}\n\n\
         ## Tensions Identified\n{}\n\n\
         ## Your Task\n\
         Synthesize these tensions into a recommendation that:\n\
         1. Acknowledges the valid concerns from each perspective\n\
         2. Identifies the key tradeoffs (do NOT force consensus)\n\
         3. Recommends a path forward with explicit acknowledgment of what is sacrificed\n\
         4. Notes which concerns remain unresolved\n\n\
         Do NOT pick a \"winner.\" Surface the tradeoffs clearly so the operator can make an informed decision.",
        session.framings.len(),
        session.problem_statement,
        framing_descriptions.join("\n"),
        tensions
    )
}

// ---------------------------------------------------------------------------
// AgentEngine integration
// ---------------------------------------------------------------------------

use crate::agent::engine::AgentEngine;

impl AgentEngine {
    /// Start a divergent session: create framings, set up a collaboration session,
    /// enqueue tasks for each framing, and return the session ID.
    ///
    /// The caller provides the problem statement and optionally custom framings.
    /// If no custom framings are provided, `generate_framing_prompts` is used.
    pub(crate) async fn start_divergent_session(
        &self,
        problem_statement: &str,
        custom_framings: Option<Vec<Framing>>,
        thread_id: &str,
        goal_run_id: Option<&str>,
    ) -> Result<String> {
        let framings =
            custom_framings.unwrap_or_else(|| generate_framing_prompts(problem_statement));

        let mut session = DivergentSession::new(problem_statement.to_string(), framings)?;

        // Create a CollaborationSession for this divergent session.
        let collab_id = format!("collab_{}", uuid::Uuid::new_v4());
        session.collaboration_session_id = collab_id.clone();

        // Create a virtual parent task ID for the collaboration session.
        let parent_task_id = format!("divergent_parent_{}", uuid::Uuid::new_v4());

        {
            use super::super::collaboration::{CollaborationSession, CollaborativeAgent};

            let collab_session = CollaborationSession {
                id: collab_id.clone(),
                parent_task_id: parent_task_id.clone(),
                thread_id: Some(thread_id.to_string()),
                goal_run_id: goal_run_id.map(|s| s.to_string()),
                mission: problem_statement.to_string(),
                agents: session
                    .framings
                    .iter()
                    .map(|f| CollaborativeAgent {
                        task_id: f.label.clone(),
                        title: f.label.clone(),
                        role: f.label.clone(),
                        confidence: 0.5,
                        status: "spawning".to_string(),
                    })
                    .collect(),
                contributions: Vec::new(),
                disagreements: Vec::new(),
                consensus: None,
                updated_at: now_millis(),
            };
            let mut collaboration = self.collaboration.write().await;
            collaboration.insert(parent_task_id.clone(), collab_session);
        }

        // Enqueue a task for each framing.
        for framing in session.framings.iter_mut() {
            let task = self
                .enqueue_task(
                    format!("Divergent: {}", framing.label),
                    format!(
                        "{}\n\n---\nProblem: {}",
                        framing.system_prompt_override, problem_statement
                    ),
                    "normal",
                    None,
                    None,
                    Vec::new(),
                    None,
                    "divergent",
                    goal_run_id.map(|s| s.to_string()),
                    Some(parent_task_id.clone()),
                    Some(thread_id.to_string()),
                    None,
                )
                .await;
            framing.task_id = Some(task.id);
        }

        // Transition to Running.
        session.transition_to(DivergentStatus::Running)?;

        let session_id = session.id.clone();
        let mut sessions = self.divergent_sessions.write().await;
        sessions.insert(session_id.clone(), session);

        tracing::info!(
            session_id = %session_id,
            problem = %problem_statement,
            "started divergent session"
        );

        Ok(session_id)
    }

    /// Record a contribution from a framing to the divergent session.
    /// Called when each framing's task completes its work.
    pub(crate) async fn record_divergent_contribution(
        &self,
        session_id: &str,
        framing_label: &str,
        content: &str,
    ) -> Result<()> {
        self.record_divergent_contribution_internal(
            session_id,
            framing_label,
            content,
            Some(format!("contrib_{}", uuid::Uuid::new_v4())),
        )
        .await
    }

    async fn record_divergent_contribution_internal(
        &self,
        session_id: &str,
        framing_label: &str,
        content: &str,
        contribution_id: Option<String>,
    ) -> Result<()> {
        // Find the collaboration session for this divergent session.
        let parent_task_id = {
            let sessions = self.divergent_sessions.read().await;
            let session = sessions
                .get(session_id)
                .ok_or_else(|| anyhow::anyhow!("unknown divergent session: {}", session_id))?;
            // Look up the collaboration session by finding the parent task ID.
            let mut found = None;
            let collab = self.collaboration.read().await;
            for (key, cs) in collab.iter() {
                if cs.id == session.collaboration_session_id {
                    found = Some(key.clone());
                    break;
                }
            }
            found.ok_or_else(|| {
                anyhow::anyhow!(
                    "no collaboration session found for divergent session {}",
                    session_id
                )
            })?
        };

        // Add contribution using framing_label as agent_id.
        {
            use super::super::collaboration::{detect_disagreements, Contribution};

            let mut collaboration = self.collaboration.write().await;
            let collab_session = collaboration.get_mut(&parent_task_id).ok_or_else(|| {
                anyhow::anyhow!("collaboration session not found for {}", parent_task_id)
            })?;

            let contribution = Contribution {
                id: contribution_id.unwrap_or_else(|| format!("contrib_{}", uuid::Uuid::new_v4())),
                task_id: framing_label.to_string(),
                topic: "primary".to_string(),
                position: content.to_string(),
                evidence: Vec::new(),
                confidence: 0.7,
                created_at: now_millis(),
            };
            collab_session.contributions.push(contribution);
            detect_disagreements(collab_session);
            collab_session.updated_at = now_millis();
        }

        Ok(())
    }

    /// Runtime hook for completed divergent tasks.
    /// Resolves session+framing from task id, records contribution output, and
    /// synthesizes session completion once all framings have contributed.
    pub(in crate::agent) async fn record_divergent_contribution_on_task_completion(
        &self,
        task: &crate::agent::types::AgentTask,
    ) -> Result<bool> {
        if task.source != "divergent" || task.status != crate::agent::types::TaskStatus::Completed {
            return Ok(false);
        }

        let resolved = {
            let sessions = self.divergent_sessions.read().await;
            sessions.iter().find_map(|(session_id, session)| {
                session
                    .framings
                    .iter()
                    .enumerate()
                    .find(|(_, framing)| framing.task_id.as_deref() == Some(task.id.as_str()))
                    .map(|(index, framing)| {
                        (
                            session_id.clone(),
                            framing.label.clone(),
                            index,
                            session
                                .framings
                                .iter()
                                .filter(|item| item.contribution_id.is_some())
                                .count(),
                            session.framings.len(),
                        )
                    })
            })
        };

        let Some((session_id, framing_label, framing_index, existing_count, total_count)) =
            resolved
        else {
            return Ok(false);
        };

        let contribution_text = task
            .result
            .as_deref()
            .or(task
                .logs
                .iter()
                .rev()
                .find(|entry| !entry.message.trim().is_empty())
                .map(|entry| entry.message.as_str()))
            .unwrap_or(task.description.as_str());
        let contribution_text = contribution_text.trim();
        if contribution_text.is_empty() {
            return Ok(false);
        }

        let contribution_id = format!("contrib_{}", uuid::Uuid::new_v4());
        let should_complete = {
            let mut sessions = self.divergent_sessions.write().await;
            let Some(session) = sessions.get_mut(&session_id) else {
                return Ok(false);
            };
            let Some(framing) = session.framings.get_mut(framing_index) else {
                return Ok(false);
            };
            if framing.contribution_id.is_some() {
                return Ok(true);
            }
            framing.contribution_id = Some(contribution_id.clone());
            existing_count + 1 == total_count
        };

        self.record_divergent_contribution_internal(
            &session_id,
            &framing_label,
            contribution_text,
            Some(contribution_id),
        )
        .await?;

        if should_complete {
            let _ = self.complete_divergent_session(&session_id).await?;
        }

        Ok(true)
    }

    /// Canonical divergent session payload for operator retrieval surfaces.
    pub(crate) async fn get_divergent_session(
        &self,
        session_id: &str,
    ) -> Result<serde_json::Value> {
        let session = {
            let sessions = self.divergent_sessions.read().await;
            sessions
                .get(session_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("unknown divergent session: {}", session_id))?
        };
        let total = session.framings.len();
        let completed = session
            .framings
            .iter()
            .filter(|framing| framing.contribution_id.is_some())
            .count();

        Ok(serde_json::json!({
            "session_id": session.id,
            "status": session.status,
            "problem_statement": session.problem_statement,
            "framing_progress": {
                "completed": completed,
                "total": total,
                "all_contributed": completed == total
            },
            "framings": session.framings.iter().map(|framing| serde_json::json!({
                "label": framing.label,
                "task_id": framing.task_id,
                "has_contribution": framing.contribution_id.is_some(),
                "contribution_id": framing.contribution_id,
            })).collect::<Vec<_>>(),
            "tensions_markdown": session.tensions_markdown,
            "mediator_prompt": session.mediator_prompt,
            "mediation_result": session.mediation_result,
            "created_at": session.created_at,
        }))
    }

    /// Complete a divergent session: detect disagreements, format tensions,
    /// generate mediator prompt, and return the prompt.
    ///
    /// NOTE: The actual LLM mediator call is triggered by the caller (goal runner
    /// or agent loop). This method prepares the prompt and returns it. The caller
    /// decides whether to make the LLM call or present tensions directly to the
    /// operator.
    pub(crate) async fn complete_divergent_session(&self, session_id: &str) -> Result<String> {
        // Look up divergent session.
        let (collab_session_id, framings, existing_prompt) = {
            let sessions = self.divergent_sessions.read().await;
            let session = sessions
                .get(session_id)
                .ok_or_else(|| anyhow::anyhow!("unknown divergent session: {}", session_id))?;
            if session.status == DivergentStatus::Complete {
                if let Some(prompt) = session.mediator_prompt.clone() {
                    return Ok(prompt);
                }
            }
            (
                session.collaboration_session_id.clone(),
                session.framings.clone(),
                session.mediator_prompt.clone(),
            )
        };

        // Find the parent_task_id for the collaboration session.
        let parent_task_id = {
            let collab = self.collaboration.read().await;
            let mut found = None;
            for (key, cs) in collab.iter() {
                if cs.id == collab_session_id {
                    found = Some(key.clone());
                    break;
                }
            }
            found.ok_or_else(|| {
                anyhow::anyhow!(
                    "no collaboration session found for divergent session {}",
                    session_id
                )
            })?
        };

        // Detect disagreements and format tensions.
        let tensions = {
            use super::super::collaboration::detect_disagreements;

            let mut collaboration = self.collaboration.write().await;
            let collab_session = collaboration.get_mut(&parent_task_id).ok_or_else(|| {
                anyhow::anyhow!("collaboration session not found for {}", parent_task_id)
            })?;

            detect_disagreements(collab_session);
            format_tensions(&collab_session.disagreements, &framings)
        };

        // Generate mediator prompt and update session.
        let mediator_prompt = {
            let mut sessions = self.divergent_sessions.write().await;
            let session = sessions
                .get_mut(session_id)
                .ok_or_else(|| anyhow::anyhow!("unknown divergent session: {}", session_id))?;

            if session.status == DivergentStatus::Running {
                session.transition_to(DivergentStatus::Mediating)?;
            }
            let prompt =
                existing_prompt.unwrap_or_else(|| format_mediator_prompt(session, &tensions));
            session.tensions_markdown = Some(tensions.clone());
            session.mediator_prompt = Some(prompt.clone());
            if session.status == DivergentStatus::Mediating {
                session.transition_to(DivergentStatus::Complete)?;
            }
            prompt
        };

        tracing::info!(
            session_id = %session_id,
            "completed divergent session — mediator prompt generated"
        );

        Ok(mediator_prompt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::collaboration::{Disagreement, Vote};
    use crate::agent::{types::TaskStatus, AgentConfig, AgentEngine};
    use crate::session_manager::SessionManager;
    use tempfile::tempdir;

    fn make_framings(count: usize) -> Vec<Framing> {
        let labels = ["analytical-lens", "pragmatic-lens", "creative-lens"];
        let prompts = ["Analyze formally", "Be pragmatic", "Think creatively"];
        (0..count)
            .map(|i| Framing {
                label: labels[i % 3].to_string(),
                system_prompt_override: prompts[i % 3].to_string(),
                task_id: Some(format!("task_{}", i)),
                contribution_id: None,
            })
            .collect()
    }

    // -- DivergentSession::new tests --

    #[test]
    fn new_session_with_2_framings_has_spawning_status() {
        let framings = make_framings(2);
        let session = DivergentSession::new("optimize db queries".to_string(), framings)
            .expect("should create session");
        assert_eq!(session.status, DivergentStatus::Spawning);
        assert_eq!(session.framings.len(), 2);
        assert!(session.id.starts_with("divergent_"));
    }

    #[test]
    fn new_session_with_3_framings_succeeds() {
        let framings = make_framings(3);
        let session = DivergentSession::new("design auth".to_string(), framings)
            .expect("should create session");
        assert_eq!(session.framings.len(), 3);
    }

    #[test]
    fn new_session_rejects_fewer_than_2_framings() {
        let framings = make_framings(1);
        let result = DivergentSession::new("problem".to_string(), framings);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("at least 2"), "error: {}", err);
    }

    #[test]
    fn new_session_rejects_more_than_3_framings() {
        let labels = ["a", "b", "c", "d"];
        let framings: Vec<Framing> = labels
            .iter()
            .map(|l| Framing {
                label: l.to_string(),
                system_prompt_override: "test".to_string(),
                task_id: None,
                contribution_id: None,
            })
            .collect();
        let result = DivergentSession::new("problem".to_string(), framings);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("at most 3"), "error: {}", err);
    }

    // -- Status transition tests --

    #[test]
    fn status_transitions_spawning_to_running() {
        let mut session = DivergentSession::new("p".to_string(), make_framings(2)).unwrap();
        assert!(session.transition_to(DivergentStatus::Running).is_ok());
        assert_eq!(session.status, DivergentStatus::Running);
    }

    #[test]
    fn status_transitions_running_to_mediating() {
        let mut session = DivergentSession::new("p".to_string(), make_framings(2)).unwrap();
        session.transition_to(DivergentStatus::Running).unwrap();
        assert!(session.transition_to(DivergentStatus::Mediating).is_ok());
        assert_eq!(session.status, DivergentStatus::Mediating);
    }

    #[test]
    fn status_transitions_mediating_to_complete() {
        let mut session = DivergentSession::new("p".to_string(), make_framings(2)).unwrap();
        session.transition_to(DivergentStatus::Running).unwrap();
        session.transition_to(DivergentStatus::Mediating).unwrap();
        assert!(session.transition_to(DivergentStatus::Complete).is_ok());
        assert_eq!(session.status, DivergentStatus::Complete);
    }

    #[test]
    fn status_transitions_full_lifecycle() {
        let mut session = DivergentSession::new("p".to_string(), make_framings(2)).unwrap();
        assert_eq!(session.status, DivergentStatus::Spawning);
        session.transition_to(DivergentStatus::Running).unwrap();
        session.transition_to(DivergentStatus::Mediating).unwrap();
        session.transition_to(DivergentStatus::Complete).unwrap();
        assert_eq!(session.status, DivergentStatus::Complete);
    }

    #[test]
    fn status_any_state_can_transition_to_failed() {
        let mut session = DivergentSession::new("p".to_string(), make_framings(2)).unwrap();
        assert!(session.transition_to(DivergentStatus::Failed).is_ok());
        assert_eq!(session.status, DivergentStatus::Failed);
    }

    #[test]
    fn status_invalid_transition_rejected() {
        let mut session = DivergentSession::new("p".to_string(), make_framings(2)).unwrap();
        // Spawning -> Complete is invalid (must go through Running -> Mediating first)
        assert!(session.transition_to(DivergentStatus::Complete).is_err());
        // Spawning -> Mediating is invalid
        assert!(session.transition_to(DivergentStatus::Mediating).is_err());
    }

    // -- generate_framing_prompts tests --

    #[test]
    fn generate_framing_prompts_produces_2_distinct_framings() {
        let framings = generate_framing_prompts("optimize database queries");
        assert_eq!(framings.len(), 2);
        assert_ne!(framings[0].label, framings[1].label);
        assert_ne!(
            framings[0].system_prompt_override,
            framings[1].system_prompt_override
        );
    }

    #[test]
    fn generate_framing_prompts_includes_problem_in_prompts() {
        let framings = generate_framing_prompts("design user auth");
        for framing in &framings {
            assert!(
                framing.system_prompt_override.contains("design user auth"),
                "framing '{}' should contain problem statement",
                framing.label
            );
        }
    }

    #[test]
    fn generate_framing_prompts_has_analytical_and_pragmatic_lenses() {
        let framings = generate_framing_prompts("any problem");
        let labels: Vec<&str> = framings.iter().map(|f| f.label.as_str()).collect();
        assert!(
            labels.contains(&"analytical-lens"),
            "missing analytical-lens"
        );
        assert!(labels.contains(&"pragmatic-lens"), "missing pragmatic-lens");
    }

    // -- format_tensions tests --

    #[test]
    fn format_tensions_no_disagreements() {
        let result = format_tensions(&[], &[]);
        assert_eq!(
            result,
            "No significant disagreements detected between framings."
        );
    }

    #[test]
    fn format_tensions_with_disagreements_produces_markdown() {
        let disagreements = vec![
            Disagreement {
                id: "d1".to_string(),
                topic: "caching strategy".to_string(),
                agents: vec!["task_0".to_string(), "task_1".to_string()],
                positions: vec!["recommend".to_string(), "reject".to_string()],
                confidence_gap: 0.3,
                resolution: "pending".to_string(),
                votes: Vec::new(),
            },
            Disagreement {
                id: "d2".to_string(),
                topic: "error handling".to_string(),
                agents: vec!["task_0".to_string(), "task_1".to_string()],
                positions: vec!["recommend".to_string(), "reject".to_string()],
                confidence_gap: 0.1,
                resolution: "pending".to_string(),
                votes: vec![Vote {
                    task_id: "task_0".to_string(),
                    position: "recommend".to_string(),
                    weight: 0.9,
                }],
            },
        ];
        let framings = make_framings(2);
        let result = format_tensions(&disagreements, &framings);

        // Should contain topic headings
        assert!(
            result.contains("### caching strategy"),
            "missing topic heading"
        );
        assert!(
            result.contains("### error handling"),
            "missing second topic"
        );
        // Should contain framing labels or position labels
        assert!(result.contains("**"), "missing bold formatting");
        // Should have evidence
        assert!(result.contains("Evidence:"), "missing evidence section");
    }

    // -- format_mediator_prompt tests --

    #[test]
    fn format_mediator_prompt_includes_problem_statement() {
        let session =
            DivergentSession::new("optimize database queries".to_string(), make_framings(2))
                .unwrap();
        let prompt = format_mediator_prompt(&session, "some tensions");
        assert!(
            prompt.contains("optimize database queries"),
            "prompt should contain problem statement"
        );
    }

    #[test]
    fn format_mediator_prompt_includes_all_framing_labels() {
        let session = DivergentSession::new("test problem".to_string(), make_framings(2)).unwrap();
        let prompt = format_mediator_prompt(&session, "tensions");
        assert!(
            prompt.contains("analytical-lens"),
            "missing analytical-lens label"
        );
        assert!(
            prompt.contains("pragmatic-lens"),
            "missing pragmatic-lens label"
        );
    }

    #[test]
    fn format_mediator_prompt_includes_tensions() {
        let session = DivergentSession::new("problem".to_string(), make_framings(2)).unwrap();
        let tensions = "### caching\n**analytical-lens:** recommend\n**pragmatic-lens:** reject";
        let prompt = format_mediator_prompt(&session, tensions);
        assert!(
            prompt.contains(tensions),
            "prompt should include tensions verbatim"
        );
    }

    #[test]
    fn format_mediator_prompt_instructs_no_forced_consensus() {
        let session = DivergentSession::new("problem".to_string(), make_framings(2)).unwrap();
        let prompt = format_mediator_prompt(&session, "tensions");
        assert!(
            prompt.contains("do NOT force consensus"),
            "prompt must instruct against forced consensus"
        );
    }

    #[test]
    fn format_mediator_prompt_instructs_acknowledge_tradeoffs() {
        let session = DivergentSession::new("problem".to_string(), make_framings(2)).unwrap();
        let prompt = format_mediator_prompt(&session, "tensions");
        assert!(
            prompt.contains("tradeoffs"),
            "prompt must mention tradeoffs"
        );
        assert!(
            prompt.contains("Do NOT pick a \"winner.\""),
            "prompt must say no winner picking"
        );
    }

    #[tokio::test]
    async fn divergent_completion_hook_records_contribution_for_completed_framing_task() {
        let root = tempdir().expect("temp dir");
        let manager = SessionManager::new_test(root.path()).await;
        let engine = AgentEngine::new_test(manager, AgentConfig::default(), root.path()).await;
        let session_id = engine
            .start_divergent_session("evaluate cache strategy", None, "thread-div-1", None)
            .await
            .expect("start divergent session");
        let framing_task_id = {
            let sessions = engine.divergent_sessions.read().await;
            sessions
                .get(&session_id)
                .and_then(|s| s.framings.first())
                .and_then(|f| f.task_id.clone())
                .expect("first framing task id")
        };
        {
            let mut tasks = engine.tasks.lock().await;
            if let Some(task) = tasks.iter_mut().find(|task| task.id == framing_task_id) {
                task.status = TaskStatus::Completed;
                task.result = Some("cache the hot set with explicit eviction policy".to_string());
            }
        }
        let updated_task = {
            let tasks = engine.tasks.lock().await;
            tasks
                .iter()
                .find(|task| task.id == framing_task_id)
                .cloned()
                .expect("task should exist")
        };
        let handled = engine
            .record_divergent_contribution_on_task_completion(&updated_task)
            .await
            .expect("completion hook should succeed");
        assert!(
            handled,
            "divergent completion hook should run for divergent task"
        );

        let sessions = engine.divergent_sessions.read().await;
        let session = sessions.get(&session_id).expect("session should exist");
        assert!(
            session.framings[0].contribution_id.is_some(),
            "framing should receive contribution_id after completion hook"
        );
    }

    #[tokio::test]
    async fn divergent_completion_hook_final_contribution_synthesizes_tensions_and_mediator_prompt()
    {
        let root = tempdir().expect("temp dir");
        let manager = SessionManager::new_test(root.path()).await;
        let engine = AgentEngine::new_test(manager, AgentConfig::default(), root.path()).await;
        let session_id = engine
            .start_divergent_session("choose migration strategy", None, "thread-div-2", None)
            .await
            .expect("start divergent session");
        let task_ids = {
            let sessions = engine.divergent_sessions.read().await;
            sessions
                .get(&session_id)
                .expect("session should exist")
                .framings
                .iter()
                .filter_map(|framing| framing.task_id.clone())
                .collect::<Vec<_>>()
        };
        for (idx, task_id) in task_ids.iter().enumerate() {
            {
                let mut tasks = engine.tasks.lock().await;
                if let Some(task) = tasks.iter_mut().find(|task| &task.id == task_id) {
                    task.status = TaskStatus::Completed;
                    task.result = Some(if idx == 0 {
                        "Prefer strict correctness checks before rollout".to_string()
                    } else {
                        "Prioritize fast staged rollout with lightweight safeguards".to_string()
                    });
                }
            }
            let task_snapshot = {
                let tasks = engine.tasks.lock().await;
                tasks
                    .iter()
                    .find(|task| &task.id == task_id)
                    .cloned()
                    .expect("task should exist")
            };
            engine
                .record_divergent_contribution_on_task_completion(&task_snapshot)
                .await
                .expect("completion hook should succeed");
        }

        let payload = engine
            .get_divergent_session(&session_id)
            .await
            .expect("session payload should be available");
        assert_eq!(
            payload.get("status").and_then(|v| v.as_str()),
            Some("complete"),
            "final contribution should complete session"
        );
        assert!(
            payload
                .get("tensions_markdown")
                .and_then(|v| v.as_str())
                .is_some_and(|v| !v.is_empty()),
            "completed payload should include tensions markdown"
        );
        assert!(
            payload
                .get("mediator_prompt")
                .and_then(|v| v.as_str())
                .is_some_and(|v| !v.is_empty()),
            "completed payload should include mediator prompt"
        );
    }

    #[tokio::test]
    async fn divergent_completion_hook_ignores_non_divergent_tasks() {
        let root = tempdir().expect("temp dir");
        let manager = SessionManager::new_test(root.path()).await;
        let engine = AgentEngine::new_test(manager, AgentConfig::default(), root.path()).await;
        let task = engine
            .enqueue_task(
                "Regular task".to_string(),
                "non divergent completion".to_string(),
                "normal",
                None,
                None,
                Vec::new(),
                None,
                "goal_run",
                None,
                None,
                Some("thread-regular".to_string()),
                None,
            )
            .await;
        {
            let mut tasks = engine.tasks.lock().await;
            if let Some(current) = tasks.iter_mut().find(|entry| entry.id == task.id) {
                current.status = TaskStatus::Completed;
                current.result = Some("done".to_string());
            }
        }
        let snapshot = {
            let tasks = engine.tasks.lock().await;
            tasks
                .iter()
                .find(|entry| entry.id == task.id)
                .cloned()
                .expect("task exists")
        };
        let handled = engine
            .record_divergent_contribution_on_task_completion(&snapshot)
            .await
            .expect("hook should not error");
        assert!(!handled, "non-divergent tasks should bypass divergent hook");
    }
}
