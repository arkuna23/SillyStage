import { type DragEvent, useCallback, useEffect, useMemo, useState } from 'react'
import { faChevronDown } from '@fortawesome/free-solid-svg-icons/faChevronDown'
import { faEye } from '@fortawesome/free-solid-svg-icons/faEye'
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
import { Select } from '../../components/ui/select'
import { Switch } from '../../components/ui/switch'
import { Textarea } from '../../components/ui/textarea'
import { useToastMessage, useToastNotice, type ToastInput } from '../../components/ui/toast-context'
import { cn } from '../../lib/cn'
import { createPreset, getPreset, updatePreset } from '../apis/api'
import {
  promptMessageRoles,
  type AgentPresetConfig,
  type AgentRoleKey,
  type PresetAgentConfigs,
  type PresetDetail,
  type PresetEntryKind,
  type PresetModuleEntry,
  type PromptMessageRole,
  type PromptModuleId,
} from '../apis/types'
import {
  createPresetRoleLabels,
  getBuiltInPromptModuleDefinition,
  getOrderedAgentRoleKeys,
  getOrderedModuleIds,
  getPromptMessageRoleLabel,
  getPromptModuleDefaultDisplayName,
  getPromptModuleLabel,
  getPresetEntryKindLabel,
  isBuiltInPromptModuleId,
} from './preset-labels'
import { PresetPromptPreviewDialog } from './preset-prompt-preview-dialog'

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

type PromptModuleFormState = {
  clientId: string
  displayName: string
  entries: PromptEntryFormState[]
  isBuiltIn: boolean
  messageRole: PromptMessageRole
  moduleId: string
  order: string
}

type AgentFormState = {
  extra: string
  maxTokens: string
  modules: PromptModuleFormState[]
  temperature: string
}

type FormState = {
  agents: Record<AgentRoleKey, AgentFormState>
  displayName: string
  presetId: string
}

type EntryDragState = {
  clientId: string
  moduleClientId: string
  roleKey: AgentRoleKey
}

type ModuleDragState = {
  clientId: string
  roleKey: AgentRoleKey
}

type PreviewDialogState = {
  initialAgent: AgentRoleKey
  initialModuleId?: string
  scopeLabel?: string
}

const presetModuleOrderStart = 10
const presetModuleOrderStep = 10
const presetEntryOrderStart = 1000
const presetEntryOrderStep = 10

let promptEntryClientIdCounter = 0
let promptModuleClientIdCounter = 0

function createPromptEntryClientId() {
  promptEntryClientIdCounter += 1
  return `preset-module-entry-${promptEntryClientIdCounter}`
}

function createPromptModuleClientId() {
  promptModuleClientIdCounter += 1
  return `preset-module-${promptModuleClientIdCounter}`
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

    return left.entry_id.localeCompare(right.entry_id, 'zh-Hans-CN-u-co-pinyin')
  })
}

function sortModules<T extends { display_name?: string | null; module_id: string; order: number }>(
  modules: T[],
) {
  return [...modules].sort((left, right) => {
    if (left.order !== right.order) {
      return left.order - right.order
    }

    return left.module_id.localeCompare(right.module_id, 'zh-Hans-CN-u-co-pinyin')
  })
}

function sortModuleStates(modules: PromptModuleFormState[]) {
  return [...modules].sort((left, right) => {
    const leftOrder = Number(left.order)
    const rightOrder = Number(right.order)

    if (Number.isInteger(leftOrder) && Number.isInteger(rightOrder) && leftOrder !== rightOrder) {
      return leftOrder - rightOrder
    }

    return left.moduleId.localeCompare(right.moduleId, 'zh-Hans-CN-u-co-pinyin')
  })
}

function renumberModuleEntries(entries: PromptEntryFormState[]) {
  return entries.map((entry, index) => ({
    ...entry,
    order: String(presetEntryOrderStart + index * presetEntryOrderStep),
  }))
}

function renumberModules(modules: PromptModuleFormState[]) {
  return modules.map((module, index) => ({
    ...module,
    order: String(presetModuleOrderStart + index * presetModuleOrderStep),
  }))
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

function createBuiltInModuleState(
  t: TranslateFn,
  moduleId: PromptModuleId,
): PromptModuleFormState {
  if (!isBuiltInPromptModuleId(moduleId)) {
    return {
      clientId: createPromptModuleClientId(),
      displayName: '',
      entries: [],
      isBuiltIn: false,
      messageRole: 'user',
      moduleId,
      order: String(1000),
    }
  }

  const definition = getBuiltInPromptModuleDefinition(moduleId)

  return {
    clientId: createPromptModuleClientId(),
    displayName: getPromptModuleDefaultDisplayName(t, moduleId, definition.defaultDisplayName),
    entries: [],
    isBuiltIn: true,
    messageRole: definition.messageRole,
    moduleId,
    order: String(definition.order),
  }
}

function createPromptModuleState(
  module: AgentPresetConfig['modules'][number],
): PromptModuleFormState {
  return {
    clientId: createPromptModuleClientId(),
    displayName: module.display_name,
    entries: sortEntries(module.entries).map((entry) => createPromptEntryState(entry)),
    isBuiltIn: isBuiltInPromptModuleId(module.module_id),
    messageRole: module.message_role,
    moduleId: module.module_id,
    order: String(module.order),
  }
}

function createModulesState(
  modules: PresetDetail['agents'][AgentRoleKey]['modules'],
  t: TranslateFn,
) {
  const nextModules = sortModules(modules).map((module) => createPromptModuleState(module))
  const existingIds = new Set(nextModules.map((module) => module.moduleId))

  for (const moduleId of getOrderedModuleIds()) {
    if (!existingIds.has(moduleId)) {
      nextModules.push(createBuiltInModuleState(t, moduleId))
    }
  }

  return sortModuleStates(nextModules)
}

function createEmptyAgentState(t: TranslateFn): AgentFormState {
  return {
    extra: '',
    maxTokens: '',
    modules: getOrderedModuleIds().map((moduleId) => createBuiltInModuleState(t, moduleId)),
    temperature: '',
  }
}

function createInitialState(t: TranslateFn): FormState {
  return {
    agents: getOrderedAgentRoleKeys().reduce<FormState['agents']>((agents, roleKey) => {
      agents[roleKey] = createEmptyAgentState(t)
      return agents
    }, {} as FormState['agents']),
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

function createExpandedModulesState(): Record<AgentRoleKey, string[]> {
  return {
    actor: [],
    architect: [],
    director: [],
    keeper: [],
    narrator: [],
    planner: [],
    replyer: [],
  }
}

function createExpandedEntryIdsState() {
  return [] as string[]
}

function createAgentFormState(agent: PresetDetail['agents'][AgentRoleKey], t: TranslateFn): AgentFormState {
  return {
    extra:
      agent.extra !== undefined && agent.extra !== null ? JSON.stringify(agent.extra, null, 2) : '',
    maxTokens: agent.max_tokens?.toString() ?? '',
    modules: createModulesState(agent.modules, t),
    temperature: agent.temperature?.toString() ?? '',
  }
}

function summarizeAgent(
  agent: AgentFormState,
  t: TranslateFn,
  emptyLabel: string,
) {
  const totalEntries = agent.modules.reduce((count, module) => count + module.entries.length, 0)
  const enabledEntries = agent.modules.reduce(
    (count, module) => count + module.entries.filter((entry) => entry.enabled).length,
    0,
  )
  const nonEmptyModules = agent.modules.filter((module) => module.entries.length > 0).length
  const parts = [
    agent.temperature.trim() ? `T ${agent.temperature.trim()}` : null,
    agent.maxTokens.trim() ? `Max ${agent.maxTokens.trim()}` : null,
    agent.extra.trim() ? t('presetsPage.list.extra') : null,
    totalEntries > 0
      ? t('presetsPage.list.moduleSummary', {
          count: nonEmptyModules,
          enabled: enabledEntries,
          entries: totalEntries,
        })
      : null,
  ].filter((value): value is string => Boolean(value))

  return parts.length > 0 ? parts : [emptyLabel]
}

function parseModuleOrder(
  roleKey: AgentRoleKey,
  module: PromptModuleFormState,
  t: TranslateFn,
  roleLabels: Record<AgentRoleKey, string>,
) {
  const parsed = Number(module.order.trim())

  if (!Number.isInteger(parsed)) {
    throw new Error(
      t('presetsPage.form.errors.moduleOrderInvalid', {
        id:
          module.moduleId.trim() ||
          module.displayName.trim() ||
          t('presetsPage.form.newModule'),
        role: roleLabels[roleKey],
      }),
    )
  }

  return parsed
}

function parseEntryOrder(
  roleKey: AgentRoleKey,
  moduleLabel: string,
  entry: PromptEntryFormState,
  t: TranslateFn,
  roleLabels: Record<AgentRoleKey, string>,
) {
  const parsed = Number(entry.order.trim())

  if (!Number.isInteger(parsed)) {
    throw new Error(
      t('presetsPage.form.errors.entryOrderInvalid', {
        id: entry.entryId.trim() || entry.displayName.trim() || '—',
        module: moduleLabel,
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

  const seenModuleIds = new Set<string>()
  const modules = sortModules(
    agent.modules.map((module, index) => {
      const moduleId = module.moduleId.trim()
      const order = parseModuleOrder(roleKey, module, t, roleLabels)

      if (!moduleId) {
        throw new Error(
          t('presetsPage.form.errors.moduleIdRequired', {
            index: index + 1,
            role: roleLabels[roleKey],
          }),
        )
      }

      if (seenModuleIds.has(moduleId)) {
        throw new Error(
          t('presetsPage.form.errors.duplicateModuleId', {
            id: moduleId,
            role: roleLabels[roleKey],
          }),
        )
      }
      seenModuleIds.add(moduleId)

      const moduleLabel = getPromptModuleLabel(t, moduleId, module.displayName)
      const seenEntryIds = new Set<string>()
      const entries = sortEntries(
        module.entries.map((entry, entryIndex) => {
          const entryId = entry.entryId.trim()
          const displayName = entry.displayName.trim()
          const text = entry.text.trim()
          const contextKey = entry.contextKey.trim()
          const entryOrder = parseEntryOrder(roleKey, moduleLabel, entry, t, roleLabels)

          if (!entryId) {
            throw new Error(
              t('presetsPage.form.errors.entryIdRequired', {
                index: entryIndex + 1,
                module: moduleLabel,
                role: roleLabels[roleKey],
              }),
            )
          }

          if (seenEntryIds.has(entryId)) {
            throw new Error(
              t('presetsPage.form.errors.duplicateEntryId', {
                id: entryId,
                module: moduleLabel,
                role: roleLabels[roleKey],
              }),
            )
          }
          seenEntryIds.add(entryId)

          if (entry.kind === 'custom_text') {
            if (!displayName) {
              throw new Error(
                t('presetsPage.form.errors.entryDisplayNameRequired', {
                  index: entryIndex + 1,
                  module: moduleLabel,
                  role: roleLabels[roleKey],
                }),
              )
            }

            if (!text) {
              throw new Error(
                t('presetsPage.form.errors.customEntryTextRequired', {
                  index: entryIndex + 1,
                  module: moduleLabel,
                  role: roleLabels[roleKey],
                }),
              )
            }
          }

          return {
            ...(contextKey ? { context_key: contextKey } : {}),
            ...(text ? { text } : {}),
            display_name: displayName || entry.entryId.trim() || entryId,
            enabled: entry.enabled,
            entry_id: entryId,
            kind: entry.kind,
            order: entryOrder,
            required: entry.required,
          } satisfies PresetModuleEntry
        }),
      )

      return {
        display_name:
          module.displayName.trim() ||
          getPromptModuleDefaultDisplayName(t, moduleId, undefined),
        entries,
        message_role: module.messageRole,
        module_id: moduleId,
        order,
      }
    }),
  )

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

function createModuleEntryLabel(entry: PromptEntryFormState, t: TranslateFn, index: number) {
  if (entry.displayName.trim()) {
    return entry.displayName.trim()
  }

  return t('presetsPage.form.untitledEntry', { index })
}

function createModuleLabel(module: PromptModuleFormState, t: TranslateFn) {
  if (module.displayName.trim() || module.moduleId.trim()) {
    return getPromptModuleLabel(t, module.moduleId.trim(), module.displayName.trim())
  }

  return t('presetsPage.form.newModule')
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
  const messageRoleItems = useMemo(
    () =>
      promptMessageRoles.map((role) => ({
        label: getPromptMessageRoleLabel(translate, role),
        value: role,
      })),
    [translate],
  )
  const [formState, setFormState] = useState<FormState>(() => createInitialState(translate))
  const [expandedAgents, setExpandedAgents] = useState<Record<AgentRoleKey, boolean>>(
    createCollapsedAgentsState,
  )
  const [expandedModelSettings, setExpandedModelSettings] = useState<Record<AgentRoleKey, boolean>>(
    createCollapsedAgentsState,
  )
  const [expandedModules, setExpandedModules] = useState<Record<AgentRoleKey, string[]>>(
    createExpandedModulesState,
  )
  const [expandedEntryIds, setExpandedEntryIds] = useState<string[]>(createExpandedEntryIdsState)
  const [draggedEntry, setDraggedEntry] = useState<EntryDragState | null>(null)
  const [dropTarget, setDropTarget] = useState<EntryDragState | null>(null)
  const [draggedModule, setDraggedModule] = useState<ModuleDragState | null>(null)
  const [moduleDropTarget, setModuleDropTarget] = useState<ModuleDragState | null>(null)
  const [loadedPreset, setLoadedPreset] = useState<PresetDetail | null>(null)
  const [previewDialogState, setPreviewDialogState] = useState<PreviewDialogState | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  const [previewNotice, setPreviewNotice] = useState<ToastInput | null>(null)
  useToastMessage(submitError)
  useToastNotice(previewNotice)

  const updateAgentModules = useCallback(
    (
      roleKey: AgentRoleKey,
      updater: (modules: PromptModuleFormState[]) => PromptModuleFormState[],
    ) => {
      setFormState((current) => ({
        ...current,
        agents: {
          ...current.agents,
          [roleKey]: {
            ...current.agents[roleKey],
            modules: updater(current.agents[roleKey].modules),
          },
        },
      }))
    },
    [],
  )

  const loadPresetFromServer = useCallback(
    async (nextPresetId: string, signal?: AbortSignal) => {
      const result = await getPreset(nextPresetId, signal)

      if (signal?.aborted) {
        return null
      }

      setFormState({
        agents: Object.fromEntries(
          getOrderedAgentRoleKeys().map((roleKey) => [
            roleKey,
            createAgentFormState(result.agents[roleKey], translate),
          ]),
        ) as FormState['agents'],
        displayName: result.display_name,
        presetId: result.preset_id,
      })
      setLoadedPreset(result)
      setExpandedAgents(createCollapsedAgentsState())
      setExpandedModelSettings(createCollapsedAgentsState())
      setExpandedModules(createExpandedModulesState())
      setExpandedEntryIds(createExpandedEntryIdsState())
      setDraggedEntry(null)
      setDropTarget(null)
      setDraggedModule(null)
      setModuleDropTarget(null)
      setPreviewDialogState(null)

      return result
    },
    [translate],
  )

  useEffect(() => {
    if (!open) {
      setFormState(createInitialState(translate))
      setExpandedAgents(createCollapsedAgentsState())
      setExpandedModelSettings(createCollapsedAgentsState())
      setExpandedModules(createExpandedModulesState())
      setExpandedEntryIds(createExpandedEntryIdsState())
      setDraggedEntry(null)
      setDropTarget(null)
      setDraggedModule(null)
      setModuleDropTarget(null)
      setLoadedPreset(null)
      setPreviewDialogState(null)
      setIsLoading(false)
      setIsSubmitting(false)
      setSubmitError(null)
      return
    }

    if (mode !== 'edit' || !presetId) {
      setFormState(createInitialState(translate))
      setExpandedAgents(createCollapsedAgentsState())
      setExpandedModelSettings(createCollapsedAgentsState())
      setExpandedModules(createExpandedModulesState())
      setExpandedEntryIds(createExpandedEntryIdsState())
      setDraggedEntry(null)
      setDropTarget(null)
      setDraggedModule(null)
      setModuleDropTarget(null)
      setLoadedPreset(null)
      setPreviewDialogState(null)
      setIsLoading(false)
      setSubmitError(null)
      return
    }

    const controller = new AbortController()
    setIsLoading(true)
    setSubmitError(null)

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
  }, [loadPresetFromServer, mode, open, presetId, t, translate])

  function updateAgentField(
    roleKey: AgentRoleKey,
    key: keyof Omit<AgentFormState, 'modules'>,
    value: string,
  ) {
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

  function toggleModule(roleKey: AgentRoleKey, moduleClientId: string) {
    setExpandedModules((current) => ({
      ...current,
      [roleKey]: current[roleKey].includes(moduleClientId)
        ? current[roleKey].filter((currentId) => currentId !== moduleClientId)
        : [...current[roleKey], moduleClientId],
    }))
  }

  function updateModuleField(
    roleKey: AgentRoleKey,
    moduleClientId: string,
    key: 'displayName' | 'messageRole' | 'moduleId' | 'order',
    value: string,
  ) {
    updateAgentModules(roleKey, (modules) =>
      modules.map((module) =>
        module.clientId === moduleClientId ? { ...module, [key]: value } : module,
      ),
    )
  }

  function addCustomModule(roleKey: AgentRoleKey) {
    const modules = formState.agents[roleKey].modules
    const highestOrder = modules.reduce((maxOrder, module) => {
      const parsed = Number(module.order)
      return Number.isInteger(parsed) ? Math.max(maxOrder, parsed) : maxOrder
    }, presetModuleOrderStart - presetModuleOrderStep)
    const nextModule: PromptModuleFormState = {
      clientId: createPromptModuleClientId(),
      displayName: '',
      entries: [],
      isBuiltIn: false,
      messageRole: 'user',
      moduleId: '',
      order: String(highestOrder + presetModuleOrderStep),
    }

    updateAgentModules(roleKey, (modulesState) => [...modulesState, nextModule])
    setExpandedAgents((current) => ({ ...current, [roleKey]: true }))
    setExpandedModules((current) => ({
      ...current,
      [roleKey]: current[roleKey].includes(nextModule.clientId)
        ? current[roleKey]
        : [...current[roleKey], nextModule.clientId],
    }))
  }

  function removeModule(roleKey: AgentRoleKey, moduleClientId: string) {
    updateAgentModules(roleKey, (modules) =>
      modules.filter((module) => module.clientId !== moduleClientId),
    )
    setExpandedModules((current) => ({
      ...current,
      [roleKey]: current[roleKey].filter((moduleId) => moduleId !== moduleClientId),
    }))
    setExpandedEntryIds((current) => {
      const removedEntryIds =
        formState.agents[roleKey].modules.find((module) => module.clientId === moduleClientId)?.entries ?? []

      if (removedEntryIds.length === 0) {
        return current
      }

      const removedSet = new Set(removedEntryIds.map((entry) => entry.clientId))
      return current.filter((entryId) => !removedSet.has(entryId))
    })

    if (draggedModule?.clientId === moduleClientId && draggedModule.roleKey === roleKey) {
      setDraggedModule(null)
    }

    if (moduleDropTarget?.clientId === moduleClientId && moduleDropTarget.roleKey === roleKey) {
      setModuleDropTarget(null)
    }
  }

  function moveModule(roleKey: AgentRoleKey, sourceClientId: string, targetClientId: string) {
    if (sourceClientId === targetClientId) {
      return
    }

    updateAgentModules(roleKey, (modules) => {
      const sourceIndex = modules.findIndex((module) => module.clientId === sourceClientId)
      const targetIndex = modules.findIndex((module) => module.clientId === targetClientId)

      if (sourceIndex === -1 || targetIndex === -1) {
        return modules
      }

      const nextModules = [...modules]
      const [movedModule] = nextModules.splice(sourceIndex, 1)
      nextModules.splice(targetIndex, 0, movedModule)
      return renumberModules(nextModules)
    })
  }

  function handleModuleDragStart(roleKey: AgentRoleKey, clientId: string) {
    setDraggedModule({ clientId, roleKey })
    setModuleDropTarget(null)
  }

  function handleModuleDragOver(
    event: DragEvent<HTMLDivElement>,
    roleKey: AgentRoleKey,
    clientId: string,
  ) {
    if (!draggedModule || draggedModule.roleKey !== roleKey || draggedModule.clientId === clientId) {
      return
    }

    event.preventDefault()

    if (moduleDropTarget?.clientId !== clientId || moduleDropTarget.roleKey !== roleKey) {
      setModuleDropTarget({ clientId, roleKey })
    }
  }

  function handleModuleDrop(
    event: DragEvent<HTMLDivElement>,
    roleKey: AgentRoleKey,
    clientId: string,
  ) {
    if (!draggedModule || draggedModule.roleKey !== roleKey) {
      return
    }

    event.preventDefault()
    moveModule(roleKey, draggedModule.clientId, clientId)
    setDraggedModule(null)
    setModuleDropTarget(null)
  }

  function handleModuleDragEnd() {
    setDraggedModule(null)
    setModuleDropTarget(null)
  }

  function updateEntryField(
    roleKey: AgentRoleKey,
    moduleClientId: string,
    clientId: string,
    key: 'displayName' | 'entryId' | 'order' | 'text',
    value: string,
  ) {
    updateAgentModules(roleKey, (modules) =>
      modules.map((module) =>
        module.clientId === moduleClientId
          ? {
              ...module,
              entries: module.entries.map((entry) =>
                entry.clientId === clientId ? { ...entry, [key]: value } : entry,
              ),
            }
          : module,
      ),
    )
  }

  function updateEntryEnabled(
    roleKey: AgentRoleKey,
    moduleClientId: string,
    clientId: string,
    enabled: boolean,
  ) {
    updateAgentModules(roleKey, (modules) =>
      modules.map((module) =>
        module.clientId === moduleClientId
          ? {
              ...module,
              entries: module.entries.map((entry) =>
                entry.clientId === clientId ? { ...entry, enabled } : entry,
              ),
            }
          : module,
      ),
    )
  }

  function addCustomEntry(roleKey: AgentRoleKey, moduleClientId: string) {
    const module = formState.agents[roleKey].modules.find((item) => item.clientId === moduleClientId)

    if (!module) {
      return
    }

    const highestOrder = module.entries.reduce((maxOrder, entry) => {
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

    updateAgentModules(roleKey, (modules) =>
      modules.map((currentModule) =>
        currentModule.clientId === moduleClientId
          ? {
              ...currentModule,
              entries: [...currentModule.entries, nextEntry],
            }
          : currentModule,
      ),
    )
    setExpandedAgents((current) => ({ ...current, [roleKey]: true }))
    setExpandedModules((current) => ({
      ...current,
      [roleKey]: current[roleKey].includes(moduleClientId)
        ? current[roleKey]
        : [...current[roleKey], moduleClientId],
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

  function removeEntry(roleKey: AgentRoleKey, moduleClientId: string, clientId: string) {
    updateAgentModules(roleKey, (modules) =>
      modules.map((module) =>
        module.clientId === moduleClientId
          ? {
              ...module,
              entries: module.entries.filter((entry) => entry.clientId !== clientId),
            }
          : module,
      ),
    )
    setExpandedEntryIds((current) => current.filter((currentId) => currentId !== clientId))

    if (
      draggedEntry?.clientId === clientId &&
      draggedEntry.roleKey === roleKey &&
      draggedEntry.moduleClientId === moduleClientId
    ) {
      setDraggedEntry(null)
    }

    if (
      dropTarget?.clientId === clientId &&
      dropTarget.roleKey === roleKey &&
      dropTarget.moduleClientId === moduleClientId
    ) {
      setDropTarget(null)
    }
  }

  function moveEntry(
    roleKey: AgentRoleKey,
    moduleClientId: string,
    sourceClientId: string,
    targetClientId: string,
  ) {
    if (sourceClientId === targetClientId) {
      return
    }

    updateAgentModules(roleKey, (modules) =>
      modules.map((module) => {
        if (module.clientId !== moduleClientId) {
          return module
        }

        const sourceIndex = module.entries.findIndex((entry) => entry.clientId === sourceClientId)
        const targetIndex = module.entries.findIndex((entry) => entry.clientId === targetClientId)

        if (sourceIndex === -1 || targetIndex === -1) {
          return module
        }

        const nextEntries = [...module.entries]
        const [movedEntry] = nextEntries.splice(sourceIndex, 1)
        nextEntries.splice(targetIndex, 0, movedEntry)

        return {
          ...module,
          entries: renumberModuleEntries(nextEntries),
        }
      }),
    )
  }

  function handleEntryDragStart(roleKey: AgentRoleKey, moduleClientId: string, clientId: string) {
    setDraggedEntry({ clientId, moduleClientId, roleKey })
    setDropTarget(null)
  }

  function handleEntryDragOver(
    event: DragEvent<HTMLDivElement>,
    roleKey: AgentRoleKey,
    moduleClientId: string,
    clientId: string,
  ) {
    if (
      !draggedEntry ||
      draggedEntry.roleKey !== roleKey ||
      draggedEntry.moduleClientId !== moduleClientId ||
      draggedEntry.clientId === clientId
    ) {
      return
    }

    event.preventDefault()

    if (
      dropTarget?.clientId !== clientId ||
      dropTarget.roleKey !== roleKey ||
      dropTarget.moduleClientId !== moduleClientId
    ) {
      setDropTarget({ clientId, moduleClientId, roleKey })
    }
  }

  function handleEntryDrop(
    event: DragEvent<HTMLDivElement>,
    roleKey: AgentRoleKey,
    moduleClientId: string,
    clientId: string,
  ) {
    if (
      !draggedEntry ||
      draggedEntry.roleKey !== roleKey ||
      draggedEntry.moduleClientId !== moduleClientId
    ) {
      return
    }

    event.preventDefault()
    moveEntry(roleKey, moduleClientId, draggedEntry.clientId, clientId)
    setDraggedEntry(null)
    setDropTarget(null)
  }

  function handleEntryDragEnd() {
    setDraggedEntry(null)
    setDropTarget(null)
  }

  function openAgentPreview(roleKey: AgentRoleKey) {
    if (!loadedPreset) {
      return
    }

    setPreviewDialogState({
      initialAgent: roleKey,
      scopeLabel: roleLabels[roleKey],
    })
  }

  function openModulePreview(roleKey: AgentRoleKey, module: PromptModuleFormState) {
    const moduleId = module.moduleId.trim()

    if (!loadedPreset || !moduleId) {
      setPreviewNotice({
        message: t('presetsPage.preview.feedback.saveModuleBeforePreview'),
        tone: 'warning',
      })
      return
    }

    const savedModule = loadedPreset.agents[roleKey].modules.find(
      (candidate) => candidate.module_id === moduleId,
    )

    if (!savedModule) {
      setPreviewNotice({
        message: t('presetsPage.preview.feedback.saveModuleBeforePreview'),
        tone: 'warning',
      })
      return
    }

    setPreviewDialogState({
      initialAgent: roleKey,
      initialModuleId: moduleId,
      scopeLabel: getPromptModuleLabel(translate, savedModule.module_id, savedModule.display_name),
    })
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
          : await updatePreset({
              agents: nextAgents,
              display_name: nextDisplayName,
              preset_id: nextPresetId,
            })

      await onCompleted({
        message:
          mode === 'create'
            ? t('presetsPage.feedback.created', { id: preset.display_name })
            : t('presetsPage.feedback.updated', { id: preset.display_name }),
        preset,
      })

      setLoadedPreset(preset)

      onOpenChange(false)
    } catch (error) {
      setSubmitError(getErrorMessage(error, t('presetsPage.form.errors.submitFailed')))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <>
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
                                <div className="flex flex-wrap items-start justify-between gap-3">
                                  <p className="text-xs leading-6 text-[var(--color-text-muted)]">
                                    {t('presetsPage.form.moduleHint')}
                                  </p>
                                  <div className="flex flex-wrap items-center justify-end gap-2">
                                    {mode === 'edit' && loadedPreset ? (
                                      <Button
                                        onClick={() => {
                                          openAgentPreview(roleKey)
                                        }}
                                        size="sm"
                                        variant="secondary"
                                      >
                                        <FontAwesomeIcon icon={faEye} />
                                        {t('presetsPage.actions.previewPrompt')}
                                      </Button>
                                    ) : null}
                                    <Button
                                      onClick={() => {
                                        addCustomModule(roleKey)
                                      }}
                                      size="sm"
                                      variant="secondary"
                                    >
                                      <FontAwesomeIcon icon={faPlus} />
                                      {t('presetsPage.actions.addModule')}
                                    </Button>
                                  </div>
                                </div>

                                {agentState.modules.map((module) => {
                                  const isModuleExpanded = expandedModules[roleKey].includes(module.clientId)
                                  const enabledCount = module.entries.filter((entry) => entry.enabled).length
                                  const isModuleDragged =
                                    draggedModule?.clientId === module.clientId &&
                                    draggedModule.roleKey === roleKey
                                  const isModuleDropTarget =
                                    moduleDropTarget?.clientId === module.clientId &&
                                    moduleDropTarget.roleKey === roleKey &&
                                    !isModuleDragged
                                  const moduleLabel = createModuleLabel(module, translate)

                                  return (
                                    <div
                                      className={cn(
                                        'overflow-hidden rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_72%,transparent)] transition duration-200',
                                        isModuleDragged && 'opacity-55',
                                        isModuleDropTarget &&
                                          'border-[var(--color-accent-gold-line)] shadow-[0_0_0_1px_var(--color-accent-gold-line)]',
                                      )}
                                      key={module.clientId}
                                      onDragOver={(event) => {
                                        handleModuleDragOver(event, roleKey, module.clientId)
                                      }}
                                      onDrop={(event) => {
                                        handleModuleDrop(event, roleKey, module.clientId)
                                      }}
                                    >
                                      <div className="flex items-center gap-3 px-4 py-4">
                                        <button
                                          aria-label={t('presetsPage.actions.dragModule')}
                                          className="inline-flex h-9 w-9 shrink-0 cursor-grab items-center justify-center rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-[var(--color-text-secondary)] transition duration-200 hover:text-[var(--color-text-primary)] active:cursor-grabbing"
                                          draggable
                                          onDragEnd={handleModuleDragEnd}
                                          onDragStart={(event) => {
                                            event.dataTransfer.effectAllowed = 'move'
                                            event.dataTransfer.setData('text/plain', module.clientId)
                                            handleModuleDragStart(roleKey, module.clientId)
                                          }}
                                          type="button"
                                        >
                                          <FontAwesomeIcon icon={faGripVertical} />
                                        </button>

                                        <button
                                          aria-expanded={isModuleExpanded}
                                          className="flex min-w-0 flex-1 items-center justify-between gap-4 text-left transition duration-200"
                                          onClick={() => {
                                            toggleModule(roleKey, module.clientId)
                                          }}
                                          type="button"
                                        >
                                          <div className="min-w-0 space-y-1">
                                            <div className="flex min-w-0 flex-wrap items-center gap-2">
                                              <p className="truncate text-sm font-medium text-[var(--color-text-primary)]">
                                                {moduleLabel}
                                              </p>
                                              <Badge variant="subtle">
                                                {getPromptMessageRoleLabel(translate, module.messageRole)}
                                              </Badge>
                                              {module.moduleId.trim() ? (
                                                <Badge variant="subtle">{module.moduleId.trim()}</Badge>
                                              ) : null}
                                            </div>
                                            <p className="text-xs leading-6 text-[var(--color-text-muted)]">
                                              {t('presetsPage.form.moduleSummary', {
                                                count: module.entries.length,
                                                enabled: enabledCount,
                                              })}
                                            </p>
                                          </div>

                                          <div className="flex items-center gap-2">
                                            {!module.isBuiltIn ? (
                                              <IconButton
                                                icon={<FontAwesomeIcon icon={faTrashCan} />}
                                                label={t('presetsPage.actions.removeModule')}
                                                onClick={(event) => {
                                                  event.stopPropagation()
                                                  removeModule(roleKey, module.clientId)
                                                }}
                                                size="sm"
                                                variant="danger"
                                              />
                                            ) : null}
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
                                          </div>
                                        </button>
                                      </div>

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
                                              <div className="grid gap-4 md:grid-cols-2">
                                                <label className="space-y-2">
                                                  <span className="block text-xs text-[var(--color-text-muted)]">
                                                    {t('presetsPage.form.fields.moduleId')}
                                                  </span>
                                                  <Input
                                                    disabled={module.isBuiltIn}
                                                    id={`preset-form-${roleKey}-${module.clientId}-module-id`}
                                                    name={`${roleKey}_module_id_${module.clientId}`}
                                                    onChange={(event) => {
                                                      updateModuleField(
                                                        roleKey,
                                                        module.clientId,
                                                        'moduleId',
                                                        event.target.value,
                                                      )
                                                    }}
                                                    placeholder={t('presetsPage.form.placeholders.moduleId')}
                                                    value={module.moduleId}
                                                  />
                                                </label>

                                                <label className="space-y-2">
                                                  <span className="block text-xs text-[var(--color-text-muted)]">
                                                    {t('presetsPage.form.fields.moduleDisplayName')}
                                                  </span>
                                                  <Input
                                                    id={`preset-form-${roleKey}-${module.clientId}-display-name`}
                                                    name={`${roleKey}_module_display_name_${module.clientId}`}
                                                    onChange={(event) => {
                                                      updateModuleField(
                                                        roleKey,
                                                        module.clientId,
                                                        'displayName',
                                                        event.target.value,
                                                      )
                                                    }}
                                                    placeholder={t(
                                                      'presetsPage.form.placeholders.moduleDisplayName',
                                                    )}
                                                    value={module.displayName}
                                                  />
                                                </label>

                                                <label className="space-y-2">
                                                  <span className="block text-xs text-[var(--color-text-muted)]">
                                                    {t('presetsPage.form.fields.messageRole')}
                                                  </span>
                                                  <Select
                                                    items={messageRoleItems}
                                                    onValueChange={(nextValue) => {
                                                      updateModuleField(
                                                        roleKey,
                                                        module.clientId,
                                                        'messageRole',
                                                        nextValue,
                                                      )
                                                    }}
                                                    textAlign="start"
                                                    value={module.messageRole}
                                                  />
                                                </label>

                                                <label className="space-y-2">
                                                  <span className="block text-xs text-[var(--color-text-muted)]">
                                                    {t('presetsPage.form.fields.order')}
                                                  </span>
                                                  <Input
                                                    id={`preset-form-${roleKey}-${module.clientId}-order`}
                                                    name={`${roleKey}_module_order_${module.clientId}`}
                                                    onChange={(event) => {
                                                      updateModuleField(
                                                        roleKey,
                                                        module.clientId,
                                                        'order',
                                                        event.target.value,
                                                      )
                                                    }}
                                                    placeholder={t('presetsPage.form.placeholders.order')}
                                                    value={module.order}
                                                  />
                                                </label>
                                              </div>

                                              <div className="flex flex-wrap items-start justify-between gap-3">
                                                <p className="text-xs leading-6 text-[var(--color-text-muted)]">
                                                  {t('presetsPage.form.moduleHint')}
                                                </p>
                                                <div className="flex flex-wrap items-center justify-end gap-2">
                                                  {mode === 'edit' && loadedPreset ? (
                                                    <Button
                                                      onClick={() => {
                                                        openModulePreview(roleKey, module)
                                                      }}
                                                      size="sm"
                                                      variant="secondary"
                                                    >
                                                      <FontAwesomeIcon icon={faEye} />
                                                      {t('presetsPage.actions.previewPrompt')}
                                                    </Button>
                                                  ) : null}
                                                  <Button
                                                    onClick={() => {
                                                      addCustomEntry(roleKey, module.clientId)
                                                    }}
                                                    size="sm"
                                                    variant="secondary"
                                                  >
                                                    <FontAwesomeIcon icon={faPlus} />
                                                    {t('presetsPage.actions.addCustomEntry')}
                                                  </Button>
                                                </div>
                                              </div>

                                              {module.entries.length > 0 ? (
                                                <div className="space-y-3">
                                                  {module.entries.map((entry, entryIndex) => {
                                                    const isExpanded = expandedEntryIds.includes(entry.clientId)
                                                    const isDragged =
                                                      draggedEntry?.clientId === entry.clientId &&
                                                      draggedEntry.roleKey === roleKey &&
                                                      draggedEntry.moduleClientId === module.clientId
                                                    const isDropTarget =
                                                      dropTarget?.clientId === entry.clientId &&
                                                      dropTarget.roleKey === roleKey &&
                                                      dropTarget.moduleClientId === module.clientId &&
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
                                                          handleEntryDragOver(
                                                            event,
                                                            roleKey,
                                                            module.clientId,
                                                            entry.clientId,
                                                          )
                                                        }}
                                                        onDrop={(event) => {
                                                          handleEntryDrop(
                                                            event,
                                                            roleKey,
                                                            module.clientId,
                                                            entry.clientId,
                                                          )
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
                                                                handleEntryDragStart(
                                                                  roleKey,
                                                                  module.clientId,
                                                                  entry.clientId,
                                                                )
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
                                                                {createModuleEntryLabel(
                                                                  entry,
                                                                  translate,
                                                                  entryIndex + 1,
                                                                )}
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
                                                                updateEntryEnabled(
                                                                  roleKey,
                                                                  module.clientId,
                                                                  entry.clientId,
                                                                  enabled,
                                                                )
                                                              }}
                                                              size="sm"
                                                            />
                                                            {canRemove ? (
                                                              <IconButton
                                                                icon={<FontAwesomeIcon icon={faTrashCan} />}
                                                                label={t('presetsPage.actions.removeEntry')}
                                                                onClick={() => {
                                                                  removeEntry(
                                                                    roleKey,
                                                                    module.clientId,
                                                                    entry.clientId,
                                                                  )
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
                                                                      id={`preset-form-${roleKey}-${module.clientId}-${entry.clientId}-id`}
                                                                      name={`${roleKey}_${module.clientId}_entry_id_${entryIndex}`}
                                                                      onChange={(event) => {
                                                                        updateEntryField(
                                                                          roleKey,
                                                                          module.clientId,
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
                                                                      id={`preset-form-${roleKey}-${module.clientId}-${entry.clientId}-order`}
                                                                      name={`${roleKey}_${module.clientId}_entry_order_${entryIndex}`}
                                                                      onChange={(event) => {
                                                                        updateEntryField(
                                                                          roleKey,
                                                                          module.clientId,
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
                                                                    id={`preset-form-${roleKey}-${module.clientId}-${entry.clientId}-display-name`}
                                                                    name={`${roleKey}_${module.clientId}_entry_display_name_${entryIndex}`}
                                                                    onChange={(event) => {
                                                                      updateEntryField(
                                                                        roleKey,
                                                                        module.clientId,
                                                                        entry.clientId,
                                                                        'displayName',
                                                                        event.target.value,
                                                                      )
                                                                    }}
                                                                    placeholder={t(
                                                                      'presetsPage.form.placeholders.entryDisplayName',
                                                                    )}
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
                                                                      id={`preset-form-${roleKey}-${module.clientId}-${entry.clientId}-text`}
                                                                      name={`${roleKey}_${module.clientId}_entry_text_${entryIndex}`}
                                                                      onChange={(event) => {
                                                                        updateEntryField(
                                                                          roleKey,
                                                                          module.clientId,
                                                                          entry.clientId,
                                                                          'text',
                                                                          event.target.value,
                                                                        )
                                                                      }}
                                                                      placeholder={t(
                                                                        'presetsPage.form.placeholders.entryText',
                                                                      )}
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

      {mode === 'edit' && loadedPreset && previewDialogState ? (
        <PresetPromptPreviewDialog
          initialAgent={previewDialogState.initialAgent}
          initialModuleId={previewDialogState.initialModuleId}
          onOpenChange={(nextOpen) => {
            if (!nextOpen) {
              setPreviewDialogState(null)
            }
          }}
          open
          preset={loadedPreset}
          scopeLabel={previewDialogState.scopeLabel}
        />
      ) : null}
    </>
  )
}
