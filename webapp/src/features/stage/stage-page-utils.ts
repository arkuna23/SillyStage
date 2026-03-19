import type { CharacterSummary } from '../characters/types'
import type { StoryDetail } from '../stories/types'
import type { StageMessage } from './stage-ui-types'
import type {
  EngineTurnResult,
  RuntimeSnapshot,
  SessionHistoryEntry,
  SessionMessageResult,
  SessionVariables,
} from './types'

const stageRoot = '/stage'

export function buildStagePath(sessionId?: string) {
  return sessionId ? `${stageRoot}/${encodeURIComponent(sessionId)}` : stageRoot
}

export function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

export function summarizeText(text: string, maxLength = 120) {
  const normalized = text.replace(/\s+/g, ' ').trim()

  if (normalized.length <= maxLength) {
    return normalized
  }

  return `${normalized.slice(0, maxLength).trimEnd()}…`
}

export function isTextLong(text: string, maxLength: number) {
  return text.replace(/\s+/g, ' ').trim().length > maxLength
}

export function isScrolledNearBottom(element: HTMLElement, threshold = 56) {
  const distanceFromBottom = element.scrollHeight - element.scrollTop - element.clientHeight
  return distanceFromBottom < threshold
}

export function buildPersistedMessages(history: SessionHistoryEntry[]): StageMessage[] {
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

export function normalizeSessionHistory(history: SessionMessageResult[]) {
  return history.map((entry, index) => ({
    ...entry,
    client_id: entry.client_id ?? entry.message_id ?? `persisted:${entry.turn_index}:${index}`,
  }))
}

export function buildCharacterMap(characters: CharacterSummary[]) {
  return new Map(characters.map((character) => [character.character_id, character]))
}

export function determineActiveCastOrder(args: {
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

export function getStoryNode(story: StoryDetail | null, snapshot: RuntimeSnapshot | null) {
  if (!story || !snapshot) {
    return null
  }

  return story.graph.nodes.find((node) => node.id === snapshot.world_state.current_node) ?? null
}

export function patchSnapshotVariables(snapshot: RuntimeSnapshot, variables: SessionVariables): RuntimeSnapshot {
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

export function patchSnapshotActiveCharacter(
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

export function createInitialThoughtState() {
  return new Set<string>()
}

export function buildHistoryEntriesFromTurnResult(args: {
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
