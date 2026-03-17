use protocol::{
    JsonRpcResponseMessage, LorebookCreateParams, LorebookDeleteParams, LorebookDeletedPayload,
    LorebookEntriesListedPayload, LorebookEntryCreateParams, LorebookEntryDeleteParams,
    LorebookEntryDeletedPayload, LorebookEntryPayload, LorebookEntryUpdateParams, LorebookPayload,
    LorebookUpdateParams, LorebooksListedPayload, ResponseResult,
};
use store::{LorebookEntryRecord, LorebookRecord};

use crate::error::HandlerError;

use super::Handler;

impl Handler {
    pub(crate) async fn handle_lorebook_create(
        &self,
        request_id: &str,
        params: LorebookCreateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let lorebook_id = normalize_lorebook_id(&params.lorebook_id)?;
        if self.store.get_lorebook(&lorebook_id).await?.is_some() {
            return Err(HandlerError::DuplicateLorebook(lorebook_id));
        }

        let mut seen_entry_ids = std::collections::HashSet::new();
        let mut entries = Vec::with_capacity(params.entries.len());
        for entry in params.entries {
            let entry_id = normalize_lorebook_entry_id(&entry.entry_id)?;
            if !seen_entry_ids.insert(entry_id.clone()) {
                return Err(HandlerError::DuplicateLorebookEntry {
                    lorebook_id: lorebook_id.clone(),
                    entry_id,
                });
            }
            entries.push(LorebookEntryRecord {
                entry_id,
                title: entry.title,
                content: entry.content,
                keywords: entry.keywords,
                enabled: entry.enabled,
                always_include: entry.always_include,
            });
        }

        let record = LorebookRecord {
            lorebook_id,
            display_name: params.display_name,
            entries,
        };
        self.store.save_lorebook(record.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Lorebook(Box::new(lorebook_payload_from_record(&record))),
        ))
    }

    pub(crate) async fn handle_lorebook_get(
        &self,
        request_id: &str,
        params: protocol::LorebookGetParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let lorebook = self
            .store
            .get_lorebook(&params.lorebook_id)
            .await?
            .ok_or_else(|| HandlerError::MissingLorebook(params.lorebook_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Lorebook(Box::new(lorebook_payload_from_record(&lorebook))),
        ))
    }

    pub(crate) async fn handle_lorebook_list(
        &self,
        request_id: &str,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let mut lorebooks = self
            .store
            .list_lorebooks()
            .await?
            .into_iter()
            .map(|record| lorebook_payload_from_record(&record))
            .collect::<Vec<_>>();
        lorebooks.sort_by(|left, right| left.lorebook_id.cmp(&right.lorebook_id));

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::LorebooksListed(LorebooksListedPayload { lorebooks }),
        ))
    }

    pub(crate) async fn handle_lorebook_update(
        &self,
        request_id: &str,
        params: LorebookUpdateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let mut record = self
            .store
            .get_lorebook(&params.lorebook_id)
            .await?
            .ok_or_else(|| HandlerError::MissingLorebook(params.lorebook_id.clone()))?;

        if let Some(display_name) = params.display_name {
            record.display_name = display_name;
        }

        self.store.save_lorebook(record.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Lorebook(Box::new(lorebook_payload_from_record(&record))),
        ))
    }

    pub(crate) async fn handle_lorebook_delete(
        &self,
        request_id: &str,
        params: LorebookDeleteParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        if self
            .store
            .list_story_resources()
            .await?
            .into_iter()
            .any(|resource| {
                resource
                    .lorebook_ids
                    .iter()
                    .any(|id| id == &params.lorebook_id)
            })
        {
            return Err(HandlerError::LorebookInUse(params.lorebook_id));
        }

        self.store
            .delete_lorebook(&params.lorebook_id)
            .await?
            .ok_or_else(|| HandlerError::MissingLorebook(params.lorebook_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::LorebookDeleted(LorebookDeletedPayload {
                lorebook_id: params.lorebook_id,
            }),
        ))
    }

    pub(crate) async fn handle_lorebook_entry_create(
        &self,
        request_id: &str,
        params: LorebookEntryCreateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let entry_id = normalize_lorebook_entry_id(&params.entry_id)?;
        let mut lorebook = self
            .store
            .get_lorebook(&params.lorebook_id)
            .await?
            .ok_or_else(|| HandlerError::MissingLorebook(params.lorebook_id.clone()))?;

        if lorebook
            .entries
            .iter()
            .any(|entry| entry.entry_id == entry_id)
        {
            return Err(HandlerError::DuplicateLorebookEntry {
                lorebook_id: lorebook.lorebook_id.clone(),
                entry_id,
            });
        }

        let entry = LorebookEntryRecord {
            entry_id: entry_id.clone(),
            title: params.title,
            content: params.content,
            keywords: params.keywords,
            enabled: params.enabled,
            always_include: params.always_include,
        };
        lorebook.entries.push(entry.clone());
        self.store.save_lorebook(lorebook).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::LorebookEntry(Box::new(lorebook_entry_payload_from_record(&entry))),
        ))
    }

    pub(crate) async fn handle_lorebook_entry_get(
        &self,
        request_id: &str,
        params: protocol::LorebookEntryGetParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let lorebook = self
            .store
            .get_lorebook(&params.lorebook_id)
            .await?
            .ok_or_else(|| HandlerError::MissingLorebook(params.lorebook_id.clone()))?;
        let entry = lorebook
            .entries
            .iter()
            .find(|entry| entry.entry_id == params.entry_id)
            .ok_or_else(|| HandlerError::MissingLorebookEntry {
                lorebook_id: params.lorebook_id.clone(),
                entry_id: params.entry_id.clone(),
            })?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::LorebookEntry(Box::new(lorebook_entry_payload_from_record(entry))),
        ))
    }

    pub(crate) async fn handle_lorebook_entry_list(
        &self,
        request_id: &str,
        params: protocol::LorebookEntryListParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let lorebook = self
            .store
            .get_lorebook(&params.lorebook_id)
            .await?
            .ok_or_else(|| HandlerError::MissingLorebook(params.lorebook_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::LorebookEntriesListed(LorebookEntriesListedPayload {
                lorebook_id: lorebook.lorebook_id.clone(),
                entries: lorebook
                    .entries
                    .iter()
                    .map(lorebook_entry_payload_from_record)
                    .collect(),
            }),
        ))
    }

    pub(crate) async fn handle_lorebook_entry_update(
        &self,
        request_id: &str,
        params: LorebookEntryUpdateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let mut lorebook = self
            .store
            .get_lorebook(&params.lorebook_id)
            .await?
            .ok_or_else(|| HandlerError::MissingLorebook(params.lorebook_id.clone()))?;

        let entry = lorebook
            .entries
            .iter_mut()
            .find(|entry| entry.entry_id == params.entry_id)
            .ok_or_else(|| HandlerError::MissingLorebookEntry {
                lorebook_id: params.lorebook_id.clone(),
                entry_id: params.entry_id.clone(),
            })?;

        if let Some(title) = params.title {
            entry.title = title;
        }
        if let Some(content) = params.content {
            entry.content = content;
        }
        if let Some(keywords) = params.keywords {
            entry.keywords = keywords;
        }
        if let Some(enabled) = params.enabled {
            entry.enabled = enabled;
        }
        if let Some(always_include) = params.always_include {
            entry.always_include = always_include;
        }

        let payload = lorebook_entry_payload_from_record(entry);
        self.store.save_lorebook(lorebook).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::LorebookEntry(Box::new(payload)),
        ))
    }

    pub(crate) async fn handle_lorebook_entry_delete(
        &self,
        request_id: &str,
        params: LorebookEntryDeleteParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let mut lorebook = self
            .store
            .get_lorebook(&params.lorebook_id)
            .await?
            .ok_or_else(|| HandlerError::MissingLorebook(params.lorebook_id.clone()))?;

        let before_len = lorebook.entries.len();
        lorebook
            .entries
            .retain(|entry| entry.entry_id != params.entry_id);
        if lorebook.entries.len() == before_len {
            return Err(HandlerError::MissingLorebookEntry {
                lorebook_id: params.lorebook_id.clone(),
                entry_id: params.entry_id.clone(),
            });
        }
        self.store.save_lorebook(lorebook).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::LorebookEntryDeleted(LorebookEntryDeletedPayload {
                lorebook_id: params.lorebook_id,
                entry_id: params.entry_id,
            }),
        ))
    }
}

fn normalize_lorebook_id(lorebook_id: &str) -> Result<String, HandlerError> {
    let trimmed = lorebook_id.trim();
    if trimmed.is_empty() {
        return Err(HandlerError::EmptyLorebookId);
    }
    Ok(trimmed.to_owned())
}

fn normalize_lorebook_entry_id(entry_id: &str) -> Result<String, HandlerError> {
    let trimmed = entry_id.trim();
    if trimmed.is_empty() {
        return Err(HandlerError::EmptyLorebookEntryId);
    }
    Ok(trimmed.to_owned())
}

fn lorebook_payload_from_record(record: &LorebookRecord) -> LorebookPayload {
    LorebookPayload {
        lorebook_id: record.lorebook_id.clone(),
        display_name: record.display_name.clone(),
        entries: record
            .entries
            .iter()
            .map(lorebook_entry_payload_from_record)
            .collect(),
    }
}

fn lorebook_entry_payload_from_record(record: &LorebookEntryRecord) -> LorebookEntryPayload {
    LorebookEntryPayload {
        entry_id: record.entry_id.clone(),
        title: record.title.clone(),
        content: record.content.clone(),
        keywords: record.keywords.clone(),
        enabled: record.enabled,
        always_include: record.always_include,
    }
}
