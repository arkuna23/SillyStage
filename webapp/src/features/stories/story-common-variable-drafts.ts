import type { CommonVariableDefinition, CommonVariableScope } from './types'

let nextStoryCommonVariableDraftId = 0

function createStoryCommonVariableDraftId() {
  nextStoryCommonVariableDraftId += 1
  return `story-common-variable-${nextStoryCommonVariableDraftId}`
}

type StoryCommonVariableValidationCode =
  | 'characterInvalid'
  | 'characterRequired'
  | 'displayNameRequired'
  | 'keyRequired'

export type StoryCommonVariableDraft = {
  character_id: string
  display_name: string
  id: string
  key: string
  pinned: boolean
  scope: CommonVariableScope
}

export type StoryCommonVariableDraftErrors = Record<
  string,
  {
    character_id?: StoryCommonVariableValidationCode
    display_name?: StoryCommonVariableValidationCode
    key?: StoryCommonVariableValidationCode
  }
>

export function createStoryCommonVariableDraft(
  definition?: Partial<CommonVariableDefinition>,
): StoryCommonVariableDraft {
  return {
    character_id: definition?.character_id ?? '',
    display_name: definition?.display_name ?? '',
    id: createStoryCommonVariableDraftId(),
    key: definition?.key ?? '',
    pinned: definition?.pinned ?? true,
    scope: definition?.scope ?? 'world',
  }
}

export function createStoryCommonVariableDrafts(
  definitions: ReadonlyArray<CommonVariableDefinition> | null | undefined,
) {
  if (!definitions || definitions.length === 0) {
    return [] as StoryCommonVariableDraft[]
  }

  return definitions.map((definition) => createStoryCommonVariableDraft(definition))
}

export function validateStoryCommonVariableDrafts(
  drafts: ReadonlyArray<StoryCommonVariableDraft>,
  validCharacterIds: ReadonlySet<string>,
) {
  const errors: StoryCommonVariableDraftErrors = {}

  drafts.forEach((draft) => {
    const key = draft.key.trim()
    const displayName = draft.display_name.trim()
    const characterId = draft.character_id.trim()
    const draftErrors: StoryCommonVariableDraftErrors[string] = {}

    if (key.length === 0) {
      draftErrors.key = 'keyRequired'
    }

    if (displayName.length === 0) {
      draftErrors.display_name = 'displayNameRequired'
    }

    if (draft.scope === 'character') {
      if (characterId.length === 0) {
        draftErrors.character_id = 'characterRequired'
      } else if (!validCharacterIds.has(characterId)) {
        draftErrors.character_id = 'characterInvalid'
      }
    }

    if (Object.keys(draftErrors).length > 0) {
      errors[draft.id] = draftErrors
    }
  })

  return errors
}

export function serializeStoryCommonVariableDrafts(
  drafts: ReadonlyArray<StoryCommonVariableDraft>,
): CommonVariableDefinition[] {
  return drafts.map((draft) => {
    const normalizedDraft = {
      display_name: draft.display_name.trim(),
      key: draft.key.trim(),
      pinned: draft.pinned,
      scope: draft.scope,
    } satisfies Omit<CommonVariableDefinition, 'character_id'>

    if (draft.scope === 'character' && draft.character_id.trim().length > 0) {
      return {
        ...normalizedDraft,
        character_id: draft.character_id.trim(),
      }
    }

    return normalizedDraft
  })
}
