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
import { cn } from '../../lib/cn'
import type { StoryGraphEditorController } from './story-graph-editor-controller'
import {
  INITIAL_STORY_GRAPH_VIEWPORT,
  isEditableTarget,
} from './story-graph-editor-controller'
import { StoryGraphEditorWorkspace } from './story-graph-editor-workspace'
import { GRAPH_MAX_ZOOM, GRAPH_MIN_ZOOM, GRAPH_ZOOM_STEP } from './story-graph-editor-utils'

type ManualStoryGraphDialogProps = {
  controller: StoryGraphEditorController
  onOpenChange: (open: boolean) => void
  open: boolean
  playerSchemaId: string
  resourceId: string
  worldSchemaId: string
}

export function ManualStoryGraphDialog({
  controller,
  onOpenChange,
  open,
  playerSchemaId,
  resourceId,
  worldSchemaId,
}: ManualStoryGraphDialogProps) {
  const { t } = useTranslation()
  const [isFullscreen, setIsFullscreen] = useState(false)

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

  if (!controller.graphDraft) {
    return null
  }

  return (
    <Dialog
      onOpenChange={(nextOpen) => {
        if (!nextOpen) {
          setIsFullscreen(false)
        }

        onOpenChange(nextOpen)
      }}
      open={open}
    >
      <DialogContent
        aria-describedby={undefined}
        contentClassName={cn(
          'motion-safe:transition-[top,right,bottom,left,width,height,max-height,transform] motion-safe:duration-300 motion-safe:ease-[cubic-bezier(0.22,1,0.36,1)] motion-safe:will-change-[top,right,bottom,left,width,height,transform]',
          isFullscreen ? '!h-screen !w-screen !max-h-none' : undefined,
        )}
        className={cn(
          'motion-safe:transition-[width,height,max-height,border-radius,border-color,box-shadow] motion-safe:duration-300 motion-safe:ease-[cubic-bezier(0.22,1,0.36,1)]',
          isFullscreen
            ? 'h-full max-h-none w-full !rounded-none !border-x-0 !border-y-0'
            : 'h-[min(92vh,60rem)] w-[min(98vw,108rem)]',
        )}
      >
        <DialogHeader className="space-y-2 border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{t('stories.manualCreate.graphDialog.title')}</DialogTitle>
          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
            {t('stories.manualCreate.graphDialog.description')}
          </p>
          <div className="flex flex-wrap gap-x-4 gap-y-1 text-xs text-[var(--color-text-muted)]">
            <span>
              {t('stories.form.fields.resourceId')} <span className="font-mono">{resourceId}</span>
            </span>
            <span>
              {t('stories.form.fields.playerSchemaId')}{' '}
              <span className="font-mono">{playerSchemaId}</span>
            </span>
            <span>
              {t('stories.form.fields.worldSchemaId')}{' '}
              <span className="font-mono">{worldSchemaId}</span>
            </span>
          </div>
        </DialogHeader>

        <StoryGraphEditorWorkspace
          controller={controller}
          graphType="story"
          isFullscreen={isFullscreen}
          onToggleFullscreen={() => {
            setIsFullscreen((currentValue) => !currentValue)
          }}
        />

        <DialogFooter className="justify-between">
          <div className="text-sm text-[var(--color-text-secondary)]">
            {t('stories.manualCreate.graphDialog.footer')}
          </div>
          <DialogClose asChild>
            <Button variant="secondary">{t('stories.actions.close')}</Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
