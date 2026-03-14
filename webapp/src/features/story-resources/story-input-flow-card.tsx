import { Badge } from '../../components/ui/badge'
import { cn } from '../../lib/cn'

type StoryInputFlowCardProps = {
  badgeLabel: string
  className?: string
  description: string
  rawDescription: string
  rawLabel: string
  refinedDescription: string
  refinedLabel: string
}

export function StoryInputFlowCard({
  badgeLabel,
  className,
  description,
  rawDescription,
  rawLabel,
  refinedDescription,
  refinedLabel,
}: StoryInputFlowCardProps) {
  return (
    <div
      className={cn(
        'rounded-[1.55rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_74%,transparent)] p-4 shadow-[var(--shadow-surface)]',
        className,
      )}
    >
      <div className="flex flex-col gap-3">
        <div className="flex flex-wrap items-center gap-3">
          <Badge className="normal-case px-3 py-1.5" variant="info">
            {badgeLabel}
          </Badge>
          <p className="text-sm leading-6 text-[var(--color-text-secondary)]">{description}</p>
        </div>

        <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] md:items-center">
          <div className="rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-3">
            <p className="text-sm font-medium text-[var(--color-text-primary)]">{rawLabel}</p>
            <p className="pt-1 text-xs leading-6 text-[var(--color-text-muted)]">
              {rawDescription}
            </p>
          </div>

          <div className="flex items-center justify-center">
            <span className="inline-flex h-9 min-w-9 items-center justify-center rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] px-3 text-sm text-[var(--color-text-secondary)]">
              →
            </span>
          </div>

          <div className="rounded-[1.2rem] border border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] px-4 py-3">
            <p className="text-sm font-medium text-[var(--color-text-primary)]">{refinedLabel}</p>
            <p className="pt-1 text-xs leading-6 text-[var(--color-text-secondary)]">
              {refinedDescription}
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}
