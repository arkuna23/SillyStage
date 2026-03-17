import { listApiGroups, listApis, listPresets } from '../apis/api'

export const STAGE_API_AVAILABILITY_REFRESH_EVENT = 'sillystage:stage-api-availability-refresh'

export type StageAccessStatus = 'blockedApiResources' | 'blockedPresets' | 'ready'

export async function getStageAccessStatus(signal?: AbortSignal): Promise<StageAccessStatus> {
  const [apis, apiGroups, presets] = await Promise.all([
    listApis(signal),
    listApiGroups(signal),
    listPresets(signal),
  ])

  if (apis.length === 0 || apiGroups.length === 0) {
    return 'blockedApiResources'
  }

  if (presets.length === 0) {
    return 'blockedPresets'
  }

  return 'ready'
}

export function dispatchStageApiAvailabilityRefresh() {
  if (typeof window === 'undefined') {
    return
  }

  window.dispatchEvent(new Event(STAGE_API_AVAILABILITY_REFRESH_EVENT))
}
