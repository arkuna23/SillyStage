import { type DragEvent, useCallback, useEffect, useMemo, useState } from 'react'
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
import {
  createPreset,
  createPresetEntry,
  deletePresetEntry,
  getPreset,
  updatePreset,
  updatePresetEntry,
} from '../apis/api'
import {
  promptModuleIds,
  type AgentPresetConfig,
  type AgentRoleKey,
  type PresetDetail,
  type PresetEntryKind,
  type PresetModuleEntry,
  type PresetAgentConfigs,
  type PromptModuleId,
} from '../apis/types'
import {
  createPresetRoleLabels,
  getOrderedAgentRoleKeys,
  getPromptModuleLabel,
  getPresetEntryKindLabel,
} from './preset-labels'

type PresetFormDialogProps = {
  existingPresetIds: ReadonlyArray<string>
  mode: 'create' | 'edit'
  onCompleted: (result: { message: string; preset: PresetDetail }) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
  presetId?: string | null
}

type TranslateFn = (key: string, options?: Record<string, unknown>) => string

type PromptEntryFormState = {
  clientId: string
  contextKey: string
  displayName: string
  enabled: boolean
  entryId: string
  kind: PresetEntryKind
  order: string
  required: boolean
  text: string
}

type ModuleEntriesState = Record<PromptModuleId, PromptEntryFormState[]>

type AgentFormState = {
  extra: string
  maxTokens: string
  modules: ModuleEntriesState
  temperature: string
}

type FormState = {
  agents: Record<AgentRoleKey, AgentFormState>
  displayName: string
  presetId: string
}

type EntryDragState = {
  clientId: string
  moduleId: PromptModuleId
  roleKey: AgentRoleKey
}

const presetEntryOrderStart = 1000
const presetEntryOrderStep = 10

let promptEntryClientIdCounter = 0

function createPromptEntryClientId() {
  promptEntryClientIdCounter += 1
  return `preset-module-entry-${promptEntryClientIdCounter}`
}

function createEmptyModulesState(): ModuleEntriesState {
  return {
    dynamic_context: [],
    output: [],
    role: [],
    static_context: [],
    task: [],
  }
}

function createEmptyAgentState(): AgentFormState {
  return {
    extra: '',
    maxTokens: '',
    modules: createEmptyModulesState(),
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

function createCollapsedModulesState(): Record<AgentRoleKey, Record<PromptModuleId, boolean>> {
  return {
    actor: {
      dynamic_context: false,
      output: false,
      role: false,
      static_context: false,
      task: false,
    },
    architect: {
      dynamic_context: false,
      output: false,
      role: false,
      static_context: false,
      task: false,
    },
    director: {
      dynamic_context: false,
      output: false,
      role: false,
      static_context: false,
      task: false,
    },
    keeper: {
      dynamic_context: false,
      output: false,
      role: false,
      static_context: false,
      task: false,
    },
    narrator: {
      dynamic_context: false,
      output: false,
      role: false,
      static_context: false,
      task: false,
    },
    planner: {
      dynamic_context: false,
      output: false,
      role: false,
      static_context: false,
      task: false,
    },
    replyer: {
      dynamic_context: false,
      output: false,
      role: false,
      static_context: false,
      task: false,
    },
  }
}

function createExpandedEntryIdsState() {
  return [] as string[]
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function sortEntries<T extends { display_name?: string | null; entry_id: string; order: number }>(
  entries: T[],
) {
  return [...entries].sort((left, right) => {
    if (left.order !== right.order) {
      return left.order - right.order
    }

    const leftLabel = (left.display_name ?? left.entry_id).trim()
    const rightLabel = (right.display_name ?? right.entry_id).trim()
    return leftLabel.localeCompare(rightLabel, 'zh-Hans-CN-u-co-pinyin')
  })
}

function createPromptEntryState(entry?: PresetModuleEntry): PromptEntryFormState {
  return {
    clientId: createPromptEntryClientId(),
    contextKey: entry?.context_key ?? '',
    displayName: entry?.display_name ?? '',
    enabled: entry?.enabled ?? true,
    entryId: entry?.entry_id ?? '',
    kind: entry?.kind ?? 'custom_text',
    order: entry?.order !== undefined ? String(entry.order) : String(presetEntryOrderStart),
    required: entry?.required ?? false,
    text: entry?.text ?? '',
  }
}

function renumberModuleEntries(entries: PromptEntryFormState[]) {
  return entries.map((entry, index) => ({
    ...entry,
    order: String(presetEntryOrderStart + index * presetEntryOrderStep),
  }))
}

function createModulesState(modules: PresetDetail['agents'][AgentRoleKey]['modules']): ModuleEntriesState {
  const nextModules = createEmptyModulesState()

  for (const moduleId of promptModuleIds) {
    const existingModule = modules.find((module) => module.module_id === moduleId)
    nextModules[moduleId] = sortEntries(existingModule?.entries ?? []).map((entry) =>
      createPromptEntryState(entry),
    )
  }

  return nextModules
}

function createAgentFormState(agent: PresetDetail['agents'][AgentRoleKey]): AgentFormState {
  return {
    extra:
      agent.extra !== undefined && agent.extra !== null ? JSON.stringify(agent.extra, null, 2) : '',
    maxTokens: agent.max_tokens?.toString() ?? '',
    modules: createModulesState(agent.modules),
    temperature: agent.temperature?.toString() ?? '',
  }
}

function summarizeAgent(
  agent: AgentFormState,
  t: TranslateFn,
  emptyLabel: string,
) {
  const totalEntries = promptModuleIds.reduce(
    (count, moduleId) => count + agent.modules[moduleId].length,
    0,
  )
  const enabledEntries = promptModuleIds.reduce(
    (count, moduleId) =>
      count + agent.modules[moduleId].filter((entry) => entry.enabled).length,
    0,
  )
  const nonEmptyModules = promptModuleIds.filter((moduleId) => agent.modules[moduleId].length > 0)
  const parts = [
    agent.temperature.trim() ? `T ${agent.temperature.trim()}` : null,
    agent.maxTokens.trim() ? `Max ${agent.maxTokens.trim()}` : null,
    agent.extra.trim() ? t('presetsPage.list.extra') : null,
    totalEntries > 0
      ? t('presetsPage.list.moduleSummary', {
          count: nonEmptyModules.length,
          enabled: enabledEntries,
          entries: totalEntries,
        })
      : null,
  ].filter((value): value is string => Boolean(value))

  return parts.length > 0 ? parts : [emptyLabel]
}

function parseEntryOrder(
  roleKey: AgentRoleKey,
  moduleId: PromptModuleId,
  entry: PromptEntryFormState,
  t: TranslateFn,
  roleLabels: Record<AgentRoleKey, string>,
) {
  const parsed = Number(entry.order.trim())

  if (!Number.isInteger(parsed)) {
      throw new Error(
        t('presetsPage.form.errors.entryOrderInvalid', {
          id: entry.entryId.trim() || entry.displayName.trim() || '—',
          module: getPromptModuleLabel(t, moduleId),
          role: roleLabels[roleKey],
        }),
      )
  }

  return parsed
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

  const modules = promptModuleIds
    .map((moduleId) => {
      const seenEntryIds = new Set<string>()
      const entries = agent.modules[moduleId].map((entry, index) => {
        const entryId = entry.entryId.trim()
        const displayName = entry.displayName.trim()
        const text = entry.text.trim()
        const contextKey = entry.contextKey.trim()
        const order = parseEntryOrder(roleKey, moduleId, entry, t, roleLabels)

        if (!entryId) {
          throw new Error(
            t('presetsPage.form.errors.entryIdRequired', {
              index: index + 1,
              module: getPromptModuleLabel(t, moduleId),
              role: roleLabels[roleKey],
            }),
          )
        }

        if (seenEntryIds.has(entryId)) {
          throw new Error(
            t('presetsPage.form.errors.duplicateEntryId', {
              id: entryId,
              module: getPromptModuleLabel(t, moduleId),
              role: roleLabels[roleKey],
            }),
          )
        }
        seenEntryIds.add(entryId)

        if (entry.kind === 'custom_text') {
          if (!displayName) {
            throw new Error(
              t('presetsPage.form.errors.entryDisplayNameRequired', {
                index: index + 1,
                module: getPromptModuleLabel(t, moduleId),
                role: roleLabels[roleKey],
              }),
            )
          }

          if (!text) {
            throw new Error(
              t('presetsPage.form.errors.customEntryTextRequired', {
                index: index + 1,
                module: getPromptModuleLabel(t, moduleId),
                role: roleLabels[roleKey],
              }),
            )
          }
        }

        return {
          ...(contextKey ? { context_key: contextKey } : {}),
          ...(text ? { text } : {}),
          display_name: displayName || entry.displayName.trim() || entryId,
          enabled: entry.enabled,
          entry_id: entryId,
          kind: entry.kind,
          order,
          required: entry.required,
        } satisfies PresetModuleEntry
      })

      return {
        entries,
        module_id: moduleId,
      }
    })
    .filter((module) => module.entries.length > 0)

  return {
    ...(temperature !== undefined ? { temperature } : {}),
    ...(maxTokens !== undefined ? { max_tokens: maxTokens } : {}),
    ...(extra !== undefined ? { extra } : {}),
    modules,
  }
}

function toPresetAgents(
  agents: FormState['agents'],
  t: TranslateFn,
  roleLabels: Record<AgentRoleKey, string>,
) {
  return Object.fromEntries(
    getOrderedAgentRoleKeys().map((roleKey) => [
      roleKey,
      parseAgentPresetConfig(roleKey, agents[roleKey], t, roleLabels),
    ]),
  ) as PresetAgentConfigs
}

function areJsonValuesEqual(left: unknown, right: unknown) {
  return JSON.stringify(left ?? null) === JSON.stringify(right ?? null)
}

function haveAgentSettingsChanged(
  originalAgents: PresetDetail['agents'],
  nextAgents: PresetAgentConfigs,
) {
  return getOrderedAgentRoleKeys().some((roleKey) => {
    const originalAgent = originalAgents[roleKey]
    const nextAgent = nextAgents[roleKey]

    return (
      (originalAgent.temperature ?? null) !== (nextAgent.temperature ?? null) ||
      (originalAgent.max_tokens ?? null) !== (nextAgent.max_tokens ?? null) ||
      !areJsonValuesEqual(originalAgent.extra, nextAgent.extra)
    )
  })
}

type FlatPresetEntry = {
  agent: AgentRoleKey
  display_name: string
  enabled: boolean
  entry_id: string
  kind: PresetEntryKind
  module_id: PromptModuleId
  order: number
  required: boolean
  text?: string | null
}

function flattenPresetEntries(agents: PresetAgentConfigs) {
  return getOrderedAgentRoleKeys().flatMap((roleKey) =>
    agents[roleKey].modules.flatMap((module) =>
      module.entries.map((entry) => ({
        agent: roleKey,
        display_name: entry.display_name,
        enabled: entry.enabled,
        entry_id: entry.entry_id,
        kind: entry.kind,
        module_id: module.module_id,
        order: entry.order,
        required: entry.required,
        text: entry.text ?? null,
      })),
    ),
  )
}

function getFlatEntryKey(entry: Pick<FlatPresetEntry, 'agent' | 'module_id' | 'entry_id'>) {
  return `${entry.agent}:${entry.module_id}:${entry.entry_id}`
}

function createEntryDiffs(original: PresetDetail['agents'], current: PresetAgentConfigs) {
  const originalEntries = flattenPresetEntries(original)
  const currentEntries = flattenPresetEntries(current)

  const originalMap = new Map(originalEntries.map((entry) => [getFlatEntryKey(entry), entry]))
  const currentMap = new Map(currentEntries.map((entry) => [getFlatEntryKey(entry), entry]))

  const createEntries = currentEntries.filter(
    (entry) => entry.kind === 'custom_text' && !originalMap.has(getFlatEntryKey(entry)),
  )
  const deleteEntries = originalEntries.filter(
    (entry) => entry.kind === 'custom_text' && !currentMap.has(getFlatEntryKey(entry)),
  )
  const updateEntries = currentEntries.filter((entry) => {
    const originalEntry = originalMap.get(getFlatEntryKey(entry))

    if (!originalEntry) {
      return false
    }

    if (entry.kind === 'custom_text') {
      return (
        entry.display_name !== originalEntry.display_name ||
        (entry.text ?? null) !== (originalEntry.text ?? null) ||
        entry.enabled !== originalEntry.enabled ||
        entry.order !== originalEntry.order
      )
    }

    return entry.enabled !== originalEntry.enabled || entry.order !== originalEntry.order
  })

  return {
    createEntries,
    deleteEntries,
    updateEntries,
  }
}

function createModuleEntryLabel(entry: PromptEntryFormState, t: TranslateFn, index: number) {
  if (entry.displayName.trim()) {
    return entry.displayName.trim()
  }

  return t('presetsPage.form.untitledEntry', { index })
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
  const translate = useCallback(
    (key: string, options?: Record<string, unknown>) =>
      String(t(key as never, options as never)),
    [t],
  )
  const roleLabels = useMemo(() => createPresetRoleLabels(translate), [translate])
  const [formState, setFormState] = useState<FormState>(createInitialState)
  const [originalPreset, setOriginalPreset] = useState<PresetDetail | null>(null)
  const [expandedAgents, setExpandedAgents] = useState<Record<AgentRoleKey, boolean>>(
    createCollapsedAgentsState,
  )
  const [expandedModelSettings, setExpandedModelSettings] = useState<Record<AgentRoleKey, boolean>>(
    createCollapsedAgentsState,
  )
  const [expandedModules, setExpandedModules] = useState<
    Record<AgentRoleKey, Record<PromptModuleId, boolean>>
  >(createCollapsedModulesState)
  const [expandedEntryIds, setExpandedEntryIds] = useState<string[]>(
    createExpandedEntryIdsState,
  )
  const [draggedEntry, setDraggedEntry] = useState<EntryDragState | null>(null)
  const [dropTarget, setDropTarget] = useState<EntryDragState | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  useToastMessage(submitError)

  async function loadPresetFromServer(nextPresetId: string, signal?: AbortSignal) {
    const result = await getPreset(nextPresetId, signal)

    if (signal?.aborted) {
      return null
    }

    setOriginalPreset(result)
    setFormState({
      agents: Object.fromEntries(
        getOrderedAgentRoleKeys().map((roleKey) => [roleKey, createAgentFormState(result.agents[roleKey])]),
      ) as FormState['agents'],
      displayName: result.display_name,
      presetId: result.preset_id,
    })
    setExpandedAgents(createCollapsedAgentsState())
    setExpandedModelSettings(createCollapsedAgentsState())
    setExpandedModules(createCollapsedModulesState())
    setExpandedEntryIds(createExpandedEntryIdsState())

    return result
  }

  useEffect(() => {
    if (!open) {
      setFormState(createInitialState())
      setOriginalPreset(null)
      setExpandedAgents(createCollapsedAgentsState())
      setExpandedModelSettings(createCollapsedAgentsState())
      setExpandedModules(createCollapsedModulesState())
      setExpandedEntryIds(createExpandedEntryIdsState())
      setDraggedEntry(null)
      setDropTarget(null)
      setIsLoading(false)
      setIsSubmitting(false)
      setSubmitError(null)
      return
    }

    if (mode !== 'edit' || !presetId) {
      setFormState(createInitialState())
      setOriginalPreset(null)
      setExpandedAgents(createCollapsedAgentsState())
      setExpandedModelSettings(createCollapsedAgentsState())
      setExpandedModules(createCollapsedModulesState())
      setExpandedEntryIds(createExpandedEntryIdsState())
      setDraggedEntry(null)
      setDropTarget(null)
      setIsLoading(false)
      setSubmitError(null)
      return
    }

    const controller = new AbortController()
    setIsLoading(true)
    setSubmitError(null)
    setDraggedEntry(null)
    setDropTarget(null)

    void loadPresetFromServer(presetId, controller.signal)
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

  function updateAgentField(roleKey: AgentRoleKey, key: keyof Omit<AgentFormState, 'modules'>, value: string) {
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

  function toggleModule(roleKey: AgentRoleKey, moduleId: PromptModuleId) {
    setExpandedModules((current) => ({
      ...current,
      [roleKey]: {
        ...current[roleKey],
        [moduleId]: !current[roleKey][moduleId],
      },
    }))
  }

  function updateEntryField(
    roleKey: AgentRoleKey,
    moduleId: PromptModuleId,
    clientId: string,
    key: 'displayName' | 'entryId' | 'order' | 'text',
    value: string,
  ) {
    setFormState((current) => ({
      ...current,
      agents: {
        ...current.agents,
        [roleKey]: {
          ...current.agents[roleKey],
          modules: {
            ...current.agents[roleKey].modules,
            [moduleId]: current.agents[roleKey].modules[moduleId].map((entry) =>
              entry.clientId === clientId ? { ...entry, [key]: value } : entry,
            ),
          },
        },
      },
    }))
  }

  function updateEntryEnabled(
    roleKey: AgentRoleKey,
    moduleId: PromptModuleId,
    clientId: string,
    enabled: boolean,
  ) {
    setFormState((current) => ({
      ...current,
      agents: {
        ...current.agents,
        [roleKey]: {
          ...current.agents[roleKey],
          modules: {
            ...current.agents[roleKey].modules,
            [moduleId]: current.agents[roleKey].modules[moduleId].map((entry) =>
              entry.clientId === clientId ? { ...entry, enabled } : entry,
            ),
          },
        },
      },
    }))
  }

  function addCustomEntry(roleKey: AgentRoleKey, moduleId: PromptModuleId) {
    const entries = formState.agents[roleKey].modules[moduleId]
    const highestOrder = entries.reduce((maxOrder, entry) => {
      const parsed = Number(entry.order)
      return Number.isInteger(parsed) ? Math.max(maxOrder, parsed) : maxOrder
    }, presetEntryOrderStart - presetEntryOrderStep)
    const nextEntry: PromptEntryFormState = {
      clientId: createPromptEntryClientId(),
      contextKey: '',
      displayName: '',
      enabled: true,
      entryId: '',
      kind: 'custom_text',
      order: String(highestOrder + presetEntryOrderStep),
      required: false,
      text: '',
    }

    setFormState((current) => ({
      ...current,
      agents: {
        ...current.agents,
        [roleKey]: {
          ...current.agents[roleKey],
          modules: {
            ...current.agents[roleKey].modules,
            [moduleId]: [...current.agents[roleKey].modules[moduleId], nextEntry],
          },
        },
      },
    }))
    setExpandedAgents((current) => ({ ...current, [roleKey]: true }))
    setExpandedModules((current) => ({
      ...current,
      [roleKey]: {
        ...current[roleKey],
        [moduleId]: true,
      },
    }))
    setExpandedEntryIds((current) =>
      current.includes(nextEntry.clientId) ? current : [...current, nextEntry.clientId],
    )
  }

  function toggleEntry(clientId: string) {
    setExpandedEntryIds((current) =>
      current.includes(clientId)
        ? current.filter((currentId) => currentId !== clientId)
        : [...current, clientId],
    )
  }

  function removeEntry(roleKey: AgentRoleKey, moduleId: PromptModuleId, clientId: string) {
    setFormState((current) => ({
      ...current,
      agents: {
        ...current.agents,
        [roleKey]: {
          ...current.agents[roleKey],
          modules: {
            ...current.agents[roleKey].modules,
            [moduleId]: current.agents[roleKey].modules[moduleId].filter(
              (entry) => entry.clientId !== clientId,
            ),
          },
        },
      },
    }))
    setExpandedEntryIds((current) => current.filter((currentId) => currentId !== clientId))

    if (draggedEntry?.clientId === clientId && draggedEntry.roleKey === roleKey && draggedEntry.moduleId === moduleId) {
      setDraggedEntry(null)
    }

    if (dropTarget?.clientId === clientId && dropTarget.roleKey === roleKey && dropTarget.moduleId === moduleId) {
      setDropTarget(null)
    }
  }

  function moveEntry(
    roleKey: AgentRoleKey,
    moduleId: PromptModuleId,
    sourceClientId: string,
    targetClientId: string,
  ) {
    if (sourceClientId === targetClientId) {
      return
    }

    setFormState((current) => {
      const entries = current.agents[roleKey].modules[moduleId]
      const sourceIndex = entries.findIndex((entry) => entry.clientId === sourceClientId)
      const targetIndex = entries.findIndex((entry) => entry.clientId === targetClientId)

      if (sourceIndex === -1 || targetIndex === -1) {
        return current
      }

      const nextEntries = [...entries]
      const [movedEntry] = nextEntries.splice(sourceIndex, 1)
      nextEntries.splice(targetIndex, 0, movedEntry)
      const renumberedEntries = renumberModuleEntries(nextEntries)

      return {
        ...current,
        agents: {
          ...current.agents,
          [roleKey]: {
            ...current.agents[roleKey],
            modules: {
              ...current.agents[roleKey].modules,
              [moduleId]: renumberedEntries,
            },
          },
        },
      }
    })
  }

  function handleEntryDragStart(roleKey: AgentRoleKey, moduleId: PromptModuleId, clientId: string) {
    setDraggedEntry({ clientId, moduleId, roleKey })
    setDropTarget(null)
  }

  function handleEntryDragOver(
    event: DragEvent<HTMLDivElement>,
    roleKey: AgentRoleKey,
    moduleId: PromptModuleId,
    clientId: string,
  ) {
    if (
      !draggedEntry ||
      draggedEntry.roleKey !== roleKey ||
      draggedEntry.moduleId !== moduleId ||
      draggedEntry.clientId === clientId
    ) {
      return
    }

    event.preventDefault()

    if (
      dropTarget?.clientId !== clientId ||
      dropTarget.roleKey !== roleKey ||
      dropTarget.moduleId !== moduleId
    ) {
      setDropTarget({ clientId, moduleId, roleKey })
    }
  }

  function handleEntryDrop(
    event: DragEvent<HTMLDivElement>,
    roleKey: AgentRoleKey,
    moduleId: PromptModuleId,
    clientId: string,
  ) {
    if (
      !draggedEntry ||
      draggedEntry.roleKey !== roleKey ||
      draggedEntry.moduleId !== moduleId
    ) {
      return
    }

    event.preventDefault()
    moveEntry(roleKey, moduleId, draggedEntry.clientId, clientId)
    setDraggedEntry(null)
    setDropTarget(null)
  }

  function handleEntryDragEnd() {
    setDraggedEntry(null)
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
      const nextAgents = toPresetAgents(formState.agents, translate, roleLabels)
      const nextPresetId = formState.presetId.trim()
      const nextDisplayName = formState.displayName.trim()

      const preset =
        mode === 'create'
          ? await createPreset({
              agents: nextAgents,
              display_name: nextDisplayName,
              preset_id: nextPresetId,
            })
          : await (async () => {
              if (!originalPreset) {
                throw new Error(t('presetsPage.feedback.loadPresetFailed'))
              }

              const { createEntries, deleteEntries, updateEntries } = createEntryDiffs(
                originalPreset.agents,
                nextAgents,
              )

              for (const entry of createEntries) {
                await createPresetEntry({
                  agent: entry.agent,
                  display_name: entry.display_name,
                  enabled: entry.enabled,
                  entry_id: entry.entry_id,
                  module_id: entry.module_id,
                  order: entry.order,
                  preset_id: nextPresetId,
                  text: entry.text ?? '',
                })
              }

              for (const entry of updateEntries) {
                if (entry.kind === 'custom_text') {
                  await updatePresetEntry({
                    agent: entry.agent,
                    display_name: entry.display_name,
                    enabled: entry.enabled,
                    entry_id: entry.entry_id,
                    module_id: entry.module_id,
                    order: entry.order,
                    preset_id: nextPresetId,
                    text: entry.text ?? '',
                  })
                } else {
                  await updatePresetEntry({
                    agent: entry.agent,
                    enabled: entry.enabled,
                    entry_id: entry.entry_id,
                    module_id: entry.module_id,
                    order: entry.order,
                    preset_id: nextPresetId,
                  })
                }
              }

              for (const entry of deleteEntries) {
                await deletePresetEntry({
                  agent: entry.agent,
                  entry_id: entry.entry_id,
                  module_id: entry.module_id,
                  preset_id: nextPresetId,
                })
              }

              const needsPresetUpdate =
                nextDisplayName !== originalPreset.display_name ||
                haveAgentSettingsChanged(originalPreset.agents, nextAgents)

              return needsPresetUpdate
                ? await updatePreset({
                    agents: nextAgents,
                    display_name: nextDisplayName,
                    preset_id: nextPresetId,
                  })
                : await getPreset(nextPresetId)
            })()

      setOriginalPreset(preset)

      await onCompleted({
        message:
          mode === 'create'
            ? t('presetsPage.feedback.created', { id: preset.display_name })
            : t('presetsPage.feedback.updated', { id: preset.display_name }),
        preset,
      })

      onOpenChange(false)
    } catch (error) {
      if (mode === 'edit' && formState.presetId.trim()) {
        try {
          await loadPresetFromServer(formState.presetId.trim())
        } catch {
          // Keep the original error message and preserve current visible state if refresh fails.
        }
      }

      setSubmitError(getErrorMessage(error, t('presetsPage.form.errors.submitFailed')))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent
        aria-describedby={undefined}
        contentClassName="w-[min(96vw,76rem)]"
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
            {mode === 'create' ? t('presetsPage.form.createTitle') : t('presetsPage.form.editTitle')}
          </DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          {isLoading ? (
            <div className="space-y-3">
              <div className="h-12 animate-pulse rounded-[1.35rem] bg-[var(--color-bg-elevated)]" />
              {Array.from({ length: getOrderedAgentRoleKeys().length }).map((_, index) => (
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
                {getOrderedAgentRoleKeys().map((roleKey) => {
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
                                  type="button"
                                >
                                  <p className="text-sm font-medium text-[var(--color-text-primary)]">
                                    {t('presetsPage.form.fields.modelSettings')}
                                  </p>

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
                                                updateAgentField(roleKey, 'temperature', event.target.value)
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
                                                updateAgentField(roleKey, 'maxTokens', event.target.value)
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
                                              updateAgentField(roleKey, 'extra', event.target.value)
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
                                {promptModuleIds.map((moduleId) => {
                                  const moduleEntries = agentState.modules[moduleId]
                                  const enabledCount = moduleEntries.filter((entry) => entry.enabled).length
                                  const isModuleExpanded = expandedModules[roleKey][moduleId]

                                  return (
                                    <div
                                      className="overflow-hidden rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_72%,transparent)]"
                                      key={`${roleKey}:${moduleId}`}
                                    >
                                      <button
                                        aria-expanded={isModuleExpanded}
                                        className="flex w-full items-center justify-between gap-4 px-4 py-4 text-left transition duration-200 hover:bg-white/5"
                                        onClick={() => {
                                          toggleModule(roleKey, moduleId)
                                        }}
                                        type="button"
                                      >
                                        <div className="min-w-0 space-y-1">
                                          <p className="text-sm font-medium text-[var(--color-text-primary)]">
                                            {getPromptModuleLabel(translate, moduleId)}
                                          </p>
                                          <p className="text-xs leading-6 text-[var(--color-text-muted)]">
                                            {t('presetsPage.form.moduleSummary', {
                                              count: moduleEntries.length,
                                              enabled: enabledCount,
                                            })}
                                          </p>
                                        </div>

                                        <motion.span
                                          animate={{ rotate: isModuleExpanded ? 180 : 0 }}
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
                                        {isModuleExpanded ? (
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
                                              <div className="flex flex-wrap items-start justify-between gap-3">
                                                <p className="text-xs leading-6 text-[var(--color-text-muted)]">
                                                  {t('presetsPage.form.moduleHint')}
                                                </p>
                                                <Button
                                                  onClick={() => {
                                                    addCustomEntry(roleKey, moduleId)
                                                  }}
                                                  size="sm"
                                                  variant="secondary"
                                                >
                                                  <FontAwesomeIcon icon={faPlus} />
                                                  {t('presetsPage.actions.addCustomEntry')}
                                                </Button>
                                              </div>

                                              {moduleEntries.length > 0 ? (
                                                <div className="space-y-3">
                                                  {moduleEntries.map((entry, index) => {
                                                    const isExpanded = expandedEntryIds.includes(entry.clientId)
                                                    const isDragged =
                                                      draggedEntry?.clientId === entry.clientId &&
                                                      draggedEntry.roleKey === roleKey &&
                                                      draggedEntry.moduleId === moduleId
                                                    const isDropTarget =
                                                      dropTarget?.clientId === entry.clientId &&
                                                      dropTarget.roleKey === roleKey &&
                                                      dropTarget.moduleId === moduleId &&
                                                      !isDragged
                                                    const canEditText = entry.kind === 'custom_text'
                                                    const canEditDisplayName = entry.kind === 'custom_text'
                                                    const canEditEntryId = entry.kind === 'custom_text'
                                                    const canRemove = entry.kind === 'custom_text'

                                                    return (
                                                      <motion.div
                                                        layout
                                                        className={cn(
                                                          'rounded-[1.1rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4 transition duration-200',
                                                          isDragged && 'opacity-55',
                                                          isDropTarget &&
                                                            'border-[var(--color-accent-gold-line)] shadow-[0_0_0_1px_var(--color-accent-gold-line)]',
                                                        )}
                                                        key={entry.clientId}
                                                        onDragOver={(event) => {
                                                          handleEntryDragOver(event, roleKey, moduleId, entry.clientId)
                                                        }}
                                                        onDrop={(event) => {
                                                          handleEntryDrop(event, roleKey, moduleId, entry.clientId)
                                                        }}
                                                        transition={
                                                          prefersReducedMotion
                                                            ? { duration: 0 }
                                                            : {
                                                                duration: 0.22,
                                                                ease: [0.22, 1, 0.36, 1],
                                                              }
                                                        }
                                                      >
                                                        <div className="flex flex-wrap items-center justify-between gap-3">
                                                          <div className="flex min-w-0 items-center gap-3">
                                                            <button
                                                              aria-label={t('presetsPage.actions.dragEntry')}
                                                              className="inline-flex h-9 w-9 shrink-0 cursor-grab items-center justify-center rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-[var(--color-text-secondary)] transition duration-200 hover:text-[var(--color-text-primary)] active:cursor-grabbing"
                                                              draggable
                                                              onDragEnd={handleEntryDragEnd}
                                                              onDragStart={(event) => {
                                                                event.dataTransfer.effectAllowed = 'move'
                                                                event.dataTransfer.setData('text/plain', entry.clientId)
                                                                handleEntryDragStart(roleKey, moduleId, entry.clientId)
                                                              }}
                                                              type="button"
                                                            >
                                                              <FontAwesomeIcon icon={faGripVertical} />
                                                            </button>

                                                            <button
                                                              aria-expanded={isExpanded}
                                                              className="flex min-w-0 flex-1 flex-wrap items-center gap-2.5 text-left"
                                                              onClick={() => {
                                                                toggleEntry(entry.clientId)
                                                              }}
                                                              type="button"
                                                            >
                                                              <p className="truncate text-sm font-medium text-[var(--color-text-primary)]">
                                                                {createModuleEntryLabel(entry, translate, index + 1)}
                                                              </p>
                                                              <Badge variant="subtle">
                                                                {getPresetEntryKindLabel(translate, entry.kind)}
                                                              </Badge>
                                                              <Badge variant="subtle">
                                                                {entry.entryId.trim() ||
                                                                  t('presetsPage.form.newEntry')}
                                                              </Badge>
                                                              {entry.required ? (
                                                                <Badge variant="info">
                                                                  {t('presetsPage.details.required')}
                                                                </Badge>
                                                              ) : null}
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
                                                                  ? t('presetsPage.actions.collapseEntry')
                                                                  : t('presetsPage.actions.expandEntry')
                                                              }
                                                              onClick={() => {
                                                                toggleEntry(entry.clientId)
                                                              }}
                                                              size="sm"
                                                              variant="secondary"
                                                            />
                                                            <span className="text-xs text-[var(--color-text-muted)]">
                                                              {t('presetsPage.form.fields.entryEnabled')}
                                                            </span>
                                                            <Switch
                                                              checked={entry.enabled}
                                                              onCheckedChange={(enabled) => {
                                                                updateEntryEnabled(roleKey, moduleId, entry.clientId, enabled)
                                                              }}
                                                              size="sm"
                                                            />
                                                            {canRemove ? (
                                                              <IconButton
                                                                icon={<FontAwesomeIcon icon={faTrashCan} />}
                                                                label={t('presetsPage.actions.removeEntry')}
                                                                onClick={() => {
                                                                  removeEntry(roleKey, moduleId, entry.clientId)
                                                                }}
                                                                size="sm"
                                                                variant="danger"
                                                              />
                                                            ) : null}
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
                                                                  : {
                                                                      duration: 0.2,
                                                                      ease: [0.22, 1, 0.36, 1],
                                                                    }
                                                              }
                                                            >
                                                              <div className="mt-4 space-y-4 border-t border-[var(--color-border-subtle)] pt-4">
                                                                <div className="grid gap-4 md:grid-cols-2">
                                                                  <label className="space-y-2">
                                                                    <span className="block text-xs text-[var(--color-text-muted)]">
                                                                      {t('presetsPage.form.fields.entryId')}
                                                                    </span>
                                                                    <Input
                                                                      disabled={!canEditEntryId}
                                                                      id={`preset-form-${roleKey}-${moduleId}-${entry.clientId}-id`}
                                                                      name={`${roleKey}_${moduleId}_entry_id_${index}`}
                                                                      onChange={(event) => {
                                                                        updateEntryField(
                                                                          roleKey,
                                                                          moduleId,
                                                                          entry.clientId,
                                                                          'entryId',
                                                                          event.target.value,
                                                                        )
                                                                      }}
                                                                      placeholder={t('presetsPage.form.placeholders.entryId')}
                                                                      value={entry.entryId}
                                                                    />
                                                                  </label>

                                                                  <label className="space-y-2">
                                                                    <span className="block text-xs text-[var(--color-text-muted)]">
                                                                      {t('presetsPage.form.fields.order')}
                                                                    </span>
                                                                    <Input
                                                                      id={`preset-form-${roleKey}-${moduleId}-${entry.clientId}-order`}
                                                                      name={`${roleKey}_${moduleId}_entry_order_${index}`}
                                                                      onChange={(event) => {
                                                                        updateEntryField(
                                                                          roleKey,
                                                                          moduleId,
                                                                          entry.clientId,
                                                                          'order',
                                                                          event.target.value,
                                                                        )
                                                                      }}
                                                                      placeholder={t('presetsPage.form.placeholders.order')}
                                                                      value={entry.order}
                                                                    />
                                                                  </label>
                                                                </div>

                                                                <label className="space-y-2">
                                                                  <span className="block text-xs text-[var(--color-text-muted)]">
                                                                    {t('presetsPage.form.fields.entryDisplayName')}
                                                                  </span>
                                                                  <Input
                                                                    disabled={!canEditDisplayName}
                                                                    id={`preset-form-${roleKey}-${moduleId}-${entry.clientId}-display-name`}
                                                                    name={`${roleKey}_${moduleId}_entry_display_name_${index}`}
                                                                    onChange={(event) => {
                                                                      updateEntryField(
                                                                        roleKey,
                                                                        moduleId,
                                                                        entry.clientId,
                                                                        'displayName',
                                                                        event.target.value,
                                                                      )
                                                                    }}
                                                                    placeholder={t('presetsPage.form.placeholders.entryDisplayName')}
                                                                    value={entry.displayName}
                                                                  />
                                                                </label>

                                                                {entry.kind === 'built_in_context_ref' ? (
                                                                  <div className="space-y-2">
                                                                    <span className="block text-xs text-[var(--color-text-muted)]">
                                                                      {t('presetsPage.form.fields.contextKey')}
                                                                    </span>
                                                                    <div className="rounded-[1rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-4 py-3 text-sm text-[var(--color-text-primary)]">
                                                                      {entry.contextKey || '—'}
                                                                    </div>
                                                                  </div>
                                                                ) : (
                                                                  <label className="block space-y-2">
                                                                    <span className="block text-xs text-[var(--color-text-muted)]">
                                                                      {t('presetsPage.form.fields.entryText')}
                                                                    </span>
                                                                    <Textarea
                                                                      className="min-h-[8rem]"
                                                                      disabled={!canEditText}
                                                                      id={`preset-form-${roleKey}-${moduleId}-${entry.clientId}-text`}
                                                                      name={`${roleKey}_${moduleId}_entry_text_${index}`}
                                                                      onChange={(event) => {
                                                                        updateEntryField(
                                                                          roleKey,
                                                                          moduleId,
                                                                          entry.clientId,
                                                                          'text',
                                                                          event.target.value,
                                                                        )
                                                                      }}
                                                                      placeholder={t('presetsPage.form.placeholders.entryText')}
                                                                      value={entry.text}
                                                                    />
                                                                  </label>
                                                                )}
                                                              </div>
                                                            </motion.div>
                                                          ) : null}
                                                        </AnimatePresence>
                                                      </motion.div>
                                                    )
                                                  })}
                                                </div>
                                              ) : (
                                                <div className="rounded-[1.1rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_62%,transparent)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
                                                  {mode === 'create'
                                                    ? t('presetsPage.form.emptyModuleEntriesCreate')
                                                    : t('presetsPage.form.emptyModuleEntries')}
                                                </div>
                                              )}
                                            </div>
                                          </motion.div>
                                        ) : null}
                                      </AnimatePresence>
                                    </div>
                                  )
                                })}
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
          <Button disabled={isSubmitting} onClick={() => onOpenChange(false)} variant="ghost">
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
