import { AnimatePresence, motion } from 'framer-motion'
import type { ReactNode } from 'react'
import { useId, useMemo, useState } from 'react'
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
import { Textarea } from '../../components/ui/textarea'
import { useToastMessage } from '../../components/ui/toast-context'
import { cn } from '../../lib/cn'
import type { ApiGroup, Preset } from '../apis/types'
import type { CharacterSummary } from '../characters/types'
import type { Lorebook } from '../lorebooks/types'
import type { SchemaResource } from '../schemas/types'
import { createStoryResource, generateAndSaveStoryPlan } from './api'
import { StoryInputFlowCard } from './story-input-flow-card'
import { StoryResourceCharacterSelector } from './story-resource-character-selector'
import { StoryResourceLorebookSelector } from './story-resource-lorebook-selector'
import type { StoryResource } from './types'

type NoticeTone = 'error' | 'success' | 'warning'
type CreateWizardStep = 'concept' | 'characters' | 'seeds' | 'planner' | 'generating'

type CreateStoryResourceDialogProps = {
  availableCharacters: ReadonlyArray<CharacterSummary>
  availableApiGroups: ReadonlyArray<ApiGroup>
  availableLorebooks: ReadonlyArray<Lorebook>
  availablePresets: ReadonlyArray<Preset>
  availableSchemas: ReadonlyArray<SchemaResource>
  onCompleted: (result: {
    message: string
    resource: StoryResource
    tone: NoticeTone
  }) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
  referencesLoading: boolean
}

type FormState = {
  apiGroupId: string
  characterIds: string[]
  lorebookIds: string[]
  presetId: string
  playerSchemaIdSeed: string
  shouldGenerate: boolean
  storyConcept: string
  worldSchemaIdSeed: string
}

function createInitialFormState(): FormState {
  return {
    apiGroupId: '',
    characterIds: [],
    lorebookIds: [],
    presetId: '',
    playerSchemaIdSeed: '',
    shouldGenerate: true,
    storyConcept: '',
    worldSchemaIdSeed: '',
  }
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function Field({
  children,
  description,
  htmlFor,
  label,
}: {
  children: ReactNode
  description?: string
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
      {description ? (
        <p className="text-xs leading-6 text-[var(--color-text-muted)]">{description}</p>
      ) : null}
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

export function CreateStoryResourceDialog({
  availableCharacters,
  availableApiGroups,
  availableLorebooks,
  availablePresets,
  availableSchemas,
  onCompleted,
  onOpenChange,
  open,
  referencesLoading,
}: CreateStoryResourceDialogProps) {
  const { t } = useTranslation()
  const fieldIdPrefix = useId()
  const [activeStep, setActiveStep] = useState<CreateWizardStep>('concept')
  const [formState, setFormState] = useState<FormState>(createInitialFormState)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  useToastMessage(submitError)
  const [generatingPhase, setGeneratingPhase] = useState<'creating' | 'planning'>('creating')
  const [generatedResourceId, setGeneratedResourceId] = useState<string | null>(null)
  const [generationStartedAtMs, setGenerationStartedAtMs] = useState<number | null>(null)

  const fieldIds = {
    apiGroupId: `${fieldIdPrefix}-api-group-id`,
    presetId: `${fieldIdPrefix}-preset-id`,
    playerSchemaIdSeed: `${fieldIdPrefix}-player-schema-seed`,
    storyConcept: `${fieldIdPrefix}-story-concept`,
    worldSchemaIdSeed: `${fieldIdPrefix}-world-schema-seed`,
  } as const

  const schemaOptions = useMemo(
    () =>
      availableSchemas.map((schema) => ({
        label: schema.display_name,
        value: schema.schema_id,
      })),
    [availableSchemas],
  )
  const apiGroupOptions = useMemo(
    () =>
      availableApiGroups.map((apiGroup) => ({
        label: apiGroup.display_name,
        value: apiGroup.api_group_id,
      })),
    [availableApiGroups],
  )
  const presetOptions = useMemo(
    () =>
      availablePresets.map((preset) => ({
        label: preset.display_name,
        value: preset.preset_id,
      })),
    [availablePresets],
  )
  const lorebookOptions = useMemo(
    () =>
      availableLorebooks.map((lorebook) => ({
        display_name: lorebook.display_name,
        lorebook_id: lorebook.lorebook_id,
      })),
    [availableLorebooks],
  )

  function resetDialogState() {
    setActiveStep('concept')
    setFormState(createInitialFormState())
    setIsSubmitting(false)
    setSubmitError(null)
    setGeneratingPhase('creating')
    setGeneratedResourceId(null)
    setGenerationStartedAtMs(null)
  }

  function handleOpenChange(nextOpen: boolean) {
    if (!nextOpen) {
      resetDialogState()
    }

    onOpenChange(nextOpen)
  }

  function toggleLorebook(lorebookId: string) {
    setFormState((currentFormState) => {
      const isSelected = currentFormState.lorebookIds.includes(lorebookId)

      return {
        ...currentFormState,
        lorebookIds: isSelected
          ? currentFormState.lorebookIds.filter((id) => id !== lorebookId)
          : [...currentFormState.lorebookIds, lorebookId],
      }
    })
  }

  function validateConceptStep() {
    if (formState.storyConcept.trim().length === 0) {
      return t('storyResources.form.errors.storyConceptRequired')
    }

    return null
  }

  function validateCharactersStep() {
    if (formState.characterIds.length === 0) {
      return t('storyResources.form.errors.charactersRequired')
    }

    return null
  }

  function goNext() {
    const validationError =
      activeStep === 'concept'
        ? validateConceptStep()
        : activeStep === 'characters'
          ? validateCharactersStep()
          : null

    if (validationError) {
      setSubmitError(validationError)
      return
    }

    setSubmitError(null)
    setActiveStep((currentStep) => {
      if (currentStep === 'concept') {
        return 'characters'
      }

      if (currentStep === 'characters') {
        return 'seeds'
      }

      if (currentStep === 'seeds') {
        return 'planner'
      }

      return currentStep
    })
  }

  function goBack() {
    setSubmitError(null)
    setActiveStep((currentStep) => {
      if (currentStep === 'planner') {
        return 'seeds'
      }

      if (currentStep === 'seeds') {
        return 'characters'
      }

      if (currentStep === 'characters') {
        return 'concept'
      }

      return currentStep
    })
  }

  async function handleCreate() {
    const conceptValidationError = validateConceptStep()
    const charactersValidationError = validateCharactersStep()

    setSubmitError(null)

    if (conceptValidationError) {
      setActiveStep('concept')
      setSubmitError(conceptValidationError)
      return
    }

    if (charactersValidationError) {
      setActiveStep('characters')
      setSubmitError(charactersValidationError)
      return
    }

    if (formState.shouldGenerate && formState.apiGroupId.trim().length === 0) {
      setActiveStep('planner')
      setSubmitError(t('storyResources.form.errors.apiGroupRequired'))
      return
    }

    if (formState.shouldGenerate && formState.presetId.trim().length === 0) {
      setActiveStep('planner')
      setSubmitError(t('storyResources.form.errors.presetRequired'))
      return
    }

    setIsSubmitting(true)

    if (formState.shouldGenerate) {
      setGeneratingPhase('creating')
      setGeneratedResourceId(null)
      setGenerationStartedAtMs(Date.now())
      setActiveStep('generating')
    }

    try {
      const savedResource = await createStoryResource({
        character_ids: [...new Set(formState.characterIds)],
        lorebook_ids: [...new Set(formState.lorebookIds)],
        ...(formState.playerSchemaIdSeed
          ? { player_schema_id_seed: formState.playerSchemaIdSeed.trim() }
          : {}),
        story_concept: formState.storyConcept.trim(),
        ...(formState.worldSchemaIdSeed
          ? { world_schema_id_seed: formState.worldSchemaIdSeed.trim() }
          : {}),
      })

      setGeneratedResourceId(savedResource.resource_id)

      if (!formState.shouldGenerate) {
        await onCompleted({
          message: t('storyResources.feedback.created', { id: savedResource.resource_id }),
          resource: savedResource,
          tone: 'success',
        })
        handleOpenChange(false)
        return
      }

      setGeneratingPhase('planning')

      try {
        const generated = await generateAndSaveStoryPlan({
          apiGroupId: formState.apiGroupId,
          presetId: formState.presetId,
          resourceId: savedResource.resource_id,
        })
        await onCompleted({
          message: t('storyResources.feedback.generated', { id: savedResource.resource_id }),
          resource: generated.resource,
          tone: 'success',
        })
      } catch (error) {
        await onCompleted({
          message: getErrorMessage(
            error,
            t('storyResources.feedback.savedButGenerateFailed', {
              id: savedResource.resource_id,
            }),
          ),
          resource: savedResource,
          tone: 'warning',
        })
      }

      handleOpenChange(false)
    } catch (error) {
      setIsSubmitting(false)
      setGenerationStartedAtMs(null)
      setSubmitError(getErrorMessage(error, t('storyResources.form.errors.submitFailed')))
      if (formState.shouldGenerate) {
        setActiveStep('planner')
      }
    }
  }

  const stepLabels = [
    t('storyResources.createWizard.steps.concept'),
    t('storyResources.createWizard.steps.characters'),
    t('storyResources.createWizard.steps.seeds'),
    t('storyResources.createWizard.steps.planner'),
  ]
  const activeStepIndex =
    activeStep === 'concept' ? 0 : activeStep === 'characters' ? 1 : activeStep === 'seeds' ? 2 : 3
  const generatingDescription =
    generatingPhase === 'creating'
      ? t('storyResources.createWizard.loading.preparing')
      : t('storyResources.createWizard.loading.generating')
  const plannerBindingsUnavailable =
    availableApiGroups.length === 0 || availablePresets.length === 0

  return (
    <Dialog onOpenChange={handleOpenChange} open={open}>
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
          <DialogTitle>{t('storyResources.createWizard.title')}</DialogTitle>
          {activeStep !== 'generating' ? (
            <div className="grid gap-2 pt-2 md:grid-cols-4">
              {stepLabels.map((label, index) => (
                <StepChip
                  active={index <= activeStepIndex}
                  index={index + 1}
                  key={label}
                  label={label}
                />
              ))}
            </div>
          ) : null}
        </DialogHeader>

        <DialogBody className="max-h-[calc(92vh-12rem)] overflow-y-auto pt-6">
          <AnimatePresence initial={false} mode="wait">
            {activeStep === 'concept' ? (
              <motion.div
                animate={{ opacity: 1, y: 0 }}
                className="space-y-5"
                initial={{ opacity: 0, y: 14 }}
                key="concept"
                transition={{ duration: 0.22, ease: [0.22, 1, 0.36, 1] }}
              >
                <div className="space-y-2">
                  <h3 className="font-display text-[1.85rem] text-[var(--color-text-primary)]">
                    {t('storyResources.createWizard.headings.concept')}
                  </h3>
                  <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                    {t('storyResources.createWizard.descriptions.concept')}
                  </p>
                </div>

                <Field
                  htmlFor={fieldIds.storyConcept}
                  label={t('storyResources.form.fields.storyConcept')}
                >
                  <Textarea
                    autoFocus
                    id={fieldIds.storyConcept}
                    onChange={(event) => {
                      setFormState((currentFormState) => ({
                        ...currentFormState,
                        storyConcept: event.target.value,
                      }))
                    }}
                    placeholder={t('storyResources.form.placeholders.storyConcept')}
                    rows={6}
                    value={formState.storyConcept}
                  />
                </Field>
              </motion.div>
            ) : null}

            {activeStep === 'characters' ? (
              <motion.div
                animate={{ opacity: 1, y: 0 }}
                className="space-y-5"
                initial={{ opacity: 0, y: 14 }}
                key="characters"
                transition={{ duration: 0.22, ease: [0.22, 1, 0.36, 1] }}
              >
                <div className="space-y-2">
                  <h3 className="font-display text-[1.85rem] text-[var(--color-text-primary)]">
                    {t('storyResources.createWizard.headings.characters')}
                  </h3>
                  <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                    {t('storyResources.createWizard.descriptions.characters')}
                  </p>
                </div>

                <Field label={t('storyResources.form.fields.characters')}>
                  <StoryResourceCharacterSelector
                    characters={availableCharacters}
                    disabled={isSubmitting}
                    loading={referencesLoading}
                    onChangeSelectedCharacterIds={(characterIds) => {
                      setFormState((currentFormState) => ({
                        ...currentFormState,
                        characterIds,
                      }))
                    }}
                    selectedCharacterIds={formState.characterIds}
                  />
                </Field>
              </motion.div>
            ) : null}

            {activeStep === 'seeds' ? (
              <motion.div
                animate={{ opacity: 1, y: 0 }}
                className="space-y-5"
                initial={{ opacity: 0, y: 14 }}
                key="seeds"
                transition={{ duration: 0.22, ease: [0.22, 1, 0.36, 1] }}
              >
                <div className="space-y-2">
                  <h3 className="font-display text-[1.85rem] text-[var(--color-text-primary)]">
                    {t('storyResources.createWizard.headings.seeds')}
                  </h3>
                  <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                    {t('storyResources.createWizard.descriptions.seeds')}
                  </p>
                </div>

                <div className="grid gap-4 md:grid-cols-2">
                  <Field
                    description={t('storyResources.createWizard.seedHints.player')}
                    htmlFor={fieldIds.playerSchemaIdSeed}
                    label={t('storyResources.form.fields.playerSchemaIdSeed')}
                  >
                    <Select
                      allowClear
                      clearLabel={t('storyResources.form.placeholders.schemaSeedClear')}
                      disabled={availableSchemas.length === 0}
                      items={schemaOptions}
                      onValueChange={(value) => {
                        setFormState((currentFormState) => ({
                          ...currentFormState,
                          playerSchemaIdSeed: value,
                        }))
                      }}
                      placeholder={t('storyResources.form.placeholders.schemaSeed')}
                      textAlign="start"
                      triggerId={fieldIds.playerSchemaIdSeed}
                      value={formState.playerSchemaIdSeed || undefined}
                    />
                  </Field>

                  <Field
                    description={t('storyResources.createWizard.seedHints.world')}
                    htmlFor={fieldIds.worldSchemaIdSeed}
                    label={t('storyResources.form.fields.worldSchemaIdSeed')}
                  >
                    <Select
                      allowClear
                      clearLabel={t('storyResources.form.placeholders.schemaSeedClear')}
                      disabled={availableSchemas.length === 0}
                      items={schemaOptions}
                      onValueChange={(value) => {
                        setFormState((currentFormState) => ({
                          ...currentFormState,
                          worldSchemaIdSeed: value,
                        }))
                      }}
                      placeholder={t('storyResources.form.placeholders.schemaSeed')}
                      textAlign="start"
                      triggerId={fieldIds.worldSchemaIdSeed}
                      value={formState.worldSchemaIdSeed || undefined}
                    />
                  </Field>
                </div>

                {availableSchemas.length === 0 ? (
                  <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-4 py-5 text-sm text-[var(--color-text-secondary)]">
                    {t('storyResources.createWizard.noSchemas')}
                  </div>
                ) : null}

                <Field
                  description={t('storyResources.createWizard.seedHints.lorebooks')}
                  label={t('storyResources.form.fields.lorebooks')}
                >
                  <StoryResourceLorebookSelector
                    emptyAction={
                      <DialogRouteButton
                        onRequestClose={() => {
                          handleOpenChange(false)
                        }}
                        to={appPaths.lorebooks}
                        variant="secondary"
                      >
                        {t('storyResources.form.openLorebooks')}
                      </DialogRouteButton>
                    }
                    emptyMessage={t('storyResources.form.emptyLorebooks')}
                    lorebooks={lorebookOptions}
                    noSelectionLabel={t('storyResources.form.emptyLorebookSelection')}
                    onToggleLorebook={toggleLorebook}
                    selectedLorebookIds={formState.lorebookIds}
                  />
                </Field>
              </motion.div>
            ) : null}

            {activeStep === 'planner' ? (
              <motion.div
                animate={{ opacity: 1, y: 0 }}
                className="space-y-5"
                initial={{ opacity: 0, y: 14 }}
                key="planner"
                transition={{ duration: 0.22, ease: [0.22, 1, 0.36, 1] }}
              >
                <div className="space-y-2">
                  <h3 className="font-display text-[1.85rem] text-[var(--color-text-primary)]">
                    {t('storyResources.createWizard.headings.planner')}
                  </h3>
                  <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                    {t('storyResources.createWizard.descriptions.planner')}
                  </p>
                </div>

                <StoryInputFlowCard
                  badgeLabel={t('storyResources.inputFlow.badge')}
                  description={t('storyResources.createWizard.flowDescription')}
                  rawDescription={t('storyResources.inputFlow.rawDescription')}
                  rawLabel={t('storyResources.inputFlow.rawLabel')}
                  refinedDescription={t('storyResources.inputFlow.refinedDescription')}
                  refinedLabel={t('storyResources.inputFlow.refinedLabel')}
                />

                <div className="grid gap-3">
                  <button
                    className={cn(
                      'rounded-[1.55rem] border px-5 py-5 text-left transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)]',
                      formState.shouldGenerate
                        ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)]'
                        : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] hover:border-[var(--color-accent-copper-soft)]',
                    )}
                    onClick={() => {
                      setFormState((currentFormState) => ({
                        ...currentFormState,
                        shouldGenerate: true,
                      }))
                    }}
                    type="button"
                  >
                    <div className="flex items-start justify-between gap-4">
                      <div className="space-y-1.5">
                        <p className="text-base font-medium text-[var(--color-text-primary)]">
                          {t('storyResources.createWizard.plannerOptions.generate.title')}
                        </p>
                        <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                          {t('storyResources.createWizard.plannerOptions.generate.description')}
                        </p>
                      </div>
                      <span
                        className={cn(
                          'inline-flex size-6 shrink-0 items-center justify-center rounded-full border text-xs',
                          formState.shouldGenerate
                            ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold)] text-[var(--color-accent-ink)]'
                            : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-transparent',
                        )}
                      >
                        ✓
                      </span>
                    </div>
                  </button>

                  <button
                    className={cn(
                      'rounded-[1.55rem] border px-5 py-5 text-left transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)]',
                      !formState.shouldGenerate
                        ? 'border-[var(--color-accent-copper-soft)] bg-[color-mix(in_srgb,var(--color-accent-copper-soft)_42%,var(--color-bg-elevated))]'
                        : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] hover:border-[var(--color-accent-copper-soft)]',
                    )}
                    onClick={() => {
                      setFormState((currentFormState) => ({
                        ...currentFormState,
                        shouldGenerate: false,
                      }))
                    }}
                    type="button"
                  >
                    <div className="flex items-start justify-between gap-4">
                      <div className="space-y-1.5">
                        <p className="text-base font-medium text-[var(--color-text-primary)]">
                          {t('storyResources.createWizard.plannerOptions.skip.title')}
                        </p>
                        <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                          {t('storyResources.createWizard.plannerOptions.skip.description')}
                        </p>
                      </div>
                      <span
                        className={cn(
                          'inline-flex size-6 shrink-0 items-center justify-center rounded-full border text-xs',
                          !formState.shouldGenerate
                            ? 'border-[var(--color-accent-copper-soft)] bg-[var(--color-bg-panel)] text-[var(--color-text-primary)]'
                            : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-transparent',
                        )}
                      >
                        ✓
                      </span>
                    </div>
                  </button>
                </div>

                {formState.shouldGenerate ? (
                  <div className="space-y-4 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                    <div className="space-y-1.5">
                      <h4 className="text-sm font-medium text-[var(--color-text-primary)]">
                        {t('storyResources.createWizard.bindings.title')}
                      </h4>
                      <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                        {t('storyResources.createWizard.bindings.description')}
                      </p>
                    </div>

                    {plannerBindingsUnavailable ? (
                      <div className="space-y-4 rounded-[1.25rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_86%,transparent)] px-4 py-4">
                        <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                          {availableApiGroups.length === 0
                            ? t('storyResources.createWizard.bindings.missingApiGroups')
                            : t('storyResources.createWizard.bindings.missingPresets')}
                        </p>
                        <div className="flex justify-end">
                          <DialogRouteButton
                            onRequestClose={() => {
                              handleOpenChange(false)
                            }}
                            to={availableApiGroups.length === 0 ? appPaths.apis : appPaths.presets}
                            variant="secondary"
                          >
                            {availableApiGroups.length === 0
                              ? t('storyResources.createWizard.bindings.openApiGroups')
                              : t('storyResources.createWizard.bindings.openPresets')}
                          </DialogRouteButton>
                        </div>
                      </div>
                    ) : (
                      <div className="grid gap-4 md:grid-cols-2">
                        <Field
                          htmlFor={fieldIds.apiGroupId}
                          label={t('storyResources.form.fields.apiGroupId')}
                        >
                          <Select
                            items={apiGroupOptions}
                            onValueChange={(value) => {
                              setFormState((currentFormState) => ({
                                ...currentFormState,
                                apiGroupId: value,
                              }))
                            }}
                            placeholder={t('storyResources.form.placeholders.apiGroupId')}
                            textAlign="start"
                            triggerId={fieldIds.apiGroupId}
                            value={formState.apiGroupId || undefined}
                          />
                        </Field>

                        <Field
                          htmlFor={fieldIds.presetId}
                          label={t('storyResources.form.fields.presetId')}
                        >
                          <Select
                            items={presetOptions}
                            onValueChange={(value) => {
                              setFormState((currentFormState) => ({
                                ...currentFormState,
                                presetId: value,
                              }))
                            }}
                            placeholder={t('storyResources.form.placeholders.presetId')}
                            textAlign="start"
                            triggerId={fieldIds.presetId}
                            value={formState.presetId || undefined}
                          />
                        </Field>
                      </div>
                    )}
                  </div>
                ) : null}
              </motion.div>
            ) : null}

            {activeStep === 'generating' ? (
              <motion.div
                animate={{ opacity: 1, y: 0 }}
                initial={{ opacity: 0, y: 14 }}
                key="generating"
                transition={{ duration: 0.22, ease: [0.22, 1, 0.36, 1] }}
              >
                {generationStartedAtMs !== null ? (
                  <GenerationLoadingStage
                    description={generatingDescription}
                    elapsedLabel={t('storyResources.createWizard.loading.elapsed')}
                    identifier={generatedResourceId}
                    startedAtMs={generationStartedAtMs}
                    statusLabel={t('storyResources.createWizard.loading.badge')}
                    title={t('storyResources.createWizard.loading.title')}
                  />
                ) : null}
              </motion.div>
            ) : null}
          </AnimatePresence>
        </DialogBody>

        {activeStep !== 'generating' ? (
          <DialogFooter className="sm:items-center">
            <DialogClose asChild>
              <Button disabled={isSubmitting} size="md" variant="ghost">
                {t('storyResources.actions.cancel')}
              </Button>
            </DialogClose>

            <div className="flex flex-col-reverse gap-3 sm:ml-auto sm:flex-row">
              {activeStep !== 'concept' ? (
                <Button disabled={isSubmitting} onClick={goBack} size="md" variant="secondary">
                  {t('storyResources.actions.back')}
                </Button>
              ) : null}

              {activeStep !== 'planner' ? (
                <Button
                  disabled={
                    isSubmitting ||
                    (referencesLoading && (activeStep === 'characters' || activeStep === 'seeds'))
                  }
                  onClick={goNext}
                  size="md"
                >
                  {t('storyResources.actions.next')}
                </Button>
              ) : (
                <Button
                  disabled={
                    isSubmitting ||
                    referencesLoading ||
                    availableCharacters.length === 0 ||
                    (formState.shouldGenerate &&
                      (plannerBindingsUnavailable ||
                        formState.apiGroupId.trim().length === 0 ||
                        formState.presetId.trim().length === 0))
                  }
                  onClick={() => {
                    void handleCreate()
                  }}
                  size="md"
                >
                  {isSubmitting
                    ? t('storyResources.actions.generating')
                    : formState.shouldGenerate
                      ? t('storyResources.actions.createAndGenerate')
                      : t('storyResources.actions.create')}
                </Button>
              )}
            </div>
          </DialogFooter>
        ) : null}
      </DialogContent>
    </Dialog>
  )
}
