import { rpcRequest } from '../../lib/rpc'
import type {
  AgentApiIdOverrides,
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
    api_key: string
    base_url: string
    model: string
    provider: LlmProvider
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
    model?: string
    provider?: LlmProvider
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
