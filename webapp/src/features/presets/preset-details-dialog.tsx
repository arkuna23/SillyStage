import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import {
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../components/ui/dialog'
import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import { useToastMessage } from '../../components/ui/toast-context'
import { getPreset } from '../apis/api'
import { agentRoleKeys, type AgentRoleKey, type Preset } from '../apis/types'

type PresetDetailsDialogProps = {
  onOpenChange: (open: boolean) => void
  open: boolean
  presetId: string | null
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function describePresetAgent(agent: Preset['agents'][AgentRoleKey], fallback: string) {
  const parts = [
    agent.temperature !== undefined && agent.temperature !== null
      ? `T ${agent.temperature}`
      : null,
    agent.max_tokens !== undefined && agent.max_tokens !== null ? `Max ${agent.max_tokens}` : null,
    agent.extra ? 'extra' : null,
  ].filter((value): value is string => Boolean(value))

  return parts.length > 0 ? parts.join(' · ') : fallback
}

export function PresetDetailsDialog({
  onOpenChange,
  open,
  presetId,
}: PresetDetailsDialogProps) {
  const { t } = useTranslation()
  const [preset, setPreset] = useState<Preset | null>(null)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  useToastMessage(errorMessage)

  const roleLabels: Record<AgentRoleKey, string> = useMemo(
    () => ({
      actor: t('presetsPage.roles.actor'),
      architect: t('presetsPage.roles.architect'),
      director: t('presetsPage.roles.director'),
      keeper: t('presetsPage.roles.keeper'),
      narrator: t('presetsPage.roles.narrator'),
      planner: t('presetsPage.roles.planner'),
      replyer: t('presetsPage.roles.replyer'),
    }),
    [t],
  )

  useEffect(() => {
    if (!open || !presetId) {
      return
    }

    const controller = new AbortController()

    void getPreset(presetId, controller.signal)
      .then((result) => {
        if (!controller.signal.aborted) {
          setPreset(result)
        }
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setErrorMessage(getErrorMessage(error, t('presetsPage.feedback.loadPresetFailed')))
        }
      })

    return () => {
      controller.abort()
    }
  }, [open, presetId, t])

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,52rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{t('presetsPage.details.title')}</DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          {preset ? (
            <>
              <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                <p className="text-xs text-[var(--color-text-muted)]">{t('presetsPage.form.fields.presetId')}</p>
                <p className="mt-2 font-medium text-[var(--color-text-primary)]">{preset.preset_id}</p>
                <p className="mt-4 text-xs text-[var(--color-text-muted)]">{t('presetsPage.form.fields.displayName')}</p>
                <p className="mt-2 text-sm text-[var(--color-text-primary)]">{preset.display_name}</p>
              </div>

              <div className="grid gap-4 md:grid-cols-2">
                {agentRoleKeys.map((roleKey) => (
                  <div
                    className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4"
                    key={roleKey}
                  >
                    <div className="flex items-center justify-between gap-3">
                      <p className="text-sm font-medium text-[var(--color-text-primary)]">
                        {roleLabels[roleKey]}
                      </p>
                      <Badge variant="subtle">
                        {preset.agents[roleKey].extra ? 'extra' : t('presetsPage.details.noExtra')}
                      </Badge>
                    </div>
                    <p className="mt-4 text-sm text-[var(--color-text-secondary)]">
                      {describePresetAgent(preset.agents[roleKey], t('presetsPage.list.unset'))}
                    </p>
                  </div>
                ))}
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
            {t('presetsPage.actions.close')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
