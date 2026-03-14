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
  schema: SchemaResource | null
}

export function DeleteSchemaDialog({
  deleting,
  onConfirm,
  onOpenChange,
  schema,
}: DeleteSchemaDialogProps) {
  const { t } = useTranslation()

  return (
    <Dialog
      onOpenChange={(open) => {
        if (!open) {
          onOpenChange(false)
        }
      }}
      open={schema !== null}
    >
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,34rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{t('schemas.deleteDialog.title')}</DialogTitle>
        </DialogHeader>

        {schema ? (
          <>
            <DialogBody className="space-y-5 pt-6">
              <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                {t('schemas.deleteDialog.message', {
                  id: schema.schema_id,
                  name: schema.display_name,
                })}
              </p>

              <div className="flex flex-wrap gap-2">
                <Badge className="normal-case px-3 py-1.5" variant="subtle">
                  {schema.schema_id}
                </Badge>
                {schema.tags.map((tag) => (
                  <Badge className="normal-case px-3 py-1.5" key={tag} variant="subtle">
                    {tag}
                  </Badge>
                ))}
              </div>
            </DialogBody>

            <DialogFooter>
              <DialogClose asChild>
                <Button disabled={deleting} variant="ghost">
                  {t('schemas.actions.cancel')}
                </Button>
              </DialogClose>

              <Button
                className="border-[var(--color-state-error-line)] bg-[var(--color-state-error)] text-[var(--color-accent-ink)] hover:bg-[color-mix(in_srgb,var(--color-state-error)_90%,black)]"
                disabled={deleting}
                onClick={onConfirm}
              >
                {deleting ? t('schemas.actions.deleting') : t('schemas.actions.delete')}
              </Button>
            </DialogFooter>
          </>
        ) : null}
      </DialogContent>
    </Dialog>
  )
}
