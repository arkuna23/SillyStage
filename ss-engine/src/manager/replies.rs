use std::sync::Arc;

use agents::replyer::{
    ReplyHistoryKind, ReplyHistoryMessage, ReplyOption, Replyer, ReplyerRequest,
};
use tracing::{debug, info};

use crate::RuntimeState;
use crate::logging::{json_for_log, summarize_reply_options};
use crate::lorebook::{LorebookPromptSections, build_lorebook_prompt_sections};

use super::{DEFAULT_REPLY_HISTORY_LIMIT, EngineManager, ManagerError};

impl EngineManager {
    pub async fn suggest_replies(
        &self,
        session_id: &str,
        limit: usize,
    ) -> Result<Vec<ReplyOption>, ManagerError> {
        let session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        let story = self
            .store
            .get_story(&session.story_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStory(session.story_id.clone()))?;
        let runtime_state = self
            .build_runtime_state_from_session(&story, &session)
            .await?;
        let api_group = self
            .resolve_api_group(&session.binding.api_group_id)
            .await?;
        let preset = self.resolve_preset(&session.binding.preset_id).await?;
        let apis = self.resolve_api_group_bindings(&api_group).await?;
        let replyer_config = self
            .registry
            .build_replyer_config(&apis.replyer, &preset.agents.replyer)?;
        let history = self.load_reply_history(session_id).await?;
        let current_node = runtime_state.current_node()?;
        let replyer = Replyer::new_with_options(
            Arc::clone(&replyer_config.client),
            replyer_config.model.clone(),
            replyer_config.temperature,
            replyer_config.max_tokens,
        )?
        .with_prompt_profile(replyer_config.prompt_profile.clone());
        let lorebook_sections = reply_lorebook_sections(&runtime_state, current_node, &history);
        let response = replyer
            .suggest(ReplyerRequest {
                current_node,
                character_cards: runtime_state.character_cards(),
                current_cast_ids: runtime_state.world_state().active_characters(),
                lorebook_base: lorebook_sections.base.as_deref(),
                lorebook_matched: lorebook_sections.matched.as_deref(),
                player_name: runtime_state.player_name(),
                player_description: runtime_state.player_description(),
                player_state_schema: runtime_state.player_state_schema(),
                world_state: runtime_state.world_state(),
                history: &history,
                limit,
            })
            .await?;

        info!(
            session_id = %session_id,
            summary = %json_for_log(&summarize_reply_options(&response.replies)),
            "replyer generated suggested replies"
        );
        debug!(
            session_id = %session_id,
            payload = %json_for_log(&response),
            "replyer response payload"
        );

        Ok(response.replies)
    }

    async fn load_reply_history(
        &self,
        session_id: &str,
    ) -> Result<Vec<ReplyHistoryMessage>, ManagerError> {
        let mut messages = self.store.list_session_messages(session_id).await?;
        messages.sort_by_key(|message| message.sequence);
        let start = messages.len().saturating_sub(DEFAULT_REPLY_HISTORY_LIMIT);
        Ok(messages
            .into_iter()
            .skip(start)
            .map(|message| ReplyHistoryMessage {
                kind: match message.kind {
                    store::SessionMessageKind::PlayerInput => ReplyHistoryKind::PlayerInput,
                    store::SessionMessageKind::Narration => ReplyHistoryKind::Narration,
                    store::SessionMessageKind::Dialogue => ReplyHistoryKind::Dialogue,
                    store::SessionMessageKind::Action => ReplyHistoryKind::Action,
                },
                turn_index: message.turn_index,
                speaker_id: message.speaker_id,
                speaker_name: message.speaker_name,
                text: message.text,
            })
            .collect())
    }
}

fn reply_lorebook_sections<'a>(
    runtime_state: &'a RuntimeState,
    current_node: &'a story::NarrativeNode,
    history: &'a [ReplyHistoryMessage],
) -> LorebookPromptSections {
    let mut match_inputs = vec![
        current_node.title.as_str(),
        current_node.scene.as_str(),
        current_node.goal.as_str(),
    ];
    match_inputs.extend(
        history
            .iter()
            .map(|message| message.text.as_str())
            .filter(|text| !text.trim().is_empty()),
    );

    build_lorebook_prompt_sections(runtime_state.lorebook_entries(), &match_inputs)
}
