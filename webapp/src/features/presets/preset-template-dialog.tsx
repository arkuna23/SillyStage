import type { ReactNode } from 'react'
import { useEffect, useMemo, useState } from 'react'
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
import { cn } from '../../lib/cn'
import type { PresetTemplateDefinition, PresetTemplateKind } from './preset-presets'

type PresetTemplateDialogProps = {
  creating: boolean
  existingPresetIds: ReadonlySet<string>
  onConfirm: (presetKinds: PresetTemplateKind[]) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
  presets: ReadonlyArray<PresetTemplateDefinition>
}

function summarizePresetTemplate(preset: PresetTemplateDefinition) {
  const actorConfig = preset.agents.actor
  const plannerConfig = preset.agents.planner

  return [
    plannerConfig.temperature !== undefined ? `Planner T ${plannerConfig.temperature}` : null,
    actorConfig.temperature !== undefined ? `Actor T ${actorConfig.temperature}` : null,
    actorConfig.max_tokens !== undefined ? `Max ${actorConfig.max_tokens}` : null,
  ].filter((value): value is string => Boolean(value))
}

function PresetCard({
  disabled,
  onClick,
  preset,
  selected,
  status,
  summaryCountLabel,
}: {
  disabled: boolean
  onClick: () => void
  preset: PresetTemplateDefinition
  selected: boolean
  status?: ReactNode
  summaryCountLabel: string
}) {
  const summaryBadges = summarizePresetTemplate(preset)

  return (
    <button
      className={cn(
        'rounded-[1.45rem] border px-4 py-4 text-left transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)]',
        disabled
          ? 'cursor-not-allowed border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] opacity-60'
          : selected
            ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)]'
            : 'border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_84%,transparent)] hover:border-[var(--color-accent-copper-soft)]',
      )}
      disabled={disabled}
      onClick={onClick}
      type="button"
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 space-y-2">
          <div className="space-y-1">
            <h3 className="truncate text-base font-medium text-[var(--color-text-primary)]">
              {preset.displayName}
            </h3>
            <p className="truncate font-mono text-[0.72rem] text-[var(--color-text-muted)]">
              {preset.presetId}
            </p>
          </div>
          <p className="text-sm leading-6 text-[var(--color-text-secondary)]">
            {preset.description}
          </p>
        </div>

        <span
          className={cn(
            'inline-flex size-6 shrink-0 items-center justify-center rounded-full border text-xs',
            selected
              ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold)] text-[var(--color-accent-ink)]'
              : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-transparent',
          )}
        >
          ✓
        </span>
      </div>

      <div className="mt-4 flex flex-wrap items-center gap-2">
        <Badge className="normal-case px-3 py-1.5" variant="subtle">
          {summaryCountLabel}
        </Badge>
        {summaryBadges.map((badge) => (
          <Badge className="normal-case px-3 py-1.5" key={badge} variant="subtle">
            {badge}
          </Badge>
        ))}
        {status}
      </div>
    </button>
  )
}

export function PresetTemplateDialog({
  creating,
  existingPresetIds,
  onConfirm,
  onOpenChange,
  open,
  presets,
}: PresetTemplateDialogProps) {
  const { t } = useTranslation()
  const [selectedPresetKinds, setSelectedPresetKinds] = useState<PresetTemplateKind[]>([])

  const selectablePresets = useMemo(
    () => presets.filter((preset) => !existingPresetIds.has(preset.presetId)),
    [existingPresetIds, presets],
  )

  useEffect(() => {
    if (!open) {
      return
    }

    const frame = window.requestAnimationFrame(() => {
      setSelectedPresetKinds(selectablePresets.map((preset) => preset.kind))
    })

    return () => {
      window.cancelAnimationFrame(frame)
    }
  }, [open, selectablePresets])

  function togglePreset(kind: PresetTemplateKind) {
    setSelectedPresetKinds((currentSelection) =>
      currentSelection.includes(kind)
        ? currentSelection.filter((currentKind) => currentKind !== kind)
        : [...currentSelection, kind],
    )
  }

  async function handleConfirm() {
    if (selectedPresetKinds.length === 0 || creating) {
      return
    }

    await onConfirm(selectedPresetKinds)
  }

  const allExisting = selectablePresets.length === 0

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent
        aria-describedby={undefined}
        className="w-[min(96vw,56rem)] overflow-hidden"
        onEscapeKeyDown={(event) => {
          if (creating) {
            event.preventDefault()
          }
        }}
        onInteractOutside={(event) => {
          if (creating) {
            event.preventDefault()
          }
        }}
      >
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{t('presetsPage.templateDialog.title')}</DialogTitle>
        </DialogHeader>

        <DialogBody className="max-h-[calc(92vh-12rem)] overflow-y-auto pt-6">
          <div className="space-y-5">
            <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
              {t('presetsPage.templateDialog.description')}
            </p>

            {allExisting ? (
              <div className="rounded-[1.35rem] border border-dashed border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-5 text-sm text-[var(--color-text-secondary)]">
                {t('presetsPage.templateDialog.allExisting')}
              </div>
            ) : null}

            <div className="grid gap-3">
              {presets.map((preset) => {
                const alreadyExists = existingPresetIds.has(preset.presetId)

                return (
                  <PresetCard
                    disabled={alreadyExists || creating}
                    key={preset.kind}
                    onClick={() => {
                      togglePreset(preset.kind)
                    }}
                    preset={preset}
                    selected={selectedPresetKinds.includes(preset.kind)}
                    summaryCountLabel={t('presetsPage.templateDialog.agentsCount', { count: 7 })}
                    status={
                      alreadyExists ? (
                        <Badge className="normal-case px-3 py-1.5" variant="info">
                          {t('presetsPage.templateDialog.alreadyExists')}
                        </Badge>
                      ) : null
                    }
                  />
                )
              })}
            </div>
          </div>
        </DialogBody>

        <DialogFooter className="sm:items-center">
          <DialogClose asChild>
            <Button disabled={creating} size="md" variant="ghost">
              {t('presetsPage.actions.cancel')}
            </Button>
          </DialogClose>

          <div className="flex flex-col-reverse gap-3 sm:ml-auto sm:flex-row sm:items-center">
            {!allExisting ? (
              <span className="text-sm text-[var(--color-text-muted)]">
                {t('presetsPage.templateDialog.selectedCount', {
                  count: selectedPresetKinds.length,
                })}
              </span>
            ) : null}
            <Button
              disabled={creating || selectedPresetKinds.length === 0}
              onClick={() => {
                void handleConfirm()
              }}
              size="md"
            >
              {creating
                ? t('presetsPage.templateDialog.creating')
                : t('presetsPage.templateDialog.createSelected')}
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
