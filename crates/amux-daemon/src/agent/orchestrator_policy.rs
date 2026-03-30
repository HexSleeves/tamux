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

pub(crate) type RetryGuardsByThread = HashMap<String, String>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PolicyAction {
    Continue,
    Pivot,
    Escalate,
    HaltRetries,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
}

pub(crate) fn validate_policy_decision(
    decision: &PolicyDecision,
) -> Result<PolicyDecision, PolicyDecisionValidationError> {
    let reason = decision.reason.trim().to_string();
    if decision.action != PolicyAction::Continue && reason.is_empty() {
        return Err(PolicyDecisionValidationError::MissingReason {
            action: decision.action.clone(),
        });
    }

    let normalize = |value: &Option<String>| {
        value
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    };

    Ok(PolicyDecision {
        action: decision.action.clone(),
        reason,
        strategy_hint: normalize(&decision.strategy_hint),
        retry_guard: normalize(&decision.retry_guard),
    })
}

pub(crate) fn should_reuse_recent_decision(
    recent_decisions: &HashMap<String, RecentPolicyDecision>,
    thread_id: &str,
    candidate: &PolicyDecision,
    now_epoch_secs: u64,
    active_window_secs: u64,
) -> bool {
    recent_decisions.get(thread_id).is_some_and(|recent| {
        recent.decision == *candidate
            && now_epoch_secs.saturating_sub(recent.decided_at_epoch_secs) <= active_window_secs
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn trigger_input(thread_id: &str) -> PolicyTriggerInput {
        PolicyTriggerInput {
            thread_id: thread_id.to_string(),
            goal_run_id: None,
            repeated_approach: false,
            awareness_stuck: false,
            should_pivot: false,
            should_escalate: false,
        }
    }

    fn evaluate_policy_context(input: &PolicyTriggerInput) -> PolicyTriggerContext {
        match evaluate_triggers(input) {
            TriggerOutcome::EvaluatePolicy(context) => context,
            TriggerOutcome::NoIntervention => panic!("expected policy evaluation"),
        }
    }

    fn decision(action: PolicyAction) -> PolicyDecision {
        PolicyDecision {
            action,
            reason: String::new(),
            strategy_hint: None,
            retry_guard: None,
        }
    }

    #[test]
    fn trigger_no_intervention_when_all_inputs_are_nominal() {
        let mut input = trigger_input("thread-1");
        input.goal_run_id = Some("goal-1".to_string());

        assert_eq!(evaluate_triggers(&input), TriggerOutcome::NoIntervention);
    }

    #[test]
    fn trigger_intervention_required_for_repeated_approach_signal() {
        let mut input = trigger_input("thread-1");
        input.goal_run_id = Some("goal-1".to_string());
        input.repeated_approach = true;

        let context = evaluate_policy_context(&input);

        assert_eq!(context.thread_id, "thread-1");
        assert_eq!(context.goal_run_id.as_deref(), Some("goal-1"));
        assert!(context.repeated_approach);
        assert!(!context.awareness_stuck);
        assert!(!context.self_assessment.should_pivot);
        assert!(!context.self_assessment.should_escalate);
    }

    #[test]
    fn trigger_intervention_required_for_awareness_stuckness() {
        let mut input = trigger_input("thread-2");
        input.awareness_stuck = true;

        let context = evaluate_policy_context(&input);

        assert_eq!(context.thread_id, "thread-2");
        assert!(context.awareness_stuck);
        assert!(!context.repeated_approach);
        assert!(!context.self_assessment.should_pivot);
        assert!(!context.self_assessment.should_escalate);
    }

    #[test]
    fn trigger_intervention_required_for_self_assessment_pivot_or_escalate() {
        let mut pivot_input = trigger_input("thread-3");
        pivot_input.goal_run_id = Some("goal-3".to_string());
        pivot_input.should_pivot = true;

        let mut escalate_input = trigger_input("thread-4");
        escalate_input.goal_run_id = Some("goal-4".to_string());
        escalate_input.should_escalate = true;

        let pivot_context = evaluate_policy_context(&pivot_input);
        let escalate_context = evaluate_policy_context(&escalate_input);

        assert!(pivot_context.self_assessment.should_pivot);
        assert!(!pivot_context.self_assessment.should_escalate);
        assert_eq!(escalate_context.goal_run_id.as_deref(), Some("goal-4"));
        assert!(!escalate_context.self_assessment.should_pivot);
        assert!(escalate_context.self_assessment.should_escalate);
    }

    #[test]
    fn trigger_aggregation_is_keyed_by_thread_id() {
        let inputs = vec![
            {
                let mut input = trigger_input("thread-1");
                input.goal_run_id = Some("goal-1".to_string());
                input.repeated_approach = true;
                input
            },
            {
                let mut input = trigger_input("thread-2");
                input.goal_run_id = Some("goal-2".to_string());
                input
            },
            {
                let mut input = trigger_input("thread-3");
                input.should_escalate = true;
                input
            },
        ];

        let contexts = aggregate_trigger_contexts(&inputs);

        assert_eq!(contexts.len(), 2);
        assert_eq!(
            contexts
                .get("thread-1")
                .and_then(|context| context.goal_run_id.as_deref()),
            Some("goal-1")
        );
        assert!(contexts["thread-1"].repeated_approach);
        assert!(contexts["thread-3"].self_assessment.should_escalate);
        assert!(!contexts.contains_key("thread-2"));
    }

    #[test]
    fn trigger_aggregation_merges_active_signals_for_same_thread() {
        let inputs = vec![
            {
                let mut input = trigger_input("thread-1");
                input.goal_run_id = Some("goal-1".to_string());
                input.repeated_approach = true;
                input
            },
            {
                let mut input = trigger_input("thread-1");
                input.goal_run_id = Some("goal-1".to_string());
                input.awareness_stuck = true;
                input.should_pivot = true;
                input
            },
        ];

        let contexts = aggregate_trigger_contexts(&inputs);
        let context = &contexts["thread-1"];

        assert_eq!(context.goal_run_id.as_deref(), Some("goal-1"));
        assert!(context.repeated_approach);
        assert!(context.awareness_stuck);
        assert!(context.self_assessment.should_pivot);
        assert!(!context.self_assessment.should_escalate);
    }

    #[test]
    fn trigger_aggregation_prefers_first_non_none_goal_run_id_for_same_thread() {
        let inputs = vec![
            {
                let mut input = trigger_input("thread-1");
                input.repeated_approach = true;
                input
            },
            {
                let mut input = trigger_input("thread-1");
                input.goal_run_id = Some("goal-1".to_string());
                input.awareness_stuck = true;
                input
            },
            {
                let mut input = trigger_input("thread-1");
                input.goal_run_id = Some("goal-2".to_string());
                input.should_escalate = true;
                input
            },
        ];

        let contexts = aggregate_trigger_contexts(&inputs);

        assert_eq!(
            contexts
                .get("thread-1")
                .and_then(|context| context.goal_run_id.as_deref()),
            Some("goal-1")
        );
    }

    #[test]
    fn trigger_assessment_adapter_captures_pivot_and_escalate_flags() {
        let assessment = Assessment {
            making_progress: false,
            approach_optimal: false,
            should_escalate: true,
            should_pivot: true,
            should_terminate: false,
            confidence: 0.2,
            reasoning: "signals indicate intervention".to_string(),
            recommendations: vec!["pivot".to_string(), "escalate".to_string()],
        };

        assert_eq!(
            PolicySelfAssessmentSummary::from(&assessment),
            PolicySelfAssessmentSummary {
                should_pivot: true,
                should_escalate: true,
            }
        );
    }

    #[test]
    fn decision_validate_continue_accepts_structured_output() {
        let decision: PolicyDecision = serde_json::from_str(
            r#"{
                "action": "continue",
                "reason": "",
                "strategy_hint": null,
                "retry_guard": null
            }"#,
        )
        .unwrap();

        assert_eq!(
            validate_policy_decision(&decision),
            Ok(PolicyDecision {
                action: PolicyAction::Continue,
                reason: String::new(),
                strategy_hint: None,
                retry_guard: None,
            })
        );
    }

    #[test]
    fn decision_validate_pivot_accepts_retry_guard() {
        let decision: PolicyDecision = serde_json::from_str(
            r#"{
                "action": "pivot",
                "reason": "Repeated failures indicate the current strategy is stuck.",
                "strategy_hint": "Switch to a narrower inspection-first plan.",
                "retry_guard": "approach-hash-1"
            }"#,
        )
        .unwrap();

        assert_eq!(
            validate_policy_decision(&decision),
            Ok(PolicyDecision {
                action: PolicyAction::Pivot,
                reason: "Repeated failures indicate the current strategy is stuck.".to_string(),
                strategy_hint: Some("Switch to a narrower inspection-first plan.".to_string()),
                retry_guard: Some("approach-hash-1".to_string()),
            })
        );
    }

    #[test]
    fn decision_invalid_action_string_is_rejected() {
        let result = serde_json::from_str::<PolicyDecision>(
            r#"{
                "action": "retry_forever",
                "reason": "keep going",
                "strategy_hint": null,
                "retry_guard": null
            }"#,
        );

        assert!(result.is_err());
    }

    #[test]
    fn decision_empty_reason_is_rejected_for_non_continue_actions() {
        let mut decision = decision(PolicyAction::Escalate);
        decision.reason = "   ".to_string();

        assert_eq!(
            validate_policy_decision(&decision),
            Err(PolicyDecisionValidationError::MissingReason {
                action: PolicyAction::Escalate,
            })
        );
    }

    #[test]
    fn decision_antithrash_suppresses_repeated_identical_decision_inside_active_window() {
        let mut candidate = decision(PolicyAction::Pivot);
        candidate.reason = "We already know this approach is looping.".to_string();
        candidate.strategy_hint = Some("Use a different tool sequence.".to_string());
        candidate.retry_guard = Some("approach-hash-1".to_string());
        let recent_decisions = HashMap::from([(
            "thread-1".to_string(),
            RecentPolicyDecision {
                decision: candidate.clone(),
                decided_at_epoch_secs: 1_000,
            },
        )]);

        assert!(should_reuse_recent_decision(
            &recent_decisions,
            "thread-1",
            &candidate,
            1_030,
            60,
        ));
        assert!(!should_reuse_recent_decision(
            &recent_decisions,
            "thread-1",
            &candidate,
            1_061,
            60,
        ));
    }
}
