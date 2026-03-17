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
import type { ApiConfig } from './types'

type DeleteApiDialogProps = {
  deleting: boolean
  onConfirm: () => void
  onOpenChange: (open: boolean) => void
  targets: ReadonlyArray<ApiConfig>
}

export function DeleteApiDialog({
  deleting,
  onConfirm,
  onOpenChange,
  targets,
}: DeleteApiDialogProps) {
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
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,34rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>
            {isBulk ? t('apis.apiDeleteDialog.titleMany') : t('apis.apiDeleteDialog.title')}
          </DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
            {isBulk
              ? t('apis.apiDeleteDialog.messageMany', { count: targets.length })
              : t('apis.apiDeleteDialog.message', {
                  id: targets[0]?.api_id ?? '—',
                })}
          </p>

          <div className="flex flex-wrap gap-2">
            {previewTargets.map((target) => (
              <Badge className="normal-case px-3 py-1.5" key={target.api_id} variant="subtle">
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
          <DialogClose asChild>
            <Button disabled={deleting} variant="ghost">
              {t('apis.actions.cancel')}
            </Button>
          </DialogClose>
          <Button disabled={deleting || targets.length === 0} onClick={onConfirm} variant="danger">
            {deleting
              ? t('apis.actions.deleting')
              : isBulk
                ? t('apis.actions.deleteSelected')
                : t('apis.actions.confirmDelete')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
