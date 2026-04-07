use super::*;
use amux_protocol::SessionId;

const MAX_SKILL_PREFLIGHT_MATCHES: usize = 3;

impl AgentEngine {
    pub(super) async fn build_skill_preflight_context(
        &self,
        content: &str,
        session_id: Option<SessionId>,
    ) -> Result<Option<String>> {
        if !should_run_skill_preflight(content) {
            return Ok(None);
        }

        let skills_root = skills_dir(&self.data_dir);
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
