use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::metacognitive::self_assessment::Assessment;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PolicyTriggerInput {
    pub thread_id: String,
    pub goal_run_id: Option<String>,
    pub repeated_approach: bool,
    pub awareness_stuck: bool,
    pub should_pivot: bool,
    pub should_escalate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PolicySelfAssessmentSummary {
    pub should_pivot: bool,
    pub should_escalate: bool,
}

impl PolicySelfAssessmentSummary {
    pub(crate) fn is_actionable(&self) -> bool {
        self.should_pivot || self.should_escalate
    }
}

impl From<&Assessment> for PolicySelfAssessmentSummary {
    fn from(value: &Assessment) -> Self {
        Self {
            should_pivot: value.should_pivot,
            should_escalate: value.should_escalate,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PolicyTriggerContext {
    pub thread_id: String,
    pub goal_run_id: Option<String>,
    pub repeated_approach: bool,
    pub awareness_stuck: bool,
    pub self_assessment: PolicySelfAssessmentSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TriggerOutcome {
    NoIntervention,
    EvaluatePolicy(PolicyTriggerContext),
}

pub(crate) fn evaluate_triggers(input: &PolicyTriggerInput) -> TriggerOutcome {
    let self_assessment = PolicySelfAssessmentSummary {
        should_pivot: input.should_pivot,
        should_escalate: input.should_escalate,
    };

    if !input.repeated_approach && !input.awareness_stuck && !self_assessment.is_actionable() {
        return TriggerOutcome::NoIntervention;
    }

    TriggerOutcome::EvaluatePolicy(PolicyTriggerContext {
        thread_id: input.thread_id.clone(),
        goal_run_id: input.goal_run_id.clone(),
        repeated_approach: input.repeated_approach,
        awareness_stuck: input.awareness_stuck,
        self_assessment,
    })
}

pub(crate) fn aggregate_trigger_contexts(
    inputs: &[PolicyTriggerInput],
) -> HashMap<String, PolicyTriggerContext> {
    let mut contexts = HashMap::new();

    for context in inputs
        .iter()
        .filter_map(|input| match evaluate_triggers(input) {
            TriggerOutcome::NoIntervention => None,
            TriggerOutcome::EvaluatePolicy(context) => Some(context),
        })
    {
        contexts
            .entry(context.thread_id.clone())
            .and_modify(|existing: &mut PolicyTriggerContext| {
                existing.goal_run_id = match (&existing.goal_run_id, &context.goal_run_id) {
                    (Some(existing_id), _) => Some(existing_id.clone()),
                    (None, Some(incoming_id)) => Some(incoming_id.clone()),
                    (None, None) => None,
                };
                existing.repeated_approach |= context.repeated_approach;
                existing.awareness_stuck |= context.awareness_stuck;
                existing.self_assessment.should_pivot |= context.self_assessment.should_pivot;
                existing.self_assessment.should_escalate |= context.self_assessment.should_escalate;
            })
            .or_insert(context);
    }

    contexts
}

pub(crate) type RecentPolicyDecisionsByScope = HashMap<PolicyDecisionScope, RecentPolicyDecision>;
pub(crate) type RetryGuardsByScope = HashMap<PolicyDecisionScope, String>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PolicyDecisionScope {
    pub thread_id: String,
    pub goal_run_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PolicyAction {
    Continue,
    Pivot,
    Escalate,
    HaltRetries,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PolicyDecision {
    pub action: PolicyAction,
    pub reason: String,
    pub strategy_hint: Option<String>,
    pub retry_guard: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecentPolicyDecision {
    pub decision: PolicyDecision,
    pub decided_at_epoch_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PolicyDecisionValidationError {
    MissingReason { action: PolicyAction },
    RetryGuardNotAllowed { action: PolicyAction },
    RetryGuardRequired { action: PolicyAction },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PolicyDecisionSemanticIdentity {
    action: PolicyAction,
    retry_guard: Option<String>,
}

impl PolicyDecision {
    fn semantic_identity(&self) -> PolicyDecisionSemanticIdentity {
        PolicyDecisionSemanticIdentity {
            action: self.action.clone(),
            retry_guard: self.retry_guard.clone(),
        }
    }
}

pub(crate) fn validate_policy_decision(
    decision: &PolicyDecision,
) -> Result<PolicyDecision, PolicyDecisionValidationError> {
    let reason = decision.reason.trim().to_string();
    let normalize = |value: &Option<String>| {
        value
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    };
    let strategy_hint = normalize(&decision.strategy_hint);
    let retry_guard = normalize(&decision.retry_guard);

    if decision.action != PolicyAction::Continue && reason.is_empty() {
        return Err(PolicyDecisionValidationError::MissingReason {
            action: decision.action.clone(),
        });
    }

    match decision.action {
        PolicyAction::Continue if retry_guard.is_some() => {
            return Err(PolicyDecisionValidationError::RetryGuardNotAllowed {
                action: PolicyAction::Continue,
            });
        }
        PolicyAction::HaltRetries if retry_guard.is_none() => {
            return Err(PolicyDecisionValidationError::RetryGuardRequired {
                action: PolicyAction::HaltRetries,
            });
        }
        _ => {}
    }

    Ok(PolicyDecision {
        action: decision.action.clone(),
        reason,
        strategy_hint,
        retry_guard,
    })
}

pub(crate) fn should_reuse_recent_decision(
    recent_decisions: &RecentPolicyDecisionsByScope,
    scope: &PolicyDecisionScope,
    candidate: &PolicyDecision,
    now_epoch_secs: u64,
    active_window_secs: u64,
) -> bool {
    recent_decisions.get(scope).is_some_and(|recent| {
        recent.decision.semantic_identity() == candidate.semantic_identity()
            && now_epoch_secs.saturating_sub(recent.decided_at_epoch_secs) <= active_window_secs
    })
}

pub(crate) fn has_active_retry_guard(
    retry_guards: &RetryGuardsByScope,
    scope: &PolicyDecisionScope,
    retry_guard: &str,
) -> bool {
    retry_guards
        .get(scope)
        .is_some_and(|active_retry_guard| active_retry_guard == retry_guard)
}

#[cfg(test)]
#[path = "orchestrator_policy_tests.rs"]
mod tests;
