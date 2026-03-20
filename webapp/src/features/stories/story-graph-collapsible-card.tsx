import { faChevronDown } from '@fortawesome/free-solid-svg-icons/faChevronDown'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { AnimatePresence, motion, useReducedMotion } from 'framer-motion'
import type { ReactNode } from 'react'

import { cn } from '../../lib/cn'

type StoryGraphCollapsibleCardProps = {
  action?: ReactNode
  children: ReactNode
  className?: string
  contentClassName?: string
  open: boolean
  onToggle: () => void
  subtitle?: ReactNode
  title: ReactNode
}

export function StoryGraphCollapsibleCard({
  action,
  children,
  className,
  contentClassName,
  open,
  onToggle,
  subtitle,
  title,
}: StoryGraphCollapsibleCardProps) {
  const prefersReducedMotion = useReducedMotion()

  return (
    <div
      className={cn(
        'rounded-[1.4rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4',
        className,
      )}
    >
      <div className="flex items-start gap-3">
        <button
          aria-expanded={open}
          className="flex min-w-0 flex-1 items-start gap-3 rounded-[1rem] text-left transition hover:text-[var(--color-text-primary)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)]"
          onClick={onToggle}
          type="button"
        >
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <h4 className="truncate text-sm font-medium text-[var(--color-text-primary)]">
                {title}
              </h4>
              <span
                className={cn(
                  'inline-flex items-center text-[0.72rem] text-[var(--color-text-muted)] transition-transform duration-200',
                  open ? 'rotate-180' : undefined,
                )}
              >
                <FontAwesomeIcon icon={faChevronDown} />
              </span>
            </div>
            {subtitle ? (
              <div className="mt-1 text-xs leading-6 text-[var(--color-text-muted)]">
                {subtitle}
              </div>
            ) : null}
          </div>
        </button>

        {action ? <div className="flex shrink-0 items-center gap-2">{action}</div> : null}
      </div>

      <AnimatePresence initial={false}>
        {open ? (
          <motion.div
            animate={prefersReducedMotion ? { opacity: 1 } : { height: 'auto', opacity: 1 }}
            className="overflow-hidden"
            exit={prefersReducedMotion ? { opacity: 0 } : { height: 0, opacity: 0 }}
            initial={prefersReducedMotion ? { opacity: 0 } : { height: 0, opacity: 0 }}
            transition={{ duration: prefersReducedMotion ? 0 : 0.2, ease: [0.22, 1, 0.36, 1] }}
          >
            <div className={cn('pt-4', contentClassName)}>{children}</div>
          </motion.div>
        ) : null}
      </AnimatePresence>
    </div>
  )
}
