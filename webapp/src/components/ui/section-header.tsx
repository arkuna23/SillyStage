import type { ReactNode } from 'react'

import { cn } from '../../lib/cn'

type SectionHeaderProps = {
  actions?: ReactNode
  className?: string
  description?: string
  eyebrow?: string
  title: string
}

export function SectionHeader({
  actions,
  className,
  description,
  eyebrow,
  title,
}: SectionHeaderProps) {
  return (
    <div
      className={cn(
        'flex flex-col gap-4 md:flex-row md:items-end md:justify-between',
        className,
      )}
    >
      <div className="space-y-3">
        {eyebrow ? (
          <p className="text-xs uppercase text-[var(--color-accent-copper)]">
            {eyebrow}
          </p>
        ) : null}
        <div className="space-y-3">
          <h2 className="font-display text-3xl leading-tight text-[var(--color-text-primary)] sm:text-4xl">
            {title}
          </h2>
          {description ? (
            <p className="max-w-2xl text-sm leading-7 text-[var(--color-text-secondary)]">
              {description}
            </p>
          ) : null}
        </div>
      </div>

      {actions ? <div className="md:shrink-0">{actions}</div> : null}
    </div>
  )
}
