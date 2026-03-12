use serde_json::json;
use ss_protocol::{
    ResponseResult, RuntimeSnapshotPayload, ServerEventMessage, StreamEventBody, StreamFrame,
};
use state::{ActorMemoryEntry, ActorMemoryKind, WorldState};

fn sample_snapshot() -> engine::RuntimeSnapshot {
    let mut world_state = WorldState::new("dock");
    world_state.set_player_state("coins", json!(5));

    engine::RuntimeSnapshot {
        story_id: "demo_story".to_owned(),
        player_description: "A quiet courier keeping their satchel close.".to_owned(),
        world_state,
        turn_index: 1,
    }
}

#[test]
fn stream_event_round_trip_preserves_fine_grained_frames() {
    let event = ServerEventMessage::event(
        "req-42",
        Some("session-42"),
        2,
        StreamEventBody::PlayerInputRecorded {
            entry: ActorMemoryEntry {
                speaker_id: "player".to_owned(),
                speaker_name: "Player".to_owned(),
                kind: ActorMemoryKind::PlayerInput,
                text: "Open the gate.".to_owned(),
            },
            snapshot: Box::new(sample_snapshot()),
        },
    );

    let json = serde_json::to_string_pretty(&event).expect("event should serialize");
    let round_trip: ServerEventMessage =
        serde_json::from_str(&json).expect("event should deserialize");

    let StreamFrame::Event {
        body: StreamEventBody::PlayerInputRecorded { entry, snapshot },
    } = round_trip.frame
    else {
        panic!("expected player_input_recorded event");
    };

    assert_eq!(entry.text, "Open the gate.");
    assert_eq!(
        snapshot.player_description,
        "A quiet courier keeping their satchel close."
    );
}

#[test]
fn stream_event_supports_actor_and_narrator_deltas_and_terminal_payload() {
    let narrator_delta = ServerEventMessage::event(
        "req-77",
        Some("session-77"),
        3,
        StreamEventBody::NarratorTextDelta {
            beat_index: 0,
            purpose: agents::director::NarratorPurpose::DescribeScene,
            delta: "Cold water slapped against the pilings.".to_owned(),
        },
    );
    let narrator_json =
        serde_json::to_string_pretty(&narrator_delta).expect("delta should serialize");
    assert!(narrator_json.contains("\"message_type\": \"stream\""));
    assert!(narrator_json.contains("\"narrator_text_delta\""));

    let actor_completed = ServerEventMessage::event(
        "req-77",
        Some("session-77"),
        4,
        StreamEventBody::ActorCompleted {
            beat_index: 1,
            speaker_id: "merchant".to_owned(),
            purpose: agents::director::ActorPurpose::AdvanceGoal,
            response: Box::new(agents::actor::ActorResponse {
                speaker_id: "merchant".to_owned(),
                speaker_name: "Haru".to_owned(),
                segments: vec![agents::actor::ActorSegment {
                    kind: agents::actor::ActorSegmentKind::Dialogue,
                    text: "Follow me.".to_owned(),
                }],
                raw_output: String::new(),
            }),
        },
    );
    let actor_json =
        serde_json::to_string_pretty(&actor_completed).expect("actor event should serialize");
    assert!(actor_json.contains("\"actor_completed\""));

    let completed = ServerEventMessage::completed(
        "req-77",
        Some("session-77"),
        5,
        ResponseResult::RuntimeSnapshot(Box::new(RuntimeSnapshotPayload {
            snapshot: sample_snapshot(),
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
}
