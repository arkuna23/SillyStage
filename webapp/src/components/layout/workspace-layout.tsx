import { Suspense } from 'react'
import { Outlet } from 'react-router-dom'
import { useTranslation } from 'react-i18next'

import { appPaths } from '../../app/paths'
import { Card } from '../ui/card'
import { WorkspaceSidebar } from '../workspace-sidebar'

function WorkspaceContentFallback() {
  return (
    <Card className="overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)]">
      <div className="border-b border-[var(--color-border-subtle)] px-6 py-6">
        <div className="h-4 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
        <div className="mt-4 h-10 w-64 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
      </div>

      <div className="grid gap-4 p-6 md:grid-cols-2 2xl:grid-cols-3">
        {Array.from({ length: 6 }).map((_, index) => (
          <div
            className="overflow-hidden rounded-[1.65rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)]"
            key={index}
          >
            <div className="h-40 animate-pulse bg-[color-mix(in_srgb,var(--color-accent-gold-soft)_55%,var(--color-bg-panel))]" />
            <div className="space-y-3 p-5">
              <div className="h-3 w-20 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
              <div className="h-6 w-2/3 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
              <div className="h-3 w-full animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
              <div className="h-3 w-4/5 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
            </div>
          </div>
        ))}
      </div>
    </Card>
  )
}

export function WorkspaceLayout() {
  const { t } = useTranslation()

  return (
    <section className="flex w-full flex-1 py-6 sm:py-8">
      <div className="grid w-full gap-6 lg:grid-cols-[19rem_minmax(0,1fr)] xl:grid-cols-[20rem_minmax(0,1fr)]">
        <WorkspaceSidebar
          ariaLabel={t('workspace.sidebar.title')}
          items={[
            {
              label: t('workspace.sidebar.items.characters.label'),
              to: appPaths.characters,
            },
          ]}
        />

        <div className="min-w-0">
          <Suspense fallback={<WorkspaceContentFallback />}>
            <Outlet />
          </Suspense>
        </div>
      </div>
    </section>
  )
}
