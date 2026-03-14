export type StoryResource = {
  character_ids: string[]
  planned_story: string | null
  player_schema_id_seed: string | null
  resource_id: string
  story_concept: string
  type: 'story_resources'
  world_schema_id_seed: string | null
}

export type StoryResourcesListedResult = {
  resources: StoryResource[]
  type: 'story_resources_listed'
}

export type StoryResourcesDeletedResult = {
  resource_id: string
  type: 'story_resources_deleted'
}

export type StoryPlannedResult = {
  resource_id: string
  story_script: string
  type: 'story_planned'
}
