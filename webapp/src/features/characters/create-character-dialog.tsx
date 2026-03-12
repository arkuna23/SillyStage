import { useEffect, useId, useState } from 'react'
import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'

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
import { createCharacter, setCharacterCover, withUpdatedCoverSummary } from './api'
import {
  characterCoverMimeTypes,
  stateValueTypes,
  type CharacterCardContent,
  type CharacterSummary,
  type JsonValue,
  type StateFieldSchema,
  type StateValueType,
} from './types'

type NoticeTone = 'success' | 'warning'

type CreateCharacterDialogResult = {
  characterId: string
  message: string
  summary: CharacterSummary
  tone: NoticeTone
}

type CreateCharacterDialogProps = {
  onCompleted: (result: CreateCharacterDialogResult) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
}

type WizardStep = 'identity' | 'voice' | 'system'

type StateSchemaRow = {
  defaultValue: string
  description: string
  id: string
  key: string
  valueType: StateValueType
}

type FormState = {
  characterId: string
  coverFile: File | null
  name: string
  personality: string
  stateRows: StateSchemaRow[]
  style: string
  systemPrompt: string
  tendencyDraft: string
  tendencies: string[]
}

type StateSchemaBuildResult =
  | { error: string }
  | { stateSchema: Record<string, StateFieldSchema> }

function createRowId() {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID()
  }

  return `row-${Date.now()}-${Math.random().toString(16).slice(2)}`
}

function createStateRow(): StateSchemaRow {
  return {
    defaultValue: '',
    description: '',
    id: createRowId(),
    key: '',
    valueType: 'string',
  }
}

function createInitialFormState(): FormState {
  return {
    characterId: '',
    coverFile: null,
    name: '',
    personality: '',
    stateRows: [],
    style: '',
    systemPrompt: '',
    tendencyDraft: '',
    tendencies: [],
  }
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function normalizeTendencies(formState: FormState) {
  const draft = formState.tendencyDraft.trim()

  if (draft.length === 0 || formState.tendencies.includes(draft)) {
    return {
      ...formState,
      tendencyDraft: '',
    }
  }

  return {
    ...formState,
    tendencies: [...formState.tendencies, draft],
    tendencyDraft: '',
  }
}

function parseJsonValue(rawValue: string) {
  return JSON.parse(rawValue) as JsonValue
}

function parseDefaultValue(
  rawValue: string,
  valueType: StateValueType,
):
  | { hasValue: boolean; value?: JsonValue }
  | { error: 'array' | 'bool' | 'float' | 'int' | 'object' } {
  const trimmedValue = rawValue.trim()

  if (valueType === 'null') {
    return { hasValue: true, value: null }
  }

  if (trimmedValue.length === 0) {
    return { hasValue: false }
  }

  switch (valueType) {
    case 'bool': {
      if (trimmedValue !== 'true' && trimmedValue !== 'false') {
        return { error: 'bool' }
      }

      return { hasValue: true, value: trimmedValue === 'true' }
    }
    case 'int': {
      const nextValue = Number(trimmedValue)

      if (!Number.isInteger(nextValue)) {
        return { error: 'int' }
      }

      return { hasValue: true, value: nextValue }
    }
    case 'float': {
      const nextValue = Number(trimmedValue)

      if (!Number.isFinite(nextValue)) {
        return { error: 'float' }
      }

      return { hasValue: true, value: nextValue }
    }
    case 'string':
      return { hasValue: true, value: rawValue }
    case 'array': {
      try {
        const nextValue = parseJsonValue(trimmedValue)

        if (!Array.isArray(nextValue)) {
          return { error: 'array' }
        }

        return { hasValue: true, value: nextValue }
      } catch {
        return { error: 'array' }
      }
    }
    case 'object': {
      try {
        const nextValue = parseJsonValue(trimmedValue)

        if (
          nextValue === null ||
          Array.isArray(nextValue) ||
          typeof nextValue !== 'object'
        ) {
          return { error: 'object' }
        }

        return { hasValue: true, value: nextValue }
      } catch {
        return { error: 'object' }
      }
    }
    default:
      return { hasValue: false }
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

export function CreateCharacterDialog({
  onCompleted,
  onOpenChange,
  open,
}: CreateCharacterDialogProps) {
  const { t } = useTranslation()
  const fieldIdPrefix = useId()
  const [currentStep, setCurrentStep] = useState<WizardStep>('identity')
  const [formState, setFormState] = useState<FormState>(createInitialFormState)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)

  const fieldIds = {
    characterId: `${fieldIdPrefix}-character-id`,
    name: `${fieldIdPrefix}-display-name`,
    personality: `${fieldIdPrefix}-personality`,
    style: `${fieldIdPrefix}-style`,
    systemPrompt: `${fieldIdPrefix}-system-prompt`,
    tendencyDraft: `${fieldIdPrefix}-tendency-draft`,
  } as const

  const steps = [
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
  ]

  const currentStepIndex = steps.findIndex((step) => step.key === currentStep)
  const isLastStep = currentStepIndex === steps.length - 1

  useEffect(() => {
    if (!open) {
      setCurrentStep('identity')
      setFormState(createInitialFormState())
      setIsSubmitting(false)
      setSubmitError(null)
    }
  }, [open])

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

  function buildStateSchema(nextFormState: FormState): StateSchemaBuildResult {
    const stateSchema: Record<string, StateFieldSchema> = {}
    const usedKeys = new Set<string>()

    for (const row of nextFormState.stateRows) {
      const trimmedKey = row.key.trim()
      const trimmedDescription = row.description.trim()
      const isBlankRow =
        trimmedKey.length === 0 &&
        trimmedDescription.length === 0 &&
        row.defaultValue.trim().length === 0

      if (isBlankRow) {
        continue
      }

      if (trimmedKey.length === 0) {
        return { error: t('characters.create.errors.stateKeyRequired') }
      }

      if (usedKeys.has(trimmedKey)) {
        return { error: t('characters.create.errors.duplicateStateKey') }
      }

      usedKeys.add(trimmedKey)

      const parsedDefaultValue = parseDefaultValue(row.defaultValue, row.valueType)

      if ('error' in parsedDefaultValue) {
        const errorKey =
          parsedDefaultValue.error === 'bool'
            ? 'characters.create.errors.invalidDefault.bool'
            : parsedDefaultValue.error === 'int'
              ? 'characters.create.errors.invalidDefault.int'
              : parsedDefaultValue.error === 'float'
                ? 'characters.create.errors.invalidDefault.float'
                : parsedDefaultValue.error === 'array'
                  ? 'characters.create.errors.invalidDefault.array'
                  : 'characters.create.errors.invalidDefault.object'

        return {
          error: t(errorKey),
        }
      }

      stateSchema[trimmedKey] = {
        ...(parsedDefaultValue.hasValue
          ? { default: parsedDefaultValue.value }
          : {}),
        ...(trimmedDescription.length > 0 ? { description: trimmedDescription } : {}),
        value_type: row.valueType,
      }
    }

    return { stateSchema }
  }

  function goToStep(nextStepIndex: number) {
    const nextStep = steps[nextStepIndex]

    if (!nextStep) {
      return
    }

    setCurrentStep(nextStep.key)
    setSubmitError(null)
  }

  function handleAddTendency() {
    setFormState((currentFormState) => normalizeTendencies(currentFormState))
  }

  function handleNextStep() {
    const normalizedFormState = normalizeTendencies(formState)

    setFormState(normalizedFormState)
    setSubmitError(null)

    const nextError =
      currentStep === 'identity'
        ? validateIdentityStep(normalizedFormState)
        : validateVoiceStep(normalizedFormState)

    if (nextError) {
      setSubmitError(nextError)
      return
    }

    goToStep(currentStepIndex + 1)
  }

  async function handleSubmit() {
    const normalizedFormState = normalizeTendencies(formState)
    const identityError = validateIdentityStep(normalizedFormState)
    const voiceError = validateVoiceStep(normalizedFormState)
    const builtStateSchema = buildStateSchema(normalizedFormState)

    setFormState(normalizedFormState)
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

    if ('error' in builtStateSchema) {
      setSubmitError(builtStateSchema.error)
      setCurrentStep('system')
      return
    }

    const content: CharacterCardContent = {
      id: normalizedFormState.characterId.trim(),
      name: normalizedFormState.name.trim(),
      personality: normalizedFormState.personality.trim(),
      state_schema: builtStateSchema.stateSchema,
      style: normalizedFormState.style.trim(),
      system_prompt: normalizedFormState.systemPrompt.trim(),
      tendencies: normalizedFormState.tendencies,
    }

    setIsSubmitting(true)

    try {
      const created = await createCharacter(content)
      let summary = created.character_summary
      let tone: NoticeTone = 'success'
      let message: string = t('characters.feedback.created', {
        name: created.character_summary.name,
      })

      if (normalizedFormState.coverFile) {
        try {
          const updatedCover = await setCharacterCover({
            characterId: created.character_id,
            coverFile: normalizedFormState.coverFile,
          })

          summary = withUpdatedCoverSummary(summary, updatedCover)
        } catch (error) {
          tone = 'warning'
          message = `${t('characters.feedback.createdWithCoverWarning', {
            name: created.character_summary.name,
          })} ${getErrorMessage(
            error,
            t('characters.feedback.coverAttachFailed'),
          )}`
        }
      }

      await onCompleted({
        characterId: created.character_id,
        message,
        summary,
        tone,
      })

      onOpenChange(false)
    } catch (error) {
      setSubmitError(
        getErrorMessage(error, t('characters.create.errors.submitFailed')),
      )
    } finally {
      setIsSubmitting(false)
    }
  }

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
          <DialogTitle>{t('characters.create.title')}</DialogTitle>
        </DialogHeader>

        <DialogBody className="max-h-[calc(92vh-12rem)] overflow-y-auto pt-6">
          <div className="grid gap-3 sm:grid-cols-3">
            {steps.map((step, index) => {
              const isActive = step.key === currentStep
              const isCompleted = index < currentStepIndex

              return (
                <button
                  className={cn(
                    'rounded-[1.35rem] border px-4 py-4 text-left transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-amber-200/70',
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
                          ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold)] text-[var(--color-accent-ink)]'
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
                  htmlFor={fieldIds.characterId}
                  label={t('characters.create.fields.characterId')}
                >
                  <Input
                    autoFocus
                    id={fieldIds.characterId}
                    onChange={(event) => {
                      setFormState((currentFormState) => ({
                        ...currentFormState,
                        characterId: event.target.value,
                      }))
                    }}
                    placeholder={t('characters.create.placeholders.characterId')}
                    value={formState.characterId}
                  />
                </Field>

                <Field htmlFor={fieldIds.name} label={t('characters.create.fields.name')}>
                  <Input
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
                  <Field label={t('characters.create.fields.cover')}>
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

                <Field label={t('characters.create.fields.tendencies')}>
                  <div className="space-y-3 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4">
                    <div className="flex flex-wrap gap-2">
                      {formState.tendencies.map((tendency) => (
                        <span
                          className="inline-flex items-center gap-2 rounded-full border border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] px-3 py-1.5 text-xs text-[var(--color-text-primary)]"
                          key={tendency}
                        >
                          {tendency}
                          <button
                            className="text-[var(--color-text-secondary)] transition hover:text-[var(--color-text-primary)]"
                            onClick={() => {
                              setFormState((currentFormState) => ({
                                ...currentFormState,
                                tendencies: currentFormState.tendencies.filter(
                                  (item) => item !== tendency,
                                ),
                              }))
                            }}
                            type="button"
                          >
                            ×
                          </button>
                        </span>
                      ))}
                    </div>

                    <div className="flex flex-col gap-3 sm:flex-row">
                      <Input
                        id={fieldIds.tendencyDraft}
                        onChange={(event) => {
                          setFormState((currentFormState) => ({
                            ...currentFormState,
                            tendencyDraft: event.target.value,
                          }))
                        }}
                        onKeyDown={(event) => {
                          if (event.key === 'Enter') {
                            event.preventDefault()
                            handleAddTendency()
                          }
                        }}
                        placeholder={t('characters.create.placeholders.tendency')}
                        value={formState.tendencyDraft}
                      />
                      <Button
                        className="sm:shrink-0"
                        onClick={handleAddTendency}
                        size="md"
                        variant="secondary"
                      >
                        {t('characters.actions.addTendency')}
                      </Button>
                    </div>
                  </div>
                </Field>
              </div>
            ) : null}

            {currentStep === 'system' ? (
              <div className="space-y-5">
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

                <div className="space-y-3">
                  <div className="flex items-center justify-between gap-3">
                    <p className="text-sm font-medium text-[var(--color-text-primary)]">
                      {t('characters.create.fields.stateSchema')}
                    </p>
                    <Button
                      onClick={() => {
                        setFormState((currentFormState) => ({
                          ...currentFormState,
                          stateRows: [...currentFormState.stateRows, createStateRow()],
                        }))
                      }}
                      size="sm"
                      variant="secondary"
                    >
                      {t('characters.actions.addStateField')}
                    </Button>
                  </div>

                  <div className="space-y-3">
                    {formState.stateRows.map((row) => (
                      <div
                        className="rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4"
                        key={row.id}
                      >
                        <div className="grid gap-3 md:grid-cols-[minmax(0,1.15fr)_13rem]">
                          <Field
                            htmlFor={`${fieldIdPrefix}-${row.id}-state-key`}
                            label={t('characters.create.fields.stateKey')}
                          >
                            <Input
                              id={`${fieldIdPrefix}-${row.id}-state-key`}
                              onChange={(event) => {
                                setFormState((currentFormState) => ({
                                  ...currentFormState,
                                  stateRows: currentFormState.stateRows.map((stateRow) =>
                                    stateRow.id === row.id
                                      ? { ...stateRow, key: event.target.value }
                                      : stateRow,
                                  ),
                                }))
                              }}
                              placeholder={t('characters.create.placeholders.stateKey')}
                              value={row.key}
                            />
                          </Field>

                          <Field
                            htmlFor={`${fieldIdPrefix}-${row.id}-state-type`}
                            label={t('characters.create.fields.stateType')}
                          >
                            <Select
                              items={stateValueTypes.map((valueType) => ({
                                label: t(
                                  `characters.create.stateTypes.${valueType}` as const,
                                ),
                                value: valueType,
                              }))}
                              onValueChange={(value) => {
                                setFormState((currentFormState) => ({
                                  ...currentFormState,
                                  stateRows: currentFormState.stateRows.map((stateRow) =>
                                    stateRow.id === row.id
                                      ? {
                                          ...stateRow,
                                          valueType: value as StateValueType,
                                        }
                                      : stateRow,
                                  ),
                                }))
                              }}
                              triggerId={`${fieldIdPrefix}-${row.id}-state-type`}
                              value={row.valueType}
                            />
                          </Field>
                        </div>

                        <div className="mt-3 grid gap-3 md:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
                          <Field
                            htmlFor={`${fieldIdPrefix}-${row.id}-state-default`}
                            label={t('characters.create.fields.stateDefault')}
                          >
                            <Input
                              id={`${fieldIdPrefix}-${row.id}-state-default`}
                              onChange={(event) => {
                                setFormState((currentFormState) => ({
                                  ...currentFormState,
                                  stateRows: currentFormState.stateRows.map((stateRow) =>
                                    stateRow.id === row.id
                                      ? {
                                          ...stateRow,
                                          defaultValue: event.target.value,
                                        }
                                      : stateRow,
                                  ),
                                }))
                              }}
                              placeholder={t('characters.create.placeholders.stateDefault')}
                              value={row.defaultValue}
                            />
                          </Field>

                          <Field
                            htmlFor={`${fieldIdPrefix}-${row.id}-state-description`}
                            label={t('characters.create.fields.stateDescription')}
                          >
                            <Input
                              id={`${fieldIdPrefix}-${row.id}-state-description`}
                              onChange={(event) => {
                                setFormState((currentFormState) => ({
                                  ...currentFormState,
                                  stateRows: currentFormState.stateRows.map((stateRow) =>
                                    stateRow.id === row.id
                                      ? {
                                          ...stateRow,
                                          description: event.target.value,
                                        }
                                      : stateRow,
                                  ),
                                }))
                              }}
                              placeholder={t('characters.create.placeholders.stateDescription')}
                              value={row.description}
                            />
                          </Field>
                        </div>

                        <div className="mt-3 flex justify-end">
                          <Button
                            onClick={() => {
                              setFormState((currentFormState) => ({
                                ...currentFormState,
                                stateRows: currentFormState.stateRows.filter(
                                  (stateRow) => stateRow.id !== row.id,
                                ),
                              }))
                            }}
                            size="sm"
                            variant="ghost"
                          >
                            {t('characters.actions.removeStateField')}
                          </Button>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            ) : null}
          </div>
        </DialogBody>

        <DialogFooter>
          <div className="flex flex-1 items-center text-sm text-rose-300">
            {submitError ? <p>{submitError}</p> : null}
          </div>

          <div className="flex flex-wrap items-center justify-end gap-3">
            <DialogClose asChild>
              <Button disabled={isSubmitting} size="md" variant="ghost">
                {t('characters.actions.cancel')}
              </Button>
            </DialogClose>

            {currentStepIndex > 0 ? (
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
              disabled={isSubmitting}
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
                  ? t('characters.actions.create')
                  : t('characters.actions.next')}
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
