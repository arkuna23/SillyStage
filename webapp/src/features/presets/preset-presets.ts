import type { TFunction } from 'i18next'

import { agentRoleKeys, type PresetAgentConfigs, type PresetPromptEntry } from '../apis/types'

export type PresetTemplateKind = 'balanced' | 'expressive'

export type PresetTemplateDefinition = {
  agents: PresetAgentConfigs
  description: string
  displayName: string
  kind: PresetTemplateKind
  presetId: string
}

function createEmptyPresetAgents(): PresetAgentConfigs {
  return agentRoleKeys.reduce<PresetAgentConfigs>((agents, roleKey) => {
    agents[roleKey] = { prompt_entries: [] }
    return agents
  }, {} as PresetAgentConfigs)
}

function createNarratorPromptEntries(t: TFunction): PresetPromptEntry[] {
  return [
    {
      content: t('presetsPage.presets.defaultPrompts.narrator.actionFocus.content'),
      enabled: true,
      entry_id: 'narrator-stage-action-focus',
      title: t('presetsPage.presets.defaultPrompts.narrator.actionFocus.title'),
    },
    {
      content: t('presetsPage.presets.defaultPrompts.narrator.variation.content'),
      enabled: true,
      entry_id: 'narrator-stage-variation',
      title: t('presetsPage.presets.defaultPrompts.narrator.variation.title'),
    },
  ]
}

function createDirectorPromptEntries(t: TFunction): PresetPromptEntry[] {
  return [
    {
      content: t('presetsPage.presets.defaultPrompts.director.narrationTiming.content'),
      enabled: true,
      entry_id: 'director-stage-narration-timing',
      title: t('presetsPage.presets.defaultPrompts.director.narrationTiming.title'),
    },
  ]
}

function createActorPromptEntries(t: TFunction): PresetPromptEntry[] {
  return [
    {
      content: t('presetsPage.presets.defaultPrompts.actor.actionVariety.content'),
      enabled: true,
      entry_id: 'actor-stage-action-variety',
      title: t('presetsPage.presets.defaultPrompts.actor.actionVariety.title'),
    },
  ]
}

function createKeeperPromptEntries(t: TFunction): PresetPromptEntry[] {
  return [
    {
      content: t('presetsPage.presets.defaultPrompts.keeper.stateProgression.content'),
      enabled: true,
      entry_id: 'keeper-stage-state-progression',
      title: t('presetsPage.presets.defaultPrompts.keeper.stateProgression.title'),
    },
  ]
}

function createReplyerPromptEntries(t: TFunction): PresetPromptEntry[] {
  return [
    {
      content: t('presetsPage.presets.defaultPrompts.replyer.actionBrackets.content'),
      enabled: true,
      entry_id: 'replyer-stage-action-brackets',
      title: t('presetsPage.presets.defaultPrompts.replyer.actionBrackets.title'),
    },
  ]
}

export function buildPresetTemplateDefinitions(t: TFunction): PresetTemplateDefinition[] {
  const balancedAgents = createEmptyPresetAgents()
  balancedAgents.planner = { max_tokens: 1400, temperature: 0.55 }
  balancedAgents.architect = { max_tokens: 8192, temperature: 0.6 }
  balancedAgents.director = {
    max_tokens: 1400,
    prompt_entries: createDirectorPromptEntries(t),
    temperature: 0.65,
  }
  balancedAgents.actor = {
    max_tokens: 512,
    prompt_entries: createActorPromptEntries(t),
    temperature: 0.85,
  }
  balancedAgents.narrator = {
    max_tokens: 900,
    prompt_entries: createNarratorPromptEntries(t),
    temperature: 0.7,
  }
  balancedAgents.keeper = {
    max_tokens: 800,
    prompt_entries: createKeeperPromptEntries(t),
    temperature: 0.3,
  }
  balancedAgents.replyer = {
    max_tokens: 420,
    prompt_entries: createReplyerPromptEntries(t),
    temperature: 0.75,
  }

  const expressiveAgents = createEmptyPresetAgents()
  expressiveAgents.planner = { max_tokens: 1800, temperature: 0.65 }
  expressiveAgents.architect = { max_tokens: 8192, temperature: 0.72 }
  expressiveAgents.director = {
    max_tokens: 1800,
    prompt_entries: createDirectorPromptEntries(t),
    temperature: 0.8,
  }
  expressiveAgents.actor = {
    max_tokens: 640,
    prompt_entries: createActorPromptEntries(t),
    temperature: 1.0,
  }
  expressiveAgents.narrator = {
    max_tokens: 1200,
    prompt_entries: createNarratorPromptEntries(t),
    temperature: 0.88,
  }
  expressiveAgents.keeper = {
    max_tokens: 900,
    prompt_entries: createKeeperPromptEntries(t),
    temperature: 0.4,
  }
  expressiveAgents.replyer = {
    max_tokens: 520,
    prompt_entries: createReplyerPromptEntries(t),
    temperature: 0.92,
  }

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
