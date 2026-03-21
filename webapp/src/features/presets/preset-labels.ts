import {
  type AgentRoleKey,
  agentRoleKeys,
  type BuiltInPromptModuleId,
  type PresetEntryKind,
  type PromptMessageRole,
  type PromptModuleId,
  promptModuleIds,
} from '../apis/types'

type TranslateLike = (key: string, options?: Record<string, unknown>) => string

type BuiltInPromptModuleDefinition = {
  defaultDisplayName: string
  messageRole: PromptMessageRole
  order: number
  translationKey: string
}

const builtInPromptModuleDefinitions: Record<BuiltInPromptModuleId, BuiltInPromptModuleDefinition> =
  {
    dynamic_context: {
      defaultDisplayName: 'Dynamic Context',
      messageRole: 'user',
      order: 40,
      translationKey: 'presetsPage.modules.dynamicContext',
    },
    output: {
      defaultDisplayName: 'Output',
      messageRole: 'system',
      order: 50,
      translationKey: 'presetsPage.modules.output',
    },
    role: {
      defaultDisplayName: 'Role',
      messageRole: 'system',
      order: 10,
      translationKey: 'presetsPage.modules.role',
    },
    static_context: {
      defaultDisplayName: 'Static Context',
      messageRole: 'user',
      order: 30,
      translationKey: 'presetsPage.modules.staticContext',
    },
    task: {
      defaultDisplayName: 'Task',
      messageRole: 'system',
      order: 20,
      translationKey: 'presetsPage.modules.task',
    },
  }

export function createPresetRoleLabels(t: TranslateLike): Record<AgentRoleKey, string> {
  return {
    actor: t('presetsPage.roles.actor'),
    architect: t('presetsPage.roles.architect'),
    director: t('presetsPage.roles.director'),
    keeper: t('presetsPage.roles.keeper'),
    narrator: t('presetsPage.roles.narrator'),
    planner: t('presetsPage.roles.planner'),
    replyer: t('presetsPage.roles.replyer'),
  }
}

export function isBuiltInPromptModuleId(
  moduleId: PromptModuleId,
): moduleId is BuiltInPromptModuleId {
  return promptModuleIds.includes(moduleId as BuiltInPromptModuleId)
}

export function getBuiltInPromptModuleDefinition(moduleId: BuiltInPromptModuleId) {
  return builtInPromptModuleDefinitions[moduleId]
}

export function getPromptModuleDefaultDisplayName(
  t: TranslateLike,
  moduleId: PromptModuleId,
  displayName?: string | null,
) {
  const normalizedDisplayName = displayName?.trim()

  if (isBuiltInPromptModuleId(moduleId)) {
    const definition = getBuiltInPromptModuleDefinition(moduleId)

    if (!normalizedDisplayName || normalizedDisplayName === definition.defaultDisplayName) {
      return t(definition.translationKey)
    }
  }

  return normalizedDisplayName || moduleId
}

export function getPromptModuleLabel(
  t: TranslateLike,
  moduleId: PromptModuleId,
  displayName?: string | null,
) {
  return getPromptModuleDefaultDisplayName(t, moduleId, displayName)
}

export function getPromptMessageRoleLabel(t: TranslateLike, role: PromptMessageRole) {
  switch (role) {
    case 'system':
      return t('presetsPage.messageRoles.system')
    case 'user':
      return t('presetsPage.messageRoles.user')
  }
}

export function getPresetEntryKindLabel(t: TranslateLike, kind: PresetEntryKind) {
  switch (kind) {
    case 'built_in_text':
      return t('presetsPage.entryKinds.builtInText')
    case 'built_in_context_ref':
      return t('presetsPage.entryKinds.builtInContextRef')
    case 'custom_text':
      return t('presetsPage.entryKinds.customText')
  }
}

export function getOrderedModuleIds() {
  return [...promptModuleIds]
}

export function getOrderedAgentRoleKeys() {
  return [...agentRoleKeys]
}
