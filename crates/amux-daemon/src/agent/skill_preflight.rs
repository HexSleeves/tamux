use super::*;
use amux_protocol::SessionId;
use crate::agent::types::SkillRecommendationConfig;

const MAX_SKILL_PREFLIGHT_MATCHES: usize = 3;

impl AgentEngine {
    pub(crate) async fn discover_skill_recommendations_public(
        &self,
        query: &str,
        session_id: Option<SessionId>,
        limit: usize,
    ) -> Result<amux_protocol::SkillDiscoveryResultPublic> {
        let skills_root = self.history.data_dir().to_path_buf();
        let context_tags = resolve_skill_context_tags(&self.session_manager, session_id).await;
        let cfg = self.config.read().await.skill_recommendation.clone();
        let result = super::skill_recommendation::discover_local_skills(
            &self.history,
            &skills_root,
            query,
            &context_tags,
            limit,
            &cfg,
        )
        .await?;

        Ok(translate_skill_discovery_result(
            query,
            &context_tags,
            &result,
            &cfg,
        ))
    }

    pub(super) async fn build_skill_preflight_context(
        &self,
        content: &str,
        session_id: Option<SessionId>,
    ) -> Result<Option<String>> {
        if !should_run_skill_preflight(content) {
            return Ok(None);
        }

        let skills_root = self.history.data_dir().to_path_buf();
        let context_tags = resolve_skill_context_tags(&self.session_manager, session_id).await;
        let cfg = self.config.read().await.skill_recommendation.clone();
        let result = super::skill_recommendation::discover_local_skills(
            &self.history,
            &skills_root,
            content,
            &context_tags,
            MAX_SKILL_PREFLIGHT_MATCHES,
            &cfg,
        )
        .await?;
        if result.recommendations.is_empty() {
            return Ok(None);
        }

        let mut body = format!(
            "Daemon skill preflight ranked local skills before tool execution. Confidence={} action={}.\n",
            confidence_label(result.confidence),
            action_label(result.recommended_action)
        );
        for recommendation in result.recommendations {
            let tags = if recommendation.record.context_tags.is_empty() {
                "none".to_string()
            } else {
                recommendation.record.context_tags.join(", ")
            };
            let summary = recommendation
                .metadata
                .summary
                .as_deref()
                .unwrap_or("No summary extracted.");
            body.push_str(&format!(
                "\n- {} [{} | status={} | uses={} | success={:.0}% | score={:.2} | tags={}]\n  Reason: {}\n  Summary: {}\n  Path: {}\n{}\n",
                recommendation.record.skill_name,
                recommendation.record.variant_name,
                recommendation.record.status,
                recommendation.record.use_count,
                recommendation.record.success_rate() * 100.0,
                recommendation.score,
                tags,
                recommendation.reason,
                summary,
                recommendation.record.relative_path,
                recommendation.excerpt
            ));
        }

        Ok(Some(body))
    }
}

fn translate_skill_discovery_result(
    query: &str,
    context_tags: &[String],
    result: &super::skill_recommendation::SkillDiscoveryResult,
    cfg: &SkillRecommendationConfig,
) -> amux_protocol::SkillDiscoveryResultPublic {
    let top_skill_name = result
        .recommendations
        .first()
        .map(|recommendation| recommendation.record.skill_name.as_str());

    amux_protocol::SkillDiscoveryResultPublic {
        query: query.to_string(),
        required: !matches!(
            result.recommended_action,
            super::skill_recommendation::SkillRecommendationAction::None
        ),
        confidence_tier: confidence_label(result.confidence).to_string(),
        recommended_action: recommended_action_label(result.recommended_action, top_skill_name),
        explicit_rationale_required: matches!(
            result.recommended_action,
            super::skill_recommendation::SkillRecommendationAction::JustifySkip
        ),
        workspace_tags: context_tags.to_vec(),
        candidates: result
            .recommendations
            .iter()
            .map(|recommendation| amux_protocol::SkillDiscoveryCandidatePublic {
                variant_id: recommendation.record.variant_id.clone(),
                skill_name: recommendation.record.skill_name.clone(),
                variant_name: recommendation.record.variant_name.clone(),
                relative_path: recommendation.record.relative_path.clone(),
                status: recommendation.record.status.clone(),
                score: recommendation.score,
                confidence_tier: candidate_confidence_label(recommendation.score, cfg).to_string(),
                reasons: split_reasons(&recommendation.reason),
                context_tags: recommendation.record.context_tags.clone(),
                use_count: recommendation.record.use_count,
                success_count: recommendation.record.success_count,
                failure_count: recommendation.record.failure_count,
            })
            .collect(),
    }
}

async fn resolve_skill_context_tags(
    session_manager: &Arc<SessionManager>,
    session_id: Option<SessionId>,
) -> Vec<String> {
    let root = if let Some(session_id) = session_id {
        let sessions = session_manager.list().await;
        sessions
            .iter()
            .find(|session| session.id == session_id)
            .and_then(|session| session.cwd.clone())
            .map(PathBuf::from)
    } else {
        None
    }
    .or_else(|| std::env::current_dir().ok());

    root.filter(|path| path.is_dir())
        .map(|path| super::semantic_env::infer_workspace_context_tags(&path))
        .unwrap_or_default()
}

fn should_run_skill_preflight(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.len() >= 48 || trimmed.lines().count() > 1 {
        return true;
    }

    let normalized = trimmed.to_ascii_lowercase();
    [
        "fix",
        "debug",
        "build",
        "implement",
        "refactor",
        "investigate",
        "review",
        "goal",
        "thread",
        "workspace",
        "terminal",
        "session",
        "tool",
    ]
    .iter()
    .any(|keyword| normalized.contains(keyword))
}

fn confidence_label(
    value: super::skill_recommendation::SkillRecommendationConfidence,
) -> &'static str {
    match value {
        super::skill_recommendation::SkillRecommendationConfidence::Strong => "strong",
        super::skill_recommendation::SkillRecommendationConfidence::Weak => "weak",
        super::skill_recommendation::SkillRecommendationConfidence::None => "none",
    }
}

fn action_label(value: super::skill_recommendation::SkillRecommendationAction) -> &'static str {
    match value {
        super::skill_recommendation::SkillRecommendationAction::ReadSkill => "read_skill",
        super::skill_recommendation::SkillRecommendationAction::JustifySkip => "justify_skip",
        super::skill_recommendation::SkillRecommendationAction::None => "none",
    }
}

fn recommended_action_label(
    action: super::skill_recommendation::SkillRecommendationAction,
    top_skill_name: Option<&str>,
) -> String {
    match (action, top_skill_name) {
        (super::skill_recommendation::SkillRecommendationAction::ReadSkill, Some(skill_name)) => {
            format!("read_skill {skill_name}")
        }
        (
            super::skill_recommendation::SkillRecommendationAction::JustifySkip,
            Some(skill_name),
        ) => format!("justify_skip {skill_name}"),
        _ => action_label(action).to_string(),
    }
}

fn candidate_confidence_label(score: f64, cfg: &SkillRecommendationConfig) -> &'static str {
    if score >= cfg.strong_match_threshold {
        "strong"
    } else if score >= cfg.weak_match_threshold {
        "weak"
    } else {
        "none"
    }
}

fn split_reasons(reason: &str) -> Vec<String> {
    let parts = reason
        .split(';')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            if let Some(rest) = value.strip_prefix("matched request terms ") {
                format!("matched {rest}")
            } else if let Some(rest) = value.strip_prefix("matched workspace tags ") {
                format!("workspace {rest}")
            } else if value.starts_with("historical success ") {
                reason_usage_summary(value)
            } else {
                value.to_string()
            }
        })
        .collect::<Vec<_>>();

    if parts.is_empty() {
        vec![reason.to_string()]
    } else {
        parts
    }
}

fn reason_usage_summary(value: &str) -> String {
    let words = value.split_whitespace().collect::<Vec<_>>();
    let uses = words
        .iter()
        .position(|word| *word == "across")
        .and_then(|index| words.get(index + 1))
        .and_then(|count| count.parse::<u32>().ok());
    let success_percent = words
        .get(2)
        .map(|value| value.trim_end_matches('%'))
        .and_then(|value| value.parse::<u32>().ok());

    match (uses, success_percent) {
        (Some(uses), Some(success_percent)) => {
            let successes = ((uses as f64) * (success_percent as f64 / 100.0)).round() as u32;
            format!("{successes}/{uses} successful uses")
        }
        _ => value.to_string(),
    }
}
