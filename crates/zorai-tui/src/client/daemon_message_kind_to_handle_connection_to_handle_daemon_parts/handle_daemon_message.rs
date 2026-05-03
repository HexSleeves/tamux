impl DaemonClient {
    async fn handle_daemon_message(
        message: DaemonMessage,
        event_tx: &mpsc::Sender<ClientEvent>,
        thread_detail_chunks: &mut Option<ThreadDetailChunkBuffer>,
    ) -> bool {
        match message {
            message @ (DaemonMessage::AgentEvent { .. }
            | DaemonMessage::AgentThreadList { .. }
            | DaemonMessage::AgentThreadDetail { .. }
            | DaemonMessage::AgentThreadDetailChunk { .. }
            | DaemonMessage::AgentTaskList { .. }
            | DaemonMessage::AgentGoalRunList { .. }
            | DaemonMessage::AgentGoalRunStarted { .. }
            | DaemonMessage::AgentGoalRunDetail { .. }
            | DaemonMessage::AgentCheckpointList { .. }
            | DaemonMessage::AgentCheckpointRestored { .. }
            | DaemonMessage::AgentTodoDetail { .. }
            | DaemonMessage::AgentWorkContextDetail { .. }
            | DaemonMessage::GitDiff { .. }
            | DaemonMessage::FilePreview { .. }
            | DaemonMessage::AgentConfigResponse { .. }
            | DaemonMessage::AgentExternalRuntimeMigrationResult { .. }
            | DaemonMessage::AgentModelsResponse { .. }
            | DaemonMessage::AgentHeartbeatItems { .. }
            | DaemonMessage::AgentEventRows { .. }
            | DaemonMessage::AgentDbMessageAck
            | DaemonMessage::SessionSpawned { .. }
            | DaemonMessage::ApprovalRequired { .. }
            | DaemonMessage::AgentTaskApprovalRules { .. }
            | DaemonMessage::ApprovalResolved { .. }
            | DaemonMessage::AgentWorkspaceSettings { .. }
            | DaemonMessage::AgentWorkspaceSettingsList { .. }
            | DaemonMessage::AgentWorkspaceTaskList { .. }
            | DaemonMessage::AgentWorkspaceTaskUpdated { .. }
            | DaemonMessage::AgentWorkspaceTaskDeleted { .. }
            | DaemonMessage::AgentWorkspaceNoticeList { .. }
            | DaemonMessage::AgentWorkspaceError { .. }) => {
                Self::handle_thread_workspace_daemon_messages(message, event_tx, thread_detail_chunks).await
            }
            message @ (DaemonMessage::AgentProviderAuthStates { .. }
            | DaemonMessage::AgentProviderCatalog { .. }
            | DaemonMessage::AgentOpenAICodexAuthStatus { .. }
            | DaemonMessage::AgentOpenAICodexAuthLoginResult { .. }
            | DaemonMessage::AgentOpenAICodexAuthLogoutResult { .. }
            | DaemonMessage::AgentProviderValidation { .. }
            | DaemonMessage::AgentSubAgentList { .. }
            | DaemonMessage::AgentSubAgentUpdated { .. }
            | DaemonMessage::AgentSubAgentRemoved { .. }
            | DaemonMessage::AgentConciergeConfig { .. }
            | DaemonMessage::PluginListResult { .. }
            | DaemonMessage::PluginGetResult { .. }
            | DaemonMessage::PluginSettingsResult { .. }
            | DaemonMessage::PluginTestConnectionResult { .. }
            | DaemonMessage::PluginActionResult { .. }
            | DaemonMessage::PluginCommandsResult { .. }
            | DaemonMessage::PluginOAuthUrl { .. }
            | DaemonMessage::PluginOAuthComplete { .. }) => {
                Self::handle_provider_plugin_daemon_messages(message, event_tx).await
            }
            message @ (DaemonMessage::AgentWhatsAppLinkStatus { .. }
            | DaemonMessage::AgentThreadMessagePinResult { .. }
            | DaemonMessage::AgentWhatsAppLinkQr { .. }
            | DaemonMessage::AgentWhatsAppLinked { .. }
            | DaemonMessage::AgentWhatsAppLinkError { .. }
            | DaemonMessage::AgentWhatsAppLinkDisconnected { .. }
            | DaemonMessage::AgentExplanation { .. }
            | DaemonMessage::AgentDivergentSessionStarted { .. }
            | DaemonMessage::AgentDivergentSession { .. }
            | DaemonMessage::AgentStatusResponse { .. }
            | DaemonMessage::AgentStatisticsResponse { .. }
            | DaemonMessage::AgentPromptInspection { .. }
            | DaemonMessage::AgentOperatorProfileSessionStarted { .. }
            | DaemonMessage::AgentOperatorProfileQuestion { .. }
            | DaemonMessage::AgentOperatorProfileProgress { .. }
            | DaemonMessage::AgentOperatorProfileSummary { .. }
            | DaemonMessage::AgentOperatorModel { .. }
            | DaemonMessage::AgentOperatorModelReset { .. }
            | DaemonMessage::AgentCollaborationSessions { .. }
            | DaemonMessage::AgentCollaborationVoteResult { .. }
            | DaemonMessage::AgentGeneratedTools { .. }
            | DaemonMessage::AgentSpeechToTextResult { .. }
            | DaemonMessage::AgentTextToSpeechResult { .. }
            | DaemonMessage::AgentGenerateImageResult { .. }
            | DaemonMessage::AgentOperatorProfileSessionCompleted { .. }
            | DaemonMessage::AgentError { .. }
            | DaemonMessage::GatewayBootstrap { .. }
            | DaemonMessage::GatewaySendRequest { .. }
            | DaemonMessage::GatewayReloadCommand { .. }
            | DaemonMessage::GatewayShutdownCommand { .. }
            | DaemonMessage::Error { .. }) => {
                Self::handle_activity_profile_gateway_daemon_messages(message, event_tx).await
            }
            other => {
                debug!("Ignoring daemon message: {:?}", other);
            }
        }

        true
    }

}
