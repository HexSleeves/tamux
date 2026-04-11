use super::*;

fn parse_participant_suggestion_response(response: &str) -> Option<(bool, String)> {
    let trimmed = response.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("NO_SUGGESTION") {
        return None;
    }

    let mut force_send = false;
    let mut message_line: Option<String> = None;
    for line in trimmed.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("FORCE:") {
            force_send = matches!(rest.trim().to_ascii_lowercase().as_str(), "yes" | "true");
            continue;
        }
        if let Some(rest) = line.strip_prefix("MESSAGE:") {
            let message = rest.trim();
            if !message.is_empty() {
                message_line = Some(message.to_string());
            }
        }
    }

    message_line
        .map(|message| (force_send, message))
        .or_else(|| Some((force_send, trimmed.to_string())))
}

fn should_hide_participant_prompt_message(message: &AgentMessage) -> bool {
    matches!(message.role, MessageRole::System)
        || message
            .tool_name
            .as_deref()
            .map(|name| name == "internal_delegate")
            .unwrap_or(false)
}

impl AgentEngine {
    pub async fn build_participant_prompt(
        &self,
        thread_id: &str,
        target_agent_id: &str,
    ) -> Result<String> {
        let participants = self.list_thread_participants(thread_id).await;
        let participant = participants
            .iter()
            .find(|participant| {
                participant.agent_id.eq_ignore_ascii_case(target_agent_id)
                    && participant.status == ThreadParticipantStatus::Active
            })
            .ok_or_else(|| anyhow::anyhow!("participant is not active on thread: {target_agent_id}"))?;
        let thread = self
            .get_thread(thread_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("thread not found: {thread_id}"))?;

        let mut prompt = String::new();
        prompt.push_str("Role: participant observer\n");
        prompt.push_str(&format!("Participant: {}\n", participant.agent_name));
        prompt.push_str(&format!("Instruction: {}\n\n", participant.instruction));
        prompt.push_str("Respond with either:\n");
        prompt.push_str("- NO_SUGGESTION\n");
        prompt.push_str("- FORCE: yes|no\n  MESSAGE: <text>\n\n");
        prompt.push_str("Visible thread:\n");
        for message in thread.messages.iter().filter(|message| !should_hide_participant_prompt_message(message)) {
            let role = match message.role {
                MessageRole::Assistant => "assistant",
                MessageRole::Tool => "tool",
                MessageRole::System => "system",
                MessageRole::User => "user",
            };
            if !message.content.trim().is_empty() {
                prompt.push_str(&format!("- {role}: {}\n", message.content.trim()));
            }
        }
        Ok(prompt)
    }

    pub async fn append_internal_delegate_message(
        &self,
        thread_id: &str,
        content: &str,
    ) -> Result<()> {
        let mut threads = self.threads.write().await;
        let thread = threads
            .get_mut(thread_id)
            .ok_or_else(|| anyhow::anyhow!("thread not found: {thread_id}"))?;
        thread.messages.push(AgentMessage {
            id: generate_message_id(),
            role: MessageRole::System,
            content: content.to_string(),
            tool_calls: None,
            tool_call_id: None,
            tool_name: Some("internal_delegate".to_string()),
            tool_arguments: None,
            tool_status: None,
            weles_review: None,
            input_tokens: 0,
            output_tokens: 0,
            provider: None,
            model: None,
            api_transport: None,
            response_id: None,
            upstream_message: None,
            provider_final_result: None,
            author_agent_id: None,
            author_agent_name: None,
            reasoning: None,
            message_kind: AgentMessageKind::Normal,
            compaction_strategy: None,
            compaction_payload: None,
            offloaded_payload_id: None,
            structural_refs: Vec::new(),
            timestamp: now_millis(),
        });
        thread.updated_at = now_millis();
        drop(threads);
        self.persist_thread_by_id(thread_id).await;
        Ok(())
    }

    pub async fn run_participant_observers(&self, thread_id: &str) -> Result<()> {
        let participants = self.list_thread_participants(thread_id).await;
        if participants.is_empty() {
            return Ok(());
        }

        let sender = self
            .thread_handoff_state(thread_id)
            .await
            .map(|state| state.active_agent_id)
            .unwrap_or_else(|| MAIN_AGENT_ID.to_string());

        for participant in participants.into_iter().filter(|participant| participant.status == ThreadParticipantStatus::Active) {
            let prompt = self.build_participant_prompt(thread_id, &participant.agent_id).await?;
            let result = self
                .send_internal_agent_message(&sender, &participant.agent_id, &prompt, None)
                .await?;
            let Some((force_send, message)) = parse_participant_suggestion_response(&result.response) else {
                continue;
            };
            self.queue_thread_participant_suggestion(
                thread_id,
                &participant.agent_id,
                &message,
                force_send,
            )
                .await?;
        }

        Ok(())
    }
}