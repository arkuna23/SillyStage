import { useCallback, useEffect, useLayoutEffect, useMemo, useState } from 'react'
import { faCheckDouble } from '@fortawesome/free-solid-svg-icons/faCheckDouble'
import { faEye } from '@fortawesome/free-solid-svg-icons/faEye'
import { faPen } from '@fortawesome/free-solid-svg-icons/faPen'
import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { faSquareCheck } from '@fortawesome/free-solid-svg-icons/faSquareCheck'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
import { faXmark } from '@fortawesome/free-solid-svg-icons/faXmark'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useTranslation } from 'react-i18next'

import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { useWorkspaceLayoutContext } from '../../components/layout/workspace-context'
import { dispatchStageApiAvailabilityRefresh } from '../stage/stage-access'
import { Badge } from '../../components/ui/badge'
import { Card, CardContent, CardHeader } from '../../components/ui/card'
import { IconButton } from '../../components/ui/icon-button'
import { SelectionToggleButton } from '../../components/ui/selection-toggle-button'
import { SectionHeader } from '../../components/ui/section-header'
import { useToastNotice } from '../../components/ui/toast-context'
import { runBatchDelete } from '../../lib/batch-delete'
import { ApiDetailsDialog } from './api-details-dialog'
import { ApiFormDialog } from './api-form-dialog'
import { ApiGroupDetailsDialog } from './api-group-details-dialog'
import { ApiGroupFormDialog } from './api-group-form-dialog'
import { deleteApi, deleteApiGroup, listApiGroups, listApis } from './api'
import { DeleteApiDialog } from './delete-api-dialog'
import { DeleteApiGroupDialog } from './delete-api-group-dialog'
import { agentRoleKeys, getAgentBindingKey, type ApiConfig, type ApiGroup } from './types'

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
  const [apiSelectionMode, setApiSelectionMode] = useState(false)
  const [groupSelectionMode, setGroupSelectionMode] = useState(false)
  const [selectedApiIds, setSelectedApiIds] = useState<string[]>([])
  const [selectedGroupIds, setSelectedGroupIds] = useState<string[]>([])
  const [deleteApiTargetIds, setDeleteApiTargetIds] = useState<string[]>([])
  const [deleteGroupTargetIds, setDeleteGroupTargetIds] = useState<string[]>([])
  useToastNotice(notice)

  const apiIds = useMemo(() => apis.map((apiConfig) => apiConfig.api_id), [apis])
  const apiGroupIds = useMemo(() => apiGroups.map((apiGroup) => apiGroup.api_group_id), [apiGroups])
  const deleteApiTargets = useMemo(
    () =>
      deleteApiTargetIds
        .map((apiId) => apis.find((apiConfig) => apiConfig.api_id === apiId))
        .filter((apiConfig): apiConfig is ApiConfig => apiConfig !== undefined),
    [apis, deleteApiTargetIds],
  )
  const deleteGroupTargets = useMemo(
    () =>
      deleteGroupTargetIds
        .map((apiGroupId) => apiGroups.find((apiGroup) => apiGroup.api_group_id === apiGroupId))
        .filter((apiGroup): apiGroup is ApiGroup => apiGroup !== undefined),
    [apiGroups, deleteGroupTargetIds],
  )
  const resolvedGroupCount = useMemo(
    () => apiGroups.filter((apiGroup) => isApiGroupResolved(apiGroup, apis)).length,
    [apiGroups, apis],
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

  useEffect(() => {
    const availableApiIds = new Set(apis.map((apiConfig) => apiConfig.api_id))
    const availableGroupIds = new Set(apiGroups.map((apiGroup) => apiGroup.api_group_id))

    setSelectedApiIds((currentSelection) => currentSelection.filter((apiId) => availableApiIds.has(apiId)))
    setDeleteApiTargetIds((currentSelection) =>
      currentSelection.filter((apiId) => availableApiIds.has(apiId)),
    )
    setSelectedGroupIds((currentSelection) =>
      currentSelection.filter((apiGroupId) => availableGroupIds.has(apiGroupId)),
    )
    setDeleteGroupTargetIds((currentSelection) =>
      currentSelection.filter((apiGroupId) => availableGroupIds.has(apiGroupId)),
    )

    if (detailsApiId !== null && !availableApiIds.has(detailsApiId)) {
      setDetailsApiId(null)
    }

    if (editApiId !== null && !availableApiIds.has(editApiId)) {
      setEditApiId(null)
    }

    if (detailsApiGroupId !== null && !availableGroupIds.has(detailsApiGroupId)) {
      setDetailsApiGroupId(null)
    }

    if (editApiGroupId !== null && !availableGroupIds.has(editApiGroupId)) {
      setEditApiGroupId(null)
    }
  }, [apiGroups, apis, detailsApiGroupId, detailsApiId, editApiGroupId, editApiId])

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
    if (deleteApiTargets.length === 0) {
      return
    }

    setIsDeletingApi(true)

    try {
      const result = await runBatchDelete(deleteApiTargets, async (target) => {
        await deleteApi(target.api_id)
      })

      setDeleteApiTargetIds([])

      if (result.deleted.length > 0) {
        const deletedIds = new Set(result.deleted.map((target) => target.api_id))

        setSelectedApiIds((currentSelection) =>
          currentSelection.filter((apiId) => !deletedIds.has(apiId)),
        )
        setDeleteApiTargetIds([])

        if (detailsApiId !== null && deletedIds.has(detailsApiId)) {
          setDetailsApiId(null)
        }

        if (editApiId !== null && deletedIds.has(editApiId)) {
          setEditApiId(null)
        }
      }

      if (result.failed.length === 0) {
        setNotice({
          message:
            result.deleted.length > 1
              ? t('apis.apiFeedback.deletedMany', { count: result.deleted.length })
              : t('apis.apiFeedback.deleted', { id: result.deleted[0]?.display_name ?? '' }),
          tone: 'success',
        })
        if (apiSelectionMode) {
          setApiSelectionMode(false)
          setSelectedApiIds([])
        }
      } else if (result.deleted.length > 0) {
        setNotice({
          message: t('apis.apiFeedback.deletedPartial', {
            failed: result.failed.length,
            success: result.deleted.length,
          }),
          tone: 'warning',
        })
      } else {
        setNotice({
          message:
            result.conflictCount > 0
              ? deleteApiTargets.length > 1
                ? t('apis.apiDeleteDialog.conflictMany')
                : t('apis.apiDeleteDialog.conflict')
              : t('apis.apiFeedback.deleteFailed'),
          tone: result.conflictCount > 0 ? 'warning' : 'error',
        })
      }

      await refreshApiResources()
    } finally {
      setIsDeletingApi(false)
    }
  }

  async function handleDeleteApiGroup() {
    if (deleteGroupTargets.length === 0) {
      return
    }

    setIsDeletingGroup(true)

    try {
      const result = await runBatchDelete(deleteGroupTargets, async (target) => {
        await deleteApiGroup(target.api_group_id)
      })

      setDeleteGroupTargetIds([])

      if (result.deleted.length > 0) {
        const deletedIds = new Set(result.deleted.map((target) => target.api_group_id))

        setSelectedGroupIds((currentSelection) =>
          currentSelection.filter((apiGroupId) => !deletedIds.has(apiGroupId)),
        )
        setDeleteGroupTargetIds([])

        if (detailsApiGroupId !== null && deletedIds.has(detailsApiGroupId)) {
          setDetailsApiGroupId(null)
        }

        if (editApiGroupId !== null && deletedIds.has(editApiGroupId)) {
          setEditApiGroupId(null)
        }
      }

      if (result.failed.length === 0) {
        setNotice({
          message:
            result.deleted.length > 1
              ? t('apis.groupFeedback.deletedMany', { count: result.deleted.length })
              : t('apis.groupFeedback.deleted', { id: result.deleted[0]?.display_name ?? '' }),
          tone: 'success',
        })
        if (groupSelectionMode) {
          setGroupSelectionMode(false)
          setSelectedGroupIds([])
        }
      } else if (result.deleted.length > 0) {
        setNotice({
          message: t('apis.groupFeedback.deletedPartial', {
            failed: result.failed.length,
            success: result.deleted.length,
          }),
          tone: 'warning',
        })
      } else {
        setNotice({
          message:
            result.conflictCount > 0
              ? deleteGroupTargets.length > 1
                ? t('apis.deleteDialog.conflictMany')
                : t('apis.deleteDialog.conflict')
              : t('apis.groupFeedback.deleteFailed'),
          tone: result.conflictCount > 0 ? 'warning' : 'error',
        })
      }

      await refreshApiResources()
    } finally {
      setIsDeletingGroup(false)
    }
  }

  function enterApiSelectionMode() {
    setGroupSelectionMode(false)
    setSelectedGroupIds([])
    setApiSelectionMode(true)
    setSelectedApiIds([])
    setDetailsApiId(null)
  }

  function enterGroupSelectionMode() {
    setApiSelectionMode(false)
    setSelectedApiIds([])
    setGroupSelectionMode(true)
    setSelectedGroupIds([])
    setDetailsApiGroupId(null)
  }

  function toggleApiSelection(apiId: string) {
    setSelectedApiIds((currentSelection) =>
      currentSelection.includes(apiId)
        ? currentSelection.filter((currentApiId) => currentApiId !== apiId)
        : [...currentSelection, apiId],
    )
  }

  function toggleGroupSelection(apiGroupId: string) {
    setSelectedGroupIds((currentSelection) =>
      currentSelection.includes(apiGroupId)
        ? currentSelection.filter((currentApiGroupId) => currentApiGroupId !== apiGroupId)
        : [...currentSelection, apiGroupId],
    )
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
        deleting={isDeletingApi}
        onConfirm={() => void handleDeleteApi()}
        onOpenChange={(open) => {
          if (!open) {
            setDeleteApiTargetIds([])
          }
        }}
        targets={deleteApiTargets}
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
        deleting={isDeletingGroup}
        onConfirm={() => void handleDeleteApiGroup()}
        onOpenChange={(open) => {
          if (!open) {
            setDeleteGroupTargetIds([])
          }
        }}
        targets={deleteGroupTargets}
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
                    <div className="flex flex-wrap items-center justify-end gap-2">
                      {apiSelectionMode ? (
                        <>
                          <Badge className="normal-case px-3.5 py-2" variant="subtle">
                            {t('apis.selection.count', { count: selectedApiIds.length })}
                          </Badge>
                          <IconButton
                            disabled={apis.length === 0}
                            icon={<FontAwesomeIcon icon={faCheckDouble} />}
                            label={t('apis.actions.selectAll')}
                            onClick={() => {
                              setSelectedApiIds(apis.map((apiConfig) => apiConfig.api_id))
                            }}
                            variant="secondary"
                          />
                          <IconButton
                            disabled={selectedApiIds.length === 0}
                            icon={<FontAwesomeIcon icon={faTrashCan} />}
                            label={t('apis.actions.deleteSelected')}
                            onClick={() => {
                              setDeleteApiTargetIds(selectedApiIds)
                            }}
                            variant="danger"
                          />
                          <IconButton
                            icon={<FontAwesomeIcon icon={faXmark} />}
                            label={t('apis.actions.cancelSelection')}
                            onClick={() => {
                              setApiSelectionMode(false)
                              setSelectedApiIds([])
                            }}
                            variant="secondary"
                          />
                        </>
                      ) : (
                        <>
                          <IconButton
                            icon={<FontAwesomeIcon icon={faSquareCheck} />}
                            label={t('apis.actions.selectMode')}
                            onClick={enterApiSelectionMode}
                            variant="secondary"
                          />
                          <IconButton
                            icon={<FontAwesomeIcon icon={faPlus} />}
                            label={t('apis.apiActions.create')}
                            onClick={() => {
                              setIsCreateApiDialogOpen(true)
                            }}
                          />
                        </>
                      )}
                    </div>
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
                        </div>

                        <div className="flex justify-start gap-2 lg:justify-end">
                          {apiSelectionMode ? (
                            <SelectionToggleButton
                              label={
                                selectedApiIds.includes(apiConfig.api_id)
                                  ? t('apis.actions.deselect')
                                  : t('apis.actions.select')
                              }
                              onClick={() => {
                                toggleApiSelection(apiConfig.api_id)
                              }}
                              selected={selectedApiIds.includes(apiConfig.api_id)}
                            />
                          ) : (
                            <>
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
                                  setDeleteApiTargetIds([apiConfig.api_id])
                                }}
                                size="sm"
                                variant="danger"
                              />
                            </>
                          )}
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
                    <div className="flex flex-wrap items-center justify-end gap-2">
                      {groupSelectionMode ? (
                        <>
                          <Badge className="normal-case px-3.5 py-2" variant="subtle">
                            {t('apis.selection.count', { count: selectedGroupIds.length })}
                          </Badge>
                          <IconButton
                            disabled={apiGroups.length === 0}
                            icon={<FontAwesomeIcon icon={faCheckDouble} />}
                            label={t('apis.actions.selectAll')}
                            onClick={() => {
                              setSelectedGroupIds(apiGroups.map((apiGroup) => apiGroup.api_group_id))
                            }}
                            variant="secondary"
                          />
                          <IconButton
                            disabled={selectedGroupIds.length === 0}
                            icon={<FontAwesomeIcon icon={faTrashCan} />}
                            label={t('apis.actions.deleteSelected')}
                            onClick={() => {
                              setDeleteGroupTargetIds(selectedGroupIds)
                            }}
                            variant="danger"
                          />
                          <IconButton
                            icon={<FontAwesomeIcon icon={faXmark} />}
                            label={t('apis.actions.cancelSelection')}
                            onClick={() => {
                              setGroupSelectionMode(false)
                              setSelectedGroupIds([])
                            }}
                            variant="secondary"
                          />
                        </>
                      ) : (
                        <>
                          <IconButton
                            icon={<FontAwesomeIcon icon={faSquareCheck} />}
                            label={t('apis.actions.selectMode')}
                            onClick={enterGroupSelectionMode}
                            variant="secondary"
                          />
                          <IconButton
                            disabled={apis.length === 0}
                            icon={<FontAwesomeIcon icon={faPlus} />}
                            label={t('apis.actions.create')}
                            onClick={() => {
                              setIsCreateApiGroupDialogOpen(true)
                            }}
                          />
                        </>
                      )}
                    </div>
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
                          <Badge variant={isApiGroupResolved(apiGroup, apis) ? 'info' : 'subtle'}>
                            {isApiGroupResolved(apiGroup, apis)
                              ? t('apis.groupList.resolved')
                              : t('apis.groupList.missing')}
                          </Badge>
                        </div>

                        <div className="flex justify-start gap-2 lg:justify-end">
                          {groupSelectionMode ? (
                            <SelectionToggleButton
                              label={
                                selectedGroupIds.includes(apiGroup.api_group_id)
                                  ? t('apis.actions.deselect')
                                  : t('apis.actions.select')
                              }
                              onClick={() => {
                                toggleGroupSelection(apiGroup.api_group_id)
                              }}
                              selected={selectedGroupIds.includes(apiGroup.api_group_id)}
                            />
                          ) : (
                            <>
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
                                  setDeleteGroupTargetIds([apiGroup.api_group_id])
                                }}
                                size="sm"
                                variant="danger"
                              />
                            </>
                          )}
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
