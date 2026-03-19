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
  tags: string[]
  folder: string
}

export type CharacterSummary = {
  character_id: string
  cover_file_name: string | null
  cover_mime_type: CharacterCoverMimeType | null
  name: string
  personality: string
  style: string
  tags: string[]
  folder: string
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

export type CharactersListedResult = {
  characters: CharacterSummary[]
  type: 'characters_listed'
}

export type CharacterDeletedResult = {
  character_id: string
  type: 'character_deleted'
}

export type ResourceFilePayload = {
  content_type: string
  file_id: string
  file_name: string | null
  resource_id: string
  size_bytes: number
}
