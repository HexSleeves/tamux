#![allow(dead_code)]

include!("mod_parts/close_request_queue_for_test.rs");
#[derive(Debug, Default)]
struct ThreadDetailChunkBuffer {
    thread_id: Option<String>,
    bytes: Vec<u8>,
}

include!("daemon_message_kind_to_handle_connection_to_handle_daemon_message.rs");
include!("handle_thread_workspace_and_provider_daemon_messages.rs");
include!("handle_activity_profile_gateway_daemon_messages.rs");
include!("is_internal_agent_thread_to_request_git_diff.rs");
include!("request_agent_status_to_defer_operator_profile_question_to_get_operator.rs");

include!("mod_parts/get_string_lossy.rs");
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    use zorai_protocol::ClientMessage;

    fn drain_request(rx: &mut mpsc::UnboundedReceiver<ClientMessage>) -> ClientMessage {
        rx.try_recv().expect("expected queued client message")
    }

    include!("tests/whatsapp_link_methods_send_expected_protocol_messages_to_resolve_task.rs");
    include!("tests/bootstrap_rearms_after_successful_connection_cycle_to_daemon_bootstrap.rs");
    include!("tests/daemon_collaboration_sessions_reply_emits_client_event_to_workspace.rs");
}
