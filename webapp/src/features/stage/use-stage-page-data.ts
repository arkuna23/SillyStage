import { useCallback, useEffect, useMemo, useRef, useState } from 'react'

import { revokeObjectUrl } from '../../lib/binary-resource'
import { listApiGroups, listApis, listPresets } from '../apis/api'
import type { ApiConfig, ApiGroup, Preset } from '../apis/types'
import { getCharacterCoverUrl, listCharacters } from '../characters/api'
import type { CharacterSummary } from '../characters/types'
import { listPlayerProfiles } from '../player-profiles/api'
import type { PlayerProfile } from '../player-profiles/types'
import { getStory, listStories } from '../stories/api'
import type { StoryDetail, StorySummary } from '../stories/types'
import { getSession, listSessionCharacters, listSessions } from './api'
import type { StageCopy } from './copy'
import type { StageAccessStatus } from './stage-access'
import { buildStageCommonVariables } from './stage-common-variable-utils'
import {
  buildCharacterMap,
  getErrorMessage,
  getStoryNode,
  normalizeSessionHistory,
} from './stage-page-utils'
import type { CoverCache, Notice, StageCommonVariable } from './stage-ui-types'
import type { RuntimeSnapshot, SessionCharacter, SessionDetail, SessionSummary } from './types'

type StageAccessStatusState = StageAccessStatus | 'checking'

type UseStagePageDataArgs = {
  copy: StageCopy
  routeSessionId?: string
  setNotice: (notice: Notice | null) => void
}

export function useStagePageData({ copy, routeSessionId, setNotice }: UseStagePageDataArgs) {
  const coverCacheRef = useRef<CoverCache>({})
  const storyDetailsRef = useRef<Record<string, StoryDetail>>({})
  const [sessions, setSessions] = useState<SessionSummary[]>([])
  const [stories, setStories] = useState<StorySummary[]>([])
  const [characters, setCharacters] = useState<CharacterSummary[]>([])
  const [playerProfiles, setPlayerProfiles] = useState<PlayerProfile[]>([])
  const [apis, setApis] = useState<ApiConfig[]>([])
  const [apiGroups, setApiGroups] = useState<ApiGroup[]>([])
  const [presets, setPresets] = useState<Preset[]>([])
  const [storyDetails, setStoryDetails] = useState<Record<string, StoryDetail>>({})
  const [coverCache, setCoverCache] = useState<CoverCache>({})
  const [sessionCharacters, setSessionCharacters] = useState<SessionCharacter[]>([])
  const [selectedSession, setSelectedSession] = useState<SessionDetail | null>(null)
  const [liveSnapshot, setLiveSnapshot] = useState<RuntimeSnapshot | null>(null)
  const [isListLoading, setIsListLoading] = useState(true)
  const [isSessionLoading, setIsSessionLoading] = useState(false)
  const [stageAccessStatus, setStageAccessStatus] = useState<StageAccessStatusState>('checking')

  useEffect(() => {
    coverCacheRef.current = coverCache
  }, [coverCache])

  useEffect(() => {
    storyDetailsRef.current = storyDetails
  }, [storyDetails])

  useEffect(() => {
    return () => {
      for (const coverUrl of Object.values(coverCacheRef.current)) {
        revokeObjectUrl(coverUrl)
      }
    }
  }, [])

  useEffect(() => {
    const availableCharacterIds = new Set(characters.map((character) => character.character_id))

    setCoverCache((current) => {
      let changed = false
      const next = { ...current }

      for (const [characterId, coverUrl] of Object.entries(current)) {
        if (availableCharacterIds.has(characterId)) {
          continue
        }

        revokeObjectUrl(coverUrl)
        delete next[characterId]
        changed = true
      }

      return changed ? next : current
    })
  }, [characters])

  const characterMap = useMemo(() => buildCharacterMap(characters), [characters])
  const sessionCharacterMap = useMemo(
    () =>
      new Map(sessionCharacters.map((character) => [character.session_character_id, character])),
    [sessionCharacters],
  )
  const storiesById = useMemo(
    () => new Map(stories.map((story) => [story.story_id, story])),
    [stories],
  )
  const selectedStoryDetail = useMemo(
    () => (selectedSession ? (storyDetails[selectedSession.story_id] ?? null) : null),
    [selectedSession, storyDetails],
  )
  const currentSnapshot = liveSnapshot ?? selectedSession?.snapshot ?? null
  const currentNode = useMemo(
    () => getStoryNode(selectedStoryDetail, currentSnapshot),
    [currentSnapshot, selectedStoryDetail],
  )
  const stageCommonVariables = useMemo<StageCommonVariable[]>(
    () => buildStageCommonVariables(selectedStoryDetail, currentSnapshot),
    [currentSnapshot, selectedStoryDetail],
  )

  const refreshCoreLists = useCallback(
    async (signal?: AbortSignal) => {
      setIsListLoading(true)
      setStageAccessStatus('checking')

      const [
        sessionsResult,
        storiesResult,
        charactersResult,
        profilesResult,
        apisResult,
        apiGroupsResult,
        presetsResult,
      ] = await Promise.allSettled([
        listSessions(signal),
        listStories(signal),
        listCharacters(signal),
        listPlayerProfiles(signal),
        listApis(signal),
        listApiGroups(signal),
        listPresets(signal),
      ])

      if (sessionsResult.status === 'fulfilled') {
        setSessions(sessionsResult.value)
      } else if (!signal?.aborted) {
        setNotice({
          message: getErrorMessage(sessionsResult.reason, copy.notice.listFailed),
          tone: 'error',
        })
      }

      if (storiesResult.status === 'fulfilled') {
        setStories(storiesResult.value)
      } else if (!signal?.aborted) {
        setNotice({
          message: getErrorMessage(storiesResult.reason, copy.notice.storiesFailed),
          tone: 'error',
        })
      }

      if (charactersResult.status === 'fulfilled') {
        setCharacters(charactersResult.value)
      }

      if (profilesResult.status === 'fulfilled') {
        setPlayerProfiles(profilesResult.value)
      } else if (!signal?.aborted) {
        setNotice({
          message: getErrorMessage(profilesResult.reason, copy.notice.playerProfilesFailed),
          tone: 'error',
        })
      }

      if (apisResult.status === 'fulfilled') {
        setApis(apisResult.value)
      } else if (!signal?.aborted) {
        setNotice({
          message: getErrorMessage(apisResult.reason, copy.notice.apiResourcesFailed),
          tone: 'error',
        })
      }

      if (apiGroupsResult.status === 'fulfilled') {
        setApiGroups(apiGroupsResult.value)
      }

      if (presetsResult.status === 'fulfilled') {
        setPresets(presetsResult.value)
      }

      if (
        apisResult.status === 'fulfilled' &&
        apiGroupsResult.status === 'fulfilled' &&
        presetsResult.status === 'fulfilled'
      ) {
        if (apisResult.value.length === 0 || apiGroupsResult.value.length === 0) {
          setStageAccessStatus('blockedApiResources')
        } else if (presetsResult.value.length === 0) {
          setStageAccessStatus('blockedPresets')
        } else {
          setStageAccessStatus('ready')
        }
      } else {
        setStageAccessStatus('ready')
      }

      if (!signal?.aborted) {
        setIsListLoading(false)
      }
    },
    [
      copy.notice.apiResourcesFailed,
      copy.notice.listFailed,
      copy.notice.playerProfilesFailed,
      copy.notice.storiesFailed,
      setNotice,
    ],
  )

  const loadSelectedSession = useCallback(
    async (nextSessionId: string, signal?: AbortSignal) => {
      setIsSessionLoading(true)

      try {
        const [session, listedSessionCharacters] = await Promise.all([
          getSession(nextSessionId, signal),
          listSessionCharacters(nextSessionId, signal),
        ])

        if (signal?.aborted) {
          return
        }

        setSelectedSession({
          ...session,
          history: normalizeSessionHistory(session.history),
        })
        setSessionCharacters(listedSessionCharacters)
        setLiveSnapshot(null)

        if (!storyDetailsRef.current[session.story_id]) {
          try {
            const story = await getStory(session.story_id, signal)

            if (signal?.aborted) {
              return
            }

            setStoryDetails((current) => ({
              ...current,
              [story.story_id]: story,
            }))
          } catch {
            // Ignore background story prefetch failures here. The stage can still load the session.
          }
        }
      } catch (error) {
        if (signal?.aborted) {
          return
        }

        setSelectedSession(null)
        setSessionCharacters([])
        setNotice({
          message: getErrorMessage(error, copy.notice.sessionLoadFailed),
          tone: 'error',
        })
      } finally {
        if (!signal?.aborted) {
          setIsSessionLoading(false)
        }
      }
    },
    [copy.notice.sessionLoadFailed, setNotice],
  )

  useEffect(() => {
    const controller = new AbortController()
    void refreshCoreLists(controller.signal)

    return () => {
      controller.abort()
    }
  }, [refreshCoreLists])

  useEffect(() => {
    if (stageAccessStatus !== 'ready') {
      setSelectedSession(null)
      setSessionCharacters([])
      setLiveSnapshot(null)
      return
    }

    if (!routeSessionId) {
      setSelectedSession(null)
      setSessionCharacters([])
      setLiveSnapshot(null)
      return
    }

    const controller = new AbortController()
    void loadSelectedSession(routeSessionId, controller.signal)

    return () => {
      controller.abort()
    }
  }, [loadSelectedSession, routeSessionId, stageAccessStatus])

  useEffect(() => {
    const activeCharacterIds = currentSnapshot?.world_state.active_characters ?? []

    if (activeCharacterIds.length === 0) {
      return
    }

    const charactersNeedingCover = activeCharacterIds.filter((characterId) => {
      if (coverCache[characterId] !== undefined) {
        return false
      }

      return Boolean(characterMap.get(characterId)?.cover_mime_type)
    })

    if (charactersNeedingCover.length === 0) {
      return
    }

    let cancelled = false

    void Promise.all(
      charactersNeedingCover.map(async (characterId) => {
        try {
          return {
            characterId,
            coverUrl: await getCharacterCoverUrl(characterId),
          }
        } catch {
          return { characterId, coverUrl: null }
        }
      }),
    ).then((entries) => {
      if (cancelled) {
        for (const entry of entries) {
          if (entry.coverUrl) {
            revokeObjectUrl(entry.coverUrl)
          }
        }

        return
      }

      setCoverCache((current) => ({
        ...current,
        ...Object.fromEntries(entries.map((entry) => [entry.characterId, entry.coverUrl])),
      }))
    })

    return () => {
      cancelled = true
    }
  }, [characterMap, coverCache, currentSnapshot])

  return {
    apiGroups,
    apis,
    characterMap,
    characters,
    coverCache,
    currentNode,
    currentSnapshot,
    isListLoading,
    isSessionLoading,
    liveSnapshot,
    playerProfiles,
    presets,
    refreshCoreLists,
    selectedSession,
    selectedStoryDetail,
    sessionCharacterMap,
    sessionCharacters,
    sessions,
    setLiveSnapshot,
    setSelectedSession,
    setSessionCharacters,
    setSessions,
    stageAccessStatus,
    stageCommonVariables,
    stories,
    storiesById,
  }
}
