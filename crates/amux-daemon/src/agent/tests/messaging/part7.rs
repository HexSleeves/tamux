use super::*;
use amux_shared::providers::PROVIDER_ID_OPENAI;
use tempfile::TempDir;

async fn make_runner_test_engine(config: AgentConfig) -> (Arc<AgentEngine>, TempDir) {
    let temp_dir = TempDir::new().expect("temp dir");
    let session_manager = SessionManager::new_test(temp_dir.path()).await;
    let history = HistoryStore::new_test_store(temp_dir.path())
        .await
        .expect("history store");
    let data_dir = temp_dir.path().join("agent");
    std::fs::create_dir_all(&data_dir).expect("create agent data dir");
    let engine = AgentEngine::new_with_storage_and_http_client(
        session_manager,
        config,
        history,
        data_dir,
        reqwest::Client::new(),
    );
    (engine, temp_dir)
}

#[tokio::test]
async fn participant_runner_enqueues_suggestion() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");

    tokio::spawn(async move {
        for _ in 0..3 {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };
            let _ = read_http_request_body(&mut socket).await.expect("read request");
            let response_body = concat!(
                "data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_participant_runner\"}}\n\n",
                "data: {\"type\":\"response.output_text.delta\",\"delta\":\"FORCE: no\\nMESSAGE: Verify claim X before sending.\"}\n\n",
                "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp_participant_runner\",\"object\":\"response\",\"status\":\"completed\",\"output\":[],\"usage\":{\"input_tokens\":6,\"output_tokens\":7},\"error\":null}}\n\n"
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            socket.write_all(response.as_bytes()).await.expect("write response");
        }
    });

    let mut config = AgentConfig::default();
    config.provider = PROVIDER_ID_OPENAI.to_string();
    config.base_url = format!("http://{addr}/v1");
    config.model = "gpt-5.4-mini".to_string();
    config.api_key = "test-key".to_string();
    config.auth_source = AuthSource::ApiKey;
    config.api_transport = ApiTransport::Responses;
    config.auto_retry = false;
    config.max_retries = 0;
    config.max_tool_loops = 1;
    let (engine, _temp_dir) = make_runner_test_engine(config).await;
    let thread_id = "thread_participant_runner";

    engine.threads.write().await.insert(
        thread_id.to_string(),
        AgentThread {
            id: thread_id.to_string(),
            agent_name: Some(crate::agent::agent_identity::MAIN_AGENT_NAME.to_string()),
            title: "Participant runner".to_string(),
            messages: vec![AgentMessage::user("Check claim X", 1)],
            pinned: false,
            upstream_thread_id: None,
            upstream_transport: None,
            upstream_provider: None,
            upstream_model: None,
            upstream_assistant_id: None,
            total_input_tokens: 0,
            total_output_tokens: 0,
            created_at: 1,
            updated_at: 1,
        },
    );
    engine
        .upsert_thread_participant(thread_id, "weles", "verify claims")
        .await
        .expect("participant should register");

    engine
        .run_participant_observers(thread_id)
        .await
        .expect("participant observers should run");

    let suggestions = engine.list_thread_participant_suggestions(thread_id).await;
    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].instruction, "Verify claim X before sending.");
}

#[tokio::test]
async fn participant_runner_skips_no_suggestion() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");

    tokio::spawn(async move {
        for _ in 0..3 {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };
            let _ = read_http_request_body(&mut socket).await.expect("read request");
            let response_body = concat!(
                "data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_participant_runner_none\"}}\n\n",
                "data: {\"type\":\"response.output_text.delta\",\"delta\":\"NO_SUGGESTION\"}\n\n",
                "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp_participant_runner_none\",\"object\":\"response\",\"status\":\"completed\",\"output\":[],\"usage\":{\"input_tokens\":4,\"output_tokens\":2},\"error\":null}}\n\n"
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            socket.write_all(response.as_bytes()).await.expect("write response");
        }
    });

    let mut config = AgentConfig::default();
    config.provider = PROVIDER_ID_OPENAI.to_string();
    config.base_url = format!("http://{addr}/v1");
    config.model = "gpt-5.4-mini".to_string();
    config.api_key = "test-key".to_string();
    config.auth_source = AuthSource::ApiKey;
    config.api_transport = ApiTransport::Responses;
    config.auto_retry = false;
    config.max_retries = 0;
    config.max_tool_loops = 1;
    let (engine, _temp_dir) = make_runner_test_engine(config).await;
    let thread_id = "thread_participant_runner_none";

    engine.threads.write().await.insert(
        thread_id.to_string(),
        AgentThread {
            id: thread_id.to_string(),
            agent_name: Some(crate::agent::agent_identity::MAIN_AGENT_NAME.to_string()),
            title: "Participant runner none".to_string(),
            messages: vec![AgentMessage::user("noop", 1)],
            pinned: false,
            upstream_thread_id: None,
            upstream_transport: None,
            upstream_provider: None,
            upstream_model: None,
            upstream_assistant_id: None,
            total_input_tokens: 0,
            total_output_tokens: 0,
            created_at: 1,
            updated_at: 1,
        },
    );
    engine
        .upsert_thread_participant(thread_id, "weles", "verify claims")
        .await
        .expect("participant should register");

    engine
        .run_participant_observers(thread_id)
        .await
        .expect("participant observers should run");

    assert!(engine
        .list_thread_participant_suggestions(thread_id)
        .await
        .is_empty());
}

#[tokio::test]
async fn participant_suggestions_not_visible_to_participants() {
    let (engine, _temp_dir) = make_runner_test_engine(AgentConfig::default()).await;
    let thread_id = "thread_participant_prompt_hidden_suggestion";

    engine.threads.write().await.insert(
        thread_id.to_string(),
        AgentThread {
            id: thread_id.to_string(),
            agent_name: Some(crate::agent::agent_identity::MAIN_AGENT_NAME.to_string()),
            title: "Prompt visibility".to_string(),
            messages: vec![AgentMessage::user("hello", 1)],
            pinned: false,
            upstream_thread_id: None,
            upstream_transport: None,
            upstream_provider: None,
            upstream_model: None,
            upstream_assistant_id: None,
            total_input_tokens: 0,
            total_output_tokens: 0,
            created_at: 1,
            updated_at: 1,
        },
    );
    engine
        .upsert_thread_participant(thread_id, "weles", "verify claims")
        .await
        .expect("participant should register");
    engine
        .queue_thread_participant_suggestion(thread_id, "weles", "Hidden", false)
        .await
        .expect("queue suggestion");

    let prompt = engine
        .build_participant_prompt(thread_id, "weles")
        .await
        .expect("prompt should build");
    assert!(!prompt.contains("Hidden"));
}

#[tokio::test]
async fn participant_prompt_excludes_internal_delegation() {
    let (engine, _temp_dir) = make_runner_test_engine(AgentConfig::default()).await;
    let thread_id = "thread_participant_prompt_internal_delegate";

    engine.threads.write().await.insert(
        thread_id.to_string(),
        AgentThread {
            id: thread_id.to_string(),
            agent_name: Some(crate::agent::agent_identity::MAIN_AGENT_NAME.to_string()),
            title: "Prompt visibility".to_string(),
            messages: vec![AgentMessage::user("hello", 1)],
            pinned: false,
            upstream_thread_id: None,
            upstream_transport: None,
            upstream_provider: None,
            upstream_model: None,
            upstream_assistant_id: None,
            total_input_tokens: 0,
            total_output_tokens: 0,
            created_at: 1,
            updated_at: 1,
        },
    );
    engine
        .upsert_thread_participant(thread_id, "weles", "verify claims")
        .await
        .expect("participant should register");
    engine
        .append_internal_delegate_message(thread_id, "secret")
        .await
        .expect("append internal delegate message");

    let prompt = engine
        .build_participant_prompt(thread_id, "weles")
        .await
        .expect("prompt should build");
    assert!(!prompt.contains("secret"));
}