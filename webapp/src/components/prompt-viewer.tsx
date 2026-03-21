import { faChevronDown } from '@fortawesome/free-solid-svg-icons/faChevronDown'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { AnimatePresence, motion, useReducedMotion } from 'framer-motion'
import { useMemo, useState } from 'react'

import { cn } from '../lib/cn'
import { Badge } from './ui/badge'

export type PromptViewerMessageRole = 'system' | 'user'
export type PromptViewerEntrySource = 'preset' | 'synthetic'

export type PromptViewerEntry = {
  entryId: string
  entryLabel: string
  source: PromptViewerEntrySource
  text: string
}

export type PromptViewerModule = {
  entries: PromptViewerEntry[]
  id: string
  moduleId: string
  moduleLabel: string
}

export type PromptViewerMessage = {
  id: string
  label: string
  messageRole: PromptViewerMessageRole
  modules: PromptViewerModule[]
}

type PromptViewerProps = {
  emptyLabel: string
  messages: PromptViewerMessage[]
  noEntryContentLabel: string
  showEntryMarkers: boolean
  syntheticEntryLabel: string
}

function getPromptViewerModuleText(module: PromptViewerModule) {
  return module.entries
    .map((entry) => entry.text.trim())
    .filter((text) => text.length > 0)
    .join('\n\n')
}

export function PromptViewer({
  emptyLabel,
  messages,
  noEntryContentLabel,
  showEntryMarkers,
  syntheticEntryLabel,
}: PromptViewerProps) {
  const prefersReducedMotion = useReducedMotion()
  const visibleMessages = useMemo(
    () => messages.filter((message) => message.modules.length > 0),
    [messages],
  )
  const moduleSignature = useMemo(
    () =>
      visibleMessages.flatMap((message) => message.modules.map((module) => module.id)).join('|'),
    [visibleMessages],
  )
  const [expandedState, setExpandedState] = useState<{
    moduleIds: Set<string>
    signature: string
  }>({
    moduleIds: new Set(),
    signature: '',
  })
  const expandedModuleIds =
    expandedState.signature === moduleSignature ? expandedState.moduleIds : new Set<string>()

  function toggleModule(moduleId: string) {
    setExpandedState((current) => {
      const activeModuleIds =
        current.signature === moduleSignature ? current.moduleIds : new Set<string>()
      const next = new Set(activeModuleIds)

      if (next.has(moduleId)) {
        next.delete(moduleId)
      } else {
        next.add(moduleId)
      }

      return {
        moduleIds: next,
        signature: moduleSignature,
      }
    })
  }

  return visibleMessages.length > 0 ? (
    <div className="space-y-4">
      {visibleMessages.map((message) => (
        <section
          className="space-y-3 rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_84%,transparent)] px-4 py-4"
          key={message.id}
        >
          <div className="flex flex-wrap items-center gap-2">
            <h4 className="text-sm font-medium text-[var(--color-text-primary)]">
              {message.label}
            </h4>
          </div>

          <div className="space-y-3">
            {message.modules.map((module) => (
              <div
                className="rounded-[1.15rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4"
                key={module.id}
              >
                <button
                  aria-expanded={expandedModuleIds.has(module.id)}
                  className="flex w-full items-start gap-3 rounded-[1rem] text-left transition hover:text-[var(--color-text-primary)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)]"
                  onClick={() => {
                    toggleModule(module.id)
                  }}
                  type="button"
                >
                  <div className="min-w-0 flex-1">
                    <div className="flex flex-wrap items-center gap-2">
                      <Badge className="normal-case" variant="gold">
                        {module.moduleLabel}
                      </Badge>
                      <Badge className="normal-case" variant="subtle">
                        {module.moduleId}
                      </Badge>
                    </div>
                  </div>

                  <span
                    className={cn(
                      'mt-1 inline-flex shrink-0 items-center text-[0.72rem] text-[var(--color-text-muted)] transition-transform duration-200',
                      expandedModuleIds.has(module.id) ? 'rotate-180' : undefined,
                    )}
                  >
                    <FontAwesomeIcon icon={faChevronDown} />
                  </span>
                </button>

                <AnimatePresence initial={false}>
                  {expandedModuleIds.has(module.id) ? (
                    <motion.div
                      animate={
                        prefersReducedMotion ? { opacity: 1 } : { height: 'auto', opacity: 1 }
                      }
                      className="overflow-hidden"
                      exit={prefersReducedMotion ? { opacity: 0 } : { height: 0, opacity: 0 }}
                      initial={prefersReducedMotion ? { opacity: 0 } : { height: 0, opacity: 0 }}
                      transition={{
                        duration: prefersReducedMotion ? 0 : 0.2,
                        ease: [0.22, 1, 0.36, 1],
                      }}
                    >
                      {showEntryMarkers ? (
                        module.entries.length > 0 ? (
                          <div className="mt-4 space-y-3">
                            {module.entries.map((entry) => (
                              <div
                                className="rounded-[1rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-3.5 py-3"
                                key={`${module.id}:${entry.entryId}`}
                              >
                                <div className="flex flex-wrap items-center gap-2">
                                  <span className="text-xs font-medium text-[var(--color-text-primary)]">
                                    {entry.entryLabel}
                                  </span>
                                  <span className="text-xs text-[var(--color-text-muted)]">
                                    {entry.entryId}
                                  </span>
                                  {entry.source === 'synthetic' ? (
                                    <Badge className="normal-case" variant="info">
                                      {syntheticEntryLabel}
                                    </Badge>
                                  ) : null}
                                </div>
                                <p
                                  className={`mt-2 whitespace-pre-wrap break-words font-mono text-sm leading-7 ${
                                    entry.text.trim()
                                      ? 'text-[var(--color-text-primary)]'
                                      : 'text-[var(--color-text-muted)]'
                                  }`}
                                >
                                  {entry.text.trim() || noEntryContentLabel}
                                </p>
                              </div>
                            ))}
                          </div>
                        ) : (
                          <p className="mt-4 whitespace-pre-wrap break-words font-mono text-sm leading-7 text-[var(--color-text-muted)]">
                            {noEntryContentLabel}
                          </p>
                        )
                      ) : (
                        <p
                          className={`mt-4 whitespace-pre-wrap break-words font-mono text-sm leading-7 ${
                            getPromptViewerModuleText(module)
                              ? 'text-[var(--color-text-primary)]'
                              : 'text-[var(--color-text-muted)]'
                          }`}
                        >
                          {getPromptViewerModuleText(module) || noEntryContentLabel}
                        </p>
                      )}
                    </motion.div>
                  ) : null}
                </AnimatePresence>
              </div>
            ))}
          </div>
        </section>
      ))}
    </div>
  ) : (
    <div className="rounded-[1.35rem] border border-dashed border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-5 py-8 text-sm text-[var(--color-text-muted)]">
      {emptyLabel}
    </div>
  )
}
