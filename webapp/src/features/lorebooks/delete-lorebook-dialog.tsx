import { useTranslation } from 'react-i18next'

import { Button } from '../../components/ui/button'
import {
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../components/ui/dialog'
import { Badge } from '../../components/ui/badge'
import type { Lorebook } from './types'

type DeleteLorebookDialogProps = {
  deleting: boolean
  lorebook: Lorebook | null
  onConfirm: () => void
  onOpenChange: (open: boolean) => void
}

export function DeleteLorebookDialog({
  deleting,
  lorebook,
  onConfirm,
  onOpenChange,
}: DeleteLorebookDialogProps) {
  const { t } = useTranslation()

  return (
    <Dialog onOpenChange={onOpenChange} open={lorebook !== null}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,32rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{t('lorebooks.deleteDialog.title')}</DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-4 pt-6">
          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
            {t('lorebooks.deleteDialog.message', {
              id: lorebook?.lorebook_id ?? '—',
            })}
          </p>

          {lorebook ? (
            <div className="flex flex-wrap gap-2">
              <Badge className="normal-case px-3 py-1.5" variant="subtle">
                {lorebook.display_name}
              </Badge>
              <Badge className="normal-case px-3 py-1.5" variant="subtle">
                {t('lorebooks.list.entriesCount', { count: lorebook.entries.length })}
              </Badge>
            </div>
          ) : null}
        </DialogBody>

        <DialogFooter className="justify-end gap-2">
          <Button
            disabled={deleting}
            onClick={() => {
              onOpenChange(false)
            }}
            variant="ghost"
          >
            {t('lorebooks.actions.cancel')}
          </Button>
          <Button disabled={deleting} onClick={onConfirm} variant="danger">
            {deleting ? t('lorebooks.actions.deleting') : t('lorebooks.actions.delete')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
