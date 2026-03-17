import { rpcRequest } from '../../lib/rpc'
import type {
  StoryPlannedResult,
  StoryResource,
  StoryResourcesDeletedResult,
  StoryResourcesListedResult,
} from './types'

export async function listStoryResources(signal?: AbortSignal) {
  const result = await rpcRequest<Record<string, never>, StoryResourcesListedResult>(
    'story_resources.list',
    {},
    { signal },
  )

  return result.resources
}

export async function getStoryResource(resourceId: string, signal?: AbortSignal) {
  return rpcRequest<{ resource_id: string }, StoryResource>(
    'story_resources.get',
    { resource_id: resourceId },
    { signal },
  )
}

export async function createStoryResource(
  params: {
    character_ids: string[]
    lorebook_ids?: string[]
    planned_story?: string
    player_schema_id_seed?: string
    story_concept: string
    world_schema_id_seed?: string
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, StoryResource>('story_resources.create', params, { signal })
}

export async function updateStoryResource(
  params: {
    character_ids?: string[]
    lorebook_ids?: string[]
    planned_story?: string
    player_schema_id_seed?: string
    resource_id: string
    story_concept?: string
    world_schema_id_seed?: string
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, StoryResource>('story_resources.update', params, { signal })
}

export async function deleteStoryResource(resourceId: string, signal?: AbortSignal) {
  return rpcRequest<{ resource_id: string }, StoryResourcesDeletedResult>(
    'story_resources.delete',
    { resource_id: resourceId },
    { signal },
  )
}

export async function generateStoryPlan(
  params: {
    api_group_id: string
    preset_id: string
    resource_id: string
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, StoryPlannedResult>(
    'story.generate_plan',
    params,
    { signal },
  )
}

export async function generateAndSaveStoryPlan(
  params: {
    apiGroupId: string
    presetId: string
    resourceId: string
  },
  signal?: AbortSignal,
) {
  const planned = await generateStoryPlan(
    {
      api_group_id: params.apiGroupId,
      preset_id: params.presetId,
      resource_id: params.resourceId,
    },
    signal,
  )
  const resource = await updateStoryResource(
    {
      planned_story: planned.story_script,
      resource_id: params.resourceId,
    },
    signal,
  )

  return {
    planned,
    resource,
  }
}
