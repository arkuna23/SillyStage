import type { TFunction } from 'i18next'

import { agentRoleKeys, type PresetAgentConfigs } from '../apis/types'

export type PresetTemplateKind = 'balanced' | 'expressive'

export type PresetTemplateDefinition = {
  agents: PresetAgentConfigs
  description: string
  displayName: string
  kind: PresetTemplateKind
  presetId: string
}

function createEmptyPresetAgents(): PresetAgentConfigs {
  return Object.fromEntries(
    agentRoleKeys.map((roleKey) => [roleKey, {}]),
  ) as PresetAgentConfigs
}

export function buildPresetTemplateDefinitions(t: TFunction): PresetTemplateDefinition[] {
  const balancedAgents = createEmptyPresetAgents()
  balancedAgents.planner = { max_tokens: 900, temperature: 0.55 }
  balancedAgents.architect = { max_tokens: 8192, temperature: 0.6 }
  balancedAgents.director = { max_tokens: 900, temperature: 0.65 }
  balancedAgents.actor = { max_tokens: 320, temperature: 0.85 }
  balancedAgents.narrator = { max_tokens: 420, temperature: 0.7 }
  balancedAgents.keeper = { max_tokens: 500, temperature: 0.3 }
  balancedAgents.replyer = { max_tokens: 220, temperature: 0.75 }

  const expressiveAgents = createEmptyPresetAgents()
  expressiveAgents.planner = { max_tokens: 1000, temperature: 0.65 }
  expressiveAgents.architect = { max_tokens: 8192, temperature: 0.72 }
  expressiveAgents.director = { max_tokens: 1000, temperature: 0.8 }
  expressiveAgents.actor = { max_tokens: 420, temperature: 1.0 }
  expressiveAgents.narrator = { max_tokens: 520, temperature: 0.88 }
  expressiveAgents.keeper = { max_tokens: 560, temperature: 0.4 }
  expressiveAgents.replyer = { max_tokens: 260, temperature: 0.92 }

  return [
    {
      agents: balancedAgents,
      description: t('presetsPage.presets.balanced.description'),
      displayName: t('presetsPage.presets.balanced.title'),
      kind: 'balanced',
      presetId: 'preset-stage-balanced',
    },
    {
      agents: expressiveAgents,
      description: t('presetsPage.presets.expressive.description'),
      displayName: t('presetsPage.presets.expressive.title'),
      kind: 'expressive',
      presetId: 'preset-stage-expressive',
    },
  ]
}
