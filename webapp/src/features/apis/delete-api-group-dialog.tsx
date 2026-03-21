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
import type { ApiGroup } from './types'

type DeleteApiGroupDialogProps = {
  deleting: boolean
  onConfirm: () => void
  onOpenChange: (open: boolean) => void
  targets: ReadonlyArray<ApiGroup>
}

export function DeleteApiGroupDialog({
  deleting,
  onConfirm,
  onOpenChange,
  targets,
}: DeleteApiGroupDialogProps) {
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
            {isBulk ? t('apis.deleteDialog.titleMany') : t('apis.deleteDialog.title')}
          </DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
            {isBulk
              ? t('apis.deleteDialog.messageMany', { count: targets.length })
              : t('apis.deleteDialog.message', {
                  id: targets[0]?.api_group_id ?? '—',
                })}
          </p>

          <div className="flex flex-wrap gap-2">
            {previewTargets.map((target) => (
              <Badge className="normal-case px-3 py-1.5" key={target.api_group_id} variant="subtle">
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
