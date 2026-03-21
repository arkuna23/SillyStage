import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import {
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../components/ui/dialog'
import { useToastMessage } from '../../components/ui/toast-context'
import { getApi } from './api'
import type { ApiConfig } from './types'

type ApiDetailsDialogProps = {
  apiId: string | null
  onOpenChange: (open: boolean) => void
  open: boolean
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

export function ApiDetailsDialog({ apiId, onOpenChange, open }: ApiDetailsDialogProps) {
  const { t } = useTranslation()
  const [apiConfig, setApiConfig] = useState<ApiConfig | null>(null)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  useToastMessage(errorMessage)

  useEffect(() => {
    if (!open || !apiId) {
      return
    }

    const controller = new AbortController()

    void getApi(apiId, controller.signal)
      .then((result) => {
        if (!controller.signal.aborted) {
          setApiConfig(result)
        }
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setErrorMessage(getErrorMessage(error, t('apis.apiFeedback.detailsLoadFailed')))
        }
      })

    return () => {
      controller.abort()
    }
  }, [apiId, open, t])

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,40rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{t('apis.apiDetails.title')}</DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          {apiConfig ? (
            <>
              <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                <p className="text-xs text-[var(--color-text-muted)]">
                  {t('apis.apiForm.fields.apiId')}
                </p>
                <p className="mt-2 font-medium text-[var(--color-text-primary)]">
                  {apiConfig.api_id}
                </p>
                <p className="mt-4 text-xs text-[var(--color-text-muted)]">
                  {t('apis.apiForm.fields.displayName')}
                </p>
                <p className="mt-2 text-sm text-[var(--color-text-primary)]">
                  {apiConfig.display_name}
                </p>
              </div>

              <div className="grid gap-4 md:grid-cols-2">
                <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                  <p className="text-xs text-[var(--color-text-muted)]">
                    {t('apis.apiForm.fields.provider')}
                  </p>
                  <p className="mt-2 text-sm text-[var(--color-text-primary)]">
                    {apiConfig.provider === 'open_ai'
                      ? t('apis.providers.open_ai')
                      : apiConfig.provider}
                  </p>
                </div>

                <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                  <p className="text-xs text-[var(--color-text-muted)]">
                    {t('apis.apiForm.fields.model')}
                  </p>
                  <p className="mt-2 text-sm text-[var(--color-text-primary)]">{apiConfig.model}</p>
                </div>

                <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4 md:col-span-2">
                  <p className="text-xs text-[var(--color-text-muted)]">
                    {t('apis.apiForm.fields.baseUrl')}
                  </p>
                  <p className="mt-2 break-all text-sm text-[var(--color-text-primary)]">
                    {apiConfig.base_url}
                  </p>
                </div>

                <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4 md:col-span-2">
                  <div className="flex items-center justify-between gap-3">
                    <p className="text-xs text-[var(--color-text-muted)]">
                      {t('apis.apiDetails.keyStatus')}
                    </p>
                    <Badge variant={apiConfig.has_api_key ? 'info' : 'subtle'}>
                      {apiConfig.has_api_key
                        ? (apiConfig.api_key_masked ?? t('apis.apiList.keyConfigured'))
                        : t('apis.apiList.keyMissing')}
                    </Badge>
                  </div>
                </div>
              </div>
            </>
          ) : errorMessage ? null : (
            <div className="space-y-4">
              {Array.from({ length: 4 }).map((_, index) => (
                <div
                  className="h-16 animate-pulse rounded-[1.35rem] bg-[var(--color-bg-elevated)]"
                  key={index}
                />
              ))}
            </div>
          )}
        </DialogBody>

        <DialogFooter className="justify-end">
          <Button onClick={() => onOpenChange(false)} variant="ghost">
            {t('apis.actions.close')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
