export type LorebookEntry = {
  always_include: boolean
  content: string
  enabled: boolean
  entry_id: string
  keywords: string[]
  title: string
  type?: 'lorebook_entry'
}

export type Lorebook = {
  display_name: string
  entries: LorebookEntry[]
  lorebook_id: string
  type: 'lorebook'
}

export type LorebooksListedResult = {
  lorebooks: Lorebook[]
  type: 'lorebooks_listed'
}

export type LorebookDeletedResult = {
  lorebook_id: string
  type: 'lorebook_deleted'
}

export type LorebookEntryDeletedResult = {
  entry_id: string
  lorebook_id: string
  type: 'lorebook_entry_deleted'
}
