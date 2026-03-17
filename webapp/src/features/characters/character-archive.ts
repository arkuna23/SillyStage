import type { CharacterCardContent, CharacterCoverMimeType, CharacterSummary } from './types'

const EOCD_SIGNATURE = 0x06054b50
const CENTRAL_DIRECTORY_SIGNATURE = 0x02014b50
const LOCAL_FILE_HEADER_SIGNATURE = 0x04034b50
const ARCHIVE_FORMAT = 'sillystage_character_card'
const ARCHIVE_VERSION = 1

type CharacterArchiveManifest = {
  character_id: string
  content_path: string
  cover_mime_type: CharacterCoverMimeType
  cover_path: string
  format: string
  version: number
}

type ZipEntry = {
  compressedSize: number
  compressionMethod: number
  fileName: string
  localHeaderOffset: number
}

function decodeZipText(bytes: Uint8Array) {
  return new TextDecoder().decode(bytes)
}

function findEndOfCentralDirectory(view: DataView) {
  const minimumOffset = Math.max(0, view.byteLength - 0xffff - 22)

  for (let offset = view.byteLength - 22; offset >= minimumOffset; offset -= 1) {
    if (view.getUint32(offset, true) === EOCD_SIGNATURE) {
      return offset
    }
  }

  throw new Error('Invalid character archive: end of central directory not found.')
}

function readZipEntries(bytes: Uint8Array) {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength)
  const eocdOffset = findEndOfCentralDirectory(view)
  const entryCount = view.getUint16(eocdOffset + 10, true)
  const centralDirectoryOffset = view.getUint32(eocdOffset + 16, true)
  const entries: ZipEntry[] = []
  let cursor = centralDirectoryOffset

  for (let index = 0; index < entryCount; index += 1) {
    if (view.getUint32(cursor, true) !== CENTRAL_DIRECTORY_SIGNATURE) {
      throw new Error('Invalid character archive: bad central directory entry.')
    }

    const compressionMethod = view.getUint16(cursor + 10, true)
    const compressedSize = view.getUint32(cursor + 20, true)
    const fileNameLength = view.getUint16(cursor + 28, true)
    const extraFieldLength = view.getUint16(cursor + 30, true)
    const commentLength = view.getUint16(cursor + 32, true)
    const localHeaderOffset = view.getUint32(cursor + 42, true)
    const fileNameStart = cursor + 46
    const fileNameEnd = fileNameStart + fileNameLength
    const fileName = decodeZipText(bytes.subarray(fileNameStart, fileNameEnd))

    entries.push({
      compressedSize,
      compressionMethod,
      fileName,
      localHeaderOffset,
    })

    cursor = fileNameEnd + extraFieldLength + commentLength
  }

  return entries
}

async function inflateRaw(bytes: Uint8Array) {
  if (typeof DecompressionStream === 'undefined') {
    throw new Error('This browser does not support importing .chr archives.')
  }

  const input = new Uint8Array(bytes.byteLength)
  input.set(bytes)
  const stream = new Blob([input]).stream().pipeThrough(new DecompressionStream('deflate-raw'))

  return new Uint8Array(await new Response(stream).arrayBuffer())
}

async function extractZipEntry(bytes: Uint8Array, entry: ZipEntry) {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength)
  const headerOffset = entry.localHeaderOffset

  if (view.getUint32(headerOffset, true) !== LOCAL_FILE_HEADER_SIGNATURE) {
    throw new Error(`Invalid character archive: bad local file header for ${entry.fileName}.`)
  }

  const fileNameLength = view.getUint16(headerOffset + 26, true)
  const extraFieldLength = view.getUint16(headerOffset + 28, true)
  const dataOffset = headerOffset + 30 + fileNameLength + extraFieldLength
  const compressedBytes = bytes.subarray(dataOffset, dataOffset + entry.compressedSize)

  if (entry.compressionMethod === 0) {
    return compressedBytes
  }

  if (entry.compressionMethod === 8) {
    return inflateRaw(compressedBytes)
  }

  throw new Error(
    `Unsupported compression method ${entry.compressionMethod} in character archive.`,
  )
}

async function readZipJsonEntry<T>(
  bytes: Uint8Array,
  entries: ZipEntry[],
  entryName: string,
) {
  const entry = entries.find((candidate) => candidate.fileName === entryName)

  if (!entry) {
    throw new Error(`Character archive is missing ${entryName}.`)
  }

  try {
    return JSON.parse(decodeZipText(await extractZipEntry(bytes, entry))) as T
  } catch {
    throw new Error(`Character archive contains an invalid ${entryName}.`)
  }
}

function isCharacterCoverMimeType(value: string): value is CharacterCoverMimeType {
  return value === 'image/png' || value === 'image/jpeg' || value === 'image/webp'
}

function validateManifest(manifest: CharacterArchiveManifest) {
  if (manifest.format !== ARCHIVE_FORMAT) {
    throw new Error(`Unsupported character archive format: ${manifest.format}.`)
  }

  if (manifest.version !== ARCHIVE_VERSION) {
    throw new Error(`Unsupported character archive version: ${manifest.version}.`)
  }

  if (!manifest.content_path.trim()) {
    throw new Error('Character archive is missing content_path.')
  }

  if (!manifest.cover_path.trim()) {
    throw new Error('Character archive is missing cover_path.')
  }

  if (!isCharacterCoverMimeType(manifest.cover_mime_type)) {
    throw new Error('Character archive contains an unsupported cover MIME type.')
  }
}

function validateContent(content: Partial<CharacterCardContent>) {
  if (typeof content.id !== 'string' || content.id.trim().length === 0) {
    throw new Error('Character archive is missing content.id.')
  }

  if (typeof content.name !== 'string' || content.name.trim().length === 0) {
    throw new Error('Character archive is missing content.name.')
  }

  if (typeof content.personality !== 'string') {
    throw new Error('Character archive is missing content.personality.')
  }

  if (typeof content.style !== 'string') {
    throw new Error('Character archive is missing content.style.')
  }

  if (typeof content.schema_id !== 'string' || content.schema_id.trim().length === 0) {
    throw new Error('Character archive is missing content.schema_id.')
  }

  if (typeof content.system_prompt !== 'string') {
    throw new Error('Character archive is missing content.system_prompt.')
  }
}

export async function parseCharacterArchive(file: File) {
  if (file.size === 0) {
    throw new Error('Cannot import an empty character card.')
  }

  const bytes = new Uint8Array(await file.arrayBuffer())
  const entries = readZipEntries(bytes)
  const manifest = await readZipJsonEntry<CharacterArchiveManifest>(
    bytes,
    entries,
    'manifest.json',
  )

  validateManifest(manifest)

  const content = await readZipJsonEntry<Partial<CharacterCardContent>>(
    bytes,
    entries,
    manifest.content_path,
  )

  validateContent(content)

  if (manifest.character_id !== content.id) {
    throw new Error(
      `Character archive id mismatch: manifest=${manifest.character_id}, content=${content.id}.`,
    )
  }

  return {
    content: content as CharacterCardContent,
    manifest,
  }
}

export function buildCharacterSummaryFromArchive(args: {
  content: CharacterCardContent
  manifest: CharacterArchiveManifest
}) {
  return {
    character_id: args.content.id,
    cover_file_name: args.manifest.cover_path,
    cover_mime_type: args.manifest.cover_mime_type,
    name: args.content.name,
    personality: args.content.personality,
    style: args.content.style,
  } satisfies CharacterSummary
}
