import { useEffect, useMemo, useState } from 'react'

import type { SelectOption } from '../../components/ui/select'
import type { StateFieldSchema } from '../../lib/state-schema'
import { getCharacter } from '../characters/api'
import { getSchema } from '../schemas/api'

type StoryCommonVariableKeySourceStatus = 'error' | 'loading' | 'missing' | 'ready'

export type StoryCommonVariableKeySource = {
  items: SelectOption[]
  status: StoryCommonVariableKeySourceStatus
}

export type StoryCommonVariableSchemaCatalog = {
  characterByCharacterId: Record<string, StoryCommonVariableKeySource>
  player: StoryCommonVariableKeySource
  world: StoryCommonVariableKeySource
}

type UseStoryCommonVariableSchemaCatalogArgs = {
  characterIds: ReadonlyArray<string>
  enabled: boolean
  playerSchemaId?: string | null
  worldSchemaId?: string | null
}

function createKeySource(
  status: StoryCommonVariableKeySourceStatus,
  items: SelectOption[] = [],
): StoryCommonVariableKeySource {
  return { items, status }
}

function normalizeIdList(ids: ReadonlyArray<string>) {
  return Array.from(new Set(ids.map((id) => id.trim()).filter((id) => id.length > 0))).sort()
}

function buildFieldOptions(fields: Record<string, StateFieldSchema>): SelectOption[] {
  return Object.entries(fields)
    .sort(([leftKey], [rightKey]) => leftKey.localeCompare(rightKey))
    .map(([fieldKey, fieldSchema]) => ({
      label: fieldSchema.description?.trim()
        ? `${fieldKey} · ${fieldSchema.description.trim()}`
        : fieldKey,
      value: fieldKey,
    }))
}

function createInitialCatalog(args: {
  characterIds: ReadonlyArray<string>
  playerSchemaId?: string | null
  worldSchemaId?: string | null
}): StoryCommonVariableSchemaCatalog {
  return {
    characterByCharacterId: Object.fromEntries(
      args.characterIds.map((characterId) => [characterId, createKeySource('loading')]),
    ),
    player: createKeySource(args.playerSchemaId?.trim() ? 'loading' : 'missing'),
    world: createKeySource(args.worldSchemaId?.trim() ? 'loading' : 'missing'),
  }
}

export function useStoryCommonVariableSchemaCatalog({
  characterIds,
  enabled,
  playerSchemaId,
  worldSchemaId,
}: UseStoryCommonVariableSchemaCatalogArgs): StoryCommonVariableSchemaCatalog {
  const normalizedCharacterIds = useMemo(() => normalizeIdList(characterIds), [characterIds])
  const normalizedPlayerSchemaId = playerSchemaId?.trim() ?? ''
  const normalizedWorldSchemaId = worldSchemaId?.trim() ?? ''
  const requestKey = `${enabled ? '1' : '0'}:${normalizedPlayerSchemaId}:${normalizedWorldSchemaId}:${normalizedCharacterIds.join('|')}`
  const fallbackCatalog = useMemo(
    () =>
      enabled
        ? createInitialCatalog({
            characterIds: normalizedCharacterIds,
            playerSchemaId: normalizedPlayerSchemaId,
            worldSchemaId: normalizedWorldSchemaId,
          })
        : createInitialCatalog({
            characterIds: [],
            playerSchemaId: '',
            worldSchemaId: '',
          }),
    [enabled, normalizedCharacterIds, normalizedPlayerSchemaId, normalizedWorldSchemaId],
  )
  const [resolvedCatalogState, setResolvedCatalogState] = useState<{
    catalog: StoryCommonVariableSchemaCatalog
    key: string
  } | null>(null)

  useEffect(() => {
    if (!enabled) {
      return
    }

    const controller = new AbortController()

    async function load() {
      const characterDetailResults = await Promise.all(
        normalizedCharacterIds.map(async (characterId) => {
          try {
            const character = await getCharacter(characterId, controller.signal)

            return {
              character,
              characterId,
              ok: true as const,
            }
          } catch {
            return {
              characterId,
              ok: false as const,
            }
          }
        }),
      )

      if (controller.signal.aborted) {
        return
      }

      const characterSchemaIdByCharacterId = new Map<string, string>()
      const characterFetchErrorIds = new Set<string>()

      characterDetailResults.forEach((result) => {
        if (!result.ok) {
          characterFetchErrorIds.add(result.characterId)
          return
        }

        const schemaId = result.character.content.schema_id.trim()

        if (schemaId.length > 0) {
          characterSchemaIdByCharacterId.set(result.characterId, schemaId)
        }
      })

      const schemaIds = normalizeIdList([
        normalizedPlayerSchemaId,
        normalizedWorldSchemaId,
        ...Array.from(characterSchemaIdByCharacterId.values()),
      ])
      const schemaResults = await Promise.all(
        schemaIds.map(async (schemaId) => {
          try {
            const schema = await getSchema(schemaId, controller.signal)

            return {
              ok: true as const,
              schema,
              schemaId,
            }
          } catch {
            return {
              ok: false as const,
              schemaId,
            }
          }
        }),
      )

      if (controller.signal.aborted) {
        return
      }

      const schemasById = new Map<string, SelectOption[]>()
      const schemaErrorIds = new Set<string>()

      schemaResults.forEach((result) => {
        if (!result.ok) {
          schemaErrorIds.add(result.schemaId)
          return
        }

        schemasById.set(result.schemaId, buildFieldOptions(result.schema.fields))
      })

      const characterByCharacterId = Object.fromEntries(
        normalizedCharacterIds.map((characterId) => {
          if (characterFetchErrorIds.has(characterId)) {
            return [characterId, createKeySource('error')]
          }

          const schemaId = characterSchemaIdByCharacterId.get(characterId)

          if (!schemaId) {
            return [characterId, createKeySource('missing')]
          }

          if (schemaErrorIds.has(schemaId)) {
            return [characterId, createKeySource('error')]
          }

          return [characterId, createKeySource('ready', schemasById.get(schemaId) ?? [])]
        }),
      )

      setResolvedCatalogState({
        catalog: {
          characterByCharacterId,
          player: !normalizedPlayerSchemaId
            ? createKeySource('missing')
            : schemaErrorIds.has(normalizedPlayerSchemaId)
              ? createKeySource('error')
              : createKeySource('ready', schemasById.get(normalizedPlayerSchemaId) ?? []),
          world: !normalizedWorldSchemaId
            ? createKeySource('missing')
            : schemaErrorIds.has(normalizedWorldSchemaId)
              ? createKeySource('error')
              : createKeySource('ready', schemasById.get(normalizedWorldSchemaId) ?? []),
        },
        key: requestKey,
      })
    }

    void load()

    return () => {
      controller.abort()
    }
  }, [
    enabled,
    normalizedCharacterIds,
    normalizedPlayerSchemaId,
    normalizedWorldSchemaId,
    requestKey,
  ])

  return resolvedCatalogState?.key === requestKey ? resolvedCatalogState.catalog : fallbackCatalog
}
