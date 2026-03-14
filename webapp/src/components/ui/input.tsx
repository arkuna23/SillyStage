import type { InputHTMLAttributes } from 'react'

import { cn } from '../../lib/cn'

export type InputProps = InputHTMLAttributes<HTMLInputElement>

export function Input({ className, ...props }: InputProps) {
  return (
    <input
      className={cn(
        'h-12 w-full rounded-2xl border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 text-sm text-[var(--color-text-primary)] outline-none transition placeholder:text-[var(--color-text-muted)] focus:border-[var(--color-accent-copper)] focus:ring-2 focus:ring-[var(--color-focus-ring)]',
        className,
      )}
      {...props}
    />
  )
}
