use agents::actor::{ActorResponse, ActorSegmentKind};
use agents::architect::{
    ArchitectDraftChunkResponse, ArchitectDraftInitResponse, ArchitectResponse,
};
use agents::director::{DirectorResult, ResponseBeat};
use agents::keeper::{KeeperPhase, KeeperResponse};
use agents::narrator::NarratorResponse;
use agents::planner::PlannerResponse;
use agents::replyer::ReplyOption;
use serde::Serialize;
use state::{StateOp, StateUpdate};
use story::StoryGraph;

const MAX_PREVIEW_CHARS: usize = 160;

pub(crate) fn json_for_log<T: Serialize>(payload: &T) -> String {
    serde_json::to_string(payload)
        .unwrap_or_else(|error| format!("{{\"serialization_error\":\"{error}\"}}"))
}

pub(crate) fn summarize_planner_response(response: &PlannerResponse) -> PlannerLogSummary {
    PlannerLogSummary {
        story_script_chars: response.story_script.chars().count(),
        story_script_lines: response.story_script.lines().count(),
        preview: truncate_text(&response.story_script),
    }
}

pub(crate) fn summarize_architect_response(response: &ArchitectResponse) -> ArchitectLogSummary {
    ArchitectLogSummary {
        node_count: response.graph.nodes.len(),
        transition_count: count_graph_transitions(&response.graph),
        world_schema_fields: response.world_state_schema.fields.len(),
        player_schema_fields: response.player_state_schema.fields.len(),
        introduction_preview: truncate_text(&response.introduction),
    }
}

pub(crate) fn summarize_architect_draft_init(
    response: &ArchitectDraftInitResponse,
    section_index: usize,
    total_sections: usize,
) -> ArchitectDraftLogSummary {
    ArchitectDraftLogSummary {
        section_index,
        total_sections,
        node_count: response.nodes.len(),
        transition_patch_count: response.transition_patches.len(),
        world_schema_fields: response.world_state_schema.fields.len(),
        player_schema_fields: response.player_state_schema.fields.len(),
        section_summary_preview: truncate_text(&response.section_summary),
        introduction_preview: truncate_text(&response.introduction),
    }
}

pub(crate) fn summarize_architect_draft_chunk(
    response: &ArchitectDraftChunkResponse,
    section_index: usize,
    total_sections: usize,
) -> ArchitectDraftLogSummary {
    ArchitectDraftLogSummary {
        section_index,
        total_sections,
        node_count: response.nodes.len(),
        transition_patch_count: response.transition_patches.len(),
        world_schema_fields: 0,
        player_schema_fields: 0,
        section_summary_preview: truncate_text(&response.section_summary),
        introduction_preview: String::new(),
    }
}

pub(crate) fn summarize_director_result(result: &DirectorResult) -> DirectorLogSummary {
    DirectorLogSummary {
        previous_node_id: result.previous_node_id.clone(),
        current_node_id: result.current_node_id.clone(),
        transitioned: result.transitioned,
        beat_count: result.response_plan.beats.len(),
        beat_types: result
            .response_plan
            .beats
            .iter()
            .map(|beat| match beat {
                ResponseBeat::Narrator { .. } => "narrator",
                ResponseBeat::Actor { .. } => "actor",
            })
            .collect(),
    }
}

pub(crate) fn summarize_narrator_response(response: &NarratorResponse) -> NarratorLogSummary {
    NarratorLogSummary {
        text_chars: response.text.chars().count(),
        preview: truncate_text(&response.text),
    }
}

pub(crate) fn summarize_actor_response(response: &ActorResponse) -> ActorLogSummary {
    let mut thought_chars = 0;
    let mut action_chars = 0;
    let mut dialogue_chars = 0;

    for segment in &response.segments {
        match segment.kind {
            ActorSegmentKind::Thought => thought_chars += segment.text.chars().count(),
            ActorSegmentKind::Action => action_chars += segment.text.chars().count(),
            ActorSegmentKind::Dialogue => dialogue_chars += segment.text.chars().count(),
        }
    }

    ActorLogSummary {
        speaker_id: response.speaker_id.clone(),
        speaker_name: response.speaker_name.clone(),
        segment_count: response.segments.len(),
        thought_chars,
        action_chars,
        dialogue_chars,
    }
}

pub(crate) fn summarize_keeper_response(
    phase: KeeperPhase,
    response: &KeeperResponse,
) -> KeeperLogSummary {
    KeeperLogSummary {
        phase,
        op_count: response.update.ops.len(),
        op_types: keeper_op_types(&response.update),
        touched_global_keys: touched_global_keys(&response.update),
        touched_player_keys: touched_player_keys(&response.update),
        touched_character_pairs: touched_character_pairs(&response.update),
    }
}

pub(crate) fn summarize_reply_options(replies: &[ReplyOption]) -> ReplyerLogSummary {
    ReplyerLogSummary {
        reply_count: replies.len(),
        reply_ids: replies.iter().map(|reply| reply.id.clone()).collect(),
        previews: replies
            .iter()
            .map(|reply| truncate_text(&reply.text))
            .collect(),
    }
}

fn count_graph_transitions(graph: &StoryGraph) -> usize {
    graph.nodes.iter().map(|node| node.transitions.len()).sum()
}

fn keeper_op_types(update: &StateUpdate) -> Vec<&'static str> {
    update
        .ops
        .iter()
        .map(|op| match op {
            StateOp::SetCurrentNode { .. } => "set_current_node",
            StateOp::SetActiveCharacters { .. } => "set_active_characters",
            StateOp::AddActiveCharacter { .. } => "add_active_character",
            StateOp::RemoveActiveCharacter { .. } => "remove_active_character",
            StateOp::SetState { .. } => "set_state",
            StateOp::RemoveState { .. } => "remove_state",
            StateOp::SetPlayerState { .. } => "set_player_state",
            StateOp::RemovePlayerState { .. } => "remove_player_state",
            StateOp::SetCharacterState { .. } => "set_character_state",
            StateOp::RemoveCharacterState { .. } => "remove_character_state",
        })
        .collect()
}

fn touched_global_keys(update: &StateUpdate) -> Vec<String> {
    update
        .ops
        .iter()
        .filter_map(|op| match op {
            StateOp::SetState { key, .. } | StateOp::RemoveState { key } => Some(key.clone()),
            _ => None,
        })
        .collect()
}

fn touched_player_keys(update: &StateUpdate) -> Vec<String> {
    update
        .ops
        .iter()
        .filter_map(|op| match op {
            StateOp::SetPlayerState { key, .. } | StateOp::RemovePlayerState { key } => {
                Some(key.clone())
            }
            _ => None,
        })
        .collect()
}

fn touched_character_pairs(update: &StateUpdate) -> Vec<String> {
    update
        .ops
        .iter()
        .filter_map(|op| match op {
            StateOp::SetCharacterState { character, key, .. }
            | StateOp::RemoveCharacterState { character, key } => {
                Some(format!("{character}:{key}"))
            }
            _ => None,
        })
        .collect()
}

fn truncate_text(text: &str) -> String {
    if text.chars().count() <= MAX_PREVIEW_CHARS {
        return text.to_owned();
    }

    let truncated: String = text.chars().take(MAX_PREVIEW_CHARS).collect();
    format!("{truncated}...[truncated]")
}

#[derive(Debug, Serialize)]
pub(crate) struct PlannerLogSummary {
    story_script_chars: usize,
    story_script_lines: usize,
    preview: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ArchitectLogSummary {
    node_count: usize,
    transition_count: usize,
    world_schema_fields: usize,
    player_schema_fields: usize,
    introduction_preview: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ArchitectDraftLogSummary {
    section_index: usize,
    total_sections: usize,
    node_count: usize,
    transition_patch_count: usize,
    world_schema_fields: usize,
    player_schema_fields: usize,
    section_summary_preview: String,
    introduction_preview: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct DirectorLogSummary {
    previous_node_id: String,
    current_node_id: String,
    transitioned: bool,
    beat_count: usize,
    beat_types: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub(crate) struct NarratorLogSummary {
    text_chars: usize,
    preview: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ActorLogSummary {
    speaker_id: String,
    speaker_name: String,
    segment_count: usize,
    thought_chars: usize,
    action_chars: usize,
    dialogue_chars: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct KeeperLogSummary {
    phase: KeeperPhase,
    op_count: usize,
    op_types: Vec<&'static str>,
    touched_global_keys: Vec<String>,
    touched_player_keys: Vec<String>,
    touched_character_pairs: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ReplyerLogSummary {
    reply_count: usize,
    reply_ids: Vec<String>,
    previews: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use agents::actor::{ActorSegment, ActorSegmentKind};
    use agents::keeper::KeeperPhase;
    use agents::replyer::ReplyOption;
    use serde_json::json;

    #[test]
    fn actor_summary_counts_segment_lengths_by_kind() {
        let summary = summarize_actor_response(&ActorResponse {
            speaker_id: "merchant".to_owned(),
            speaker_name: "Haru".to_owned(),
            segments: vec![
                ActorSegment {
                    kind: ActorSegmentKind::Thought,
                    text: "Think".to_owned(),
                },
                ActorSegment {
                    kind: ActorSegmentKind::Action,
                    text: "Act".to_owned(),
                },
                ActorSegment {
                    kind: ActorSegmentKind::Dialogue,
                    text: "Speak".to_owned(),
                },
            ],
            raw_output: String::new(),
        });

        assert_eq!(summary.segment_count, 3);
        assert_eq!(summary.thought_chars, 5);
        assert_eq!(summary.action_chars, 3);
        assert_eq!(summary.dialogue_chars, 5);
    }

    #[test]
    fn keeper_summary_tracks_op_types_and_keys() {
        let summary = summarize_keeper_response(
            KeeperPhase::AfterTurnOutputs,
            &KeeperResponse {
                update: StateUpdate {
                    ops: vec![
                        StateOp::SetState {
                            key: "flood_level".to_owned(),
                            value: json!(2),
                        },
                        StateOp::SetPlayerState {
                            key: "coins".to_owned(),
                            value: json!(3),
                        },
                        StateOp::SetCharacterState {
                            character: "merchant".to_owned(),
                            key: "trust".to_owned(),
                            value: json!(1),
                        },
                    ],
                },
                output: llm::ChatResponse {
                    message: llm::Message::new(llm::Role::Assistant, "{}"),
                    model: "test-model".to_owned(),
                    finish_reason: Some("stop".to_owned()),
                    usage: None,
                    structured_output: None,
                },
            },
        );

        assert_eq!(summary.op_count, 3);
        assert!(summary.op_types.contains(&"set_state"));
        assert_eq!(summary.touched_global_keys, vec!["flood_level"]);
        assert_eq!(summary.touched_player_keys, vec!["coins"]);
        assert_eq!(summary.touched_character_pairs, vec!["merchant:trust"]);
    }

    #[test]
    fn replyer_summary_truncates_long_texts() {
        let summary = summarize_reply_options(&[ReplyOption {
            id: "reply-1".to_owned(),
            text: "a".repeat(200),
        }]);

        assert_eq!(summary.reply_count, 1);
        assert!(summary.previews[0].contains("[truncated]"));
    }
}
