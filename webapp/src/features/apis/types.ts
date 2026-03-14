export const llmProviders = ['open_ai'] as const

export const agentApiRoleKeys = [
  'planner_api_id',
  'architect_api_id',
  'director_api_id',
  'actor_api_id',
  'narrator_api_id',
  'keeper_api_id',
  'replyer_api_id',
] as const

export type LlmProvider = (typeof llmProviders)[number]
export type AgentApiRoleKey = (typeof agentApiRoleKeys)[number]

export type AgentApiIds = Record<AgentApiRoleKey, string>
export type AgentApiIdOverrides = Partial<AgentApiIds>

export type LlmApi = {
  api_id: string
  api_key_masked?: string | null
  base_url: string
  has_api_key: boolean
  max_tokens?: number | null
  model: string
  provider: LlmProvider
  temperature?: number | null
  type: 'llm_api'
}

export type LlmApisListedResult = {
  apis: LlmApi[]
  type: 'llm_apis_listed'
}

export type LlmApiDeletedResult = {
  api_id: string
  type: 'llm_api_deleted'
}

export type DefaultLlmConfigPayload = {
  api_key_masked?: string | null
  base_url: string
  has_api_key: boolean
  max_tokens?: number | null
  model: string
  provider: LlmProvider
  temperature?: number | null
}

export type DefaultLlmConfigState = {
  effective?: DefaultLlmConfigPayload | null
  saved?: DefaultLlmConfigPayload | null
  type: 'default_llm_config'
}

export type GlobalConfigResult = {
  api_ids: AgentApiIds | null
  type: 'global_config'
}
