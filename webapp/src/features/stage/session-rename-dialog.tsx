import { useEffect, useId, useState } from 'react'

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
import { Input } from '../../components/ui/input'
import { useToastMessage } from '../../components/ui/toast-context'
import { updateSession } from './api'
import type { StageCopy } from './copy'
import type { SessionDetail, SessionSummary } from './types'

type SessionRenameDialogProps = {
  copy: StageCopy
  onCompleted: (session: SessionDetail) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
  session: SessionDetail | SessionSummary | null
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

export function SessionRenameDialog({
  copy,
  onCompleted,
  onOpenChange,
  open,
  session,
}: SessionRenameDialogProps) {
  const fieldId = useId()
  const [displayName, setDisplayName] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  useToastMessage(submitError)

  useEffect(() => {
    if (!open || !session) {
      setDisplayName('')
      setIsSubmitting(false)
      setSubmitError(null)
      return
    }

    setDisplayName(session.display_name)
    setIsSubmitting(false)
    setSubmitError(null)
  }, [open, session])

  async function handleSubmit() {
    if (!session) {
      return
    }

    const nextDisplayName = displayName.trim()

    if (nextDisplayName.length === 0) {
      setSubmitError(copy.renameSession.errors.displayNameRequired)
      return
    }

    if (nextDisplayName === session.display_name) {
      onOpenChange(false)
      return
    }

    setIsSubmitting(true)
    setSubmitError(null)

    try {
      const updated = await updateSession(session.session_id, {
        display_name: nextDisplayName,
      })

      await onCompleted(updated)
      onOpenChange(false)
    } catch (error) {
      setSubmitError(getErrorMessage(error, copy.notice.sessionRenameFailed))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,34rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{copy.renameSession.title}</DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
            <p className="text-xs text-[var(--color-text-muted)]">{copy.renameSession.sessionId}</p>
            <p className="mt-2 font-mono text-sm leading-6 text-[var(--color-text-primary)]">
              {session?.session_id ?? '—'}
            </p>
          </div>

          <div className="space-y-2.5">
            <label
              className="block text-sm font-medium text-[var(--color-text-primary)]"
              htmlFor={fieldId}
            >
              {copy.renameSession.displayName}
            </label>
            <Input
              autoFocus
              disabled={isSubmitting}
              id={fieldId}
              name="session-rename-display-name"
              onChange={(event) => {
                setDisplayName(event.target.value)
              }}
              placeholder={copy.renameSession.placeholder}
              value={displayName}
            />
          </div>

        </DialogBody>

        <DialogFooter className="justify-end">
          <DialogClose asChild>
            <Button disabled={isSubmitting} variant="secondary">
              {copy.renameSession.cancel}
            </Button>
          </DialogClose>

          <Button disabled={isSubmitting} onClick={() => void handleSubmit()}>
            {isSubmitting ? copy.renameSession.saving : copy.renameSession.save}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
