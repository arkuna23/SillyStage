import type { ReactNode } from 'react'

import { CardDescription, CardHeader, CardTitle } from '../../components/ui/card'
import { cn } from '../../lib/cn'

const COVER_OBJECT_POSITION = 'center 26%'

function getCharacterMonogram(name: string) {
  return Array.from(name.trim())[0] ?? '?'
}

export function CharacterAvatar({ coverUrl, name }: { coverUrl?: string | null; name: string }) {
  const monogram = getCharacterMonogram(name)

  return (
    <div className="size-10 overflow-hidden rounded-full border border-[var(--color-border-subtle)] bg-[linear-gradient(135deg,var(--color-accent-gold-soft),var(--color-accent-copper-soft))] shadow-[0_12px_24px_rgba(0,0,0,0.12)]">
      {coverUrl ? (
        <img
          alt={name}
          className="h-full w-full object-cover"
          src={coverUrl}
          style={{ objectPosition: COVER_OBJECT_POSITION }}
        />
      ) : (
        <div className="flex h-full w-full items-center justify-center">
          <span className="font-display text-sm text-[var(--color-text-primary)]">{monogram}</span>
        </div>
      )}
    </div>
  )
}

export function SessionListSkeleton() {
  return (
    <div className="space-y-3">
      {Array.from({ length: 5 }).map((_, index) => (
        <div
          className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4"
          key={index}
        >
          <div className="h-5 w-28 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
          <div className="mt-3 h-3 w-20 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
          <div className="mt-3 h-3 w-full animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
          <div className="mt-2 h-3 w-4/5 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
        </div>
      ))}
    </div>
  )
}

export function ConversationSkeleton() {
  return (
    <div className="space-y-5">
      {Array.from({ length: 6 }).map((_, index) => (
        <div
          className={cn('flex gap-3', index % 3 === 1 ? 'justify-center' : 'justify-start')}
          key={index}
        >
          {index % 3 === 1 ? null : (
            <div className="size-10 rounded-full bg-[var(--color-bg-elevated)]" />
          )}
          <div
            className={cn(
              'animate-pulse rounded-[1.4rem] bg-[var(--color-bg-elevated)]',
              index % 3 === 1 ? 'h-16 w-[min(72%,28rem)]' : 'h-20 w-[min(78%,32rem)]',
            )}
          />
        </div>
      ))}
    </div>
  )
}

export function RightPanelSection({
  action,
  children,
  description,
  title,
}: {
  action?: ReactNode
  children: ReactNode
  description?: string
  title: string
}) {
  return (
    <section className="space-y-3">
      <div className="space-y-1.5">
        <div className="flex items-center justify-between gap-3">
          <CardTitle className="text-[1.15rem] leading-snug">{title}</CardTitle>
          {action ? <div className="shrink-0">{action}</div> : null}
        </div>
        {description ? (
          <CardDescription className="text-sm leading-6">{description}</CardDescription>
        ) : null}
      </div>
      {children}
    </section>
  )
}

export function StagePanelHeader({
  actions,
  title,
  titleClassName,
}: {
  actions?: ReactNode
  title: string
  titleClassName?: string
}) {
  return (
    <CardHeader className="h-[5.25rem] border-b border-[var(--color-border-subtle)] px-6 py-4">
      <div className="flex min-h-0 flex-1 items-center justify-between gap-4">
        <div className="min-w-0 flex-1">
          <CardTitle className={cn('truncate leading-none', titleClassName)}>{title}</CardTitle>
        </div>
        <div className="flex h-10 shrink-0 items-center justify-end">
          {actions ? <div className="flex items-center gap-2">{actions}</div> : null}
        </div>
      </div>
    </CardHeader>
  )
}
