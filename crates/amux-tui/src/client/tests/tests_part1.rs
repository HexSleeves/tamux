    #[test]
    fn whatsapp_link_methods_send_expected_protocol_messages() {
        let (event_tx, _event_rx) = mpsc::channel(8);
        let client = DaemonClient::new(event_tx);
        let mut rx = client.request_rx.lock().unwrap().take().unwrap();

        client.whatsapp_link_start().unwrap();
        assert!(matches!(
            drain_request(&mut rx),
            ClientMessage::AgentWhatsAppLinkStart
        ));

        client.whatsapp_link_status().unwrap();
        assert!(matches!(
            drain_request(&mut rx),
            ClientMessage::AgentWhatsAppLinkStatus
        ));

        client.whatsapp_link_subscribe().unwrap();
        assert!(matches!(
            drain_request(&mut rx),
            ClientMessage::AgentWhatsAppLinkSubscribe
        ));

        client.whatsapp_link_unsubscribe().unwrap();
        assert!(matches!(
            drain_request(&mut rx),
            ClientMessage::AgentWhatsAppLinkUnsubscribe
        ));

        client.whatsapp_link_reset().unwrap();
        assert!(matches!(
            drain_request(&mut rx),
            ClientMessage::AgentWhatsAppLinkReset
        ));

        client.whatsapp_link_stop().unwrap();
        assert!(matches!(
            drain_request(&mut rx),
            ClientMessage::AgentWhatsAppLinkStop
        ));
    }

    #[tokio::test]
    async fn done_event_parses_reasoning_payload() {
        let (event_tx, mut event_rx) = mpsc::channel(8);

        DaemonClient::dispatch_agent_event(
            serde_json::json!({
                "type": "done",
                "thread_id": "thread-1",
                "input_tokens": 10,
                "output_tokens": 20,
                "provider": "github-copilot",
                "model": "gpt-5.4",
                "reasoning": "Final reasoning summary"
            }),
            &event_tx,
        )
        .await;

        match event_rx.recv().await.expect("expected done event") {
            ClientEvent::Done {
                thread_id,
                reasoning,
                ..
            } => {
                assert_eq!(thread_id, "thread-1");
                assert_eq!(reasoning.as_deref(), Some("Final reasoning summary"));
            }
            other => panic!("expected done event, got {:?}", other),
        }
    }
