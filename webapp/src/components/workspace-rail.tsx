import { useTranslation } from 'react-i18next'

import { cn } from '../lib/cn'
import type { WorkspaceRailContent } from './layout/workspace-context'
import { WorkspacePanelShell } from './layout/workspace-panel-shell'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from './ui/card'

type WorkspaceRailProps = {
  className?: string
  content: WorkspaceRailContent
}

export function WorkspaceRail({ className, content }: WorkspaceRailProps) {
  const { t } = useTranslation()

  return (
    <aside className={cn('h-full min-h-0 min-w-0 pl-1', className)}>
      <WorkspacePanelShell className="h-full min-h-0">
        <Card className="flex h-full min-h-0 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_92%,transparent)] shadow-none">
          <CardHeader className="gap-4 border-b border-[var(--color-border-subtle)]">
            <p className="text-xs uppercase text-[var(--color-accent-copper)]">
              {t('workspace.rail.heading')}
            </p>

            <div className="space-y-2.5">
              <CardTitle className="text-[1.65rem] leading-tight">{content.title}</CardTitle>
              {content.description ? (
                <CardDescription className="text-sm leading-6">
                  {content.description}
                </CardDescription>
              ) : null}
            </div>
          </CardHeader>

          <CardContent className="min-h-0 flex-1 overflow-y-auto pt-6">
            <div className="space-y-3 pr-1">
              {content.stats.map((stat) => (
                <div
                  className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4"
                  key={stat.label}
                >
                  <p className="text-xs text-[var(--color-text-muted)]">{stat.label}</p>
                  <p className="mt-2.5 font-display text-3xl leading-none text-[var(--color-text-primary)]">
                    {stat.value}
                  </p>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </WorkspacePanelShell>
    </aside>
  )
}

export function WorkspaceRailSkeleton({ className }: { className?: string }) {
  return (
    <aside className={cn('h-full min-h-0 min-w-0 pl-1', className)}>
      <WorkspacePanelShell className="h-full min-h-0">
        <Card className="flex h-full min-h-0 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_92%,transparent)] shadow-none">
          <CardHeader className="gap-4 border-b border-[var(--color-border-subtle)]">
            <div className="h-3 w-20 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />

            <div className="space-y-2.5">
              <div className="h-8 w-32 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
              <div className="h-3 w-full animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
              <div className="h-3 w-5/6 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            </div>
          </CardHeader>

          <CardContent className="min-h-0 flex-1 overflow-y-auto pt-6">
            <div className="space-y-3 pr-1">
              <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                <div className="h-3 w-16 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
                <div className="mt-3 h-8 w-14 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
              </div>
            </div>
          </CardContent>
        </Card>
      </WorkspacePanelShell>
    </aside>
  )
}
