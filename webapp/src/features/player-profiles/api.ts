import { rpcRequest } from '../../lib/rpc'
import type { PlayerProfile, PlayerProfileDeletedResult, PlayerProfilesListedResult } from './types'

export async function listPlayerProfiles(signal?: AbortSignal) {
  const result = await rpcRequest<Record<string, never>, PlayerProfilesListedResult>(
    'player_profile.list',
    {},
    { signal },
  )

  return result.player_profiles
}

export async function getPlayerProfile(playerProfileId: string, signal?: AbortSignal) {
  return rpcRequest<{ player_profile_id: string }, PlayerProfile>(
    'player_profile.get',
    { player_profile_id: playerProfileId },
    { signal },
  )
}

export async function createPlayerProfile(
  params: {
    description: string
    display_name: string
    player_profile_id: string
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, PlayerProfile>('player_profile.create', params, { signal })
}

export async function updatePlayerProfile(
  params: {
    description?: string
    display_name?: string
    player_profile_id: string
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, PlayerProfile>('player_profile.update', params, { signal })
}

export async function deletePlayerProfile(playerProfileId: string, signal?: AbortSignal) {
  return rpcRequest<{ player_profile_id: string }, PlayerProfileDeletedResult>(
    'player_profile.delete',
    { player_profile_id: playerProfileId },
    { signal },
  )
}
