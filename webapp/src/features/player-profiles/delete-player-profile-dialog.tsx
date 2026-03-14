import { useTranslation } from 'react-i18next'

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
  open: boolean
  profile: PlayerProfile | null
}

export function DeletePlayerProfileDialog({
  deleting,
  onConfirm,
  onOpenChange,
  open,
  profile,
}: DeletePlayerProfileDialogProps) {
  const { t } = useTranslation()

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,32rem)]">
        {profile ? (
          <>
            <DialogHeader className="border-b border-[var(--color-border-subtle)]">
              <DialogTitle>{t('playerProfiles.deleteDialog.title')}</DialogTitle>
            </DialogHeader>

            <DialogBody className="space-y-5 pt-6">
              <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                {t('playerProfiles.deleteDialog.message', {
                  id: profile.player_profile_id,
                  name: profile.display_name,
                })}
              </p>
            </DialogBody>

            <DialogFooter>
              <DialogClose asChild>
                <Button disabled={deleting} size="md" variant="ghost">
                  {t('playerProfiles.actions.cancel')}
                </Button>
              </DialogClose>

              <Button
                disabled={deleting}
                onClick={onConfirm}
                size="md"
                variant="danger"
              >
                {deleting
                  ? t('playerProfiles.actions.deleting')
                  : t('playerProfiles.actions.confirmDelete')}
              </Button>
            </DialogFooter>
          </>
        ) : null}
      </DialogContent>
    </Dialog>
  )
}
