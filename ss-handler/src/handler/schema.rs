use protocol::{
    JsonRpcResponseMessage, ResponseResult, SchemaCreateParams, SchemaDeleteParams,
    SchemaDeletedPayload, SchemaPayload, SchemaUpdateParams, SchemasListedPayload,
};
use store::SchemaRecord;

use crate::error::HandlerError;

use super::Handler;

impl Handler {
    pub(crate) async fn handle_schema_create(
        &self,
        request_id: &str,
        params: SchemaCreateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let schema_id = params.schema_id.trim().to_owned();
        if schema_id.is_empty() {
            return Err(HandlerError::EmptySchemaId);
        }

        if self.store.get_schema(&schema_id).await?.is_some() {
            return Err(HandlerError::DuplicateSchema(schema_id));
        }
        validate_schema_fields(&params.fields)?;

        let record = SchemaRecord {
            schema_id,
            display_name: params.display_name,
            tags: params.tags,
            fields: params.fields,
        };
        self.store.save_schema(record.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Schema(Box::new(schema_payload_from_record(&record))),
        ))
    }

    pub(crate) async fn handle_schema_get(
        &self,
        request_id: &str,
        params: protocol::SchemaGetParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let record = self
            .store
            .get_schema(&params.schema_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSchema(params.schema_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Schema(Box::new(schema_payload_from_record(&record))),
        ))
    }

    pub(crate) async fn handle_schema_list(
        &self,
        request_id: &str,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let schemas = self
            .store
            .list_schemas()
            .await?
            .into_iter()
            .map(|record| schema_payload_from_record(&record))
            .collect();

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::SchemasListed(SchemasListedPayload { schemas }),
        ))
    }

    pub(crate) async fn handle_schema_update(
        &self,
        request_id: &str,
        params: SchemaUpdateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let mut record = self
            .store
            .get_schema(&params.schema_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSchema(params.schema_id.clone()))?;

        if let Some(display_name) = params.display_name {
            record.display_name = display_name;
        }
        if let Some(tags) = params.tags {
            record.tags = tags;
        }
        if let Some(fields) = params.fields {
            validate_schema_fields(&fields)?;
            record.fields = fields;
        }

        self.store.save_schema(record.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Schema(Box::new(schema_payload_from_record(&record))),
        ))
    }

    pub(crate) async fn handle_schema_delete(
        &self,
        request_id: &str,
        params: SchemaDeleteParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let schema_id = params.schema_id.clone();

        if self
            .store
            .list_characters()
            .await?
            .into_iter()
            .any(|character| character.content.schema_id == schema_id)
        {
            return Err(HandlerError::SchemaInUse(params.schema_id));
        }

        if self
            .store
            .list_story_resources()
            .await?
            .into_iter()
            .any(|resource| {
                resource.player_schema_id_seed.as_deref() == Some(schema_id.as_str())
                    || resource.world_schema_id_seed.as_deref() == Some(schema_id.as_str())
            })
        {
            return Err(HandlerError::SchemaInUse(params.schema_id));
        }

        if self
            .store
            .list_stories()
            .await?
            .into_iter()
            .any(|story| story.player_schema_id == schema_id || story.world_schema_id == schema_id)
        {
            return Err(HandlerError::SchemaInUse(params.schema_id));
        }

        if self
            .store
            .list_sessions()
            .await?
            .into_iter()
            .any(|session| session.player_schema_id == schema_id)
        {
            return Err(HandlerError::SchemaInUse(params.schema_id));
        }

        self.store
            .delete_schema(&params.schema_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSchema(params.schema_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::SchemaDeleted(SchemaDeletedPayload {
                schema_id: params.schema_id,
            }),
        ))
    }
}

fn schema_payload_from_record(record: &SchemaRecord) -> SchemaPayload {
    SchemaPayload {
        schema_id: record.schema_id.clone(),
        display_name: record.display_name.clone(),
        tags: record.tags.clone(),
        fields: record.fields.clone(),
    }
}

fn validate_schema_fields(
    fields: &std::collections::HashMap<String, state::StateFieldSchema>,
) -> Result<(), HandlerError> {
    for (key, field) in fields {
        field.validate().map_err(|error| {
            HandlerError::InvalidSchemaDefinition(format!("field '{key}' {error}"))
        })?;
    }

    Ok(())
}
