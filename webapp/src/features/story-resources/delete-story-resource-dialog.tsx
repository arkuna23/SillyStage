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
import type { StoryResource } from './types'

type DeleteStoryResourceDialogProps = {
  deleting: boolean
  onConfirm: () => void
  onOpenChange: (open: boolean) => void
  targets: ReadonlyArray<StoryResource>
}

export function DeleteStoryResourceDialog({
  deleting,
  onConfirm,
  onOpenChange,
  targets,
}: DeleteStoryResourceDialogProps) {
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
        {open ? (
          <>
            <DialogHeader className="border-b border-[var(--color-border-subtle)]">
              <DialogTitle>
                {isBulk
                  ? t('storyResources.deleteDialog.titleMany')
                  : t('storyResources.deleteDialog.title')}
              </DialogTitle>
            </DialogHeader>

            <DialogBody className="space-y-5 pt-6">
              <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                {isBulk
                  ? t('storyResources.deleteDialog.messageMany', { count: targets.length })
                  : t('storyResources.deleteDialog.message', {
                      id: targets[0]?.resource_id ?? '—',
                    })}
              </p>

              <div className="flex flex-wrap gap-2">
                {previewTargets.map((target) => (
                  <Badge
                    className="normal-case px-3 py-1.5"
                    key={target.resource_id}
                    variant="subtle"
                  >
                    {target.resource_id}
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
                  {t('storyResources.actions.cancel')}
                </Button>
              </DialogClose>

              <Button
                disabled={deleting || targets.length === 0}
                onClick={onConfirm}
                size="md"
                variant="danger"
              >
                {deleting
                  ? t('storyResources.actions.deleting')
                  : isBulk
                    ? t('storyResources.actions.deleteSelected')
                    : t('storyResources.actions.confirmDelete')}
              </Button>
            </DialogFooter>
          </>
        ) : null}
      </DialogContent>
    </Dialog>
  )
}
