import { faCheckDouble } from '@fortawesome/free-solid-svg-icons/faCheckDouble'
import { faPen } from '@fortawesome/free-solid-svg-icons/faPen'
import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { faSquareCheck } from '@fortawesome/free-solid-svg-icons/faSquareCheck'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
import { faWandMagicSparkles } from '@fortawesome/free-solid-svg-icons/faWandMagicSparkles'
import { faXmark } from '@fortawesome/free-solid-svg-icons/faXmark'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useCallback, useEffect, useLayoutEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useWorkspaceLayoutContext } from '../../components/layout/workspace-context'
import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '../../components/ui/card'
import { IconButton } from '../../components/ui/icon-button'
import { SectionHeader } from '../../components/ui/section-header'
import { SelectionToggleButton } from '../../components/ui/selection-toggle-button'
import { useToastNotice } from '../../components/ui/toast-context'
import { runBatchDelete } from '../../lib/batch-delete'
import { deleteStoryResource, listStoryResources } from './api'
import { CreateStoryResourceDialog } from './create-story-resource-dialog'
import { DeleteStoryResourceDialog } from './delete-story-resource-dialog'
import { GenerateStoryPlanDialog } from './generate-story-plan-dialog'
import { StoryResourceFormDialog } from './story-resource-form-dialog'
import type { StoryResource } from './types'
import { useStoryResourceReferences } from './use-story-resource-references'

type NoticeTone = 'error' | 'success' | 'warning'

type Notice = {
  message: string
  tone: NoticeTone
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function StoryResourcesListSkeleton() {
  return (
    <div className="divide-y divide-[var(--color-border-subtle)]">
      {Array.from({ length: 5 }).map((_, index) => (
        <div
          className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1.15fr)_minmax(0,0.95fr)_auto] lg:items-center"
          key={index}
        >
          <div className="space-y-2.5">
            <div className="h-5 w-36 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-3 w-full animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-3 w-4/5 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
          </div>
          <div className="flex flex-wrap gap-2">
            <div className="h-8 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-8 w-20 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
          </div>
          <div className="flex justify-start gap-2 lg:justify-end">
            <div className="h-9 w-9 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-9 w-9 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-9 w-9 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
          </div>
        </div>
      ))}
    </div>
  )
}

function summarizeStoryInput(resource: StoryResource) {
  return (resource.planned_story?.trim() || resource.story_concept).replace(/\s+/g, ' ').trim()
}

function countRefinedInputs(resources: ReadonlyArray<StoryResource>) {
  return resources.filter((resource) => resource.planned_story?.trim().length).length
}

export function StoryResourcesPage() {
  const { t } = useTranslation()
  const { setRailContent } = useWorkspaceLayoutContext()
  const [resources, setResources] = useState<StoryResource[]>([])
  const [notice, setNotice] = useState<Notice | null>(null)
  const [isListLoading, setIsListLoading] = useState(true)
  const [isDeleting, setIsDeleting] = useState(false)
  const [generatingResourceId, setGeneratingResourceId] = useState<string | null>(null)
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [editingResourceId, setEditingResourceId] = useState<string | null>(null)
  const [generateTarget, setGenerateTarget] = useState<StoryResource | null>(null)
  const [selectionMode, setSelectionMode] = useState(false)
  const [selectedResourceIds, setSelectedResourceIds] = useState<string[]>([])
  const [deleteTargetIds, setDeleteTargetIds] = useState<string[]>([])
  useToastNotice(notice)
  const handleReferencesLoadError = useCallback((message: string) => {
    setNotice({ message, tone: 'error' })
  }, [])

  const {
    availableApiGroups,
    availableCharacters,
    availableLorebooks,
    availablePresets,
    availableSchemas,
    referencesLoading,
  } = useStoryResourceReferences({
    onLoadError: handleReferencesLoadError,
  })

  const refinedCount = useMemo(() => countRefinedInputs(resources), [resources])
  const deleteTargets = useMemo(
    () =>
      deleteTargetIds
        .map((resourceId) => resources.find((resource) => resource.resource_id === resourceId))
        .filter((resource): resource is StoryResource => resource !== undefined),
    [deleteTargetIds, resources],
  )

  const refreshResources = useCallback(
    async (signal?: AbortSignal) => {
      setIsListLoading(true)

      try {
        const nextResources = await listStoryResources(signal)

        if (!signal?.aborted) {
          setResources(nextResources)
        }
      } catch (error) {
        if (!signal?.aborted) {
          setNotice({
            message: getErrorMessage(error, t('storyResources.feedback.loadFailed')),
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

  useEffect(() => {
    const controller = new AbortController()

    void refreshResources(controller.signal)

    return () => {
      controller.abort()
    }
  }, [refreshResources])

  useEffect(() => {
    const availableResourceIds = new Set(resources.map((resource) => resource.resource_id))

    setSelectedResourceIds((currentSelection) =>
      currentSelection.filter((resourceId) => availableResourceIds.has(resourceId)),
    )
    setDeleteTargetIds((currentSelection) =>
      currentSelection.filter((resourceId) => availableResourceIds.has(resourceId)),
    )

    if (generateTarget !== null && !availableResourceIds.has(generateTarget.resource_id)) {
      setGenerateTarget(null)
    }

    if (editingResourceId !== null && !availableResourceIds.has(editingResourceId)) {
      setEditingResourceId(null)
    }
  }, [editingResourceId, generateTarget, resources])

  useLayoutEffect(() => {
    setRailContent({
      description: t('storyResources.rail.description'),
      stats: [
        { label: t('storyResources.metrics.total'), value: resources.length },
        { label: t('storyResources.metrics.planned'), value: refinedCount },
      ],
      title: t('storyResources.title'),
    })

    return () => {
      setRailContent(null)
    }
  }, [refinedCount, resources.length, setRailContent, t])

  async function handleDeleteResource() {
    if (deleteTargets.length === 0) {
      return
    }

    setIsDeleting(true)

    try {
      const result = await runBatchDelete(deleteTargets, async (target) => {
        await deleteStoryResource(target.resource_id)
      })

      setDeleteTargetIds([])

      if (result.deleted.length > 0) {
        const deletedIds = new Set(result.deleted.map((target) => target.resource_id))
        setSelectedResourceIds((currentSelection) =>
          currentSelection.filter((resourceId) => !deletedIds.has(resourceId)),
        )
        setDeleteTargetIds([])

        if (generateTarget !== null && deletedIds.has(generateTarget.resource_id)) {
          setGenerateTarget(null)
        }
      }

      if (result.failed.length === 0) {
        setNotice({
          message:
            result.deleted.length > 1
              ? t('storyResources.feedback.deletedMany', { count: result.deleted.length })
              : t('storyResources.feedback.deleted', { id: result.deleted[0]?.resource_id ?? '' }),
          tone: 'success',
        })
        if (selectionMode) {
          setSelectionMode(false)
          setSelectedResourceIds([])
        }
      } else if (result.deleted.length > 0) {
        setNotice({
          message: t('storyResources.feedback.deletedPartial', {
            failed: result.failed.length,
            success: result.deleted.length,
          }),
          tone: 'warning',
        })
      } else {
        setNotice({
          message:
            result.conflictCount > 0
              ? deleteTargets.length > 1
                ? t('storyResources.deleteDialog.conflictMany')
                : t('storyResources.deleteDialog.conflict')
              : t('storyResources.feedback.deleteFailed'),
          tone: result.conflictCount > 0 ? 'warning' : 'error',
        })
      }

      await refreshResources()
    } finally {
      setIsDeleting(false)
    }
  }

  function toggleResourceSelection(resourceId: string) {
    setSelectedResourceIds((currentSelection) =>
      currentSelection.includes(resourceId)
        ? currentSelection.filter((currentResourceId) => currentResourceId !== resourceId)
        : [...currentSelection, resourceId],
    )
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-6">
      <CreateStoryResourceDialog
        availableCharacters={availableCharacters}
        availableApiGroups={availableApiGroups}
        availableLorebooks={availableLorebooks}
        availablePresets={availablePresets}
        availableSchemas={availableSchemas}
        onCompleted={async ({ message, tone }) => {
          setNotice({ message, tone })
          await refreshResources()
        }}
        onOpenChange={setIsCreateDialogOpen}
        open={isCreateDialogOpen}
        referencesLoading={referencesLoading}
      />

      <GenerateStoryPlanDialog
        apiGroups={availableApiGroups}
        onCompleted={async ({ message, tone }) => {
          setNotice({ message, tone })
          await refreshResources()
        }}
        onOpenChange={(open) => {
          if (!open) {
            setGenerateTarget(null)
            setGeneratingResourceId(null)
          }
        }}
        open={generateTarget !== null}
        presets={availablePresets}
        resource={generateTarget}
      />

      <StoryResourceFormDialog
        availableApiGroups={availableApiGroups}
        availableCharacters={availableCharacters}
        availableLorebooks={availableLorebooks}
        availablePresets={availablePresets}
        availableSchemas={availableSchemas}
        onCompleted={async ({ message, tone }) => {
          setNotice({ message, tone })
          await refreshResources()
        }}
        onOpenChange={(open) => {
          if (!open) {
            setEditingResourceId(null)
          }
        }}
        open={editingResourceId !== null}
        referencesLoading={referencesLoading}
        resourceId={editingResourceId}
      />

      <DeleteStoryResourceDialog
        deleting={isDeleting}
        onConfirm={() => {
          void handleDeleteResource()
        }}
        onOpenChange={() => {
          setDeleteTargetIds([])
        }}
        targets={deleteTargets}
      />

      <WorkspacePanelShell className="flex min-h-0 flex-1">
        <Card className="flex min-h-0 flex-1 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
          <CardHeader className="gap-4 border-b border-[var(--color-border-subtle)] px-6 py-5 md:min-h-[5.75rem] md:px-7 md:py-5">
            <SectionHeader
              actions={
                <div className="flex min-h-10 flex-wrap items-center justify-end gap-2">
                  {selectionMode ? (
                    <>
                      <Badge className="normal-case px-3.5 py-2" variant="subtle">
                        {t('storyResources.selection.count', { count: selectedResourceIds.length })}
                      </Badge>
                      <IconButton
                        disabled={resources.length === 0}
                        icon={<FontAwesomeIcon icon={faCheckDouble} />}
                        label={t('storyResources.actions.selectAll')}
                        onClick={() => {
                          setSelectedResourceIds(
                            resources
                              .filter((resource) => resource.resource_id !== generatingResourceId)
                              .map((resource) => resource.resource_id),
                          )
                        }}
                        size="md"
                        variant="secondary"
                      />
                      <IconButton
                        disabled={selectedResourceIds.length === 0}
                        icon={<FontAwesomeIcon icon={faTrashCan} />}
                        label={t('storyResources.actions.deleteSelected')}
                        onClick={() => {
                          setDeleteTargetIds(selectedResourceIds)
                        }}
                        size="md"
                        variant="danger"
                      />
                      <IconButton
                        icon={<FontAwesomeIcon icon={faXmark} />}
                        label={t('storyResources.actions.cancelSelection')}
                        onClick={() => {
                          setSelectionMode(false)
                          setSelectedResourceIds([])
                        }}
                        size="md"
                        variant="secondary"
                      />
                    </>
                  ) : (
                    <>
                      <IconButton
                        icon={<FontAwesomeIcon icon={faSquareCheck} />}
                        label={t('storyResources.actions.selectMode')}
                        onClick={() => {
                          setSelectionMode(true)
                          setSelectedResourceIds([])
                        }}
                        size="md"
                        variant="secondary"
                      />
                      <IconButton
                        icon={<FontAwesomeIcon icon={faPlus} />}
                        label={t('storyResources.actions.create')}
                        onClick={() => {
                          setIsCreateDialogOpen(true)
                        }}
                        size="md"
                      />
                    </>
                  )}
                </div>
              }
              title={t('storyResources.title')}
            />
          </CardHeader>

          <CardContent className="min-h-0 flex-1 overflow-y-auto pt-6">
            <div className="space-y-6 pr-1">
              {isListLoading ? (
                <StoryResourcesListSkeleton />
              ) : resources.length === 0 ? (
                <div className="py-12 text-center">
                  <h3 className="font-display text-3xl text-[var(--color-text-primary)]">
                    {t('storyResources.empty.title')}
                  </h3>

                  <p className="mt-3 text-sm leading-7 text-[var(--color-text-secondary)]">
                    {t('storyResources.empty.description')}
                  </p>

                  <div className="mt-7 flex justify-center">
                    <Button
                      onClick={() => {
                        setIsCreateDialogOpen(true)
                      }}
                      size="md"
                    >
                      {t('storyResources.actions.create')}
                    </Button>
                  </div>
                </div>
              ) : (
                <div className="space-y-5">
                  <CardTitle className="text-[1.85rem]">{t('storyResources.list.title')}</CardTitle>

                  <div className="divide-y divide-[var(--color-border-subtle)]">
                    {resources.map((resource) => {
                      const isGenerating = generatingResourceId === resource.resource_id

                      return (
                        <div
                          className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1.15fr)_minmax(0,0.95fr)_auto] lg:items-center"
                          key={resource.resource_id}
                        >
                          <div className="min-w-0 space-y-2">
                            <h3 className="truncate font-display text-[1.2rem] leading-tight text-[var(--color-text-primary)]">
                              {resource.resource_id}
                            </h3>
                            <p className="line-clamp-2 text-sm leading-7 text-[var(--color-text-secondary)]">
                              {summarizeStoryInput(resource)}
                            </p>
                          </div>

                          <div className="flex flex-wrap gap-2">
                            <Badge className="normal-case px-3 py-1.5" variant="subtle">
                              {t('storyResources.list.charactersCount', {
                                count: resource.character_ids.length,
                              })}
                            </Badge>
                            <Badge className="normal-case px-3 py-1.5" variant="subtle">
                              {t('storyResources.list.lorebooksCount', {
                                count: resource.lorebook_ids.length,
                              })}
                            </Badge>
                            <Badge className="normal-case px-3 py-1.5" variant="subtle">
                              {resource.planned_story?.trim().length
                                ? t('storyResources.list.planned')
                                : t('storyResources.list.notPlanned')}
                            </Badge>
                          </div>

                          <div className="flex justify-start gap-2 lg:justify-end">
                            {selectionMode ? (
                              <SelectionToggleButton
                                disabled={isGenerating}
                                label={
                                  selectedResourceIds.includes(resource.resource_id)
                                    ? t('storyResources.actions.deselect')
                                    : t('storyResources.actions.select')
                                }
                                onClick={() => {
                                  toggleResourceSelection(resource.resource_id)
                                }}
                                selected={selectedResourceIds.includes(resource.resource_id)}
                              />
                            ) : (
                              <>
                                <IconButton
                                  disabled={isGenerating}
                                  icon={<FontAwesomeIcon icon={faWandMagicSparkles} />}
                                  label={t('storyResources.actions.generate')}
                                  onClick={() => {
                                    setGeneratingResourceId(resource.resource_id)
                                    setGenerateTarget(resource)
                                  }}
                                  size="sm"
                                  variant="secondary"
                                />
                                <IconButton
                                  disabled={isGenerating}
                                  icon={<FontAwesomeIcon icon={faPen} />}
                                  label={t('storyResources.actions.edit')}
                                  onClick={() => {
                                    setEditingResourceId(resource.resource_id)
                                  }}
                                  size="sm"
                                  variant="secondary"
                                />
                                <IconButton
                                  disabled={isGenerating}
                                  icon={<FontAwesomeIcon icon={faTrashCan} />}
                                  label={t('storyResources.actions.delete')}
                                  onClick={() => {
                                    setDeleteTargetIds([resource.resource_id])
                                  }}
                                  size="sm"
                                  variant="danger"
                                />
                              </>
                            )}
                          </div>
                        </div>
                      )
                    })}
                  </div>
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      </WorkspacePanelShell>
    </div>
  )
}
