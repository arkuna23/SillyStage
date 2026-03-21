import { faBookOpen } from '@fortawesome/free-solid-svg-icons/faBookOpen'
import { faCheckDouble } from '@fortawesome/free-solid-svg-icons/faCheckDouble'
import { faDiagramProject } from '@fortawesome/free-solid-svg-icons/faDiagramProject'
import { faEye } from '@fortawesome/free-solid-svg-icons/faEye'
import { faPen } from '@fortawesome/free-solid-svg-icons/faPen'
import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { faRotateRight } from '@fortawesome/free-solid-svg-icons/faRotateRight'
import { faSquareCheck } from '@fortawesome/free-solid-svg-icons/faSquareCheck'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
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
import {
  PopupMenu,
  PopupMenuContent,
  PopupMenuItem,
  PopupMenuTrigger,
} from '../../components/ui/popup-menu'
import { SectionHeader } from '../../components/ui/section-header'
import { SegmentedSelector } from '../../components/ui/segmented-selector'
import { SelectionToggleButton } from '../../components/ui/selection-toggle-button'
import { useToastNotice } from '../../components/ui/toast-context'
import { runBatchDelete } from '../../lib/batch-delete'
import { cn } from '../../lib/cn'
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
import { DeleteStoryDialog } from './delete-story-dialog'
import { DeleteStoryDraftDialog } from './delete-story-draft-dialog'
import { getDraftSectionProgress } from './draft-progress'
import { GenerateStoryDialog } from './generate-story-dialog'
import { ManualStoryCreateDialog } from './manual-story-create-dialog'
import { StoryDetailsDialog } from './story-details-dialog'
import { StoryDraftDetailsDialog } from './story-draft-details-dialog'
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

function StoryCreateMenu({
  compact = false,
  onOpenManualCreate,
  onOpenResourceGenerate,
}: {
  compact?: boolean
  onOpenManualCreate: () => void
  onOpenResourceGenerate: () => void
}) {
  const { t } = useTranslation()

  return (
    <PopupMenu>
      <PopupMenuTrigger asChild>
        {compact ? (
          <IconButton
            icon={<FontAwesomeIcon icon={faPlus} />}
            label={t('stories.actions.createMenu')}
            size="md"
          />
        ) : (
          <Button>
            <FontAwesomeIcon icon={faPlus} />
            {t('stories.actions.createMenu')}
          </Button>
        )}
      </PopupMenuTrigger>

      <PopupMenuContent align="end" className="w-64">
        <PopupMenuItem
          onSelect={() => {
            onOpenResourceGenerate()
          }}
        >
          <span className="inline-flex h-9 w-9 items-center justify-center rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] text-[var(--color-text-primary)]">
            <FontAwesomeIcon icon={faDiagramProject} />
          </span>
          <div className="min-w-0 flex-1">
            <p className="text-sm font-medium text-[var(--color-text-primary)]">
              {t('stories.actions.createFromResource')}
            </p>
            <p className="text-xs leading-6 text-[var(--color-text-muted)]">
              {t('stories.actions.createFromResourceDescription')}
            </p>
          </div>
        </PopupMenuItem>
        <PopupMenuItem
          onSelect={() => {
            onOpenManualCreate()
          }}
        >
          <span className="inline-flex h-9 w-9 items-center justify-center rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] text-[var(--color-text-primary)]">
            <FontAwesomeIcon icon={faPen} />
          </span>
          <div className="min-w-0 flex-1">
            <p className="text-sm font-medium text-[var(--color-text-primary)]">
              {t('stories.actions.createManual')}
            </p>
            <p className="text-xs leading-6 text-[var(--color-text-muted)]">
              {t('stories.actions.createManualDescription')}
            </p>
          </div>
        </PopupMenuItem>
      </PopupMenuContent>
    </PopupMenu>
  )
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
  const [isManualCreateDialogOpen, setIsManualCreateDialogOpen] = useState(false)
  const [detailsStoryId, setDetailsStoryId] = useState<string | null>(null)
  const [detailsDraftId, setDetailsDraftId] = useState<string | null>(null)
  const [editStoryId, setEditStoryId] = useState<string | null>(null)
  const [selectionMode, setSelectionMode] = useState(false)
  const [draftSelectionMode, setDraftSelectionMode] = useState(false)
  const [selectedStoryIds, setSelectedStoryIds] = useState<string[]>([])
  const [selectedDraftIds, setSelectedDraftIds] = useState<string[]>([])
  const [deleteTargetIds, setDeleteTargetIds] = useState<string[]>([])
  const [deleteDraftTargetIds, setDeleteDraftTargetIds] = useState<string[]>([])
  useToastNotice(notice)

  const distinctResourceCount = useMemo(
    () => new Set(stories.map((story) => story.resource_id)).size,
    [stories],
  )
  const readyDraftCount = useMemo(
    () => drafts.filter((draft) => draft.status === 'ready_to_finalize').length,
    [drafts],
  )
  const deleteTargets = useMemo(
    () =>
      deleteTargetIds
        .map((storyId) => stories.find((story) => story.story_id === storyId))
        .filter((story): story is StorySummary => story !== undefined),
    [deleteTargetIds, stories],
  )
  const deleteDraftTargets = useMemo(
    () =>
      deleteDraftTargetIds
        .map((draftId) => drafts.find((draft) => draft.draft_id === draftId))
        .filter((draft): draft is StoryDraftSummary => draft !== undefined),
    [deleteDraftTargetIds, drafts],
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

  useEffect(() => {
    const availableStoryIds = new Set(stories.map((story) => story.story_id))

    setSelectedStoryIds((currentSelection) =>
      currentSelection.filter((storyId) => availableStoryIds.has(storyId)),
    )
    setDeleteTargetIds((currentSelection) =>
      currentSelection.filter((storyId) => availableStoryIds.has(storyId)),
    )

    if (detailsStoryId !== null && !availableStoryIds.has(detailsStoryId)) {
      setDetailsStoryId(null)
    }

    if (editStoryId !== null && !availableStoryIds.has(editStoryId)) {
      setEditStoryId(null)
    }
  }, [detailsStoryId, editStoryId, stories])

  useEffect(() => {
    const availableDraftIds = new Set(drafts.map((draft) => draft.draft_id))

    setSelectedDraftIds((currentSelection) =>
      currentSelection.filter((draftId) => availableDraftIds.has(draftId)),
    )
    setDeleteDraftTargetIds((currentSelection) =>
      currentSelection.filter((draftId) => availableDraftIds.has(draftId)),
    )

    if (detailsDraftId !== null && !availableDraftIds.has(detailsDraftId)) {
      setDetailsDraftId(null)
    }
  }, [detailsDraftId, drafts])

  useEffect(() => {
    if (viewMode !== 'stories') {
      setSelectionMode(false)
      setSelectedStoryIds([])
    }

    if (viewMode !== 'drafts') {
      setDraftSelectionMode(false)
      setSelectedDraftIds([])
    }
  }, [viewMode])

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
  }, [
    distinctResourceCount,
    drafts.length,
    readyDraftCount,
    setRailContent,
    stories.length,
    t,
    viewMode,
  ])

  async function handleDeleteStory() {
    if (deleteTargets.length === 0) {
      return
    }

    setIsDeleting(true)

    try {
      const result = await runBatchDelete(deleteTargets, async (target) => {
        await deleteStory(target.story_id)
      })

      setDeleteTargetIds([])

      if (result.deleted.length > 0) {
        const deletedIds = new Set(result.deleted.map((target) => target.story_id))
        setSelectedStoryIds((currentSelection) =>
          currentSelection.filter((storyId) => !deletedIds.has(storyId)),
        )
        setDeleteTargetIds([])

        if (detailsStoryId !== null && deletedIds.has(detailsStoryId)) {
          setDetailsStoryId(null)
        }

        if (editStoryId !== null && deletedIds.has(editStoryId)) {
          setEditStoryId(null)
        }
      }

      if (result.failed.length === 0) {
        setNotice({
          message:
            result.deleted.length > 1
              ? t('stories.feedback.deletedMany', { count: result.deleted.length })
              : t('stories.feedback.deleted', { name: result.deleted[0]?.display_name ?? '' }),
          tone: 'success',
        })
        if (selectionMode) {
          setSelectionMode(false)
          setSelectedStoryIds([])
        }
      } else if (result.deleted.length > 0) {
        setNotice({
          message: t('stories.feedback.deletedPartial', {
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
                ? t('stories.deleteDialog.conflictMany')
                : t('stories.deleteDialog.conflict')
              : t('stories.feedback.deleteFailed'),
          tone: result.conflictCount > 0 ? 'warning' : 'error',
        })
      }

      await refreshStories()
    } finally {
      setIsDeleting(false)
    }
  }

  function toggleStorySelection(storyId: string) {
    setSelectedStoryIds((currentSelection) =>
      currentSelection.includes(storyId)
        ? currentSelection.filter((currentStoryId) => currentStoryId !== storyId)
        : [...currentSelection, storyId],
    )
  }

  function toggleDraftSelection(draftId: string) {
    setSelectedDraftIds((currentSelection) =>
      currentSelection.includes(draftId)
        ? currentSelection.filter((currentDraftId) => currentDraftId !== draftId)
        : [...currentSelection, draftId],
    )
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
    if (deleteDraftTargets.length === 0) {
      return
    }

    setIsDeletingDraft(true)

    try {
      const result = await runBatchDelete(deleteDraftTargets, async (target) => {
        await deleteStoryDraft(target.draft_id)
      })
      setDeleteDraftTargetIds([])

      if (result.deleted.length > 0) {
        const deletedIds = new Set(result.deleted.map((target) => target.draft_id))

        setSelectedDraftIds((currentSelection) =>
          currentSelection.filter((draftId) => !deletedIds.has(draftId)),
        )

        if (detailsDraftId !== null && deletedIds.has(detailsDraftId)) {
          setDetailsDraftId(null)
        }
      }

      if (result.failed.length === 0) {
        setNotice({
          message:
            result.deleted.length > 1
              ? t('stories.drafts.feedback.deletedMany', { count: result.deleted.length })
              : t('stories.drafts.feedback.deleted', {
                  name: result.deleted[0]?.display_name ?? '',
                }),
          tone: 'success',
        })

        if (draftSelectionMode) {
          setDraftSelectionMode(false)
          setSelectedDraftIds([])
        }
      } else if (result.deleted.length > 0) {
        setNotice({
          message: t('stories.drafts.feedback.deletedPartial', {
            failed: result.failed.length,
            success: result.deleted.length,
          }),
          tone: 'warning',
        })
      } else {
        setNotice({
          message: t('stories.drafts.feedback.deleteFailed'),
          tone: 'error',
        })
      }

      await refreshDrafts()
    } catch (error) {
      setDeleteDraftTargetIds([])
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

      <ManualStoryCreateDialog
        availableCharacters={characters}
        onCompleted={async ({ message }) => {
          setNotice({ message, tone: 'success' })
          await refreshStories()
        }}
        onOpenChange={setIsManualCreateDialogOpen}
        open={isManualCreateDialogOpen}
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
          setDeleteTargetIds([])
        }}
        targets={deleteTargets}
      />

      <DeleteStoryDraftDialog
        deleting={isDeletingDraft}
        onConfirm={() => {
          void handleDeleteDraft()
        }}
        onOpenChange={() => {
          setDeleteDraftTargetIds([])
        }}
        open={deleteDraftTargetIds.length > 0}
        targets={deleteDraftTargets}
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
                  {viewMode === 'stories' && selectionMode ? (
                    <>
                      <Badge className="normal-case px-3.5 py-2" variant="subtle">
                        {t('stories.selection.count', { count: selectedStoryIds.length })}
                      </Badge>
                      <IconButton
                        disabled={stories.length === 0}
                        icon={<FontAwesomeIcon icon={faCheckDouble} />}
                        label={t('stories.actions.selectAll')}
                        onClick={() => {
                          setSelectedStoryIds(stories.map((story) => story.story_id))
                        }}
                        size="md"
                        variant="secondary"
                      />
                      <IconButton
                        disabled={selectedStoryIds.length === 0}
                        icon={<FontAwesomeIcon icon={faTrashCan} />}
                        label={t('stories.actions.deleteSelected')}
                        onClick={() => {
                          setDeleteTargetIds(selectedStoryIds)
                        }}
                        size="md"
                        variant="danger"
                      />
                      <IconButton
                        icon={<FontAwesomeIcon icon={faXmark} />}
                        label={t('stories.actions.cancelSelection')}
                        onClick={() => {
                          setSelectionMode(false)
                          setSelectedStoryIds([])
                        }}
                        size="md"
                        variant="secondary"
                      />
                    </>
                  ) : viewMode === 'drafts' && draftSelectionMode ? (
                    <>
                      <Badge className="normal-case px-3.5 py-2" variant="subtle">
                        {t('stories.selection.count', { count: selectedDraftIds.length })}
                      </Badge>
                      <IconButton
                        disabled={drafts.length === 0}
                        icon={<FontAwesomeIcon icon={faCheckDouble} />}
                        label={t('stories.actions.selectAll')}
                        onClick={() => {
                          setSelectedDraftIds(drafts.map((draft) => draft.draft_id))
                        }}
                        size="md"
                        variant="secondary"
                      />
                      <IconButton
                        disabled={selectedDraftIds.length === 0}
                        icon={<FontAwesomeIcon icon={faTrashCan} />}
                        label={t('stories.actions.deleteSelected')}
                        onClick={() => {
                          setDeleteDraftTargetIds(selectedDraftIds)
                        }}
                        size="md"
                        variant="danger"
                      />
                      <IconButton
                        icon={<FontAwesomeIcon icon={faXmark} />}
                        label={t('stories.actions.cancelSelection')}
                        onClick={() => {
                          setDraftSelectionMode(false)
                          setSelectedDraftIds([])
                        }}
                        size="md"
                        variant="secondary"
                      />
                    </>
                  ) : (
                    <>
                      {viewMode === 'stories' ? (
                        <IconButton
                          icon={<FontAwesomeIcon icon={faSquareCheck} />}
                          label={t('stories.actions.selectMode')}
                          onClick={() => {
                            setSelectionMode(true)
                            setSelectedStoryIds([])
                          }}
                          size="md"
                          variant="secondary"
                        />
                      ) : (
                        <IconButton
                          icon={<FontAwesomeIcon icon={faSquareCheck} />}
                          label={t('stories.actions.selectMode')}
                          onClick={() => {
                            setDraftSelectionMode(true)
                            setSelectedDraftIds([])
                          }}
                          size="md"
                          variant="secondary"
                        />
                      )}
                      <StoryCreateMenu
                        compact
                        onOpenManualCreate={() => {
                          setIsManualCreateDialogOpen(true)
                        }}
                        onOpenResourceGenerate={() => {
                          setIsCreateDialogOpen(true)
                        }}
                      />
                    </>
                  )}
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
                    {viewMode === 'stories'
                      ? t('stories.list.title')
                      : t('stories.drafts.list.title')}
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
                        <StoryCreateMenu
                          onOpenManualCreate={() => {
                            setIsManualCreateDialogOpen(true)
                          }}
                          onOpenResourceGenerate={() => {
                            setIsCreateDialogOpen(true)
                          }}
                        />
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
                            {selectionMode ? (
                              <SelectionToggleButton
                                label={
                                  selectedStoryIds.includes(story.story_id)
                                    ? t('stories.actions.deselect')
                                    : t('stories.actions.select')
                                }
                                onClick={() => {
                                  toggleStorySelection(story.story_id)
                                }}
                                selected={selectedStoryIds.includes(story.story_id)}
                              />
                            ) : (
                              <>
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
                                    setDeleteTargetIds([story.story_id])
                                  }}
                                  size="sm"
                                  variant="danger"
                                >
                                  <FontAwesomeIcon icon={faTrashCan} />
                                  {t('stories.actions.delete')}
                                </Button>
                              </>
                            )}
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
                      <StoryCreateMenu
                        onOpenManualCreate={() => {
                          setIsManualCreateDialogOpen(true)
                        }}
                        onOpenResourceGenerate={() => {
                          setIsCreateDialogOpen(true)
                        }}
                      />
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
                            {draftSelectionMode ? (
                              <SelectionToggleButton
                                label={
                                  selectedDraftIds.includes(draft.draft_id)
                                    ? t('stories.actions.deselect')
                                    : t('stories.actions.select')
                                }
                                onClick={() => {
                                  toggleDraftSelection(draft.draft_id)
                                }}
                                selected={selectedDraftIds.includes(draft.draft_id)}
                              />
                            ) : (
                              <>
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
                                    <FontAwesomeIcon
                                      className={cn(isWorking ? 'animate-spin' : '')}
                                      icon={faRotateRight}
                                    />
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
                                    setDeleteDraftTargetIds([draft.draft_id])
                                  }}
                                  size="sm"
                                  variant="danger"
                                >
                                  <FontAwesomeIcon icon={faTrashCan} />
                                  {t('stories.actions.delete')}
                                </Button>
                              </>
                            )}
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
