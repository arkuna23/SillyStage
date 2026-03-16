import type { StoryDraftDetail, StoryDraftSummary } from './types'

type DraftProgressSource = Pick<StoryDraftDetail | StoryDraftSummary, 'next_section_index' | 'total_sections'>

export type DraftSectionProgress = {
  current: number
  total: number
}

export function getDraftSectionProgress(draft: DraftProgressSource): DraftSectionProgress | null {
  const nextSectionIndex =
    typeof draft.next_section_index === 'number' && Number.isFinite(draft.next_section_index)
      ? draft.next_section_index
      : null
  const totalSections =
    typeof draft.total_sections === 'number' && Number.isFinite(draft.total_sections) && draft.total_sections > 0
      ? draft.total_sections
      : null

  if (nextSectionIndex === null || totalSections === null) {
    return null
  }

  return {
    current: Math.min(Math.max(nextSectionIndex, 0), totalSections),
    total: totalSections,
  }
}
