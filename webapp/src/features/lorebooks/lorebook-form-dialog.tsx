import { faChevronDown } from '@fortawesome/free-solid-svg-icons/faChevronDown'
import { faChevronUp } from '@fortawesome/free-solid-svg-icons/faChevronUp'
import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { faTags } from '@fortawesome/free-solid-svg-icons/faTags'
import { faThumbtack } from '@fortawesome/free-solid-svg-icons/faThumbtack'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { AnimatePresence, motion, useReducedMotion } from 'framer-motion'
import type { ReactNode } from 'react'
import { useEffect, useId, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import {
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../components/ui/dialog'
import { IconButton } from '../../components/ui/icon-button'
import { Input } from '../../components/ui/input'
import { Switch } from '../../components/ui/switch'
import { Textarea } from '../../components/ui/textarea'
import { useToastMessage } from '../../components/ui/toast-context'
import { cn } from '../../lib/cn'
import {
  createLorebook,
  createLorebookEntry,
  deleteLorebookEntry,
  getLorebook,
  updateLorebook,
  updateLorebookEntry,
} from './api'
import type { Lorebook, LorebookEntry } from './types'

type LorebookFormDialogMode = 'create' | 'edit'

type LorebookFormDialogProps = {
  existingLorebookIds: ReadonlyArray<string>
  lorebookId?: string | null
  mode: LorebookFormDialogMode
  onCompleted: (result: { lorebook: Lorebook; message: string }) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
}

type LorebookEntryFormState = {
  alwaysInclude: boolean
  clientId: string
  content: string
  enabled: boolean
  entryId: string
  keywords: string
  title: string
}

type FormState = {
  displayName: string
  entries: LorebookEntryFormState[]
  lorebookId: string
}

function createInitialEntry(): LorebookEntryFormState {
  return {
    alwaysInclude: false,
    clientId: createClientId(),
    content: '',
    enabled: true,
    entryId: '',
    keywords: '',
    title: '',
  }
}

function createClientId() {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID()
  }

  return `entry-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`
}

function createInitialFormState(): FormState {
  return {
    displayName: '',
    entries: [],
    lorebookId: '',
  }
}

function createEntryFormState(entry: LorebookEntry): LorebookEntryFormState {
  return {
    alwaysInclude: entry.always_include,
    clientId: createClientId(),
    content: entry.content,
    enabled: entry.enabled,
    entryId: entry.entry_id,
    keywords: entry.keywords.join(', '),
    title: entry.title,
  }
}

function createFormStateFromLorebook(lorebook: Lorebook): FormState {
  return {
    displayName: lorebook.display_name,
    entries: lorebook.entries.map(createEntryFormState),
    lorebookId: lorebook.lorebook_id,
  }
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function normalizeKeywords(value: string) {
  return Array.from(
    new Set(
      value
        .split(/[,\n，]/g)
        .map((keyword) => keyword.trim())
        .filter((keyword) => keyword.length > 0),
    ),
  )
}

function normalizeEntry(entry: LorebookEntryFormState): LorebookEntry {
  return {
    always_include: entry.alwaysInclude,
    content: entry.content.trim(),
    enabled: entry.enabled,
    entry_id: entry.entryId.trim(),
    keywords: normalizeKeywords(entry.keywords),
    title: entry.title.trim(),
  }
}

function areEntriesEqual(left: LorebookEntry, right: LorebookEntry) {
  if (
    left.entry_id !== right.entry_id ||
    left.title !== right.title ||
    left.content !== right.content ||
    left.enabled !== right.enabled ||
    left.always_include !== right.always_include ||
    left.keywords.length !== right.keywords.length
  ) {
    return false
  }

  return left.keywords.every((keyword, index) => keyword === right.keywords[index])
}

function LoadingSkeleton() {
  return (
    <div className="space-y-5">
      <div className="grid gap-4 md:grid-cols-2">
        <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-elevated)]" />
        <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-elevated)]" />
      </div>
      {Array.from({ length: 2 }).map((_, index) => (
        <div
          className="rounded-[1.6rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-5"
          key={index}
        >
          <div className="grid gap-4 md:grid-cols-2">
            <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-panel)]" />
            <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-panel)]" />
          </div>
          <div className="mt-4 h-28 animate-pulse rounded-[1.4rem] bg-[var(--color-bg-panel)]" />
        </div>
      ))}
    </div>
  )
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

export function LorebookFormDialog({
  existingLorebookIds,
  lorebookId,
  mode,
  onCompleted,
  onOpenChange,
  open,
}: LorebookFormDialogProps) {
  const { t } = useTranslation()
  const prefersReducedMotion = useReducedMotion()
  const fieldIdPrefix = useId()
  const [formState, setFormState] = useState<FormState>(createInitialFormState)
  const [initialLorebook, setInitialLorebook] = useState<Lorebook | null>(null)
  const [expandedEntryIds, setExpandedEntryIds] = useState<string[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  useToastMessage(submitError)

  const isEditMode = mode === 'edit'
  const normalizedEntries = useMemo(
    () => formState.entries.map(normalizeEntry),
    [formState.entries],
  )

  const fieldIds = {
    displayName: `${fieldIdPrefix}-display-name`,
    lorebookId: `${fieldIdPrefix}-lorebook-id`,
  } as const

  useEffect(() => {
    if (!open) {
      setFormState(createInitialFormState())
      setInitialLorebook(null)
      setExpandedEntryIds([])
      setIsLoading(false)
      setIsSubmitting(false)
      setSubmitError(null)
      return
    }

    if (mode === 'create') {
      setFormState(createInitialFormState())
      setInitialLorebook(null)
      setExpandedEntryIds([])
      setIsLoading(false)
      setIsSubmitting(false)
      setSubmitError(null)
      return
    }

    if (!lorebookId) {
      return
    }

    const controller = new AbortController()
    setIsLoading(true)
    setIsSubmitting(false)
    setSubmitError(null)

    void getLorebook(lorebookId, controller.signal)
      .then((lorebook) => {
        if (controller.signal.aborted) {
          return
        }

        setInitialLorebook(lorebook)
        setFormState(createFormStateFromLorebook(lorebook))
        setExpandedEntryIds([])
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setSubmitError(getErrorMessage(error, t('lorebooks.feedback.loadLorebookFailed')))
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
  }, [lorebookId, mode, open, t])

  function updateEntry(index: number, update: Partial<LorebookEntryFormState>) {
    setFormState((current) => ({
      ...current,
      entries: current.entries.map((entry, entryIndex) =>
        entryIndex === index ? { ...entry, ...update } : entry,
      ),
    }))
  }

  function appendEntry() {
    const nextEntry = createInitialEntry()

    setFormState((current) => ({
      ...current,
      entries: [...current.entries, nextEntry],
    }))
    setExpandedEntryIds((current) =>
      current.includes(nextEntry.clientId) ? current : [...current, nextEntry.clientId],
    )
  }

  function removeEntry(index: number) {
    setFormState((current) => {
      const removedEntry = current.entries[index]
      const nextEntries = current.entries.filter((_, entryIndex) => entryIndex !== index)

      if (removedEntry) {
        setExpandedEntryIds((currentExpandedIds) =>
          currentExpandedIds.filter((entryId) => entryId !== removedEntry.clientId),
        )
      }

      return {
        ...current,
        entries: nextEntries,
      }
    })
  }

  function toggleEntryExpanded(clientId: string) {
    setExpandedEntryIds((current) =>
      current.includes(clientId)
        ? current.filter((entryId) => entryId !== clientId)
        : [...current, clientId],
    )
  }

  function validateForm() {
    const nextLorebookId = formState.lorebookId.trim()

    if (nextLorebookId.length === 0) {
      return t('lorebooks.form.errors.lorebookIdRequired')
    }

    if (!isEditMode && existingLorebookIds.some((existingId) => existingId === nextLorebookId)) {
      return t('lorebooks.form.errors.lorebookIdDuplicate')
    }

    if (formState.displayName.trim().length === 0) {
      return t('lorebooks.form.errors.displayNameRequired')
    }

    const seenEntryIds = new Set<string>()
    for (const entry of normalizedEntries) {
      if (entry.entry_id.length === 0) {
        return t('lorebooks.form.errors.entryIdRequired')
      }

      if (seenEntryIds.has(entry.entry_id)) {
        return t('lorebooks.form.errors.entryIdDuplicate')
      }

      seenEntryIds.add(entry.entry_id)

      if (entry.title.length === 0) {
        return t('lorebooks.form.errors.titleRequired')
      }

      if (entry.content.length === 0) {
        return t('lorebooks.form.errors.contentRequired')
      }
    }

    return null
  }

  async function handleSubmit() {
    const validationError = validateForm()

    setSubmitError(null)

    if (validationError) {
      setSubmitError(validationError)
      return
    }

    const nextLorebookId = formState.lorebookId.trim()
    const nextDisplayName = formState.displayName.trim()
    const nextEntries = normalizedEntries

    setIsSubmitting(true)

    try {
      let result: Lorebook

      if (!isEditMode) {
        result = await createLorebook({
          display_name: nextDisplayName,
          entries: nextEntries,
          lorebook_id: nextLorebookId,
        })
      } else {
        if (!initialLorebook) {
          throw new Error(t('lorebooks.feedback.loadLorebookFailed'))
        }

        const initialEntriesById = new Map(
          initialLorebook.entries.map((entry) => [entry.entry_id, entry]),
        )
        const nextEntriesById = new Map(nextEntries.map((entry) => [entry.entry_id, entry]))
        const deletedEntries = initialLorebook.entries.filter(
          (entry) => !nextEntriesById.has(entry.entry_id),
        )
        const createdEntries = nextEntries.filter(
          (entry) => !initialEntriesById.has(entry.entry_id),
        )
        const updatedEntries = nextEntries.filter((entry) => {
          const initialEntry = initialEntriesById.get(entry.entry_id)

          return initialEntry ? !areEntriesEqual(initialEntry, entry) : false
        })
        const hasDisplayNameChanged = initialLorebook.display_name !== nextDisplayName

        if (hasDisplayNameChanged) {
          result = await updateLorebook({
            display_name: nextDisplayName,
            lorebook_id: nextLorebookId,
          })
        } else {
          result = initialLorebook
        }

        for (const entry of deletedEntries) {
          await deleteLorebookEntry({
            entry_id: entry.entry_id,
            lorebook_id: nextLorebookId,
          })
        }

        for (const entry of createdEntries) {
          await createLorebookEntry({
            always_include: entry.always_include,
            content: entry.content,
            enabled: entry.enabled,
            entry_id: entry.entry_id,
            keywords: entry.keywords,
            lorebook_id: nextLorebookId,
            title: entry.title,
          })
        }

        for (const entry of updatedEntries) {
          await updateLorebookEntry({
            always_include: entry.always_include,
            content: entry.content,
            enabled: entry.enabled,
            entry_id: entry.entry_id,
            keywords: entry.keywords,
            lorebook_id: nextLorebookId,
            title: entry.title,
          })
        }

        result =
          !hasDisplayNameChanged &&
          deletedEntries.length === 0 &&
          createdEntries.length === 0 &&
          updatedEntries.length === 0
            ? initialLorebook
            : await getLorebook(nextLorebookId)
      }

      await onCompleted({
        lorebook: result,
        message: isEditMode
          ? t('lorebooks.feedback.updated', { name: result.display_name })
          : t('lorebooks.feedback.created', { name: result.display_name }),
      })

      onOpenChange(false)
    } catch (error) {
      setSubmitError(getErrorMessage(error, t('lorebooks.form.errors.submitFailed')))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent
        aria-describedby={undefined}
        className="w-[min(92vw,72rem)]"
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
            {isEditMode ? t('lorebooks.form.editTitle') : t('lorebooks.form.createTitle')}
          </DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-6 pt-6">
          {isLoading ? (
            <LoadingSkeleton />
          ) : (
            <>
              <div className="grid gap-4 md:grid-cols-2">
                <Field label={t('lorebooks.form.fields.lorebookId')} htmlFor={fieldIds.lorebookId}>
                  <Input
                    disabled={isEditMode}
                    id={fieldIds.lorebookId}
                    name="lorebook_id"
                    placeholder={t('lorebooks.form.placeholders.lorebookId')}
                    value={formState.lorebookId}
                    onChange={(event) => {
                      setFormState((current) => ({
                        ...current,
                        lorebookId: event.target.value,
                      }))
                    }}
                  />
                </Field>

                <Field
                  label={t('lorebooks.form.fields.displayName')}
                  htmlFor={fieldIds.displayName}
                >
                  <Input
                    id={fieldIds.displayName}
                    name="display_name"
                    placeholder={t('lorebooks.form.placeholders.displayName')}
                    value={formState.displayName}
                    onChange={(event) => {
                      setFormState((current) => ({
                        ...current,
                        displayName: event.target.value,
                      }))
                    }}
                  />
                </Field>
              </div>

              <div className="space-y-4">
                <div className="flex items-center justify-between gap-3">
                  <div className="space-y-1">
                    <h3 className="font-display text-[1.45rem] text-[var(--color-text-primary)]">
                      {t('lorebooks.form.fields.entries')}
                    </h3>
                    <p className="text-xs leading-6 text-[var(--color-text-muted)]">
                      {t('lorebooks.form.hints.keywords')}
                    </p>
                  </div>

                  <Button onClick={appendEntry} size="sm" variant="secondary">
                    <FontAwesomeIcon icon={faPlus} />
                    {t('lorebooks.actions.addEntry')}
                  </Button>
                </div>

                {formState.entries.length === 0 ? (
                  <div className="rounded-[1.6rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_68%,transparent)] px-5 py-6 text-sm leading-7 text-[var(--color-text-secondary)]">
                    {t('lorebooks.form.emptyEntries')}
                  </div>
                ) : (
                  <motion.div layout className="space-y-4">
                    <AnimatePresence initial={false}>
                      {formState.entries.map((entry, index) => {
                        const entryFieldIds = {
                          content: `${fieldIdPrefix}-entry-${index}-content`,
                          entryId: `${fieldIdPrefix}-entry-${index}-id`,
                          keywords: `${fieldIdPrefix}-entry-${index}-keywords`,
                          title: `${fieldIdPrefix}-entry-${index}-title`,
                        } as const
                        const isExpanded = expandedEntryIds.includes(entry.clientId)
                        const entryIdLabel =
                          entry.entryId.trim() || t('lorebooks.list.emptyEntryId')
                        const titleLabel = entry.title.trim() || t('lorebooks.list.emptyEntryTitle')

                        return (
                          <motion.div
                            key={entry.clientId}
                            layout
                            animate={{ opacity: 1, scale: 1, y: 0 }}
                            className="overflow-hidden"
                            exit={{
                              opacity: 0,
                              scale: prefersReducedMotion ? 1 : 0.96,
                              y: prefersReducedMotion ? 0 : -10,
                            }}
                            initial={{
                              opacity: 0,
                              scale: prefersReducedMotion ? 1 : 0.96,
                              y: prefersReducedMotion ? 0 : 16,
                            }}
                            transition={
                              prefersReducedMotion
                                ? { duration: 0 }
                                : { duration: 0.24, ease: [0.22, 1, 0.36, 1] }
                            }
                          >
                            <div
                              className={cn(
                                'rounded-[1.65rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-5 transition duration-200 ease-out',
                                isExpanded ? 'py-5' : 'py-4',
                              )}
                            >
                              <div className="flex items-start justify-between gap-3">
                                <div className="min-w-0 flex-1 space-y-3">
                                  <div className="flex flex-wrap items-center gap-2">
                                    <h4 className="font-display text-[1.2rem] text-[var(--color-text-primary)]">
                                      {t('lorebooks.form.entryTitle', { index: index + 1 })}
                                    </h4>
                                    <Badge
                                      className="normal-case px-3 py-1.5"
                                      variant={entry.alwaysInclude ? 'info' : 'subtle'}
                                    >
                                      <FontAwesomeIcon
                                        className="text-[0.7rem]"
                                        icon={entry.alwaysInclude ? faThumbtack : faTags}
                                      />
                                      {entry.alwaysInclude
                                        ? t('lorebooks.form.preview.alwaysInclude')
                                        : t('lorebooks.form.preview.keywordTriggered')}
                                    </Badge>
                                  </div>

                                  <div className="grid gap-3 md:grid-cols-2">
                                    <div className="min-w-0 rounded-[1.1rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] px-4 py-3">
                                      <p className="text-[0.72rem] uppercase tracking-[0.08em] text-[var(--color-text-muted)]">
                                        {t('lorebooks.form.fields.entryId')}
                                      </p>
                                      <p className="mt-1 truncate text-sm font-medium text-[var(--color-text-primary)]">
                                        {entryIdLabel}
                                      </p>
                                    </div>

                                    <div className="min-w-0 rounded-[1.1rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] px-4 py-3">
                                      <p className="text-[0.72rem] uppercase tracking-[0.08em] text-[var(--color-text-muted)]">
                                        {t('lorebooks.form.fields.title')}
                                      </p>
                                      <p className="mt-1 truncate text-sm font-medium text-[var(--color-text-primary)]">
                                        {titleLabel}
                                      </p>
                                    </div>
                                  </div>
                                </div>

                                <div className="flex shrink-0 items-center gap-2">
                                  <IconButton
                                    disabled={isSubmitting}
                                    icon={
                                      <FontAwesomeIcon
                                        icon={isExpanded ? faChevronUp : faChevronDown}
                                      />
                                    }
                                    label={
                                      isExpanded
                                        ? t('lorebooks.actions.collapseEntry')
                                        : t('lorebooks.actions.expandEntry')
                                    }
                                    onClick={() => {
                                      toggleEntryExpanded(entry.clientId)
                                    }}
                                    size="sm"
                                    variant="secondary"
                                  />
                                  <IconButton
                                    disabled={isSubmitting}
                                    icon={<FontAwesomeIcon icon={faTrashCan} />}
                                    label={t('lorebooks.actions.removeEntry')}
                                    onClick={() => {
                                      removeEntry(index)
                                    }}
                                    size="sm"
                                    variant="ghost"
                                  />
                                </div>
                              </div>

                              <AnimatePresence initial={false}>
                                {isExpanded ? (
                                  <motion.div
                                    animate={{ height: 'auto', opacity: 1, y: 0 }}
                                    className="overflow-hidden"
                                    exit={{
                                      height: 0,
                                      opacity: 0,
                                      y: prefersReducedMotion ? 0 : -8,
                                    }}
                                    initial={{
                                      height: 0,
                                      opacity: 0,
                                      y: prefersReducedMotion ? 0 : -8,
                                    }}
                                    transition={
                                      prefersReducedMotion
                                        ? { duration: 0 }
                                        : { duration: 0.22, ease: [0.22, 1, 0.36, 1] }
                                    }
                                  >
                                    <div className="mt-5 space-y-4 border-t border-[var(--color-border-subtle)] pt-5">
                                      <div className="grid gap-4 md:grid-cols-2">
                                        <Field
                                          htmlFor={entryFieldIds.entryId}
                                          label={t('lorebooks.form.fields.entryId')}
                                        >
                                          <Input
                                            id={entryFieldIds.entryId}
                                            name={`entry_id_${index}`}
                                            placeholder={t('lorebooks.form.placeholders.entryId')}
                                            value={entry.entryId}
                                            onChange={(event) => {
                                              updateEntry(index, { entryId: event.target.value })
                                            }}
                                          />
                                        </Field>

                                        <Field
                                          htmlFor={entryFieldIds.title}
                                          label={t('lorebooks.form.fields.title')}
                                        >
                                          <Input
                                            id={entryFieldIds.title}
                                            name={`title_${index}`}
                                            placeholder={t('lorebooks.form.placeholders.title')}
                                            value={entry.title}
                                            onChange={(event) => {
                                              updateEntry(index, { title: event.target.value })
                                            }}
                                          />
                                        </Field>
                                      </div>

                                      <div>
                                        <Field
                                          htmlFor={entryFieldIds.content}
                                          label={t('lorebooks.form.fields.content')}
                                        >
                                          <Textarea
                                            id={entryFieldIds.content}
                                            name={`content_${index}`}
                                            placeholder={t('lorebooks.form.placeholders.content')}
                                            rows={5}
                                            value={entry.content}
                                            onChange={(event) => {
                                              updateEntry(index, { content: event.target.value })
                                            }}
                                          />
                                        </Field>
                                      </div>

                                      <div>
                                        <Field
                                          description={t('lorebooks.form.hints.keywords')}
                                          htmlFor={entryFieldIds.keywords}
                                          label={t('lorebooks.form.fields.keywords')}
                                        >
                                          <Input
                                            id={entryFieldIds.keywords}
                                            name={`keywords_${index}`}
                                            placeholder={t('lorebooks.form.placeholders.keywords')}
                                            value={entry.keywords}
                                            onChange={(event) => {
                                              updateEntry(index, { keywords: event.target.value })
                                            }}
                                          />
                                        </Field>
                                      </div>

                                      <div className="grid gap-3 md:grid-cols-2">
                                        <div className="flex items-center justify-between rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] px-4 py-3">
                                          <div className="space-y-0.5">
                                            <p className="text-sm font-medium text-[var(--color-text-primary)]">
                                              {t('lorebooks.form.fields.enabled')}
                                            </p>
                                            <p className="text-xs leading-6 text-[var(--color-text-muted)]">
                                              {t('lorebooks.form.fields.enabledDescription')}
                                            </p>
                                          </div>
                                          <Switch
                                            checked={entry.enabled}
                                            onCheckedChange={(checked) => {
                                              updateEntry(index, { enabled: checked })
                                            }}
                                            size="sm"
                                          />
                                        </div>

                                        <div className="flex items-center justify-between rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] px-4 py-3">
                                          <div className="space-y-0.5">
                                            <p className="text-sm font-medium text-[var(--color-text-primary)]">
                                              {t('lorebooks.form.fields.alwaysInclude')}
                                            </p>
                                            <p className="text-xs leading-6 text-[var(--color-text-muted)]">
                                              {t('lorebooks.form.fields.alwaysIncludeDescription')}
                                            </p>
                                          </div>
                                          <Switch
                                            checked={entry.alwaysInclude}
                                            onCheckedChange={(checked) => {
                                              updateEntry(index, { alwaysInclude: checked })
                                            }}
                                            size="sm"
                                          />
                                        </div>
                                      </div>
                                    </div>
                                  </motion.div>
                                ) : null}
                              </AnimatePresence>
                            </div>
                          </motion.div>
                        )
                      })}
                    </AnimatePresence>
                  </motion.div>
                )}
              </div>
            </>
          )}
        </DialogBody>

        <DialogFooter className="justify-end gap-2">
          <Button
            disabled={isSubmitting}
            onClick={() => {
              onOpenChange(false)
            }}
            variant="ghost"
          >
            {t('lorebooks.actions.cancel')}
          </Button>
          <Button
            disabled={isLoading || isSubmitting}
            onClick={() => {
              void handleSubmit()
            }}
          >
            {isSubmitting ? t('lorebooks.actions.saving') : t('lorebooks.actions.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
