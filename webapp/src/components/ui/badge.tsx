import type { HTMLAttributes } from 'react'

import { cn } from '../../lib/cn'

type BadgeVariant = 'gold' | 'subtle' | 'info'

const badgeVariants: Record<BadgeVariant, string> = {
  gold: 'border border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-accent-gold-strong)]',
  subtle:
    'border border-[var(--color-border-subtle)] bg-white/6 text-[var(--color-text-secondary)]',
  info: 'border border-[var(--color-state-info-line)] bg-[var(--color-state-info-soft)] text-[var(--color-text-primary)]',
}

export type BadgeProps = HTMLAttributes<HTMLSpanElement> & {
  variant?: BadgeVariant
}

export function Badge({ className, variant = 'subtle', ...props }: BadgeProps) {
  return (
    <span
      className={cn(
        'inline-flex items-center rounded-full px-3 py-1 text-xs font-medium uppercase',
        badgeVariants[variant],
        className,
      )}
      {...props}
    />
  )
}
