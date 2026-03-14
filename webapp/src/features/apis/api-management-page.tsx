import { useCallback, useEffect, useLayoutEffect, useState } from 'react'
import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useTranslation } from 'react-i18next'

import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { useWorkspaceLayoutContext } from '../../components/layout/workspace-context'
import { Button } from '../../components/ui/button'
import { IconButton } from '../../components/ui/icon-button'
import {
  Dialog,
  DialogBody,
  DialogClose,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../components/ui/dialog'
import { Select } from '../../components/ui/select'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../../components/ui/card'
import { SectionHeader } from '../../components/ui/section-header'
import { Badge } from '../../components/ui/badge'
import { cn } from '../../lib/cn'
import { isRpcConflict } from '../../lib/rpc'
import {
  deleteLlmApi,
  getGlobalApiConfig,
  listLlmApis,
  updateGlobalApiConfig,
} from './api'
import { LlmApiDetailsDialog } from './llm-api-details-dialog'
import { LlmApiFormDialog } from './llm-api-form-dialog'
import { agentApiRoleKeys, type AgentApiIds, type AgentApiRoleKey, type LlmApi } from './types'

type NoticeTone = 'error' | 'success' | 'warning'

type Notice = {
  message: string
  tone: NoticeTone
}

const roleOrder = [
  'planner_api_id',
  'architect_api_id',
  'director_api_id',
  'actor_api_id',
  'narrator_api_id',
  'keeper_api_id',
] as const satisfies ReadonlyArray<AgentApiRoleKey>

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function countAssignedRoles(apiIds: AgentApiIds | null) {
  if (!apiIds) {
    return 0
  }

  return roleOrder.reduce((count, roleKey) => count + (apiIds[roleKey].trim() ? 1 : 0), 0)
}

function providerLabel(
  provider: LlmApi['provider'],
  t: ReturnType<typeof useTranslation>['t'],
) {
  return provider === 'open_ai' ? t('apis.providers.open_ai') : provider
}

function roleLabel(
  roleKey: AgentApiRoleKey,
  t: ReturnType<typeof useTranslation>['t'],
) {
  switch (roleKey) {
    case 'planner_api_id':
      return t('apis.assignments.roles.planner_api_id')
    case 'architect_api_id':
      return t('apis.assignments.roles.architect_api_id')
    case 'director_api_id':
      return t('apis.assignments.roles.director_api_id')
    case 'actor_api_id':
      return t('apis.assignments.roles.actor_api_id')
    case 'narrator_api_id':
      return t('apis.assignments.roles.narrator_api_id')
    case 'keeper_api_id':
      return t('apis.assignments.roles.keeper_api_id')
  }
}

function keyLabel(api: LlmApi, t: ReturnType<typeof useTranslation>['t']) {
  if (!api.has_api_key) {
    return t('apis.list.keyMissing')
  }

  return api.api_key_masked ?? t('apis.list.keyConfigured')
}

function StatusNotice({ notice }: { notice: Notice }) {
  return (
    <div
      className={cn(
        'rounded-[1.4rem] border px-4 py-3 text-sm leading-7 shadow-[0_14px_38px_rgba(0,0,0,0.12)] backdrop-blur',
        notice.tone === 'success'
          ? 'border-[var(--color-state-success-line)] bg-[var(--color-state-success-soft)] text-[var(--color-text-primary)]'
          : notice.tone === 'warning'
            ? 'border-[var(--color-state-warning-line)] bg-[var(--color-state-warning-soft)] text-[var(--color-text-primary)]'
            : 'border-[var(--color-state-error-line)] bg-[var(--color-state-error-soft)] text-[var(--color-text-primary)]',
      )}
      role="status"
    >
      {notice.message}
    </div>
  )
}

function AssignmentsSkeleton() {
  return (
    <div className="space-y-5">
      <div className="h-8 w-40 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
      <div className="grid gap-4 md:grid-cols-2 2xl:grid-cols-3">
        {Array.from({ length: roleOrder.length }).map((_, index) => (
          <div className="space-y-2.5" key={index}>
            <div className="h-3 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-elevated)]" />
          </div>
        ))}
      </div>
    </div>
  )
}

function ApiListSkeleton() {
  return (
    <div className="space-y-5">
      <div className="h-8 w-36 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
      <div className="divide-y divide-[var(--color-border-subtle)]">
        {Array.from({ length: 5 }).map((_, index) => (
          <div
            className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1.15fr)_minmax(0,0.9fr)_minmax(0,0.75fr)_auto]"
            key={index}
          >
            <div className="space-y-2.5">
              <div className="h-5 w-28 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
              <div className="h-3 w-full animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            </div>
            <div className="space-y-2.5">
              <div className="h-3 w-14 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
              <div className="h-4 w-32 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            </div>
            <div className="space-y-2.5">
              <div className="h-3 w-14 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
              <div className="h-4 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            </div>
            <div className="flex gap-2">
              <div className="h-9 w-16 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
              <div className="h-9 w-16 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

function DeleteApiDialog({
  api,
  deleting,
  onConfirm,
  onOpenChange,
}: {
  api: LlmApi | null
  deleting: boolean
  onConfirm: () => void
  onOpenChange: (open: boolean) => void
}) {
  const { t } = useTranslation()

  return (
    <Dialog
      onOpenChange={(open) => {
        if (!open) {
          onOpenChange(false)
        }
      }}
      open={api !== null}
    >
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,30rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{t('apis.deleteDialog.title')}</DialogTitle>
        </DialogHeader>

        <DialogBody className="pt-6">
          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
            {t('apis.deleteDialog.message', { id: api?.api_id ?? '' })}
          </p>
        </DialogBody>

        <DialogFooter>
          <DialogClose asChild>
            <Button disabled={deleting} variant="ghost">
              {t('apis.actions.cancel')}
            </Button>
          </DialogClose>

          <Button
            className="border-[var(--color-state-error-line)] bg-[var(--color-state-error)] text-[var(--color-accent-ink)] hover:bg-[color-mix(in_srgb,var(--color-state-error)_90%,black)]"
            disabled={deleting}
            onClick={onConfirm}
          >
            {deleting ? t('apis.actions.deleting') : t('apis.actions.confirmDelete')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export function ApiManagementPage() {
  const { t } = useTranslation()
  const { setRailContent } = useWorkspaceLayoutContext()
  const [apis, setApis] = useState<LlmApi[]>([])
  const [assignments, setAssignments] = useState<AgentApiIds | null>(null)
  const [draftAssignments, setDraftAssignments] = useState<AgentApiIds | null>(null)
  const [isAssignmentsLoading, setIsAssignmentsLoading] = useState(true)
  const [isListLoading, setIsListLoading] = useState(true)
  const [isSavingAssignments, setIsSavingAssignments] = useState(false)
  const [isDeleting, setIsDeleting] = useState(false)
  const [notice, setNotice] = useState<Notice | null>(null)
  const [detailsApiId, setDetailsApiId] = useState<string | null>(null)
  const [editApiId, setEditApiId] = useState<string | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<LlmApi | null>(null)
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)

  const apiOptions = apis.map((api) => ({
    label: api.api_id,
    value: api.api_id,
  }))

  const assignmentsDirty =
    assignments !== null &&
    draftAssignments !== null &&
    roleOrder.some((roleKey) => assignments[roleKey] !== draftAssignments[roleKey])

  const refreshApis = useCallback(
    async (signal?: AbortSignal) => {
      setIsListLoading(true)

      try {
        const nextApis = await listLlmApis(signal)

        if (!signal?.aborted) {
          setApis(nextApis)
        }
      } catch (error) {
        if (!signal?.aborted) {
          setNotice({
            message: getErrorMessage(error, t('apis.feedback.loadApisFailed')),
            tone: 'error',
          })
        }
      } finally {
        if (!signal?.aborted) {
          setIsListLoading(false)
        }
      }
    },
    [t],
  )

  const refreshAssignments = useCallback(
    async (signal?: AbortSignal) => {
      setIsAssignmentsLoading(true)

      try {
        const nextConfig = await getGlobalApiConfig(signal)

        if (!signal?.aborted) {
          setAssignments(nextConfig.api_ids)
          setDraftAssignments(nextConfig.api_ids)
        }
      } catch (error) {
        if (!signal?.aborted) {
          setNotice({
            message: getErrorMessage(error, t('apis.feedback.loadConfigFailed')),
            tone: 'error',
          })
        }
      } finally {
        if (!signal?.aborted) {
          setIsAssignmentsLoading(false)
        }
      }
    },
    [t],
  )

  useEffect(() => {
    const controller = new AbortController()

    void Promise.allSettled([
      refreshApis(controller.signal),
      refreshAssignments(controller.signal),
    ])

    return () => {
      controller.abort()
    }
  }, [refreshApis, refreshAssignments])

  useLayoutEffect(() => {
    setRailContent({
      description: t('apis.rail.description'),
      stats: [
        {
          label: t('apis.metrics.total'),
          value: apis.length,
        },
        {
          label: t('apis.metrics.assigned'),
          value: assignments ? `${countAssignedRoles(assignments)}/${agentApiRoleKeys.length}` : '—',
        },
      ],
      title: t('apis.title'),
    })

    return () => {
      setRailContent(null)
    }
  }, [apis.length, assignments, setRailContent, t])

  async function handleSaveAssignments() {
    if (!assignments || !draftAssignments || !assignmentsDirty) {
      return
    }

    const overrides = roleOrder.reduce<Partial<AgentApiIds>>((current, roleKey) => {
      if (assignments[roleKey] !== draftAssignments[roleKey]) {
        current[roleKey] = draftAssignments[roleKey]
      }

      return current
    }, {})

    setIsSavingAssignments(true)

    try {
      const updatedConfig = await updateGlobalApiConfig(overrides)

      setAssignments(updatedConfig.api_ids)
      setDraftAssignments(updatedConfig.api_ids)
      setNotice({
        message: t('apis.feedback.defaultsSaved'),
        tone: 'success',
      })
    } catch (error) {
      setNotice({
        message: getErrorMessage(error, t('apis.feedback.loadConfigFailed')),
        tone: 'error',
      })
    } finally {
      setIsSavingAssignments(false)
    }
  }

  async function handleDelete() {
    if (!deleteTarget) {
      return
    }

    setIsDeleting(true)

    try {
      await deleteLlmApi(deleteTarget.api_id)
      setDeleteTarget(null)
      setDetailsApiId((current) => (current === deleteTarget.api_id ? null : current))
      setEditApiId((current) => (current === deleteTarget.api_id ? null : current))
      setNotice({
        message: t('apis.feedback.deleted', { id: deleteTarget.api_id }),
        tone: 'success',
      })
      await refreshApis()
    } catch (error) {
      setNotice({
        message: isRpcConflict(error)
          ? t('apis.deleteDialog.conflict')
          : getErrorMessage(error, t('apis.feedback.deleteFailed')),
        tone: isRpcConflict(error) ? 'warning' : 'error',
      })
    } finally {
      setIsDeleting(false)
    }
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-6">
      <LlmApiDetailsDialog
        apiId={detailsApiId}
        key={detailsApiId ?? 'details-dialog'}
        onOpenChange={(open) => {
          if (!open) {
            setDetailsApiId(null)
          }
        }}
        open={detailsApiId !== null}
      />

      <LlmApiFormDialog
        existingApiIds={apis.map((api) => api.api_id)}
        mode="create"
        onCompleted={async ({ message }) => {
          setNotice({ message, tone: 'success' })
          await refreshApis()
        }}
        onOpenChange={setIsCreateDialogOpen}
        open={isCreateDialogOpen}
      />

      <LlmApiFormDialog
        apiId={editApiId}
        existingApiIds={apis.map((api) => api.api_id)}
        mode="edit"
        onCompleted={async ({ message }) => {
          setNotice({ message, tone: 'success' })
          await refreshApis()
        }}
        onOpenChange={(open) => {
          if (!open) {
            setEditApiId(null)
          }
        }}
        open={editApiId !== null}
      />

      <DeleteApiDialog
        api={deleteTarget}
        deleting={isDeleting}
        onConfirm={() => {
          void handleDelete()
        }}
        onOpenChange={() => {
          setDeleteTarget(null)
        }}
      />

      <WorkspacePanelShell className="flex min-h-0 flex-1">
        <Card className="flex min-h-0 flex-1 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
        <CardHeader className="gap-4 border-b border-[var(--color-border-subtle)] px-6 py-5 md:min-h-[5.75rem] md:px-7 md:py-5">
          <SectionHeader
            actions={
              <div className="flex min-h-10 items-center justify-end">
                <IconButton
                  icon={<FontAwesomeIcon icon={faPlus} />}
                  label={t('apis.actions.create')}
                  onClick={() => {
                    setIsCreateDialogOpen(true)
                  }}
                  size="md"
                />
              </div>
            }
            title={t('apis.title')}
          />
        </CardHeader>

        <CardContent className="min-h-0 flex-1 overflow-y-auto pt-6">
          <div className="space-y-6 pr-1">
            {notice ? <StatusNotice notice={notice} /> : null}

            <section className="space-y-5">
              <div className="flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
                <div className="space-y-2">
                  <CardTitle className="text-[1.85rem]">{t('apis.assignments.title')}</CardTitle>
                  <CardDescription>{t('apis.assignments.description')}</CardDescription>
                </div>

                <Button
                  disabled={!assignmentsDirty || isSavingAssignments || apiOptions.length === 0}
                  onClick={() => void handleSaveAssignments()}
                  size="sm"
                >
                  {isSavingAssignments ? t('apis.actions.saving') : t('apis.actions.saveAssignments')}
                </Button>
              </div>

              {isAssignmentsLoading ? (
                <AssignmentsSkeleton />
              ) : apiOptions.length === 0 ? (
                <div className="py-3 text-sm leading-7 text-[var(--color-text-secondary)]">
                  {t('apis.assignments.empty')}
                </div>
              ) : (
                <div className="grid gap-4 md:grid-cols-2 2xl:grid-cols-3">
                  {roleOrder.map((roleKey) => (
                    <div className="space-y-2.5" key={roleKey}>
                      <p className="text-sm font-medium text-[var(--color-text-primary)]">
                        {roleLabel(roleKey, t)}
                      </p>
                      <Select
                        disabled={isSavingAssignments || draftAssignments === null}
                        items={apiOptions}
                        onValueChange={(value) => {
                          setDraftAssignments((current) =>
                            current
                              ? {
                                  ...current,
                                  [roleKey]: value,
                                }
                              : current,
                          )
                        }}
                        placeholder={t('apis.assignments.selectPlaceholder')}
                        value={draftAssignments?.[roleKey]}
                      />
                    </div>
                  ))}
                </div>
              )}
            </section>

            <div className="border-t border-[var(--color-border-subtle)]" />

            <section className="space-y-5">
              <CardTitle className="text-[1.85rem]">{t('apis.list.title')}</CardTitle>

              {isListLoading ? (
                <ApiListSkeleton />
              ) : apis.length === 0 ? (
                <div className="py-12 text-center">
                  <h3 className="font-display text-3xl text-[var(--color-text-primary)]">
                    {t('apis.list.emptyTitle')}
                  </h3>

                  <div className="mt-7 flex justify-center">
                    <Button
                      onClick={() => {
                        setIsCreateDialogOpen(true)
                      }}
                    >
                      {t('apis.actions.create')}
                    </Button>
                  </div>
                </div>
              ) : (
                <div className="divide-y divide-[var(--color-border-subtle)]">
                  {apis.map((api) => (
                    <div
                      className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1.15fr)_minmax(0,0.95fr)_minmax(0,0.8fr)_auto] lg:items-center"
                      key={api.api_id}
                    >
                      <div className="min-w-0 space-y-2">
                        <div className="flex flex-wrap items-center gap-2.5">
                          <h3 className="truncate font-display text-[1.32rem] leading-tight text-[var(--color-text-primary)]">
                            {api.api_id}
                          </h3>
                          <Badge className="uppercase" variant="subtle">
                            {providerLabel(api.provider, t)}
                          </Badge>
                        </div>

                        <p className="truncate font-mono text-[0.76rem] leading-5 text-[var(--color-text-muted)]">
                          {api.base_url}
                        </p>
                      </div>

                      <div className="min-w-0 space-y-1.5">
                        <p className="text-xs text-[var(--color-text-muted)]">
                          {t('apis.list.model')}
                        </p>
                        <p className="truncate text-sm text-[var(--color-text-primary)]">
                          {api.model}
                        </p>
                      </div>

                      <div className="min-w-0 space-y-1.5">
                        <p className="text-xs text-[var(--color-text-muted)]">
                          {t('apis.list.apiKey')}
                        </p>
                        <p className="truncate text-sm text-[var(--color-text-primary)]">
                          {keyLabel(api, t)}
                        </p>
                      </div>

                      <div className="flex flex-wrap items-center justify-start gap-2 lg:justify-end">
                        <Button
                          className="text-[var(--color-text-primary)]"
                          onClick={() => {
                            setDetailsApiId(api.api_id)
                          }}
                          size="sm"
                          variant="ghost"
                        >
                          {t('apis.actions.view')}
                        </Button>
                        <Button
                          className="border-[var(--color-accent-copper-soft)] bg-[color-mix(in_srgb,var(--color-accent-copper-soft)_78%,var(--color-bg-panel-strong))] text-[var(--color-accent-copper)] hover:bg-[color-mix(in_srgb,var(--color-accent-copper-soft)_100%,var(--color-bg-panel-strong))] hover:text-[var(--color-text-primary)]"
                          onClick={() => {
                            setEditApiId(api.api_id)
                          }}
                          size="sm"
                          variant="ghost"
                        >
                          {t('apis.actions.edit')}
                        </Button>
                        <Button
                          className="border-[var(--color-state-error-line)] bg-[var(--color-state-error-soft)] text-[var(--color-state-error)] hover:bg-[color-mix(in_srgb,var(--color-state-error-soft)_100%,var(--color-bg-panel-strong))] hover:text-[var(--color-text-primary)]"
                          onClick={() => {
                            setDeleteTarget(api)
                          }}
                          size="sm"
                          variant="ghost"
                        >
                          {t('apis.actions.delete')}
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </section>
          </div>
        </CardContent>
        </Card>
      </WorkspacePanelShell>
    </div>
  )
}
