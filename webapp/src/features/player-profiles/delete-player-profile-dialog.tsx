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
import type { PlayerProfile } from './types'

type DeletePlayerProfileDialogProps = {
  deleting: boolean
  onConfirm: () => void
  onOpenChange: (open: boolean) => void
  targets: ReadonlyArray<PlayerProfile>
}

export function DeletePlayerProfileDialog({
  deleting,
  onConfirm,
  onOpenChange,
  targets,
}: DeletePlayerProfileDialogProps) {
  const { t } = useTranslation()
  const open = targets.length > 0
  const isBulk = targets.length > 1
  const previewTargets = targets.slice(0, 5)

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,32rem)]">
        {open ? (
          <>
            <DialogHeader className="border-b border-[var(--color-border-subtle)]">
              <DialogTitle>
                {isBulk
                  ? t('playerProfiles.deleteDialog.titleMany')
                  : t('playerProfiles.deleteDialog.title')}
              </DialogTitle>
            </DialogHeader>

            <DialogBody className="space-y-5 pt-6">
              <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                {isBulk
                  ? t('playerProfiles.deleteDialog.messageMany', { count: targets.length })
                  : t('playerProfiles.deleteDialog.message', {
                      id: targets[0]?.player_profile_id ?? '—',
                      name: targets[0]?.display_name ?? '—',
                    })}
              </p>

              <div className="flex flex-wrap gap-2">
                {previewTargets.map((target) => (
                  <Badge
                    className="normal-case px-3 py-1.5"
                    key={target.player_profile_id}
                    variant="subtle"
                  >
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
                <Button disabled={deleting} size="md" variant="ghost">
                  {t('playerProfiles.actions.cancel')}
                </Button>
              </DialogClose>

              <Button
                disabled={deleting || targets.length === 0}
                onClick={onConfirm}
                size="md"
                variant="danger"
              >
                {deleting
                  ? t('playerProfiles.actions.deleting')
                  : isBulk
                    ? t('playerProfiles.actions.deleteSelected')
                    : t('playerProfiles.actions.confirmDelete')}
              </Button>
            </DialogFooter>
          </>
        ) : null}
      </DialogContent>
    </Dialog>
  )
}
