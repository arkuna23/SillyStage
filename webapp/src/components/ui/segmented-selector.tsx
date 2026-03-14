import { motion, useReducedMotion } from 'framer-motion'
import { useId, type ReactNode } from 'react'

import { cn } from '../../lib/cn'

type SegmentedSelectorItem = {
  ariaLabel?: string
  disabled?: boolean
  icon?: ReactNode
  label: ReactNode
  value: string
}

type SegmentedSelectorProps = {
  ariaLabel: string
  className?: string
  items: ReadonlyArray<SegmentedSelectorItem>
  layoutId?: string
  onValueChange?: (value: string) => void
  value: string
}

export function SegmentedSelector({
  ariaLabel,
  className,
  items,
  layoutId,
  onValueChange,
  value,
}: SegmentedSelectorProps) {
  const prefersReducedMotion = useReducedMotion()
  const instanceLayoutId = useId()
  const resolvedLayoutId = layoutId ?? `segmented-selector-active-surface-${instanceLayoutId}`

  return (
    <div
      aria-label={ariaLabel}
      className={cn(
        'relative inline-flex items-stretch gap-1 rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] p-1 shadow-[inset_0_1px_0_rgba(255,255,255,0.02)]',
        className,
      )}
      role="group"
    >
      {items.map((item) => {
        const selected = item.value === value

        return (
          <button
            aria-label={item.ariaLabel}
            aria-current={selected ? 'page' : undefined}
            className={cn(
              'relative inline-flex h-10 min-w-[2.9rem] items-center justify-center self-stretch rounded-[0.95rem] px-3 text-[0.82rem] font-medium leading-none transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)] xl:min-w-[6rem] xl:px-3.5 disabled:pointer-events-none disabled:opacity-40',
              selected
                ? 'text-[color:var(--color-accent-ink)]'
                : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]',
            )}
            disabled={item.disabled}
            key={item.value}
            onClick={() => {
              if (item.disabled || item.value === value) {
                return
              }

              onValueChange?.(item.value)
            }}
            type="button"
          >
            {selected ? (
              <motion.span
                className="absolute inset-0 rounded-[0.95rem] border border-[var(--color-accent-gold-line)] bg-[linear-gradient(135deg,color-mix(in_srgb,var(--color-accent-gold)_86%,var(--color-bg-curtain)),color-mix(in_srgb,var(--color-accent-gold-strong)_82%,var(--color-bg-curtain)))] shadow-[0_10px_24px_var(--color-accent-glow-soft)]"
                layoutId={resolvedLayoutId}
                transition={
                  prefersReducedMotion
                    ? { duration: 0 }
                    : { damping: 34, mass: 0.34, stiffness: 420, type: 'spring' }
                }
              />
            ) : null}
            <span className="relative z-10 inline-flex items-center gap-2 xl:gap-2.5">
              {item.icon ? (
                <span aria-hidden="true" className="inline-flex size-4 items-center justify-center">
                  {item.icon}
                </span>
              ) : null}
              <span>{item.label}</span>
            </span>
          </button>
        )
      })}
    </div>
  )
}
