export const characterCoverMimeTypes = [
  'image/png',
  'image/jpeg',
  'image/webp',
] as const

export type CharacterCoverMimeType = (typeof characterCoverMimeTypes)[number]

export type CharacterCardContent = {
  id: string
  name: string
  personality: string
  schema_id: string
  style: string
  system_prompt: string
}

export type CharacterSummary = {
  character_id: string
  cover_file_name: string | null
  cover_mime_type: CharacterCoverMimeType | null
  name: string
  personality: string
  style: string
}

export type CharacterCreateResult = {
  character_id: string
  character_summary: CharacterSummary
  type: 'character_created'
}

export type CharacterSchemaResult = {
  character_id: string
  content: CharacterCardContent
  cover_file_name: string | null
  cover_mime_type: CharacterCoverMimeType | null
  type: 'character'
}

export type CharacterCoverResult = {
  character_id: string
  cover_base64: string
  cover_file_name: string
  cover_mime_type: CharacterCoverMimeType
  type: 'character_cover'
}

export type CharacterCoverUpdatedResult = {
  character_id: string
  cover_file_name: string
  cover_mime_type: CharacterCoverMimeType
  type: 'character_cover_updated'
}

export type CharacterExportResult = {
  character_id: string
  chr_base64: string
  content_type: string
  file_name: string
  type: 'character_chr_export'
}

export type CharactersListedResult = {
  characters: CharacterSummary[]
  type: 'characters_listed'
}

export type CharacterDeletedResult = {
  character_id: string
  type: 'character_deleted'
}

export type CharacterCardUploadedResult = {
  character_id: string
  character_summary: CharacterSummary
  type: 'character_card_uploaded'
}

export type UploadChunkAcceptedResult = {
  received_bytes: number
  received_chunk_index: number
  type: 'upload_chunk_accepted'
  upload_id: string
}

export type UploadInitializedResult = {
  chunk_size_hint: number
  type: 'upload_initialized'
  upload_id: string
}
