import { rpcRequest } from '../../lib/rpc'
import type {
  Lorebook,
  LorebookDeletedResult,
  LorebookEntry,
  LorebookEntryDeletedResult,
  LorebooksListedResult,
} from './types'

export async function listLorebooks(signal?: AbortSignal) {
  const result = await rpcRequest<Record<string, never>, LorebooksListedResult>(
    'lorebook.list',
    {},
    { signal },
  )

  return result.lorebooks
}

export async function getLorebook(lorebookId: string, signal?: AbortSignal) {
  return rpcRequest<{ lorebook_id: string }, Lorebook>(
    'lorebook.get',
    { lorebook_id: lorebookId },
    { signal },
  )
}

export async function updateLorebook(
  params: {
    display_name?: string
    lorebook_id: string
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, Lorebook>('lorebook.update', params, { signal })
}

export async function createLorebook(
  params: {
    display_name: string
    entries?: LorebookEntry[]
    lorebook_id: string
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, Lorebook>('lorebook.create', params, { signal })
}

export async function deleteLorebook(lorebookId: string, signal?: AbortSignal) {
  return rpcRequest<{ lorebook_id: string }, LorebookDeletedResult>(
    'lorebook.delete',
    { lorebook_id: lorebookId },
    { signal },
  )
}

export async function createLorebookEntry(
  params: {
    always_include?: boolean
    content: string
    enabled?: boolean
    entry_id: string
    keywords?: string[]
    lorebook_id: string
    title: string
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, LorebookEntry>('lorebook_entry.create', params, { signal })
}

export async function updateLorebookEntry(
  params: {
    always_include?: boolean
    content?: string
    enabled?: boolean
    entry_id: string
    keywords?: string[]
    lorebook_id: string
    title?: string
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, LorebookEntry>('lorebook_entry.update', params, { signal })
}

export async function deleteLorebookEntry(
  params: {
    entry_id: string
    lorebook_id: string
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, LorebookEntryDeletedResult>('lorebook_entry.delete', params, {
    signal,
  })
}
