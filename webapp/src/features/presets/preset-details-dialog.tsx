import { useCallback, useEffect, useMemo, useState } from 'react'
import { faChevronDown } from '@fortawesome/free-solid-svg-icons/faChevronDown'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { AnimatePresence, motion, useReducedMotion } from 'framer-motion'
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
import { cn } from '../../lib/cn'
import { getPreset } from '../apis/api'
import {
  getEnabledPresetModuleEntryCount,
  getPresetModuleCount,
  getPresetModuleEntryCount,
  type PresetDetail,
} from '../apis/types'
import {
  createPresetRoleLabels,
  getOrderedAgentRoleKeys,
  getPromptModuleLabel,
  getPresetEntryKindLabel,
} from './preset-labels'

type PresetDetailsDialogProps = {
  onOpenChange: (open: boolean) => void
  open: boolean
  presetId: string | null
}

type TranslateFn = (key: string, options?: Record<string, unknown>) => string

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function describeAgentSummary(
  agent: PresetDetail['agents'][keyof PresetDetail['agents']],
  t: TranslateFn,
  fallback: string,
) {
  const parts = [
    agent.temperature !== undefined && agent.temperature !== null ? `T ${agent.temperature}` : null,
    agent.max_tokens !== undefined && agent.max_tokens !== null ? `Max ${agent.max_tokens}` : null,
    agent.extra ? t('presetsPage.details.extra') : null,
    getPresetModuleEntryCount(agent) > 0
      ? t('presetsPage.details.moduleSummary', {
          count: getPresetModuleCount(agent),
          enabled: getEnabledPresetModuleEntryCount(agent),
          entries: getPresetModuleEntryCount(agent),
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
  roleLabels: ReturnType<typeof createPresetRoleLabels>
  t: TranslateFn
}) {
  const prefersReducedMotion = useReducedMotion()
  const [preset, setPreset] = useState<PresetDetail | null>(null)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const [expandedModules, setExpandedModules] = useState<string[]>([])
  useToastMessage(errorMessage)

  useEffect(() => {
    const controller = new AbortController()

    void getPreset(presetId, controller.signal)
      .then((result) => {
        if (!controller.signal.aborted) {
          setErrorMessage(null)
          setPreset(result)
          setExpandedModules([])
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

  function toggleModule(moduleKey: string) {
    setExpandedModules((current) =>
      current.includes(moduleKey)
        ? current.filter((key) => key !== moduleKey)
        : [...current, moduleKey],
    )
  }

  return (
    <DialogBody className="space-y-5 pt-6">
      {preset ? (
        <>
          <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
            <p className="text-xs text-[var(--color-text-muted)]">
              {t('presetsPage.form.fields.presetId')}
            </p>
            <p className="mt-2 font-medium text-[var(--color-text-primary)]">{preset.preset_id}</p>
            <p className="mt-4 text-xs text-[var(--color-text-muted)]">
              {t('presetsPage.form.fields.displayName')}
            </p>
            <p className="mt-2 text-sm text-[var(--color-text-primary)]">{preset.display_name}</p>
          </div>

          <div className="space-y-4">
            {getOrderedAgentRoleKeys().map((roleKey) => {
              const agent = preset.agents[roleKey]

              return (
                <div
                  className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4"
                  key={roleKey}
                >
                  <div className="flex flex-wrap items-center justify-between gap-3">
                    <div className="space-y-2">
                      <p className="text-sm font-medium text-[var(--color-text-primary)]">
                        {roleLabels[roleKey]}
                      </p>
                      <p className="text-sm text-[var(--color-text-secondary)]">
                        {describeAgentSummary(agent, t, t('presetsPage.list.unset'))}
                      </p>
                    </div>

                    <div className="flex flex-wrap gap-2">
                      <Badge variant="subtle">
                        {t('presetsPage.details.moduleCount', {
                          count: getPresetModuleCount(agent),
                        })}
                      </Badge>
                      <Badge variant="subtle">
                        {t('presetsPage.details.entryCount', {
                          count: getPresetModuleEntryCount(agent),
                        })}
                      </Badge>
                    </div>
                  </div>

                  <div className="mt-4 space-y-3">
                    {agent.modules.length > 0 ? (
                      agent.modules.map((module) => {
                        const moduleKey = `${roleKey}:${module.module_id}`
                        const isExpanded = expandedModules.includes(moduleKey)

                        return (
                          <div
                            className="overflow-hidden rounded-[1.15rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_72%,transparent)]"
                            key={moduleKey}
                          >
                            <button
                              aria-expanded={isExpanded}
                              className="flex w-full items-center justify-between gap-4 px-4 py-4 text-left transition duration-200 hover:bg-white/5"
                              onClick={() => {
                                toggleModule(moduleKey)
                              }}
                              type="button"
                            >
                              <div className="min-w-0 space-y-1">
                                <p className="text-sm font-medium text-[var(--color-text-primary)]">
                                  {getPromptModuleLabel(t, module.module_id)}
                                </p>
                                <p className="text-xs text-[var(--color-text-muted)]">
                                  {t('presetsPage.details.entryCount', {
                                    count: module.entries.length,
                                  })}
                                </p>
                              </div>

                              <motion.span
                                animate={{ rotate: isExpanded ? 180 : 0 }}
                                className="inline-flex h-9 w-9 items-center justify-center rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-[var(--color-text-secondary)]"
                                transition={
                                  prefersReducedMotion
                                    ? { duration: 0 }
                                    : { duration: 0.2, ease: [0.22, 1, 0.36, 1] }
                                }
                              >
                                <FontAwesomeIcon icon={faChevronDown} />
                              </motion.span>
                            </button>

                            <AnimatePresence initial={false}>
                              {isExpanded ? (
                                <motion.div
                                  animate={{ height: 'auto', opacity: 1, y: 0 }}
                                  className="overflow-hidden"
                                  exit={{
                                    height: 0,
                                    opacity: 0,
                                    y: prefersReducedMotion ? 0 : -8,
                                  }}
                                  initial={{
                                    height: 0,
                                    opacity: 0,
                                    y: prefersReducedMotion ? 0 : -8,
                                  }}
                                  transition={
                                    prefersReducedMotion
                                      ? { duration: 0 }
                                      : { duration: 0.22, ease: [0.22, 1, 0.36, 1] }
                                  }
                                >
                                  <div className="space-y-3 border-t border-[var(--color-border-subtle)] px-4 py-4">
                                    {module.entries.map((entry) => (
                                      <div
                                        className="rounded-[1rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4"
                                        key={`${moduleKey}:${entry.entry_id}`}
                                      >
                                        <div className="flex flex-wrap items-center justify-between gap-3">
                                          <div className="min-w-0 space-y-1">
                                            <p className="truncate text-sm font-medium text-[var(--color-text-primary)]">
                                              {entry.display_name}
                                            </p>
                                            <p className="truncate text-xs text-[var(--color-text-muted)]">
                                              {entry.entry_id}
                                            </p>
                                          </div>

                                          <div className="flex flex-wrap gap-2">
                                            <Badge variant="subtle">
                                              {getPresetEntryKindLabel(t, entry.kind)}
                                            </Badge>
                                            <Badge variant={entry.enabled ? 'gold' : 'subtle'}>
                                              {entry.enabled
                                                ? t('presetsPage.details.enabled')
                                                : t('presetsPage.details.disabled')}
                                            </Badge>
                                            {entry.required ? (
                                              <Badge variant="info">
                                                {t('presetsPage.details.required')}
                                              </Badge>
                                            ) : null}
                                          </div>
                                        </div>

                                        <div className="mt-4 grid gap-4 md:grid-cols-2">
                                          <div className="space-y-1">
                                            <p className="text-xs text-[var(--color-text-muted)]">
                                              {t('presetsPage.details.order')}
                                            </p>
                                            <p className="text-sm text-[var(--color-text-primary)]">
                                              {entry.order}
                                            </p>
                                          </div>

                                          {entry.kind === 'built_in_context_ref' ? (
                                            <div className="space-y-1">
                                              <p className="text-xs text-[var(--color-text-muted)]">
                                                {t('presetsPage.details.contextKey')}
                                              </p>
                                              <p className="break-all text-sm text-[var(--color-text-primary)]">
                                                {entry.context_key ?? '—'}
                                              </p>
                                            </div>
                                          ) : (
                                            <div className="space-y-1 md:col-span-2">
                                              <p className="text-xs text-[var(--color-text-muted)]">
                                                {t('presetsPage.details.entryText')}
                                              </p>
                                              <p
                                                className={cn(
                                                  'whitespace-pre-wrap text-sm leading-7 text-[var(--color-text-primary)]',
                                                  !entry.text && 'text-[var(--color-text-muted)]',
                                                )}
                                              >
                                                {entry.text?.trim() || t('presetsPage.details.noText')}
                                              </p>
                                            </div>
                                          )}
                                        </div>
                                      </div>
                                    ))}
                                  </div>
                                </motion.div>
                              ) : null}
                            </AnimatePresence>
                          </div>
                        )
                      })
                    ) : (
                      <p className="text-xs leading-6 text-[var(--color-text-muted)]">
                        {t('presetsPage.details.noModules')}
                      </p>
                    )}
                  </div>
                </div>
              )
            })}
          </div>
        </>
      ) : errorMessage ? null : (
        <div className="space-y-4">
          <div className="h-12 animate-pulse rounded-[1.35rem] bg-[var(--color-bg-elevated)]" />
          {Array.from({ length: 3 }).map((_, index) => (
            <div
              className="h-44 animate-pulse rounded-[1.35rem] bg-[var(--color-bg-elevated)]"
              key={index}
            />
          ))}
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
  const roleLabels = useMemo(() => createPresetRoleLabels(translate), [translate])

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,58rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{t('presetsPage.details.title')}</DialogTitle>
        </DialogHeader>

        {open && presetId ? (
          <PresetDetailsContent
            key={presetId}
            presetId={presetId}
            roleLabels={roleLabels}
            t={translate}
          />
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
