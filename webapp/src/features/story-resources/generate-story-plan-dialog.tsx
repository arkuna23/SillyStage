import { useEffect, useId, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { appPaths } from '../../app/paths'
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
import { DialogRouteButton } from '../../components/ui/dialog-route-button'
import { GenerationLoadingStage } from '../../components/ui/generation-loading-stage'
import { Select } from '../../components/ui/select'
import { useToastMessage } from '../../components/ui/toast-context'
import type { ApiGroup, Preset } from '../apis/types'
import { generateAndSaveStoryPlan } from './api'
import { getStoryResourceDisplayName } from './story-resource-display'
import type { StoryResource } from './types'

type NoticeTone = 'error' | 'success' | 'warning'

type GenerateStoryPlanDialogProps = {
  apiGroups: ReadonlyArray<ApiGroup>
  onCompleted: (result: {
    message: string
    resource: StoryResource
    tone: NoticeTone
  }) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
  presets: ReadonlyArray<Preset>
  resource: StoryResource | null
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

export function GenerateStoryPlanDialog({
  apiGroups,
  onCompleted,
  onOpenChange,
  open,
  presets,
  resource,
}: GenerateStoryPlanDialogProps) {
  const { t } = useTranslation()
  const fieldIdPrefix = useId()
  const [apiGroupId, setApiGroupId] = useState('')
  const [presetId, setPresetId] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  const [generationStartedAtMs, setGenerationStartedAtMs] = useState<number | null>(null)
  useToastMessage(submitError)

  const fieldIds = {
    apiGroupId: `${fieldIdPrefix}-api-group-id`,
    presetId: `${fieldIdPrefix}-preset-id`,
  } as const

  const apiGroupOptions = useMemo(
    () =>
      apiGroups.map((apiGroup) => ({
        label: apiGroup.display_name,
        value: apiGroup.api_group_id,
      })),
    [apiGroups],
  )
  const presetOptions = useMemo(
    () =>
      presets.map((preset) => ({
        label: preset.display_name,
        value: preset.preset_id,
      })),
    [presets],
  )
  const bindingsUnavailable = apiGroups.length === 0 || presets.length === 0

  useEffect(() => {
    if (!open) {
      setApiGroupId('')
      setPresetId('')
      setIsSubmitting(false)
      setSubmitError(null)
      setGenerationStartedAtMs(null)
    }
  }, [open])

  async function handleSubmit() {
    if (!resource) {
      return
    }

    if (apiGroupId.trim().length === 0) {
      setSubmitError(t('storyResources.form.errors.apiGroupRequired'))
      return
    }

    if (presetId.trim().length === 0) {
      setSubmitError(t('storyResources.form.errors.presetRequired'))
      return
    }

    setSubmitError(null)
    setIsSubmitting(true)
    setGenerationStartedAtMs(Date.now())

    try {
      const generated = await generateAndSaveStoryPlan({
        apiGroupId,
        presetId,
        resourceId: resource.resource_id,
      })

      await onCompleted({
        message: t('storyResources.feedback.generated', { id: resource.resource_id }),
        resource: generated.resource,
        tone: 'success',
      })

      onOpenChange(false)
    } catch (error) {
      setGenerationStartedAtMs(null)
      setSubmitError(getErrorMessage(error, t('storyResources.feedback.generateFailed')))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent
        aria-describedby={undefined}
        className="w-[min(92vw,42rem)]"
        onEscapeKeyDown={(event) => {
          if (isSubmitting) {
            event.preventDefault()
          }
        }}
        onInteractOutside={(event) => {
          if (isSubmitting) {
            event.preventDefault()
          }
        }}
      >
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{t('storyResources.planDialog.title')}</DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          {generationStartedAtMs !== null && resource ? (
            <GenerationLoadingStage
              description={t('storyResources.planDialog.generatingDescription', {
                id: resource.resource_id,
              })}
              elapsedLabel={t('storyResources.createWizard.loading.elapsed')}
              identifier={resource.resource_id}
              startedAtMs={generationStartedAtMs}
              statusLabel={t('storyResources.actions.generating')}
              title={t('storyResources.planDialog.generatingTitle')}
            />
          ) : bindingsUnavailable ? (
            <div className="space-y-5">
              <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
                {apiGroups.length === 0
                  ? t('storyResources.form.emptyApiGroups')
                  : t('storyResources.form.emptyPresets')}
              </div>

              <div className="flex justify-end">
                <DialogRouteButton
                  onRequestClose={() => {
                    onOpenChange(false)
                  }}
                  to={apiGroups.length === 0 ? appPaths.apis : appPaths.presets}
                  variant="secondary"
                >
                  {apiGroups.length === 0
                    ? t('storyResources.form.openApiGroups')
                    : t('storyResources.form.openPresets')}
                </DialogRouteButton>
              </div>
            </div>
          ) : (
            <>
              {resource ? (
                <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                  <p className="text-xs text-[var(--color-text-muted)]">
                    {t('storyResources.form.fields.displayName')}
                  </p>
                  <p className="mt-2 text-sm leading-7 text-[var(--color-text-primary)]">
                    {getStoryResourceDisplayName(resource)}
                  </p>
                  <p className="mt-1 font-mono text-xs leading-6 text-[var(--color-text-muted)]">
                    {resource.resource_id}
                  </p>
                </div>
              ) : null}

              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2.5">
                  <label
                    className="block text-sm font-medium text-[var(--color-text-primary)]"
                    htmlFor={fieldIds.apiGroupId}
                  >
                    {t('storyResources.form.fields.apiGroupId')}
                  </label>
                  <Select
                    items={apiGroupOptions}
                    onValueChange={setApiGroupId}
                    placeholder={t('storyResources.form.placeholders.apiGroupId')}
                    textAlign="start"
                    triggerId={fieldIds.apiGroupId}
                    value={apiGroupId || undefined}
                  />
                </div>

                <div className="space-y-2.5">
                  <label
                    className="block text-sm font-medium text-[var(--color-text-primary)]"
                    htmlFor={fieldIds.presetId}
                  >
                    {t('storyResources.form.fields.presetId')}
                  </label>
                  <Select
                    items={presetOptions}
                    onValueChange={setPresetId}
                    placeholder={t('storyResources.form.placeholders.presetId')}
                    textAlign="start"
                    triggerId={fieldIds.presetId}
                    value={presetId || undefined}
                  />
                </div>
              </div>
            </>
          )}
        </DialogBody>

        {generationStartedAtMs === null ? (
          <DialogFooter className="justify-end">
            <DialogClose asChild>
              <Button disabled={isSubmitting} variant="secondary">
                {t('storyResources.actions.cancel')}
              </Button>
            </DialogClose>
            {!bindingsUnavailable ? (
              <Button
                disabled={
                  isSubmitting || apiGroupId.trim().length === 0 || presetId.trim().length === 0
                }
                onClick={() => {
                  void handleSubmit()
                }}
              >
                {t('storyResources.actions.generate')}
              </Button>
            ) : null}
          </DialogFooter>
        ) : (
          <DialogFooter className="justify-end">
            <Button disabled variant="secondary">
              {t('storyResources.actions.generating')}
            </Button>
          </DialogFooter>
        )}
      </DialogContent>
    </Dialog>
  )
}
