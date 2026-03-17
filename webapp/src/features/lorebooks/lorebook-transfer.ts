import type { Lorebook } from './types'

export type LorebookBundle = {
  lorebooks: Lorebook[]
  type: 'lorebook_bundle'
  version: 1
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === 'string')
}

function isLorebookEntry(value: unknown) {
  if (!isObject(value)) {
    return false
  }

  return (
    typeof value.entry_id === 'string' &&
    typeof value.title === 'string' &&
    typeof value.content === 'string' &&
    typeof value.enabled === 'boolean' &&
    typeof value.always_include === 'boolean' &&
    isStringArray(value.keywords)
  )
}

function isLorebook(value: unknown): value is Lorebook {
  if (!isObject(value) || value.type !== 'lorebook') {
    return false
  }

  return (
    typeof value.lorebook_id === 'string' &&
    typeof value.display_name === 'string' &&
    Array.isArray(value.entries) &&
    value.entries.every(isLorebookEntry)
  )
}

export function createLorebookBundle(lorebooks: ReadonlyArray<Lorebook>): LorebookBundle {
  return {
    lorebooks: [...lorebooks],
    type: 'lorebook_bundle',
    version: 1,
  }
}

export function isLorebookBundle(value: unknown): value is LorebookBundle {
  if (!isObject(value) || value.type !== 'lorebook_bundle' || value.version !== 1) {
    return false
  }

  return Array.isArray(value.lorebooks) && value.lorebooks.every(isLorebook)
}
