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

export type ResourceFileRef = {
  file_id: string
  resource_id: string
}

export type ResourceFile = ResourceFileRef & {
  content_type: string
  file_name: string | null
  size_bytes: number
}

export type DataPackageResourceSummary = {
  count: number
  ids: string[]
}

export type DataPackageContents = {
  characters: DataPackageResourceSummary
  lorebooks: DataPackageResourceSummary
  player_profiles: DataPackageResourceSummary
  presets: DataPackageResourceSummary
  schemas: DataPackageResourceSummary
  stories: DataPackageResourceSummary
  story_resources: DataPackageResourceSummary
}

export type DataPackageExportPrepareParams = {
  character_ids?: string[]
  include_dependencies?: boolean
  lorebook_ids?: string[]
  player_profile_ids?: string[]
  preset_ids?: string[]
  schema_ids?: string[]
  story_ids?: string[]
  story_resource_ids?: string[]
}

export type DataPackageExportPreparedResult = {
  archive: ResourceFileRef
  contents: DataPackageContents
  export_id: string
  type: 'data_package_export_prepared'
}

export type DataPackageImportPreparedResult = {
  archive: ResourceFileRef
  import_id: string
  type: 'data_package_import_prepared'
}

export type DataPackageImportCommittedResult = {
  contents: DataPackageContents
  import_id: string
  type: 'data_package_import_committed'
}
