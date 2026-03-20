import type {
  ArchitectPromptMode,
  PresetPromptPreviewEntry,
  PresetPromptPreviewModule,
  PresetPromptPreview,
  PromptPreviewMessageRole,
} from '../apis/types'
import type {
  PromptViewerMessage,
} from '../../components/prompt-viewer'
import { getPromptMessageRoleLabel, getPromptModuleLabel } from './preset-labels'

export type PromptPreviewTranslateFn = (key: string, options?: Record<string, unknown>) => string

function sortModules(modules: PresetPromptPreviewModule[]) {
  return [...modules].sort((left, right) => {
    if (left.order !== right.order) {
      return left.order - right.order
    }

    return left.module_id.localeCompare(right.module_id, 'zh-Hans-CN-u-co-pinyin')
  })
}

function sortEntries(entries: PresetPromptPreviewEntry[]) {
  return [...entries].sort((left, right) => {
    if (left.order !== right.order) {
      return left.order - right.order
    }

    return left.entry_id.localeCompare(right.entry_id, 'zh-Hans-CN-u-co-pinyin')
  })
}

export function buildPromptPreviewViewerMessages(args: {
  preview: PresetPromptPreview | null
  t: PromptPreviewTranslateFn
}) {
  if (!args.preview) {
    return [] satisfies PromptViewerMessage[]
  }

  return args.preview.messages.map((message, messageIndex) => ({
    id: `preview:${message.role}:${messageIndex}`,
    label: getPromptMessageRoleLabel(args.t, message.role),
    messageRole: message.role,
    modules: sortModules(message.modules).map((module, moduleIndex) => ({
      entries: sortEntries(module.entries).map((entry) => ({
        entryId: entry.entry_id,
        entryLabel: entry.display_name.trim() || entry.entry_id,
        source: entry.source,
        text: entry.compiled_text,
      })),
      id: `preview:${message.role}:${module.module_id}:${moduleIndex}`,
      moduleId: module.module_id,
      moduleLabel: getPromptModuleLabel(
        args.t,
        module.module_id,
        module.display_name,
      ),
    })),
  }))
}

export function getPromptPreviewRoleLabel(
  t: PromptPreviewTranslateFn,
  role: PromptPreviewMessageRole,
) {
  switch (role) {
    case 'full':
      return t('presetsPage.preview.messageRoles.full')
    case 'system':
      return getPromptMessageRoleLabel(t, 'system')
    case 'user':
      return getPromptMessageRoleLabel(t, 'user')
  }
}

export function getPromptPreviewArchitectModeLabel(
  t: PromptPreviewTranslateFn,
  mode: ArchitectPromptMode,
) {
  switch (mode) {
    case 'draft_continue':
      return t('presetsPage.preview.architectModes.draftContinue')
    case 'draft_init':
      return t('presetsPage.preview.architectModes.draftInit')
    case 'graph':
      return t('presetsPage.preview.architectModes.graph')
  }
}
