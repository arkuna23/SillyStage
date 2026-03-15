import type { ApiGroup, Preset } from '../apis/types'
import type { CharacterSummary } from '../characters/types'
import type { StoryDetail, StorySummary } from '../stories/types'

export type SessionConfig = {
  api_group_id: string
  preset_id: string
  type?: 'session_config'
}

export type SessionHistoryEntryKind = 'player_input' | 'narration' | 'dialogue' | 'action'
export type ActorMemoryKind = 'player_input' | 'dialogue' | 'thought' | 'action'
export type ActorPurpose = 'advance_goal' | 'react_to_player' | 'comment_on_scene'
export type NarratorPurpose = 'describe_transition' | 'describe_scene' | 'describe_result'
export type ActorSegmentKind = 'dialogue' | 'thought' | 'action'

export type SessionHistoryEntry = {
  client_id?: string
  created_at_ms?: number
  kind: SessionHistoryEntryKind
  message_id?: string
  recorded_at_ms: number
  sequence?: number
  speaker_id: string
  speaker_name: string
  text: string
  turn_index: number
  updated_at_ms?: number
}

export type SessionMessageResult = SessionHistoryEntry & {
  type?: 'session_message'
}

export type SessionMessagesListedResult = {
  messages: SessionMessageResult[]
  type: 'session_messages_listed'
}

export type SessionMessageDeletedResult = {
  message_id: string
  type: 'session_message_deleted'
}

export type SessionVariables = {
  character_state: Record<string, Record<string, unknown>>
  custom: Record<string, unknown>
  player_state: Record<string, unknown>
}

export type SessionVariablesResult = SessionVariables & {
  type: 'session_variables'
}

export type VariableStateOp =
  | {
      key: string
      type: 'RemoveState'
    }
  | {
      key: string
      type: 'RemovePlayerState'
    }
  | {
      character: string
      key: string
      type: 'RemoveCharacterState'
    }
  | {
      key: string
      type: 'SetState'
      value: unknown
    }
  | {
      key: string
      type: 'SetPlayerState'
      value: unknown
    }
  | {
      character: string
      key: string
      type: 'SetCharacterState'
      value: unknown
    }

export type StateUpdate = {
  ops: VariableStateOp[]
}

export type UpdateSessionVariablesParams = {
  update: StateUpdate
}

export type UpdateSessionMessageParams = {
  kind: SessionHistoryEntryKind
  message_id: string
  speaker_id: string
  speaker_name: string
  text: string
}

export type DeleteSessionMessageParams = {
  message_id: string
}

export type ReplySuggestion = {
  reply_id: string
  text: string
}

export type SuggestedRepliesResult = {
  replies: ReplySuggestion[]
  type: 'suggested_replies'
}

export type SuggestRepliesParams = {
  limit?: number
}

export type ActorMemoryEntry = {
  kind: ActorMemoryKind
  speaker_id: string
  speaker_name: string
  text: string
}

export type SessionWorldState = {
  active_characters: string[]
  actor_private_memory?: Record<string, ActorMemoryEntry[]>
  actor_shared_history: ActorMemoryEntry[]
  character_state: Record<string, Record<string, unknown>>
  current_node: string
  custom: Record<string, unknown>
  player_state: Record<string, unknown>
}

export type RuntimeSnapshot = {
  player_description: string
  story_id: string
  turn_index: number
  world_state: SessionWorldState
}

export type SessionSummary = {
  api_group_id: string
  created_at_ms?: number | null
  display_name: string
  player_profile_id?: string | null
  player_schema_id: string
  preset_id: string
  session_id: string
  story_id: string
  turn_index: number
  updated_at_ms?: number | null
}

export type SessionDetail = SessionSummary & {
  config: SessionConfig
  history: SessionHistoryEntry[]
  snapshot: RuntimeSnapshot
  type: 'session'
}

export type SessionsListedResult = {
  sessions: SessionSummary[]
  type: 'sessions_listed'
}

export type SessionDeletedResult = {
  session_id: string
  type: 'session_deleted'
}

export type SessionStartedResult = {
  api_group_id: string
  character_summaries: CharacterSummary[]
  config: SessionConfig
  created_at_ms?: number | null
  display_name: string
  history: SessionHistoryEntry[]
  player_profile_id?: string | null
  player_schema_id: string
  preset_id: string
  session_id: string
  snapshot: RuntimeSnapshot
  story_id: string
  type: 'session_started'
  updated_at_ms?: number | null
}

export type StartedSession = SessionStartedResult & {
  session_id: string
}

export type StartSessionInput = {
  api_group_id: string
  display_name?: string
  player_profile_id?: string
  preset_id: string
  story_id: string
}

export type RunTurnInput = {
  player_input: string
}

export type UpdateSessionConfigParams = {
  api_group_id?: string
  preset_id?: string
}

export type UpdateSessionParams = {
  display_name: string
}

export type SetPlayerProfileParams = {
  player_profile_id?: string
}

export type UpdatePlayerDescriptionParams = {
  player_description: string
}

export type PlayerDescriptionUpdatedResult = {
  snapshot: RuntimeSnapshot
  type: 'player_description_updated'
}

export type ResponseBeat =
  | {
      purpose: NarratorPurpose
      type: 'narrator'
    }
  | {
      purpose: ActorPurpose
      speaker_id: string
      type: 'actor'
    }

export type DirectorResult = {
  current_node_id: string
  previous_node_id: string
  response_plan: {
    beats: ResponseBeat[]
  }
  transitioned: boolean
}

export type ActorSegment = {
  kind: ActorSegmentKind
  text: string
}

export type ActorResponse = {
  raw_output: string
  segments: ActorSegment[]
  speaker_id: string
  speaker_name: string
}

export type NarratorResponse = {
  raw_output: string
  text: string
}

export type ExecutedBeat =
  | {
      purpose: NarratorPurpose
      response: NarratorResponse
      type: 'narrator'
    }
  | {
      purpose: ActorPurpose
      response: ActorResponse
      speaker_id: string
      type: 'actor'
    }

export type EngineTurnResult = {
  completed_beats: ExecutedBeat[]
  director: DirectorResult
  first_keeper?: unknown
  player_input: string
  second_keeper?: unknown
  snapshot: RuntimeSnapshot
  turn_index: number
}

export type TurnStreamAcceptedResult = {
  type: 'turn_stream_accepted'
}

export type TurnCompletedResult = {
  result: EngineTurnResult
  type: 'turn_completed'
}

export type RuntimeSnapshotResult = {
  snapshot: RuntimeSnapshot
  type: 'runtime_snapshot'
}

export type StreamEventBody =
  | {
      next_turn_index: number
      player_input: string
      type: 'turn_started'
    }
  | {
      entry: ActorMemoryEntry
      snapshot: RuntimeSnapshot
      type: 'player_input_recorded'
    }
  | {
      phase: string
      snapshot: RuntimeSnapshot
      type: 'keeper_applied'
      update: unknown
    }
  | {
      result: DirectorResult
      snapshot: RuntimeSnapshot
      type: 'director_completed'
    }
  | {
      beat_index: number
      purpose: NarratorPurpose
      type: 'narrator_started'
    }
  | {
      beat_index: number
      delta: string
      purpose: NarratorPurpose
      type: 'narrator_text_delta'
    }
  | {
      beat_index: number
      purpose: NarratorPurpose
      response: NarratorResponse
      type: 'narrator_completed'
    }
  | {
      beat_index: number
      purpose: ActorPurpose
      speaker_id: string
      type: 'actor_started'
    }
  | {
      beat_index: number
      delta: string
      speaker_id: string
      type: 'actor_thought_delta'
    }
  | {
      beat_index: number
      speaker_id: string
      text: string
      type: 'actor_action_complete'
    }
  | {
      beat_index: number
      delta: string
      speaker_id: string
      type: 'actor_dialogue_delta'
    }
  | {
      beat_index: number
      purpose: ActorPurpose
      response: ActorResponse
      speaker_id: string
      type: 'actor_completed'
    }

export type StreamFrame =
  | { type: 'started' }
  | { body: StreamEventBody; type: 'event' }
  | { response: TurnCompletedResult | RuntimeSnapshotResult; type: 'completed' }
  | { error: { code: number; data?: unknown; message: string }; type: 'failed' }

export type ServerEventMessage = {
  frame: StreamFrame
  message_type: 'stream'
  request_id: string
  sequence: number
  session_id?: string | null
}

export type TurnStreamAck = {
  id: string
  jsonrpc: '2.0'
  result: TurnStreamAcceptedResult
  session_id?: string
}

export type StreamAckEnvelope<TResult> =
  | {
      id: string
      jsonrpc: '2.0'
      result: TResult
      session_id?: string
    }
  | {
      error: {
        code: number
        data?: unknown
        message: string
      }
      id: string | null
      jsonrpc: '2.0'
      session_id?: string
    }

export type SessionStreamMessage = ServerEventMessage

export type SessionListItem = SessionSummary & {
  intro_excerpt: string
  story_display_name: string
}

export type StageCharacterSummary = CharacterSummary & {
  cover_url?: string
}

export type StageStoryRecord = StoryDetail | StorySummary

export type StageApiBindingResource = {
  apiGroups: ApiGroup[]
  presets: Preset[]
}
