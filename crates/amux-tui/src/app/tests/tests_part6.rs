    #[test]
    fn submit_operator_profile_answer_allows_empty_input_when_question_is_optional() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.operator_profile.visible = true;
        model.operator_profile.question = Some(OperatorProfileQuestionVm {
            session_id: "sess-1".to_string(),
            question_id: "nickname".to_string(),
            field_key: "nickname".to_string(),
            prompt: "Nickname?".to_string(),
            input_kind: "text".to_string(),
            optional: true,
        });

        assert!(model.submit_operator_profile_answer());
        assert!(
            model.operator_profile.loading,
            "optional empty answer should begin submission"
        );
        assert!(
            model.operator_profile.question.is_none(),
            "question should clear when submission starts"
        );

        let sent = cmd_rx
            .try_recv()
            .expect("submitting optional empty answer should emit daemon command");
        match sent {
            DaemonCommand::SubmitOperatorProfileAnswer {
                session_id,
                question_id,
                answer_json,
            } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(question_id, "nickname");
                assert_eq!(answer_json, "null");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn submit_operator_profile_answer_blocks_empty_input_when_question_is_required() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.operator_profile.visible = true;
        model.operator_profile.question = Some(OperatorProfileQuestionVm {
            session_id: "sess-1".to_string(),
            question_id: "name".to_string(),
            field_key: "name".to_string(),
            prompt: "What should I call you?".to_string(),
            input_kind: "text".to_string(),
            optional: false,
        });

        assert!(model.submit_operator_profile_answer());
        assert!(
            !model.operator_profile.loading,
            "required empty answer should not start submission"
        );
        assert!(
            model.operator_profile.question.is_some(),
            "question should remain while awaiting required answer"
        );
        assert!(
            cmd_rx.try_recv().is_err(),
            "required empty answer should not emit daemon command"
        );
    }

    #[test]
    fn skip_operator_profile_question_clears_stale_question_immediately() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.operator_profile.visible = true;
        model.operator_profile.question = Some(OperatorProfileQuestionVm {
            session_id: "sess-1".to_string(),
            question_id: "name".to_string(),
            field_key: "name".to_string(),
            prompt: "What should I call you?".to_string(),
            input_kind: "text".to_string(),
            optional: false,
        });

        assert!(model.skip_operator_profile_question());
        assert!(model.operator_profile.loading);
        assert!(
            model.operator_profile.question.is_none(),
            "question should clear when skip starts"
        );

        let sent = cmd_rx.try_recv().expect("skip should emit daemon command");
        match sent {
            DaemonCommand::SkipOperatorProfileQuestion {
                session_id,
                question_id,
                reason,
            } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(question_id, "name");
                assert_eq!(reason.as_deref(), Some("tui_skip_shortcut"));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn defer_operator_profile_question_clears_stale_question_immediately() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.operator_profile.visible = true;
        model.operator_profile.question = Some(OperatorProfileQuestionVm {
            session_id: "sess-1".to_string(),
            question_id: "name".to_string(),
            field_key: "name".to_string(),
            prompt: "What should I call you?".to_string(),
            input_kind: "text".to_string(),
            optional: false,
        });

        assert!(model.defer_operator_profile_question());
        assert!(model.operator_profile.loading);
        assert!(
            model.operator_profile.question.is_none(),
            "question should clear when defer starts"
        );

        let sent = cmd_rx.try_recv().expect("defer should emit daemon command");
        match sent {
            DaemonCommand::DeferOperatorProfileQuestion {
                session_id,
                question_id,
                defer_until_unix_ms,
            } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(question_id, "name");
                assert!(defer_until_unix_ms.is_some());
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }
