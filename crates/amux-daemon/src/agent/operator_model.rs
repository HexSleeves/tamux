//! Local aggregate-only operator model.

use std::collections::HashMap;

use amux_protocol::ApprovalDecision;
use serde::{Deserialize, Serialize};

use super::*;

const OPERATOR_MODEL_VERSION: &str = "1.0";
const OPERATOR_PROFILE_VERSION: &str = "1.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum OperatorProfileInputKind {
    Boolean,
}

impl OperatorProfileInputKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct OperatorProfileQuestion {
    pub id: String,
    pub field_key: String,
    pub prompt: String,
    pub input_kind: OperatorProfileInputKind,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(super) struct OperatorProfileQuestionState {
    pub answer_json: Option<String>,
    pub skipped: bool,
    pub skip_reason: Option<String>,
    pub deferred_until_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OperatorProfileSessionState {
    pub version: String,
    pub session_id: String,
    pub kind: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub questions: Vec<OperatorProfileQuestion>,
    pub answers: HashMap<String, OperatorProfileQuestionState>,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OperatorProfileQuestionPayload {
    pub session_id: String,
    pub question_id: String,
    pub field_key: String,
    pub prompt: String,
    pub input_kind: String,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OperatorProfileProgressPayload {
    pub session_id: String,
    pub answered: u32,
    pub remaining: u32,
    pub completion_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OperatorProfileCompletionPayload {
    pub session_id: String,
    pub updated_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OperatorProfileSessionStarted {
    pub session_id: String,
    pub kind: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub(super) enum VerbosityPreference {
    Terse,
    #[default]
    Moderate,
    Verbose,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub(super) enum ReadingDepth {
    Skim,
    #[default]
    Standard,
    Deep,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub(super) enum RiskTolerance {
    Conservative,
    #[default]
    Moderate,
    Aggressive,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(super) struct CognitiveStyle {
    pub avg_message_length: f64,
    pub question_frequency: f64,
    pub confirmation_seeking: f64,
    pub verbosity_preference: VerbosityPreference,
    pub reading_depth: ReadingDepth,
    pub message_count: u64,
    pub question_count: u64,
    pub confirmation_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(super) struct RiskFingerprint {
    pub approval_rate_by_category: HashMap<String, f64>,
    pub avg_response_time_secs: f64,
    pub risk_tolerance: RiskTolerance,
    pub approval_requests: u64,
    pub approvals: u64,
    pub denials: u64,
    pub category_requests: HashMap<String, u64>,
    pub category_approvals: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(super) struct SessionRhythm {
    pub typical_start_hour_utc: Option<u8>,
    pub session_duration_avg_minutes: f64,
    pub peak_activity_hours_utc: Vec<u8>,
    pub session_count: u64,
    pub start_hour_histogram: HashMap<u8, u64>,
    pub activity_hour_histogram: HashMap<u8, u64>,
    pub total_observed_session_minutes: u64,
    /// EMA-smoothed activity histogram persisted across daemon restarts. Per D-02.
    #[serde(default)]
    pub smoothed_activity_histogram: HashMap<u8, f64>,
}

// ---------------------------------------------------------------------------
// EMA smoothing pure functions (BEAT-06/D-02)
// ---------------------------------------------------------------------------

/// Exponential moving average update: `alpha * sample + (1 - alpha) * current`.
pub(super) fn ema_update(current: f64, sample: f64, alpha: f64) -> f64 {
    alpha * sample + (1.0 - alpha) * current
}

/// Apply EMA smoothing to an activity histogram across all 24 hours.
///
/// Hours with no observation decay toward zero; observed hours adapt toward
/// the new count. Returns a full 24-hour histogram.
pub(super) fn smooth_activity_histogram(
    current: &HashMap<u8, f64>,
    observation: &HashMap<u8, u64>,
    alpha: f64,
) -> HashMap<u8, f64> {
    let mut smoothed = current.clone();
    for hour in 0..24u8 {
        let observed = observation.get(&hour).copied().unwrap_or(0) as f64;
        let old = smoothed.get(&hour).copied().unwrap_or(0.0);
        smoothed.insert(hour, ema_update(old, observed, alpha));
    }
    smoothed
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(super) struct AttentionTopology {
    pub dominant_surface: Option<String>,
    pub common_surfaces: Vec<String>,
    pub top_transitions: Vec<String>,
    pub surface_histogram: HashMap<String, u64>,
    pub transition_histogram: HashMap<String, u64>,
    pub dwell_histogram: HashMap<String, u64>,
    pub focus_event_count: u64,
    pub avg_surface_dwell_secs: f64,
    pub rapid_switch_count: u64,
    pub deep_focus_surface: Option<String>,
    pub last_surface: Option<String>,
    pub last_surface_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(super) struct ImplicitFeedback {
    pub tool_hesitation_count: u64,
    pub revision_message_count: u64,
    pub correction_message_count: u64,
    pub fast_denial_count: u64,
    pub fallback_histogram: HashMap<String, u64>,
    pub top_tool_fallbacks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct OperatorModel {
    pub version: String,
    pub last_updated: u64,
    pub session_count: u64,
    pub cognitive_style: CognitiveStyle,
    pub risk_fingerprint: RiskFingerprint,
    pub session_rhythm: SessionRhythm,
    pub attention_topology: AttentionTopology,
    pub implicit_feedback: ImplicitFeedback,
    /// Distinct tool names the operator has interacted with (Phase 10).
    #[serde(default)]
    pub unique_tools_seen: Vec<String>,
    /// Number of successfully completed goal runs (Phase 10).
    #[serde(default)]
    pub goal_runs_completed: u64,
}

#[derive(Debug, Clone)]
pub(super) struct PendingApprovalObservation {
    pub requested_at: u64,
    pub category: String,
}

impl Default for OperatorModel {
    fn default() -> Self {
        Self {
            version: OPERATOR_MODEL_VERSION.to_string(),
            last_updated: 0,
            session_count: 0,
            cognitive_style: CognitiveStyle::default(),
            risk_fingerprint: RiskFingerprint::default(),
            session_rhythm: SessionRhythm::default(),
            attention_topology: AttentionTopology::default(),
            implicit_feedback: ImplicitFeedback::default(),
            unique_tools_seen: Vec::new(),
            goal_runs_completed: 0,
        }
    }
}

impl AgentEngine {
    pub(super) async fn refresh_operator_model(&self) -> Result<()> {
        if !self.config.read().await.operator_model.enabled {
            *self.operator_model.write().await = OperatorModel::default();
            self.active_operator_sessions.write().await.clear();
            self.pending_operator_approvals.write().await.clear();
            return Ok(());
        }
        ensure_operator_model_file(&self.data_dir).await?;
        let raw = tokio::fs::read_to_string(operator_model_path(&self.data_dir)).await?;
        let parsed = serde_json::from_str::<OperatorModel>(&raw).unwrap_or_default();
        *self.operator_model.write().await = parsed;
        Ok(())
    }

    pub async fn operator_model_json(&self) -> Result<String> {
        if !self.config.read().await.operator_model.enabled {
            return Ok(serde_json::to_string_pretty(
                &*self.operator_model.read().await,
            )?);
        }
        ensure_operator_model_file(&self.data_dir).await?;
        tokio::fs::read_to_string(operator_model_path(&self.data_dir))
            .await
            .map_err(Into::into)
    }

    pub async fn reset_operator_model(&self) -> Result<()> {
        let reset = OperatorModel::default();
        *self.operator_model.write().await = reset.clone();
        self.active_operator_sessions.write().await.clear();
        self.pending_operator_approvals.write().await.clear();
        self.operator_profile_sessions.write().await.clear();
        for row in self.history.list_operator_profile_sessions().await? {
            self.history
                .delete_operator_profile_session(&row.session_id)
                .await?;
        }
        if self.config.read().await.operator_model.enabled {
            persist_operator_model(&self.data_dir, &reset)?;
        } else {
            match tokio::fs::remove_file(operator_model_path(&self.data_dir)).await {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }

    fn operator_profile_questions_for_kind(kind: &str) -> Vec<OperatorProfileQuestion> {
        let mut questions = Vec::new();
        let mut push_flag = |id: &str, field_key: &str, prompt: &str| {
            questions.push(OperatorProfileQuestion {
                id: id.to_string(),
                field_key: field_key.to_string(),
                prompt: prompt.to_string(),
                input_kind: OperatorProfileInputKind::Boolean,
                optional: false,
            });
        };

        // Keep a stable baseline regardless of kind; kind is still persisted for future branching.
        let _ = kind;
        push_flag(
            "enabled",
            "enabled",
            "Enable operator modeling overall?",
        );
        push_flag(
            "allow_message_statistics",
            "allow_message_statistics",
            "Allow learning from message statistics?",
        );
        push_flag(
            "allow_approval_learning",
            "allow_approval_learning",
            "Allow learning from approval decisions?",
        );
        push_flag(
            "allow_attention_tracking",
            "allow_attention_tracking",
            "Allow attention surface tracking?",
        );
        push_flag(
            "allow_implicit_feedback",
            "allow_implicit_feedback",
            "Allow implicit feedback learning (revisions/fallbacks)?",
        );
        questions
    }

    fn parse_bool_answer(answer_json: &str) -> Result<bool> {
        if let Ok(value) = serde_json::from_str::<bool>(answer_json) {
            return Ok(value);
        }
        let value: serde_json::Value =
            serde_json::from_str(answer_json).map_err(|error| {
                anyhow::anyhow!("invalid answer_json payload for boolean consent: {error}")
            })?;
        value
            .as_bool()
            .ok_or_else(|| anyhow::anyhow!("answer_json must decode to a boolean"))
    }

    fn operator_profile_progress(
        session: &OperatorProfileSessionState,
    ) -> OperatorProfileProgressPayload {
        let now = now_millis();
        let answered = session
            .questions
            .iter()
            .filter(|question| {
                session.answers.get(&question.id).is_some_and(|state| {
                    if state.answer_json.is_some() || state.skipped {
                        return true;
                    }
                    match state.deferred_until_unix_ms {
                        Some(until) => until > now,
                        None => false,
                    }
                })
            })
            .count() as u32;
        let total = session.questions.len() as u32;
        let remaining = total.saturating_sub(answered);
        let completion_ratio = if total == 0 {
            1.0
        } else {
            answered as f64 / total as f64
        };
        OperatorProfileProgressPayload {
            session_id: session.session_id.clone(),
            answered,
            remaining,
            completion_ratio,
        }
    }

    fn next_operator_profile_question(
        session: &OperatorProfileSessionState,
    ) -> Option<OperatorProfileQuestionPayload> {
        let now = now_millis();
        session.questions.iter().find_map(|question| {
            let state = session.answers.get(&question.id);
            let already_done = state.is_some_and(|s| s.answer_json.is_some() || s.skipped);
            if already_done {
                return None;
            }
            if state
                .and_then(|s| s.deferred_until_unix_ms)
                .is_some_and(|until| until > now)
            {
                return None;
            }
            Some(OperatorProfileQuestionPayload {
                session_id: session.session_id.clone(),
                question_id: question.id.clone(),
                field_key: question.field_key.clone(),
                prompt: question.prompt.clone(),
                input_kind: question.input_kind.as_str().to_string(),
                optional: question.optional,
            })
        })
    }

    fn apply_operator_profile_answers(
        config: &mut AgentConfig,
        session: &OperatorProfileSessionState,
    ) -> Result<Vec<String>> {
        let mut updated_fields = Vec::new();
        for question in &session.questions {
            let Some(state) = session.answers.get(&question.id) else {
                continue;
            };
            let Some(answer_json) = state.answer_json.as_deref() else {
                continue;
            };
            let granted = Self::parse_bool_answer(answer_json)?;
            match question.field_key.as_str() {
                "enabled" => {
                    config.operator_model.enabled = granted;
                    updated_fields.push("enabled".to_string());
                }
                "allow_message_statistics" => {
                    config.operator_model.allow_message_statistics = granted;
                    updated_fields.push("allow_message_statistics".to_string());
                }
                "allow_approval_learning" => {
                    config.operator_model.allow_approval_learning = granted;
                    updated_fields.push("allow_approval_learning".to_string());
                }
                "allow_attention_tracking" => {
                    config.operator_model.allow_attention_tracking = granted;
                    updated_fields.push("allow_attention_tracking".to_string());
                }
                "allow_implicit_feedback" => {
                    config.operator_model.allow_implicit_feedback = granted;
                    updated_fields.push("allow_implicit_feedback".to_string());
                }
                _ => {}
            }
        }
        Ok(updated_fields)
    }

    pub async fn start_operator_profile_session(
        &self,
        kind: &str,
    ) -> Result<OperatorProfileSessionStarted> {
        let session_id = format!("ops_{}", uuid::Uuid::new_v4());
        let now = now_millis();
        let session = OperatorProfileSessionState {
            version: OPERATOR_PROFILE_VERSION.to_string(),
            session_id: session_id.clone(),
            kind: kind.to_string(),
            created_at: now,
            updated_at: now,
            questions: Self::operator_profile_questions_for_kind(kind),
            answers: HashMap::new(),
            completed: false,
        };
        let session_json = serde_json::to_string(&session)?;
        self.history
            .upsert_operator_profile_session(&session_id, kind, &session_json, now)
            .await?;
        self.operator_profile_sessions
            .write()
            .await
            .insert(session_id.clone(), session);
        Ok(OperatorProfileSessionStarted {
            session_id,
            kind: kind.to_string(),
        })
    }

    pub async fn next_operator_profile_question_for_session(
        &self,
        session_id: &str,
    ) -> Result<(Option<OperatorProfileQuestionPayload>, OperatorProfileProgressPayload)> {
        let session = {
            let sessions = self.operator_profile_sessions.read().await;
            sessions
                .get(session_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("unknown operator profile session: {session_id}"))?
        };
        let question = Self::next_operator_profile_question(&session);
        let progress = Self::operator_profile_progress(&session);
        Ok((question, progress))
    }

    pub async fn submit_operator_profile_answer(
        &self,
        session_id: &str,
        question_id: &str,
        answer_json: &str,
    ) -> Result<(Option<OperatorProfileQuestionPayload>, OperatorProfileProgressPayload)> {
        let mut sessions = self.operator_profile_sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("unknown operator profile session: {session_id}"))?;
        if !session.questions.iter().any(|q| q.id == question_id) {
            anyhow::bail!("unknown question_id for session {session_id}: {question_id}");
        }
        let state = session.answers.entry(question_id.to_string()).or_default();
        state.answer_json = Some(answer_json.to_string());
        state.skipped = false;
        state.skip_reason = None;
        state.deferred_until_unix_ms = None;
        session.updated_at = now_millis();
        let session_json = serde_json::to_string(session)?;
        self.history
            .upsert_operator_profile_session(
                &session.session_id,
                &session.kind,
                &session_json,
                session.updated_at,
            )
            .await?;
        let question = Self::next_operator_profile_question(session);
        let progress = Self::operator_profile_progress(session);
        Ok((question, progress))
    }

    pub async fn skip_operator_profile_question(
        &self,
        session_id: &str,
        question_id: &str,
        reason: Option<&str>,
    ) -> Result<(Option<OperatorProfileQuestionPayload>, OperatorProfileProgressPayload)> {
        let mut sessions = self.operator_profile_sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("unknown operator profile session: {session_id}"))?;
        if !session.questions.iter().any(|q| q.id == question_id) {
            anyhow::bail!("unknown question_id for session {session_id}: {question_id}");
        }
        let state = session.answers.entry(question_id.to_string()).or_default();
        state.answer_json = None;
        state.skipped = true;
        state.skip_reason = reason.map(str::to_string);
        state.deferred_until_unix_ms = None;
        session.updated_at = now_millis();
        let session_json = serde_json::to_string(session)?;
        self.history
            .upsert_operator_profile_session(
                &session.session_id,
                &session.kind,
                &session_json,
                session.updated_at,
            )
            .await?;
        let question = Self::next_operator_profile_question(session);
        let progress = Self::operator_profile_progress(session);
        Ok((question, progress))
    }

    pub async fn defer_operator_profile_question(
        &self,
        session_id: &str,
        question_id: &str,
        defer_until_unix_ms: Option<u64>,
    ) -> Result<(Option<OperatorProfileQuestionPayload>, OperatorProfileProgressPayload)> {
        let mut sessions = self.operator_profile_sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("unknown operator profile session: {session_id}"))?;
        if !session.questions.iter().any(|q| q.id == question_id) {
            anyhow::bail!("unknown question_id for session {session_id}: {question_id}");
        }
        let now = now_millis();
        let state = session.answers.entry(question_id.to_string()).or_default();
        state.deferred_until_unix_ms = Some(defer_until_unix_ms.unwrap_or(now + 15 * 60_000));
        state.skipped = false;
        session.updated_at = now;
        let session_json = serde_json::to_string(session)?;
        self.history
            .upsert_operator_profile_session(
                &session.session_id,
                &session.kind,
                &session_json,
                session.updated_at,
            )
            .await?;
        let question = Self::next_operator_profile_question(session);
        let progress = Self::operator_profile_progress(session);
        Ok((question, progress))
    }

    pub async fn get_operator_profile_summary_json(&self) -> Result<String> {
        let model = self.operator_model.read().await.clone();
        let config = self.config.read().await.operator_model.clone();
        Ok(serde_json::to_string(&serde_json::json!({
            "model": model,
            "consents": {
                "enabled": config.enabled,
                "allow_message_statistics": config.allow_message_statistics,
                "allow_approval_learning": config.allow_approval_learning,
                "allow_attention_tracking": config.allow_attention_tracking,
                "allow_implicit_feedback": config.allow_implicit_feedback
            }
        }))?)
    }

    pub async fn set_operator_profile_consent(
        &self,
        consent_key: &str,
        granted: bool,
    ) -> Result<Vec<String>> {
        let mut config = self.config.read().await.clone();
        let mut updated_fields = Vec::new();
        match consent_key {
            "enabled" => {
                config.operator_model.enabled = granted;
                updated_fields.push("enabled".to_string());
            }
            "allow_message_statistics" => {
                config.operator_model.allow_message_statistics = granted;
                updated_fields.push("allow_message_statistics".to_string());
            }
            "allow_approval_learning" => {
                config.operator_model.allow_approval_learning = granted;
                updated_fields.push("allow_approval_learning".to_string());
            }
            "allow_attention_tracking" => {
                config.operator_model.allow_attention_tracking = granted;
                updated_fields.push("allow_attention_tracking".to_string());
            }
            "allow_implicit_feedback" => {
                config.operator_model.allow_implicit_feedback = granted;
                updated_fields.push("allow_implicit_feedback".to_string());
            }
            _ => anyhow::bail!("unknown operator profile consent key: {consent_key}"),
        }
        self.set_config(config).await;
        self.refresh_operator_model().await?;
        Ok(updated_fields)
    }

    pub async fn complete_operator_profile_session(
        &self,
        session_id: &str,
    ) -> Result<OperatorProfileCompletionPayload> {
        let session = {
            let mut sessions = self.operator_profile_sessions.write().await;
            let mut session = sessions
                .remove(session_id)
                .ok_or_else(|| anyhow::anyhow!("unknown operator profile session: {session_id}"))?;
            session.completed = true;
            session.updated_at = now_millis();
            session
        };

        let mut config = self.config.read().await.clone();
        let updated_fields = Self::apply_operator_profile_answers(&mut config, &session)?;
        self.set_config(config).await;
        self.refresh_operator_model().await?;
        self.history
            .delete_operator_profile_session(&session.session_id)
            .await?;
        Ok(OperatorProfileCompletionPayload {
            session_id: session.session_id,
            updated_fields,
        })
    }

    pub(super) async fn build_operator_model_prompt_summary(&self) -> Option<String> {
        let settings = self.config.read().await.operator_model.clone();
        if !settings.enabled {
            return None;
        }
        let model = self.operator_model.read().await;
        if model.cognitive_style.message_count == 0
            && model.risk_fingerprint.approval_requests == 0
            && model.attention_topology.focus_event_count == 0
        {
            return None;
        }

        let mut lines = Vec::new();
        if settings.allow_message_statistics && model.cognitive_style.message_count > 0 {
            lines.push(format!(
                "- Output density: {} (avg {:.1} words per message, questions {:.0}%)",
                verbosity_label(model.cognitive_style.verbosity_preference),
                model.cognitive_style.avg_message_length,
                model.cognitive_style.question_frequency * 100.0,
            ));
        }
        if settings.allow_approval_learning && model.risk_fingerprint.approval_requests > 0 {
            lines.push(format!(
                "- Risk tolerance: {} ({} approvals across {} approval requests, avg response {:.1}s)",
                risk_tolerance_label(model.risk_fingerprint.risk_tolerance),
                model.risk_fingerprint.approvals,
                model.risk_fingerprint.approval_requests,
                model.risk_fingerprint.avg_response_time_secs,
            ));
        }
        if settings.allow_message_statistics {
            if let Some(hour) = model.session_rhythm.typical_start_hour_utc {
                lines.push(format!(
                    "- Session rhythm: usually starts around {:02}:00 UTC; avg observed session {:.1}m",
                    hour, model.session_rhythm.session_duration_avg_minutes,
                ));
            }
        }
        if settings.allow_attention_tracking && model.attention_topology.focus_event_count > 0 {
            let dominant_surface = model
                .attention_topology
                .dominant_surface
                .as_deref()
                .unwrap_or("unknown");
            let transitions = if model.attention_topology.top_transitions.is_empty() {
                "no stable transitions yet".to_string()
            } else {
                model.attention_topology.top_transitions.join(", ")
            };
            lines.push(format!(
                "- Attention topology: mainly {} ({} focus events, avg dwell {:.1}s, rapid switches {}); common transitions {}; deep focus {}",
                dominant_surface,
                model.attention_topology.focus_event_count,
                model.attention_topology.avg_surface_dwell_secs,
                model.attention_topology.rapid_switch_count,
                transitions,
                model.attention_topology.deep_focus_surface.as_deref().unwrap_or("unknown"),
            ));
        }
        if settings.allow_implicit_feedback
            && (model.implicit_feedback.tool_hesitation_count > 0
                || model.implicit_feedback.revision_message_count > 0
                || model.implicit_feedback.fast_denial_count > 0)
        {
            let fallback = model
                .implicit_feedback
                .top_tool_fallbacks
                .first()
                .cloned()
                .unwrap_or_else(|| "none yet".to_string());
            lines.push(format!(
                "- Implicit feedback: {} tool fallback(s), {} revision-style operator message(s), {} fast denial(s); common fallback {}",
                model.implicit_feedback.tool_hesitation_count,
                model.implicit_feedback.revision_message_count,
                model.implicit_feedback.fast_denial_count,
                fallback,
            ));
        }
        if lines.is_empty() {
            return None;
        }

        Some(format!("## Operator Model\n{}", lines.join("\n")))
    }

    pub(super) async fn record_operator_message(
        &self,
        thread_id: &str,
        content: &str,
        is_new_thread: bool,
    ) -> Result<()> {
        let settings = self.config.read().await.operator_model.clone();
        if !settings.enabled
            || (!settings.allow_message_statistics && !settings.allow_implicit_feedback)
        {
            return Ok(());
        }
        ensure_operator_model_file(&self.data_dir).await?;
        let now = now_millis();
        let word_count = count_words(content) as f64;
        let is_question = content.contains('?');
        let confirmation_like = contains_confirmation_phrase(content);
        let revision_kind = detect_revision_signal(content);
        let current_hour_utc = current_utc_hour(now);

        let thread_created_at = {
            let threads = self.threads.read().await;
            threads.get(thread_id).map(|thread| thread.created_at)
        };

        let observed_minutes_delta = {
            let mut active_sessions = self.active_operator_sessions.write().await;
            if is_new_thread {
                active_sessions.insert(thread_id.to_string(), 0);
            }

            if let Some(created_at) = thread_created_at {
                let observed_minutes = now.saturating_sub(created_at) / 60_000;
                if let Some(previous_minutes) = active_sessions.get_mut(thread_id) {
                    if observed_minutes > *previous_minutes {
                        let delta = observed_minutes - *previous_minutes;
                        *previous_minutes = observed_minutes;
                        Some(delta)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        {
            let mut model = self.operator_model.write().await;
            model.last_updated = now;

            if settings.allow_message_statistics {
                let next_count = model.cognitive_style.message_count + 1;
                model.cognitive_style.avg_message_length = update_running_average(
                    model.cognitive_style.avg_message_length,
                    model.cognitive_style.message_count,
                    word_count,
                );
                model.cognitive_style.message_count = next_count;
                if is_question {
                    model.cognitive_style.question_count += 1;
                }
                if confirmation_like {
                    model.cognitive_style.confirmation_count += 1;
                }
                model.cognitive_style.question_frequency =
                    model.cognitive_style.question_count as f64 / next_count as f64;
                model.cognitive_style.confirmation_seeking =
                    model.cognitive_style.confirmation_count as f64 / next_count as f64;
                model.cognitive_style.verbosity_preference =
                    verbosity_preference_for_length(model.cognitive_style.avg_message_length);
                model.cognitive_style.reading_depth =
                    reading_depth_for_length(model.cognitive_style.avg_message_length);
            }
            if settings.allow_implicit_feedback {
                if revision_kind.is_revision() {
                    model.implicit_feedback.revision_message_count += 1;
                }
                if revision_kind.is_correction() {
                    model.implicit_feedback.correction_message_count += 1;
                }
            }

            if settings.allow_message_statistics {
                *model
                    .session_rhythm
                    .activity_hour_histogram
                    .entry(current_hour_utc)
                    .or_insert(0) += 1;
                model.session_rhythm.peak_activity_hours_utc =
                    top_hours(&model.session_rhythm.activity_hour_histogram, 3);

                if is_new_thread {
                    model.session_count += 1;
                    model.session_rhythm.session_count += 1;
                    *model
                        .session_rhythm
                        .start_hour_histogram
                        .entry(current_hour_utc)
                        .or_insert(0) += 1;
                    model.session_rhythm.typical_start_hour_utc =
                        most_common_hour(&model.session_rhythm.start_hour_histogram);
                }

                if let Some(delta) = observed_minutes_delta {
                    model.session_rhythm.total_observed_session_minutes += delta;
                    if model.session_rhythm.session_count > 0 {
                        model.session_rhythm.session_duration_avg_minutes =
                            model.session_rhythm.total_observed_session_minutes as f64
                                / model.session_rhythm.session_count as f64;
                    }
                }
            }

            persist_operator_model(&self.data_dir, &model)?;
        }
        self.record_behavioral_event(
            "operator_message",
            BehavioralEventContext {
                thread_id: Some(thread_id),
                task_id: None,
                goal_run_id: None,
                approval_id: None,
            },
            serde_json::json!({
                "is_new_thread": is_new_thread,
                "word_count": count_words(content),
                "is_question": is_question,
                "confirmation_like": confirmation_like,
                "revision_signal": format!("{revision_kind:?}").to_ascii_lowercase(),
            }),
        )
        .await?;

        Ok(())
    }

    pub(crate) async fn record_operator_approval_requested(
        &self,
        pending_approval: &ToolPendingApproval,
    ) -> Result<()> {
        let settings = self.config.read().await.operator_model.clone();
        if !settings.enabled || !settings.allow_approval_learning {
            return Ok(());
        }
        ensure_operator_model_file(&self.data_dir).await?;
        let category =
            classify_command_category(&pending_approval.command, &pending_approval.risk_level);
        self.pending_operator_approvals.write().await.insert(
            pending_approval.approval_id.clone(),
            PendingApprovalObservation {
                requested_at: now_millis(),
                category: category.to_string(),
            },
        );

        let mut model = self.operator_model.write().await;
        model.last_updated = now_millis();
        model.risk_fingerprint.approval_requests += 1;
        *model
            .risk_fingerprint
            .category_requests
            .entry(category.to_string())
            .or_insert(0) += 1;
        refresh_risk_metrics(&mut model.risk_fingerprint);
        persist_operator_model(&self.data_dir, &model)?;
        self.record_behavioral_event(
            "approval_requested",
            BehavioralEventContext {
                thread_id: None,
                task_id: None,
                goal_run_id: None,
                approval_id: Some(&pending_approval.approval_id),
            },
            serde_json::json!({
                "category": category,
                "command": pending_approval.command,
                "risk_level": pending_approval.risk_level,
            }),
        )
        .await?;
        Ok(())
    }

    pub async fn record_tool_hesitation(
        &self,
        from_tool: &str,
        to_tool: &str,
        from_error: bool,
        to_error: bool,
    ) -> Result<()> {
        let settings = self.config.read().await.operator_model.clone();
        if !settings.enabled || !settings.allow_implicit_feedback {
            return Ok(());
        }
        if !from_error || to_error {
            return Ok(());
        }
        let from_tool = from_tool.trim();
        let to_tool = to_tool.trim();
        if from_tool.is_empty() || to_tool.is_empty() || from_tool.eq_ignore_ascii_case(to_tool) {
            return Ok(());
        }

        ensure_operator_model_file(&self.data_dir).await?;
        let now = now_millis();
        let mut model = self.operator_model.write().await;
        model.last_updated = now;
        model.implicit_feedback.tool_hesitation_count += 1;
        let pair = format!("{from_tool} -> {to_tool}");
        *model
            .implicit_feedback
            .fallback_histogram
            .entry(pair)
            .or_insert(0) += 1;
        model.implicit_feedback.top_tool_fallbacks =
            top_keys(&model.implicit_feedback.fallback_histogram, 3);
        persist_operator_model(&self.data_dir, &model)?;
        self.record_behavioral_event(
            "tool_fallback",
            BehavioralEventContext {
                thread_id: None,
                task_id: None,
                goal_run_id: None,
                approval_id: None,
            },
            serde_json::json!({
                "from_tool": from_tool,
                "to_tool": to_tool,
                "from_error": from_error,
                "to_error": to_error,
            }),
        )
        .await?;
        Ok(())
    }

    pub async fn record_attention_surface(&self, surface: &str) -> Result<()> {
        let settings = self.config.read().await.operator_model.clone();
        if !settings.enabled || !settings.allow_attention_tracking {
            return Ok(());
        }
        let normalized = normalize_attention_surface(surface);
        if normalized.is_empty() {
            return Ok(());
        }

        ensure_operator_model_file(&self.data_dir).await?;
        let now = now_millis();
        let mut model = self.operator_model.write().await;
        model.last_updated = now;
        record_attention_event(&mut model, &normalized, now);
        persist_operator_model(&self.data_dir, &model)?;
        self.record_behavioral_event(
            "attention_surface",
            BehavioralEventContext {
                thread_id: None,
                task_id: None,
                goal_run_id: None,
                approval_id: None,
            },
            serde_json::json!({
                "surface": normalized,
            }),
        )
        .await?;
        Ok(())
    }

    pub async fn record_operator_approval_resolution(
        &self,
        approval_id: &str,
        decision: ApprovalDecision,
    ) -> Result<()> {
        let settings = self.config.read().await.operator_model.clone();
        if !settings.enabled || !settings.allow_approval_learning {
            return Ok(());
        }
        ensure_operator_model_file(&self.data_dir).await?;
        let pending = self
            .pending_operator_approvals
            .write()
            .await
            .remove(approval_id);
        let now = now_millis();

        let mut model = self.operator_model.write().await;
        model.last_updated = now;
        if matches!(
            decision,
            ApprovalDecision::ApproveOnce | ApprovalDecision::ApproveSession
        ) {
            model.risk_fingerprint.approvals += 1;
        } else {
            model.risk_fingerprint.denials += 1;
        }
        if let Some(pending) = pending {
            if matches!(
                decision,
                ApprovalDecision::ApproveOnce | ApprovalDecision::ApproveSession
            ) {
                *model
                    .risk_fingerprint
                    .category_approvals
                    .entry(pending.category)
                    .or_insert(0) += 1;
            }
            let response_secs = now.saturating_sub(pending.requested_at) as f64 / 1000.0;
            let responses = model.risk_fingerprint.approvals + model.risk_fingerprint.denials;
            model.risk_fingerprint.avg_response_time_secs = update_running_average(
                model.risk_fingerprint.avg_response_time_secs,
                responses.saturating_sub(1),
                response_secs,
            );
            if settings.allow_implicit_feedback
                && matches!(decision, ApprovalDecision::Deny)
                && response_secs <= 8.0
            {
                model.implicit_feedback.fast_denial_count += 1;
            }
        }
        refresh_risk_metrics(&mut model.risk_fingerprint);
        persist_operator_model(&self.data_dir, &model)?;
        self.record_behavioral_event(
            "approval_resolved",
            BehavioralEventContext {
                thread_id: None,
                task_id: None,
                goal_run_id: None,
                approval_id: Some(approval_id),
            },
            serde_json::json!({
                "decision": format!("{decision:?}").to_ascii_lowercase(),
            }),
        )
        .await?;
        Ok(())
    }

    pub(crate) fn operator_profile_diagnostics_snapshot(&self) -> serde_json::Value {
        let sync_state = match super::operator_profile::user_sync::current_user_sync_state() {
            super::operator_profile::user_sync::UserProfileSyncState::Clean => "clean",
            super::operator_profile::user_sync::UserProfileSyncState::Dirty => "dirty",
            super::operator_profile::user_sync::UserProfileSyncState::Reconciling => "reconciling",
        };
        serde_json::json!({
            "operator_profile_sync_state": sync_state,
            "operator_profile_sync_dirty": sync_state != "clean",
            "operator_profile_scheduler_fallback": false,
        })
    }
}

pub(super) async fn ensure_operator_model_file(agent_data_dir: &std::path::Path) -> Result<()> {
    let path = operator_model_path(agent_data_dir);
    if !path.exists() {
        let default_json = serde_json::to_string_pretty(&OperatorModel::default())?;
        tokio::fs::write(path, default_json).await?;
    }
    Ok(())
}

fn persist_operator_model(agent_data_dir: &std::path::Path, model: &OperatorModel) -> Result<()> {
    let path = operator_model_path(agent_data_dir);
    let json = serde_json::to_string_pretty(model)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub(super) fn operator_model_path(agent_data_dir: &std::path::Path) -> std::path::PathBuf {
    agent_data_dir.join("operator_model.json")
}

fn count_words(content: &str) -> usize {
    content
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .count()
}

fn contains_confirmation_phrase(content: &str) -> bool {
    let lowered = content.to_ascii_lowercase();
    [
        "are you sure",
        "double check",
        "double-check",
        "confirm",
        "really",
    ]
    .iter()
    .any(|needle| lowered.contains(needle))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RevisionSignal {
    None,
    Revision,
    Correction,
}

impl RevisionSignal {
    fn is_revision(self) -> bool {
        matches!(self, Self::Revision | Self::Correction)
    }

    fn is_correction(self) -> bool {
        matches!(self, Self::Correction)
    }
}

fn detect_revision_signal(content: &str) -> RevisionSignal {
    let lowered = content.trim().to_ascii_lowercase();
    if lowered.is_empty() {
        return RevisionSignal::None;
    }

    let correction_markers = [
        "actually",
        "instead",
        "rather than",
        "undo",
        "revert",
        "change that",
        "not that",
        "no, ",
        "don't do that",
    ];
    if correction_markers
        .iter()
        .any(|needle| lowered.contains(needle))
    {
        return RevisionSignal::Correction;
    }

    let revision_markers = ["use ", "prefer ", "switch to ", "next time", "better to "];
    if revision_markers
        .iter()
        .any(|needle| lowered.contains(needle))
    {
        return RevisionSignal::Revision;
    }

    RevisionSignal::None
}

fn current_utc_hour(timestamp_ms: u64) -> u8 {
    ((timestamp_ms / 3_600_000) % 24) as u8
}

fn update_running_average(current: f64, sample_count: u64, new_value: f64) -> f64 {
    if sample_count == 0 {
        return new_value;
    }
    ((current * sample_count as f64) + new_value) / (sample_count as f64 + 1.0)
}

fn verbosity_preference_for_length(avg_words: f64) -> VerbosityPreference {
    if avg_words < 10.0 {
        VerbosityPreference::Terse
    } else if avg_words > 35.0 {
        VerbosityPreference::Verbose
    } else {
        VerbosityPreference::Moderate
    }
}

fn reading_depth_for_length(avg_words: f64) -> ReadingDepth {
    if avg_words < 10.0 {
        ReadingDepth::Skim
    } else if avg_words > 35.0 {
        ReadingDepth::Deep
    } else {
        ReadingDepth::Standard
    }
}

fn classify_command_category(command: &str, risk_level: &str) -> &'static str {
    let lowered = command.to_ascii_lowercase();
    if lowered.contains("rm ") || lowered.contains("rm -") || lowered.contains("del ") {
        "destructive_delete"
    } else if lowered.contains("curl ")
        || lowered.contains("wget ")
        || lowered.contains("http")
        || lowered.contains("ssh ")
    {
        "network_request"
    } else if lowered.contains("git ") {
        "git"
    } else if lowered.contains("mv ")
        || lowered.contains("cp ")
        || lowered.contains("tee ")
        || lowered.contains("sed -i")
        || lowered.contains("python")
    {
        "file_write"
    } else if !risk_level.trim().is_empty() {
        match risk_level {
            "highest" => "high_risk",
            "lowest" | "yolo" => "low_risk",
            _ => "moderate_risk",
        }
    } else {
        "other"
    }
}

fn refresh_risk_metrics(risk: &mut RiskFingerprint) {
    risk.approval_rate_by_category = risk
        .category_requests
        .iter()
        .map(|(category, requests)| {
            let approvals = risk.category_approvals.get(category).copied().unwrap_or(0);
            let rate = if *requests == 0 {
                0.0
            } else {
                approvals as f64 / *requests as f64
            };
            (category.clone(), rate)
        })
        .collect();

    let total_resolved = risk.approvals + risk.denials;
    let approval_rate = if total_resolved == 0 {
        0.0
    } else {
        risk.approvals as f64 / total_resolved as f64
    };
    risk.risk_tolerance = if approval_rate < 0.35 {
        RiskTolerance::Conservative
    } else if approval_rate > 0.75 {
        RiskTolerance::Aggressive
    } else {
        RiskTolerance::Moderate
    };
}

fn most_common_hour(histogram: &HashMap<u8, u64>) -> Option<u8> {
    histogram
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(hour, _)| *hour)
}

fn top_hours(histogram: &HashMap<u8, u64>, limit: usize) -> Vec<u8> {
    let mut entries = histogram
        .iter()
        .map(|(hour, count)| (*hour, *count))
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    entries
        .into_iter()
        .take(limit)
        .map(|(hour, _)| hour)
        .collect()
}

fn most_common_key(histogram: &HashMap<String, u64>) -> Option<String> {
    histogram
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(key, _)| key.clone())
}

fn top_keys(histogram: &HashMap<String, u64>, limit: usize) -> Vec<String> {
    let mut entries = histogram
        .iter()
        .map(|(key, count)| (key.clone(), *count))
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    entries
        .into_iter()
        .take(limit)
        .map(|(key, _)| key)
        .collect()
}

fn record_attention_event(model: &mut OperatorModel, normalized_surface: &str, now_ms: u64) {
    model.attention_topology.focus_event_count += 1;
    *model
        .attention_topology
        .surface_histogram
        .entry(normalized_surface.to_string())
        .or_insert(0) += 1;

    if let (Some(previous), Some(previous_at)) = (
        model.attention_topology.last_surface.as_ref(),
        model.attention_topology.last_surface_at,
    ) {
        let dwell_secs = now_ms.saturating_sub(previous_at) / 1000;
        if dwell_secs > 0 {
            *model
                .attention_topology
                .dwell_histogram
                .entry(previous.clone())
                .or_insert(0) += dwell_secs;
            model.attention_topology.avg_surface_dwell_secs = update_running_average(
                model.attention_topology.avg_surface_dwell_secs,
                model.attention_topology.focus_event_count.saturating_sub(2),
                dwell_secs as f64,
            );
            if dwell_secs <= 15 && previous != normalized_surface {
                model.attention_topology.rapid_switch_count += 1;
            }
        }
        if previous != normalized_surface {
            let transition = format!("{previous} -> {normalized_surface}");
            *model
                .attention_topology
                .transition_histogram
                .entry(transition)
                .or_insert(0) += 1;
        }
    }

    model.attention_topology.last_surface = Some(normalized_surface.to_string());
    model.attention_topology.last_surface_at = Some(now_ms);
    model.attention_topology.dominant_surface =
        most_common_key(&model.attention_topology.surface_histogram);
    model.attention_topology.common_surfaces =
        top_keys(&model.attention_topology.surface_histogram, 3);
    model.attention_topology.top_transitions =
        top_keys(&model.attention_topology.transition_histogram, 3);
    model.attention_topology.deep_focus_surface =
        most_common_key(&model.attention_topology.dwell_histogram);
}

fn normalize_attention_surface(surface: &str) -> String {
    surface
        .trim()
        .chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, ':' | '_' | '-') {
                Some(ch.to_ascii_lowercase())
            } else if ch.is_whitespace() {
                Some('_')
            } else {
                None
            }
        })
        .collect()
}

fn verbosity_label(value: VerbosityPreference) -> &'static str {
    match value {
        VerbosityPreference::Terse => "terse",
        VerbosityPreference::Moderate => "moderate",
        VerbosityPreference::Verbose => "verbose",
    }
}

fn risk_tolerance_label(value: RiskTolerance) -> &'static str {
    match value {
        RiskTolerance::Conservative => "conservative",
        RiskTolerance::Moderate => "moderate",
        RiskTolerance::Aggressive => "aggressive",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cognitive_style_prefers_terse_for_short_messages() {
        assert_eq!(
            verbosity_preference_for_length(6.0),
            VerbosityPreference::Terse
        );
        assert_eq!(reading_depth_for_length(6.0), ReadingDepth::Skim);
    }

    #[test]
    fn risk_metrics_compute_category_rates_and_tolerance() {
        let mut risk = RiskFingerprint {
            approvals: 4,
            denials: 1,
            category_requests: HashMap::from([
                ("git".to_string(), 2),
                ("network_request".to_string(), 3),
            ]),
            category_approvals: HashMap::from([
                ("git".to_string(), 2),
                ("network_request".to_string(), 2),
            ]),
            ..RiskFingerprint::default()
        };

        refresh_risk_metrics(&mut risk);

        assert_eq!(risk.risk_tolerance, RiskTolerance::Aggressive);
        assert_eq!(
            risk.approval_rate_by_category.get("git").copied(),
            Some(1.0)
        );
        assert_eq!(
            risk.approval_rate_by_category
                .get("network_request")
                .copied(),
            Some(2.0 / 3.0)
        );
    }

    #[test]
    fn classify_command_category_uses_command_heuristics_first() {
        assert_eq!(
            classify_command_category("rm -rf target", "highest"),
            "destructive_delete"
        );
        assert_eq!(
            classify_command_category("curl https://example.com", "moderate"),
            "network_request"
        );
    }

    #[test]
    fn normalize_attention_surface_keeps_supported_separators() {
        assert_eq!(
            normalize_attention_surface(" modal:settings:SubAgents "),
            "modal:settings:subagents"
        );
    }

    #[test]
    fn top_keys_orders_by_count_then_name() {
        let mut histogram = HashMap::new();
        histogram.insert("conversation:chat".to_string(), 4);
        histogram.insert("conversation:input".to_string(), 4);
        histogram.insert("modal:settings:provider".to_string(), 1);

        assert_eq!(
            top_keys(&histogram, 2),
            vec![
                "conversation:chat".to_string(),
                "conversation:input".to_string()
            ]
        );
    }

    #[test]
    fn detect_revision_signal_finds_corrections() {
        assert_eq!(
            detect_revision_signal("Actually, use ripgrep instead."),
            RevisionSignal::Correction
        );
        assert_eq!(
            detect_revision_signal("Next time prefer the shorter answer."),
            RevisionSignal::Revision
        );
        assert_eq!(
            detect_revision_signal("Please run tests."),
            RevisionSignal::None
        );
    }

    // ── EMA smoothing tests (BEAT-06/D-02) ────────────────────────────

    #[test]
    fn ema_update_basic_calculation() {
        let result = ema_update(10.0, 20.0, 0.3);
        // 0.3 * 20 + 0.7 * 10 = 6 + 7 = 13
        assert!((result - 13.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ema_update_converges_toward_sample() {
        let mut value = 0.0;
        for _ in 0..50 {
            value = ema_update(value, 100.0, 0.3);
        }
        // After 50 iterations with alpha=0.3, value should be very close to 100
        assert!((value - 100.0).abs() < 0.01);
    }

    #[test]
    fn smooth_activity_histogram_applies_ema_to_all_24_hours() {
        let current: HashMap<u8, f64> = HashMap::new();
        let mut observation: HashMap<u8, u64> = HashMap::new();
        observation.insert(9, 5);
        observation.insert(14, 3);

        let smoothed = smooth_activity_histogram(&current, &observation, 0.3);
        assert_eq!(smoothed.len(), 24);
        // hour 9: ema_update(0.0, 5.0, 0.3) = 1.5
        assert!((smoothed[&9] - 1.5).abs() < f64::EPSILON);
        // hour 14: ema_update(0.0, 3.0, 0.3) = 0.9
        assert!((smoothed[&14] - 0.9).abs() < f64::EPSILON);
        // hour 0 (unobserved): ema_update(0.0, 0.0, 0.3) = 0.0
        assert!((smoothed[&0]).abs() < f64::EPSILON);
    }

    #[test]
    fn smooth_activity_histogram_decays_unobserved_hours() {
        let mut current: HashMap<u8, f64> = HashMap::new();
        current.insert(9, 10.0); // previously active at hour 9
        let observation: HashMap<u8, u64> = HashMap::new(); // no activity this session

        let smoothed = smooth_activity_histogram(&current, &observation, 0.3);
        // hour 9: ema_update(10.0, 0.0, 0.3) = 0.7 * 10.0 = 7.0
        assert!((smoothed[&9] - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn record_attention_event_tracks_dwell_and_rapid_switches() {
        let mut model = OperatorModel::default();
        record_attention_event(&mut model, "conversation:chat", 1_000);
        record_attention_event(&mut model, "modal:settings", 6_000);
        record_attention_event(&mut model, "conversation:chat", 10_000);
        record_attention_event(&mut model, "conversation:chat", 50_000);

        assert_eq!(model.attention_topology.focus_event_count, 4);
        assert_eq!(model.attention_topology.rapid_switch_count, 2);
        assert_eq!(
            model.attention_topology.deep_focus_surface.as_deref(),
            Some("conversation:chat")
        );
        assert!(model.attention_topology.avg_surface_dwell_secs > 0.0);
    }
}
