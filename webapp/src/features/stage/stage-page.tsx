import { faComments } from '@fortawesome/free-solid-svg-icons/faComments'
import { faDatabase } from '@fortawesome/free-solid-svg-icons/faDatabase'
import { faPlug } from '@fortawesome/free-solid-svg-icons/faPlug'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useReducedMotion } from 'framer-motion'
import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate, useParams } from 'react-router-dom'

import { Card, CardContent } from '../../components/ui/card'
import { SegmentedSelector } from '../../components/ui/segmented-selector'
import { useToastNotice } from '../../components/ui/toast-context'
import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { appPaths } from '../../app/paths'
import { isRpcConflict } from '../../lib/rpc'
import { listApiGroups, listApis, listPresets } from '../apis/api'
import type { ApiConfig, ApiGroup, Preset } from '../apis/types'
import { CharacterDetailsDialog } from '../characters/character-details-dialog'
import { createCoverDataUrl, getCharacterCover, listCharacters } from '../characters/api'
import type { CharacterSummary } from '../characters/types'
import { listPlayerProfiles } from '../player-profiles/api'
import type { PlayerProfile } from '../player-profiles/types'
import { getStory, listStories } from '../stories/api'
import type { StoryDetail, StorySummary } from '../stories/types'
import { getStageCopy } from './copy'
import {
  deleteSessionCharacter,
  deleteSessionMessage,
  deleteSession,
  enterSessionCharacterScene,
  getRuntimeSnapshot,
  listSessionCharacters,
  getSession,
  leaveSessionCharacterScene,
  listSessions,
  runSessionTurnStream,
  setSessionPlayerProfile,
  suggestSessionReplies,
  updateSessionCharacter,
  updateSessionPlayerDescription,
  updateSessionConfig,
  updateSessionMessage,
} from './api'
import { SessionCharacterDialog } from './session-character-dialog'
import { SessionDeleteDialog } from './session-delete-dialog'
import { SessionRenameDialog } from './session-rename-dialog'
import { buildStageCommonVariables } from './stage-common-variable-utils'
import { StageCharacterVariablesPanel } from './stage-character-variables-panel'
import { StageDialoguePanel } from './stage-dialogue-panel'
import { StagePanelHeader } from './stage-panel-shared'
import { StageRightPanel } from './stage-right-panel'
import { StageSessionListPanel } from './stage-session-list-panel'
import { SessionStartDialog } from './session-start-dialog'
import { StageSessionSettingsPanel } from './stage-session-settings-panel'
import { StageSessionVariablesPanel } from './stage-session-variables-panel'
import type {
  CoverCache,
  StageCastMember,
  StageMessage,
  StageRightRailTab,
  TurnWorkerStatus,
} from './stage-ui-types'
import type {
  EngineTurnResult,
  ReplySuggestion,
  RuntimeSnapshot,
  SessionCharacter,
  SessionDetail,
  SessionHistoryEntry,
  SessionMessageResult,
  SessionVariables,
  SessionSummary,
  StartedSession,
  StreamEventBody,
  UpdateSessionConfigParams,
} from './types'

const stageRoot = '/stage'

type PanelMode = 'dialogue' | 'settings' | 'variables'
type ComposerMode = 'input' | 'suggestions'
type NoticeTone = 'error' | 'success' | 'warning'

type Notice = {
  message: string
  tone: NoticeTone
}

function getDefaultRightPanelTab(hasCommonVariables: boolean): StageRightRailTab {
  return hasCommonVariables ? 'variables' : 'status'
}

function buildStagePath(sessionId?: string) {
  return sessionId ? `${stageRoot}/${encodeURIComponent(sessionId)}` : stageRoot
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function summarizeText(text: string, maxLength = 120) {
  const normalized = text.replace(/\s+/g, ' ').trim()

  if (normalized.length <= maxLength) {
    return normalized
  }

  return `${normalized.slice(0, maxLength).trimEnd()}…`
}

function isTextLong(text: string, maxLength: number) {
  return text.replace(/\s+/g, ' ').trim().length > maxLength
}

function isScrolledNearBottom(element: HTMLElement, threshold = 56) {
  const distanceFromBottom = element.scrollHeight - element.scrollTop - element.clientHeight
  return distanceFromBottom < threshold
}

function buildPersistedMessages(history: SessionHistoryEntry[]): StageMessage[] {
  return history.map((entry, index) => ({
    id: entry.client_id ?? entry.message_id ?? `persisted:${entry.turn_index}:${index}`,
    messageId: entry.message_id,
    speakerId: entry.speaker_id,
    speakerName: entry.speaker_name,
    text: entry.text,
    turnIndex: entry.turn_index,
    updatedAtMs: entry.updated_at_ms,
    variant:
      entry.kind === 'player_input'
        ? 'player'
        : entry.kind === 'narration'
          ? 'narration'
          : entry.kind === 'action'
            ? 'action'
            : 'dialogue',
  }))
}

function normalizeSessionHistory(history: SessionMessageResult[]) {
  return history.map((entry, index) => ({
    ...entry,
    client_id: entry.client_id ?? entry.message_id ?? `persisted:${entry.turn_index}:${index}`,
  }))
}

function buildCharacterMap(characters: CharacterSummary[]) {
  return new Map(characters.map((character) => [character.character_id, character]))
}

function determineActiveCastOrder(args: {
  activeCharacterIds: string[]
  beatSpeakerIds: string[]
  currentSpeakerId: string | null
}) {
  const beatOrder = args.beatSpeakerIds.filter((characterId) =>
    args.activeCharacterIds.includes(characterId),
  )
  const rest = args.activeCharacterIds.filter((characterId) => !beatOrder.includes(characterId))
  let orderedIds = [...beatOrder, ...rest]

  if (args.currentSpeakerId && orderedIds.includes(args.currentSpeakerId)) {
    orderedIds = [
      args.currentSpeakerId,
      ...orderedIds.filter((characterId) => characterId !== args.currentSpeakerId),
    ]
  }

  return orderedIds
}

function getStoryNode(story: StoryDetail | null, snapshot: RuntimeSnapshot | null) {
  if (!story || !snapshot) {
    return null
  }

  return story.graph.nodes.find((node) => node.id === snapshot.world_state.current_node) ?? null
}

function patchSnapshotVariables(snapshot: RuntimeSnapshot, variables: SessionVariables): RuntimeSnapshot {
  return {
    ...snapshot,
    world_state: {
      ...snapshot.world_state,
      character_state: variables.character_state,
      custom: variables.custom,
      player_state: variables.player_state,
    },
  }
}

function patchSnapshotActiveCharacter(
  snapshot: RuntimeSnapshot,
  sessionCharacterId: string,
  inScene: boolean,
): RuntimeSnapshot {
  const currentIds = snapshot.world_state.active_characters
  const nextIds = inScene
    ? currentIds.includes(sessionCharacterId)
      ? currentIds
      : [...currentIds, sessionCharacterId]
    : currentIds.filter((characterId) => characterId !== sessionCharacterId)

  return {
    ...snapshot,
    world_state: {
      ...snapshot.world_state,
      active_characters: nextIds,
    },
  }
}

function createInitialThoughtState() {
  return new Set<string>()
}

function buildHistoryEntriesFromTurnResult(args: {
  narratorName: string
  playerName: string
  result: EngineTurnResult
  sessionId: string
}): SessionHistoryEntry[] {
  const recordedAtMs = Date.now()
  const turnIndex = args.result.turn_index
  const entries: SessionHistoryEntry[] = [
    {
      client_id: `stream:player:${args.sessionId}:${turnIndex}`,
      kind: 'player_input',
      recorded_at_ms: recordedAtMs,
      speaker_id: 'player',
      speaker_name: args.playerName,
      text: args.result.player_input,
      turn_index: turnIndex,
    },
  ]

  args.result.completed_beats.forEach((beat, beatIndex) => {
    if (beat.type === 'narrator') {
      entries.push({
        client_id: `stream:narrator:${beatIndex}`,
        kind: 'narration',
        recorded_at_ms: recordedAtMs,
        speaker_id: 'narrator',
        speaker_name: args.narratorName,
        text: beat.response.text,
        turn_index: turnIndex,
      })
      return
    }

    const kindCounts: Record<'action' | 'dialogue', number> = {
      action: 0,
      dialogue: 0,
    }

    beat.response.segments.forEach((segment) => {
      if (segment.kind === 'thought') {
        return
      }

      const segmentKind = segment.kind === 'action' ? 'action' : 'dialogue'
      const kindIndex = kindCounts[segmentKind]
      kindCounts[segmentKind] += 1

      entries.push({
        client_id: `stream:actor:${beatIndex}:${segmentKind}:${kindIndex}`,
        kind: segmentKind,
        recorded_at_ms: recordedAtMs,
        speaker_id: beat.speaker_id,
        speaker_name: beat.response.speaker_name,
        text: segment.text,
        turn_index: turnIndex,
      })
    })
  })

  return entries
}

export function StagePage() {
  const navigate = useNavigate()
  const { i18n } = useTranslation()
  const { sessionId: routeSessionId } = useParams<{ sessionId: string }>()
  const copy = getStageCopy(i18n.language)
  const prefersReducedMotion = useReducedMotion()
  const streamAbortRef = useRef<AbortController | null>(null)
  const suggestionsAbortRef = useRef<AbortController | null>(null)
  const autoResolvedRightPanelTabSessionRef = useRef<string | null>(null)
  const conversationScrollRef = useRef<HTMLDivElement | null>(null)
  const shouldStickToBottomRef = useRef(true)
  const composerRef = useRef<HTMLTextAreaElement | null>(null)
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
  const [streamMessages, setStreamMessages] = useState<StageMessage[]>([])
  const [panelMode, setPanelMode] = useState<PanelMode>('dialogue')
  const [composerMode, setComposerMode] = useState<ComposerMode>('input')
  const [replySuggestionsEnabled, setReplySuggestionsEnabled] = useState(false)
  const [composerInput, setComposerInput] = useState('')
  const [replySuggestions, setReplySuggestions] = useState<ReplySuggestion[]>([])
  const [isSuggestingReplies, setIsSuggestingReplies] = useState(false)
  const [suggestionsError, setSuggestionsError] = useState<string | null>(null)
  const [editingPlayerMessageId, setEditingPlayerMessageId] = useState<string | null>(null)
  const [editingPlayerDraft, setEditingPlayerDraft] = useState('')
  const [savingPlayerMessageId, setSavingPlayerMessageId] = useState<string | null>(null)
  const [deletingPlayerMessageId, setDeletingPlayerMessageId] = useState<string | null>(null)
  const [expandedThoughtIds, setExpandedThoughtIds] = useState<Set<string>>(createInitialThoughtState)
  const [isStoryIntroExpanded, setIsStoryIntroExpanded] = useState(false)
  const [isStoryNodeExpanded, setIsStoryNodeExpanded] = useState(false)
  const [rightPanelTab, setRightPanelTab] = useState<StageRightRailTab>('status')
  const [detailsCharacterId, setDetailsCharacterId] = useState<string | null>(null)
  const [detailsSessionCharacterId, setDetailsSessionCharacterId] = useState<string | null>(null)
  const [notice, setNotice] = useState<Notice | null>(null)
  useToastNotice(notice)
  const [isListLoading, setIsListLoading] = useState(true)
  const [isSessionLoading, setIsSessionLoading] = useState(false)
  const [isRunningTurn, setIsRunningTurn] = useState(false)
  const [isRefreshingList, setIsRefreshingList] = useState(false)
  const [isStartDialogOpen, setIsStartDialogOpen] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<SessionSummary | null>(null)
  const [renameTarget, setRenameTarget] = useState<SessionSummary | SessionDetail | null>(null)
  const [isDeleting, setIsDeleting] = useState(false)
  const [activeSpeakerId, setActiveSpeakerId] = useState<string | null>(null)
  const [beatSpeakerIds, setBeatSpeakerIds] = useState<string[]>([])
  const [turnWorkerStatus, setTurnWorkerStatus] = useState<TurnWorkerStatus | null>(null)
  const [stageAccessStatus, setStageAccessStatus] = useState<
    'blockedApiResources' | 'blockedPresets' | 'checking' | 'ready'
  >('checking')

  const dateFormatter = useMemo(
    () =>
      new Intl.DateTimeFormat(i18n.language.startsWith('zh') ? 'zh-CN' : 'en', {
        dateStyle: 'medium',
        timeStyle: 'short',
      }),
    [i18n.language],
  )

  const characterMap = useMemo(() => buildCharacterMap(characters), [characters])
  const sessionCharacterMap = useMemo(
    () => new Map(sessionCharacters.map((character) => [character.session_character_id, character])),
    [sessionCharacters],
  )
  const storiesById = useMemo(() => new Map(stories.map((story) => [story.story_id, story])), [stories])
  const selectedStoryDetail = useMemo(
    () => (selectedSession ? storyDetails[selectedSession.story_id] ?? null : null),
    [selectedSession, storyDetails],
  )
  const selectedStageCharacter = useMemo(
    () => (detailsCharacterId ? characterMap.get(detailsCharacterId) ?? null : null),
    [characterMap, detailsCharacterId],
  )
  const selectedSessionCharacter = useMemo(
    () =>
      detailsSessionCharacterId
        ? sessionCharacterMap.get(detailsSessionCharacterId) ?? null
        : null,
    [detailsSessionCharacterId, sessionCharacterMap],
  )
  const currentSnapshot = liveSnapshot ?? selectedSession?.snapshot ?? null
  const currentNode = useMemo(
    () => getStoryNode(selectedStoryDetail, currentSnapshot),
    [currentSnapshot, selectedStoryDetail],
  )
  const stageCommonVariables = useMemo(
    () => buildStageCommonVariables(selectedStoryDetail, currentSnapshot),
    [currentSnapshot, selectedStoryDetail],
  )
  const getSpeakerDisplayName = useCallback(
    (speakerId: string) =>
      characterMap.get(speakerId)?.name ??
      sessionCharacterMap.get(speakerId)?.display_name ??
      speakerId,
    [characterMap, sessionCharacterMap],
  )
  const sessionMessages = useMemo(
    () => [...(selectedSession ? buildPersistedMessages(selectedSession.history) : []), ...streamMessages],
    [selectedSession, streamMessages],
  )
  const orderedActiveCastIds = useMemo(
    () =>
      determineActiveCastOrder({
        activeCharacterIds: currentSnapshot?.world_state.active_characters ?? [],
        beatSpeakerIds,
        currentSpeakerId: activeSpeakerId,
      }),
    [activeSpeakerId, beatSpeakerIds, currentSnapshot],
  )
  const refreshCoreLists = useCallback(async () => {
    setIsListLoading(true)
    setStageAccessStatus('checking')

    const [sessionsResult, storiesResult, charactersResult, profilesResult, apisResult, apiGroupsResult, presetsResult] =
      await Promise.allSettled([
        listSessions(),
        listStories(),
        listCharacters(),
        listPlayerProfiles(),
        listApis(),
        listApiGroups(),
        listPresets(),
      ])

    if (sessionsResult.status === 'fulfilled') {
      setSessions(sessionsResult.value)
    } else {
      setNotice({
        message: getErrorMessage(sessionsResult.reason, copy.notice.listFailed),
        tone: 'error',
      })
    }

    if (storiesResult.status === 'fulfilled') {
      setStories(storiesResult.value)
    } else {
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
    } else {
      setNotice({
        message: getErrorMessage(profilesResult.reason, copy.notice.playerProfilesFailed),
        tone: 'error',
      })
    }

    if (apisResult.status === 'fulfilled') {
      setApis(apisResult.value)
    } else {
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

    setIsListLoading(false)
  }, [
    copy.notice.apiResourcesFailed,
    copy.notice.listFailed,
    copy.notice.playerProfilesFailed,
    copy.notice.storiesFailed,
  ])

  const loadSelectedSession = useCallback(
    async (nextSessionId: string) => {
      setIsSessionLoading(true)

      try {
        const [session, listedSessionCharacters] = await Promise.all([
          getSession(nextSessionId),
          listSessionCharacters(nextSessionId),
        ])
        shouldStickToBottomRef.current = true
        setSelectedSession({
          ...session,
          history: normalizeSessionHistory(session.history),
        })
        setSessionCharacters(listedSessionCharacters)
        setLiveSnapshot(null)
        setStreamMessages([])
        setEditingPlayerMessageId(null)
        setEditingPlayerDraft('')
        setSavingPlayerMessageId(null)
        setDeletingPlayerMessageId(null)
        setReplySuggestions([])
        setSuggestionsError(null)
        setComposerMode('input')
        setExpandedThoughtIds(createInitialThoughtState())
        setBeatSpeakerIds([])
        setActiveSpeakerId(null)
        setTurnWorkerStatus(null)

        if (!storyDetails[session.story_id]) {
          try {
            const story = await getStory(session.story_id)
            setStoryDetails((current) => ({
              ...current,
              [story.story_id]: story,
            }))
          } catch {
            // Ignore background story prefetch failures here. The stage can still load the session.
          }
        }
      } catch (error) {
        setSelectedSession(null)
        setSessionCharacters([])
        setNotice({
          message: getErrorMessage(error, copy.notice.sessionLoadFailed),
          tone: 'error',
        })
      } finally {
        setIsSessionLoading(false)
      }
    },
    [copy.notice.sessionLoadFailed, storyDetails],
  )

  useEffect(() => {
    void refreshCoreLists()
  }, [refreshCoreLists])

  useEffect(() => {
    if (stageAccessStatus === 'blockedApiResources') {
      navigate(appPaths.apis, { replace: true })
    }
    if (stageAccessStatus === 'blockedPresets') {
      navigate(appPaths.presets, { replace: true })
    }
  }, [navigate, stageAccessStatus])

  useEffect(() => {
    if (stageAccessStatus !== 'ready') {
      return
    }

    if (!routeSessionId) {
      autoResolvedRightPanelTabSessionRef.current = null
      shouldStickToBottomRef.current = true
      setSelectedSession(null)
      setSessionCharacters([])
      setLiveSnapshot(null)
      setStreamMessages([])
      setComposerInput('')
      setReplySuggestions([])
      setSuggestionsError(null)
      setComposerMode('input')
      setEditingPlayerMessageId(null)
      setEditingPlayerDraft('')
      setSavingPlayerMessageId(null)
      setDeletingPlayerMessageId(null)
      setIsStoryIntroExpanded(false)
      setIsStoryNodeExpanded(false)
      setRightPanelTab(getDefaultRightPanelTab(false))
      setDetailsCharacterId(null)
      setDetailsSessionCharacterId(null)
      setBeatSpeakerIds([])
      setActiveSpeakerId(null)
      setTurnWorkerStatus(null)
      return
    }

    if (streamAbortRef.current) {
      streamAbortRef.current.abort()
      streamAbortRef.current = null
    }

    if (suggestionsAbortRef.current) {
      suggestionsAbortRef.current.abort()
      suggestionsAbortRef.current = null
    }

    void loadSelectedSession(routeSessionId)
  }, [loadSelectedSession, routeSessionId, stageAccessStatus])

  useEffect(() => {
    autoResolvedRightPanelTabSessionRef.current = null

    if (suggestionsAbortRef.current) {
      suggestionsAbortRef.current.abort()
      suggestionsAbortRef.current = null
    }

    setIsSuggestingReplies(false)
    setComposerInput('')
    setReplySuggestions([])
    setSuggestionsError(null)
    setComposerMode('input')
    setEditingPlayerMessageId(null)
    setEditingPlayerDraft('')
    setSavingPlayerMessageId(null)
    setDeletingPlayerMessageId(null)
    setIsStoryIntroExpanded(false)
    setIsStoryNodeExpanded(false)
    setRightPanelTab(getDefaultRightPanelTab(false))
    setDetailsCharacterId(null)
    setDetailsSessionCharacterId(null)
  }, [routeSessionId])

  useEffect(() => {
    if (!routeSessionId || !selectedSession) {
      return
    }

    if (selectedSession.session_id !== routeSessionId) {
      return
    }

    if (autoResolvedRightPanelTabSessionRef.current === routeSessionId) {
      return
    }

    if (isSessionLoading) {
      return
    }

    setRightPanelTab(getDefaultRightPanelTab(stageCommonVariables.length > 0))
    autoResolvedRightPanelTabSessionRef.current = routeSessionId
  }, [isSessionLoading, routeSessionId, selectedSession, stageCommonVariables.length])

  useEffect(() => {
    if (composerMode !== 'input') {
      return
    }

    const frame = requestAnimationFrame(() => {
      composerRef.current?.focus()
    })

    return () => {
      cancelAnimationFrame(frame)
    }
  }, [composerMode, routeSessionId])

  useEffect(() => {
    if (orderedActiveCastIds.length === 0) {
      return
    }

    const charactersNeedingCover = orderedActiveCastIds.filter((characterId) => {
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
          const cover = await getCharacterCover(characterId)

          return {
            characterId,
            coverUrl: createCoverDataUrl({
              coverBase64: cover.cover_base64,
              coverMimeType: cover.cover_mime_type,
            }),
          }
        } catch {
          return { characterId, coverUrl: null }
        }
      }),
    ).then((entries) => {
      if (cancelled) {
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
  }, [characterMap, coverCache, orderedActiveCastIds])

  useLayoutEffect(() => {
    const container = conversationScrollRef.current

    if (!container) {
      return
    }

    if (shouldStickToBottomRef.current) {
      container.scrollTop = container.scrollHeight
    }
  }, [routeSessionId, sessionMessages])

  function selectSession(sessionId: string) {
    navigate(buildStagePath(sessionId))
  }

  async function handleRefreshSessions() {
    setIsRefreshingList(true)

    try {
      const nextSessions = await listSessions()
      setSessions(nextSessions)
    } catch (error) {
      setNotice({
        message: getErrorMessage(error, copy.notice.listFailed),
        tone: 'error',
      })
    } finally {
      setIsRefreshingList(false)
    }
  }

  async function handleSaveSessionConfig(params: UpdateSessionConfigParams) {
    if (!selectedSession) {
      return
    }

    const nextConfig = await updateSessionConfig(selectedSession.session_id, params)
    const runtime = await getRuntimeSnapshot(selectedSession.session_id)
    setSelectedSession((current: SessionDetail | null) =>
      current
        ? {
            ...current,
            config: nextConfig,
            snapshot: runtime.snapshot,
          }
        : current,
    )
    setLiveSnapshot(runtime.snapshot)
  }

  async function handleSetPlayerProfile(playerProfileId: string | null) {
    if (!selectedSession) {
      return
    }

    const session = await setSessionPlayerProfile(selectedSession.session_id, {
      ...(playerProfileId ? { player_profile_id: playerProfileId } : {}),
    })

    setSelectedSession({
      ...session,
      history: normalizeSessionHistory(session.history),
    })
    setLiveSnapshot(session.snapshot)
    setSessions((current) =>
      current.map((entry) =>
        entry.session_id === session.session_id
          ? {
              ...entry,
              display_name: session.display_name,
              player_profile_id: session.player_profile_id,
              player_schema_id: session.player_schema_id,
              turn_index: session.turn_index,
              updated_at_ms: session.updated_at_ms,
            }
          : entry,
      ),
    )
  }

  async function handleUpdatePlayerDescription(playerDescription: string) {
    if (!selectedSession) {
      return
    }

    const result = await updateSessionPlayerDescription(selectedSession.session_id, {
      player_description: playerDescription,
    })

    setLiveSnapshot(result.snapshot)
    setSelectedSession((current) =>
      current
        ? {
            ...current,
            player_profile_id: null,
            snapshot: result.snapshot,
          }
        : current,
    )
    setSessions((current) =>
      current.map((entry) =>
        entry.session_id === selectedSession.session_id
          ? {
              ...entry,
              player_profile_id: null,
            }
          : entry,
      ),
    )
  }

  async function handleRefreshRuntimeSnapshot() {
    if (!selectedSession) {
      return
    }

    const result = await getRuntimeSnapshot(selectedSession.session_id)
    setLiveSnapshot(result.snapshot)
    setSelectedSession((current) =>
      current
        ? {
            ...current,
            snapshot: result.snapshot,
          }
        : current,
    )
  }

  const handleVariablesApplied = useCallback((variables: SessionVariables) => {
    setSelectedSession((current) =>
      current
        ? {
            ...current,
            snapshot: patchSnapshotVariables(current.snapshot, variables),
          }
        : current,
    )
    setLiveSnapshot((current) => (current ? patchSnapshotVariables(current, variables) : current))
  }, [])

  async function handleSaveSessionCharacter(character: SessionCharacter) {
    if (!selectedSession) {
      return
    }

    try {
      const updatedCharacter = await updateSessionCharacter(selectedSession.session_id, {
        display_name: character.display_name,
        personality: character.personality,
        session_character_id: character.session_character_id,
        style: character.style,
        system_prompt: character.system_prompt,
      })

      setSessionCharacters((current) =>
        current.map((entry) =>
          entry.session_character_id === updatedCharacter.session_character_id ? updatedCharacter : entry,
        ),
      )
      setNotice({
        message: copy.notice.sessionCharacterSaved,
        tone: 'success',
      })
    } catch (error) {
      setNotice({
        message: getErrorMessage(error, copy.notice.sessionCharacterUpdateFailed),
        tone: 'error',
      })
    }
  }

  async function handleSetSessionCharacterScene(sessionCharacterId: string, inScene: boolean) {
    if (!selectedSession) {
      return
    }

    try {
      const updatedCharacter = inScene
        ? await enterSessionCharacterScene(selectedSession.session_id, {
            session_character_id: sessionCharacterId,
          })
        : await leaveSessionCharacterScene(selectedSession.session_id, {
            session_character_id: sessionCharacterId,
          })

      setSessionCharacters((current) =>
        current.map((entry) =>
          entry.session_character_id === updatedCharacter.session_character_id ? updatedCharacter : entry,
        ),
      )
      setSelectedSession((current) =>
        current
          ? {
              ...current,
              snapshot: patchSnapshotActiveCharacter(current.snapshot, sessionCharacterId, inScene),
            }
          : current,
      )
      setLiveSnapshot((current) =>
        current ? patchSnapshotActiveCharacter(current, sessionCharacterId, inScene) : current,
      )
    } catch (error) {
      setNotice({
        message: getErrorMessage(
          error,
          inScene ? copy.notice.sessionCharacterEnterFailed : copy.notice.sessionCharacterLeaveFailed,
        ),
        tone: 'error',
      })
    }
  }

  async function handleDeleteSessionCharacter(sessionCharacterId: string) {
    if (!selectedSession) {
      return
    }

    try {
      await deleteSessionCharacter(selectedSession.session_id, {
        session_character_id: sessionCharacterId,
      })

      setSessionCharacters((current) =>
        current.filter((entry) => entry.session_character_id !== sessionCharacterId),
      )
      setSelectedSession((current) =>
        current
          ? {
              ...current,
              snapshot: patchSnapshotActiveCharacter(current.snapshot, sessionCharacterId, false),
            }
          : current,
      )
      setLiveSnapshot((current) =>
        current ? patchSnapshotActiveCharacter(current, sessionCharacterId, false) : current,
      )
      if (detailsSessionCharacterId === sessionCharacterId) {
        setDetailsSessionCharacterId(null)
      }
    } catch (error) {
      setNotice({
        message: getErrorMessage(error, copy.notice.sessionCharacterDeleteFailed),
        tone: 'error',
      })
    }
  }

  async function handleCreateSession(result: { message: string; session: StartedSession }) {
    setNotice({ message: result.message, tone: 'success' })
    const nextSessions = await listSessions()
    setSessions(nextSessions)
    navigate(buildStagePath(result.session.session_id))
  }

  async function handleDeleteSession() {
    if (!deleteTarget) {
      return
    }

    setIsDeleting(true)

    try {
      await deleteSession(deleteTarget.session_id)
      setNotice({
        message: copy.notice.deleted,
        tone: 'success',
      })
      setDeleteTarget(null)
      const nextSessions = await listSessions()
      setSessions(nextSessions)

      if (routeSessionId === deleteTarget.session_id) {
        navigate(buildStagePath())
      }
    } catch (error) {
      setDeleteTarget(null)
      setNotice({
        message: isRpcConflict(error)
          ? copy.notice.deleteFailed
          : getErrorMessage(error, copy.notice.deleteFailed),
        tone: isRpcConflict(error) ? 'warning' : 'error',
      })
    } finally {
      setIsDeleting(false)
    }
  }

  function pushStreamMessage(message: StageMessage) {
    setStreamMessages((current) => {
      const existingIndex = current.findIndex((entry) => entry.id === message.id)

      if (existingIndex === -1) {
        return [...current, message]
      }

      return current.map((entry, index) => (index === existingIndex ? message : entry))
    })
  }

  function removeStreamMessage(messageId: string) {
    setStreamMessages((current) => current.filter((entry) => entry.id !== messageId))
  }

  function appendStreamText(id: string, factory: () => StageMessage, delta: string) {
    setStreamMessages((current) => {
      const existingIndex = current.findIndex((entry) => entry.id === id)

      if (existingIndex === -1) {
        return [...current, { ...factory(), text: delta }]
      }

      return current.map((entry, index) =>
        index === existingIndex
          ? {
              ...entry,
              text: `${entry.text}${delta}`,
            }
          : entry,
      )
    })
  }

  function syncActorCompleted(body: Extract<StreamEventBody, { type: 'actor_completed' }>, turnIndex: number) {
    const kindCounts: Record<'action' | 'dialogue' | 'thought', number> = {
      action: 0,
      dialogue: 0,
      thought: 0,
    }

    body.response.segments.forEach((segment) => {
      const kindIndex = kindCounts[segment.kind]
      kindCounts[segment.kind] += 1
      const id = `stream:actor:${body.beat_index}:${segment.kind}:${kindIndex}`

      if (segment.kind === 'thought') {
        pushStreamMessage({
          id,
          speakerId: body.speaker_id,
          speakerName: body.response.speaker_name,
          text: segment.text,
          turnIndex,
          variant: 'thought',
        })
        return
      }

      if (segment.kind === 'action') {
        pushStreamMessage({
          id,
          speakerId: body.speaker_id,
          speakerName: body.response.speaker_name,
          text: segment.text,
          turnIndex,
          variant: 'action',
        })
        return
      }

      pushStreamMessage({
        id,
        speakerId: body.speaker_id,
        speakerName: body.response.speaker_name,
        text: segment.text,
        turnIndex,
        variant: 'dialogue',
      })
    })
  }

  function handleEditPlayerMessage(message: StageMessage) {
    if (!message.messageId) {
      return
    }

    setEditingPlayerMessageId(message.messageId)
    setEditingPlayerDraft(message.text)
  }

  async function handleDeletePlayerMessage(message: StageMessage) {
    if (!selectedSession || !message.messageId || deletingPlayerMessageId) {
      return
    }

    setDeletingPlayerMessageId(message.messageId)

    try {
      await deleteSessionMessage(selectedSession.session_id, {
        message_id: message.messageId,
      })
      setSelectedSession((current) =>
        current
          ? {
              ...current,
              history: current.history.filter((entry) => entry.message_id !== message.messageId),
            }
          : current,
      )

      if (editingPlayerMessageId === message.messageId) {
        setEditingPlayerMessageId(null)
        setEditingPlayerDraft('')
      }
    } catch (error) {
      setNotice({
        message: getErrorMessage(error, copy.notice.messageDeleteFailed),
        tone: 'error',
      })
    } finally {
      setDeletingPlayerMessageId(null)
    }
  }

  async function handleSaveEditedPlayerMessage() {
    if (!selectedSession || !editingPlayerMessageId || !editingPlayerDraft.trim()) {
      return
    }

    const currentMessage = selectedSession.history.find(
      (entry) => entry.message_id === editingPlayerMessageId,
    )

    if (!currentMessage) {
      setEditingPlayerMessageId(null)
      setEditingPlayerDraft('')
      return
    }

    setSavingPlayerMessageId(editingPlayerMessageId)

    try {
      const updatedMessage = await updateSessionMessage(selectedSession.session_id, {
        kind: currentMessage.kind,
        message_id: editingPlayerMessageId,
        speaker_id: currentMessage.speaker_id,
        speaker_name: currentMessage.speaker_name,
        text: editingPlayerDraft.trim(),
      })

      setSelectedSession((current) =>
        current
          ? {
              ...current,
              history: current.history.map((entry) =>
                entry.message_id === editingPlayerMessageId
                  ? {
                      ...entry,
                      ...updatedMessage,
                      client_id: entry.client_id,
                    }
                  : entry,
              ),
            }
          : current,
      )
      setEditingPlayerMessageId(null)
      setEditingPlayerDraft('')
    } catch (error) {
      setNotice({
        message: getErrorMessage(error, copy.notice.messageUpdateFailed),
        tone: 'error',
      })
    } finally {
      setSavingPlayerMessageId(null)
    }
  }

  async function handleSuggestReplies() {
    if (!selectedSession || isRunningTurn || isSuggestingReplies) {
      return
    }

    if (suggestionsAbortRef.current) {
      suggestionsAbortRef.current.abort()
    }

    const controller = new AbortController()
    suggestionsAbortRef.current = controller
    setSuggestionsError(null)
    setIsSuggestingReplies(true)

    try {
      const result = await suggestSessionReplies(
        selectedSession.session_id,
        { limit: 3 },
        controller.signal,
      )

      if (controller.signal.aborted) {
        return
      }

      setReplySuggestions(result.replies)
      setComposerMode('suggestions')
    } catch (error) {
      if (controller.signal.aborted) {
        return
      }

      setSuggestionsError(getErrorMessage(error, copy.notice.suggestRepliesFailed))
      setNotice({
        message: getErrorMessage(error, copy.notice.suggestRepliesFailed),
        tone: 'error',
      })
    } finally {
      if (suggestionsAbortRef.current === controller) {
        suggestionsAbortRef.current = null
      }

      if (!controller.signal.aborted) {
        setIsSuggestingReplies(false)
      }
    }
  }

  function handleUseReplySuggestion(suggestion: ReplySuggestion) {
    setComposerInput(suggestion.text)
    setComposerMode('input')
    requestAnimationFrame(() => {
      composerRef.current?.focus()
    })
  }

  function handleToggleReplySuggestions(checked: boolean) {
    setReplySuggestionsEnabled(checked)

    if (!checked) {
      if (suggestionsAbortRef.current) {
        suggestionsAbortRef.current.abort()
        suggestionsAbortRef.current = null
      }

      setIsSuggestingReplies(false)
      setReplySuggestions([])
      setSuggestionsError(null)
      setComposerMode('input')
      requestAnimationFrame(() => {
        composerRef.current?.focus()
      })
      return
    }

    if (!selectedSession || isRunningTurn) {
      return
    }

    if (replySuggestions.length > 0 && !suggestionsError) {
      setComposerMode('suggestions')
    } else {
      void handleSuggestReplies()
    }
  }

  async function handleRunTurn() {
    if (!selectedSession || !composerInput.trim() || isRunningTurn) {
      return
    }

    const nextPlayerInput = composerInput.trim()
    const nextTurnIndex = selectedSession.snapshot.turn_index + 1
    const optimisticPlayerMessageId = `stream:player:${selectedSession.session_id}:${nextTurnIndex}`
    const controller = new AbortController()
    streamAbortRef.current = controller
    shouldStickToBottomRef.current = conversationScrollRef.current
      ? isScrolledNearBottom(conversationScrollRef.current)
      : shouldStickToBottomRef.current
    setIsRunningTurn(true)
    setTurnWorkerStatus({ label: copy.statusBar.recordingInput })
    setStreamMessages([])
    setComposerInput('')
    setReplySuggestions([])
    setSuggestionsError(null)
    setComposerMode('input')
    setExpandedThoughtIds(createInitialThoughtState())
    setLiveSnapshot(selectedSession.snapshot)
    setBeatSpeakerIds([])
    setActiveSpeakerId(null)
    pushStreamMessage({
      id: optimisticPlayerMessageId,
      speakerId: 'player',
      speakerName: copy.messages.player,
      text: nextPlayerInput,
      turnIndex: nextTurnIndex,
      variant: 'player',
    })

    try {
      const result = await runSessionTurnStream({
        onMessage: (message) => {
          if (message.frame.type !== 'event') {
            return
          }

          const body = message.frame.body

          if (body.type === 'player_input_recorded') {
            setLiveSnapshot(body.snapshot)
            setTurnWorkerStatus({ label: copy.statusBar.updatingState })
            pushStreamMessage({
              id: optimisticPlayerMessageId,
              speakerId: body.entry.speaker_id,
              speakerName: body.entry.speaker_name,
              text: body.entry.text,
              turnIndex: nextTurnIndex,
              variant: 'player',
            })
            return
          }

          if (body.type === 'keeper_applied') {
            setLiveSnapshot(body.snapshot)
            setTurnWorkerStatus({ label: copy.statusBar.updatingState })
            return
          }

          if (body.type === 'director_completed') {
            setLiveSnapshot(body.snapshot)
            setTurnWorkerStatus({ label: copy.statusBar.director })
            setBeatSpeakerIds(
              body.result.response_plan.beats.flatMap((beat) =>
                beat.type === 'actor' ? [beat.speaker_id] : [],
              ),
            )
            return
          }

          if (body.type === 'session_character_created') {
            setLiveSnapshot(body.snapshot)
            setSessionCharacters((current) => {
              const existingIndex = current.findIndex(
                (entry) => entry.session_character_id === body.session_character.session_character_id,
              )

              if (existingIndex === -1) {
                return [...current, body.session_character]
              }

              return current.map((entry, index) =>
                index === existingIndex ? body.session_character : entry,
              )
            })
            return
          }

          if (body.type === 'session_character_entered_scene') {
            setLiveSnapshot(body.snapshot)
            setSessionCharacters((current) =>
              current.map((entry) =>
                entry.session_character_id === body.session_character_id
                  ? {
                      ...entry,
                      in_scene: true,
                    }
                  : entry,
              ),
            )
            return
          }

          if (body.type === 'session_character_left_scene') {
            setLiveSnapshot(body.snapshot)
            setSessionCharacters((current) =>
              current.map((entry) =>
                entry.session_character_id === body.session_character_id
                  ? {
                      ...entry,
                      in_scene: false,
                    }
                  : entry,
              ),
            )
            return
          }

          if (body.type === 'narrator_started') {
            setActiveSpeakerId(null)
            setTurnWorkerStatus({ label: copy.statusBar.narrator })
            return
          }

          if (body.type === 'narrator_text_delta') {
            appendStreamText(
              `stream:narrator:${body.beat_index}`,
              () => ({
                id: `stream:narrator:${body.beat_index}`,
                speakerId: 'narrator',
                speakerName: copy.messages.narrator,
                text: '',
                turnIndex: selectedSession.snapshot.turn_index + 1,
                variant: 'narration',
              }),
              body.delta,
            )
            return
          }

          if (body.type === 'narrator_completed') {
            pushStreamMessage({
              id: `stream:narrator:${body.beat_index}`,
              speakerId: 'narrator',
              speakerName: copy.messages.narrator,
              text: body.response.text,
              turnIndex: selectedSession.snapshot.turn_index + 1,
              variant: 'narration',
            })
            return
          }

          if (body.type === 'actor_started') {
            setActiveSpeakerId(body.speaker_id)
            setTurnWorkerStatus({
              label: copy.statusBar.actor.replace('{name}', getSpeakerDisplayName(body.speaker_id)),
            })
            return
          }

          if (body.type === 'actor_dialogue_delta') {
            appendStreamText(
              `stream:actor:${body.beat_index}:dialogue:0`,
              () => ({
                id: `stream:actor:${body.beat_index}:dialogue:0`,
                speakerId: body.speaker_id,
                speakerName: getSpeakerDisplayName(body.speaker_id),
                text: '',
                turnIndex: selectedSession.snapshot.turn_index + 1,
                variant: 'dialogue',
              }),
              body.delta,
            )
            return
          }

          if (body.type === 'actor_action_complete') {
            pushStreamMessage({
              id: `stream:actor:${body.beat_index}:action:0`,
              speakerId: body.speaker_id,
              speakerName: getSpeakerDisplayName(body.speaker_id),
              text: body.text,
              turnIndex: selectedSession.snapshot.turn_index + 1,
              variant: 'action',
            })
            return
          }

          if (body.type === 'actor_thought_delta') {
            appendStreamText(
              `stream:actor:${body.beat_index}:thought:0`,
              () => ({
                id: `stream:actor:${body.beat_index}:thought:0`,
                speakerId: body.speaker_id,
                speakerName: getSpeakerDisplayName(body.speaker_id),
                text: '',
                turnIndex: selectedSession.snapshot.turn_index + 1,
                variant: 'thought',
              }),
              body.delta,
            )
            return
          }

          if (body.type === 'actor_completed') {
            syncActorCompleted(body, selectedSession.snapshot.turn_index + 1)
          }
        },
        playerInput: nextPlayerInput,
        sessionId: selectedSession.session_id,
        signal: controller.signal,
      })

      const committedEntries = buildHistoryEntriesFromTurnResult({
        narratorName: copy.messages.narrator,
        playerName: copy.messages.player,
        result: result.result,
        sessionId: selectedSession.session_id,
      })
      const updatedAtMs = Date.now()

      setSelectedSession((current) =>
        current
          ? {
              ...current,
              history: [...current.history, ...committedEntries],
              snapshot: result.result.snapshot,
              turn_index: result.result.turn_index,
              updated_at_ms: updatedAtMs,
            }
          : current,
      )
      setSessions((current) =>
        current.map((entry) =>
          entry.session_id === selectedSession.session_id
            ? {
                ...entry,
                turn_index: result.result.turn_index,
                updated_at_ms: updatedAtMs,
              }
            : entry,
        ),
      )
      setLiveSnapshot(result.result.snapshot)
      setStreamMessages([])

      if (replySuggestionsEnabled) {
        void handleSuggestReplies()
      }
    } catch (error) {
      if (!controller.signal.aborted) {
        removeStreamMessage(optimisticPlayerMessageId)
        setComposerInput(nextPlayerInput)
        setNotice({
          message: getErrorMessage(error, copy.notice.runFailed),
          tone: 'error',
        })
      }
    } finally {
      streamAbortRef.current = null
      setIsRunningTurn(false)
      setActiveSpeakerId(null)
      setTurnWorkerStatus(null)
      setExpandedThoughtIds(createInitialThoughtState())
    }
  }

  const activeCast: StageCastMember[] = orderedActiveCastIds.map((characterId) => {
    const character = characterMap.get(characterId)
    const sessionCharacter = sessionCharacterMap.get(characterId)

    if (sessionCharacter) {
      return {
        id: characterId,
        isSessionCharacter: true,
        name: sessionCharacter.display_name,
      }
    }

    return {
      coverUrl: coverCache[characterId],
      id: characterId,
      isSessionCharacter: false,
      name: character?.name ?? characterId,
    }
  })
  const storyIntroduction = selectedStoryDetail?.introduction?.trim() ?? ''
  const storyIntroNeedsExpand = isTextLong(storyIntroduction, 140)
  const visibleStoryIntroduction =
    storyIntroduction.length === 0
      ? copy.intro.empty
      : isStoryIntroExpanded || !storyIntroNeedsExpand
        ? storyIntroduction
        : summarizeText(storyIntroduction, 140)
  const hasExpandableNodeDetails = Boolean(
    currentSnapshot?.world_state.current_node || currentNode?.scene,
  )
  const overlayStatus = isSuggestingReplies
    ? { label: copy.statusBar.suggestingReplies }
    : turnWorkerStatus

  return (
    <section className="flex h-full min-h-0 w-full flex-1 overflow-visible py-6 sm:py-8">
      <SessionStartDialog
        apis={apis}
        apiGroups={apiGroups}
        onCompleted={(result) => handleCreateSession(result)}
        onOpenChange={setIsStartDialogOpen}
        open={isStartDialogOpen}
        playerProfiles={playerProfiles}
        presets={presets}
        stories={stories}
      />

      <SessionDeleteDialog
        copy={copy}
        deleting={isDeleting}
        onConfirm={() => void handleDeleteSession()}
        onOpenChange={(open) => {
          if (!open) {
            setDeleteTarget(null)
          }
        }}
        open={deleteTarget !== null}
        session={deleteTarget}
      />

      <SessionRenameDialog
        copy={copy}
        onCompleted={async (session) => {
          setSessions((current) =>
            current.map((entry) =>
              entry.session_id === session.session_id
                ? {
                    ...entry,
                    created_at_ms: session.created_at_ms,
                    display_name: session.display_name,
                    player_profile_id: session.player_profile_id,
                    player_schema_id: session.player_schema_id,
                    story_id: session.story_id,
                    turn_index: session.turn_index,
                    updated_at_ms: session.updated_at_ms,
                  }
                : entry,
            ),
          )
          setSelectedSession((current) =>
            current?.session_id === session.session_id
              ? {
                  ...session,
                  history: normalizeSessionHistory(session.history),
                }
              : current,
          )
          setNotice({
            message: copy.notice.sessionRenamed,
            tone: 'success',
          })
        }}
        onOpenChange={(open) => {
          if (!open) {
            setRenameTarget(null)
          }
        }}
        open={renameTarget !== null}
        session={renameTarget}
      />

      <CharacterDetailsDialog
        coverUrl={detailsCharacterId ? coverCache[detailsCharacterId] ?? undefined : undefined}
        exporting={false}
        onOpenChange={(open) => {
          if (!open) {
            setDetailsCharacterId(null)
          }
        }}
        open={detailsCharacterId !== null}
        showActions={false}
        stageTabs={
          selectedStageCharacter && selectedSession
            ? {
                detailsLabel: copy.characterDialog.details,
                variablesContent: (
                  <StageCharacterVariablesPanel
                    character={selectedStageCharacter}
                    copy={copy}
                    onVariablesApplied={handleVariablesApplied}
                    runtimeSnapshot={currentSnapshot}
                    sessionId={selectedSession.session_id}
                  />
                ),
                variablesLabel: copy.characterDialog.variables,
              }
            : undefined
        }
        summary={selectedStageCharacter}
      />

      <SessionCharacterDialog
        character={selectedSessionCharacter}
        copy={copy}
        onDelete={(sessionCharacterId) => {
          void handleDeleteSessionCharacter(sessionCharacterId)
        }}
        onOpenChange={(open) => {
          if (!open) {
            setDetailsSessionCharacterId(null)
          }
        }}
        onSave={(character) => {
          void handleSaveSessionCharacter(character)
        }}
        onToggleScene={(sessionCharacterId, inScene) => {
          void handleSetSessionCharacterScene(sessionCharacterId, inScene)
        }}
        open={detailsSessionCharacterId !== null}
      />

      <div className="grid h-full min-h-0 w-full gap-5 overflow-visible lg:grid-cols-[17rem_minmax(0,1fr)_18rem]">
        <StageSessionListPanel
          copy={copy}
          dateFormatter={dateFormatter}
          isListLoading={isListLoading}
          isRefreshingList={isRefreshingList}
          onDeleteSession={setDeleteTarget}
          onEditSession={setRenameTarget}
          onRefreshSessions={() => void handleRefreshSessions()}
          onSelectSession={selectSession}
          onStartSession={() => {
            setIsStartDialogOpen(true)
          }}
          routeSessionId={routeSessionId}
          sessions={sessions}
          storiesById={storiesById}
        />

        <WorkspacePanelShell className="h-full min-h-0">
          <Card className="flex h-full min-h-0 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
            <StagePanelHeader
              actions={
                <SegmentedSelector
                  ariaLabel={copy.stage.title}
                  className="shrink-0"
                  items={[
                    {
                      icon: <FontAwesomeIcon icon={faComments} />,
                      label: copy.tabs.dialogue,
                      value: 'dialogue',
                    },
                    {
                      icon: <FontAwesomeIcon icon={faPlug} />,
                      label: copy.tabs.settings,
                      value: 'settings',
                    },
                    {
                      icon: <FontAwesomeIcon icon={faDatabase} />,
                      label: copy.tabs.variables,
                      value: 'variables',
                    },
                  ]}
                  onValueChange={(value) => {
                    setPanelMode(value as PanelMode)
                  }}
                  value={panelMode}
                />
              }
              title={selectedSession?.display_name ?? copy.stage.title}
              titleClassName="text-[1.95rem]"
            />

            <CardContent className="min-h-0 flex-1 pt-6">
              {panelMode === 'settings' ? (
                selectedSession ? (
                  <div className="scrollbar-none h-full overflow-y-auto pr-1">
                    <StageSessionSettingsPanel
                      apiGroups={apiGroups}
                      config={selectedSession.config}
                      copy={copy}
                      currentPlayerProfileId={selectedSession.player_profile_id}
                      onRefreshSnapshot={handleRefreshRuntimeSnapshot}
                      onSavePlayerDescription={handleUpdatePlayerDescription}
                      onSavePlayerProfile={handleSetPlayerProfile}
                      onSaveSessionConfig={handleSaveSessionConfig}
                      onSessionCharacterDelete={(sessionCharacterId) => {
                        void handleDeleteSessionCharacter(sessionCharacterId)
                      }}
                      onSessionCharacterOpen={setDetailsSessionCharacterId}
                      onSessionCharacterToggleScene={(sessionCharacterId, inScene) => {
                        void handleSetSessionCharacterScene(sessionCharacterId, inScene)
                      }}
                      playerProfiles={playerProfiles}
                      presets={presets}
                      runtimeSnapshot={currentSnapshot}
                      sessionCharacters={sessionCharacters}
                    />
                  </div>
                ) : (
                  <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-5 py-6 text-sm leading-7 text-[var(--color-text-secondary)]">
                    {copy.empty.stage}
                  </div>
                )
              ) : panelMode === 'variables' ? (
                selectedSession ? (
                  <div className="scrollbar-none h-full overflow-y-auto pr-1">
                    <StageSessionVariablesPanel
                      characterMap={characterMap}
                      copy={copy}
                      onVariablesApplied={handleVariablesApplied}
                      runtimeSnapshot={currentSnapshot}
                      sessionId={selectedSession.session_id}
                      story={selectedStoryDetail}
                    />
                  </div>
                ) : (
                  <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-5 py-6 text-sm leading-7 text-[var(--color-text-secondary)]">
                    {copy.variables.empty}
                  </div>
                )
              ) : (
                <StageDialoguePanel
                  characterCovers={coverCache}
                  characterMap={characterMap}
                  composerInput={composerInput}
                  composerLocked={isRunningTurn}
                  composerMode={composerMode}
                  composerRef={composerRef}
                  conversationScrollRef={conversationScrollRef}
                  copy={copy}
                  deletingPlayerMessageId={deletingPlayerMessageId}
                  editingPlayerDraft={editingPlayerDraft}
                  editingPlayerMessageId={editingPlayerMessageId}
                  expandedThoughtIds={expandedThoughtIds}
                  isLoading={Boolean(routeSessionId) && isSessionLoading}
                  isRunningTurn={isRunningTurn}
                  isSuggestingReplies={isSuggestingReplies}
                  messages={sessionMessages}
                  onCancelEditPlayerMessage={() => {
                    setEditingPlayerMessageId(null)
                    setEditingPlayerDraft('')
                  }}
                  onChangeComposerInput={setComposerInput}
                  onChangePlayerMessageDraft={setEditingPlayerDraft}
                  onDeletePlayerMessage={(message) => void handleDeletePlayerMessage(message)}
                  onEditPlayerMessage={handleEditPlayerMessage}
                  onGenerateReplySuggestions={() => void handleSuggestReplies()}
                  onRunTurn={() => void handleRunTurn()}
                  onSavePlayerMessage={() => void handleSaveEditedPlayerMessage()}
                  onScrollConversation={(element) => {
                    shouldStickToBottomRef.current = isScrolledNearBottom(element)
                  }}
                  onSelectReplySuggestion={handleUseReplySuggestion}
                  onToggleReplySuggestions={handleToggleReplySuggestions}
                  onToggleThought={(messageId) => {
                    setExpandedThoughtIds((current) => {
                      const next = new Set(current)

                      if (next.has(messageId)) {
                        next.delete(messageId)
                      } else {
                        next.add(messageId)
                      }

                      return next
                    })
                  }}
                  overlayStatus={overlayStatus}
                  prefersReducedMotion={prefersReducedMotion}
                  replySuggestions={replySuggestions}
                  replySuggestionsEnabled={replySuggestionsEnabled}
                  savingPlayerMessageId={savingPlayerMessageId}
                  selectedSessionExists={Boolean(selectedSession)}
                  suggestionsError={suggestionsError}
                />
              )}
            </CardContent>
          </Card>
        </WorkspacePanelShell>

        <StageRightPanel
          activeCast={activeCast}
          activeSpeakerId={activeSpeakerId}
          commonVariables={stageCommonVariables}
          copy={copy}
          currentNode={currentNode}
          currentSnapshot={currentSnapshot}
          hasExpandableNodeDetails={hasExpandableNodeDetails}
          isStoryIntroExpanded={isStoryIntroExpanded}
          isStoryNodeExpanded={isStoryNodeExpanded}
          onChangeRailTab={setRightPanelTab}
          onOpenCharacter={setDetailsCharacterId}
          onOpenSessionCharacter={setDetailsSessionCharacterId}
          onToggleStoryIntro={() => {
            setIsStoryIntroExpanded((current) => !current)
          }}
          onToggleStoryNode={() => {
            setIsStoryNodeExpanded((current) => !current)
          }}
          prefersReducedMotion={prefersReducedMotion}
          railTab={rightPanelTab}
          storyIntroNeedsExpand={storyIntroNeedsExpand}
          visibleStoryIntroduction={visibleStoryIntroduction}
        />
      </div>
    </section>
  )
}
