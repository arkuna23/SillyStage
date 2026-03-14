import { useEffect, useId, useMemo, useState } from 'react'
import type { ReactNode } from 'react'
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
import { Input } from '../../components/ui/input'
import { Select } from '../../components/ui/select'
import { Textarea } from '../../components/ui/textarea'
import { cn } from '../../lib/cn'
import type { CharacterSummary } from '../characters/types'
import type { SchemaResource } from '../schemas/types'
import {
  createStoryResource,
  generateAndSaveStoryPlan,
  getStoryResource,
  updateStoryResource,
} from './api'
import type { StoryResource } from './types'

type NoticeTone = 'error' | 'success' | 'warning'
type SubmitIntent = 'generate' | 'save'
type StoryResourceFormMode = 'create' | 'edit'

type StoryResourceFormDialogProps = {
  availableCharacters: ReadonlyArray<CharacterSummary>
  availableSchemas: ReadonlyArray<SchemaResource>
  mode: StoryResourceFormMode
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
  plannedStory: string
  playerSchemaIdSeed: string
  storyConcept: string
  worldSchemaIdSeed: string
}

function createInitialFormState(): FormState {
  return {
    characterIds: [],
    plannedStory: '',
    playerSchemaIdSeed: '',
    storyConcept: '',
    worldSchemaIdSeed: '',
  }
}

function createFormStateFromResource(resource: StoryResource): FormState {
  return {
    characterIds: [...resource.character_ids],
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
        <label className="block text-sm font-medium text-[var(--color-text-primary)]" htmlFor={htmlFor}>
          {label}
        </label>
      ) : (
        <span className="block text-sm font-medium text-[var(--color-text-primary)]">
          {label}
        </span>
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
      <div className="h-24 animate-pulse rounded-[1.5rem] bg-[var(--color-bg-elevated)]" />
      <div className="h-40 animate-pulse rounded-[1.5rem] bg-[var(--color-bg-elevated)]" />
      <div className="grid gap-4 md:grid-cols-2">
        <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-elevated)]" />
        <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-elevated)]" />
      </div>
      <div className="h-32 animate-pulse rounded-[1.5rem] bg-[var(--color-bg-elevated)]" />
    </div>
  )
}

export function StoryResourceFormDialog({
  availableCharacters,
  availableSchemas,
  mode,
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
  const isEditMode = mode === 'edit'

  const fieldIds = {
    plannedStory: `${fieldIdPrefix}-planned-story`,
    playerSchemaIdSeed: `${fieldIdPrefix}-player-schema-seed`,
    resourceId: `${fieldIdPrefix}-resource-id`,
    storyConcept: `${fieldIdPrefix}-story-concept`,
    worldSchemaIdSeed: `${fieldIdPrefix}-world-schema-seed`,
  } as const

  const characterLookup = useMemo(
    () => new Map(availableCharacters.map((character) => [character.character_id, character])),
    [availableCharacters],
  )

  const schemaOptions = useMemo(
    () =>
      availableSchemas.map((schema) => ({
        label: schema.display_name,
        value: schema.schema_id,
      })),
    [availableSchemas],
  )

  useEffect(() => {
    if (!open) {
      setFormState(createInitialFormState())
      setInitialResource(null)
      setIsLoading(false)
      setIsSubmitting(false)
      setSubmitError(null)
      setSubmitIntent('save')
      return
    }

    if (mode === 'create') {
      setFormState(createInitialFormState())
      setInitialResource(null)
      setIsLoading(false)
      setIsSubmitting(false)
      setSubmitError(null)
      setSubmitIntent('save')
      return
    }

    if (!resourceId) {
      return
    }

    const controller = new AbortController()
    setIsLoading(true)
    setIsSubmitting(false)
    setSubmitError(null)
    setSubmitIntent('save')

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
  }, [mode, open, resourceId, t])

  function toggleCharacter(characterId: string) {
    setFormState((currentFormState) => {
      const isSelected = currentFormState.characterIds.includes(characterId)

      return {
        ...currentFormState,
        characterIds: isSelected
          ? currentFormState.characterIds.filter((id) => id !== characterId)
          : [...currentFormState.characterIds, characterId],
      }
    })
  }

  function validateForm(nextFormState: FormState): string | null {
    if (nextFormState.storyConcept.trim().length === 0) {
      return t('storyResources.form.errors.storyConceptRequired')
    }

    if (nextFormState.characterIds.length === 0) {
      return t('storyResources.form.errors.charactersRequired')
    }

    return null
  }

  async function handleSubmit(intent: SubmitIntent) {
    const nextFormState = {
      ...formState,
      characterIds: [...new Set(formState.characterIds)],
      plannedStory: formState.plannedStory.trim(),
      playerSchemaIdSeed: formState.playerSchemaIdSeed.trim(),
      storyConcept: formState.storyConcept.trim(),
      worldSchemaIdSeed: formState.worldSchemaIdSeed.trim(),
    }

    const validationError = validateForm(nextFormState)

    setFormState(nextFormState)
    setSubmitError(null)

    if (validationError) {
      setSubmitError(validationError)
      return
    }

    if (isEditMode && !initialResource) {
      setSubmitError(t('storyResources.feedback.loadResourceFailed'))
      return
    }

    setIsSubmitting(true)
    setSubmitIntent(intent)

    try {
      const savedResource =
        mode === 'create'
          ? await createStoryResource({
              character_ids: nextFormState.characterIds,
              ...(nextFormState.plannedStory
                ? { planned_story: nextFormState.plannedStory }
                : {}),
              ...(nextFormState.playerSchemaIdSeed
                ? { player_schema_id_seed: nextFormState.playerSchemaIdSeed }
                : {}),
              story_concept: nextFormState.storyConcept,
              ...(nextFormState.worldSchemaIdSeed
                ? { world_schema_id_seed: nextFormState.worldSchemaIdSeed }
                : {}),
            })
          : await updateStoryResource({
              character_ids: nextFormState.characterIds,
              planned_story: nextFormState.plannedStory,
              ...(nextFormState.playerSchemaIdSeed
                ? { player_schema_id_seed: nextFormState.playerSchemaIdSeed }
                : {}),
              resource_id: initialResource!.resource_id,
              story_concept: nextFormState.storyConcept,
              ...(nextFormState.worldSchemaIdSeed
                ? { world_schema_id_seed: nextFormState.worldSchemaIdSeed }
                : {}),
            })

      let noticeTone: NoticeTone = 'success'
      let noticeMessage: string =
        mode === 'create'
          ? t('storyResources.feedback.created', { id: savedResource.resource_id })
          : t('storyResources.feedback.updated', { id: savedResource.resource_id })
      let nextResource = savedResource

      if (intent === 'generate') {
        try {
          const generated = await generateAndSaveStoryPlan(savedResource.resource_id)
          nextResource = generated.resource
          noticeMessage = t('storyResources.feedback.generated', {
            id: savedResource.resource_id,
          })
        } catch (error) {
          noticeTone = 'warning'
          noticeMessage = getErrorMessage(
            error,
            t('storyResources.feedback.savedButGenerateFailed', {
              id: savedResource.resource_id,
            }),
          )
        }
      }

      await onCompleted({
        message: noticeMessage,
        resource: nextResource,
        tone: noticeTone,
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
        className="w-[min(96vw,58rem)] overflow-hidden"
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
            {isEditMode
              ? t('storyResources.form.editTitle')
              : t('storyResources.form.createTitle')}
          </DialogTitle>
        </DialogHeader>

        <DialogBody className="max-h-[calc(92vh-12rem)] overflow-y-auto pt-6">
          {submitError ? (
            <div className="mb-5 rounded-[1.25rem] border border-[var(--color-state-error-line)] bg-[var(--color-state-error-soft)] px-4 py-3 text-sm text-[var(--color-text-primary)]">
              {submitError}
            </div>
          ) : null}

          {isLoading ? (
            <LoadingSkeleton />
          ) : (
            <div className="space-y-5">
              {isEditMode && initialResource ? (
                <Field
                  description={t('storyResources.form.fields.resourceIdHint')}
                  htmlFor={fieldIds.resourceId}
                  label={t('storyResources.form.fields.resourceId')}
                >
                  <Input id={fieldIds.resourceId} readOnly value={initialResource.resource_id} />
                </Field>
              ) : null}

              <Field
                description={t('storyResources.form.fieldDescriptions.storyConcept')}
                htmlFor={fieldIds.storyConcept}
                label={t('storyResources.form.fields.storyConcept')}
              >
                <Textarea
                  autoFocus={!isEditMode}
                  id={fieldIds.storyConcept}
                  onChange={(event) => {
                    setFormState((currentFormState) => ({
                      ...currentFormState,
                      storyConcept: event.target.value,
                    }))
                  }}
                  placeholder={t('storyResources.form.placeholders.storyConcept')}
                  rows={5}
                  value={formState.storyConcept}
                />
              </Field>

              <Field label={t('storyResources.form.fields.characters')}>
                {referencesLoading ? (
                  <div className="h-28 animate-pulse rounded-[1.45rem] bg-[var(--color-bg-elevated)]" />
                ) : availableCharacters.length === 0 ? (
                  <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-5 text-sm text-[var(--color-text-secondary)]">
                    {t('storyResources.form.emptyCharacters')}
                  </div>
                ) : (
                  <div className="space-y-3">
                    <div className="flex flex-wrap gap-2">
                      {formState.characterIds.length > 0 ? (
                        formState.characterIds.map((characterId) => (
                          <Badge className="normal-case px-3 py-1.5" key={characterId} variant="subtle">
                            {characterLookup.get(characterId)?.name ?? characterId}
                          </Badge>
                        ))
                      ) : (
                        <span className="text-sm text-[var(--color-text-muted)]">
                          {t('storyResources.form.emptySelection')}
                        </span>
                      )}
                    </div>

                    <div className="grid gap-2 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-3 sm:grid-cols-2">
                      {availableCharacters.map((character) => {
                        const isSelected = formState.characterIds.includes(character.character_id)

                        return (
                          <button
                            className={cn(
                              'rounded-[1.2rem] border px-3 py-3 text-left transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)]',
                              isSelected
                                ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)]'
                                : 'border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_84%,transparent)] text-[var(--color-text-secondary)] hover:border-[var(--color-accent-copper-soft)] hover:text-[var(--color-text-primary)]',
                            )}
                            key={character.character_id}
                            onClick={() => {
                              toggleCharacter(character.character_id)
                            }}
                            type="button"
                          >
                            <div className="truncate text-sm font-medium">{character.name}</div>
                            <div className="truncate pt-1 font-mono text-[0.74rem] text-[var(--color-text-muted)]">
                              {character.character_id}
                            </div>
                          </button>
                        )
                      })}
                    </div>
                  </div>
                )}
              </Field>

              <div className="grid gap-4 md:grid-cols-2">
                <Field
                  htmlFor={fieldIds.playerSchemaIdSeed}
                  label={t('storyResources.form.fields.playerSchemaIdSeed')}
                >
                  <Select
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
                  htmlFor={fieldIds.worldSchemaIdSeed}
                  label={t('storyResources.form.fields.worldSchemaIdSeed')}
                >
                  <Select
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
                  rows={10}
                  value={formState.plannedStory}
                />
              </Field>
            </div>
          )}
        </DialogBody>

        <DialogFooter className="sm:items-center">
          <DialogClose asChild>
            <Button disabled={isSubmitting} size="md" variant="ghost">
              {t('storyResources.actions.cancel')}
            </Button>
          </DialogClose>

          <div className="flex flex-col-reverse gap-3 sm:ml-auto sm:flex-row">
            <Button
              disabled={isSubmitting || referencesLoading || availableCharacters.length === 0}
              onClick={() => {
                void handleSubmit('save')
              }}
              size="md"
              variant="secondary"
            >
              {isSubmitting && submitIntent === 'save'
                ? t('storyResources.actions.saving')
                : isEditMode
                  ? t('storyResources.actions.saveChanges')
                  : t('storyResources.actions.create')}
            </Button>

            <Button
              disabled={isSubmitting || referencesLoading || availableCharacters.length === 0}
              onClick={() => {
                void handleSubmit('generate')
              }}
              size="md"
            >
              {isSubmitting && submitIntent === 'generate'
                ? t('storyResources.actions.generating')
                : isEditMode
                  ? t('storyResources.actions.saveAndGenerate')
                  : t('storyResources.actions.createAndGenerate')}
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
