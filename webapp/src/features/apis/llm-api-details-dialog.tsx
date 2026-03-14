import { useEffect, useState } from 'react'
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
import { getLlmApi } from './api'
import type { LlmApi } from './types'

type LlmApiDetailsDialogProps = {
  apiId: string | null
  onOpenChange: (open: boolean) => void
  open: boolean
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function DetailRow({
  label,
  value,
}: {
  label: string
  value: string
}) {
  return (
    <div className="rounded-[1.4rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-3.5">
      <p className="text-xs text-[var(--color-text-muted)]">{label}</p>
      <p className="mt-2 text-sm leading-6 text-[var(--color-text-primary)]">{value}</p>
    </div>
  )
}

function providerLabel(api: LlmApi, t: ReturnType<typeof useTranslation>['t']) {
  return api.provider === 'open_ai' ? t('apis.providers.open_ai') : api.provider
}

export function LlmApiDetailsDialog({
  apiId,
  onOpenChange,
  open,
}: LlmApiDetailsDialogProps) {
  const { t } = useTranslation()
  const [api, setApi] = useState<LlmApi | null>(null)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const isLoading = open && apiId !== null && api?.api_id !== apiId && errorMessage === null

  useEffect(() => {
    if (!open || !apiId) {
      return
    }

    const controller = new AbortController()

    void getLlmApi(apiId, controller.signal)
      .then((result) => {
        if (!controller.signal.aborted) {
          setApi(result)
        }
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setErrorMessage(getErrorMessage(error, t('apis.feedback.detailsLoadFailed')))
        }
      })

    return () => {
      controller.abort()
    }
  }, [apiId, open, t])

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,38rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{t('apis.details.title')}</DialogTitle>
        </DialogHeader>

        <DialogBody className="pt-6">
          {errorMessage ? (
            <div className="rounded-[1.25rem] border border-[var(--color-state-error-line)] bg-[var(--color-state-error-soft)] px-4 py-3 text-sm text-[var(--color-text-primary)]">
              {errorMessage}
            </div>
          ) : isLoading ? (
            <div className="grid gap-3">
              {Array.from({ length: 5 }).map((_, index) => (
                <div
                  className="h-20 animate-pulse rounded-[1.4rem] bg-[var(--color-bg-elevated)]"
                  key={index}
                />
              ))}
            </div>
          ) : api ? (
            <div className="grid gap-3">
              <DetailRow label={t('apis.form.fields.apiId')} value={api.api_id} />
              <DetailRow
                label={t('apis.form.fields.provider')}
                value={providerLabel(api, t)}
              />
              <DetailRow label={t('apis.form.fields.baseUrl')} value={api.base_url} />
              <DetailRow label={t('apis.form.fields.model')} value={api.model} />
              <DetailRow
                label={t('apis.details.apiKey')}
                value={
                  api.has_api_key
                    ? (api.api_key_masked ?? t('apis.list.keyConfigured'))
                    : t('apis.list.keyMissing')
                }
              />
            </div>
          ) : null}
        </DialogBody>

        <DialogFooter className="justify-end">
          <DialogClose asChild>
            <Button variant="secondary">{t('apis.actions.close')}</Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
