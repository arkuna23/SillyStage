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
import type { CharacterSummary } from './types'

type DeleteCharacterDialogProps = {
  deleting: boolean
  onConfirm: () => void
  onOpenChange: (open: boolean) => void
  targets: ReadonlyArray<CharacterSummary>
}

export function DeleteCharacterDialog({
  deleting,
  onConfirm,
  onOpenChange,
  targets,
}: DeleteCharacterDialogProps) {
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
            {isBulk
              ? t('characters.deleteDialog.titleMany')
              : t('characters.deleteDialog.titleOne')}
          </DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
            {isBulk
              ? t('characters.deleteDialog.messageMany', { count: targets.length })
              : t('characters.deleteDialog.messageOne', {
                  name: targets[0]?.name ?? '',
                })}
          </p>

          <div className="flex flex-wrap gap-2">
            {previewTargets.map((target) => (
              <Badge className="normal-case px-3 py-1.5" key={target.character_id} variant="subtle">
                {target.name}
              </Badge>
            ))}
            {targets.length > previewTargets.length ? (
              <Badge className="normal-case px-3 py-1.5" variant="subtle">
                {t('characters.deleteDialog.more', {
                  count: targets.length - previewTargets.length,
                })}
              </Badge>
            ) : null}
          </div>
        </DialogBody>

        <DialogFooter>
          <DialogClose asChild>
            <Button disabled={deleting} variant="ghost">
              {t('characters.actions.cancel')}
            </Button>
          </DialogClose>

          <Button
            disabled={deleting}
            onClick={onConfirm}
            variant="danger"
          >
            {deleting
              ? t('characters.actions.deleting')
              : isBulk
                ? t('characters.actions.deleteSelected')
                : t('characters.actions.delete')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
