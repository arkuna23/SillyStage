import { Slot } from '@radix-ui/react-slot'
import type { ButtonHTMLAttributes } from 'react'

import { cn } from '../../lib/cn'

type ButtonVariant = 'primary' | 'secondary' | 'ghost'
type ButtonSize = 'sm' | 'md' | 'lg'

const variantClasses: Record<ButtonVariant, string> = {
  primary:
    'border border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold)] px-5 text-[var(--color-accent-ink)] shadow-[0_18px_50px_rgba(217,167,74,0.28)] hover:bg-[var(--color-accent-gold-strong)]',
  secondary:
    'border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] text-[var(--color-text-primary)] hover:border-[var(--color-accent-copper-soft)] hover:bg-white/10',
  ghost:
    'border border-transparent bg-transparent text-[var(--color-text-secondary)] hover:bg-white/5 hover:text-[var(--color-text-primary)]',
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
  type = 'button',
  variant = 'primary',
  ...props
}: ButtonProps) {
  const Comp = asChild ? Slot : 'button'

  return (
    <Comp
      className={cn(
        'inline-flex items-center justify-center gap-2 whitespace-nowrap font-medium tracking-[0.02em] transition duration-200 ease-out focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-amber-200/70 focus-visible:ring-offset-0 disabled:pointer-events-none disabled:opacity-45',
        variantClasses[variant],
        sizeClasses[size],
        className,
      )}
      type={type}
      {...props}
    />
  )
}
