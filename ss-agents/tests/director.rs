mod common;

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;
use ss_agents::actor::CharacterCard;
use ss_agents::director::{ActorPurpose, Director, NarratorPurpose, ResponseBeat};
use state::{
    ActorMemoryEntry, ActorMemoryKind, PlayerStateSchema, StateFieldSchema, StateOp,
    StateValueType, WorldState,
};
use story::runtime_graph::RuntimeStoryGraph;
use story::{NarrativeNode, StoryGraph};

use common::{MockLlm, assistant_response, context_entry, prompt_profile};

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
    let director = Director::new(llm.clone(), "test-model")
        .expect("director should build")
        .with_prompt_profile(prompt_profile(
            "ROLE:\nDirector Mode\nPrefer compact beat plans.\n\nTASK:\nYou may output any number of beats.\nYou may interleave Narrator and Actor beats in any order.",
            vec![
                context_entry("player", "PLAYER", "player"),
                context_entry("current-cast", "CURRENT_CAST", "current_cast"),
                context_entry("current-node", "CURRENT_NODE", "current_node"),
                context_entry(
                    "player-state-schema",
                    "PLAYER_STATE_SCHEMA",
                    "player_state_schema",
                ),
                context_entry(
                    "transitioned",
                    "TRANSITIONED_THIS_TURN",
                    "transitioned_this_turn",
                ),
            ],
            vec![
                context_entry("world-state", "WORLD_STATE", "world_state"),
                context_entry("shared-history", "SHARED_HISTORY", "shared_history"),
            ],
        ));
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
            vec![StateOp::SetState {
                key: "entered_intro".to_owned(),
                value: json!(true),
            }],
        )],
    ))
    .expect("runtime graph should build");

    let mut world_state = WorldState::new("merchant_intro");
    world_state.set_state("flood_gate_open", json!(false));
    world_state.set_player_state("coins", json!(12));
    world_state.set_character_state("merchant", "trust", json!(2));
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
                personality: "greedy but friendly trader trust={{trust}}".to_owned(),
                style: "talkative".to_owned(),
                state_schema: HashMap::from([(
                    "trust".to_owned(),
                    StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
                )]),
                system_prompt: "Stay in character.".to_owned(),
            }],
            None,
            None,
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
    assert!(system_message.content.contains("Director Mode"));
    assert!(
        system_message
            .content
            .contains("Prefer compact beat plans.")
    );
    assert!(!system_message.content.contains("PRESET_PROMPT_ENTRIES"));
    assert!(user_message.contains("CURRENT_CAST"));
    assert!(user_message.contains("merchant | Old Merchant"));
    assert!(user_message.contains("trust=2"));
    assert!(!user_message.contains("ResponsePlan schema"));
    assert!(!user_message.contains("Stay in character."));
    assert!(user_message.contains("PLAYER_STATE_SCHEMA"));
    assert!(!user_message.contains("on_enter_updates"));
    assert!(!user_message.contains("entered_intro"));
    assert!(!user_message.contains("PLAYER_NAME"));
    assert!(user_message.contains("PLAYER:"));
    assert!(
        user_message.contains("A stubborn courier trying to protect a sealed medical satchel.")
    );
    assert!(user_message.contains("player_state"));
    assert!(user_message.contains("coins=12"));
    assert!(user_message.contains("SHARED_HISTORY"));
    assert!(user_message.contains("Keep this between us."));
    assert!(!user_message.contains("This should stay hidden from the director."));
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
