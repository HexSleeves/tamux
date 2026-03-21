//! Message and thread metadata parsing/building helpers.

use super::*;

pub(super) struct ParsedMessageMetadata {
    pub tool_call_id: Option<String>,
    pub tool_name: Option<String>,
    pub tool_arguments: Option<String>,
    pub tool_status: Option<String>,
    pub api_transport: Option<ApiTransport>,
    pub response_id: Option<String>,
}

pub(super) struct ParsedThreadMetadata {
    pub upstream_thread_id: Option<String>,
    pub upstream_transport: Option<ApiTransport>,
    pub upstream_provider: Option<String>,
    pub upstream_model: Option<String>,
    pub upstream_assistant_id: Option<String>,
}

pub(super) fn parse_message_metadata(metadata_json: Option<&str>) -> ParsedMessageMetadata {
    let metadata =
        metadata_json.and_then(|json| serde_json::from_str::<serde_json::Value>(json).ok());
    let get_str = |keys: &[&str]| -> Option<String> {
        metadata.as_ref().and_then(|value| {
            keys.iter()
                .find_map(|key| value.get(*key).and_then(|entry| entry.as_str()))
                .map(ToOwned::to_owned)
        })
    };
    let api_transport = metadata
        .as_ref()
        .and_then(|value| {
            value
                .get("api_transport")
                .or_else(|| value.get("apiTransport"))
        })
        .and_then(|value| serde_json::from_value::<ApiTransport>(value.clone()).ok());

    ParsedMessageMetadata {
        tool_call_id: get_str(&["tool_call_id", "toolCallId"]),
        tool_name: get_str(&["tool_name", "toolName"]),
        tool_arguments: get_str(&["tool_arguments", "toolArguments"]),
        tool_status: get_str(&["tool_status", "toolStatus"]),
        api_transport,
        response_id: get_str(&["response_id", "responseId"]),
    }
}

pub(super) fn parse_thread_metadata(metadata_json: Option<&str>) -> ParsedThreadMetadata {
    let metadata =
        metadata_json.and_then(|json| serde_json::from_str::<serde_json::Value>(json).ok());
    let get_str = |keys: &[&str]| -> Option<String> {
        metadata.as_ref().and_then(|value| {
            keys.iter()
                .find_map(|key| value.get(*key).and_then(|entry| entry.as_str()))
                .map(ToOwned::to_owned)
        })
    };
    let upstream_transport = metadata
        .as_ref()
        .and_then(|value| {
            value
                .get("upstream_transport")
                .or_else(|| value.get("upstreamTransport"))
        })
        .and_then(|value| serde_json::from_value::<ApiTransport>(value.clone()).ok());

    ParsedThreadMetadata {
        upstream_thread_id: get_str(&["upstream_thread_id", "upstreamThreadId"]),
        upstream_transport,
        upstream_provider: get_str(&["upstream_provider", "upstreamProvider"]),
        upstream_model: get_str(&["upstream_model", "upstreamModel"]),
        upstream_assistant_id: get_str(&["upstream_assistant_id", "upstreamAssistantId"]),
    }
}

pub(super) fn build_message_metadata_json(message: &AgentMessage) -> Option<String> {
    serde_json::to_string(&serde_json::json!({
        "tool_call_id": message.tool_call_id,
        "tool_name": message.tool_name,
        "toolName": message.tool_name,
        "toolCallId": message.tool_call_id,
        "toolArguments": message.tool_arguments,
        "toolStatus": message.tool_status,
        "api_transport": message.api_transport,
        "response_id": message.response_id,
    }))
    .ok()
}

pub(super) fn build_thread_metadata_json(thread: &AgentThread) -> Option<String> {
    serde_json::to_string(&serde_json::json!({
        "upstream_thread_id": thread.upstream_thread_id,
        "upstreamThreadId": thread.upstream_thread_id,
        "upstream_transport": thread.upstream_transport,
        "upstreamTransport": thread.upstream_transport,
        "upstream_provider": thread.upstream_provider,
        "upstreamProvider": thread.upstream_provider,
        "upstream_model": thread.upstream_model,
        "upstreamModel": thread.upstream_model,
        "upstream_assistant_id": thread.upstream_assistant_id,
        "upstreamAssistantId": thread.upstream_assistant_id,
    }))
    .ok()
}
