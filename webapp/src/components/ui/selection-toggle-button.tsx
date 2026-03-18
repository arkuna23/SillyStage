import { cn } from '../../lib/cn'

type SelectionIndicatorProps = {
  className?: string
  selected: boolean
}

type SelectionToggleButtonProps = {
  className?: string
  disabled?: boolean
  label: string
  onClick: () => void
  selected: boolean
}

export function SelectionIndicator({ className, selected }: SelectionIndicatorProps) {
  return (
    <span
      aria-hidden="true"
      className={cn(
        'inline-flex size-7 items-center justify-center rounded-full border text-xs shadow-[0_10px_24px_rgba(0,0,0,0.16)] transition',
        selected
          ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold)] text-[color:var(--color-accent-ink)]'
          : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] text-[var(--color-text-muted)]',
        className,
      )}
    >
      {selected ? '✓' : ''}
    </span>
  )
}

export function SelectionToggleButton({
  className,
  disabled = false,
  label,
  onClick,
  selected,
}: SelectionToggleButtonProps) {
  return (
    <button
      aria-label={label}
      aria-pressed={selected}
      className={cn(
        'inline-flex rounded-full focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)] disabled:pointer-events-none disabled:opacity-45',
        className,
      )}
      disabled={disabled}
      onClick={onClick}
      title={label}
      type="button"
    >
      <SelectionIndicator selected={selected} />
    </button>
  )
}
