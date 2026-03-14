use protocol::{
    JsonRpcResponseMessage, PlayerProfileCreateParams, PlayerProfileDeleteParams,
    PlayerProfileDeletedPayload, PlayerProfilePayload, PlayerProfileUpdateParams,
    PlayerProfilesListedPayload, ResponseResult,
};
use store::PlayerProfileRecord;

use crate::error::HandlerError;

use super::Handler;

impl Handler {
    pub(crate) async fn handle_player_profile_create(
        &self,
        request_id: &str,
        params: PlayerProfileCreateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let player_profile_id = params.player_profile_id.trim().to_owned();
        if player_profile_id.is_empty() {
            return Err(HandlerError::EmptyPlayerProfileId);
        }

        if self
            .store
            .get_player_profile(&player_profile_id)
            .await?
            .is_some()
        {
            return Err(HandlerError::DuplicatePlayerProfile(player_profile_id));
        }

        let record = PlayerProfileRecord {
            player_profile_id,
            display_name: params.display_name,
            description: params.description,
        };
        self.store.save_player_profile(record.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::PlayerProfile(Box::new(player_profile_payload_from_record(&record))),
        ))
    }

    pub(crate) async fn handle_player_profile_get(
        &self,
        request_id: &str,
        params: protocol::PlayerProfileGetParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let record = self
            .store
            .get_player_profile(&params.player_profile_id)
            .await?
            .ok_or_else(|| HandlerError::MissingPlayerProfile(params.player_profile_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::PlayerProfile(Box::new(player_profile_payload_from_record(&record))),
        ))
    }

    pub(crate) async fn handle_player_profile_list(
        &self,
        request_id: &str,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let player_profiles = self
            .store
            .list_player_profiles()
            .await?
            .into_iter()
            .map(|record| player_profile_payload_from_record(&record))
            .collect();

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::PlayerProfilesListed(PlayerProfilesListedPayload { player_profiles }),
        ))
    }

    pub(crate) async fn handle_player_profile_update(
        &self,
        request_id: &str,
        params: PlayerProfileUpdateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let mut record = self
            .store
            .get_player_profile(&params.player_profile_id)
            .await?
            .ok_or_else(|| HandlerError::MissingPlayerProfile(params.player_profile_id.clone()))?;

        if let Some(display_name) = params.display_name {
            record.display_name = display_name;
        }
        if let Some(description) = params.description {
            record.description = description;
        }

        self.store.save_player_profile(record.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::PlayerProfile(Box::new(player_profile_payload_from_record(&record))),
        ))
    }

    pub(crate) async fn handle_player_profile_delete(
        &self,
        request_id: &str,
        params: PlayerProfileDeleteParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        if self
            .store
            .list_sessions()
            .await?
            .into_iter()
            .any(|session| {
                session.player_profile_id.as_deref() == Some(params.player_profile_id.as_str())
            })
        {
            return Err(HandlerError::PlayerProfileInUse(params.player_profile_id));
        }

        self.store
            .delete_player_profile(&params.player_profile_id)
            .await?
            .ok_or_else(|| HandlerError::MissingPlayerProfile(params.player_profile_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::PlayerProfileDeleted(PlayerProfileDeletedPayload {
                player_profile_id: params.player_profile_id,
            }),
        ))
    }
}

fn player_profile_payload_from_record(record: &PlayerProfileRecord) -> PlayerProfilePayload {
    PlayerProfilePayload {
        player_profile_id: record.player_profile_id.clone(),
        display_name: record.display_name.clone(),
        description: record.description.clone(),
    }
}
