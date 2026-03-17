export const llmProviders = ['open_ai'] as const

export const agentRoleKeys = [
  'planner',
  'architect',
  'director',
  'actor',
  'narrator',
  'keeper',
  'replyer',
] as const

export type LlmProvider = (typeof llmProviders)[number]
export type AgentRoleKey = (typeof agentRoleKeys)[number]
export type AgentRoleRecord<T> = Record<AgentRoleKey, T>
export type AgentBindingKey = `${AgentRoleKey}_api_id`

export type ApiConfigInput = {
  api_key: string
  base_url: string
  model: string
  provider: LlmProvider
}

export type ApiConfig = {
  api_id: string
  api_key_masked?: string | null
  base_url: string
  display_name: string
  has_api_key: boolean
  model: string
  provider: LlmProvider
  type: 'api'
}

export type ApisListedResult = {
  apis: ApiConfig[]
  type: 'apis_listed'
}

export type ApiModelsListedResult = {
  base_url: string
  models: string[]
  provider: LlmProvider
  type: 'api_models_listed'
}

export type ApiDeletedResult = {
  api_id: string
  type: 'api_deleted'
}

export type ApiGroupBindings = Record<AgentBindingKey, string>

export type ApiGroup = {
  api_group_id: string
  bindings: ApiGroupBindings
  display_name: string
  type: 'api_group'
}

export type ApiGroupsListedResult = {
  api_groups: ApiGroup[]
  type: 'api_groups_listed'
}

export type ApiGroupDeletedResult = {
  api_group_id: string
  type: 'api_group_deleted'
}

export type AgentPresetConfig = {
  extra?: unknown | null
  max_tokens?: number | null
  prompt_entries?: PresetPromptEntry[] | null
  prompt_entry_count?: number | null
  temperature?: number | null
}

export type PresetPromptEntry = {
  content?: string
  enabled: boolean
  entry_id: string
  title: string
}

export type PresetAgentConfigs = AgentRoleRecord<AgentPresetConfig>

export type Preset = {
  agents: PresetAgentConfigs
  display_name: string
  preset_id: string
  type: 'preset'
}

export type PresetsListedResult = {
  presets: Preset[]
  type: 'presets_listed'
}

export type PresetDeletedResult = {
  preset_id: string
  type: 'preset_deleted'
}

export type GlobalConfigResult = {
  api_group_id?: string | null
  preset_id?: string | null
  type: 'global_config'
}

export function getPresetPromptEntries(agent: AgentPresetConfig) {
  return agent.prompt_entries ?? []
}

export function getPresetPromptEntryCount(agent: AgentPresetConfig) {
  return typeof agent.prompt_entry_count === 'number'
    ? agent.prompt_entry_count
    : getPresetPromptEntries(agent).length
}

export function getEnabledPresetPromptEntryCount(agent: AgentPresetConfig) {
  return getPresetPromptEntries(agent).filter((entry) => entry.enabled).length
}

export function hasPresetAgentConfiguration(agent: AgentPresetConfig) {
  return (
    agent.temperature !== undefined &&
    agent.temperature !== null
  ) || (
    agent.max_tokens !== undefined &&
    agent.max_tokens !== null
  ) || (
    agent.extra !== undefined &&
    agent.extra !== null
  ) || getPresetPromptEntryCount(agent) > 0
}

export function getAgentBindingKey(roleKey: AgentRoleKey): AgentBindingKey {
  return `${roleKey}_api_id` as AgentBindingKey
}
