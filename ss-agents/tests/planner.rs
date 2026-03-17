mod common;

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;
use ss_agents::SystemPromptEntry;
use ss_agents::actor::CharacterCard;
use ss_agents::planner::{Planner, PlannerRequest};
use state::schema::{StateFieldSchema, StateValueType};

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
    let planner = Planner::new(llm.clone(), "test-model")
        .expect("planner should build")
        .with_system_prompt_entries(&[SystemPromptEntry {
            entry_id: "planner-tone".to_owned(),
            title: "Planner Tone".to_owned(),
            content: "Favor concise story plans.".to_owned(),
        }]);
    let available_characters = vec![CharacterCard {
        id: "merchant".to_owned(),
        name: "Haru".to_owned(),
        personality: "trust={{trust}}".to_owned(),
        style: "talkative".to_owned(),
        state_schema: sample_state_schema(),
        system_prompt: "Stay in character.".to_owned(),
    }];

    let response = planner
        .plan(PlannerRequest {
            story_concept: "A courier negotiates passage through a flooded dock.",
            available_characters: &available_characters,
            lorebook_base: None,
            lorebook_matched: None,
        })
        .await
        .expect("planner should succeed");

    assert!(response.story_script.contains("Title:"));
    assert!(response.story_script.contains("Opening Situation:"));

    let requests = llm.recorded_requests();
    let request = requests.first().expect("request should be recorded");
    let system_message = request
        .messages
        .iter()
        .find(|message| matches!(message.role, llm::Role::System))
        .expect("system message should exist");
    let user_message = joined_user_messages(request);

    assert!(system_message.content.contains("PRESET_PROMPT_ENTRIES"));
    assert!(
        system_message
            .content
            .contains("[planner-tone] Planner Tone")
    );
    assert!(
        system_message
            .content
            .contains("Favor concise story plans.")
    );
    assert!(user_message.contains("STORY_CONCEPT"));
    assert!(user_message.contains("merchant | Haru"));
    assert!(user_message.contains("trust=0"));
    assert!(!user_message.contains("Stay in character."));
}
