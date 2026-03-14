import { useCallback, useEffect, useLayoutEffect, useMemo, useState } from 'react'
import { faPen } from '@fortawesome/free-solid-svg-icons/faPen'
import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
import { faWandMagicSparkles } from '@fortawesome/free-solid-svg-icons/faWandMagicSparkles'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useTranslation } from 'react-i18next'

import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { useWorkspaceLayoutContext } from '../../components/layout/workspace-context'
import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '../../components/ui/card'
import { IconButton } from '../../components/ui/icon-button'
import { SectionHeader } from '../../components/ui/section-header'
import { cn } from '../../lib/cn'
import { isRpcConflict } from '../../lib/rpc'
import { listCharacters } from '../characters/api'
import type { CharacterSummary } from '../characters/types'
import { listSchemas } from '../schemas/api'
import type { SchemaResource } from '../schemas/types'
import {
  deleteStoryResource,
  generateAndSaveStoryPlan,
  listStoryResources,
} from './api'
import { DeleteStoryResourceDialog } from './delete-story-resource-dialog'
import { StoryResourceFormDialog } from './story-resource-form-dialog'
import type { StoryResource } from './types'

type NoticeTone = 'error' | 'success' | 'warning'

type Notice = {
  message: string
  tone: NoticeTone
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
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

function summarizeStoryConcept(storyConcept: string) {
  return storyConcept.replace(/\s+/g, ' ').trim()
}

function countPlannedResources(resources: ReadonlyArray<StoryResource>) {
  return resources.filter((resource) => resource.planned_story?.trim().length).length
}

export function StoryResourcesPage() {
  const { t } = useTranslation()
  const { setRailContent } = useWorkspaceLayoutContext()
  const [resources, setResources] = useState<StoryResource[]>([])
  const [availableCharacters, setAvailableCharacters] = useState<CharacterSummary[]>([])
  const [availableSchemas, setAvailableSchemas] = useState<SchemaResource[]>([])
  const [notice, setNotice] = useState<Notice | null>(null)
  const [isListLoading, setIsListLoading] = useState(true)
  const [referencesLoading, setReferencesLoading] = useState(true)
  const [isDeleting, setIsDeleting] = useState(false)
  const [generatingResourceId, setGeneratingResourceId] = useState<string | null>(null)
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [editResourceId, setEditResourceId] = useState<string | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<StoryResource | null>(null)

  const plannedCount = useMemo(() => countPlannedResources(resources), [resources])

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

  const refreshReferences = useCallback(
    async (signal?: AbortSignal) => {
      setReferencesLoading(true)

      try {
        const [characters, schemas] = await Promise.all([
          listCharacters(signal),
          listSchemas(signal),
        ])

        if (!signal?.aborted) {
          setAvailableCharacters(characters)
          setAvailableSchemas(schemas)
        }
      } catch (error) {
        if (!signal?.aborted) {
          setNotice({
            message: getErrorMessage(error, t('storyResources.feedback.loadReferencesFailed')),
            tone: 'error',
          })
        }
      } finally {
        if (!signal?.aborted) {
          setReferencesLoading(false)
        }
      }
    },
    [t],
  )

  useEffect(() => {
    const controller = new AbortController()

    void Promise.all([
      refreshResources(controller.signal),
      refreshReferences(controller.signal),
    ])

    return () => {
      controller.abort()
    }
  }, [refreshReferences, refreshResources])

  useLayoutEffect(() => {
    setRailContent({
      description: t('storyResources.rail.description'),
      stats: [
        { label: t('storyResources.metrics.total'), value: resources.length },
        { label: t('storyResources.metrics.planned'), value: plannedCount },
      ],
      title: t('storyResources.title'),
    })

    return () => {
      setRailContent(null)
    }
  }, [plannedCount, resources.length, setRailContent, t])

  async function handleDeleteResource() {
    if (!deleteTarget) {
      return
    }

    setIsDeleting(true)

    try {
      await deleteStoryResource(deleteTarget.resource_id)
      setNotice({
        message: t('storyResources.feedback.deleted', { id: deleteTarget.resource_id }),
        tone: 'success',
      })
      setDeleteTarget(null)
      await refreshResources()
    } catch (error) {
      setNotice({
        message: isRpcConflict(error)
          ? t('storyResources.deleteDialog.conflict')
          : getErrorMessage(error, t('storyResources.feedback.deleteFailed')),
        tone: isRpcConflict(error) ? 'warning' : 'error',
      })
    } finally {
      setIsDeleting(false)
    }
  }

  async function handleGenerateDraft(resourceId: string) {
    setGeneratingResourceId(resourceId)

    try {
      await generateAndSaveStoryPlan(resourceId)
      setNotice({
        message: t('storyResources.feedback.generated', { id: resourceId }),
        tone: 'success',
      })
      await refreshResources()
    } catch (error) {
      setNotice({
        message: getErrorMessage(error, t('storyResources.feedback.generateFailed')),
        tone: 'error',
      })
    } finally {
      setGeneratingResourceId(null)
    }
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-6">
      <StoryResourceFormDialog
        availableCharacters={availableCharacters}
        availableSchemas={availableSchemas}
        mode="create"
        onCompleted={async ({ message, tone }) => {
          setNotice({ message, tone })
          await refreshResources()
        }}
        onOpenChange={setIsCreateDialogOpen}
        open={isCreateDialogOpen}
        referencesLoading={referencesLoading}
      />

      <StoryResourceFormDialog
        availableCharacters={availableCharacters}
        availableSchemas={availableSchemas}
        mode="edit"
        onCompleted={async ({ message, tone }) => {
          setNotice({ message, tone })
          await refreshResources()
        }}
        onOpenChange={(open) => {
          if (!open) {
            setEditResourceId(null)
          }
        }}
        open={editResourceId !== null}
        referencesLoading={referencesLoading}
        resourceId={editResourceId}
      />

      <DeleteStoryResourceDialog
        deleting={isDeleting}
        onConfirm={() => {
          void handleDeleteResource()
        }}
        onOpenChange={() => {
          setDeleteTarget(null)
        }}
        resource={deleteTarget}
      />

      <WorkspacePanelShell className="flex min-h-0 flex-1">
        <Card className="flex min-h-0 flex-1 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
        <CardHeader className="gap-4 border-b border-[var(--color-border-subtle)] px-6 py-5 md:min-h-[5.75rem] md:px-7 md:py-5">
          <SectionHeader
            actions={
              <div className="flex min-h-10 items-center justify-end">
                <IconButton
                  icon={<FontAwesomeIcon icon={faPlus} />}
                  label={t('storyResources.actions.create')}
                  onClick={() => {
                    setIsCreateDialogOpen(true)
                  }}
                  size="md"
                />
              </div>
            }
            title={t('storyResources.title')}
          />
        </CardHeader>

        <CardContent className="min-h-0 flex-1 overflow-y-auto pt-6">
          <div className="space-y-6 pr-1">
            {notice ? <StatusNotice notice={notice} /> : null}

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
                <CardTitle className="text-[1.85rem]">
                  {t('storyResources.list.title')}
                </CardTitle>

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
                            {summarizeStoryConcept(resource.story_concept)}
                          </p>
                        </div>

                        <div className="flex flex-wrap gap-2">
                          <Badge className="normal-case px-3 py-1.5" variant="subtle">
                            {t('storyResources.list.charactersCount', {
                              count: resource.character_ids.length,
                            })}
                          </Badge>
                          <Badge className="normal-case px-3 py-1.5" variant="subtle">
                            {resource.planned_story?.trim().length
                              ? t('storyResources.list.planned')
                              : t('storyResources.list.notPlanned')}
                          </Badge>
                        </div>

                        <div className="flex justify-start gap-2 lg:justify-end">
                          <IconButton
                            disabled={isGenerating}
                            icon={<FontAwesomeIcon icon={faWandMagicSparkles} />}
                            label={t('storyResources.actions.generate')}
                            onClick={() => {
                              void handleGenerateDraft(resource.resource_id)
                            }}
                            size="sm"
                            variant="secondary"
                          />
                          <IconButton
                            disabled={isGenerating}
                            icon={<FontAwesomeIcon icon={faPen} />}
                            label={t('storyResources.actions.edit')}
                            onClick={() => {
                              setEditResourceId(resource.resource_id)
                            }}
                            size="sm"
                            variant="ghost"
                          />
                          <IconButton
                            className="text-[var(--color-state-error)] hover:bg-[var(--color-state-error-soft)] hover:text-[var(--color-text-primary)]"
                            disabled={isGenerating}
                            icon={<FontAwesomeIcon icon={faTrashCan} />}
                            label={t('storyResources.actions.delete')}
                            onClick={() => {
                              setDeleteTarget(resource)
                            }}
                            size="sm"
                            variant="ghost"
                          />
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
