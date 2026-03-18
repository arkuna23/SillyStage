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
import type { StoryDraftSummary } from './types'

type DeleteStoryDraftDialogProps = {
  deleting: boolean
  onConfirm: () => void
  onOpenChange: (open: boolean) => void
  open: boolean
  targets: StoryDraftSummary[]
}

export function DeleteStoryDraftDialog({
  deleting,
  onConfirm,
  onOpenChange,
  open,
  targets,
}: DeleteStoryDraftDialogProps) {
  const { t } = useTranslation()
  const isBulk = targets.length > 1
  const primaryTarget = targets[0] ?? null

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,30rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>
            {isBulk ? t('stories.drafts.deleteDialog.titleMany') : t('stories.drafts.deleteDialog.title')}
          </DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-4 pt-6">
          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
            {isBulk
              ? t('stories.drafts.deleteDialog.messageMany', { count: targets.length })
              : primaryTarget
                ? t('stories.drafts.deleteDialog.message', { id: primaryTarget.draft_id })
                : null}
          </p>

          {targets.length > 0 ? (
            <div className="flex flex-wrap gap-2">
              {targets.map((target) => (
                <Badge className="normal-case px-3 py-1.5" key={target.draft_id} variant="subtle">
                  {target.display_name}
                </Badge>
              ))}
              {isBulk ? (
                <Badge className="normal-case px-3 py-1.5" variant="subtle">
                  {t('stories.selection.count', { count: targets.length })}
                </Badge>
              ) : primaryTarget ? (
                <Badge className="normal-case px-3 py-1.5" variant="subtle">
                  {primaryTarget.draft_id}
                </Badge>
              ) : null}
            </div>
          ) : null}
        </DialogBody>

        <DialogFooter className="justify-end">
          <DialogClose asChild>
            <Button disabled={deleting} variant="secondary">
              {t('stories.actions.cancel')}
            </Button>
          </DialogClose>
          <Button disabled={deleting} onClick={onConfirm} variant="danger">
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
