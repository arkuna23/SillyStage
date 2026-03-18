import { useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react'

import type { CharacterSummary } from '../characters/types'
import { deleteSessionMessage, runSessionTurnStream, suggestSessionReplies, updateSessionMessage } from './api'
import type { StageCopy } from './copy'
import {
  buildHistoryEntriesFromTurnResult,
  buildPersistedMessages,
  createInitialThoughtState,
  getErrorMessage,
  isScrolledNearBottom,
} from './stage-page-utils'
import type { ComposerMode, Notice, StageMessage, TurnWorkerStatus } from './stage-ui-types'
import type {
  ReplySuggestion,
  RuntimeSnapshot,
  SessionCharacter,
  SessionDetail,
  SessionSummary,
  StreamEventBody,
} from './types'

type UseStagePageTurnArgs = {
  characterMap: Map<string, CharacterSummary>
  copy: StageCopy
  routeSessionId?: string
  selectedSession: SessionDetail | null
  sessionCharacterMap: Map<string, SessionCharacter>
  setLiveSnapshot: (snapshot: RuntimeSnapshot | null | ((current: RuntimeSnapshot | null) => RuntimeSnapshot | null)) => void
  setNotice: (notice: Notice | null) => void
  setSelectedSession: (
    value: SessionDetail | null | ((current: SessionDetail | null) => SessionDetail | null),
  ) => void
  setSessionCharacters: (
    value: SessionCharacter[] | ((current: SessionCharacter[]) => SessionCharacter[]),
  ) => void
  setSessions: (value: SessionSummary[] | ((current: SessionSummary[]) => SessionSummary[])) => void
}

export function useStagePageTurn({
  characterMap,
  copy,
  routeSessionId,
  selectedSession,
  sessionCharacterMap,
  setLiveSnapshot,
  setNotice,
  setSelectedSession,
  setSessionCharacters,
  setSessions,
}: UseStagePageTurnArgs) {
  const streamAbortRef = useRef<AbortController | null>(null)
  const suggestionsAbortRef = useRef<AbortController | null>(null)
  const conversationScrollRef = useRef<HTMLDivElement | null>(null)
  const shouldStickToBottomRef = useRef(true)
  const composerRef = useRef<HTMLTextAreaElement | null>(null)
  const [streamMessages, setStreamMessages] = useState<StageMessage[]>([])
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
  const [isRunningTurn, setIsRunningTurn] = useState(false)
  const [activeSpeakerId, setActiveSpeakerId] = useState<string | null>(null)
  const [beatSpeakerIds, setBeatSpeakerIds] = useState<string[]>([])
  const [turnWorkerStatus, setTurnWorkerStatus] = useState<TurnWorkerStatus | null>(null)

  const sessionMessages = useMemo(
    () => [...(selectedSession ? buildPersistedMessages(selectedSession.history) : []), ...streamMessages],
    [selectedSession, streamMessages],
  )
  const overlayStatus = isSuggestingReplies
    ? { label: copy.statusBar.suggestingReplies }
    : turnWorkerStatus

  useEffect(() => {
    return () => {
      streamAbortRef.current?.abort()
      suggestionsAbortRef.current?.abort()
    }
  }, [])

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
    streamAbortRef.current?.abort()
    streamAbortRef.current = null
    suggestionsAbortRef.current?.abort()
    suggestionsAbortRef.current = null
    shouldStickToBottomRef.current = true
    setStreamMessages([])
    setComposerInput('')
    setReplySuggestions([])
    setSuggestionsError(null)
    setComposerMode('input')
    setEditingPlayerMessageId(null)
    setEditingPlayerDraft('')
    setSavingPlayerMessageId(null)
    setDeletingPlayerMessageId(null)
    setExpandedThoughtIds(createInitialThoughtState())
    setBeatSpeakerIds([])
    setActiveSpeakerId(null)
    setTurnWorkerStatus(null)
  }, [routeSessionId])

  useLayoutEffect(() => {
    const container = conversationScrollRef.current

    if (!container) {
      return
    }

    if (shouldStickToBottomRef.current) {
      container.scrollTop = container.scrollHeight
    }
  }, [routeSessionId, sessionMessages])

  function getSpeakerDisplayName(speakerId: string) {
    return characterMap.get(speakerId)?.name ?? sessionCharacterMap.get(speakerId)?.display_name ?? speakerId
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

    suggestionsAbortRef.current?.abort()

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
      suggestionsAbortRef.current?.abort()
      suggestionsAbortRef.current = null
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

  function handleCancelEditPlayerMessage() {
    setEditingPlayerMessageId(null)
    setEditingPlayerDraft('')
  }

  function handleToggleThought(messageId: string) {
    setExpandedThoughtIds((current) => {
      const next = new Set(current)

      if (next.has(messageId)) {
        next.delete(messageId)
      } else {
        next.add(messageId)
      }

      return next
    })
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
                turnIndex: nextTurnIndex,
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
              turnIndex: nextTurnIndex,
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
                turnIndex: nextTurnIndex,
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
              turnIndex: nextTurnIndex,
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
                turnIndex: nextTurnIndex,
                variant: 'thought',
              }),
              body.delta,
            )
            return
          }

          if (body.type === 'actor_completed') {
            syncActorCompleted(body, nextTurnIndex)
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

  return {
    activeSpeakerId,
    beatSpeakerIds,
    composerInput,
    composerMode,
    composerRef,
    conversationScrollRef,
    deletingPlayerMessageId,
    editingPlayerDraft,
    editingPlayerMessageId,
    expandedThoughtIds,
    handleDeletePlayerMessage,
    handleCancelEditPlayerMessage,
    handleEditPlayerMessage,
    handleRunTurn,
    handleSaveEditedPlayerMessage,
    handleSuggestReplies,
    handleToggleReplySuggestions,
    handleToggleThought,
    handleUseReplySuggestion,
    isRunningTurn,
    isSuggestingReplies,
    overlayStatus,
    replySuggestions,
    replySuggestionsEnabled,
    savingPlayerMessageId,
    sessionMessages,
    setComposerInput,
    setEditingPlayerDraft,
    suggestionsError,
    updateConversationScrollState: (element: HTMLDivElement) => {
      shouldStickToBottomRef.current = isScrolledNearBottom(element)
    },
  }
}
