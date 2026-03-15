import { useCallback, useEffect, useLayoutEffect, useMemo, useState } from 'react'
import { faEye } from '@fortawesome/free-solid-svg-icons/faEye'
import { faPen } from '@fortawesome/free-solid-svg-icons/faPen'
import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useTranslation } from 'react-i18next'

import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { useWorkspaceLayoutContext } from '../../components/layout/workspace-context'
import { dispatchStageApiAvailabilityRefresh } from '../stage/stage-access'
import { Badge } from '../../components/ui/badge'
import { Card, CardContent, CardHeader } from '../../components/ui/card'
import { IconButton } from '../../components/ui/icon-button'
import { SectionHeader } from '../../components/ui/section-header'
import { useToastNotice } from '../../components/ui/toast-context'
import { isRpcConflict } from '../../lib/rpc'
import { ApiDetailsDialog } from './api-details-dialog'
import { ApiFormDialog } from './api-form-dialog'
import { ApiGroupDetailsDialog } from './api-group-details-dialog'
import { ApiGroupFormDialog } from './api-group-form-dialog'
import { deleteApi, deleteApiGroup, listApiGroups, listApis } from './api'
import { DeleteApiDialog } from './delete-api-dialog'
import { DeleteApiGroupDialog } from './delete-api-group-dialog'
import { agentRoleKeys, getAgentBindingKey, type AgentRoleKey, type ApiConfig, type ApiGroup } from './types'

type NoticeTone = 'error' | 'success' | 'warning'

type Notice = {
  message: string
  tone: NoticeTone
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function SectionSkeleton({ rows = 4 }: { rows?: number }) {
  return (
    <div className="space-y-5">
      <div className="h-8 w-40 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
      <div className="divide-y divide-[var(--color-border-subtle)]">
        {Array.from({ length: rows }).map((_, index) => (
          <div
            className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto] lg:items-center"
            key={index}
          >
            <div className="space-y-2.5">
              <div className="h-5 w-32 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
              <div className="h-3 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            </div>
            <div className="flex flex-wrap gap-2">
              {Array.from({ length: 3 }).map((__, badgeIndex) => (
                <div
                  className="h-8 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]"
                  key={badgeIndex}
                />
              ))}
            </div>
            <div className="flex justify-end gap-2">
              <div className="h-9 w-10 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
              <div className="h-9 w-10 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
              <div className="h-9 w-10 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

function isApiGroupResolved(apiGroup: ApiGroup, apis: ReadonlyArray<ApiConfig>) {
  return agentRoleKeys.every((roleKey) =>
    apis.some((apiConfig) => apiConfig.api_id === apiGroup.bindings[getAgentBindingKey(roleKey)]),
  )
}

export function ApiManagementPage() {
  const { t } = useTranslation()
  const { setRailContent } = useWorkspaceLayoutContext()
  const [apis, setApis] = useState<ApiConfig[]>([])
  const [apiGroups, setApiGroups] = useState<ApiGroup[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [notice, setNotice] = useState<Notice | null>(null)
  const [isDeletingApi, setIsDeletingApi] = useState(false)
  const [isDeletingGroup, setIsDeletingGroup] = useState(false)
  const [isCreateApiDialogOpen, setIsCreateApiDialogOpen] = useState(false)
  const [isCreateApiGroupDialogOpen, setIsCreateApiGroupDialogOpen] = useState(false)
  const [editApiId, setEditApiId] = useState<string | null>(null)
  const [editApiGroupId, setEditApiGroupId] = useState<string | null>(null)
  const [detailsApiId, setDetailsApiId] = useState<string | null>(null)
  const [detailsApiGroupId, setDetailsApiGroupId] = useState<string | null>(null)
  const [deleteApiTarget, setDeleteApiTarget] = useState<ApiConfig | null>(null)
  const [deleteGroupTarget, setDeleteGroupTarget] = useState<ApiGroup | null>(null)
  useToastNotice(notice)

  const apiIds = useMemo(() => apis.map((apiConfig) => apiConfig.api_id), [apis])
  const apiGroupIds = useMemo(() => apiGroups.map((apiGroup) => apiGroup.api_group_id), [apiGroups])
  const resolvedGroupCount = useMemo(
    () => apiGroups.filter((apiGroup) => isApiGroupResolved(apiGroup, apis)).length,
    [apiGroups, apis],
  )

  const roleLabels: Record<AgentRoleKey, string> = useMemo(
    () => ({
      actor: t('apis.roles.actor'),
      architect: t('apis.roles.architect'),
      director: t('apis.roles.director'),
      keeper: t('apis.roles.keeper'),
      narrator: t('apis.roles.narrator'),
      planner: t('apis.roles.planner'),
      replyer: t('apis.roles.replyer'),
    }),
    [t],
  )

  const refreshApiResources = useCallback(
    async (signal?: AbortSignal) => {
      setIsLoading(true)

      try {
        const [nextApis, nextApiGroups] = await Promise.all([
          listApis(signal),
          listApiGroups(signal),
        ])

        if (!signal?.aborted) {
          setApis(nextApis)
          setApiGroups(nextApiGroups)
          dispatchStageApiAvailabilityRefresh()
        }
      } catch (error) {
        if (!signal?.aborted) {
          setNotice({
            message: getErrorMessage(error, t('apis.feedback.loadListFailed')),
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
    void refreshApiResources(controller.signal)

    return () => {
      controller.abort()
    }
  }, [refreshApiResources])

  useLayoutEffect(() => {
    setRailContent({
      description: t('apis.rail.description'),
      stats: [
        {
          label: t('apis.metrics.apis'),
          value: apis.length,
        },
        {
          label: t('apis.metrics.groups'),
          value: apiGroups.length,
        },
        {
          label: t('apis.metrics.resolved'),
          value: resolvedGroupCount,
        },
      ],
      title: t('apis.title'),
    })

    return () => {
      setRailContent(null)
    }
  }, [apiGroups.length, apis.length, resolvedGroupCount, setRailContent, t])

  async function handleDeleteApi() {
    if (!deleteApiTarget) {
      return
    }

    const target = deleteApiTarget
    setIsDeletingApi(true)

    try {
      await deleteApi(target.api_id)
      setNotice({
        message: t('apis.apiFeedback.deleted', { id: target.display_name }),
        tone: 'success',
      })
      setDeleteApiTarget(null)
      await refreshApiResources()
    } catch (error) {
      setDeleteApiTarget(null)
      setNotice({
        message: isRpcConflict(error)
          ? t('apis.apiDeleteDialog.conflict')
          : getErrorMessage(error, t('apis.apiFeedback.deleteFailed')),
        tone: isRpcConflict(error) ? 'warning' : 'error',
      })
    } finally {
      setIsDeletingApi(false)
    }
  }

  async function handleDeleteApiGroup() {
    if (!deleteGroupTarget) {
      return
    }

    const target = deleteGroupTarget
    setIsDeletingGroup(true)

    try {
      await deleteApiGroup(target.api_group_id)
      setNotice({
        message: t('apis.groupFeedback.deleted', { id: target.display_name }),
        tone: 'success',
      })
      setDeleteGroupTarget(null)
      await refreshApiResources()
    } catch (error) {
      setDeleteGroupTarget(null)
      setNotice({
        message: isRpcConflict(error)
          ? t('apis.deleteDialog.conflict')
          : getErrorMessage(error, t('apis.groupFeedback.deleteFailed')),
        tone: isRpcConflict(error) ? 'warning' : 'error',
      })
    } finally {
      setIsDeletingGroup(false)
    }
  }

  function describeApiGroupBinding(apiGroup: ApiGroup, roleKey: AgentRoleKey) {
    const apiId = apiGroup.bindings[getAgentBindingKey(roleKey)]
    const apiConfig = apis.find((entry) => entry.api_id === apiId)

    return apiConfig
      ? `${roleLabels[roleKey]} · ${apiConfig.display_name}`
      : `${roleLabels[roleKey]} · ${t('apis.groupList.missing')}`
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-6">
      <ApiFormDialog
        existingApiIds={apiIds}
        mode="create"
        onCompleted={async ({ message }) => {
          setNotice({ message, tone: 'success' })
          await refreshApiResources()
        }}
        onOpenChange={setIsCreateApiDialogOpen}
        open={isCreateApiDialogOpen}
      />

      <ApiFormDialog
        apiId={editApiId}
        existingApiIds={apiIds}
        mode="edit"
        onCompleted={async ({ message }) => {
          setNotice({ message, tone: 'success' })
          await refreshApiResources()
        }}
        onOpenChange={(open) => {
          if (!open) {
            setEditApiId(null)
          }
        }}
        open={editApiId !== null}
      />

      <ApiDetailsDialog
        apiId={detailsApiId}
        onOpenChange={(open) => {
          if (!open) {
            setDetailsApiId(null)
          }
        }}
        open={detailsApiId !== null}
      />

      <DeleteApiDialog
        apiConfig={deleteApiTarget}
        deleting={isDeletingApi}
        onConfirm={() => void handleDeleteApi()}
        onOpenChange={(open) => {
          if (!open) {
            setDeleteApiTarget(null)
          }
        }}
        open={deleteApiTarget !== null}
      />

      <ApiGroupFormDialog
        apis={apis}
        existingApiGroupIds={apiGroupIds}
        mode="create"
        onCompleted={async ({ message }) => {
          setNotice({ message, tone: 'success' })
          await refreshApiResources()
        }}
        onOpenChange={setIsCreateApiGroupDialogOpen}
        open={isCreateApiGroupDialogOpen}
      />

      <ApiGroupFormDialog
        apiGroupId={editApiGroupId}
        apis={apis}
        existingApiGroupIds={apiGroupIds}
        mode="edit"
        onCompleted={async ({ message }) => {
          setNotice({ message, tone: 'success' })
          await refreshApiResources()
        }}
        onOpenChange={(open) => {
          if (!open) {
            setEditApiGroupId(null)
          }
        }}
        open={editApiGroupId !== null}
      />

      <ApiGroupDetailsDialog
        apiGroupId={detailsApiGroupId}
        apis={apis}
        onOpenChange={(open) => {
          if (!open) {
            setDetailsApiGroupId(null)
          }
        }}
        open={detailsApiGroupId !== null}
      />

      <DeleteApiGroupDialog
        apiGroup={deleteGroupTarget}
        deleting={isDeletingGroup}
        onConfirm={() => void handleDeleteApiGroup()}
        onOpenChange={(open) => {
          if (!open) {
            setDeleteGroupTarget(null)
          }
        }}
        open={deleteGroupTarget !== null}
      />

      <WorkspacePanelShell className="h-full min-h-0">
        <Card className="flex h-full min-h-0 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
          <CardHeader className="border-b border-[var(--color-border-subtle)] md:min-h-[92px]">
            <SectionHeader title={t('apis.title')} />
          </CardHeader>

          <CardContent className="scrollbar-none min-h-0 flex-1 overflow-y-auto pt-6">
            <div className="space-y-8">
              <section className="space-y-5">
                <SectionHeader
                  actions={
                    <IconButton
                      icon={<FontAwesomeIcon icon={faPlus} />}
                      label={t('apis.apiActions.create')}
                      onClick={() => {
                        setIsCreateApiDialogOpen(true)
                      }}
                    />
                  }
                  description={t('apis.apiSection.description')}
                  title={t('apis.apiSection.title')}
                />

                {isLoading ? (
                  <SectionSkeleton />
                ) : apis.length === 0 ? (
                  <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-4 py-5 text-sm leading-7 text-[var(--color-text-secondary)]">
                    {t('apis.apiEmpty.title')}
                  </div>
                ) : (
                  <div className="divide-y divide-[var(--color-border-subtle)]">
                    {apis.map((apiConfig) => (
                      <div
                        className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto] lg:items-center"
                        key={apiConfig.api_id}
                      >
                        <div className="space-y-2">
                          <p className="font-medium text-[var(--color-text-primary)]">
                            {apiConfig.display_name}
                          </p>
                          <p className="text-xs text-[var(--color-text-muted)]">{apiConfig.api_id}</p>
                        </div>

                        <div className="flex flex-wrap gap-2">
                          <Badge variant="subtle">
                            {apiConfig.provider === 'open_ai' ? t('apis.providers.open_ai') : apiConfig.provider}
                          </Badge>
                          <Badge variant="subtle">{apiConfig.model}</Badge>
                          <Badge variant={apiConfig.has_api_key ? 'info' : 'subtle'}>
                            {apiConfig.has_api_key
                              ? apiConfig.api_key_masked ?? t('apis.apiList.keyConfigured')
                              : t('apis.apiList.keyMissing')}
                          </Badge>
                        </div>

                        <div className="flex justify-start gap-2 lg:justify-end">
                          <IconButton
                            icon={<FontAwesomeIcon icon={faEye} />}
                            label={t('apis.actions.view')}
                            onClick={() => {
                              setDetailsApiId(apiConfig.api_id)
                            }}
                            size="sm"
                            variant="ghost"
                          />
                          <IconButton
                            icon={<FontAwesomeIcon icon={faPen} />}
                            label={t('apis.actions.edit')}
                            onClick={() => {
                              setEditApiId(apiConfig.api_id)
                            }}
                            size="sm"
                            variant="secondary"
                          />
                          <IconButton
                            icon={<FontAwesomeIcon icon={faTrashCan} />}
                            label={t('apis.actions.delete')}
                            onClick={() => {
                              setDeleteApiTarget(apiConfig)
                            }}
                            size="sm"
                            variant="danger"
                          />
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </section>

              <div className="border-t border-[var(--color-border-subtle)]" />

              <section className="space-y-5">
                <SectionHeader
                  actions={
                    <IconButton
                      disabled={apis.length === 0}
                      icon={<FontAwesomeIcon icon={faPlus} />}
                      label={t('apis.actions.create')}
                      onClick={() => {
                        setIsCreateApiGroupDialogOpen(true)
                      }}
                    />
                  }
                  description={t('apis.groupSection.description')}
                  title={t('apis.groupSection.title')}
                />

                {isLoading ? (
                  <SectionSkeleton />
                ) : apiGroups.length === 0 ? (
                  <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-4 py-5 text-sm leading-7 text-[var(--color-text-secondary)]">
                    {apis.length === 0 ? t('apis.groupEmpty.noApis') : t('apis.groupEmpty.title')}
                  </div>
                ) : (
                  <div className="divide-y divide-[var(--color-border-subtle)]">
                    {apiGroups.map((apiGroup) => (
                      <div
                        className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto] lg:items-center"
                        key={apiGroup.api_group_id}
                      >
                        <div className="space-y-2">
                          <p className="font-medium text-[var(--color-text-primary)]">
                            {apiGroup.display_name}
                          </p>
                          <p className="text-xs text-[var(--color-text-muted)]">
                            {apiGroup.api_group_id}
                          </p>
                        </div>

                        <div className="flex flex-wrap gap-2">
                          {agentRoleKeys.slice(0, 3).map((roleKey) => (
                            <Badge key={roleKey} variant="subtle">
                              {describeApiGroupBinding(apiGroup, roleKey)}
                            </Badge>
                          ))}
                          <Badge variant={isApiGroupResolved(apiGroup, apis) ? 'info' : 'subtle'}>
                            {isApiGroupResolved(apiGroup, apis)
                              ? t('apis.groupList.resolved')
                              : t('apis.groupList.missing')}
                          </Badge>
                        </div>

                        <div className="flex justify-start gap-2 lg:justify-end">
                          <IconButton
                            icon={<FontAwesomeIcon icon={faEye} />}
                            label={t('apis.actions.view')}
                            onClick={() => {
                              setDetailsApiGroupId(apiGroup.api_group_id)
                            }}
                            size="sm"
                            variant="ghost"
                          />
                          <IconButton
                            icon={<FontAwesomeIcon icon={faPen} />}
                            label={t('apis.actions.edit')}
                            onClick={() => {
                              setEditApiGroupId(apiGroup.api_group_id)
                            }}
                            size="sm"
                            variant="secondary"
                          />
                          <IconButton
                            icon={<FontAwesomeIcon icon={faTrashCan} />}
                            label={t('apis.actions.delete')}
                            onClick={() => {
                              setDeleteGroupTarget(apiGroup)
                            }}
                            size="sm"
                            variant="danger"
                          />
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
