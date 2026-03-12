use engine::LlmApiRegistry;
use llm::{OpenAiClient, OpenAiConfig};
use std::sync::Arc;

use crate::config::{AppConfig, LlmProvider};
use crate::error::AppError;

pub fn build_registry(config: &AppConfig) -> Result<LlmApiRegistry, AppError> {
    let mut registry = LlmApiRegistry::new();

    for (api_id, api) in &config.llm.apis {
        match api.provider {
            LlmProvider::OpenAi => {
                let openai = OpenAiConfig::builder()
                    .api_key(&api.api_key)
                    .base_url(&api.base_url)
                    .default_model(&api.model)
                    .build()?;
                let client: Arc<dyn llm::LlmApi> = Arc::new(OpenAiClient::new(openai)?);
                registry = registry.register(api_id.clone(), client, api.model.clone());
            }
        }
    }

    Ok(registry)
}
