import { useEffect, useMemo, useState } from 'react'

import { Badge } from './badge'

type GenerationLoadingStageProps = {
  description: string
  elapsedLabel: string
  identifier?: string | null
  progressLabel?: string
  progressText?: string | null
  startedAtMs: number
  statusLabel: string
  title: string
}

function formatElapsedTime(elapsedMs: number) {
  const totalSeconds = Math.max(0, Math.floor(elapsedMs / 1000))
  const minutes = Math.floor(totalSeconds / 60)
  const seconds = totalSeconds % 60

  return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
}

export function GenerationLoadingStage({
  description,
  elapsedLabel,
  identifier,
  progressLabel,
  progressText,
  startedAtMs,
  statusLabel,
  title,
}: GenerationLoadingStageProps) {
  const [nowMs, setNowMs] = useState(() => startedAtMs)

  useEffect(() => {
    const timer = window.setInterval(() => {
      setNowMs(Date.now())
    }, 1000)

    return () => {
      window.clearInterval(timer)
    }
  }, [startedAtMs])

  const elapsedText = useMemo(() => formatElapsedTime(nowMs - startedAtMs), [nowMs, startedAtMs])

  return (
    <div className="flex min-h-[24rem] flex-col items-center justify-center gap-6 py-4 text-center">
      <div className="relative flex size-24 items-center justify-center">
        <span className="absolute inset-0 rounded-full border border-[var(--color-accent-gold-line)] opacity-35" />
        <span className="absolute inset-2 rounded-full border border-[var(--color-accent-copper-soft)] opacity-50 animate-ping" />
        <span className="absolute inset-[1.15rem] rounded-full border-2 border-transparent border-t-[var(--color-accent-gold)] border-r-[var(--color-accent-gold-soft)] animate-spin" />
        <span className="inline-flex size-8 rounded-full bg-[var(--color-accent-gold)] shadow-[0_0_28px_var(--color-accent-glow)]" />
      </div>

      <div className="space-y-3">
        <div className="flex justify-center">
          <Badge className="normal-case px-3 py-1.5" variant="info">
            {statusLabel}
          </Badge>
        </div>
        <h3 className="font-display text-[2rem] leading-tight text-[var(--color-text-primary)]">
          {title}
        </h3>
        <p className="mx-auto max-w-xl text-sm leading-7 text-[var(--color-text-secondary)]">
          {description}
        </p>
      </div>

      <div className="flex items-center gap-2">
        <span className="size-2 rounded-full bg-[var(--color-accent-gold)] animate-pulse" />
        <span className="size-2 rounded-full bg-[var(--color-accent-gold-soft)] animate-pulse [animation-delay:120ms]" />
        <span className="size-2 rounded-full bg-[var(--color-accent-copper-soft)] animate-pulse [animation-delay:220ms]" />
      </div>

      <div className="flex flex-col items-center gap-3">
        {progressText ? (
          <div className="rounded-full border border-[var(--color-accent-copper-soft)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_82%,var(--color-accent-copper-soft)_18%)] px-4 py-2 text-sm text-[var(--color-text-secondary)]">
            <span className="text-[var(--color-text-muted)]">{progressLabel}</span>
            <span className="ml-2 font-mono text-[var(--color-text-primary)]">{progressText}</span>
          </div>
        ) : null}

        <div className="rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-2 text-sm text-[var(--color-text-secondary)]">
          <span className="text-[var(--color-text-muted)]">{elapsedLabel}</span>
          <span className="ml-2 font-mono text-[var(--color-text-primary)]">{elapsedText}</span>
        </div>

        {identifier ? (
          <div className="rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-2 font-mono text-xs text-[var(--color-text-muted)]">
            {identifier}
          </div>
        ) : null}
      </div>
    </div>
  )
}
