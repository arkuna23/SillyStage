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
import { getStory } from './api'
import { StoryGraphEditorDialog } from './story-graph-editor-dialog'
import type { StoryDetail } from './types'

type StoryDetailsDialogProps = {
  availableCharacters: ReadonlyArray<CharacterSummary>
  onOpenChange: (open: boolean) => void
  open: boolean
  storyId: string | null
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

export function StoryDetailsDialog({
  availableCharacters,
  onOpenChange,
  open,
  storyId,
}: StoryDetailsDialogProps) {
  const { t } = useTranslation()
  const [story, setStory] = useState<StoryDetail | null>(null)
  const [errorState, setErrorState] = useState<{ message: string; storyId: string } | null>(null)
  const [isGraphEditorOpen, setIsGraphEditorOpen] = useState(false)
  const visibleStory = open && storyId !== null && story?.story_id === storyId ? story : null
  const errorMessage =
    open && storyId !== null && errorState?.storyId === storyId ? errorState.message : null
  const isLoading = open && storyId !== null && visibleStory === null && errorMessage === null
  useToastMessage(errorMessage)

  useEffect(() => {
    if (!open || !storyId) {
      return
    }

    const controller = new AbortController()

    void getStory(storyId, controller.signal)
      .then((result) => {
        if (!controller.signal.aborted) {
          setStory(result)
          setErrorState(null)
        }
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setErrorState({
            message: getErrorMessage(error, t('stories.feedback.loadStoryFailed')),
            storyId,
          })
        }
      })

    return () => {
      controller.abort()
    }
  }, [open, storyId, t])

  const graphSummary = useMemo(() => {
    if (!visibleStory) {
      return null
    }

    return {
      nodeCount: visibleStory.graph.nodes.length,
      startNode: visibleStory.graph.start_node,
      terminalCount: visibleStory.graph.nodes.filter((node) => node.transitions.length === 0)
        .length,
    }
  }, [visibleStory])

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,44rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{t('stories.details.title')}</DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          {isLoading ? (
            <div className="grid gap-3">
              {Array.from({ length: 5 }).map((_, index) => (
                <div
                  className="h-20 animate-pulse rounded-[1.4rem] bg-[var(--color-bg-elevated)]"
                  key={index}
                />
              ))}
            </div>
          ) : visibleStory ? (
            <>
              <div className="space-y-2">
                <h3 className="font-display text-[2rem] leading-tight text-[var(--color-text-primary)]">
                  {visibleStory.display_name}
                </h3>
                <div className="flex flex-wrap gap-2">
                  <Badge className="normal-case px-3 py-1.5" variant="subtle">
                    {visibleStory.story_id}
                  </Badge>
                  <Badge className="normal-case px-3 py-1.5" variant="subtle">
                    {t('stories.details.resourcePrefix', { id: visibleStory.resource_id })}
                  </Badge>
                </div>
              </div>

              <div className="grid gap-3 md:grid-cols-2">
                <DetailRow
                  label={t('stories.form.fields.playerSchemaId')}
                  value={visibleStory.player_schema_id}
                />
                <DetailRow
                  label={t('stories.form.fields.worldSchemaId')}
                  value={visibleStory.world_schema_id}
                />
              </div>

              <DetailRow
                label={t('stories.form.fields.introduction')}
                value={visibleStory.introduction}
              />

              {graphSummary ? (
                <div className="rounded-[1.4rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                  <div className="flex items-start justify-between gap-4">
                    <div>
                      <p className="text-xs text-[var(--color-text-muted)]">
                        {t('stories.details.graph')}
                      </p>
                      <div className="mt-3 flex flex-wrap gap-2">
                        <Badge className="normal-case px-3 py-1.5" variant="info">
                          {t('stories.details.nodeCount', { count: graphSummary.nodeCount })}
                        </Badge>
                        <Badge className="normal-case px-3 py-1.5" variant="subtle">
                          {t('stories.details.startNode', { id: graphSummary.startNode })}
                        </Badge>
                        <Badge className="normal-case px-3 py-1.5" variant="subtle">
                          {t('stories.details.terminalCount', {
                            count: graphSummary.terminalCount,
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
        graph={visibleStory?.graph ?? null}
        graphType="story"
        onGraphSaved={(nextGraph) => {
          setStory((currentStory) =>
            currentStory && currentStory.story_id === storyId
              ? { ...currentStory, graph: nextGraph }
              : currentStory,
          )
        }}
        onOpenChange={setIsGraphEditorOpen}
        open={open && isGraphEditorOpen}
        playerSchemaId={visibleStory?.player_schema_id}
        resourceId={visibleStory?.story_id ?? ''}
        worldSchemaId={visibleStory?.world_schema_id}
      />
    </Dialog>
  )
}
