use std::collections::HashMap;

use agents::actor::CharacterCard;
use serde_json::json;
use ss_protocol::{
    ErrorCode, ErrorPayload, RequestBody, RequestMessage, ResponseBody, ResponseMessage,
    RuntimeSnapshotPayload, ServerMessage, StoryPlannedPayload, StreamEventBody, StreamFrame,
    StreamResponseMessage,
};
use state::{PlayerStateSchema, StateFieldSchema, StateValueType, WorldState};

fn sample_character_cards() -> Vec<CharacterCard> {
    vec![CharacterCard {
        id: "merchant".to_owned(),
        name: "Haru".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative".to_owned(),
        tendencies: vec!["likes profitable deals".to_owned()],
        state_schema: HashMap::new(),
        system_prompt: "Stay in character.".to_owned(),
    }]
}

fn sample_player_state_schema() -> PlayerStateSchema {
    let mut schema = PlayerStateSchema::new();
    schema.insert_field(
        "coins",
        StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
    );
    schema
}

fn sample_story_resources() -> engine::StoryResources {
    engine::StoryResources::new(
        "demo_story",
        "A flooded harbor story.",
        sample_character_cards(),
        sample_player_state_schema(),
    )
    .expect("story resources should build")
}

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
fn request_and_response_messages_round_trip() {
    let request = RequestMessage::new(
        "req-1",
        None::<String>,
        RequestBody::GenerateStoryPlan {
            resources: sample_story_resources(),
        },
    );
    let request_json = serde_json::to_string_pretty(&request).expect("request should serialize");
    let request_round_trip: RequestMessage =
        serde_json::from_str(&request_json).expect("request should deserialize");

    assert!(matches!(
        request_round_trip.body,
        RequestBody::GenerateStoryPlan { .. }
    ));

    let response = ResponseMessage::ok(
        "req-1",
        None::<String>,
        ResponseBody::StoryPlanned(StoryPlannedPayload {
            story_script: "Title:\nFlooded Harbor".to_owned(),
        }),
    );
    let response_json =
        serde_json::to_string_pretty(&response).expect("response should serialize");
    let response_round_trip: ResponseMessage =
        serde_json::from_str(&response_json).expect("response should deserialize");

    assert!(matches!(
        &response_round_trip.outcome,
        ss_protocol::ResponseOutcome::Ok { body }
            if matches!(&**body, ResponseBody::StoryPlanned(_))
    ));

    let error_message = ResponseMessage::err(
        "req-2",
        Some("session-1"),
        ErrorPayload::new(ErrorCode::InvalidRequest, "missing player input")
            .retryable(false)
            .with_details(json!({ "field": "player_input" })),
    );
    let error_json =
        serde_json::to_string_pretty(&error_message).expect("error response should serialize");
    let error_round_trip: ResponseMessage =
        serde_json::from_str(&error_json).expect("error response should deserialize");

    assert!(matches!(
        error_round_trip.outcome,
        ss_protocol::ResponseOutcome::Err { .. }
    ));

    let server_message = ServerMessage::Response { message: response };
    let server_json =
        serde_json::to_string_pretty(&server_message).expect("server message should serialize");
    let server_round_trip: ServerMessage =
        serde_json::from_str(&server_json).expect("server message should deserialize");

    assert!(matches!(server_round_trip, ServerMessage::Response { .. }));
}

#[test]
fn stream_messages_encode_started_event_completed_and_failed_frames() {
    let started = StreamResponseMessage::started("req-3", Some("session-9"), 0);
    let started_json =
        serde_json::to_string_pretty(&started).expect("started frame should serialize");
    let started_round_trip: StreamResponseMessage =
        serde_json::from_str(&started_json).expect("started frame should deserialize");
    assert!(matches!(started_round_trip.frame, StreamFrame::Started));

    let event = StreamResponseMessage::event(
        "req-3",
        Some("session-9"),
        1,
        StreamEventBody::TurnStarted {
            next_turn_index: 3,
            player_input: "Open the gate.".to_owned(),
        },
    );
    let event_json = serde_json::to_string_pretty(&event).expect("event frame should serialize");
    let event_round_trip: StreamResponseMessage =
        serde_json::from_str(&event_json).expect("event frame should deserialize");
    assert!(matches!(
        event_round_trip.frame,
        StreamFrame::Event {
            body: StreamEventBody::TurnStarted { .. }
        }
    ));

    let completed = StreamResponseMessage::completed(
        "req-3",
        Some("session-9"),
        9,
        ResponseBody::RuntimeSnapshot(RuntimeSnapshotPayload {
            snapshot: sample_runtime_snapshot(),
        }),
    );
    let completed_json =
        serde_json::to_string_pretty(&completed).expect("completed frame should serialize");
    let completed_round_trip: StreamResponseMessage =
        serde_json::from_str(&completed_json).expect("completed frame should deserialize");
    assert!(matches!(
        &completed_round_trip.frame,
        StreamFrame::Completed { response }
            if matches!(&**response, ResponseBody::RuntimeSnapshot(_))
    ));

    let failed = StreamResponseMessage::failed(
        "req-3",
        Some("session-9"),
        10,
        ErrorPayload::new(ErrorCode::StreamError, "stream interrupted"),
    );
    let failed_json =
        serde_json::to_string_pretty(&failed).expect("failed frame should serialize");
    let failed_round_trip: StreamResponseMessage =
        serde_json::from_str(&failed_json).expect("failed frame should deserialize");
    assert!(matches!(failed_round_trip.frame, StreamFrame::Failed { .. }));
}
