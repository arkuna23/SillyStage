import { AnimatePresence, motion } from 'framer-motion'
import { type ReactNode, useEffect, useId, useMemo, useState } from 'react'
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
import { Input } from '../../components/ui/input'
import { Select } from '../../components/ui/select'
import { useToastMessage } from '../../components/ui/toast-context'
import { cn } from '../../lib/cn'
import type { ApiGroup, Preset } from '../apis/types'
import type { CharacterSummary } from '../characters/types'
import { getStoryResourceOptionLabel } from '../story-resources/story-resource-display'
import type { StoryResource } from '../story-resources/types'
import { continueStoryDraft, finalizeStoryDraft, startStoryDraft } from './api'
import { getDraftSectionProgress } from './draft-progress'
import {
  createStoryCommonVariableDrafts,
  type StoryCommonVariableDraft,
  type StoryCommonVariableDraftErrors,
  serializeStoryCommonVariableDrafts,
  validateStoryCommonVariableDrafts,
} from './story-common-variable-drafts'
import { useStoryCommonVariableSchemaCatalog } from './story-common-variable-schema-catalog'
import { StoryCommonVariablesEditor } from './story-common-variables-editor'
import type { StoryDetail, StoryDraftDetail } from './types'

type GenerateStoryDialogProps = {
  availableCharacters: ReadonlyArray<CharacterSummary>
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
  commonVariables: StoryCommonVariableDraft[]
  displayName: string
  presetId: string
  resourceId: string
}

type DialogStage = 'basic' | 'generating' | 'variables'

function createInitialFormState(): FormState {
  return {
    apiGroupId: '',
    commonVariables: createStoryCommonVariableDrafts([]),
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

function Field({
  children,
  htmlFor,
  label,
}: {
  children: ReactNode
  htmlFor?: string
  label: string
}) {
  return (
    <div className="space-y-2.5">
      {htmlFor ? (
        <label
          className="block text-sm font-medium text-[var(--color-text-primary)]"
          htmlFor={htmlFor}
        >
          {label}
        </label>
      ) : (
        <span className="block text-sm font-medium text-[var(--color-text-primary)]">{label}</span>
      )}
      {children}
    </div>
  )
}

function StepChip({ active, index, label }: { active: boolean; index: number; label: string }) {
  return (
    <div
      className={cn(
        'flex min-w-0 items-center gap-2 rounded-full border px-3 py-2 transition',
        active
          ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)]'
          : 'border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_72%,transparent)] text-[var(--color-text-muted)]',
      )}
    >
      <span
        className={cn(
          'inline-flex size-6 items-center justify-center rounded-full border text-[0.72rem] font-medium',
          active
            ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold)] text-[var(--color-accent-ink)]'
            : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] text-[var(--color-text-muted)]',
        )}
      >
        {index}
      </span>
      <span className="truncate text-sm font-medium">{label}</span>
    </div>
  )
}

export function GenerateStoryDialog({
  availableCharacters,
  apiGroups,
  onCompleted,
  onDraftsChanged,
  onOpenChange,
  open,
  presets,
  resources,
}: GenerateStoryDialogProps) {
  const { t } = useTranslation()
  const fieldIdPrefix = useId()
  const [formState, setFormState] = useState<FormState>(createInitialFormState)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  const [dialogStage, setDialogStage] = useState<DialogStage>('basic')
  const [generationStartedAtMs, setGenerationStartedAtMs] = useState<number | null>(null)
  const [draftIdentifier, setDraftIdentifier] = useState<string | null>(null)
  const [generatingDescription, setGeneratingDescription] = useState<string | null>(null)
  const [generatingProgressText, setGeneratingProgressText] = useState<string | null>(null)
  const [commonVariableErrors, setCommonVariableErrors] = useState<StoryCommonVariableDraftErrors>(
    {},
  )
  useToastMessage(submitError)

  const fieldIds = {
    apiGroupId: `${fieldIdPrefix}-api-group-id`,
    displayName: `${fieldIdPrefix}-display-name`,
    presetId: `${fieldIdPrefix}-preset-id`,
    resourceId: `${fieldIdPrefix}-resource-id`,
  } as const

  const selectedResource = useMemo(
    () => resources.find((resource) => resource.resource_id === formState.resourceId) ?? null,
    [formState.resourceId, resources],
  )
  const resourceCharacterIds = useMemo(
    () => selectedResource?.character_ids ?? [],
    [selectedResource],
  )
  const commonVariableCharacterIds = useMemo(() => {
    const knownCharacterIds = new Set(resourceCharacterIds)

    formState.commonVariables.forEach((draft) => {
      if (draft.scope !== 'character') {
        return
      }

      const characterId = draft.character_id.trim()

      if (characterId.length > 0) {
        knownCharacterIds.add(characterId)
      }
    })

    return Array.from(knownCharacterIds)
  }, [formState.commonVariables, resourceCharacterIds])
  const commonVariableSchemaCatalog = useStoryCommonVariableSchemaCatalog({
    characterIds: commonVariableCharacterIds,
    enabled: open && Boolean(selectedResource),
    playerSchemaId: selectedResource?.player_schema_id_seed,
    worldSchemaId: selectedResource?.world_schema_id_seed,
  })
  const resourceOptions = useMemo(
    () =>
      resources.map((resource) => ({
        label: getStoryResourceOptionLabel(resource),
        value: resource.resource_id,
      })),
    [resources],
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
  const wizardStepLabels = [
    t('stories.createWizard.steps.basic'),
    t('stories.createWizard.steps.variables'),
  ]
  const wizardStepIndex = dialogStage === 'basic' ? 0 : 1

  useEffect(() => {
    if (!open) {
      setFormState(createInitialFormState())
      setIsSubmitting(false)
      setSubmitError(null)
      setDialogStage('basic')
      setGenerationStartedAtMs(null)
      setDraftIdentifier(null)
      setGeneratingDescription(null)
      setGeneratingProgressText(null)
      setCommonVariableErrors({})
      return
    }

    setFormState({
      ...createInitialFormState(),
      resourceId: resources[0]?.resource_id ?? '',
    })
    setIsSubmitting(false)
    setSubmitError(null)
    setDialogStage('basic')
    setGenerationStartedAtMs(null)
    setDraftIdentifier(null)
    setGeneratingDescription(null)
    setGeneratingProgressText(null)
    setCommonVariableErrors({})
  }, [open, resources])

  function validateBasicStep() {
    if (formState.resourceId.trim().length === 0) {
      return t('stories.form.errors.resourceRequired')
    }

    if (formState.displayName.trim().length === 0) {
      return t('stories.form.errors.displayNameRequired')
    }

    if (formState.apiGroupId.trim().length === 0) {
      return t('stories.form.errors.apiGroupRequired')
    }

    if (formState.presetId.trim().length === 0) {
      return t('stories.form.errors.presetRequired')
    }

    return null
  }

  function validateVariablesStep() {
    const nextCommonVariableErrors = validateStoryCommonVariableDrafts(
      formState.commonVariables,
      new Set(resourceCharacterIds),
    )

    setCommonVariableErrors(nextCommonVariableErrors)

    if (Object.keys(nextCommonVariableErrors).length > 0) {
      return t('stories.form.errors.commonVariablesInvalid')
    }

    return null
  }

  function goNext() {
    const validationError = validateBasicStep()

    if (validationError) {
      setSubmitError(validationError)
      return
    }

    setSubmitError(null)
    setDialogStage('variables')
  }

  function goBack() {
    setSubmitError(null)
    setDialogStage('basic')
  }

  async function runDraftGeneration(initialDraft: StoryDraftDetail) {
    let draft = initialDraft
    let progress = getDraftSectionProgress(initialDraft)

    setDraftIdentifier(initialDraft.draft_id)
    setGeneratingDescription(
      selectedResource
        ? t('stories.generating.descriptionWithResource', { id: selectedResource.resource_id })
        : t('stories.generating.description'),
    )
    setGeneratingProgressText(progress ? t('stories.generating.progressValue', progress) : null)

    while (draft.status === 'building') {
      draft = await continueStoryDraft({ draft_id: draft.draft_id })
      progress = getDraftSectionProgress(draft)

      setDraftIdentifier(draft.draft_id)
      setGeneratingDescription(
        selectedResource
          ? t('stories.generating.descriptionWithResource', { id: selectedResource.resource_id })
          : t('stories.generating.description'),
      )
      setGeneratingProgressText(progress ? t('stories.generating.progressValue', progress) : null)
    }

    progress = getDraftSectionProgress(draft)
    setGeneratingDescription(t('stories.generating.finalizing'))
    setGeneratingProgressText(progress ? t('stories.generating.progressValue', progress) : null)

    return finalizeStoryDraft({ draft_id: draft.draft_id })
  }

  async function handleSubmit() {
    const basicStepError = validateBasicStep()

    if (basicStepError) {
      setDialogStage('basic')
      setSubmitError(basicStepError)
      return
    }

    const variablesStepError = validateVariablesStep()

    if (variablesStepError) {
      setDialogStage('variables')
      setSubmitError(variablesStepError)
      return
    }

    let createdDraftId: string | null = null
    const displayName = formState.displayName.trim()
    setSubmitError(null)
    setIsSubmitting(true)
    setDialogStage('generating')
    setGenerationStartedAtMs(Date.now())
    setDraftIdentifier(null)
    setGeneratingDescription(t('stories.generating.starting'))
    setGeneratingProgressText(null)

    try {
      const initialDraft = await startStoryDraft({
        api_group_id: formState.apiGroupId.trim(),
        common_variables: serializeStoryCommonVariableDrafts(formState.commonVariables),
        display_name: displayName,
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
      setDialogStage('variables')
      setGenerationStartedAtMs(null)
      setDraftIdentifier(null)
      setGeneratingDescription(null)
      setGeneratingProgressText(null)
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

  const defaultGeneratingDescription = selectedResource
    ? t('stories.generating.descriptionWithResource', { id: selectedResource.resource_id })
    : t('stories.generating.description')

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent
        aria-describedby={undefined}
        className="w-[min(96vw,60rem)] overflow-hidden"
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
          <DialogTitle>{t('stories.createWizard.title')}</DialogTitle>
          {canGenerate && dialogStage !== 'generating' ? (
            <div className="grid gap-2 pt-2 md:grid-cols-2">
              {wizardStepLabels.map((label, index) => (
                <StepChip
                  active={index <= wizardStepIndex}
                  index={index + 1}
                  key={label}
                  label={label}
                />
              ))}
            </div>
          ) : null}
        </DialogHeader>

        <DialogBody className="max-h-[calc(92vh-12rem)] overflow-y-auto pt-6">
          {dialogStage === 'generating' && generationStartedAtMs !== null ? (
            <GenerationLoadingStage
              description={generatingDescription ?? defaultGeneratingDescription}
              elapsedLabel={t('stories.generating.elapsed')}
              identifier={(draftIdentifier ?? formState.resourceId) || null}
              progressLabel={t('stories.generating.progressLabel')}
              progressText={generatingProgressText}
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
            <AnimatePresence initial={false} mode="wait">
              {dialogStage === 'basic' ? (
                <motion.div
                  animate={{ opacity: 1, x: 0 }}
                  className="space-y-5"
                  initial={{ opacity: 0, x: 18 }}
                  key="basic"
                  transition={{ duration: 0.22, ease: [0.22, 1, 0.36, 1] }}
                >
                  <div className="space-y-2">
                    <h3 className="font-display text-[1.85rem] text-[var(--color-text-primary)]">
                      {t('stories.createWizard.headings.basic')}
                    </h3>
                    <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                      {t('stories.createWizard.descriptions.basic')}
                    </p>
                  </div>

                  <Field htmlFor={fieldIds.resourceId} label={t('stories.form.fields.resourceId')}>
                    <Select
                      items={resourceOptions}
                      textAlign="start"
                      triggerId={fieldIds.resourceId}
                      value={formState.resourceId}
                      onValueChange={(resourceId) => {
                        setCommonVariableErrors({})
                        setFormState((currentFormState) => ({
                          ...currentFormState,
                          resourceId,
                        }))
                      }}
                    />
                  </Field>

                  <div className="grid gap-4 md:grid-cols-2">
                    <Field
                      htmlFor={fieldIds.apiGroupId}
                      label={t('stories.form.fields.apiGroupId')}
                    >
                      <Select
                        items={apiGroupOptions}
                        placeholder={t('stories.form.placeholders.apiGroupId')}
                        textAlign="start"
                        triggerId={fieldIds.apiGroupId}
                        value={formState.apiGroupId || undefined}
                        onValueChange={(apiGroupId) => {
                          setFormState((currentFormState) => ({
                            ...currentFormState,
                            apiGroupId,
                          }))
                        }}
                      />
                    </Field>

                    <Field htmlFor={fieldIds.presetId} label={t('stories.form.fields.presetId')}>
                      <Select
                        items={presetOptions}
                        placeholder={t('stories.form.placeholders.presetId')}
                        textAlign="start"
                        triggerId={fieldIds.presetId}
                        value={formState.presetId || undefined}
                        onValueChange={(presetId) => {
                          setFormState((currentFormState) => ({
                            ...currentFormState,
                            presetId,
                          }))
                        }}
                      />
                    </Field>
                  </div>

                  {selectedResource ? (
                    <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                      <p className="text-xs text-[var(--color-text-muted)]">
                        {t('stories.form.fields.resourceId')}
                      </p>
                      <p className="mt-2 text-sm leading-7 text-[var(--color-text-primary)]">
                        {getStoryResourceOptionLabel(selectedResource)}
                      </p>
                    </div>
                  ) : null}

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

                  <Field
                    htmlFor={fieldIds.displayName}
                    label={t('stories.form.fields.displayName')}
                  >
                    <Input
                      id={fieldIds.displayName}
                      name={fieldIds.displayName}
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
                  </Field>
                </motion.div>
              ) : (
                <motion.div
                  animate={{ opacity: 1, x: 0 }}
                  className="space-y-5"
                  initial={{ opacity: 0, x: 18 }}
                  key="variables"
                  transition={{ duration: 0.22, ease: [0.22, 1, 0.36, 1] }}
                >
                  <div className="space-y-2">
                    <h3 className="font-display text-[1.85rem] text-[var(--color-text-primary)]">
                      {t('stories.createWizard.headings.variables')}
                    </h3>
                    <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                      {t('stories.createWizard.descriptions.variables')}
                    </p>
                  </div>

                  {selectedResource ? (
                    <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                      <p className="text-xs text-[var(--color-text-muted)]">
                        {t('stories.form.fields.resourceId')}
                      </p>
                      <p className="mt-2 text-sm leading-7 text-[var(--color-text-primary)]">
                        {getStoryResourceOptionLabel(selectedResource)}
                      </p>
                    </div>
                  ) : null}

                  <StoryCommonVariablesEditor
                    characters={availableCharacters}
                    disabled={isSubmitting}
                    drafts={formState.commonVariables}
                    errors={commonVariableErrors}
                    resourceCharacterIds={resourceCharacterIds}
                    schemaCatalog={commonVariableSchemaCatalog}
                    onChange={(commonVariables) => {
                      setCommonVariableErrors({})
                      setFormState((currentFormState) => ({
                        ...currentFormState,
                        commonVariables,
                      }))
                    }}
                  />
                </motion.div>
              )}
            </AnimatePresence>
          )}
        </DialogBody>

        {dialogStage === 'generating' ? (
          <DialogFooter className="justify-end">
            <Button disabled variant="secondary">
              {t('stories.actions.creating')}
            </Button>
          </DialogFooter>
        ) : (
          <DialogFooter className="justify-between">
            <div>
              {dialogStage === 'variables' ? (
                <Button disabled={isSubmitting} onClick={goBack} variant="secondary">
                  {t('stories.actions.back')}
                </Button>
              ) : null}
            </div>
            <div className="flex flex-wrap items-center justify-end gap-3">
              <DialogClose asChild>
                <Button disabled={isSubmitting} variant="secondary">
                  {t('stories.actions.cancel')}
                </Button>
              </DialogClose>
              {canGenerate ? (
                dialogStage === 'basic' ? (
                  <Button disabled={isSubmitting} onClick={goNext}>
                    {t('stories.actions.next')}
                  </Button>
                ) : (
                  <Button disabled={isSubmitting} onClick={() => void handleSubmit()}>
                    {isSubmitting ? t('stories.actions.creating') : t('stories.actions.create')}
                  </Button>
                )
              ) : null}
            </div>
          </DialogFooter>
        )}
      </DialogContent>
    </Dialog>
  )
}
