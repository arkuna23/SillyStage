import { faPaperPlane } from '@fortawesome/free-solid-svg-icons/faPaperPlane'
import { faRotateRight } from '@fortawesome/free-solid-svg-icons/faRotateRight'
import { faSpinner } from '@fortawesome/free-solid-svg-icons/faSpinner'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { AnimatePresence, motion } from 'framer-motion'
import type { RefObject } from 'react'

import { Button } from '../../components/ui/button'
import { IconButton } from '../../components/ui/icon-button'
import { Switch } from '../../components/ui/switch'
import { Textarea } from '../../components/ui/textarea'
import { cn } from '../../lib/cn'
import type { CharacterSummary } from '../characters/types'
import type { StageCopy } from './copy'
import { StageConversation, TurnStatusBar } from './stage-conversation'
import type { CoverCache, StageMessage, TurnWorkerStatus } from './stage-ui-types'
import type { ReplySuggestion } from './types'

const panelEase = [0.16, 1, 0.3, 1] as const

type StageDialoguePanelProps = {
  characterCovers: CoverCache
  characterMap: Map<string, CharacterSummary>
  composerInput: string
  composerLocked: boolean
  composerMode: 'input' | 'suggestions'
  composerRef: RefObject<HTMLTextAreaElement | null>
  conversationScrollRef: RefObject<HTMLDivElement | null>
  copy: StageCopy
  deletingPlayerMessageId: string | null
  editingPlayerDraft: string
  editingPlayerMessageId: string | null
  expandedThoughtIds: Set<string>
  isLoading: boolean
  isRunningTurn: boolean
  isSuggestingReplies: boolean
  messages: StageMessage[]
  onCancelEditPlayerMessage: () => void
  onChangeComposerInput: (value: string) => void
  onChangePlayerMessageDraft: (value: string) => void
  onDeletePlayerMessage: (message: StageMessage) => void
  onEditPlayerMessage: (message: StageMessage) => void
  onGenerateReplySuggestions: () => void
  onRunTurn: () => void
  onSavePlayerMessage: () => void
  onScrollConversation: (element: HTMLDivElement) => void
  onSelectReplySuggestion: (suggestion: ReplySuggestion) => void
  onToggleReplySuggestions: (checked: boolean) => void
  onToggleThought: (messageId: string) => void
  overlayStatus: TurnWorkerStatus | null
  prefersReducedMotion: boolean | null
  replySuggestions: ReplySuggestion[]
  replySuggestionsEnabled: boolean
  savingPlayerMessageId: string | null
  selectedSessionExists: boolean
  suggestionsError: string | null
}

export function StageDialoguePanel({
  characterCovers,
  characterMap,
  composerInput,
  composerLocked,
  composerMode,
  composerRef,
  conversationScrollRef,
  copy,
  deletingPlayerMessageId,
  editingPlayerDraft,
  editingPlayerMessageId,
  expandedThoughtIds,
  isLoading,
  isRunningTurn,
  isSuggestingReplies,
  messages,
  onCancelEditPlayerMessage,
  onChangeComposerInput,
  onChangePlayerMessageDraft,
  onDeletePlayerMessage,
  onEditPlayerMessage,
  onGenerateReplySuggestions,
  onRunTurn,
  onSavePlayerMessage,
  onScrollConversation,
  onSelectReplySuggestion,
  onToggleReplySuggestions,
  onToggleThought,
  overlayStatus,
  prefersReducedMotion,
  replySuggestions,
  replySuggestionsEnabled,
  savingPlayerMessageId,
  selectedSessionExists,
  suggestionsError,
}: StageDialoguePanelProps) {
  return (
    <div className="flex h-full min-h-0 flex-col">
      <div
        className="scrollbar-none min-h-0 flex-1 overflow-y-auto pr-1"
        onScroll={(event) => {
          onScrollConversation(event.currentTarget)
        }}
        ref={conversationScrollRef}
      >
        <StageConversation
          composerLocked={composerLocked}
          characterCovers={characterCovers}
          characterMap={characterMap}
          copy={copy}
          deletingPlayerMessageId={deletingPlayerMessageId}
          editingPlayerDraft={editingPlayerDraft}
          editingPlayerMessageId={editingPlayerMessageId}
          expandedThoughtIds={expandedThoughtIds}
          isLoading={isLoading}
          messages={messages}
          onCancelEditPlayerMessage={onCancelEditPlayerMessage}
          onChangePlayerMessageDraft={onChangePlayerMessageDraft}
          onDeletePlayerMessage={onDeletePlayerMessage}
          onEditPlayerMessage={onEditPlayerMessage}
          onSavePlayerMessage={onSavePlayerMessage}
          onToggleThought={onToggleThought}
          prefersReducedMotion={prefersReducedMotion}
          savingPlayerMessageId={savingPlayerMessageId}
        />
      </div>

      <div className="relative mt-6 border-t border-[var(--color-border-subtle)] pt-7">
        <AnimatePresence>
          <TurnStatusBar status={overlayStatus} />
        </AnimatePresence>
        <div className="flex items-start gap-3">
          <div className="min-w-0 flex-1">
            <AnimatePresence initial={false} mode="wait">
              {composerMode === 'input' ? (
                <motion.div
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: -10 }}
                  initial={{ opacity: 0, x: 10 }}
                  key="composer-input"
                  transition={
                    prefersReducedMotion ? { duration: 0 } : { duration: 0.22, ease: panelEase }
                  }
                >
                  <Textarea
                    className="min-h-[7.5rem] flex-1"
                    id="stage-composer-input"
                    name="stage-composer-input"
                    ref={composerRef}
                    onChange={(event) => {
                      onChangeComposerInput(event.target.value)
                    }}
                    placeholder={copy.composer.placeholder}
                    value={composerInput}
                  />
                </motion.div>
              ) : (
                <motion.div
                  animate={{ opacity: 1, x: 0 }}
                  className="rounded-[1.4rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-3.5"
                  exit={{ opacity: 0, x: 10 }}
                  initial={{ opacity: 0, x: -10 }}
                  key="composer-suggestions"
                  transition={
                    prefersReducedMotion ? { duration: 0 } : { duration: 0.22, ease: panelEase }
                  }
                >
                  <div className="space-y-3">
                    <div className="flex items-start justify-between gap-3">
                      <div className="space-y-1">
                        <p className="text-sm font-medium text-[var(--color-text-primary)]">
                          {copy.composer.suggestions}
                        </p>
                        <p className="text-sm leading-6 text-[var(--color-text-secondary)]">
                          {copy.composer.suggestionsDescription}
                        </p>
                      </div>

                      <Button
                        disabled={isSuggestingReplies || isRunningTurn}
                        onClick={onGenerateReplySuggestions}
                        size="sm"
                        variant="ghost"
                      >
                        {copy.composer.suggestionsGenerate}
                      </Button>
                    </div>
                    {suggestionsError ? (
                      <div className="rounded-[1rem] border border-[var(--color-state-error-line)] bg-[var(--color-state-error-soft)] px-3.5 py-3 text-sm leading-6 text-[var(--color-text-primary)]">
                        {suggestionsError}
                      </div>
                    ) : isRunningTurn ? (
                      <div className="rounded-[1rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_80%,transparent)] px-4 py-4 text-sm leading-6 text-[var(--color-text-secondary)]">
                        {copy.composer.suggestionsUnavailable}
                      </div>
                    ) : null}
                    {isSuggestingReplies ? (
                      <div className="flex min-h-[5.75rem] items-center justify-center rounded-[1rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_80%,transparent)] px-4 py-4 text-sm text-[var(--color-text-secondary)]">
                        <span className="inline-flex items-center gap-2">
                          <FontAwesomeIcon className="animate-spin" icon={faSpinner} />
                          {copy.composer.suggestionsLoading}
                        </span>
                      </div>
                    ) : replySuggestions.length === 0 ? (
                      <div className="rounded-[1rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_80%,transparent)] px-4 py-4 text-sm leading-6 text-[var(--color-text-secondary)]">
                        {copy.composer.suggestionsEmpty}
                      </div>
                    ) : (
                      <div className="space-y-2">
                        {replySuggestions.map((suggestion) => (
                          <button
                            className="w-full rounded-[1rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-3 text-left transition hover:border-[var(--color-accent-copper-soft)] hover:bg-[color-mix(in_srgb,var(--color-bg-panel)_88%,white)]"
                            key={suggestion.reply_id}
                            onClick={() => {
                              onSelectReplySuggestion(suggestion)
                            }}
                            type="button"
                          >
                            <p className="text-sm leading-7 text-[var(--color-text-primary)]">
                              {suggestion.text}
                            </p>
                          </button>
                        ))}
                      </div>
                    )}
                  </div>
                </motion.div>
              )}
            </AnimatePresence>
          </div>
          <div className="flex w-[4.5rem] shrink-0 flex-col items-center gap-2.5">
            <p className="w-full text-center text-xs leading-5 text-[var(--color-text-secondary)]">
              {copy.composer.suggestions}
            </p>
            <Switch
              aria-label={copy.composer.suggestions}
              checked={replySuggestionsEnabled}
              disabled={!selectedSessionExists || isRunningTurn || isSuggestingReplies}
              onCheckedChange={onToggleReplySuggestions}
              size="md"
            />
            <IconButton
              className="w-11"
              disabled={!selectedSessionExists || !composerInput.trim() || isRunningTurn}
              icon={
                <FontAwesomeIcon
                  className={cn(isRunningTurn ? 'animate-spin' : '')}
                  icon={isRunningTurn ? faRotateRight : faPaperPlane}
                />
              }
              label={isRunningTurn ? copy.composer.running : copy.composer.send}
              onClick={onRunTurn}
              variant="primary"
            />
          </div>
        </div>
      </div>
    </div>
  )
}
