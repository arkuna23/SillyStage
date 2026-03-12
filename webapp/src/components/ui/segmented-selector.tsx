import { motion, useReducedMotion } from 'framer-motion'
import type { ReactNode } from 'react'

import { cn } from '../../lib/cn'

type SegmentedSelectorItem = {
  disabled?: boolean
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
  layoutId = 'segmented-selector-active-surface',
  onValueChange,
  value,
}: SegmentedSelectorProps) {
  const prefersReducedMotion = useReducedMotion()

  return (
    <div
      aria-label={ariaLabel}
      className={cn(
        'relative inline-flex items-center gap-1 rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] p-1 shadow-[inset_0_1px_0_rgba(255,255,255,0.02)]',
        className,
      )}
      role="group"
    >
      {items.map((item) => {
        const selected = item.value === value

        return (
          <button
            aria-current={selected ? 'page' : undefined}
            className={cn(
              'relative inline-flex min-w-[5.75rem] items-center justify-center rounded-[0.95rem] px-3.5 py-2 text-[0.82rem] font-medium transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-amber-200/70 sm:min-w-[6.5rem] disabled:pointer-events-none disabled:opacity-40',
              selected
                ? 'text-[var(--color-accent-ink)]'
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
                className="absolute inset-0 rounded-[0.95rem] border border-[var(--color-accent-gold-line)] bg-[linear-gradient(135deg,rgba(243,211,140,0.98),rgba(217,167,74,0.96))] shadow-[0_10px_28px_rgba(217,167,74,0.22)]"
                layoutId={layoutId}
                transition={
                  prefersReducedMotion
                    ? { duration: 0 }
                    : { damping: 34, mass: 0.34, stiffness: 420, type: 'spring' }
                }
              />
            ) : null}
            <span className="relative z-10">{item.label}</span>
          </button>
        )
      })}
    </div>
  )
}
