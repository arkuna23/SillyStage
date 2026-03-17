import type { ReactNode } from 'react'
import { useMemo } from 'react'

import { Badge } from '../../components/ui/badge'
import { cn } from '../../lib/cn'

type LorebookOption = {
  display_name: string
  lorebook_id: string
}

type StoryResourceLorebookSelectorProps = {
  disabled?: boolean
  emptyAction?: ReactNode
  emptyMessage: string
  lorebooks: ReadonlyArray<LorebookOption>
  noSelectionLabel: string
  onToggleLorebook: (lorebookId: string) => void
  selectedLorebookIds: ReadonlyArray<string>
}

export function StoryResourceLorebookSelector({
  disabled = false,
  emptyAction,
  emptyMessage,
  lorebooks,
  noSelectionLabel,
  onToggleLorebook,
  selectedLorebookIds,
}: StoryResourceLorebookSelectorProps) {
  const selectedLorebookLabels = useMemo(() => {
    const lorebookLookup = new Map(
      lorebooks.map((lorebook) => [lorebook.lorebook_id, lorebook.display_name]),
    )

    return selectedLorebookIds.map(
      (lorebookId) => lorebookLookup.get(lorebookId) ?? lorebookId,
    )
  }, [lorebooks, selectedLorebookIds])

  if (lorebooks.length === 0) {
    return (
      <div className="space-y-4 rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-5 text-sm text-[var(--color-text-secondary)]">
        <p>{emptyMessage}</p>
        {emptyAction ? <div className="flex justify-end">{emptyAction}</div> : null}
      </div>
    )
  }

  return (
    <div className="space-y-3">
      <div className="flex flex-wrap gap-2">
        {selectedLorebookLabels.length > 0 ? (
          selectedLorebookLabels.map((label) => (
            <Badge className="normal-case px-3 py-1.5" key={label} variant="subtle">
              {label}
            </Badge>
          ))
        ) : (
          <span className="text-sm text-[var(--color-text-muted)]">{noSelectionLabel}</span>
        )}
      </div>

      <div className="grid gap-2 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-3 sm:grid-cols-2">
        {lorebooks.map((lorebook) => {
          const isSelected = selectedLorebookIds.includes(lorebook.lorebook_id)

          return (
            <button
              className={cn(
                'rounded-[1.2rem] border px-3 py-3 text-left transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)] disabled:pointer-events-none disabled:opacity-45',
                isSelected
                  ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)]'
                  : 'border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_84%,transparent)] text-[var(--color-text-secondary)] hover:border-[var(--color-accent-copper-soft)] hover:text-[var(--color-text-primary)]',
              )}
              disabled={disabled}
              key={lorebook.lorebook_id}
              onClick={() => {
                onToggleLorebook(lorebook.lorebook_id)
              }}
              type="button"
            >
              <div className="truncate text-sm font-medium">{lorebook.display_name}</div>
              <div className="truncate pt-1 font-mono text-[0.74rem] text-[var(--color-text-muted)]">
                {lorebook.lorebook_id}
              </div>
            </button>
          )
        })}
      </div>
    </div>
  )
}
