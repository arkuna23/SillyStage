import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { listApiGroups, listPresets } from '../apis/api'
import type { ApiGroup, Preset } from '../apis/types'
import { listCharacters } from '../characters/api'
import type { CharacterSummary } from '../characters/types'
import { listLorebooks } from '../lorebooks/api'
import type { Lorebook } from '../lorebooks/types'
import { listSchemas } from '../schemas/api'
import type { SchemaResource } from '../schemas/types'

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

type UseStoryResourceReferencesOptions = {
  onLoadError?: (message: string) => void
}

export function useStoryResourceReferences({
  onLoadError,
}: UseStoryResourceReferencesOptions = {}) {
  const { t } = useTranslation()
  const [availableCharacters, setAvailableCharacters] = useState<CharacterSummary[]>([])
  const [availableApiGroups, setAvailableApiGroups] = useState<ApiGroup[]>([])
  const [availableLorebooks, setAvailableLorebooks] = useState<Lorebook[]>([])
  const [availablePresets, setAvailablePresets] = useState<Preset[]>([])
  const [availableSchemas, setAvailableSchemas] = useState<SchemaResource[]>([])
  const [referencesLoading, setReferencesLoading] = useState(true)

  const refreshReferences = useCallback(
    async (signal?: AbortSignal) => {
      setReferencesLoading(true)

      try {
        const [apiGroups, presets, characters, lorebooks, schemas] = await Promise.all([
          listApiGroups(signal),
          listPresets(signal),
          listCharacters(signal),
          listLorebooks(signal),
          listSchemas(signal),
        ])

        if (!signal?.aborted) {
          setAvailableApiGroups(apiGroups)
          setAvailablePresets(presets)
          setAvailableCharacters(characters)
          setAvailableLorebooks(lorebooks)
          setAvailableSchemas(schemas)
        }
      } catch (error) {
        if (!signal?.aborted) {
          onLoadError?.(
            getErrorMessage(error, t('storyResources.feedback.loadReferencesFailed')),
          )
        }
      } finally {
        if (!signal?.aborted) {
          setReferencesLoading(false)
        }
      }
    },
    [onLoadError, t],
  )

  useEffect(() => {
    const controller = new AbortController()

    void refreshReferences(controller.signal)

    return () => {
      controller.abort()
    }
  }, [refreshReferences])

  return {
    availableApiGroups,
    availableCharacters,
    availableLorebooks,
    availablePresets,
    availableSchemas,
    referencesLoading,
    refreshReferences,
  }
}
