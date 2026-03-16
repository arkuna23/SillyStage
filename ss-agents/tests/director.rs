mod common;

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;
use ss_agents::actor::CharacterCard;
use ss_agents::director::{ActorPurpose, Director, NarratorPurpose, ResponseBeat};
use state::{
    ActorMemoryEntry, ActorMemoryKind, PlayerStateSchema, StateFieldSchema, StateValueType,
    WorldState,
};
use story::runtime_graph::RuntimeStoryGraph;
use story::{NarrativeNode, StoryGraph};

use common::{MockLlm, assistant_response};

fn joined_user_messages(request: &llm::ChatRequest) -> String {
    request
        .messages
        .iter()
        .filter(|message| matches!(message.role, llm::Role::User))
        .map(|message| message.content.as_str())
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[tokio::test]
async fn director_prompt_uses_current_cast_summary_and_speaker_ids() {
    let llm = Arc::new(MockLlm::with_chat_response(assistant_response(
        "{\"beats\":[{\"type\":\"Narrator\",\"purpose\":\"DescribeScene\"},{\"type\":\"Actor\",\"speaker_id\":\"merchant\",\"purpose\":\"AdvanceGoal\"},{\"type\":\"Narrator\",\"purpose\":\"DescribeResult\"},{\"type\":\"Actor\",\"speaker_id\":\"merchant\",\"purpose\":\"CommentOnScene\"}]}",
        Some(json!({
            "beats": [
                {
                    "type": "Narrator",
                    "purpose": "DescribeScene"
                },
                {
                    "type": "Actor",
                    "speaker_id": "merchant",
                    "purpose": "AdvanceGoal"
                },
                {
                    "type": "Narrator",
                    "purpose": "DescribeResult"
                },
                {
                    "type": "Actor",
                    "speaker_id": "merchant",
                    "purpose": "CommentOnScene"
                }
            ]
        })),
    )));
    let director = Director::new(llm.clone(), "test-model").expect("director should build");
    let mut player_state_schema = PlayerStateSchema::new();
    player_state_schema.insert_field(
        "coins",
        StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
    );

    let runtime_graph = RuntimeStoryGraph::from_story_graph(StoryGraph::new(
        "merchant_intro",
        vec![NarrativeNode::new(
            "merchant_intro",
            "Flooded Dock",
            "A flooded dock at dusk.",
            "Decide whether to trust the guide.",
            vec!["merchant".to_owned()],
            vec![],
            vec![],
        )],
    ))
    .expect("runtime graph should build");

    let mut world_state = WorldState::new("merchant_intro");
    world_state.set_state("flood_gate_open", json!(false));
    world_state.set_player_state("coins", json!(12));
    world_state.add_active_character("merchant");
    world_state.push_actor_shared_history(
        ActorMemoryEntry {
            speaker_id: "merchant".to_owned(),
            speaker_name: "Old Merchant".to_owned(),
            kind: ActorMemoryKind::Dialogue,
            text: "Keep this between us.".to_owned(),
        },
        8,
    );
    world_state.push_actor_private_memory(
        "merchant",
        ActorMemoryEntry {
            speaker_id: "merchant".to_owned(),
            speaker_name: "Old Merchant".to_owned(),
            kind: ActorMemoryKind::Thought,
            text: "This should stay hidden from the director.".to_owned(),
        },
        8,
    );

    let result = director
        .decide_strict(
            &runtime_graph,
            &mut world_state,
            &[CharacterCard {
                id: "merchant".to_owned(),
                name: "Old Merchant".to_owned(),
                personality: "greedy but friendly trader".to_owned(),
                style: "talkative".to_owned(),
                state_schema: HashMap::new(),
                system_prompt: "Stay in character.".to_owned(),
            }],
            Some("Courier"),
            "A stubborn courier trying to protect a sealed medical satchel.",
            &player_state_schema,
        )
        .await
        .expect("director should succeed");

    let requests = llm.recorded_requests();
    let request = requests.first().expect("request should be recorded");
    let system_message = request
        .messages
        .iter()
        .find(|message| matches!(message.role, llm::Role::System))
        .expect("system message should exist");
    let user_message = joined_user_messages(request);

    assert!(
        system_message
            .content
            .contains("You may output any number of beats")
    );
    assert!(
        system_message
            .content
            .contains("You may interleave Narrator and Actor beats in any order")
    );
    assert!(user_message.contains("CURRENT_CAST"));
    assert!(user_message.contains("merchant | Old Merchant"));
    assert!(!user_message.contains("ResponsePlan schema"));
    assert!(!user_message.contains("Stay in character."));
    assert!(!user_message.contains("Keep this between us."));
    assert!(user_message.contains("PLAYER_STATE_SCHEMA"));
    assert!(!user_message.contains("PLAYER_NAME"));
    assert!(user_message.contains("PLAYER_DESCRIPTION"));
    assert!(
        user_message.contains("A stubborn courier trying to protect a sealed medical satchel.")
    );
    assert!(user_message.contains("player_state"));
    assert!(user_message.contains("coins=12"));
    assert!(!user_message.contains("This should stay hidden from the director."));
    assert!(!user_message.contains("actor_shared_history"));
    assert!(!user_message.contains("actor_private_memory"));
    assert_eq!(result.response_plan.beats.len(), 4);
    assert!(matches!(
        result.response_plan.beats[0],
        ResponseBeat::Narrator {
            purpose: NarratorPurpose::DescribeScene
        }
    ));
    assert!(matches!(
        &result.response_plan.beats[1],
        ResponseBeat::Actor {
            speaker_id,
            purpose: ActorPurpose::AdvanceGoal
        } if speaker_id == "merchant"
    ));
    assert!(matches!(
        result.response_plan.beats[2],
        ResponseBeat::Narrator {
            purpose: NarratorPurpose::DescribeResult
        }
    ));
    assert!(matches!(
        &result.response_plan.beats[3],
        ResponseBeat::Actor {
            speaker_id,
            purpose: ActorPurpose::CommentOnScene
        } if speaker_id == "merchant"
    ));
}
