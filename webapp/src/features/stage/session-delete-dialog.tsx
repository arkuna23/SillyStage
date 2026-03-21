import { Button } from '../../components/ui/button'
import {
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../components/ui/dialog'
import type { StageCopy } from './copy'
import type { SessionSummary } from './types'

type SessionDeleteDialogProps = {
  copy: StageCopy
  deleting: boolean
  onConfirm: () => void
  onOpenChange: (open: boolean) => void
  open: boolean
  session: SessionSummary | null
}

export function SessionDeleteDialog({
  copy,
  deleting,
  onConfirm,
  onOpenChange,
  open,
  session,
}: SessionDeleteDialogProps) {
  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,30rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{copy.deleteSession.title}</DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-4 pt-6">
          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
            {copy.deleteSession.message}
          </p>

          {session ? (
            <div className="space-y-1 text-sm text-[var(--color-text-secondary)]">
              <p className="font-medium text-[var(--color-text-primary)]">{session.display_name}</p>
              <p className="font-mono text-xs text-[var(--color-text-muted)]">
                {session.session_id}
              </p>
            </div>
          ) : null}
        </DialogBody>

        <DialogFooter className="justify-end">
          <Button
            disabled={deleting}
            onClick={() => {
              onOpenChange(false)
            }}
            variant="secondary"
          >
            {copy.createSession.cancel}
          </Button>
          <Button disabled={deleting} onClick={onConfirm} variant="danger">
            {deleting ? copy.deleteSession.deleting : copy.deleteSession.confirm}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
