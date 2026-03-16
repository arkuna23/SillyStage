import { backendPaths } from '../../app/paths'
import { rpcRequest } from '../../lib/rpc'

import type {
  SessionCharacterActionParams,
  SessionCharacter,
  SessionCharacterDeletedResult,
  SessionCharactersListedResult,
  DeleteSessionMessageParams,
  SessionMessageDeletedResult,
  SessionMessagesListedResult,
  PlayerDescriptionUpdatedResult,
  SessionConfig,
  SessionDeletedResult,
  SessionDetail,
  SessionMessageResult,
  SessionStartedResult,
  SessionStreamMessage,
  SessionsListedResult,
  SuggestedRepliesResult,
  SuggestRepliesParams,
  StartSessionInput,
  StreamAckEnvelope,
  TurnCompletedResult,
  TurnStreamAcceptedResult,
  SetPlayerProfileParams,
  RuntimeSnapshotResult,
  UpdatePlayerDescriptionParams,
  UpdateSessionCharacterParams,
  UpdateSessionMessageParams,
  UpdateSessionParams,
  UpdateSessionConfigParams,
  SessionVariablesResult,
  UpdateSessionVariablesParams,
} from './types'

const rpcEndpoint = backendPaths.rpc

let requestCounter = 0

function createRequestId() {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return `req-${crypto.randomUUID()}`
  }

  requestCounter += 1
  return `req-${Date.now()}-${requestCounter}`
}

async function readResponseError(response: Response) {
  const contentType = response.headers.get('content-type') ?? ''

  if (contentType.includes('application/json')) {
    try {
      const payload = (await response.json()) as
        | { error?: { message?: string } }
        | { message?: string }

      if ('error' in payload && payload.error?.message) {
        return new Error(payload.error.message)
      }

      if ('message' in payload && payload.message) {
        return new Error(payload.message)
      }
    } catch {
      return new Error(`RPC request failed with status ${response.status}`)
    }
  }

  const fallbackMessage = await response.text().catch(() => '')
  return new Error(fallbackMessage.trim() || `RPC request failed with status ${response.status}`)
}

type ParsedSseEvent = {
  data: string
  event: string
}

function extractSseEvents(buffer: string) {
  const events: ParsedSseEvent[] = []
  let rest = buffer

  while (true) {
    const separatorIndex = rest.indexOf('\n\n')

    if (separatorIndex === -1) {
      break
    }

    const rawEvent = rest.slice(0, separatorIndex)
    rest = rest.slice(separatorIndex + 2)

    const event = parseSseEvent(rawEvent)

    if (event) {
      events.push(event)
    }
  }

  return { events, rest }
}

function parseSseEvent(rawEvent: string): ParsedSseEvent | null {
  let eventName = 'message'
  const dataLines: string[] = []

  for (const line of rawEvent.split(/\r?\n/)) {
    if (!line || line.startsWith(':')) {
      continue
    }

    if (line.startsWith('event:')) {
      eventName = line.slice(6).trim()
      continue
    }

    if (line.startsWith('data:')) {
      dataLines.push(line.slice(5).trimStart())
    }
  }

  if (dataLines.length === 0) {
    return null
  }

  return {
    data: dataLines.join('\n'),
    event: eventName,
  }
}

export async function listSessions(signal?: AbortSignal) {
  const result = await rpcRequest<Record<string, never>, SessionsListedResult>(
    'session.list',
    {},
    { signal },
  )

  return result.sessions
}

export async function getSession(sessionId: string, signal?: AbortSignal) {
  return rpcRequest<Record<string, never>, SessionDetail>('session.get', {}, { signal, sessionId })
}

export async function listSessionMessages(sessionId: string, signal?: AbortSignal) {
  const result = await rpcRequest<Record<string, never>, SessionMessagesListedResult>(
    'session_message.list',
    {},
    { signal, sessionId },
  )

  return result.messages
}

export async function getSessionCharacter(
  sessionId: string,
  sessionCharacterId: string,
  signal?: AbortSignal,
) {
  return rpcRequest<{ session_character_id: string }, SessionCharacter>(
    'session_character.get',
    { session_character_id: sessionCharacterId },
    { signal, sessionId },
  )
}

export async function listSessionCharacters(sessionId: string, signal?: AbortSignal) {
  const result = await rpcRequest<Record<string, never>, SessionCharactersListedResult>(
    'session_character.list',
    {},
    { signal, sessionId },
  )

  return result.session_characters
}

export async function updateSessionCharacter(
  sessionId: string,
  params: UpdateSessionCharacterParams,
  signal?: AbortSignal,
) {
  return rpcRequest<UpdateSessionCharacterParams, SessionCharacter>('session_character.update', params, {
    signal,
    sessionId,
  })
}

export async function deleteSessionCharacter(
  sessionId: string,
  params: SessionCharacterActionParams,
  signal?: AbortSignal,
) {
  return rpcRequest<SessionCharacterActionParams, SessionCharacterDeletedResult>(
    'session_character.delete',
    params,
    {
      signal,
      sessionId,
    },
  )
}

export async function enterSessionCharacterScene(
  sessionId: string,
  params: SessionCharacterActionParams,
  signal?: AbortSignal,
) {
  return rpcRequest<SessionCharacterActionParams, SessionCharacter>('session_character.enter_scene', params, {
    signal,
    sessionId,
  })
}

export async function leaveSessionCharacterScene(
  sessionId: string,
  params: SessionCharacterActionParams,
  signal?: AbortSignal,
) {
  return rpcRequest<SessionCharacterActionParams, SessionCharacter>('session_character.leave_scene', params, {
    signal,
    sessionId,
  })
}

export async function updateSessionMessage(
  sessionId: string,
  params: UpdateSessionMessageParams,
  signal?: AbortSignal,
) {
  return rpcRequest<UpdateSessionMessageParams, SessionMessageResult>('session_message.update', params, {
    signal,
    sessionId,
  })
}

export async function deleteSessionMessage(
  sessionId: string,
  params: DeleteSessionMessageParams,
  signal?: AbortSignal,
) {
  return rpcRequest<DeleteSessionMessageParams, SessionMessageDeletedResult>(
    'session_message.delete',
    params,
    {
      signal,
      sessionId,
    },
  )
}

export async function deleteSession(sessionId: string, signal?: AbortSignal) {
  return rpcRequest<Record<string, never>, SessionDeletedResult>(
    'session.delete',
    {},
    { signal, sessionId },
  )
}

export async function updateSession(
  sessionId: string,
  params: UpdateSessionParams,
  signal?: AbortSignal,
) {
  return rpcRequest<UpdateSessionParams, SessionDetail>('session.update', params, {
    signal,
    sessionId,
  })
}

export async function setSessionPlayerProfile(
  sessionId: string,
  params: SetPlayerProfileParams,
  signal?: AbortSignal,
) {
  return rpcRequest<SetPlayerProfileParams, SessionDetail>('session.set_player_profile', params, {
    signal,
    sessionId,
  })
}

export async function updateSessionPlayerDescription(
  sessionId: string,
  params: UpdatePlayerDescriptionParams,
  signal?: AbortSignal,
) {
  return rpcRequest<UpdatePlayerDescriptionParams, PlayerDescriptionUpdatedResult>(
    'session.update_player_description',
    params,
    {
      signal,
      sessionId,
    },
  )
}

export async function getRuntimeSnapshot(sessionId: string, signal?: AbortSignal) {
  return rpcRequest<Record<string, never>, RuntimeSnapshotResult>(
    'session.get_runtime_snapshot',
    {},
    {
      signal,
      sessionId,
    },
  )
}

export async function getSessionVariables(sessionId: string, signal?: AbortSignal) {
  return rpcRequest<Record<string, never>, SessionVariablesResult>(
    'session.get_variables',
    {},
    {
      signal,
      sessionId,
    },
  )
}

export async function updateSessionVariables(
  sessionId: string,
  params: UpdateSessionVariablesParams,
  signal?: AbortSignal,
) {
  return rpcRequest<UpdateSessionVariablesParams, SessionVariablesResult>(
    'session.update_variables',
    params,
    {
      signal,
      sessionId,
    },
  )
}

export async function suggestSessionReplies(
  sessionId: string,
  params: SuggestRepliesParams,
  signal?: AbortSignal,
) {
  return rpcRequest<SuggestRepliesParams, SuggestedRepliesResult>('session.suggest_replies', params, {
    signal,
    sessionId,
  })
}

export async function startSessionFromStory(params: StartSessionInput, signal?: AbortSignal) {
  const request = {
    id: createRequestId(),
    jsonrpc: '2.0' as const,
    method: 'story.start_session',
    params,
  }

  const response = await fetch(rpcEndpoint, {
    body: JSON.stringify(request),
    headers: {
      'Content-Type': 'application/json',
    },
    method: 'POST',
    signal,
  })

  if (!response.ok) {
    throw await readResponseError(response)
  }

  const payload = (await response.json()) as StreamAckEnvelope<SessionStartedResult>

  if ('error' in payload) {
    throw new Error(payload.error.message)
  }

  return {
    ...payload.result,
    session_id: payload.session_id ?? '',
  }
}

export const startSession = startSessionFromStory

export async function updateSessionConfig(
  sessionId: string,
  params: UpdateSessionConfigParams,
  signal?: AbortSignal,
) {
  return rpcRequest<UpdateSessionConfigParams, SessionConfig>('session.update_config', params, {
    signal,
    sessionId,
  })
}

export async function runSessionTurnStream(args: {
  onAck?: (ack: TurnStreamAcceptedResult) => void
  onMessage?: (message: SessionStreamMessage) => void
  playerInput: string
  sessionId: string
  signal?: AbortSignal
}) {
  const request = {
    id: createRequestId(),
    jsonrpc: '2.0' as const,
    method: 'session.run_turn',
    params: {
      player_input: args.playerInput,
    },
    session_id: args.sessionId,
  }

  const response = await fetch(rpcEndpoint, {
    body: JSON.stringify(request),
    headers: {
      'Content-Type': 'application/json',
    },
    method: 'POST',
    signal: args.signal,
  })

  if (!response.ok) {
    throw await readResponseError(response)
  }

  if (!response.body) {
    throw new Error('The stage stream did not return a readable body.')
  }

  const reader = response.body.getReader()
  const decoder = new TextDecoder()
  let buffer = ''
  let finalResult: TurnCompletedResult | null = null

  while (true) {
    const { done, value } = await reader.read()

    if (done) {
      buffer += decoder.decode()
    } else if (value) {
      buffer += decoder.decode(value, { stream: true })
    }

    const { events, rest } = extractSseEvents(buffer)
    buffer = rest

    for (const event of events) {
      if (event.event === 'ack') {
        const payload = JSON.parse(event.data) as StreamAckEnvelope<TurnStreamAcceptedResult>

        if ('error' in payload) {
          throw new Error(payload.error.message)
        }

        args.onAck?.(payload.result)
        continue
      }

      if (event.event !== 'message') {
        continue
      }

      const message = JSON.parse(event.data) as SessionStreamMessage
      args.onMessage?.(message)

      if (message.frame.type === 'failed') {
        throw new Error(message.frame.error.message)
      }

      if (message.frame.type === 'completed') {
        if (message.frame.response.type !== 'turn_completed') {
          throw new Error('The stage stream ended without a turn completion payload.')
        }

        finalResult = message.frame.response
      }
    }

    if (done) {
      break
    }
  }

  if (!finalResult) {
    throw new Error('The turn stream completed without a final result.')
  }

  return finalResult
}
