import { faComments } from '@fortawesome/free-solid-svg-icons/faComments'
import { faChevronDown } from '@fortawesome/free-solid-svg-icons/faChevronDown'
import { faPaperPlane } from '@fortawesome/free-solid-svg-icons/faPaperPlane'
import { faPen } from '@fortawesome/free-solid-svg-icons/faPen'
import { faPlug } from '@fortawesome/free-solid-svg-icons/faPlug'
import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { faRotateRight } from '@fortawesome/free-solid-svg-icons/faRotateRight'
import { faSpinner } from '@fortawesome/free-solid-svg-icons/faSpinner'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { AnimatePresence, LayoutGroup, motion, useReducedMotion } from 'framer-motion'
import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react'
import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate, useParams } from 'react-router-dom'

import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '../../components/ui/card'
import { IconButton } from '../../components/ui/icon-button'
import { SegmentedSelector } from '../../components/ui/segmented-selector'
import { Switch } from '../../components/ui/switch'
import { Textarea } from '../../components/ui/textarea'
import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { cn } from '../../lib/cn'
import { isRpcConflict } from '../../lib/rpc'
import { appPaths } from '../../app/paths'
import { getDefaultLlmConfig, listLlmApis } from '../apis/api'
import type { LlmApi } from '../apis/types'
import { CharacterDetailsDialog } from '../characters/character-details-dialog'
import { listCharacters, getCharacterCover, createCoverDataUrl } from '../characters/api'
import type { CharacterSummary } from '../characters/types'
import { listPlayerProfiles } from '../player-profiles/api'
import type { PlayerProfile } from '../player-profiles/types'
import { getStory, listStories } from '../stories/api'
import type { StoryDetail, StorySummary } from '../stories/types'
import { getStageCopy } from './copy'
import {
  deleteSessionMessage,
  deleteSession,
  getRuntimeSnapshot,
  getSession,
  listSessionMessages,
  listSessions,
  runSessionTurnStream,
  setSessionPlayerProfile,
  suggestSessionReplies,
  updateSessionPlayerDescription,
  updateSessionConfig,
  updateSessionMessage,
} from './api'
import { SessionDeleteDialog } from './session-delete-dialog'
import { SessionRenameDialog } from './session-rename-dialog'
import { SessionStartDialog } from './session-start-dialog'
import { StageSessionSettingsPanel } from './stage-session-settings-panel'
import type {
  EngineTurnResult,
  ReplySuggestion,
  RuntimeSnapshot,
  SessionDetail,
  SessionHistoryEntry,
  SessionMessageResult,
  SessionSummary,
  StartedSession,
  StreamEventBody,
  UpdateSessionConfigParams,
} from './types'

const stageRoot = '/stage'
const COVER_OBJECT_POSITION = 'center 26%'
const panelEase = [0.16, 1, 0.3, 1] as const

type PanelMode = 'api' | 'session'
type ComposerMode = 'input' | 'suggestions'
type NoticeTone = 'error' | 'success' | 'warning'
type StageMessageVariant = 'action' | 'dialogue' | 'narration' | 'player' | 'thought'

type Notice = {
  message: string
  tone: NoticeTone
}

type TurnWorkerStatus = {
  label: string
}

type StageMessage = {
  id: string
  messageId?: string
  speakerId: string
  speakerName: string
  text: string
  turnIndex: number
  updatedAtMs?: number
  variant: StageMessageVariant
}

type CoverCache = Record<string, string | null | undefined>

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

function isSameTranscriptMessage(a: SessionHistoryEntry | undefined, b: SessionHistoryEntry) {
  if (!a) {
    return false
  }

  if (a.message_id && b.message_id) {
    return a.message_id === b.message_id
  }

  return (
    a.kind === b.kind &&
    a.speaker_id === b.speaker_id &&
    a.speaker_name === b.speaker_name &&
    a.text === b.text &&
    a.turn_index === b.turn_index
  )
}

function createTranscriptFingerprint(entry: Pick<SessionHistoryEntry, 'kind' | 'speaker_id' | 'speaker_name' | 'text' | 'turn_index'>) {
  return [entry.turn_index, entry.kind, entry.speaker_id, entry.speaker_name, entry.text].join('::')
}

function hydrateHistoryClientIds(
  currentHistory: SessionHistoryEntry[],
  nextHistory: SessionHistoryEntry[],
) {
  const currentClientIdsByMessageId = new Map(
    currentHistory
      .filter((entry): entry is SessionHistoryEntry & { client_id: string; message_id: string } =>
        Boolean(entry.client_id && entry.message_id),
      )
      .map((entry) => [entry.message_id, entry.client_id]),
  )
  const localClientIdsByFingerprint = new Map<string, string[]>()

  currentHistory.forEach((entry) => {
    if (entry.message_id || !entry.client_id) {
      return
    }

    const fingerprint = createTranscriptFingerprint(entry)
    const bucket = localClientIdsByFingerprint.get(fingerprint) ?? []
    bucket.push(entry.client_id)
    localClientIdsByFingerprint.set(fingerprint, bucket)
  })

  return nextHistory.map((entry, index) => {
    const currentEntry = currentHistory[index]

    if (entry.message_id && currentClientIdsByMessageId.has(entry.message_id)) {
      return {
        ...entry,
        client_id: currentClientIdsByMessageId.get(entry.message_id),
      }
    }

    if (isSameTranscriptMessage(currentEntry, entry) && currentEntry?.client_id) {
      return {
        ...entry,
        client_id: currentEntry.client_id,
      }
    }

    const fingerprint = createTranscriptFingerprint(entry)
    const bucket = localClientIdsByFingerprint.get(fingerprint)

    if (bucket?.length) {
      const clientId = bucket.shift()

      return {
        ...entry,
        client_id: clientId,
      }
    }

    return entry
  })
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

function formatTime(dateFormatter: Intl.DateTimeFormat, value?: number | null) {
  if (!value) {
    return null
  }

  return dateFormatter.format(value)
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

function getCharacterMonogram(name: string) {
  return Array.from(name.trim())[0] ?? '?'
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

function StageNotice({ notice }: { notice: Notice }) {
  return (
    <div
      className={cn(
        'rounded-[1.35rem] border px-4 py-3 text-sm leading-7',
        notice.tone === 'success'
          ? 'border-[var(--color-state-success-line)] bg-[var(--color-state-success-soft)] text-[var(--color-text-primary)]'
          : notice.tone === 'warning'
            ? 'border-[var(--color-state-warning-line)] bg-[var(--color-state-warning-soft)] text-[var(--color-text-primary)]'
            : 'border-[var(--color-state-error-line)] bg-[var(--color-state-error-soft)] text-[var(--color-text-primary)]',
      )}
      role="status"
    >
      {notice.message}
    </div>
  )
}

function CharacterAvatar({
  coverUrl,
  name,
}: {
  coverUrl?: string | null
  name: string
}) {
  const monogram = getCharacterMonogram(name)

  return (
    <div className="size-10 overflow-hidden rounded-full border border-[var(--color-border-subtle)] bg-[linear-gradient(135deg,var(--color-accent-gold-soft),var(--color-accent-copper-soft))] shadow-[0_12px_24px_rgba(0,0,0,0.12)]">
      {coverUrl ? (
        <img
          alt={name}
          className="h-full w-full object-cover"
          src={coverUrl}
          style={{ objectPosition: COVER_OBJECT_POSITION }}
        />
      ) : (
        <div className="flex h-full w-full items-center justify-center">
          <span className="font-display text-sm text-[var(--color-text-primary)]">{monogram}</span>
        </div>
      )}
    </div>
  )
}

function SessionListSkeleton() {
  return (
    <div className="space-y-3">
      {Array.from({ length: 5 }).map((_, index) => (
        <div
          className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4"
          key={index}
        >
          <div className="h-5 w-28 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
          <div className="mt-3 h-3 w-20 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
          <div className="mt-3 h-3 w-full animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
          <div className="mt-2 h-3 w-4/5 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
        </div>
      ))}
    </div>
  )
}

function ConversationSkeleton() {
  return (
    <div className="space-y-5">
      {Array.from({ length: 6 }).map((_, index) => (
        <div className={cn('flex gap-3', index % 3 === 1 ? 'justify-center' : 'justify-start')} key={index}>
          {index % 3 === 1 ? null : (
            <div className="size-10 rounded-full bg-[var(--color-bg-elevated)]" />
          )}
          <div
            className={cn(
              'animate-pulse rounded-[1.4rem] bg-[var(--color-bg-elevated)]',
              index % 3 === 1 ? 'h-16 w-[min(72%,28rem)]' : 'h-20 w-[min(78%,32rem)]',
            )}
          />
        </div>
      ))}
    </div>
  )
}

function RightPanelSection({
  action,
  children,
  description,
  title,
}: {
  action?: ReactNode
  children: ReactNode
  description?: string
  title: string
}) {
  return (
    <section className="space-y-3">
      <div className="space-y-1.5">
        <div className="flex items-center justify-between gap-3">
          <CardTitle className="text-[1.15rem] leading-snug">{title}</CardTitle>
          {action ? <div className="shrink-0">{action}</div> : null}
        </div>
        {description ? (
          <CardDescription className="text-sm leading-6">{description}</CardDescription>
        ) : null}
      </div>
      {children}
    </section>
  )
}

function StagePanelHeader({
  actions,
  description,
  title,
  titleClassName,
}: {
  actions?: ReactNode
  description: string
  title: string
  titleClassName?: string
}) {
  return (
    <CardHeader className="h-[7rem] justify-between gap-3 border-b border-[var(--color-border-subtle)] px-6 py-5">
      <div className="flex min-h-0 items-start justify-between gap-4">
        <div className="min-w-0 flex-1">
          <CardTitle className={cn('truncate', titleClassName)}>{title}</CardTitle>
        </div>
        <div className="flex h-10 shrink-0 items-center justify-end">
          {actions ? <div className="flex items-center gap-2">{actions}</div> : null}
        </div>
      </div>

      <CardDescription className="line-clamp-1 min-h-6 text-sm leading-6">
        {description}
      </CardDescription>
    </CardHeader>
  )
}

function ThoughtBubble({
  copy,
  expanded,
  message,
  onToggle,
}: {
  copy: ReturnType<typeof getStageCopy>
  expanded: boolean
  message: StageMessage
  onToggle: () => void
}) {
  if (!expanded) {
    return (
      <div className="inline-flex max-w-fit items-center gap-3 rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_88%,transparent)] px-4 py-2.5 text-left transition hover:border-[var(--color-accent-copper-soft)]">
        <p className="whitespace-nowrap text-xs uppercase text-[var(--color-text-muted)]">
          {copy.messages.thinking}
        </p>
        <IconButton
          className="h-5 w-5 min-h-0 shrink-0 rounded-full px-0"
          icon={<FontAwesomeIcon className="text-[0.58rem]" icon={faChevronDown} />}
          label={copy.messages.expandThought}
          onClick={onToggle}
          size="sm"
          variant="ghost"
        />
      </div>
    )
  }

  return (
    <div className="max-w-[min(72%,30rem)] rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_88%,transparent)] px-5 py-3.5 text-left transition hover:border-[var(--color-accent-copper-soft)]">
      <div className="flex items-center justify-between gap-3">
        <p className="whitespace-nowrap text-xs uppercase text-[var(--color-text-muted)]">
          {copy.messages.thinking}
        </p>
        <IconButton
          className="h-5 w-5 min-h-0 shrink-0 rounded-full px-0"
          icon={
            <FontAwesomeIcon
              className={cn('text-[0.58rem] transition-transform', expanded ? 'rotate-180' : '')}
              icon={faChevronDown}
            />
          }
          label={copy.messages.expandThought}
          onClick={onToggle}
          size="sm"
          variant="ghost"
        />
      </div>
      {expanded ? (
        <p className="mt-2 text-sm leading-7 text-[var(--color-text-secondary)]">{message.text}</p>
      ) : null}
    </div>
  )
}

function TurnStatusBar({
  status,
}: {
  status: TurnWorkerStatus | null
}) {
  if (!status) {
    return null
  }

  return (
    <motion.div
      animate={{ opacity: 1, y: 0 }}
      className="pointer-events-none absolute left-1/2 top-0 z-10 -translate-x-1/2 -translate-y-1/2"
      exit={{ opacity: 0, y: -6 }}
      initial={{ opacity: 0, y: -8 }}
      transition={{ duration: 0.22, ease: panelEase }}
    >
      <div className="inline-flex h-8 items-center gap-2 rounded-full border border-[var(--color-accent-gold-line)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] px-3.5 text-[0.82rem] text-[var(--color-text-secondary)] shadow-[0_10px_22px_rgba(0,0,0,0.12)] backdrop-blur-sm">
        <FontAwesomeIcon className="animate-spin text-[0.68rem] text-[var(--color-accent-copper)]" icon={faSpinner} />
        <span className="whitespace-nowrap">{status.label}</span>
      </div>
    </motion.div>
  )
}

function StageConversation({
  composerLocked,
  characterCovers,
  characterMap,
  copy,
  deletingPlayerMessageId,
  expandedThoughtIds,
  editingPlayerDraft,
  editingPlayerMessageId,
  isLoading,
  messages,
  onCancelEditPlayerMessage,
  onChangePlayerMessageDraft,
  onDeletePlayerMessage,
  onEditPlayerMessage,
  onSavePlayerMessage,
  onToggleThought,
  prefersReducedMotion,
  savingPlayerMessageId,
}: {
  composerLocked: boolean
  characterCovers: CoverCache
  characterMap: Map<string, CharacterSummary>
  copy: ReturnType<typeof getStageCopy>
  deletingPlayerMessageId: string | null
  expandedThoughtIds: Set<string>
  editingPlayerDraft: string
  editingPlayerMessageId: string | null
  isLoading: boolean
  messages: StageMessage[]
  onCancelEditPlayerMessage: () => void
  onChangePlayerMessageDraft: (value: string) => void
  onDeletePlayerMessage: (message: StageMessage) => void
  onEditPlayerMessage: (message: StageMessage) => void
  onSavePlayerMessage: () => void
  onToggleThought: (messageId: string) => void
  prefersReducedMotion: boolean | null
  savingPlayerMessageId: string | null
}) {
  if (isLoading) {
    return <ConversationSkeleton />
  }

  if (messages.length === 0) {
    return (
      <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-5 py-6 text-sm leading-7 text-[var(--color-text-secondary)]">
        {copy.messages.noMessages}
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <AnimatePresence initial={false}>
      {messages.map((message, index) => {
        const previous = messages[index - 1]
        const next = messages[index + 1]
        const isActorMessage =
          message.variant === 'dialogue' || message.variant === 'action' || message.variant === 'thought'
        const sameAsPrevious =
          isActorMessage &&
          previous &&
          (previous.variant === 'dialogue' || previous.variant === 'action' || previous.variant === 'thought') &&
          previous.speakerId === message.speakerId
        const sameAsNext =
          isActorMessage &&
          next &&
          (next.variant === 'dialogue' || next.variant === 'action' || next.variant === 'thought') &&
          next.speakerId === message.speakerId
        const coverUrl = characterCovers[message.speakerId]
        const character = characterMap.get(message.speakerId)

        if (message.variant === 'narration') {
          return (
            <motion.div
              animate={{ opacity: 1, y: 0 }}
              className="flex justify-center"
              exit={{ opacity: 0, y: -8 }}
              initial={{ opacity: 0, y: 10 }}
              key={message.id}
              transition={prefersReducedMotion ? { duration: 0 } : { duration: 0.22, ease: panelEase }}
            >
              <div className="max-w-[min(72%,30rem)] rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_56%,transparent)] px-4 py-3 text-center text-sm leading-7 text-[var(--color-text-secondary)]">
                {message.text}
              </div>
            </motion.div>
          )
        }

        if (message.variant === 'player') {
          const isEditing = Boolean(message.messageId) && editingPlayerMessageId === message.messageId
          const isDeleting = Boolean(message.messageId) && deletingPlayerMessageId === message.messageId
          const isSaving = Boolean(message.messageId) && savingPlayerMessageId === message.messageId
          const canMutate = Boolean(message.messageId)

          return (
            <motion.div
              animate={{ opacity: 1, x: 0, y: 0 }}
              className="flex justify-end"
              exit={{ opacity: 0, x: 10, y: -6 }}
              initial={{ opacity: 0, x: 16, y: 10 }}
              key={message.id}
              transition={prefersReducedMotion ? { duration: 0 } : { duration: 0.24, ease: panelEase }}
            >
              <div className="max-w-[min(76%,30rem)] space-y-2">
                {isEditing ? (
                  <div className="space-y-3 rounded-[1.35rem] border border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] px-4 py-3">
                    <Textarea
                      className="min-h-[7rem] border-[var(--color-accent-gold-line)] bg-[color-mix(in_srgb,var(--color-bg-panel)_86%,white)] text-[var(--color-text-primary)]"
                      onChange={(event) => {
                        onChangePlayerMessageDraft(event.target.value)
                      }}
                      value={editingPlayerDraft}
                    />
                    <div className="flex justify-end gap-2">
                      <Button
                        disabled={composerLocked || isSaving}
                        onClick={onCancelEditPlayerMessage}
                        size="sm"
                        variant="ghost"
                      >
                        {copy.messages.cancelEditPlayer}
                      </Button>
                      <Button
                        disabled={composerLocked || isSaving || !editingPlayerDraft.trim()}
                        onClick={onSavePlayerMessage}
                        size="sm"
                        variant="secondary"
                      >
                        {copy.messages.savePlayer}
                      </Button>
                    </div>
                  </div>
                ) : (
                  <>
                    <div className="rounded-[1.35rem] border border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] px-4 py-3 text-sm leading-7 text-[var(--color-text-primary)]">
                      {message.text}
                    </div>
                    <div className="flex justify-end gap-2">
                      <IconButton
                        className="h-8 w-8 rounded-full px-0"
                        disabled={composerLocked || !canMutate || isDeleting || isSaving}
                        icon={<FontAwesomeIcon className="text-xs" icon={faPen} />}
                        label={copy.messages.editPlayer}
                        onClick={() => {
                          onEditPlayerMessage(message)
                        }}
                        size="sm"
                        variant="ghost"
                      />
                      <IconButton
                        className="h-8 w-8 rounded-full px-0"
                        disabled={composerLocked || !canMutate || isDeleting || isSaving}
                        icon={<FontAwesomeIcon className="text-xs" icon={faTrashCan} />}
                        label={copy.messages.deletePlayer}
                        onClick={() => {
                          onDeletePlayerMessage(message)
                        }}
                        size="sm"
                        variant="ghost"
                      />
                    </div>
                  </>
                )}
              </div>
            </motion.div>
          )
        }

        return (
          <motion.div
            animate={{ opacity: 1, x: 0, y: 0 }}
            className="flex items-end gap-3"
            exit={{ opacity: 0, x: -10, y: -6 }}
            initial={{ opacity: 0, x: -14, y: 10 }}
            key={message.id}
            transition={prefersReducedMotion ? { duration: 0 } : { duration: 0.24, ease: panelEase }}
          >
            <div className="w-10 shrink-0">
              {!sameAsPrevious ? (
                <CharacterAvatar coverUrl={coverUrl} name={character?.name ?? message.speakerName} />
              ) : null}
            </div>
            <div className={cn('flex min-w-0 flex-col gap-2', sameAsNext ? 'pb-1' : '')}>
              {!sameAsPrevious ? (
                <p className="text-xs text-[var(--color-text-muted)]">{message.speakerName}</p>
              ) : null}

              {message.variant === 'thought' ? (
                <ThoughtBubble
                  copy={copy}
                  expanded={expandedThoughtIds.has(message.id)}
                  message={message}
                  onToggle={() => {
                    onToggleThought(message.id)
                  }}
                />
              ) : (
                <div
                  className={cn(
                    'max-w-[min(76%,32rem)] rounded-[1.35rem] border px-4 py-3 text-sm leading-7 shadow-[0_12px_26px_rgba(0,0,0,0.1)]',
                    message.variant === 'action'
                      ? 'border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_90%,transparent)] text-[var(--color-text-secondary)] italic'
                      : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] text-[var(--color-text-primary)]',
                  )}
                >
                  {message.text}
                </div>
              )}
            </div>
          </motion.div>
        )
      })}
      </AnimatePresence>
    </div>
  )
}

export function StagePage() {
  const navigate = useNavigate()
  const { i18n } = useTranslation()
  const { sessionId: routeSessionId } = useParams<{ sessionId: string }>()
  const copy = getStageCopy(i18n.language)
  const prefersReducedMotion = useReducedMotion()
  const streamAbortRef = useRef<AbortController | null>(null)
  const suggestionsAbortRef = useRef<AbortController | null>(null)
  const conversationScrollRef = useRef<HTMLDivElement | null>(null)
  const shouldStickToBottomRef = useRef(true)
  const composerRef = useRef<HTMLTextAreaElement | null>(null)
  const [sessions, setSessions] = useState<SessionSummary[]>([])
  const [stories, setStories] = useState<StorySummary[]>([])
  const [characters, setCharacters] = useState<CharacterSummary[]>([])
  const [playerProfiles, setPlayerProfiles] = useState<PlayerProfile[]>([])
  const [apis, setApis] = useState<LlmApi[]>([])
  const [storyDetails, setStoryDetails] = useState<Record<string, StoryDetail>>({})
  const [coverCache, setCoverCache] = useState<CoverCache>({})
  const [selectedSession, setSelectedSession] = useState<SessionDetail | null>(null)
  const [liveSnapshot, setLiveSnapshot] = useState<RuntimeSnapshot | null>(null)
  const [streamMessages, setStreamMessages] = useState<StageMessage[]>([])
  const [panelMode, setPanelMode] = useState<PanelMode>('session')
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
  const [detailsCharacterId, setDetailsCharacterId] = useState<string | null>(null)
  const [notice, setNotice] = useState<Notice | null>(null)
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
  const [stageAccessStatus, setStageAccessStatus] = useState<'blocked' | 'checking' | 'ready'>(
    'checking',
  )

  const dateFormatter = useMemo(
    () =>
      new Intl.DateTimeFormat(i18n.language.startsWith('zh') ? 'zh-CN' : 'en', {
        dateStyle: 'medium',
        timeStyle: 'short',
      }),
    [i18n.language],
  )

  const characterMap = useMemo(() => buildCharacterMap(characters), [characters])
  const storiesById = useMemo(() => new Map(stories.map((story) => [story.story_id, story])), [stories])
  const selectedStoryDetail = useMemo(
    () => (selectedSession ? storyDetails[selectedSession.story_id] ?? null : null),
    [selectedSession, storyDetails],
  )
  const selectedStageCharacter = useMemo(
    () => (detailsCharacterId ? characterMap.get(detailsCharacterId) ?? null : null),
    [characterMap, detailsCharacterId],
  )
  const currentSnapshot = liveSnapshot ?? selectedSession?.snapshot ?? null
  const currentNode = useMemo(() => getStoryNode(selectedStoryDetail, currentSnapshot), [currentSnapshot, selectedStoryDetail])
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
  const replyerApiOverride = useMemo(() => {
    const replyerApiId =
      selectedSession?.config.session_api_ids?.replyer_api_id?.trim() ||
      selectedSession?.config.effective_api_ids.replyer_api_id?.trim()

    if (!replyerApiId) {
      return undefined
    }

    return {
      replyer_api_id: replyerApiId,
    }
  }, [selectedSession?.config.effective_api_ids.replyer_api_id, selectedSession?.config.session_api_ids?.replyer_api_id])
  const refreshCoreLists = useCallback(async () => {
    setIsListLoading(true)
    setStageAccessStatus('checking')

    const [
      sessionsResult,
      storiesResult,
      charactersResult,
      profilesResult,
      apisResult,
      defaultConfigResult,
    ] =
      await Promise.allSettled([
        listSessions(),
        listStories(),
        listCharacters(),
        listPlayerProfiles(),
        listLlmApis(),
        getDefaultLlmConfig(),
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
    }

    if (apisResult.status === 'fulfilled' && defaultConfigResult.status === 'fulfilled') {
      setStageAccessStatus(
        apisResult.value.length === 0 && !defaultConfigResult.value.effective
          ? 'blocked'
          : 'ready',
      )
    } else {
      setStageAccessStatus('ready')
    }

    setIsListLoading(false)
  }, [copy.notice.listFailed, copy.notice.playerProfilesFailed, copy.notice.storiesFailed])

  const loadSelectedSession = useCallback(
    async (nextSessionId: string) => {
      setIsSessionLoading(true)

      try {
        const session = await getSession(nextSessionId)
        shouldStickToBottomRef.current = true
        setSelectedSession({
          ...session,
          history: normalizeSessionHistory(session.history),
        })
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

  const syncSessionMessages = useCallback(async (sessionId: string) => {
    try {
      const messages = await listSessionMessages(sessionId)

      setSelectedSession((current) => {
        if (!current || current.session_id !== sessionId) {
          return current
        }

        return {
          ...current,
          history: hydrateHistoryClientIds(current.history, normalizeSessionHistory(messages)),
        }
      })
    } catch {
      // Keep the current staged transcript if background sync fails.
    }
  }, [])

  useEffect(() => {
    void refreshCoreLists()
  }, [refreshCoreLists])

  useEffect(() => {
    if (stageAccessStatus === 'blocked') {
      navigate(appPaths.apis, { replace: true })
    }
  }, [navigate, stageAccessStatus])

  useEffect(() => {
    if (stageAccessStatus !== 'ready') {
      return
    }

    if (!routeSessionId) {
      shouldStickToBottomRef.current = true
      setSelectedSession(null)
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
      setDetailsCharacterId(null)
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
    setDetailsCharacterId(null)
  }, [routeSessionId])

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

    if (shouldStickToBottomRef.current || isRunningTurn) {
      container.scrollTop = container.scrollHeight
    }
  }, [isRunningTurn, routeSessionId, sessionMessages])

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
        {
          ...(replyerApiOverride ? { api_overrides: replyerApiOverride } : {}),
          limit: 3,
        },
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
    shouldStickToBottomRef.current = true
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
              label: copy.statusBar.actor.replace(
                '{name}',
                characterMap.get(body.speaker_id)?.name ?? body.speaker_id,
              ),
            })
            return
          }

          if (body.type === 'actor_dialogue_delta') {
            appendStreamText(
              `stream:actor:${body.beat_index}:dialogue:0`,
              () => ({
                id: `stream:actor:${body.beat_index}:dialogue:0`,
                speakerId: body.speaker_id,
                speakerName: characterMap.get(body.speaker_id)?.name ?? body.speaker_id,
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
              speakerName: characterMap.get(body.speaker_id)?.name ?? body.speaker_id,
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
                speakerName: characterMap.get(body.speaker_id)?.name ?? body.speaker_id,
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
      void syncSessionMessages(selectedSession.session_id)

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

  const activeCast = orderedActiveCastIds.map((characterId) => {
    const character = characterMap.get(characterId)

    return {
      coverUrl: coverCache[characterId],
      description: character ? summarizeText(character.personality, 72) : '',
      id: characterId,
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
        onCompleted={(result) => handleCreateSession(result)}
        onOpenChange={setIsStartDialogOpen}
        open={isStartDialogOpen}
        playerProfiles={playerProfiles}
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
        summary={selectedStageCharacter}
      />

      <div className="grid h-full min-h-0 w-full gap-5 overflow-visible lg:grid-cols-[17rem_minmax(0,1fr)_18rem]">
        <WorkspacePanelShell className="h-full min-h-0">
          <Card className="flex h-full min-h-0 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
            <StagePanelHeader
              actions={
                <>
                  <IconButton
                    disabled={isRefreshingList}
                    icon={
                      <FontAwesomeIcon
                        className={cn(isRefreshingList ? 'animate-spin' : '')}
                        icon={faRotateRight}
                      />
                    }
                    label={copy.list.refresh}
                    onClick={() => void handleRefreshSessions()}
                    variant="ghost"
                  />
                  <IconButton
                    icon={<FontAwesomeIcon icon={faPlus} />}
                    label={copy.createSession.title}
                    onClick={() => {
                      setIsStartDialogOpen(true)
                    }}
                  />
                </>
              }
              description={copy.list.subtitle}
              title={copy.list.section}
              titleClassName="text-[1.35rem]"
            />

            <CardContent className="min-h-0 flex-1 overflow-y-auto pt-5">
              <div className="space-y-4 pr-1">
                {notice ? <StageNotice notice={notice} /> : null}

                {isListLoading ? (
                  <SessionListSkeleton />
                ) : sessions.length === 0 ? (
                  <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-4 py-5 text-sm leading-7 text-[var(--color-text-secondary)]">
                    {copy.list.empty}
                  </div>
                ) : (
                  <div className="space-y-3">
                    {sessions.map((session) => {
                      const story = storiesById.get(session.story_id)
                      const isActive = session.session_id === routeSessionId
                      const timeText =
                        formatTime(dateFormatter, session.updated_at_ms ?? session.created_at_ms) ??
                        copy.time.unknown

                      return (
                        <button
                          className={cn(
                            'w-full rounded-[1.4rem] border px-4 py-4 text-left transition',
                            isActive
                              ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)] shadow-[0_18px_40px_var(--color-accent-glow-soft)]'
                              : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] text-[var(--color-text-secondary)] hover:border-[var(--color-accent-copper-soft)] hover:text-[var(--color-text-primary)]',
                          )}
                          key={session.session_id}
                          onClick={() => {
                            selectSession(session.session_id)
                          }}
                          type="button"
                        >
                          <div className="flex items-start justify-between gap-3">
                            <div className="min-w-0 space-y-2">
                              <p className="truncate font-display text-[1.05rem] leading-tight">
                                {session.display_name}
                              </p>
                              <p className="text-xs text-[var(--color-text-muted)]">{timeText}</p>
                              <p className="line-clamp-2 text-sm leading-6">
                                {story?.introduction
                                  ? summarizeText(story.introduction, 88)
                                  : copy.list.untitledStory}
                              </p>
                            </div>
                            <IconButton
                              className="shrink-0"
                              icon={<FontAwesomeIcon icon={faPen} />}
                              label={copy.editSession}
                              onClick={(event) => {
                                event.stopPropagation()
                                setRenameTarget(session)
                              }}
                              size="sm"
                              variant="secondary"
                            />
                            <IconButton
                              className="shrink-0"
                              icon={<FontAwesomeIcon icon={faTrashCan} />}
                              label={copy.deleteSession.title}
                              onClick={(event) => {
                                event.stopPropagation()
                                setDeleteTarget(session)
                              }}
                              size="sm"
                              variant="danger"
                            />
                          </div>
                        </button>
                      )
                    })}
                  </div>
                )}
              </div>
            </CardContent>
          </Card>
        </WorkspacePanelShell>

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
                      label: copy.tabs.session,
                      value: 'session',
                    },
                    {
                      icon: <FontAwesomeIcon icon={faPlug} />,
                      label: copy.tabs.api,
                      value: 'api',
                    },
                  ]}
                  onValueChange={(value) => {
                    setPanelMode(value as PanelMode)
                  }}
                  value={panelMode}
                />
              }
              description={copy.stage.subtitle}
              title={selectedSession?.display_name ?? copy.stage.title}
              titleClassName="text-[1.95rem]"
            />

            <CardContent className="min-h-0 flex-1 pt-6">
              {panelMode === 'api' ? (
                selectedSession ? (
                  <div className="scrollbar-none h-full overflow-y-auto pr-1">
                    <StageSessionSettingsPanel
                      apis={apis}
                      config={selectedSession.config}
                      copy={copy}
                      currentPlayerProfileId={selectedSession.player_profile_id}
                      onRefreshSnapshot={handleRefreshRuntimeSnapshot}
                      onSavePlayerDescription={handleUpdatePlayerDescription}
                      onSavePlayerProfile={handleSetPlayerProfile}
                      onSaveSessionConfig={handleSaveSessionConfig}
                      playerProfiles={playerProfiles}
                      runtimeSnapshot={currentSnapshot}
                    />
                  </div>
                ) : (
                  <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-5 py-6 text-sm leading-7 text-[var(--color-text-secondary)]">
                    {copy.empty.stage}
                  </div>
                )
              ) : (
                <div className="flex h-full min-h-0 flex-col">
                  <div
                    className="scrollbar-none min-h-0 flex-1 overflow-y-auto pr-1"
                    onScroll={(event) => {
                      const target = event.currentTarget
                      const distanceFromBottom =
                        target.scrollHeight - target.scrollTop - target.clientHeight
                      shouldStickToBottomRef.current = distanceFromBottom < 56
                    }}
                    ref={conversationScrollRef}
                  >
                    <StageConversation
                      composerLocked={isRunningTurn}
                      characterCovers={coverCache}
                      characterMap={characterMap}
                      copy={copy}
                      deletingPlayerMessageId={deletingPlayerMessageId}
                      editingPlayerDraft={editingPlayerDraft}
                      editingPlayerMessageId={editingPlayerMessageId}
                      expandedThoughtIds={expandedThoughtIds}
                      isLoading={Boolean(routeSessionId) && isSessionLoading}
                      messages={sessionMessages}
                      onCancelEditPlayerMessage={() => {
                        setEditingPlayerMessageId(null)
                        setEditingPlayerDraft('')
                      }}
                      onChangePlayerMessageDraft={setEditingPlayerDraft}
                      onDeletePlayerMessage={(message) => void handleDeletePlayerMessage(message)}
                      onEditPlayerMessage={handleEditPlayerMessage}
                      onSavePlayerMessage={() => void handleSaveEditedPlayerMessage()}
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
                      prefersReducedMotion={prefersReducedMotion}
                      savingPlayerMessageId={savingPlayerMessageId}
                    />
                  </div>

                  <div className="relative mt-6 border-t border-[var(--color-border-subtle)] pt-7">
                    <AnimatePresence>
                      <TurnStatusBar status={overlayStatus} />
                    </AnimatePresence>
                    <div className="flex items-start gap-3">
                      <div className="min-w-0 flex-1">
                        <AnimatePresence initial={false} mode="wait">
                          {composerMode === 'input' ? (
                            <motion.div
                              animate={{ opacity: 1, x: 0 }}
                              exit={{ opacity: 0, x: -10 }}
                              initial={{ opacity: 0, x: 10 }}
                              key="composer-input"
                              transition={prefersReducedMotion ? { duration: 0 } : { duration: 0.22, ease: panelEase }}
                            >
                              <Textarea
                                className="min-h-[7.5rem] flex-1"
                                ref={composerRef}
                                onChange={(event) => {
                                  setComposerInput(event.target.value)
                                }}
                                placeholder={copy.composer.placeholder}
                                value={composerInput}
                              />
                            </motion.div>
                          ) : (
                            <motion.div
                              animate={{ opacity: 1, x: 0 }}
                              className="rounded-[1.4rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-3.5"
                              exit={{ opacity: 0, x: 10 }}
                              initial={{ opacity: 0, x: -10 }}
                              key="composer-suggestions"
                              transition={prefersReducedMotion ? { duration: 0 } : { duration: 0.22, ease: panelEase }}
                            >
                              <div className="space-y-3">
                                <div className="flex items-start justify-between gap-3">
                                  <div className="space-y-1">
                                    <p className="text-sm font-medium text-[var(--color-text-primary)]">
                                      {copy.composer.suggestions}
                                    </p>
                                    <p className="text-sm leading-6 text-[var(--color-text-secondary)]">
                                      {copy.composer.suggestionsDescription}
                                    </p>
                                  </div>

                                  <Button
                                    disabled={isSuggestingReplies || isRunningTurn}
                                    onClick={() => void handleSuggestReplies()}
                                    size="sm"
                                    variant="ghost"
                                  >
                                    {copy.composer.suggestionsGenerate}
                                  </Button>
                                </div>
                                {suggestionsError ? (
                                  <div className="rounded-[1rem] border border-[var(--color-state-error-line)] bg-[var(--color-state-error-soft)] px-3.5 py-3 text-sm leading-6 text-[var(--color-text-primary)]">
                                    {suggestionsError}
                                  </div>
                                ) : isRunningTurn ? (
                                  <div className="rounded-[1rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_80%,transparent)] px-4 py-4 text-sm leading-6 text-[var(--color-text-secondary)]">
                                    {copy.composer.suggestionsUnavailable}
                                  </div>
                                ) : null}
                                {isSuggestingReplies ? (
                                  <div className="flex min-h-[5.75rem] items-center justify-center rounded-[1rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_80%,transparent)] px-4 py-4 text-sm text-[var(--color-text-secondary)]">
                                    <span className="inline-flex items-center gap-2">
                                      <FontAwesomeIcon className="animate-spin" icon={faSpinner} />
                                      {copy.composer.suggestionsLoading}
                                    </span>
                                  </div>
                                ) : replySuggestions.length === 0 ? (
                                  <div className="rounded-[1rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_80%,transparent)] px-4 py-4 text-sm leading-6 text-[var(--color-text-secondary)]">
                                    {copy.composer.suggestionsEmpty}
                                  </div>
                                ) : (
                                  <div className="space-y-2">
                                    {replySuggestions.map((suggestion) => (
                                      <button
                                        className="w-full rounded-[1rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-3 text-left transition hover:border-[var(--color-accent-copper-soft)] hover:bg-[color-mix(in_srgb,var(--color-bg-panel)_88%,white)]"
                                        key={suggestion.reply_id}
                                        onClick={() => {
                                          handleUseReplySuggestion(suggestion)
                                        }}
                                        type="button"
                                      >
                                        <p className="text-sm leading-7 text-[var(--color-text-primary)]">
                                          {suggestion.text}
                                        </p>
                                      </button>
                                    ))}
                                  </div>
                                )}
                              </div>
                            </motion.div>
                          )}
                        </AnimatePresence>
                      </div>
                      <div className="flex w-[4.5rem] shrink-0 flex-col items-center gap-2.5">
                        <p className="w-full text-center text-xs leading-5 text-[var(--color-text-secondary)]">
                          {copy.composer.suggestions}
                        </p>
                        <Switch
                          aria-label={copy.composer.suggestions}
                          checked={replySuggestionsEnabled}
                          disabled={!selectedSession || isRunningTurn || isSuggestingReplies}
                          onCheckedChange={handleToggleReplySuggestions}
                          size="md"
                        />
                        <IconButton
                          className="w-11"
                          disabled={!selectedSession || !composerInput.trim() || isRunningTurn}
                          icon={
                            <FontAwesomeIcon
                              className={cn(isRunningTurn ? 'animate-spin' : '')}
                              icon={isRunningTurn ? faRotateRight : faPaperPlane}
                            />
                          }
                          label={isRunningTurn ? copy.composer.running : copy.composer.send}
                          onClick={() => void handleRunTurn()}
                          variant="primary"
                        />
                      </div>
                    </div>
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        </WorkspacePanelShell>

        <WorkspacePanelShell className="h-full min-h-0">
          <Card className="flex h-full min-h-0 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_92%,transparent)] shadow-none">
            <StagePanelHeader
              description={copy.rail.subtitle}
              title={copy.stage.title}
              titleClassName="text-[1.5rem]"
            />

            <CardContent className="min-h-0 flex-1 overflow-y-auto pt-6">
              <div className="space-y-6 pr-1">
                <RightPanelSection title={copy.cast.section}>
                  {activeCast.length === 0 ? (
                    <div className="rounded-[1.35rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
                      {copy.cast.empty}
                    </div>
                  ) : (
                    <LayoutGroup id="stage-cast">
                      <div className="space-y-3">
                        <AnimatePresence initial={false}>
                          {activeCast.map((character) => {
                            const isActive = character.id === activeSpeakerId

                            return (
                              <motion.button
                                animate={{ opacity: 1, y: 0 }}
                                className={cn(
                                  'w-full rounded-[1.2rem] border px-3.5 py-3 text-left',
                                  isActive
                                    ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] shadow-[0_16px_38px_var(--color-accent-glow-soft)]'
                                    : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)]',
                                )}
                                exit={prefersReducedMotion ? { opacity: 1 } : { opacity: 0, y: -12 }}
                                initial={prefersReducedMotion ? { opacity: 1, y: 0 } : { opacity: 0, y: 18 }}
                                key={character.id}
                                layout
                                onClick={() => {
                                  setDetailsCharacterId(character.id)
                                }}
                                transition={
                                  prefersReducedMotion
                                    ? { duration: 0 }
                                    : { duration: 0.24, ease: panelEase }
                                }
                                type="button"
                              >
                                <div className="flex items-start gap-2.5">
                                  <CharacterAvatar coverUrl={character.coverUrl} name={character.name} />
                                  <div className="min-w-0 flex-1 space-y-0.5">
                                    <div className="flex items-center gap-2">
                                      <p className="truncate text-sm font-medium text-[var(--color-text-primary)]">
                                        {character.name}
                                      </p>
                                      {isActive ? (
                                        <Badge variant="subtle">{copy.cast.active}</Badge>
                                      ) : null}
                                    </div>
                                    <p className="truncate font-mono text-[0.68rem] text-[var(--color-text-muted)]">
                                      {character.id}
                                    </p>
                                    <p className="line-clamp-2 text-xs leading-5 text-[var(--color-text-secondary)]">
                                      {character.description}
                                    </p>
                                  </div>
                                </div>
                              </motion.button>
                            )
                          })}
                        </AnimatePresence>
                      </div>
                    </LayoutGroup>
                  )}
                </RightPanelSection>

                <RightPanelSection
                  action={
                    hasExpandableNodeDetails ? (
                      <Button
                        onClick={() => {
                          setIsStoryNodeExpanded((current) => !current)
                        }}
                        size="sm"
                        variant="ghost"
                      >
                        {isStoryNodeExpanded ? copy.storyNode.collapse : copy.storyNode.expand}
                      </Button>
                    ) : null
                  }
                  title={copy.storyNode.section}
                >
                  <div className="space-y-3">
                    <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                      <p className="text-xs text-[var(--color-text-muted)]">{copy.storyNode.goal}</p>
                      <p className="mt-2 text-sm leading-6 text-[var(--color-text-primary)]">
                        {currentNode?.goal ?? '—'}
                      </p>
                    </div>

                    {isStoryNodeExpanded ? (
                      <>
                        <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                          <p className="text-xs text-[var(--color-text-muted)]">{copy.storyNode.nodeId}</p>
                          <p className="mt-2 font-mono text-sm text-[var(--color-text-primary)]">
                            {currentSnapshot?.world_state.current_node ?? '—'}
                          </p>
                        </div>
                        <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                          <p className="text-xs text-[var(--color-text-muted)]">{copy.storyNode.scene}</p>
                          <p className="mt-2 text-sm leading-6 text-[var(--color-text-primary)]">
                            {currentNode?.scene ?? '—'}
                          </p>
                        </div>
                      </>
                    ) : null}
                  </div>
                </RightPanelSection>

                <RightPanelSection
                  action={
                    storyIntroNeedsExpand ? (
                      <Button
                        onClick={() => {
                          setIsStoryIntroExpanded((current) => !current)
                        }}
                        size="sm"
                        variant="ghost"
                      >
                        {isStoryIntroExpanded ? copy.intro.collapse : copy.intro.expand}
                      </Button>
                    ) : null
                  }
                  title={copy.intro.section}
                >
                  <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4 text-sm leading-6 text-[var(--color-text-secondary)]">
                    {visibleStoryIntroduction}
                  </div>
                </RightPanelSection>
              </div>
            </CardContent>
          </Card>
        </WorkspacePanelShell>
      </div>
    </section>
  )
}
