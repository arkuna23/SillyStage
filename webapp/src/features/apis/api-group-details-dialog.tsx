import { useEffect, useMemo, useState } from 'react'
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
import { getApiGroup } from './api'
import {
  type AgentRoleKey,
  type ApiConfig,
  type ApiGroup,
  agentRoleKeys,
  getAgentBindingKey,
} from './types'

type ApiGroupDetailsDialogProps = {
  apiGroupId: string | null
  apis: ReadonlyArray<ApiConfig>
  onOpenChange: (open: boolean) => void
  open: boolean
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

export function ApiGroupDetailsDialog({
  apiGroupId,
  apis,
  onOpenChange,
  open,
}: ApiGroupDetailsDialogProps) {
  const { t } = useTranslation()
  const [apiGroup, setApiGroup] = useState<ApiGroup | null>(null)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  useToastMessage(errorMessage)

  useEffect(() => {
    if (!open || !apiGroupId) {
      return
    }

    const controller = new AbortController()

    void getApiGroup(apiGroupId, controller.signal)
      .then((result) => {
        if (!controller.signal.aborted) {
          setApiGroup(result)
        }
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setErrorMessage(getErrorMessage(error, t('apis.groupFeedback.detailsLoadFailed')))
        }
      })

    return () => {
      controller.abort()
    }
  }, [apiGroupId, open, t])

  const roleLabels: Record<AgentRoleKey, string> = useMemo(
    () => ({
      actor: t('apis.roles.actor'),
      architect: t('apis.roles.architect'),
      director: t('apis.roles.director'),
      keeper: t('apis.roles.keeper'),
      narrator: t('apis.roles.narrator'),
      planner: t('apis.roles.planner'),
      replyer: t('apis.roles.replyer'),
    }),
    [t],
  )

  function getApiSummary(apiId: string) {
    return apis.find((entry) => entry.api_id === apiId) ?? null
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,52rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{t('apis.groupDetails.title')}</DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          {apiGroup ? (
            <>
              <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                <p className="text-xs text-[var(--color-text-muted)]">
                  {t('apis.groupForm.fields.apiGroupId')}
                </p>
                <p className="mt-2 font-medium text-[var(--color-text-primary)]">
                  {apiGroup.api_group_id}
                </p>
                <p className="mt-4 text-xs text-[var(--color-text-muted)]">
                  {t('apis.groupForm.fields.displayName')}
                </p>
                <p className="mt-2 text-sm text-[var(--color-text-primary)]">
                  {apiGroup.display_name}
                </p>
              </div>

              <div className="grid gap-4 md:grid-cols-2">
                {agentRoleKeys.map((roleKey) => {
                  const apiId = apiGroup.bindings[getAgentBindingKey(roleKey)]
                  const apiConfig = getApiSummary(apiId)

                  return (
                    <div
                      className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4"
                      key={roleKey}
                    >
                      <div className="flex items-center justify-between gap-3">
                        <p className="text-sm font-medium text-[var(--color-text-primary)]">
                          {roleLabels[roleKey]}
                        </p>
                        <Badge variant={apiConfig ? 'info' : 'subtle'}>
                          {apiConfig ? t('apis.groupList.bound') : t('apis.groupList.missing')}
                        </Badge>
                      </div>
                      <div className="mt-4 space-y-2 text-sm text-[var(--color-text-secondary)]">
                        <p className="font-medium text-[var(--color-text-primary)]">
                          {apiConfig?.display_name ?? t('apis.groupForm.missingApi', { id: apiId })}
                        </p>
                        <p className="text-xs text-[var(--color-text-muted)]">{apiId}</p>
                        {apiConfig ? <p>{apiConfig.model}</p> : null}
                      </div>
                    </div>
                  )
                })}
              </div>
            </>
          ) : errorMessage ? null : (
            <div className="space-y-4">
              <div className="h-12 animate-pulse rounded-[1.35rem] bg-[var(--color-bg-elevated)]" />
              <div className="grid gap-4 md:grid-cols-2">
                {Array.from({ length: agentRoleKeys.length }).map((_, index) => (
                  <div
                    className="h-28 animate-pulse rounded-[1.35rem] bg-[var(--color-bg-elevated)]"
                    key={index}
                  />
                ))}
              </div>
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
