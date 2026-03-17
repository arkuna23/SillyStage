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
import type { StorySummary } from './types'

type DeleteStoryDialogProps = {
  deleting: boolean
  onConfirm: () => void
  onOpenChange: (open: boolean) => void
  targets: ReadonlyArray<StorySummary>
}

export function DeleteStoryDialog({
  deleting,
  onConfirm,
  onOpenChange,
  targets,
}: DeleteStoryDialogProps) {
  const { t } = useTranslation()
  const open = targets.length > 0
  const isBulk = targets.length > 1
  const previewTargets = targets.slice(0, 5)

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,30rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>
            {isBulk ? t('stories.deleteDialog.titleMany') : t('stories.deleteDialog.title')}
          </DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-4 pt-6">
          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
            {isBulk
              ? t('stories.deleteDialog.messageMany', { count: targets.length })
              : targets[0]
                ? t('stories.deleteDialog.message', { id: targets[0].story_id })
                : null}
          </p>

          <div className="flex flex-wrap gap-2">
            {previewTargets.map((target) => (
              <Badge className="normal-case px-3 py-1.5" key={target.story_id} variant="subtle">
                {target.display_name}
              </Badge>
            ))}
            {targets.length > previewTargets.length ? (
              <Badge className="normal-case px-3 py-1.5" variant="subtle">
                +{targets.length - previewTargets.length}
              </Badge>
            ) : null}
          </div>
        </DialogBody>

        <DialogFooter className="justify-end">
          <DialogClose asChild>
            <Button disabled={deleting} variant="secondary">
              {t('stories.actions.cancel')}
            </Button>
          </DialogClose>
          <Button disabled={deleting || targets.length === 0} onClick={onConfirm} variant="danger">
            {deleting
              ? t('stories.actions.deleting')
              : isBulk
                ? t('stories.actions.deleteSelected')
                : t('stories.actions.delete')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
