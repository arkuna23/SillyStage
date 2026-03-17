import { type DragEvent, useEffect, useMemo, useState } from 'react'
import { faChevronDown } from '@fortawesome/free-solid-svg-icons/faChevronDown'
import { faGripVertical } from '@fortawesome/free-solid-svg-icons/faGripVertical'
import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { AnimatePresence, motion, useReducedMotion } from 'framer-motion'
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
import { createPreset, getPreset, updatePreset } from '../apis/api'
import {
  agentRoleKeys,
  getPresetPromptEntries,
  type AgentPresetConfig,
  type AgentRoleKey,
  type Preset,
  type PresetAgentConfigs,
  type PresetPromptEntry,
} from '../apis/types'

type PresetFormDialogProps = {
  existingPresetIds: ReadonlyArray<string>
  mode: 'create' | 'edit'
  onCompleted: (result: { message: string; preset: Preset }) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
  presetId?: string | null
}

type TranslateFn = (key: string, options?: Record<string, unknown>) => string

type PromptEntryFormState = {
  clientId: string
  content: string
  enabled: boolean
  entryId: string
  title: string
}

type AgentFormState = {
  extra: string
  maxTokens: string
  promptEntries: PromptEntryFormState[]
  temperature: string
}

type FormState = {
  agents: Record<AgentRoleKey, AgentFormState>
  displayName: string
  presetId: string
}

type PromptEntryDragState = {
  clientId: string
  roleKey: AgentRoleKey
}

let promptEntryClientIdCounter = 0

function createPromptEntryClientId() {
  promptEntryClientIdCounter += 1
  return `preset-prompt-entry-${promptEntryClientIdCounter}`
}

function createPromptEntryState(entry?: PresetPromptEntry): PromptEntryFormState {
  return {
    clientId: createPromptEntryClientId(),
    content: entry?.content ?? '',
    enabled: entry?.enabled ?? true,
    entryId: entry?.entry_id ?? '',
    title: entry?.title ?? '',
  }
}

function createEmptyAgentState(): AgentFormState {
  return {
    extra: '',
    maxTokens: '',
    promptEntries: [],
    temperature: '',
  }
}

function createInitialState(): FormState {
  return {
    agents: {
      actor: createEmptyAgentState(),
      architect: createEmptyAgentState(),
      director: createEmptyAgentState(),
      keeper: createEmptyAgentState(),
      narrator: createEmptyAgentState(),
      planner: createEmptyAgentState(),
      replyer: createEmptyAgentState(),
    },
    displayName: '',
    presetId: '',
  }
}

function createCollapsedAgentsState(): Record<AgentRoleKey, boolean> {
  return {
    actor: false,
    architect: false,
    director: false,
    keeper: false,
    narrator: false,
    planner: false,
    replyer: false,
  }
}

function createCollapsedPromptEntriesState() {
  return [] as string[]
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function createAgentFormState(agent: Preset['agents'][AgentRoleKey]): AgentFormState {
  return {
    extra: agent.extra !== undefined && agent.extra !== null ? JSON.stringify(agent.extra, null, 2) : '',
    maxTokens: agent.max_tokens?.toString() ?? '',
    promptEntries: getPresetPromptEntries(agent).map((entry) => createPromptEntryState(entry)),
    temperature: agent.temperature?.toString() ?? '',
  }
}

function summarizeAgent(
  agent: AgentFormState,
  t: TranslateFn,
  emptyLabel: string,
) {
  const promptEntryCount = agent.promptEntries.length
  const enabledPromptEntryCount = agent.promptEntries.filter((entry) => entry.enabled).length
  const parts = [
    agent.temperature.trim() ? `T ${agent.temperature.trim()}` : null,
    agent.maxTokens.trim() ? `Max ${agent.maxTokens.trim()}` : null,
    agent.extra.trim() ? t('presetsPage.list.extra') : null,
    promptEntryCount > 0
      ? t('presetsPage.list.promptEntriesSummary', {
          count: promptEntryCount,
          enabled: enabledPromptEntryCount,
        })
      : null,
  ].filter((value): value is string => Boolean(value))

  return parts.length > 0 ? parts : [emptyLabel]
}

function parseAgentPresetConfig(
  roleKey: AgentRoleKey,
  agent: AgentFormState,
  t: TranslateFn,
  roleLabels: Record<AgentRoleKey, string>,
): AgentPresetConfig {
  let extra: unknown | null | undefined

  if (agent.extra.trim()) {
    try {
      extra = JSON.parse(agent.extra)
    } catch {
      throw new Error(
        t('presetsPage.form.errors.extraInvalid', {
          role: roleLabels[roleKey],
        }),
      )
    }
  }

  let temperature: number | undefined
  if (agent.temperature.trim()) {
    const parsed = Number(agent.temperature)
    if (!Number.isFinite(parsed)) {
      throw new Error(
        t('presetsPage.form.errors.temperatureInvalid', {
          role: roleLabels[roleKey],
        }),
      )
    }
    temperature = parsed
  }

  let maxTokens: number | undefined
  if (agent.maxTokens.trim()) {
    const parsed = Number(agent.maxTokens)
    if (!Number.isInteger(parsed) || parsed <= 0) {
      throw new Error(
        t('presetsPage.form.errors.maxTokensInvalid', {
          role: roleLabels[roleKey],
        }),
      )
    }
    maxTokens = parsed
  }

  const seenEntryIds = new Set<string>()
  const promptEntries = agent.promptEntries.map((entry, index) => {
    const entryId = entry.entryId.trim()
    const title = entry.title.trim()
    const content = entry.content.trim()

    if (!entryId) {
      throw new Error(
        t('presetsPage.form.errors.promptEntryIdRequired', {
          index: index + 1,
          role: roleLabels[roleKey],
        }),
      )
    }

    if (seenEntryIds.has(entryId)) {
      throw new Error(
        t('presetsPage.form.errors.duplicatePromptEntryId', {
          id: entryId,
          role: roleLabels[roleKey],
        }),
      )
    }
    seenEntryIds.add(entryId)

    if (!title) {
      throw new Error(
        t('presetsPage.form.errors.promptEntryTitleRequired', {
          index: index + 1,
          role: roleLabels[roleKey],
        }),
      )
    }

    if (!content) {
      throw new Error(
        t('presetsPage.form.errors.promptEntryContentRequired', {
          index: index + 1,
          role: roleLabels[roleKey],
        }),
      )
    }

    return {
      content,
      enabled: entry.enabled,
      entry_id: entryId,
      title,
    }
  })

  return {
    ...(temperature !== undefined ? { temperature } : {}),
    ...(maxTokens !== undefined ? { max_tokens: maxTokens } : {}),
    ...(extra !== undefined ? { extra } : {}),
    prompt_entries: promptEntries,
  }
}

function toPresetAgents(
  agents: FormState['agents'],
  t: TranslateFn,
  roleLabels: Record<AgentRoleKey, string>,
) {
  return Object.fromEntries(
    agentRoleKeys.map((roleKey) => [
      roleKey,
      parseAgentPresetConfig(roleKey, agents[roleKey], t, roleLabels),
    ]),
  ) as PresetAgentConfigs
}

export function PresetFormDialog({
  existingPresetIds,
  mode,
  onCompleted,
  onOpenChange,
  open,
  presetId,
}: PresetFormDialogProps) {
  const { t } = useTranslation()
  const prefersReducedMotion = useReducedMotion()
  const translate = (key: string, options?: Record<string, unknown>) =>
    String(t(key as never, options as never))
  const [formState, setFormState] = useState<FormState>(createInitialState)
  const [expandedAgents, setExpandedAgents] = useState<Record<AgentRoleKey, boolean>>(
    createCollapsedAgentsState,
  )
  const [expandedModelSettings, setExpandedModelSettings] = useState<Record<AgentRoleKey, boolean>>(
    createCollapsedAgentsState,
  )
  const [expandedPromptEntryIds, setExpandedPromptEntryIds] = useState<string[]>(
    createCollapsedPromptEntriesState,
  )
  const [draggedPromptEntry, setDraggedPromptEntry] = useState<PromptEntryDragState | null>(null)
  const [dropTarget, setDropTarget] = useState<PromptEntryDragState | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  useToastMessage(submitError)

  const roleLabels: Record<AgentRoleKey, string> = useMemo(
    () => ({
      actor: t('presetsPage.roles.actor'),
      architect: t('presetsPage.roles.architect'),
      director: t('presetsPage.roles.director'),
      keeper: t('presetsPage.roles.keeper'),
      narrator: t('presetsPage.roles.narrator'),
      planner: t('presetsPage.roles.planner'),
      replyer: t('presetsPage.roles.replyer'),
    }),
    [t],
  )

  useEffect(() => {
    if (!open) {
      setFormState(createInitialState())
      setExpandedAgents(createCollapsedAgentsState())
      setExpandedModelSettings(createCollapsedAgentsState())
      setExpandedPromptEntryIds(createCollapsedPromptEntriesState())
      setDraggedPromptEntry(null)
      setDropTarget(null)
      setIsLoading(false)
      setIsSubmitting(false)
      setSubmitError(null)
      return
    }

    if (mode !== 'edit' || !presetId) {
      setFormState(createInitialState())
      setExpandedAgents(createCollapsedAgentsState())
      setExpandedModelSettings(createCollapsedAgentsState())
      setExpandedPromptEntryIds(createCollapsedPromptEntriesState())
      setDraggedPromptEntry(null)
      setDropTarget(null)
      setIsLoading(false)
      setSubmitError(null)
      return
    }

    const controller = new AbortController()
    setIsLoading(true)
    setSubmitError(null)
    setExpandedPromptEntryIds(createCollapsedPromptEntriesState())
    setDraggedPromptEntry(null)
    setDropTarget(null)

    void getPreset(presetId, controller.signal)
      .then((result) => {
        if (controller.signal.aborted) {
          return
        }

        setFormState({
          agents: Object.fromEntries(
            agentRoleKeys.map((roleKey) => [roleKey, createAgentFormState(result.agents[roleKey])]),
          ) as FormState['agents'],
          displayName: result.display_name,
          presetId: result.preset_id,
        })
        setExpandedAgents(createCollapsedAgentsState())
        setExpandedModelSettings(createCollapsedAgentsState())
        setExpandedPromptEntryIds(createCollapsedPromptEntriesState())
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setSubmitError(getErrorMessage(error, t('presetsPage.feedback.loadPresetFailed')))
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
  }, [mode, open, presetId, t])

  function updateAgent(roleKey: AgentRoleKey, key: keyof AgentFormState, value: string) {
    setFormState((current) => ({
      ...current,
      agents: {
        ...current.agents,
        [roleKey]: {
          ...current.agents[roleKey],
          [key]: value,
        },
      },
    }))
  }

  function toggleAgent(roleKey: AgentRoleKey) {
    setExpandedAgents((current) => ({
      ...current,
      [roleKey]: !current[roleKey],
    }))
  }

  function toggleModelSettings(roleKey: AgentRoleKey) {
    setExpandedModelSettings((current) => ({
      ...current,
      [roleKey]: !current[roleKey],
    }))
  }

  function updatePromptEntryField(
    roleKey: AgentRoleKey,
    clientId: string,
    key: 'content' | 'entryId' | 'title',
    value: string,
  ) {
    setFormState((current) => ({
      ...current,
      agents: {
        ...current.agents,
        [roleKey]: {
          ...current.agents[roleKey],
          promptEntries: current.agents[roleKey].promptEntries.map((entry) =>
            entry.clientId === clientId ? { ...entry, [key]: value } : entry,
          ),
        },
      },
    }))
  }

  function updatePromptEntryEnabled(roleKey: AgentRoleKey, clientId: string, enabled: boolean) {
    setFormState((current) => ({
      ...current,
      agents: {
        ...current.agents,
        [roleKey]: {
          ...current.agents[roleKey],
          promptEntries: current.agents[roleKey].promptEntries.map((entry) =>
            entry.clientId === clientId ? { ...entry, enabled } : entry,
          ),
        },
      },
    }))
  }

  function addPromptEntry(roleKey: AgentRoleKey) {
    const nextEntry = createPromptEntryState()

    setFormState((current) => ({
      ...current,
      agents: {
        ...current.agents,
        [roleKey]: {
          ...current.agents[roleKey],
          promptEntries: [...current.agents[roleKey].promptEntries, nextEntry],
        },
      },
    }))
    setExpandedAgents((current) => ({
      ...current,
      [roleKey]: true,
    }))
    setExpandedPromptEntryIds((current) =>
      current.includes(nextEntry.clientId) ? current : [...current, nextEntry.clientId],
    )
  }

  function togglePromptEntry(clientId: string) {
    setExpandedPromptEntryIds((current) =>
      current.includes(clientId)
        ? current.filter((currentId) => currentId !== clientId)
        : [...current, clientId],
    )
  }

  function removePromptEntry(roleKey: AgentRoleKey, clientId: string) {
    setFormState((current) => ({
      ...current,
      agents: {
        ...current.agents,
        [roleKey]: {
          ...current.agents[roleKey],
          promptEntries: current.agents[roleKey].promptEntries.filter(
            (entry) => entry.clientId !== clientId,
          ),
        },
      },
    }))
    setExpandedPromptEntryIds((current) => current.filter((currentId) => currentId !== clientId))

    if (draggedPromptEntry?.clientId === clientId && draggedPromptEntry.roleKey === roleKey) {
      setDraggedPromptEntry(null)
    }

    if (dropTarget?.clientId === clientId && dropTarget.roleKey === roleKey) {
      setDropTarget(null)
    }
  }

  function movePromptEntry(roleKey: AgentRoleKey, sourceClientId: string, targetClientId: string) {
    if (sourceClientId === targetClientId) {
      return
    }

    setFormState((current) => {
      const entries = current.agents[roleKey].promptEntries
      const sourceIndex = entries.findIndex((entry) => entry.clientId === sourceClientId)
      const targetIndex = entries.findIndex((entry) => entry.clientId === targetClientId)

      if (sourceIndex === -1 || targetIndex === -1) {
        return current
      }

      const nextEntries = [...entries]
      const [movedEntry] = nextEntries.splice(sourceIndex, 1)
      nextEntries.splice(targetIndex, 0, movedEntry)

      return {
        ...current,
        agents: {
          ...current.agents,
          [roleKey]: {
            ...current.agents[roleKey],
            promptEntries: nextEntries,
          },
        },
      }
    })
  }

  function handlePromptEntryDragStart(roleKey: AgentRoleKey, clientId: string) {
    setDraggedPromptEntry({ clientId, roleKey })
    setDropTarget(null)
  }

  function handlePromptEntryDragOver(
    event: DragEvent<HTMLDivElement>,
    roleKey: AgentRoleKey,
    clientId: string,
  ) {
    if (!draggedPromptEntry || draggedPromptEntry.roleKey !== roleKey || draggedPromptEntry.clientId === clientId) {
      return
    }

    event.preventDefault()

    if (dropTarget?.clientId !== clientId || dropTarget.roleKey !== roleKey) {
      setDropTarget({ clientId, roleKey })
    }
  }

  function handlePromptEntryDrop(
    event: DragEvent<HTMLDivElement>,
    roleKey: AgentRoleKey,
    clientId: string,
  ) {
    if (!draggedPromptEntry || draggedPromptEntry.roleKey !== roleKey) {
      return
    }

    event.preventDefault()
    movePromptEntry(roleKey, draggedPromptEntry.clientId, clientId)
    setDraggedPromptEntry(null)
    setDropTarget(null)
  }

  function handlePromptEntryDragEnd() {
    setDraggedPromptEntry(null)
    setDropTarget(null)
  }

  async function handleSubmit() {
    if (!formState.presetId.trim()) {
      setSubmitError(t('presetsPage.form.errors.presetIdRequired'))
      return
    }

    if (mode === 'create' && existingPresetIds.includes(formState.presetId.trim())) {
      setSubmitError(t('presetsPage.form.errors.presetIdDuplicate'))
      return
    }

    if (!formState.displayName.trim()) {
      setSubmitError(t('presetsPage.form.errors.displayNameRequired'))
      return
    }

    setSubmitError(null)
    setIsSubmitting(true)

    try {
      const agents = toPresetAgents(formState.agents, translate, roleLabels)
      const preset =
        mode === 'create'
          ? await createPreset({
              agents,
              display_name: formState.displayName.trim(),
              preset_id: formState.presetId.trim(),
            })
          : await updatePreset({
              agents,
              display_name: formState.displayName.trim(),
              preset_id: formState.presetId.trim(),
            })

      await onCompleted({
        message:
          mode === 'create'
            ? t('presetsPage.feedback.created', { id: preset.display_name })
            : t('presetsPage.feedback.updated', { id: preset.display_name }),
        preset,
      })

      onOpenChange(false)
    } catch (error) {
      setSubmitError(getErrorMessage(error, t('presetsPage.form.errors.submitFailed')))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent
        aria-describedby={undefined}
        contentClassName="w-[min(96vw,72rem)]"
      >
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>
            {mode === 'create' ? t('presetsPage.form.createTitle') : t('presetsPage.form.editTitle')}
          </DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          {isLoading ? (
            <div className="space-y-3">
              <div className="h-12 animate-pulse rounded-[1.35rem] bg-[var(--color-bg-elevated)]" />
              {Array.from({ length: agentRoleKeys.length }).map((_, index) => (
                <div
                  className="h-20 animate-pulse rounded-[1.35rem] bg-[var(--color-bg-elevated)]"
                  key={index}
                />
              ))}
            </div>
          ) : (
            <>
              <div className="grid gap-4 md:grid-cols-2">
                <label className="space-y-2.5">
                  <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                    {t('presetsPage.form.fields.presetId')}
                  </span>
                  <Input
                    disabled={mode === 'edit'}
                    id="preset-form-preset-id"
                    name="preset_id"
                    onChange={(event) => {
                      setFormState((current) => ({
                        ...current,
                        presetId: event.target.value,
                      }))
                    }}
                    placeholder={t('presetsPage.form.placeholders.presetId')}
                    value={formState.presetId}
                  />
                </label>

                <label className="space-y-2.5">
                  <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                    {t('presetsPage.form.fields.displayName')}
                  </span>
                  <Input
                    id="preset-form-display-name"
                    name="display_name"
                    onChange={(event) => {
                      setFormState((current) => ({
                        ...current,
                        displayName: event.target.value,
                      }))
                    }}
                    placeholder={t('presetsPage.form.placeholders.displayName')}
                    value={formState.displayName}
                  />
                </label>
              </div>

              <motion.div layout className="space-y-3">
                {agentRoleKeys.map((roleKey) => {
                  const agentState = formState.agents[roleKey]
                  const isExpanded = expandedAgents[roleKey]
                  const isModelSettingsExpanded = expandedModelSettings[roleKey]
                  const summaryItems = summarizeAgent(
                    agentState,
                    translate,
                    t('presetsPage.list.unset'),
                  )
                  return (
                    <motion.section
                      layout
                      className="overflow-hidden rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)]"
                      key={roleKey}
                      transition={
                        prefersReducedMotion
                          ? { duration: 0 }
                          : { duration: 0.24, ease: [0.22, 1, 0.36, 1] }
                      }
                    >
                      <button
                        aria-expanded={isExpanded}
                        className="flex w-full items-center justify-between gap-4 px-4 py-4 text-left transition duration-200 hover:bg-white/5"
                        onClick={() => {
                          toggleAgent(roleKey)
                        }}
                        title={
                          isExpanded
                            ? t('presetsPage.actions.collapseAgent')
                            : t('presetsPage.actions.expandAgent')
                        }
                        type="button"
                      >
                        <div className="flex min-w-0 flex-1 flex-wrap items-center gap-2.5">
                          <p className="shrink-0 text-sm font-medium text-[var(--color-text-primary)]">
                            {roleLabels[roleKey]}
                          </p>
                          <div className="flex min-w-0 flex-wrap items-center gap-2">
                            {summaryItems.map((item) => (
                              <Badge key={`${roleKey}-${item}`} variant="subtle">
                                {item}
                              </Badge>
                            ))}
                          </div>
                        </div>

                        <motion.span
                          animate={{ rotate: isExpanded ? 180 : 0 }}
                          className="inline-flex h-9 w-9 items-center justify-center rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-[var(--color-text-secondary)]"
                          transition={
                            prefersReducedMotion
                              ? { duration: 0 }
                              : { duration: 0.22, ease: [0.22, 1, 0.36, 1] }
                          }
                        >
                          <FontAwesomeIcon icon={faChevronDown} />
                        </motion.span>
                      </button>

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
                            <div className="space-y-5 border-t border-[var(--color-border-subtle)] px-4 py-4">
                              <div className="overflow-hidden rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_72%,transparent)]">
                                <button
                                  aria-expanded={isModelSettingsExpanded}
                                  className="flex w-full items-center justify-between gap-4 px-4 py-4 text-left transition duration-200 hover:bg-white/5"
                                  onClick={() => {
                                    toggleModelSettings(roleKey)
                                  }}
                                  title={
                                    isModelSettingsExpanded
                                      ? t('presetsPage.actions.collapseModelSettings')
                                      : t('presetsPage.actions.expandModelSettings')
                                  }
                                  type="button"
                                >
                                  <div className="flex min-w-0 flex-1 items-center gap-2.5">
                                    <p className="shrink-0 text-sm font-medium text-[var(--color-text-primary)]">
                                      {t('presetsPage.form.fields.modelSettings')}
                                    </p>
                                  </div>

                                  <motion.span
                                    animate={{ rotate: isModelSettingsExpanded ? 180 : 0 }}
                                    className="inline-flex h-9 w-9 items-center justify-center rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-[var(--color-text-secondary)]"
                                    transition={
                                      prefersReducedMotion
                                        ? { duration: 0 }
                                        : { duration: 0.2, ease: [0.22, 1, 0.36, 1] }
                                    }
                                  >
                                    <FontAwesomeIcon icon={faChevronDown} />
                                  </motion.span>
                                </button>

                                <AnimatePresence initial={false}>
                                  {isModelSettingsExpanded ? (
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
                                          : { duration: 0.2, ease: [0.22, 1, 0.36, 1] }
                                      }
                                    >
                                      <div className="space-y-4 border-t border-[var(--color-border-subtle)] px-4 py-4">
                                        <div className="grid gap-4 md:grid-cols-2">
                                          <label className="space-y-2">
                                            <span className="block text-xs text-[var(--color-text-muted)]">
                                              {t('presetsPage.form.fields.temperature')}
                                            </span>
                                            <Input
                                              id={`preset-form-${roleKey}-temperature`}
                                              name={`${roleKey}_temperature`}
                                              onChange={(event) => {
                                                updateAgent(roleKey, 'temperature', event.target.value)
                                              }}
                                              placeholder={t('presetsPage.form.placeholders.temperature')}
                                              value={agentState.temperature}
                                            />
                                          </label>

                                          <label className="space-y-2">
                                            <span className="block text-xs text-[var(--color-text-muted)]">
                                              {t('presetsPage.form.fields.maxTokens')}
                                            </span>
                                            <Input
                                              id={`preset-form-${roleKey}-max-tokens`}
                                              name={`${roleKey}_max_tokens`}
                                              onChange={(event) => {
                                                updateAgent(roleKey, 'maxTokens', event.target.value)
                                              }}
                                              placeholder={t('presetsPage.form.placeholders.maxTokens')}
                                              value={agentState.maxTokens}
                                            />
                                          </label>
                                        </div>

                                        <label className="space-y-2">
                                          <span className="block text-xs text-[var(--color-text-muted)]">
                                            {t('presetsPage.form.fields.extra')}
                                          </span>
                                          <Textarea
                                            className="min-h-[8rem]"
                                            id={`preset-form-${roleKey}-extra`}
                                            name={`${roleKey}_extra`}
                                            onChange={(event) => {
                                              updateAgent(roleKey, 'extra', event.target.value)
                                            }}
                                            placeholder={t('presetsPage.form.placeholders.extra')}
                                            value={agentState.extra}
                                          />
                                        </label>
                                      </div>
                                    </motion.div>
                                  ) : null}
                                </AnimatePresence>
                              </div>

                              <div className="space-y-3">
                                <div className="flex flex-wrap items-start justify-between gap-3">
                                  <div className="space-y-1">
                                    <p className="text-sm font-medium text-[var(--color-text-primary)]">
                                      {t('presetsPage.form.fields.promptEntries')}
                                    </p>
                                    <p className="text-xs leading-6 text-[var(--color-text-muted)]">
                                      {t('presetsPage.form.promptEntriesHint')}
                                    </p>
                                  </div>

                                  <Button
                                    onClick={() => {
                                      addPromptEntry(roleKey)
                                    }}
                                    size="sm"
                                    variant="secondary"
                                  >
                                    <FontAwesomeIcon icon={faPlus} />
                                    {t('presetsPage.actions.addPromptEntry')}
                                  </Button>
                                </div>

                                {agentState.promptEntries.length > 0 ? (
                                  <div className="space-y-3">
                                    {agentState.promptEntries.map((entry, index) => {
                                      const isExpanded = expandedPromptEntryIds.includes(entry.clientId)
                                      const isDragged =
                                        draggedPromptEntry?.clientId === entry.clientId &&
                                        draggedPromptEntry.roleKey === roleKey
                                      const isDropTarget =
                                        dropTarget?.clientId === entry.clientId &&
                                        dropTarget.roleKey === roleKey &&
                                        !isDragged

                                      return (
                                        <motion.div
                                          layout
                                          className={cn(
                                            'rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_72%,transparent)] px-4 py-4 transition duration-200',
                                            isDragged && 'opacity-55',
                                            isDropTarget &&
                                              'border-[var(--color-accent-gold-line)] shadow-[0_0_0_1px_var(--color-accent-gold-line)]',
                                          )}
                                          key={entry.clientId}
                                          onDragOver={(event) => {
                                            handlePromptEntryDragOver(event, roleKey, entry.clientId)
                                          }}
                                          onDrop={(event) => {
                                            handlePromptEntryDrop(event, roleKey, entry.clientId)
                                          }}
                                          transition={
                                            prefersReducedMotion
                                              ? { duration: 0 }
                                              : { duration: 0.22, ease: [0.22, 1, 0.36, 1] }
                                          }
                                        >
                                          <div className="flex flex-wrap items-center justify-between gap-3">
                                            <div className="flex min-w-0 items-center gap-3">
                                              <button
                                                aria-label={t('presetsPage.actions.dragPromptEntry')}
                                                className="inline-flex h-9 w-9 shrink-0 cursor-grab items-center justify-center rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-[var(--color-text-secondary)] transition duration-200 hover:text-[var(--color-text-primary)] active:cursor-grabbing"
                                                draggable
                                                onDragEnd={handlePromptEntryDragEnd}
                                                onDragStart={(event) => {
                                                  event.dataTransfer.effectAllowed = 'move'
                                                  event.dataTransfer.setData('text/plain', entry.clientId)
                                                  handlePromptEntryDragStart(roleKey, entry.clientId)
                                                }}
                                                type="button"
                                              >
                                                <FontAwesomeIcon icon={faGripVertical} />
                                              </button>

                                              <button
                                                aria-expanded={isExpanded}
                                                className="flex min-w-0 flex-1 flex-wrap items-center gap-2.5 text-left"
                                                onClick={() => {
                                                  togglePromptEntry(entry.clientId)
                                                }}
                                                title={
                                                  isExpanded
                                                    ? t('presetsPage.actions.collapsePromptEntry')
                                                    : t('presetsPage.actions.expandPromptEntry')
                                                }
                                                type="button"
                                              >
                                                <p className="truncate text-sm font-medium text-[var(--color-text-primary)]">
                                                  {entry.title.trim() ||
                                                    t('presetsPage.form.untitledPromptEntry', {
                                                      index: index + 1,
                                                    })}
                                                </p>
                                                <Badge variant="subtle">
                                                  {entry.entryId.trim() ||
                                                    t('presetsPage.form.newPromptEntry')}
                                                </Badge>
                                              </button>
                                            </div>

                                            <div className="flex items-center gap-2">
                                              <IconButton
                                                icon={
                                                  <motion.span
                                                    animate={{ rotate: isExpanded ? 180 : 0 }}
                                                    transition={
                                                      prefersReducedMotion
                                                        ? { duration: 0 }
                                                        : {
                                                            duration: 0.2,
                                                            ease: [0.22, 1, 0.36, 1],
                                                          }
                                                    }
                                                  >
                                                    <FontAwesomeIcon icon={faChevronDown} />
                                                  </motion.span>
                                                }
                                                label={
                                                  isExpanded
                                                    ? t('presetsPage.actions.collapsePromptEntry')
                                                    : t('presetsPage.actions.expandPromptEntry')
                                                }
                                                onClick={() => {
                                                  togglePromptEntry(entry.clientId)
                                                }}
                                                size="sm"
                                                variant="secondary"
                                              />
                                              <span className="text-xs text-[var(--color-text-muted)]">
                                                {t('presetsPage.form.fields.promptEntryEnabled')}
                                              </span>
                                              <Switch
                                                checked={entry.enabled}
                                                onCheckedChange={(enabled) => {
                                                  updatePromptEntryEnabled(roleKey, entry.clientId, enabled)
                                                }}
                                                size="sm"
                                              />
                                              <IconButton
                                                icon={<FontAwesomeIcon icon={faTrashCan} />}
                                                label={t('presetsPage.actions.removePromptEntry')}
                                                onClick={() => {
                                                  removePromptEntry(roleKey, entry.clientId)
                                                }}
                                                size="sm"
                                                variant="danger"
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
                                                    : { duration: 0.2, ease: [0.22, 1, 0.36, 1] }
                                                }
                                              >
                                                <div className="mt-4 space-y-4 border-t border-[var(--color-border-subtle)] pt-4">
                                                  <div className="grid gap-4 md:grid-cols-2">
                                                    <label className="space-y-2">
                                                      <span className="block text-xs text-[var(--color-text-muted)]">
                                                        {t('presetsPage.form.fields.promptEntryId')}
                                                      </span>
                                                      <Input
                                                        id={`preset-form-${roleKey}-${entry.clientId}-id`}
                                                        name={`${roleKey}_prompt_entry_id_${index}`}
                                                        onChange={(event) => {
                                                          updatePromptEntryField(
                                                            roleKey,
                                                            entry.clientId,
                                                            'entryId',
                                                            event.target.value,
                                                          )
                                                        }}
                                                        placeholder={t('presetsPage.form.placeholders.promptEntryId')}
                                                        value={entry.entryId}
                                                      />
                                                    </label>

                                                    <label className="space-y-2">
                                                      <span className="block text-xs text-[var(--color-text-muted)]">
                                                        {t('presetsPage.form.fields.promptEntryTitle')}
                                                      </span>
                                                      <Input
                                                        id={`preset-form-${roleKey}-${entry.clientId}-title`}
                                                        name={`${roleKey}_prompt_entry_title_${index}`}
                                                        onChange={(event) => {
                                                          updatePromptEntryField(
                                                            roleKey,
                                                            entry.clientId,
                                                            'title',
                                                            event.target.value,
                                                          )
                                                        }}
                                                        placeholder={t('presetsPage.form.placeholders.promptEntryTitle')}
                                                        value={entry.title}
                                                      />
                                                    </label>
                                                  </div>

                                                  <label className="block space-y-2">
                                                    <span className="block text-xs text-[var(--color-text-muted)]">
                                                      {t('presetsPage.form.fields.promptEntryContent')}
                                                    </span>
                                                    <Textarea
                                                      className="min-h-[8rem]"
                                                      id={`preset-form-${roleKey}-${entry.clientId}-content`}
                                                      name={`${roleKey}_prompt_entry_content_${index}`}
                                                      onChange={(event) => {
                                                        updatePromptEntryField(
                                                          roleKey,
                                                          entry.clientId,
                                                          'content',
                                                          event.target.value,
                                                        )
                                                      }}
                                                      placeholder={t('presetsPage.form.placeholders.promptEntryContent')}
                                                      value={entry.content}
                                                    />
                                                  </label>
                                                </div>
                                              </motion.div>
                                            ) : null}
                                          </AnimatePresence>
                                        </motion.div>
                                      )
                                    })}
                                  </div>
                                ) : (
                                  <div className="rounded-[1.2rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_62%,transparent)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
                                    {t('presetsPage.form.emptyPromptEntries')}
                                  </div>
                                )}
                              </div>
                            </div>
                          </motion.div>
                        ) : null}
                      </AnimatePresence>
                    </motion.section>
                  )
                })}
              </motion.div>
            </>
          )}
        </DialogBody>

        <DialogFooter className="justify-end gap-2">
          <Button onClick={() => onOpenChange(false)} variant="ghost">
            {t('presetsPage.actions.cancel')}
          </Button>
          <Button disabled={isLoading || isSubmitting} onClick={() => void handleSubmit()}>
            {isSubmitting ? t('presetsPage.actions.saving') : t('presetsPage.actions.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
