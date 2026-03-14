import type { ButtonHTMLAttributes } from 'react'

import { cn } from '../../lib/cn'

type SwitchSize = 'sm' | 'md'

const trackSizeClasses: Record<SwitchSize, string> = {
  sm: 'h-6 w-10',
  md: 'h-7 w-11',
}

const thumbSizeClasses: Record<SwitchSize, string> = {
  sm: 'h-[1.125rem] w-[1.125rem] data-[checked=true]:translate-x-[1rem]',
  md: 'h-5 w-5 data-[checked=true]:translate-x-[1rem]',
}

export type SwitchProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'onChange'> & {
  checked: boolean
  onCheckedChange?: (checked: boolean) => void
  size?: SwitchSize
}

export function Switch({
  checked,
  className,
  disabled,
  onCheckedChange,
  size = 'md',
  ...props
}: SwitchProps) {
  return (
    <button
      aria-checked={checked}
      className={cn(
        'relative inline-flex shrink-0 items-center rounded-full border transition duration-200 ease-out focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)] focus-visible:ring-offset-0 disabled:pointer-events-none disabled:opacity-45',
        checked
          ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)]'
          : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)]',
        trackSizeClasses[size],
        className,
      )}
      data-checked={checked}
      disabled={disabled}
      onClick={() => {
        if (!disabled) {
          onCheckedChange?.(!checked)
        }
      }}
      role="switch"
      type="button"
      {...props}
    >
      <span
        aria-hidden="true"
        className={cn(
          'inline-flex translate-x-[0.2rem] items-center justify-center rounded-full bg-[var(--color-bg-panel-strong)] shadow-[0_8px_18px_rgba(0,0,0,0.18)] transition duration-200 ease-out',
          checked
            ? 'border border-[var(--color-accent-gold-line)]'
            : 'border border-[var(--color-border-subtle)]',
          thumbSizeClasses[size],
        )}
        data-checked={checked}
      />
    </button>
  )
}
