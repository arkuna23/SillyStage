export type PlayerProfile = {
  description: string
  display_name: string
  player_profile_id: string
  type: 'player_profile'
}

export type PlayerProfilesListedResult = {
  player_profiles: PlayerProfile[]
  type: 'player_profiles_listed'
}

export type PlayerProfileDeletedResult = {
  player_profile_id: string
  type: 'player_profile_deleted'
}
