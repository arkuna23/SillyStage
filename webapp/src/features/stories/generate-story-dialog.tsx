import { useEffect, useId, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { appPaths } from '../../app/paths'
import type { ApiGroup, Preset } from '../apis/types'
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
import { GenerationLoadingStage } from '../../components/ui/generation-loading-stage'
import { DialogRouteButton } from '../../components/ui/dialog-route-button'
import { Input } from '../../components/ui/input'
import { Select } from '../../components/ui/select'
import { useToastMessage } from '../../components/ui/toast-context'
import {
  continueStoryDraft,
  finalizeStoryDraft,
  startStoryDraft,
} from './api'
import type { StoryDetail, StoryDraftDetail } from './types'
import type { StoryResource } from '../story-resources/types'

type GenerateStoryDialogProps = {
  apiGroups: ReadonlyArray<ApiGroup>
  onCompleted: (result: { message: string; story: StoryDetail }) => Promise<void> | void
  onDraftsChanged?: () => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
  presets: ReadonlyArray<Preset>
  resources: ReadonlyArray<StoryResource>
}

type FormState = {
  apiGroupId: string
  displayName: string
  presetId: string
  resourceId: string
}

type DialogStage = 'form' | 'generating'

function createInitialFormState(): FormState {
  return {
    apiGroupId: '',
    displayName: '',
    presetId: '',
    resourceId: '',
  }
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function summarizeStoryInput(resource: StoryResource) {
  return (resource.planned_story?.trim() || resource.story_concept).replace(/\s+/g, ' ').trim()
}

function getDraftProgressMessage(
  labels: {
    draftProgress: (values: { current: number; total: number }) => string
    draftProgressUnknown: string
  },
  draft: Pick<StoryDraftDetail, 'next_section_index' | 'total_sections'>,
) {
  const nextSectionIndex =
    typeof draft.next_section_index === 'number' && Number.isFinite(draft.next_section_index)
      ? draft.next_section_index
      : null
  const totalSections =
    typeof draft.total_sections === 'number' && Number.isFinite(draft.total_sections) && draft.total_sections > 0
      ? draft.total_sections
      : null

  if (nextSectionIndex === null || totalSections === null) {
    return labels.draftProgressUnknown
  }

  return labels.draftProgress({
    current: Math.min(nextSectionIndex + 1, totalSections),
    total: totalSections,
  })
}

export function GenerateStoryDialog({
  apiGroups,
  onCompleted,
  onDraftsChanged,
  onOpenChange,
  open,
  presets,
  resources,
}: GenerateStoryDialogProps) {
  const { t } = useTranslation()
  const draftProgressLabels = {
    draftProgress: (values: { current: number; total: number }) =>
      String(t('stories.generating.draftProgress', values)),
    draftProgressUnknown: String(t('stories.generating.draftProgressUnknown')),
  }
  const fieldIdPrefix = useId()
  const [formState, setFormState] = useState<FormState>(createInitialFormState)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  const [dialogStage, setDialogStage] = useState<DialogStage>('form')
  const [generationStartedAtMs, setGenerationStartedAtMs] = useState<number | null>(null)
  const [draftIdentifier, setDraftIdentifier] = useState<string | null>(null)
  const [generatingMessage, setGeneratingMessage] = useState<string | null>(null)
  useToastMessage(submitError)

  const fieldIds = {
    apiGroupId: `${fieldIdPrefix}-api-group-id`,
    displayName: `${fieldIdPrefix}-display-name`,
    presetId: `${fieldIdPrefix}-preset-id`,
    resourceId: `${fieldIdPrefix}-resource-id`,
  } as const

  const resourceOptions = useMemo(
    () =>
      resources.map((resource) => ({
        label: resource.resource_id,
        value: resource.resource_id,
      })),
    [resources],
  )

  const selectedResource = useMemo(
    () => resources.find((resource) => resource.resource_id === formState.resourceId) ?? null,
    [formState.resourceId, resources],
  )
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
  const missingApiGroups = apiGroups.length === 0
  const missingPresets = presets.length === 0
  const canGenerate = resources.length > 0 && !missingApiGroups && !missingPresets

  useEffect(() => {
    if (!open) {
      setFormState(createInitialFormState())
      setIsSubmitting(false)
      setSubmitError(null)
      setDialogStage('form')
      setGenerationStartedAtMs(null)
      setDraftIdentifier(null)
      setGeneratingMessage(null)
      return
    }

    setFormState((currentFormState) => ({
      ...currentFormState,
      apiGroupId: '',
      presetId: '',
      resourceId:
        currentFormState.resourceId || resources[0]?.resource_id || '',
    }))
    setIsSubmitting(false)
    setSubmitError(null)
    setDialogStage('form')
    setGenerationStartedAtMs(null)
    setDraftIdentifier(null)
    setGeneratingMessage(null)
  }, [open, resources])

  async function runDraftGeneration(initialDraft: StoryDraftDetail) {
    let draft = initialDraft

    setDraftIdentifier(initialDraft.draft_id)
    setGeneratingMessage(getDraftProgressMessage(draftProgressLabels, initialDraft))

    while (draft.status === 'building') {
      draft = await continueStoryDraft({ draft_id: draft.draft_id })

      setDraftIdentifier(draft.draft_id)
      setGeneratingMessage(getDraftProgressMessage(draftProgressLabels, draft))
    }

    setGeneratingMessage(t('stories.generating.finalizing'))

    return finalizeStoryDraft({ draft_id: draft.draft_id })
  }

  async function handleSubmit() {
    if (formState.resourceId.trim().length === 0) {
      setSubmitError(t('stories.form.errors.resourceRequired'))
      return
    }

    if (formState.apiGroupId.trim().length === 0) {
      setSubmitError(t('stories.form.errors.apiGroupRequired'))
      return
    }

    if (formState.presetId.trim().length === 0) {
      setSubmitError(t('stories.form.errors.presetRequired'))
      return
    }

    let createdDraftId: string | null = null
    setSubmitError(null)
    setIsSubmitting(true)
    setDialogStage('generating')
    setGenerationStartedAtMs(Date.now())
    setDraftIdentifier(null)
    setGeneratingMessage(t('stories.generating.starting'))

    try {
      const initialDraft = await startStoryDraft({
        api_group_id: formState.apiGroupId.trim(),
        ...(formState.displayName.trim()
          ? { display_name: formState.displayName.trim() }
          : {}),
        preset_id: formState.presetId.trim(),
        resource_id: formState.resourceId.trim(),
      })
      createdDraftId = initialDraft.draft_id
      const result = await runDraftGeneration(initialDraft)

      await onCompleted({
        message: t('stories.feedback.created', { name: result.display_name }),
        story: {
          ...result,
          type: 'story',
        },
      })

      onOpenChange(false)
    } catch (error) {
      if (createdDraftId !== null) {
        await onDraftsChanged?.()
      }
      setDialogStage('form')
      setGenerationStartedAtMs(null)
      setDraftIdentifier(null)
      setGeneratingMessage(null)
      setSubmitError(
        getErrorMessage(
          error,
          createdDraftId !== null
            ? t('stories.form.errors.draftSubmitFailed')
            : t('stories.form.errors.submitFailed'),
        ),
      )
    } finally {
      setIsSubmitting(false)
    }
  }

  const generatingDescription = selectedResource
    ? t('stories.generating.descriptionWithResource', { id: selectedResource.resource_id })
    : t('stories.generating.description')

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
          <DialogTitle>{t('stories.form.createTitle')}</DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          {dialogStage === 'generating' && generationStartedAtMs !== null ? (
            <GenerationLoadingStage
              description={generatingMessage ?? generatingDescription}
              elapsedLabel={t('stories.generating.elapsed')}
              identifier={(draftIdentifier ?? formState.resourceId) || null}
              startedAtMs={generationStartedAtMs}
              statusLabel={t('stories.generating.badge')}
              title={t('stories.generating.title')}
            />
          ) : resources.length === 0 ? (
            <div className="space-y-5">
              <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
                {t('stories.form.emptyResources')}
              </div>

              <div className="flex justify-end">
                <DialogRouteButton
                  onRequestClose={() => {
                    onOpenChange(false)
                  }}
                  to={appPaths.storyResources}
                  variant="secondary"
                >
                  {t('stories.form.openResources')}
                </DialogRouteButton>
              </div>
            </div>
          ) : missingApiGroups || missingPresets ? (
            <div className="space-y-5">
              <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
                {missingApiGroups
                  ? t('stories.form.emptyApiGroups')
                  : t('stories.form.emptyPresets')}
              </div>

              <div className="flex justify-end">
                <DialogRouteButton
                  onRequestClose={() => {
                    onOpenChange(false)
                  }}
                  to={missingApiGroups ? appPaths.apis : appPaths.presets}
                  variant="secondary"
                >
                  {missingApiGroups
                    ? t('stories.form.openApiGroups')
                    : t('stories.form.openPresets')}
                </DialogRouteButton>
              </div>
            </div>
          ) : (
            <>
              <div className="space-y-2.5">
                <label
                  className="block text-sm font-medium text-[var(--color-text-primary)]"
                  htmlFor={fieldIds.resourceId}
                >
                  {t('stories.form.fields.resourceId')}
                </label>
                <Select
                  items={resourceOptions}
                  textAlign="start"
                  triggerId={fieldIds.resourceId}
                  value={formState.resourceId}
                  onValueChange={(resourceId) => {
                    setFormState((currentFormState) => ({
                      ...currentFormState,
                      resourceId,
                    }))
                  }}
                />
              </div>

              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2.5">
                  <label
                    className="block text-sm font-medium text-[var(--color-text-primary)]"
                    htmlFor={fieldIds.apiGroupId}
                  >
                    {t('stories.form.fields.apiGroupId')}
                  </label>
                  <Select
                    items={apiGroupOptions}
                    onValueChange={(apiGroupId) => {
                      setFormState((currentFormState) => ({
                        ...currentFormState,
                        apiGroupId,
                      }))
                    }}
                    placeholder={t('stories.form.placeholders.apiGroupId')}
                    textAlign="start"
                    triggerId={fieldIds.apiGroupId}
                    value={formState.apiGroupId || undefined}
                  />
                </div>

                <div className="space-y-2.5">
                  <label
                    className="block text-sm font-medium text-[var(--color-text-primary)]"
                    htmlFor={fieldIds.presetId}
                  >
                    {t('stories.form.fields.presetId')}
                  </label>
                  <Select
                    items={presetOptions}
                    onValueChange={(presetId) => {
                      setFormState((currentFormState) => ({
                        ...currentFormState,
                        presetId,
                      }))
                    }}
                    placeholder={t('stories.form.placeholders.presetId')}
                    textAlign="start"
                    triggerId={fieldIds.presetId}
                    value={formState.presetId || undefined}
                  />
                </div>
              </div>

              {selectedResource ? (
                <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                  <p className="text-xs text-[var(--color-text-muted)]">
                    {t('stories.form.fields.inputPreview')}
                  </p>
                  <p className="mt-2 text-sm leading-7 text-[var(--color-text-primary)]">
                    {summarizeStoryInput(selectedResource)}
                  </p>
                </div>
              ) : null}

              <div className="space-y-2.5">
                <label
                  className="block text-sm font-medium text-[var(--color-text-primary)]"
                  htmlFor={fieldIds.displayName}
                >
                  {t('stories.form.fields.displayName')}
                </label>
                <Input
                  id={fieldIds.displayName}
                  placeholder={t('stories.form.placeholders.displayName')}
                  value={formState.displayName}
                  onChange={(event) => {
                    const { value } = event.target

                    setFormState((currentFormState) => ({
                      ...currentFormState,
                      displayName: value,
                    }))
                  }}
                />
              </div>

            </>
          )}
        </DialogBody>

        {dialogStage === 'form' ? (
          <DialogFooter className="justify-end">
            <DialogClose asChild>
              <Button disabled={isSubmitting} variant="secondary">
                {t('stories.actions.cancel')}
              </Button>
            </DialogClose>
            {canGenerate ? (
              <Button disabled={isSubmitting} onClick={() => void handleSubmit()}>
                {isSubmitting ? t('stories.actions.creating') : t('stories.actions.create')}
              </Button>
            ) : null}
          </DialogFooter>
        ) : (
          <DialogFooter className="justify-end">
            <Button disabled variant="secondary">
              {t('stories.actions.creating')}
            </Button>
          </DialogFooter>
        )}
      </DialogContent>
    </Dialog>
  )
}
