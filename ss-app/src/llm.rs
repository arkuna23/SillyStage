use engine::LlmApiRegistry;
use std::sync::Arc;
use store::{DefaultLlmConfigRecord, LlmApiRecord, LlmProvider, Store};

use crate::config::AppConfig;
use crate::error::AppError;

pub async fn seed_store_and_build_registry(
    store: &Arc<dyn Store>,
    config: &AppConfig,
) -> Result<(LlmApiRegistry, Option<DefaultLlmConfigRecord>), AppError> {
    let registry = LlmApiRegistry::new();

    for record in store.list_llm_apis().await? {
        registry.upsert_record(&record)?;
    }

    for (api_id, api) in &config.llm.apis {
        if store.get_llm_api(api_id).await?.is_some() {
            continue;
        }

        let record = LlmApiRecord {
            api_id: api_id.clone(),
            provider: match api.provider {
                LlmProvider::OpenAi => LlmProvider::OpenAi,
            },
            base_url: api.base_url.clone(),
            api_key: api.api_key.clone(),
            model: api.model.clone(),
            temperature: api.temperature,
            max_tokens: api.max_tokens,
        };
        store.save_llm_api(record.clone()).await?;
        registry.upsert_record(&record)?;
    }

    let saved_default = store.get_default_llm_config().await?;
    let effective_default = config
        .llm
        .default_config
        .as_ref()
        .map(|config| DefaultLlmConfigRecord {
            provider: config.provider,
            base_url: config.base_url.clone(),
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            temperature: config.temperature,
            max_tokens: config.max_tokens,
        })
        .or(saved_default);

    Ok((registry, effective_default))
}
