use super::types::{Argument, ArgumentPoint, Role};

fn tool_specific_caution_claim(tool_name: &str, action_summary: &str) -> Option<String> {
    match tool_name {
        "enqueue_task" => Some(format!(
            "Schedule this background task for the operator's typical working window instead of dispatching it immediately: {}.",
            crate::agent::summarize_text(action_summary, 96)
        )),
        "spawn_subagent" => Some(format!(
            "Reduce permissions by constraining the child to a smaller tool-call budget and wall-clock window before delegating {}.",
            crate::agent::summarize_text(action_summary, 96)
        )),
        _ => None,
    }
}

pub(crate) fn build_argument(
    tool_name: &str,
    action_summary: &str,
    reasons: &[String],
) -> Argument {
    let mut points = Vec::new();

    let risk_weight = if reasons.is_empty() { 0.28 } else { 0.76 };
    points.push(ArgumentPoint {
        claim: format!(
            "'{}' can mutate state or propagate effects beyond a trivial read-only operation.",
            tool_name
        ),
        weight: risk_weight,
        evidence: vec![format!("tool:{}", tool_name)],
    });

    if !reasons.is_empty() {
        points.push(ArgumentPoint {
            claim: "Governance already detected suspicious characteristics that warrant extra scrutiny."
                .to_string(),
            weight: 0.82,
            evidence: reasons
                .iter()
                .take(4)
                .map(|reason| format!("governance:{reason}"))
                .collect(),
        });
    }

    points.push(ArgumentPoint {
        claim: format!(
            "Safer alternatives may exist: narrow scope, reduce permissions, or seek operator confirmation before applying {}.",
            crate::agent::summarize_text(action_summary, 96)
        ),
        weight: if reasons.is_empty() { 0.32 } else { 0.63 },
        evidence: vec!["heuristic:prefer_narrower_scope".to_string()],
    });

    if let Some(claim) = tool_specific_caution_claim(tool_name, action_summary) {
        points.push(ArgumentPoint {
            claim,
            weight: if reasons.is_empty() { 0.57 } else { 0.74 },
            evidence: vec![format!("tool_specific:{tool_name}:narrower_execution")],
        });
    }

    Argument {
        role: Role::Critic,
        points,
        overall_confidence: if reasons.is_empty() { 0.34 } else { 0.81 },
    }
}
