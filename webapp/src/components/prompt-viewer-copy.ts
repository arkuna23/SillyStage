import type { PromptViewerMessage, PromptViewerModule } from './prompt-viewer'

function getPromptViewerModuleText(module: PromptViewerModule) {
  return module.entries
    .map((entry) => entry.text.trim())
    .filter((text) => text.length > 0)
    .join('\n\n')
}

export function buildPromptViewerCopyText(args: {
  entryLabel: string
  messages: PromptViewerMessage[]
  moduleLabel: string
  noEntryContentLabel: string
  showEntryMarkers: boolean
}) {
  return args.messages
    .filter((message) => message.modules.length > 0)
    .map((message) => {
      const sections = message.modules.map((module) => {
        const moduleHeader = `${args.moduleLabel}: ${module.moduleLabel} (${module.moduleId})`

        if (args.showEntryMarkers) {
          const entryBodies =
            module.entries.length > 0
              ? module.entries.map((entry) => {
                  const entryHeader = `${args.entryLabel}: ${entry.entryLabel} (${entry.entryId})`

                  return `${entryHeader}\n${entry.text.trim() || args.noEntryContentLabel}`
                })
              : [args.noEntryContentLabel]

          return [moduleHeader, ...entryBodies].join('\n\n')
        }

        const moduleBody = getPromptViewerModuleText(module)

        return moduleBody ? `${module.moduleLabel}:\n${moduleBody}` : `${module.moduleLabel}:`
      })

      return [message.label, ...sections].join('\n\n')
    })
    .join('\n\n\n')
    .trim()
}
