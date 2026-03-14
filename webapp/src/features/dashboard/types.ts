import type { AgentApiIds } from '../apis/types'

export type DashboardHealthStatus = 'ok'

export type DashboardHealth = {
  status: DashboardHealthStatus
}

export type DashboardCounts = {
  characters_total: number
  characters_with_cover: number
  story_resources_total: number
  stories_total: number
  sessions_total: number
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
    api_ids: AgentApiIds | null
  }
  health: DashboardHealth
  recent_sessions: DashboardSessionSummary[]
  recent_stories: DashboardStorySummary[]
}
