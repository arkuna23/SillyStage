import { backendPaths } from '../app/paths'

const rpcEndpoint = backendPaths.rpc

let requestCounter = 0

type JsonRpcVersion = '2.0'

type RpcRequestEnvelope<TParams> = {
  id: string
  jsonrpc: JsonRpcVersion
  method: string
  params: TParams
  session_id?: string
}

type RpcSuccessEnvelope<TResult> = {
  id: string
  jsonrpc: JsonRpcVersion
  result: TResult
  session_id?: string
}

type RpcErrorPayload = {
  code: number
  data?: unknown
  message: string
}

type RpcErrorEnvelope = {
  error: RpcErrorPayload
  id: string | null
  jsonrpc: JsonRpcVersion
}

type RpcEnvelope<TResult> = RpcSuccessEnvelope<TResult> | RpcErrorEnvelope

export class RpcError extends Error {
  code: number
  data?: unknown

  constructor({ code, data, message }: RpcErrorPayload) {
    super(message)
    this.code = code
    this.data = data
    this.name = 'RpcError'
  }
}

function createRequestId() {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return `req-${crypto.randomUUID()}`
  }

  requestCounter += 1
  return `req-${Date.now()}-${requestCounter}`
}

async function readErrorBody(response: Response) {
  const contentType = response.headers.get('content-type') ?? ''

  if (contentType.includes('application/json')) {
    try {
      const payload = (await response.json()) as Partial<RpcErrorEnvelope>

      if (payload.error) {
        return new RpcError(payload.error)
      }
    } catch {
      return new Error(`RPC request failed with status ${response.status}`)
    }
  }

  const fallbackMessage = await response.text().catch(() => '')
  return new Error(fallbackMessage.trim() || `RPC request failed with status ${response.status}`)
}

export async function rpcRequest<TParams, TResult>(
  method: string,
  params: TParams,
  options?: {
    sessionId?: string
    signal?: AbortSignal
  },
) {
  const request: RpcRequestEnvelope<TParams> = {
    id: createRequestId(),
    jsonrpc: '2.0',
    method,
    params,
    ...(options?.sessionId ? { session_id: options.sessionId } : {}),
  }

  const response = await fetch(rpcEndpoint, {
    body: JSON.stringify(request),
    headers: {
      'Content-Type': 'application/json',
    },
    method: 'POST',
    signal: options?.signal,
  })

  if (!response.ok) {
    throw await readErrorBody(response)
  }

  const payload = (await response.json()) as RpcEnvelope<TResult>

  if ('error' in payload) {
    throw new RpcError(payload.error)
  }

  return payload.result
}

export function isRpcError(error: unknown): error is RpcError {
  return error instanceof RpcError
}

export function isRpcConflict(error: unknown): error is RpcError {
  return isRpcError(error) && error.code === 40909
}
