import type { ReactNode } from 'react'
import { useCallback, useEffect, useId, useMemo, useState } from 'react'
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
import { Input } from '../../components/ui/input'
import { SegmentedSelector } from '../../components/ui/segmented-selector'
import { Select } from '../../components/ui/select'
import { Textarea } from '../../components/ui/textarea'
import { useToastMessage } from '../../components/ui/toast-context'
import type { ApiGroup, Preset } from '../apis/types'
import type { CharacterSummary } from '../characters/types'
import type { Lorebook } from '../lorebooks/types'
import type { SchemaResource } from '../schemas/types'
import { generateAndSaveStoryPlan, getStoryResource, updateStoryResource } from './api'
import { StoryResourceCharacterSelector } from './story-resource-character-selector'
import { StoryResourceLorebookSelector } from './story-resource-lorebook-selector'
import type { StoryResource } from './types'

type NoticeTone = 'error' | 'success' | 'warning'
type SubmitIntent = 'generate' | 'save'
type EditTab = 'basic' | 'characters' | 'draft' | 'settings'

type StoryResourceFormDialogProps = {
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
  resourceId?: string | null
}

type FormState = {
  characterIds: string[]
  lorebookIds: string[]
  plannedStory: string
  playerSchemaIdSeed: string
  storyConcept: string
  worldSchemaIdSeed: string
}

function createInitialFormState(): FormState {
  return {
    characterIds: [],
    lorebookIds: [],
    plannedStory: '',
    playerSchemaIdSeed: '',
    storyConcept: '',
    worldSchemaIdSeed: '',
  }
}

function createFormStateFromResource(resource: StoryResource): FormState {
  return {
    characterIds: [...resource.character_ids],
    lorebookIds: [...resource.lorebook_ids],
    plannedStory: resource.planned_story ?? '',
    playerSchemaIdSeed: resource.player_schema_id_seed ?? '',
    storyConcept: resource.story_concept,
    worldSchemaIdSeed: resource.world_schema_id_seed ?? '',
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

function LoadingSkeleton() {
  return (
    <div className="space-y-5">
      <div className="grid gap-3 md:grid-cols-4">
        {Array.from({ length: 4 }).map((_, index) => (
          <div
            className="h-11 animate-pulse rounded-[1rem] bg-[var(--color-bg-elevated)]"
            key={index}
          />
        ))}
      </div>
      <div className="space-y-4 rounded-[1.5rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-5">
        <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-panel)]" />
        <div className="h-36 animate-pulse rounded-[1.35rem] bg-[var(--color-bg-panel)]" />
        <div className="grid gap-4 md:grid-cols-2">
          <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-panel)]" />
          <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-panel)]" />
        </div>
      </div>
    </div>
  )
}

export function StoryResourceFormDialog({
  availableCharacters,
  availableApiGroups,
  availableLorebooks,
  availablePresets,
  availableSchemas,
  onCompleted,
  onOpenChange,
  open,
  referencesLoading,
  resourceId,
}: StoryResourceFormDialogProps) {
  const { t } = useTranslation()
  const fieldIdPrefix = useId()
  const [formState, setFormState] = useState<FormState>(createInitialFormState)
  const [initialResource, setInitialResource] = useState<StoryResource | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  const [submitIntent, setSubmitIntent] = useState<SubmitIntent>('save')
  const [plannerApiGroupId, setPlannerApiGroupId] = useState('')
  const [plannerPresetId, setPlannerPresetId] = useState('')
  const [activeTab, setActiveTab] = useState<EditTab>('basic')
  useToastMessage(submitError)

  const fieldIds = {
    apiGroupId: `${fieldIdPrefix}-api-group-id`,
    plannedStory: `${fieldIdPrefix}-planned-story`,
    presetId: `${fieldIdPrefix}-preset-id`,
    playerSchemaIdSeed: `${fieldIdPrefix}-player-schema-seed`,
    resourceId: `${fieldIdPrefix}-resource-id`,
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
  const plannerBindingsUnavailable =
    availableApiGroups.length === 0 || availablePresets.length === 0
  const lorebookOptions = useMemo(
    () =>
      availableLorebooks.map((lorebook) => ({
        display_name: lorebook.display_name,
        lorebook_id: lorebook.lorebook_id,
      })),
    [availableLorebooks],
  )

  useEffect(() => {
    if (!open) {
      setFormState(createInitialFormState())
      setInitialResource(null)
      setIsLoading(false)
      setIsSubmitting(false)
      setSubmitError(null)
      setSubmitIntent('save')
      setPlannerApiGroupId('')
      setPlannerPresetId('')
      setActiveTab('basic')
      return
    }

    if (!resourceId) {
      setInitialResource(null)
      setIsLoading(false)
      return
    }

    const controller = new AbortController()

    setIsLoading(true)
    setIsSubmitting(false)
    setSubmitError(null)
    setSubmitIntent('save')
    setPlannerApiGroupId('')
    setPlannerPresetId('')
    setActiveTab('basic')

    void getStoryResource(resourceId, controller.signal)
      .then((resource) => {
        if (controller.signal.aborted) {
          return
        }

        setInitialResource(resource)
        setFormState(createFormStateFromResource(resource))
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setInitialResource(null)
          setSubmitError(getErrorMessage(error, t('storyResources.feedback.loadResourceFailed')))
        }
      })
      .finally(() => {
        if (!controller.signal.aborted) {
          setIsLoading(false)
        }
      })

    return () => {
      controller.abort()
    }
  }, [open, resourceId, t])

  const setActiveTabFromValue = useCallback((value: string) => {
    if (value === 'basic' || value === 'characters' || value === 'settings' || value === 'draft') {
      setActiveTab(value)
    }
  }, [])

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

  function validateForm(nextFormState: FormState) {
    if (nextFormState.storyConcept.trim().length === 0) {
      return {
        error: t('storyResources.form.errors.storyConceptRequired'),
        tab: 'basic' as const,
      }
    }

    if (nextFormState.characterIds.length === 0) {
      return {
        error: t('storyResources.form.errors.charactersRequired'),
        tab: 'characters' as const,
      }
    }

    return { error: null, tab: null }
  }

  function hasUnsupportedSeedClear(nextFormState: FormState) {
    if (!initialResource) {
      return false
    }

    const hadPlayerSeed = Boolean(initialResource.player_schema_id_seed?.trim())
    const hadWorldSeed = Boolean(initialResource.world_schema_id_seed?.trim())

    return (
      (hadPlayerSeed && nextFormState.playerSchemaIdSeed.length === 0) ||
      (hadWorldSeed && nextFormState.worldSchemaIdSeed.length === 0)
    )
  }

  async function handleSubmit(intent: SubmitIntent) {
    const nextFormState = {
      ...formState,
      characterIds: [...new Set(formState.characterIds)],
      lorebookIds: [...new Set(formState.lorebookIds)],
      plannedStory: formState.plannedStory.trim(),
      playerSchemaIdSeed: formState.playerSchemaIdSeed.trim(),
      storyConcept: formState.storyConcept.trim(),
      worldSchemaIdSeed: formState.worldSchemaIdSeed.trim(),
    }

    const validationResult = validateForm(nextFormState)

    setFormState(nextFormState)
    setSubmitError(null)

    if (validationResult.error) {
      setActiveTab(validationResult.tab ?? 'basic')
      setSubmitError(validationResult.error)
      return
    }

    if (!initialResource) {
      setSubmitError(t('storyResources.feedback.loadResourceFailed'))
      return
    }

    if (hasUnsupportedSeedClear(nextFormState)) {
      setActiveTab('settings')
      setSubmitError(t('storyResources.form.errors.schemaSeedClearUnsupported'))
      return
    }

    if (intent === 'generate' && plannerApiGroupId.trim().length === 0) {
      setActiveTab('draft')
      setSubmitError(t('storyResources.form.errors.apiGroupRequired'))
      return
    }

    if (intent === 'generate' && plannerPresetId.trim().length === 0) {
      setActiveTab('draft')
      setSubmitError(t('storyResources.form.errors.presetRequired'))
      return
    }

    setIsSubmitting(true)
    setSubmitIntent(intent)

    try {
      const savedResource = await updateStoryResource({
        character_ids: nextFormState.characterIds,
        lorebook_ids: nextFormState.lorebookIds,
        planned_story: nextFormState.plannedStory,
        ...(nextFormState.playerSchemaIdSeed
          ? { player_schema_id_seed: nextFormState.playerSchemaIdSeed }
          : {}),
        resource_id: initialResource.resource_id,
        story_concept: nextFormState.storyConcept,
        ...(nextFormState.worldSchemaIdSeed
          ? { world_schema_id_seed: nextFormState.worldSchemaIdSeed }
          : {}),
      })

      let nextResource = savedResource
      let tone: NoticeTone = 'success'
      let message: string = t('storyResources.feedback.updated', {
        id: savedResource.resource_id,
      })

      if (intent === 'generate') {
        try {
          const generated = await generateAndSaveStoryPlan({
            apiGroupId: plannerApiGroupId,
            presetId: plannerPresetId,
            resourceId: savedResource.resource_id,
          })

          nextResource = generated.resource
          message = t('storyResources.feedback.generated', {
            id: savedResource.resource_id,
          })
        } catch (error) {
          tone = 'warning'
          message = getErrorMessage(
            error,
            t('storyResources.feedback.savedButGenerateFailed', {
              id: savedResource.resource_id,
            }),
          )
        }
      }

      setInitialResource(nextResource)
      setFormState(createFormStateFromResource(nextResource))
      setActiveTab(intent === 'generate' ? 'draft' : activeTab)

      await onCompleted({
        message,
        resource: nextResource,
        tone,
      })

      onOpenChange(false)
    } catch (error) {
      setSubmitError(getErrorMessage(error, t('storyResources.form.errors.submitFailed')))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent
        aria-describedby={undefined}
        className="w-[min(96vw,72rem)] overflow-hidden"
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
          <DialogTitle>
            {initialResource?.resource_id ?? t('storyResources.form.editTitle')}
          </DialogTitle>
          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
            {t('storyResources.editPage.description')}
          </p>
        </DialogHeader>

        <DialogBody className="max-h-[calc(92vh-12rem)] space-y-6 overflow-y-auto pt-6">
          {isLoading ? (
            <LoadingSkeleton />
          ) : !initialResource ? (
            <div className="rounded-[1.6rem] border border-dashed border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-6 py-12 text-center">
              <h3 className="font-display text-3xl text-[var(--color-text-primary)]">
                {t('storyResources.editPage.loadErrorTitle')}
              </h3>
              <p className="mt-3 text-sm leading-7 text-[var(--color-text-secondary)]">
                {submitError || t('storyResources.feedback.loadResourceFailed')}
              </p>
            </div>
          ) : (
            <div className="space-y-6">
              <SegmentedSelector
                ariaLabel={t('storyResources.editPage.tabsLabel')}
                items={[
                  { label: t('storyResources.editPage.tabs.basic'), value: 'basic' },
                  { label: t('storyResources.editPage.tabs.characters'), value: 'characters' },
                  { label: t('storyResources.editPage.tabs.settings'), value: 'settings' },
                  { label: t('storyResources.editPage.tabs.draft'), value: 'draft' },
                ]}
                layoutId="story-resource-edit-tabs-dialog"
                onValueChange={setActiveTabFromValue}
                value={activeTab}
              />

              <div className="rounded-[1.55rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-5">
                {activeTab === 'basic' ? (
                  <div className="space-y-5">
                    <Field
                      description={t('storyResources.form.fields.resourceIdHint')}
                      htmlFor={fieldIds.resourceId}
                      label={t('storyResources.form.fields.resourceId')}
                    >
                      <Input
                        id={fieldIds.resourceId}
                        readOnly
                        value={initialResource.resource_id}
                      />
                    </Field>

                    <Field
                      description={t('storyResources.form.fieldDescriptions.storyConcept')}
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
                        rows={7}
                        value={formState.storyConcept}
                      />
                    </Field>
                  </div>
                ) : null}

                {activeTab === 'characters' ? (
                  <div className="space-y-5">
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
                  </div>
                ) : null}

                {activeTab === 'settings' ? (
                  <div className="space-y-5">
                    <div className="grid gap-4 md:grid-cols-2">
                      <Field
                        htmlFor={fieldIds.playerSchemaIdSeed}
                        label={t('storyResources.form.fields.playerSchemaIdSeed')}
                      >
                        <Select
                          allowClear
                          clearLabel={t('storyResources.form.placeholders.schemaSeedClear')}
                          disabled={availableSchemas.length === 0 || isSubmitting}
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
                        htmlFor={fieldIds.worldSchemaIdSeed}
                        label={t('storyResources.form.fields.worldSchemaIdSeed')}
                      >
                        <Select
                          allowClear
                          clearLabel={t('storyResources.form.placeholders.schemaSeedClear')}
                          disabled={availableSchemas.length === 0 || isSubmitting}
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

                    <Field
                      description={t('storyResources.form.fieldDescriptions.lorebooks')}
                      label={t('storyResources.form.fields.lorebooks')}
                    >
                      <StoryResourceLorebookSelector
                        disabled={isSubmitting || referencesLoading}
                        emptyAction={
                          <DialogRouteButton
                            onRequestClose={() => {
                              onOpenChange(false)
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
                  </div>
                ) : null}

                {activeTab === 'draft' ? (
                  <div className="space-y-5">
                    <Field
                      description={t('storyResources.form.fieldDescriptions.plannedStory')}
                      htmlFor={fieldIds.plannedStory}
                      label={t('storyResources.form.fields.plannedStory')}
                    >
                      <Textarea
                        id={fieldIds.plannedStory}
                        onChange={(event) => {
                          setFormState((currentFormState) => ({
                            ...currentFormState,
                            plannedStory: event.target.value,
                          }))
                        }}
                        placeholder={t('storyResources.form.placeholders.plannedStory')}
                        rows={12}
                        value={formState.plannedStory}
                      />
                    </Field>

                    <div className="space-y-4 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_86%,transparent)] px-4 py-4">
                      <div className="space-y-1.5">
                        <h3 className="text-sm font-medium text-[var(--color-text-primary)]">
                          {t('storyResources.form.generationBindingsTitle')}
                        </h3>
                        <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                          {t('storyResources.form.generationBindingsDescription')}
                        </p>
                      </div>

                      {plannerBindingsUnavailable ? (
                        <div className="space-y-4 rounded-[1.25rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-4 py-4">
                          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                            {availableApiGroups.length === 0
                              ? t('storyResources.form.emptyApiGroups')
                              : t('storyResources.form.emptyPresets')}
                          </p>
                          <div className="flex justify-end">
                            <DialogRouteButton
                              onRequestClose={() => {
                                onOpenChange(false)
                              }}
                              to={
                                availableApiGroups.length === 0 ? appPaths.apis : appPaths.presets
                              }
                              variant="secondary"
                            >
                              {availableApiGroups.length === 0
                                ? t('storyResources.form.openApiGroups')
                                : t('storyResources.form.openPresets')}
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
                              onValueChange={setPlannerApiGroupId}
                              placeholder={t('storyResources.form.placeholders.apiGroupId')}
                              textAlign="start"
                              triggerId={fieldIds.apiGroupId}
                              value={plannerApiGroupId || undefined}
                            />
                          </Field>

                          <Field
                            htmlFor={fieldIds.presetId}
                            label={t('storyResources.form.fields.presetId')}
                          >
                            <Select
                              items={presetOptions}
                              onValueChange={setPlannerPresetId}
                              placeholder={t('storyResources.form.placeholders.presetId')}
                              textAlign="start"
                              triggerId={fieldIds.presetId}
                              value={plannerPresetId || undefined}
                            />
                          </Field>
                        </div>
                      )}
                    </div>
                  </div>
                ) : null}
              </div>
            </div>
          )}
        </DialogBody>

        <DialogFooter className="sm:items-center">
          <DialogClose asChild>
            <Button disabled={isSubmitting} size="md" variant="secondary">
              {t('storyResources.actions.cancel')}
            </Button>
          </DialogClose>

          {initialResource ? (
            <div className="flex flex-col-reverse gap-2 sm:ml-auto sm:flex-row">
              <Button
                disabled={isSubmitting || referencesLoading}
                onClick={() => {
                  void handleSubmit('save')
                }}
                size="md"
                variant="secondary"
              >
                {isSubmitting && submitIntent === 'save'
                  ? t('storyResources.actions.saving')
                  : t('storyResources.actions.saveChanges')}
              </Button>

              <Button
                disabled={
                  isSubmitting ||
                  referencesLoading ||
                  plannerBindingsUnavailable ||
                  plannerApiGroupId.trim().length === 0 ||
                  plannerPresetId.trim().length === 0
                }
                onClick={() => {
                  void handleSubmit('generate')
                }}
                size="md"
              >
                {isSubmitting && submitIntent === 'generate'
                  ? t('storyResources.actions.generating')
                  : t('storyResources.actions.saveAndGenerate')}
              </Button>
            </div>
          ) : null}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
