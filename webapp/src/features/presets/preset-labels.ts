import {
  agentRoleKeys,
  promptModuleIds,
  type AgentRoleKey,
  type PresetEntryKind,
  type PromptModuleId,
} from '../apis/types'

type TranslateLike = (key: string, options?: Record<string, unknown>) => string

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

export function getPromptModuleLabel(t: TranslateLike, moduleId: PromptModuleId) {
  switch (moduleId) {
    case 'role':
      return t('presetsPage.modules.role')
    case 'task':
      return t('presetsPage.modules.task')
    case 'static_context':
      return t('presetsPage.modules.staticContext')
    case 'dynamic_context':
      return t('presetsPage.modules.dynamicContext')
    case 'output':
      return t('presetsPage.modules.output')
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
