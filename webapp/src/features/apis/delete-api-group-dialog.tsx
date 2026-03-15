import { useTranslation } from 'react-i18next'

import {
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../components/ui/dialog'
import { Button } from '../../components/ui/button'
import type { ApiGroup } from './types'

type DeleteApiGroupDialogProps = {
  apiGroup: ApiGroup | null
  deleting: boolean
  onConfirm: () => void
  onOpenChange: (open: boolean) => void
  open: boolean
}

export function DeleteApiGroupDialog({
  apiGroup,
  deleting,
  onConfirm,
  onOpenChange,
  open,
}: DeleteApiGroupDialogProps) {
  const { t } = useTranslation()

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,28rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{t('apis.deleteDialog.title')}</DialogTitle>
        </DialogHeader>

        <DialogBody className="pt-6">
          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
            {t('apis.deleteDialog.message', {
              id: apiGroup?.api_group_id ?? '—',
            })}
          </p>
        </DialogBody>

        <DialogFooter className="justify-end gap-2">
          <Button onClick={() => onOpenChange(false)} variant="ghost">
            {t('apis.actions.cancel')}
          </Button>
          <Button disabled={deleting || !apiGroup} onClick={onConfirm} variant="danger">
            {deleting ? t('apis.actions.deleting') : t('apis.actions.confirmDelete')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
