export type StageMessageVariant = 'action' | 'dialogue' | 'narration' | 'player' | 'thought'

export type TurnWorkerStatus = {
  label: string
}

export type StageMessage = {
  id: string
  messageId?: string
  speakerId: string
  speakerName: string
  text: string
  turnIndex: number
  updatedAtMs?: number
  variant: StageMessageVariant
}

export type CoverCache = Record<string, string | null | undefined>

export type StageCastMember = {
  coverUrl?: string | null
  description: string
  id: string
  isSessionCharacter: boolean
  name: string
}
