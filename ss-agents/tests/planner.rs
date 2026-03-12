mod common;

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;
use ss_agents::actor::CharacterCard;
use ss_agents::planner::{Planner, PlannerRequest};
use state::schema::{StateFieldSchema, StateValueType};

use common::{MockLlm, assistant_response};

fn sample_state_schema() -> HashMap<String, StateFieldSchema> {
    HashMap::from([(
        "trust".to_owned(),
        StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
    )])
}

#[tokio::test]
async fn planner_returns_editable_story_script_and_character_summary() {
    let llm = Arc::new(MockLlm::with_chat_response(assistant_response(
        "Title:\nFlooded Dock Bargain\n\nOpening Situation:\nThe courier arrives at a flooded dock.\n\nCore Conflict:\nThe courier must decide whether to trust the merchant.\n\nCharacter Roles:\nHaru (merchant) wants profit.\n\nSuggested Beats:\nThe player questions the route.\n\nState Hints:\nTrust may rise or fall.",
        None,
    )));
    let planner = Planner::new(llm.clone(), "test-model").expect("planner should build");
    let available_characters = vec![CharacterCard {
        id: "merchant".to_owned(),
        name: "Haru".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative".to_owned(),
        tendencies: vec!["likes profitable deals".to_owned()],
        state_schema: sample_state_schema(),
        system_prompt: "Stay in character.".to_owned(),
    }];

    let response = planner
        .plan(PlannerRequest {
            story_concept: "A courier negotiates passage through a flooded dock.",
            available_characters: &available_characters,
        })
        .await
        .expect("planner should succeed");

    assert!(response.story_script.contains("Title:"));
    assert!(response.story_script.contains("Opening Situation:"));

    let requests = llm.recorded_requests();
    let request = requests.first().expect("request should be recorded");
    let user_message = request
        .messages
        .iter()
        .find(|message| matches!(message.role, llm::Role::User))
        .expect("user message should exist");

    assert!(user_message.content.contains("STORY_CONCEPT"));
    assert!(user_message.content.contains("\"id\": \"merchant\""));
    assert!(!user_message.content.contains("Stay in character."));
}
