import { rpcRequest } from '../../lib/rpc'
import type { DashboardPayload } from './types'

export async function getDashboard(signal?: AbortSignal) {
  return rpcRequest<Record<string, never>, DashboardPayload>('dashboard.get', {}, { signal })
}
