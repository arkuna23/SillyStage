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
import type { SchemaResource } from './types'

type DeleteSchemaDialogProps = {
  deleting: boolean
  onConfirm: () => void
  onOpenChange: (open: boolean) => void
  targets: ReadonlyArray<SchemaResource>
}

export function DeleteSchemaDialog({
  deleting,
  onConfirm,
  onOpenChange,
  targets,
}: DeleteSchemaDialogProps) {
  const { t } = useTranslation()
  const open = targets.length > 0
  const isBulk = targets.length > 1
  const previewTargets = targets.slice(0, 5)

  return (
    <Dialog
      onOpenChange={(open) => {
        if (!open) {
          onOpenChange(false)
        }
      }}
      open={open}
    >
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,34rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>
            {isBulk ? t('schemas.deleteDialog.titleMany') : t('schemas.deleteDialog.title')}
          </DialogTitle>
        </DialogHeader>

        {open ? (
          <>
            <DialogBody className="space-y-5 pt-6">
              <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                {isBulk
                  ? t('schemas.deleteDialog.messageMany', { count: targets.length })
                  : t('schemas.deleteDialog.message', {
                      id: targets[0]?.schema_id ?? '—',
                      name: targets[0]?.display_name ?? '—',
                    })}
              </p>

              <div className="flex flex-wrap gap-2">
                {previewTargets.map((target) => (
                  <Badge className="normal-case px-3 py-1.5" key={target.schema_id} variant="subtle">
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

            <DialogFooter>
              <DialogClose asChild>
                <Button disabled={deleting} variant="ghost">
                  {t('schemas.actions.cancel')}
                </Button>
              </DialogClose>

              <Button
                disabled={deleting || targets.length === 0}
                onClick={onConfirm}
                variant="danger"
              >
                {deleting
                  ? t('schemas.actions.deleting')
                  : isBulk
                    ? t('schemas.actions.deleteSelected')
                    : t('schemas.actions.delete')}
              </Button>
            </DialogFooter>
          </>
        ) : null}
      </DialogContent>
    </Dialog>
  )
}
