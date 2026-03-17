import { agentRoleKeys, type AgentRoleKey, type Preset } from '../apis/types'

export type PresetBundle = {
  presets: Preset[]
  type: 'preset_bundle'
  version: 1
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

function isPromptEntry(value: unknown) {
  if (!isObject(value)) {
    return false
  }

  return (
    typeof value.entry_id === 'string' &&
    typeof value.title === 'string' &&
    typeof value.content === 'string' &&
    typeof value.enabled === 'boolean'
  )
}

function isAgentPresetConfig(value: unknown) {
  if (!isObject(value)) {
    return false
  }

  if (
    'temperature' in value &&
    value.temperature !== undefined &&
    value.temperature !== null &&
    typeof value.temperature !== 'number'
  ) {
    return false
  }

  if (
    'max_tokens' in value &&
    value.max_tokens !== undefined &&
    value.max_tokens !== null &&
    typeof value.max_tokens !== 'number'
  ) {
    return false
  }

  if (
    'prompt_entries' in value &&
    value.prompt_entries !== undefined &&
    value.prompt_entries !== null &&
    (!Array.isArray(value.prompt_entries) || !value.prompt_entries.every(isPromptEntry))
  ) {
    return false
  }

  return true
}

function isPresetAgents(value: unknown): value is Preset['agents'] {
  if (!isObject(value)) {
    return false
  }

  return agentRoleKeys.every((roleKey: AgentRoleKey) => isAgentPresetConfig(value[roleKey]))
}

function isPreset(value: unknown): value is Preset {
  if (!isObject(value) || value.type !== 'preset') {
    return false
  }

  return (
    typeof value.preset_id === 'string' &&
    typeof value.display_name === 'string' &&
    isPresetAgents(value.agents)
  )
}

export function createPresetBundle(presets: ReadonlyArray<Preset>): PresetBundle {
  return {
    presets: [...presets],
    type: 'preset_bundle',
    version: 1,
  }
}

export function isPresetBundle(value: unknown): value is PresetBundle {
  if (!isObject(value) || value.type !== 'preset_bundle' || value.version !== 1) {
    return false
  }

  return Array.isArray(value.presets) && value.presets.every(isPreset)
}
