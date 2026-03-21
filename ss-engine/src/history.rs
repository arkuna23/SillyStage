use store::AgentPresetConfig;

pub(crate) const DEFAULT_MESSAGE_HISTORY_LIMIT: usize = 8;

pub(crate) fn resolve_director_shared_history_limit(config: &AgentPresetConfig) -> usize {
    config
        .director_shared_history_limit
        .unwrap_or(DEFAULT_MESSAGE_HISTORY_LIMIT)
}

pub(crate) fn resolve_actor_shared_history_limit(config: &AgentPresetConfig) -> usize {
    config
        .actor_shared_history_limit
        .unwrap_or(DEFAULT_MESSAGE_HISTORY_LIMIT)
}

pub(crate) fn resolve_actor_private_memory_limit(config: &AgentPresetConfig) -> usize {
    config
        .actor_private_memory_limit
        .unwrap_or(DEFAULT_MESSAGE_HISTORY_LIMIT)
}

pub(crate) fn resolve_narrator_shared_history_limit(config: &AgentPresetConfig) -> usize {
    config
        .narrator_shared_history_limit
        .unwrap_or(DEFAULT_MESSAGE_HISTORY_LIMIT)
}

pub(crate) fn resolve_replyer_session_history_limit(config: &AgentPresetConfig) -> usize {
    config
        .replyer_session_history_limit
        .unwrap_or(DEFAULT_MESSAGE_HISTORY_LIMIT)
}

pub(crate) fn resolve_runtime_shared_memory_limit(
    director: &AgentPresetConfig,
    actor: &AgentPresetConfig,
    narrator: &AgentPresetConfig,
) -> usize {
    resolve_director_shared_history_limit(director)
        .max(resolve_actor_shared_history_limit(actor))
        .max(resolve_narrator_shared_history_limit(narrator))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_shared_memory_limit_uses_largest_runtime_window() {
        let director = AgentPresetConfig {
            director_shared_history_limit: Some(12),
            ..super::super::prompt::default_agent_preset_config(
                super::super::prompt::PromptAgentKind::Director,
            )
        };
        let actor = AgentPresetConfig {
            actor_shared_history_limit: Some(3),
            ..super::super::prompt::default_agent_preset_config(
                super::super::prompt::PromptAgentKind::Actor,
            )
        };
        let narrator = AgentPresetConfig {
            narrator_shared_history_limit: Some(20),
            ..super::super::prompt::default_agent_preset_config(
                super::super::prompt::PromptAgentKind::Narrator,
            )
        };

        assert_eq!(
            resolve_runtime_shared_memory_limit(&director, &actor, &narrator),
            20
        );
    }
}
