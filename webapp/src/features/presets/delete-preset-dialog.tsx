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
import type { Preset } from '../apis/types'

type DeletePresetDialogProps = {
  deleting: boolean
  onConfirm: () => void
  onOpenChange: (open: boolean) => void
  open: boolean
  preset: Preset | null
}

export function DeletePresetDialog({
  deleting,
  onConfirm,
  onOpenChange,
  open,
  preset,
}: DeletePresetDialogProps) {
  const { t } = useTranslation()

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,28rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{t('presetsPage.deleteDialog.title')}</DialogTitle>
        </DialogHeader>

        <DialogBody className="pt-6">
          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
            {t('presetsPage.deleteDialog.message', {
              id: preset?.preset_id ?? '—',
            })}
          </p>
        </DialogBody>

        <DialogFooter className="justify-end gap-2">
          <Button onClick={() => onOpenChange(false)} variant="ghost">
            {t('presetsPage.actions.cancel')}
          </Button>
          <Button disabled={deleting || !preset} onClick={onConfirm} variant="danger">
            {deleting ? t('presetsPage.actions.deleting') : t('presetsPage.actions.confirmDelete')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
