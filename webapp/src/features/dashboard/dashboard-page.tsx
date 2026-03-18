import { useCallback, useEffect, useLayoutEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { useWorkspaceLayoutContext } from '../../components/layout/workspace-context'
import { Badge } from '../../components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../../components/ui/card'
import { SectionHeader } from '../../components/ui/section-header'
import { useToastNotice } from '../../components/ui/toast-context'
import { getDashboard } from './api'
import { DashboardDataPackageActions } from './dashboard-data-package-actions'
import type { DashboardPayload } from './types'

type Notice = {
  message: string
}

const emptyDashboard: DashboardPayload = {
  counts: {
    characters_total: 0,
    characters_with_cover: 0,
    sessions_total: 0,
    stories_total: 0,
    story_resources_total: 0,
  },
  global_config: {
    api_group_id: null,
    preset_id: null,
  },
  health: {
    status: 'ok',
  },
  recent_sessions: [],
  recent_stories: [],
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function DashboardMetric({
  label,
  value,
}: {
  label: string
  value: number | string
}) {
  return (
    <div className="rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
      <p className="text-xs text-[var(--color-text-muted)]">{label}</p>
      <p className="mt-3 font-display text-3xl leading-none text-[var(--color-text-primary)]">
        {value}
      </p>
    </div>
  )
}

function DashboardSkeleton() {
  return (
    <div className="space-y-8">
      <section className="space-y-5">
        <div className="h-7 w-40 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
          {Array.from({ length: 4 }).map((_, index) => (
            <div
              className="rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4"
              key={index}
            >
              <div className="h-3 w-20 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
              <div className="mt-4 h-8 w-14 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
            </div>
          ))}
        </div>
      </section>

      <div className="border-t border-[var(--color-border-subtle)]" />

      <section className="grid gap-6 xl:grid-cols-[minmax(0,0.95fr)_minmax(0,1.05fr)]">
        {Array.from({ length: 2 }).map((_, index) => (
          <div
            className="rounded-[1.5rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-5 py-5"
            key={index}
          >
            <div className="h-6 w-32 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
            <div className="mt-4 space-y-3">
              {Array.from({ length: index === 0 ? 2 : 3 }).map((__, rowIndex) => (
                <div className="h-10 rounded-[1rem] bg-[var(--color-bg-panel)]" key={rowIndex} />
              ))}
            </div>
          </div>
        ))}
      </section>

      <div className="border-t border-[var(--color-border-subtle)]" />

      {Array.from({ length: 2 }).map((_, index) => (
        <section className="space-y-5" key={index}>
          <div className="h-7 w-44 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
          <div className="divide-y divide-[var(--color-border-subtle)]">
            {Array.from({ length: 3 }).map((__, rowIndex) => (
              <div className="space-y-3 py-4" key={rowIndex}>
                <div className="h-5 w-40 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
                <div className="h-3 w-full animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
                <div className="h-3 w-3/4 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
              </div>
            ))}
          </div>
        </section>
      ))}
    </div>
  )
}

function EmptySection({ label }: { label: string }) {
  return (
    <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-4 py-5 text-sm text-[var(--color-text-secondary)]">
      {label}
    </div>
  )
}

function BindingSummaryCard({
  label,
  value,
}: {
  label: string
  value: string
}) {
  return (
    <div className="rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
      <p className="text-xs text-[var(--color-text-muted)]">{label}</p>
      <p className="mt-3 text-sm font-medium text-[var(--color-text-primary)]">{value}</p>
    </div>
  )
}

export function DashboardPage() {
  const { t, i18n } = useTranslation()
  const { setRailContent } = useWorkspaceLayoutContext()
  const [dashboard, setDashboard] = useState<DashboardPayload | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [notice, setNotice] = useState<Notice | null>(null)
  useToastNotice(notice)

  const dateFormatter = useMemo(
    () =>
      new Intl.DateTimeFormat(i18n.language.startsWith('zh') ? 'zh-CN' : 'en', {
        dateStyle: 'medium',
        timeStyle: 'short',
      }),
    [i18n.language],
  )

  const currentDashboard = dashboard ?? emptyDashboard
  const healthLabel =
    dashboard?.health.status === 'ok' ? t('dashboard.health.ok') : '—'
  const totalResourceCount =
    currentDashboard.counts.characters_total +
    currentDashboard.counts.story_resources_total +
    currentDashboard.counts.stories_total +
    currentDashboard.counts.sessions_total
  const recentActivityCount =
    currentDashboard.recent_stories.length + currentDashboard.recent_sessions.length
  const currentApiGroup =
    currentDashboard.global_config.api_group_id?.trim() || t('dashboard.config.emptyValue')
  const currentPreset =
    currentDashboard.global_config.preset_id?.trim() || t('dashboard.config.emptyValue')

  const refreshDashboard = useCallback(
    async (signal?: AbortSignal) => {
      setIsLoading(true)

      try {
        const nextDashboard = await getDashboard(signal)

        if (!signal?.aborted) {
          setDashboard(nextDashboard)
        }
      } catch (error) {
        if (!signal?.aborted) {
          setNotice({
            message: getErrorMessage(error, t('dashboard.feedback.loadFailed')),
          })
        }
      } finally {
        if (!signal?.aborted) {
          setIsLoading(false)
        }
      }
    },
    [t],
  )

  useEffect(() => {
    const controller = new AbortController()
    void refreshDashboard(controller.signal)

    return () => {
      controller.abort()
    }
  }, [refreshDashboard])

  useLayoutEffect(() => {
    if (isLoading) {
      setRailContent(null)

      return () => {
        setRailContent(null)
      }
    }

    setRailContent({
      description: t('dashboard.rail.description'),
      stats: [
        {
          label: t('dashboard.metrics.status'),
          value: healthLabel,
        },
        {
          label: t('dashboard.metrics.resources'),
          value: totalResourceCount,
        },
        {
          label: t('dashboard.metrics.activity'),
          value: recentActivityCount,
        },
      ],
      title: t('dashboard.title'),
    })

    return () => {
      setRailContent(null)
    }
  }, [healthLabel, isLoading, recentActivityCount, setRailContent, t, totalResourceCount])

  return (
    <div className="flex h-full min-h-0 flex-col gap-6">
      <WorkspacePanelShell className="h-full min-h-0">
          <Card className="flex h-full min-h-0 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
          <CardHeader className="border-b border-[var(--color-border-subtle)] md:min-h-[92px]">
            <SectionHeader
              actions={<DashboardDataPackageActions onImported={() => refreshDashboard()} />}
              title={t('dashboard.title')}
            />
          </CardHeader>

          <CardContent className="scrollbar-none min-h-0 flex-1 overflow-y-auto pt-6">
            <div className="space-y-8">
              {isLoading ? (
                <DashboardSkeleton />
              ) : (
                <>
                  <section className="space-y-5">
                    <SectionHeader title={t('dashboard.sections.overview')} />
                    <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                      <DashboardMetric
                        label={t('dashboard.counts.characters')}
                        value={currentDashboard.counts.characters_total}
                      />
                      <DashboardMetric
                        label={t('dashboard.counts.storyResources')}
                        value={currentDashboard.counts.story_resources_total}
                      />
                      <DashboardMetric
                        label={t('dashboard.counts.stories')}
                        value={currentDashboard.counts.stories_total}
                      />
                      <DashboardMetric
                        label={t('dashboard.counts.sessions')}
                        value={currentDashboard.counts.sessions_total}
                      />
                    </div>
                  </section>

                  <div className="border-t border-[var(--color-border-subtle)]" />

                  <section className="grid gap-6 xl:grid-cols-[minmax(0,0.95fr)_minmax(0,1.05fr)]">
                    <div className="rounded-[1.5rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-5 py-5">
                      <div className="space-y-2">
                        <CardTitle className="text-2xl">{t('dashboard.sections.health')}</CardTitle>
                        <CardDescription>{t('dashboard.health.ok')}</CardDescription>
                      </div>
                      <div className="mt-4">
                        <Badge>{healthLabel}</Badge>
                      </div>
                    </div>

                    <div className="rounded-[1.5rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-5 py-5">
                      <div className="space-y-2">
                        <CardTitle className="text-2xl">{t('dashboard.sections.defaults')}</CardTitle>
                        <CardDescription>{t('dashboard.config.description')}</CardDescription>
                      </div>
                      <div className="mt-4 grid gap-3 md:grid-cols-2">
                        <BindingSummaryCard
                          label={t('dashboard.config.apiGroupLabel')}
                          value={currentApiGroup}
                        />
                        <BindingSummaryCard
                          label={t('dashboard.config.presetLabel')}
                          value={currentPreset}
                        />
                      </div>
                    </div>
                  </section>

                  <div className="border-t border-[var(--color-border-subtle)]" />

                  <section className="space-y-5">
                    <SectionHeader title={t('dashboard.sections.recentStories')} />
                    {currentDashboard.recent_stories.length === 0 ? (
                      <EmptySection label={t('dashboard.recentStories.empty')} />
                    ) : (
                      <div className="divide-y divide-[var(--color-border-subtle)]">
                        {currentDashboard.recent_stories.map((story) => (
                          <div className="space-y-3 py-4" key={story.story_id}>
                            <div className="flex items-center justify-between gap-3">
                              <p className="font-medium text-[var(--color-text-primary)]">
                                {story.display_name}
                              </p>
                              <p className="text-xs text-[var(--color-text-muted)]">
                                {story.updated_at_ms
                                  ? dateFormatter.format(story.updated_at_ms)
                                  : t('stage.time.unknown')}
                              </p>
                            </div>
                            <p className="text-sm text-[var(--color-text-secondary)]">
                              {t('dashboard.recentStories.resourcePrefix', {
                                id: story.resource_id,
                              })}
                            </p>
                            <p className="text-sm leading-7 text-[var(--color-text-primary)]">
                              {story.introduction}
                            </p>
                          </div>
                        ))}
                      </div>
                    )}
                  </section>

                  <div className="border-t border-[var(--color-border-subtle)]" />

                  <section className="space-y-5">
                    <SectionHeader title={t('dashboard.sections.recentSessions')} />
                    {currentDashboard.recent_sessions.length === 0 ? (
                      <EmptySection label={t('dashboard.recentSessions.empty')} />
                    ) : (
                      <div className="divide-y divide-[var(--color-border-subtle)]">
                        {currentDashboard.recent_sessions.map((session) => (
                          <div className="space-y-3 py-4" key={session.session_id}>
                            <div className="flex items-center justify-between gap-3">
                              <p className="font-medium text-[var(--color-text-primary)]">
                                {session.display_name}
                              </p>
                              <p className="text-xs text-[var(--color-text-muted)]">
                                {session.updated_at_ms
                                  ? dateFormatter.format(session.updated_at_ms)
                                  : t('stage.time.unknown')}
                              </p>
                            </div>
                            <div className="flex flex-wrap gap-2">
                              <Badge variant="subtle">
                                {t('dashboard.recentSessions.storyPrefix', { id: session.story_id })}
                              </Badge>
                              <Badge variant="subtle">
                                {t('dashboard.recentSessions.turnPrefix', { turn: session.turn_index })}
                              </Badge>
                            </div>
                          </div>
                        ))}
                      </div>
                    )}
                  </section>
                </>
              )}
            </div>
          </CardContent>
        </Card>
      </WorkspacePanelShell>
    </div>
  )
}
