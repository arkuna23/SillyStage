import { listApis } from '../apis/api'

export const STAGE_API_AVAILABILITY_REFRESH_EVENT = 'sillystage:stage-api-availability-refresh'

export async function hasConfiguredStageApis(signal?: AbortSignal) {
  const apis = await listApis(signal)

  return apis.length > 0
}

export function dispatchStageApiAvailabilityRefresh() {
  if (typeof window === 'undefined') {
    return
  }

  window.dispatchEvent(new Event(STAGE_API_AVAILABILITY_REFRESH_EVENT))
}
