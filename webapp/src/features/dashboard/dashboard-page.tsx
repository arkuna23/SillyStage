import { useCallback, useEffect, useLayoutEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { agentApiRoleKeys, type AgentApiIds, type AgentApiRoleKey } from '../apis/types'
import { getDashboard } from './api'
import type { DashboardPayload } from './types'
import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { useWorkspaceLayoutContext } from '../../components/layout/workspace-context'
import { Badge } from '../../components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../../components/ui/card'
import { SectionHeader } from '../../components/ui/section-header'
import { cn } from '../../lib/cn'

type NoticeTone = 'error'

type Notice = {
  message: string
  tone: NoticeTone
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
    api_ids: null,
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

function countAssignedApis(apiIds: AgentApiIds | null | undefined) {
  if (!apiIds) {
    return 0
  }

  return agentApiRoleKeys.reduce((count, roleKey) => {
    return count + (apiIds[roleKey].trim() ? 1 : 0)
  }, 0)
}

function StatusNotice({ notice }: { notice: Notice }) {
  return (
    <div
      className={cn(
        'rounded-[1.4rem] border px-4 py-3 text-sm leading-7 shadow-[0_14px_38px_rgba(0,0,0,0.12)] backdrop-blur',
        'border-[var(--color-state-error-line)] bg-[var(--color-state-error-soft)] text-[var(--color-text-primary)]',
      )}
      role="status"
    >
      {notice.message}
    </div>
  )
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

      <section className="grid gap-6 xl:grid-cols-[minmax(0,0.9fr)_minmax(0,1.1fr)]">
        {Array.from({ length: 2 }).map((_, index) => (
          <div
            className="rounded-[1.5rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-5 py-5"
            key={index}
          >
            <div className="h-6 w-32 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
            <div className="mt-4 space-y-3">
              {Array.from({ length: index === 0 ? 2 : 6 }).map((__, rowIndex) => (
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

export function DashboardPage() {
  const { t, i18n } = useTranslation()
  const { setRailContent } = useWorkspaceLayoutContext()
  const [dashboard, setDashboard] = useState<DashboardPayload | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [notice, setNotice] = useState<Notice | null>(null)

  const dateFormatter = useMemo(() => {
    return new Intl.DateTimeFormat(i18n.language.startsWith('zh') ? 'zh-CN' : 'en', {
      dateStyle: 'medium',
      timeStyle: 'short',
    })
  }, [i18n.language])

  const currentDashboard = dashboard ?? emptyDashboard
  const assignedApiCount = useMemo(
    () => countAssignedApis(currentDashboard.global_config.api_ids),
    [currentDashboard.global_config.api_ids],
  )
  const roleLabels: Record<AgentApiRoleKey, string> = useMemo(
    () => ({
      actor_api_id: t('apis.assignments.roles.actor_api_id'),
      architect_api_id: t('apis.assignments.roles.architect_api_id'),
      director_api_id: t('apis.assignments.roles.director_api_id'),
      keeper_api_id: t('apis.assignments.roles.keeper_api_id'),
      narrator_api_id: t('apis.assignments.roles.narrator_api_id'),
      planner_api_id: t('apis.assignments.roles.planner_api_id'),
      replyer_api_id: t('apis.assignments.roles.replyer_api_id'),
    }),
    [t],
  )
  const healthLabel =
    dashboard?.health.status === 'ok' ? t('dashboard.health.ok') : '—'
  const totalResourceCount =
    currentDashboard.counts.characters_total +
    currentDashboard.counts.story_resources_total +
    currentDashboard.counts.stories_total +
    currentDashboard.counts.sessions_total
  const recentActivityCount =
    currentDashboard.recent_stories.length + currentDashboard.recent_sessions.length

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
            tone: 'error',
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
  }, [
    healthLabel,
    isLoading,
    recentActivityCount,
    setRailContent,
    t,
    totalResourceCount,
  ])

  function formatUpdatedAt(updatedAtMs?: number | null) {
    if (!updatedAtMs) {
      return null
    }

    return dateFormatter.format(new Date(updatedAtMs))
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-6">
      <WorkspacePanelShell className="flex min-h-0 flex-1">
        <Card className="flex min-h-0 flex-1 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
          <CardHeader className="gap-4 border-b border-[var(--color-border-subtle)] px-6 py-5 md:min-h-[5.75rem] md:px-7 md:py-5">
            <SectionHeader title={t('dashboard.title')} />
          </CardHeader>

          <CardContent className="min-h-0 flex-1 overflow-y-auto pt-6">
            <div className="space-y-8 pr-1">
              {notice ? <StatusNotice notice={notice} /> : null}

              {isLoading ? (
                <DashboardSkeleton />
              ) : (
                <>
                  <section className="space-y-5">
                    <div className="space-y-2">
                      <CardTitle className="text-[1.85rem]">
                        {t('dashboard.sections.overview')}
                      </CardTitle>
                    </div>

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

                  <section className="grid gap-6 xl:grid-cols-[minmax(0,0.9fr)_minmax(0,1.1fr)]">
                    <div className="rounded-[1.5rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-5 py-5">
                      <div className="flex items-center justify-between gap-3">
                        <CardTitle className="text-[1.4rem]">
                          {t('dashboard.sections.health')}
                        </CardTitle>
                        <Badge variant="info">{healthLabel}</Badge>
                      </div>

                      <div className="mt-5 space-y-3">
                        <div className="rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_76%,transparent)] px-4 py-3.5">
                          <p className="text-xs text-[var(--color-text-muted)]">
                            {t('dashboard.metrics.status')}
                          </p>
                          <p className="mt-2 text-base font-medium text-[var(--color-text-primary)]">
                            {healthLabel}
                          </p>
                        </div>

                        <div className="rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_76%,transparent)] px-4 py-3.5">
                          <p className="text-xs text-[var(--color-text-muted)]">
                            {t('dashboard.config.summaryLabel')}
                          </p>
                          <p className="mt-2 text-base font-medium text-[var(--color-text-primary)]">
                            {t('dashboard.config.summary', {
                              assigned: assignedApiCount,
                              total: agentApiRoleKeys.length,
                            })}
                          </p>
                        </div>
                      </div>
                    </div>

                    <div className="rounded-[1.5rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-5 py-5">
                      <div className="space-y-1.5">
                        <CardTitle className="text-[1.4rem]">
                          {t('dashboard.sections.defaults')}
                        </CardTitle>
                        <CardDescription className="leading-6">
                          {t('dashboard.config.description')}
                        </CardDescription>
                      </div>

                      <div className="mt-5 grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
                        {agentApiRoleKeys.map((roleKey) => {
                          const apiId = currentDashboard.global_config.api_ids?.[roleKey]

                          return (
                            <div
                              className="rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_76%,transparent)] px-4 py-3.5"
                              key={roleKey}
                            >
                              <p className="text-xs text-[var(--color-text-muted)]">
                                {roleLabels[roleKey]}
                              </p>
                              <p className="mt-2 truncate text-sm font-medium text-[var(--color-text-primary)]">
                                {apiId || t('dashboard.config.emptyValue')}
                              </p>
                            </div>
                          )
                        })}
                      </div>
                    </div>
                  </section>

                  <div className="border-t border-[var(--color-border-subtle)]" />

                  <section className="space-y-5">
                    <div className="space-y-2">
                      <CardTitle className="text-[1.85rem]">
                        {t('dashboard.sections.recentStories')}
                      </CardTitle>
                    </div>

                    {currentDashboard.recent_stories.length === 0 ? (
                      <EmptySection label={t('dashboard.recentStories.empty')} />
                    ) : (
                      <div className="divide-y divide-[var(--color-border-subtle)]">
                        {currentDashboard.recent_stories.map((story) => (
                          <div className="space-y-3 py-4 first:pt-0 last:pb-0" key={story.story_id}>
                            <div className="flex flex-wrap items-start justify-between gap-3">
                              <div className="min-w-0 space-y-2">
                                <h3 className="truncate font-display text-[1.32rem] leading-tight text-[var(--color-text-primary)]">
                                  {story.display_name}
                                </h3>
                                <div className="flex flex-wrap gap-2">
                                  <Badge variant="subtle">{story.story_id}</Badge>
                                  <Badge variant="subtle">
                                    {t('dashboard.recentStories.resourcePrefix', {
                                      id: story.resource_id,
                                    })}
                                  </Badge>
                                </div>
                              </div>

                              {formatUpdatedAt(story.updated_at_ms) ? (
                                <p className="shrink-0 text-xs text-[var(--color-text-muted)]">
                                  {formatUpdatedAt(story.updated_at_ms)}
                                </p>
                              ) : null}
                            </div>

                            <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                              {story.introduction}
                            </p>
                          </div>
                        ))}
                      </div>
                    )}
                  </section>

                  <div className="border-t border-[var(--color-border-subtle)]" />

                  <section className="space-y-5">
                    <div className="space-y-2">
                      <CardTitle className="text-[1.85rem]">
                        {t('dashboard.sections.recentSessions')}
                      </CardTitle>
                    </div>

                    {currentDashboard.recent_sessions.length === 0 ? (
                      <EmptySection label={t('dashboard.recentSessions.empty')} />
                    ) : (
                      <div className="divide-y divide-[var(--color-border-subtle)]">
                        {currentDashboard.recent_sessions.map((session) => (
                          <div
                            className="space-y-3 py-4 first:pt-0 last:pb-0"
                            key={session.session_id}
                          >
                            <div className="flex flex-wrap items-start justify-between gap-3">
                              <div className="min-w-0 space-y-2">
                                <h3 className="truncate font-display text-[1.32rem] leading-tight text-[var(--color-text-primary)]">
                                  {session.display_name}
                                </h3>
                                <div className="flex flex-wrap gap-2">
                                  <Badge variant="subtle">{session.session_id}</Badge>
                                  <Badge variant="subtle">
                                    {t('dashboard.recentSessions.storyPrefix', {
                                      id: session.story_id,
                                    })}
                                  </Badge>
                                  <Badge variant="subtle">
                                    {t('dashboard.recentSessions.turnPrefix', {
                                      turn: session.turn_index,
                                    })}
                                  </Badge>
                                </div>
                              </div>

                              {formatUpdatedAt(session.updated_at_ms) ? (
                                <p className="shrink-0 text-xs text-[var(--color-text-muted)]">
                                  {formatUpdatedAt(session.updated_at_ms)}
                                </p>
                              ) : null}
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
