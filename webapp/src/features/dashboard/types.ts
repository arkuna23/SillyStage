export type DashboardHealthStatus = 'ok'

export type DashboardHealth = {
  status: DashboardHealthStatus
}

export type DashboardCounts = {
  characters_total: number
  characters_with_cover: number
  sessions_total: number
  stories_total: number
  story_resources_total: number
}

export type DashboardStorySummary = {
  display_name: string
  introduction: string
  resource_id: string
  story_id: string
  updated_at_ms?: number | null
}

export type DashboardSessionSummary = {
  display_name: string
  session_id: string
  story_id: string
  turn_index: number
  updated_at_ms?: number | null
}

export type DashboardPayload = {
  counts: DashboardCounts
  global_config: {
    api_group_id?: string | null
    preset_id?: string | null
  }
  health: DashboardHealth
  recent_sessions: DashboardSessionSummary[]
  recent_stories: DashboardStorySummary[]
}
