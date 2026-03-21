import type { TFunction } from 'i18next'

import { agentRoleKeys, type PresetAgentConfigs } from '../apis/types'

export type PresetTemplateKind = 'default'

export type PresetTemplateDefinition = {
  agents: PresetAgentConfigs
  description: string
  displayName: string
  kind: PresetTemplateKind
  presetId: string
}

function createEmptyPresetAgents(): PresetAgentConfigs {
  return agentRoleKeys.reduce<PresetAgentConfigs>((agents, roleKey) => {
    agents[roleKey] = { modules: [] }
    return agents
  }, {} as PresetAgentConfigs)
}

export function buildPresetTemplateDefinitions(t: TFunction): PresetTemplateDefinition[] {
  const defaultAgents = createEmptyPresetAgents()
  defaultAgents.planner = { max_tokens: 8192, modules: [], temperature: 0.55 }
  defaultAgents.architect = { max_tokens: 8192, modules: [], temperature: 0.6 }
  defaultAgents.director = {
    director_shared_history_limit: 10,
    max_tokens: 8192,
    modules: [],
    temperature: 0.65,
  }
  defaultAgents.actor = {
    actor_private_memory_limit: 4,
    actor_shared_history_limit: 12,
    max_tokens: 8192,
    modules: [],
    temperature: 0.85,
  }
  defaultAgents.narrator = {
    max_tokens: 8192,
    modules: [],
    narrator_shared_history_limit: 6,
    temperature: 0.7,
  }
  defaultAgents.keeper = { max_tokens: 8192, modules: [], temperature: 0.3 }
  defaultAgents.replyer = {
    max_tokens: 8192,
    modules: [],
    replyer_session_history_limit: 5,
    temperature: 0.75,
  }

  return [
    {
      agents: defaultAgents,
      description: t('presetsPage.presets.default.description'),
      displayName: t('presetsPage.presets.default.title'),
      kind: 'default',
      presetId: 'preset-stage-default',
    },
  ]
}
