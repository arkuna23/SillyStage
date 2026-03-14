import { rpcRequest } from '../../lib/rpc'
import type {
  AgentApiIdOverrides,
  DefaultLlmConfigState,
  GlobalConfigResult,
  LlmApi,
  LlmApiDeletedResult,
  LlmApisListedResult,
  LlmProvider,
} from './types'

export async function listLlmApis(signal?: AbortSignal) {
  const result = await rpcRequest<Record<string, never>, LlmApisListedResult>(
    'llm_api.list',
    {},
    { signal },
  )

  return result.apis
}

export async function getLlmApi(apiId: string, signal?: AbortSignal) {
  return rpcRequest<{ api_id: string }, LlmApi>(
    'llm_api.get',
    { api_id: apiId },
    { signal },
  )
}

export async function createLlmApi(
  params: {
    api_id: string
    api_key?: string
    base_url?: string
    max_tokens?: number
    model?: string
    provider?: LlmProvider
    temperature?: number
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, LlmApi>('llm_api.create', params, { signal })
}

export async function updateLlmApi(
  params: {
    api_id: string
    api_key?: string
    base_url?: string
    max_tokens?: number
    model?: string
    provider?: LlmProvider
    temperature?: number
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, LlmApi>('llm_api.update', params, { signal })
}

export async function deleteLlmApi(apiId: string, signal?: AbortSignal) {
  return rpcRequest<{ api_id: string }, LlmApiDeletedResult>(
    'llm_api.delete',
    { api_id: apiId },
    { signal },
  )
}

export async function getGlobalApiConfig(signal?: AbortSignal) {
  return rpcRequest<Record<string, never>, GlobalConfigResult>('config.get_global', {}, { signal })
}

export async function updateGlobalApiConfig(
  apiOverrides: AgentApiIdOverrides,
  signal?: AbortSignal,
) {
  return rpcRequest<{ api_overrides: AgentApiIdOverrides }, GlobalConfigResult>(
    'config.update_global',
    { api_overrides: apiOverrides },
    { signal },
  )
}

export async function getDefaultLlmConfig(signal?: AbortSignal) {
  return rpcRequest<Record<string, never>, DefaultLlmConfigState>(
    'default_llm_config.get',
    {},
    { signal },
  )
}

export async function updateDefaultLlmConfig(
  params: {
    api_key: string
    base_url: string
    max_tokens?: number
    model: string
    provider: LlmProvider
    temperature?: number
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, DefaultLlmConfigState>(
    'default_llm_config.update',
    params,
    { signal },
  )
}
