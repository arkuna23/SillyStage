import type { TextareaHTMLAttributes } from 'react'

import { cn } from '../../lib/cn'

export type TextareaProps = TextareaHTMLAttributes<HTMLTextAreaElement>

export function Textarea({ className, ...props }: TextareaProps) {
  return (
    <textarea
      className={cn(
        'scrollbar-none min-h-28 w-full rounded-[1.4rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-3 text-sm leading-7 text-[var(--color-text-primary)] outline-none transition placeholder:text-[var(--color-text-muted)] focus:border-[var(--color-accent-copper)] focus:ring-2 focus:ring-[var(--color-focus-ring)]',
        className,
      )}
      {...props}
    />
  )
}
