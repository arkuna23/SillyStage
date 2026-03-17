import { useCallback, useEffect, useMemo, useState } from 'react'
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
import { getPreset } from '../apis/api'
import {
  agentRoleKeys,
  getEnabledPresetPromptEntryCount,
  getPresetPromptEntries,
  getPresetPromptEntryCount,
  type AgentRoleKey,
  type Preset,
} from '../apis/types'

type PresetDetailsDialogProps = {
  onOpenChange: (open: boolean) => void
  open: boolean
  presetId: string | null
}

type TranslateFn = (key: string, options?: Record<string, unknown>) => string

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function describePresetAgent(
  agent: Preset['agents'][AgentRoleKey],
  t: TranslateFn,
  fallback: string,
) {
  const promptEntryCount = getPresetPromptEntryCount(agent)
  const parts = [
    agent.temperature !== undefined && agent.temperature !== null
      ? `T ${agent.temperature}`
      : null,
    agent.max_tokens !== undefined && agent.max_tokens !== null ? `Max ${agent.max_tokens}` : null,
    agent.extra ? t('presetsPage.details.extra') : null,
    promptEntryCount > 0
      ? t('presetsPage.details.promptEntriesSummary', {
          count: promptEntryCount,
          enabled: getEnabledPresetPromptEntryCount(agent),
        })
      : null,
  ].filter((value): value is string => Boolean(value))

  return parts.length > 0 ? parts.join(' · ') : fallback
}

function PresetDetailsContent({
  presetId,
  roleLabels,
  t,
}: {
  presetId: string
  roleLabels: Record<AgentRoleKey, string>
  t: TranslateFn
}) {
  const [preset, setPreset] = useState<Preset | null>(null)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  useToastMessage(errorMessage)

  useEffect(() => {
    const controller = new AbortController()

    void getPreset(presetId, controller.signal)
      .then((result) => {
        if (!controller.signal.aborted) {
          setErrorMessage(null)
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
  }, [presetId, t])

  return (
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
                  <div className="flex flex-wrap justify-end gap-2">
                    <Badge variant="subtle">
                      {preset.agents[roleKey].extra
                        ? t('presetsPage.details.extra')
                        : t('presetsPage.details.noExtra')}
                    </Badge>
                    <Badge variant="subtle">
                      {t('presetsPage.details.promptEntriesCount', {
                        count: getPresetPromptEntryCount(preset.agents[roleKey]),
                      })}
                    </Badge>
                  </div>
                </div>
                <p className="mt-4 text-sm text-[var(--color-text-secondary)]">
                  {describePresetAgent(preset.agents[roleKey], t, t('presetsPage.list.unset'))}
                </p>
                {getPresetPromptEntries(preset.agents[roleKey]).length > 0 ? (
                  <div className="mt-4 space-y-2">
                    {getPresetPromptEntries(preset.agents[roleKey]).map((entry) => (
                      <div
                        className="rounded-[1rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_72%,transparent)] px-3 py-3"
                        key={`${roleKey}-${entry.entry_id}`}
                      >
                        <div className="flex items-center justify-between gap-3">
                          <div className="min-w-0">
                            <p className="truncate text-sm font-medium text-[var(--color-text-primary)]">
                              {entry.title}
                            </p>
                            <p className="truncate text-xs text-[var(--color-text-muted)]">
                              {entry.entry_id}
                            </p>
                          </div>
                          <Badge variant={entry.enabled ? 'gold' : 'subtle'}>
                            {entry.enabled
                              ? t('presetsPage.details.enabled')
                              : t('presetsPage.details.disabled')}
                          </Badge>
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="mt-4 text-xs leading-6 text-[var(--color-text-muted)]">
                    {t('presetsPage.details.noPromptEntries')}
                  </p>
                )}
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
  )
}

export function PresetDetailsDialog({
  onOpenChange,
  open,
  presetId,
}: PresetDetailsDialogProps) {
  const { t } = useTranslation()
  const translate = useCallback(
    (key: string, options?: Record<string, unknown>) =>
      String(t(key as never, options as never)),
    [t],
  )

  const roleLabels: Record<AgentRoleKey, string> = useMemo(
    () => ({
      actor: translate('presetsPage.roles.actor'),
      architect: translate('presetsPage.roles.architect'),
      director: translate('presetsPage.roles.director'),
      keeper: translate('presetsPage.roles.keeper'),
      narrator: translate('presetsPage.roles.narrator'),
      planner: translate('presetsPage.roles.planner'),
      replyer: translate('presetsPage.roles.replyer'),
    }),
    [translate],
  )

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,52rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{t('presetsPage.details.title')}</DialogTitle>
        </DialogHeader>

        {open && presetId ? (
          <PresetDetailsContent key={presetId} presetId={presetId} roleLabels={roleLabels} t={translate} />
        ) : (
          <DialogBody className="pt-6" />
        )}

        <DialogFooter className="justify-end">
          <Button onClick={() => onOpenChange(false)} variant="ghost">
            {t('presetsPage.actions.close')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
