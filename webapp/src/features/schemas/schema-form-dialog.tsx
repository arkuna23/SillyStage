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
import {
  stateValueTypes,
  type JsonValue,
  type StateFieldSchema,
  type StateValueType,
} from '../../lib/state-schema'
import { createSchema, getSchema, updateSchema } from './api'
import type { SchemaResource } from './types'

type SchemaFormDialogMode = 'create' | 'edit'

type SchemaFormDialogProps = {
  existingSchemaIds: ReadonlyArray<string>
  mode: SchemaFormDialogMode
  onCompleted: (result: { message: string; schema: SchemaResource }) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
  schemaId?: string | null
}

type FieldRow = {
  defaultValue: string
  description: string
  id: string
  key: string
  valueType: StateValueType
}

type FormState = {
  displayName: string
  schemaId: string
  tagDraft: string
  tags: string[]
  rows: FieldRow[]
}

function createRowId() {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID()
  }

  return `schema-row-${Date.now()}-${Math.random().toString(16).slice(2)}`
}

function createFieldRow(): FieldRow {
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
    displayName: '',
    schemaId: '',
    tagDraft: '',
    tags: [],
    rows: [],
  }
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function normalizeTags(formState: FormState) {
  const draft = formState.tagDraft.trim()

  if (draft.length === 0 || formState.tags.includes(draft)) {
    return {
      ...formState,
      tagDraft: '',
    }
  }

  return {
    ...formState,
    tagDraft: '',
    tags: [...formState.tags, draft],
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
    case 'bool':
      if (trimmedValue !== 'true' && trimmedValue !== 'false') {
        return { error: 'bool' }
      }

      return { hasValue: true, value: trimmedValue === 'true' }
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
    case 'array':
      try {
        const nextValue = parseJsonValue(trimmedValue)

        if (!Array.isArray(nextValue)) {
          return { error: 'array' }
        }

        return { hasValue: true, value: nextValue }
      } catch {
        return { error: 'array' }
      }
    case 'object':
      try {
        const nextValue = parseJsonValue(trimmedValue)

        if (nextValue === null || Array.isArray(nextValue) || typeof nextValue !== 'object') {
          return { error: 'object' }
        }

        return { hasValue: true, value: nextValue }
      } catch {
        return { error: 'object' }
      }
    default:
      return { hasValue: false }
  }
}

function serializeDefaultValue(valueType: StateValueType, value?: JsonValue) {
  if (value === undefined || valueType === 'null') {
    return ''
  }

  if (valueType === 'string' && typeof value === 'string') {
    return value
  }

  if (valueType === 'bool' && typeof value === 'boolean') {
    return value ? 'true' : 'false'
  }

  if ((valueType === 'int' || valueType === 'float') && typeof value === 'number') {
    return String(value)
  }

  return JSON.stringify(value)
}

function createFormStateFromSchema(schema: SchemaResource): FormState {
  return {
    displayName: schema.display_name,
    schemaId: schema.schema_id,
    tagDraft: '',
    tags: [...schema.tags],
    rows: Object.entries(schema.fields).map(([key, field]) => ({
      defaultValue: serializeDefaultValue(field.value_type, field.default),
      description: field.description ?? '',
      id: createRowId(),
      key,
      valueType: field.value_type,
    })),
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

function buildFields(
  rows: FieldRow[],
  messages: {
    duplicateFieldKey: string
    fieldKeyRequired: string
    invalidDefault: Record<'array' | 'bool' | 'float' | 'int' | 'object', string>
  },
): { fields: Record<string, StateFieldSchema> } | { error: string } {
  const fields: Record<string, StateFieldSchema> = {}
  const usedKeys = new Set<string>()

  for (const row of rows) {
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
      return { error: messages.fieldKeyRequired }
    }

    if (usedKeys.has(trimmedKey)) {
      return { error: messages.duplicateFieldKey }
    }

    usedKeys.add(trimmedKey)

    const parsedDefaultValue = parseDefaultValue(row.defaultValue, row.valueType)

    if ('error' in parsedDefaultValue) {
      return { error: messages.invalidDefault[parsedDefaultValue.error] }
    }

    fields[trimmedKey] = {
      ...(parsedDefaultValue.hasValue ? { default: parsedDefaultValue.value } : {}),
      ...(trimmedDescription.length > 0 ? { description: trimmedDescription } : {}),
      value_type: row.valueType,
    }
  }

  return { fields }
}

export function SchemaFormDialog({
  existingSchemaIds,
  mode,
  onCompleted,
  onOpenChange,
  open,
  schemaId,
}: SchemaFormDialogProps) {
  const { t } = useTranslation()
  const fieldErrorMessages = {
    duplicateFieldKey: String(t('schemas.form.errors.duplicateFieldKey')),
    fieldKeyRequired: String(t('schemas.form.errors.fieldKeyRequired')),
    invalidDefault: {
      array: String(t('schemas.form.errors.invalidDefault.array')),
      bool: String(t('schemas.form.errors.invalidDefault.bool')),
      float: String(t('schemas.form.errors.invalidDefault.float')),
      int: String(t('schemas.form.errors.invalidDefault.int')),
      object: String(t('schemas.form.errors.invalidDefault.object')),
    } as const satisfies Record<'array' | 'bool' | 'float' | 'int' | 'object', string>,
  }
  const fieldIdPrefix = useId()
  const [formState, setFormState] = useState<FormState>(createInitialFormState)
  const [initialSchema, setInitialSchema] = useState<SchemaResource | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  const isEditMode = mode === 'edit'

  useEffect(() => {
    if (!open) {
      setFormState(createInitialFormState())
      setInitialSchema(null)
      setIsLoading(false)
      setIsSubmitting(false)
      setSubmitError(null)
      return
    }

    if (mode === 'create') {
      setFormState(createInitialFormState())
      setInitialSchema(null)
      setIsLoading(false)
      setIsSubmitting(false)
      setSubmitError(null)
      return
    }

    if (!schemaId) {
      return
    }

    const controller = new AbortController()

    setIsLoading(true)
    setIsSubmitting(false)
    setSubmitError(null)

    void getSchema(schemaId, controller.signal)
      .then((schema) => {
        if (controller.signal.aborted) {
          return
        }

        setInitialSchema(schema)
        setFormState(createFormStateFromSchema(schema))
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setSubmitError(getErrorMessage(error, t('schemas.feedback.loadSchemaFailed')))
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
  }, [mode, open, schemaId, t])

  function validateForm(): string | null {
    const nextFormState = normalizeTags(formState)
    const trimmedSchemaId = nextFormState.schemaId.trim()
    const trimmedDisplayName = nextFormState.displayName.trim()

    setFormState(nextFormState)

    if (trimmedSchemaId.length === 0) {
      return t('schemas.form.errors.schemaIdRequired')
    }

    if (mode === 'create' && existingSchemaIds.includes(trimmedSchemaId)) {
      return t('schemas.form.errors.schemaIdDuplicate')
    }

    if (trimmedDisplayName.length === 0) {
      return t('schemas.form.errors.displayNameRequired')
    }

    const builtFields = buildFields(nextFormState.rows, fieldErrorMessages)

    if ('error' in builtFields) {
      return builtFields.error
    }

    return null
  }

  async function handleSubmit() {
    const validationError = validateForm()

    if (validationError) {
      setSubmitError(validationError)
      return
    }

    const nextFormState = normalizeTags(formState)
    const builtFields = buildFields(nextFormState.rows, fieldErrorMessages)

    if ('error' in builtFields) {
      setSubmitError(builtFields.error)
      return
    }

    setFormState(nextFormState)
    setIsSubmitting(true)
    setSubmitError(null)

    try {
      const trimmedSchemaId = nextFormState.schemaId.trim()
      const trimmedDisplayName = nextFormState.displayName.trim()
      const nextTags = nextFormState.tags.map((tag) => tag.trim()).filter(Boolean)

      if (mode === 'edit' && !initialSchema) {
        setSubmitError(t('schemas.feedback.loadSchemaFailed'))
        return
      }

      const result =
        mode === 'create'
          ? await createSchema({
              display_name: trimmedDisplayName,
              fields: builtFields.fields,
              schema_id: trimmedSchemaId,
              tags: nextTags,
            })
          : await updateSchema({
              ...(initialSchema && trimmedDisplayName !== initialSchema.display_name
                ? { display_name: trimmedDisplayName }
                : {}),
              ...(initialSchema && JSON.stringify(builtFields.fields) !== JSON.stringify(initialSchema.fields)
                ? { fields: builtFields.fields }
                : {}),
              schema_id: trimmedSchemaId,
              ...(initialSchema && JSON.stringify(nextTags) !== JSON.stringify(initialSchema.tags)
                ? { tags: nextTags }
                : {}),
            })

      await onCompleted({
        message:
          mode === 'create'
            ? t('schemas.feedback.created', { name: result.display_name })
            : t('schemas.feedback.updated', { name: result.display_name }),
        schema: result,
      })

      onOpenChange(false)
    } catch (error) {
      setSubmitError(getErrorMessage(error, t('schemas.form.errors.submitFailed')))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="max-h-[92vh] overflow-hidden">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>
            {isEditMode ? t('schemas.form.editTitle') : t('schemas.form.createTitle')}
          </DialogTitle>
        </DialogHeader>

        <DialogBody className="max-h-[calc(92vh-12rem)] overflow-y-auto pt-6">
          <div className="space-y-5">
            {submitError ? (
              <div className="rounded-[1.25rem] border border-[var(--color-state-error-line)] bg-[var(--color-state-error-soft)] px-4 py-3 text-sm text-[var(--color-text-primary)]">
                {submitError}
              </div>
            ) : null}

            {isLoading ? (
              <div className="space-y-4">
                {Array.from({ length: 4 }).map((_, index) => (
                  <div className="space-y-2.5" key={index}>
                    <div className="h-3 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
                    <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-elevated)]" />
                  </div>
                ))}
              </div>
            ) : (
              <>
                <div className="grid gap-4 md:grid-cols-2">
                  <Field
                    description={isEditMode ? t('schemas.form.fields.schemaIdHint') : undefined}
                    htmlFor={`${fieldIdPrefix}-schema-id`}
                    label={t('schemas.form.fields.schemaId')}
                  >
                    <Input
                      id={`${fieldIdPrefix}-schema-id`}
                      onChange={(event) => {
                        setFormState((current) => ({ ...current, schemaId: event.target.value }))
                      }}
                      placeholder={t('schemas.form.placeholders.schemaId')}
                      readOnly={isEditMode}
                      value={formState.schemaId}
                    />
                  </Field>

                  <Field
                    htmlFor={`${fieldIdPrefix}-display-name`}
                    label={t('schemas.form.fields.displayName')}
                  >
                    <Input
                      id={`${fieldIdPrefix}-display-name`}
                      onChange={(event) => {
                        setFormState((current) => ({ ...current, displayName: event.target.value }))
                      }}
                      placeholder={t('schemas.form.placeholders.displayName')}
                      value={formState.displayName}
                    />
                  </Field>
                </div>

                <Field label={t('schemas.form.fields.tags')}>
                  <div className="space-y-3 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4">
                    <div className="flex flex-wrap gap-2">
                      {formState.tags.map((tag) => (
                        <span
                          className="inline-flex items-center gap-2 rounded-full border border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] px-3 py-1.5 text-xs text-[var(--color-text-primary)]"
                          key={tag}
                        >
                          {tag}
                          <button
                            className="text-[var(--color-text-secondary)] transition hover:text-[var(--color-text-primary)]"
                            onClick={() => {
                              setFormState((current) => ({
                                ...current,
                                tags: current.tags.filter((currentTag) => currentTag !== tag),
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
                        id={`${fieldIdPrefix}-tag-draft`}
                        onChange={(event) => {
                          setFormState((current) => ({ ...current, tagDraft: event.target.value }))
                        }}
                        onKeyDown={(event) => {
                          if (event.key === 'Enter') {
                            event.preventDefault()
                            setFormState((current) => normalizeTags(current))
                          }
                        }}
                        placeholder={t('schemas.form.placeholders.tag')}
                        value={formState.tagDraft}
                      />
                      <Button
                        className="sm:shrink-0"
                        onClick={() => {
                          setFormState((current) => normalizeTags(current))
                        }}
                        size="md"
                        variant="secondary"
                      >
                        {t('schemas.actions.addTag')}
                      </Button>
                    </div>
                  </div>
                </Field>

                <div className="space-y-3">
                  <div className="flex items-center justify-between gap-3">
                    <p className="text-sm font-medium text-[var(--color-text-primary)]">
                      {t('schemas.form.fields.fields')}
                    </p>
                    <Button
                      onClick={() => {
                        setFormState((current) => ({
                          ...current,
                          rows: [...current.rows, createFieldRow()],
                        }))
                      }}
                      size="sm"
                      variant="secondary"
                    >
                      {t('schemas.actions.addField')}
                    </Button>
                  </div>

                  <div className="space-y-3">
                    {formState.rows.length === 0 ? (
                      <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-5 text-sm leading-7 text-[var(--color-text-secondary)]">
                        {t('schemas.form.emptyFields')}
                      </div>
                    ) : null}

                    {formState.rows.map((row) => (
                      <div
                        className="rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4"
                        key={row.id}
                      >
                        <div className="grid gap-3 md:grid-cols-[minmax(0,1.15fr)_13rem]">
                          <Field
                            htmlFor={`${fieldIdPrefix}-${row.id}-field-key`}
                            label={t('schemas.form.fields.fieldKey')}
                          >
                            <Input
                              id={`${fieldIdPrefix}-${row.id}-field-key`}
                              onChange={(event) => {
                                setFormState((current) => ({
                                  ...current,
                                  rows: current.rows.map((fieldRow) =>
                                    fieldRow.id === row.id
                                      ? { ...fieldRow, key: event.target.value }
                                      : fieldRow,
                                  ),
                                }))
                              }}
                              placeholder={t('schemas.form.placeholders.fieldKey')}
                              value={row.key}
                            />
                          </Field>

                          <Field
                            htmlFor={`${fieldIdPrefix}-${row.id}-field-type`}
                            label={t('schemas.form.fields.fieldType')}
                          >
                            <Select
                              items={stateValueTypes.map((valueType) => ({
                                label: t(`schemas.form.valueTypes.${valueType}` as const),
                                value: valueType,
                              }))}
                              onValueChange={(value) => {
                                setFormState((current) => ({
                                  ...current,
                                  rows: current.rows.map((fieldRow) =>
                                    fieldRow.id === row.id
                                      ? { ...fieldRow, valueType: value as StateValueType }
                                      : fieldRow,
                                  ),
                                }))
                              }}
                              triggerId={`${fieldIdPrefix}-${row.id}-field-type`}
                              value={row.valueType}
                            />
                          </Field>
                        </div>

                        <div className="mt-3 grid gap-3 md:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
                          <Field
                            htmlFor={`${fieldIdPrefix}-${row.id}-field-default`}
                            label={t('schemas.form.fields.fieldDefault')}
                          >
                            <Input
                              id={`${fieldIdPrefix}-${row.id}-field-default`}
                              onChange={(event) => {
                                setFormState((current) => ({
                                  ...current,
                                  rows: current.rows.map((fieldRow) =>
                                    fieldRow.id === row.id
                                      ? { ...fieldRow, defaultValue: event.target.value }
                                      : fieldRow,
                                  ),
                                }))
                              }}
                              placeholder={t('schemas.form.placeholders.fieldDefault')}
                              value={row.defaultValue}
                            />
                          </Field>

                          <Field
                            htmlFor={`${fieldIdPrefix}-${row.id}-field-description`}
                            label={t('schemas.form.fields.fieldDescription')}
                          >
                            <Input
                              id={`${fieldIdPrefix}-${row.id}-field-description`}
                              onChange={(event) => {
                                setFormState((current) => ({
                                  ...current,
                                  rows: current.rows.map((fieldRow) =>
                                    fieldRow.id === row.id
                                      ? { ...fieldRow, description: event.target.value }
                                      : fieldRow,
                                  ),
                                }))
                              }}
                              placeholder={t('schemas.form.placeholders.fieldDescription')}
                              value={row.description}
                            />
                          </Field>
                        </div>

                        <div className="mt-3 flex justify-end">
                          <Button
                            onClick={() => {
                              setFormState((current) => ({
                                ...current,
                                rows: current.rows.filter((fieldRow) => fieldRow.id !== row.id),
                              }))
                            }}
                            size="sm"
                            variant="ghost"
                          >
                            {t('schemas.actions.removeField')}
                          </Button>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </>
            )}
          </div>
        </DialogBody>

        <DialogFooter>
          <DialogClose asChild>
            <Button disabled={isSubmitting} variant="ghost">
              {t('schemas.actions.cancel')}
            </Button>
          </DialogClose>

          <Button disabled={isLoading || isSubmitting} onClick={() => void handleSubmit()}>
            {isSubmitting ? t('schemas.actions.saving') : t('schemas.actions.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
