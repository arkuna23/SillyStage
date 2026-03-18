use serde_json::json;
use ss_protocol::{
    CharacterCardSummaryPayload, CharacterCreatedPayload, DashboardCountsPayload,
    DashboardHealthPayload, DashboardHealthStatus, DashboardPayload,
    DashboardSessionSummaryPayload, DashboardStorySummaryPayload, ErrorCode, ErrorPayload,
    GenerateStoryPlanParams, GlobalConfigPayload, JsonRpcOutcome, JsonRpcRequestMessage,
    JsonRpcResponseMessage, RequestParams, ResourceFilePayload, ResponseResult,
    RuntimeSnapshotPayload, ServerEventMessage, SessionDetailPayload, SessionMessageKind,
    SessionMessagePayload, StoryPlannedPayload, StreamEventBody, StreamFrame,
};
use state::WorldState;

fn sample_runtime_snapshot() -> engine::RuntimeSnapshot {
    let mut world_state = WorldState::new("dock");
    world_state.set_player_state("coins", json!(7));

    engine::RuntimeSnapshot {
        story_id: "demo_story".to_owned(),
        player_description: "A stubborn courier carrying medicine.".to_owned(),
        world_state,
        turn_index: 2,
    }
}

#[test]
fn json_rpc_request_and_response_round_trip() {
    let request = JsonRpcRequestMessage::new(
        "req-1",
        None::<String>,
        RequestParams::StoryGeneratePlan(GenerateStoryPlanParams {
            resource_id: "res-1".to_owned(),
            api_group_id: Some("group-default".to_owned()),
            preset_id: Some("preset-default".to_owned()),
        }),
    );

    let request_json = serde_json::to_string_pretty(&request).expect("request should serialize");
    assert!(request_json.contains("\"jsonrpc\": \"2.0\""));
    assert!(request_json.contains("\"method\": \"story.generate_plan\""));
    assert!(!request_json.contains("\"type\""));

    let request_round_trip: JsonRpcRequestMessage =
        serde_json::from_str(&request_json).expect("request should deserialize");
    assert!(matches!(
        request_round_trip.params,
        RequestParams::StoryGeneratePlan(GenerateStoryPlanParams {
            resource_id,
            api_group_id,
            preset_id,
        })
            if resource_id == "res-1"
                && api_group_id.as_deref() == Some("group-default")
                && preset_id.as_deref() == Some("preset-default")
    ));

    let response = JsonRpcResponseMessage::ok(
        "req-1",
        None::<String>,
        ResponseResult::StoryPlanned(StoryPlannedPayload {
            resource_id: "res-1".to_owned(),
            story_script: "Title:\nFlooded Harbor".to_owned(),
        }),
    );
    let response_json = serde_json::to_string_pretty(&response).expect("response should serialize");
    let response_round_trip: JsonRpcResponseMessage =
        serde_json::from_str(&response_json).expect("response should deserialize");

    assert!(matches!(
        response_round_trip.outcome,
        JsonRpcOutcome::Ok(result) if matches!(*result, ResponseResult::StoryPlanned(_))
    ));

    let error_message = JsonRpcResponseMessage::err(
        "req-2",
        Some("session-1"),
        ErrorPayload::new(ErrorCode::InvalidParams, "missing player input")
            .with_data(json!({ "field": "player_input" })),
    );
    let error_json =
        serde_json::to_string_pretty(&error_message).expect("error response should serialize");
    let error_round_trip: JsonRpcResponseMessage =
        serde_json::from_str(&error_json).expect("error response should deserialize");

    assert!(matches!(
        error_round_trip.outcome,
        JsonRpcOutcome::Err(ErrorPayload { code: -32602, .. })
    ));

    let config_response = JsonRpcResponseMessage::ok(
        "req-3",
        None::<String>,
        ResponseResult::GlobalConfig(GlobalConfigPayload {
            api_group_id: Some("group-default".to_owned()),
            preset_id: Some("preset-default".to_owned()),
        }),
    );
    let config_json =
        serde_json::to_string_pretty(&config_response).expect("config response should serialize");
    let config_round_trip: JsonRpcResponseMessage =
        serde_json::from_str(&config_json).expect("config response should deserialize");
    assert!(matches!(
        config_round_trip.outcome,
        JsonRpcOutcome::Ok(result) if matches!(*result, ResponseResult::GlobalConfig(_))
    ));

    let created_response = JsonRpcResponseMessage::ok(
        "req-6",
        None::<String>,
        ResponseResult::CharacterCreated(CharacterCreatedPayload {
            character_id: "merchant".to_owned(),
            character_summary: CharacterCardSummaryPayload {
                character_id: "merchant".to_owned(),
                name: "Haru".to_owned(),
                personality: "greedy but friendly trader".to_owned(),
                style: "talkative, casual".to_owned(),
                tags: vec!["merchant".to_owned()],
                folder: "harbor".to_owned(),
                cover_file_name: None,
                cover_mime_type: None,
            },
        }),
    );
    let created_json =
        serde_json::to_string_pretty(&created_response).expect("created response should serialize");
    let created_round_trip: JsonRpcResponseMessage =
        serde_json::from_str(&created_json).expect("created response should deserialize");
    assert!(matches!(
        created_round_trip.outcome,
        JsonRpcOutcome::Ok(result) if matches!(*result, ResponseResult::CharacterCreated(_))
    ));

    let dashboard_response = JsonRpcResponseMessage::ok(
        "req-8",
        None::<String>,
        ResponseResult::Dashboard(Box::new(DashboardPayload {
            health: DashboardHealthPayload {
                status: DashboardHealthStatus::Ok,
            },
            counts: DashboardCountsPayload {
                characters_total: 3,
                characters_with_cover: 2,
                story_resources_total: 1,
                stories_total: 2,
                sessions_total: 4,
            },
            global_config: GlobalConfigPayload {
                api_group_id: Some("group-default".to_owned()),
                preset_id: Some("preset-default".to_owned()),
            },
            recent_stories: vec![DashboardStorySummaryPayload {
                story_id: "story-1".to_owned(),
                display_name: "Flooded Harbor".to_owned(),
                resource_id: "resource-1".to_owned(),
                introduction: "At the dock.".to_owned(),
                updated_at_ms: Some(1_000),
            }],
            recent_sessions: vec![DashboardSessionSummaryPayload {
                session_id: "session-1".to_owned(),
                story_id: "story-1".to_owned(),
                display_name: "Courier Run".to_owned(),
                turn_index: 2,
                updated_at_ms: Some(2_000),
            }],
        })),
    );
    let dashboard_json =
        serde_json::to_string_pretty(&dashboard_response).expect("dashboard should serialize");
    let dashboard_round_trip: JsonRpcResponseMessage =
        serde_json::from_str(&dashboard_json).expect("dashboard should deserialize");
    assert!(matches!(
        dashboard_round_trip.outcome,
        JsonRpcOutcome::Ok(result) if matches!(*result, ResponseResult::Dashboard(_))
    ));

    let session_response = JsonRpcResponseMessage::ok(
        "req-9",
        Some("session-1"),
        ResponseResult::Session(Box::new(SessionDetailPayload {
            session_id: "session-1".to_owned(),
            story_id: "story-1".to_owned(),
            display_name: "Courier Run".to_owned(),
            player_profile_id: Some("profile-courier".to_owned()),
            player_schema_id: "schema-player".to_owned(),
            api_group_id: "group-default".to_owned(),
            preset_id: "preset-default".to_owned(),
            snapshot: sample_runtime_snapshot(),
            history: vec![
                SessionMessagePayload {
                    message_id: "message-1".to_owned(),
                    kind: SessionMessageKind::PlayerInput,
                    sequence: 0,
                    turn_index: 1,
                    recorded_at_ms: 1_000,
                    created_at_ms: 1_000,
                    updated_at_ms: 1_000,
                    speaker_id: "player".to_owned(),
                    speaker_name: "Player".to_owned(),
                    text: "Open the gate.".to_owned(),
                },
                SessionMessagePayload {
                    message_id: "message-2".to_owned(),
                    kind: SessionMessageKind::Narration,
                    sequence: 1,
                    turn_index: 1,
                    recorded_at_ms: 1_001,
                    created_at_ms: 1_001,
                    updated_at_ms: 1_001,
                    speaker_id: "narrator".to_owned(),
                    speaker_name: "Narrator".to_owned(),
                    text: "Water churned beneath the dock.".to_owned(),
                },
            ],
            created_at_ms: Some(500),
            updated_at_ms: Some(1_001),
            config: ss_protocol::SessionConfigPayload {
                api_group_id: "group-default".to_owned(),
                preset_id: "preset-default".to_owned(),
            },
        })),
    );
    let session_json =
        serde_json::to_string_pretty(&session_response).expect("session should serialize");
    let session_round_trip: JsonRpcResponseMessage =
        serde_json::from_str(&session_json).expect("session should deserialize");
    assert!(matches!(
        session_round_trip.outcome,
        JsonRpcOutcome::Ok(result) if matches!(*result, ResponseResult::Session(_))
    ));
}

#[test]
fn resource_file_payload_round_trip() {
    let payload = ResourceFilePayload {
        resource_id: "character:merchant".to_owned(),
        file_id: "cover".to_owned(),
        file_name: Some("cover.png".to_owned()),
        content_type: "image/png".to_owned(),
        size_bytes: 42,
    };

    let json = serde_json::to_string_pretty(&payload).expect("payload should serialize");
    let round_trip: ResourceFilePayload =
        serde_json::from_str(&json).expect("payload should deserialize");

    assert_eq!(round_trip.resource_id, "character:merchant");
    assert_eq!(round_trip.file_id, "cover");
    assert_eq!(round_trip.file_name.as_deref(), Some("cover.png"));
    assert_eq!(round_trip.content_type, "image/png");
    assert_eq!(round_trip.size_bytes, 42);
}

#[test]
fn server_event_messages_encode_started_event_completed_and_failed_frames() {
    let started = ServerEventMessage::started("req-3", Some("session-9"), 0);
    let started_json =
        serde_json::to_string_pretty(&started).expect("started frame should serialize");
    let started_round_trip: ServerEventMessage =
        serde_json::from_str(&started_json).expect("started frame should deserialize");
    assert!(matches!(started_round_trip.frame, StreamFrame::Started));

    let event = ServerEventMessage::event(
        "req-3",
        Some("session-9"),
        1,
        StreamEventBody::TurnStarted {
            next_turn_index: 3,
            player_input: "Open the gate.".to_owned(),
        },
    );
    let event_json = serde_json::to_string_pretty(&event).expect("event frame should serialize");
    let event_round_trip: ServerEventMessage =
        serde_json::from_str(&event_json).expect("event frame should deserialize");
    assert!(matches!(
        event_round_trip.frame,
        StreamFrame::Event {
            body: StreamEventBody::TurnStarted { .. }
        }
    ));

    let completed = ServerEventMessage::completed(
        "req-3",
        Some("session-9"),
        9,
        ResponseResult::RuntimeSnapshot(Box::new(RuntimeSnapshotPayload {
            snapshot: sample_runtime_snapshot(),
        })),
    );
    let completed_json =
        serde_json::to_string_pretty(&completed).expect("completed frame should serialize");
    let completed_round_trip: ServerEventMessage =
        serde_json::from_str(&completed_json).expect("completed frame should deserialize");
    assert!(matches!(
        &completed_round_trip.frame,
        StreamFrame::Completed { response }
            if matches!(&**response, ResponseResult::RuntimeSnapshot(_))
    ));

    let failed = ServerEventMessage::failed(
        "req-3",
        Some("session-9"),
        10,
        ErrorPayload::new(ErrorCode::StreamError, "stream interrupted"),
    );
    let failed_json = serde_json::to_string_pretty(&failed).expect("failed frame should serialize");
    let failed_round_trip: ServerEventMessage =
        serde_json::from_str(&failed_json).expect("failed frame should deserialize");
    assert!(matches!(
        failed_round_trip.frame,
        StreamFrame::Failed { .. }
    ));
}
