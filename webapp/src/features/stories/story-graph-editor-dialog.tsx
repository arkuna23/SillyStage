import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Button } from '../../components/ui/button'
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../components/ui/dialog'
import { useToast } from '../../components/ui/toast-context'
import { cn } from '../../lib/cn'
import { updateStoryDraftGraph, updateStoryGraph } from './api'
import {
  INITIAL_STORY_GRAPH_VIEWPORT,
  isEditableTarget,
  normalizeControlledStoryGraph,
  useStoryGraphEditorController,
} from './story-graph-editor-controller'
import { StoryGraphEditorWorkspace } from './story-graph-editor-workspace'
import {
  getStoryGraphValidationMessage,
  GRAPH_MAX_ZOOM,
  GRAPH_MIN_ZOOM,
  GRAPH_ZOOM_STEP,
} from './story-graph-editor-utils'
import type { StoryGraph } from './types'

type StoryGraphEditorDialogProps = {
  graph: StoryGraph | null
  graphType: 'draft' | 'story'
  onGraphSaved?: (graph: StoryGraph) => void
  onOpenChange: (open: boolean) => void
  open: boolean
  readOnly?: boolean
  resourceId: string
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

export function StoryGraphEditorDialog({
  graph,
  graphType,
  onGraphSaved,
  onOpenChange,
  open,
  readOnly = false,
  resourceId,
}: StoryGraphEditorDialogProps) {
  const { t } = useTranslation()
  const { pushToast } = useToast()
  const [isSaving, setIsSaving] = useState(false)
  const [isFullscreen, setIsFullscreen] = useState(false)
  const controller = useStoryGraphEditorController({
    graph,
    onLastNodeWarning: () => {
      pushToast({
        message: t('stories.graph.errors.lastNode'),
        tone: 'warning',
      })
    },
    open,
    readOnly,
  })

  const title = graphType === 'draft' ? t('stories.graph.editDraftTitle') : t('stories.graph.editStoryTitle')
  const subtitle =
    graphType === 'draft'
      ? t('stories.graph.draftSubtitle')
      : t('stories.graph.storySubtitle')

  useEffect(() => {
    if (!open) {
      return
    }

    function handleKeyDown(event: KeyboardEvent) {
      if (!(event.ctrlKey || event.metaKey) || isEditableTarget(event.target)) {
        return
      }

      if (event.key === '=' || event.key === '+') {
        event.preventDefault()
        controller.setViewport((currentViewport) => ({
          ...currentViewport,
          zoom: Math.min(
            GRAPH_MAX_ZOOM,
            Number((currentViewport.zoom + GRAPH_ZOOM_STEP).toFixed(2)),
          ),
        }))
        return
      }

      if (event.key === '-') {
        event.preventDefault()
        controller.setViewport((currentViewport) => ({
          ...currentViewport,
          zoom: Math.max(
            GRAPH_MIN_ZOOM,
            Number((currentViewport.zoom - GRAPH_ZOOM_STEP).toFixed(2)),
          ),
        }))
        return
      }

      if (event.key === '0') {
        event.preventDefault()
        controller.setViewport(INITIAL_STORY_GRAPH_VIEWPORT)
      }
    }

    window.addEventListener('keydown', handleKeyDown)

    return () => {
      window.removeEventListener('keydown', handleKeyDown)
    }
  }, [controller, open])

  useEffect(() => {
    if (!open) {
      setIsSaving(false)
      setIsFullscreen(false)
    }
  }, [open])

  async function handleSave() {
    const normalized = normalizeControlledStoryGraph(controller)

    if (!normalized.graph) {
      pushToast({
        message: getStoryGraphValidationMessage(
          (key, options) => t(key as never, options),
          normalized.errors[0] ?? 'invalid_graph',
        ),
        tone: 'error',
      })
      return
    }

    setIsSaving(true)

    try {
      const nextGraph =
        graphType === 'draft'
          ? (
              await updateStoryDraftGraph({
                draft_id: resourceId,
                partial_graph: normalized.graph,
              })
            ).partial_graph
          : (
              await updateStoryGraph({
                graph: normalized.graph,
                story_id: resourceId,
              })
            ).graph

      controller.syncBaselineGraph(nextGraph)
      onGraphSaved?.(nextGraph)
      pushToast({
        message: t('stories.graph.feedback.saved'),
        tone: 'success',
      })
    } catch (error) {
      pushToast({
        message: getErrorMessage(error, t('stories.graph.feedback.saveFailed')),
        tone: 'error',
      })
    } finally {
      setIsSaving(false)
    }
  }

  if (!controller.graphDraft) {
    return null
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent
        aria-describedby={undefined}
        contentClassName={cn(
          'motion-safe:transition-[top,right,bottom,left,width,height,max-height,transform] motion-safe:duration-300 motion-safe:ease-[cubic-bezier(0.22,1,0.36,1)] motion-safe:will-change-[top,right,bottom,left,width,height,transform]',
          isFullscreen
            ? '!h-screen !w-screen !max-h-none'
            : undefined,
        )}
        className={cn(
          'motion-safe:transition-[width,height,max-height,border-radius,border-color,box-shadow] motion-safe:duration-300 motion-safe:ease-[cubic-bezier(0.22,1,0.36,1)]',
          isFullscreen
            ? 'h-full max-h-none w-full !rounded-none !border-x-0 !border-y-0'
            : 'h-[min(92vh,60rem)] w-[min(98vw,108rem)]',
        )}
      >
        <DialogHeader className="space-y-2 border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{title}</DialogTitle>
          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">{subtitle}</p>
        </DialogHeader>

        <StoryGraphEditorWorkspace
          controller={controller}
          graphType={graphType}
          isFullscreen={isFullscreen}
          onToggleFullscreen={() => {
            setIsFullscreen((currentValue) => !currentValue)
          }}
          readOnly={readOnly}
        />

        <DialogFooter className="justify-between">
          <div className="text-sm text-[var(--color-text-secondary)]">
            {readOnly ? t('stories.graph.readOnlyHint') : t('stories.graph.footerHint')}
          </div>
          <div className="flex flex-col-reverse gap-3 sm:flex-row">
            {!readOnly ? (
              <Button onClick={controller.handleResetChanges} variant="ghost">
                {t('stories.graph.resetChanges')}
              </Button>
            ) : null}
            <DialogClose asChild>
              <Button variant="secondary">{t('stories.actions.close')}</Button>
            </DialogClose>
            {!readOnly ? (
              <Button disabled={!controller.isDirty || isSaving} onClick={() => void handleSave()}>
                {isSaving ? t('stories.graph.saving') : t('stories.graph.save')}
              </Button>
            ) : null}
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
