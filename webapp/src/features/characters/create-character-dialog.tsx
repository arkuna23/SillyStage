import { useEffect, useId, useMemo, useState } from 'react'
import type { ReactNode } from 'react'
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
import { Select } from '../../components/ui/select'
import { Textarea } from '../../components/ui/textarea'
import { useToastMessage } from '../../components/ui/toast-context'
import { cn } from '../../lib/cn'
import { listSchemas } from '../schemas/api'
import { listCharacters } from './api'
import {
  loadCharacterFolderRegistry,
  normalizeCharacterFolderRegistryName,
} from './folder-registry'
import type { SchemaResource } from '../schemas/types'
import {
  createCharacter,
  getCharacter,
  setCharacterCover,
  updateCharacter,
  withUpdatedCoverSummary,
} from './api'
import {
  characterCoverMimeTypes,
  type CharacterCardContent,
  type CharacterSchemaResult,
  type CharacterSummary,
} from './types'

type NoticeTone = 'success' | 'warning'
type CharacterFormDialogMode = 'create' | 'edit'

type CharacterFormDialogResult = {
  characterId: string
  coverUpdated: boolean
  message: string
  summary: CharacterSummary
  tone: NoticeTone
}

type CharacterFormDialogProps = {
  characterId?: string | null
  mode: CharacterFormDialogMode
  onCompleted: (result: CharacterFormDialogResult) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
}

type WizardStep = 'identity' | 'voice' | 'system'

type FormState = {
  characterId: string
  coverFile: File | null
  name: string
  personality: string
  schemaId: string
  style: string
  systemPrompt: string
  tagsText: string
  folder: string
  folderMode: 'existing' | 'new'
}

function createInitialFormState(): FormState {
  return {
    characterId: '',
    coverFile: null,
    name: '',
    personality: '',
    schemaId: '',
    style: '',
    systemPrompt: '',
    tagsText: '',
    folder: '',
    folderMode: 'existing',
  }
}

function parseTags(value: string) {
  const normalized: string[] = []

  for (const segment of value.split(/[\n,，]/)) {
    const tag = segment.trim()

    if (!tag || normalized.includes(tag)) {
      continue
    }

    normalized.push(tag)
  }

  return normalized
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function createSummaryFromCharacter(character: CharacterSchemaResult): CharacterSummary {
  return {
    character_id: character.character_id,
    cover_file_name: character.cover_file_name,
    cover_mime_type: character.cover_mime_type,
    name: character.content.name,
    personality: character.content.personality,
    style: character.content.style,
    tags: character.content.tags,
    folder: character.content.folder,
  }
}

function createFormStateFromCharacter(character: CharacterSchemaResult): FormState {
  return {
    characterId: character.character_id,
    coverFile: null,
    name: character.content.name,
    personality: character.content.personality,
    schemaId: character.content.schema_id,
    style: character.content.style,
    systemPrompt: character.content.system_prompt,
    tagsText: character.content.tags.join(', '),
    folder: character.content.folder,
    folderMode: 'existing',
  }
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
    <div className="block space-y-2.5">
      {htmlFor ? (
        <label
          className="block text-sm font-medium text-[var(--color-text-primary)]"
          htmlFor={htmlFor}
        >
          {label}
        </label>
      ) : (
        <span className="block text-sm font-medium text-[var(--color-text-primary)]">
          {label}
        </span>
      )}
      {children}
      {description ? (
        <span className="block text-xs leading-6 text-[var(--color-text-muted)]">
          {description}
        </span>
      ) : null}
    </div>
  )
}

function LoadingSkeleton() {
  return (
    <div className="space-y-4">
      <div className="grid gap-3 sm:grid-cols-3">
        {Array.from({ length: 3 }).map((_, index) => (
          <div
            className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4"
            key={index}
          >
            <div className="h-4 w-20 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
          </div>
        ))}
      </div>

      <div className="space-y-4 rounded-[1.55rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-5">
        <div className="h-10 animate-pulse rounded-2xl bg-[var(--color-bg-panel)]" />
        <div className="h-10 animate-pulse rounded-2xl bg-[var(--color-bg-panel)]" />
        <div className="h-24 animate-pulse rounded-[1.45rem] bg-[var(--color-bg-panel)]" />
      </div>
    </div>
  )
}

function SchemaSelectionEmptyState({
  onClose,
  onNavigate,
}: {
  onClose: () => void
  onNavigate: () => void
}) {
  const { t } = useTranslation()

  return (
    <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-5 py-6 text-center">
      <h3 className="font-display text-2xl text-[var(--color-text-primary)]">
        {t('characters.create.schemaEmpty.title')}
      </h3>
      <p className="mt-2 text-sm leading-7 text-[var(--color-text-secondary)]">
        {t('characters.create.schemaEmpty.description')}
      </p>
      <div className="mt-5 flex justify-center">
        <DialogRouteButton
          onRequestClose={() => {
            onNavigate()
            onClose()
          }}
          to={appPaths.schemas}
          variant="secondary"
        >
          {t('characters.create.schemaEmpty.action')}
        </DialogRouteButton>
      </div>
    </div>
  )
}

export function CharacterFormDialog({
  characterId,
  mode,
  onCompleted,
  onOpenChange,
  open,
}: CharacterFormDialogProps) {
  const { t } = useTranslation()
  const fieldIdPrefix = useId()
  const [currentStep, setCurrentStep] = useState<WizardStep>('identity')
  const [formState, setFormState] = useState<FormState>(createInitialFormState)
  const [initialCharacter, setInitialCharacter] = useState<CharacterSchemaResult | null>(null)
  const [availableSchemas, setAvailableSchemas] = useState<SchemaResource[]>([])
  const [availableFolders, setAvailableFolders] = useState<string[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  useToastMessage(submitError)

  const fieldIds = {
    characterId: `${fieldIdPrefix}-character-id`,
    name: `${fieldIdPrefix}-display-name`,
    personality: `${fieldIdPrefix}-personality`,
    schemaId: `${fieldIdPrefix}-schema-id`,
    style: `${fieldIdPrefix}-style`,
    systemPrompt: `${fieldIdPrefix}-system-prompt`,
    tags: `${fieldIdPrefix}-tags`,
    folder: `${fieldIdPrefix}-folder`,
  } as const

  const steps = useMemo(
    () => [
      {
        key: 'identity' as const,
        label: t('characters.create.steps.identity.label'),
      },
      {
        key: 'voice' as const,
        label: t('characters.create.steps.voice.label'),
      },
      {
        key: 'system' as const,
        label: t('characters.create.steps.system.label'),
      },
    ],
    [t],
  )

  const currentStepIndex = steps.findIndex((step) => step.key === currentStep)
  const isLastStep = currentStepIndex === steps.length - 1
  const isEditMode = mode === 'edit'

  useEffect(() => {
    if (!open) {
      setAvailableSchemas([])
      setAvailableFolders([])
      setCurrentStep('identity')
      setFormState(createInitialFormState())
      setInitialCharacter(null)
      setIsLoading(false)
      setIsSubmitting(false)
      setSubmitError(null)
      return
    }

    const controller = new AbortController()
    setIsLoading(true)
    setIsSubmitting(false)
    setSubmitError(null)

    const loadCharacterPromise =
      mode === 'edit' && characterId
        ? getCharacter(characterId, controller.signal)
        : Promise.resolve(null)

    void Promise.all([
      listSchemas(controller.signal),
      listCharacters(controller.signal),
      loadCharacterPromise,
    ])
      .then(([schemas, characters, character]) => {
        if (controller.signal.aborted) {
          return
        }

        setAvailableFolders(
          Array.from(
            new Set(
              [
                ...loadCharacterFolderRegistry(),
                ...characters.map((item) =>
                  normalizeCharacterFolderRegistryName(item.folder),
                ),
              ]
                .filter((folder) => folder.length > 0),
            ),
          ).sort((left, right) => left.localeCompare(right)),
        )

        setCurrentStep('identity')
        setInitialCharacter(character)

        if (character) {
          const nextFormState = createFormStateFromCharacter(character)
          setFormState(nextFormState)

          if (
            nextFormState.schemaId.trim().length > 0 &&
            !schemas.some((schema) => schema.schema_id === nextFormState.schemaId)
          ) {
            setAvailableSchemas([
              {
                display_name: nextFormState.schemaId,
                fields: {},
                schema_id: nextFormState.schemaId,
                tags: [],
                type: 'schema',
              },
              ...schemas,
            ])
            return
          }
        } else {
          setFormState(createInitialFormState())
        }

        setAvailableSchemas(schemas)
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setSubmitError(
            getErrorMessage(
              error,
              mode === 'edit'
                ? t('characters.feedback.loadCharacterFailed')
                : t('characters.feedback.loadSchemasFailed'),
            ),
          )
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
  }, [characterId, mode, open, t])

  function validateIdentityStep(nextFormState: FormState) {
    if (nextFormState.characterId.trim().length === 0) {
      return t('characters.create.errors.idRequired')
    }

    if (nextFormState.name.trim().length === 0) {
      return t('characters.create.errors.nameRequired')
    }

    if (
      nextFormState.coverFile &&
      !characterCoverMimeTypes.includes(
        nextFormState.coverFile.type as (typeof characterCoverMimeTypes)[number],
      )
    ) {
      return t('characters.create.errors.coverTypeInvalid')
    }

    return null
  }

  function validateVoiceStep(nextFormState: FormState) {
    if (nextFormState.personality.trim().length === 0) {
      return t('characters.create.errors.personalityRequired')
    }

    if (nextFormState.style.trim().length === 0) {
      return t('characters.create.errors.styleRequired')
    }

    return null
  }

  function validateSystemStep(nextFormState: FormState) {
    if (nextFormState.schemaId.trim().length === 0) {
      return t('characters.create.errors.schemaIdRequired')
    }

    return null
  }

  function goToStep(nextStepIndex: number) {
    const nextStep = steps[nextStepIndex]

    if (!nextStep) {
      return
    }

    setCurrentStep(nextStep.key)
    setSubmitError(null)
  }

  function handleNextStep() {
    const normalizedFormState = formState
    setSubmitError(null)

    const nextError =
      currentStep === 'identity'
        ? validateIdentityStep(normalizedFormState)
        : currentStep === 'voice'
          ? validateVoiceStep(normalizedFormState)
          : validateSystemStep(normalizedFormState)

    if (nextError) {
      setSubmitError(nextError)
      return
    }

    goToStep(currentStepIndex + 1)
  }

  async function handleSubmit() {
    const normalizedFormState = formState
    const identityError = validateIdentityStep(normalizedFormState)
    const voiceError = validateVoiceStep(normalizedFormState)
    const systemError = validateSystemStep(normalizedFormState)

    setSubmitError(null)

    if (identityError) {
      setSubmitError(identityError)
      setCurrentStep('identity')
      return
    }

    if (voiceError) {
      setSubmitError(voiceError)
      setCurrentStep('voice')
      return
    }

    if (systemError) {
      setSubmitError(systemError)
      setCurrentStep('system')
      return
    }

    if (mode === 'edit' && !initialCharacter) {
      setSubmitError(t('characters.feedback.loadCharacterFailed'))
      return
    }

    const content: CharacterCardContent = {
      id: normalizedFormState.characterId.trim(),
      name: normalizedFormState.name.trim(),
      personality: normalizedFormState.personality.trim(),
      schema_id: normalizedFormState.schemaId.trim(),
      style: normalizedFormState.style.trim(),
      system_prompt: normalizedFormState.systemPrompt.trim(),
      tags: parseTags(normalizedFormState.tagsText),
      folder: normalizeCharacterFolderRegistryName(normalizedFormState.folder),
    }

    setIsSubmitting(true)

    try {
      let summary: CharacterSummary
      let tone: NoticeTone = 'success'
      let coverUpdated = false
      let message: string

      if (mode === 'create') {
        const createdCharacter = await createCharacter(content)
        summary = createdCharacter.character_summary
        message = t('characters.feedback.created', { name: summary.name })
      } else {
        const updatedCharacter = await updateCharacter({
          characterId: initialCharacter!.character_id,
          content,
        })

        summary = createSummaryFromCharacter(updatedCharacter)
        message = t('characters.feedback.updated', { name: summary.name })
      }

      if (normalizedFormState.coverFile) {
        try {
          const updatedCover = await setCharacterCover({
            characterId: summary.character_id,
            coverFile: normalizedFormState.coverFile,
          })

          summary = withUpdatedCoverSummary(summary, updatedCover)
          coverUpdated = true
        } catch (error) {
          const coverWarning =
            mode === 'create'
              ? t('characters.feedback.createdWithCoverWarning', { name: summary.name })
              : t('characters.feedback.updatedWithCoverWarning', { name: summary.name })

          tone = 'warning'
          message = `${coverWarning} ${getErrorMessage(
            error,
            t('characters.feedback.coverAttachFailed'),
          )}`
        }
      }

      await onCompleted({
        characterId: summary.character_id,
        coverUpdated,
        message,
        summary,
        tone,
      })

      onOpenChange(false)
    } catch (error) {
      setSubmitError(getErrorMessage(error, t('characters.create.errors.submitFailed')))
    } finally {
      setIsSubmitting(false)
    }
  }

  const schemaOptions = availableSchemas.map((schema) => ({
    label: schema.display_name,
    value: schema.schema_id,
  }))
  const folderOptions = availableFolders.map((folder) => ({
    label: folder,
    value: folder,
  }))

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent
        aria-describedby={undefined}
        className="max-h-[92vh] overflow-hidden"
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
            {isEditMode ? t('characters.edit.title') : t('characters.create.title')}
          </DialogTitle>
        </DialogHeader>

        <DialogBody className="max-h-[calc(92vh-12rem)] overflow-y-auto pt-6">
          {isLoading ? (
            <LoadingSkeleton />
          ) : (
            <>
              <div className="grid gap-3 sm:grid-cols-3">
                {steps.map((step, index) => {
                  const isActive = step.key === currentStep
                  const isCompleted = index < currentStepIndex

                  return (
                    <button
                      className={cn(
                        'rounded-[1.35rem] border px-4 py-4 text-left transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)]',
                        isActive
                          ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)]'
                          : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] hover:border-[var(--color-accent-copper-soft)]',
                      )}
                      key={step.key}
                      onClick={() => {
                        if (index <= currentStepIndex) {
                          goToStep(index)
                        }
                      }}
                      type="button"
                    >
                      <div className="flex items-center justify-between gap-3">
                        <span
                          className={cn(
                            'text-sm font-medium',
                            isActive
                              ? 'text-[var(--color-text-primary)]'
                              : 'text-[var(--color-text-secondary)]',
                          )}
                        >
                          {step.label}
                        </span>
                        <span
                          className={cn(
                            'inline-flex h-7 w-7 items-center justify-center rounded-full border text-xs',
                            isCompleted || isActive
                              ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold)] text-[color:var(--color-accent-ink)]'
                              : 'border-[var(--color-border-subtle)] text-[var(--color-text-muted)]',
                          )}
                        >
                          {index + 1}
                        </span>
                      </div>
                    </button>
                  )
                })}
              </div>

              <div className="mt-6 space-y-5">
                {currentStep === 'identity' ? (
                  <div className="grid gap-4 md:grid-cols-2">
                    <Field
                      description={isEditMode ? t('characters.edit.characterIdHint') : undefined}
                      htmlFor={fieldIds.characterId}
                      label={t('characters.create.fields.characterId')}
                    >
                      <Input
                        autoFocus={!isEditMode}
                        id={fieldIds.characterId}
                        onChange={(event) => {
                          setFormState((currentFormState) => ({
                            ...currentFormState,
                            characterId: event.target.value,
                          }))
                        }}
                        placeholder={t('characters.create.placeholders.characterId')}
                        readOnly={isEditMode}
                        value={formState.characterId}
                      />
                    </Field>

                    <Field htmlFor={fieldIds.name} label={t('characters.create.fields.name')}>
                      <Input
                        autoFocus={isEditMode}
                        id={fieldIds.name}
                        onChange={(event) => {
                          setFormState((currentFormState) => ({
                            ...currentFormState,
                            name: event.target.value,
                          }))
                        }}
                        placeholder={t('characters.create.placeholders.name')}
                        value={formState.name}
                      />
                    </Field>

                    <div className="md:col-span-2">
                      <Field
                        description={
                          isEditMode && initialCharacter?.cover_file_name
                            ? t('characters.edit.coverHint', {
                                fileName: initialCharacter.cover_file_name,
                              })
                            : undefined
                        }
                        label={t('characters.create.fields.cover')}
                      >
                        <div className="flex flex-wrap items-center gap-3 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                          <label className="cursor-pointer">
                            <input
                              accept={characterCoverMimeTypes.join(',')}
                              className="sr-only"
                              onChange={(event) => {
                                const nextFile = event.target.files?.[0] ?? null

                                setFormState((currentFormState) => ({
                                  ...currentFormState,
                                  coverFile: nextFile,
                                }))
                              }}
                              type="file"
                            />
                            <span className="inline-flex h-11 items-center justify-center rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] px-5 text-sm text-[var(--color-text-primary)] transition hover:border-[var(--color-accent-copper-soft)]">
                              {formState.coverFile
                                ? t('characters.actions.replaceCover')
                                : t('characters.actions.chooseCover')}
                            </span>
                          </label>

                          {formState.coverFile ? (
                            <>
                              <div className="min-w-0 flex-1">
                                <p className="truncate text-sm text-[var(--color-text-primary)]">
                                  {formState.coverFile.name}
                                </p>
                                <p className="text-xs leading-6 text-[var(--color-text-muted)]">
                                  {formState.coverFile.type || 'application/octet-stream'}
                                </p>
                              </div>
                              <Button
                                className="px-4"
                                onClick={() => {
                                  setFormState((currentFormState) => ({
                                    ...currentFormState,
                                    coverFile: null,
                                  }))
                                }}
                                size="sm"
                                variant="ghost"
                              >
                                {t('characters.actions.clearCover')}
                              </Button>
                            </>
                          ) : (
                            <p className="text-sm leading-7 text-[var(--color-text-muted)]">
                              {t('characters.create.placeholders.cover')}
                            </p>
                          )}
                        </div>
                      </Field>
                    </div>

                    <Field
                      htmlFor={fieldIds.folder}
                      label={t('characters.create.fields.folder')}
                    >
                      <div className="space-y-3">
                        <div className="flex flex-wrap gap-2">
                          {(['existing', 'new'] as const).map((mode) => (
                            <button
                              className={cn(
                                'rounded-full border px-3 py-1.5 text-sm transition',
                                formState.folderMode === mode
                                  ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)]'
                                  : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]',
                              )}
                              key={mode}
                              onClick={() => {
                                setFormState((currentFormState) => ({
                                  ...currentFormState,
                                  folderMode: mode,
                                  folder:
                                    mode === 'existing' &&
                                    currentFormState.folder.trim().length > 0 &&
                                    !availableFolders.includes(currentFormState.folder.trim())
                                      ? ''
                                      : currentFormState.folder,
                                }))
                              }}
                              type="button"
                            >
                              {mode === 'existing'
                                ? t('characters.create.folderModes.existing')
                                : t('characters.create.folderModes.new')}
                            </button>
                          ))}
                        </div>

                        {formState.folderMode === 'existing' ? (
                          <Select
                            allowClear
                            clearLabel={t('characters.create.folderClear')}
                            items={folderOptions}
                            onValueChange={(value) => {
                              setFormState((currentFormState) => ({
                                ...currentFormState,
                                folder: value,
                              }))
                            }}
                            placeholder={t('characters.create.placeholders.folderSelect')}
                            textAlign="start"
                            triggerId={fieldIds.folder}
                            value={formState.folder}
                          />
                        ) : (
                          <Input
                            id={fieldIds.folder}
                            onChange={(event) => {
                              setFormState((currentFormState) => ({
                                ...currentFormState,
                                folder: event.target.value,
                              }))
                            }}
                            placeholder={t('characters.create.placeholders.folder')}
                            value={formState.folder}
                          />
                        )}
                      </div>
                    </Field>

                    <Field
                      description={t('characters.create.hints.tags')}
                      htmlFor={fieldIds.tags}
                      label={t('characters.create.fields.tags')}
                    >
                      <Textarea
                        className="min-h-24"
                        id={fieldIds.tags}
                        onChange={(event) => {
                          setFormState((currentFormState) => ({
                            ...currentFormState,
                            tagsText: event.target.value,
                          }))
                        }}
                        placeholder={t('characters.create.placeholders.tags')}
                        value={formState.tagsText}
                      />
                    </Field>
                  </div>
                ) : null}

                {currentStep === 'voice' ? (
                  <div className="space-y-4">
                    <div className="grid gap-4 md:grid-cols-2">
                      <Field
                        htmlFor={fieldIds.personality}
                        label={t('characters.create.fields.personality')}
                      >
                        <Textarea
                          className="min-h-32"
                          id={fieldIds.personality}
                          onChange={(event) => {
                            setFormState((currentFormState) => ({
                              ...currentFormState,
                              personality: event.target.value,
                            }))
                          }}
                          placeholder={t('characters.create.placeholders.personality')}
                          value={formState.personality}
                        />
                      </Field>

                      <Field htmlFor={fieldIds.style} label={t('characters.create.fields.style')}>
                        <Textarea
                          className="min-h-32"
                          id={fieldIds.style}
                          onChange={(event) => {
                            setFormState((currentFormState) => ({
                              ...currentFormState,
                              style: event.target.value,
                            }))
                          }}
                          placeholder={t('characters.create.placeholders.style')}
                          value={formState.style}
                        />
                      </Field>
                    </div>
                  </div>
                ) : null}

                {currentStep === 'system' ? (
                  <div className="space-y-5">
                    <Field
                      htmlFor={fieldIds.schemaId}
                      label={t('characters.create.fields.schemaId')}
                    >
                      {schemaOptions.length > 0 ? (
                        <Select
                          items={schemaOptions}
                          onValueChange={(value) => {
                            setFormState((currentFormState) => ({
                              ...currentFormState,
                              schemaId: value,
                            }))
                          }}
                          placeholder={t('characters.create.placeholders.schemaId')}
                          textAlign="start"
                          triggerId={fieldIds.schemaId}
                          value={formState.schemaId}
                        />
                      ) : (
                        <SchemaSelectionEmptyState
                          onClose={() => {
                            onOpenChange(false)
                          }}
                          onNavigate={() => {
                            setSubmitError(null)
                          }}
                        />
                      )}
                    </Field>

                    <Field
                      htmlFor={fieldIds.systemPrompt}
                      label={t('characters.create.fields.systemPrompt')}
                    >
                      <Textarea
                        className="min-h-36"
                        id={fieldIds.systemPrompt}
                        onChange={(event) => {
                          setFormState((currentFormState) => ({
                            ...currentFormState,
                            systemPrompt: event.target.value,
                          }))
                        }}
                        placeholder={t('characters.create.placeholders.systemPrompt')}
                        value={formState.systemPrompt}
                      />
                    </Field>
                  </div>
                ) : null}
              </div>
            </>
          )}
        </DialogBody>

        <DialogFooter>
          <div className="flex flex-wrap items-center justify-end gap-3">
            <DialogClose asChild>
              <Button disabled={isSubmitting} size="md" variant="ghost">
                {t('characters.actions.cancel')}
              </Button>
            </DialogClose>

            {currentStepIndex > 0 && !isLoading ? (
              <Button
                disabled={isSubmitting}
                onClick={() => {
                  goToStep(currentStepIndex - 1)
                }}
                size="md"
                variant="secondary"
              >
                {t('characters.actions.back')}
              </Button>
            ) : null}

            <Button
              disabled={isSubmitting || isLoading}
              onClick={() => {
                if (isLastStep) {
                  void handleSubmit()
                  return
                }

                handleNextStep()
              }}
              size="md"
            >
              {isSubmitting
                ? t('characters.actions.saving')
                : isLastStep
                  ? isEditMode
                    ? t('characters.actions.saveChanges')
                    : t('characters.actions.create')
                  : t('characters.actions.next')}
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
