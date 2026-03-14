use engine::LlmApiRegistry;
use std::sync::Arc;
use store::{LlmApiRecord, LlmProvider, Store};

use crate::config::AppConfig;
use crate::error::AppError;

pub async fn seed_store_and_build_registry(
    store: &Arc<dyn Store>,
    config: &AppConfig,
) -> Result<LlmApiRegistry, AppError> {
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
        };
        store.save_llm_api(record.clone()).await?;
        registry.upsert_record(&record)?;
    }

    Ok(registry)
}
