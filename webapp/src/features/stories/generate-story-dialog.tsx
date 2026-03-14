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
import { GenerationLoadingStage } from '../../components/ui/generation-loading-stage'
import { DialogRouteButton } from '../../components/ui/dialog-route-button'
import { Input } from '../../components/ui/input'
import { Select } from '../../components/ui/select'
import {
  continueStoryDraft,
  finalizeStoryDraft,
  startStoryDraft,
} from './api'
import type { StoryDetail, StoryDraftDetail } from './types'
import type { StoryResource } from '../story-resources/types'

type GenerateStoryDialogProps = {
  onCompleted: (result: { message: string; story: StoryDetail }) => Promise<void> | void
  onDraftsChanged?: () => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
  resources: ReadonlyArray<StoryResource>
}

type FormState = {
  displayName: string
  resourceId: string
}

type DialogStage = 'form' | 'generating'

function createInitialFormState(): FormState {
  return {
    displayName: '',
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
  onCompleted,
  onDraftsChanged,
  onOpenChange,
  open,
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

  const fieldIds = {
    displayName: `${fieldIdPrefix}-display-name`,
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

    let createdDraftId: string | null = null
    setSubmitError(null)
    setIsSubmitting(true)
    setDialogStage('generating')
    setGenerationStartedAtMs(Date.now())
    setDraftIdentifier(null)
    setGeneratingMessage(t('stories.generating.starting'))

    try {
      const initialDraft = await startStoryDraft({
        ...(formState.displayName.trim()
          ? { display_name: formState.displayName.trim() }
          : {}),
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

              {submitError ? (
                <div className="rounded-[1.25rem] border border-[var(--color-state-error-line)] bg-[var(--color-state-error-soft)] px-4 py-3 text-sm text-[var(--color-text-primary)]">
                  {submitError}
                </div>
              ) : null}
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
            {resources.length > 0 ? (
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
