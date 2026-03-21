import {
  type AgentRoleKey,
  agentRoleKeys,
  type PresetDetail,
  type PromptMessageRole,
  presetEntryKinds,
} from '../apis/types'

export type PresetBundle = {
  presets: PresetDetail[]
  type: 'preset_bundle'
  version: 2
}

const optionalNumericPresetConfigKeys = [
  'temperature',
  'max_tokens',
  'director_shared_history_limit',
  'actor_shared_history_limit',
  'actor_private_memory_limit',
  'narrator_shared_history_limit',
  'replyer_session_history_limit',
] as const

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

function isPromptEntryKind(value: unknown) {
  return typeof value === 'string' && presetEntryKinds.includes(value as never)
}

function isPromptMessageRole(value: unknown): value is PromptMessageRole {
  return value === 'system' || value === 'user'
}

function isPresetModuleEntry(value: unknown) {
  if (!isObject(value)) {
    return false
  }

  if (
    typeof value.entry_id !== 'string' ||
    typeof value.display_name !== 'string' ||
    typeof value.enabled !== 'boolean' ||
    typeof value.order !== 'number' ||
    typeof value.required !== 'boolean' ||
    !isPromptEntryKind(value.kind)
  ) {
    return false
  }

  if (
    'text' in value &&
    value.text !== undefined &&
    value.text !== null &&
    typeof value.text !== 'string'
  ) {
    return false
  }

  if (
    'context_key' in value &&
    value.context_key !== undefined &&
    value.context_key !== null &&
    typeof value.context_key !== 'string'
  ) {
    return false
  }

  return true
}

function isPresetModule(value: unknown) {
  if (!isObject(value)) {
    return false
  }

  return (
    typeof value.display_name === 'string' &&
    typeof value.module_id === 'string' &&
    typeof value.order === 'number' &&
    isPromptMessageRole(value.message_role) &&
    Array.isArray(value.entries) &&
    value.entries.every(isPresetModuleEntry)
  )
}

function isAgentPresetConfig(value: unknown) {
  if (!isObject(value)) {
    return false
  }

  for (const key of optionalNumericPresetConfigKeys) {
    if (
      key in value &&
      value[key] !== undefined &&
      value[key] !== null &&
      typeof value[key] !== 'number'
    ) {
      return false
    }
  }

  if (
    !('modules' in value) ||
    !Array.isArray(value.modules) ||
    !value.modules.every(isPresetModule)
  ) {
    return false
  }

  return true
}

function isPresetAgents(value: unknown): value is PresetDetail['agents'] {
  if (!isObject(value)) {
    return false
  }

  return agentRoleKeys.every((roleKey: AgentRoleKey) => isAgentPresetConfig(value[roleKey]))
}

function isPreset(value: unknown): value is PresetDetail {
  if (!isObject(value)) {
    return false
  }

  return (
    typeof value.preset_id === 'string' &&
    typeof value.display_name === 'string' &&
    isPresetAgents(value.agents)
  )
}

export function createPresetBundle(presets: ReadonlyArray<PresetDetail>): PresetBundle {
  return {
    presets: [...presets],
    type: 'preset_bundle',
    version: 2,
  }
}

export function isPresetBundle(value: unknown): value is PresetBundle {
  if (!isObject(value) || value.type !== 'preset_bundle' || value.version !== 2) {
    return false
  }

  return Array.isArray(value.presets) && value.presets.every(isPreset)
}
