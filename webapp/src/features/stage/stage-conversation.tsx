import { faChevronDown } from '@fortawesome/free-solid-svg-icons/faChevronDown'
import { faPen } from '@fortawesome/free-solid-svg-icons/faPen'
import { faSpinner } from '@fortawesome/free-solid-svg-icons/faSpinner'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { AnimatePresence, motion } from 'framer-motion'

import { Button } from '../../components/ui/button'
import { IconButton } from '../../components/ui/icon-button'
import { Textarea } from '../../components/ui/textarea'
import { cn } from '../../lib/cn'
import type { CharacterSummary } from '../characters/types'
import type { StageCopy } from './copy'
import { CharacterAvatar, ConversationSkeleton } from './stage-panel-shared'
import type { CoverCache, StageMessage, TurnWorkerStatus } from './stage-ui-types'

const panelEase = [0.16, 1, 0.3, 1] as const

function ThoughtBubble({
  copy,
  expanded,
  message,
  onToggle,
}: {
  copy: StageCopy
  expanded: boolean
  message: StageMessage
  onToggle: () => void
}) {
  if (!expanded) {
    return (
      <div className="inline-flex max-w-fit items-center gap-3 rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_88%,transparent)] px-4 py-2.5 text-left transition hover:border-[var(--color-accent-copper-soft)]">
        <p className="whitespace-nowrap text-xs uppercase text-[var(--color-text-muted)]">
          {copy.messages.thinking}
        </p>
        <IconButton
          className="h-5 w-5 min-h-0 shrink-0 rounded-full px-0"
          icon={<FontAwesomeIcon className="text-[0.58rem]" icon={faChevronDown} />}
          label={copy.messages.expandThought}
          onClick={onToggle}
          size="sm"
          variant="ghost"
        />
      </div>
    )
  }

  return (
    <div className="max-w-[min(72%,30rem)] rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_88%,transparent)] px-5 py-3.5 text-left transition hover:border-[var(--color-accent-copper-soft)]">
      <div className="flex items-center justify-between gap-3">
        <p className="whitespace-nowrap text-xs uppercase text-[var(--color-text-muted)]">
          {copy.messages.thinking}
        </p>
        <IconButton
          className="h-5 w-5 min-h-0 shrink-0 rounded-full px-0"
          icon={
            <FontAwesomeIcon
              className={cn('text-[0.58rem] transition-transform', expanded ? 'rotate-180' : '')}
              icon={faChevronDown}
            />
          }
          label={copy.messages.expandThought}
          onClick={onToggle}
          size="sm"
          variant="ghost"
        />
      </div>
      {expanded ? (
        <p className="mt-2 text-sm leading-7 text-[var(--color-text-secondary)]">{message.text}</p>
      ) : null}
    </div>
  )
}

export function TurnStatusBar({
  status,
}: {
  status: TurnWorkerStatus | null
}) {
  if (!status) {
    return null
  }

  return (
    <motion.div
      animate={{ opacity: 1, y: 0 }}
      className="pointer-events-none absolute left-1/2 top-0 z-10 -translate-x-1/2 -translate-y-1/2"
      exit={{ opacity: 0, y: -6 }}
      initial={{ opacity: 0, y: -8 }}
      transition={{ duration: 0.22, ease: panelEase }}
    >
      <div className="inline-flex h-8 items-center gap-2 rounded-full border border-[var(--color-accent-gold-line)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] px-3.5 text-[0.82rem] text-[var(--color-text-secondary)] shadow-[0_10px_22px_rgba(0,0,0,0.12)] backdrop-blur-sm">
        <FontAwesomeIcon className="animate-spin text-[0.68rem] text-[var(--color-accent-copper)]" icon={faSpinner} />
        <span className="whitespace-nowrap">{status.label}</span>
      </div>
    </motion.div>
  )
}

export function StageConversation({
  composerLocked,
  characterCovers,
  characterMap,
  copy,
  deletingPlayerMessageId,
  expandedThoughtIds,
  editingPlayerDraft,
  editingPlayerMessageId,
  isLoading,
  messages,
  onCancelEditPlayerMessage,
  onChangePlayerMessageDraft,
  onDeletePlayerMessage,
  onEditPlayerMessage,
  onSavePlayerMessage,
  onToggleThought,
  prefersReducedMotion,
  savingPlayerMessageId,
}: {
  composerLocked: boolean
  characterCovers: CoverCache
  characterMap: Map<string, CharacterSummary>
  copy: StageCopy
  deletingPlayerMessageId: string | null
  expandedThoughtIds: Set<string>
  editingPlayerDraft: string
  editingPlayerMessageId: string | null
  isLoading: boolean
  messages: StageMessage[]
  onCancelEditPlayerMessage: () => void
  onChangePlayerMessageDraft: (value: string) => void
  onDeletePlayerMessage: (message: StageMessage) => void
  onEditPlayerMessage: (message: StageMessage) => void
  onSavePlayerMessage: () => void
  onToggleThought: (messageId: string) => void
  prefersReducedMotion: boolean | null
  savingPlayerMessageId: string | null
}) {
  if (isLoading) {
    return <ConversationSkeleton />
  }

  if (messages.length === 0) {
    return (
      <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-5 py-6 text-sm leading-7 text-[var(--color-text-secondary)]">
        {copy.messages.noMessages}
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <AnimatePresence initial={false}>
        {messages.map((message, index) => {
          const previous = messages[index - 1]
          const next = messages[index + 1]
          const isActorMessage =
            message.variant === 'dialogue' || message.variant === 'action' || message.variant === 'thought'
          const sameAsPrevious =
            isActorMessage &&
            previous &&
            (previous.variant === 'dialogue' || previous.variant === 'action' || previous.variant === 'thought') &&
            previous.speakerId === message.speakerId
          const sameAsNext =
            isActorMessage &&
            next &&
            (next.variant === 'dialogue' || next.variant === 'action' || next.variant === 'thought') &&
            next.speakerId === message.speakerId
          const coverUrl = characterCovers[message.speakerId]
          const character = characterMap.get(message.speakerId)

          if (message.variant === 'narration') {
            return (
              <motion.div
                animate={{ opacity: 1, y: 0 }}
                className="flex justify-center"
                exit={{ opacity: 0, y: -8 }}
                initial={{ opacity: 0, y: 10 }}
                key={message.id}
                transition={prefersReducedMotion ? { duration: 0 } : { duration: 0.22, ease: panelEase }}
              >
                <div className="max-w-[min(72%,30rem)] rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_56%,transparent)] px-4 py-3 text-center text-sm leading-7 text-[var(--color-text-secondary)]">
                  {message.text}
                </div>
              </motion.div>
            )
          }

          if (message.variant === 'player') {
            const isEditing = Boolean(message.messageId) && editingPlayerMessageId === message.messageId
            const isDeleting = Boolean(message.messageId) && deletingPlayerMessageId === message.messageId
            const isSaving = Boolean(message.messageId) && savingPlayerMessageId === message.messageId
            const canMutate = Boolean(message.messageId)

            return (
              <motion.div
                animate={{ opacity: 1, x: 0, y: 0 }}
                className="flex justify-end"
                exit={{ opacity: 0, x: 10, y: -6 }}
                initial={{ opacity: 0, x: 16, y: 10 }}
                key={message.id}
                transition={prefersReducedMotion ? { duration: 0 } : { duration: 0.24, ease: panelEase }}
              >
                <div className="max-w-[min(76%,30rem)] space-y-2">
                  {isEditing ? (
                    <div className="space-y-3 rounded-[1.35rem] border border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] px-4 py-3">
                      <Textarea
                        className="min-h-[7rem] border-[var(--color-accent-gold-line)] bg-[color-mix(in_srgb,var(--color-bg-panel)_86%,white)] text-[var(--color-text-primary)]"
                        id={`stage-player-message-edit-${message.messageId ?? message.id}`}
                        name={`stage-player-message-edit-${message.messageId ?? message.id}`}
                        onChange={(event) => {
                          onChangePlayerMessageDraft(event.target.value)
                        }}
                        value={editingPlayerDraft}
                      />
                      <div className="flex justify-end gap-2">
                        <Button
                          disabled={composerLocked || isSaving}
                          onClick={onCancelEditPlayerMessage}
                          size="sm"
                          variant="ghost"
                        >
                          {copy.messages.cancelEditPlayer}
                        </Button>
                        <Button
                          disabled={composerLocked || isSaving || !editingPlayerDraft.trim()}
                          onClick={onSavePlayerMessage}
                          size="sm"
                          variant="secondary"
                        >
                          {copy.messages.savePlayer}
                        </Button>
                      </div>
                    </div>
                  ) : (
                    <>
                      <div className="rounded-[1.35rem] border border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] px-4 py-3 text-sm leading-7 text-[var(--color-text-primary)]">
                        {message.text}
                      </div>
                      <div className="flex justify-end gap-2">
                        <IconButton
                          className="h-8 w-8 rounded-full px-0"
                          disabled={composerLocked || !canMutate || isDeleting || isSaving}
                          icon={<FontAwesomeIcon className="text-xs" icon={faPen} />}
                          label={copy.messages.editPlayer}
                          onClick={() => {
                            onEditPlayerMessage(message)
                          }}
                          size="sm"
                          variant="ghost"
                        />
                        <IconButton
                          className="h-8 w-8 rounded-full px-0"
                          disabled={composerLocked || !canMutate || isDeleting || isSaving}
                          icon={<FontAwesomeIcon className="text-xs" icon={faTrashCan} />}
                          label={copy.messages.deletePlayer}
                          onClick={() => {
                            onDeletePlayerMessage(message)
                          }}
                          size="sm"
                          variant="ghost"
                        />
                      </div>
                    </>
                  )}
                </div>
              </motion.div>
            )
          }

          return (
            <motion.div
              animate={{ opacity: 1, x: 0, y: 0 }}
              className="flex items-end gap-3"
              exit={{ opacity: 0, x: -10, y: -6 }}
              initial={{ opacity: 0, x: -14, y: 10 }}
              key={message.id}
              transition={prefersReducedMotion ? { duration: 0 } : { duration: 0.24, ease: panelEase }}
            >
              <div className="w-10 shrink-0">
                {!sameAsPrevious ? (
                  <CharacterAvatar coverUrl={coverUrl} name={character?.name ?? message.speakerName} />
                ) : null}
              </div>
              <div className={cn('flex min-w-0 flex-col gap-2', sameAsNext ? 'pb-1' : '')}>
                {!sameAsPrevious ? (
                  <p className="text-xs text-[var(--color-text-muted)]">{message.speakerName}</p>
                ) : null}

                {message.variant === 'thought' ? (
                  <ThoughtBubble
                    copy={copy}
                    expanded={expandedThoughtIds.has(message.id)}
                    message={message}
                    onToggle={() => {
                      onToggleThought(message.id)
                    }}
                  />
                ) : (
                  <div
                    className={cn(
                      'max-w-[min(76%,32rem)] rounded-[1.35rem] border px-4 py-3 text-sm leading-7 shadow-[0_12px_26px_rgba(0,0,0,0.1)]',
                      message.variant === 'action'
                        ? 'border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_90%,transparent)] text-[var(--color-text-secondary)] italic'
                        : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] text-[var(--color-text-primary)]',
                    )}
                  >
                    {message.text}
                  </div>
                )}
              </div>
            </motion.div>
          )
        })}
      </AnimatePresence>
    </div>
  )
}
