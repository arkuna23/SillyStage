use engine::AgentApiIds;
use serde_json::json;
use ss_protocol::{
    CharacterCardSummaryPayload, CharacterChrExportPayload, CharacterCoverMimeType,
    CharacterCoverPayload, CharacterCoverUpdatedPayload, CharacterCreatedPayload, ErrorCode,
    ErrorPayload, GenerateStoryPlanParams, GlobalConfigPayload, JsonRpcOutcome,
    JsonRpcRequestMessage, JsonRpcResponseMessage, RequestParams, ResponseResult,
    RuntimeSnapshotPayload, ServerEventMessage, StoryPlannedPayload, StreamEventBody, StreamFrame,
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

fn sample_api_ids() -> AgentApiIds {
    AgentApiIds {
        planner_api_id: "planner-default".to_owned(),
        architect_api_id: "architect-default".to_owned(),
        director_api_id: "director-default".to_owned(),
        actor_api_id: "actor-default".to_owned(),
        narrator_api_id: "narrator-default".to_owned(),
        keeper_api_id: "keeper-default".to_owned(),
    }
}

#[test]
fn json_rpc_request_and_response_round_trip() {
    let request = JsonRpcRequestMessage::new(
        "req-1",
        None::<String>,
        RequestParams::StoryGeneratePlan(GenerateStoryPlanParams {
            resource_id: "res-1".to_owned(),
            planner_api_id: Some("planner-fast".to_owned()),
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
        RequestParams::StoryGeneratePlan(GenerateStoryPlanParams { resource_id, planner_api_id })
            if resource_id == "res-1" && planner_api_id.as_deref() == Some("planner-fast")
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
            api_ids: sample_api_ids(),
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

    let cover_response = JsonRpcResponseMessage::ok(
        "req-4",
        None::<String>,
        ResponseResult::CharacterCover(Box::new(CharacterCoverPayload {
            character_id: "merchant".to_owned(),
            cover_file_name: "cover.png".to_owned(),
            cover_mime_type: CharacterCoverMimeType::Png,
            cover_base64: "ZmFrZS1jb3Zlcg==".to_owned(),
        })),
    );
    let cover_json =
        serde_json::to_string_pretty(&cover_response).expect("cover response should serialize");
    let cover_round_trip: JsonRpcResponseMessage =
        serde_json::from_str(&cover_json).expect("cover response should deserialize");
    assert!(matches!(
        cover_round_trip.outcome,
        JsonRpcOutcome::Ok(result) if matches!(*result, ResponseResult::CharacterCover(_))
    ));

    let chr_export_response = JsonRpcResponseMessage::ok(
        "req-5",
        None::<String>,
        ResponseResult::CharacterChrExport(Box::new(CharacterChrExportPayload {
            character_id: "merchant".to_owned(),
            file_name: "merchant.chr".to_owned(),
            content_type: "application/x-sillystage-character-card".to_owned(),
            chr_base64: "UEsDBAoAAAAAA".to_owned(),
        })),
    );
    let chr_export_json = serde_json::to_string_pretty(&chr_export_response)
        .expect("chr export response should serialize");
    let chr_export_round_trip: JsonRpcResponseMessage =
        serde_json::from_str(&chr_export_json).expect("chr export response should deserialize");
    assert!(matches!(
        chr_export_round_trip.outcome,
        JsonRpcOutcome::Ok(result) if matches!(*result, ResponseResult::CharacterChrExport(_))
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
                tendencies: vec!["likes profitable deals".to_owned()],
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

    let cover_updated_response = JsonRpcResponseMessage::ok(
        "req-7",
        None::<String>,
        ResponseResult::CharacterCoverUpdated(CharacterCoverUpdatedPayload {
            character_id: "merchant".to_owned(),
            cover_file_name: "cover.png".to_owned(),
            cover_mime_type: CharacterCoverMimeType::Png,
        }),
    );
    let cover_updated_json = serde_json::to_string_pretty(&cover_updated_response)
        .expect("cover updated response should serialize");
    let cover_updated_round_trip: JsonRpcResponseMessage = serde_json::from_str(&cover_updated_json)
        .expect("cover updated response should deserialize");
    assert!(matches!(
        cover_updated_round_trip.outcome,
        JsonRpcOutcome::Ok(result) if matches!(*result, ResponseResult::CharacterCoverUpdated(_))
    ));
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
