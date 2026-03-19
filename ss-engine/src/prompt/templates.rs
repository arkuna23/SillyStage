use store::{PromptEntryKind, PromptModuleId};

use super::contracts::{
    ACTOR_OUTPUT_CONTRACT, ARCHITECT_OUTPUT_CORE, DIRECTOR_OUTPUT_CONTRACT, KEEPER_OUTPUT_CONTRACT,
    REPLYER_OUTPUT_CONTRACT,
};
use super::types::{BuiltInEntryTemplate, PromptAgentKind};

const PLANNER_TEMPLATES: [BuiltInEntryTemplate; 7] = [
    text_entry(
        PromptModuleId::Role,
        "planner_role_core",
        "Core Role",
        10,
        true,
        "You are Planner. Turn the story concept into a concise planning brief for Architect.",
    ),
    text_entry(
        PromptModuleId::Task,
        "planner_task_core",
        "Planning Task",
        10,
        true,
        "Work only from the provided concept, lorebook, and available characters. Keep the plan coherent and specific.",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "planner_story_concept",
        "Story Concept",
        10,
        true,
        "story_concept",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "planner_lorebook_base",
        "Lorebook Base",
        20,
        false,
        "lorebook_base",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "planner_available_characters",
        "Available Characters",
        30,
        true,
        "available_characters",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "planner_lorebook_matched",
        "Lorebook Matched",
        10,
        false,
        "lorebook_matched",
    ),
    text_entry(
        PromptModuleId::Output,
        "planner_output_core",
        "Output Contract",
        10,
        true,
        "Return plain text only. Summarize the story goal, core conflict, key characters, tone, and likely progression.",
    ),
];

const ARCHITECT_TEMPLATES: [BuiltInEntryTemplate; 15] = [
    text_entry(
        PromptModuleId::Role,
        "architect_role_core",
        "Core Role",
        10,
        true,
        "You are Architect. Build structured story graph data from the planning input.",
    ),
    text_entry(
        PromptModuleId::Task,
        "architect_task_core",
        "Architecture Task",
        10,
        true,
        "Create compact, valid story structure. Reuse the provided cast and keep transitions and schema fields consistent. Use only AVAILABLE_CHARACTERS ids in graph data; temporary runtime characters belong to Director, not the graph.",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "architect_story_concept",
        "Story Concept",
        10,
        true,
        "story_concept",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "architect_lorebook_base",
        "Lorebook Base",
        20,
        false,
        "lorebook_base",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "architect_planned_story",
        "Planned Story",
        30,
        false,
        "planned_story",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "architect_available_characters",
        "Available Characters",
        40,
        true,
        "available_characters",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "architect_world_state_schema_seed",
        "World State Schema Seed",
        50,
        false,
        "world_state_schema_seed",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "architect_player_state_schema_seed",
        "Player State Schema Seed",
        60,
        false,
        "player_state_schema_seed",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "architect_world_state_schema",
        "World State Schema",
        70,
        false,
        "world_state_schema",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "architect_player_state_schema",
        "Player State Schema",
        80,
        false,
        "player_state_schema",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "architect_section_summaries",
        "Section Summaries",
        90,
        false,
        "section_summaries",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "architect_current_section",
        "Current Section",
        10,
        false,
        "current_section",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "architect_section_index",
        "Section Index",
        20,
        false,
        "section_index",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "architect_total_sections",
        "Total Sections",
        30,
        false,
        "total_sections",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "architect_target_node_count",
        "Target Node Count",
        40,
        false,
        "target_node_count",
    ),
];

const ARCHITECT_TEMPLATES_TAIL: [BuiltInEntryTemplate; 3] = [
    context_entry(
        PromptModuleId::DynamicContext,
        "architect_graph_summary",
        "Graph Summary",
        50,
        false,
        "graph_summary",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "architect_recent_section_detail",
        "Recent Section Detail",
        60,
        false,
        "recent_section_detail",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "architect_lorebook_matched",
        "Lorebook Matched",
        70,
        false,
        "lorebook_matched",
    ),
];

const DIRECTOR_TEMPLATES: [BuiltInEntryTemplate; 10] = [
    text_entry(
        PromptModuleId::Role,
        "director_role_core",
        "Core Role",
        10,
        true,
        "You are Director. Decide what should happen this turn and which beats should execute.",
    ),
    text_entry(
        PromptModuleId::Task,
        "director_task_core",
        "Turn Task",
        10,
        true,
        "Plan a clear response for the current turn. Use only visible context and keep the response plan actionable. Actor beats may use only CURRENT_CAST ids or ids created in this same response via create_and_enter.",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "director_lorebook_base",
        "Lorebook Base",
        10,
        false,
        "lorebook_base",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "director_player",
        "Player",
        20,
        true,
        "player",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "director_current_cast",
        "Current Cast",
        30,
        true,
        "current_cast",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "director_current_node",
        "Current Node",
        40,
        true,
        "current_node",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "director_player_state_schema",
        "Player State Schema",
        50,
        false,
        "player_state_schema",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "director_lorebook_matched",
        "Lorebook Matched",
        10,
        false,
        "lorebook_matched",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "director_world_state",
        "World State",
        20,
        true,
        "world_state",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "director_shared_history",
        "Shared History",
        30,
        true,
        "shared_history",
    ),
];

const DIRECTOR_TEMPLATES_TAIL: [BuiltInEntryTemplate; 2] = [
    context_entry(
        PromptModuleId::DynamicContext,
        "director_player_input",
        "Player Input",
        40,
        true,
        "player_input",
    ),
    text_entry(
        PromptModuleId::Output,
        "director_output_core",
        "Output Contract",
        10,
        true,
        DIRECTOR_OUTPUT_CONTRACT,
    ),
];

const ACTOR_TEMPLATES: [BuiltInEntryTemplate; 10] = [
    text_entry(
        PromptModuleId::Role,
        "actor_role_core",
        "Core Role",
        10,
        true,
        "You are Actor. Perform as the assigned character while staying grounded in the current scene.",
    ),
    text_entry(
        PromptModuleId::Task,
        "actor_task_core",
        "Performance Task",
        10,
        true,
        "Respond from the character's perspective and pursue the supplied purpose without using hidden information that the character cannot access.",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "actor_lorebook_base",
        "Lorebook Base",
        10,
        false,
        "lorebook_base",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "actor_player",
        "Player",
        20,
        true,
        "player",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "actor_current_cast",
        "Current Cast",
        30,
        true,
        "current_cast",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "actor_current_node",
        "Current Node",
        40,
        true,
        "current_node",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "actor_lorebook_matched",
        "Lorebook Matched",
        10,
        false,
        "lorebook_matched",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "actor_world_state",
        "World State",
        20,
        true,
        "world_state",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "actor_shared_history",
        "Shared History",
        30,
        true,
        "shared_history",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "actor_private_memory",
        "Private Memory",
        40,
        false,
        "private_memory",
    ),
];

const ACTOR_TEMPLATES_TAIL: [BuiltInEntryTemplate; 2] = [
    context_entry(
        PromptModuleId::DynamicContext,
        "actor_purpose",
        "Actor Purpose",
        50,
        true,
        "actor_purpose",
    ),
    text_entry(
        PromptModuleId::Output,
        "actor_output_core",
        "Output Contract",
        10,
        true,
        ACTOR_OUTPUT_CONTRACT,
    ),
];

const NARRATOR_TEMPLATES: [BuiltInEntryTemplate; 11] = [
    text_entry(
        PromptModuleId::Role,
        "narrator_role_core",
        "Core Role",
        10,
        true,
        "You are Narrator. Describe scene changes, transitions, and outcomes that the audience can observe.",
    ),
    text_entry(
        PromptModuleId::Task,
        "narrator_task_core",
        "Narration Task",
        10,
        true,
        "Describe what changed in the scene and support the current beat purpose without using private memories.",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "narrator_lorebook_base",
        "Lorebook Base",
        10,
        false,
        "lorebook_base",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "narrator_player",
        "Player",
        20,
        true,
        "player",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "narrator_previous_node",
        "Previous Node",
        30,
        false,
        "previous_node",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "narrator_current_node",
        "Current Node",
        40,
        true,
        "current_node",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "narrator_previous_cast",
        "Previous Cast",
        50,
        false,
        "previous_cast",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "narrator_current_cast",
        "Current Cast",
        60,
        true,
        "current_cast",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "narrator_player_state_schema",
        "Player State Schema",
        70,
        false,
        "player_state_schema",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "narrator_purpose",
        "Narrator Purpose",
        80,
        true,
        "narrator_purpose",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "narrator_lorebook_matched",
        "Lorebook Matched",
        10,
        false,
        "lorebook_matched",
    ),
];

const NARRATOR_TEMPLATES_TAIL: [BuiltInEntryTemplate; 3] = [
    context_entry(
        PromptModuleId::DynamicContext,
        "narrator_world_state",
        "World State",
        20,
        true,
        "world_state",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "narrator_shared_history",
        "Shared History",
        30,
        true,
        "shared_history",
    ),
    text_entry(
        PromptModuleId::Output,
        "narrator_output_core",
        "Output Contract",
        10,
        true,
        "Return plain narration text only.",
    ),
];

const KEEPER_TEMPLATES: [BuiltInEntryTemplate; 14] = [
    text_entry(
        PromptModuleId::Role,
        "keeper_role_core",
        "Core Role",
        10,
        true,
        "You are Keeper. Update system state, progression, and node transitions after each turn phase.",
    ),
    text_entry(
        PromptModuleId::Task,
        "keeper_task_core",
        "State Task",
        10,
        true,
        "Reflect real progress in the state update. Use the current node, recent turn history, and completed beats to decide what changed. Do not introduce new character ids into active_characters; temporary characters must be created by Director.",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "keeper_lorebook_base",
        "Lorebook Base",
        10,
        false,
        "lorebook_base",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "keeper_player",
        "Player",
        20,
        true,
        "player",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "keeper_previous_node",
        "Previous Node",
        30,
        false,
        "previous_node",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "keeper_current_node",
        "Current Node",
        40,
        true,
        "current_node",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "keeper_previous_cast",
        "Previous Cast",
        50,
        false,
        "previous_cast",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "keeper_current_cast",
        "Current Cast",
        60,
        true,
        "current_cast",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "keeper_player_state_schema",
        "Player State Schema",
        70,
        false,
        "player_state_schema",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "keeper_phase",
        "Keeper Phase",
        80,
        true,
        "keeper_phase",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "keeper_lorebook_matched",
        "Lorebook Matched",
        10,
        false,
        "lorebook_matched",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "keeper_world_state",
        "World State",
        20,
        true,
        "world_state",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "keeper_shared_history",
        "Shared History",
        30,
        true,
        "shared_history",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "keeper_player_input",
        "Player Input",
        40,
        true,
        "player_input",
    ),
];

const KEEPER_TEMPLATES_TAIL: [BuiltInEntryTemplate; 4] = [
    context_entry(
        PromptModuleId::DynamicContext,
        "keeper_completed_beats",
        "Completed Beats",
        50,
        false,
        "completed_beats",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "keeper_node_change",
        "Node Change",
        90,
        false,
        "node_change",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "keeper_progression_hints",
        "Progression Hints",
        100,
        false,
        "progression_hints",
    ),
    text_entry(
        PromptModuleId::Output,
        "keeper_output_core",
        "Output Contract",
        10,
        true,
        KEEPER_OUTPUT_CONTRACT,
    ),
];

const REPLYER_TEMPLATES: [BuiltInEntryTemplate; 9] = [
    text_entry(
        PromptModuleId::Role,
        "replyer_role_core",
        "Core Role",
        10,
        true,
        "You are Replyer. Suggest several player reply options that fit the current state of the scene.",
    ),
    text_entry(
        PromptModuleId::Task,
        "replyer_task_core",
        "Reply Task",
        10,
        true,
        "Offer concise, distinct reply options grounded in the visible conversation history and current world state.\nLet the options vary naturally in tone, intent, and commitment level when they fit the scene.",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "replyer_lorebook_base",
        "Lorebook Base",
        10,
        false,
        "lorebook_base",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "replyer_player",
        "Player",
        20,
        true,
        "player",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "replyer_limit",
        "Reply Limit",
        30,
        true,
        "reply_limit",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "replyer_current_cast",
        "Current Cast",
        40,
        true,
        "current_cast",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "replyer_current_node",
        "Current Node",
        50,
        true,
        "current_node",
    ),
    context_entry(
        PromptModuleId::StaticContext,
        "replyer_player_state_schema",
        "Player State Schema",
        60,
        false,
        "player_state_schema",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "replyer_world_state",
        "World State",
        10,
        true,
        "world_state",
    ),
];

const REPLYER_TEMPLATES_TAIL: [BuiltInEntryTemplate; 3] = [
    context_entry(
        PromptModuleId::DynamicContext,
        "replyer_session_history",
        "Session History",
        20,
        true,
        "session_history",
    ),
    context_entry(
        PromptModuleId::DynamicContext,
        "replyer_lorebook_matched",
        "Lorebook Matched",
        30,
        false,
        "lorebook_matched",
    ),
    text_entry(
        PromptModuleId::Output,
        "replyer_output_core",
        "Output Contract",
        10,
        true,
        REPLYER_OUTPUT_CONTRACT,
    ),
];

const fn text_entry(
    module_id: PromptModuleId,
    entry_id: &'static str,
    display_name: &'static str,
    order: i32,
    required: bool,
    text: &'static str,
) -> BuiltInEntryTemplate {
    BuiltInEntryTemplate {
        module_id,
        entry_id,
        display_name,
        kind: PromptEntryKind::BuiltInText,
        required,
        order,
        text: Some(text),
        context_key: None,
    }
}

const fn context_entry(
    module_id: PromptModuleId,
    entry_id: &'static str,
    display_name: &'static str,
    order: i32,
    required: bool,
    context_key: &'static str,
) -> BuiltInEntryTemplate {
    BuiltInEntryTemplate {
        module_id,
        entry_id,
        display_name,
        kind: PromptEntryKind::BuiltInContextRef,
        required,
        order,
        text: None,
        context_key: Some(context_key),
    }
}

pub(super) fn templates_for_agent(agent: PromptAgentKind) -> Vec<BuiltInEntryTemplate> {
    let mut templates = match agent {
        PromptAgentKind::Planner => PLANNER_TEMPLATES.to_vec(),
        PromptAgentKind::Architect => {
            let mut items = ARCHITECT_TEMPLATES.to_vec();
            items.extend_from_slice(&ARCHITECT_TEMPLATES_TAIL);
            items.push(text_entry(
                PromptModuleId::Output,
                "architect_output_core",
                "Output Contract",
                10,
                true,
                ARCHITECT_OUTPUT_CORE,
            ));
            items
        }
        PromptAgentKind::Director => {
            let mut items = DIRECTOR_TEMPLATES.to_vec();
            items.extend_from_slice(&DIRECTOR_TEMPLATES_TAIL);
            items
        }
        PromptAgentKind::Actor => {
            let mut items = ACTOR_TEMPLATES.to_vec();
            items.extend_from_slice(&ACTOR_TEMPLATES_TAIL);
            items
        }
        PromptAgentKind::Narrator => {
            let mut items = NARRATOR_TEMPLATES.to_vec();
            items.extend_from_slice(&NARRATOR_TEMPLATES_TAIL);
            items
        }
        PromptAgentKind::Keeper => {
            let mut items = KEEPER_TEMPLATES.to_vec();
            items.extend_from_slice(&KEEPER_TEMPLATES_TAIL);
            items
        }
        PromptAgentKind::Replyer => {
            let mut items = REPLYER_TEMPLATES.to_vec();
            items.extend_from_slice(&REPLYER_TEMPLATES_TAIL);
            items
        }
    };
    templates.sort_by(|left, right| {
        module_sort_key(left.module_id)
            .cmp(&module_sort_key(right.module_id))
            .then_with(|| left.order.cmp(&right.order))
            .then_with(|| left.entry_id.cmp(right.entry_id))
    });
    templates
}

fn module_sort_key(module_id: PromptModuleId) -> i32 {
    match module_id {
        PromptModuleId::Role => 0,
        PromptModuleId::Task => 1,
        PromptModuleId::StaticContext => 2,
        PromptModuleId::DynamicContext => 3,
        PromptModuleId::Output => 4,
    }
}
