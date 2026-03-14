use protocol::{
    DashboardCountsPayload, DashboardHealthPayload, DashboardHealthStatus, DashboardPayload,
    DashboardSessionSummaryPayload, DashboardStorySummaryPayload, GlobalConfigPayload,
    JsonRpcResponseMessage, ResponseResult,
};

use crate::error::HandlerError;

use super::Handler;

const DASHBOARD_RECENT_LIMIT: usize = 5;

impl Handler {
    pub(crate) async fn handle_dashboard_get(
        &self,
        request_id: &str,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let global_config = self.manager.get_global_config().await?;
        let characters = self.store.list_characters().await?;
        let story_resources = self.store.list_story_resources().await?;
        let mut stories = self.store.list_stories().await?;
        let mut sessions = self.store.list_sessions().await?;

        stories.sort_by(|left, right| {
            right
                .updated_at_ms
                .unwrap_or(0)
                .cmp(&left.updated_at_ms.unwrap_or(0))
                .then_with(|| {
                    right
                        .created_at_ms
                        .unwrap_or(0)
                        .cmp(&left.created_at_ms.unwrap_or(0))
                })
                .then_with(|| right.story_id.cmp(&left.story_id))
        });
        sessions.sort_by(|left, right| {
            right
                .updated_at_ms
                .unwrap_or(0)
                .cmp(&left.updated_at_ms.unwrap_or(0))
                .then_with(|| {
                    right
                        .created_at_ms
                        .unwrap_or(0)
                        .cmp(&left.created_at_ms.unwrap_or(0))
                })
                .then_with(|| right.session_id.cmp(&left.session_id))
        });

        let payload = DashboardPayload {
            health: DashboardHealthPayload {
                status: DashboardHealthStatus::Ok,
            },
            counts: DashboardCountsPayload {
                characters_total: characters.len(),
                characters_with_cover: characters
                    .iter()
                    .filter(|character| character_has_cover(character))
                    .count(),
                story_resources_total: story_resources.len(),
                stories_total: stories.len(),
                sessions_total: sessions.len(),
            },
            global_config: GlobalConfigPayload {
                api_ids: global_config,
            },
            recent_stories: stories
                .into_iter()
                .take(DASHBOARD_RECENT_LIMIT)
                .map(|story| DashboardStorySummaryPayload {
                    story_id: story.story_id,
                    display_name: story.display_name,
                    resource_id: story.resource_id,
                    introduction: story.introduction,
                    updated_at_ms: story.updated_at_ms,
                })
                .collect(),
            recent_sessions: sessions
                .into_iter()
                .take(DASHBOARD_RECENT_LIMIT)
                .map(|session| DashboardSessionSummaryPayload {
                    session_id: session.session_id,
                    story_id: session.story_id,
                    display_name: session.display_name,
                    turn_index: session.snapshot.turn_index,
                    updated_at_ms: session.updated_at_ms,
                })
                .collect(),
        };

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Dashboard(Box::new(payload)),
        ))
    }
}

fn character_has_cover(character: &store::CharacterCardRecord) -> bool {
    character.cover_file_name.is_some()
        && character.cover_mime_type.is_some()
        && character.cover_bytes.is_some()
}
