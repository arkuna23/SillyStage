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
  draft: StoryDraftSummary | null
  onConfirm: () => void
  onOpenChange: (open: boolean) => void
  open: boolean
}

export function DeleteStoryDraftDialog({
  deleting,
  draft,
  onConfirm,
  onOpenChange,
  open,
}: DeleteStoryDraftDialogProps) {
  const { t } = useTranslation()

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,30rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{t('stories.drafts.deleteDialog.title')}</DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-4 pt-6">
          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
            {draft ? t('stories.drafts.deleteDialog.message', { id: draft.draft_id }) : null}
          </p>

          {draft ? (
            <div className="flex flex-wrap gap-2">
              <Badge className="normal-case px-3 py-1.5" variant="subtle">
                {draft.display_name}
              </Badge>
              <Badge className="normal-case px-3 py-1.5" variant="subtle">
                {draft.draft_id}
              </Badge>
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
            {deleting ? t('stories.actions.deleting') : t('stories.actions.delete')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
