import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import {
  Dialog,
  DialogBody,
  DialogClose,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../components/ui/dialog'
import { useToastMessage } from '../../components/ui/toast-context'
import type { CharacterSummary } from '../characters/types'
import { getStoryDraft } from './api'
import { StoryGraphEditorDialog } from './story-graph-editor-dialog'
import type { StoryDraftDetail } from './types'

type StoryDraftDetailsDialogProps = {
  availableCharacters: ReadonlyArray<CharacterSummary>
  draftId: string | null
  onOpenChange: (open: boolean) => void
  open: boolean
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function DetailRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-[1.4rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-3.5">
      <p className="text-xs text-[var(--color-text-muted)]">{label}</p>
      <p className="mt-2 text-sm leading-6 text-[var(--color-text-primary)]">{value}</p>
    </div>
  )
}

export function StoryDraftDetailsDialog({
  availableCharacters,
  draftId,
  onOpenChange,
  open,
}: StoryDraftDetailsDialogProps) {
  const { t } = useTranslation()
  const [draft, setDraft] = useState<StoryDraftDetail | null>(null)
  const [errorState, setErrorState] = useState<{ draftId: string; message: string } | null>(null)
  const [isGraphEditorOpen, setIsGraphEditorOpen] = useState(false)

  const visibleDraft = open && draftId !== null && draft?.draft_id === draftId ? draft : null
  const errorMessage =
    open && draftId !== null && errorState?.draftId === draftId ? errorState.message : null
  const isLoading = open && draftId !== null && visibleDraft === null && errorMessage === null
  useToastMessage(errorMessage)

  useEffect(() => {
    if (!open || !draftId) {
      return
    }

    const controller = new AbortController()

    void getStoryDraft(draftId, controller.signal)
      .then((result) => {
        if (!controller.signal.aborted) {
          setDraft(result)
          setErrorState(null)
        }
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setErrorState({
            draftId,
            message: getErrorMessage(error, t('stories.drafts.feedback.loadDraftFailed')),
          })
        }
      })

    return () => {
      controller.abort()
    }
  }, [draftId, open, t])

  const partialGraphSummary = useMemo(() => {
    if (!visibleDraft) {
      return null
    }

    return {
      nodeCount: visibleDraft.partial_graph.nodes.length,
      startNode: visibleDraft.partial_graph.start_node,
      terminalCount: visibleDraft.partial_graph.nodes.filter(
        (node) => node.transitions.length === 0,
      ).length,
    }
  }, [visibleDraft])

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,48rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{t('stories.drafts.details.title')}</DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          {isLoading ? (
            <div className="grid gap-3">
              {Array.from({ length: 6 }).map((_, index) => (
                <div
                  className="h-20 animate-pulse rounded-[1.4rem] bg-[var(--color-bg-elevated)]"
                  key={index}
                />
              ))}
            </div>
          ) : visibleDraft ? (
            <>
              <div className="space-y-2">
                <h3 className="font-display text-[2rem] leading-tight text-[var(--color-text-primary)]">
                  {visibleDraft.display_name}
                </h3>
                <div className="flex flex-wrap gap-2">
                  <Badge className="normal-case px-3 py-1.5" variant="subtle">
                    {visibleDraft.draft_id}
                  </Badge>
                  <Badge className="normal-case px-3 py-1.5" variant="subtle">
                    {t('stories.drafts.list.resourcePrefix', { id: visibleDraft.resource_id })}
                  </Badge>
                  <Badge className="normal-case px-3 py-1.5" variant="info">
                    {t(`stories.drafts.status.${visibleDraft.status}` as const)}
                  </Badge>
                </div>
              </div>

              <div className="grid gap-3 md:grid-cols-2">
                <DetailRow
                  label={t('stories.form.fields.playerSchemaId')}
                  value={visibleDraft.player_schema_id}
                />
                <DetailRow
                  label={t('stories.form.fields.worldSchemaId')}
                  value={visibleDraft.world_schema_id}
                />
              </div>

              <DetailRow
                label={t('stories.drafts.details.plannedStory')}
                value={visibleDraft.planned_story}
              />

              <div className="rounded-[1.4rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                <p className="text-xs text-[var(--color-text-muted)]">
                  {t('stories.drafts.details.outlineSections')}
                </p>
                <div className="mt-3 space-y-2">
                  {visibleDraft.outline_sections.map((section, index) => (
                    <p
                      className="text-sm leading-7 text-[var(--color-text-primary)]"
                      key={`${index}:${section}`}
                    >
                      {index + 1}. {section}
                    </p>
                  ))}
                </div>
              </div>

              {visibleDraft.section_summaries.length > 0 ? (
                <div className="rounded-[1.4rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                  <p className="text-xs text-[var(--color-text-muted)]">
                    {t('stories.drafts.details.sectionSummaries')}
                  </p>
                  <div className="mt-3 space-y-2">
                    {visibleDraft.section_summaries.map((summary, index) => (
                      <p
                        className="text-sm leading-7 text-[var(--color-text-primary)]"
                        key={`${index}:${summary}`}
                      >
                        {index + 1}. {summary}
                      </p>
                    ))}
                  </div>
                </div>
              ) : null}

              {partialGraphSummary ? (
                <div className="rounded-[1.4rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                  <div className="flex items-start justify-between gap-4">
                    <div>
                      <p className="text-xs text-[var(--color-text-muted)]">
                        {t('stories.drafts.details.partialGraph')}
                      </p>
                      <div className="mt-3 flex flex-wrap gap-2">
                        <Badge className="normal-case px-3 py-1.5" variant="info">
                          {t('stories.details.nodeCount', { count: partialGraphSummary.nodeCount })}
                        </Badge>
                        <Badge className="normal-case px-3 py-1.5" variant="subtle">
                          {t('stories.details.startNode', { id: partialGraphSummary.startNode })}
                        </Badge>
                        <Badge className="normal-case px-3 py-1.5" variant="subtle">
                          {t('stories.details.terminalCount', {
                            count: partialGraphSummary.terminalCount,
                          })}
                        </Badge>
                      </div>
                    </div>
                    <Button
                      onClick={() => {
                        setIsGraphEditorOpen(true)
                      }}
                      size="sm"
                      variant="secondary"
                    >
                      {t('stories.actions.editGraph')}
                    </Button>
                  </div>
                </div>
              ) : null}
            </>
          ) : null}
        </DialogBody>

        <DialogFooter className="justify-end">
          <DialogClose asChild>
            <Button variant="secondary">{t('stories.actions.close')}</Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>

      <StoryGraphEditorDialog
        availableCharacters={availableCharacters}
        graph={visibleDraft?.partial_graph ?? null}
        graphType="draft"
        onGraphSaved={(nextGraph) => {
          setDraft((currentDraft) =>
            currentDraft && currentDraft.draft_id === draftId
              ? { ...currentDraft, partial_graph: nextGraph }
              : currentDraft,
          )
        }}
        onOpenChange={setIsGraphEditorOpen}
        open={open && isGraphEditorOpen}
        playerSchemaId={visibleDraft?.player_schema_id}
        readOnly={visibleDraft?.status === 'finalized'}
        resourceId={visibleDraft?.draft_id ?? ''}
        worldSchemaId={visibleDraft?.world_schema_id}
      />
    </Dialog>
  )
}
