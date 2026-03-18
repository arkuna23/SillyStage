export type StageMessageVariant = 'action' | 'dialogue' | 'narration' | 'player' | 'thought'

export type PanelMode = 'dialogue' | 'settings' | 'variables'
export type ComposerMode = 'input' | 'suggestions'
export type NoticeTone = 'error' | 'success' | 'warning'

export type Notice = {
  message: string
  tone: NoticeTone
}

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
  id: string
  isSessionCharacter: boolean
  name: string
}

export type StageCommonVariable = {
  id: string
  label: string
  value: string
}

export type StageRightRailTab = 'status' | 'variables'
