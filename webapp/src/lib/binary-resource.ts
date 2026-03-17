import { backendPaths } from '../app/paths'
import { RpcError } from './rpc'

type ErrorPayload = {
  code: number
  data?: unknown
  message: string
}

export type BinaryDownloadResult = {
  blob: Blob
  contentType: string
  fileName: string | null
}

function parseContentDispositionFileName(contentDisposition: string | null) {
  if (!contentDisposition) {
    return null
  }

  const utf8Match = contentDisposition.match(/filename\*=UTF-8''([^;]+)/i)

  if (utf8Match) {
    try {
      return decodeURIComponent(utf8Match[1])
    } catch {
      return utf8Match[1]
    }
  }

  const quotedMatch = contentDisposition.match(/filename="([^"]+)"/i)

  if (quotedMatch) {
    return quotedMatch[1]
  }

  const unquotedMatch = contentDisposition.match(/filename=([^;]+)/i)
  return unquotedMatch ? unquotedMatch[1].trim() : null
}

async function readBinaryErrorBody(response: Response) {
  const contentType = response.headers.get('content-type') ?? ''

  if (contentType.includes('application/json')) {
    try {
      const payload = (await response.json()) as Partial<ErrorPayload>

      if (
        typeof payload.code === 'number' &&
        typeof payload.message === 'string'
      ) {
        return new RpcError({
          code: payload.code,
          data: payload.data,
          message: payload.message,
        })
      }
    } catch {
      return new Error(`Binary request failed with status ${response.status}`)
    }
  }

  const fallbackMessage = await response.text().catch(() => '')
  return new Error(
    fallbackMessage.trim() || `Binary request failed with status ${response.status}`,
  )
}

export async function uploadBinaryResource<TResult>(args: {
  body: Blob | Uint8Array | ArrayBuffer
  contentType: string
  fileId: string
  fileName?: string | null
  resourceId: string
  signal?: AbortSignal
}) {
  const requestBody =
    args.body instanceof Uint8Array
      ? (() => {
          const bytes = new Uint8Array(args.body.byteLength)
          bytes.set(args.body)
          return bytes
        })()
      : args.body

  const response = await fetch(backendPaths.upload(args.resourceId, args.fileId), {
    body: requestBody,
    headers: {
      'Content-Type': args.contentType,
      ...(args.fileName?.trim() ? { 'x-file-name': args.fileName.trim() } : {}),
    },
    method: 'POST',
    signal: args.signal,
  })

  if (!response.ok) {
    throw await readBinaryErrorBody(response)
  }

  return (await response.json()) as TResult
}

export async function downloadBinaryResource(args: {
  fileId: string
  resourceId: string
  signal?: AbortSignal
}) {
  const response = await fetch(backendPaths.download(args.resourceId, args.fileId), {
    method: 'GET',
    signal: args.signal,
  })

  if (!response.ok) {
    throw await readBinaryErrorBody(response)
  }

  const blob = await response.blob()

  return {
    blob,
    contentType: response.headers.get('content-type') ?? blob.type,
    fileName: parseContentDispositionFileName(
      response.headers.get('content-disposition'),
    ),
  } satisfies BinaryDownloadResult
}

export function createObjectUrl(blob: Blob) {
  return URL.createObjectURL(blob)
}

export function revokeObjectUrl(objectUrl: string | null | undefined) {
  if (!objectUrl) {
    return
  }

  URL.revokeObjectURL(objectUrl)
}

export function triggerBlobDownload(args: {
  blob: Blob
  fileName: string
}) {
  const objectUrl = createObjectUrl(args.blob)
  const anchor = document.createElement('a')

  anchor.href = objectUrl
  anchor.download = args.fileName
  anchor.click()

  window.setTimeout(() => {
    revokeObjectUrl(objectUrl)
  }, 0)
}
