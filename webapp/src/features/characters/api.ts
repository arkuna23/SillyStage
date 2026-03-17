import { rpcRequest } from '../../lib/rpc'
import {
  createObjectUrl,
  downloadBinaryResource,
  triggerBlobDownload,
  uploadBinaryResource,
} from '../../lib/binary-resource'
import { buildCharacterSummaryFromArchive, parseCharacterArchive } from './character-archive'
import type {
  CharacterCardContent,
  CharacterCreateResult,
  CharacterDeletedResult,
  CharacterSchemaResult,
  CharactersListedResult,
  CharacterSummary,
  CharacterCoverMimeType,
  ResourceFilePayload,
} from './types'

const characterArchiveContentType = 'application/x-sillystage-character-card'
const supportedImportExtension = '.chr'

function buildCharacterResourceId(characterId: string) {
  return `character:${characterId}`
}

function createSummaryFromCharacter(character: CharacterSchemaResult): CharacterSummary {
  return {
    character_id: character.character_id,
    cover_file_name: character.cover_file_name,
    cover_mime_type: character.cover_mime_type,
    name: character.content.name,
    personality: character.content.personality,
    style: character.content.style,
  }
}

function isCharacterCoverMimeType(value: string): value is CharacterCoverMimeType {
  return value === 'image/png' || value === 'image/jpeg' || value === 'image/webp'
}

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
  return downloadBinaryResource({
    fileId: 'cover',
    resourceId: buildCharacterResourceId(characterId),
    signal,
  })
}

export async function getCharacterCoverUrl(characterId: string, signal?: AbortSignal) {
  const cover = await getCharacterCover(characterId, signal)
  return createObjectUrl(cover.blob)
}

export async function createCharacter(content: CharacterCardContent, signal?: AbortSignal) {
  return rpcRequest<{ content: CharacterCardContent }, CharacterCreateResult>(
    'character.create',
    { content },
    { signal },
  )
}

export async function getCharacter(characterId: string, signal?: AbortSignal) {
  return rpcRequest<{ character_id: string }, CharacterSchemaResult>(
    'character.get',
    { character_id: characterId },
    { signal },
  )
}

export async function updateCharacter(args: {
  characterId: string
  content: CharacterCardContent
  signal?: AbortSignal
}) {
  return rpcRequest<{ character_id: string; content: CharacterCardContent }, CharacterSchemaResult>(
    'character.update',
    {
      character_id: args.characterId,
      content: args.content,
    },
    { signal: args.signal },
  )
}

export async function setCharacterCover(args: {
  characterId: string
  coverFile: File
  signal?: AbortSignal
}) {
  return uploadBinaryResource<ResourceFilePayload>({
    body: args.coverFile,
    contentType: args.coverFile.type,
    fileId: 'cover',
    fileName: args.coverFile.name,
    resourceId: buildCharacterResourceId(args.characterId),
    signal: args.signal,
  })
}

export async function exportCharacterArchive(characterId: string, signal?: AbortSignal) {
  return downloadBinaryResource({
    fileId: 'archive',
    resourceId: buildCharacterResourceId(characterId),
    signal,
  })
}

export async function downloadCharacterArchive(characterId: string, signal?: AbortSignal) {
  const result = await exportCharacterArchive(characterId, signal)

  triggerBlobDownload({
    blob: result.blob,
    fileName: result.fileName ?? `${characterId}.chr`,
  })

  return result
}

export async function importCharacterArchive(file: File) {
  if (!hasCharacterCardExtension(file.name)) {
    throw new Error('Only .chr files are supported.')
  }

  const archive = await parseCharacterArchive(file)

  await uploadBinaryResource<ResourceFilePayload>({
    body: file,
    contentType: file.type || characterArchiveContentType,
    fileId: 'archive',
    fileName: file.name,
    resourceId: buildCharacterResourceId(archive.content.id),
  })

  try {
    const importedCharacter = await getCharacter(archive.content.id)
    return createSummaryFromCharacter(importedCharacter)
  } catch {
    return buildCharacterSummaryFromArchive(archive)
  }
}

export async function deleteCharacter(characterId: string, signal?: AbortSignal) {
  return rpcRequest<{ character_id: string }, CharacterDeletedResult>(
    'character.delete',
    { character_id: characterId },
    { signal },
  )
}

export function withUpdatedCoverSummary(
  summary: CharacterSummary,
  cover: ResourceFilePayload,
) {
  return {
    ...summary,
    cover_file_name: cover.file_name,
    cover_mime_type: isCharacterCoverMimeType(cover.content_type)
      ? cover.content_type
      : summary.cover_mime_type,
  }
}
