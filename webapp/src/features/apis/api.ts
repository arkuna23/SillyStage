import { rpcRequest } from '../../lib/rpc'
import type {
  ApiConfig,
  ApiConfigInput,
  ApiDeletedResult,
  ApiGroup,
  ApiGroupBindings,
  ApiGroupDeletedResult,
  ApiGroupsListedResult,
  ApisListedResult,
  GlobalConfigResult,
  Preset,
  PresetAgentConfigs,
  PresetDeletedResult,
  PresetsListedResult,
} from './types'

export async function listApis(signal?: AbortSignal) {
  const result = await rpcRequest<Record<string, never>, ApisListedResult>(
    'api.list',
    {},
    { signal },
  )

  return result.apis
}

export async function getApi(apiId: string, signal?: AbortSignal) {
  return rpcRequest<{ api_id: string }, ApiConfig>('api.get', { api_id: apiId }, { signal })
}

export async function createApi(
  params: {
    api_id: string
    display_name: string
  } & ApiConfigInput,
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, ApiConfig>('api.create', params, { signal })
}

export async function updateApi(
  params: {
    api_id: string
    display_name?: string
  } & Partial<ApiConfigInput>,
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, ApiConfig>('api.update', params, { signal })
}

export async function deleteApi(apiId: string, signal?: AbortSignal) {
  return rpcRequest<{ api_id: string }, ApiDeletedResult>('api.delete', { api_id: apiId }, { signal })
}

export async function listApiGroups(signal?: AbortSignal) {
  const result = await rpcRequest<Record<string, never>, ApiGroupsListedResult>(
    'api_group.list',
    {},
    { signal },
  )

  return result.api_groups
}

export async function getApiGroup(apiGroupId: string, signal?: AbortSignal) {
  return rpcRequest<{ api_group_id: string }, ApiGroup>(
    'api_group.get',
    { api_group_id: apiGroupId },
    { signal },
  )
}

export async function createApiGroup(
  params: {
    api_group_id: string
    bindings: ApiGroupBindings
    display_name: string
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, ApiGroup>('api_group.create', params, { signal })
}

export async function updateApiGroup(
  params: {
    api_group_id: string
    bindings?: ApiGroupBindings
    display_name?: string
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, ApiGroup>('api_group.update', params, { signal })
}

export async function deleteApiGroup(apiGroupId: string, signal?: AbortSignal) {
  return rpcRequest<{ api_group_id: string }, ApiGroupDeletedResult>(
    'api_group.delete',
    { api_group_id: apiGroupId },
    { signal },
  )
}

export async function listPresets(signal?: AbortSignal) {
  const result = await rpcRequest<Record<string, never>, PresetsListedResult>(
    'preset.list',
    {},
    { signal },
  )

  return result.presets
}

export async function getPreset(presetId: string, signal?: AbortSignal) {
  return rpcRequest<{ preset_id: string }, Preset>(
    'preset.get',
    { preset_id: presetId },
    { signal },
  )
}

export async function createPreset(
  params: {
    agents: PresetAgentConfigs
    display_name: string
    preset_id: string
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, Preset>('preset.create', params, { signal })
}

export async function updatePreset(
  params: {
    agents?: PresetAgentConfigs
    display_name?: string
    preset_id: string
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, Preset>('preset.update', params, { signal })
}

export async function deletePreset(presetId: string, signal?: AbortSignal) {
  return rpcRequest<{ preset_id: string }, PresetDeletedResult>(
    'preset.delete',
    { preset_id: presetId },
    { signal },
  )
}

export async function getGlobalConfig(signal?: AbortSignal) {
  return rpcRequest<Record<string, never>, GlobalConfigResult>('config.get_global', {}, { signal })
}
