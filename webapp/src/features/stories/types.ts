export type StoryGraphTransition = {
  to: string
}

export type StoryGraphNode = {
  characters: string[]
  goal: string
  id: string
  scene: string
  title: string
  transitions: StoryGraphTransition[]
}

export type StoryGraph = {
  nodes: StoryGraphNode[]
  start_node: string
}

export type StorySummary = {
  display_name: string
  introduction: string
  player_schema_id: string
  resource_id: string
  story_id: string
  world_schema_id: string
}

type StoryRecord = StorySummary & {
  graph: StoryGraph
}

export type StoryDetail = StoryRecord & {
  type: 'story'
}

export type StoryGeneratedResult = StoryRecord & {
  type: 'story_generated'
}

export type StoryDraftStatus = 'building' | 'finalized' | 'ready_to_finalize'

export type StoryDraftSummary = {
  created_at_ms?: number | null
  display_name: string
  draft_id: string
  final_story_id?: string | null
  next_section_index: number
  partial_node_count: number
  resource_id: string
  status: StoryDraftStatus
  total_sections: number
  updated_at_ms?: number | null
}

export type StoryDraftDetail = {
  created_at_ms?: number | null
  display_name: string
  draft_id: string
  final_story_id?: string | null
  introduction: string
  next_section_index: number
  outline_sections: string[]
  partial_graph: StoryGraph
  planned_story: string
  player_schema_id: string
  resource_id: string
  section_summaries: string[]
  status: StoryDraftStatus
  total_sections: number
  updated_at_ms?: number | null
  type: 'story_draft'
  world_schema_id: string
}

export type StoriesListedResult = {
  stories: StorySummary[]
  type: 'stories_listed'
}

export type StoryDeletedResult = {
  story_id: string
  type: 'story_deleted'
}

export type StoryDraftsListedResult = {
  drafts: StoryDraftSummary[]
  type: 'story_drafts_listed'
}

export type StoryDraftDeletedResult = {
  draft_id: string
  type: 'story_draft_deleted'
}
