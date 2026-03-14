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
  open: boolean
  story: StorySummary | null
}

export function DeleteStoryDialog({
  deleting,
  onConfirm,
  onOpenChange,
  open,
  story,
}: DeleteStoryDialogProps) {
  const { t } = useTranslation()

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,30rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{t('stories.deleteDialog.title')}</DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-4 pt-6">
          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
            {story ? t('stories.deleteDialog.message', { id: story.story_id }) : null}
          </p>

          {story ? (
            <div className="flex flex-wrap gap-2">
              <Badge className="normal-case px-3 py-1.5" variant="subtle">
                {story.display_name}
              </Badge>
              <Badge className="normal-case px-3 py-1.5" variant="subtle">
                {story.story_id}
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
