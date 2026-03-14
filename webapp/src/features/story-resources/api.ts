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

export async function generateStoryPlan(resourceId: string, signal?: AbortSignal) {
  return rpcRequest<{ resource_id: string }, StoryPlannedResult>(
    'story.generate_plan',
    { resource_id: resourceId },
    { signal },
  )
}

export async function generateAndSaveStoryPlan(resourceId: string, signal?: AbortSignal) {
  const planned = await generateStoryPlan(resourceId, signal)
  const resource = await updateStoryResource(
    {
      planned_story: planned.story_script,
      resource_id: resourceId,
    },
    signal,
  )

  return {
    planned,
    resource,
  }
}
