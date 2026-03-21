use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use agents::actor::ActorSegmentKind;
use agents::architect::{GraphSummaryNode, NodeTransitionPatch};
use agents::replyer::{ReplyHistoryKind, ReplyHistoryMessage};
use state::StateFieldSchema;
use store::{
    SessionCharacterRecord, SessionMessageKind, SessionMessageRecord, SessionRecord,
    StoryResourcesRecord,
};
use story::{NarrativeNode, StoryGraph, validate_graph_state_conventions};

use crate::{EngineTurnResult, ExecutedBeat};

use super::ManagerError;

pub(super) fn now_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

pub(super) fn non_empty_planned_story(planned_story: Option<&str>) -> Option<String> {
    planned_story
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
}

pub(super) fn effective_planned_story_text(resource: &StoryResourcesRecord) -> String {
    non_empty_planned_story(resource.planned_story.as_deref())
        .unwrap_or_else(|| resource.story_concept.clone())
}

pub(super) fn effective_story_resources_display_name(resource: &StoryResourcesRecord) -> String {
    resource.effective_display_name()
}

pub(super) fn extract_outline_sections(planned_story: &str) -> Vec<String> {
    let mut sections = Vec::new();
    let mut in_suggested_beats = false;

    for raw_line in planned_story.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(opening) = line.strip_prefix("Opening Situation:") {
            let opening = opening.trim();
            if !opening.is_empty() {
                sections.push(opening.to_owned());
            }
            in_suggested_beats = false;
            continue;
        }

        if line == "Suggested Beats:" {
            in_suggested_beats = true;
            continue;
        }

        if matches!(
            line,
            "Title:" | "Core Conflict:" | "Character Roles:" | "State Hints:"
        ) {
            in_suggested_beats = false;
            continue;
        }

        if in_suggested_beats {
            let beat = line
                .trim_start_matches(|c: char| {
                    c == '-' || c == '*' || c.is_ascii_digit() || c == '.' || c == ')'
                })
                .trim();
            if !beat.is_empty() {
                sections.push(beat.to_owned());
            }
        }
    }

    if sections.is_empty() {
        planned_story
            .split("\n\n")
            .map(str::trim)
            .filter(|section| !section.is_empty() && !section.ends_with(':'))
            .map(ToOwned::to_owned)
            .collect()
    } else {
        sections
    }
}

pub(super) fn build_graph_summary(graph: &StoryGraph) -> Vec<GraphSummaryNode> {
    graph
        .nodes
        .iter()
        .map(|node| GraphSummaryNode {
            id: node.id.clone(),
            title: node.title.clone(),
            scene_summary: truncate_text(&node.scene, 200),
            goal: truncate_text(&node.goal, 120),
            characters: node.characters.clone(),
            transition_targets: node
                .transitions
                .iter()
                .map(|transition| transition.to.clone())
                .collect(),
        })
        .collect()
}

pub(super) fn truncate_text(text: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (idx, ch) in text.chars().enumerate() {
        if idx >= max_chars {
            out.push_str("...");
            break;
        }
        out.push(ch);
    }
    out
}

pub(super) fn build_reply_history(
    messages: Vec<SessionMessageRecord>,
    history_limit: usize,
) -> Vec<ReplyHistoryMessage> {
    let start = messages.len().saturating_sub(history_limit);
    filter_replyer_history_to_latest_narration_turn(
        messages
            .into_iter()
            .skip(start)
            .map(session_message_to_reply_history)
            .collect(),
    )
}

pub(super) fn filter_replyer_history_to_latest_narration_turn(
    history: Vec<ReplyHistoryMessage>,
) -> Vec<ReplyHistoryMessage> {
    let latest_narration_turn = history
        .iter()
        .filter(|message| matches!(message.kind, ReplyHistoryKind::Narration))
        .map(|message| message.turn_index)
        .max();

    history
        .into_iter()
        .filter(|message| {
            !matches!(message.kind, ReplyHistoryKind::Narration)
                || latest_narration_turn == Some(message.turn_index)
        })
        .collect()
}

pub(super) fn session_message_to_reply_history(
    message: SessionMessageRecord,
) -> ReplyHistoryMessage {
    ReplyHistoryMessage {
        kind: match message.kind {
            SessionMessageKind::PlayerInput => ReplyHistoryKind::PlayerInput,
            SessionMessageKind::Narration => ReplyHistoryKind::Narration,
            SessionMessageKind::Dialogue => ReplyHistoryKind::Dialogue,
            SessionMessageKind::Action => ReplyHistoryKind::Action,
        },
        turn_index: message.turn_index,
        speaker_id: message.speaker_id,
        speaker_name: message.speaker_name,
        text: message.text,
    }
}

pub(super) fn merge_story_chunk(
    graph: &mut StoryGraph,
    nodes: &[NarrativeNode],
    transition_patches: &[NodeTransitionPatch],
) -> Result<(), ManagerError> {
    for node in nodes {
        if graph.has_node(&node.id) {
            return Err(ManagerError::InvalidDraft(format!(
                "architect draft reused existing partial graph node id '{}' as a new node; existing nodes must only be referenced in transition targets or transition_patches",
                node.id
            )));
        }
        graph.nodes.push(node.clone());
    }

    apply_transition_patches(graph, transition_patches)?;
    validate_story_graph(graph)?;
    Ok(())
}

pub(super) fn apply_transition_patches(
    graph: &mut StoryGraph,
    transition_patches: &[NodeTransitionPatch],
) -> Result<(), ManagerError> {
    for patch in transition_patches {
        let node = graph.get_node_mut(&patch.node_id).ok_or_else(|| {
            ManagerError::InvalidDraft(format!(
                "architect draft attempted to patch missing node '{}'",
                patch.node_id
            ))
        })?;
        node.transitions.extend(patch.add_transitions.clone());
    }
    Ok(())
}

pub(super) fn validate_story_graph(graph: &StoryGraph) -> Result<(), ManagerError> {
    if graph.is_empty() {
        return Err(ManagerError::InvalidDraft(
            "story graph must contain at least one node".to_owned(),
        ));
    }

    let mut node_ids = HashSet::new();
    for node in &graph.nodes {
        if !node_ids.insert(node.id.clone()) {
            return Err(ManagerError::InvalidDraft(format!(
                "story graph contains duplicate node id '{}'",
                node.id
            )));
        }
    }

    if !node_ids.contains(graph.start_node()) {
        return Err(ManagerError::InvalidDraft(format!(
            "story graph start node '{}' does not exist",
            graph.start_node()
        )));
    }

    for node in &graph.nodes {
        for transition in &node.transitions {
            if !node_ids.contains(&transition.to) {
                return Err(ManagerError::InvalidDraft(format!(
                    "transition from '{}' points to missing node '{}'",
                    node.id, transition.to
                )));
            }
        }
    }

    validate_graph_state_conventions(graph)
        .map_err(|error| ManagerError::InvalidDraft(error.to_string()))?;

    Ok(())
}

pub(super) fn ensure_session_character_belongs(
    session_id: &str,
    character: &SessionCharacterRecord,
) -> Result<(), ManagerError> {
    if character.session_id == session_id {
        return Ok(());
    }

    Err(ManagerError::MissingSessionCharacter(
        character.session_character_id.clone(),
    ))
}

pub(super) fn next_session_message_sequence(existing: &[SessionMessageRecord]) -> u64 {
    existing
        .iter()
        .map(|message| message.sequence)
        .max()
        .map(|sequence| sequence.saturating_add(1))
        .unwrap_or(0)
}

pub(super) fn build_session_messages(
    session_id: &str,
    session: &SessionRecord,
    result: &EngineTurnResult,
    recorded_at_ms: u64,
    starting_sequence: u64,
) -> Vec<SessionMessageRecord> {
    let mut next_sequence = starting_sequence;
    let mut messages = vec![SessionMessageRecord {
        message_id: format!("{}-message-{}", session_id, next_sequence),
        session_id: session.session_id.clone(),
        kind: SessionMessageKind::PlayerInput,
        sequence: next_sequence,
        turn_index: result.turn_index,
        recorded_at_ms,
        created_at_ms: recorded_at_ms,
        updated_at_ms: recorded_at_ms,
        speaker_id: "player".to_owned(),
        speaker_name: "Player".to_owned(),
        text: result.player_input.clone(),
    }];
    next_sequence = next_sequence.saturating_add(1);

    for beat in &result.completed_beats {
        match beat {
            ExecutedBeat::Narrator { response, .. } => {
                let text = response.text.trim();
                if !text.is_empty() {
                    messages.push(SessionMessageRecord {
                        message_id: format!("{}-message-{}", session_id, next_sequence),
                        session_id: session.session_id.clone(),
                        kind: SessionMessageKind::Narration,
                        sequence: next_sequence,
                        turn_index: result.turn_index,
                        recorded_at_ms,
                        created_at_ms: recorded_at_ms,
                        updated_at_ms: recorded_at_ms,
                        speaker_id: "narrator".to_owned(),
                        speaker_name: "Narrator".to_owned(),
                        text: text.to_owned(),
                    });
                    next_sequence = next_sequence.saturating_add(1);
                }
            }
            ExecutedBeat::Actor { response, .. } => {
                for segment in &response.segments {
                    let kind = match segment.kind {
                        ActorSegmentKind::Dialogue => Some(SessionMessageKind::Dialogue),
                        ActorSegmentKind::Action => Some(SessionMessageKind::Action),
                        ActorSegmentKind::Thought => None,
                    };

                    let Some(kind) = kind else {
                        continue;
                    };

                    let text = segment.text.trim();
                    if text.is_empty() {
                        continue;
                    }

                    messages.push(SessionMessageRecord {
                        message_id: format!("{}-message-{}", session_id, next_sequence),
                        session_id: session.session_id.clone(),
                        kind,
                        sequence: next_sequence,
                        turn_index: result.turn_index,
                        recorded_at_ms,
                        created_at_ms: recorded_at_ms,
                        updated_at_ms: recorded_at_ms,
                        speaker_id: response.speaker_id.clone(),
                        speaker_name: response.speaker_name.clone(),
                        text: text.to_owned(),
                    });
                    next_sequence = next_sequence.saturating_add(1);
                }
            }
        }
    }

    messages
}

pub(super) fn validate_schema_fields(
    fields: &HashMap<String, StateFieldSchema>,
) -> Result<(), ManagerError> {
    for (key, field) in fields {
        field.validate().map_err(|error| {
            ManagerError::InvalidGeneratedSchema(format!("field '{key}' {error}"))
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session_message(
        sequence: u64,
        turn_index: u64,
        kind: SessionMessageKind,
        text: &str,
    ) -> SessionMessageRecord {
        SessionMessageRecord {
            message_id: format!("message-{sequence}"),
            session_id: "session-1".to_owned(),
            kind,
            sequence,
            turn_index,
            recorded_at_ms: 0,
            created_at_ms: 0,
            updated_at_ms: 0,
            speaker_id: "speaker".to_owned(),
            speaker_name: "Speaker".to_owned(),
            text: text.to_owned(),
        }
    }

    #[test]
    fn build_reply_history_keeps_only_latest_turn_narration() {
        let history = build_reply_history(
            vec![
                session_message(0, 1, SessionMessageKind::PlayerInput, "Where is the gate?"),
                session_message(1, 1, SessionMessageKind::Narration, "The square is crowded."),
                session_message(2, 1, SessionMessageKind::Dialogue, "This way."),
                session_message(3, 2, SessionMessageKind::PlayerInput, "Lead on."),
                session_message(4, 2, SessionMessageKind::Narration, "Rain starts to fall."),
                session_message(5, 2, SessionMessageKind::Narration, "The torches hiss."),
                session_message(6, 2, SessionMessageKind::Action, "The guide raises a lamp."),
            ],
            7,
        );

        assert_eq!(history.len(), 6);
        assert!(history.iter().any(|message| {
            message.kind == ReplyHistoryKind::Narration
                && message.turn_index == 2
                && message.text == "Rain starts to fall."
        }));
        assert!(history.iter().any(|message| {
            message.kind == ReplyHistoryKind::Narration
                && message.turn_index == 2
                && message.text == "The torches hiss."
        }));
        assert!(!history.iter().any(|message| {
            message.kind == ReplyHistoryKind::Narration && message.turn_index == 1
        }));
    }

    #[test]
    fn build_reply_history_honors_zero_limit() {
        let history = build_reply_history(
            vec![session_message(
                0,
                1,
                SessionMessageKind::Narration,
                "A bell rings in the distance.",
            )],
            0,
        );

        assert!(history.is_empty());
    }
}
