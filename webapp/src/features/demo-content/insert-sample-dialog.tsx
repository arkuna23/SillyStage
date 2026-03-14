import {
  Dialog,
  DialogBody,
  DialogClose,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../components/ui/dialog'
import { Button } from '../../components/ui/button'
import { Badge } from '../../components/ui/badge'
import { cn } from '../../lib/cn'

type SampleItemStatus = 'existing' | 'new'

type SampleItem = {
  description?: string
  label: string
  status?: SampleItemStatus
}

type InsertSampleDialogProps = {
  cancelLabel: string
  confirmLabel: string
  confirmDisabled?: boolean
  description?: string
  items: ReadonlyArray<SampleItem>
  newLabel: string
  onConfirm: () => void
  onOpenChange: (open: boolean) => void
  open: boolean
  existingLabel: string
  pending?: boolean
  pendingLabel?: string
  title: string
}

export function InsertSampleDialog({
  cancelLabel,
  confirmLabel,
  confirmDisabled = false,
  description,
  existingLabel,
  items,
  newLabel,
  onConfirm,
  onOpenChange,
  open,
  pending = false,
  pendingLabel,
  title,
}: InsertSampleDialogProps) {
  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,34rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{title}</DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          {description ? (
            <p className="text-sm leading-7 text-[var(--color-text-secondary)]">{description}</p>
          ) : null}

          <div className="space-y-3">
            {items.map((item) => (
              <div
                className="flex items-start justify-between gap-4 rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-3"
                key={item.label}
              >
                <div className="min-w-0 space-y-1">
                  <p className="text-sm font-medium text-[var(--color-text-primary)]">
                    {item.label}
                  </p>
                  {item.description ? (
                    <p className="text-xs leading-6 text-[var(--color-text-muted)]">
                      {item.description}
                    </p>
                  ) : null}
                </div>

                {item.status ? (
                  <Badge
                    className={cn(
                      'shrink-0 normal-case px-3 py-1.5',
                      item.status === 'existing'
                        ? 'border-[var(--color-state-warning-line)] bg-[var(--color-state-warning-soft)] text-[var(--color-text-primary)]'
                        : '',
                    )}
                    variant={item.status === 'existing' ? 'subtle' : 'info'}
                  >
                    {item.status === 'existing' ? existingLabel : newLabel}
                  </Badge>
                ) : null}
              </div>
            ))}
          </div>
        </DialogBody>

        <DialogFooter className="justify-end">
          <DialogClose asChild>
            <Button disabled={pending} variant="secondary">
              {cancelLabel}
            </Button>
          </DialogClose>
          <Button disabled={pending || confirmDisabled} onClick={onConfirm}>
            {pending ? pendingLabel ?? confirmLabel : confirmLabel}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
