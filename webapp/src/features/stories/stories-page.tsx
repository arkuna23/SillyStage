import { useCallback, useEffect, useLayoutEffect, useMemo, useState } from 'react'
import { faBookOpen } from '@fortawesome/free-solid-svg-icons/faBookOpen'
import { faDiagramProject } from '@fortawesome/free-solid-svg-icons/faDiagramProject'
import { faEye } from '@fortawesome/free-solid-svg-icons/faEye'
import { faPen } from '@fortawesome/free-solid-svg-icons/faPen'
import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { faRotateRight } from '@fortawesome/free-solid-svg-icons/faRotateRight'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useTranslation } from 'react-i18next'

import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { useWorkspaceLayoutContext } from '../../components/layout/workspace-context'
import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '../../components/ui/card'
import { IconButton } from '../../components/ui/icon-button'
import { SegmentedSelector } from '../../components/ui/segmented-selector'
import { SectionHeader } from '../../components/ui/section-header'
import { useToastNotice } from '../../components/ui/toast-context'
import { cn } from '../../lib/cn'
import { isRpcConflict } from '../../lib/rpc'
import { listApiGroups, listPresets } from '../apis/api'
import type { ApiGroup, Preset } from '../apis/types'
import { listCharacters } from '../characters/api'
import type { CharacterSummary } from '../characters/types'
import { listStoryResources } from '../story-resources/api'
import type { StoryResource } from '../story-resources/types'
import {
  continueStoryDraft,
  deleteStory,
  deleteStoryDraft,
  finalizeStoryDraft,
  listStories,
  listStoryDrafts,
} from './api'
import { getDraftSectionProgress } from './draft-progress'
import { DeleteStoryDraftDialog } from './delete-story-draft-dialog'
import { DeleteStoryDialog } from './delete-story-dialog'
import { GenerateStoryDialog } from './generate-story-dialog'
import { StoryDraftDetailsDialog } from './story-draft-details-dialog'
import { StoryDetailsDialog } from './story-details-dialog'
import { StoryFormDialog } from './story-form-dialog'
import type { StoryDraftDetail, StoryDraftSummary, StorySummary } from './types'

type NoticeTone = 'error' | 'success' | 'warning'
type StoryViewMode = 'drafts' | 'stories'

type Notice = {
  message: string
  tone: NoticeTone
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function StoriesListSkeleton() {
  return (
    <div className="divide-y divide-[var(--color-border-subtle)]">
      {Array.from({ length: 5 }).map((_, index) => (
        <div
          className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1.15fr)_minmax(0,0.95fr)_auto] lg:items-center"
          key={index}
        >
          <div className="space-y-2.5">
            <div className="h-5 w-32 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-3 w-28 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-3 w-full animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
          </div>
          <div className="flex flex-wrap gap-2">
            <div className="h-8 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-8 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
          </div>
          <div className="flex justify-start gap-2 lg:justify-end">
            <div className="h-9 w-16 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-9 w-16 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-9 w-16 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
          </div>
        </div>
      ))}
    </div>
  )
}

function StoryDraftsListSkeleton() {
  return (
    <div className="divide-y divide-[var(--color-border-subtle)]">
      {Array.from({ length: 5 }).map((_, index) => (
        <div
          className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto] lg:items-center"
          key={index}
        >
          <div className="space-y-2.5">
            <div className="h-5 w-32 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-3 w-28 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-3 w-40 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
          </div>
          <div className="flex flex-wrap gap-2">
            <div className="h-8 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-8 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-8 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
          </div>
          <div className="flex justify-start gap-2 lg:justify-end">
            <div className="h-9 w-16 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-9 w-16 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-9 w-16 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
          </div>
        </div>
      ))}
    </div>
  )
}

function summarizeIntroduction(introduction: string) {
  return introduction.replace(/\s+/g, ' ').trim()
}

export function StoriesPage() {
  const { t } = useTranslation()
  const { setRailContent } = useWorkspaceLayoutContext()
  const [stories, setStories] = useState<StorySummary[]>([])
  const [drafts, setDrafts] = useState<StoryDraftSummary[]>([])
  const [resources, setResources] = useState<StoryResource[]>([])
  const [characters, setCharacters] = useState<CharacterSummary[]>([])
  const [apiGroups, setApiGroups] = useState<ApiGroup[]>([])
  const [presets, setPresets] = useState<Preset[]>([])
  const [notice, setNotice] = useState<Notice | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [isDraftsLoading, setIsDraftsLoading] = useState(true)
  const [isDeleting, setIsDeleting] = useState(false)
  const [isDeletingDraft, setIsDeletingDraft] = useState(false)
  const [activeDraftActionId, setActiveDraftActionId] = useState<string | null>(null)
  const [viewMode, setViewMode] = useState<StoryViewMode>('stories')
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [detailsStoryId, setDetailsStoryId] = useState<string | null>(null)
  const [detailsDraftId, setDetailsDraftId] = useState<string | null>(null)
  const [editStoryId, setEditStoryId] = useState<string | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<StorySummary | null>(null)
  const [deleteDraftTarget, setDeleteDraftTarget] = useState<StoryDraftSummary | null>(null)
  useToastNotice(notice)

  const distinctResourceCount = useMemo(
    () => new Set(stories.map((story) => story.resource_id)).size,
    [stories],
  )
  const readyDraftCount = useMemo(
    () => drafts.filter((draft) => draft.status === 'ready_to_finalize').length,
    [drafts],
  )

  const refreshStories = useCallback(
    async (signal?: AbortSignal) => {
      setIsLoading(true)

      try {
        const nextStories = await listStories(signal)

        if (!signal?.aborted) {
          setStories(nextStories)
        }
      } catch (error) {
        if (!signal?.aborted) {
          setNotice({
            message: getErrorMessage(error, t('stories.feedback.loadListFailed')),
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

  const refreshDrafts = useCallback(
    async (signal?: AbortSignal) => {
      setIsDraftsLoading(true)

      try {
        const nextDrafts = await listStoryDrafts(signal)

        if (!signal?.aborted) {
          setDrafts(nextDrafts)
        }
      } catch (error) {
        if (!signal?.aborted) {
          setNotice({
            message: getErrorMessage(error, t('stories.drafts.feedback.loadListFailed')),
            tone: 'error',
          })
        }
      } finally {
        if (!signal?.aborted) {
          setIsDraftsLoading(false)
        }
      }
    },
    [t],
  )

  const refreshResources = useCallback(
    async (signal?: AbortSignal) => {
      try {
        const [nextCharacters, nextResources, nextApiGroups, nextPresets] = await Promise.all([
          listCharacters(signal),
          listStoryResources(signal),
          listApiGroups(signal),
          listPresets(signal),
        ])

        if (!signal?.aborted) {
          setCharacters(nextCharacters)
          setResources(nextResources)
          setApiGroups(nextApiGroups)
          setPresets(nextPresets)
        }
      } catch (error) {
        if (!signal?.aborted) {
          setNotice({
            message: getErrorMessage(error, t('stories.feedback.loadReferencesFailed')),
            tone: 'error',
          })
        }
      }
    },
    [t],
  )

  useEffect(() => {
    const controller = new AbortController()

    void Promise.all([
      refreshStories(controller.signal),
      refreshDrafts(controller.signal),
      refreshResources(controller.signal),
    ])

    return () => {
      controller.abort()
    }
  }, [refreshDrafts, refreshResources, refreshStories])

  useLayoutEffect(() => {
    setRailContent({
      description:
        viewMode === 'stories'
          ? t('stories.rail.description')
          : t('stories.drafts.rail.description'),
      stats: [
        viewMode === 'stories'
          ? { label: t('stories.metrics.total'), value: stories.length }
          : { label: t('stories.drafts.metrics.total'), value: drafts.length },
        viewMode === 'stories'
          ? { label: t('stories.metrics.resources'), value: distinctResourceCount }
          : { label: t('stories.drafts.metrics.ready'), value: readyDraftCount },
      ],
      title: t('stories.title'),
    })

    return () => {
      setRailContent(null)
    }
  }, [distinctResourceCount, drafts.length, readyDraftCount, setRailContent, stories.length, t, viewMode])

  async function handleDeleteStory() {
    if (!deleteTarget) {
      return
    }

    const target = deleteTarget
    setIsDeleting(true)

    try {
      await deleteStory(target.story_id)
      setNotice({
        message: t('stories.feedback.deleted', { name: target.display_name }),
        tone: 'success',
      })
      setDeleteTarget(null)
      await refreshStories()
    } catch (error) {
      setDeleteTarget(null)
      setNotice({
        message: isRpcConflict(error)
          ? t('stories.deleteDialog.conflict')
          : getErrorMessage(error, t('stories.feedback.deleteFailed')),
        tone: isRpcConflict(error) ? 'warning' : 'error',
      })
    } finally {
      setIsDeleting(false)
    }
  }

  async function handleContinueDraft(draft: StoryDraftSummary) {
    setActiveDraftActionId(draft.draft_id)

    try {
      let currentDraft: StoryDraftSummary | StoryDraftDetail = draft

      while (currentDraft.status === 'building') {
        currentDraft = await continueStoryDraft({ draft_id: currentDraft.draft_id })
      }

      if (currentDraft.status === 'ready_to_finalize') {
        const result = await finalizeStoryDraft({ draft_id: currentDraft.draft_id })

        setNotice({
          message: t('stories.feedback.created', { name: result.display_name }),
          tone: 'success',
        })
        await Promise.all([refreshStories(), refreshDrafts()])
        setViewMode('stories')
        return
      }

      const progress = getDraftSectionProgress(currentDraft)
      setNotice({
        message: progress
          ? t('stories.drafts.feedback.continued', progress)
          : t('stories.drafts.feedback.continuedUnknown'),
        tone: 'success',
      })
      await refreshDrafts()
    } catch (error) {
      await Promise.all([refreshStories(), refreshDrafts()])
      setNotice({
        message: getErrorMessage(error, t('stories.drafts.feedback.continueFailed')),
        tone: 'error',
      })
    } finally {
      setActiveDraftActionId(null)
    }
  }

  async function handleFinalizeDraft(draft: StoryDraftSummary) {
    setActiveDraftActionId(draft.draft_id)

    try {
      const result = await finalizeStoryDraft({ draft_id: draft.draft_id })
      setNotice({
        message: t('stories.feedback.created', { name: result.display_name }),
        tone: 'success',
      })
      await Promise.all([refreshStories(), refreshDrafts()])
      setViewMode('stories')
    } catch (error) {
      setNotice({
        message: getErrorMessage(error, t('stories.drafts.feedback.finalizeFailed')),
        tone: 'error',
      })
    } finally {
      setActiveDraftActionId(null)
    }
  }

  async function handleDeleteDraft() {
    if (!deleteDraftTarget) {
      return
    }

    const target = deleteDraftTarget
    setIsDeletingDraft(true)

    try {
      await deleteStoryDraft(target.draft_id)
      setNotice({
        message: t('stories.drafts.feedback.deleted', { name: target.display_name }),
        tone: 'success',
      })
      setDeleteDraftTarget(null)
      await refreshDrafts()
    } catch (error) {
      setDeleteDraftTarget(null)
      setNotice({
        message: getErrorMessage(error, t('stories.drafts.feedback.deleteFailed')),
        tone: 'error',
      })
    } finally {
      setIsDeletingDraft(false)
    }
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-6">
      <GenerateStoryDialog
        availableCharacters={characters}
        apiGroups={apiGroups}
        onCompleted={async ({ message }) => {
          setNotice({ message, tone: 'success' })
          await Promise.all([refreshStories(), refreshDrafts()])
        }}
        onDraftsChanged={refreshDrafts}
        onOpenChange={setIsCreateDialogOpen}
        open={isCreateDialogOpen}
        presets={presets}
        resources={resources}
      />

      <StoryFormDialog
        availableCharacters={characters}
        availableResources={resources}
        onCompleted={async ({ message }) => {
          setNotice({ message, tone: 'success' })
          await refreshStories()
        }}
        onOpenChange={(open) => {
          if (!open) {
            setEditStoryId(null)
          }
        }}
        open={editStoryId !== null}
        storyId={editStoryId}
      />

      <StoryDetailsDialog
        onOpenChange={(open) => {
          if (!open) {
            setDetailsStoryId(null)
          }
        }}
        open={detailsStoryId !== null}
        storyId={detailsStoryId}
      />

      <StoryDraftDetailsDialog
        draftId={detailsDraftId}
        onOpenChange={(open) => {
          if (!open) {
            setDetailsDraftId(null)
          }
        }}
        open={detailsDraftId !== null}
      />

      <DeleteStoryDialog
        deleting={isDeleting}
        onConfirm={() => {
          void handleDeleteStory()
        }}
        onOpenChange={() => {
          setDeleteTarget(null)
        }}
        open={deleteTarget !== null}
        story={deleteTarget}
      />

      <DeleteStoryDraftDialog
        deleting={isDeletingDraft}
        draft={deleteDraftTarget}
        onConfirm={() => {
          void handleDeleteDraft()
        }}
        onOpenChange={() => {
          setDeleteDraftTarget(null)
        }}
        open={deleteDraftTarget !== null}
      />

      <WorkspacePanelShell className="flex min-h-0 flex-1">
        <Card className="flex min-h-0 flex-1 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
          <CardHeader className="gap-4 border-b border-[var(--color-border-subtle)] px-6 py-5 md:min-h-[5.75rem] md:px-7 md:py-5">
            <SectionHeader
              actions={
                <div className="flex min-h-10 items-center justify-end gap-3">
                  <SegmentedSelector
                    ariaLabel={t('stories.view.label')}
                    className="shrink-0"
                    items={[
                      {
                        icon: <FontAwesomeIcon icon={faBookOpen} />,
                        label: t('stories.view.stories'),
                        value: 'stories',
                      },
                      {
                        icon: <FontAwesomeIcon icon={faDiagramProject} />,
                        label: t('stories.view.drafts'),
                        value: 'drafts',
                      },
                    ]}
                    onValueChange={(value) => {
                      setViewMode(value as StoryViewMode)
                    }}
                    value={viewMode}
                  />
                  <IconButton
                    icon={<FontAwesomeIcon icon={faPlus} />}
                    label={t('stories.actions.createDraft')}
                    onClick={() => {
                      setIsCreateDialogOpen(true)
                    }}
                    size="md"
                  />
                </div>
              }
              title={t('stories.title')}
            />
          </CardHeader>

          <CardContent className="min-h-0 flex-1 overflow-y-auto pt-6">
            <div className="space-y-6 pr-1">
              <section className="space-y-5">
                <div className="space-y-2">
                  <CardTitle className="text-[1.85rem]">
                    {viewMode === 'stories' ? t('stories.list.title') : t('stories.drafts.list.title')}
                  </CardTitle>
                </div>

                {viewMode === 'stories' ? (
                  isLoading ? (
                    <StoriesListSkeleton />
                  ) : stories.length === 0 ? (
                  <div className="py-12 text-center">
                    <h3 className="font-display text-3xl text-[var(--color-text-primary)]">
                      {t('stories.empty.title')}
                    </h3>

                    <p className="mt-3 text-sm leading-7 text-[var(--color-text-secondary)]">
                      {t('stories.empty.description')}
                    </p>

                    <div className="mt-7 flex justify-center">
                      <Button
                        onClick={() => {
                          setIsCreateDialogOpen(true)
                        }}
                      >
                        {t('stories.actions.createDraft')}
                      </Button>
                    </div>
                  </div>
                ) : (
                  <div className="divide-y divide-[var(--color-border-subtle)]">
                    {stories.map((story) => (
                      <div
                        className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1.15fr)_minmax(0,0.95fr)_auto] lg:items-center"
                        key={story.story_id}
                      >
                        <div className="min-w-0 space-y-2">
                          <div className="space-y-1">
                            <h3 className="truncate font-display text-[1.2rem] leading-tight text-[var(--color-text-primary)]">
                              {story.display_name}
                            </h3>
                            <p className="truncate font-mono text-[0.76rem] leading-5 text-[var(--color-text-muted)]">
                              {story.story_id}
                            </p>
                          </div>
                          <p className="line-clamp-2 text-sm leading-7 text-[var(--color-text-secondary)]">
                            {summarizeIntroduction(story.introduction)}
                          </p>
                        </div>

                        <div className="flex flex-wrap gap-2">
                          <Badge className="normal-case px-3 py-1.5" variant="subtle">
                            {t('stories.list.resourcePrefix', { id: story.resource_id })}
                          </Badge>
                          <Badge className="normal-case px-3 py-1.5" variant="subtle">
                            {t('stories.list.playerSchemaPrefix', { id: story.player_schema_id })}
                          </Badge>
                          <Badge className="normal-case px-3 py-1.5" variant="subtle">
                            {t('stories.list.worldSchemaPrefix', { id: story.world_schema_id })}
                          </Badge>
                        </div>

                        <div className="flex flex-wrap items-center justify-start gap-2 lg:justify-end">
                          <Button
                            onClick={() => {
                              setDetailsStoryId(story.story_id)
                            }}
                            size="sm"
                            variant="ghost"
                          >
                            <FontAwesomeIcon icon={faEye} />
                            {t('stories.actions.view')}
                          </Button>
                          <Button
                            onClick={() => {
                              setEditStoryId(story.story_id)
                            }}
                            size="sm"
                            variant="secondary"
                          >
                            <FontAwesomeIcon icon={faPen} />
                            {t('stories.actions.edit')}
                          </Button>
                          <Button
                            onClick={() => {
                              setDeleteTarget(story)
                            }}
                            size="sm"
                            variant="danger"
                          >
                            <FontAwesomeIcon icon={faTrashCan} />
                            {t('stories.actions.delete')}
                          </Button>
                        </div>
                      </div>
                    ))}
                  </div>
                  )
                ) : isDraftsLoading ? (
                  <StoryDraftsListSkeleton />
                ) : drafts.length === 0 ? (
                  <div className="py-12 text-center">
                    <h3 className="font-display text-3xl text-[var(--color-text-primary)]">
                      {t('stories.drafts.empty.title')}
                    </h3>

                    <p className="mt-3 text-sm leading-7 text-[var(--color-text-secondary)]">
                      {t('stories.drafts.empty.description')}
                    </p>

                    <div className="mt-7 flex justify-center">
                      <Button
                        onClick={() => {
                          setIsCreateDialogOpen(true)
                        }}
                      >
                        {t('stories.actions.createDraft')}
                      </Button>
                    </div>
                  </div>
                ) : (
                  <div className="divide-y divide-[var(--color-border-subtle)]">
                    {drafts.map((draft) => {
                      const isWorking = activeDraftActionId === draft.draft_id
                      const progress = getDraftSectionProgress(draft)

                      return (
                        <div
                          className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto] lg:items-center"
                          key={draft.draft_id}
                        >
                          <div className="min-w-0 space-y-2">
                            <div className="space-y-1">
                              <h3 className="truncate font-display text-[1.2rem] leading-tight text-[var(--color-text-primary)]">
                                {draft.display_name}
                              </h3>
                              <p className="truncate font-mono text-[0.76rem] leading-5 text-[var(--color-text-muted)]">
                                {draft.draft_id}
                              </p>
                            </div>
                            <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                              {progress
                                ? t('stories.drafts.list.progress', progress)
                                : t('stories.drafts.list.progressUnknown')}
                            </p>
                          </div>

                          <div className="flex flex-wrap gap-2">
                            <Badge className="normal-case px-3 py-1.5" variant="subtle">
                              {t('stories.drafts.list.resourcePrefix', { id: draft.resource_id })}
                            </Badge>
                            <Badge className="normal-case px-3 py-1.5" variant="subtle">
                              {t(`stories.drafts.status.${draft.status}` as const)}
                            </Badge>
                            <Badge className="normal-case px-3 py-1.5" variant="subtle">
                              {t('stories.drafts.list.nodes', { count: draft.partial_node_count })}
                            </Badge>
                          </div>

                          <div className="flex flex-wrap items-center justify-start gap-2 lg:justify-end">
                            <Button
                              onClick={() => {
                                setDetailsDraftId(draft.draft_id)
                              }}
                              size="sm"
                              variant="ghost"
                            >
                              <FontAwesomeIcon icon={faEye} />
                              {t('stories.actions.view')}
                            </Button>
                            {draft.status === 'building' ? (
                              <Button
                                disabled={isWorking}
                                onClick={() => {
                                  void handleContinueDraft(draft)
                                }}
                                size="sm"
                                variant="secondary"
                              >
                                <FontAwesomeIcon className={cn(isWorking ? 'animate-spin' : '')} icon={faRotateRight} />
                                {t('stories.actions.continueDraft')}
                              </Button>
                            ) : null}
                            {draft.status === 'ready_to_finalize' ? (
                              <Button
                                disabled={isWorking}
                                onClick={() => {
                                  void handleFinalizeDraft(draft)
                                }}
                                size="sm"
                              >
                                <FontAwesomeIcon icon={faPlus} />
                                {t('stories.actions.finalizeDraft')}
                              </Button>
                            ) : null}
                            <Button
                              onClick={() => {
                                setDeleteDraftTarget(draft)
                              }}
                              size="sm"
                              variant="danger"
                            >
                              <FontAwesomeIcon icon={faTrashCan} />
                              {t('stories.actions.delete')}
                            </Button>
                          </div>
                        </div>
                      )
                    })}
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
