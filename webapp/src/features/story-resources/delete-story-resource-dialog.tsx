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
  resource: StoryResource | null
}

export function DeleteStoryResourceDialog({
  deleting,
  onConfirm,
  onOpenChange,
  resource,
}: DeleteStoryResourceDialogProps) {
  const { t } = useTranslation()

  return (
    <Dialog
      onOpenChange={(open) => {
        if (!open) {
          onOpenChange(false)
        }
      }}
      open={resource !== null}
    >
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,34rem)]">
        {resource ? (
          <>
            <DialogHeader className="border-b border-[var(--color-border-subtle)]">
              <DialogTitle>{t('storyResources.deleteDialog.title')}</DialogTitle>
            </DialogHeader>

            <DialogBody className="space-y-5 pt-6">
              <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                {t('storyResources.deleteDialog.message', { id: resource.resource_id })}
              </p>

              <div className="flex flex-wrap gap-2">
                <Badge className="normal-case px-3 py-1.5" variant="subtle">
                  {resource.resource_id}
                </Badge>
                <Badge className="normal-case px-3 py-1.5" variant="subtle">
                  {t('storyResources.list.charactersCount', {
                    count: resource.character_ids.length,
                  })}
                </Badge>
              </div>
            </DialogBody>

            <DialogFooter>
              <DialogClose asChild>
                <Button disabled={deleting} size="md" variant="ghost">
                  {t('storyResources.actions.cancel')}
                </Button>
              </DialogClose>

              <Button
                disabled={deleting}
                onClick={onConfirm}
                size="md"
                variant="danger"
              >
                {deleting
                  ? t('storyResources.actions.deleting')
                  : t('storyResources.actions.confirmDelete')}
              </Button>
            </DialogFooter>
          </>
        ) : null}
      </DialogContent>
    </Dialog>
  )
}
