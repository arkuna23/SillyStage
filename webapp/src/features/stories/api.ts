import { rpcRequest } from '../../lib/rpc'

import type {
  StoryDraftDeletedResult,
  StoryDraftDetail,
  StoryDraftsListedResult,
  StoryDeletedResult,
  StoryDetail,
  StoryGeneratedResult,
  StoriesListedResult,
} from './types'

export async function listStories(signal?: AbortSignal) {
  const result = await rpcRequest<Record<string, never>, StoriesListedResult>(
    'story.list',
    {},
    { signal },
  )

  return result.stories
}

export async function getStory(storyId: string, signal?: AbortSignal) {
  return rpcRequest<{ story_id: string }, StoryDetail>(
    'story.get',
    { story_id: storyId },
    { signal },
  )
}

export async function generateStory(
  params: {
    display_name?: string
    resource_id: string
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, StoryGeneratedResult>('story.generate', params, { signal })
}

export async function listStoryDrafts(signal?: AbortSignal) {
  const result = await rpcRequest<Record<string, never>, StoryDraftsListedResult>(
    'story_draft.list',
    {},
    { signal },
  )

  return result.drafts
}

export async function getStoryDraft(draftId: string, signal?: AbortSignal) {
  return rpcRequest<{ draft_id: string }, StoryDraftDetail>(
    'story_draft.get',
    { draft_id: draftId },
    { signal },
  )
}

export async function startStoryDraft(
  params: {
    display_name?: string
    resource_id: string
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, StoryDraftDetail>('story_draft.start', params, { signal })
}

export async function continueStoryDraft(
  params: {
    draft_id: string
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, StoryDraftDetail>('story_draft.continue', params, { signal })
}

export async function finalizeStoryDraft(
  params: {
    draft_id: string
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, StoryGeneratedResult>('story_draft.finalize', params, { signal })
}

export async function deleteStoryDraft(draftId: string, signal?: AbortSignal) {
  return rpcRequest<{ draft_id: string }, StoryDraftDeletedResult>(
    'story_draft.delete',
    { draft_id: draftId },
    { signal },
  )
}

export async function updateStory(
  params: {
    display_name: string
    story_id: string
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, StoryDetail>('story.update', params, { signal })
}

export async function deleteStory(storyId: string, signal?: AbortSignal) {
  return rpcRequest<{ story_id: string }, StoryDeletedResult>(
    'story.delete',
    { story_id: storyId },
    { signal },
  )
}
