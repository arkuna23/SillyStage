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
  onConfirm: () => void
  onOpenChange: (open: boolean) => void
  targets: ReadonlyArray<Lorebook>
}

export function DeleteLorebookDialog({
  deleting,
  onConfirm,
  onOpenChange,
  targets,
}: DeleteLorebookDialogProps) {
  const { t } = useTranslation()
  const open = targets.length > 0
  const isBulk = targets.length > 1
  const previewTargets = targets.slice(0, 5)

  return (
    <Dialog
      onOpenChange={(nextOpen) => {
        if (!nextOpen) {
          onOpenChange(false)
        }
      }}
      open={open}
    >
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,32rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>
            {isBulk ? t('lorebooks.deleteDialog.titleMany') : t('lorebooks.deleteDialog.title')}
          </DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-4 pt-6">
          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
            {isBulk
              ? t('lorebooks.deleteDialog.messageMany', { count: targets.length })
              : t('lorebooks.deleteDialog.message', {
                  id: targets[0]?.lorebook_id ?? '—',
                })}
          </p>

          <div className="flex flex-wrap gap-2">
            {previewTargets.map((target) => (
              <Badge className="normal-case px-3 py-1.5" key={target.lorebook_id} variant="subtle">
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
          <Button disabled={deleting || targets.length === 0} onClick={onConfirm} variant="danger">
            {deleting
              ? t('lorebooks.actions.deleting')
              : isBulk
                ? t('lorebooks.actions.deleteSelected')
                : t('lorebooks.actions.delete')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
