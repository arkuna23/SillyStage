import { Slot } from '@radix-ui/react-slot'
import type { ButtonHTMLAttributes } from 'react'

import { cn } from '../../lib/cn'

type ButtonVariant = 'primary' | 'secondary' | 'ghost'
type ButtonSize = 'sm' | 'md' | 'lg'

const variantClasses: Record<ButtonVariant, string> = {
  primary:
    'border border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold)] px-5 text-[color:var(--color-accent-ink)] shadow-[0_18px_50px_var(--color-accent-glow)] hover:bg-[var(--color-accent-gold-strong)] hover:text-[color:var(--color-accent-ink)] active:text-[color:var(--color-accent-ink)] visited:text-[color:var(--color-accent-ink)]',
  secondary:
    'border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] text-[var(--color-text-primary)] hover:border-[var(--color-accent-copper-soft)] hover:bg-white/10',
  ghost:
    'border border-[var(--color-ghost-border)] bg-[var(--color-ghost-bg)] text-[var(--color-text-secondary)] hover:bg-[var(--color-ghost-bg-hover)] hover:text-[var(--color-text-primary)]',
}

const sizeClasses: Record<ButtonSize, string> = {
  sm: 'h-9 rounded-full px-3.5 text-sm',
  md: 'h-11 rounded-full px-5 text-sm',
  lg: 'h-12 rounded-full px-6 text-base',
}

export type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  asChild?: boolean
  variant?: ButtonVariant
  size?: ButtonSize
}

export function Button({
  asChild = false,
  className,
  size = 'md',
  style,
  type = 'button',
  variant = 'primary',
  ...props
}: ButtonProps) {
  const Comp = asChild ? Slot : 'button'
  const resolvedStyle =
    variant === 'primary'
      ? {
          color: 'var(--color-accent-ink)',
          ...style,
        }
      : style

  return (
    <Comp
      className={cn(
        'inline-flex items-center justify-center gap-2 whitespace-nowrap font-medium tracking-[0.02em] transition duration-200 ease-out focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)] focus-visible:ring-offset-0 disabled:pointer-events-none disabled:opacity-45',
        variantClasses[variant],
        sizeClasses[size],
        className,
      )}
      style={resolvedStyle}
      type={type}
      {...props}
    />
  )
}
