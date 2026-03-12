import {
  downloadBase64File,
  fileToBase64,
  fileToBytes,
  sha256Hex,
  toDataUrl,
  bytesToBase64,
} from '../../lib/base64'
import { rpcRequest } from '../../lib/rpc'
import type {
  CharacterCardContent,
  CharacterCardUploadedResult,
  CharacterCoverMimeType,
  CharacterCoverResult,
  CharacterCoverUpdatedResult,
  CharacterCreateResult,
  CharacterExportResult,
  CharactersListedResult,
  CharacterSummary,
  UploadChunkAcceptedResult,
  UploadInitializedResult,
} from './types'

const fallbackArchiveContentType = 'application/octet-stream'
const supportedImportExtension = '.chr'

export function hasCharacterCardExtension(fileName: string) {
  return fileName.toLowerCase().endsWith(supportedImportExtension)
}

export async function listCharacters(signal?: AbortSignal) {
  const result = await rpcRequest<Record<string, never>, CharactersListedResult>(
    'character.list',
    {},
    { signal },
  )

  return result.characters
}

export async function getCharacterCover(characterId: string, signal?: AbortSignal) {
  return rpcRequest<{ character_id: string }, CharacterCoverResult>(
    'character.get_cover',
    { character_id: characterId },
    { signal },
  )
}

export async function createCharacter(content: CharacterCardContent, signal?: AbortSignal) {
  return rpcRequest<{ content: CharacterCardContent }, CharacterCreateResult>(
    'character.create',
    { content },
    { signal },
  )
}

export async function setCharacterCover(args: {
  characterId: string
  coverFile: File
  signal?: AbortSignal
}) {
  const coverBase64 = await fileToBase64(args.coverFile)

  return rpcRequest<
    {
      character_id: string
      cover_base64: string
      cover_mime_type: CharacterCoverMimeType
    },
    CharacterCoverUpdatedResult
  >(
    'character.set_cover',
    {
      character_id: args.characterId,
      cover_base64: coverBase64,
      cover_mime_type: args.coverFile.type as CharacterCoverMimeType,
    },
    { signal: args.signal },
  )
}

export async function exportCharacterArchive(characterId: string, signal?: AbortSignal) {
  return rpcRequest<{ character_id: string }, CharacterExportResult>(
    'character.export_chr',
    { character_id: characterId },
    { signal },
  )
}

export async function downloadCharacterArchive(characterId: string, signal?: AbortSignal) {
  const result = await exportCharacterArchive(characterId, signal)

  downloadBase64File({
    base64: result.chr_base64,
    contentType: result.content_type,
    fileName: result.file_name,
  })

  return result
}

export async function importCharacterArchive(file: File) {
  if (!hasCharacterCardExtension(file.name)) {
    throw new Error('Only .chr files are supported.')
  }

  const bytes = await fileToBytes(file)

  if (bytes.byteLength === 0) {
    throw new Error('Cannot import an empty character card.')
  }

  const sha256 = await sha256Hex(bytes)
  const initialized = await rpcRequest<
    {
      content_type: string
      file_name: string
      sha256: string
      target_kind: 'character_card'
      total_size: number
    },
    UploadInitializedResult
  >('upload.init', {
    content_type: file.type || fallbackArchiveContentType,
    file_name: file.name,
    sha256,
    target_kind: 'character_card',
    total_size: bytes.byteLength,
  })

  const chunkSize = Math.max(1, Number(initialized.chunk_size_hint) || 65536)
  let chunkIndex = 0
  let offset = 0

  while (offset < bytes.byteLength) {
    const nextOffset = Math.min(offset + chunkSize, bytes.byteLength)
    const chunk = bytes.slice(offset, nextOffset)

    await rpcRequest<
      {
        chunk_index: number
        is_last: boolean
        offset: number
        payload_base64: string
        upload_id: string
      },
      UploadChunkAcceptedResult
    >('upload.chunk', {
      chunk_index: chunkIndex,
      is_last: nextOffset >= bytes.byteLength,
      offset,
      payload_base64: bytesToBase64(chunk),
      upload_id: initialized.upload_id,
    })

    chunkIndex += 1
    offset = nextOffset
  }

  const completed = await rpcRequest<
    { upload_id: string },
    CharacterCardUploadedResult
  >('upload.complete', { upload_id: initialized.upload_id })

  return completed.character_summary
}

export function createCoverDataUrl(args: {
  coverBase64: string
  coverMimeType: CharacterCoverMimeType
}) {
  return toDataUrl(args.coverBase64, args.coverMimeType)
}

export function withUpdatedCoverSummary(
  summary: CharacterSummary,
  cover: CharacterCoverUpdatedResult,
) {
  return {
    ...summary,
    cover_file_name: cover.cover_file_name,
    cover_mime_type: cover.cover_mime_type,
  }
}
