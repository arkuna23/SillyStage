export type StoryGraphTransition = {
  condition?: StoryGraphCondition | null
  to: string
}

export type ConditionScope = 'character' | 'global' | 'player'

export type ConditionOperator = 'contains' | 'eq' | 'gte' | 'gt' | 'lte' | 'lt' | 'ne'

export type StoryGraphCondition = {
  character?: string | null
  key: string
  op: ConditionOperator
  scope?: ConditionScope
  value: unknown
}

export type StoryGraphStateOpType =
  | 'AddActiveCharacter'
  | 'RemoveActiveCharacter'
  | 'RemoveCharacterState'
  | 'RemovePlayerState'
  | 'RemoveState'
  | 'SetActiveCharacters'
  | 'SetCharacterState'
  | 'SetCurrentNode'
  | 'SetPlayerState'
  | 'SetState'

export type StoryGraphStateOp =
  | {
      key: string
      type: 'RemovePlayerState' | 'RemoveState'
    }
  | {
      character: string
      key: string
      type: 'RemoveCharacterState'
    }
  | {
      characters: string[]
      type: 'SetActiveCharacters'
    }
  | {
      character: string
      type: 'AddActiveCharacter' | 'RemoveActiveCharacter'
    }
  | {
      key: string
      type: 'SetPlayerState' | 'SetState'
      value: unknown
    }
  | {
      character: string
      key: string
      type: 'SetCharacterState'
      value: unknown
    }
  | {
      node_id: string
      type: 'SetCurrentNode'
    }

export type StoryGraphNode = {
  characters: string[]
  goal: string
  id: string
  on_enter_updates?: StoryGraphStateOp[]
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
  api_group_id?: string | null
  created_at_ms?: number | null
  display_name: string
  draft_id: string
  final_story_id?: string | null
  next_section_index: number
  partial_node_count: number
  preset_id?: string | null
  resource_id: string
  status: StoryDraftStatus
  total_sections: number
  updated_at_ms?: number | null
}

export type StoryDraftDetail = {
  api_group_id?: string | null
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
  preset_id?: string | null
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
