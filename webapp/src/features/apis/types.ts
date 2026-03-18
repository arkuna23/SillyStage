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

export const promptModuleIds = [
  'role',
  'task',
  'static_context',
  'dynamic_context',
  'output',
] as const

export const presetEntryKinds = [
  'built_in_text',
  'built_in_context_ref',
  'custom_text',
] as const

export type LlmProvider = (typeof llmProviders)[number]
export type AgentRoleKey = (typeof agentRoleKeys)[number]
export type AgentRoleRecord<T> = Record<AgentRoleKey, T>
export type AgentBindingKey = `${AgentRoleKey}_api_id`
export type PromptModuleId = (typeof promptModuleIds)[number]
export type PresetEntryKind = (typeof presetEntryKinds)[number]

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

export type PresetModuleEntryBase = {
  display_name: string
  enabled: boolean
  entry_id: string
  kind: PresetEntryKind
  order: number
  required: boolean
}

export type PresetModuleEntry = PresetModuleEntryBase & {
  context_key?: string | null
  text?: string | null
}

export type PresetModuleEntrySummary = PresetModuleEntryBase

export type PresetPromptModule = {
  entries: PresetModuleEntry[]
  module_id: PromptModuleId
}

export type PresetPromptModuleSummary = {
  entries: PresetModuleEntrySummary[]
  entry_count: number
  module_id: PromptModuleId
}

export type AgentPresetConfig = {
  extra?: unknown | null
  max_tokens?: number | null
  modules: PresetPromptModule[]
  temperature?: number | null
}

export type AgentPresetConfigSummary = {
  entry_count?: number | null
  extra?: unknown | null
  max_tokens?: number | null
  module_count?: number | null
  modules: PresetPromptModuleSummary[]
  temperature?: number | null
}

export type PresetAgentConfigs = AgentRoleRecord<AgentPresetConfig>
export type PresetAgentSummaryConfigs = AgentRoleRecord<AgentPresetConfigSummary>

export type Preset = {
  agents: PresetAgentSummaryConfigs
  display_name: string
  preset_id: string
}

export type PresetDetail = {
  agents: PresetAgentConfigs
  display_name: string
  preset_id: string
}

export type PresetsListedResult = {
  presets: Preset[]
  type: 'presets_listed'
}

export type PresetDeletedResult = {
  preset_id: string
  type: 'preset_deleted'
}

export type PresetEntryMutationResult = {
  agent: AgentRoleKey
  entry: PresetModuleEntry
  module_id: PromptModuleId
  preset_id: string
  type: 'preset_entry'
}

export type PresetEntryDeletedResult = {
  agent: AgentRoleKey
  entry_id: string
  module_id: PromptModuleId
  preset_id: string
  type: 'preset_entry_deleted'
}

export type GlobalConfigResult = {
  api_group_id?: string | null
  preset_id?: string | null
  type: 'global_config'
}

export type AnyPresetModuleEntry = PresetModuleEntry | PresetModuleEntrySummary
export type AnyPresetPromptModule = PresetPromptModule | PresetPromptModuleSummary
export type AnyAgentPresetConfig = AgentPresetConfig | AgentPresetConfigSummary

export function getPresetModules(agent: AnyAgentPresetConfig) {
  return agent.modules ?? []
}

export function getPresetModuleCount(agent: AnyAgentPresetConfig) {
  return typeof (agent as AgentPresetConfigSummary).module_count === 'number'
    ? (agent as AgentPresetConfigSummary).module_count ?? 0
    : getPresetModules(agent).length
}

export function getPresetModuleEntryCount(agent: AnyAgentPresetConfig) {
  return typeof (agent as AgentPresetConfigSummary).entry_count === 'number'
    ? (agent as AgentPresetConfigSummary).entry_count ?? 0
    : getPresetModules(agent).reduce((count, module) => count + module.entries.length, 0)
}

export function getEnabledPresetModuleEntryCount(agent: AnyAgentPresetConfig) {
  return getPresetModules(agent).reduce(
    (count, module) => count + module.entries.filter((entry) => entry.enabled).length,
    0,
  )
}

export function hasPresetAgentConfiguration(agent: AnyAgentPresetConfig) {
  return (
    agent.temperature !== undefined &&
    agent.temperature !== null
  ) || (
    agent.max_tokens !== undefined &&
    agent.max_tokens !== null
  ) || (
    agent.extra !== undefined &&
    agent.extra !== null
  ) || getPresetModuleEntryCount(agent) > 0
}

export function getAgentBindingKey(roleKey: AgentRoleKey): AgentBindingKey {
  return `${roleKey}_api_id` as AgentBindingKey
}
